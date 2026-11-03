# Digital Discipline v0.3.3 — Direct localhost connection

## Root cause addressed

The previous installed Brave companion reported **Specified native messaging host not found** even after Windows registration. v0.3.3 removes native messaging from the normal connection flow and uses a local HTTP transport instead, with explicit per-extension allowlisting. This bypasses the failing registry lookup, but still needs Windows + Brave end-to-end validation.

## Changes

- New `agent/src/bridge.rs` restricts the loopback bridge to explicit `chrome-extension://ID` origins; regular websites are rejected. Only the authenticated desktop API can enroll and revoke extensions. The list of authorized IDs and last contact time is retained in the local SQLite database.
- The MV3 background service worker forwards `event` and `sync` directly to the agent at `127.0.0.1:48761`. It retains queued events if offline and retries after authorization. The extension manifest no longer requests `nativeMessaging`, but adds a narrow localhost host permission.
- The desktop Setup Wizard displays **Paired browsers**, last connected time and **Unpair**, and no longer requires registry diagnostics. Browser setup remains consent-based.
- An in-place PowerShell update helper preserves the existing unpacked folder path so existing Chrome extension ID and browser-managed queue data can survive an upgrade.
- The Video History gallery now correctly labels regular YouTube watch videos instead of calling them Reels.
- Existing tray controls, dark monochrome UI improvements, custom selects, file uploader and command palette fixes from 0.3.2 remain.

## Validation limits

JS tests were run under Node against a simulated extension + loopback response. Database migration test uses Python SQLite. This environment does not have Rust MSVC toolchains or a running Brave browser and therefore **no Windows installer or Brave end-to-end success is claimed**. Consult `docs/TEST-DIRECT-LOCAL-v0.3.3.md` for Windows build and acceptance testing.
