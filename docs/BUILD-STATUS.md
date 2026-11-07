# Current build and acceptance status

Repository: `motionedits246-max/digital-discipline-windows`. Branch: `feature/master-spec-complete-2026-11`. The owner explicitly authorized updates and GitHub Actions in the transferred repository on 7 November 2026. Main is unchanged.

Current source: desktop/agent **0.4.2**, companion **0.3.8**, event protocol **1.0**, database schema **13**. The entire master specification is not accepted; remaining work is recorded in the [implementation ledger](IMPLEMENTATION-LEDGER.md).

## Current verification

| Check | Evidence |
|---|---|
| JavaScript policies, queue recovery and adapters | 50 passed |
| Rust agent on Linux | 54 passed |
| SQLite legacy migrations | 8 passed |
| TypeScript and production frontend | Passed |
| Renderer interactions | Six checks passed in isolated local Chromium processes; latest Windows rerun pending |
| Latest Windows installer | Build to be run for this source; no 0.4.2 installer is claimed yet |

The Linux test browser required isolated processes because its serverless Chromium build cannot reliably reuse contexts in this runtime. Windows CI uses installed Microsoft Edge. No live provider request, real API key or owner browsing history was used by the tests.

## Earlier Windows evidence

[Run 42224610368](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42224610368) at `cdf49e0e1800f706d7322c3294e81469e4f040f0` succeeded: 45 JavaScript tests, 47 Windows Rust tests including DPAPI, 8 SQLite checks, privileged IPC validation, three Edge renderer checks, real agent restart/duplicate/pause/shutdown tests and the 0.4.0 installer. Installer SHA-256: `dfc1063159848850b252ba160ba55af1b2eaaaab6e53e34b1f94f2a022a60c06`.

[Run 42325309409](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42325309409) at `8e4d4339e48edf5a00ad91d7ef5efad4c256005e` passed 47 JavaScript and 51 Windows Rust tests, the expanded real agent workflow/private/restart/Learning Mode/exception/export/review checks and five of six renderer checks. The sixth exposed a misplaced unlock panel; the current source fixes the location and mixed-text dropdown labels.

## Scope and remaining acceptance

Current work includes custom calendar/time/select controls; a visible privacy shield that preserves drafts; durable classification memory and bounded automatic second review; request budgets/leases; compact live state and independent history reads; conservative browser/profile attribution; observed playback/replay and separate navigation visits; version-aware agent recovery; clock rollback protection; timed private collection windows, deletion tombstones and complete local export; focus workflows and local status records.

Full encrypted backup/restore, trusted updating, automatic multi-device sync, earned entertainment rewards, planner/correction/passive-learning and later expanded roadmap items remain open. Native application intervention is a visible warning capability while foreground recording is active; it does not block or kill apps. Desktop notifications are inbox-only. Resource numbers currently measure the agent process.

Real owner-installed Windows/Brave upgrade/recording, multi-monitor/high-DPI, browser adapter edge cases, long recovery and 24-hour resource soak remain acceptance tasks. CI executables use self-signed development certificates, which do not establish Windows publisher trust.
