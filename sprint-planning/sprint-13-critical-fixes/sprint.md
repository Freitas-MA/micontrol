# Sprint 13 — Post-Audit Critical Fixes (P0)

## Sprint Metadata

| Field                 | Value                                                                                                                                                                  |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Sprint Name**       | Post-Audit Critical Fixes                                                                                                                                              |
| **Sprint Goal**       | Fix all CRITICAL findings from the post-sprint stability report — SHA pinning, consent audit log, policy version, ACL hotkeys, CI hardening, i18n consent translations |
| **Duration Estimate** | ~5 days                                                                                                                                                                |
| **Priority**          | P0 — Critical, blocks GA                                                                                                                                               |
| **Sprint Type**       | Multi-domain (Security, DevOps, RAI)                                                                                                                                   |
| **Primary Owner**     | Full-stack engineer                                                                                                                                                    |
| **Source**            | `docs/stability-report-2026-06-24-post-sprints.md`                                                                                                                     |
| **Depends On**        | Sprint 9, 10, 11, 12                                                                                                                                                   |

## ⚠️ MANDATORY COMPLETION REQUIREMENT

> **OBRIGATÓRIO: 100% dos tickets desta sprint devem ser concluídos. A sprint não será aceita como entregue se qualquer ticket permanecer incompleto.**
>
> **MANDATORY: 100% of the tickets in this sprint MUST be completed. The sprint will NOT be accepted as delivered if any ticket remains incomplete.**

Every ticket must pass its acceptance criteria AND the full health check suite (9/9) before the sprint commit is made. Partially completed tickets from previous sprints (e.g., S12-015 SHA pinning that only added TODO comments) are explicitly forbidden — if a ticket is picked up, it must be fully done.

---

## Sprint Goal Statement

The post-sprint stability audit revealed 17 CRITICAL findings across 8 domains. Several Sprint 9–12 tickets were only partially completed — SHA pinning added TODO comments instead of actual SHAs, consent audit log functions were defined but never called, and the policy version mismatch was never fixed. This sprint closes every CRITICAL gap with no exceptions.

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

---

## Tickets

### S13-001 — Pin all GitHub Actions to commit SHAs

| Field                | Value                                                         |
| -------------------- | ------------------------------------------------------------- |
| **Ticket ID**        | S13-001                                                       |
| **Title**            | Pin all GitHub Actions to commit SHAs (replace floating tags) |
| **Priority**         | P0                                                            |
| **Type**             | Security / DevOps                                             |
| **Estimated Effort** | XS                                                            |
| **Source Finding**   | SEC-001 / DEV-001 (CRITICAL, CWE-1357)                        |
| **Regression of**    | S12-015 (incomplete — only TODO comments added)               |

#### Context

S12-015 claimed to pin GitHub Actions to commit SHAs but only added TODO comments. All actions still use floating tags (`@v4`, `@stable`, `@v5`). This is a supply-chain attack vector.

#### Acceptance Criteria

- [ ] Every `uses:` in `.github/workflows/ci.yml` uses full 40-char commit SHA with `# v4` comment
- [ ] Every `uses:` in `.github/workflows/release.yml` uses full 40-char commit SHA with comment
- [ ] Every `uses:` in `.github/workflows/driverstore-integrity.yml` uses full 40-char commit SHA with comment
- [ ] No floating tags (`@v4`, `@stable`, `@latest`, `@main`) remain in any workflow file
- [ ] All TODO comments from S12-015 are removed (replaced by actual SHAs)

---

### S13-002 — Wire consent audit log calls into consent flow

| Field                | Value                                                                          |
| -------------------- | ------------------------------------------------------------------------------ |
| **Ticket ID**        | S13-002                                                                        |
| **Title**            | Wire `log_consent_granted()` and `log_consent_revoked()` into the consent flow |
| **Priority**         | P0                                                                             |
| **Type**             | RAI / Security                                                                 |
| **Estimated Effort** | S                                                                              |
| **Source Finding**   | RAI-002 / DOC-002 (CRITICAL, GDPR Art.30 violation)                            |
| **Regression of**    | S11-014 (incomplete — functions defined but never called)                      |

