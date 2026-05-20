/// ECRAM (Embedded Controller RAM) reader via IoTDriver.sys
///
/// IoTDriver.sys exposes physical ECRAM memory via two IOCTL codes:
///   0x22E000 — READ:  input {phys_addr:u64, byte_count:u64, zeros[0x100]}
///                     output {zeros[0x10], data[byte_count], zeros[...]}
///   0x22E004 — WRITE: same layout, driver writes data into EC RAM
///
/// The driver device is enumerated with GUID {AB7924A1-3162-4010-B33B-837E87E25FBC}.
/// Access requires an elevated process (SeTokenIsAdmin check in the driver).
///
/// Physical ECRAM layout (discovered via IoTService.exe RE):
///   0xFE0B0F00 [4 bytes]  — EC status flags (byte[0..3])
///   0xFE0B0F08 [0x78 bytes] — large sensor/power data block
///
/// Finding charger wattage: read the 0x78-byte block and use the debug
/// command to identify which register changes when plugging/unplugging the charger.

use anyhow::{Context, Result};

/// Physical ECRAM base address.
pub const ECRAM_BASE: u64 = 0xFE0B0F00;
/// Physical address of the 0x78-byte sensor/power block.
pub const ECRAM_SENSOR_BLOCK: u64 = 0xFE0B0F08;
/// Size of the sensor block.
pub const ECRAM_SENSOR_SIZE: usize = 0x78;
/// IOCTL code for ECRAM read.
const IOCTL_ECRAM_READ: u32 = 0x22E000;
/// IoT driver device interface GUID: {AB7924A1-3162-4010-B33B-837E87E25FBC}
#[cfg(windows)]
const IOT_GUID: windows::core::GUID = windows::core::GUID {
    data1: 0xAB7924A1,
    data2: 0x3162,
    data3: 0x4010,
    data4: [0xB3, 0x3B, 0x83, 0x7E, 0x87, 0xE2, 0x5F, 0xBC],
};

/// Total IOCTL buffer size (driver requires exactly 0x110 bytes for both in and out).
const IOCTL_BUF_SIZE: usize = 0x110;

/// IOCTL buffer layout: matches the driver's expected input/output format.
///   Bytes  0–7:   physical_address (u64 LE)
///   Bytes  8–15:  byte_count (u64 LE)
///   Bytes 16–271: on input: zeros (padding); on output: EC data starting at byte 16
#[repr(C)]
struct EcramBuf {
    physical_address: u64,
    byte_count: u64,
    data: [u8; 0x100],
}

const _: () = {
    assert!(std::mem::size_of::<EcramBuf>() == IOCTL_BUF_SIZE);
};

/// Read `byte_count` bytes from ECRAM at `phys_addr`.
///
/// Returns a `Vec<u8>` of length `byte_count` on success.
/// Requires the process to be running as administrator.
///
/// # Errors
/// Returns an error if the device cannot be opened (driver not loaded,
/// insufficient privileges) or if the IOCTL fails.
pub fn read_ecram(phys_addr: u64, byte_count: usize) -> Result<Vec<u8>> {
    assert!(
        byte_count <= 0x100,
        "byte_count must be ≤ 0x100 (driver limit)"
    );

    #[cfg(windows)]
    {
        let device_path = find_iot_device_path()
            .context("IoT driver device not found (is IoTDriver.sys loaded?)")?;
        read_ecram_inner(&device_path, phys_addr, byte_count)
    }

    #[cfg(not(windows))]
    {
        let _ = (phys_addr, byte_count);
        anyhow::bail!("ECRAM read is only supported on Windows")
    }
}

/// Convenience: read the full 0x78-byte sensor/power block at 0xFE0B0F08.
pub fn read_sensor_block() -> Result<Vec<u8>> {
    read_ecram(ECRAM_SENSOR_BLOCK, ECRAM_SENSOR_SIZE)
}

/// Convenience: read the first 8 EC status bytes at 0xFE0B0F00.
pub fn read_status_bytes() -> Result<Vec<u8>> {
    read_ecram(ECRAM_BASE, 8)
}

