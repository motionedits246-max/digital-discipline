# 05 — Windows activity tracking and session accounting

## 1. Scope

Detect which **foreground application** the user is interacting with, whether the user is idle, and whether a supported browser tab has detailed data. This is not a keylogger and does not require recording screenshots. Examples include Cursor, Antigravity, VS Code, IntelliJ, PyCharm, terminals, Figma, games, file explorers and unknown executables.

## 2. Windows observation sources

| Signal | Candidate native mechanism | Purpose / caution |
|---|---|---|
| Foreground window | `GetForegroundWindow`, `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` | Know top-level foreground changes; event-hook thread/message loop required |
| Window → process identity | `GetWindowThreadProcessId`, process image metadata | Executable, install path metadata, publisher if feasible |
| Display/window metadata | `GetWindowText` only if user permits | Window titles can contain private filenames, pages or conversations; default minimal storage |
| Input idle | `GetLastInputInfo` | Session-specific last-input time, not per-app or a universal engagement proof |
| Lock/unlock/session switch | WTS session notifications | Stop counting locked-session intervals |
| Suspend/resume | Windows power/session notifications | Prevent sleep from becoming phantom usage |
| Process lifecycles | Process information + caches | Avoid repeated expensive file signature queries |

Authoritative Windows API references are listed in `15-Research-Sources.md`. Foreground callbacks should enqueue minimal events quickly; no network or disk-heavy work inside hook callbacks.

## 3. Stable application identity

Never classify all `Code.exe`/all unknown windows as coding. Normalized app identity may consider:

1. Verified process executable path and file identity (privacy-safe hash/normalized ID where appropriate).
2. Publisher/signature/company, product name, package identity (MSIX), installed-app record.
3. Parent process and known terminal/IDE relationships (with caution).
4. Local known-app lookup registry and **user manual override**.
5. Optional asynchronous WinGet/public metadata and AI JSON fallback.

Data model separates `app_identity`, `app_instance` and `window_session`: one application may open many windows, have helper subprocesses or rename product metadata over versions. Avoid falsely labeling command prompt as Development solely because it is a terminal; use a user rule or context evidence.

## 4. Session state machine

**States:** ForegroundActive, ForegroundIdle, BackgroundPlayingMedia (separate measure), Locked, Sleeping, AgentOffline, Excluded, Unknown.

Proposed progression:

- Foreground changes → close previous foreground segment at switch time; open new segment for normalized app ID.
- Last input exceeds configurable threshold → split foreground segment into active and idle portions; threshold default should be empirically chosen and user configurable.
- Media playback signal while no input → record playback separately, but do not count two active minutes in a minute of elapsed time.
- Lock/sleep → close active segment and mark state; resume creates new segment with explicit gap.
- Agent offline/reboot → record coverage gap, not zero and not extrapolated foreground duration.

Use UTC wall clock for persisted timestamps and a monotonic reference to calculate durations while running. Handle changed wall clock, daylight saving and timezone relocation explicitly. Lost source events require bounded inference and “estimated” flag.

## 5. Anti-double-count invariants

- Within a user session, overlapping app foreground sessions for the same moment are not allowed in **primary foreground totals**.
- Browser tabs may be open in parallel, but only foreground browser/window/tab contributes to *foreground website time*.
- Media may play in background; playback time is a **separate metric** and must never be blindly added to foreground totals.
- The tracker UI must not be misclassified as the user's intended Development focus. Record it as App Management/Neutral or exclude it by policy.
- If two monitors show a browser and IDE, keyboard focus still yields one foreground active app. The UI must disclose this limitation rather than infer gaze/attention.
- Multi-user Windows sessions are tracked only with appropriate active session scope; no cross-user monitoring.

## 6. Generic IDE and task categorization

The product detects installed/running applications generically. Curated mappings may recognize Cursor, Antigravity, VS Code, IntelliJ, Visual Studio, Sublime, Vim/Neovim etc., but an unknown IDE is a first-class situation, not an error. Classification answers **what kind of tool this is**, not whether every second using it was coding.

App category is distinct from user goal qualification:

- `Cursor` may be categorized as `Development`.
- A browser at `github.com` may be Development for one goal and neutral for another.
- A terminal may be Development when executing an authorized project workflow, or `Utility` when doing unrelated tasks.
- “Coding” focus should follow user-approved rules across IDEs, not a fixed program name.

## 7. Polling and performance

Prefer native foreground-change events with modest watchdog polling as failure recovery; avoid aggressive per-millisecond polling. Cache expensive executable metadata. Debounce switch events (<1 sec jitter) without hiding actual short sessions. Database writes can be batched at short intervals, but process state transitions must be durable enough to recover gracefully.

## 8. Error/coverage model

Expose source status: Native Collector Running/Paused/Error, last event time, current process, idle status and coverage gaps. On permission denial, secure desktop, high-privilege process or unusual OS surface, return Unknown/Unavailable. Do not pretend full omniscience.

## 9. Test cases

- Switch from Cursor to Antigravity to a never-seen IDE → app names and new identities distinct, category first known/unknown pending, no time loss.
- Open several windows of same app → correct user-facing grouping.
- Keep VS Code visible while away 20m → idle time separated.
- Watch tutorial with no input → video-playback evidence recorded, foreground duration remains bounded.
- Sleep 1h; resume → no phantom usage.
- Lock Windows or change user → no active counting during lock.
- Change timezone/clock → accurate elapsed durations and daily bucket representation.
- Rapid Alt+Tab → controlled debounce and correct totals.
- Close GUI while agent running → continued measurements.
- Open overlay → self-tracking behavior consistent, no false Development time.
