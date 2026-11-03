# Digital Discipline v0.3 — 32-feature implementation and acceptance audit

**Scope:** The 32 requested A1–D8 features from `docs/blueprint/docs/01-Feature-Matrix.md` remain the requirement of record. **Source present** means implementation exists in this archive; it **does not** mean the Windows application or extension has been compiled, installed or integration-tested. **Partial** means an acceptance criterion is not fully covered. The source is a *release candidate for developer review*, not a certified production release.

| ID | Feature | v0.3 source coverage | Validation and limitations before acceptance |
|---|---|---|---|
| A1 | Universal app/IDE tracking | **Source present** | Rust foreground executable observation works for arbitrary app basenames in design; physical Windows idle/sleep/lock and high-volume tests pending. |
| A2 | Shorts / Reels count | **Source present, best effort** | Extension observes visible playing video with ≥2 sec progress; verify current site DOM, replay, unique count. |
| A3 | App/content categorization | **Partial** | Local rules, overrides, nonblocking queue, explicit BYOK basename research; no automatic publisher/signature verification or hands-off remote research. |
| A4 | User-defined distraction time | **Source present** | Users select categories; intervals are calculated locally and never labeled intrinsically wasteful. |
| A5 | Video history | **Source present, privacy-limited** | Search platform/video ID, duration, repeats, retention/visibility. Titles not collected by design. Verify UI. |
| A6 | Attention switches | **Source present** | Observed contiguous session transitions with debounce and gap suppression; real-world accuracy needs validation. |
| A7 | Browser comparison | **Partial** | Brave/Chrome/Edge foreground time plus opt-in extension profile ID when attributable. Multiple browser windows/profiles cannot always be correlated with Windows foreground: shows *unattributed* when unknown. |
| A8 | Activity timeline | **Source present** | Session/day view and heatmap. Verify local midnight, DST and idle boundaries on Windows. |
| B1 | Website blocker | **Source present** | Scoped MV3 DNR blocks with TTL, path prefixes and emergency release; live browser checks needed. |
| B2 | Usage-based auto-block | **Source present** | Non-idle observed use since local midnight drives policy; browser must be paired and rules synced. |
| B3 | Short-video cap | **Source present** | Play/unique mode and daily reset; platform observations may undercount. |
| B4 | Automation builder | **Source present** | IF/AND/THEN form, multiple criteria, exceptions, preview and transition audit. Does not guarantee arbitrary nested graph rules. |
| B5 | Strict focus | **Partial** | Browser-only temporary blocks and emergency recovery; no OS-level unbypassable blocking. This is intentional and disclosed. |
| B6 | Partial site restriction | **Source present, best effort** | YouTube/Instagram hide-feed/autoplay adapters; site changes can break selectors. |
| B7 | Shared time budget | **Source present** | Combined domains, native app basenames and chosen categories, union of intervals avoids double counting; test overlapping observations. |
| B8 | Scheduled rules | **Source present** | Weekdays/overnight local time and rule lease boundaries; DST/time-change tests pending. |
| C1 | Floating overlay | **Source present** | Animated pill → quick panel → full overlay + optional popout, black/white dashboard; DPI/accessibility and multiple monitors pending. |
| C2 | Independent agent | **Source present** | Agent runs apart from GUI and opt-in HKCU sign-in registration; release bundle launch unverified. |
| C3 | Analytics dashboard | **Source present** | Heatmap, timeline, drill-down, weekly insights, video stats; completeness depends on observed coverage. |
| C4 | Goal planner | **Source present** | Named app/domain targets, goal progress, archive and focus association, independent of editor brand. |
| C5 | Command bar | **Source present** | Ctrl+K in-app and Ctrl+Shift+D global shortcut in Tauri Windows code; key collision and registration unverified. |
| C6 | Distraction-free web UI | **Source present, best effort** | Reversible hide feed / stop autoplay for two adapters; site-specific live regression needed. |
| C7 | Smart notifications | **Source present** | Opt-in browser notices, 1-hour dedup/cooldown, DND and overnight quiet hours; OS notification delivery unverified. |
| C8 | Privacy control | **Source present** | Pause, separate browser recording consent, exclusions, 1–3650-day retention, export and delete; test on-device data purge. |
| D1 | Doomscroll detection | **Source present** | Rolling qualified-play window, user-adjustable 10–600 sec gap, explainable observed streak. |
| D2 | Goal-aware intervention | **Source present, bounded** | In-goal/out-of-goal hint for sustained known activity with snooze/dismiss choice; no psychological inference or auto-punishment. |
| D3 | Optional AI coach | **Source present** | Explicit aggregate-sharing opt-in, BYOK/provider selection, no persisted key; needs provider integration check. |
| D4 | Distraction prediction | **Source present, conservatively gated** | Local opt-in hourly empirical signal, 21 completed observed days, ≥12 hour samples, leave-one-out Brier vs base-rate, uncertainty bands. Insufficient/unhelpful data yields **no prediction**; no clinical or out-of-dataset calibration claim. |
| D5 | Time recovery simulator | **Source present** | Hypothetical adjustment sliders; never shown as observed savings. |
| D6 | Cross-device tracking | **Partial** | Encrypted **manual** transfer between installations, per-device IDs/deduplication, overlapping interval union and forget; **no continuous sync, Android app, auto-discovery or automatic device pairing**. |
| D7 | Focus difficulty modes | **Source present** | Gentle: timer; Balanced: reversible feed reduction/opt-in notification; Strict: temporary extension navigation deny. All have explicit emergency recovery. |
| D8 | Weekly insights | **Source present** | Week comparison, observed switches, time categories, hourly split across actual boundaries, attribution caveats; verify locale/timezones. |

