(() => {
  // Observe only player telemetry on the user's own, visible short-form page.
  // No messages, credentials, forms, screenshots, transcripts or titles are read.
  let video = null, key = null, id = null, watchedMs = 0, lastClock = 0, lastVideoTime = 0, observedUrl = null, observedKind = null;
  let collecting = false;
  chrome.storage.local.get('collectionEnabled').then(x=>{collecting=x.collectionEnabled===true;}).catch(()=>{});
  chrome.storage.onChanged.addListener((changes,area)=>{if(area==='local'&&changes.collectionEnabled){collecting=changes.collectionEnabled.newValue===true; if(!collecting) reset(null,null);}});
  let qualified = false, lastProgress = 0, lastRoute = location.pathname+location.search, blockedOverlay = null;
  let contentStyle = null,endedObserved=false,seekObserved=false,manualUntil=0,stopAutoplay=false,allowedRoute=null;
  const autoplayOriginal=new Map();
  const routeKey=()=>location.pathname+location.search;
  function bindPlayer(player){
    if(!player||player.dataset?.ddTelemetryBound)return;
    if(player.dataset)player.dataset.ddTelemetryBound='1';
    player.addEventListener('ended',()=>{if(player===video)endedObserved=true;});
    player.addEventListener('seeking',()=>{if(player===video)seekObserved=true;});
  }
  function userIntent(event){
    if(!event.isTrusted||event.type==='keydown'&&![' ','Enter','k','MediaPlayPause'].includes(event.key))return;
    if(event.target?.closest?.('video,#movie_player,.html5-video-player,[data-testid="video-player"],button[aria-label*="Play"],button[aria-label*="Pause"]'))manualUntil=performance.now()+1500;
  }
  document.addEventListener('pointerdown',userIntent,true);document.addEventListener('keydown',userIntent,true);
  document.addEventListener('play',event=>{
    const player=event.target;if(player?.tagName!=='VIDEO'||!stopAutoplay)return;
    const route=routeKey();if(manualUntil>0&&performance.now()<=manualUntil){allowedRoute=route;return;}
    if(allowedRoute!==route)player.pause();
  },true);
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
    endedObserved=false;seekObserved=false;bindPlayer(video);lastProgress=0;lastClock=performance.now();lastVideoTime=video?.currentTime || 0;
  }
  function activeVideo() {
    const matches=[...document.querySelectorAll('video')].filter(v=>{
      const box=v.getBoundingClientRect();
      return box.width>80 && box.height>80 && box.bottom>0 && box.top<innerHeight && box.right>0 && box.left<innerWidth;
    });
    return matches.sort((a,b)=>b.clientWidth*b.clientHeight-a.clientWidth*a.clientHeight)[0] || null;
  }
  let lastAdapterReport=0;
  function tick() {
    if (!collecting) return;
    const currentId=match();
    const currentVideo=currentId?activeVideo():null;
    if(visible()&&performance.now()-lastAdapterReport>5000){lastAdapterReport=performance.now();chrome.runtime.sendMessage({op:'adapterStatus',status:currentId&&currentVideo?'ready':'no_player'}).catch(()=>{});}
    const currentKind=location.pathname==='/watch'?'watch':location.pathname.startsWith('/shorts/')?'shorts':'reels';
    if (currentId!==id || currentKind!==observedKind || (!video && currentVideo)) reset(currentId,currentVideo);
    else if (currentVideo!==video) {
      // DOM player replacement on the same short is not automatically another play.
      video=currentVideo;bindPlayer(video);endedObserved=false;seekObserved=false;
      lastVideoTime=currentVideo?.currentTime||0;
      lastClock=performance.now();
    }
    if (!currentId || !currentVideo) return;
    const now=performance.now();
    const media=currentVideo.currentTime;
    const wall=now-lastClock;
    const mediaDelta=media-lastVideoTime;
    // An ordinary seek to zero is not a replay. Require an ended signal or a
    // verified loop boundary, and qualify the new instance from observed media.
    if(DDPlayback.replayConfirmed({ended:endedObserved,loop:currentVideo.loop,previous:lastVideoTime,media,duration:currentVideo.duration,seeking:seekObserved||currentVideo.seeking,wall,manual:manualUntil>0&&now<=manualUntil})){
      reset(currentId,currentVideo);return;
    }
    watchedMs+=DDPlayback.observedDelta({wall,delta:mediaDelta,rate:currentVideo.playbackRate||1,visible:visible(),paused:currentVideo.paused,seeking:seekObserved||currentVideo.seeking,ready:currentVideo.readyState});
    seekObserved=false;
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
      contentStyle=document.createElement('style');contentStyle.id='digital-discipline-soft-policy';
      contentStyle.textContent=location.hostname==='www.youtube.com'
        ? 'ytd-rich-grid-renderer, ytd-reel-shelf-renderer, ytd-rich-shelf-renderer, ytd-watch-next-secondary-results-renderer {display:none!important}'
        : location.pathname==='/'?'main [role="feed"] {display:none!important}':'';
      document.documentElement.appendChild(contentStyle);
    }
    if(stop&&!stopAutoplay){allowedRoute=[...document.querySelectorAll('video')].some(p=>!p.paused)?routeKey():null;}
    stopAutoplay=stop;
    if(stop)for(const player of document.querySelectorAll('video')){
      if(!autoplayOriginal.has(player))autoplayOriginal.set(player,{attribute:player.getAttribute('autoplay'),property:player.autoplay});
      player.autoplay=false;player.removeAttribute('autoplay');
    }
    else{
      for(const [player,original] of autoplayOriginal){player.autoplay=original.property;if(original.attribute===null)player.removeAttribute('autoplay');else player.setAttribute('autoplay',original.attribute);}
      autoplayOriginal.clear();allowedRoute=null;
    }
  }

  function checkBlock() {
    chrome.runtime.sendMessage({op:'checkBlocked'}).then(x=>{applyBlock(!!x?.blocked);applySoftControls(x?.blocks||[]);}).catch(()=>{applyBlock(false);applySoftControls([]);});
  }
  chrome.runtime.onMessage.addListener(msg=>{if(msg?.op==='blocksUpdated') checkBlock();});
  window.addEventListener('pagehide',()=>send('video.progress'));
  document.addEventListener('visibilitychange',()=>{send('video.progress');checkBlock();});
  setInterval(()=>{
    const route=location.pathname+location.search; if (route!==lastRoute){lastRoute=route;contentStyle?.remove();contentStyle=null;checkBlock();}
    tick();
    if(Date.now()%5000<1000)checkBlock();
  },1000);
  checkBlock();
})();
