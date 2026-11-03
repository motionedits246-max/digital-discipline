# 15 — Research sources and verification notes

**Reference check date:** 2026-11-03. These links are primary/official product or platform references, not a promise that behavior, API terms, quotas and pricing remain unchanged. Recheck on implementation.

## 1. Windows native collector and security

1. **Microsoft — `SetWinEventHook`**, foreground event-hook contract and message-loop requirements: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook
2. **Microsoft — `GetLastInputInfo`**, per-session input-idle signal and caveats: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getlastinputinfo
3. **Microsoft — `GetForegroundWindow`**, foreground window source: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow
4. **Microsoft — WinCred API**, Credential Manager integration: https://learn.microsoft.com/en-us/windows/win32/api/wincred/
5. **Microsoft — Windows Firewall dynamic keywords**, FQDN limitations and prerequisite configuration: https://learn.microsoft.com/en-us/windows/security/operating-system-security/network-security/windows-firewall/dynamic-keywords
6. **Microsoft — WinGet show**, app name/publisher/description/tags if submitted: https://learn.microsoft.com/windows/package-manager/winget/show

## 2. Tauri and desktop shell

7. **Tauri v2 architecture:** https://v2.tauri.app/concept/architecture/
8. **Tauri v2 configuration/window options:** https://v2.tauri.app/reference/config/
9. **Tauri v2 plugins:** https://v2.tauri.app/plugin/
10. **Tauri autostart documentation:** https://v2.tauri.app/plugin/autostart/

Caveat: overlay focus, click-through, always-on-top, multiple windows and WebView2 memory should be validated on target Windows builds rather than inferred from API names.

## 3. Browser extension and native bridge

11. **Chrome — Native Messaging:** https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging
12. **Chrome — MV3 service-worker lifecycle:** https://developer.chrome.com/docs/extensions/develop/concepts/service-workers/lifecycle
13. **Chrome — webNavigation events:** https://developer.chrome.com/docs/extensions/reference/api/webNavigation
14. **Chrome — declarativeNetRequest:** https://developer.chrome.com/docs/extensions/reference/api/declarativeNetRequest
15. **Chrome — runtime optional permissions:** https://developer.chrome.com/docs/extensions/reference/api/permissions
16. **Chrome — permission declarations:** https://developer.chrome.com/docs/extensions/develop/concepts/declare-permissions
17. **Chrome — protect user privacy:** https://developer.chrome.com/docs/extensions/develop/security-privacy/user-privacy

Read carefully: DNR dynamic rules can persist across browser sessions, whereas session rules are cleared under defined circumstances; MV3 workers can be suspended; Native Messaging must verify sender and host origins; optional host permissions let the user grant relevant sites when supported.

## 4. Structured output and BYOK AI

18. **OpenAI — Structured Outputs:** https://developers.openai.com/api/docs/guides/structured-outputs
19. **Google Gemini — Structured Outputs:** https://ai.google.dev/gemini-api/docs/structured-output
20. **Ollama — Structured Outputs:** https://ollama.com/blog/structured-outputs
21. **Ollama — API reference:** https://github.com/ollama/ollama/blob/main/docs/api.md
22. **Anthropic API documentation:** https://docs.anthropic.com/
23. **Groq API documentation:** https://console.groq.com/docs/overview
24. **OpenRouter API documentation:** https://openrouter.ai/docs

Model availability, tool/JSON support, context lengths, rates, usage reporting and prices are capability- and plan-dependent. Use capability negotiation and a local schema validator. No official vendor here provides guaranteed unmetered hosted AI for arbitrary commercial use.

## 5. Public application metadata lookup

25. **WinGet metadata:** https://learn.microsoft.com/windows/package-manager/winget/show
26. **MediaWiki API portal:** https://www.mediawiki.org/wiki/API:Main_page
27. **Brave Search API:** https://brave.com/search/api/

Treat metadata as untrusted. Consult terms, User-Agent expectations, rate limits and licensing before querying; do not assume every website allows programmatic scraping. Prefer official app metadata and direct APIs to search results.

## 6. Related existing product categories (market context, not dependencies)

- ActivityWatch: https://activitywatch.net/
- RescueTime: https://www.rescuetime.com/
- Freedom: https://freedom.to/
- LeechBlock NG: https://www.proginosko.com/leechblock/
- Rize: https://rize.io/

Validate feature/pricing comparisons fresh before publication or marketing claims; they change frequently. Our distinction is integration and measurement semantics, not “first app to track anything.”

## 7. Known uncertainty log

- Exact Brave Native Messaging registration/extension store interoperability must be tested on target builds.
- Instagram Reels player/page DOM is prone to change and may not expose all desired playback signals.
- Browser extensions can be disabled by the local user and do not cover unpaired browsers/profiles.
- Windows Firewall FQDN blocking cannot guarantee universal domain enforcement across DoH/VPN/proxy/cache.
- Public metadata/search availability is incomplete; some applications will remain Unknown.
- LLM confidence is not calibrated by default; do not turn it into a user-facing fact without evaluation.

## 8. Source citation convention for future product docs

When implementing a feature, add an ADR/testing entry including: exact official documentation URL, version/date checked, supported Windows/browser/API version, observed runtime behavior and any divergence. Do not rely on screenshot concept mockups as technical evidence.
