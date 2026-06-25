# AI Features Documentation

miPC includes optional AI-powered features that analyse your hardware data and provide personalised recommendations. This document explains what data is sent, which models are supported, privacy implications, and usage limits.

> **TL;DR** — AI features are **off by default**. No data leaves your PC unless you explicitly grant telemetry consent and configure an AI provider. Your API key is stored in the Windows Credential Manager and never exposed to the frontend.

---

## What AI Features Exist

### 1. System Analysis

When triggered (manually or on a schedule), miPC collects a snapshot of your hardware performance data and sends it to your configured AI model. The model returns a concise analysis covering:

- **Thermal health** — Are CPU/GPU temperatures within safe range? Is throttling likely?
- **Performance mode** — Is the current mode (Silent / Balanced / Smart / Turbo) optimal for the observed workload?
- **Battery** — Is power consumption normal? Any unexpected drain?
- **Top processes** — Are any resource-heavy processes worth investigating?
- **Recommendation** — Suggested performance mode for the current usage pattern.

The analysis is rendered in-app as formatted markdown and stored locally in `localStorage`.

### 2. Log Analysis

miPC periodically logs hardware snapshots (CPU/GPU temperature, TDP, usage %, battery level, top processes) at a configurable polling interval (15 s – 5 min). When you trigger an AI analysis, these logs are aggregated into statistical summaries (averages, peaks, trends) and included in the prompt sent to the AI model.

---

## What Data Is Sent to the AI Provider

When an AI analysis is performed, the following data is included in the prompt sent to your configured AI provider:

| Data Category           | Specific Data                                                          | Notes                                    |
| ----------------------- | ---------------------------------------------------------------------- | ---------------------------------------- |
| **CPU**                 | Model name, core count, temperature (avg & peak), usage % (avg & peak) |                                          |
| **GPU**                 | Temperature (avg & peak), usage % (avg & peak)                         |                                          |
| **Power**               | TDP / package power (avg & peak, if available)                         |                                          |
| **Battery**             | Charge level (start → end), charging/discharging status                | Only if battery data exists              |
| **Performance mode**    | Current mode name (e.g. "Smart", "Turbo")                              |                                          |
| **Top processes**       | Process name, CPU %, memory (MB) — top 6 from latest snapshot          | **Process names are sent in plain text** |
| **Device info**         | Device model name, RAM used / total                                    |                                          |
| **Log metadata**        | Number of snapshots, time span covered                                 |                                          |
| **Language preference** | Your selected UI language (for response language)                      |                                          |

### What Is NOT Sent

- ❌ Your API key is **never** included in the prompt — it is used only in the `Authorization` header.
- ❌ No file system paths, user account names, or personal identifiers.
- ❌ No network configuration, IP addresses, or browsing history.
- ❌ No other application data or settings.

### Input Sanitisation

All input is sanitised before being sent:

- Control characters (0x00–0x1F) are stripped, except `\n`, `\r`, `\t`.
- Input is truncated to 50,000 characters maximum.
- Suspicious patterns (prompt-injection attempts) are logged at `warn` level.

---

## Supported Models

miPC uses the **OpenAI Chat Completions API** format (`/v1/chat/completions`). Any provider that implements this API is supported.

### Preset Models

| Model           | Description                                                 |
| --------------- | ----------------------------------------------------------- |
| **GPT-4o Mini** | Fast and cost-effective — recommended for hardware analysis |
| **GPT-4o**      | Best quality, higher cost per request                       |
| **GPT-4 Turbo** | Previous-generation high-quality model                      |

### Custom Providers

You can enter any model name and point the **API Base URL** to a custom endpoint. Supported custom providers include:

