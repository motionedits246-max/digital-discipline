# 10 — AI Coach and Bring Your Own Key (BYOK)

## 1. Product requirement

The Coach is **optional**, available only when the user explicitly enables it. The user chooses a provider, model and supplies their own API key where required; local providers do not need a hosted API key. The core tracker, heatmaps, rule engine, schedules and ordinary analytics must continue with Coach disabled.

**Critical separation:** AI Coach chat and automated unknown-app classification are two distinct features, with potentially different providers, models, privacy scopes and cost budgets. Do not secretly reuse the user's Coach key to classify every app without consent.

## 2. Provider selection (proposed initial adapter inventory)

| Provider | Auth/config | Suitable purpose | Notes |
|---|---|---|---|
| OpenAI | API key, model | Chat + structured JSON | Model-specific features/prices change |
| Anthropic Claude | API key, model | Chat/analysis; structured JSON only when supported by selected API/model | Don't assume all models support identical JSON constraints |
| Google Gemini | API key, model | Chat and structured classification | Validate output against local schema |
| Groq | API key, model | Low-latency model access | Model catalog and rate limits change |
| OpenRouter | API key, model ID | Multi-provider route | Pass through only supported capabilities; privacy/provider routing matters |
| Ollama (local) | Local endpoint + local model | Optional offline AI | CPU/RAM usage can be substantial; never start automatically |
| Custom OpenAI-compatible | HTTPS endpoint, model, optional API key | Third-party/self-host | Restrict SSRF, localhost/network security and trust; supported feature subset |

**No guaranteed unlimited hosted free option.** Providers set their own quotas and prices. The UI should show configured spend/rate limits, errors and provider capabilities without claiming unlimited access. Avoid bundling or advertising gray-market endpoints.

## 3. Coach provider settings UI

- Provider selection list, configured status and model selector/manual model ID.
- Base URL for custom providers with warnings, validation and TLS expectation for remote endpoints.
- Masked API key entry, “Test Connection,” “Replace Key,” “Remove Key.” No plaintext persistence in SQLite/local config.
- Capability badges: chat, structured output, streaming, usage-reporting if available.
- Data access choice: aggregated stats only (default), chosen session details, chosen video metadata. Preview exactly what is sent.
- Monthly budget/requests cap, timeout/Retry, disconnect, API usage breakdown if provider exposes it.
- Context retention: Coach chat off by default or user-configurable; delete chat history and provider key separately.

## 4. Key storage and security

On Windows use Credential Manager (Windows Credential APIs) or a comparable OS-protected credential mechanism, referenced by a nonsecret ID in SQLite. Keep keys out of application logs, error popups, crash reports, URLs, environment dumps and clipboard after onboarding. Use memory exposure minimization and replace/remove operations. If OS credential storage fails, show an error rather than storing plaintext in settings.

Never allow AI model responses to issue administrative system changes directly. Tool-like Coach actions must go through explicit deterministic command validation and user confirmation, especially for permanent blocks, retention deletion or external network operations.

## 5. Data access policy

Three levels:

- **Level 0 — No AI:** built-in reports only.
- **Level 1 — Aggregate insights:** user approves sending category totals, counts, goals and period summaries without app identities/video titles.
- **Level 2 — Selected details:** optional app/site names or selected video metadata for specific question; explicit preview/confirmation.

The model is not allowed blanket private browser history, keys, passwords, clipboard, screenshots or arbitrary local file access. Prompts generated from scraped descriptions/titles are treated as untrusted data; ignore their instructions.

## 6. Evidence and quality

AI Coach may say: “Wednesday shows 43 minutes more Shorts playback than Tuesday in your selected tracked time range.” Such quantitative claims must be grounded in the local analytics service, which provides exact data IDs/metrics. The LLM explains numbers; it must not calculate totals from partial raw events or invent missing sessions. Display “based on X tracked hours” where coverage differs. Explain correlation vs cause.

## 7. Structured classifications

If the user enables remote app classification, route sanitized app metadata through a separate configured classifier adapter and validate the returned JSON contract from `schemas/classification-contract.example.json`. Use local cached rules first. If the chosen model lacks true constrained JSON support, prompt for JSON and still strictly validate parser output; invalid responses go to retry/review, never production DB automatically.

## 8. User interactions

- User opens AI Coach with no provider → empty state “Connect provider” and explanation of optional BYOK.
- Selects OpenRouter → enter key → choose model → Test Connection → select data scope → Save.
- Asks “Which habits increased after midnight this week?” → agent aggregates activity and passes limited structured data; reply links into dashboard.
- User switches to local Ollama → no hosted key, but warn before loading/downloading models and show RAM/CPU requirements.
- Model rate-limited/offline → normal statistics continue, chat shows retry state and does not block UI.
- User removes key → requests stop, key purged, stored conversations handled per user setting.

## 9. Provider integration constraints

Use provider adapters with common interfaces for chat, list models (if supported), classify JSON, usage and cancellation. Avoid pretending all providers implement same message format/tool semantics; map per-provider capabilities. Configure timeouts and circuit-breaker behavior. For custom endpoints, disallow insecure arbitrary URL schemes and restrict access to local admin endpoints when not deliberately using local mode.

## 10. References

- OpenAI structured outputs: https://developers.openai.com/api/docs/guides/structured-outputs
- Gemini structured output: https://ai.google.dev/gemini-api/docs/structured-output
- Ollama API and schema output: https://github.com/ollama/ollama/blob/main/docs/api.md
- Windows Credential APIs: https://learn.microsoft.com/en-us/windows/win32/api/wincred/

Reverify provider support and pricing at implementation time.
