# Digital Discipline v0.3.2 — Local Brave test build

This is an **unsigned Windows source release**. The developer must compile it on Windows. It has not been installed or exercised in Brave in this delivery environment. This revision targets issues observed in the actual v0.3.1 Windows screenshots and build logs; it does not imply all 32 roadmap features have been fully verified.

## What changed

- **Browser pairing:** Real native host reply is required before the popup says connected; connection failure is displayed explicitly. Sync no longer reports success merely for opening a port. The native messaging agent stays a console-subsystem executable so stdin/stdout byte pipes can be inherited properly; background starts it with `CREATE_NO_WINDOW`. Missing token and agent HTTP error return readable protocol errors instead of silent failures. Windows pairing doctor and native-protocol smoke test are included.
- **Pending events:** queue limit raised from 150 to 2,000 events; successful handshake drains acknowledged events. Individual events remain deduplicated by UUID on the agent. Existing queued data is not cleared during the update if the *same installed extension identity* is preserved.
- **YouTube:** Adds observed regular `youtube.com/watch?v=...` playback, explicitly separate from YouTube Shorts and Instagram Reels totals. Uses visible/foreground playback progression, not a page view, and does not collect titles, transcripts or full browsing URLs in the stored database. On YouTube SPA route changes, the extension sends the route at the time each observation was recorded.
- **Dashboard:** 3-second updates in the workspace / 5-second pill polling. Separate regular YouTube metrics in Video Intelligence. Still polling, not a pushed real-time socket stream.
- **Desktop UI:** Replace native `<select>` controls with portal-based, scroll-safe custom menus with keyboard navigation; fix command palette input vs scroll results; style range slider, scrollbar, and file upload with drop target.
- **Companion popup:** layered graphite contrast, native connection error, segmented browser selection, readable switches, consent labels and control spacing.
- **Installer & onboarding:** bundle MV3 companion extension in the NSIS resource directory and offer an illustrated five-step setup wizard with folder opener, pairing check and event testing. Chromium/Brave *still require a deliberate user action* to install an unpacked extension; silent forced installation is not supported.
- **System tray:** Open Dashboard, Pause, Resume and Quit. Closing the window hides it. Quit asks the local agent to shut down through authenticated loopback HTTP.

## Build on Windows

```powershell
cd C:\path\to\DigitalDiscipline
.\scripts\build-windows.ps1
```

When compilation succeeds, check `release-output\Digital Discipline_0.3.2_x64-setup.exe` and `release-output\extension.zip`. Installation requires no developer mode in the Windows app, but Brave Developer Mode is needed to **Load unpacked**.

## Upgrade safely from the local v0.3.1 project

**IMPORTANT: you previously had about 99 queued unsent events. Do not remove the existing Brave extension until those have synced.** To retain the current extension ID and local queue, copy the v0.3.2 source files **over your existing DigitalDiscipline project directory** (especially `extension`), then click Reload for that existing extension in `brave://extensions`. Keep the same unpacked extension folder path. Avoid using “Load unpacked” again for a different directory, because it may create a second extension identity and a separate empty queue.

Close the old desktop app and end any older `digital-discipline-agent.exe` from Task Manager or restart Windows before testing the new version. The old agent might still be listening on localhost:48761; the new desktop should launch the v0.3.2 agent. Install the new NSIS installer, start from Windows Start Menu, open Settings & Privacy → browser setup wizard, then **Pair** the existing extension ID again and press **Test connection & sync events** in the existing extension popup.

If still disconnected, copy the **exact error text** now shown in the extension popup. Run:

```powershell
.\scripts\diagnose-browser-pair.ps1 -Browser Brave -ExtensionId YOUR_EXISTING_32_LETTER_EXTENSION_ID
node .\scripts\test-native-bridge.mjs Brave
```

Do **not** paste `agent.token`, API keys, database contents or private browsing data in support messages.

## Acceptance checklist

1. Pairing popup changes to *Connected to Windows agent* **only when an actual reply arrives**; queued events fall to zero (or at least decrease) after sync.
2. Play 3 distinct YouTube Shorts for ≥5 seconds each: Shorts count and unique count rise; repeat-play semantics are observed rather than inferred from visits.
3. Play 2 ordinary YouTube videos (`/watch?v=`) for ≥5 seconds: **regular video count** increases separately; Shorts total stays unchanged.
4. Block `/shorts/` only, verify ordinary `/watch` continues, test Gentle/Balanced/Strict and emergency release.
5. Desktop new dropdowns stay inside screen bounds in scroll containers, Ctrl+K search input does not overlap results, slider and file uploader have consistent appearance.
6. Close desktop window: tray remains; choose Open, Pause, Resume; Quit closes app and managed agent. Relauch once to check no duplicates.
7. Restart Windows and verify manually configured startup behavior and app tracking.
8. Inspect Resource Monitor / Task Manager for CPU/RAM regressions while viewing and hiding the overlay.

## What this build does NOT establish

No real Windows Tauri compilation, installer packaging, Brave native handshake, live player integration, CPU/RAM regression, or all-feature end-to-end acceptance has run here. Build errors and site/player-specific counting gaps remain possible. This version is a **candidate for your next Windows test**, not a production-certified final release.
