use tauri::State;
use crate::state::{AppState, PerformanceMode};
use crate::hw::performance::{get_performance_mode as hw_get_perf, PerformanceResult, PerfDebugInfo, get_perf_debug as hw_perf_debug};
use crate::hw::charging::{get_charging_threshold as hw_get_charge, ChargingResult};
use crate::elev_bridge;

#[tauri::command]
pub async fn get_performance_mode(_state: State<'_, AppState>) -> Result<PerformanceMode, String> {
    hw_get_perf().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_performance_mode(
    mode: PerformanceMode,
    state: State<'_, AppState>,
) -> Result<PerformanceResult, String> {
    let raw = elev_bridge::run_elevated(
        "set_performance_mode",
        serde_json::json!({ "mode": mode }),
    )
    .await?;
    let result: PerformanceResult = serde_json::from_value(raw)
        .map_err(|e| format!("Unexpected elevated result: {e}"))?;
    *state.performance_mode.lock().unwrap() = result.mode;
    Ok(result)
}

#[tauri::command]
pub async fn get_charging_threshold(_state: State<'_, AppState>) -> Result<u8, String> {
    hw_get_charge().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_charging_threshold(
    threshold: u8,
    state: State<'_, AppState>,
) -> Result<ChargingResult, String> {
    let raw = elev_bridge::run_elevated(
        "set_charging_threshold",
        serde_json::json!({ "threshold": threshold }),
    )
    .await?;
    let result: ChargingResult = serde_json::from_value(raw)
        .map_err(|e| format!("Unexpected elevated result: {e}"))?;
    *state.charging_threshold.lock().unwrap() = result.threshold;
    Ok(result)
}

/// Returns diagnostic information about the performance mode control channel:
/// - which WMI instance was found
/// - whether a live SetPerformanceMode call succeeds
/// - current registry and overlay mode
/// - VHF device path if discovered
/// This runs in the main (non-elevated) process since it's read-only.
#[tauri::command]
pub async fn get_perf_debug() -> Result<PerfDebugInfo, String> {
    Ok(hw_perf_debug())
}

/// Read all ACPI ERAM fields via IoTDriver (direct or shim path).
///
/// On first call the shim (`ecram_shim.exe`) is deployed to the IoTDriver
/// DriverStore directory using SeRestorePrivilege.  Subsequent calls skip
/// deployment if the binary is already current.
///
/// Returns the decoded `EramMap` with all known register fields.
#[tauri::command]
pub async fn get_ecram_map() -> Result<crate::hw::ecram::EramMap, String> {
    tokio::task::spawn_blocking(crate::hw::ecram::read_eram_map)
        .await
        .map_err(|e| format!("blocking task panicked: {e}"))?
        .map_err(|e| e.to_string())
}

/// Read a named IoT region through the DriverStore shim and return it as hex.
///
/// Supported values: `ERAM`, `SMA2`, `IOT_STATUS`, `IOT_SENSORS`.
#[tauri::command]
pub async fn get_iot_region_hex(region: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || crate::hw::ecram::read_named_region_via_shim(&region))
        .await
        .map_err(|e| format!("blocking task panicked: {e}"))?
        .map(|bytes| bytes.iter().map(|b| format!("{b:02x}")).collect())
        .map_err(|e| e.to_string())
}

/// Write raw hex bytes into EC RAM through the DriverStore shim.
#[tauri::command]
pub async fn write_iot_hex(address: String, hex_data: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let addr = u64::from_str_radix(address.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow::anyhow!("invalid address: {e}"))?;

        let normalized: String = hex_data
            .chars()
            .filter(|c| !c.is_ascii_whitespace() && *c != ',' && *c != '-')
            .collect();

        anyhow::ensure!(
            !normalized.is_empty() && normalized.len() % 2 == 0,
            "hex_data must contain an even number of hex digits"
        );

        let bytes = (0..normalized.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&normalized[i..i + 2], 16).map_err(Into::into))
            .collect::<anyhow::Result<Vec<u8>>>()?;

        crate::hw::ecram::write_ecram_via_shim(addr, &bytes)
    })
    .await
    .map_err(|e| format!("blocking task panicked: {e}"))?
    .map_err(|e| e.to_string())
}

/// Read `count` bytes (1–256) from ECRAM at `address` via the DriverStore shim.
///
/// Returns the bytes as a lowercase hex string.  Requires the process to be
/// running elevated (administrator).
#[tauri::command]
pub async fn read_ecram_raw(address: String, count: u32) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let addr = u64::from_str_radix(address.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow::anyhow!("invalid address: {e}"))?;

        anyhow::ensure!(count >= 1 && count <= 256, "count must be 1–256");

        let bytes = crate::hw::ecram::read_ecram_via_shim(addr, count as usize)?;
        Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
    })
    .await
    .map_err(|e| format!("blocking task panicked: {e}"))?
    .map_err(|e: anyhow::Error| e.to_string())
}

/// Returns whether the current process is running with an elevated (Administrator) token.
#[tauri::command]
pub fn is_elevated() -> bool {
    crate::hw::ecram::is_process_elevated()
}

/// Re-launch the application as administrator (UAC prompt) and exit the current instance.
///
/// This triggers the standard Windows UAC prompt.  If the user approves, a new
/// elevated instance of the app starts and this instance exits.
#[tauri::command]
pub async fn relaunch_as_admin(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(windows)]
    {
        crate::elev_bridge::relaunch_self_as_admin()?;
        app.exit(0);
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        return Err("re-launch as admin is only supported on Windows".to_string());
    }
    #[allow(unreachable_code)]
    Ok(())
}
