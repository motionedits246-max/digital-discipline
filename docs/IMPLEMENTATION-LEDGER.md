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
| F02 | Pairing/Tracking Doctor | Per-profile staged Doctor implemented; real Brave acceptance open | `bridge.rs; BrowserSetupWizard.tsx` |
| F03 | Video counting/replay | Source present; real adapter matrix pending | `extension/src/video.js` |
| F04 | Event queue/rejection/retry | Recovery/ACK/retention implementation verified in automated tests; real MV3 acceptance pending | `queue-policy.js; background.js; ingestion_tests.rs; tests/queue-recovery.test.mjs` |
| F05 | Efficient live/aggregates | Compact live API; separate streamed history reader; stale response suppression tested | `agent/src/main.rs` |
| F06 | Profile attribution | Partial; per-profile reliability unverified | `agent/src/bridge.rs; extension/src/background.js` |
| F07 | Autoplay/manual playback | Replay/seek/speed/visibility/player replacement fixtures pass; autoplay restores original attributes; real adapter matrix open | `extension/src/video.js` |
| F08 | Custom dropdowns | Existing control; keyboard/accessibility audit open | `CustomSelect.tsx` |
| F09 | Command palette | Arrow-key/Enter command navigation added; focus trap runtime acceptance open | `App.tsx` |
| F10 | Form controls/import | Existing; high-DPI/manual testing open | `FileDropzone.tsx; style.css` |
| F11 | Onboarding/upgrade | Existing tour; real upgrade/version evidence open | `GuidedTour.tsx; BrowserSetupWizard.tsx` |
| F12 | Tray/agent lifetime | Authenticated version negotiation and bounded supervisor; real Windows agent restart/duplicate/stop tests pass; GUI crash/soak open | `src-tauri/src/main.rs` |
| F13 | Time/idle/lock | Rollback watermark and gap tests pass; lock/real sleep/passive acceptance open | `agent/src/domain.rs; windows.rs` |
| F14 | Policy conflict/explanations | Existing; enforcement accounting tests open | `agent/src/policy.rs` |
| F15 | Classification worker | Durable decision memory, one bounded review, leases, request budget, config and data-epoch guards tested; live provider acceptance open | `agent/src/classification.rs` |
| F16 | Retention/export/purge | Streamed consistent full local snapshots; private windows and scoped/global erasure tombstones tested; encrypted restore remains N08 | `agent/src/main.rs` |
| F17 | Analytics and coverage | Partial; full reconciliation/uncertainty tests open | `agent/src/main.rs; analytics.rs` |
| F18 | Deterministic release | Cargo/npm lockfiles and enforced installs added; Windows CI passed; installed upgrade/compatibility acceptance open | `Cargo.lock; package-lock.json; scripts/build-windows.ps1` |
| F19 | Trust boundaries | Existing IPC/loopback checks; adversarial coverage open | `agent/src/bridge.rs` |
| F20 | Tests/runtime acceptance | Current checkpoint: 51 JS, 56 Windows Rust, 8 SQLite, 1 IPC, 7 Edge renderer tests and Windows agent/workflow lifecycle pass; installed acceptance remains open | BUILD-STATUS.md and milestone evidence below |
| F21 | Storage crash recovery | Ingestion rollback/retry failure injection passes; whole-store crash/disk recovery open | `agent/src/ingestion_tests.rs`; existing health reporting retained |
| F22 | Overlay/multimonitor | Existing controls; high DPI/recovery tests open | `src-tauri/src/main.rs` |

## N01–N20 product roadmap

All N features remain **not accepted** until data contracts, UI, recovery, tests, and runtime behavior pass. Similar functionality in the baseline does not automatically satisfy the expanded scope.

