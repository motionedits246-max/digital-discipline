# 16 — Screen-by-screen GUI specification

This document converts the selected functionality into an implementable navigation inventory. Every card should define its data source, empty state, click action, accessibility text and refresh cadence before frontend implementation.

## Shared app shell

Left collapsible sidebar: Overview, Live Activity, Activity History, Video Intelligence, Focus & Goals, Blocking Center, Automations, Behavior Insights, AI Coach, Settings & Privacy. Top: global search/command bar, selected date range, privacy/tracking status, window controls (pin/overlay/pop out/close). Right contextual details drawer for selected timeline/session/video/rule. Footer/tray: last sync/extension status and app version, no dark-pattern upsell.

## S01 — Overview

**Question answered:** “How was my digital day?”

- KPI row: active time, focus time, distraction time, qualifying short-video plays, context switches. Each metric tooltip gives definition and coverage.
- Activity timeline with user-chosen category colors/patterns and browser/app drilldown.
- Large GitHub-like heatmap with toggle for tracked time/focus/shorts/distraction and month labels.
- Active goal cards with planned vs actual; top apps/sites; focus blocks; current restriction; insight-of-the-day.
- Period switch: Today / Last 7 Days / Last 30 Days / Custom. Comparing periods must normalize for tracked availability.
- Empty state: “Tracking begins when you enable the background agent; sample visualization available.”

## S02 — Live Activity

**Question:** “What does the system currently think I am doing?”

- Foreground application name, publisher/process, category, confidence, clock and elapsed duration.
- Browser profile and domain (only when permissioned); active video ID/type, playback signal and current count.
- Compact event stream for troubleshooting (privacy-redacted), heartbeat and event queue state.
- Direct “Change Category,” “Exclude App/Domain,” “Start Focus,” “Block Current Domain,” “Pause Tracking.”
- Rule evaluation status: allowed / blocked / partial / unsupported.

## S03 — Activity History

**Question:** “When did each activity happen?”

- Day picker and 24-hour stacked session filmstrip; zoom to 15-minute resolution.
- Toggle Foreground, Active, Video Playback, Idle, Focus, Distracting, Unknown.
- Detail drawer for app/site, exact start/end, duration, source and confidence.
- App/domain comparison list, attention-switch transition graphic, daily notes (opt-in).
- GitHub calendar heatmap or linked Overview heatmap; keyboard date navigation.
- Partial periods visually indicated; no claim that unplugged/offline tracking proves zero usage.

## S04 — Video Intelligence

**Question:** “What kinds of videos did I actually play?”

- Metric tabs: short-form qualifying plays, unique videos, replay counts, playback minutes, scroll streaks, estimated skips.
- Platform filters: YouTube Shorts, Instagram Reels, future supported adapters; adapter health status.
- History gallery/list: thumb if available and allowed, video title if stored, source, duration, category, evidence, viewed-at, repeats.
- Category distribution, day-hour grid, top recurring themes, known/unknown ratios, last-session breakdown.
- Manual correction: category, false view, duplicate video ID; recompute affected metrics.

## S05 — Focus & Goals

**Question:** “Did I do what I intended?”

- Create goal with period, target, category/app/domain rule, start/end schedule and timezone.
- Starter goal templates are optional and editable; never hardcode VS Code as Coding.
- Focus session controller with custom duration, included activities, break policy, pause behavior, difficulty level.
- Progress ring/bar, timeline overlay of intended vs observed category, notifications and historical completion.
- Clarify that a site can be both “learning” and “distraction” under a user-selected goal.

## S06 — Blocking Center

**Question:** “What is blocked, why and until when?”

- Active restrictions, saved blocklists, domain/path scope, time budgets, category/site packages.
- New rule wizard: choose target → method → schedule/budget → strength → override/recovery → preview → save.
- Rule detail: what sites/profile/device covered, last execution, expiry, exemptions, enforcement degradation and supported method.
- Partial blocking controls: YouTube Shorts, recommendations, autoplay, feed elements (with “may break when site updates” note).
- Owned-rule rollback action and emergency recovery; no silent addition of system-wide firewall rules.

## S07 — Automations

**Question:** “When X happens, what should the application do?”

- Rule canvas cards: IF trigger, AND conditions, THEN action, exceptions, recurrence/reset.
- Templates: 15 Shorts → pause Shorts for 60m; YouTube + Instagram total >45m → block; 10 PM–6 AM → block social; Focus begins → hide recommendations.
- “Explain execution” simulator based on sample/recorded event; validation of incompatible scopes.
- History: timestamp, trigger source, matched condition, attempted action, enforcement result and reason.
- Conflict dialog when user adds a rule inconsistent with an existing strict allow/deny.

## S08 — Behavior Insights

**Question:** “What patterns can I change?”

- Doomscroll streaks, time-of-week matrix, attention-switch flow, before/after restriction summaries.
- Weekly report cards with comparisons against days with similar tracking coverage.
- Time-recovery simulator: choose 10/20/30/60 minutes daily avoided; compute theoretical weekly/monthly time (not guaranteed savings).
- Optional D4 prediction later: explain inputs, confidence, opt-out; never pretend it knows the future.

## S09 — AI Coach

**Question:** “Can an assistant explain my own numbers?”

- Chat surface, model/provider pill, permission summary, date/scope picker, suggestions grounded in local aggregate measurements.
- Example: “Why did my focus time decrease on Tuesday?” response links to day timeline and observed changes, not invented causes.
- BYOK quick entry in Settings, secure key storage, model list/manual ID, test connection, caps, token/cost display where provider supports them.
- Disabled state still offers manual/SQL-derived stats. Sending raw browsing history is never the default.

## S10 — Settings & Privacy

Tabs: General, Appearance, Overlay, Tracking Scope, Browsers, Categories, Focus, Notifications, Rules, AI & Providers, Local Data, Diagnostics, Advanced System Blocking.

- General: startup, run-in-tray, exit semantics, timezone, locale.
- Overlay: size, position, style, opacity, pin, keyboard shortcut, fullscreen behavior, pop-out preference.
- Tracking: permission tiers, excluded apps/sites, input-idle threshold, session definition, pause.
- Browsers: pairing, profile list, permissions, adapter statuses and coverage.
- Categories: hierarchy, user overrides, application mapping, unknown queue review.
- AI: coach and classification modes separate, BYOK, model IDs, key removal, request permission, spend limits.
- Data: export, retention, purge, integrity, backups and encrypted backup controls.
- Diagnostics: event loss, queue health, blocking status, CPU/RAM, logs with redaction.

## Interaction details common to all screens

Each chart supports tooltip and accessible table alternative. Every irreversible action requires confirmation. Users should be able to customize Overview card order without making the default dashboard overwhelming. Preserve page position/filter state when switching overlay ↔ pop-out. Announce data coverage for every long-term comparison.
