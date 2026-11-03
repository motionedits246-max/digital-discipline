# Digital Discipline v0.3.3 — Direct localhost bridge (Brave)

This version **removes native-messaging registration from the normal browser connection path**. The MV3 extension talks to the local Windows agent on `http://127.0.0.1:48761` using host permission and an explicit allowlist of extension IDs. The Windows agent binds only to IPv4 loopback, never your local network. Only the desktop app (authenticated through its existing agent token) can add or revoke an Extension ID. Browser HTTP callers must present a valid `chrome-extension://<id>` Origin matching the allowlist, and CORS never permits regular websites. Like any user-level localhost service, this is not protection against malware running under the same Windows account.

## Upgrade without losing the 99+ queued events

1. **Do not uninstall the existing Brave extension or clear extension storage.** Do not install a duplicate extension from a new folder.
2. Extract v0.3.3 source over the existing project root (or use the full source archive). Fully exit old Digital Discipline from the tray and terminate its old agent if still running; the new agent must own port `48761`.
3. Run `./scripts/build-windows.ps1` in the project root. Install the resulting `release-output/Digital Discipline_0.3.3_x64-setup.exe`. An unsigned installer may trigger SmartScreen; use only a trusted local build.
4. Find the current unpacked extension folder in `brave://extensions` > Details. If that existing folder IS the project's `extension` directory, it already has the new files; just press Reload. If it is elsewhere, run:
   ```powershell
   .\scripts\upgrade-brave-extension.ps1 -ExistingExtensionFolder 'C:\path\to\your\existing\extension-folder'
   ```
   This backs up the unpacked files, updates them in place, and does not intentionally alter extension ID or browser-managed queue data. The extension's new manifest asks for localhost permission; accept if Brave prompts.
5. Open installed Digital Discipline > Settings & Privacy > **Connect your browser**. In step 3 choose Brave, open the extension popup and press **Connect directly & sync events**. Go back to the desktop **Waiting for approval** list, press **Find extensions**, then click **Approve** next to its detected ID. Manual ID paste remains available if discovery fails.
6. The **Paired browsers** section must list that ID. Before connecting, it says **awaiting first connection**. After a successful browser request, press Refresh to see **Last connected**.
7. In Brave's extension popup, ensure **Record browser activity** is enabled and click **Connect directly & sync events**. It should show **Connected to local Windows agent** and queued count should start falling. Events older than 24 hours are not accepted by the agent.
8. Play three distinct Shorts for ≥5 seconds and two ordinary YouTube videos for ≥5 seconds; check separate counts under Video Intelligence.
9. To disconnect a browser, press **Unpair** in the desktop's paired list. Further event/rule requests will be denied by the agent.

## What this does *not* do

- It does **not** sniff HTTPS packets, install a TLS certificate, scan unrelated browser history, or read browser cookies. URL paths in HTTPS are encrypted; counting visits to `/shorts/` from packet captures is not a reliable way to determine playback.
- It does not silently install the unpacked extension in Brave. The user still approves **Load unpacked** once. A future browser-store distribution could streamline installation further.
- A visit to a Shorts URL does not necessarily equal a play; the existing content script counts observed visible playback, with a two-second qualifying threshold.

## If the popup still says disconnected

- `extension_not_authorized_open_desktop_setup`: check the current ID in the Paired browsers list; authorization is separate for each Chrome/Brave profile with a different ID.
- `Failed to fetch`: close the old agent and ensure the v0.3.3 Windows agent is running at `127.0.0.1:48761`; check `brave://extensions` for localhost host access.
- `HTTP 404` or `Cannot POST /bridge/sync`: you are still connected to the old agent. Fully Quit and relaunch the updated app; check Task Manager for duplicate `digital-discipline-agent.exe`.
- CORS/Origin errors: copy the **service worker** Console error; never share tokens, private browsing history, or API keys.

**Verification:** source-level JavaScript tests cover reconnects, queued events and regular-watch separation. Rust integration and actual Brave behavior still need the Windows build and user test; this ZIP is source, not a precompiled signed installer.