#### Context

`consent_audit.rs` defines `log_consent_granted()` and `log_consent_revoked()` but they are dead code — never called from anywhere. This means the consent audit log is never written, violating GDPR Art.30 (records of processing activities).

#### Acceptance Criteria

- [ ] `log_consent_granted()` is called when user grants consent (in the consent grant Tauri command)
- [ ] `log_consent_revoked()` is called when user revokes consent (in the consent revoke Tauri command)
- [ ] Audit log entries include: timestamp, user action (grant/revoke), policy version, consent scope
- [ ] Audit log file is created at `%APPDATA%/miPC/consent_audit.log` on first write
- [ ] Unit test verifies log entry is written when consent is granted
- [ ] Unit test verifies log entry is written when consent is revoked
- [ ] No clippy warnings introduced

---

### S13-003 — Fix policy version mismatch (v1 → v2 in frontend)

| Field                | Value                                                                      |
| -------------------- | -------------------------------------------------------------------------- |
| **Ticket ID**        | S13-003                                                                    |
| **Title**            | Fix `policyVersion: 1` → `2` in `useSettings.ts` and export POLICY_VERSION |
| **Priority**         | P0                                                                         |
| **Type**             | RAI                                                                        |
| **Estimated Effort** | XS                                                                         |
| **Source Finding**   | RAI-001 / DOC-003 (CRITICAL)                                               |
| **Regression of**    | S11-015 (incomplete — frontend never updated)                              |

#### Context

Rust backend uses `POLICY_VERSION = 2`, but frontend `setTelemetryConsent()` in `useSettings.ts:376` hardcodes `policyVersion: 1`. The frontend has no `POLICY_VERSION` constant at all. This means consent records store the wrong version, invalidating audit trail integrity.

#### Acceptance Criteria

- [ ] `useSettings.ts` imports or defines `POLICY_VERSION = 2` matching the Rust constant
- [ ] `setTelemetryConsent()` uses the constant instead of hardcoded `1`
- [ ] `npm run version:check` passes (all versions consistent)
- [ ] No other hardcoded policy version values exist in the frontend
- [ ] Unit test or type check confirms the value matches

---

### S13-004 — Add ACL protection to hotkey config file

| Field                | Value                                                                                |
| -------------------- | ------------------------------------------------------------------------------------ |
| **Ticket ID**        | S13-004                                                                              |
| **Title**            | Add ACL protection to `hotkeys.json` config file (Script/LaunchApp injection vector) |
| **Priority**         | P0                                                                                   |
| **Type**             | Security                                                                             |
| **Estimated Effort** | S                                                                                    |
| **Source Finding**   | SEC-002 (CRITICAL, CWE-1104)                                                         |

#### Context

The hotkey config file (`hotkeys.json`) stores `Script` and `LaunchApp` actions but has no ACL protection. Any process can modify it to inject arbitrary scripts or launch arbitrary applications when a hotkey is pressed.

#### Acceptance Criteria

