import test from 'node:test';
import assert from 'node:assert/strict';
import {classifyDelivery,pruneQueue,withQuarantine,nextRetryDelay} from '../extension/src/queue-policy.js';
const item=(id,time)=>({event_id:id,kind:'video.progress',occurred_at_ms:time});
test('network and authorization failures remain retryable',()=>{
 for(const code of [0,401,403,408,429,500,503])assert.equal(classifyDelivery(code),'retry');
 for(const code of [400,404,413,422])assert.equal(classifyDelivery(code),'terminal');
 assert.equal(classifyDelivery(200),'accepted');
});
test('expiry and overflow accounted for separately',()=>{
 const now=2_000_000_000_000;
 const items=[item('old',now-86_400_001),...Array.from({length:2004},(_,i)=>item('id'+i,now))];
 const out=pruneQueue(items,now);
 assert.equal(out.queue.length,2000);assert.equal(out.expired,1);assert.equal(out.overflow,4);
 assert.equal(out.queue[0].event_id,'id4');
});
test('quarantine is bounded and contains metadata only',()=>{
 let q=[];for(let i=0;i<150;i++)q=withQuarantine(q,item('id'+i,Date.now()),'invalid_payload');
 assert.equal(q.length,100);assert.equal(q[0].event_id,'id50');
 assert.equal(q[99].reason,'invalid_payload');assert.equal('payload' in q[0],false);
});
test('retry delay is bounded',()=>{
 assert.equal(nextRetryDelay(0),2000);assert.equal(nextRetryDelay(1),4000);
 assert.ok(nextRetryDelay(50)<=300000);
});
