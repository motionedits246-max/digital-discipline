# Digital Discipline — Windows

**Digital Discipline** is a Windows-first, local-first productivity app for understanding how time is spent, managing distractions and building better focus habits. It combines a lightweight Rust background agent, a Tauri/React desktop interface and an optional Brave-first browser companion.

**Current Windows source:** desktop **v0.5.3**, agent **v0.5.3**, browser companion **v0.4.0**. This is the source version on `main`, **not a claim that the latest installer has passed CI or been publicly signed**.

[Windows build workflow](.github/workflows/windows-ci.yml) · [GitHub Actions builds](https://github.com/motionedits246-max/digital-discipline/actions/workflows/windows-ci.yml) · [Releases](https://github.com/motionedits246-max/digital-discipline/releases) · [Build and acceptance evidence](docs/BUILD-STATUS.md)

> **Platform scope:** `main` tracks the Windows desktop product. Android and Linux prototypes are developed on separate feature branches. Do not use Windows installation instructions to install those versions.

## Features

- **Activity tracking:** Windows foreground application and idle-time monitoring, local SQLite history, customizable activity categories, and analytics with timelines and activity heatmaps.
- **Distraction controls:** Goals, schedules, browser website/path restrictions, focus sessions, configurable usage budgets, reversible pauses and emergency browser unblock.
- **Floating interface:** A small draggable desktop pill, expandable quick panel, dashboard/workspace, optional separate window and tray controls.
- **To-dos and notes:** Quick-panel to-do list with completion progress and scrollable tasks; notes with headings and text, saved locally.
- **Browser insights:** Optional YouTube Shorts, regular YouTube and Instagram Reels tracking through the Brave MV3 companion. Counts are best-effort observations based on visible playback progression, not proof that every platform video was viewed.
- **Planning and productivity:** Profiles, task-linked focus, persistent work/break cycles, learning controls, local inbox and activity review.
- **Settings and diagnostics:** Customized controls, privacy exclusions, app/site labels, tracking diagnostics, companion delivery visibility, process resource information and manual data export/import.
- **Optional intelligence:** User-controlled app classification and BYOK AI Coach. Core tracking does not require cloud login or an API key.

Some advanced features have partial acceptance coverage; see the [implementation ledger](docs/IMPLEMENTATION-LEDGER.md) and [build status](docs/BUILD-STATUS.md). Source code present does not establish full real-device acceptance.

## Install on Windows

**Recommended for testers:** Check the [latest completed Windows Actions run](https://github.com/motionedits246-max/digital-discipline/actions/workflows/windows-ci.yml) or a [published release](https://github.com/motionedits246-max/digital-discipline/releases). Select the Windows installer artifact from a **successful** run, download/extract it and launch the `*-setup.exe`. GitHub Actions artifacts may require GitHub sign-in and can expire. Verify the included `SHA256SUMS.txt` where available.

The CI workflow creates an **ephemeral self-signed development certificate**. It is *not* a publicly trusted publisher signature and does not guarantee Windows SmartScreen acceptance. A new source commit is not itself a downloadable installer; check the corresponding workflow run.

**Build from source (Windows 10/11):** Install stable Rust with MSVC target, Visual Studio C++ Build Tools and Windows SDK, Node.js 22+, Python 3 and Microsoft Edge WebView2. From the repository root, run PowerShell:

```powershell
git clone https://github.com/motionedits246-max/digital-discipline.git
cd digital-discipline
.\scripts\build-windows.ps1
```

A successful build places the Windows installer, standalone agent and companion ZIP in `release-output/`. For local development, run `.\scripts\dev-windows.ps1`.

### Browser companion setup

1. Install or launch Digital Discipline, then open **Settings & Privacy → Browser setup**.
2. In `brave://extensions`, turn on Developer mode and choose **Load unpacked**; select the bundled extension folder (not the ZIP itself).
3. Authorize the companion extension ID in the desktop application and test its local connection.
4. Enable **Record browser activity** only if you want browser observations; it is disabled by default. For a test, refresh the video tab and play a Short/Reel visibly for several seconds.
5. Keep the same unpacked extension folder when updating to preserve its extension ID and local pending-event queue. Removing an extension may delete its browser-local data.

Browser pairing uses a restricted localhost bridge; normal operation does not require native-messaging registry installation.

## Verification

The Windows build script runs the JavaScript policy tests, database/migration tests, Rust tests, agent runtime tests, frontend build, renderer UI tests and Tauri NSIS packaging. On the `main` branch, the Windows GitHub Actions workflow runs on code pushes (documentation-only changes are ignored), or can be run manually.

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

**Important:** Earlier verification results in `docs/BUILD-STATUS.md` refer to the exact historical builds identified there. They should not be presented as validation of today's v0.5.3 promotion until a matching Windows Actions run succeeds.

## Privacy and data

Activity history stays on the local Windows device by default. The agent records foreground executable basenames, timing/idle observations and opted-in browser activity rather than continuously capturing screenshots, typed text, chats or clipboard contents. Browser recording and AI sharing have separate controls. Saved provider credentials use Windows current-user DPAPI protection; the SQLite history itself is not represented as fully encrypted.

Pause tracking independently from browser restrictions. Local bridge authentication protects against ordinary unauthorized connections but does not protect against malware already running as your Windows account. Export or delete local history using the application's privacy controls.

## Project and contribution

- [Windows desktop](apps/desktop/) — Tauri/React UI
- [Background agent](agent/) — Rust tracking and local services
- [Brave companion](extension/) — opt-in browser playback and restrictions
- [Windows build scripts](scripts/) and [CI workflow](.github/workflows/windows-ci.yml)
- [Original blueprint](docs/blueprint/README.md), [implementation ledger](docs/IMPLEMENTATION-LEDGER.md), and [historical release notes](docs/)

Suggestions, reproducible bug reports and contributions are welcome. If this project is useful, consider **starring the repository** to support development.

[Open the Digital Discipline repository](https://github.com/motionedits246-max/digital-discipline)
