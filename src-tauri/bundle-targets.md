# Bundle Targets

## Why only NSIS?

`bundle.targets` in `tauri.conf.json` is set to `["nsis"]` (not `"all"` or `["msi", "appx", "msix"]`).

### Exclusion of MSIX/AppX

MSIX/AppX installers require **code signing** to be functional:

- Unsigned MSIX/AppX packages will fail to install on most systems.
- Windows SmartScreen and security policies block unsigned AppX packages.
- The release workflow (`.github/workflows/release.yml`) conditionally signs installers — if no certificate is configured, only Authenticode signing is attempted, but MSIX/AppX would still be non-functional unsigned.

NSIS installers, on the other hand, can be distributed unsigned with only a SmartScreen warning, making them a more practical default.

### If you want MSIX/AppX

1. Obtain a code signing certificate (e.g., DigiCert, Sectigo).
2. Configure `WINDOWS_CERTIFICATE` and `WINDOWS_CERTIFICATE_PASSWORD` secrets in GitHub.
3. Add `"msi"`, `"appx"`, or `"msix"` to `bundle.targets` in `tauri.conf.json`.