## Acceptance checks to run on the owner's Windows PC

1. **Build:** `./scripts/build-windows.ps1` produces unsigned NSIS installer, agent executable and extension ZIP. Test all scripts in a fresh Windows 11 user profile. Do not claim this passed solely because source exists.
2. **First launch:** Pill appears and expands; Ctrl+Shift+D works or reports collision; popout/drag/HiDPI/multi-monitor remain usable.
3. **Tracking:** Try Cursor, VS Code, Antigravity, JetBrains and an unknown editor; check sleep, idle, lock, resume, and unknown-category accuracy; confirm missing data is never replaced with imaginary data.
4. **Browser:** Pair at least two profiles as desired, independently enable recording, check normal YouTube watch vs `/shorts/`, Instagram Reels, URL path block, feed hiding, native bridge loss, rule TTL and emergency recovery.
5. **Automation:** Create mixed app/site/category shared budget, an exclusion, a scheduled rule and a configurable rolling streak; check live preview and rule-transition audit.
6. **Focus:** Verify Gentle, Balanced, Strict differ and remain reversible; goal hint + snooze/dismiss; no OS-wide shutdown or tamper-proof claim.
7. **Intelligence:** Set selected distraction categories, confirm opt-in forecast hides until enough history, and Coach/AI classification remain optional while the agent keeps observing with the internet disconnected.
8. **Devices:** Export a bundle with a unique secret, import into a separate installation, verify incorrect secret fails, re-import deduplicates, overlap is not double-counted, forget erases imported device sessions. Confirm this is manual and no server is contacted.
9. **Privacy/security:** Pause activity recording but keep rules active, test exclusions/retention/export/delete, audit data fields, check no API keys appear in DB or logs.
10. **Performance/release:** Run at least 24 hours on a low-RAM laptop, inspect CPU, RAM, SQLite growth and crash recovery; run accessibility/keyboard/security review and sign release if distributing to consumers.

## Remaining non-negotiable release blockers

- Rust/Tauri and React production compilation, native Windows installation, MV3 pairing, current YouTube/Instagram behavior and performance have **not** been executed in the provided environment.
- D6 does **not** fulfill automatic cross-device sync; A3 lacks publisher/signature evidence and A7 cannot fully disambiguate simultaneous browser profiles; Strict is limited to paired browsers. These are gaps against the broad original vision, not hidden completion claims.
- Installer is unsigned. Do not redistribute as a verified release before completing the acceptance checks.
