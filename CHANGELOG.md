# Changelog

All notable changes to miPC will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Custom IoTService.exe replacement binary** (`src-tauri/src/bin/ecram_service.rs`) — Rust binary that proxies ECRAM read/write IOCTLs to IoTDriver.sys via named pipe IPC (`\\.\pipe\ecram_service`, JSON protocol). Passes driver security check by being named `IoTService.exe` and placed in the DriverStore directory.
- **Pipe client in ecram.rs** — `read_ecram_via_pipe()` and `is_pipe_broker_available()` functions for communicating with the custom IoTService.exe via named pipe.
- **RE Analysis Report** (`docs/RE_ANALYSIS_REPORT.md`) — Complete reverse engineering documentation of IoTDriver.sys and IoTService.exe: IOCTL codes (`0x22E000`/`0x22E004`), buffer layout (0x110 bytes), allowed physical address ranges, security check mechanism, custom replacement design, test results, and limitations.
- **WORKING FORM comments** — 12 reverse-engineering findings documented across 5 Rust source files (`battery.rs`, `ecram.rs`, `fan.rs`, `wmi_cache.rs`, `wmi_ec.rs`) marking verified code patterns that must not be modified without re-testing against real hardware.

### Changed

- Updated `docs/iotservice-re-analysis.md` — Added Phase 2 findings (radare2 deep analysis), cross-referenced with RE_ANALYSIS_REPORT.md, updated viability assessment and next steps.
- Updated `docs/HARDWARE_INVESTIGATION.md` — Added Session 6 findings (custom IoTService.exe, allowed address ranges, ERAM/SMA2 inaccessibility, pipe client integration).
- Updated `README.md` — Added EC RAM Access feature description and architecture details for custom IoTService.exe.
- Updated `AGENTS.md` — Added hardware module inventory, key hardware interfaces table, RE documentation references, and WORKING FORM editing rules.
- Updated `docs/frontend-architecture.md` — Corrected tab count from 17 to 18 (includes dev-only ecrdebug tab).

### Known Limitations

- **ERAM region (0xFE0B0300) not accessible** — IoTDriver.sys hardcoded address ranges do not include ERAM. AC adapter wattage (ADPW at ERAM+0x81) cannot be read via driver. Use WMI as alternative.
- **SMA2 region (0xFE0B0A00) not accessible** — Same limitation as ERAM.
- **Secure Boot prevents driver modification** — IoTDriver.sys cannot be patched to add ERAM/SMA2 ranges without disabling Secure Boot.

## [1.0.0] - 2025-01-XX

### Added

- First-run onboarding wizard
- Hardware profile JSON integrity check (HMAC-signed)
- HMAC key rotation mechanism (30-day rotation, 7-day grace period)
- Nonce persistence with TTL for replay protection
- Rate limiting for IoTService IPC writes (100 writes/second)
- Consent audit log with HMAC integrity verification
- WiFi password encryption (XOR cipher with HMAC key)
- URL validation for hotkey OpenUrl (http/https only)
- Local font loading (removed Google Fonts CDN dependency)
- Manual chunks in Vite config for optimized bundle splitting
- React.memo optimization for Sidebar component
- WMI static data caching (BatteryStaticData, CPU logical processors)
- WMI cache selective invalidation (only on connection errors)
- Comprehensive clippy lint curation
- CI/CD pipeline with SHA-pinned actions, i18n checker, version checker
- Code of Conduct and Contributing guidelines
- CODEOWNERS file for code review routing
- Pre-commit hooks (tsc, version:check)
- Keyboard shortcuts for tab switching (Alt+1 through Alt+9)
- AI cost estimation and usage tracking
- User-facing error reporting channel
- Accessible labels and ARIA attributes for skeleton loaders
- prefers-reduced-motion media query for all animations

### Changed

- Migrated all hw/ modules from anyhow::Result to typed HardwareResult<T>
- Migrated commands/system.rs to Result<T, ErrorResponse>
- Replaced tokio "full" with explicit features
- Extracted Sidebar to React.memo component
- Bumped @vitest/coverage-v8 to ^3.2.2

### Removed

- Dead code (get_profile, read_or_recover, write_or_recover, spawn_with_recovery)
- Google Fonts CDN dependency

### Security

- HMAC-signed audit log with tamper detection
- Encrypted WiFi password storage
- Replay attack protection with persisted nonces
- Rate limiting on IPC writes

## [0.1.0] - 2024-XX-XX

### Added

- Initial release
- Basic hardware control (fan, battery, display, audio, keyboard, touchpad)
- IoT Service integration
- Driver management
- Multi-language support (en, pt, es, fr)

[Unreleased]: https://github.com/arcane-D7/micontrol/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/arcane-D7/micontrol/releases/tag/v1.0.0
[0.1.0]: https://github.com/arcane-D7/micontrol/releases/tag/v0.1.0
