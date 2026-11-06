# Digital Discipline

Windows-first, local-first activity tracking and reversible browser restrictions, with a Rust agent, Tauri/React desktop and Brave-first MV3 companion.

**Development branch:** `feature/master-spec-complete-2026-11`. Desktop/agent version: **0.3.9**. Companion: **0.3.5**. Event protocol: **1.0**. This branch implements the master specification in stages; it is not a complete or production-accepted release.

Read the [implementation ledger](docs/IMPLEMENTATION-LEDGER.md) for all F01–F22, N01–N20 and original A1–D8 requirements, and the [current foundation milestone](docs/FOUNDATION-MILESTONE-2026-11-06.md) for changes, test evidence and remaining acceptance.

## Current capabilities

- Independent local Windows foreground/idle tracking, SQLite history and owner-defined categories.
- Floating pill, quick panel, dashboard overlay, optional separate window and tray controls.
- Opt-in observed playback for YouTube Shorts, ordinary YouTube videos and Instagram Reels. A play requires **2 seconds of visible playback progression**; route visits are not watched plays.
- Domain/path browser restrictions, schedules, usage budgets, automation, goals and browser-only focus sessions.
- Optional BYOK Coach and separately authorized automatic classification of batches of up to ten public hostnames/application basenames.
- Current-user Windows DPAPI encryption for explicitly saved provider keys.
- Local profiles/tasks, presentation display masking and health/Lite Mode foundations; expanded roadmap acceptance remains open.
- Manual encrypted exchange of recent device sessions. This is not a full backup or automatic sync.

## Build and check

Windows requires Rust stable MSVC, Visual Studio C++ Build Tools/Windows SDK, Node.js 22+, Python 3 and WebView2.

```powershell
node --test tests/*.test.mjs
python tests/migration-contract.py
python tests/browser-pair-migration.py
python tests/workflow-migration.py
cargo test --locked -p digital-discipline-agent
cd apps/desktop
npm ci
npm run build
```

For development use `scripts/dev-windows.ps1`. For Windows packaging use:

```powershell
.\scripts\build-windows.ps1
```

Committed Cargo/npm lockfiles pin dependency resolution. The Windows workflow records actual toolchain versions and builds this feature branch on source changes. Documentation-only commits do not rebuild the installer. Artifacts and SHA256SUMS are available only after a successful run; see [build status](docs/BUILD-STATUS.md).

Without a configured certificate the script may produce unsigned files. CI uses an ephemeral **self-signed development certificate**. This is not publicly trusted signing and does not guarantee SmartScreen acceptance.

## Connect or upgrade Brave

1. Open Settings & Privacy → browser setup. Use the bundled companion folder.
2. In `brave://extensions`, enable Developer mode and load the unpacked folder.
3. Authorize its extension ID in the desktop app, then test the direct local connection in the companion.
4. Turn on **Record browser activity** only if you want browser observations; it is off by default. Browser restrictions sync separately.
5. Refresh an open YouTube/Instagram tab and play visibly for 3–5 seconds for an unambiguous test of the 2-second threshold.

**Upgrade in place:** preserve the existing unpacked folder and extension ID, replace its files and click Reload. Removing the extension erases its local queue. Normal companion connection uses the restricted loopback bridge, not native messaging or registry setup.

The companion's Delivery & recovery section distinguishes acknowledged events, paused/excluded events, terminal rejections, expiry, capacity losses and deliberate queue deletion. These are delivery counters, not video totals.

## Privacy and recovery

- The agent stores executable basenames, observed intervals/idle state and opted-in domains/video IDs under the current Windows user. No default window titles, full executable paths, screenshots, clipboard, typed text, private messages, video titles or transcripts.
- Recording, notifications, automatic classification and Coach sharing have separate preferences. Classification can make background provider calls only after explicit enablement. The Coach is initiated by the owner.
- Saved keys are DPAPI encrypted; one-time keys are not saved. The activity database is not described as fully encrypted. Settings reports key-file presence and request status without displaying a key.
- Turning companion recording off stops new collection and retains pending observations for local sync. Delete pending observations explicitly if desired.
- Queue retention is bounded to 2,000 events or 2 MB for 24 hours. Rejection metadata is limited to 100 entries for seven days and contains no event payloads. Browser wake/sleep can delay retries.
- Pausing tracking does not disable restrictions. Browser policies use short fail-open leases (at most three minutes without renewal); emergency recovery is available in the companion.
- Closing the desktop keeps intended background activity; use explicit Quit to stop it. Installed lifecycle/crash acceptance still needs the scenarios in the ledger.
- Local API authentication protects against ordinary unauthorized calls, not malware running as the same Windows user.
- Existing exports can be truncated and exclusion does not automatically purge old history. Full backup, scoped purge, verified updates and automatic sync remain roadmap work.

The original [blueprint](docs/blueprint/README.md) and its 32 feature definitions are retained as design history. Historical release notes describe their own revisions; use the current ledger and build evidence for this branch.