| ID | Feature | Status |
|---|---|---|
| N01 | Task-based focus profiles | Foundations implemented: editable profiles, persisted task/focus context snapshots and separate owner outcomes; expanded profile-policy/runtime acceptance open |
| N02 | YouTube Learning Mode | Direct-video Learning Mode and selected-rule allowances implemented; other hard rules still apply; real adapter acceptance open |
| N03 | Native Windows interventions | Warning-only selected native apps implemented with recovery/assistive exclusions; no process blocking or termination |
| N04 | Earned entertainment ledger | Planned / not accepted |
| N05 | Scoped temporary unlock | 2/5/10-minute per-rule/site exceptions and optional profile scope implemented; expiry/conflict tests pass; browser runtime acceptance open |
| N06 | Focus/break cycles | Persisted work/break/paused/finished state and versioned controls implemented; restart/sleep simulation tested |
| N07 | End-session review/resume | Observed/planned session review, owner outcome and editable/deletable local bookmark/resume card implemented |
| N08 | Full encrypted backup | Planned / not accepted |
| N09 | Verified updater | Planned / not accepted |
| N10 | Opt-in automatic device sync / phone | Planned / not accepted |
| N11 | Daily planner | Planned / not accepted |
| N12 | Presets and command actions | Planned / not accepted |
| N13 | Measured intervention impact | Planned / not accepted |
| N14 | Manual corrections | Planned / not accepted |
| N15 | Gentle intervention escalation | Planned / not accepted |
| N16 | Passive learning/reading | Planned / not accepted |
| N17 | Quiet notification inbox | Grouped local inbox with persistent dismissal and retention implemented; desktop toast delivery unsupported |
| N18 | Weekly goals and recovery | Planned / not accepted |
| N19 | Presentation/private sessions | Visible draft-preserving presentation shield, tray status and timed private sessions with companion suppression implemented; installed acceptance open |
| N20 | Resource health/Lite Mode | Measured agent resources and Lite UI/history/AI/prediction scheduling implemented; GUI-process metrics and long soak open |

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
- F15: durable memory, bounded automatic review, stale-job recovery, request-budget accounting, provider result validation, exclusions and override regression checks pass. Live-provider and installed enable/disable/restart acceptance remain open.
- CI compilation is not installed RDP/browser acceptance; no synthetic reports or success claims for unimplemented N items.
- Linux local verification cannot exercise the Windows-only DPAPI round-trip test. It must run in Windows CI and then installed acceptance.
- Current signing is an ephemeral **self-signed development certificate**, not a trusted publisher identity.

## External approval and safety boundaries

- Keep `main` as protected owner backup. No merge, public release, external account setup, app store publication or automatic sync activation without separate authorization.
- Core tracking and restrictions remain local-first. Remote classification is opt-in; background network work must never block collection.
- Do not call the full roadmap complete while any required implementation, privacy, recovery, platform or runtime acceptance gate is open.

## Initial 6 November feature-branch checkpoint (historical)

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

Baseline: 23 JavaScript tests, 8 SQLite migration checks and the desktop TypeScript/Vite build passed before editing. Current local validation: 35 JavaScript tests, 31 Linux Rust tests, 8 SQLite checks, TypeScript/Vite and extension syntax/whitespace checks passed. Environment: Linux x86_64, rustc 1.99.1, Node 24.22.1, npm 11.9.0. No installed Windows/Brave test or live provider call was performed here.

| ID | Baseline | Phase/dependency | Final behavior in this slice | Source/tests | Runtime evidence | Remaining limitation |
|---|---|---|---|---|---|---|
| F01 / C8 / D3 | Partial source | Phase 1 privacy truth | Independent Coach/classifier consent, key-file state, provider/model and last request; unavailable is Unknown | NetworkPrivacyStatus.tsx; classification.rs; frontend build | Windows/provider pending | Saved-file presence does not establish DPAPI/provider usability; installed enable/disable/restart matrix remains |
| F04 / A2 / A5 | Partial queue recovery | Phase 1 observation integrity | Persistent jitter/backoff/wake recovery, exact ACK dispositions, poison progression, count/byte/age bounds and visible losses; explicit queue deletion | queue-policy and queue-recovery tests; Rust ingestion tests | Synthetic worker/storage/network and real in-memory SQLite tests passed | Real browser scheduler, worker termination, disk/quota and reconnect acceptance remains |
| F15 / A3 | Existing stale-job patch | Bounded support for F01 | Static request/error status and revision guard against stale configuration results | classification.rs; Rust build/tests | No provider request made | Request budgets, full attempt leases/local evidence and runtime override/config race acceptance remain |
| F18 | No lockfiles; source CI gated | Phase 0/reproducibility | Committed Cargo/npm locks, enforced installs, tool-version evidence, source-only Windows CI and checksum artifact | Cargo.lock; package-lock.json; build script/workflow | Local dependency resolution and builds passed | New Windows build evidence and fresh/upgrade acceptance remain; stable compiler/action pinning is not complete |
| F19 / F20 | Existing boundary and mocks | Phase 1 receiving boundary | Unsafe ACKs cannot remove data; redacted quarantine; regression tests exercise worker recovery | queue recovery tests; ingestion_tests.rs | 35 JS / 31 Linux Rust / 8 SQLite pass | Full adversarial API/import and real adapter matrix remain |
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


