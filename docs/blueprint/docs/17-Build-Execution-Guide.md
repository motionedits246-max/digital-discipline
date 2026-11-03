# 17 — How to build it (future execution playbook; NO CODE YET)

## 1. Purpose

This is a developer/agent-oriented implementation plan explaining sequence, module boundaries, language selection, dependencies, verification and packaging. **It does not authorize application implementation** and intentionally contains no source-code scaffolding.

## 2. Preferred tools and toolchain

| Need | Recommended tool | Reason |
|---|---|---|
| Native app/agent | Stable Rust toolchain and Cargo | Efficient native collector, shared policies |
| Win32 integration | Maintained `windows` crate or equivalent | Typed Win32 FFI, reduced manual bindings |
| UI shell | Tauri v2 and Windows WebView2 | Native windows, tray, Rust↔webview bridge |
| UI language | TypeScript + React | Type-safe reusable panels and state management |
| Extension | TypeScript, Chromium Manifest V3 | Brave/Chrome/Edge APIs and content adapters |
| Design language | CSS variables, responsive layout, SVG icons | Monochrome and scalable visual system |
| Persistence | SQLite library with safe migrations | Local reliable event store |
| Unit/integration | Rust test framework, Vitest/Jest, browser automation | Deterministic logic and adapters |
| Native E2E | Windows virtual machine + UI automation | Tests native overlay, UAC, focus and tray |
| Quality automation | CI on Windows runners | Repeatable builds/tests and packaging |
| Source control | Git with ADRs and changelogs | Auditable incremental development |

Specific library names/versions require compatibility and maintenance checks at implementation time. Prefer minimal dependencies over building a large framework stack simply because it exists.

## 3. Proposed future repository layout (illustrative)

```text
repo/  (does not exist yet)
  apps/
    desktop/                 Tauri shell and React user interface
    browser-extension/       TypeScript MV3 extension (content + service worker)
  crates/
    activity-agent/          Rust background executable
    windows-observer/        Native OS event source
    event-contracts/         Versioned event types + validation
    session-reconciler/      Interval union, clock/coverage handling
    category-engine/         Overrides, caches, async classification jobs
    rule-engine/             Deterministic policy core
    storage/                 SQLite persistence + migrations
    provider-adapters/       Optional metadata search and BYOK AI
    native-host/             Minimal validated Native Messaging process
    os-block-helper/         Future optional elevated component
  shared/
    schemas/                 JSON Schema for contracts
    design-tokens/           UI colors/sizes/type definitions
  tests/
    fixtures/                Synthetic activity traces and site adapter fixtures
    end-to-end/              Native/browser/Windows lifecycle tests
  docs/                      This approved specification, ADRs, runbooks
  tooling/                   Build/release scripts after user approval
```

Important: Browser extension should not depend on compiling the full React dashboard. The headless agent should not depend on starting WebView2. Unit-testable logic should be isolated from Windows APIs where possible.

## 4. Stepwise build with demonstrations and gates

### Work package 1 — Native session observer

Implement foreground change events, per-session input idle, lock/sleep/resume and app identity. Feed **synthetic traces** into a pure session reconciler before using live events. Deliver an event inspector and test report, not dashboard polish. Gate: no phantom time after sleep, no double count across apps.

### Work package 2 — Event database and coverage reporting

Add versioned events, source sequence, SQLite sessions, aggregates, migration and recovery. Benchmarks on 24-hour synthetic activity. Gate: source-of-truth math and export reproducibility.

### Work package 3 — Browser extension handshake

Build minimal MV3 extension with active tab/domain observation, source permission model and native host pairing. Test Brave current versions and Chrome separately. Implement reconnection and worker sleep resilience. Gate: no unseen profile falsely shown as tracked.

### Work package 4 — Short-video adapters

Create isolated YouTube Shorts adapter with video ID/qualified play logic and SPA route handling; add Instagram adapter only after feasibility proof. Gate: 20 rapid impressions and 3 qualifying plays produce exactly 3 qualifying plays. Persist detector health.

### Work package 5 — Blocking/rules vertical slice

Implement one deterministic timer rule, basic domain/path DNR block, expiry after agent crash/browser restart, reason/override interstitial. Gate: no expired stale lock, no misleading success state, no forced tab closure.

### Work package 6 — Overlay GUI

Tauri shell with three presentation states: pill, quick panel, full overlay; optional pop-out. Shared read-only state subscription. Start with Overview KPI + timeline + heatmap + Blocking Center + Settings. Gate: close GUI without stopping agent, focus/stacking/multi-DPI acceptance.

### Work package 7 — Data review, categories and privacy

Add category overrides, Unknown reporting, exclusions/retention/delete/export, audit logs and charts. Gate: deletion truly removes downstream details and any derived indexes.

### Work package 8 — Queue-based classification

Implement the bounded async job model separately. Add local cache, WinGet lookup and explicit optional BYOK classifier; schema validator and backfill. Gate: 30-second API delay with zero critical-loop delay and no uncontrolled repeated calls.

### Work package 9 — Deep v1 features

Video gallery, attention switches, comparison charts, shared budgets, visual automation canvas, goals, doomscroll detector, partial UI blocking and weekly reports. Ship each with behavior tests and degradation states.

### Work package 10 — AI Coach and later device sync

Implement BYOK secret storage and provider adapters, an explicit aggregate-data permission gate and grounded, source-linked answers. Cross-device later as separate security project.

## 5. Development order within a typical feature

1. Define expected user behavior with examples and edge cases.
2. Update event/schema/policy definitions and privacy classifications.
3. Create unit tests for time/rule semantics.
4. Implement backend/protocol in isolation.
5. Add browser/native integration as required.
6. Add concise frontend visualization and empty/unsupported/error states.
7. Run performance and failure tests.
8. Add security review, docs, migration and release notes.
9. Demonstrate to user; accept or revise.

Do not implement all 32 features in one code-generation request. Limit each coding agent to a bounded, testable feature with acceptance gates.

## 6. Working with AI coding agents

Recommended handoff prompt structure: link `README.md` + relevant docs + feature ID + allowed scope + test/exit conditions + explicit “do not expand permissions or add cloud accounts without user approval.” After the user authorizes coding, require an implementation plan and files-to-change list for each milestone. Keep human review checkpoints for installation, privacy boundaries, blocking and BYOK secret handling.

## 7. Windows tooling tests

Test in a standard Windows user account before administrator mode. Include Brave/Chrome browser profiles and fresh installs. Maintain a Windows VM for crash/reboot/installer tests; a low-RAM development laptop need not run all CI locally. CI can run tests without keeping a background agent active on the developer's daily machine.

## 8. Definition of integration complete

Native agent is the **only timer/data authority**. Browser extension sends bounded validated events and owns browser enforcement. GUI subscribes to summary state and never changes timekeeping merely because it opened. External metadata/AI jobs use separate bounded queues. Every operation has a documented failure/consent state and can recover gracefully.

## 9. Explicit stop point

Stop after documentation and user design approval. Do **not** install packages, create a repository, start a daemon, ask for API keys, modify Windows startup, install extensions or edit firewall policies until the user explicitly requests development.
