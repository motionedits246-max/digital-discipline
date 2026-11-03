# Digital Discipline v0.3.2 · Browser connection repair and setup

This is **source code for a new Windows build**, not a precompiled installer. The v0.3.1 installer was successfully built on the user's machine. The v0.3.2 source must also be built on Windows before its new desktop/agent changes can be used.

## What the user's screenshots established

- The desktop form wrote Windows pairing registration and displayed "Paired Brave".
- The extension still showed "Agent unavailable / not paired", despite recording consent being enabled.
- Its local retry queue rose from 34 to roughly 99 events. A queue may contain active-tab events as well as playback observations; it does **not** prove all of them are qualifying Shorts plays.
- Source review found a real observability bug: the extension's Sync button unconditionally returned `{ok:true}` without checking the native host reply, and `connectNative()` itself was being treated as a successful connection. It did not preserve native-host failure reasons for the popup.
- The *underlying* browser native-host failure remains undetermined until its actual error is collected on the user's Windows system.

## What's changed

1. `extension/src/background.js`: Connection is only considered healthy after a successful reply from the host/agent. It reports disconnect errors, makes Sync report failure honestly, coalesces overlapping Sync requests, and drains queued events immediately after a successful handshake. Emergency unblock behavior remains intact.
2. `extension/popup.*`: New graphite-color visual hierarchy, custom browser segment buttons, readable consent switches, and one-button connection test with detailed errors.
3. `apps/desktop/src/BrowserSetupWizard.tsx`: Five-step illustrated setup, with folder opener, developer-mode/load-unpacked guidance, pairing, local Windows diagnostics, and video testing.
4. `src-tauri`: Bundle unpacked extension **inside NSIS installer**; expose read-only registration diagnostics and a safe File Explorer opener. Installer cannot silently add a Brave/Chrome extension: browser consent is still required.
5. Windows tray: Open dashboard, Pause, Resume, and Quit. Closing the window hides it. Quit attempts a localhost-authenticated graceful agent shutdown.
6. `scripts/diagnose-browser-pair.ps1` and `scripts/test-native-bridge.mjs` inspect the Windows registration and exercise the native-messaging binary directly.

## Build on Windows

```powershell
cd C:\path\to\DigitalDiscipline
.\scripts\build-windows.ps1
```

This should (if compilation and packaging pass) create `release-output\Digital Discipline_0.3.2_x64-setup.exe`. It is unsigned and may trigger SmartScreen. No installer has been produced by this delivery environment.

After installing the rebuilt app, open **Settings & Privacy → Connect your browser in five steps** and follow the illustrations. You no longer have to extract the extension ZIP manually: press **Open included extension folder** and use Brave → `brave://extensions` → Developer mode → Load unpacked → select that folder. Copy the ID from the extension popup and paste it in Step 3. Approve pairing. Test the connection in Step 4.

### Safety during migration from v0.3.1

**Do not remove the existing extension while it has queued events.** Uninstalling an extension usually deletes its extension-local storage. If possible, preserve the existing extension ID and storage until its queue has synced. Avoid operating two simultaneous copies of the companion extension. If you update the original unpacked folder in place, reload the same extension and preserve its original path to retain its ID and queued state. Back up the old folder before changing any files; this does not back up browser-managed extension storage.

## Diagnostic flow BEFORE/AFTER rebuilding

Run this from the full source directory with PowerShell:

```powershell
.\scripts\diagnose-browser-pair.ps1 -Browser Brave -ExtensionId diakokddlmicacfakghilkajnojloick
node .\scripts\test-native-bridge.mjs Brave
```

The first command is read-only: checks HKCU registry path, JSON manifest, allowed extension origin, executable presence and loopback port. The second actually starts the native messaging host in test mode and sends a framed sync message (no privileged changes). **All checks passing does not prove Brave is connected.** The extension popup's exact native error and the Service Worker console are the next evidence.

If the native test returns `ok:true` but the popup reports "host not found", examine Brave's actual native host registry key / browser profile and restart Brave. If the native test cannot run, inspect the host path, token availability, and whether the agent is running under the same Windows user as Brave. If it returns `agent_unavailable`, start Digital Discipline from Windows Start Menu and retry.

Never paste the contents of `agent.token` or private API keys in bug reports.

## Remaining known limitations

- **Ordinary YouTube watch-page counts** (`/watch?v=...`) are newly implemented as a separate provisional metric, but need Brave end-to-end validation.
- Native Windows/Tauri compilation and real Brave native messaging have not been executed in this Linux source-package environment. The new tray/installer changes require Windows acceptance testing.
- App-level custom dropdowns, the encrypted-file importer, command-palette layout and scrollbar/range styling have source changes; their UI behavior still needs Windows testing.
