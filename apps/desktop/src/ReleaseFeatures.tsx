import {useEffect, useState} from 'react';
import type {FormEvent} from 'react';
import {agent, openDataFolder} from './api';
import {CustomSelect} from './CustomSelect';
import {TimePicker} from './TimePicker';
import {useConfirm} from './ConfirmDialog';
import {useAgentPanel} from './ProgressFeatures';

type BackupData = {files:[string,number][]; last_backup_ms:number|null; scheduled:boolean};
type BackupPreview = {digest:string; created_ms:number; history:boolean; rows:Record<string,number>};

export function BackupPanel({disabled}: {disabled:boolean}) {
  const panel=useAgentPanel<BackupData>('/backup',disabled);
  const confirm=useConfirm();
  const [secret,setSecret]=useState('');
  const [history,setHistory]=useState(true);
  const [filename,setFilename]=useState('');
  const [preview,setPreview]=useState<BackupPreview|null>(null);
  const [message,setMessage]=useState('');
  if(disabled)return null;

  async function restore() {
    if(!preview || !await confirm({
      title:'Restore this backup?',
      description:'The included data will be replaced. An encrypted recovery copy is saved before the restore.',
      confirmLabel:'Restore backup', danger:true,
    }))return;
    if(await panel.change(async()=>{
      const result=await agent<{recovery:string}>('POST','/backup/restore',{filename,passphrase:secret,digest:preview.digest});
      setMessage('Restore complete. Recovery file: '+result.recovery);
    })){setPreview(null);setSecret('');}
  }

  return <section className="surface padded">
    <div className="panel-header">
      <div><h3>Encrypted backups</h3><p>Save rules, plans and selected history in a passphrase-protected file.</p></div>
      <button className="secondary-btn" onClick={()=>void openDataFolder('backups').catch(e=>setMessage(String(e)))}>Open backup folder</button>
    </div>
    {panel.error&&<p role="alert">{panel.error}</p>}
    {message&&<p role="status">{message}</p>}
    <form className="release-form" onSubmit={e=>{
      e.preventDefault();
      void panel.change(async()=>{
        const result=await agent<{filename:string;bytes:number}>('POST','/backup',{passphrase:secret,history});
        setMessage(`Saved ${result.filename} · ${(result.bytes/1048576).toFixed(1)} MB`);
      });
    }}>
      <label>Backup passphrase<input type="password" autoComplete="new-password" minLength={12} maxLength={256}
        required value={secret} onChange={e=>{setSecret(e.target.value);setPreview(null);}}/></label>
      <label className="release-check"><input type="checkbox" checked={history} onChange={e=>setHistory(e.target.checked)}/> Include activity history</label>
      <div className="release-actions"><button className="primary-btn" disabled={panel.busy}>{panel.busy?'Working…':'Create encrypted backup'}</button></div>
    </form>
    <div className="release-actions">
      <button className="secondary-btn" disabled={panel.busy||(!panel.data?.scheduled&&secret.length<12)}
        onClick={()=>void panel.change(()=>agent('POST','/backup/schedule',{enabled:!panel.data?.scheduled,passphrase:panel.data?.scheduled?null:secret,history}))}>
        {panel.data?.scheduled?'Turn off daily backups':'Schedule daily backups'}
      </button>
      {panel.data?.last_backup_ms&&<span>Last backup {new Date(panel.data.last_backup_ms).toLocaleString()}</span>}
    </div>
    <p className="form-hint">Daily backups keep seven files. Keep your passphrase with your recovery records.</p>
    <div className="release-form release-subsection">
      <label>Restore file<CustomSelect value={filename} onChange={e=>{setFilename(e.target.value);setPreview(null);}} aria-label="Restore file">
        <option value="">Choose a backup</option>
        {panel.data?.files.map(([name,bytes])=><option key={name} value={name}>{name} · {Math.ceil(bytes/1024)} KB</option>)}
      </CustomSelect></label>
      <div className="release-actions">
        <button className="text-action" onClick={()=>void panel.reload()}>Refresh files</button>
        <button className="secondary-btn" disabled={!filename||secret.length<12||panel.busy} onClick={()=>void panel.change(async()=>{
          setPreview(await agent<BackupPreview>('POST','/backup/preview',{filename,passphrase:secret}));
        })}>Preview restore</button>
      </div>
    </div>
    <p className="form-hint">To restore from another PC, copy its .ddbackup file into the backup folder and refresh the list.</p>
    {preview&&<div className="release-subsection">
      <strong>Backup from {new Date(preview.created_ms).toLocaleString()}</strong>
      <p>{preview.history?'Planning, rules and activity history':'Planning, rules and preferences'} · {Object.values(preview.rows).reduce((a,b)=>a+b,0)} records</p>
      <div className="release-actions">
        <button className="secondary-btn" onClick={()=>setPreview(null)}>Cancel</button>
        <button className="danger-btn" disabled={panel.busy} onClick={()=>void restore()}>Review and restore</button>
      </div>
    </div>}
  </section>;
}

