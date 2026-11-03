# 14 — Open decisions, assumptions, risks and ADR backlog

These are explicitly **not settled** by the user; developers must not convert them into permanent implicit behavior. The docs offer recommended defaults to facilitate progress after authorization.

## 1. Locked user preferences

- Windows-first. Brave-first technical validation.
- All 32 feature IDs retained in staged roadmap.
- Black/white/charcoal YouTube-style UI references.
- Tiny animated activity-aware floating pill; click to quick panel; expand full UI in-place overlay; normal window optional.
- Rich analytics, including a GitHub-style heatmap.
- No IDE-specific assumptions: Cursor, Antigravity, VS Code, others and unknowns treated generically.
- Unknown apps may be classified asynchronously by optional allowed web research + model JSON; primary processes remain nonblocking.
- Optional AI Coach with provider selector/BYOK.
- No application implementation code yet.

## 2. Decisions pending user feedback

| ID | Decision | Current proposal | Trade-off / note |
|---|---|---|---|
| Q01 | Final product name/logo | Digital Discipline working name | Branding later |
| Q02 | Default pill position | Top-center/near taskbar with user adjustment | Avoid covering editor controls |
| Q03 | Pill auto-hide | On fullscreen only by default | User may want always visible |
| Q04 | Full overlay dimensions | 80% work area, resizable | Small laptop visual density |
| Q05 | Window title storage | Off by default | More context vs privacy |
| Q06 | URL granularity | Domain default, detailed routes opt-in | Accurate Shorts require site-specific access |
| Q07 | Qualifying video threshold | 2 cumulative playback seconds | Should be tested and user configurable |
| Q08 | Idle threshold | 3–5 minutes default candidate | Video watching vs input inactivity |
| Q09 | Default categories | Development, Education, Work, Entertainment, Gaming, Social, Utility, Unknown | User-customizable taxonomy |
| Q10 | Remote app research | Opt-in only | Privacy, quota and cost |
| Q11 | Default AI | Disabled | Local rule fallback always |
| Q12 | Strict focus override | Emergency override + optional friction | Don't create irreversible lock |
| Q13 | Windows Firewall option | Future advanced module only | High compatibility risk |
| Q14 | Heatmap intensity | Monochrome fixed thresholds or quantiles | Comparability vs readability |
| Q15 | AI Coach history | No saving by default | User can opt into local chats |
| Q16 | Data retention | Raw 30d / metadata 14–30d / aggregates 1y | Storage vs analytical history |
| Q17 | Cross-device sync | Future, opt-in, encrypted | Threat model changes |
| Q18 | Activity label | Prefer “Distraction” over “Wasted” | Nonjudgmental language |
| Q19 | Packaging channel | Windows signed installer, extension store | Initial developer builds separate |
| Q20 | Exact behavior when tracking paused | Pause observer; blocking separately toggled | Avoid hidden enforcement surprise |

## 3. Architecture decision records (ADRs to draft after Phase 0)

- ADR-001: Rust/Tauri vs C#/.NET/Electron after actual memory/overlay benchmarks.
- ADR-002: Separate headless process vs Tauri single-process architecture.
- ADR-003: SQLite write ownership, event journal and retention.
- ADR-004: Per-user Native Messaging host installation for Brave/Chrome/Edge.
- ADR-005: Foreground vs background playback accounting and idle threshold.
- ADR-006: Content-script adapter strategy for YouTube/Instagram UI updates.
- ADR-007: Rules engine conflict/exception semantics.
- ADR-008: OS-level firewall helper feasibility vs no helper.
- ADR-009: AI model JSON adapter abstraction and privacy modes.
- ADR-010: Credential store integration and secret lifecycle.
- ADR-011: Future cross-device dedup, sync protocol and encryption.
- ADR-012: Auto-update, code signing, distribution and telemetry policy.

## 4. Biggest product risks

**Accuracy risk:** Dashboards may appear precise despite incomplete browser/video signals. Mitigation: provenance, measured coverage, explicit unknowns and manual calibration.

**Site fragility:** Instagram and YouTube can change player DOM/events. Mitigation: versioned adapters, health checks, regression tests, visible degraded state.

**Over-scoping:** 32 selected features encourage building too much before reliable core. Mitigation: phase gates and vertical slice MVP.

**Performance:** Full GUI/LLM may consume RAM even when only pill shown. Mitigation: separate lifecycle, lazy load, resource budgets and optional model.

**Blocking hazards:** Stale browser/firewall rules can strand user; DNS-based OS blocks can disrupt shared IP services. Mitigation: browser-first, expiry guarantees, owned rules, safe recovery and isolated privileged helper.

**Privacy:** App/window/video metadata extremely revealing. Mitigation: minimal defaults, consent, credential storage, redaction, local-first deletion/export.

**Unbounded third-party costs:** Unrestricted “unlimited” AI is not a dependable dependency. Mitigation: local classifier, per-provider budget, cache, bounded queues and offline fallbacks.

## 5. Risk acceptance process

Every major open decision requires: (a) user-facing effect; (b) proposed implementation; (c) alternatives; (d) privacy and resource impact; (e) tests; (f) reversible migration. Capture final choices in an ADR. Do not claim a decision is user-approved because it appears in a concept image.
