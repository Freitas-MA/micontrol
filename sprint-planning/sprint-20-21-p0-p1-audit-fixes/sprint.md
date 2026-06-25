# Sprint 20–21 — P0/P1: Post-v1 Audit CRITICAL & HIGH Fixes

## Sprint Metadata

| Field                 | Value                                                                |
| --------------------- | -------------------------------------------------------------------- |
| **Sprint Name**       | Post-v1 Audit CRITICAL & HIGH Fixes                                  |
| **Sprint Goal**       | Resolve all CRITICAL and HIGH findings from the post-sprint-19 audit |
| **Duration Estimate** | ~2 days                                                              |
| **Priority**          | P0 + P1 — Critical & High                                            |
| **Sprint Type**       | Multi-domain (Security, Architecture, UI/UX, DevOps)                 |
| **Primary Owner**     | Full-stack engineer                                                  |
| **Source**            | Post-sprint-19 multi-agent audit (3 parallel subagents)              |
| **Depends On**        | Sprint 19                                                            |
| **Status**            | ✅ COMPLETE — commit `d514bdf`                                       |

## ⚠️ MANDATORY COMPLETION REQUIREMENT

> **OBRIGATÓRIO: 100% dos tickets desta sprint devem ser concluídos. A sprint não será aceita como entregue se qualquer ticket permanecer incompleto.**
>
> **MANDATORY: 100% of the tickets in this sprint MUST be completed. The sprint will NOT be accepted as delivered if any ticket remains incomplete.**

Every ticket must pass its acceptance criteria AND the full health check suite (9/9) before the sprint commit is made.

---

## Sprint Goal Statement

After completing Sprints 16–19, a multi-agent audit (3 parallel subagents using `umans-glm-5.2`) was run to verify the codebase state. The audit found 2 CRITICAL and 7 HIGH findings that were either regressions or missed by the previous sprints. This sprint resolves all of them in a single combined commit, plus 6 additional MEDIUM fixes found during implementation.

**Audit result:** 2 CRITICAL + 7 HIGH → 0 CRITICAL + 0 HIGH after this sprint.

---

