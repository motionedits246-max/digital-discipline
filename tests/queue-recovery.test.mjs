import test from 'node:test';
import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
import path from 'node:path';

let workerVersion=0;
const observation=(id=crypto.randomUUID(),occurred=Date.now())=>({
 schema_version:'1.0',source:'extension',event_id:id,occurred_at_ms:occurred,
 kind:'video.progress',payload:{platform:'youtube_shorts',video_id:'example123',instance_id:crypto.randomUUID(),watched_ms:2300}
});
async function worker(state={},deliver){
 const handlers={},alarms=new Map(),requests=[],notifications=[];
 const event=name=>({addListener(fn){handlers[name]=fn;}});
 globalThis.chrome={
  runtime:{id:'a'.repeat(32),getURL:p=>'chrome-extension://'+'a'.repeat(32)+'/'+p,getManifest:()=>({version:'0.3.9'}),reload:()=>{state.reloads=(state.reloads||0)+1;},onInstalled:event('install'),onStartup:event('startup'),onMessage:event('message')},
  storage:{local:{async get(keys){return structuredClone(Object.fromEntries(keys.map(k=>[k,state[k]])));},async set(changes){
   if(state.failAckWrite&&changes.eventQueue?.length===0&&changes.queueDiagnostics?.last_delivery_at_ms)throw Error('disk error');
   Object.assign(state,structuredClone(changes));
  }}},
  alarms:{async create(name,opts){alarms.set(name,opts);},async clear(name){alarms.delete(name);},onAlarm:event('alarm')},
  tabs:{onActivated:event('tab'),onUpdated:event('update'),async query(){return [];}},
  windows:{onFocusChanged:event('focus'),async getLastFocused(){return {focused:false};}},
  webNavigation:{onHistoryStateUpdated:event('navigation')},
  declarativeNetRequest:{async updateDynamicRules(){}},notifications:{async create(id,body){notifications.push(body);}}
 };
 globalThis.fetch=async(url,options)=>{
  if(url.endsWith('/bridge/companion/update')){state.updateCalls=(state.updateCalls||0)+1;return Response.json(state.updateReply||{ok:true,version:'0.3.9',reload_ready:true});}
  if(url.endsWith('/bridge/companion/status'))return Response.json({ok:true,version:'0.3.9'});
  if(url.endsWith('/bridge/sync'))return Response.json({ok:true,schema_version:'1.0',blocks:[],...(state.syncReply||{})});
  const e=JSON.parse(options.body);
  if(e.kind==='block.ack')return Response.json({ok:true,event_id:e.event_id});
  requests.push(e);
  return deliver?deliver(e):Response.json({ok:true,event_id:e.event_id,disposition:'accepted'});
 };
 await import(pathToFileURL(path.resolve('extension/src/background.js')).href+'?worker='+workerVersion++);
 await new Promise(resolve=>setImmediate(resolve));
 const message=(body,sender={})=>new Promise((resolve,reject)=>{
  if(handlers.message(body,sender,resolve)!==true)reject(Error('Response channel was not held'));
 });
 return {state,requests,alarms,notifications,message,installed:()=>handlers.install({reason:'update'}),alarm:()=>handlers.alarm({name:'queue-retry'})};
}

