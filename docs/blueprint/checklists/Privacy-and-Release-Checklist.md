# Security, privacy and release readiness checklist

## Permissions and transparency

- [ ] Intro explains collected app/domain/video data in language ordinary users understand.
- [ ] Window titles and full URLs off by default unless specifically consented.
- [ ] Incognito tracking off by default, following browser support and privacy conventions.
- [ ] App/site exclusions, pause and retention controls are immediately accessible.
- [ ] External metadata research disabled by default; preview what is sent.
- [ ] BYOK and AI Coach optional, opt-in, data selection visible.
- [ ] Export and full/partial deletion tested against all derived tables.

## Browser bridge and data security

- [ ] Native host allows only specified extension IDs/origins.
- [ ] Messages validated, sanitized, rate-limited and bounded; no arbitrary commands.
- [ ] Sites cannot inject tool instructions into agent/LLM workflows.
- [ ] No password/cookie/form/clipboard/screenshot/audio capture as hidden feature.
- [ ] Windows credential storage protects keys; key removal tested.
- [ ] SQLite and export files restricted to appropriate user account.
- [ ] Sensitive paths/URLs/headers/API keys redacted from logs and crash reports.

## Blocking safety

- [ ] User can recover from any timed or strict restriction.
- [ ] Expired blocks always stop (test crash, sleep, clock change).
- [ ] Disabling extension/unpaired profiles represented honestly.
- [ ] Any elevated OS helper opt-in with exact rule ownership and rollback.
- [ ] Safe uninstall removes owned system changes only.

## Release process

- [ ] Primary Windows 10/11 environments tested.
- [ ] Brave compatibility confirmed, version recorded.
- [ ] Chrome/Edge support marked confirmed or explicitly unavailable.
- [ ] 24+ hour overhead and long-session accuracy benchmarked.
- [ ] Database upgrade/downgrade and recovery tested.
- [ ] Dependency licenses and updates reviewed.
- [ ] Installer and binaries signed where appropriate.
- [ ] Extension store policies and privacy disclosures reviewed.
- [ ] Users can report false positive classification and broken platform adapters.
