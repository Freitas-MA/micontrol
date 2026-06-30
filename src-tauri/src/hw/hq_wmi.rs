//! HQWmiCommonInterface — BIOS/EC control via WMI.
//!
//! This module wraps the `HQWmiCommonInterface` WMI class (ROOT\WMI namespace),
//! which provides BIOS-level operations on Xiaomi Book devices.
//!
//! All methods take a `String` parameter named `req` and return a `String` named `ret`.
//! The WMI instance path is `ACPI\PNP0C14\0_0`.
//!
//! Methods implemented:
//! - `SetPerformanceMode(req)` — Set BIOS performance mode
//! - `ChangeBootOption(req)` — Change boot device order
//! - `LoadDefault(req)` — Load BIOS default settings
//! - `S5RTCWakeEnable(req)` — Enable/disable S5 RTC wake
//! - `EnablePXEBoot(req)` — Enable/disable PXE boot
//! - `LoadDefaultKey(req)` — Load default security key
//! - `ClearKey(req)` — Clear security key
//! - `SetOOBTestMode(req)` — Set OOB test mode
//! - `ClearOOBTestMode(req)` — Clear OOB test mode
//! - `WifiCountryCode(req)` — Set WiFi country code
//! - `ShippingCountryCode(req)` — Set shipping country code

use crate::hw::errors::{HardwareError, HardwareResult};
use serde::{Deserialize, Serialize};

/// Response from an HQWmiCommonInterface method call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HqWmiResponse {
    /// The method that was called.
    pub method: String,
    /// The request string sent to the method.
    pub req: String,
    /// The return string from the method.
    pub ret: String,
    /// Whether the call succeeded (ret is non-empty and not an error message).
    pub success: bool,
}

// ── WMI implementation ─────────────────────────────────────────────────────

#[cfg(windows)]
mod imp {
    use super::*;
    use crate::hw::wmi_cache;
    use windows::core::{BSTR, HSTRING, PCWSTR, VARIANT};
    use windows::Win32::System::Wmi::{
        IWbemClassObject, WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY,
        WBEM_GENERIC_FLAG_TYPE,
    };

    const VT_BSTR_VAL: u16 = windows::core::imp::VT_BSTR;

    /// Helper to create a PCWSTR from a string.
    fn pcwstr(s: &str) -> PCWSTR {
        let h = HSTRING::from(s);
        PCWSTR::from_raw(h.as_ptr())
    }