type Update = {
  installed:string;available:boolean;expected_companion:string;protocol:string;
  release:{version:string;companion_version:string;sha256:string;signing:string;notes:string};
};
export function UpdatePanel({disabled}: {disabled:boolean}) {
  const [result,setResult]=useState<Update|null>(null);
  const [message,setMessage]=useState('');
  const [busy,setBusy]=useState(false);
  if(disabled)return null;

  async function check() {
    setBusy(true);setMessage('');
    try { setResult(await agent<Update>('GET','/updates/check')); }
    catch(error){setMessage(String(error));}
    finally{setBusy(false);}
  }
  async function verify() {
    if(!result)return;
    setBusy(true);setMessage('Downloading and verifying the installer…');
    try {
      const reply=await agent<{trusted_publisher:boolean;signature:{status:string}}>('POST','/updates/verify',{
        version:result.release.version,sha256:result.release.sha256,
      });
      setMessage(reply.trusted_publisher
        ? 'Checksum and Windows publisher verified. Installer saved in Updates.'
        : 'Development installer saved in Updates. Checksum verified. Windows signature status: '+reply.signature.status);
    }catch(error){setMessage(String(error));}
    finally{setBusy(false);}
  }
  return <section className="surface padded">
    <div className="panel-header">
      <div><h3>Application updates</h3><p>Check the current release and verify the installer before opening it.</p></div>
      <button className="secondary-btn" disabled={busy} onClick={()=>void check()}>Check for updates</button>
    </div>
    {result&&<>
      <div className="release-row"><strong>Installed {result.installed}</strong><span>{result.available?'Available':'Published version'} {result.release.version}</span></div>
      <p className="form-hint">Expected companion {result.expected_companion} · Release companion {result.release.companion_version} · Protocol {result.protocol}</p>
      <p className="muted-paragraph">{result.release.notes}</p>
      <div className="release-actions">
        <button className="secondary-btn" disabled={busy} onClick={()=>void verify()}>Verify and save installer</button>
        <button className="text-action" onClick={()=>void openDataFolder('updates').catch(e=>setMessage(String(e)))}>Open updates folder</button>
      </div>
    </>}
    {message&&<p role="status">{message}</p>}
  </section>;
}

