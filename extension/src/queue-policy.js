// Shared delivery/retention policy. Never retain rejected event payloads.
export const MAX_QUEUE_EVENTS = 2000;
export const MAX_QUEUE_BYTES = 2_000_000;
export const MAX_EVENT_BYTES = 8192;
export const MAX_QUEUE_AGE_MS = 86_400_000;
export const MAX_QUARANTINE = 100;
export const MAX_QUARANTINE_AGE_MS = 7 * 86_400_000;
const kinds = new Set(['browser.active_tab', 'browser.window_blurred', 'browser.health', 'block.ack', 'video.progress', 'video.qualified_play', 'video.navigation']);
const reasons = new Set(['invalid_envelope', 'invalid_browser_payload', 'invalid_browser_domain', 'invalid_video_payload', 'invalid_video_data', 'invalid_block_ack', 'invalid_rule_count', 'unsupported_kind', 'extension_not_authorized_open_desktop_setup', 'extension_origin_required', 'loopback_host_required', 'invalid_origin', 'storage_failed', 'profile_revoked_authorize_in_desktop','invalid_profile_status','profile_identity_mismatch','event_identity_conflict','playback_identity_conflict']);
const encoder = new TextEncoder();

export function safeDeliveryReason(status, reason) {
  if (reasons.has(reason)) return reason;
  if (status === 413) return 'event_too_large';
  if (status === 400 || status === 422) return 'invalid_event_json';
  if (status === 404) return 'bridge_endpoint_unavailable';
  if (status === 0) return 'agent_unreachable';
  return 'agent_http_' + (Number.isInteger(status) ? status : 0);
}
export function classifyDelivery(status) {
  if (status === 200 || status === 201) return 'accepted';
  // 404 may indicate an older agent: it is not proof of a poison event.
  if ([400, 413, 422].includes(status)) return 'terminal';
  return 'retry';
}
export function acknowledgement(reply, eventId) {
  if (reply?.ok !== true) return null;
  if (reply.event_id != null && reply.event_id !== eventId) return null;
  const disposition = reply.disposition || (reply.duplicate ? 'duplicate' : reply.ignored_due_to_pause ? 'ignored_paused' : reply.ignored_due_to_exclusion ? 'ignored_excluded' : 'accepted');
  // Older 1.0 agents return ok plus legacy flags; keep in-place upgrades working.
  return ['accepted', 'duplicate', 'ignored_paused', 'ignored_excluded', 'ignored_private', 'ignored_deleted'].includes(disposition) ? disposition : null;
}
export function pruneQueue(input, now = Date.now()) {
  const queue = [], dropped = [];
  let bytes = 2, expired = 0, overflow = 0, invalid = 0;
  for (const item of Array.isArray(input) ? input : []) {
    let size = 0;
    try { size = encoder.encode(JSON.stringify(item)).byteLength + 1; } catch {}
    if (!item || typeof item.event_id !== 'string' || item.event_id.length < 8 || item.event_id.length > 80 || !Number.isFinite(item.occurred_at_ms) || !size || size > MAX_EVENT_BYTES) {
      invalid++; dropped.push({item, reason: 'invalid_queue_entry'}); continue;
    }
    if (item.occurred_at_ms <= now - MAX_QUEUE_AGE_MS) {
      expired++; dropped.push({item, reason: 'expired'}); continue;
    }
    queue.push({item, size}); bytes += size;
  }
  while (queue.length > MAX_QUEUE_EVENTS || bytes > MAX_QUEUE_BYTES) {
    const entry = queue.shift(); bytes -= entry.size;
    overflow++; dropped.push({item: entry.item, reason: 'overflow'});
  }
  return {queue: queue.map(x => x.item), bytes, dropped, expired, overflow, invalid};
}
function safeId(id) { return typeof id === 'string' && /^[a-zA-Z0-9_-]{8,80}$/.test(id) ? id : 'invalid_id'; }
export function pruneQuarantine(history, at = Date.now()) {
  return Array.isArray(history) ? history.filter(x => Number.isFinite(x?.at) && x.at > at - MAX_QUARANTINE_AGE_MS).map(x => ({
    event_id: safeId(x.event_id),
    kind: kinds.has(x.kind) ? x.kind : 'unknown',
    reason: reasons.has(x.reason) || ['expired', 'overflow', 'invalid_queue_entry', 'event_too_large', 'invalid_event_json'].includes(x.reason) ? x.reason : 'invalid_event_json',
    at: x.at
  })).slice(-MAX_QUARANTINE) : [];
}
export function withQuarantine(history, item, reason, at = Date.now()) {
  // Restrict metadata too: neither an HTTP body nor corrupt stored values are diagnostics.
  const code = /^(expired|overflow|invalid_queue_entry|event_too_large)$/.test(reason) ? reason : safeDeliveryReason(400, reason);
  return [...pruneQuarantine(history, at), {event_id: safeId(item?.event_id), kind: kinds.has(item?.kind) ? item.kind : 'unknown', reason: code, at}].slice(-MAX_QUARANTINE);
}
export function nextRetryDelay(attempt, random = Math.random) {
  const bounded = Math.max(0, Math.min(7, Number.isFinite(attempt) ? attempt : 0));
  const jitter = Math.max(0, Math.min(1, random()));
  return Math.min(300_000, Math.max(30_000, Math.round(30_000 * 2 ** bounded * (0.8 + jitter * 0.4))));
}
