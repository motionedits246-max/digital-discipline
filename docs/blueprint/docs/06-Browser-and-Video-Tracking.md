# 06 — Browser extension and video instrumentation

## 1. Why an extension exists

Windows sees a browser process and top-level window but cannot reliably know which tab, URL, in-page SPA route or video player is active. A **Chromium Manifest V3 extension** is required for detailed Brave/Chrome/Edge behavior. Browser-wide permissions must be explained and narrowed where possible.

## 2. Extension modules

- **Browser session observer:** tabs, window focus, activation, navigation, incognito policy, profile identification and restricted domain data.
- **Web activity normalizer:** domain/route metadata, active tab state and allowable URL storage scope.
- **Site adapters:** site-specific short-video/player observations for YouTube Shorts and Instagram Reels initially. Later adapters independent and versioned.
- **Enforcement adapter:** DNR for eligible URL-level blocks, content script for page UI changes and user-approved partial controls.
- **Native bridge:** validated events to agent, state handshake and reconnect; local bounded queue for survivable temporary outages.
- **Health reporting:** which sites are supported, version, last detection and missing-permission state.

## 3. Extension API candidates

- `tabs` / `windows`: foreground tab and browser window tracking, within granted permissions.
- `webNavigation.onHistoryStateUpdated`: detect SPA history changes (e.g. Shorts switching without full page reload).
- `webNavigation` navigation events: confirm route transitions.
- `content_scripts`: detect visible player elements/attributes/playback state, respecting CSP/isolation.
- `declarativeNetRequest`: eligible domain/URL block and redirect rules.
- `storage` or IndexedDB: durable bounded extension state when service worker sleeps.
- `nativeMessaging`: bridge to installed agent; explicitly validate sender and payload.

MV3 service workers are not always running. Do **not** keep critical counters/timers only in service worker globals. Keep synced state or reconstruct from timestamped events. Persistent native host connections may influence lifetime, but engineering must support suspension/reconnect regardless.

## 4. Short-video event model

**Observed events:** platform video appeared (impression), playback started, pause, seek, timeupdate (rate-limited), visibility changed, active tab changed, video identity changed, playback ended, route changed.

**Main definitions:**

- **Impression:** element eligible/visible in viewport; not automatically a view.
- **Qualified play:** observed play reaches configurable threshold (proposed 2 cumulative seconds, subject to testing).
- **Unique video:** distinct normalized platform/video ID in selected reporting period.
- **Replay:** second qualifying play of same ID; distinguish within-session replay and later re-open.
- **Watch duration:** actual playback intervals as observed, excluding pauses; speed-adjusted playback clock vs wall-time should be separate if needed.
- **Estimated skip:** video navigation away before user-selected qualifying/completion threshold; only when evidence supports it.
- **Scroll streak:** chain of qualifying short videos with small inter-video gaps; interrupted by long inactivity or leaving platform.

These are measurement definitions, not official platform analytics, and should never be called guaranteed platform-authoritative “view counts.”

## 5. YouTube Shorts adapter

Detect Shorts route, current video identifier, player, `play`/`pause`/state and visibility. URL-only detection cannot distinguish a quick scroll from watching; adapters must observe player or DOM state. Track robustly across same-document navigation. Deduplicate same-ID events and avoid counting sub-2-second impressions as views.

Test: open Shorts directly, swipe rapidly, play/pause, change browser tabs, back/forward, autoplay, load old video again, watch tutorial page, play YouTube in picture-in-picture. Maintain a clear degraded status if video signals are absent.

## 6. Instagram Reels adapter

SPA routing, login states and player DOM may differ; do not rely on one permanent selector. Support only documented/accessible page data the user has permission to inspect through their own browser. Avoid defeating platform protections, restricted endpoints or private-content scraping. If DOM/player signals are insufficient, report unknown/estimated rather than fabricate exact views.

## 7. Browser coverage and attribution

- By default track **only browsers/profiles where extension is installed and enabled**.
- Distinguish browser identity and profile ID; never assume browser count means complete device-wide coverage.
- Active window changes cross-reference Windows and extension events to avoid duplicate active time.
- Incognito permission is separate and off by default; don't infer private browsing details from process metadata.
- URL recording modes: domain-only, path-enabled on allowlisted sites, full detail (explicit opt-in). Strip query parameters potentially carrying tokens, IDs, searches; canonicalize known routes carefully.
- Sites accessed through different browsers can count toward shared budget only when both provide compatible coverage; show coverage badge.

## 8. Blocking methods

- Entire domain/URL path: DNR rule wherever supported.
- User-facing block UI: redirect/interstitial when feasible, show reason and expiry.
- Partial UI: content script may hide Shorts shelf/recommendation widgets or stop a specific feed interaction; it is fragile to site changes.
- **Never auto-close tabs by default**; risk of unsaved work.
- Normal long-form video should remain allowed when only `/shorts/` is blocked.
- DNR does not prevent user from disabling extension or using an uninstrumented browser. Clearly disclose this.

## 9. Site adapter reliability protocol

Every adapter publishes platform, version, supported route patterns, observed signal quality, confidence, last successful detection and error code. Define a fixture suite for site DOM snapshots/routes and a manual playback regression test after site changes. Release a hotfix mechanism through normal browser store/update rules; no unsafe remote code loading.

## 10. Privacy and permissions

Do not capture page text indiscriminately. Do not read private messages, passwords, form contents or authentication headers. Keep metadata extraction limited to visible player identifiers/titles and site routing needed for chosen features. Never send raw browsing history to categorization services automatically.

## 11. Validation scenarios

- 20 fast-scroll impressions, 3 qualifying plays → 3 plays, not 20 views.
- Same Short qualified twice → unique 1, plays 2.
- Active browser with 12 tabs → only foreground tab gets foreground time.
- Browser minified/idle with audio in background → playback distinct from foreground use.
- User switches Brave to Chrome → no overlapping foreground sessions.
- Extension service worker suspended → reconstruct state correctly after wake.
- Site DOM updated → detector marks degraded/unknown instead of returning zeros and claiming accuracy.
