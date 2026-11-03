# 12 — Performance budgets, quality and reliability test plan

## 1. Realistic constraints

The target includes Windows laptops with limited RAM and no discrete GPU. Core tracking must run without an LLM, screen capture, GPU, Node runtime, full web dashboard or remote API service in the background. Optional Ollama may be heavy; user chooses when to enable/load it.

## 2. Proposed engineering targets (not achieved measurements)

| Metric | Initial target | Measurement protocol |
|---|---|---|
| Agent idle private working set | Target <60 MiB after warm-up | 30-min steady-state, representative machine |
| Agent idle CPU | Target <1% average of total reference CPU | 30-min, browser running/not running |
| Foreground event-to-pill update | Target p95 <500 ms | Instrumented input and panel visible |
| Browser rule trigger-to-block | Target p95 <2s under normal conditions | Event/rule/action ack timestamps |
| Agent UI-closed function | 100% continued operations in supported configuration | Close overlay/GUI, run 1h tests |
| AI/API stall impact | 0 missed critical timer transitions attributable to remote job | Artificial 30s latency and disconnect |
| DB compaction/cleanup | No noticeable blocking of live tracking | Background cleanup with stress activity |
| Crash recovery | No phantom active time over unobserved gap | Kill/relaunch process tests |
| Uninstall | No orphan native host or owned firewall rules | Clean VM tests |
| Peak memory | Defined after profiling | Include WebView2/renderer and extension separately |

Do not claim these are already attained. Build instrumentation before optimization. Acceptable CPU thresholds depend on host CPU count and measuring convention; document both percent and absolute utilization samples.

## 3. Load and endurance tests

- 24/72-hour endurance on laptop with Brave open and frequent focus changes.
- 100+ open tabs but 1 foreground tab; accurate attribution and bounded extension memory.
- 10k+ historical sessions; UI opens Timeline and yearly heatmap without loading raw events all at once.
- Thousands of short-video impressions and rapid swipe transitions; no duplicates, deterministic qualifying threshold.
- 300 configured rules, overlapping schedules, budget scopes; predictable execution and inspectable conflicts.
- Unknown-app queue flood (100 unfamiliar executables); bounded concurrency and stable RAM.
- Artificial slow/429/timeout AI/search provider; no UI or tracker stalls.
- Laptop battery saver / low-end hardware / 125% and 200% Windows scaling.

## 4. Crash, restart and lifecycle tests

- Agent crash mid-session → restore last committed state; mark gap unknown and avoid double counting.
- Tauri GUI crash → agent/extension unaffected.
- Browser exit/restart → offline coverage and proper block reinstatement/expiration.
- Native host crash → extension reconnect or clear degraded state, no lost mandatory rule expiry.
- Windows sleep, hibernate, lock, user switch, timezone and clock adjustment.
- SQLite locked/corrupted disk/full storage → expose partial operation and safe fallback; no busy-loop.
- Security software denies process metadata or hook → degrade source, never block all PC activity.
- OS firewall helper UAC denied → no partial elevated changes, browser-only feature unaffected.

## 5. Measurement truth table

| Input situation | Expected foreground active | Playback | Count |
|---|---|---|---|
| IDE foreground 10m, active input | ~10m IDE | 0 | 0 |
| IDE foreground 10m, idle 7m | ~3m active IDE | 0 | 0 |
| Brave foreground Shorts 5m playing | <=5m browser active by policy | up to 5m observed | qualified videos only |
| Background YouTube video while IDE focused | IDE time only | YouTube separate playback | Video if qualified per adapter |
| Browser 5 open tabs, one focused | 1 active tab's time | background media separately | Only observed qualified videos |
| System locked 60m | 0 active | 0 or known-stopped | 0 |
| Agent stopped 30m | Unknown coverage gap | Unknown | No inferred plays |

Exact active-idle rules require product decisions and are documented; do not equate lack of input with zero actual attention in a video player.

## 6. Unit, integration and end-to-end testing

**Unit:** session interval union, time-zone conversion, classification precedence, budget thresholds, short replay dedup, JSON validation, rule conflict resolution, reset schedules.

**Integration:** Windows event API → agent events; extension content script → service worker → native bridge; local SQLite migration; DNR policy install; GUI subscription reconnect; credential-store save/remove; metadata queue.

**End-to-end:** Fresh Windows install, onboarding, pair Brave, record active day, set 20-min limit, reach threshold, see block, close GUI, observe expiry, inspect dashboard, delete all data, uninstall.

**Visual:** test at laptop resolutions (1366×768, 1920×1080, 2560×1440), high DPI, multi-monitor, fullscreen, HDR/dark desktop where available, reduced-motion/contrast preferences.

## 7. Observability without privacy leaks

Add counters for event ingest/drop, queue depth, session reconciliation warnings, rule evaluation latency, browser native-host handshake failures, video adapter version, SQLite retries and anonymized resource samples. Default logs must be redacted and locally rotated. Diagnostic exports require preview and explicit user choice.

## 8. Acceptance failure conditions

Do not ship an MVP if: blocks persist after expiry without safe recovery; duplicate foreground time is routinely added; extension silently loses rule state; disabling remote AI prevents tracking; raw private titles/API keys reach logs; overlay cannot close without killing agent; degraded browser coverage is falsely shown as successful.