### Credential replacement failure-path review

A follow-up source review found that overwriting one key file before committing provider settings could leave a mismatched key/provider after a failed DB write. F15/F21 now use versioned encrypted classifier blobs and a transactionally committed active revision. Two failure/cleanup regression tests pass with isolated dummy ciphertext; Windows DPAPI and provider runtime acceptance remain open. Existing legacy keys are read without migration until the owner replaces them.


### Verified Windows build evidence

[Run 42154963954](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42154963954) passed for source commit `767adde5806f78c27f79907b5096ed4debc90aba`: 35 JavaScript tests, 32 Windows Rust tests (including DPAPI round-trip), 8 SQLite checks, frontend/agent/Tauri packaging, expected development signer identity, and artifact upload. See BUILD-STATUS.md for CI-computed installer and outer-ZIP hashes. The superseded run 42154482188 was cancelled. Documentation-only follow-ups do not change the tested code.

Windows CI is not installed Windows/Brave acceptance. F01/F04/F15/F18/F19/F20/F21 retain the remaining runtime/platform limits above; Phase 1 and the full roadmap remain open.


## v0.4.0 integrated continuation — implementation in progress

The user requested one combined handoff, including a custom date selector, a visible Presentation Mode privacy screen, automatic bounded AI review and saved classifications. This continuation also adds profile-scoped Tracking Doctor stages, compact live refreshes, independent streaming history reads, conservative browser foreground attribution, agent version negotiation, bounded recovery and clock rollback protection.

Local evidence before the Windows run: 40 JavaScript tests, 40 Linux Rust tests, 8 SQLite smoke tests, one pure IPC route test, and TypeScript/Vite build. New renderer interaction and isolated Windows agent lifecycle tests are now required in CI. This entry is not installed Windows/Brave acceptance or completion of the full roadmap. Later changes will update this evidence and the remaining work.


### Integrated Windows checkpoint verified

Source `d4f5944bbd306b55d2296f78a8573709d7421c28`, run [42223016774](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42223016774), succeeded. Evidence: 40 JavaScript tests, 41 Windows agent tests (including DPAPI), 8 SQLite tests, 1 privileged IPC route test, 3 renderer interaction tests in Edge, real agent version/duplicate/pause/restart/shutdown checks, frontend and Windows installer build. This is automated Windows evidence, not owner-installed Brave acceptance. Installer SHA-256: `e6b7278615528b44e4090380cc09ae42a88ed5b4c2790b9f74fe9b6b02b24f7d`. Development signatures remain self-signed and untrusted.

The next source checkpoint adds tested player adapter/replay/speed/seek/autoplay behavior, separately labeled route visits, schema-11 private intervals and deletion tombstones, AI data-epoch guards, scoped erasure, complete streamed local snapshots (provider credentials excluded; not an encrypted restore system), measured agent resources and persisted Lite scheduling. Further workflow implementation and final combined validation are in progress.


## 7 November continuation — focus workflow foundation

The canonical repository moved to `motionedits246-max/digital-discipline-windows`; the branch and source history were preserved. Checkpoint `cdf49e0e1800f706d7322c3294e81469e4f040f0` passed [Windows run 42224610368](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42224610368): 45 JS, 47 Windows Rust, 8 SQLite, 3 Edge interaction tests and real agent lifecycle/snapshot checks. Installer SHA-256 `dfc1063159848850b252ba160ba55af1b2eaaaab6e53e34b1f94f2a022a60c06`.

Current source advances desktop/agent to 0.4.1, companion to 0.3.7 and schema to 12:

