// Minimal, testable queue semantics for the MV3 browser companion.
// Terminal schema errors are quarantined locally; transport failures are retried.
export const MAX_QUEUE_EVENTS=2000;
export const MAX_QUEUE_AGE_MS=86_400_000;
export const MAX_QUARANTINE=100;
export function classifyDelivery(status,reason='') {
  if (status===200||status===201) return 'accepted';
  // Unauthorized/revoked pairing is recoverable and must not destroy observations.
  if (status===401||status===403||status===408||status===409||status===425||status===429||status>=500||status===0) return 'retry';
  if (status===400||status===404||status===413||status===422) return 'terminal';
  return 'retry';
}
export function pruneQueue(queue,now=Date.now()){
  const accepted=queue.filter(x=>x&&typeof x.event_id==='string'&&Number.isFinite(x.occurred_at_ms));
  const expired=accepted.filter(x=>x.occurred_at_ms<=now-MAX_QUEUE_AGE_MS).length;
  const fresh=accepted.filter(x=>x.occurred_at_ms>now-MAX_QUEUE_AGE_MS);
  const overflow=Math.max(0,fresh.length-MAX_QUEUE_EVENTS);
  return {queue:fresh.slice(-MAX_QUEUE_EVENTS),expired,overflow,invalid:queue.length-accepted.length};
}
export function withQuarantine(history,item,reason,at=Date.now()){
  const sanitized=String(reason).slice(0,80).replace(/[^a-zA-Z0-9_:-]/g,'_');
  return [...history,{event_id:item.event_id,kind:item.kind||'unknown',reason:sanitized,at}].slice(-MAX_QUARANTINE);
}
export function nextRetryDelay(attempt){
  const bounded=Math.max(0,Math.min(7,Number.isFinite(attempt)?attempt:0));
  return Math.min(5*60_000,2000*2**bounded);
}
