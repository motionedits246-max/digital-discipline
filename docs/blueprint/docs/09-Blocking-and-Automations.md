# 09 — Blocking, focus rules and automation semantics

## 1. Scope

B1–B8 must converge on **one unified policy engine**. A website blocker, scroll-count budget, time-budget limit, scheduled rule and Strict Focus session should not contain contradictory, duplicated logic. Every enforcement action needs an owner, an expiry/disable path and an execution log.

## 2. Conceptual rule format

A rule consists of:

1. `rule_id`, `version`, `enabled`, human-readable `name`.
2. `target`: website domain/path, application, short-video platform/route, content type, category or group.
3. `trigger`: time threshold, view count, scheduled window, app switch, focus start, streak event, manual command.
4. `conditions`: active goal, weekday/timezone, browser profile, threshold scope, user override.
5. `action`: notify, wait, hide feed, block navigation, show interstitial, stop autoplay, start cooldown, launch focus.
6. `exception`: explicitly approved allowlist, emergency bypass, grace quota.
7. `duration/reset`: one-time expiry, local daily reset, rolling window, weekly reset, recurring schedule.
8. `enforcement_method`: extension URL/DNR, in-page adapter, optional OS helper; select compatible adapters.
9. `reason` and `audit`: why, when, and with what outcome it executed.

See `schemas/automation-rule.example.json` for non-executable illustrative data.

## 3. Trigger/accounting definitions

**Usage minutes:** Based on valid foreground/active intervals after reconciliation, not number of open tabs. A shared budget spanning YouTube, Instagram and Reddit takes **union** of active intervals, never sums parallel background tabs.

**Scroll limit:** Counts qualifying plays by default; optional unique-video count. User sets whether replay counts; count resets must be explicit.

**Scheduled blocks:** Local timezone/calendar-based, persisted as UTC occurrence/expiry boundaries. Plan for DST, clock changes and missed schedules while agent/browser offline.

**Permanent:** Indefinite until user disables/deletes rule; not unremovable at OS level.

**Strict:** User precommits to harder overrides during a bounded session with an explicit emergency escape. Never trap someone in a broken state.

## 4. Difficulty presets

| Preset | Default behavior | Override policy |
|---|---|---|
| Gentle | Warning, delay, suggestion | Dismiss immediately |
| Balanced | Blocks after trigger with grace/cooldown | User-configured confirmation or cooldown |
| Strict | Locked session for duration where supported | Clearly shown emergency unlock path; logs reason; not unbypassable |

A rule's **duration** (temporary vs indefinite) and **difficulty** (Gentle/Balanced/Strict) are orthogonal. Do not conflate them.

## 5. Browser blocking and in-page restrictions

Primary enforcement for Brave/Chrome/Edge uses documented extension capabilities. `declarativeNetRequest` can block/redirect matching network requests. A content script can hide or disable selected page UI (e.g. Shorts shelf or recommended content) but selectors are unstable. Some in-app SPA transitions require in-page adapters; measure actual efficacy.

Preferred blocked-state experience: a local extension interstitial showing site, active rule, reason, remaining time, end condition and available override. Never silently close user tabs by default. Do not promise that a content script can block every possible embedded video or third-party app.

Partial blocking examples:

- Block `youtube.com/shorts/...`, allow standard `youtube.com/watch?v=...`.
- Hide YouTube homepage recommendations but allow Search, Subscriptions and approved playlists.
- Block Instagram Reels route while other Instagram areas remain accessible where technically feasible.
- During Focus Goal, hide autoplay and feed distractions while allowing direct tutorial URL.

## 6. OS-level blockers (optional only)

Windows Firewall FQDN/dynamic-keyword blocking is **not** a universal domain blocker. Microsoft notes dependencies on DNS observation, Defender Network Protection, DNS over HTTPS, VPN/proxy and cached IPs. Blocking at IP level can inadvertently affect shared CDN addresses. This helper must not be in initial MVP's critical path.

If added later:

- Obtain explicit admin permission and show exactly which rules/domains will be changed.
- Prefix/identify every owned firewall rule and record ownership + expiry for rollback.
- Validate on Windows versions/configurations; declare unsupported cases.
- Do not blanket-block system connectivity, Microsoft login, DNS or emergency domains.
- Uninstall/recovery deletes only rules owned by this application.
- Protect against crash/clock jump causing indefinitely stuck expired restrictions.

Reference: https://learn.microsoft.com/en-us/windows/security/operating-system-security/network-security/windows-firewall/dynamic-keywords

## 7. Conflict precedence and semantics

Suggested deterministic order:

1. Emergency recovery and mandatory protected exceptions.
2. Active user-created Strict deny (scoped by explicit target/profile/expiry).
3. Other active deny actions.
4. Explicit narrow allowance exceptions with compatible policies.
5. Soft reminders or recommendations.

Important: The narrowest/most specific exception may override a broader block **only if its owning rules allow exceptions**. Policy compiler should detect contradictory states at save time and explain effective outcome. No implicit “allow always wins” or “deny always wins” surprises.

## 8. Rule lifecycle

**Draft → Validated → Armed → Triggered → Enforced/Degraded → Expired/Disabled.** Persist rule version at trigger. Rules that require content adapter must reveal adapter status; if enforcement unavailable, mark degraded and optionally notify. A visual rule cannot report a success when the browser actually allowed the URL.

**Time expiry invariant:** an expired block must stop blocking automatically even if desktop UI is closed or background worker restarts. Browser-side active rule revalidation may be necessary. Define a maximum expiry horizon and sanity check local clock shifts.

**Rule transaction:** Save new rule → validate intended effect → compile browser-friendly policy → apply → await ack or timeout → persist effective health. On partial failure, roll back or label partial instead of asserting success.

## 9. Automation editor

Wizard for beginners; visual IF→AND→THEN editor for experts. Supports trigger templates and dry-run preview. Every action links to a plain-English explanation: “If qualified Shorts plays reach 15 between 9 AM and 1 PM, block Shorts for 1 hour on paired browsers.”

Defaults should be reversible and avoid closing documents. Notification cooldowns and rule dedup prevent multiple identical alerts.

## 10. Example scenarios

**Shorts cap:** Count qualifying plays (not impressions) across paired browsers; block Shorts after 15; reset at midnight in configured timezone; distinguish individual profile coverage.

**Shared budget:** YouTube + Instagram + Reddit active foreground usage combined reaches 45 minutes; stop further matching navigation until next calendar day. Missing browser coverage shown.

**Focus goal:** During chosen 45-minute focus, allow coding IDEs and explicitly allowlisted learning sites. A browser active in a project documentation page may qualify even though app is technically Brave.

**Bedtime:** 22:00–06:00 recurring, respect local timezone and restart; show expiry next morning.

**Indefinite block:** Domain remains blocked until disabled by user; document extension-based bypass limit.

**Doomscroll rule:** If >=12 qualifying Shorts within 15 minutes and no planned break, offer gentle prompt or user-configured Balanced block; must be opt-in.

## 11. Test scenarios

- Rule threshold boundary exact seconds/count, replays configured correctly.
- Browser crash/restart: rule remains consistent and expires.
- User closes GUI: restriction remains.
- Agent stops/restarts: expired rule cleared, stale events not applied.
- Two overlapping rules targeting same site: no duplicate firewall/DNR side effects.
- User disables tracking but blocking remains: explicitly configurable and clearly shown.
- Temporary block with excluded domain: allowlist respected only as declared.
- Browser with no extension: UI explicitly shows not protected.
- OS-level helper fails: browser blocking remains unaffected.
