import {shouldNotify} from "./notifications.js";
import {classifyDelivery, acknowledgement, safeDeliveryReason, pruneQueue, pruneQuarantine, withQuarantine, nextRetryDelay} from './queue-policy.js';
import {isValidDomain,safePrefix,matchesRule,regexFor} from "./rules.js";
// The master agent token never enters extension storage or browser pages.
const BRIDGE = 'http://127.0.0.1:48761';
const RULE_ID_START = 800000;
const RETRY_ALARM = 'queue-retry';
let connected = false;
let syncInFlight = null;
let lastConnectionError = '';
let lastQueueError = '';
let lastNativeReplyAt = 0;
let lastConnectAttemptAt = 0;
let adapterState='unknown';let adapterAt=0;let foregroundTimer=null;
function domainValid(value){return isValidDomain(value);}
function ruleMatches(url,rule){return matchesRule(url,rule);}
async function storage() { return chrome.storage.local.get(['eventQueue', 'eventQuarantine', 'queueDiagnostics', 'blocks', 'ownedIds', 'emergencyUntil', 'browserKind', 'collectionEnabled','notificationsEnabled','dndEnabled','quietStart','quietEnd','lastNotifications','videoObserved']); }
async function request(message) {
  lastConnectAttemptAt=Date.now();
  const sync=message?.op==='sync';
  try {
    let syncBody='{}';
    if(sync){const opts=await storage(),d=opts.queueDiagnostics||{};syncBody=JSON.stringify({profile_id:await profileId(),browser:await browserInfo(),recording_enabled:opts.collectionEnabled===true,queued:(opts.eventQueue||[]).length,
      adapter:opts.collectionEnabled!==true?'recording_off':Date.now()-adapterAt<30000?adapterState:'unknown',qualified_at_ms:opts.videoObserved?.lastAt||0,
      companion_version:chrome.runtime.getManifest?.().version||'0.3.6',loss_count:(d.expired||0)+(d.overflow||0)+(d.invalid||0)+(d.rejected||0)});}
    const response=await fetch(BRIDGE+(sync?'/bridge/sync':'/bridge/event'),{
      method:'POST',headers:{'X-DD-Bridge':'1','Content-Type':'application/json'},
      body:sync?syncBody:JSON.stringify(message.event),cache:'no-store',credentials:'omit',signal:AbortSignal.timeout(6000)
    });
    const raw=await response.text();
    let reply;
    try{reply=JSON.parse(raw);}catch{reply={ok:false,error:raw.trim()};}
    if(!response.ok || reply?.ok!==true){
      connected=[400,413,422].includes(response.status);
      lastConnectionError=safeDeliveryReason(response.status,reply?.error);
      return {ok:false,error:lastConnectionError,http_status:response.status,delivery:classifyDelivery(response.status)};
    }
    connected=true;lastConnectionError='';lastNativeReplyAt=Date.now();
    return reply;
  } catch {
    connected=false;lastConnectionError='agent_unreachable';
    return {ok:false,error:lastConnectionError,delivery:'retry'};
  }
}
let queueOps = Promise.resolve();
function withQueueLock(fn) {
  const next = queueOps.then(fn, fn);
  queueOps = next.catch(() => {});
  return next;
}
const count = value => Number.isSafeInteger(value) && value>=0 ? value : 0;
function queueUpdate(state, input) {
  const cleaned=pruneQueue(input), previous=state.queueDiagnostics||{};
  return {
    eventQueue:cleaned.queue,
    eventQuarantine:cleaned.dropped.reduce((q,x)=>withQuarantine(q,x.item,x.reason),pruneQuarantine(state.eventQuarantine)),
    queueDiagnostics:{...previous,
      expired:count(previous.expired)+cleaned.expired,
      overflow:count(previous.overflow)+cleaned.overflow,
      invalid:count(previous.invalid)+cleaned.invalid,
      bytes:cleaned.bytes}
  };
}
function newEvent(kind,payload,profile) {
  return {schema_version:'1.0',source:'extension',event_id:crypto.randomUUID(),occurred_at_ms:Date.now(),kind,payload,profile_id:profile};
}
async function pushEvent(kind, payload) {
  const saved=await withQueueLock(async () => {
    const state=await storage();
    // Check consent in the same serialized operation as the queue write.
    if(state.collectionEnabled!==true)return false;
    await chrome.storage.local.set(queueUpdate(state,[...(state.eventQueue||[]),newEvent(kind,payload,await profileId())]));
    return true;
  });
  if(saved)await drainQueue();
}
function wakeQueue(when=Date.now()+30_000) {
  // Browser alarms can be delayed, including across sleep. Retention is rechecked on wake.
  return chrome.alarms.create(RETRY_ALARM,{when:Math.max(Date.now()+30_000,when)});
}
async function restoreRetryAlarm() {
  const {eventQueue=[],queueDiagnostics={}}=await storage();
  if(eventQueue.length)await wakeQueue(queueDiagnostics.retry_at_ms||0);
}
let drainInFlight = null;
function drainQueue({force=false}={}) {
  if(drainInFlight)return drainInFlight;
  drainInFlight=(async()=>{
    await withQueueLock(async()=>{
      const state=await storage();
      await chrome.storage.local.set(queueUpdate(state,state.eventQueue||[]));
    });
    const initial=await storage();
    if(!force && initial.queueDiagnostics?.retry_at_ms>Date.now()){
      if(initial.eventQueue?.length)await wakeQueue(initial.queueDiagnostics.retry_at_ms);
      return;
    }
    const started=Date.now();
    let processed=0;
    while(processed<50 && Date.now()-started<10_000) {
      const item=await withQueueLock(async()=> (await storage()).eventQueue?.[0]);
      if(!item)break;
      const reply=await request({op:'event',event:item});
      const disposition=acknowledgement(reply,item.event_id);
      if(!disposition && reply.delivery!=='terminal'){
        await withQueueLock(async()=>{
          const {queueDiagnostics={}}=await storage();
          const attempt=count(queueDiagnostics.retry_attempt);
          const retryAt=Date.now()+nextRetryDelay(attempt);
          await chrome.storage.local.set({queueDiagnostics:{...queueDiagnostics,
            retry_attempt:Math.min(8,attempt+1),retry_at_ms:retryAt,
            last_error:reply.ok?'invalid_acknowledgement':reply.error||'agent_unreachable'}});
          await wakeQueue(retryAt);
        });
        return;
      }
      // Queue removal and its diagnostics are one durable local write. If this write
      // fails after the agent committed, replay the SAME ID and accept its duplicate ACK.
      await withQueueLock(async()=>{
        const state=await storage(), diagnostics=state.queueDiagnostics||{};
        // A user may have purged this observation while it was in flight.
        if(!(state.eventQueue||[]).some(e=>e?.event_id===item.event_id))return;
        const field=disposition||'rejected';
        const next={...diagnostics,[field]:count(diagnostics[field])+1,
          retry_attempt:0,retry_at_ms:0,last_error:disposition?'':reply.error,
          last_delivery_at_ms:Date.now()};
        if(disposition==='accepted'||disposition==='duplicate')next.last_accepted_at_ms=Date.now();
        const eventQueue=state.eventQueue.filter(e=>e?.event_id!==item.event_id);
        const update={eventQueue,queueDiagnostics:{...next,bytes:pruneQueue(eventQueue).bytes}};
        if(!disposition)update.eventQuarantine=withQuarantine(state.eventQuarantine,item,reply.error);
        await chrome.storage.local.set(update);
      });
      processed++;
    }
    lastQueueError='';
    const remaining=(await storage()).eventQueue||[];
    if(remaining.length)await wakeQueue();
    else await chrome.alarms.clear(RETRY_ALARM);
  })().catch(error=>{lastQueueError='queue_storage_failed';throw error;}).finally(()=>{drainInFlight=null;});
  return drainInFlight;
}
async function purgePending() {
  return withQueueLock(async()=>{
    const {eventQueue=[],queueDiagnostics={}}=await storage();
    await chrome.storage.local.set({eventQueue:[],queueDiagnostics:{...queueDiagnostics,
      purged:count(queueDiagnostics.purged)+eventQueue.length,retry_attempt:0,retry_at_ms:0,bytes:2}});
    await chrome.alarms.clear(RETRY_ALARM);
    return eventQueue.length;
  });
}
let profilePromise=null;
function profileId(){if(profilePromise)return profilePromise;profilePromise=makeProfileId().catch(error=>{profilePromise=null;throw error;});return profilePromise;}
async function makeProfileId() {
  const existing=(await chrome.storage.local.get(['profileId'])).profileId;
  if(typeof existing==='string' && /^[a-f0-9-]{36}$/.test(existing))return existing;
  const fresh=crypto.randomUUID();
  await chrome.storage.local.set({profileId:fresh});
  return fresh;
}
async function browserInfo() {
  const {browserKind='brave'} = await storage();
  return ['brave','chrome','edge'].includes(browserKind)?browserKind:'brave';
}
function videoRoute(raw){
 try{const url=new URL(raw);if(url.protocol!=='https:')return null;const host=url.hostname;
 const watch=host==='www.youtube.com'&&url.pathname==='/watch';
 const id=watch?url.searchParams.get('v')?.match(/^[a-zA-Z0-9_-]{5,128}$/)?.[0]:host==='www.youtube.com'?url.pathname.match(/^\/shorts\/([a-zA-Z0-9_-]{1,128})\/?$/)?.[1]:host==='www.instagram.com'?url.pathname.match(/^\/reels?\/([a-zA-Z0-9_-]{1,128})\/?$/)?.[1]:null;
 return id?{platform:watch?'youtube_watch':host==='www.youtube.com'?'youtube_shorts':'instagram_reels',video_id:id}:null;}catch{return null;}
}
async function navigationVisit(tabId,raw){
 const route=videoRoute(raw);if(!route||!Number.isInteger(tabId))return;
 const saved=await withQueueLock(async()=>{
  const state=await storage();if(state.collectionEnabled!==true)return false;
  const {navigationRoutes={}}=await chrome.storage.local.get(['navigationRoutes']);const now=Date.now(),key=route.platform+':'+route.video_id;
  const prior=navigationRoutes[tabId];if(prior?.key===key&&now-prior.at<2000)return false;
  navigationRoutes[tabId]={key,at:now};const recent=Object.fromEntries(Object.entries(navigationRoutes).sort((a,b)=>b[1].at-a[1].at).slice(0,50));
  await chrome.storage.local.set({...queueUpdate(state,[...(state.eventQueue||[]),newEvent('video.navigation',route,await profileId())]),navigationRoutes:recent});return true;
 });if(saved)await drainQueue();
}
async function tabChanged() {
  try {
    const window=await chrome.windows.getLastFocused();
    const collecting=(await storage()).collectionEnabled===true;
    if(window.focused&&collecting&&!foregroundTimer)foregroundTimer=setInterval(()=>{void tabChanged().then(()=>syncRules()).catch(()=>{});},15000);
    if((!window.focused||!collecting)&&foregroundTimer){clearInterval(foregroundTimer);foregroundTimer=null;}
    if (!window.focused || window.id === chrome.windows.WINDOW_ID_NONE) {
      await pushEvent('browser.window_blurred',{browser:await browserInfo(),domain:null,focused:false,profile_id:await profileId()}); return;
    }
    const tabs=await chrome.tabs.query({active:true,windowId:window.id});
    const tab=tabs[0];
    let domain=null;
    if (tab?.url) {
      const url = new URL(tab.url);
      if (['https:','http:'].includes(url.protocol) && domainValid(url.hostname)) domain=url.hostname.toLowerCase();
    }
    await pushEvent('browser.active_tab',{browser:await browserInfo(),domain,focused:true,profile_id:await profileId()});
  } catch { /* background permission/site inaccessible; leave current domain unknown */ }
}
function makeDnrRule(block, id) {
  if (!domainValid(block.domain) || !safePrefix(block.path_prefix)) return null;
  const regexFilter=regexFor(block);
  const canShowInterstitial=['www.youtube.com','www.instagram.com'].includes(block.domain);
  // For arbitrary hostnames, DNR 'block' works without requesting blanket host access.
  const action=canShowInterstitial?{type:'redirect',redirect:{extensionPath:'/blocked.html'}}:{type:'block'};
  return {id, priority:1,action,condition:{regexFilter,resourceTypes:['main_frame']}};
}
async function removeOwnedRules() {
  const {ownedIds=[]}=await storage();
  if (ownedIds.length) await chrome.declarativeNetRequest.updateDynamicRules({removeRuleIds:ownedIds});
  await chrome.storage.local.set({ownedIds:[],blocks:[]});
}
async function applyBlocks(blocks) {
  const {ownedIds=[],notificationsEnabled=false,dndEnabled=false,quietStart='',quietEnd='',lastNotifications={}}=await storage();
  const now=Date.now();
  const safe=blocks.filter(x=>x && x.expires_at_ms>now && x.expires_at_ms<=now+240000 && domainValid(x.domain) && ['block','hide_feed','stop_autoplay','notify'].includes(x.action||'block')).slice(0,500);
  // Only navigation-deny policies become DNR rules. Visual policies belong to the content adapter.
  const next=safe.filter(b=>(b.action||'block')==='block').map((block,i)=>makeDnrRule(block,RULE_ID_START+i)).filter(Boolean);
  await chrome.declarativeNetRequest.updateDynamicRules({removeRuleIds:ownedIds,addRules:next});
  await chrome.storage.local.set({ownedIds:next.map(x=>x.id),blocks:safe});
  // Rule acknowledgement is separate from optional browser activity recording.
  void request({op:'event',event:{schema_version:'1.0',source:'extension',event_id:crypto.randomUUID(),
    occurred_at_ms:now,profile_id:await profileId(),kind:'block.ack',payload:{applied_count:next.length}}});
  if (notificationsEnabled) {
    const cooled={...lastNotifications};
    for (const block of safe) {
      const key=String(block.id)+':'+String(block.action||'block');
      if(!shouldNotify(new Date(now),{notificationsEnabled,dndEnabled,quietStart,quietEnd},cooled[key]))continue;
      const label=typeof block.reason==='string'?block.reason.slice(0,70):'Restriction';
      try {
        await chrome.notifications.create('digital-discipline-'+key+'-'+now,{
          type:'basic',iconUrl:'icons/icon128.png',title:'Digital Discipline',
          message:`${label} reached its trigger. ${block.action==='notify'?'This is a reminder.':'The browser policy is now active.'}`
        });
        cooled[key]=now;
      } catch {} // Browser may suppress notifications; enforcement continues.
    }
    for(const [k,v] of Object.entries(cooled)) if(now-v>7*86400000)delete cooled[k];
    await chrome.storage.local.set({lastNotifications:cooled});
  }
  // SPA navigation doesn't perform network main_frame navigation; notify all relevant tabs.
  const tabs=await chrome.tabs.query({url:['https://www.youtube.com/*','https://www.instagram.com/*']});
  for (const tab of tabs) if (tab.id) chrome.tabs.sendMessage(tab.id,{op:'blocksUpdated',blocks:safe}).catch(()=>{});
}
function syncRules({force=false}={}) {
  if (syncInFlight) return syncInFlight;
  syncInFlight=(async()=>{
    const {emergencyUntil=0,blocks=[]}=await storage();
    if (Date.now()<emergencyUntil) await removeOwnedRules();
    if (Date.now()>=emergencyUntil && blocks.some(b=>!b.expires_at_ms || b.expires_at_ms<=Date.now())) await applyBlocks(blocks);
    const reply=await request({op:'sync'});
    if (!reply.ok) return reply;
    if (reply.schema_version==='1.0' && Array.isArray(reply.blocks) && Date.now()>=emergencyUntil) await applyBlocks(reply.blocks);
    await drainQueue({force});
    return reply;
  })().finally(()=>{syncInFlight=null;});
  return syncInFlight;
}