    /// Call a method on HQWmiCommonInterface with a String parameter.
    pub fn call_method(method: &str, req: &str) -> HardwareResult<HqWmiResponse> {
        wmi_cache::with_wmi(|conn| {
            let svc = &conn.svc;

            // Get the HQWmiCommonInterface class to obtain method signature
            let class_path = BSTR::from("HQWmiCommonInterface");
            let mut class_obj: Option<IWbemClassObject> = None;
            unsafe {
                svc.GetObject(
                    &class_path,
                    WBEM_GENERIC_FLAG_TYPE(0),
                    None,
                    Some(&mut class_obj),
                    None,
                )?;
            }
            let class_obj = class_obj.ok_or_else(|| {
                anyhow::anyhow!(HardwareError::Wmi("GetObject returned null".into()))
            })?;

            // Get method signature
            let method_name = pcwstr(method);
            let mut in_sig: Option<IWbemClassObject> = None;
            let mut out_sig: Option<IWbemClassObject> = None;
            unsafe {
                class_obj.GetMethod(method_name, 0, &mut in_sig, &mut out_sig)?;
            }

            // Spawn input parameters
            let in_params = if let Some(in_sig) = in_sig {
                let params = unsafe { in_sig.SpawnInstance(0)? };
                // Set the "req" parameter — build a VARIANT containing a BSTR
                let prop_name = pcwstr("req");
                let req_bstr = BSTR::from(req);
                let in_data_var = build_bstr_variant(&req_bstr)?;
                unsafe { params.Put(prop_name, 0, &in_data_var, 0)? };
                Some(params)
            } else {
                None
            };

            // Query for the active HQWmiCommonInterface instance to get its __Path
            let query = BSTR::from("SELECT * FROM HQWmiCommonInterface WHERE Active = TRUE");
            let wql = BSTR::from("WQL");
            let flags = WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY;
            let enumerator = unsafe { svc.ExecQuery(&wql, &query, flags, None)? };

            let mut objs: [Option<IWbemClassObject>; 1] = [None];
            let mut returned: u32 = 0;
            let hr = unsafe { enumerator.Next(-1, &mut objs, &mut returned) };
            hr.ok()?;

            let instance = objs[0].as_ref().ok_or_else(|| {
                anyhow::anyhow!(HardwareError::Wmi(
                    "No active HQWmiCommonInterface instance found".into()
                ))
            })?;

            // Get the instance path
            let path_name = pcwstr("__Path");
            let mut path_var = VARIANT::new();
            let mut cim_type: i32 = 0;
            unsafe {
                instance.Get(path_name, 0, &mut path_var, Some(&mut cim_type), None)?;
            }
            let path_str = variant_to_string(&path_var)?;

            // Call ExecMethod on IWbemServices
            let path_bstr = BSTR::from(path_str);
            let method_bstr = BSTR::from(method);
            let mut out_params: Option<IWbemClassObject> = None;
            unsafe {
                svc.ExecMethod(
                    &path_bstr,
                    &method_bstr,
                    WBEM_GENERIC_FLAG_TYPE(0),
                    None,
                    in_params.as_ref(),
                    Some(&mut out_params),
                    None,
                )?;
            }

            let out_params = out_params.ok_or_else(|| {
                anyhow::anyhow!(HardwareError::Wmi(
                    "ExecMethod returned null out_params".into()
                ))
            })?;

            // Read the "ret" return value
            let ret_name = pcwstr("ret");
            let mut ret_var = VARIANT::new();
            unsafe {
                out_params.Get(ret_name, 0, &mut ret_var, None, None)?;
            }

            let ret = variant_to_string(&ret_var)?;

            let success = !ret.is_empty()
                && !ret.to_lowercase().contains("fail")
                && !ret.to_lowercase().contains("error");

            Ok(HqWmiResponse {
                method: method.to_string(),
                req: req.to_string(),
                ret,
                success,
            })
        })
    }

    /// Build a VARIANT containing a BSTR value.
    fn build_bstr_variant(bstr: &BSTR) -> anyhow::Result<VARIANT> {
        let raw = windows::core::imp::VARIANT {
            Anonymous: windows::core::imp::VARIANT_0 {
                Anonymous: windows::core::imp::VARIANT_0_0 {
                    vt: VT_BSTR_VAL,
                    wReserved1: 0,
                    wReserved2: 0,
                    wReserved3: 0,
                    Anonymous: windows::core::imp::VARIANT_0_0_0 {
                        bstrVal: bstr.as_ptr(),
                    },
                },
            },
        };
        Ok(unsafe { VARIANT::from_raw(raw) })
    }

    /// Convert a VARIANT to a String.
    fn variant_to_string(var: &VARIANT) -> anyhow::Result<String> {
        let raw = var.as_raw();
        let vt: u16 = unsafe { raw.Anonymous.Anonymous.vt };

        if vt == VT_BSTR_VAL {
            let bstr_ptr = unsafe { raw.Anonymous.Anonymous.Anonymous.bstrVal };
            if bstr_ptr.is_null() {
                return Ok(String::new());
            }
            let bstr = unsafe { BSTR::from_raw(bstr_ptr) };
            Ok(bstr.to_string())
        } else {
            Err(anyhow::anyhow!(HardwareError::Wmi(format!(
                "ret has unexpected vt=0x{vt:04X}"
            ))))
        }
    }
}

// ── Public API ─────────────────────────────────────────────────────────────

/// Set BIOS performance mode via HQWmiCommonInterface.
///
/// `req` is typically a string like "1" (performance), "2" (balanced), "3" (quiet).
pub fn set_performance_mode(req: &str) -> HardwareResult<HqWmiResponse> {
    #[cfg(windows)]
    {
        imp::call_method("SetPerformanceMode", req)
    }
    #[cfg(not(windows))]
    {
        let _ = req;
        Err(HardwareError::NotSupported(
            "WMI only available on Windows".into(),
        ))
    }
}

