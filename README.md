# Digital Discipline v0.3.2 — browser pairing & onboarding repair

**Build status:** Source release only. Windows compilation/integration has NOT been verified for this revision. The previous v0.3.1 build was successfully compiled on the user's machine.

**Browser setup changed:** The rebuilt Windows installer includes the Brave extension files. Open Settings & Privacy → **Connect your browser in five steps** for a guided visual setup. On the first step choose **Open included extension folder**, then follow the onscreen instructions. Browser security still requires the user to explicitly load the unpacked extension once.

**Before uninstalling an older extension,** ensure its queued events have synchronized; removing it could erase unsent events.

For the fix, diagnostics and limitations, read [`docs/PAIRING-REPAIR-v0.3.2.md`](docs/PAIRING-REPAIR-v0.3.2.md).

---

# Digital Discipline — v0.3 Windows-first expanded review candidate

**A local-first desktop activity and distraction-control application, implemented from the included Markdown blueprint.** This release is a substantial **source-code review candidate**, not a verified, signed consumer installer. The original blueprint and all 32 selected feature definitions are preserved in `docs/blueprint/`.

## In the project

- **Rust Windows agent** (`agent/`): foreground executable and idle observations; bounded reconciled sessions; encrypted-in-transit optional vendor requests; local SQLite; privacy exclusions/retention/delete/export; deterministic block policy evaluation; app-category overrides; goals; focus sessions; opted-in classification jobs, on-demand BYOK Coach and weekly insights. Nothing requires a cloud login to track or block.
- **Tauri v2 desktop application** (`apps/desktop/`): monochrome pill, quick panel, expanded workspace and pop-out; timeline and heatmap; browser/device health; user-defined distraction categories; goals/focus modes; rule builder; short-form gallery and separate regular YouTube playback counts; analytics; consent, recovery and pairing controls; in-window Ctrl+K and Windows-wide Ctrl+Shift+D command palette.
- **Brave-first MV3 extension** (`extension/`): browser active-tab domain observation and visible-play progression for YouTube Shorts, regular YouTube watch pages and Instagram Reels; reversible DNR route restrictions; hide-feed / stop-autoplay adapters; opt-in notifications; emergency unblock, bounded offline queue and expiring policies. Supports per-profile UUID telemetry where attributable; compatible in principle with Chrome/Edge, not independently certified in each browser.
- **Windows PowerShell scripts** (`scripts/`) for development, manual host pairing, optional sign-in startup, and an automated release build (`scripts/build-windows.ps1`). A GitHub Actions Windows build pipeline is provided; it has not been run from this environment.

## Prerequisites and Windows review

Use **Windows 10 or 11**, **stable Rust MSVC**, **Visual Studio C++ Build Tools + Windows SDK**, **Node.js 22+**, **npm**, and **WebView2**. Installing Rust/npm dependencies requires internet access on your PC. No GPU, cloud server, account or API key is needed for core features.

Extract the ZIP and open PowerShell at the repository root:

```powershell
.\scripts\dev-windows.ps1
```

That script runs tests, compiles the Rust agent, starts the agent independently, installs frontend dependencies and launches the Tauri desktop in development mode. Click the small floating pill, expand the quick panel and choose **Open workspace**.

For a Windows **unsigned** installer build plus separately packaged extension:

```powershell
.\scripts\build-windows.ps1
```

Look inside `release-output/` for the installer `.exe`, independent agent `.exe`, and `extension.zip`. The script must succeed before treating any installer as valid. The Tauri installer bundles the native agent as an application resource; launching the app starts the agent if it is not already running. **It is not automatically registered at Windows startup**: opt into this explicitly in Settings & Privacy → Independent agent. This leaves normal user control and avoids surprise persistence.

### Pair Brave

1. Navigate to `brave://extensions`, enable Developer mode, choose **Load unpacked**, and select the extracted `extension/` directory (not the ZIP).
2. Copy the 32-character extension ID. Open the desktop Settings & Privacy screen, enter it in **Pair the native browser extension**, choose Brave and click **Pair**. For source development without a packaged Tauri agent, use `./scripts/register-native-host.ps1 -ExtensionId YOUR_ID -Browser Brave` from PowerShell instead.
3. Reload the browser extension. Open its popup; choose Brave; opt into **Enable local browser activity recording** if you want browser telemetry. Default is OFF. Browser rules sync regardless of recording consent.
4. Visit a YouTube Short or Instagram Reel and **play visibly for at least 2 seconds** to qualify a play. Replays and unique videos are tracked separately, best effort. Verify using Video Intelligence.
5. Create a rule scoped to `www.youtube.com` with `/shorts/` if you want to restrict Shorts while leaving normal YouTube watch pages. For a reversible override, open the extension or blocked interstitial and choose **Emergency unblock (30 min)**.

Chrome/Edge can be paired the same way with the appropriate browser selection and separate extension ID. Native host registration is **per Windows user**, does not need administrator privileges, and is reversible. Avoid pairing unsupported browser profiles without verifying their native-host registration independently.

## Feature scope, modes and review readiness

**32 selected feature IDs A1–D8 are tracked individually in `docs/REVIEW-ACCEPTANCE.md`.** Code and UI are present for a majority of features, but **some are partial**. Source present does not mean runtime acceptance on Windows.

**Desktop presentation modes:** tiny floating pill; expandable quick panel; full black-and-white analytics/dashboard overlay; optional pop-out window. The UI provides Ctrl+K command search and a registered Windows global Ctrl+Shift+D hotkey (registration needs real Windows testing).