async function emergencyUnblock() {
  const until=Date.now()+30*60_000;
  await chrome.storage.local.set({emergencyUntil:until});
  await removeOwnedRules();
  return until;
}

chrome.runtime.onInstalled.addListener(async()=>{
  chrome.alarms.create('heartbeat',{periodInMinutes:1});
  await tabChanged();await syncRules();
});
chrome.runtime.onStartup.addListener(()=>{chrome.alarms.create('heartbeat',{periodInMinutes:1});tabChanged();syncRules();});
chrome.alarms.onAlarm.addListener(async alarm=>{
  try {
    if(alarm.name===RETRY_ALARM)await drainQueue();
    if(alarm.name==='heartbeat'){await tabChanged();await drainQueue();await syncRules();}
  }catch{lastQueueError='queue_storage_failed';}
});
chrome.tabs.onActivated.addListener(tabChanged);
chrome.tabs.onUpdated.addListener((id, info, tab)=>{if(info.url){void navigationVisit(id,info.url).catch(()=>{});}if(info.url || info.status==='complete') tabChanged();});
chrome.windows.onFocusChanged.addListener(tabChanged);
chrome.webNavigation.onHistoryStateUpdated.addListener(d=>{if(d.frameId===0){tabChanged();void navigationVisit(d.tabId,d.url).catch(()=>{});}});
chrome.webNavigation.onCommitted?.addListener(d=>{if(d.frameId===0)void navigationVisit(d.tabId,d.url).catch(()=>{});});
chrome.runtime.onMessage.addListener((msg,sender,sendResponse)=>{
  if(msg?.op==='adapterStatus'){
    const tab=sender.tab;
    if(tab?.active===true&&sender.frameId===0&&/^https:\/\/www\.(youtube\.com|instagram\.com)\//.test(tab.url||'')&&['ready','no_player','unsupported'].includes(msg.status)){
      adapterState=msg.status;adapterAt=Date.now();sendResponse({ok:true});
    }else sendResponse({ok:false,error:'invalid_adapter_status'});
    return;
  }
  if (msg?.op==='video') {
    // Only accept playback observations from extension's permitted content-script origins.
    let url,tabUrl;
    try {url=new URL(msg.page_url||sender.tab?.url||'');tabUrl=new URL(sender.tab?.url||'');}catch {sendResponse({ok:false,error:'invalid_url'});return;}
    const host=url.hostname;
    // A content script can only provide a page URL under its actual tab origin.
    const allowed=(host==='www.youtube.com' || host==='www.instagram.com') && url.origin===tabUrl.origin && sender.tab && sender.frameId===0;
    if (!allowed || !['video.progress','video.qualified_play'].includes(msg.kind)) {sendResponse({ok:false,error:'untrusted_video_event'});return;}
    if (!Number.isFinite(msg.watched_ms) || msg.watched_ms<0 || msg.watched_ms>86400000 || !/^[a-f0-9-]{36}$/.test(String(msg.instance_id))) {sendResponse({ok:false,error:'invalid_playback'});return;}
    const watch=host==='www.youtube.com'&&url.pathname==='/watch';
    const plat=watch?'youtube_watch':host==='www.youtube.com'?'youtube_shorts':'instagram_reels';
    const id=watch ? url.searchParams.get('v')?.match(/^[a-zA-Z0-9_-]{5,128}$/)?.[0] : host==='www.youtube.com' ? url.pathname.match(/^\/shorts\/([a-zA-Z0-9_-]+)/)?.[1] : url.pathname.match(/^\/reels?\/([a-zA-Z0-9_-]+)/)?.[1];
    if (!id) {sendResponse({ok:false,error:'unrecognized_video_route'});return;}
    (async()=>{
      const opts=await chrome.storage.local.get(['collectionEnabled','videoObserved']);
      if(opts.collectionEnabled!==true){sendResponse({ok:false,error:'recording_off'});return;}
      if(msg.kind==='video.qualified_play'){
        const observed=opts.videoObserved||{shorts:0,watches:0,reels:0,lastAt:0};
        if(plat==='youtube_shorts')observed.shorts=(observed.shorts||0)+1;
        else if(plat==='youtube_watch')observed.watches=(observed.watches||0)+1;
        else observed.reels=(observed.reels||0)+1;
        observed.lastAt=Date.now();
        await chrome.storage.local.set({videoObserved:observed});
      }
      await pushEvent(msg.kind,{platform:plat,video_id:id,instance_id:msg.instance_id,watched_ms:Math.floor(msg.watched_ms)});
      sendResponse({ok:true});
    })().catch(()=>sendResponse({ok:false,error:'queue_write_failed'}));
    return true;
  }
  if (msg?.op==='getStatus') {
    storage().then(({blocks=[],emergencyUntil=0,eventQueue=[],eventQuarantine=[],queueDiagnostics={},collectionEnabled=false,videoObserved={}})=>sendResponse({connected,blocks,emergencyUntil,queued:eventQueue.length,quarantined:eventQuarantine.length,queueDiagnostics,retryAfter:queueDiagnostics.retry_at_ms||0,collectionEnabled,videoObserved,lastQueueError,lastConnectionError,lastNativeReplyAt,lastConnectAttemptAt,transport:"Direct localhost"})).catch(()=>sendResponse({ok:false,error:'queue_storage_failed'}));return true;
  }
  if (msg?.op==='checkBlocked') {
    storage().then(({blocks=[],emergencyUntil=0})=>sendResponse({blocked:Date.now()>=emergencyUntil&&blocks.some(b=>b.action==='block' && b.expires_at_ms>Date.now() && ruleMatches(sender.tab?.url || '',b)),blocks:Date.now()>=emergencyUntil?blocks.filter(b=>b.expires_at_ms>Date.now()):[]}));return true;
  }
  if (msg?.op==='emergencyUnblock') {emergencyUnblock().then(until=>sendResponse({ok:true,until}));return true;}
  if (msg?.op==='resync') {syncRules({force:true}).then(reply=>sendResponse({ok:reply?.ok===true,error:reply?.error||'',queuedUnknown:!connected})).catch(()=>sendResponse({ok:false,error:'queue_storage_failed'}));return true;}
  if (msg?.op==='purgePending') {purgePending().then(removed=>sendResponse({ok:true,removed})).catch(()=>sendResponse({ok:false,error:'queue_storage_failed'}));return true;}
  if (msg?.op==='notificationsEnable') {
    chrome.storage.local.set({notificationsEnabled:msg.value===true}).then(()=>sendResponse({ok:true}));return true;
  }
  if (msg?.op==='notificationPreferences') {
    const valid=v=>v===''||/^([01]\d|2[0-3]):[0-5]\d$/.test(v);
    if(!valid(msg.quietStart)||!valid(msg.quietEnd)) {sendResponse({ok:false});return;}
    chrome.storage.local.set({dndEnabled:msg.dndEnabled===true,quietStart:msg.quietStart,quietEnd:msg.quietEnd})
      .then(()=>sendResponse({ok:true})).catch(()=>sendResponse({ok:false}));return true;
  }
  if (msg?.op==='collectionEnable') {
    const enable = msg.value === true;
    (async () => {
      const profile=await profileId(),browser=await browserInfo();
      await withQueueLock(async()=>{
        const state=await storage(),pending=[...(state.eventQueue||[])];
        if(!enable&&state.collectionEnabled===true)pending.push(newEvent('browser.window_blurred',{browser,domain:null,focused:false,profile_id:profile},profile));
        // Stop new collection atomically while retaining the existing queue for sync.
        await chrome.storage.local.set({...queueUpdate(state,pending),collectionEnabled:enable});
      });
      if(enable) await tabChanged();
      else {if(foregroundTimer){clearInterval(foregroundTimer);foregroundTimer=null;}await drainQueue();}
      sendResponse({ok:true,enabled:enable});
    })().catch(()=>sendResponse({ok:false,error:'queue_storage_failed'}));
    return true;
  }
  if (msg?.op==='browserKind') {
    if (!['brave','chrome','edge'].includes(msg.value)) {sendResponse({ok:false});return;}
    chrome.storage.local.set({browserKind:msg.value}).then(()=>tabChanged()).then(()=>sendResponse({ok:true}));return true;
  }
});
chrome.alarms.create('heartbeat',{periodInMinutes:1});
void restoreRetryAlarm().catch(()=>{lastQueueError='queue_storage_failed';});
