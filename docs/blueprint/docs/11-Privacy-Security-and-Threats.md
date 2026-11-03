# 11 — Privacy, security and threat model

## 1. Security stance

Digital Discipline processes detailed app, website and potential video metadata. These records may reveal private habits, identities, accounts, health interests, searches or confidential work. The correct architecture is **local-first, minimized, consent-gated and auditable**. This is a self-control product for its owner, not covert surveillance.

## 2. Trust boundaries

1. Windows OS observation APIs → agent.
2. Untrusted websites and page DOM → content script/extension.
3. Extension service worker → native messaging host → local agent.
4. Agent → SQLite and Windows credential storage.
5. Agent → optional public metadata search or BYOK provider.
6. Agent → optional elevated OS blocking helper.
7. Local application UI → commands that may affect data and rules.
8. Optional export/backup → user-selected filesystem/cloud location.

Every boundary needs explicit schemas, validation, least privilege, origin allowlisting, rate limits and safe error handling.

## 3. Data classification

| Data | Sensitivity | Default handling |
|---|---|---|
| App display name, publisher | Medium | Local storage with configurable retention |
| Full executable path/window title | Medium/high | Avoid or redact by default; per-feature consent |
| Domains | High for many contexts | Optional tracking scope; sensitive-site exclusions |
| Full URL/query/search text | High | Off by default; sanitize paths, tokens and query strings |
| Video ID/title/view history | High | Detailed tracking opt-in; bounded retention |
| Goal and behavior profiles | Medium/high | Local, delete/export control |
| API key | Secret | OS credential storage only |
| Derived aggregate day/hour totals | Lower but still personal | Local, user controls retention |
| Crash/diagnostic logs | Potentially high | Redact/rotate/short TTL; no silent remote upload |

## 4. Primary threat scenarios

**T1: Malicious webpage sends fake events or commands.** A content script shares an environment with untrusted page context. Validate sender origin in extension context and again validate native payloads. Never accept raw “run shell command” / “change firewall” actions from webpages.

**T2: Another local process impersonates a native bridge client.** Restrict IPC with current-user access controls, per-install secret/challenge or OS identity as appropriate, and strict message vocabulary. Native messaging host registration should allow only approved extension IDs. Do not expose unauthenticated localhost HTTP endpoints.

**T3: AI prompt injection in app descriptions or video titles.** Treat all title/website/vendor text as data, not instructions. Limit outbound payloads, constrain classification schema, validate response and require deterministic user confirmation for consequential actions.

**T4: API key theft.** Store via Windows Credential Manager; avoid keys in URLs, local config, logs, exports or crash reports. Do not log HTTP Authorization headers or model full prompts by default.

**T5: Sensitive browsing exposure.** Domain-only default, exclusion lists, incognito default-off, raw URL sanitization and privacy mode. Use UI badges to disclose which scope is currently monitored.

**T6: Improper blocking harms connectivity.** Restrict normal blocking to extension routes initially. Optional elevated helper limits operations to owned rules, validates targets, provides rollback and expiry watchdog. No arbitrary firewall IP blanket blocks.

**T7: Data loss/corruption.** SQLite transaction/journaling strategy, backups before migration, bounded queue, idempotent replay and integrity check. Never rewrite raw events based solely on an AI output.

**T8: Unsafe app updater/dependency.** Signed/reputable releases, dependency scanning, reproducible builds if feasible, extension store policies, least privilege, no remote code injection.

**T9: Shared/local account risk.** Data directory restricted to current Windows user account; avoid global-readable telemetry. If multiple users use same computer, segregate tracking per user session.

**T10: Strict mode traps user.** Clear expiry, emergency override and safe uninstall. Never make recoverability depend solely on a network API or a single unresponsive process.

## 5. Explicit nonfeatures

Never implement without fresh explicit user approval: stealth monitoring, keystroke/clipboard collection, microphone activation, hidden screen capture, browser-cookie capture, login interception, anti-uninstall persistence, security-product disabling, bypass of OS/browser protections, private-account scraping or sending another user's data to third parties.

## 6. Permission and consent model

Separate switches, each with an explanation and accessible revocation:

- Windows app/process names, basic elapsed time.
- Window title recording (optional).
- Domain-level browser recording.
- Detailed URL route history (optional).
- Video ID/title/watch observations (optional).
- Automatic app metadata search (off by default).
- Remote AI classification (off by default).
- AI Coach send-data levels (off by default).
- OS-level blocking (off by default, elevated if enabled).
- Cross-device linking (future, off by default).

“Pause tracking” suspends collection and explains whether already-saved rule enforcement continues under the separate Blocking toggle. No deceptive partially-on/off behavior.

## 7. Threat-modeling checklist

For each new feature, ask: what can be observed, what can be sent, who can read it, which elevated action can it trigger, can it be spoofed, what happens on failure, can it be reversed, and does the user know? Record mitigations and tests. Particular attention to browser extension host permissions and OS firewall changes.

## 8. Legal/platform policies

Do not scrape sites that prohibit it or bypass authorization/anti-bot controls. Use licensed public metadata sources, documented APIs and users' own browser observations within granted permissions. Review extension store policies, third-party API terms, privacy notices and data export/deletion duties for any public release. This document is engineering guidance, not legal advice.

## 9. Security quality gates

- No API keys/raw private URLs in default logs.
- IPC origin spoofing tests fail closed.
- Agent cannot be driven to arbitrary shell/file/network execution from extension input.
- Model prompt injection cannot change block policy or reveal keys.
- Expired rules reliably release after crash/restart.
- Per-user data directories protected; delete/export works.
- Optional elevated helper is absent from baseline MVP installation.
- Documented disclosure of extension/OS blocking bypass limitations.
