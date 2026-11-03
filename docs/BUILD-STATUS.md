# Engineering validation record — v0.3 review candidate

**Created:** 2026-11-03. **Deliverable:** complete current source tree with bundled original Markdown blueprint and Windows installer-building workflow. **Not a prebuilt Windows `.exe`.** This environment cannot invoke `rustc`/`cargo` and does not have the frontend's npm packages, so a compiled native artifact and functional Windows/browser certification are unavailable.

## Checks actually executed here

| Gate | Result | Proves / does not prove |
|---|---|---|
| `node --test tests/*.test.mjs` | **14 passed** | 8 browser URL/DNR tests, 3 notification-policy tests, 3 AES-256-GCM bundle tests. Does not prove real Chromium. |
| `python tests/migration-contract.py` | **3 passed** | Executes extracted SQL from `main.rs` / `features.rs` through schema v7 including legacy upgrade, plus device table uniqueness. Does not prove Rust compiles. |
| `node --check` on extension scripts | **Passed** | JS parse only. |
| Global TypeScript `transpileModule` diagnostics | **0 syntax diagnostics** | Syntax only; no `tsc --noEmit`, React typechecking or Vite bundling. |
| JSON manifest/config parsing | **Passed** | Valid JSON, not complete runtime validation. |
| Windows Rust/Tauri build and execution | **Not run** | Required for any production-release claim. |
| Chromium real extension and OS integration | **Not run** | Required for accurate enforcement and counting claims. |

## v0.3 implementation delta over v0.2

- D4: optional local historical prediction with sample-size and hold-out calibration gates (`agent/src/prediction.rs`). Never assert forecasting skill without evidence.
- D6: manual AES-256-GCM encrypted bundle import/export (`apps/desktop/src/secureBundle.mjs`, `agent/src/devices.rs`); bounded, repeatable device-scoped ingestion, deduplication and overlap-unioned reporting. This is not automatic sync.
- B4/B7: user-editable app/category shared budgets, rule exceptions, dry-run preview, and local audit of policy transitions (`agent/src/analytics.rs`, `agent/src/main.rs`, Automations UI).
- C7: opt-in notification DND, quiet hours including overnight windows, cooldown (`extension/src/notifications.js`, popup).
- D1: configurable consecutive-play gap. D2: user snooze/dismiss for goal hints with audit.
- C5: requested system-wide Ctrl+Shift+D invocation registered using Tauri global shortcut on Windows; conflicts fail open, Windows behavior not tested.
- A5: searchable recorded video source IDs; A6: gap-aware switch counting; A7: best-effort per-profile telemetry labeling and up to 16 paired extension origins/browser; D8: exact local-hour interval splitting, with coverage caveats.
- SQLite schema upgraded to v7. Current migrations use `PRAGMA user_version` and preserve older sessions via `ALTER TABLE`.

## Explicit limitations and precautions

1. **No unsigned or signed installer is delivered**; `scripts/build-windows.ps1` and `.github/workflows/windows-ci.yml` are provided to produce an **unsigned** NSIS installer on Windows. Compiler and integration defects may be found by that build.
2. **Not a completed full 32-feature product**: some features are intentionally limited or partial; see `docs/REVIEW-ACCEPTANCE.md` for a requirement-by-requirement audit.
3. Per-profile Windows foreground attribution is best effort and cannot prove identity if concurrent browser windows/profiles confuse the extension/native process association. Missing observations remain unattributed.
4. Cross-device history uses a manual user-operated encrypted file; no automatic background sync. Do not reuse weak passphrases or send an unencrypted raw agent API export to others.
5. The optional forecaster is empirically gated and may be unavailable indefinitely. Its calibration check does not establish scientific validity or generalization.
6. The agent API is restricted to authenticated loopback; same-user malicious code is outside its threat boundary. Extension policy leases expire if the bridge/agent fails. Strict Focus is browser-only and reversible.
7. Browser video watch-time is estimated from qualified visible media progression only. YouTube/Instagram markup and playback APIs are changing external dependencies and must be exercised on target browsers.
8. App classification is basename-based unless the user explicitly authorizes per-subject AI processing; signature and publisher verification are not implemented. No window titles, keystrokes, clipboard, screen capture or document paths are collected.
9. Export is capped to 50k local sessions, and manual device exchange to 1k recent sessions. Neither is a comprehensive unlimited database backup.
10. 24-hour CPU/RAM/power benchmarks, sleep/wake, high-DPI accessibility, installer uninstall, startup, and real-model/provider tests have not been performed.

## Windows commands

Open PowerShell in the extracted repository root. Install Rust MSVC, Visual Studio C++ Build Tools + Windows SDK, Node.js 22+, npm and WebView2 first.

```powershell
.\scripts\build-windows.ps1
```

Only **if the script exits successfully** look under `release-output/` for the unsigned Windows installer, agent executable and `extension.zip`. For developer mode use `./scripts/dev-windows.ps1`. To verify the local tests separately:

```powershell
$testFiles = @(Get-ChildItem -Path tests -Filter '*.test.mjs' | ForEach-Object { $_.FullName })
node --test $testFiles
python tests/migration-contract.py
cargo test -p digital-discipline-agent
cd apps\desktop
npm install
npm run build
```

Do not interpret passing unit tests as evidence of a successful Windows release. Sign only after the owner runs the acceptance checklist and security review.
