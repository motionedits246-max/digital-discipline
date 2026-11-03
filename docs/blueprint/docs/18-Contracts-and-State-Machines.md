# 18 — System contracts and state machines (design only)

## 1. Event envelope conventions

All OS/browser/AI events must have stable source identity, a schema version, generated event ID, UTC occurred/received timestamps, session or browser-profile reference, type and a payload limited to permitted fields. The runtime must reject unexpected fields, malformed URLs, dangerous timestamps and oversized messages at trust boundaries.

| Conceptual event | Source | Fields | Consumer |
|---|---|---|---|
| `os.foreground_changed` | Native collector | prior/next app identity, timestamp | Session reconciler |
| `os.input_idle_started` | Native collector | session, idle threshold, observed_at | Session reconciler |
| `os.input_idle_ended` | Native collector | session, duration | Session reconciler |
| `os.session_locked` | Native collector | session, start | Session reconciler |
| `os.session_unlocked` | Native collector | session, resume | Session reconciler |
| `browser.tab_activated` | Extension | browser/profile/tab, normalized resource | Browser reconciler |
| `browser.route_changed` | Extension | route class, sanitized URL/ID | Browser reconciler |
| `video.player_started` | Site adapter | platform, video ID, visible, timestamp | Video sessioner |
| `video.player_paused` | Site adapter | platform, video ID, timestamp | Video sessioner |
| `video.qualified_play` | Site adapter | video ID, qualifying threshold, dedupe token | Counter + rules |
| `video.adapter_degraded` | Extension | platform, capability, reason | Health/coverage |
| `rule.threshold_reached` | Rule engine | rule ID/version, measurement window | Policy executor |
| `rule.enforcement_result` | Browser/OS adapter | action, method, result, error code | Audit/UI |
| `category.classification_completed` | Background worker | subject, validated category, evidence IDs | DB + aggregates |
| `agent.coverage_gap` | Agent | source, start/end, cause | Coverage UI |

## 2. Commands from UI to agent

Examples: `SetTrackingPaused`, `CreateRuleDraft`, `ValidateRule`, `CommitRule`, `DisableRule`, `StartFocus`, `EndFocus`, `SetCategoryOverride`, `ForgetDateRange`, `SetProvider`, `RemoveCredential`, `RequestClassifyUnknown`, `ExportAggregate`. They must carry caller identity/permission context and result in deterministic audited operations. All privileged/destructive actions require separate confirmation. Commands never contain arbitrary shell script content.

## 3. Activity state machine

`Unknown → ActiveForeground → IdleForeground → ActiveForeground` on input; `ActiveForeground/IdleForeground → Locked/Sleeping → ActiveForeground/Unknown` on unlock; `Any → Offline` on agent shutdown or loss of observation. Current foreground app pointer may exist while input idle, but active time computation must respect idle. Browser playback is an **orthogonal state** and can coexist with other foreground usage.

## 4. Video playback state machine

`NotObserved → Impression → ObservedPlaying → Paused/Hidden → Playing/Ended/NavigatedAway`. A qualifying-play event fires once per qualifying play instance when threshold is reached; replay/new instance rules must be defined. `UniqueVideo` is derived by normalized ID plus reporting scope. A timer cannot simply increment views on every URL change.

## 5. Job state machine

`Queued → Running → Succeeded`, or `Running → RateLimited → ScheduledRetry → Running`, or `Running → FailedFinal`, or `Queued/Running → Cancelled`. Jobs are idempotent by subject/version. Queue records and errors are local, sanitized, capped and privacy-scoped. `Succeeded` may be category `Unknown`; it doesn't necessarily mean a productive category was inferred.

## 6. Restriction state machine

`Draft → Validated → Armed → Triggered → Enforcing → Expired/Disabled`; transitions may produce `Degraded` when browser lacks extension permissions. An active block record must include owner rules and an explicit expiry; rule dispatcher can't just leave a DNR rule forever without an ownership state. Uninstall removes only owned restrictions.

## 7. Overlay state machine

`UIHidden` (agent alive) ↔ `Pill` ↔ `QuickPanel` ↔ `FullOverlay` ↔ `PopOutWindow`. The pill may be hidden independently from tracking. State transitions must preserve current page/date/filter choices. `UIHidden` is not identical to “tracking paused.”

## 8. API/contract versioning

Use `major.minor` schema version and negotiate capabilities. A major breaking change must not silently allow unsupported actions. Browser adapters and agent may upgrade at different times, so error modes should be explicit and recoverable. Design migrations to preserve old data and rollback if installation fails.

## 9. Test fixtures to author before code

- Timeline traces with foreground switch, idle, lock, sleep, browser tab changes.
- Video event traces with impressions, start/pause, fast swipe, replay and route change.
- Rule traces with local midnight, schedule and conflict priority.
- Classifier results with valid/invalid JSON, unsupported categories, fake evidence and 429 errors.
- GUI state transition traces over multiple monitors and restore after crash.

## 10. Cross-contract invariants

1. Every displayed time sum is a deterministic function of normalized interval data.
2. Source coverage can be Unknown; incomplete observations do not become zero.
3. All external AI responses are untrusted and never directly enforce security-sensitive rules.
4. The UI can be killed without killing the tracking/policy agent.
5. Critical queues are independent from network and optional classification queues.
6. Browser enforcement changes require acknowledgements and persisted ownership/expiry.
