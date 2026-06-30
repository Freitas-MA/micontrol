# Repository Agent Rules — miPC/micontrol

## Project Type

This is a **Tauri v2 + React/TypeScript desktop application** with Rust backend and web frontend.

## Architecture

- `src-tauri/` — Rust backend, Tauri configuration, native APIs
- `src-tauri/src/hw/` — Hardware abstraction layer (HAL) modules:
  - `battery.rs` — Battery health & AC adapter via WMI
  - `display.rs` — Brightness & HDR
  - `ecram.rs` — EC RAM access via IoTDriver.sys IOCTLs + named pipe client for custom IoTService.exe
  - `fan.rs` — Fan speed monitoring & performance mode via WMI
  - `wmi_ec.rs` — WMI-based EC read/write (MICommonInterface, root\WMI)
  - `wmi_cache.rs` — WMI connection caching with RefCell pattern
  - `thermal.rs`, `hotkeys/`, etc.
- `src-tauri/src/bin/ecram_service.rs` — Custom IoTService.exe replacement binary (Windows service + named pipe server + IOCTL proxy)
- `src/` — Frontend React application (Vite-based, 18 lazy-loaded tabs)
- `index.html` — Entrypoint
- Uses **Vite** as frontend build tool (`vite.config.ts`)

### Key Hardware Interfaces

| Interface                                    | Purpose                                                  | Status                                 |
| -------------------------------------------- | -------------------------------------------------------- | -------------------------------------- |
| WMI (`MICommonInterface`, root\WMI)          | Performance mode, battery health, fan RPM, adapter power | ✅ Working                             |
| IoTDriver.sys IOCTLs (`0x22E000`/`0x22E004`) | EC RAM read/write                                        | ✅ Working (via custom IoTService.exe) |
| Named pipe (`\\.\pipe\ecram_service`)        | IPC between MiControl and custom IoTService.exe          | ✅ Working                             |
| ERAM/SMA2 regions                            | AC adapter wattage, additional EC data                   | ❌ Not accessible (driver blocks)      |

### Reverse Engineering Documentation

- `docs/RE_ANALYSIS_REPORT.md` — Complete RE report (IoTDriver.sys IOCTLs, buffer layout, security check, allowed address ranges, custom replacement binary)
- `docs/HARDWARE_INVESTIGATION.md` — Consolidated hardware investigation (ACPI DSDT, WMI WMAA, EC RAM field map, hotkey events, IoTService IPC protocol)
- `docs/iotservice-re-analysis.md` — Phase 1 analysis (Ghidra strings, IPC command mapping, original IoTService.exe architecture)
- `docs/architecture.md` — System architecture overview with EC RAM access architecture diagram
- `docs/adding-a-hardware-feature.md` — Guide for adding new HAL modules, includes WORKING FORM guidelines

## Commands

Use commands defined in `package.json` and Tauri CLI. Common commands:

```bash
# Install frontend deps
npm install

# Dev (Vite + Tauri)
npm run tauri dev

# Build desktop app
npm run tauri build

# Frontend only
npm run dev
```

For Rust side, standard Cargo commands apply in `src-tauri/`:

```bash
# Check Rust
cargo check --manifest-path src-tauri/Cargo.toml

# Build Rust
cargo build --manifest-path src-tauri/Cargo.toml

# Build ecram_service.exe (custom IoTService.exe replacement)
cargo build --manifest-path src-tauri/Cargo.toml --release --bin ecram_service
```

## Validation

Before finishing executable code changes:

- If frontend changed: run `npm run build` (Vite build must pass).
- If Rust changed: run `cargo check --manifest-path src-tauri/Cargo.toml`.
- Tauri dev must start without runtime errors.

## Editing Rules

- Frontend vs backend separation: `src/` is web, `src-tauri/src/` is Rust.
- Use Tauri commands (invoke/handle) for IPC between frontend and backend.
- Do not bypass Tauri security model.
- Keep frontend framework-agnostic where possible (Tauri supports any web frontend).
- **WORKING FORM comments** — Code marked with `// WORKING FORM — DO NOT MODIFY` has been reverse-engineered and verified against the real hardware. Do NOT change the logic, API call patterns, or buffer layouts in these sections without re-testing against the actual driver/WMI interface. See `docs/RE_ANALYSIS_REPORT.md` for technical context.
- **ecram.rs pipe client** — The `read_ecram_via_pipe()` and `is_pipe_broker_available()` functions communicate with the custom `ecram_service.exe` binary via named pipe. The pipe name is `\\.\pipe\ecram_service`.
- **ecram_service.rs** — Must be named `IoTService.exe` when deployed, and placed in the IoTDriver DriverStore directory to pass the driver's security check.

## Do Not Assume

- Do not assume this repository is Brainiak.
- Do not apply Brainiak project structure here.
- Do not assume monorepo or workspace boundaries unless confirmed.
