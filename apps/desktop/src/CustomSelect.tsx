import {Children,isValidElement,useEffect,useId,useMemo,useRef,useState} from 'react';
import type {KeyboardEvent,ReactNode} from 'react';
import {createPortal} from 'react-dom';

type Opt={value:string;label:string;disabled:boolean};
type Props={onOpenChange?:(open:boolean)=>void;value:string|number;onChange:(event:{target:{value:string}})=>void;children:ReactNode;disabled?:boolean;required?:boolean;className?:string;'aria-label'?:string};

function optionText(node:ReactNode):string{
 return Children.toArray(node).map(child=>typeof child==='string'||typeof child==='number'?String(child):isValidElement(child)?optionText((child.props as {children?:ReactNode}).children):'').join('');
}

/** Portal-backed, keyboard-operable listbox: avoids native Windows dropdown paint and scroll clipping. */
export function CustomSelect({value,onChange,children,disabled=false,className='',onOpenChange,...props}:Props){
 const options=useMemo<Opt[]>(()=>Children.toArray(children).filter(isValidElement).map(child=>{
   const item=child.props as {value?:string|number;children?:ReactNode;disabled?:boolean};
   const label=optionText(item.children);return {value:String(item.value??label),label,disabled:!!item.disabled};
 }),[children]);
 const [open,setOpen]=useState(false);const [position,setPosition]=useState({left:0,top:0,width:0,up:false,maxHeight:230});const [active,setActive]=useState(0);
 useEffect(()=>{onOpenChange?.(open);},[open]);
 const button=useRef<HTMLButtonElement>(null);const menu=useRef<HTMLDivElement>(null);const uid=useId();
 const current=options.find(o=>o.value===String(value));
 const pos=()=>{const r=button.current?.getBoundingClientRect();if(!r)return;const below=window.innerHeight-r.bottom-12;const above=r.top-12;const up=below<195&&above>below;const height=Math.max(40,Math.min(250,up?above:below,window.innerHeight-16));const width=Math.min(r.width,window.innerWidth-16);setPosition({left:Math.max(8,Math.min(r.left,window.innerWidth-width-8)),top:up?r.top-5:r.bottom+5,width,up,maxHeight:height});};
 const select=(option:Opt)=>{if(option.disabled)return;onChange({target:{value:option.value}});setOpen(false);button.current?.focus();};
 useEffect(()=>{if(!open)return;pos();const update=()=>pos();const mask=()=>setOpen(false);const outside=(e:PointerEvent)=>{if(!button.current?.contains(e.target as Node)&&!menu.current?.contains(e.target as Node))setOpen(false);};window.addEventListener('resize',update);window.addEventListener('scroll',update,true);window.addEventListener('dd:privacy-mask',mask);document.addEventListener('pointerdown',outside);return()=>{window.removeEventListener('resize',update);window.removeEventListener('scroll',update,true);window.removeEventListener('dd:privacy-mask',mask);document.removeEventListener('pointerdown',outside);};},[open]);
 useEffect(()=>{if(open){const idx=options.findIndex(o=>o.value===String(value));setActive(Math.max(0,idx));}},[open,value]);
 useEffect(()=>{if(open){const el=menu.current?.querySelector(`[data-index="${active}"]`);el?.scrollIntoView({block:'nearest'});}},[active,open]);
 function keyboard(e:KeyboardEvent){if(disabled)return;const n=options.length;if(!n)return;
   if(e.key==='Escape'){if(open){e.preventDefault();e.stopPropagation();setOpen(false);button.current?.focus();}return;}
   if(e.key==='Enter'||e.key===' '){e.preventDefault();if(open)select(options[active]);else setOpen(true);return;}
   if(e.key==='ArrowDown'||e.key==='ArrowUp'||e.key==='Home'||e.key==='End'){
     e.preventDefault();if(!open){setOpen(true);return;}
     const direction=e.key==='ArrowDown'?1:e.key==='ArrowUp'?-1:0;
     const start=e.key==='Home'?0:e.key==='End'?n-1:active+direction;
     for(let i=0;i<n;i++){const j=(start+n+(direction||1)*i)%n;if(!options[j].disabled){setActive(j);break;}}return;
   }
   if(open&&e.key.length===1&&!e.ctrlKey&&!e.altKey&&!e.metaKey){const idx=options.findIndex(o=>o.label.toLowerCase().startsWith(e.key.toLowerCase())&&!o.disabled);if(idx>=0)setActive(idx);}
 }
 return <span className={'dd-select '+className}>
   <button ref={button} type="button" className="dd-select-trigger" disabled={disabled} role="combobox" aria-label={props['aria-label']} aria-controls={open?uid:undefined} aria-activedescendant={open?uid+'-'+active:undefined} aria-expanded={open} aria-haspopup="listbox" aria-required={props.required} onClick={()=>setOpen(!open)} onKeyDown={keyboard}>
     <span>{current?.label||'Select an option'}</span><svg width="15" height="15" viewBox="0 0 24 24" aria-hidden="true"><path d="m6 9 6 6 6-6" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/></svg>
   </button>
   {open&&createPortal(<div ref={menu} id={uid} role="listbox" aria-label={props['aria-label']||'Select option'} className={'dd-select-list'+(position.up?' upwards':'')} style={{position:'fixed',left:position.left,top:position.top,width:position.width,maxHeight:position.maxHeight,transform:position.up?'translateY(-100%)':undefined}} onKeyDown={keyboard}>
     {options.map((o,i)=><button type="button" key={o.value+'-'+i} data-index={i} id={uid+'-'+i} role="option" aria-selected={o.value===String(value)} aria-disabled={o.disabled} tabIndex={-1} disabled={o.disabled} className={'dd-select-option'+(i===active?' active':'')+(o.value===String(value)?' selected':'')} onPointerEnter={()=>setActive(i)} onClick={()=>select(o)}><span>{o.label}</span>{o.value===String(value)&&<span aria-hidden="true">✓</span>}</button>)}
   </div>,document.body)}
 </span>;
}
