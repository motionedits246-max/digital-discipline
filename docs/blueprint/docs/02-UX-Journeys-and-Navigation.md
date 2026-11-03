# 02 — UX journeys, navigation and interaction design

## 1. Interaction philosophy

Three presentation levels share one stateful background agent:

1. **Live pill:** ambient glanceable status. Always on top *if enabled*; ultra-compact and movable.
2. **Quick panel:** click pill to show context and direct actions without a traditional application window.
3. **Full overlay workspace:** “Full Panel” expands into a large, resizable top-level overlay *over the user's current desktop*. A **Pop Out** control optionally converts it to a normal standalone desktop window. The user's explicit preference is overlay-first, not window-first.

The native engine should run with all three invisible. Overlay UI should not be used to keep the tracking timer alive.

## 2. First-run onboarding journey

**Step 1: Welcome** — concise explanation, “what is tracked,” offline-first design. No forced login.

**Step 2: Select tracking scope** — app names/processes only; optional window titles; optional full website domains; optional detailed URL/video tracking. Default data minimization. Show sample results for each permission level.

**Step 3: Install/pair browser extension** — explain why Windows cannot know all active URLs or Shorts events from process names. Detect installed supported browsers; pair Brave first. Show status: connected / missing / permission denied / incompatible / degraded. Multiple browser profiles are distinct and must be paired or recognized separately.

**Step 4: Category preferences** — categories Development, Education, Work, Entertainment, Gaming, Social, Communication, News, Neutral, Unclassified. Let user remap app/domain or mark intentional breaks.

**Step 5: Focus goals (optional)** — e.g. 4 hours of Development during chosen interval; never assume all coding occurs in VS Code.

**Step 6: Mini controller** — choose default placement, size, opacity, auto-hide, hotkey. Preview current pill states.

**Step 7: Optional BYOK** — not part of required onboarding. Separate toggles for metadata research, remote classification, and Coach.

**Step 8: First-session finish** — agent starts in user session; permission summary and pause control prominent.

## 3. Daily normal use journey

- Windows starts → background agent runs (if autostart enabled); the pill appears at configured location.
- User switches between editor/IDE, browser and other applications → status and time update without modal dialogs.
- User watches Shorts → short-video adapter emits qualifying playback events and count increments.
- User clicks pill → quick panel shows current activity, focus/distraction counters, current rule, Start Focus and Block Site.
- User chooses Full Panel → overlay appears without launching a mandatory separate taskbar application window.
- User opens Activity Timeline → date filter, 24-hour ribbon, category segments and click-through events.
- User reviews video gallery or app categories → manual correction writes a stable override, recomputes affected aggregates asynchronously.
- User sets a 20-minute Shorts budget → rule preview explains scope, reset and whether it covers all supported browser profiles.
- Threshold reached → extension blocks future Shorts navigation/plays using best available mechanism; a clear block page or inline intercept explains the reason and time remaining.
- The agent persists when the GUI closes. The tracker explicitly excludes its own overlay from “active coding” accounting.

## 4. Empty, partial and error states

| Situation | UI response |
|---|---|
| No tracking permission / first day | Explain required permission and show demo chart clearly labeled sample |
| Extension not installed | “Browser details unavailable; app-level usage still tracking” + Pair extension |
| Unknown IDE | “Classifying in background” and conservative Unclassified category |
| AI request delayed | Show queue state only in optional details; never a blocking spinner over the app |
| Network offline | Local tracking continues, external lookup queued or skipped |
| Activity is excluded | “Private activity — excluded” rather than exposing underlying details |
| Missing content metadata | “Video category unavailable”; retain play/count observation if consent permits |
| Site adapter outdated | Degraded-state badge and avoid fabricated video counts |
| Block rule cannot enforce | Explain unsupported browser/profile, actionable remedy; do not silently claim success |
| DB recovery | Reconstruct from event journal and mark uncertain time ranges as partial |
| AI Coach disabled | Educate about BYOK; manual insights remain accessible |

## 5. Navigation architecture

**Overview** — hero KPIs, today/last 7d, timeline ribbon, annual heatmap, active goal, latest event/rule cards.

**Live Activity** — current process/app, browser domain if allowed, session elapsed, direct pause, focus shortcuts, tracking diagnostics.

**Activity History** — calendar/heatmap/date range, timeline, session list, context-switch graph, app breakdown and export.

**Video Intelligence** — Shorts vs Reels, qualifying plays, unique IDs, watch duration, repeat/skip/unknown, categories, history gallery and streaks.

**Focus & Goals** — plan/goals, focus sessions, time targets, difficulty presets, achieved-vs-planned views.

**Blocking Center** — site/domain/path rules, budgets, partial-page controls, current restriction state, block log and recovery.

**Automations** — IF/AND/THEN builder, schedules, rule validation/simulation, history and priorities.

**Behavior Insights** — day/hour matrix, context-switch flow, doomscroll analysis, weekly reports, time-recovery simulation, intervention outcomes.

**AI Coach** — opt-in natural-language explanation, provider status and citations back to user's own aggregate data; no AI required for conventional reports.

**Settings & Privacy** — profile preferences, tracking inclusion/exclusion, appearance, overlay, browser pairing, notification controls, local storage/retention, metadata/AI permissions, BYOK keys, diagnostics.

## 6. Search/command UX

Command bar supports `/focus 25`, `block current domain for 30m`, `today's video views`, `pause tracking` (example human-language commands, not implemented syntax). Search across app names, dates, rule names and video titles only if history/title storage enabled. Confirm destructive/elevated operations; no silent changes to firewall rules.

## 7. Filters and drilldowns

Every major view should support date scope (today, yesterday, week, month, custom), app/domain/category/device scope (device later), browser profile if available, and privacy-safe CSV/JSON export. A card click opens a contextual drawer; do not force navigation away from the current insight. Heatmap day opens the same day in timeline view and preserves active filters.

## 8. Accessibility and presentation requirements

- Minimum target size for pill and overlay buttons; minimum legible type; scaling at 100/125/150/200%.
- Keyboard focus rings, accessible names for icons, tab order, screen-reader summary of heatmap, non-color-only indicators and tooltips.
- Contrast in both full canvas and semi-transparent overlay; dark and high-contrast override settings.
- Reduced-motion preference removes count animations and panel spring effects.
- Window position must account for taskbar, multi-monitor DPI, safe areas and fullscreen game/movie use.
- Offer “overlay hidden during fullscreen” by default and respect secure desktop contexts.

## 9. Avoid these anti-patterns

No forced login, no deceptive “you wasted 6 hours” language, no automatic uploading of full browser history, no intrusive constant popups, no permanent always-visible screen obstruction, no massive dashboard WebView kept alive in background, no spurious motivational streaks, no dangerous irreversible strict mode.

## 10. UI test journeys

Test the complete path: startup → open any IDE → pill switches to Coding → open a Short → counter changes → click pill → full overlay → inspect heatmap → start 20m block → close GUI → rule still active → expiry unblocks → turn off tracking → all observers pause as disclosed.