- N01: editable app/site profiles; persisted focus/task association and immutable per-session relevance snapshot. Starting focus moves draft/paused task to active, never to completed. Owner outcomes remain separate.
- N02: explicit direct-video Learning Mode; selected-rule exceptions and separate home/related/Shorts/autoplay controls. Another hard rule still blocks the same video. Playlist/channel/embedded exceptions are explicitly unsupported; no remote transcript/content analysis.
- N05: 2/5/10-minute exceptions for one selected rule and configured site, optional profile scope and predefined reason; emergency recovery stays separate. Companion expiry alarms and short leases remain fail-open when the agent is unavailable.
- N06: persisted work/break/paused/finished cycle, bounded scheduled catch-up, optimistic version checks, skip/extend/finish and original-rule restoration. Timers never mint measured work.
- N07: reviews show planned versus observed eligible/activity time, coverage, switches and owner outcomes; editable/deletable local next-step notes and a dismissible quick-panel resume card.
- N19: private session and presentation state are distinct; privacy shield is visible and tray can display/toggle presentation state alongside recording/private status.

Local evidence for this source checkpoint: 47 JS and 50 Linux Rust tests, TypeScript/Vite build. Added renderer workflow/private/unlock interaction checks and expanded real Windows agent restart/learning/exception/review/snapshot checks; their CI result is pending. No claim that all N features or installed Brave acceptance are complete. Native app intervention, reward ledger, encrypted restore, verified updates/sync, planner, corrections, passive-learning eligibility and later review/notification features remain to be implemented or expanded.


## 7 November — approved transfer and integrated 0.4.2 update

The owner explicitly confirmed `motionedits246-max/digital-discipline-windows` as the update/Actions destination and added its connected account. The earlier automatic approval block on the changed owner is resolved by that instruction. Source updates continue on the existing feature branch.

Desktop/agent 0.4.2, companion 0.3.8 and schema 13 add a grouped local inbox, safe warning-only native app selection, independent WAL reads for reviews, corrected Blocking Center unlock placement and mixed-text selector labels. Private-window status reaches the companion; new player observations are suppressed without revoking consent. Global deletion clears prior companion queues/counters/quarantine on successful sync while preserving identity. Snapshot exports allowlist preferences, omit credentials/enrollment/device identity, and cancel if deletion/privacy changes invalidate their source epoch. Lite Mode also pauses optional prediction work.

Current evidence: 51 JavaScript, 56 Windows Rust (55 Linux), 8 SQLite, 1 privileged-route and 7 Edge renderer checks pass. The real Windows agent/workflow restart checks, frontend, Tauri/NSIS installer, development-signature verification and artifact upload also pass. No entire-spec completion or publicly trusted installer claim is made.


Final review also carries presentation masking to companion notification delivery. Browser notifications use a generic message while Presentation Mode is active; the agent's foreground recording and approved rules remain independent. Tracking Doctor explains private-window suppression and shows the expected companion version rather than prompting the owner to re-enable consent during a private session. Added regression checks verify notification label redaction and continued recording under display masking.

The classifier now rechecks exclusions at the request-claim boundary, including old queued jobs. Filtering occurs before the batch limit, and manually categorized review entries cannot starve eligible new jobs. A regression covers app-name case, domain/subdomain exclusions, an unrelated lookalike domain, automatic review and removal of a prior exclusion. No excluded job consumes an attempt in that scenario.

Installer auditing found that the static Tauri resource map omitted the new playback-model and queue-policy modules. Both are now packaged. The new dependency-walk regression checks installed companion manifest entry points, HTML references and module imports, and reproduced the omission before the fix. Tray-driven masking also covers open portaled controls and preserves browser setup drafts.

## Verified 0.4.2 build handoff — 8 November

[Run 42370611230](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42370611230) succeeded for exact source `ea1c89a65acc00f65497bd22240461ae7ff21951`. Artifact `13573294448` includes the corrected companion resources. Downloaded bytes match the recorded artifact digest and all internal checksums; see [build status](BUILD-STATUS.md) for hashes and the download. Documentation-only commits following this source do not rebuild or change those binaries. Main remains untouched. The remaining roadmap and owner-installed Windows/Brave acceptance above remain open.

## 8 November — owner-requested 0.4.3 continuation

Desktop/agent 0.4.3, companion 0.3.9 and schema 14 implement custom confirmations, animated workspace collapse, in-place companion refresh and compact service classification prompts. This extends F08/F10/F11/F15/F16/F19/F22; it does not complete the remaining N roadmap.

Classification follows saved/manual choices, Windows executable descriptions and conservative keyword rules, then optional bounded public-root/DuckDuckGo metadata. Public lookups are off until opted in. They use public-address checks, pinned DNS, no redirects/cookies/proxy forwarding, deadlines and response limits. Full URLs, browser page content, window titles and executable paths do not leave the device. Ambiguous app-name web summaries are suggestions until confirmed. All LLM providers, including Pollinations, require a saved encrypted key, classifier consent and per-service approval tied to the current configuration revision. Old queued jobs are not implicitly approved by migration. Existing accepted categories remain reusable.