type SyncData = {
  state:{folder:string;enabled:boolean;last_sync_ms:number;last_error:string}|null;code:string|null;
  peers:{id:string;name:string;revision:number;revoked:boolean}[];
};
export function SyncPanel({disabled}: {disabled:boolean}) {
  const panel=useAgentPanel<SyncData>('/sync',disabled);
  const confirm=useConfirm();
  const [folder,setFolder]=useState<string|null>(null);
  const [secret,setSecret]=useState('');
  const [peerName,setPeerName]=useState('');
  const [peerCode,setPeerCode]=useState('');
  const [message,setMessage]=useState('');
  if(disabled)return null;
  const state=panel.data?.state;
  async function save(event:FormEvent) {
    event.preventDefault();
    if(await panel.change(()=>agent('POST','/sync',{enabled:true,folder:folder??state?.folder,passphrase:secret})))setSecret('');
  }
  return <section className="surface padded">
    <div className="panel-header"><div><h3>Encrypted folder sync</h3><p>Sync up to 30 days of activity between enrolled Windows devices through a shared folder.</p></div></div>
    {panel.error&&<p role="alert">{panel.error}</p>}
    {message&&<p role="status">{message}</p>}
    <form className="release-form" onSubmit={save}>
      <label>Shared folder path<input value={folder??state?.folder??''} onChange={e=>setFolder(e.target.value)} required/></label>
      <label>Shared passphrase<input type="password" autoComplete="new-password" minLength={12} maxLength={256}
        required value={secret} onChange={e=>setSecret(e.target.value)}/></label>
      <div className="release-actions"><button className="primary-btn" disabled={panel.busy}>Save sync settings</button></div>
    </form>
    <p className="form-hint">Use the same folder and passphrase on each PC. Shared activity appears in device reports.</p>
    {state?.enabled&&<>
      <div className="release-actions">
        <button className="secondary-btn" disabled={panel.busy} onClick={()=>void panel.change(()=>agent('POST','/sync/now',{}))}>Sync now</button>
        <button className="text-action" disabled={panel.busy} onClick={()=>void panel.change(()=>agent('POST','/sync',{enabled:false}))}>Turn off sync</button>
      </div>
      <p role="status">{state.last_error?state.last_error.replaceAll('_',' '):state.last_sync_ms?'Last sync '+new Date(state.last_sync_ms).toLocaleString():'Waiting for first sync'}</p>
    </>}
    {panel.data?.code&&<div className="release-row"><strong>This device’s enrollment code</strong>
      <button className="secondary-btn" onClick={async()=>{
        try{await navigator.clipboard.writeText(panel.data!.code!);setMessage('Device code copied.');}
        catch{setMessage('Could not copy the device code.');}
      }}>Copy device code</button>
    </div>}
    <form className="release-form release-subsection" onSubmit={e=>{
      e.preventDefault();
      void panel.change(()=>agent('POST','/sync/peer',{name:peerName,code:peerCode,revoke:false})).then(ok=>{
        if(ok){setPeerName('');setPeerCode('');}
      });
    }}>
      <label>Device name<input value={peerName} onChange={e=>setPeerName(e.target.value)} maxLength={80} required/></label>
      <label>Other device’s code<input value={peerCode} onChange={e=>setPeerCode(e.target.value)} maxLength={101} required spellCheck={false}/></label>
      <div className="release-actions"><button className="secondary-btn" disabled={panel.busy}>Enroll device</button></div>
    </form>
    {panel.data?.peers.map(peer=><div key={peer.id} className="release-row">
      <span>{peer.name} · {peer.revoked?'Revoked':peer.revision?'Synced':'Awaiting activity'}</span>
      {!peer.revoked&&<button className="text-action" disabled={panel.busy} onClick={async()=>{
        if(await confirm({title:'Revoke this device?',description:'Remove its imported activity and reject future sync packets.',confirmLabel:'Revoke device',danger:true})){
          await panel.change(()=>agent('POST','/sync/peer',{name:peer.name,code:peer.id,revoke:true}));
        }
      }}>Revoke device</button>}
    </div>)}
  </section>;
}

type Quiet = {dnd:boolean;start:number|null;end:number|null};
type Progression = {enabled:boolean;platform:string;window_minutes:number;stages:number[];block_minutes:number};
type Attention = {
  quiet:Quiet;muted:boolean;reading:{app:string;confirm_by_ms:number}|null;
  terms:Progression;intervention:{enabled:boolean;level:number;plays?:number};
};
function hhmm(minute:number|null){
  return minute===null?'':[Math.floor(minute/60),minute%60].map(value=>String(value).padStart(2,'0')).join(':');
}
function minuteOfDay(value:string){const [h,m]=value.split(':').map(Number);return h*60+m;}

