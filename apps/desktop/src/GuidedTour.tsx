import {useEffect,useState} from 'react';
import type {Page} from './types';

type Step={title:string;body:string;page:Page;selector:string};
const steps:Step[]=[
 {title:'Your activity at a glance',body:'Overview shows observed application time, your activity map and the current status. Missing observations are never presented as zero usage.',page:'Overview',selector:'[data-tour="overview"]'},
 {title:'Live activity',body:'See which application is currently active. Tracking continues in the background when you hide the workspace.',page:'Live Activity',selector:'[data-tour="live"]'},
 {title:'Your activity history',body:'Inspect recorded sessions and idle time. Individual window titles and document contents are not required.',page:'Activity History',selector:'[data-tour="history"]'},
 {title:'Shorts and video counts',body:'Video Intelligence shows playback observed through your browser companion. The extension needs recording permission to count.',page:'Video Intelligence',selector:'[data-tour="videos"]'},
 {title:'Focus and goals',body:'Set a goal, choose your focus difficulty and review your actual progress.',page:'Focus & Goals',selector:'[data-tour="focus"]'},
 {title:'Blocking Center',body:'Apply reversible browser restrictions. Emergency unblock remains accessible in the companion.',page:'Blocking Center',selector:'[data-tour="blocking"]'},
 {title:'Automations',body:'Create optional limits and scheduled rules. Rules only activate when enabled.',page:'Automations',selector:'[data-tour="automations"]'},
 {title:'Insights',body:'Review measured patterns and the uncertainty in any prediction or hypothetical time recovery.',page:'Behavior Insights',selector:'[data-tour="insights"]'},
 {title:'AI Coach',body:'Ask questions in a chat. External requests require your consent; keys can be protected locally by Windows DPAPI.',page:'AI Coach',selector:'[data-tour="coach"]'},
 {title:'Settings and privacy',body:'Manage recorded data, browser connections, encrypted classifier settings, and exporting or deleting your history.',page:'Settings & Privacy',selector:'[data-tour="settings"]'},
 {title:'Navigate from anywhere',body:'Use Ctrl+K to find any page, and Ctrl+Shift+D to open the workspace. The tray menu can reopen or fully quit the app.',page:'Overview',selector:'[data-tour="command"]'}
];
type Rect={top:number;left:number;width:number;height:number};
export function GuidedTour({active,onFinish,onNavigate}:{active:boolean;onFinish:()=>void;onNavigate:(page:Page)=>void}){
 const [index,setIndex]=useState(0),[spot,setSpot]=useState<Rect|null>(null);
 const [position,setPosition]=useState<{top:number;left:number}>({top:100,left:320});
 const current=steps[index];
 useEffect(()=>{if(active){setIndex(0);onNavigate(steps[0].page);}},[active]);
 useEffect(()=>{
  if(!active)return;
  onNavigate(current.page);
  let frame=0;
  const place=()=>{
   const el=document.querySelector(current.selector) as HTMLElement|null;
   if(el){el.scrollIntoView({block:'nearest',inline:'nearest',behavior:'instant'});}
   const rect=el?.getBoundingClientRect();
   if(!rect||!rect.width||!rect.height){setSpot(null);setPosition({top:Math.max(16,(window.innerHeight-280)/2),left:Math.max(16,(window.innerWidth-350)/2)});return;}
   const pad=5,dialogW=Math.min(350,window.innerWidth-28),dialogH=252,spacing=19;
   const box:Rect={top:Math.max(0,rect.top-pad),left:Math.max(0,rect.left-pad),width:Math.min(window.innerWidth,rect.width+pad*2),height:Math.min(window.innerHeight,rect.height+pad*2)};
   let left=rect.right+spacing,top=rect.top;
   if(left+dialogW>window.innerWidth-14){left=rect.left-dialogW-spacing;}
   if(left<14){left=Math.min(window.innerWidth-dialogW-14,Math.max(14,rect.left));top=rect.bottom+spacing;}
   if(top+dialogH>window.innerHeight-14){top=rect.top-dialogH-spacing;}
   setSpot(box);setPosition({left:Math.max(14,Math.min(left,window.innerWidth-dialogW-14)),top:Math.max(14,Math.min(top,window.innerHeight-dialogH-14))});
  };
  const reposition=()=>{window.cancelAnimationFrame(frame);frame=window.requestAnimationFrame(place);};
  const timer=window.setTimeout(reposition,90);
  window.addEventListener('resize',reposition);
  window.addEventListener('scroll',reposition,true);
  return()=>{window.clearTimeout(timer);window.cancelAnimationFrame(frame);window.removeEventListener('resize',reposition);window.removeEventListener('scroll',reposition,true);};
 },[active,index,current.page,current.selector]);
 if(!active)return null;
 const next=()=>{if(index===steps.length-1){onFinish();return;}setIndex(i=>i+1);};
 return <div className="dd-tour" role="dialog" aria-modal="true" aria-label="Digital Discipline guided walkthrough">
  <div className="dd-tour-backdrop" aria-hidden="true"/>
  {spot&&<div className="dd-tour-spot" aria-hidden="true" style={{top:spot.top,left:spot.left,width:spot.width,height:spot.height}}/>}
  <div className="dd-tour-popover" style={{top:position.top,left:position.left}}>
   <div className="dd-tour-top"><span>GETTING STARTED</span><span>{index+1} / {steps.length}</span></div>
   <h3>{current.title}</h3><p>{current.body}</p>
   <div className="dd-tour-progress">{steps.map((_,i)=><span key={i} className={i<=index?'active':''}/>)}</div>
   <div className="dd-tour-actions">
    <button type="button" className="dd-tour-skip" onClick={onFinish}>Skip tour</button>
    <div className="dd-tour-actions-right">{index>0&&<button type="button" onClick={()=>setIndex(i=>i-1)}>Back</button>}
    <button type="button" className="primary-btn" onClick={next}>{index===steps.length-1?'Finish':'Next'} <span aria-hidden="true">→</span></button></div>
   </div>
  </div>
 </div>;
}