test('poison and expired entries do not block later valid observations',async()=>{
 const bad=observation(),valid=observation(),expired=observation(crypto.randomUUID(),Date.now()-86400001);
 const w=await worker({eventQueue:[null,expired,bad,valid]},e=>e.event_id===bad.event_id?
  new Response('invalid_video_data',{status:400}):Response.json({ok:true,event_id:e.event_id}));
 await w.message({op:'resync'});
 assert.equal(w.state.eventQueue.length,0);
 assert.equal(w.state.queueDiagnostics.expired,1);assert.equal(w.state.queueDiagnostics.invalid,1);
 assert.equal(w.state.queueDiagnostics.rejected,1);assert.equal(w.state.queueDiagnostics.accepted,1);
 assert.equal(w.state.eventQuarantine.length,3);
 assert.ok(w.requests.some(e=>e.event_id===valid.event_id));
 assert.equal(JSON.stringify(w.state.eventQuarantine).includes('video_id'),false);
});
test('HTTP 404 and foreign acknowledgements retain data for a compatible agent',async()=>{
 for(const response of [()=>new Response('old agent',{status:404}),()=>Response.json({ok:true,event_id:'wrong-event'})]){
  const w=await worker({eventQueue:[observation()]},response);
  await w.message({op:'resync'});
  assert.equal(w.state.eventQueue.length,1);assert.equal(w.state.eventQuarantine.length,0);
  assert.equal(w.state.queueDiagnostics.retry_attempt,1);
  assert.ok(w.alarms.has('queue-retry'));
 }
});
test('lost acknowledgement replays a stable event ID without duplicating the server result',async()=>{
 const committed=new Set();let lose=true;
 const w=await worker({eventQueue:[observation()]},e=>{
  const duplicate=committed.has(e.event_id);committed.add(e.event_id);
  if(lose){lose=false;throw Error('connection dropped after commit');}
  return Response.json({ok:true,event_id:e.event_id,disposition:duplicate?'duplicate':'accepted'});
 });
 await w.message({op:'resync'});assert.equal(w.state.eventQueue.length,1);
 await w.message({op:'resync'});
 assert.equal(w.state.eventQueue.length,0);assert.equal(committed.size,1);
 assert.equal(w.state.queueDiagnostics.duplicate,1);
 assert.equal(w.requests[0].event_id,w.requests[1].event_id);
});
test('failed local acknowledgement write keeps the observation replayable',async()=>{
 const w=await worker({eventQueue:[observation()],failAckWrite:true});
 const failed=await w.message({op:'resync'});
 assert.equal(failed.ok,false);assert.equal(w.state.eventQueue.length,1);
 w.state.failAckWrite=false;
 await w.message({op:'resync'});assert.equal(w.state.eventQueue.length,0);
 assert.equal(w.requests[0].event_id,w.requests[1].event_id);
});
test('worker restart restores backoff and its wake alarm; manual retry is available',async()=>{
 const state={eventQueue:[observation()]};
 const w=await worker(state,()=>{throw Error('offline');});
 await w.message({op:'resync'});
 const retryAt=state.queueDiagnostics.retry_at_ms;
 const restarted=await worker(state);
 await restarted.alarm();
 assert.equal(restarted.requests.length,0);assert.equal(state.queueDiagnostics.retry_at_ms,retryAt);
 assert.ok(restarted.alarms.get('queue-retry').when>=retryAt);
 await restarted.message({op:'resync'});
 assert.equal(state.eventQueue.length,0);assert.equal(state.queueDiagnostics.retry_attempt,0);
});
test('recording-off retains pending data and rejects new playback; purge is explicit',async()=>{
 const w=await worker({eventQueue:[observation()],collectionEnabled:true},()=>{throw Error('offline');});
 await w.message({op:'collectionEnable',value:false});
 assert.equal(w.state.collectionEnabled,false);assert.equal(w.state.eventQueue.length,2);
 const sender={frameId:0,tab:{url:'https://www.youtube.com/shorts/AbCdEfGh123'}};
 const reply=await w.message({op:'video',kind:'video.qualified_play',page_url:sender.tab.url,instance_id:crypto.randomUUID(),watched_ms:2300},sender);
 assert.equal(reply.error,'recording_off');assert.equal(w.state.eventQueue.length,2);
 const purge=await w.message({op:'purgePending'});
 assert.equal(purge.removed,2);assert.equal(w.state.eventQueue.length,0);
 assert.equal(w.state.queueDiagnostics.purged,2);assert.equal(w.state.collectionEnabled,false);
});
test('pause and exclusion receipts are distinct from accepted playback',async()=>{
 let i=0;
 const w=await worker({eventQueue:[observation(),observation()]},e=>Response.json({ok:true,event_id:e.event_id,disposition:i++?'ignored_excluded':'ignored_paused'}));
 await w.message({op:'resync'});
 assert.equal(w.state.eventQueue.length,0);
 assert.equal(w.state.queueDiagnostics.ignored_paused,1);assert.equal(w.state.queueDiagnostics.ignored_excluded,1);
 assert.equal(w.state.queueDiagnostics.accepted,undefined);assert.equal(w.state.queueDiagnostics.last_accepted_at_ms,undefined);
});
test('large backlog yields bounded batches and continues on a browser wake',async()=>{
 const w=await worker({eventQueue:Array.from({length:60},()=>observation())});
 await w.message({op:'resync'});
 assert.equal(w.requests.length,50);assert.equal(w.state.eventQueue.length,10);assert.ok(w.alarms.has('queue-retry'));
 await w.alarm();assert.equal(w.state.eventQueue.length,0);assert.equal(w.requests.length,60);
});
test('expired recovery metadata is removed even when there are no new rejections',async()=>{
 const w=await worker({eventQueue:[],eventQuarantine:[{at:Date.now()-8*86400000,event_id:'old-event-id',kind:'video.progress',reason:'expired'}]});
 await w.message({op:'resync'});assert.deepEqual(w.state.eventQuarantine,[]);
});

