import test from 'node:test';
import assert from 'node:assert/strict';
import {classifyDelivery,pruneQueue,withQuarantine,nextRetryDelay,MAX_QUEUE_BYTES,MAX_EVENT_BYTES,acknowledgement,safeDeliveryReason} from '../extension/src/queue-policy.js';
const item=(id,time)=>({event_id:'event-'+String(id).padStart(8,'0'),kind:'video.progress',occurred_at_ms:time});
test('network and authorization failures remain retryable',()=>{
 for(const code of [0,401,403,404,408,409,425,429,500,503])assert.equal(classifyDelivery(code),'retry');
 for(const code of [400,413,422])assert.equal(classifyDelivery(code),'terminal');
 assert.equal(classifyDelivery(200),'accepted');
});
test('expiry and overflow accounted for separately',()=>{
 const now=2_000_000_000_000;
 const items=[item('old',now-86_400_001),...Array.from({length:2004},(_,i)=>item('id'+i,now))];
 const out=pruneQueue(items,now);
 assert.equal(out.queue.length,2000);assert.equal(out.expired,1);assert.equal(out.overflow,4);
 assert.equal(out.queue[0].event_id,item('id4',now).event_id);
});
test('quarantine is bounded and contains metadata only',()=>{
 let q=[];for(let i=0;i<150;i++)q=withQuarantine(q,item('id'+i,Date.now()),'invalid_payload');
 assert.equal(q.length,100);assert.equal(q[0].event_id,item('id50',0).event_id);
 assert.equal(q[99].reason,'invalid_event_json');assert.equal('payload' in q[0],false);
});
test('retry delay is bounded',()=>{
 assert.equal(nextRetryDelay(0,()=>0.5),30000);assert.equal(nextRetryDelay(1,()=>0.5),60000);
 assert.notEqual(nextRetryDelay(2,()=>0),nextRetryDelay(2,()=>1));
 assert.ok(nextRetryDelay(50)<=300000);
});
test('queue byte and individual event bounds protect storage quota',()=>{
 const now=Date.now(),large={...item('large',now),payload:'x'.repeat(7000)};
 const out=pruneQueue(Array.from({length:400},(_,i)=>({...large,event_id:'event-large-'+i})),now);
 assert.ok(out.bytes<=MAX_QUEUE_BYTES);assert.ok(out.overflow>0);
 assert.ok(new TextEncoder().encode(JSON.stringify(out.queue)).byteLength<=MAX_QUEUE_BYTES);
 const oversized={...large,payload:'x'.repeat(MAX_EVENT_BYTES)};
 assert.equal(pruneQueue([null,oversized,item('good',now)],now).invalid,2);
});
test('acknowledgements distinguish ignored observations and reject a foreign event',()=>{
 assert.equal(acknowledgement({ok:true,event_id:'different'},'expected'),null);
 assert.equal(acknowledgement({ok:false},'expected'),null);
 assert.equal(acknowledgement({ok:true,disposition:'unexpected'},'expected'),null);
 assert.equal(acknowledgement({ok:true,duplicate:true},'expected'),'duplicate');
 assert.equal(acknowledgement({ok:true,ignored_due_to_pause:true},'expected'),'ignored_paused');
 assert.equal(acknowledgement({ok:true,ignored_due_to_exclusion:true},'expected'),'ignored_excluded');
});
test('quarantine redacts arbitrary HTTP bodies and expires old metadata',()=>{
 const now=Date.now(),privateValue='https://private.example/secret?api_key=secret';
 const out=withQuarantine([{at:now-8*86400000,payload:privateValue}],{event_id:privateValue,kind:privateValue,payload:privateValue},privateValue,now);
 assert.equal(out.length,1);assert.equal(JSON.stringify(out).includes('secret'),false);
 assert.equal(safeDeliveryReason(404,privateValue),'bridge_endpoint_unavailable');
});