**Focus modes:** Gentle is timer/goal tracking only; Balanced applies soft feed reduction/opt-in notification; Strict applies reversible **browser** navigation blocks for user-selected sites, and always preserves an emergency release. Strict does not lock Windows or tamper-proof other applications.

**Rule and notification modes:** temporary or indefinite browser block; site/path restriction; daily minutes; Shorts/Reels qualifying plays or distinct videos; shared app/site/category usage budget; configurable doomscroll streak; local scheduled windows; notification-only/soft controls where supported; exception lists, dry-run preview and transition audit. Browser notification consent, DND and quiet hours are separate from browser recording consent.

**Insights and intelligence:** local activity heatmap and session timeline; per-browser, best-effort per-profile attribution; video ID search/history; attention transitions; actual hour-split weekly distribution; opt-in local historical prediction (minimum observed sample and hold-out check); time-recovery what-if slider; human-initiated BYOK Coach and classification requests.

**Device mode:** optional encrypted, password-protected, **manual** import/export of at most 1,000 recent local sessions; per-device deduplication and overlapping-session union, with forget controls. **This is not continuous sync and does not include an Android client.**

**Unfinished or partial acceptance:** verified publisher/signature metadata (A3), complete multi-profile native attribution (A7), OS-wide strict focus (B5), automatic continuous cross-device sync (D6), and native installation/testing/performance/accessibility validation. See `docs/BUILD-STATUS.md` and the full matrix before reviewing.

## Data & security properties

- Database: `%LOCALAPPDATA%\DigitalDiscipline\discipline.sqlite3`. Local API: `127.0.0.1:48761`, authenticated by a random bearer token in the same per-user directory. This protects against ordinary unauthenticated browser calls but is **not** a security boundary against malware running as the same Windows user.
- Collector stores executable basename, interval/idle and optional opted-in domain. No titles, full paths, screenshots, clipboard, typed keys, chats, video titles or transcripts. The extension observes short video IDs and visible playback progression; it does not guarantee platform-complete views.
- Exclusions are honored for **new** observations. Purge previously collected activity explicitly. History retention defaults to 90 days; first/next hourly cleanup applies new retention choices. Pausing tracking **does not silently switch off active blocking rules**.
- Local app-classification queue is automatic, but **remote research happens only when you select a subject and approve an individual request**. Provider credentials are accepted for one request and are not written to SQLite/browser storage. Category output is enum/schema validated and basename-only confidence is capped. Do not use an LLM label as proof of publisher identity.
- The Coach sends only the day’s aggregate milliseconds and short-play count to your selected model after explicit consent. API providers may charge or retain prompts under their own policies; choosing Ollama keeps requests on local loopback, assuming your Ollama installation is trusted.
- Rule leases expire locally after a maximum of three minutes without successful synchronization. Emergency bypass lasts 30 minutes. This is a self-directed wellness/productivity tool, not an anti-tamper product or privileged system blocker.
- Sample UI preview is clearly labeled and does not create user data.

## Local verification commands

```powershell
$testFiles = @(Get-ChildItem -Path tests -Filter '*.test.mjs' | ForEach-Object { $_.FullName })
node --test $testFiles
python tests/migration-contract.py
cargo test -p digital-discipline-agent
cargo build --release -p digital-discipline-agent
cd apps\desktop
npm install
npm run build
npm run tauri:build
```

**Status at package creation:** 14 JavaScript unit tests and 3 SQLite migration tests passed; extension JS syntax, TS/TSX syntax transpilation and JSON manifests parsed. **Rust/Tauri compilation, frontend React type-check and production bundle, Windows runtime collector/native bridge, Chromium extension integration and installer were not run in this environment.** It lacks Rust toolchain and locally installable frontend dependencies. A Windows build and the 32-feature owner acceptance checklist are required before anyone can call this a final consumer version. See `docs/RELEASE-NOTES-v0.3.md`.

## Recovery / uninstall

`./scripts/unregister-native-host.ps1 -Browser Brave` removes the source-development native host association. `./scripts/disable-autostart.ps1` removes source-mode auto-start; the packaged desktop has its own Settings toggle. Disable/remove the extension at any time to suspend browser restrictions. Use Settings → Delete activity to purge raw history, then close the app and uninstall it in Windows Settings. The application **does not automatically delete user-owned SQLite data upon uninstall**; remove `%LOCALAPPDATA%\DigitalDiscipline` manually only when you explicitly want all local data and the auth token erased.

## Latest local test release

See [v0.3.2 changes and upgrade safety](docs/RELEASE-NOTES-v0.3.2.md) and [Brave quickstart](docs/TEST-BRAVE-v0.3.2.md). This is an unsigned source build requiring Windows compilation and real Brave acceptance testing; do not treat it as a verified production release.

## v0.3.3 Brave direct-local upgrade

Native messaging is no longer needed for the companion's normal operation. After building and installing v0.3.3, preserve your existing unpacked extension and its `chrome.storage.local` queued events, update the folder **in place**, reload it in `brave://extensions`, then authorize its ID under Settings & Privacy → Browser setup. The new **Paired browsers** panel displays authorizations and last handshake time. No Windows registry native messaging host is required. See `docs/TEST-DIRECT-LOCAL-v0.3.3.md` for detailed upgrade and debugging steps.
