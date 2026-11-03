# 07 — Logical data model, statistics and aggregation

No migration or SQL code has been generated. This is a **logical schema** to guide future migrations. Actual table structures and indexes should be finalized after the event protocol is validated.

## 1. Persistence principles

- Local SQLite DB is authoritative; a single agent manages writes.
- Raw events and normalized interval sessions are distinct: raw for audit/reconciliation, normalized for reporting.
- Event identity, deduplication and provenance are first-class.
- User corrections and policies must not destroy underlying raw facts.
- Default retention is configurable; recommend shorter retention for high-sensitivity video/URL metadata than derived aggregates.
- Keep DB schema version, migrations, last successful checkpoint, integrity status and time-zone/clock-change history.

## 2. Proposed logical entities

| Entity | Main fields | Notes |
|---|---|---|
| `devices` | device_id, label, platform, first_seen | Single local device initially |
| `app_identities` | app_id, executable identity, display_name, publisher, product, source | Avoid raw/private paths unless needed and consented |
| `app_processes` | process_instance_id, app_id, pid, boot_id, start/end | Reused PID does not equal same session |
| `browser_profiles` | browser_id, profile_pseudonym, connection_state, consent | Multiple browsers/profiles |
| `web_resources` | resource_id, canonical domain, route class, privacy tier | Full URL optional and safely normalized |
| `raw_events` | event_id, source, kind, occurred_at, received_at, sequence, payload_ref, coverage | Idempotent ingest |
| `activity_sessions` | session_id, start_utc, end_utc, app_id, browser_id, resource_id, foreground_active_ms, idle_ms, provenance | Materialized interval accounting |
| `video_entities` | platform, canonical_video_id, optional_title, metadata_status | Titles opt-in; no transcript requirement |
| `video_play_intervals` | session_id, video_id, start/end, player_state, played_ms, visible, qualifying | Independent from app foreground totals |
| `category_taxonomy` | category_id, parent, label, sort_order | Customizable hierarchy |
| `classifications` | subject_type/id, category_id, source, confidence, version, applied_from | Model/rule provenance |
| `user_overrides` | subject, category, scope, created, revision | Highest priority |
| `goals` | goal_id, target, schedule, qualifying_policy, status | Goal-specific productive rules |
| `focus_sessions` | goal_id, start/end, planned_duration, observed_duration | Manual and scheduled |
| `automation_rules` | rule_id, version, trigger, conditions, action, expiry, status | Versioned and auditable |
| `rule_executions` | execution_id, rule_id/version, input_window, result, reason, timestamp | Explainability |
| `block_state` | target, method, owner_rule_ids, start, expiry, health | Ownership allows reliable rollback |
| `jobs` | id, type, state, next_attempt, attempt_count, dedupe_key, error_redacted | Persistent background work |
| `provider_profiles` | provider, model, display_name, credential_reference, limits | NEVER store plaintext key in DB |
| `app_settings` | setting_key, user_scope, value, version | Privacy scope and UI choices |
| `aggregate_daily` | day, category, tracked_ms, active_ms, focus_ms, distraction_ms, coverage_ms | Recomputable materializations |
| `aggregate_hourly` | hour, category, minutes/plays, source coverage | Heatmaps and week-hour matrix |

## 3. Time accounting equations

Let `T` be elapsed time inside report scope; `F` the union of foreground windows; `I` input-idle intervals; `L` lock/sleep intervals; `U` unknown/source coverage gaps.

- `ForegroundTracked = measure(F ∩ tracked_coverage)`.
- `ActiveForeground = measure(F \ (I ∪ L))`, under selected input threshold; confirmed passive media playback is tracked in separate `Playback` metric.
- `Distraction = measure(ActiveForeground ∩ UserDistractingPolicy)` with category/goal scope; if rules overlap, use union of intervals, not summed durations.
- `Unclassified = measure(ActiveForeground ∩ UnclassifiedActivities)` and should not be placed silently into Work or Distraction.
- `Coverage = observed_tracked_seconds / eligible_scope_seconds`. The denominator depends on what the user intended to monitor; declare it.
- `FocusGoalProgress = qualified_time / goal_target`, bounded/defined for zero targets. Goal qualification is policy-based, not identical to “one app was open.”

Important: `Playback` can occur simultaneously with foreground app use (e.g. audio in background); it **must not** be naively added to foreground totals.

## 4. Example data flow

At 10:00 a user focuses an unfamiliar IDE. Event stored immediately, session begins as Unclassified, background metadata job scheduled. At 10:15 the user switches to YouTube Shorts; session closes. At 10:16 a qualified play is observed; video counter increments. At 10:20 AI finally classifies IDE as Development; classification table updated, aggregates for affected historical interval recalculated and a subtle UI refresh sent. **No 20-minute gap or frozen screen** was introduced.

## 5. Aggregation and corrected history

- Raw events immutable except deletion per privacy controls.
- Classification changes append a new revision or update effective label with audit metadata.
- Time spans remain exactly the same; only category attribution recalculates.
- Recompute narrowly by affected subject/date range, preferably in low-priority jobs.
- Persist classifier version so rule/model changes don't make historical reports mysteriously shift without explanation.
- Classification can be backfilled by user choice or only for new events; default should be clear.

## 6. Heatmap design

A GitHub-style calendar heatmap has daily cells with user-selected metric: focus minutes, distracting minutes, unique short videos, qualifying plays, goal completion or active minutes. Each cell includes day, value, comparable-coverage flag, category filter and tooltip. Five bins can be quantile-based or fixed thresholds (declare which). Missing day/permission gap uses patterned/missing data, not same tone as measured zero.

## 7. Other visualizations

- Activity ribbon: session intervals as proportion of day with pause/idle markers and hover source.
- Week-hour matrix: 7 × 24 bins with valid exposure time and category.
- Attention-switch Sankey/network: nodes = app categories/apps, edges = meaningful transitions, filtered by minimum duration and debounced.
- Video distribution: platform/category, unique vs repeated and elapsed playback.
- Rule impact: compare periods after adjusting for observed coverage and weekday differences. Observational outcomes are not proof of causation.
- Weekly insights: source-linked totals and counts, with “estimated” qualifier for hypothetical recovered hours.

## 8. Retention defaults to propose, not hardcode

- Fine-grained app raw events: e.g. 30 days default.
- Detailed URL/video titles: opt-in and shorter default (e.g. 14 or 30 days).
- Daily aggregates: e.g. 1 year or until deleted.
- Local logs: capped size/short TTL and redacted by default.
- AI request/response storage: off by default except user-saved Coach chats.

The user must be able to configure retention and delete either all data or a selected category/date range.

## 9. Export and portability

JSON export with schema version, timezone, anonymization toggle and a CSV subset. Allow selective export of aggregates only. Import should validate version and reject hostile paths/oversized payloads. Optional backups should be encrypted if they contain detailed browsing history.

## 10. Data quality indicators

All reports expose uncertainty where relevant: unknown app share, unsupported browser profile share, extension disconnected minutes, source event loss, video detector health, classification pending and history coverage. An accurate product is one that can explain why some data is missing.