export function AttentionPanel({disabled,mode}: {disabled:boolean;mode:'reading'|'quiet'|'progression'}) {
  const panel=useAgentPanel<Attention>('/attention',disabled);
  const [app,setApp]=useState('');
  const [length,setLength]=useState(10);
  const [quietDraft,setQuietDraft]=useState<Quiet|null>(null);
  const [termsDraft,setTermsDraft]=useState<Progression|null>(null);
  useEffect(()=>{
    if(disabled||mode==='quiet')return;
    const timer=setInterval(()=>void panel.reload(),12000);
    return()=>clearInterval(timer);
  },[disabled,mode,panel.reload]);
  if(disabled)return null;
  const data=panel.data;
  if(!data)return <div className="surface padded" role="status">{panel.error||'Loading attention settings…'}</div>;
  const quiet=quietDraft??data.quiet,terms=termsDraft??data.terms;
  const error=panel.error&&<p role="alert">{panel.error}</p>;

  if(mode==='quiet')return <section className="surface padded">
    <div className="panel-header"><div><h3>Quiet hours and Do Not Disturb</h3><p>Mute desktop and companion alerts while inbox history and approved limits continue.</p></div></div>
    {error}
    <form className="release-form" onSubmit={e=>{e.preventDefault();void panel.change(()=>agent('POST','/attention/quiet',quiet)).then(ok=>{if(ok)setQuietDraft(null);});}}>
      <label className="release-check"><input type="checkbox" checked={quiet.dnd} onChange={e=>setQuietDraft({...quiet,dnd:e.target.checked})}/> Do Not Disturb</label>
      <label className="release-check"><input type="checkbox" checked={quiet.start!==null} onChange={e=>setQuietDraft({...quiet,start:e.target.checked?1320:null,end:e.target.checked?420:null})}/> Schedule quiet hours</label>
      {quiet.start!==null&&<>
        <label>From<TimePicker value={hhmm(quiet.start)} onChange={e=>setQuietDraft({...quiet,start:minuteOfDay(e.target.value)})} aria-label="Quiet hours start"/></label>
        <label>Until<TimePicker value={hhmm(quiet.end)} onChange={e=>setQuietDraft({...quiet,end:minuteOfDay(e.target.value)})} aria-label="Quiet hours end"/></label>
      </>}
      <div className="release-actions"><button className="primary-btn" disabled={panel.busy}>Save quiet hours</button></div>
    </form>
    <p role="status">{data.muted?'Shared alerts muted':'Shared alerts available'}</p>
  </section>;

  if(mode==='reading')return <section className="surface padded">
    <div className="panel-header"><div><h3>Reading session</h3><p>Confirm up to three minutes of foreground reading at a time. Confirmed reading has its own weekly measure.</p></div></div>
    {error}
    {data.reading?<><p>Reading in {data.reading.app} · Confirm by {new Date(data.reading.confirm_by_ms).toLocaleTimeString()}</p>
      <div className="release-actions">
        <button className="primary-btn" disabled={panel.busy} onClick={()=>void panel.change(()=>agent('POST','/attention/reading',{action:'confirm'}))}>Confirm reading</button>
        <button className="secondary-btn" disabled={panel.busy} onClick={()=>void panel.change(()=>agent('POST','/attention/reading',{action:'stop'}))}>End reading</button>
      </div></>:<form className="release-form" onSubmit={e=>{
      e.preventDefault();void panel.change(()=>agent('POST','/attention/reading',{action:'start',app:app.trim().toLowerCase(),minutes:length}));
    }}>
      <label>Reading app<input placeholder="acrord32.exe" value={app} onChange={e=>setApp(e.target.value)} required/></label>
      <label>Session length<CustomSelect value={length} onChange={e=>setLength(Number(e.target.value))} aria-label="Reading session length">
        {[5,10,20,30].map(n=><option key={n} value={n}>{n} min</option>)}
      </CustomSelect></label>
      <div className="release-actions"><button className="primary-btn" disabled={panel.busy}>Start reading</button></div>
    </form>}
  </section>;

  const stages=['Pill hint','Reminder','Reduce feeds','Timed restriction'];
  return <section className="surface padded">
    <div className="panel-header"><div><h3>Gradual short-feed limits</h3><p>Choose how the app responds as observed short-form plays accumulate.</p></div></div>
    {error}
    {data.intervention.enabled&&<div className="release-row">
      <span>{data.intervention.plays??0} observed plays · {data.intervention.level?stages[data.intervention.level-1]:'Watching for your threshold'}</span>
      {data.intervention.level>0&&<button className="secondary-btn" disabled={panel.busy} onClick={()=>void panel.change(()=>agent('POST','/attention/snooze',{}))}>Snooze for 10 min</button>}
    </div>}
    <form className="release-form" onSubmit={e=>{e.preventDefault();void panel.change(()=>agent('POST','/attention/intervention',terms)).then(ok=>{if(ok)setTermsDraft(null);});}}>
      <label className="release-check"><input type="checkbox" checked={terms.enabled} onChange={e=>setTermsDraft({...terms,enabled:e.target.checked})}/> Enable progression</label>
      <label>Feed<CustomSelect value={terms.platform} onChange={e=>setTermsDraft({...terms,platform:e.target.value})} aria-label="Short feed">
        <option value="youtube_shorts">YouTube Shorts</option><option value="instagram_reels">Instagram Reels</option></CustomSelect></label>
      <label>Play window, minutes<input type="number" min={3} max={60} value={terms.window_minutes} onChange={e=>setTermsDraft({...terms,window_minutes:Number(e.target.value)})}/></label>
      {stages.map((stage,index)=><label key={stage}>{stage} plays<input type="number" min={1} max={100} value={terms.stages[index]}
        onChange={e=>setTermsDraft({...terms,stages:terms.stages.map((value,i)=>i===index?Number(e.target.value):value)})}/></label>)}
      <label>Restriction duration, minutes<input type="number" min={1} max={30} value={terms.block_minutes} onChange={e=>setTermsDraft({...terms,block_minutes:Number(e.target.value)})}/></label>
      <div className="release-actions"><button className="primary-btn" disabled={panel.busy}>Save progression</button></div>
    </form>
  </section>;
}

