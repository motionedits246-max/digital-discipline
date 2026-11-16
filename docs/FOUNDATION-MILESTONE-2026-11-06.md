# Foundation milestone — 6 November 2026

## Scope and resulting behavior

This is the first continuation from feature-branch integration base `c71592e54039b3200d31662335fcd65a408c5ef1`, following Phase 0 and a bounded Phase 1 slice of the master specification.

- **F04, F19, F20:** retry state survives MV3 worker restarts; browser alarms restore pending work. Backoff includes jitter and is bounded to 30 seconds–5 minutes, subject to browser scheduling/sleep. Explicit Test/Sync can retry immediately.
- **F04:** HTTP 404 is retryable for an old/unavailable bridge. Terminal malformed-event errors are quarantined without blocking later valid events. A queue pass handles at most 50 events/10 seconds before yielding.
- **F04:** storage bounds are explicit: 2,000 events, 2 MB, 24-hour maximum age, 8 KiB per stored envelope. Evictions/expiry/invalid data are counted. Up to 100 redacted metadata records expire after seven days during queue maintenance.
- **F04/F21:** agent receipts include the event ID and accepted/duplicate/ignored-paused/ignored-excluded disposition. Legacy 1.0 receipt flags remain supported. A foreign ID or unexpected disposition cannot delete the pending event.
- **F04/C8:** recording-off retains pending data for local sync. Queue deletion is separate, with a review explaining that an already transmitted observation may still arrive.
- **F01:** Settings independently reports automatic classification, provider/model, encrypted key-file presence, last request result/time, Coach sharing and saved Coach provider names. Unavailable status is shown as Unknown rather than Off.
- **F01/F15:** classifier request status is stored as timestamps and fixed result/error codes, never provider bodies or prompts. A changed provider/key/configuration invalidates an older in-flight result; configuration versions prevent a quick disable/re-enable from applying it.
- **F15/F21:** classifier replacement writes a new versioned encrypted blob, then transactionally commits its provider/model/revision. A failed configuration write retains the previous key/provider pair. Successful replacement cleans up old classifier blobs; Disable and delete also removes unreachable classifier blobs left after a crash. Legacy unversioned keys remain readable until replaced.
- **F18:** Cargo/npm lockfiles are committed and packaging enforces locked Cargo tests/build plus npm ci. Source changes on this branch run Windows CI; documentation-only changes do not restart it. Successful packaging emits SHA256SUMS.txt.

Playback detection in `extension/src/video.js`, the 2-second threshold, authorization IDs, profile IDs, existing rules and recording consent defaults were preserved.

## Data and compatibility

No new SQLite schema version is required. New classifier keys use versioned DPAPI filenames; the active revision is stored with the provider configuration. No plaintext key is written. Classifier status/configuration revision uses the existing settings table. Extension diagnostics add local counters and retry metadata. Rejection history stores only bounded event IDs, known kinds, fixed reason codes and timestamps.

Events retain their original IDs on retry. No event replay/age window was expanded. Provider keys remain in the Windows-user DPAPI vault and are not sent to diagnostics or portable exports.

Desktop/agent 0.3.9 and companion 0.3.5 use additive event protocol 1.0 receipts. The companion accepts older 1.0 legacy acknowledgement flags for in-place upgrades. A key-file-present status does not claim that DPAPI decryption or a live provider call has been verified.

## Automated evidence

- 35 Node tests pass (23 at baseline; 12 added). Nine recovery scenarios execute the real companion worker with synthetic Chrome/network/storage surfaces.
- 31 Rust tests pass on Linux. Added tests exercise real in-memory SQLite ingestion: lost-reply duplicate replay; rollback after an injected derived-write failure; pause/exclusion receipts. Two additional filesystem/SQLite tests cover failed configuration commits and scoped key cleanup using dummy encrypted fixtures.
- [Windows CI run 42154963954](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42154963954) passed for source commit `767adde5806f78c27f79907b5096ed4debc90aba`: 32 Windows Rust tests including DPAPI round-trip, 35 Node tests, eight SQLite checks, production agent/desktop/installer, development signer checks and artifact upload. Installer/ZIP hashes are recorded in BUILD-STATUS.md.
- Eight Python/SQLite migration checks pass.
- Desktop strict TypeScript check and Vite production build pass.
- Extension syntax checks and git diff whitespace checks pass.

Mocked browser surfaces do not prove real MV3 scheduling, Brave permissions, or YouTube/Instagram adapter compatibility. No real provider request or private data was needed.

## Required installed checks

1. Update the existing companion folder in place; verify authorization/profile and recording preference remain.
2. Disconnect the agent, generate observations, terminate/restart the MV3 worker, and reconnect. Verify pending data progresses once and retry status is visible.
3. Pause recording, confirm new playback is not counted, inspect retained pending data and test explicit queue deletion.
4. Test paused/excluded receipts and compare local observations with acknowledged dashboard metrics; do not equate event-delivery counts with videos.
5. Enable classification with a test key, verify Settings status, restart and verify encrypted-key/consent state. Change configuration during a request and confirm the old result is ignored.
6. Disable/remove classification and verify no new calls begin. Already transmitted provider data cannot be recalled.
7. Complete the broader T01–T40 Windows/browser scenarios before calling Phase 1 accepted.

## Remaining work

F02/Tracking Doctor, full F03 playback/visit/replay coverage, efficient live/history separation (F05), conservative per-profile attribution (F06), full autoplay behavior (F07), lifecycle/time/policy correctness (F12–F14), complete data lifecycle/reconciliation (F16–F17), and remaining input/storage failure acceptance (F19/F21) are still open. F15 still needs bounded request budgets, full attempt leases and local publisher evidence. Windows/Brave runtime acceptance is pending.

The N01–N20 roadmap remains staged in the canonical ledger. No reward minting, native blocking guarantee, full backup/updater or automatic sync is claimed in this milestone.

Technical references checked: [Chrome alarms](https://developer.chrome.com/docs/extensions/reference/api/alarms) and [Chrome storage](https://developer.chrome.com/docs/extensions/reference/api/storage). Alarm timing and persistence vary by browser/version; the worker recreates its queue wake alarm rather than assuming persistence.
