# Current build and acceptance status

Repository: `motionedits246-max/digital-discipline-windows`. Branch: `feature/master-spec-complete-2026-11`. Main remains unchanged.

Current source: desktop/agent **0.4.3**, companion **0.3.9**, protocol **1.0**, schema **14**. Windows CI for this source is pending. The full roadmap remains open in the [implementation ledger](IMPLEMENTATION-LEDGER.md).

## 0.4.3 changes and local verification

Custom confirmation dialogs replace native browser prompts. Closing the workspace animates into the pill. A compact service overlay offers a category without taking focus and disappears after about six seconds when unattended; timeout never approves an LLM call.

Saved/manual categories and small local rules run before Windows File Description/Product Name inference. Optional public-root metadata and DuckDuckGo summaries have a shared 20-request daily cap. Public lookups are off until opted in and never send full URLs or window/executable paths. Pollinations and the other LLM providers need a saved key, classifier consent and individual service approval. A configuration change invalidates pending service approval; uncertain approved results retain the existing bounded review/retry limits.

The companion can update from the installed desktop bundle and reload the same extension. Browser setup supports registering an existing custom folder; user data, profile ID, consent and pending events stay in browser storage. Owned-file rollback and dependency packaging checks cover the update. A one-time Reload of 0.3.8 is necessary to get its new button; removing the extension is unnecessary.

| Check | Current evidence |
|---|---|
| JavaScript policy/queue/update tests | 53 passed locally |
| Rust agent | 68 passed locally on Linux; Windows pending |
| SQLite legacy migrations | 8 passed |
| TypeScript/production frontend | Passed |
| Renderer interactions | 12 listed; new Windows run pending |
| Windows installer | Pending for 0.4.3 |

No live provider key, owner browsing history or owner-installed Brave update was used in these local checks. Actual Windows app-description extraction, native popup placement/DPI/focus, installed browser reload and long resource soak still require runtime acceptance. Executables from CI use self-signed development certificates, without Windows publisher trust.

## Last verified installer — 0.4.2

[Run 42370611230](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42370611230) succeeded for `ea1c89a65acc00f65497bd22240461ae7ff21951`: 51 JavaScript, 56 Windows Rust, 8 migration, 1 privileged-route and 7 Edge renderer checks, expanded real agent lifecycle/workflows and Tauri/NSIS packaging.

[Complete 0.4.2 bundle](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42370611230/artifacts/13573294448). Installer SHA-256: `a6281a93e357956372ce791780d29ac961b50d86b55cb75d981a893015ff0dc2`. Artifact ZIP SHA-256: `5d0f7f6f5276c744252fd64551f7791def69357c6e903061b49a9cfe25bc343d`. Downloaded bytes and all internal checksums were independently verified on 8 November.

Full encrypted restore, trusted application updating, automatic device sync, earned entertainment, planner/correction/passive-learning and other expanded roadmap items remain open. Native application intervention is warning-only. Schema 14 is additive; older agents reject newer schemas instead of modifying them.
