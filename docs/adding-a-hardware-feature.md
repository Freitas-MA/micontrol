# Adding a Hardware Feature

This guide walks through adding a new hardware module to the miPC HAL (Hardware Abstraction Layer).

## 6-Step Checklist

### 1. Create the HAL Module

Create `src-tauri/src/hw/your_feature.rs`:

```rust
//! Hardware access module for [your feature].

use crate::hw::errors::{HardwareError, HardwareResult};

/// Get the current state of [your feature].
pub fn get_your_feature_info() -> HardwareResult<YourFeatureInfo> {
    // Implementation
}

#[derive(serde::Serialize)]
pub struct YourFeatureInfo {
    pub value: u32,
}
```

### 2. Register the Module

Add `pub mod your_feature;` to `src-tauri/src/hw/mod.rs`.

### 3. Add Error Variants

Add error variants to `src-tauri/src/hw/errors.rs`:

```rust
pub enum HardwareError {
    // ... existing variants
    YourFeatureError(String),
}
```

### 4. Create Command Handlers

Add command functions in `src-tauri/src/commands/hardware.rs`:

```rust
#[tauri::command]
pub fn get_your_feature() -> Result<YourFeatureInfo, ErrorResponse> {
    hw::your_feature::get_your_feature_info()
        .map_err(|e| ErrorResponse::from(e))
}
```

### 5. Register Commands

Add the command to `generate_handler!` in `src-tauri/src/lib.rs`:

```rust
commands::hardware::get_your_feature,
```

### 6. Add Frontend Support

1. Add a hook in `src/hooks/useHardware.ts`
2. Add a UI component in `src/components/` or `src/pages/`
3. Add i18n keys to all locale files
4. Add tests

## Testing

Add unit tests in the HAL module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_feature_info() {
        // Test the info structure
    }
}
```

Add integration tests in `src-tauri/tests/` if the feature involves the elevated bridge.