type Preset = {
  id?:string;revision?:number;name:string;profile_id:string|null;minutes:number;difficulty:string;
  block_domains:string[];native_apps:string[];minimize_selected_apps:boolean;
};
const initialPreset:Preset={name:'Coding sprint',profile_id:null,minutes:25,difficulty:'Gentle',block_domains:[],native_apps:[],minimize_selected_apps:false};

export function FocusPresets({disabled,onStart}:{disabled:boolean;onStart:()=>void}) {
  const panel=useAgentPanel<{presets:Preset[]}>('/focus/presets',disabled);
  const confirm=useConfirm();
  const [profiles,setProfiles]=useState<{id:string;name:string}[]>([]);
  const [preset,setPreset]=useState<Preset>(initialPreset);
  const [domains,setDomains]=useState('');
  const [apps,setApps]=useState('');
  useEffect(()=>{
    if(!disabled)void agent<{profiles:typeof profiles}>('GET','/workflows').then(result=>setProfiles(result.profiles)).catch(()=>{});
  },[disabled]);
  if(disabled)return null;
  const values=(text:string)=>text.split(',').map(value=>value.trim().toLowerCase()).filter(Boolean);
  function edit(item:Preset){setPreset(item);setDomains(item.block_domains.join(', '));setApps(item.native_apps.join(', '));}
  return <section id="focus-presets" className="surface padded">
    <div className="panel-header"><div><h3>Reusable focus presets</h3><p>Save the time, profile and boundaries you use often.</p></div></div>
    {panel.error&&<p role="alert">{panel.error}</p>}
    <div className="release-actions">
      <button className="text-action" onClick={()=>edit(initialPreset)}>Coding sprint</button>
      <button className="text-action" onClick={()=>edit({...initialPreset,name:'Study session',minutes:45})}>Study session</button>
      <button className="text-action" onClick={()=>edit({...initialPreset,name:'Client editing',minutes:50})}>Client editing</button>
      <button className="text-action" onClick={()=>edit({...initialPreset,name:'Night wind-down',minutes:20,difficulty:'Balanced',block_domains:['www.youtube.com','www.instagram.com']})}>Night wind-down</button>
    </div>
    <form className="release-form" onSubmit={e=>{
      e.preventDefault();void panel.change(()=>agent('POST','/focus/presets',{...preset,block_domains:values(domains),native_apps:values(apps)})).then(ok=>{if(ok)edit(initialPreset);});
    }}>
      <label>Preset name<input value={preset.name} onChange={e=>setPreset({...preset,name:e.target.value})} required maxLength={80}/></label>
      <label>Profile<CustomSelect value={preset.profile_id??''} onChange={e=>setPreset({...preset,profile_id:e.target.value||null})} aria-label="Preset profile">
        <option value="">General focus</option>{profiles.map(profile=><option key={profile.id} value={profile.id}>{profile.name}</option>)}
      </CustomSelect></label>
      <label>Minutes<input type="number" min={1} max={240} value={preset.minutes} onChange={e=>setPreset({...preset,minutes:Number(e.target.value)})}/></label>
      <label>Restriction level<CustomSelect value={preset.difficulty} onChange={e=>setPreset({...preset,difficulty:e.target.value,minimize_selected_apps:false})} aria-label="Preset restriction level">
        <option>Gentle</option><option>Balanced</option><option>Strict</option>
      </CustomSelect></label>
      <label>Browser targets<input value={domains} onChange={e=>setDomains(e.target.value)} placeholder="youtube.com, instagram.com"/></label>
      <label>Selected native apps<input value={apps} onChange={e=>setApps(e.target.value)} placeholder="my-game.exe"/></label>
      {preset.difficulty==='Strict'&&<label className="release-check"><input type="checkbox" checked={preset.minimize_selected_apps}
        onChange={e=>setPreset({...preset,minimize_selected_apps:e.target.checked})}/> Minimize each selected app once</label>}
      <div className="release-actions"><button className="primary-btn" disabled={panel.busy}>{preset.id?'Save changes':'Save preset'}</button></div>
    </form>
    {panel.data?.presets.map(item=><div className="release-row" key={item.id}>
      <div><strong>{item.name}</strong><p>{item.minutes} min · {item.difficulty} · {item.block_domains.join(', ')||'Timer'}</p>
        {item.native_apps.length>0&&<p>{item.minimize_selected_apps?'Minimize once':'Warn once'} · {item.native_apps.join(', ')}</p>}</div>
      <div className="release-actions">
        <button className="secondary-btn" disabled={panel.busy} onClick={async()=>{
          const apps=item.native_apps.length?` Selected apps: ${item.native_apps.join(', ')}. ${item.minimize_selected_apps?'Each can be minimized once.':'The app will show warnings.'}`:'';
          if(!await confirm({title:'Start '+item.name+'?',description:`${item.minutes} minutes of ${item.difficulty} focus. Browser targets: ${item.block_domains.join(', ')||'none'}.${apps}`,confirmLabel:'Start focus'}))return;
          if(await panel.change(()=>agent('POST','/focus/presets/start',{id:item.id,revision:item.revision})))onStart();
        }}>Start preset</button>
        <button className="text-action" onClick={()=>edit(item)}>Edit</button>
        <button className="text-action" disabled={panel.busy} onClick={()=>void panel.change(()=>agent('POST','/focus/presets',{...item,remove:true}))}>Remove</button>
      </div>
    </div>)}
  </section>;
}
