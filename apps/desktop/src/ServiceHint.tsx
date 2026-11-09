import {useCallback,useEffect,useMemo,useRef,useState} from 'react';
import {agent,isTauri,deliverServiceHint,deliverDesktopNotice,setServiceHintWindow,readServiceHint,openServiceReview,openNoticePage,getBrowserPairState} from './api';
import {CustomSelect} from './CustomSelect';
import {SERVICE_TYPES,serviceName,type Service} from './ServiceReviewPanel';
import {createNoticeLifetime} from './noticeLifetime.mjs';
import {createNoticeFeed} from './noticeFeed.mjs';
import type {Page} from './types';

export type DesktopNotice={kind:'browser'|'status';title:string;detail:string;page:Page};
export type OverlayNotice={kind:'service';service:Service}|DesktopNotice;
type StatusItem={id:string;kind:string;title:string;created_ms:number;dismissed_ms:number|null};

export function useServiceHints(enabled:boolean,lite=false){
 const feed=useRef(createNoticeFeed());
 useEffect(()=>{
  if(!enabled||!isTauri)return;
  let alive=true,busy=false;
  const poll=async()=>{
   if(busy)return;busy=true;
   try{
    // Preserve an engaged popup and do not consume another service suggestion.
    if(await readServiceHint()||!alive)return;
    const pairs=await getBrowserPairState();if(!alive)return;
    const inbox=await agent<{items:StatusItem[]}>('GET','/inbox').catch(()=>({items:[]}));if(!alive)return;
    const pending=feed.current.next(pairs,inbox.items);
    if(pending){if(alive&&await deliverDesktopNotice(pending.notice))feed.current.acknowledge(pending.key);return;}
    const r=await agent<{service:Service|null}>('POST','/services/next',{});
    if(alive&&r.service)await deliverServiceHint(r.service);
   }catch{}finally{busy=false;}
  };
  void poll();const id=setInterval(()=>void poll(),lite?12000:3000);
  return()=>{alive=false;clearInterval(id);};
 },[enabled,lite]);
}

export function ServiceHintSurface(){
 const [notice,setNotice]=useState<OverlayNotice|null>(null),[category,setCategory]=useState('Unclassified'),[busy,setBusy]=useState(false),[leaving,setLeaving]=useState(false),[message,setMessage]=useState('');
 const generation=useRef(0);
 const lifetime=useMemo(()=>createNoticeLifetime({onShow:()=>setLeaving(false),onExit:()=>setLeaving(true),onHide:()=>{generation.current++;void setServiceHintWindow(false,false);}}),[]);
 const show=useCallback((n:OverlayNotice)=>{
  generation.current++;setNotice(n);setCategory(n.kind==='service'?n.service.category:'Unclassified');setBusy(false);setMessage('');
  void setServiceHintWindow(true,false);lifetime.show();
 },[lifetime]);
 const away=useCallback(()=>{
  window.dispatchEvent(new Event('dd:notice-away'));
  if(lifetime.away())void setServiceHintWindow(true,false);
 },[lifetime]);
 useEffect(()=>{
  document.body.classList.add('service-hint-window');
  try{document.body.classList.toggle('dd-lite-mode',localStorage.getItem('dd:lite-mode')==='true');}catch{}
  let disposed=false;const subscriptions:(()=>void)[]=[];
  void import('@tauri-apps/api/event').then(async({listen})=>{
   const offShow=await listen<OverlayNotice>('dd:service-hint',e=>show(e.payload));if(disposed)offShow();else subscriptions.push(offShow);
   const offAway=await listen('dd:notice-away',away);if(disposed)offAway();else subscriptions.push(offAway);
   if(!disposed){const initial=await readServiceHint();if(!disposed&&initial)show(initial);}
  }).catch(()=>{});
  window.addEventListener('blur',away);
  const outside=(event:PointerEvent)=>{if(!(event.target as Element).closest('.service-hint-card,.dd-select-list'))away();};
  document.addEventListener('pointerdown',outside);
  return()=>{disposed=true;subscriptions.forEach(off=>off());lifetime.dispose();window.removeEventListener('blur',away);document.removeEventListener('pointerdown',outside);document.body.classList.remove('service-hint-window');};
 },[show,away,lifetime]);
 const menu=useCallback((open:boolean)=>{if(open)lifetime.interact();void setServiceHintWindow(true,open);},[lifetime]);
 async function decide(action:'category'|'approve_ai'){
  if(notice?.kind!=='service'||busy)return;const version=generation.current;setBusy(true);
  try{await agent('POST','/services/decision',{subject:notice.service.subject,action,category:action==='category'?category:null});if(version===generation.current)setMessage(action==='category'?'Remembered for next time':'Approved · review queued');}
  catch{if(version===generation.current)setMessage('Open Service discovery to review this item.');}
  finally{if(version===generation.current)setBusy(false);}
 }
 if(!notice)return null;
 const service=notice.kind==='service'?notice.service:null;
 const title=notice.kind==='service'?serviceName(notice.service.subject):notice.title;
 return <div className={'service-hint-card '+(leaving?'leaving':'')} role="region" aria-live="polite" aria-label={service?'New service suggestion':'Desktop notification'} onFocusCapture={()=>lifetime.interact()} onPointerDownCapture={()=>lifetime.interact()} onKeyDown={e=>{if(e.key==='Escape'){e.preventDefault();lifetime.dismiss();}}}>
  <div className="service-hint-heading"><span className="service-hint-symbol" aria-hidden="true">◐</span><div><small>{service?(service.category==='Unclassified'?'CHOOSE A SERVICE TYPE':'SERVICE TYPE SUGGESTION'):notice.kind==='browser'?'BROWSER COMPANION':'DIGITAL DISCIPLINE'}</small><strong title={title}>{title}</strong></div><button type="button" className="icon-btn" aria-label={service?'Dismiss service suggestion':'Dismiss notification'} onClick={()=>lifetime.dismiss()}>×</button></div>
  {service?<><div className="service-hint-controls"><CustomSelect aria-label="Service type" value={category} onChange={e=>setCategory(e.target.value)} onOpenChange={menu}>{SERVICE_TYPES.map(c=><option key={c}>{c}</option>)}</CustomSelect><button type="button" className="secondary-btn" disabled={busy} onClick={()=>void decide('category')}>Save</button><button type="button" className="secondary-btn" disabled={busy} onClick={()=>service.ai_enabled?void decide('approve_ai'):void openServiceReview()}>{service.ai_enabled?'Ask AI':'Review'}</button></div><small className="service-hint-foot" role="status">{message||(service.category==='Unclassified'?'No LLM request until you approve.':service.source==='duckduckgo_summary'&&service.subject.startsWith('app:')?'Public match · Save to confirm this type':'From '+service.source.replaceAll('_',' ')+' · change if needed')}</small></>:notice.kind!=='service'&&<><p className="desktop-notice-detail">{notice.detail}</p><button type="button" className="secondary-btn desktop-notice-action" onClick={()=>void openNoticePage(notice.page)}>Open details</button></>}
 </div>;
}