- **[Ollama](https://ollama.ai/)** — Run models locally (e.g. `llama3`, `mistral`, `phi3`). Set base URL to `http://localhost:11434/v1`.
- **[LM Studio](https://lmstudio.ai/)** — Local model server. Set base URL to `http://localhost:1234/v1`.
- **Azure OpenAI** — Set base URL to your Azure endpoint.
- **Any OpenAI-compatible API** — If it accepts `/v1/chat/completions`, it works.

### URL Validation

- **HTTPS** is required for all remote endpoints.
- **HTTP** is allowed **only** for `localhost` or `127.0.0.1` (for local Ollama / LM Studio instances).
- All other schemes are rejected.

---

## Privacy Implications & Consent

### Consent Is Required

AI features are **disabled by default**. Before any data is sent to an AI provider, you must:

1. **Grant telemetry consent** — Available in **Settings → Privacy**. Consent is stored in the Windows Credential Manager and can be revoked at any time.
2. **Configure an AI provider** — Enter your API key and select a model in **Settings → AI System Advisor**.

If consent is denied or not set, all AI requests are blocked and return a `consent_denied` error.

### Data Flow

```
miPC app (your PC)
  │
  ├─ API key → Windows Credential Manager (stored locally, never sent to frontend)
  │
  ├─ Telemetry consent → Windows Credential Manager (stored locally)
  │
  ├─ Hardware data → sanitised → sent to AI provider via HTTPS
  │
  └─ AI response → validated for injection → displayed in-app
```

### Custom Endpoint Warning

If you configure a base URL that does not point to `openai.com`, miPC displays a warning reminding you to verify that you trust the custom provider. **Your hardware data will be sent to whatever endpoint you configure** — ensure you trust the provider.

### API Key Security

- Your API key is stored in the **Windows Credential Manager** (OS keyring), not in plain-text config files.
- The key is read in the Rust backend and used only in the `Authorization: Bearer` header — it is **never** exposed to the React frontend.
- Error messages returned to the frontend are generic and never include API response bodies.

### Output Validation

AI responses are checked for prompt-injection patterns before being displayed. If suspicious patterns are detected (e.g. "ignore previous instructions", "system:", "new instructions:"), the response is rejected and an error is shown.

---

## Rate Limiting & Usage Tracking

### Daily Analysis Limit

miPC enforces a **backend-authoritative daily limit** on AI analyses to prevent unexpected API costs:

- Configurable in **AI Analysis → Settings → Daily analyses**: 1, 2, 4, 6, 12, or 24 per day.
- A value of **0 means unlimited** (no limit enforced).
- The limit is checked **server-side** (in the Rust backend) and cannot be bypassed by a modified frontend.
- The counter resets at the start of each new day (midnight UTC).
- `test_connection` requests also count against the daily limit.

### Usage Statistics

miPC tracks the following usage metrics locally:

| Metric              | Description                                                                  |
| ------------------- | ---------------------------------------------------------------------------- |
| **Total requests**  | Cumulative count of all AI requests                                          |
| **Today's count**   | Requests made today (resets daily)                                           |
| **Input tokens**    | Estimated total input tokens (≈ chars ÷ 4)                                   |
| **Output tokens**   | Estimated total output tokens (≈ chars ÷ 4)                                  |
| **Estimated cost**  | Rough cost estimate based on example rates ($0.10/1M input, $0.30/1M output) |
| **Per-model usage** | Request count broken down by model name                                      |

> **Note:** Token counts and cost estimates are **approximate**. Actual costs depend on your provider's pricing. Check your provider's dashboard for precise billing.

### Usage Data Storage

- Usage statistics are stored locally in `%LOCALAPPDATA%\MiControl\ai_usage.json`.
- The file has restricted ACL permissions — only your Windows user account can read it.
- Usage data is **never** sent anywhere — it exists solely on your machine.
- You can reset usage statistics at any time via the **AI Usage** panel in the AI Analysis tab.

---

## Disabling AI Features

To fully disable AI features:

1. **Toggle off** "AI Analysis" in the AI Analysis tab settings.
2. **Revoke telemetry consent** in Settings → Privacy.
3. **Delete your API key** from the Settings → AI System Advisor section.

You can also use **Settings → Privacy → Delete All My Data** to remove all stored data including logs, credentials, and settings.

---

## Limitations

- AI-generated content **may be inaccurate**. Always verify critical information before acting on it.
- The analysis is based on a snapshot of your system at a point in time — it is not continuous monitoring.
- Token and cost estimates are approximate.
- The system prompt instructs the model to treat all hardware data as untrusted input, but no guarantee can be made about model behaviour.
- If using a local model (Ollama, LM Studio), analysis quality depends on the model you choose.

---

## Further Reading

- [Privacy Policy](privacy-policy-versioning.md)
- [Crash Reporting](crash-reporting.md)
- [Frontend Architecture](frontend-architecture.md)
