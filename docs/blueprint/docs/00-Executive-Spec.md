# 00 — Executive product specification

## 1. Product statement

**Working name: Digital Discipline.** A lightweight Windows-first, privacy-first, offline-capable system that measures the user's digital activity, connects that activity to self-defined goals, and enforces self-selected limits with a minimal floating overlay and a comprehensive analytics workspace.

**Promise:** Understand how digital attention was spent, intervene when the user wishes, and objectively report the effects of interventions. Not a parental-control product, surveillance tool, moral judge or mandatory AI assistant.

## 2. User's confirmed requirements

- All 32 features A1–A8, B1–B8, C1–C8 and D1–D8 are desired across phased releases.
- Windows first; Brave first. Consider Chrome/Edge and later Android/cross-device.
- Two operating states: *GUI visible* and *headless/lightweight background*. A floating controller is a third visual presentation of the same running system.
- Live pill changes meaning with activity: coding in **any** recognized IDE, Shorts/Reels counter, browsing, focus countdown, idle, blocked.
- The pill expands into a quick panel; **Full Panel** opens an in-place overlay on top of the current desktop; a standalone full window is optional.
- Monochrome charcoal/black/white inspired by YouTube controls; high-information displays including GitHub-like heatmap, timeline, drilldowns, video history and behavior maps.
- Universal app categorization using local rules and an **asynchronous** unknown-app queue; optional public metadata lookup and AI structured JSON. The system must stay responsive while classification is pending.
- AI Coach has BYOK and provider selection. Classification and Coach providers may be configured separately.
- No coding was requested at the time of this document. Treat the package as a specification only.

## 3. Users and context

**Primary:** An individual knowledge worker, student, developer or creator who alternates between focused work and distraction-heavy online content. They want accurate context (learning vs entertainment), not a simplistic website blacklist.

**Secondary:** A user who wants hard limits or scheduled controls without extensive manual configuration. The system must not assume everyone considers the same app “productive.”

## 4. Outcomes, not feature counts

- User sees what actually happened each day, at app/domain/qualified-video/session level subject to consent.
- User can explain why an activity was counted as productive, neutral, distracting or unknown.
- User can configure a 15-video or 20-minute rule and observe it being enforced and reset reliably.
- User can start Windows, work, close the GUI and still get accurate tracking and safe rule behavior.
- User can review how automation changed time allocation (correlation, not ungrounded causal claims).
- User can turn off classification AI and AI Coach and retain core functionality.

## 5. Business/product differentiation

Competitors already implement individual pieces: ActivityWatch (open-source local activity tracking), RescueTime (activity categorization/goal tools), Freedom/LeechBlock (blocking), Rize (focus analytics), and dedicated short-video counters. Differentiation is the **joined-up workflow**: granular observation → contextual goals → transparent interventions → measurable outcomes, in a lightweight overlay-first product.

Do not claim the concept or any isolated feature is entirely novel.

## 6. Product principles

1. **Local-first:** Core functions do not require an account, a cloud service or AI.
2. **Understandable:** Stats show definitions, coverage and rule provenance.
3. **Nonblocking:** API latency never pauses tracking or site blocking.
4. **Conservative:** Uncertain activities remain unknown; incomplete sessions are flagged.
5. **User-directed:** Goals and categories are editable, with overrides taking precedence.
6. **Transparent enforcement:** Restrictions display reason/expiry/recovery; no stealth controls.
7. **Low overhead:** Resource budgets tracked; nonessential views and remote calls remain dormant.
8. **Accessible:** Keyboard, scaling, reduced motion, contrast, screen reader semantics.
9. **Maintainable:** Site adapters, schemas and provider integrations are modular.

## 7. Non-goals for MVP

- Android/mobile device synchronization; device-wide capture from another computer.
- Video transcript downloads or automatic full-content ingestion.
- Monitoring another person, hidden tracking or remote administrator capabilities.
- Tamper-proof/unbypassable blocking on user-owned Windows machines.
- Mandatory model download, GPU compute, cloud account, training a custom LLM.
- Perfect inference of intention, attention or actual learning from activity metadata.
- Exact video category inference across every content platform and language.

## 8. Explicit product meanings

**Active time:** foreground exposure adjusted for declared input-idle/confirmed playback rules.  
**Video playback:** intervals derived from player events when available; cannot prove watching.  
**Distraction time:** user-defined classification/policy over recorded intervals.  
**Focus time:** time matched against an active goal/rule definition, not simply time in VS Code.  
**Wasted time:** display label customizable; default prefer *Distraction time*.  
**Unknown:** honest attribution when app/category or signals are incomplete.

## 9. Proposed measurable KPIs

Product quality: session attribution accuracy, no double counts, block rule correctness, data-loss rate, time-to-intervention, battery/CPU/RAM impact, classification coverage and permission clarity.

User outcomes: goal attainment, distraction duration, blocked attempts, override counts, session completion and voluntary engagement with insights. Avoid “streak anxiety” and manipulative engagement tactics.

## 10. Release definition

The first shipping MVP is a narrow but complete vertical slice: installed Windows agent + Brave extension + local data + accurate activity sessioning + qualified Shorts counts + basic categories/neutral-vs-distracting + browser block/schedules/time budgets + live pill/quick panel/full overlay + data privacy controls. Everything else expands from that foundation.

See [Feature Matrix](01-Feature-Matrix.md), [Roadmap](13-Roadmap-and-Definition-of-Done.md) and [Open Decisions](14-Open-Decisions-and-Risks.md).
