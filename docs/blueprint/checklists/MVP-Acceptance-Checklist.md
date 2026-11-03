# MVP acceptance checklist (future build)

## Functional and attribution

- [ ] Any foreground IDE/app works (not hardcoded to VS Code).
- [ ] Unknown apps remain tracked and categorized later without data loss.
- [ ] Foreground, active, idle, playback and unknown time are distinct.
- [ ] Tab/window overlap does not inflate aggregate usage.
- [ ] YouTube Shorts adapter separates impressions, qualifying plays, unique videos and replays.
- [ ] Instagram support is truthful (supported, partial or unsupported visibly disclosed).
- [ ] Local basic categories and manual overrides exist; no required AI.
- [ ] Day timeline and GitHub-style heatmap drill into underlying sessions.
- [ ] Day boundaries, system clock changes, lock, sleep and agent crash are tested.

## GUI

- [ ] Tiny live pill works over Windows applications and does not block typing.
- [ ] Pill changes state for coding, Shorts, Focus, Blocked and Idle.
- [ ] Clicking pill opens quick panel.
- [ ] Full Panel opens in-place overlay, not requiring a separate app window.
- [ ] Optional Pop Out opens normal resizable window.
- [ ] GUI closes while agent continues running.
- [ ] Monochrome theme matches approved reference direction.
- [ ] Heatmap/timeline and status controls accessible with keyboard and reduced motion.

## Blocking

- [ ] Domain/path block works in supported paired browser.
- [ ] Time-based budget triggers at defined threshold.
- [ ] Scheduled rules activate/deactivate correctly; expiry works without open GUI.
- [ ] Browser without extension is disclosed as unsupported for browser blocking.
- [ ] Expired block clears after crash/browser restart.
- [ ] User can see reason, duration and recovery/override controls.
- [ ] OS-level helper is excluded from MVP unless separately reviewed and approved.

## Reliability and privacy

- [ ] Offline core functionality; AI/search outage has no effect on tracking/blocking.
- [ ] Resource tests meet agreed budgets; measure agent, GUI WebView and extension separately.
- [ ] No default remote upload or account requirement.
- [ ] Privacy permissions, excluded sites, incognito, pause and data deletion documented and working.
- [ ] API keys absent from logs and local plain settings.
- [ ] Clean uninstall removes native messaging registration and owned rules.
- [ ] Explicit coverage/degraded-state indicators for missing browser details.