/// Change boot device order via HQWmiCommonInterface.
pub fn change_boot_option(req: &str) -> HardwareResult<HqWmiResponse> {
    #[cfg(windows)]
    {
        imp::call_method("ChangeBootOption", req)
    }
    #[cfg(not(windows))]
    {
        let _ = req;
        Err(HardwareError::NotSupported(
            "WMI only available on Windows".into(),
        ))
    }
}

/// Load BIOS default settings.
pub fn load_default(req: &str) -> HardwareResult<HqWmiResponse> {
    #[cfg(windows)]
    {
        imp::call_method("LoadDefault", req)
    }
    #[cfg(not(windows))]
    {
        let _ = req;
        Err(HardwareError::NotSupported(
            "WMI only available on Windows".into(),
        ))
    }
}

/// Enable/disable S5 RTC wake.
pub fn s5_rtc_wake_enable(req: &str) -> HardwareResult<HqWmiResponse> {
    #[cfg(windows)]
    {
        imp::call_method("S5RTCWakeEnable", req)
    }
    #[cfg(not(windows))]
    {
        let _ = req;
        Err(HardwareError::NotSupported(
            "WMI only available on Windows".into(),
        ))
    }
}

/// Enable/disable PXE boot.
pub fn enable_pxe_boot(req: &str) -> HardwareResult<HqWmiResponse> {
    #[cfg(windows)]
    {
        imp::call_method("EnablePXEBoot", req)
    }
    #[cfg(not(windows))]
    {
        let _ = req;
        Err(HardwareError::NotSupported(
            "WMI only available on Windows".into(),
        ))
    }
}

/// Load default security key.
pub fn load_default_key(req: &str) -> HardwareResult<HqWmiResponse> {
    #[cfg(windows)]
    {
        imp::call_method("LoadDefaultKey", req)
    }
    #[cfg(not(windows))]
    {
        let _ = req;
        Err(HardwareError::NotSupported(
            "WMI only available on Windows".into(),
        ))
    }
}

/// Clear security key.
pub fn clear_key(req: &str) -> HardwareResult<HqWmiResponse> {
    #[cfg(windows)]
    {
        imp::call_method("ClearKey", req)
    }
    #[cfg(not(windows))]
    {
        let _ = req;
        Err(HardwareError::NotSupported(
            "WMI only available on Windows".into(),
        ))
    }
}

/// Set OOB test mode.
pub fn set_oob_test_mode(req: &str) -> HardwareResult<HqWmiResponse> {
    #[cfg(windows)]
    {
        imp::call_method("SetOOBTestMode", req)
    }
    #[cfg(not(windows))]
    {
        let _ = req;
        Err(HardwareError::NotSupported(
            "WMI only available on Windows".into(),
        ))
    }
}

/// Clear OOB test mode.
pub fn clear_oob_test_mode(req: &str) -> HardwareResult<HqWmiResponse> {
    #[cfg(windows)]
    {
        imp::call_method("ClearOOBTestMode", req)
    }
    #[cfg(not(windows))]
    {
        let _ = req;
        Err(HardwareError::NotSupported(
            "WMI only available on Windows".into(),
        ))
    }
}

/// Set WiFi country code.
pub fn set_wifi_country_code(req: &str) -> HardwareResult<HqWmiResponse> {
    #[cfg(windows)]
    {
        imp::call_method("WifiCountryCode", req)
    }
    #[cfg(not(windows))]
    {
        let _ = req;
        Err(HardwareError::NotSupported(
            "WMI only available on Windows".into(),
        ))
    }
}

/// Set shipping country code.
pub fn set_shipping_country_code(req: &str) -> HardwareResult<HqWmiResponse> {
    #[cfg(windows)]
    {
        imp::call_method("ShippingCountryCode", req)
    }
    #[cfg(not(windows))]
    {
        let _ = req;
        Err(HardwareError::NotSupported(
            "WMI only available on Windows".into(),
        ))
    }
}
