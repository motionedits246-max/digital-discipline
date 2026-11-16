import {createContext,useContext,useEffect,useId,useRef,useState} from 'react';
import type {ReactNode} from 'react';
import {createPortal} from 'react-dom';
type Request={title:string;description:string;confirmLabel?:string;danger?:boolean};
type Pending=Request&{resolve:(value:boolean)=>void;focus:HTMLElement|null};
const Context=createContext<(request:Request)=>Promise<boolean>>(async()=>false);
export const useConfirm=()=>useContext(Context);
export function ConfirmProvider({children}:{children:ReactNode}){
 const [pending,setPending]=useState<Pending|null>(null),current=useRef<Pending|null>(null),dialog=useRef<HTMLDivElement>(null),id=useId();
 const finish=(answer:boolean)=>{const p=current.current;if(!p)return;current.current=null;setPending(null);p.resolve(answer);requestAnimationFrame(()=>{if(p.focus?.isConnected&&!p.focus.closest('[inert]'))p.focus.focus();});};
 useEffect(()=>{const privacy=()=>finish(false);window.addEventListener('dd:privacy-mask',privacy);window.addEventListener('dd:surface-collapse',privacy);return()=>{window.removeEventListener('dd:surface-collapse',privacy);window.removeEventListener('dd:privacy-mask',privacy);current.current?.resolve(false);};},[]);
 useEffect(()=>{if(!pending)return;dialog.current?.querySelector<HTMLButtonElement>('[data-cancel]')?.focus();},[pending]);
 const ask=(request:Request)=>new Promise<boolean>(resolve=>{current.current?.resolve(false);const next={...request,resolve,focus:document.activeElement as HTMLElement|null};current.current=next;setPending(next);});
 return <Context.Provider value={ask}><div className="app-root" inert={!!pending||undefined}>{children}</div>{pending&&createPortal(<div className="dd-dialog-mask" onPointerDown={e=>{if(e.target===e.currentTarget)finish(false);}}><div ref={dialog} role="alertdialog" aria-modal="true" aria-labelledby={id+'-title'} aria-describedby={id+'-body'} className="dd-confirm-dialog" onKeyDown={e=>{if(e.key==='Escape'){e.preventDefault();e.stopPropagation();finish(false);}if(e.key==='Tab'){const buttons=Array.from(dialog.current?.querySelectorAll<HTMLButtonElement>('button')||[]);const first=buttons[0],last=buttons.at(-1);if(e.shiftKey&&document.activeElement===first){e.preventDefault();last?.focus();}else if(!e.shiftKey&&document.activeElement===last){e.preventDefault();first?.focus();}}}}><span className="eyebrow">DIGITAL DISCIPLINE</span><h2 id={id+'-title'}>{pending.title}</h2><p id={id+'-body'}>{pending.description}</p><div className="device-actions"><button type="button" data-cancel className="secondary-btn" onClick={()=>finish(false)}>Cancel</button><button type="button" className={pending.danger?'danger-btn':'primary-btn'} onClick={()=>finish(true)}>{pending.confirmLabel||'Continue'}</button></div></div></div>,document.body)}</Context.Provider>;
}
