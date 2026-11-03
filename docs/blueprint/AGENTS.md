# Instructions for the future implementation agent

> **PAUSE: PLANNING-ONLY PACKAGE. Do not write application code merely because this folder exists.** The user must explicitly say that implementation can begin. Documentation changes and design revisions are okay when requested.

## Product context

Build a Windows-first, local-first activity tracker, behavioral insight system and distraction-control application. It supports any application or IDE without special casing only VS Code. The application has a native background agent, desktop overlay-first UI, browser extension, unified rules, optional nonblocking classification pipeline and BYOK AI Coach. Full feature scope is 32 features; shipping priority is phased.

## How to use these docs

Read in order: `README.md` → `docs/00-Executive-Spec.md` → `docs/01-Feature-Matrix.md` → `docs/04-Architecture-and-Tech-Stack.md` → domain-specific docs → `docs/13-Roadmap-and-Definition-of-Done.md` → checklists. Resolve open decisions in `docs/14-Open-Decisions-and-Risks.md` with the user rather than silently altering scope.

## Non-negotiable invariants

1. Track and enforce rules independently of the GUI and all optional remote calls.
2. Never block core operations on AI, search, metadata lookup or analytics queries.
3. Keep background resource use small and measurable, with event-driven collection where appropriate.
4. Never double-count overlapping foreground windows/tabs in aggregate totals.
5. Distinguish platform playback signals from claimed/deduced watch duration.
6. Store only approved, minimal data; no credential content, passwords, screen recording or keylogging.
7. Permit scoped browser permissions and sensitive-site exclusions.
8. Validate all messages crossing boundaries (web content → extension → native bridge → agent; AI response → classification DB).
9. Never trust an LLM's classification unconditionally; use enum validation, confidence/evidence and user overrides.
10. Do not claim permanent/strict blocking is tamper-proof; preserve emergency recovery and guaranteed expiry.
11. Display degraded/unclassified conditions transparently.
12. Keep all selected A1–D8 features visible in the roadmap; never implement all 32 simultaneously.

## Architecture boundaries

- Rust agent: source of truth for events, sessions, persistence, policy decisions and work scheduling.
- Tauri UI with React/TypeScript: presentational and command interface; detachable pill and overlay.
- MV3 browser extension: browser metadata, video adapters, DNR rules and in-page controls.
- Bridge: allowlisted, authenticated/restricted native messaging and explicit event schemas.
- AI classification: separate bounded queue, cache, retries, timeouts and backfill; opt-in external processing.
- AI Coach: separately configured provider adapter, BYOK, credential storage and data minimization.
- Windows OS-level helper: *optional only* with elevated privileges when user enables system-wide restrictions; owned-rule rollback.

## Engineering process (once explicitly approved)

First validate implementation prerequisites and current library/browser support; create unit tests for time reconciliation and policy semantics before UI integration. Implement smallest vertical slice: track one Windows foreground session, send one browser event, persist, render live pill, apply one browser block, recover after crash. Expand only after that works reliably.

Every feature PR should include: feature ID(s), updated documentation, test plan, privacy impact, performance impact, observability, failure modes and migration requirements. New external dependencies require maintainability/security/license checks. Never log API keys or raw private browsing data.

## Defer until approved

No scaffolding, package installation, repositories, deployment, executable source, automated browser extension, system registry changes, firewall rules or API calls with user's credentials should be created from this planning brief alone.