/// Read all ECRAM bytes available from IoTService's known ranges.
/// Returns (status[8], sensor[0x78]) combined as a flat Vec<u8> of length 0x80.
pub fn read_all() -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(0x80);
    buf.extend_from_slice(&read_ecram(ECRAM_BASE, 8)?);
    buf.extend_from_slice(&read_ecram(ECRAM_SENSOR_BLOCK, 0x78)?);
    Ok(buf)
}

/// Try to extract AC adapter input power (in milliwatts) from the sensor block.
///
/// The exact register layout is device-specific. This function tries the most
/// common layouts for Xiaomi laptop ECs. Returns `None` if no plausible value
/// is found (e.g. charger not connected, unknown layout, or read fails).
///
/// Use `debug_ecram_hex()` to inspect raw bytes and identify the correct offset.
pub fn try_get_ac_power_mw() -> Option<i32> {
    let data = read_sensor_block().ok()?;

    // Probe candidate offsets for AC adapter wattage.
    // Typical EC layouts use 16-bit or 32-bit LE integers in mW, mA×mV, or dW.
    // We check if any makes sense as charger wattage (10–250 W = 10000–250000 mW).

    // Attempt 1: bytes [0..3] as u32 mW (direct wattage register)
    if data.len() >= 4 {
        let v = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if (10_000..=250_000).contains(&v) {
            return Some(v as i32);
        }
    }

    // Attempt 2: bytes [0..1] as u16 in mW (capped at 0xFFFF = 65535 mW)
    if data.len() >= 2 {
        let v = u16::from_le_bytes([data[0], data[1]]) as u32;
        if (10_000..=65_000).contains(&v) {
            return Some(v as i32);
        }
    }

    // Attempt 3: bytes [0..1] as u16 in 10mW units (deciwatts × 10)
    if data.len() >= 2 {
        let v = u16::from_le_bytes([data[0], data[1]]) as u32 * 10;
        if (10_000..=250_000).contains(&v) {
            return Some(v as i32);
        }
    }

    // Attempt 4: voltage (bytes[0..1]) × current (bytes[2..3]) — both in mV / mA
    if data.len() >= 4 {
        let volts_mv = u16::from_le_bytes([data[0], data[1]]) as u64;
        let amps_ma = u16::from_le_bytes([data[2], data[3]]) as u64;
        let mw = volts_mv * amps_ma / 1000;
        if (10_000..=250_000).contains(&(mw as u32)) {
            return Some(mw as i32);
        }
    }

    None
}

/// Return a hex dump string of all known ECRAM bytes for debugging.
/// Format: "offset: XX XX XX XX ..."
pub fn debug_ecram_hex() -> Result<String> {
    let all = read_all()?;
    let mut out = String::new();
    for (i, chunk) in all.chunks(16).enumerate() {
        let offset = i * 16;
        let addr = ECRAM_BASE + offset as u64;
        let hex: Vec<String> = chunk.iter().map(|b| format!("{b:02X}")).collect();
        out.push_str(&format!("0x{addr:08X}: {}\n", hex.join(" ")));
    }
    Ok(out)
}

// ── Windows implementation ────────────────────────────────────────────────────

