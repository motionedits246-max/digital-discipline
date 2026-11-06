import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
const listeners={};
const state={eventQueue:[],blocks:[],ownedIds:[],emergencyUntil:0,collectionEnabled:true};
let authorized=false,reachable=true;
const network=[];
const simpleEvent=()=>({addListener:()=>{}});
globalThis.fetch=async (url,options)=>{
  network.push({url,options});
  if(!reachable)throw new Error('Failed to fetch');
  if(!authorized)return new Response('extension_not_authorized_open_desktop_setup',{status:403});
  if(url.endsWith('/bridge/sync'))return Response.json({ok:true,schema_version:'1.0',blocks:[]});
  if(url.endsWith('/bridge/event'))return Response.json({ok:true});
  return new Response('wrong route',{status:404});
};
globalThis.chrome={
 runtime:{id:'diakokddlmicacfakghilkajnojloick',onInstalled:simpleEvent(),onStartup:simpleEvent(),onMessage:{addListener(fn){listeners.onMessage=fn;}}},
 storage:{local:{async get(keys){return Object.fromEntries(keys.map(k=>[k,state[k]]));},async set(changes){Object.assign(state,changes);}}},
 alarms:{create(){},async clear(){},onAlarm:simpleEvent()},tabs:{onActivated:simpleEvent(),onUpdated:simpleEvent(),async query(){return [];}},
 windows:{onFocusChanged:simpleEvent()},webNavigation:{onHistoryStateUpdated:simpleEvent()},
 declarativeNetRequest:{async updateDynamicRules(){}},notifications:{async create(){}},
};
await import(pathToFileURL(path.resolve('extension/src/background.js')).href);
function msg(op){return new Promise(resolve=>{
 const keep=listeners.onMessage({op},{},resolve);
 if(keep!==true)throw Error('Async response channel not held');
});}
test('extension not authorized: direct localhost reply is not mistaken for success',async()=>{
 const result=await msg('resync');
 assert.equal(result.ok,false);assert.match(result.error,/not_authorized/);
 const status=await msg('getStatus');assert.equal(status.connected,false);
 assert.match(status.lastConnectionError,/not_authorized/);
});
test('agent offline: error remains visible and queued events are preserved',async()=>{
 reachable=false;
 const result=await msg('resync');assert.equal(result.ok,false);
 assert.equal(result.error,'agent_unreachable');
 state.eventQueue=[{schema_version:'1.0',source:'extension',event_id:crypto.randomUUID(),occurred_at_ms:Date.now(),kind:'video.progress',payload:{platform:'youtube_shorts',video_id:'abc123',instance_id:crypto.randomUUID(),watched_ms:4300}}];
 const result2=await msg('resync');assert.equal(result2.ok,false);
 assert.equal(state.eventQueue.length,1);
 reachable=true;
});
test('authorized direct transport reconnects and flushes prior queue',async()=>{
 authorized=true;
 const result=await msg('resync');assert.equal(result.ok,true);
 const status=await msg('getStatus');assert.equal(status.connected,true);
 assert.equal(status.queued,0);assert.equal(state.eventQueue.length,0);
 assert.ok(network.some(x=>x.url.endsWith('/bridge/event')));
 assert.ok(network.every(x=>x.url.startsWith('http://127.0.0.1:48761/')));
 assert.ok(network.every(x=>x.options.credentials==='omit'));
});
test('regular YouTube video is counted separately from Shorts',async()=>{
 const sender={tab:{url:'https://www.youtube.com/watch?v=AbCdEfGh123'},frameId:0};
 const result=await new Promise(resolve=>listeners.onMessage({op:'video',kind:'video.qualified_play',watched_ms:2600,instance_id:crypto.randomUUID(),page_url:sender.tab.url},sender,resolve));
 assert.equal(result.ok,true);
 const delivered=network.find(x=>x.url.endsWith('/bridge/event') && JSON.parse(x.options.body).payload?.platform==='youtube_watch');
 assert.ok(delivered);assert.equal(JSON.parse(delivered.options.body).payload.video_id,'AbCdEfGh123');
});

test('manifest uses valid localhost host pattern without nativeMessaging',async()=>{
 const manifest=JSON.parse((await import('node:fs')).readFileSync('extension/manifest.json','utf8'));
 assert.ok(manifest.host_permissions.includes('http://127.0.0.1/*'));
 assert.equal(manifest.permissions.includes('nativeMessaging'),false);
 assert.ok(manifest.host_permissions.every(x=>!x.includes(':48761')));
});
