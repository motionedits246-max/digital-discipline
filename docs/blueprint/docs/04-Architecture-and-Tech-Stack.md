# 04 — Technical architecture and language decisions

## 1. Recommended stack

| Layer | Language / technology | Why | Important caveat |
|---|---|---|---|
| Windows background collector | **Rust** + Win32 APIs (via maintained Windows bindings) | Native Windows event access, controlled memory, efficient long-running agent | Win32 hook threading/session behavior must be correct |
| Domain/policy engine | **Rust** | One source of truth for sessions and rule evaluation, deterministic testing | Rules must not inherit UI lifetimes |
| Native messaging bridge | **Rust** dedicated small executable or agent bridge | Strict JSON IPC validation and Windows registration | Extension-origin allowlist and message size/rate checks |
| Desktop GUI | **Tauri v2** | Native window/tray APIs, Rust integration, uses OS WebView | WebView2 runtime behavior and renderer RAM measured separately |
| UI components and data views | **React + TypeScript** | Maintainable component model, charts, interactions, state | Keep large data aggregation in backend |
| Styling | CSS variables / Tailwind optional | Monochrome tokens, theme consistency | Avoid overloading with bulky component libraries |
| Browser extension | **TypeScript + Manifest V3** | Chromium browser events, content scripts, DNR | Service worker lifecycle, browser/profile permissions |
| Storage | **SQLite** | Offline, reliable local DB, indexes, migrations | Single-writer ownership, retention and migration plan |
| Validation | JSON Schema / typed data contracts | Validate local IPC, AI JSON and automation rules | Schemas evolve with versioning |
| AI classification | Optional provider adapter; local rules first | Scalable/replaceable, no mandatory API | Remote availability/pricing/rates not guaranteed |
| AI Coach | BYOK vendor adapters | User-selected model and key | Vendor rate limits, API differences, privacy |
| Cross-device (future) | Protocol TBD | Avoid premature networking | Android permissions and conflict semantics differ |

**Do not select Electron solely because it's familiar.** Compare Tauri/WebView2 startup and memory on target Windows laptops during Phase 0. If serious overlay compatibility problems occur, document them and reconsider the UI shell before full implementation.

## 2. Process architecture

```text
Windows interactive user session
  ├─ native agent (Rust, headless, autostart optional)
  │    ├─ OS activity watcher: foreground/app/idle/sleep/lock
  │    ├─ browser bridge receiver + consent gate
  │    ├─ event ingest + reconciliation + category cache
  │    ├─ sqlite single writer + read aggregate endpoints
  │    ├─ policy/schedule engine + enforcement dispatcher
  │    ├─ bounded async queues: metadata, classification, reports
  │    └─ optional secure credential/provider client
  ├─ extension MV3 (Brave first)
  │    ├─ tab/navigation/visibility observation
  │    ├─ platform-specific Shorts/Reels adapters
  │    ├─ DNR / in-page blocking adapter
  │    └─ reconnecting native messaging bridge
  ├─ Tauri GUI process (when opened)
  │    ├─ tiny pill and quick panel
  │    ├─ in-place full overlay
  │    ├─ optional pop-out window
  │    └─ charts/search/commands/preferences
  └─ optional Windows OS blocking helper (off by default)
       └─ elevated changes only with explicit permission and rollback
```

## 3. Responsibility boundaries

**Agent owns:** timer/session state, category classifications, queue, rule truth, DB writes and updates. GUI does not become another time tracker.

**Extension owns:** per-browser web observations, page-specific video signals, URL/DOM site enforcement. It is not allowed to write directly to SQLite or call remote AI without the agent and explicit policy.

**GUI owns:** rendering, settings commands, accessible controls, local read subscriptions. It should not keep the entire event history in memory or spawn duplicate collectors.

**Privileged helper owns (future):** only system-level restrictions (if user explicitly enables them); narrow allowed operation set; no browsing-history access; automatic cleanup.

## 4. IPC and event contracts

- Versioned event envelopes: `schema_version`, `event_id`, `source`, `device_id`, `browser_profile_id` (if any), `occurred_at_utc`, `monotonic_context`/sequence, `received_at_utc`, `type`, `payload`, `privacy_class`.
- Validate maximum payload size, source origin, enums, timestamp sanity, rate limits and content types.
- Dedupe by event ID/source sequence; acknowledge receipt; ensure idempotent replay after crash.
- If native bridge is unavailable, extension queues bounded non-sensitive events for best-effort replay (based on consent/limits). Never promise flawless recovery of events not observed.
- The agent should publish state snapshots plus incremental updates to GUI; GUI can reconnect and reload.

Chrome native messaging uses host manifest allowlists and standard-input/output exchanges. An extension content script must relay via the extension context; sender origins must be checked before forwarding to native code. Official: https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging

## 5. Concurrency model

**Latency-critical lanes:** foreground/idle ingest, session changes, clock/timers, policy evaluations, enforced block state. Use small bounded queues and explicit backpressure. When over capacity, preserve session boundary events and emit diagnostics rather than making up durations.

**Best-effort lanes:** app description research, AI classification, thumbnail retrieval, weekly report synthesis, Coach Q&A. Separate concurrency/rate budgets; never block the critical lanes. Keep CPU, network and model loading off the UI thread.

**Database:** a single logical writer, short transactions, batch small periodic events, periodic checkpoints. Use a read-only snapshot/connection for aggregated charts to avoid interactive stalls. Preserve migrations and rollback/backup strategy.

## 6. Installation and lifecycle

- Signed installer as release goal; test Windows 10 and Windows 11 supported builds.
- Per-user install and startup default; no UAC prompt unless optional OS helper enabled.
- Native messaging host installation/registration for supported browser(s). **Do not assume Brave uses identical registration discovery paths to Chrome**; verify on target Brave versions and both per-user/machine installs.
- Browser extension permission/pairing onboarding with version negotiation.
- Shutdown choices must distinguish Hide UI / Stop Tracking / Exit Engine / Uninstall.
- On Windows account logout: flush data, stop timers, remove overlays. On restart: record sleep/off interval as unknown/offline, not usage.
- On uninstall: unregister native messaging manifests, remove owned OS rules, preserve/delete local history according to explicit user choice, and avoid orphan processes.

## 7. Tech alternatives considered

- **Python agent:** good for rapid prototypes and AI integrations but typically heavier packaging/startup for always-on collector. Can be used for one-time research or tests, not preferred production background process.
- **C#/.NET:** legitimate Windows-native alternative, excellent OS integration and deployment tooling; if developer team strongly prefers .NET and measured overhead is good, reconsider with ADR. Currently Rust wins for a compact unified core and Tauri integration.
- **Electron:** mature UI ecosystem, but not the default for the requested lightweight agent/overlay; must be measured rather than categorically rejected.
- **Built-in local LLM:** optional for users with sufficient hardware; should not be autostarted or required on low-RAM/no-GPU machines.

## 8. Deployment/versioning

- Version agent, extension, DB schema, IPC protocol and site adapters separately.
- Support capability negotiation for mixed-version installs; show explicit incompatible versions.
- Privacy/security fixes take priority; avoid silent behavioral changes to rules after updates.
- Automated rollback for bad database migrations only where safe; backups before destructive changes.

## 9. Acceptance criteria

A stable Windows session can collect foreground/idle events, receive browser events, persist normalized sessions, enforce a test domain rule, close/reopen UI without stopping timers, recover after agent crash without phantom active time, and report sources/unknown coverage. Component tests must cover shutdown, sleep, VPN/proxy constraints and native-host disconnection.
