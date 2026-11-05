# Master specification — implementation evidence ledger

**Branch:** `feature/master-spec-complete-2026-11` (main is intentionally untouched)

**Starting baseline:** `7530b7e0dad07dc78e7c8c24538ffef732fc8086` (0.3.8)

**Reference:** Digital Discipline — AI Implementation Master Specification v1.0 (5 November 2026)

This ledger distinguishes source work from installed Windows/Brave acceptance. A successful CI build alone does not mean a requirement is complete. Do not merge to main or publish as production without owner approval and gates.

## F01–F22 repair and hardening

| ID | Requirement | Current state | Evidence / gap |
|---|---|---|---|
| F01 | Privacy/network/key status | Implementation updated; runtime pending | `NetworkPrivacyStatus.tsx` |
| F02 | Pairing/Tracking Doctor | Existing pairing; doctor incomplete | `bridge.rs; BrowserSetupWizard.tsx` |
| F03 | Video counting/replay | Source present; real adapter matrix pending | `extension/src/video.js` |
| F04 | Event queue/rejection/retry | Improved; CI and real offline test pending | `queue-policy.js; background.js; tests/queue-policy.test.mjs` |
| F05 | Efficient live/aggregates | Open: snapshot still scans 100,000 sessions | `agent/src/main.rs` |
| F06 | Profile attribution | Partial; per-profile reliability unverified | `agent/src/bridge.rs; extension/src/background.js` |
| F07 | Autoplay/manual playback | Manual playback regression patched; full autoplay policy incomplete | `extension/src/video.js` |
| F08 | Custom dropdowns | Existing control; keyboard/accessibility audit open | `CustomSelect.tsx` |
| F09 | Command palette | Arrow-key/Enter command navigation added; focus trap runtime acceptance open | `App.tsx` |
| F10 | Form controls/import | Existing; high-DPI/manual testing open | `FileDropzone.tsx; style.css` |
| F11 | Onboarding/upgrade | Existing tour; real upgrade/version evidence open | `GuidedTour.tsx; BrowserSetupWizard.tsx` |
| F12 | Tray/agent lifetime | Existing; full shutdown/crash/duplicate process tests open | `src-tauri/src/main.rs` |
| F13 | Time/idle/lock | Existing; lock/sleep/clock correction tests open | `agent/src/domain.rs; windows.rs` |
| F14 | Policy conflict/explanations | Existing; enforcement accounting tests open | `agent/src/policy.rs` |
| F15 | Classification worker | Crash recovery patched; limits/evidence/owner override still open | `agent/src/classification.rs` |
| F16 | Retention/export/purge | Partial; full export and deletion semantics open | `agent/src/main.rs` |
| F17 | Analytics and coverage | Partial; full reconciliation/uncertainty tests open | `agent/src/main.rs; analytics.rs` |
| F18 | Deterministic release | CI branch enabled; lockfiles and compatibility checks open | `.github/workflows/windows-ci.yml` |
| F19 | Trust boundaries | Existing IPC/loopback checks; adversarial coverage open | `agent/src/bridge.rs` |
| F20 | Tests/runtime acceptance | Queue tests added; full Windows/Brave acceptance open | `tests/queue-policy.test.mjs` |
| F21 | Storage crash recovery | Recent SQLite write failures reported; full crash/disk fault recovery tests open | `agent/src/main.rs` |
| F22 | Overlay/multimonitor | Existing controls; high DPI/recovery tests open | `src-tauri/src/main.rs` |

## N01–N20 product roadmap

All N features remain **not accepted** until data contracts, UI, recovery, tests, and runtime behavior pass. Similar functionality in the baseline does not automatically satisfy the expanded scope.

