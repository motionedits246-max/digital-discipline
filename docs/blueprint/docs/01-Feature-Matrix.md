# 01 — Complete feature matrix: all 32 selected features

**Release codes:** P1 = initial MVP, P2 = advanced v1, P3 = optional/advanced v2+. Features marked P2/P3 remain accepted scope, not omitted. The exact release bundle may change after feasibility testing.

## A. Tracking and interpretation

| ID | Feature | Phase | Data/input | Main UX | Success criterion |
|---|---|---|---|---|---|
| A1 | Universal app/time tracker | P1 | Windows foreground, input idle, process identity | Live pill, Overview | Any IDE/app detectable, unknowns labeled, active totals not double counted |
| A2 | Shorts/Reels counter | P1 | Video adapter play/visibility/content ID | Pill, Video Intelligence | Unique vs qualifying plays and replay separated |
| A3 | Content categorization | P2 | Rules, metadata, optional AI | Categories, overrides | Evidence, confidence and override supported |
| A4 | Distraction/wasted-time calculator | P1 | User-defined policies, reconciled intervals | Overview, Insights | No objective “waste” assertion; attributable totals |
| A5 | Video history | P2 | Opt-in metadata, play intervals | Searchable video gallery | Duration, repeats, source and retention controlled |
| A6 | Attention switches | P2 | App/domain transitions | Activity Flow, Insights | Sessionized transitions, no duplicate event spam |
| A7 | Browser comparison | P2 | Browser/profile identity | Comparison charts | Brave/Chrome/Edge totals without cross-browser overlap |
| A8 | Daily activity timeline | P1 | Session spans | Interactive timeline | Expand into apps/sites/idle with stable timezone |

## B. Restrictions and automation

| ID | Feature | Phase | Data/input | Main UX | Success criterion |
|---|---|---|---|---|---|
| B1 | Website blocker | P1 | Domain/path rules | Blocking Center | Temporary, timed, indefinite; safe rollback |
| B2 | Usage-based auto-block | P1 | Session counter/threshold | Rule editor | Deterministic execution and reset |
| B3 | Scroll count limit | P2 | Qualified short-video counts | Rule editor | Unique/repeat choice and reset transparent |
| B4 | Automation builder | P2 | Trigger/condition/action model | Visual IF→THEN editor | Validation, preview, trace, exceptions |
| B5 | Strict focus | P2 | Time-window policies | Focus panel | Clear duration, override and emergency recovery |
| B6 | Partial website restrictions | P2 | URL/content selectors | Blocking Center | Shorts/feed disabled without blocking all watch pages |
| B7 | Shared time budget | P2 | Sets of domains/apps/categories | Budgets UI | One cumulative allowance, no double count |
| B8 | Scheduled rules | P1 | Local timezone schedule | Calendar rules | DST/time changes and restart handled |

## C. Desktop UI and workflow

| ID | Feature | Phase | Data/input | Main UX | Success criterion |
|---|---|---|---|---|---|
| C1 | Floating controller | P1 | Current status, statistics | Tiny pill → quick panel | Draggable, display toggles, overlay expansion |
| C2 | Independent background engine | P1 | OS events, rule scheduler | Tray/status | No GUI dependency or mandatory cloud calls |
| C3 | Analytics dashboard | P1 | Local aggregates | Overview | Timeline, heatmap, drilldowns, date filters |
| C4 | Goal planner | P2 | Goal/schedule/activity conditions | Focus & Goals | Any editor/IDE can qualify, user overrides |
| C5 | Command bar | P2 | App commands, search | Global shortcut | Keyboard-accessible and context-safe |
| C6 | Distraction-free web UI | P2 | Browser DOM content adapter | Blocking Center | Hide shelves/feed/autoplay; reversible |
| C7 | Smart notifications | P2 | Rules, attention events | Toast/pill | Dedup, cooldown, do-not-disturb, quiet hours |
| C8 | Privacy settings | P1 | Consent/exclusions/retention | Settings | Tracking pause, delete, export and scope shown |

## D. Behavioral intelligence

| ID | Feature | Phase | Data/input | Main UX | Success criterion |
|---|---|---|---|---|---|
| D1 | Doomscroll detection | P2 | Consecutive qualifying plays | Insights | Explainable threshold, configurable gaps |
| D2 | Goal-aware intervention | P2 | Active goal + classified activity | Quick panel/rules | Explicit trigger and user choice |
| D3 | AI productivity coach | P3 | User-authorized summaries/aggregates | AI Coach | BYOK, provider models, traceable stats, opt-in sending |
| D4 | Distraction prediction | P3 | Multiweek personal patterns | Insights | Confidence/uncertainty, opt-in, no overclaiming |
| D5 | Time-recovery simulator | P2 | User-entered hypothetical change | Insights | Labeled forecast—not observed savings |
| D6 | Cross-device tracking | P3 | Optional linked device events | Device settings | Consent, conflicts and overlapping-time reconciliation |
| D7 | Focus difficulty levels | P2 | Gentle/Balanced/Strict presets | Focus rule setup | Predictable enforcement and documented bypass limits |
| D8 | Weekly insights | P2 | Historical time/rule data | Weekly review | Reproducible figures, comparison caveats |

## Release sequencing rationale

- A1 + C2 + A8 → foundational tracker; this precedes most analytics and goal functionality.
- Browser extension + A2 → short-form metrics; B3 and D1 cannot be implemented correctly first.
- Normalized usage totals + B1/B2/B8 → meaningful time budgets; B7 follows after dedup and shared-set semantics are stable.
- A3 + manual overrides → D2 context-aware interventions and D3 grounded coaching.
- C8 from day one → privacy constraints shape every data field and API boundary.
- D6 later → cross-device overlaps require an already verified session model.

## User stories by persona

**Developer:** “When I edit code in Cursor, Antigravity or an unfamiliar IDE, the system should recognize ‘Development’ or leave it pending while my focus timer keeps running.”

**Student:** “If I watch a normal tutorial video during study hours, I want it treated differently from a Shorts streak based on my rules.”

**Strict-mode user:** “I want a 45-minute restriction, clear lock expiry and an emergency option so I cannot accidentally lose control over my machine.”

**Privacy-conscious user:** “I want app-level time statistics without storing full URLs or video titles, and I want to know exactly what a BYOK model receives.”

**Performance-sensitive user:** “My laptop has limited resources. The tracker should keep working when the dashboard is closed and should not load an LLM just to count time.”
