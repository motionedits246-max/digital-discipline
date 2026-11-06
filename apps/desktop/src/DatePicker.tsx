import {useEffect,useId,useRef,useState} from 'react';
import type {KeyboardEvent} from 'react';
import {createPortal} from 'react-dom';
import {CustomSelect} from './CustomSelect';
import {isoDay,parseDay,shiftDay,shiftMonth,monthDays} from './calendar.mjs';

type Props={value:string;onChange:(value:string)=>void;min?:string;max?:string;disabled?:boolean;'aria-label'?:string};
export function DatePicker({value,onChange,min='1900-01-01',max='2100-12-31',disabled=false,'aria-label':label='Choose day'}:Props){
 const today=isoDay(new Date()),selected=parseDay(value)?value:today;
 const [open,setOpen]=useState(false),[active,setActive]=useState(selected),[position,setPosition]=useState({left:8,top:8,width:296,height:360});
 const trigger=useRef<HTMLButtonElement>(null),panel=useRef<HTMLDivElement>(null),uid=useId();
 const valid=(day:string)=>!!parseDay(day)&&day>=min&&day<=max;
 const close=(restore=true)=>{setOpen(false);if(restore)trigger.current?.focus();};
 const choose=(day:string)=>{if(!valid(day))return;onChange(day);close();};
 const move=(day:string)=>{if(valid(day))setActive(day);};
 useEffect(()=>{
  if(!open)return;
  const place=()=>{const box=trigger.current?.getBoundingClientRect();if(!box)return;const width=Math.min(296,window.innerWidth-16),height=Math.min(366,window.innerHeight-16);
   const below=window.innerHeight-box.bottom-8;setPosition({width,height,left:Math.max(8,Math.min(box.left,window.innerWidth-width-8)),top:Math.max(8,Math.min(below>=height?box.bottom+6:box.top-height-6,window.innerHeight-height-8))});};
  const outside=(e:PointerEvent)=>{const node=e.target as Node;if(!trigger.current?.contains(node)&&!panel.current?.contains(node)&&!(node instanceof Element&&node.closest('.dd-select-list')))close(false);};
  place();window.addEventListener('resize',place);window.addEventListener('scroll',place,true);document.addEventListener('pointerdown',outside);
  return()=>{window.removeEventListener('resize',place);window.removeEventListener('scroll',place,true);document.removeEventListener('pointerdown',outside);};
 },[open]);
 useEffect(()=>{if(open){const button=panel.current?.querySelector<HTMLButtonElement>('[data-day="'+active+'"]');button?.focus();button?.scrollIntoView({block:'nearest'});}},[open,active]);
 const current=parseDay(active)!,months=Array.from({length:12},(_,i)=>new Date(2000,i,1).toLocaleDateString(undefined,{month:'long'}));
 function key(e:KeyboardEvent){
  const shifts:Record<string,number>={ArrowLeft:-1,ArrowRight:1,ArrowUp:-7,ArrowDown:7};
  if(e.key==='Escape'){e.preventDefault();e.stopPropagation();close();}
  else if(e.target instanceof HTMLElement&&e.target.dataset.day){
   if(e.key in shifts){e.preventDefault();move(shiftDay(active,shifts[e.key]));}
   else if(e.key==='Home'||e.key==='End'){e.preventDefault();move(shiftDay(active,(e.key==='Home'?0:6)-current.getDay()));}
   else if(e.key==='PageUp'||e.key==='PageDown'){e.preventDefault();move(shiftMonth(active,(e.key==='PageUp'?-1:1)*(e.shiftKey?12:1)));}
  }
 }
 return <span className="dd-date"><button ref={trigger} type="button" className="dd-date-trigger" disabled={disabled} aria-label={label} aria-haspopup="dialog" aria-expanded={open} aria-controls={open?uid:undefined} onClick={()=>{setActive(selected<min?min:selected>max?max:selected);setOpen(v=>!v);}}>
  <span>{parseDay(value)?.toLocaleDateString(undefined,{day:'2-digit',month:'short',year:'numeric'})||'Choose date'}</span><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" aria-hidden="true"><rect x="3" y="5" width="18" height="16" rx="3"/><path d="M3 10h18M8 2v6M16 2v6"/></svg>
 </button>{open&&createPortal(<div ref={panel} id={uid} role="dialog" aria-label={label} className="dd-calendar" style={{position:'fixed',left:position.left,top:position.top,width:position.width,maxHeight:position.height}} onKeyDown={key}>
  <div className="calendar-heading"><button type="button" className="icon-btn" aria-label="Previous month" disabled={!valid(shiftMonth(active,-1))} onClick={()=>move(shiftMonth(active,-1))}>‹</button>
   <CustomSelect aria-label="Month" value={current.getMonth()} onChange={e=>move(shiftMonth(active,+e.target.value-current.getMonth()))}>{months.map((name,i)=><option value={i} key={name}>{name}</option>)}</CustomSelect>
   <CustomSelect aria-label="Year" value={current.getFullYear()} onChange={e=>move(shiftMonth(active,(+e.target.value-current.getFullYear())*12))}>{Array.from({length:211},(_,i)=><option value={1900+i} key={1900+i}>{1900+i}</option>)}</CustomSelect>
   <button type="button" className="icon-btn" aria-label="Next month" disabled={!valid(shiftMonth(active,1))} onClick={()=>move(shiftMonth(active,1))}>›</button></div>
  <div role="grid" aria-label={months[current.getMonth()]+' '+current.getFullYear()} className="calendar-grid"><div role="row" className="calendar-week">{['Su','Mo','Tu','We','Th','Fr','Sa'].map(d=><span role="columnheader" key={d}>{d}</span>)}</div>
   {Array.from({length:6},(_,week)=><div role="row" className="calendar-week" key={week}>{monthDays(active).slice(week*7,week*7+7).map(day=><button role="gridcell" type="button" data-day={day} aria-label={parseDay(day)!.toLocaleDateString(undefined,{dateStyle:'full'})} aria-selected={day===value} aria-current={day===today?'date':undefined} tabIndex={day===active?0:-1} disabled={!valid(day)} className={(day.slice(0,7)!==active.slice(0,7)?'adjacent ':'')+(day===value?'selected ':'')+(day===today?'today':'')} key={day} onClick={()=>choose(day)}>{+day.slice(8)}</button>)}</div>)}</div>
  <div className="calendar-footer"><button type="button" className="secondary-btn" disabled={!valid(today)} onClick={()=>choose(today)}>Today</button><button type="button" className="secondary-btn" onClick={()=>close()}>Close</button></div>
 </div>,document.body)}</span>;
}