| ID | Feature | Status |
|---|---|---|
| N01 | Task-based focus profiles | Partial implementation: profiles, self-reported task states, persistence, UI and tests; focus-task linkage acceptance open |
| N02 | YouTube Learning Mode | Planned / not accepted |
| N03 | Native Windows interventions | Planned / not accepted |
| N04 | Earned entertainment ledger | Planned / not accepted |
| N05 | Scoped temporary unlock | Planned / not accepted |
| N06 | Focus/break cycles | Planned / not accepted |
| N07 | End-session review/resume | Planned / not accepted |
| N08 | Full encrypted backup | Planned / not accepted |
| N09 | Verified updater | Planned / not accepted |
| N10 | Opt-in automatic device sync / phone | Planned / not accepted |
| N11 | Daily planner | Planned / not accepted |
| N12 | Presets and command actions | Planned / not accepted |
| N13 | Measured intervention impact | Planned / not accepted |
| N14 | Manual corrections | Planned / not accepted |
| N15 | Gentle intervention escalation | Planned / not accepted |
| N16 | Passive learning/reading | Planned / not accepted |
| N17 | Quiet notification inbox | Planned / not accepted |
| N18 | Weekly goals and recovery | Planned / not accepted |
| N19 | Presentation/private sessions | Partial implementation: presentation display masking; timed private session incomplete |
| N20 | Resource health/Lite Mode | Partial implementation: health panel, SQLite errors, Lite Mode; resource soak/metrics incomplete |

## Original A1–D8 coverage

Existing code represents the original 32 product concepts, but detailed per-feature installed acceptance is not complete. The master specification's coverage matrix controls the evidence requirements. These IDs remain carried forward: 

- A1–A8: baseline source present or partial; Windows/Brave acceptance and dependent F/N items remain open.
- B1–B8: baseline source present or partial; Windows/Brave acceptance and dependent F/N items remain open.
- C1–C8: baseline source present or partial; Windows/Brave acceptance and dependent F/N items remain open.
- D1–D8: baseline source present or partial; Windows/Brave acceptance and dependent F/N items remain open.

## Validation and release gates

- Baseline CI: Windows workflow from the master branch's audited commit previously passed; this is historical evidence, not evidence for this branch.
- This branch: enabled CI on `feature/master-spec-complete-2026-11`; follow branch Actions for exact result and artifact SHA.
- F04: added automated retry/quarantine/overflow tests; real offline/restart test still required.
- F07: removed unintended pause of manually playing video; verifying suppression of automatic next playback remains open.
- F15: recovery of stale Running jobs added; classification budget, provider error states, safe evidence and override tests remain open.
- CI compilation is not installed RDP/browser acceptance; no synthetic reports or success claims for unimplemented N items.
- Current signing is an ephemeral **self-signed development certificate**, not a trusted publisher identity.

## External approval and safety boundaries

- Keep `main` as protected owner backup. No merge, public release, external account setup, app store publication or automatic sync activation without separate authorization.
- Core tracking and restrictions remain local-first. Remote classification is opt-in; background network work must never block collection.
- Do not call the full roadmap complete while any required implementation, privacy, recovery, platform or runtime acceptance gate is open.

## Current feature-branch changes (not production-accepted)

- F01: truthful optional AI classification and credential-state display rather than false static 'Off' claims.
- F04: queue diagnostics, retry/backoff, terminal event quarantine, keep pending queue when consent is switched off; requires real MV3 offline/acknowledgement validation.
- F07: no pause of manually playing videos during stop-autoplay; next-play prevention remains adapter-dependent and unverified.
- F09: keyboard-selected command navigation with Arrow/Enter/Escape support; full accessibility check open.
- F15: stale background classification job recovery.
- F21: count and report recent SQLite collection write failures and drop unpersisted intervals; disk-error injection and data reconciliation still open.
- N01: local focus profiles and task workflow; task outcomes explicitly self-reported, not measured success.
- N19: screen-share presentation display masking only. Does not pause recording or implement timed private sessions.
- N20: redacted resource/health status and UI Lite Mode. Does not yet measure full CPU/memory profiles or implement all health/soak requirements.

**IMPORTANT:** This branch is a development integration, not the complete 42-item implementation. The remaining items and runtime tests above must stay open until actually delivered.
