(() => {
  // Observe only player telemetry on the user's own, visible short-form page.
  // No messages, credentials, forms, screenshots, transcripts or titles are read.
  let video = null, key = null, id = null, watchedMs = 0, lastClock = 0, lastVideoTime = 0, observedUrl = null, observedKind = null;
  let collecting = false;
  chrome.storage.local.get('collectionEnabled').then(x=>{collecting=x.collectionEnabled===true;}).catch(()=>{});
  chrome.storage.onChanged.addListener((changes,area)=>{if(area==='local'&&changes.collectionEnabled){collecting=changes.collectionEnabled.newValue===true; if(!collecting) reset(null,null);}});
  let qualified = false, lastProgress = 0, lastRoute = location.pathname+location.search, blockedOverlay = null;
  let contentStyle = null;
  const match = () => {
    if (location.hostname === 'www.youtube.com') {
      if (location.pathname === '/watch') return new URLSearchParams(location.search).get('v')?.match(/^[\w-]{5,128}$/)?.[0] || null;
      return location.pathname.match(/^\/shorts\/([\w-]+)/)?.[1] || null;
    }
    return location.pathname.match(/^\/reels?\/([\w-]+)/)?.[1] || null;
  };
  const visible = () => document.visibilityState === 'visible' && document.hasFocus();
  const send = kind => {
    if (!id || !key) return;
    chrome.runtime.sendMessage({op:'video',kind,instance_id:key,watched_ms:Math.floor(watchedMs),page_url:observedUrl || location.href}).catch(()=>{});
  };
  function reset(newId, newVideo) {
    if (id && watchedMs > 0) send('video.progress');
    id=newId;video=newVideo;observedUrl=location.href;observedKind=location.pathname==='/watch'?'watch':location.pathname.startsWith('/shorts/')?'shorts':'reels';key=crypto.randomUUID();watchedMs=0;qualified=false;
    lastProgress=0;lastClock=performance.now();lastVideoTime=video?.currentTime || 0;
  }
  function activeVideo() {
    const matches=[...document.querySelectorAll('video')].filter(v=>{
      const box=v.getBoundingClientRect();
      return box.width>80 && box.height>80 && box.bottom>0 && box.top<innerHeight && box.right>0 && box.left<innerWidth;
    });
    return matches.sort((a,b)=>b.clientWidth*b.clientHeight-a.clientWidth*a.clientHeight)[0] || null;
  }
  function tick() {
    if (!collecting) return;
    const currentId=match();
    const currentVideo=currentId?activeVideo():null;
    const currentKind=location.pathname==='/watch'?'watch':location.pathname.startsWith('/shorts/')?'shorts':'reels';
    if (currentId!==id || currentKind!==observedKind || (!video && currentVideo)) reset(currentId,currentVideo);
    else if (currentVideo!==video) {
      // DOM player replacement on the same short is not automatically another play.
      video=currentVideo;
      lastVideoTime=currentVideo?.currentTime||0;
      lastClock=performance.now();
    }
    if (!currentId || !currentVideo) return;
    const now=performance.now();
    const media=currentVideo.currentTime;
    const wall=now-lastClock;
    const mediaDelta=media-lastVideoTime;
    // Same media ID replaying from the beginning is a new observed play instance.
    // Ordinary seeks are not automatically counted as replays.
    if(mediaDelta < -2 && lastVideoTime>4 && media<1.5 && !currentVideo.seeking){reset(currentId,currentVideo);return;}
    // Observed progression—not just a URL impression or time spent with a paused tab.
    if (visible() && !currentVideo.paused && !currentVideo.seeking && currentVideo.readyState>=2 &&
        mediaDelta>0 && wall>0 && wall<2500) {
      watchedMs+=Math.min(wall,mediaDelta*1000,1200);
    }
    lastClock=now;lastVideoTime=media;
    if (!qualified && watchedMs>=2000) {qualified=true;send('video.qualified_play');}
    if (watchedMs-lastProgress>=5000) {send('video.progress');lastProgress=watchedMs;}
  }
  function applyBlock(blocked) {
    if (!blocked) {blockedOverlay?.remove();blockedOverlay=null;return;}
    if (blockedOverlay) return;
    blockedOverlay=document.createElement('div');
    blockedOverlay.id='dd-privacy-block';
    blockedOverlay.setAttribute('role','alertdialog');
    Object.assign(blockedOverlay.style,{position:'fixed',inset:'0',zIndex:'2147483647',background:'#0b0b0df7',color:'#f5f5f6',display:'grid',placeItems:'center',fontFamily:'system-ui'});
    const frame=document.createElement('div');
    Object.assign(frame.style,{maxWidth:'400px',padding:'36px',textAlign:'center'});
    const title=document.createElement('h1');title.textContent='This route is paused';
    const subtitle=document.createElement('p');subtitle.textContent='Digital Discipline is applying your local restriction. Other pages on this site may still work.';
    const button=document.createElement('button');button.textContent='Emergency unblock · 30 min';
    Object.assign(button.style,{padding:'12px 18px',background:'#fff',color:'#111',border:'none',borderRadius:'10px',cursor:'pointer'});
    button.addEventListener('click',()=>chrome.runtime.sendMessage({op:'emergencyUnblock'}).then(()=>applyBlock(false)));
    frame.append(title,subtitle,button);blockedOverlay.append(frame);document.documentElement.appendChild(blockedOverlay);
    if(video) video.pause();
  }
  function applySoftControls(blocks) {
    const relevant=blocks.filter(b=>b.expires_at_ms>Date.now() &&
      (location.hostname===b.domain || location.hostname.endsWith('.'+b.domain)) &&
      (!b.path_prefix || location.pathname.startsWith(b.path_prefix)));
    const hide=relevant.some(b=>b.action==='hide_feed');
    const stop=relevant.some(b=>b.action==='stop_autoplay');
    if(!hide){contentStyle?.remove();contentStyle=null;}
    else if(!contentStyle) {
      contentStyle=document.createElement('style');
      contentStyle.id='digital-discipline-soft-policy';
      contentStyle.textContent=location.hostname==='www.youtube.com'
        ? 'ytd-rich-grid-renderer, ytd-reel-shelf-renderer, ytd-rich-shelf-renderer, ytd-watch-next-secondary-results-renderer {display:none!important}'
        : '[role="feed"], article {display:none!important}';
      document.documentElement.appendChild(contentStyle);
    }
    if(stop) for(const player of document.querySelectorAll('video')) {
      // Stop automatic playback permission without interrupting deliberate user
      // playback. Pausing every playing video made the control unusable.
      player.autoplay=false;
      player.removeAttribute('autoplay');
    }
  }
  function checkBlock() {
    chrome.runtime.sendMessage({op:'checkBlocked'}).then(x=>{applyBlock(!!x?.blocked);applySoftControls(x?.blocks||[]);}).catch(()=>{applyBlock(false);applySoftControls([]);});
  }
  chrome.runtime.onMessage.addListener(msg=>{if(msg?.op==='blocksUpdated') checkBlock();});
  window.addEventListener('pagehide',()=>send('video.progress'));
  document.addEventListener('visibilitychange',()=>{send('video.progress');checkBlock();});
  setInterval(()=>{
    const route=location.pathname+location.search; if (route!==lastRoute){lastRoute=route;checkBlock();}
    tick();
    if(Date.now()%5000<1000)checkBlock();
  },1000);
  checkBlock();
})();
