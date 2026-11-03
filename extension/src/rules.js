// Pure helper duplicated in unit tests: validate URL scopes before giving DNR a rule.
export function isValidDomain(value) {
  return typeof value === 'string' && value.length <= 253 && value.includes('.') && !/^(?:\d{1,3}\.){3}\d{1,3}$/.test(value) &&
    value.split('.').every(s => s.length > 0 && s.length <= 63 && !s.startsWith('-') && !s.endsWith('-') && /^[a-z0-9-]+$/.test(s));
}
export function safePrefix(value) {
  return value == null || (typeof value === 'string' && value.length < 160 && /^\/[a-zA-Z0-9/_~.%-]*$/.test(value) && !value.includes('..'));
}
export function matchesRule(urlString, rule) {
  try {
    if (!isValidDomain(rule.domain) || !safePrefix(rule.path_prefix)) return false;
    const url = new URL(urlString);
    if (!['http:', 'https:'].includes(url.protocol)) return false;
    const host = url.hostname.toLowerCase();
    if (host !== rule.domain && !host.endsWith('.' + rule.domain)) return false;
    return !rule.path_prefix || url.pathname.startsWith(rule.path_prefix);
  } catch { return false; }
}
export function regexFor(rule) {
  if (!isValidDomain(rule.domain) || !safePrefix(rule.path_prefix)) throw new Error('Invalid block target');
  const domain = rule.domain.replace(/\./g, '\\.');
  const path = rule.path_prefix ? rule.path_prefix.replace(/[.*+?^${}()|[\]\\]/g, '\\$&') : '';
  return `^https?://(?:[^./]+\\.)*${domain}(?::[0-9]+)?${path || '(?:[/?#]|$)'}`;
}
