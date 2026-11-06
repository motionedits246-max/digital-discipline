const $ = id => document.getElementById(id);
const browsers=['brave','chrome','edge'];
async function refresh({preferences=true}={}) {
 $('extensionId').textContent=chrome.runtime.id;
 const r=await chrome.runtime.sendMessage({op:'getStatus'});
 if(!r||r.ok===false){$('connection').textContent='Local queue status unavailable';$('status').textContent='Could not read extension storage. Reload the extension and check available disk space.';return;}
 const linked=r.connected===true;
 $('connection').textContent=linked?'Connected to local Windows agent':'Not connected to Windows agent';
 $('led').className='led '+(linked?'ok':'fail');
 const error=String(r.lastConnectionError||'');
 $('errorDetail').hidden=linked||!error;
 $('errorDetail').textContent=error;
 $('count').textContent=r.blocks.length;
 $('queued').textContent=r.queued;
 const d=r.queueDiagnostics||{};
 $('delivered').textContent=(d.accepted||0)+(d.duplicate||0);
 $('ignored').textContent=(d.ignored_paused||0)+(d.ignored_excluded||0);
 $('rejected').textContent=(d.rejected||0)+(d.invalid||0);
 for(const field of ['expired','overflow','purged'])$(field).textContent=d[field]||0;
 $('purgePending').disabled=!r.queued;
 $('deliveryState').textContent=r.lastQueueError?'Could not update local queue storage; observations may need retry.':r.retryAfter>Date.now()?
  'Waiting to retry after '+new Date(r.retryAfter).toLocaleTimeString()+' · '+(d.last_error||'Connection unavailable'):
  r.queued?'Pending observations will sync while the browser is awake.':d.last_accepted_at_ms?
  'Last acknowledged event: '+new Date(d.last_accepted_at_ms).toLocaleTimeString():'No acknowledged events recorded yet.';
 $('recordingHint').hidden=r.collectionEnabled===true;
 $('localShorts').textContent=r.videoObserved?.shorts||0;
 $('localWatches').textContent=r.videoObserved?.watches||0;
 $('localReels').textContent=r.videoObserved?.reels||0;
 $('lastObserved').textContent=r.videoObserved?.lastAt?('Last observed video: '+new Date(r.videoObserved.lastAt).toLocaleTimeString()+' · Local counts are separate from acknowledged dashboard metrics.'):'No qualifying playback detected yet. Enable recording, refresh the YouTube tab, then play for at least 3 seconds.';
 if(preferences){
 const {browserKind='brave',collectionEnabled=false,notificationsEnabled=false,dndEnabled=false,quietStart='',quietEnd=''}=await chrome.storage.local.get(['browserKind','collectionEnabled','notificationsEnabled','dndEnabled','quietStart','quietEnd']);
 for(const b of browsers){const el=document.querySelector(`[data-browser="${b}"]`);el.classList.toggle('selected',b===browserKind);el.setAttribute('aria-pressed',String(b===browserKind));}
 $('enabled').checked=collectionEnabled;
 $('notifications').checked=notificationsEnabled;
 $('dnd').checked=dndEnabled;
 $('quietStart').value=quietStart;
 $('quietEnd').value=quietEnd;
 }
 if(r.emergencyUntil>Date.now()) $('status').textContent='Recovery active until '+new Date(r.emergencyUntil).toLocaleTimeString();
}
$('enabled').addEventListener('change',async e=>{
 const enabled=e.target.checked;
 const reply=await chrome.runtime.sendMessage({op:'collectionEnable',value:enabled});
 await refresh();
 $('status').textContent=!reply?.ok?'Could not save recording preferences. Check local storage.':enabled?'Recording enabled. Refresh open YouTube/Instagram tabs, then play for 3–5 seconds.':'Recording off. Pending observations are retained for local sync.';
});
$('purgePending').addEventListener('click',async()=>{
 if(!window.confirm('Delete observations waiting in this browser profile? Already delivered data stays in the desktop app. An observation being sent may still arrive.'))return;
 const reply=await chrome.runtime.sendMessage({op:'purgePending'});
 await refresh();
 $('status').textContent=reply?.ok?reply.removed+' pending observations deleted. Recording preference unchanged.':'Could not delete pending observations.';
});
$('notifications').addEventListener('change',async e=>{await chrome.runtime.sendMessage({op:'notificationsEnable',value:e.target.checked});refresh();});
async function notifyPrefs(){
 for(const id of ['quietStart','quietEnd']){const m=$(id).value.match(/^(\d{1,2}):(\d{1,2})$/);if(m&&+m[1]<24&&+m[2]<60)$(id).value=m[1].padStart(2,'0')+':'+m[2].padStart(2,'0');if($(id).value&&!/^([01]\d|2[0-3]):[0-5]\d$/.test($(id).value)){$('status').textContent='Use a 24-hour time, for example 22:30.';return;}}
 const result=await chrome.runtime.sendMessage({op:'notificationPreferences',dndEnabled:$('dnd').checked,quietStart:$('quietStart').value,quietEnd:$('quietEnd').value});if(result?.ok)await refresh();else $('status').textContent='Could not save notification times.';
}
for(const id of ['dnd','quietStart','quietEnd'])$(id).addEventListener('change',notifyPrefs);
for(const button of document.querySelectorAll('[data-browser]'))button.addEventListener('click',async()=>{await chrome.runtime.sendMessage({op:'browserKind',value:button.dataset.browser});refresh();});
$('sync').addEventListener('click',async()=>{
 $('sync').disabled=true;
 $('status').textContent='Testing direct localhost connection and sending queued events…';
 try{
   const reply=await chrome.runtime.sendMessage({op:'resync'});
   await refresh();
   $('status').textContent=reply?.ok?'Connected. Queued events are being delivered.':('Sync failed: '+String(reply?.error||'Local agent unavailable / extension not authorized'));
 }catch(err){$('status').textContent='Sync failed: '+String(err?.message||err);}finally{$('sync').disabled=false;}
});
$('recover').addEventListener('click',async()=>{await chrome.runtime.sendMessage({op:'emergencyUnblock'});refresh();});
$('copyId').addEventListener('click',async()=>{try{await navigator.clipboard.writeText(chrome.runtime.id);$('status').textContent='Extension ID copied.';}catch{$('status').textContent='Select and copy the ID above.';}});
// Attempt a real handshake on every popup open, not simply port creation.
chrome.runtime.sendMessage({op:'resync'}).catch(()=>{}).finally(refresh);
chrome.storage.onChanged.addListener((changes,area)=>{if(area==='local'&&(changes.eventQueue||changes.queueDiagnostics))void refresh({preferences:false});});
