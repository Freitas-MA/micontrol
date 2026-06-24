# Release Process

This document describes how to cut a release of MiControl, including build, signing, and publication.

## Prerequisites

### Tauri Updater Signing Key

The Tauri updater uses an Ed25519 keypair to sign update bundles. The private key signs the update; the public key (embedded in the app) verifies it.

**Generate a keypair:**

```bash
npx @tauri-apps/cli signer generate -w tauri-key
```

This creates a file containing the private key and prints the public key.

**Configure GitHub Secrets:**

- `TAURI_SIGNING_PRIVATE_KEY` — the contents of the private key file
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — the password (if you set one)

**Configure the public key:**

Replace the `pubkey` placeholder in `src-tauri/tauri.conf.json` with the generated public key.

### Authenticode Code Signing (Optional but recommended)

To avoid SmartScreen warnings:

1. Obtain a code signing certificate (OV or EV) from a trusted CA.
2. Export it as a PFX file.
3. Base64-encode it: `base64 -w0 cert.pfx > cert.b64`
4. Add two repository secrets:
   - `WINDOWS_CERTIFICATE`: the base64-encoded PFX
   - `WINDOWS_CERTIFICATE_PASSWORD`: the PFX password

If these secrets are not set, the release will succeed but the installer will be unsigned.

## Cutting a Release

1. **Ensure all versions are synced:**

   ```bash
   npm run version:check
   ```

2. **Bump the version (if not already done):**

   ```bash
   npm run version:bump 1.2.3
   git add -A
   git commit -m "chore: bump version to 1.2.3"
   ```

3. **Create and push a tag:**

   ```bash
   git tag v1.2.3
   git push origin v1.2.3
   ```

4. **The release workflow runs automatically:**

   - Builds the Tauri app for Windows
   - Signs the update bundle with the Tauri signing key
   - Creates a GitHub Release with the artifacts

5. **Verify the release:**

   - Check the GitHub Actions run completed successfully
   - Download the artifacts and verify the signature
   - Test the updater by installing the previous version and updating

## Key Rotation

### Tauri Updater Key

1. Generate a new keypair
2. Update the `pubkey` in `tauri.conf.json`
3. Update the `TAURI_SIGNING_PRIVATE_KEY` GitHub secret
4. Release a new version with the new key
5. Old versions will not be able to update to the new version (key mismatch) — document this as a breaking change

### Windows Code Signing Certificate

1. Obtain a new certificate before the old one expires
2. Update the `WINDOWS_CERTIFICATE` and `WINDOWS_CERTIFICATE_PASSWORD` GitHub secrets
3. No app-side changes needed (the OS trusts the CA, not a specific cert)

## Certificate Expiry Tracking

- **Tauri updater key**: No expiry (Ed25519 keys don't expire)
- **Windows code signing cert**: Track the expiry date in your team calendar; renew at least 30 days before expiry

## Rollback Procedure

If a release needs to be rolled back:

### 1. Mark the Release as Draft (GitHub)

1. Go to the GitHub Releases page
2. Find the release to roll back
3. Click "Edit" (pencil icon)
4. Change the status to "Draft"
5. This hides the release from public view

### 2. Revert the Commit

```bash
git revert <release-commit-hash>
git push origin master
```

### 3. Re-publish Previous Version

1. Find the previous stable release tag
2. Re-publish the release artifacts from the previous tag
3. Update `latest.json` to point to the previous version

### 4. Notify Users

- Post an announcement in the release notes
- Update the download page if applicable

### 5. Post-Mortem

- Document the reason for rollback
- Add a regression test to prevent recurrence

## Troubleshooting

- **Build fails**: Check that all versions are synced (`npm run version:check`)
- **Signing fails**: Verify the `TAURI_SIGNING_PRIVATE_KEY` secret is set correctly
- **Release not created**: Ensure the workflow has `permissions: contents: write`