- [ ] `hotkeys.json` is created with ACL restricting write access to the current user only
- [ ] ACL is applied on first write (not just on existing files)
- [ ] If file already exists without ACL, it is upgraded on next load
- [ ] Unit test verifies ACL is applied (or at least that the function is called)
- [ ] Log warning if ACL cannot be applied (but don't crash)
- [ ] No clippy warnings introduced

---

### S13-005 — Encrypt WiFi passwords in IoT pipe

| Field                | Value                                                    |
| -------------------- | -------------------------------------------------------- |
| **Ticket ID**        | S13-005                                                  |
| **Title**            | Encrypt WiFi passwords transmitted over local named pipe |
| **Priority**         | P0                                                       |
| **Type**             | Security                                                 |
| **Estimated Effort** | M                                                        |
| **Source Finding**   | SEC-003 (CRITICAL, CWE-312)                              |

#### Context

WiFi passwords are transmitted in plaintext over the local named pipe between the frontend service and the IoT service. Any process with pipe access can sniff passwords.

#### Acceptance Criteria

- [ ] WiFi passwords are encrypted before being written to the pipe
- [ ] Decryption uses a session key established at pipe connection time
- [ ] Plaintext password never appears in pipe payload
- [ ] Unit test verifies password is encrypted in transit
- [ ] Backward compatibility: if old format is received, handle gracefully (or reject with clear error)
- [ ] No clippy warnings introduced

---

### S13-006 — Add `permissions: read-all` to CI workflow

| Field                | Value                                                  |
| -------------------- | ------------------------------------------------------ |
| **Ticket ID**        | S13-006                                                |
| **Title**            | Add explicit `permissions: read-all` block to `ci.yml` |
| **Priority**         | P0                                                     |
| **Type**             | DevOps                                                 |
| **Estimated Effort** | XS                                                     |
| **Source Finding**   | DEV-002 (CRITICAL)                                     |

#### Context

`ci.yml` has no explicit `permissions:` block, so GitHub defaults to `write-all` for the `GITHUB_TOKEN`. This is a privilege escalation risk if a PR is crafted to exfiltrate the token.

#### Acceptance Criteria

- [ ] `permissions: read-all` is set at the top level of `ci.yml`
- [ ] Individual jobs that need write access (if any) have explicit `permissions:` overrides
- [ ] CI workflow still passes after the change

---

### S13-007 — Add `version:check` job to CI

| Field                | Value                                  |
| -------------------- | -------------------------------------- |
| **Ticket ID**        | S13-007                                |
| **Title**            | Add `version:check` job to CI workflow |
| **Priority**         | P0                                     |
| **Type**             | DevOps                                 |
| **Estimated Effort** | XS                                     |
| **Source Finding**   | DEV-003 (HIGH)                         |

#### Context

`npm run version:check` exists locally but is not run in CI. Version mismatches between `package.json`, `Cargo.toml`, and `tauri.conf.json` can slip through.

#### Acceptance Criteria

- [ ] New `version-check` job added to `ci.yml`
- [ ] Job runs `npm run version:check`
- [ ] Job is required for branch protection (documented in PR template or CONTRIBUTING)
- [ ] CI passes with the new job

---

### S13-008 — Fix `@vitest/coverage-v8` version mismatch

| Field                | Value                                                              |
| -------------------- | ------------------------------------------------------------------ |
| **Ticket ID**        | S13-008                                                            |
| **Title**            | Fix `@vitest/coverage-v8` major version mismatch (v2 vs vitest v3) |
| **Priority**         | P0                                                                 |
| **Type**             | DevOps                                                             |
| **Estimated Effort** | XS                                                                 |
| **Source Finding**   | DEV-005 (HIGH)                                                     |

#### Context

`package.json` has `@vitest/coverage-v8` at v2 but `vitest` at v3. This causes coverage reporting to fail silently or produce incorrect results.

#### Acceptance Criteria

- [ ] `@vitest/coverage-v8` version matches `vitest` major version (v3)
- [ ] `npm install` completes without peer dependency warnings
- [ ] `npm run build` passes
- [ ] Coverage reporting works (if configured)

---

### S13-009 — Add `-- -D warnings` to CI clippy step

| Field                | Value                                                                          |
| -------------------- | ------------------------------------------------------------------------------ |
| **Ticket ID**        | S13-009                                                                        |
| **Title**            | Add `-- -D warnings` to CI clippy step (deny all warnings)                     |
| **Priority**         | P0                                                                             |
| **Type**             | DevOps                                                                         |
| **Estimated Effort** | XS                                                                             |
| **Source Finding**   | DEV-006 (HIGH, REGRESSION)                                                     |
| **Regression of**    | S11-024 (incomplete — `continue-on-error` removed but `-D warnings` not added) |

#### Context

S11-024 claimed to remove `continue-on-error: true` from CI clippy, but the CI still doesn't use `-- -D warnings`. Clippy warnings may pass CI silently.

#### Acceptance Criteria

- [ ] CI clippy step uses `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
- [ ] No `continue-on-error` on the clippy step
- [ ] Local `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` passes with 0 warnings
- [ ] CI passes with the stricter clippy

---

### S13-010 — Add i18n completeness checker to CI

| Field                | Value                                        |
| -------------------- | -------------------------------------------- |
| **Ticket ID**        | S13-010                                      |
| **Title**            | Add i18n completeness checker to CI workflow |
| **Priority**         | P0                                           |
| **Type**             | DevOps                                       |
| **Estimated Effort** | XS                                           |
| **Source Finding**   | DEV-004 (HIGH)                               |

#### Context

There is no CI check for i18n completeness. Missing translations in PT/ES/FR slip through undetected.

#### Acceptance Criteria

- [ ] A script (e.g., `scripts/check-i18n.mjs` or similar) compares all locale files against `en.json`
- [ ] Script reports missing keys in any locale
- [ ] CI job runs the script and fails if any keys are missing
- [ ] Script is documented in CONTRIBUTING.md
- [ ] CI passes with the new job

---

### S13-011 — Complete consent dialog + privacy policy translations (pt/es/fr)

| Field                | Value                                                                  |
| -------------------- | ---------------------------------------------------------------------- |
| **Ticket ID**        | S13-011                                                                |
| **Title**            | Complete consent dialog and privacy policy translations for PT/ES/FR   |
| **Priority**         | P0                                                                     |
| **Type**             | RAI                                                                    |
| **Estimated Effort** | M                                                                      |
| **Source Finding**   | RAI-004 (HIGH)                                                         |
| **Regression of**    | S11-029 (incomplete — 261 keys added but major sections still missing) |

#### Context

PT/ES/FR locales are missing entire sections: consent dialog, privacy policy, keyboard, AI analysis. S11-029 added 261 keys but left critical sections untranslated.

#### Acceptance Criteria

- [ ] All keys in `en.json` have corresponding translations in `pt.json`, `es.json`, `fr.json`
- [ ] Consent dialog strings are translated in all 4 locales
- [ ] Privacy policy strings are translated in all 4 locales
- [ ] AI analysis strings are translated in all 4 locales
- [ ] Keyboard shortcut strings are translated in all 4 locales
- [ ] i18n checker (S13-010) passes with 0 missing keys
- [ ] No placeholder mismatches (`{{count}}`, `{{name}}`, etc.)

---

### S13-012 — Fix `data_deletion.rs` to clear keyring consent

| Field                | Value                                                                  |
| -------------------- | ---------------------------------------------------------------------- |
| **Ticket ID**        | S13-012                                                                |
| **Title**            | Fix data deletion to clear keyring consent and purge consent audit log |
| **Priority**         | P0                                                                     |
| **Type**             | RAI / Security                                                         |
| **Estimated Effort** | XS                                                                     |
| **Source Finding**   | RAI-003 (HIGH) + SEC-007 (HIGH, GDPR Art.17)                           |

#### Context

When user revokes consent, `data_deletion.rs` does not clear the keyring consent entry. Additionally, the consent audit log is not purged during data deletion, violating GDPR Art.17 (right to erasure).

#### Acceptance Criteria

- [ ] `data_deletion.rs` clears the keyring consent entry when consent is revoked
- [ ] `data_deletion.rs` purges the consent audit log file on full data deletion
- [ ] Unit test verifies keyring consent is cleared
- [ ] Unit test verifies audit log file is deleted
- [ ] No clippy warnings introduced

---

### S13-013 — Add HMAC integrity to consent audit log

| Field                | Value                                                      |
| -------------------- | ---------------------------------------------------------- |
| **Ticket ID**        | S13-013                                                    |
| **Title**            | Add HMAC integrity protection to consent audit log entries |
| **Priority**         | P0                                                         |
| **Type**             | Security                                                   |
| **Estimated Effort** | S                                                          |
| **Source Finding**   | SEC-006 (HIGH, CWE-778)                                    |

#### Context

The consent audit log has no integrity protection. An attacker with file access can modify or delete log entries without detection.

#### Acceptance Criteria

- [ ] Each audit log entry is HMAC-signed with a dedicated key
- [ ] HMAC key is stored separately from the log file (e.g., in keyring or OS credential store)
- [ ] A verification function exists that can validate the entire log chain
- [ ] Tampered entries are detected and logged as warnings
- [ ] Unit test verifies HMAC is appended to each entry
- [ ] Unit test verifies tampered entry is detected
- [ ] No clippy warnings introduced

---

### S13-014 — Validate URL scheme in OpenUrl hotkey action

| Field                | Value                                                                   |
| -------------------- | ----------------------------------------------------------------------- |
| **Ticket ID**        | S13-014                                                                 |
| **Title**            | Validate URL scheme in `OpenUrl` hotkey action (prevent code execution) |
| **Priority**         | P0                                                                      |
| **Type**             | Security                                                                |
| **Estimated Effort** | XS                                                                      |
| **Source Finding**   | SEC-008 (HIGH, CWE-807)                                                 |

#### Context

`OpenUrl` uses `explorer.exe` subprocess to open URLs. Without scheme validation, a crafted `file://` or `javascript:` URL could execute arbitrary code.

#### Acceptance Criteria

- [ ] Only `http://` and `https://` URL schemes are allowed
- [ ] Invalid schemes are rejected with a clear error message
- [ ] Unit test verifies `http://` and `https://` are accepted
- [ ] Unit test verifies `file://`, `javascript:`, `data:` are rejected
- [ ] No clippy warnings introduced

---

### S13-015 — Migrate `commands/system.rs` to `Result<T, ErrorResponse>`

| Field                | Value                                                                                    |
| -------------------- | ---------------------------------------------------------------------------------------- |
| **Ticket ID**        | S13-015                                                                                  |
| **Title**            | Migrate `commands/system.rs` to `Result<T, ErrorResponse>` for consistent error handling |
| **Priority**         | P0                                                                                       |
| **Type**             | Architecture                                                                             |
| **Estimated Effort** | M                                                                                        |
| **Source Finding**   | ARCH-002 (CRITICAL)                                                                      |

#### Context

`commands/hardware.rs` uses `ErrorResponse` but `commands/system.rs` still uses `anyhow::Result`. This creates inconsistent error boundaries at the IPC layer.

#### Acceptance Criteria

- [ ] All functions in `commands/system.rs` return `Result<T, ErrorResponse>`
- [ ] Error messages are user-friendly (not raw `anyhow` chains)
- [ ] Frontend receives structured error responses with error codes
- [ ] `cargo check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] No frontend changes needed (error format is backward compatible)

---

## Sprint Completion Checklist

Before committing the sprint, verify ALL of the following:

- [ ] All 15 tickets have their acceptance criteria fully met
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml --check` passes
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml` passes
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` passes (0 warnings)
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` passes (all tests green)
- [ ] `npx tsc --noEmit` passes (0 errors)
- [ ] `npm run lint` passes (0 warnings)
- [ ] `npm run format:check` passes
- [ ] `npm run build` passes
- [ ] `npm run version:check` passes
- [ ] No TODO comments left from S12-015 (replaced by actual SHAs)
- [ ] No dead code from S11-014 (consent audit functions are now called)
- [ ] No policy version mismatch (frontend uses v2)
- [ ] i18n checker passes with 0 missing keys
- [ ] Sprint commit message follows format: `feat(sprint-13): <summary>`

---

## Commit Message Template

```
feat(sprint-13): fix all critical post-audit findings

S13-001: Pin all GitHub Actions to commit SHAs
S13-002: Wire consent audit log calls into consent flow
S13-003: Fix policy version mismatch (v1 → v2 in frontend)
S13-004: Add ACL protection to hotkey config file
S13-005: Encrypt WiFi passwords in IoT pipe
S13-006: Add permissions: read-all to CI workflow
S13-007: Add version:check job to CI
S13-008: Fix @vitest/coverage-v8 version mismatch
S13-009: Add -- -D warnings to CI clippy step
S13-010: Add i18n completeness checker to CI
S13-011: Complete consent dialog + privacy policy translations (pt/es/fr)
S13-012: Fix data_deletion.rs to clear keyring consent
S13-013: Add HMAC integrity to consent audit log
S13-014: Validate URL scheme in OpenUrl hotkey action
S13-015: Migrate commands/system.rs to Result<T, ErrorResponse>
```
