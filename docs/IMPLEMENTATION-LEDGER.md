# Master specification — implementation evidence ledger

**Branch:** `feature/master-spec-complete-2026-11` (main is intentionally untouched)

**Starting baseline:** `7530b7e0dad07dc78e7c8c24538ffef732fc8086` (0.3.8)

**Continuation baseline (6 November 2026):** `c71592e54039b3200d31662335fcd65a408c5ef1` (desktop/agent 0.3.9, schema 8)

**Reference:** Digital Discipline — AI Implementation Master Specification v1.0 (5 November 2026)

This ledger distinguishes source work from installed Windows/Brave acceptance. A successful CI build alone does not mean a requirement is complete. Do not merge to main or publish as production without owner approval and gates.

## F01–F22 repair and hardening

| ID | Requirement | Current state | Evidence / gap |
|---|---|---|---|
| F01 | Privacy/network/key status | Live independent Coach/classifier states and request history implemented; Windows/provider acceptance pending | `NetworkPrivacyStatus.tsx; classification.rs`; milestone below |
| F02 | Pairing/Tracking Doctor | Existing pairing; doctor incomplete | `bridge.rs; BrowserSetupWizard.tsx` |
| F03 | Video counting/replay | Source present; real adapter matrix pending | `extension/src/video.js` |
| F04 | Event queue/rejection/retry | Recovery/ACK/retention implementation verified in automated tests; real MV3 acceptance pending | `queue-policy.js; background.js; ingestion_tests.rs; tests/queue-recovery.test.mjs` |
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
| F15 | Classification worker | Request status and configuration revision guards added; budgets/full lease/evidence acceptance open | `agent/src/classification.rs` |
| F16 | Retention/export/purge | Partial; full export and deletion semantics open | `agent/src/main.rs` |
| F17 | Analytics and coverage | Partial; full reconciliation/uncertainty tests open | `agent/src/main.rs; analytics.rs` |
| F18 | Deterministic release | Cargo/npm lockfiles and enforced installs added; integrated Windows run/upgrade compatibility acceptance open | `Cargo.lock; package-lock.json; scripts/build-windows.ps1` |
| F19 | Trust boundaries | Existing IPC/loopback checks; adversarial coverage open | `agent/src/bridge.rs` |
| F20 | Tests/runtime acceptance | 35 JS, 29 Linux Rust, 8 SQLite checks pass; real Windows/Brave acceptance open | Milestone evidence below |
| F21 | Storage crash recovery | Ingestion rollback/retry failure injection passes; whole-store crash/disk recovery open | `agent/src/ingestion_tests.rs`; existing health reporting retained |
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
- This branch: CI was temporarily gated at `c71592e`. Re-enabled for this integrated source milestone; documentation-only changes are ignored. Follow the exact new Actions run for build and artifact evidence.
- F04: worker restart/backoff, poison/expiry progression, lost/foreign ACK, local write failure, bounded batches, purge and redaction checks now pass; real MV3 offline/restart acceptance is still required.
- F07: removed unintended pause of manually playing video; verifying suppression of automatic next playback remains open.
- F15: recovery of stale Running jobs added; classification budget, provider error states, safe evidence and override tests remain open.
- CI compilation is not installed RDP/browser acceptance; no synthetic reports or success claims for unimplemented N items.
- Linux local verification cannot exercise the Windows-only DPAPI round-trip test. It must run in Windows CI and then installed acceptance.
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


## 6 November 2026 — foundation continuation

This slice follows master-spec Phase 0 and part of Phase 1; it does not close Phase 1 or the full roadmap. See [the milestone](FOUNDATION-MILESTONE-2026-11-06.md) and [current build status](BUILD-STATUS.md).

Baseline: 23 JavaScript tests, 8 SQLite migration checks and the desktop TypeScript/Vite build passed before editing. Current local validation: 35 JavaScript tests, 29 Linux Rust tests, 8 SQLite checks, TypeScript/Vite and extension syntax/whitespace checks passed. Environment: Linux x86_64, rustc 1.99.1, Node 24.22.1, npm 11.9.0. No installed Windows/Brave test or live provider call was performed here.