## Health Check Commands (must pass 9/9 before commit)

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npx tsc --noEmit
npm run lint
npm run format:check
npm run build
npm run version:check
```

**Result:** All 9/9 passed. 261 Rust tests pass.

---

## Tickets

### Sprint 20 — P0 CRITICAL

#### S20-001 — EC RAM write validation in EcrDebugPanel

| Field                | Value                                                                                |
| -------------------- | ------------------------------------------------------------------------------------ |
| **Ticket ID**        | S20-001                                                                              |
| **Title**            | Add hex format validation, address range check, and confirm dialog for EC RAM writes |
| **Priority**         | P0                                                                                   |
| **Type**             | Security / Safety                                                                    |
| **Estimated Effort** | S                                                                                    |
| **Source Finding**   | C1 (CRITICAL)                                                                        |
| **Status**           | ✅ Complete                                                                          |

#### Context

`EcrDebugPanel` allowed writing arbitrary hex data to any EC RAM address without validation. This could brick the embedded controller if invalid data was written to a critical register.

#### Changes

- Hex format validation on input
- Address range check (must be within ERAM range)
- Confirmation dialog before write

---

#### S20-002 — Raw physical address write confirmation in setup.tsx

| Field                | Value                                                                 |
| -------------------- | --------------------------------------------------------------------- |
| **Ticket ID**        | S20-002                                                               |
| **Title**            | Add hex validation and danger warning for raw physical address writes |
| **Priority**         | P0                                                                    |
| **Type**             | Security / Safety                                                     |
| **Estimated Effort** | S                                                                     |
| **Source Finding**   | C2 (CRITICAL)                                                         |
| **Status**           | ✅ Complete                                                           |

#### Context

`setup.tsx` allowed writing to raw physical addresses without validation or warning. This is extremely dangerous — writing to the wrong physical address can crash the system or corrupt memory.

#### Changes

- Hex format validation on address input
- Danger warning displayed before write
- Confirmation required

---

#### S20-003 — Replace `assert!` panics with proper `Err` returns in ecram.rs

| Field                | Value                                                                          |
| -------------------- | ------------------------------------------------------------------------------ |
| **Ticket ID**        | S20-003                                                                        |
| **Title**            | Replace `assert!()` panics in `read_ecram`/`write_ecram` with `Result` returns |
| **Priority**         | P0                                                                             |
| **Type**             | Stability / Error Handling                                                     |
| **Estimated Effort** | S                                                                              |
| **Source Finding**   | H1 (HIGH)                                                                      |
| **Status**           | ✅ Complete                                                                    |

#### Context

`ecram.rs` used `assert!()` to validate parameters in `read_ecram` and `write_ecram`. These panics would crash the entire application instead of returning an error.

#### Changes

- `assert!()` replaced with `return Err(...)` in `read_ecram`
- `assert!()` replaced with `return Err(...)` in `write_ecram`
- Proper `HardwareError` variants used

---

#### S20-004 — ACL-protect nonces.json in elevated.rs

| Field                | Value                                                        |
| -------------------- | ------------------------------------------------------------ |
| **Ticket ID**        | S20-004                                                      |
| **Title**            | Call `restrict_file_acl` on `nonces.json` in `save_nonces()` |
| **Priority**         | P0                                                           |
| **Type**             | Security                                                     |
| **Estimated Effort** | XS                                                           |
| **Source Finding**   | H2 (HIGH)                                                    |
| **Status**           | ✅ Complete                                                  |

#### Context

`save_nonces()` in `elevated.rs` wrote the nonce replay-protection file without ACL restriction. Any process could read or modify the nonces, enabling replay attacks.

#### Changes

- `restrict_file_acl` called after writing `nonces.json`

---

#### S20-005 — Add `ErrorResponse::new()` constructor for non-Windows builds

| Field                | Value                                                         |
| -------------------- | ------------------------------------------------------------- |
| **Ticket ID**        | S20-005                                                       |
| **Title**            | Add `ErrorResponse::new()` constructor for non-Windows builds |
| **Priority**         | P0                                                            |
| **Type**             | Build / Cross-platform                                        |
| **Estimated Effort** | XS                                                            |
| **Source Finding**   | H3 (HIGH)                                                     |
| **Status**           | ✅ Complete                                                   |

#### Context

`ErrorResponse` had no constructor that worked on non-Windows builds, causing compilation failures on CI.

#### Changes

- `ErrorResponse::new()` constructor added in `errors.rs`

---

### Sprint 21 — P1 HIGH

#### S21-001 — Tighten CSP connect-src to specific domains

| Field                | Value                                                                            |
| -------------------- | -------------------------------------------------------------------------------- |
| **Ticket ID**        | S21-001                                                                          |
| **Title**            | Tighten CSP `connect-src` to specific domains, add `img-src` + `frame-ancestors` |
| **Priority**         | P1                                                                               |
| **Type**             | Security                                                                         |
| **Estimated Effort** | S                                                                                |
| **Source Finding**   | H4 (HIGH)                                                                        |
| **Status**           | ✅ Complete                                                                      |

#### Context

The CSP `connect-src` was too permissive (`*`), allowing connections to any origin. Missing `img-src` and `frame-ancestors` directives.

#### Changes

- `connect-src` restricted to specific domains
- `img-src 'self' data:` added
- `frame-ancestors 'none'` added

---

#### S21-002 — Use PowerShell env var for cert password in release.yml

| Field                | Value                                                                            |
| -------------------- | -------------------------------------------------------------------------------- |
| **Ticket ID**        | S21-002                                                                          |
| **Title**            | Use PowerShell environment variable for certificate password in release workflow |
| **Priority**         | P1                                                                               |
| **Type**             | Security / DevOps                                                                |
| **Estimated Effort** | XS                                                                               |
| **Source Finding**   | H5 (HIGH)                                                                        |
| **Status**           | ✅ Complete                                                                      |

#### Context

The release workflow passed the certificate password as a CLI argument, which is visible in process listings and CI logs.

#### Changes

- Password passed via PowerShell environment variable instead of CLI arg

---

#### S21-003 — Replace suspicious SHA-pinned actions with version tags

| Field                | Value                                                                       |
| -------------------- | --------------------------------------------------------------------------- |
| **Ticket ID**        | S21-003                                                                     |
| **Title**            | Replace SHA-pinned GitHub Actions with version tags in ci.yml + release.yml |
| **Priority**         | P1                                                                          |
| **Type**             | DevOps / Security                                                           |
| **Estimated Effort** | XS                                                                          |
| **Source Finding**   | H6 (HIGH)                                                                   |
| **Status**           | ✅ Complete                                                                 |

#### Context

SHA-pinned actions from S13 used SHAs that couldn't be verified against the official repositories. The audit flagged these as suspicious. Since SHA pinning was added prematurely (without a verification process), version tags were restored.

#### Changes

- SHA-pinned actions in `ci.yml` replaced with `@v4` / `@v5` version tags
- SHA-pinned actions in `release.yml` replaced with version tags

---

#### S21-004 — Remove `shell:default` permission from capabilities

| Field                | Value                                                              |
| -------------------- | ------------------------------------------------------------------ |
| **Ticket ID**        | S21-004                                                            |
| **Title**            | Remove `shell:default` permission from `capabilities/default.json` |
| **Priority**         | P1                                                                 |
| **Type**             | Security / Attack Surface                                          |
| **Estimated Effort** | XS                                                                 |
| **Source Finding**   | H7 (HIGH)                                                          |
| **Status**           | ✅ Complete                                                        |

#### Context

`capabilities/default.json` granted `shell:default` permission, allowing any shell command to be executed. This is an unnecessary attack surface for a hardware control app.

#### Changes

- `shell:default` permission removed from `capabilities/default.json`

---

### Additional MEDIUM Fixes (included in same commit)

| Ticket | Finding | Description                                                                   |
| ------ | ------- | ----------------------------------------------------------------------------- |
| S20-M1 | M1      | Make timestamp freshness check mandatory (fail-closed) in `auth.rs`           |
| S20-M2 | M2      | Make nonce anti-replay check mandatory (fail-closed) in `elevated.rs`         |
| S20-M3 | M3      | Validate `reveal_in_explorer` path is within app data dir in `privacy.rs`     |
| S20-M4 | M4      | Add error handling to `StartupManager` toggle (revert on failure)             |
| S20-M5 | M5      | Add accessibility attributes to `ProcessModal` (role, aria-modal, aria-label) |
| S20-M6 | M6      | Use dynamic `__APP_VERSION__` in `about.tsx` instead of hardcoded `0.1.0`     |

---

## Files Changed (15 files, +236 / -75)

| File                                      | Changes                                                    |
| ----------------------------------------- | ---------------------------------------------------------- |
| `.github/workflows/ci.yml`                | S21-003: Replace SHA-pinned actions with version tags      |
| `.github/workflows/release.yml`           | S21-002: Cert password via env var; S21-003: Version tags  |
| `src-tauri/capabilities/default.json`     | S21-004: Remove shell:default                              |
| `src-tauri/gen/schemas/capabilities.json` | S21-004: Schema update                                     |
| `src-tauri/src/commands/privacy.rs`       | S20-M3: Path validation for reveal_in_explorer             |
| `src-tauri/src/elevated.rs`               | S20-004: ACL on nonces.json; S20-M2: Mandatory nonce check |
| `src-tauri/src/hw/ecram.rs`               | S20-003: Replace assert! with Err returns                  |
| `src-tauri/src/hw/errors.rs`              | S20-005: ErrorResponse::new() constructor                  |
| `src-tauri/src/util/auth.rs`              | S20-M1: Mandatory timestamp check                          |
| `src-tauri/tauri.conf.json`               | S21-001: Tightened CSP                                     |
| `src/components/EcrDebugPanel.tsx`        | S20-001: EC RAM write validation                           |
| `src/components/ProcessModal.tsx`         | S20-M5: Accessibility attributes                           |
| `src/components/StartupManager.tsx`       | S20-M4: Error handling on toggle                           |
| `src/pages/tabs/about.tsx`                | S20-M6: Dynamic **APP_VERSION**                            |
| `src/pages/tabs/setup.tsx`                | S20-002: Raw address write confirmation                    |

---

## Sprint Commit

```
d514bdf fix(sprint-20-21): resolve all CRITICAL and HIGH audit findings
```
