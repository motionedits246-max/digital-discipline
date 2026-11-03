# Brave pairing quickstart for local testers

**Keep the old extension installed while it holds unsent events.** Upgrading its files at the same unpacked directory path and using **Reload** preserves its extension ID; switching folders or removing it may lose queued data.

1. Build on Windows: `./scripts/build-windows.ps1` in PowerShell.
2. Stop old Digital Discipline agent/UI (Task Manager), or reboot Windows, then install the newly generated NSIS EXE and launch it.
3. If first install: Open **Settings & Privacy → Browser setup wizard**, open bundled extension folder, `brave://extensions` → Developer mode → Load unpacked. If updating existing extension: simply Reload the existing unpacked extension; do not install a duplicate.
4. In setup wizard paste the existing 32-character ID, click Pair and check the green Windows diagnostics. Then click Test connection & sync events in the companion popup.
5. If popup shows connected, allow events to drain. Test three Shorts and two regular watch videos, then check Video Intelligence. Browser activity recording must be enabled voluntarily.
6. If disconnected, the popup now displays a specific error. From project source:
   - `./scripts/diagnose-browser-pair.ps1 -Browser Brave -ExtensionId <your-extension-ID>`
   - `node ./scripts/test-native-bridge.mjs Brave`
   Send only the non-sensitive errors, not your token or private history.
7. Test tray Quit, dropdowns, upload picker, sliders, command palette, blocking modes and emergency release.

**Connection errors**: “host not found” → registry/manifest location; “Access forbidden” → allowed ID mismatch; “agent_unavailable” → background agent not running; “agent_token_missing” → app/host launched in wrong user session or agent not initialized; “native_timeout” → native host crash, timeout or stalled agent. These categories are diagnostic aids, not guarantees.
