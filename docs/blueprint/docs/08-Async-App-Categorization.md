# 08 — Universal categorization: rules, research and async AI

## 1. Requirement

**No hardcoded assumption that coding = VS Code.** Any Windows application—including Cursor, Antigravity, JetBrains, terminal editors, niche tools or a future IDE—must enter the same recognition pipeline. Unknown apps are never discarded. A background worker may research/AI-classify them using an agreed **JSON input → JSON output contract**, without blocking core event collection, restrictions, timers or GUI.

## 2. Pipeline and priority

```text
Observed application
  ↓ (synchronous, local, fast)
User override? → Use override and end
  ↓
Exact identity cache? → Use cached category and end
  ↓
Installed known-app/rule database? → Use source-scored mapping
  ↓
Heuristic metadata (product, publisher, file identity)?
  ↓ (if confidence insufficient)
Persist Unclassified immediately + queue lookup job
  ├── public app metadata API / WinGet (optional, permitted)
  ├── cached and licensed public search results (optional)
  └── structured JSON AI model (optional BYOK or local provider)
         ↓
Validate schema + evidence + confidence + contradiction checks
         ↓
Commit classification provenance + narrow historical backfill
         ↓
Emit dashboard/category refresh (never change raw time)
```

## 3. Classification taxonomy and objective

Two separate axes:

**Tool taxonomy** (what is the application?) — Development, Design, Productivity, Communication, Education, Entertainment, Gaming, Social, News, Utility, System, Browser, Unknown. Hierarchical categories and new user categories are allowed.

**Behavior judgment** (does it serve today's goal?) — Productive, Neutral, Distracting, Unknown. This is user-policy-dependent. AI may suggest an app's likely domain but must not assign “wasted time” by default.

For unknown apps, classify *app identity* with name/product/publisher evidence. Window/document titles are **excluded by default** because private filenames can leak. URL/video categorization is a separate subject type and requires user consent and adapter evidence.

## 4. Prewritten JSON input contract

The `schemas/classification-contract.example.json` file has illustrative input/output examples. Logical input fields:

- `request_id` and `schema_version`.
- `subject.type` = application / domain / video.
- `subject.display_name`, normalized signed publisher and minimal executable product metadata when consented.
- `known_evidence`: list of source URI/text and provenance; do not fabricate evidence.
- `candidate_categories`: allowed enumeration from user taxonomy.
- `privacy_tier`: classify-only sanitized metadata, not full browsing history.
- `instruction`: choose best-supported category; return unknown when evidence insufficient; no invented confidence.

Logical output fields:

- `category` one of allowed IDs or `unknown`.
- `subcategory` optional.
- `confidence` in 0..1, explicitly model-estimated (not a verified statistical probability).
- `short_reason`, max length and free of private data.
- `evidence_ids` linking to real observed metadata/research, no fabricated URLs.
- `status`: classified / ambiguous / unknown.
- `model_id`, `provider`, `classifier_version` maintained by agent metadata.

The receiving agent enforces schema and semantics. Invalid JSON, unrecognized category, high confidence with no evidence, contradictory signals, provider error or oversized response → mark for review/retry, do not apply blindly.

## 5. Sources: accurate limits

| Source | Value | Availability/limitations |
|---|---|---|
| Local executable metadata | Name, product, signature/publisher | Cheap, but names may mislead |
| User overrides and known-app rules | High-priority reliable classification | Curated by user/local data |
| Microsoft WinGet | Package description, publisher, tags, package ID | Metadata only if package indexed; matching must be verified |
| Wikimedia APIs | Public app descriptions where available | Must respect User-Agent, rate limits, content licensing and policies |
| Brave Search API | App homepage and search snippets | BYOK/usage-based service; not guaranteed free or unlimited |
| Official vendor website | Best source of app function | Use documented APIs or website terms-compliant access; no indiscriminate scraping |
| Ollama local models | Offline JSON output | Requires optional model/resource budget; not auto-loaded on limited laptops |
| Hosted AI via BYOK | Flexible JSON classification | Paid/limited, not guaranteed accurate or available |

**Do not assume there is a free, unlimited hosted model or website that allows unrestricted scraping.** No scraper should bypass robots.txt/site terms, access restrictions, CAPTCHA or login. Prefer documented APIs and caching, then fallback to Unknown/manual override.

## 6. Background queue semantics

- Persistent job keyed by normalized app identity + classifier version + locale; dedupe multiple observations.
- Network jobs opt-in; default no uploads.
- Priority classes: tracking/blocking (always highest), user-requested classification, background classification, periodic backfill.
- Bounded worker concurrency, e.g. 1–2 network jobs initially (tune after real measurements); one worker can multiplex I/O async without a heavyweight process per app.
- Per-provider deadline, exponential backoff with jitter, maximum attempts and circuit breaker on rate-limit/repeated errors.
- Usage caps: requests/day, token/cost caps, permitted metadata fields, timeout; disable remote lookup entirely by preference.
- Cancellation: stopping tracking doesn't necessarily erase queued jobs, but user can Cancel/Clear Queue and disable remote sending.
- Job state: `queued`, `running`, `rate_limited`, `awaiting_consent`, `succeeded`, `failed_retryable`, `failed_final`, `cancelled`.
- Catch-up/backfill runs in small batches when PC idle and not under load, and never waits on the GUI thread.

## 7. Cache and identity safety

Never classify an executable solely by basename where an impostor could use a familiar name. Prefer product/publisher/signature/verified package identity where available. Maintain mapping expiry, evidence version and user override. Avoid hashing sensitive path text into network requests; any externally sent identity should be a safe public product field.

## 8. Human review UX

“Unknown apps” page shows name, observed duration, classification status, last source, evidence and quick category assignment. Bulk-correct recurring apps. The user can mark an app `Always Development`, `Always Neutral`, `Ask every time`, or `Never send to AI`. User overrides dominate automatic decisions. Visible confidence is for transparency; avoid false precision.

## 9. Failure examples

- AI returns valid JSON but calls a game IDE → compare with publisher/local context, keep ambiguous if contradicted.
- API fails 429 → nonblocking backoff, preserve uncategorized time, later refresh.
- User turns off internet → all local functions continue; pending job remains safe.
- Same app launched 50 times → one deduped classification job, not 50 API calls.
- New app classified after a week → previous intervals can be reclassified with clear history update and policy.
- Model sends a category not in taxonomy → reject and log only redacted error.
- Process name resembles an IDE but is not signed/known → do not overtrust basename.

## 10. Accuracy evaluation

Create a manually labeled corpus of common apps, obscure IDEs, renamed executables, dual-purpose apps and intentionally misleading binaries. Compare local-only vs search-assisted vs AI outputs on accuracy, unknown rate, latency/cost and privacy impact. Manual override correctness > automated output. Do not promise perfect classification.