#[cfg(windows)]
fn find_iot_device_path() -> Result<String> {
    use windows::{
        Win32::{
            Devices::DeviceAndDriverInstallation::{
                SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInterfaces,
                SetupDiGetClassDevsW, SetupDiGetDeviceInterfaceDetailW,
                DIGCF_DEVICEINTERFACE, DIGCF_PRESENT, SP_DEVICE_INTERFACE_DATA,
                SP_DEVICE_INTERFACE_DETAIL_DATA_W,
            },
        },
    };

    unsafe {
        let dev_info = SetupDiGetClassDevsW(
            Some(&IOT_GUID),
            None,
            None,
            DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
        )
        .context("SetupDiGetClassDevsW for IoT GUID")?;

        let mut iface = SP_DEVICE_INTERFACE_DATA {
            cbSize: std::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
            ..std::mem::zeroed()
        };

        let enum_result = SetupDiEnumDeviceInterfaces(dev_info, None, &IOT_GUID, 0, &mut iface);
        if enum_result.is_err() {
            let _ = SetupDiDestroyDeviceInfoList(dev_info);
            anyhow::bail!("No IoT device interface found (GUID {{AB7924A1-...}})");
        }

        // First call: get required buffer size
        let mut required = 0u32;
        let _ = SetupDiGetDeviceInterfaceDetailW(
            dev_info,
            &iface,
            None,
            0,
            Some(&mut required),
            None,
        );

        if required == 0 || required > 4096 {
            let _ = SetupDiDestroyDeviceInfoList(dev_info);
            anyhow::bail!("Invalid required size {required} for IoT device path");
        }

        // Second call: get the device path
        let mut buf = vec![0u8; required as usize];
        let detail_ptr = buf.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
        (*detail_ptr).cbSize =
            std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;

        let detail_result = SetupDiGetDeviceInterfaceDetailW(
            dev_info,
            &iface,
            Some(detail_ptr),
            required,
            None,
            None,
        );
        let _ = SetupDiDestroyDeviceInfoList(dev_info);
        detail_result.context("SetupDiGetDeviceInterfaceDetailW")?;

        // Parse the UTF-16 device path (starts at offset 4, after the cbSize u32)
        let path_offset = 4usize;
        let wide_slice = std::slice::from_raw_parts(
            buf.as_ptr().add(path_offset) as *const u16,
            (required as usize - path_offset) / 2,
        );
        let null_pos = wide_slice
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(wide_slice.len());
        let path = String::from_utf16(&wide_slice[..null_pos])
            .context("Invalid UTF-16 device path")?;

        Ok(path)
    }
}

#[cfg(windows)]
fn read_ecram_inner(device_path: &str, phys_addr: u64, byte_count: usize) -> Result<Vec<u8>> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE},
            Storage::FileSystem::{
                CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE,
                OPEN_EXISTING,
            },
            System::IO::DeviceIoControl,
        },
    };

    let path_w: Vec<u16> = OsStr::new(device_path)
        .encode_wide()
        .chain(Some(0))
        .collect();

    unsafe {
        let handle = CreateFileW(
            PCWSTR(path_w.as_ptr()),
            (GENERIC_READ | GENERIC_WRITE).0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            HANDLE::default(),
        )
        .context("Open IoT driver device")?;

        if handle == INVALID_HANDLE_VALUE {
            anyhow::bail!("INVALID_HANDLE_VALUE opening IoT driver device");
        }

        // Build input buffer
        let in_buf = EcramBuf {
            physical_address: phys_addr,
            byte_count: byte_count as u64,
            data: [0u8; 0x100],
        };

        // Output buffer (driver writes EC data at byte offset 0x10)
        let mut out_buf = EcramBuf {
            physical_address: 0,
            byte_count: 0,
            data: [0u8; 0x100],
        };

        let mut bytes_returned = 0u32;
        let result = DeviceIoControl(
            handle,
            IOCTL_ECRAM_READ,
            Some((&raw const in_buf).cast()),
            IOCTL_BUF_SIZE as u32,
            Some((&raw mut out_buf).cast()),
            IOCTL_BUF_SIZE as u32,
            Some(&mut bytes_returned),
            None,
        );

        CloseHandle(handle).ok();
        result.context("DeviceIoControl IOCTL_ECRAM_READ")?;

        // EC data starts at offset 0x10 in output (= out_buf.data[0..byte_count])
        // out_buf layout: [physical_address:8][byte_count:8][data:0x100]
        // The driver fills at out_buf+0x10 which corresponds to out_buf.data[0..byte_count]
        let ec_bytes = out_buf.data[..byte_count].to_vec();
        Ok(ec_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecram_buf_size() {
        assert_eq!(std::mem::size_of::<EcramBuf>(), IOCTL_BUF_SIZE);
    }
}
