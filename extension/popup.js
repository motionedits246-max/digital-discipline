const $ = id => document.getElementById(id);
const browsers=['brave','chrome','edge'];
async function refresh() {
 $('extensionId').textContent=chrome.runtime.id;
 const r=await chrome.runtime.sendMessage({op:'getStatus'});
 const linked=r.connected===true;
 $('connection').textContent=linked?'Connected to local Windows agent':'Not connected to Windows agent';
 $('led').className='led '+(linked?'ok':'fail');
 const error=String(r.lastConnectionError||'');
 $('errorDetail').hidden=linked||!error;
 $('errorDetail').textContent=error;
 $('count').textContent=r.blocks.length;
 $('queued').textContent=r.queued;
 const {browserKind='brave',collectionEnabled=false,notificationsEnabled=false,dndEnabled=false,quietStart='',quietEnd=''}=await chrome.storage.local.get(['browserKind','collectionEnabled','notificationsEnabled','dndEnabled','quietStart','quietEnd']);
 for(const b of browsers){const el=document.querySelector(`[data-browser="${b}"]`);el.classList.toggle('selected',b===browserKind);el.setAttribute('aria-pressed',String(b===browserKind));}
 $('enabled').checked=collectionEnabled;
 $('notifications').checked=notificationsEnabled;
 $('dnd').checked=dndEnabled;
 $('quietStart').value=quietStart;
 $('quietEnd').value=quietEnd;
 if(r.emergencyUntil>Date.now()) $('status').textContent='Recovery active until '+new Date(r.emergencyUntil).toLocaleTimeString();
}
$('enabled').addEventListener('change',async e=>{await chrome.runtime.sendMessage({op:'collectionEnable',value:e.target.checked});refresh();});
$('notifications').addEventListener('change',async e=>{await chrome.runtime.sendMessage({op:'notificationsEnable',value:e.target.checked});refresh();});
async function notifyPrefs(){
 await chrome.runtime.sendMessage({op:'notificationPreferences',dndEnabled:$('dnd').checked,quietStart:$('quietStart').value,quietEnd:$('quietEnd').value});refresh();
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
