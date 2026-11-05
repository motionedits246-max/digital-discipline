import {shouldNotify} from "./notifications.js";
// MV3 module worker; pure policy helpers share the same code as unit tests.
import {isValidDomain,safePrefix,matchesRule,regexFor} from "./rules.js";
// Browser service worker talks directly to a restricted loopback API.
// No nativeMessaging permission, registry edits, master agent token or extra EXE.
// The desktop authorizes this specific extension ID. Other origins are denied.
const BRIDGE = 'http://127.0.0.1:48761';
const RULE_ID_START = 800000;
const MAX_EVENTS = 2000;
let connected = false;
let syncInFlight = null;
let lastConnectionError = '';
let lastNativeReplyAt = 0; // kept for backward-compatible status consumers
let lastConnectAttemptAt = 0;
function domainValid(value){return isValidDomain(value);}
function ruleMatches(url,rule){return matchesRule(url,rule);}
async function storage() { return chrome.storage.local.get(['eventQueue', 'blocks', 'ownedIds', 'emergencyUntil', 'browserKind', 'collectionEnabled','notificationsEnabled','lastNotifications','videoObserved']); }
async function request(message) {
  lastConnectAttemptAt=Date.now();
  const sync=message?.op==='sync';
  const url=BRIDGE+(sync?'/bridge/sync':'/bridge/event');
  try {
    const response=await fetch(url,{method:'POST',headers:{'X-DD-Bridge':'1','Content-Type':'application/json'},
      body:sync?'{}':JSON.stringify(message.event),cache:'no-store',credentials:'omit',signal:AbortSignal.timeout(6000)});
    const raw=await response.text();
    let reply;
    try{reply=JSON.parse(raw);}catch{reply={ok:false,error:raw.slice(0,160)||`agent_http_${response.status}`};}
    if(!response.ok || reply?.ok!==true){
      connected=false;
      lastConnectionError=String(reply?.error||`agent_http_${response.status}`);
      return {ok:false,error:lastConnectionError};
    }
    connected=true;
    lastConnectionError='';
    lastNativeReplyAt=Date.now();
    return reply;
  } catch(err){
    connected=false;
    lastConnectionError=String(err?.message||err||'Local agent is not running');
    return {ok:false,error:lastConnectionError};
  }
}
let queueOps = Promise.resolve();
function withQueueLock(fn) {
  const next = queueOps.then(fn, fn);
  queueOps = next.catch(() => {});
  return next;
}
async function pushEvent(kind, payload) {
  const {collectionEnabled=false}=await storage();
  if (!collectionEnabled) return;
  const item = {schema_version:'1.0', source:'extension',event_id:crypto.randomUUID(),occurred_at_ms:Date.now(),kind,payload};
  await withQueueLock(async () => {
    const {eventQueue=[]} = await storage();
    const fresh = eventQueue.filter(e => e.occurred_at_ms > Date.now()-86_400_000);
    await chrome.storage.local.set({eventQueue:[...fresh,item].slice(-MAX_EVENTS)});
  });
  await drainQueue();
}
let draining = false;
async function drainQueue() {
  if (draining) return;
  draining = true;
  try {
    while (true) {
      const item = await withQueueLock(async()=> (await storage()).eventQueue?.[0]);
      if (!item) break;
      const reply = await request({op:'event',event:item});
      if (!reply.ok) break;
      // Preserve newly arrived events; remove only this acknowledged, stable event ID.
      await withQueueLock(async () => {
        const {eventQueue=[]}=await storage();
        await chrome.storage.local.set({eventQueue:eventQueue.filter(e=>e.event_id!==item.event_id)});
      });
    }
  } finally { draining=false; }
}
async function profileId() {
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
async function tabChanged() {
  try {
    const window=await chrome.windows.getLastFocused();
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
    occurred_at_ms:now,kind:'block.ack',payload:{applied_count:next.length}}});
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
function syncRules() {
  if (syncInFlight) return syncInFlight;
  syncInFlight=(async()=>{
    const {emergencyUntil=0,blocks=[]}=await storage();
    if (Date.now()<emergencyUntil) await removeOwnedRules();
    if (Date.now()>=emergencyUntil && blocks.some(b=>!b.expires_at_ms || b.expires_at_ms<=Date.now())) await applyBlocks(blocks);
    const reply=await request({op:'sync'});
    if (!reply.ok) return reply;
    if (reply.schema_version==='1.0' && Array.isArray(reply.blocks) && Date.now()>=emergencyUntil) await applyBlocks(reply.blocks);
    await drainQueue();
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
chrome.alarms.onAlarm.addListener(async alarm=>{if(alarm.name==='heartbeat'){await tabChanged();await drainQueue();await syncRules();}});
chrome.tabs.onActivated.addListener(tabChanged);
chrome.tabs.onUpdated.addListener((id, info, tab)=>{if(info.url || info.status==='complete') tabChanged();});
chrome.windows.onFocusChanged.addListener(tabChanged);
chrome.webNavigation.onHistoryStateUpdated.addListener(d=>{if(d.frameId===0) tabChanged();});
chrome.runtime.onMessage.addListener((msg,sender,sendResponse)=>{
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
    storage().then(({blocks=[],emergencyUntil=0,eventQueue=[],collectionEnabled=false,videoObserved={}})=>sendResponse({connected,blocks,emergencyUntil,queued:eventQueue.length,collectionEnabled,videoObserved,lastConnectionError,lastNativeReplyAt,lastConnectAttemptAt,transport:"Direct localhost"}));return true;
  }
  if (msg?.op==='checkBlocked') {
    storage().then(({blocks=[],emergencyUntil=0})=>sendResponse({blocked:Date.now()>=emergencyUntil&&blocks.some(b=>b.action==='block' && b.expires_at_ms>Date.now() && ruleMatches(sender.tab?.url || '',b)),blocks:Date.now()>=emergencyUntil?blocks.filter(b=>b.expires_at_ms>Date.now()):[]}));return true;
  }
  if (msg?.op==='emergencyUnblock') {emergencyUnblock().then(until=>sendResponse({ok:true,until}));return true;}
  if (msg?.op==='resync') {syncRules().then(reply=>sendResponse({ok:reply?.ok===true,error:reply?.error||'',queuedUnknown:!connected})).catch(err=>sendResponse({ok:false,error:String(err)}));return true;}
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
      if(!enable) await pushEvent('browser.window_blurred',{browser:await browserInfo(),domain:null,focused:false,profile_id:await profileId()});
      await chrome.storage.local.set({collectionEnabled:enable,eventQueue:enable?(await storage()).eventQueue||[]:[]});
      if(enable) await tabChanged();
      sendResponse({ok:true,enabled:enable});
    })();
    return true;
  }
  if (msg?.op==='browserKind') {
    if (!['brave','chrome','edge'].includes(msg.value)) {sendResponse({ok:false});return;}
    chrome.storage.local.set({browserKind:msg.value}).then(()=>tabChanged()).then(()=>sendResponse({ok:true}));return true;
  }
});
chrome.alarms.create('heartbeat',{periodInMinutes:1});
