import {CompanionUpdater} from './CompanionUpdater';
import {useConfirm} from './ConfirmDialog';
import {useEffect,useState} from 'react';
import {openExtensionFolder,pairBrowser,forgetBrowserPair,getBrowserPairState,isTauri} from './api';
import type {BrowserPair,PendingBrowser} from './api';

type Browser='Brave'|'Chrome'|'Edge';
const steps=[
  {title:'Find extension',hint:'Everything is included with the Windows installer. No extra download.'},
  {title:'Load in browser',hint:'Brave requires a one-time permission to install an unpacked extension.'},
  {title:'Authorize extension',hint:'Approve this Extension ID in Digital Discipline. No Windows registry setup.'},
  {title:'Enable & verify',hint:'Direct local connection on 127.0.0.1. No native host.'},
  {title:'Test video counts',hint:'Verify the connection and live Shorts playback before adding restrictions.'}
];
function Visual({kind}:{kind:number}){
  const titles=['Installed files','Brave extensions','Direct local bridge','Consent & connection','Shorts · local dashboard'];
  return <div className="wizard-illustration" role="img" aria-label={`Illustration of ${titles[kind]}`}>
    <div className="wizard-faux-top"><span/><span/><span/><small>{titles[kind]}</small></div>
    {kind===0&&<div className="wizard-visual-body"><svg viewBox="0 0 360 140" width="100%" height="130" aria-hidden="true"><rect x="40" y="20" width="280" height="108" rx="9" fill="#2e2e35" stroke="#73737a"/><path d="M68 54h39l12 12h75v45H68z" fill="#bdbdc6"/><rect x="206" y="45" width="86" height="10" rx="5" fill="#6f6f7b"/><rect x="206" y="64" width="70" height="8" rx="4" fill="#555561"/><rect x="206" y="83" width="80" height="8" rx="4" fill="#555561"/><text x="130" y="134" fontSize="10" fill="#dfdfe4" textAnchor="middle">browser-extension /</text></svg></div>}
    {kind===1&&<div className="wizard-visual-body"><div className="wizard-faux-row"><span>Developer mode</span><span className="wizard-faux-toggle"/></div><div className="wizard-faux-action"><span>Load unpacked</span><span>+</span></div><p className="wizard-faux-caption">Select the folder from step 1</p></div>}
    {kind===2&&<div className="wizard-visual-body"><div className="wizard-faux-row"><span>Brave extension</span><span className="wizard-faux-tag">ID</span></div><div className="wizard-faux-input">32-letter extension ID</div><div className="wizard-faux-row"><span>Desktop agent</span><strong className="wizard-faux-connected">● local only</strong></div><div className="wizard-faux-connector"/></div>}
    {kind===3&&<div className="wizard-visual-body"><div className="wizard-faux-row"><span>Record browser activity</span><span className="wizard-faux-toggle"/></div><div className="wizard-faux-row"><span>Agent connected (localhost)</span><strong className="wizard-faux-connected">● connected</strong></div><p className="wizard-faux-caption">Your extension must show this status after testing.</p></div>}
    {kind===4&&<div className="wizard-visual-body"><div className="wizard-mini-stats"><div><strong>3</strong><small>qualified plays</small></div><div><strong>3</strong><small>unique Shorts</small></div><div><strong>0</strong><small>queued events</small></div></div><p className="wizard-faux-caption">Illustrative targets for a clean three-Short test, not live data.</p></div>}
  </div>;
}
export function BrowserSetupWizard(){
 const confirm=useConfirm();
 const [step,setStep]=useState(0);const [browser,setBrowser]=useState<Browser>('Brave');const [id,setId]=useState('');
 const [folder,setFolder]=useState('');const [notice,setNotice]=useState('');const [busy,setBusy]=useState(false);
 const [pairs,setPairs]=useState<BrowserPair[]>([]);
 const [pending,setPending]=useState<PendingBrowser[]>([]);
 async function reloadPairs(){try{const state=await getBrowserPairState();setPairs(state.pairs);setPending(state.pending);}catch(e){setNotice(String(e));}}
 useEffect(()=>{if(isTauri)void reloadPairs();},[]);
 const valid=/^[a-p]{32}$/.test(id.trim());
 const alreadyPaired=pairs.some(p=>p.extension_id===id.trim());
 useEffect(()=>{setNotice('');},[step]);
 async function folderOpen(){setBusy(true);try{setFolder(await openExtensionFolder());setNotice('File Explorer opened. In Brave, select this exact folder.');}catch(e){setNotice(String(e));}finally{setBusy(false);}}
 async function pair(){if(!valid)return;setBusy(true);try{const msg=await pairBrowser(browser,id.trim());setNotice(msg);await reloadPairs();}catch(e){setNotice(String(e));}finally{setBusy(false);}}
 async function approvePending(extensionId:string){setBusy(true);try{setNotice(await pairBrowser(browser,extensionId));await reloadPairs();}catch(e){setNotice(String(e));}finally{setBusy(false);}}
 async function unpair(extensionId:string){if(!await confirm({title:'Disconnect this browser?',description:'Remove this extension from the local allowlist. Its browser profile, pending queue and recording preference stay on your device.',confirmLabel:'Disconnect browser',danger:true}))return;
  setBusy(true);try{await forgetBrowserPair(extensionId);await reloadPairs();setNotice('Authorization removed. Extension will disconnect on its next request.');}catch(e){setNotice(String(e));}finally{setBusy(false);}}
 const next=()=>setStep(n=>Math.min(4,n+1));
 return <article className="surface padded wizard-root" aria-label="Browser setup wizard">
   <div className="wizard-header"><div><p className="eyebrow">GUIDED CONNECTION</p><h3>Connect your browser in five steps</h3><p className="muted-paragraph">No command line, registry or native host. Direct local connection requires this extension to be authorized once.</p></div><span className="wizard-counter">{step+1} / 5</span></div>
   <div className="wizard-progress" aria-label={`Step ${step+1} of 5`}>{steps.map((v,i)=><button key={v.title} className={step===i?'current':step>i?'complete':''} onClick={()=>setStep(i)} title={v.title} aria-label={`Go to step ${i+1}: ${v.title}`}/>)}</div>
   <div className="wizard-layout"><Visual kind={step}/><div className="wizard-body"><h4>{steps[step].title}</h4><p>{steps[step].hint}</p>
   {step===0&&<><p>For first-time setup, open the extension folder included with the installer. Already have an unpacked extension with queued events? Keep its existing folder and ID; update its source files in place and Reload it instead of installing a second copy.</p><button type="button" className="primary-btn" disabled={busy||!isTauri} onClick={()=>void folderOpen()}>Open included extension folder</button>{folder&&<code className="wizard-path">{folder}</code>}</>}
   {step===1&&<><div className="wizard-numbered"><span>1. Open <code>{browser.toLowerCase()==='brave'?'brave://extensions':browser==='Chrome'?'chrome://extensions':'edge://extensions'}</code> in your browser.</span><span>2. Turn on <b>Developer mode</b>.</span><span>3. If new, press <b>Load unpacked</b> and choose the folder from Step 1. If upgrading, Reload your existing extension in place.</span><span>4. Pin the Digital Discipline extension; open its popup and press Connect. It should appear in the pending list below (or copy its ID manually).</span></div><p className="wizard-caution">Existing extension with unsent events? Do not remove it or Load unpacked from a different folder. Keep the same existing folder path, replace its extension source files in place, and press Reload. This preserves its Chrome extension ID and local event queue.</p></>}
   {step===2&&<><label>Browser<div className="wizard-browser-options">{(['Brave','Chrome','Edge'] as Browser[]).map(b=><button type="button" key={b} aria-pressed={browser===b} className={browser===b?'selected':''} onClick={()=>{setBrowser(b);}}>{b}</button>)}</div></label><label>Extension ID<input spellCheck={false} autoCapitalize="off" value={id} maxLength={32} onChange={e=>{setId(e.target.value.trim().toLowerCase());}} placeholder="Optional: paste the 32-letter ID" /></label><button type="button" disabled={!valid||busy||alreadyPaired} className="primary-btn" onClick={()=>void pair()}>{alreadyPaired?"Already authorized":"Authorize this extension"}</button><p className="form-hint">Simplest path: open the extension popup → Connect, then Refresh the pending list below and click Approve. You can also paste the ID manually. No registry or native host.</p></>}
   {step===3&&<><div className="wizard-numbered"><span>1. Open <code>{browser.toLowerCase()==='brave'?'brave://extensions':browser==='Chrome'?'chrome://extensions':'edge://extensions'}</code> and click <b>Reload</b> on the extension.</span><span>2. Open its popup; enable <b>Record browser activity</b> if you want video counts.</span><span>3. Click <b>Connect directly & sync events</b>.</span><span>4. Confirm <b>Connected to local Windows agent</b> and check that queued events decrease.</span></div><button type="button" className="secondary-btn" disabled={!valid||busy} onClick={()=>void reloadPairs()}>Refresh paired devices</button><p className="wizard-caution">Extension popup should say Connected to local Windows agent. If it shows not authorized, check the ID below. If it shows Failed to fetch, confirm the agent is running and Brave allows the extension to connect to 127.0.0.1.</p></>}
   {step===4&&<><div className="wizard-numbered"><span>1. In Brave, open three different <code>youtube.com/shorts/…</code> URLs.</span><span>2. Let each Short visibly play for at least five seconds.</span><span>3. Return to <b>Video Intelligence</b> in the desktop dashboard, then refresh.</span><span>4. If counts stay at zero, reopen the extension popup: the queued count should fall, not keep climbing. Then open two regular watch videos for separate counting.</span></div><p className="wizard-caution">Regular <code>youtube.com/watch?v=…</code> observations now have separate counts. Play two different ordinary videos after checking Shorts and confirm the two categories do not mix.</p></>}
   </div></div>
   <CompanionUpdater/>
   <div className="wizard-diagnostics" aria-label="Browser connection management">
    <div className="eyebrow">NEW CONNECTION</div>
    <div className="panel-header"><strong>Connect a new browser ({pending.length} waiting)</strong><button type="button" className="secondary-btn" onClick={()=>void reloadPairs()} disabled={busy}>Find extensions</button></div>
    {pending.length?pending.map(p=><div className="settings-row" key={p.extension_id}><div><strong>Browser companion detected</strong><small>{p.extension_id}</small><small>Requested {new Date(p.requested_ms).toLocaleTimeString()} · choose browser above</small></div><button type="button" className="primary-btn" disabled={busy} onClick={()=>void approvePending(p.extension_id)}>Approve</button></div>):<p className="form-hint">No pending requests. Open the extension popup and click Connect directly, then click Find extensions here.</p>}
    <div className="small-divider"/>
    <div className="eyebrow" style={{marginTop:18}}>CONNECTED BROWSERS</div>
    <div className="panel-header"><strong>Authorized browser extensions ({pairs.length})</strong><button type="button" className="secondary-btn" disabled={busy} onClick={()=>void reloadPairs()}>Refresh</button></div>
     {pairs.length?pairs.map(p=><div key={p.extension_id} className="settings-row"><div><strong>{p.browser} · {p.extension_id.slice(0,6)}…{p.extension_id.slice(-6)}</strong><small>Extension ID: {p.extension_id}</small><small>{p.last_seen_ms>0?'Last connected: '+new Date(p.last_seen_ms).toLocaleString():'Authorized · awaiting first connection'}</small></div><button className="secondary-btn" type="button" disabled={busy} onClick={()=>void unpair(p.extension_id)}>Unpair</button></div>):<p className="form-hint">No authorized extensions. Pair your Brave extension in step 3.</p>}
     <p className="form-hint">These are actual agent authorizations. A Last connected timestamp updates when the companion communicates. You can revoke access any time.</p>
   </div>
   {notice&&<p className="wizard-notice" role="status">{notice}</p>}
   <div className="wizard-footer"><button className="secondary-btn" onClick={()=>setStep(n=>Math.max(0,n-1))} disabled={step===0}>Previous</button><button className="primary-btn" onClick={next} disabled={step===4}>Next step</button></div>
 </article>;
}