The approximately 356×156 logical-pixel service window does not take focus, expires after about 5.5 seconds plus its exit animation, pauses while being used and is muted by presentation/private/pause controls. Only one recent service is shown at a time, with a twenty-second cadence cap. Timeout leaves the record in Service discovery. The helper window is created only when needed and closes afterward.

Companion updates use the already-installed desktop bundle. They update known owned locations or a desktop-approved existing companion folder, preserve unrelated files, stage replacements with the manifest last, and roll back write failures. The extension cannot supply arbitrary update destinations. This is not the trusted internet application updater in N09. One initial Reload of an existing 0.3.8 companion is required to load its new update button; removing/re-adding is unnecessary.

Verified checks: 53 JavaScript, 69 Windows Rust (68 Linux), 8 SQLite migration checks, 1 privileged-route test, all 12 Edge renderer checks, the production frontend, real Windows agent/update/enrollment runtime checks and Tauri/NSIS packaging pass. No live owner provider call, installed Brave update, actual Hive metadata or multi-monitor popup acceptance is claimed before those scenarios run.

Primary API references: [Chrome extension runtime lifecycle](https://developer.chrome.com/docs/extensions/reference/api/runtime), [Pollinations authentication](https://gen.pollinations.ai/docs), [Tauri webview-window builder](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html).

The final 0.4.3 source is 55bb568299fe6d76305c352808bc1b5c469cec19, validated by [run 42485544736](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42485544736). The downloaded artifact and all internal checksums match; see BUILD-STATUS.md for the installer hash. Manual category display, one in-flight AI request, saved-rule precedence, manifest public-key retention and downgrade prevention are included. The initial renderer failures were test timing assertions; the final run waits for asynchronous saves and separately advances timeout/exit animation. Main is unchanged. Remaining roadmap and installed-runtime acceptance remain open.

## 9 November — v0.4.4 workspace sizing and shared notifications

The owner assigned website work to another agent. This continuation changes the desktop/agent feature branch only and extends F10/F22.

Full workspace and popout modes now use a 960×640 logical-pixel minimum. On displays whose usable logical work area is smaller, the available area caps the minimum so the window remains reachable. Pill and quick-panel modes restore their own constraints and cannot be accidentally resized into a broken layout. Placement excludes the taskbar, and crossing monitors or changing DPI reapplies the limits.

Service suggestions, new companion authorization requests/approvals and new grouped inbox items share the compact notification surface. An unattended notice closes within three seconds including its exit animation. Pointer/keyboard interaction holds it open; native window deactivation or an outside click starts a new three-second dismissal period. Native deactivation also closes portaled dropdowns, so stale DOM focus cannot hold a popup indefinitely. Returning during the exit animation cancels the stale close. New notices wait while another is active, and grouped items/pending retries are not repeatedly announced. Timing out leaves service and inbox records available for later review.

Browser events refer to the local companion connection, not website account sign-ins. Notices do not authorize extensions, enable recording or approve AI. Presentation Mode, private sessions and tracking pause suppress delivery. Desktop overlays are separate from the companion's opt-in Windows notifications and its DND/quiet-hours policy.

Final source `f1248d2e14837c664da29eda5ecfc94e7d6e45ef` passed [Windows run 42574830666](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42574830666): 60 JavaScript, 69 Windows Rust, 3 native sizing, 8 SQLite and 1 privileged-route checks; all 15 Edge renderer tests; real agent lifecycle/workflow/private restart, snapshot and companion update/enrollment checks; production frontend and Tauri/NSIS packaging. The downloaded artifact and all internal checksums match. Installer SHA-256: `cee81d66fe23008bc8042de4e762210fc696898ef08575dee5f020298497f9d3`. Installed Windows/Brave/physical multi-monitor acceptance and the remaining N roadmap stay open.

API references: [Tauri monitor work area](https://docs.rs/tauri/latest/tauri/window/struct.Monitor.html#method.work_area), [window minimum size](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindow.html#method.set_min_size), [window events](https://docs.rs/tauri/latest/tauri/enum.WindowEvent.html).