| ID | Baseline | Phase/dependency | Final behavior in this slice | Source/tests | Runtime evidence | Remaining limitation |
|---|---|---|---|---|---|---|
| F01 / C8 / D3 | Partial source | Phase 1 privacy truth | Independent Coach/classifier consent, key-file state, provider/model and last request; unavailable is Unknown | NetworkPrivacyStatus.tsx; classification.rs; frontend build | Windows/provider pending | Saved-file presence does not establish DPAPI/provider usability; installed enable/disable/restart matrix remains |
| F04 / A2 / A5 | Partial queue recovery | Phase 1 observation integrity | Persistent jitter/backoff/wake recovery, exact ACK dispositions, poison progression, count/byte/age bounds and visible losses; explicit queue deletion | queue-policy and queue-recovery tests; Rust ingestion tests | Synthetic worker/storage/network and real in-memory SQLite tests passed | Real browser scheduler, worker termination, disk/quota and reconnect acceptance remains |
| F15 / A3 | Existing stale-job patch | Bounded support for F01 | Static request/error status and revision guard against stale configuration results | classification.rs; Rust build/tests | No provider request made | Request budgets, full attempt leases/local evidence and runtime override/config race acceptance remain |
| F18 | No lockfiles; source CI gated | Phase 0/reproducibility | Committed Cargo/npm locks, enforced installs, tool-version evidence, source-only Windows CI and checksum artifact | Cargo.lock; package-lock.json; build script/workflow | Local dependency resolution and builds passed | New Windows build evidence and fresh/upgrade acceptance remain; stable compiler/action pinning is not complete |
| F19 / F20 | Existing boundary and mocks | Phase 1 receiving boundary | Unsafe ACKs cannot remove data; redacted quarantine; regression tests exercise worker recovery | queue recovery tests; ingestion_tests.rs | 35 JS / 29 Linux Rust / 8 SQLite pass | Full adversarial API/import and real adapter matrix remain |
| F21 | Transactional ingestion present | Phase 1 storage integrity | Injected derived-write failure rolls back raw dedup ID; subsequent stable-ID retry succeeds | ingestion_tests.rs | In-memory SQLite fault scenario passed | Disk full/corrupt store/process-kill/recovery-copy scenarios remain |

No SQL schema migration was added. New classifier metadata uses existing settings. Extension diagnostics are local, bounded and contain no payloads; retained pending events follow recording consent and the declared queue policy. Existing player qualification logic, authorizations, profile IDs and restriction leases were preserved.

## Individual original feature ledger

Each original ID is retained below. The interpretation is source present/partial, not an installed-runtime pass. The changed slice supports A2/A3/A5/C8/D3; broader acceptance remains mapped to the listed F/N requirements. No new milestone runtime acceptance is claimed for any of these rows.

| ID | Original feature | Baseline/source interpretation | Dependent work and remaining acceptance |
|---|---|---|---|
| A1 | Universal app/time tracker | Present; arbitrary app basenames observed; categorization is separate | F13, F15, F21, N14, N16 |
| A2 | Shorts/Reels counter | Present, best effort; visible qualifying playback | F03, F04, F06 |
| A3 | App/content categorization | Rules, overrides, optional automatic AI exist; local publisher evidence is incomplete | F01, F15, N01, N02 |
| A4 | User-defined distraction time | Present; owner-selected categories | F17, N13, N19 |
| A5 | Video history | Present; IDs/durations, without default titles/transcripts | F03, F16, N02 |
| A6 | Attention switches | Present; observed transitions, not psychological conclusions | F13, F17, N07 |
| A7 | Browser comparison | Partial profile/window attribution | F06, F17 |
| A8 | Activity timeline | Present; day boundaries and collection gaps need continued validation | F13, F17, N14 |
| B1 | Website blocker | Present in paired browser profiles | F14, F19, N05 |
| B2 | Usage-based auto-block | Present; depends on reliable observed usage | F03, F04, F14 |
| B3 | Short-video/scroll cap | Present for qualifying observed plays; not arbitrary physical swipes | F03, F14 |
| B4 | Automation builder | Present for bounded IF/AND/THEN rules | F14, N12, N15 |
| B5 | Strict focus | Browser-only in baseline | F12, F14, N03, N05 |
| B6 | Partial site restrictions | Present, site-adapter dependent | F07, N02 |
| B7 | Shared time budget | Present; accounting must not double-count overlap | F14, F17, N04, N10 |
| B8 | Scheduled rules | Present; local-time restart/clock behavior needs verification | F13, F14, N11 |
| C1 | Floating controller | Pill, quick panel, workspace, popout represented | F09, F12, F22, N07 |
| C2 | Independent background engine | Present; reliability and explicit stop behavior remain important | F12, F21, N20 |
| C3 | Analytics dashboard | Present; data quality and query efficiency need work | F05, F17, N13 |
| C4 | Goal planner | Present, mainly app/domain time targets | N01, N06, N11, N18 |
| C5 | Command bar | Present; search/layout/accessibility verification needed | F09, N12 |
| C6 | Distraction-free web UI | Present for supported adapters | F07, N02 |
| C7 | Smart notifications | Present; consent, DND, quiet hours | F10, N15, N17 |
| C8 | Privacy settings | Present; copy and complete data lifecycle must match behavior | F01, F16, F19, N19 |
| D1 | Doomscroll detection | Present; explainable play/window thresholds | F03, F14, N15 |
| D2 | Goal-aware intervention | Present, bounded app/domain hint with dismissal | N01, N05, N15 |
| D3 | AI productivity coach | Present, BYOK, aggregate sharing, optional saved keys | F01, F15, N13 |
| D4 | Distraction prediction | Present with opt-in/history/evaluation gates | F17, N13 |
| D5 | Time-recovery simulator | Present; hypothetical values remain distinct | F17, N13 |
| D6 | Cross-device tracking | Partial; manual encrypted session exchange only | N08, N10 |
| D7 | Difficulty levels | Gentle, Balanced, Strict represented | F14, N03, N15 |
| D8 | Weekly insights | Present; comparable coverage and actionable review needed | F17, N13, N18 |