test('desktop erasure clears companion history metadata without changing profile or recording consent',async()=>{
 const cutoff=Date.now()-5000,old=observation(crypto.randomUUID(),cutoff-1),recent=observation(),profile=crypto.randomUUID();
 const w=await worker({collectionEnabled:true,profileId:profile,eventQueue:[old,recent],videoObserved:{shorts:4},eventQuarantine:[{event_id:old.event_id,kind:old.kind,reason:'invalid_envelope',at:cutoff-100}],syncReply:{purge_before_ms:cutoff}});
 await w.message({op:'resync'});assert.equal(w.state.profileId,profile);assert.equal(w.state.collectionEnabled,true);assert.equal(w.state.videoObserved.shorts,0);assert.equal(w.state.eventQuarantine.length,0);assert.equal(w.requests.some(e=>e.event_id===old.event_id),false);assert.ok(w.requests.some(e=>e.event_id===recent.event_id));
});
test('agent private window suppresses new companion playback without revoking consent',async()=>{
 const w=await worker({collectionEnabled:true,eventQueue:[],syncReply:{private_until_ms:Date.now()+60000}});await w.message({op:'resync'});
 const sender={frameId:0,tab:{url:'https://www.youtube.com/shorts/Example123'}};
 const reply=await w.message({op:'video',kind:'video.qualified_play',page_url:sender.tab.url,instance_id:crypto.randomUUID(),watched_ms:2300},sender);assert.equal(reply.error,'private_session_active');assert.equal(w.state.eventQueue.length,0);assert.equal(w.state.collectionEnabled,true);
});

test('presentation mode removes private rule labels from delivered browser notifications',async()=>{
 const w=await worker({notificationsEnabled:true,syncReply:{presentation_private:true,blocks:[{id:'private-rule',domain:'www.youtube.com',action:'notify',reason:'Private rule label',expires_at_ms:Date.now()+60000}]}});await w.message({op:'resync'});assert.equal(w.notifications.length,1);assert.equal(w.notifications[0].message.includes('Private rule label'),false);assert.match(w.notifications[0].message,/approved browser policy/);
});

test('companion update reloads in place while preserving profile, consent and pending observations',async()=>{
 const queued=observation(),profile=crypto.randomUUID();const w=await worker({eventQueue:[queued],profileId:profile,collectionEnabled:true},async()=>{throw Error('offline during update');});
 const reply=await w.message({op:'companionUpdate'},{id:'a'.repeat(32),url:'chrome-extension://'+'a'.repeat(32)+'/popup.html'});
 assert.equal(reply.reload_ready,true);assert.deepEqual(w.state.eventQueue,[queued]);assert.equal(w.state.profileId,profile);assert.equal(w.state.collectionEnabled,true);
 await new Promise(resolve=>setTimeout(resolve,500));assert.equal(w.state.reloads,1);await w.installed();assert.ok(w.state.eventQueue.some(e=>e.event_id===queued.event_id));assert.equal(w.state.profileId,profile);assert.equal(w.state.collectionEnabled,true);
});
test('content scripts cannot trigger file updates and a failed update never reloads',async()=>{
 const w=await worker({updateReply:{ok:false,error:'update_failed'}});
 const blocked=await w.message({op:'companionUpdate'},{id:'a'.repeat(32),url:'https://www.youtube.com/watch?v=example',tab:{id:1}});
 assert.equal(blocked.error,'popup_action_required');assert.equal(w.state.updateCalls,undefined);
 const result=await w.message({op:'companionUpdate'},{id:'a'.repeat(32),url:'chrome-extension://'+'a'.repeat(32)+'/popup.html'});
 assert.equal(result.ok,false);await new Promise(resolve=>setTimeout(resolve,500));assert.equal(w.state.reloads,undefined);
});
