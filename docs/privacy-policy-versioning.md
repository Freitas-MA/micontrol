# Privacy Policy Versioning

## How It Works

The app tracks a `POLICY_VERSION` constant (currently `2`). When the privacy policy changes:

1. Update `POLICY_VERSION` in `src-tauri/src/util/consent_audit.rs`
2. Update the privacy policy text in `src/components/ConsentDialog.tsx`
3. Add translations for the updated text

When the app starts, it compares the stored policy version with the current one. If they differ, the ConsentDialog is re-shown, requiring the user to re-consent.

## Audit Log

All consent events (grant/revoke) are logged to `consent_audit.log` in the app data directory with:

- Timestamp
- Event type (CONSENT_GRANTED / CONSENT_REVOKED)
- Policy version at the time of the event

## Frontend Integration

- `useSettings()` exposes `getStoredPolicyVersion()` to read the version stored with the consent record
- `MainWindow.tsx` checks both consent status and policy version on mount
- If consent is `null` (first launch) or the stored version differs from `POLICY_VERSION`, the ConsentDialog is shown
- Consent records now store `policyVersion` in the JSON payload

## Adding a New Policy Version

1. Increment `POLICY_VERSION` in `src-tauri/src/util/consent_audit.rs`
2. Update `POLICY_VERSION` in `src/pages/MainWindow.tsx`
3. Update the consent dialog text in `src/components/ConsentDialog.tsx`
4. Update translations in the i18n files
5. The next app start will detect the version mismatch and re-prompt
