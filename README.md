# Digital Discipline

Windows-first, local-first activity tracking and reversible browser restrictions, with a Rust agent, Tauri/React desktop and Brave-first MV3 companion.

**Development branch:** `feature/master-spec-complete-2026-11`. Desktop/agent source version: **0.4.4**. Companion: **0.3.9**. Event protocol: **1.0**. This branch implements the master specification in stages; it is not a complete or production-accepted release.

Read the [implementation ledger](docs/IMPLEMENTATION-LEDGER.md) for all F01–F22, N01–N20 and original A1–D8 requirements, and the [current build status](docs/BUILD-STATUS.md) for changes, test evidence and remaining acceptance.

The integrated **0.4.4 Windows development build** is available in the [verified Actions bundle](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42574830666/artifacts/13655195843). It uses a self-signed development certificate. See build status for the tested source and SHA-256 checksums.

## Current capabilities

- Independent local Windows foreground/idle tracking, SQLite history and owner-defined categories.
- Floating pill, quick panel, dashboard overlay, optional separate window and tray controls.
- Opt-in observed playback for YouTube Shorts, ordinary YouTube videos and Instagram Reels. A play requires **2 seconds of visible playback progression**; route visits are not watched plays.
- Domain/path browser restrictions, schedules, usage budgets, automation, goals and browser-only focus sessions.
- Optional BYOK Coach and a lightweight service-classification ladder. Saved choices and executable descriptions run locally first; public metadata/DuckDuckGo lookups are separately opt-in. LLM batches contain only individually approved service names.
- Current-user Windows DPAPI encryption for explicitly saved provider keys.
- Editable profiles, task-linked focus, persistent work/break cycles, selected-video Learning Mode, scoped temporary unlocks and local session reviews/resume cards.
- Visible Presentation Mode, timed private sessions, profile-scoped Tracking Doctor, grouped local inbox and warning-only selected native-app interventions.
- Complete streamed local snapshots with allowlisted preferences; credentials/enrollment omitted. Measured agent resources and Lite scheduling are available. Expanded roadmap acceptance remains open.
- Manual encrypted exchange of recent device sessions. This is not a full backup or automatic sync.

## Recent improvements — 0.4.3–0.4.4

- Custom confirmation dialogs replace browser-native prompts in the desktop and companion.
- Workspace close animates into the pill; reduced-motion/Lite preferences are respected.
- Full GUI mode has a native 960×640 minimum, bounded by the usable display area; compact controls retain their own sizes.
- Shared service/browser/inbox overlays expire within three seconds, stay open during interaction and dismiss after app/window switches or outside clicks. Timeout never approves an LLM request; unresolved service items remain in Activity History → Service discovery.
- Windows File Description/Product Name and conservative keyword rules can identify an AI development environment locally. Optional public lookups are bounded to 20 requests per day. Pollinations is supported as a keyed, explicitly approved LLM provider.
- The companion popup can update from the installed desktop bundle and reload in place. Browser setup can register a custom existing extension folder. Updates retain extension ID, profile, consent and pending observations; owned files have rollback protection.

**Upgrade from companion 0.3.8:** install the desktop update, then Reload your existing unpacked extension once to get the new Update & reload button. For a custom folder, register that same folder in desktop browser setup first. Keep its existing location and refresh already-open video tabs after reload. New installations use the stable DigitalDiscipline/browser-extension user folder.

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
- The complete local snapshot includes a completion footer and is cancelled if privacy or deletion changes invalidate it. Exclusions apply to future collection; use scoped or full deletion to erase existing history. Full encrypted restore, verified updates and automatic sync remain roadmap work.

The original [blueprint](docs/blueprint/README.md) and its 32 feature definitions are retained as design history. Historical release notes describe their own revisions; use the current ledger and build evidence for this branch.
