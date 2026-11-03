# 03 — Visual design system and overlay mechanics

## 1. References and visual mood

The user provided dark toolbar and pill-style references resembling YouTube control surfaces; generated concepts are packaged in `../reference-images/overlay-flow-concept.png` and `../reference-images/dashboard-and-ai-concept.png`. Treat image typography/data as illustrative, not exact specifications.

**Style:** matte near-black canvas; dark neutral surfaces; pure/soft white primary text; gray secondary labels; thin dividers; soft borders; rounded controls; controlled motion. No default rainbow dashboard or neon tech aesthetic. The UI should be **minimal in controls, rich in information**.

## 2. Token proposal (to be validated visually)

| Token | Proposed value | Role |
|---|---|---|
| background/base | `#0B0B0D` | Dark desktop canvas |
| surface/1 | `#151517` | Normal card/pill panel |
| surface/2 | `#212124` | Interactive/elevated |
| surface/3 | `#2D2D31` | Selected state |
| outline | `#3A3A40` | Hairline borders |
| white | `#F7F7F8` | Primary text and buttons |
| muted | `#A8A8B0` | Secondary labels |
| subdued | `#73737C` | Disabled/tertiary |
| status colors | very limited | Only optional semantic success/warning/error indications; never sole signal |

Typography: Windows-native-looking sans; propose **Inter** or a licensed system stack, tabular numerals for time/counters, compact section headings and generous spacing around high-priority KPIs. Use approved font licensing, do not package unlicensed fonts.

## 3. Geometry and density

- Pill: default about **150–260 × 34–44 CSS px** depending on status; compact/icon-only mode smaller. No oversized wide bar.
- Quick panel: approximately **330–430 px wide**, fit height to content; avoid clipped controls.
- Full overlay: around **75–90% of available monitor work area** by default, resizable; set minimum size and compact responsive layout.
- Sidebar: 200–250 px desktop, collapsible to icons; top command bar remains reachable.
- Card radius: 12–18 px, pills fully rounded. Border generally 1 px, not excessive shadows.
- Add a maximum readable line width in text-heavy settings and explanations.

Numbers are initial design targets, not immovable constraints. Verify at Windows DPI settings and with user feedback.

## 4. Pill states: content priority

State machine (highest priority first):

1. User-disabled / paused: `Tracking paused` with clear pause icon.
2. Critical error / disconnected: `Partial tracking` + non-alarming diagnostic affordance.
3. Active restriction event: e.g. `Blocked · Shorts` for short transition, then return to current activity.
4. Focus countdown: `Focus · 42:18` if explicitly prioritized by user.
5. Foreground short video: `Shorts · 28` and optionally watch time `12m`.
6. Known Development category in any IDE: `Coding · 01:24` or app name when space allows.
7. Other recognized category: `Design · 22m`, `Browsing · 12m`, `Gaming · 45m` etc.
8. Unknown application: `Unknown app · 03m`, optional nonblocking categorization dot.
9. Idle/break: `Idle · 05:28` / `Break · 10m`.

The user can choose priority (e.g. “always show focus countdown”). Activity switching should debounce rapid transitions to prevent jitter; a 0.2–0.6 second transition is a design hypothesis, not tested.

## 5. Pill interaction rules

- **Click:** Quick panel toggles; reopening should restore recent state.
- **Drag:** Move pill, snap to screen edges or remember absolute anchor; don't interpret drag as click.
- **Hover:** Optional subtle background/size change, no obstructive popover.
- **Right-click:** Context menu with Open Panel, Pause Tracking, Hide Pill, Settings, Quit UI. Ensure Quit UI vs Quit Engine are distinct commands.
- **Keyboard:** Configurable summon/expand shortcut, accessible control when pill hidden.
- **Fullscreen:** Suppress overlay by default for games/videos unless user opts in.
- **Multiple monitors:** Remember per-display anchor, handle resolution changes, monitor disconnect and DPI changes.
- **Focus stealing:** Avoid aggressive activation when opening tiny quick panel; intentional Full Panel action may accept keyboard focus.
- **Cursor occlusion:** Avoid covering primary interactive application regions, allow click-through only in strictly non-interactive visual states (never leave hidden controls intercepting clicks).

## 6. Quick panel content hierarchy

Top: current app icon/title/category, elapsed session timer, tracking status and overflow menu. Middle: today active/focus/distraction/Shorts count; optional current goal. Bottom: Start/Pause Focus, Block Current Site, Open Full Overlay. Context-specific action changes with site and user restrictions. If unsupported, action is disabled with explanation.

Contextual examples:

- Coding in unknown editor: category pending, “Teach app category” action.
- Shorts: current qualifying count, daily remaining budget and “Stop Shorts for 30m”.
- Rule blocked: remaining lock duration, blocked URL or site name, view rule.
- Idle: muted visuals, avoid pretending the user is actively working.

## 7. Full overlay mechanics

The **Full Panel** must appear in-place, centered or near the pill origin, above the user's existing workspace. It is a native top-level window with custom chrome, not a content rectangle embedded into the previously focused app. The underlying desktop stays visible, optionally with a subtle dimmed backdrop. The UI can be resized, repositioned and closed back to pill.

**Pop Out** changes presentation to a normal taskbar-enabled desktop window. Whether this is a new native window or reconfiguration of the same UI instance is an implementation decision; state (selected date/page/filters) should persist seamlessly. Minimize flicker and duplicated agents. The user can select default opening behavior in settings.

Consider Windows focus activation, overlay stacking, Alt+Tab visibility, accessibility, secure desktop and graphics driver quirks. Always-on-top is best-effort relative to OS-managed exclusive/fullscreen surfaces, not an absolute promise.

## 8. Dashboard visual grammar

- **Hero metrics:** 4–5 concise KPIs with small movement indicators and direct drilldowns.
- **GitHub-style heatmap:** 7 × 52/53 squares with monochrome intensity; toggles Focus, Distraction, Short Videos, Goal Completion; periods of no data visually different from measured zero.
- **Activity filmstrip:** 24-hour row (or 1-hour zoom) with category segments and exact hover intervals.
- **Week-hour matrix:** 7 × 24 cells, label observed vs missing intervals.
- **Context-switch diagram:** weighted app/category transitions, focus preserved by default; show small dataset safely.
- **Video gallery:** thumbnails if enabled, otherwise textual history; do not cache full frames by default.
- **Rule-effectiveness comparison:** before/after with caveats and sample sizes.
- **Goal cards:** planned-vs-actual and clearly defined activity policy.

## 9. Motion principles

Motion should convey state changes and interaction only: 150–250 ms expanding surfaces, subtle number rolling after a confirmed video increment, low-frequency progress pulse during active focus, fade-in toasts on completed rules. Avoid constantly animated backgrounds; disable nonessential effects on power saver or reduced-motion.

## 10. Review checklist

Does the UI still feel like a tiny utility when collapsed? Can a user open every major page inside an overlay without an extra taskbar window? Are heatmaps and timelines actually readable at laptop resolutions? Does category unknown/pending have a dignified state? Is the pill neutral rather than judging the user? Are there accurate loading/empty/offline statuses? Are colors monochrome except rare important statuses?
