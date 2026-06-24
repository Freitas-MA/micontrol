# Crash Reporting

This application uses [Sentry](https://sentry.io) for crash reporting.

## Privacy

- Crash reports are ONLY sent if the user has granted telemetry consent
- Reports contain: error messages, stack traces, app version, OS info
- Reports do NOT contain: personal data, API keys, or file contents
- Users can revoke consent at any time in Settings → Privacy

## Setup

### Backend (Rust)

Set the `SENTRY_DSN` environment variable before building:

```bash
export SENTRY_DSN="https://your-dsn@sentry.io/project-id"
```

Or set it at runtime (the app reads `SENTRY_DSN` from the environment on startup).

### Frontend (React)

Set the `VITE_SENTRY_DSN` environment variable in `.env`:

```
VITE_SENTRY_DSN=https://your-dsn@sentry.io/project-id
```

## Behavior

- If the DSN is not set, Sentry is a no-op — no data is sent.
- If the DSN is set but telemetry consent has not been granted, Sentry is initialized on the Rust side but the frontend skips initialization.
- Telemetry consent is stored in the OS credential store and checked at startup.

## Architecture

| Layer          | Library                | Init condition                                                      |
| -------------- | ---------------------- | ------------------------------------------------------------------- |
| Rust backend   | `sentry` 0.34          | `SENTRY_DSN` env var is set and non-empty                           |
| React frontend | `@sentry/react` ^8.0.0 | `VITE_SENTRY_DSN` env var is set AND telemetry consent is `granted` |
