# 13 — Implementation roadmap and definition of done

**Status:** Implementation is **not authorized** by this document. It is a future execution plan only. All 32 selected features are retained; MVP must remain narrow.

## Phase 0 — Feasibility/prototype gate

Research and tests:

- Validate Rust Windows foreground + idle + lock/sleep events with acceptable precision/overhead.
- Validate MV3 extension in current Brave; native messaging installation, reconnect, profile detection and basic route observation.
- Validate YouTube Shorts qualified-play signal stability; Instagram Reels feasibility and uncertainty reporting.
- Validate Tauri window modes: tiny click-through/interactive pill, quick panel, large in-place overlay, optional pop-out, fullscreen/multimonitor behavior.
- Benchmark headless agent, WebView2 and extension independently on a resource-constrained Windows PC.
- Prototype safe DNR block, expiry and recovery; confirm `youtube.com/shorts` vs `youtube.com/watch` policies.

**Gate:** Working proof, measured constraints, resolved architectural deal-breakers and updated decisions. Do not build full UX before these tests.

## Phase 1 — Core MVP (ship-worthy vertical slice)

Feature bundle: A1, A2, A4, A8, B1, B2, B8, C1, C2, C3, C8. Related minimal classifications/known categories are included as dependencies, but sophisticated A3 AI fallback ships later.

Substeps:

1. Agent lifecycle, event schema and SQLite migrations.
2. OS foreground/idle and app identity normalization.
3. Brave extension pairing, active-tab/source coverage and native bridge.
4. YouTube Shorts adapter plus minimal Instagram adapter only if validated; if not, explicitly label unsupported in MVP instead of faking support.
5. Reconciler and aggregates: active/idle/foreground/playback, view counting and daily timeline.
6. Unified simple rule engine: block domain/path, threshold budget, schedule, expiry, audit.
7. Mini pill + quick panel + overlay Overview/Activity Timeline/Blocking/Privacy pages.
8. Consent, exclusions, pause, delete and export; graceful offline recovery.
9. Installer/onboarding/tests and performance hardening.

**Gate:** The user can install once, pair Brave, use any IDE, view basic accurate stats, set a limit and close the GUI while enforcement and tracking continue. Security and recovery checklists pass.

## Phase 2 — Deep observability and personalized controls

A3, A5, A6, A7, B3–B7, C4–C7, D1, D2, D5, D7, D8.

- Searchable video history with retention controls, content categorization with user overrides and asynchronous metadata/AI fallback.
- Attention-switch data, browser comparisons and richer timeline/heatmaps.
- Automation canvas and compound shared budgets, scroll count rules, content-level restrictions, strict focus presets and safe override.
- Goal planner, command palette, context-aware pill states, in-page distraction hiding and notification throttling.
- Doomscroll explanation, weekly insights and hypothetical time recovery.

**Gate:** User can trace every statistic and rule execution; classification errors correctable; nonblocking AI queue proven; strict mode never strands user.

## Phase 3 — Advanced opt-in intelligence

D3, D4, D6.

- AI Coach with BYOK/provider selection and data-permission preview.
- Confidence-aware pattern forecasts that require sufficient historical coverage and include uncertainty.
- Optional Android/other-device recording/sync: separate mobile architecture, authentication, encrypted transport, event conflict resolution and consent.
- Optional advanced Windows OS-level blocking after independent threat/compatibility evaluation.

**Gate:** Basic app remains offline capable and fully useful with AI/cross-device disabled.

## Phase 4 — Release hardening

This phase is final validation, not the first security/testing pass. Include crash/restore, 24+ hour resource tests, site adapter drift checks, extension store requirements, signed installers, migration rollbacks, accessibility, audit logging, privacy notices and safe uninstaller.

**Gate:** Public release checklist complete with documented limitations. No false statements about impossible “unbypassable” browser blocking or exact user attention.

## Delivery packages per phase

Every phase creates: updated requirements, architecture change records (ADRs), UX approved mockups, user-facing explanation of permissions, unit/integration tests, performance measurements, threat-model delta, changelog, uninstall/recovery test results and release notes. If AI agents implement, tasks must be sliced into independently testable milestones instead of one huge prompt.

## Suggested first build tasks after explicit approval

1. **Spike 01:** Windows foreground events and session boundary test harness.
2. **Spike 02:** Brave MV3 native messaging host handshake + versioned event envelope.
3. **Spike 03:** YouTube Shorts qualifying play counter and SPA transition tests.
4. **Spike 04:** Tauri three-state overlay window lifecycle.
5. **Spike 05:** Timed DNR rule with expiry and a crash-safe recovery test.

Each spike receives a written decision: viable, viable with caveats or blocked. Do not silently broaden permissions to make the demo work.

## Definition of Done — Core MVP

- [ ] Any foreground app/IDE produces traceable sessions and an unknown-safe classification state.
- [ ] GUI closed does not stop background tracking or rule engine.
- [ ] Browser active time never double-counts parallel tabs and switching browsers.
- [ ] Qualifying plays, unique videos and repeats are independently measured.
- [ ] Supported/unsupported browser/profile coverage is shown accurately.
- [ ] Basic domain/path and scheduled blocks enforce and expire safely.
- [ ] GitHub-style heatmap and interactive timeline reflect only observed data.
- [ ] Pill adapts to current activity and expands to in-place overlay; pop-out optional.
- [ ] Users control permissions, pause, exclusions, export and deletion.
- [ ] Async jobs cannot stall timekeeping or blocking.
- [ ] No compulsory accounts, remote AI or GPU.
- [ ] Critical performance, resilience, security and UX tests pass.
