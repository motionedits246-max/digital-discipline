# Current build and acceptance status

Repository: `motionedits246-max/digital-discipline-windows`. Branch: `feature/master-spec-complete-2026-11`. The owner explicitly authorized updates and GitHub Actions in the transferred repository on 7 November 2026. Main is unchanged.

Current source: desktop/agent **0.4.2**, companion **0.3.8**, event protocol **1.0**, database schema **13**. The entire master specification is not accepted; remaining work is recorded in the [implementation ledger](IMPLEMENTATION-LEDGER.md).

Tested source: `ea1c89a65acc00f65497bd22240461ae7ff21951`. [Windows run 42370611230](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42370611230) completed successfully on 7 November 2026. Its downloaded artifact was independently checked on 8 November. Documentation-only follow-ups do not change the tested binaries.

## Current verification

| Check | Evidence |
|---|---|
| JavaScript policies, queue recovery, adapters and installed companion dependencies | 51 passed |
| Rust agent | 56 passed on Windows, including DPAPI; 55 passed locally on Linux |
| SQLite legacy migrations | 8 passed |
| Privileged desktop route validation | 1 passed |
| TypeScript and production frontend | Passed |
| Renderer interactions | 7 passed in Microsoft Edge on Windows |
| Real agent lifecycle/workflows | Migration, duplicate instance, pause persistence, restart, task/profile, cycle/private recovery, Learning Mode, scoped unlock, complete export, review and graceful shutdown passed |
| Tauri/NSIS installer and artifact upload | Passed; development signature and checksums verified |

The renderer checks exercise both pointer and native-event privacy activation, draft preservation, calendar keyboard/viewport behavior, saved workflows, private sessions and scoped unlocks. No live provider request, real API key or owner browsing history was used by the tests.

## Verified download

[Download the complete Actions bundle](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42370611230/artifacts/13573294448): `DigitalDiscipline-MasterSpec-Branch-v0.4.2-DEV-SIGNED` (13,874,187 bytes; Actions retention until 7 December 2026).

- Installer: `Digital Discipline_0.4.2_x64-setup.exe`, 5,817,664 bytes. SHA-256: `a6281a93e357956372ce791780d29ac961b50d86b55cb75d981a893015ff0dc2`.
- Full artifact ZIP SHA-256: `5d0f7f6f5276c744252fd64551f7791def69357c6e903061b49a9cfe25bc343d`.
- Download verification matched the GitHub artifact digest and every entry in `SHA256SUMS.txt`. The companion ZIP contains both playback-model and queue-policy modules.
- The bundle includes the installer, agent, desktop executable, companion ZIP and public development certificate. The certificate does not establish Windows publisher trust.

## Earlier Windows evidence

[Run 42224610368](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42224610368) at `cdf49e0e1800f706d7322c3294e81469e4f040f0` succeeded: 45 JavaScript tests, 47 Windows Rust tests including DPAPI, 8 SQLite checks, privileged IPC validation, three Edge renderer checks, real agent restart/duplicate/pause/shutdown tests and the 0.4.0 installer. Installer SHA-256: `dfc1063159848850b252ba160ba55af1b2eaaaab6e53e34b1f94f2a022a60c06`.

[Run 42325309409](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42325309409) at `8e4d4339e48edf5a00ad91d7ef5efad4c256005e` passed 47 JavaScript and 51 Windows Rust tests, the expanded real agent workflow/private/restart/Learning Mode/exception/export/review checks and five of six renderer checks. The sixth exposed a misplaced unlock panel; the current source fixes the location and mixed-text dropdown labels.

## Scope and remaining acceptance

Current work includes custom calendar/time/select controls; a visible privacy shield that preserves drafts; durable classification memory and bounded automatic second review; request budgets/leases; compact live state and independent history reads; conservative browser/profile attribution; observed playback/replay and separate navigation visits; version-aware agent recovery; clock rollback protection; timed private collection windows, deletion tombstones and complete local export; focus workflows and local status records.

Queued AI jobs recheck the current app/domain exclusions when claimed. Excluded reviews and owner overrides do not prevent other eligible services from being classified; exclusion matching includes domain boundaries and case-insensitive Windows app names. Tests cover exclusions added after queuing and later removed, without making provider requests.

Presentation masking also closes portaled calendars/dropdowns when activated without a pointer event, including the native tray, and masks the browser setup screen without discarding its draft. The renderer privacy check covers open calendar/month popups and draft restoration.

The installer resource map includes the new playback model and queue policy modules. A packaging regression walks manifest entry points, HTML assets and local module imports against the installed companion resources; it failed on the missing playback module before the map was corrected.

Full encrypted backup/restore, trusted updating, automatic multi-device sync, earned entertainment rewards, planner/correction/passive-learning and later expanded roadmap items remain open. Native application intervention is a visible warning capability while foreground recording is active; it does not block or kill apps. Desktop notifications are inbox-only. Resource numbers currently measure the agent process.

Real owner-installed Windows/Brave upgrade/recording, multi-monitor/high-DPI, browser adapter edge cases, long recovery and 24-hour resource soak remain acceptance tasks. CI executables use self-signed development certificates, which do not establish Windows publisher trust.
