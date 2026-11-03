document.querySelector('#back').addEventListener('click',()=>history.back());
document.querySelector('#recover').addEventListener('click',async()=>{
  const result=await chrome.runtime.sendMessage({op:'emergencyUnblock'});
  document.querySelector('#message').textContent=result?.ok?'Blocking paused for 30 minutes. You can navigate back.':'Recovery could not be completed.';
});

chrome.runtime.sendMessage({op:'getStatus'}).then(r=>{
  const now=Date.now();
  const names=(r?.blocks||[]).filter(b=>b.action==='block'&&b.expires_at_ms>now).slice(0,3)
    .map(b=>`${b.reason||'Browser restriction'} · renews until ${new Date(b.expires_at_ms).toLocaleTimeString()}`);
  document.querySelector('#details').textContent=names.length?names.join(' | '):'Restrictions may have expired; try returning to the site.';
}).catch(()=>{document.querySelector('#details').textContent='Browser extension status is unavailable.';});
