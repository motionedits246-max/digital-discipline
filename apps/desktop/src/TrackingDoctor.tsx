import {useEffect,useState} from 'react';
import {agent} from './api';
type Profile={extension_id:string;profile_id:string;browser:string;online:boolean;authorized:boolean;recording_enabled:boolean|null;queued:number;adapter:string;qualified_at_ms:number;accepted_at_ms:number;rendered_ms:number;policy_ack_ms:number;companion_version:string;loss_count:number;last_seen_ms:number;dashboard_received:boolean;policy_acknowledged:boolean;accepted_events:number};
type Report={profiles:Profile[];checked_at_ms:number;play_threshold_ms:number;native_attribution:string;agent_version?:string;expected_companion_version?:string};
const oldCompanion=(actual:string,required:string)=>{if(!/^\d+\.\d+\.\d+$/.test(actual))return true;const a=actual.split('.').map(Number),r=required.split('.').map(Number);for(let i=0;i<3;i++){if(a[i]!==r[i])return a[i]<r[i];}return false;};
const at=(n:number)=>n?new Date(n).toLocaleTimeString():'Not confirmed';
export function TrackingDoctor({readonly}:{readonly:boolean}){
 const [report,setReport]=useState<Report|null>(null),[error,setError]=useState(''),[busy,setBusy]=useState(false);
 async function refresh(){if(readonly||busy)return;setBusy(true);try{setReport(await agent<Report>('GET','/tracking/doctor'));setError('');}catch{setError('Tracking Doctor could not reach the local agent. Last verified values remain below.');}finally{setBusy(false);}}
 useEffect(()=>{if(readonly)return;let disposed=false,running=false;
  const read=async()=>{if(running)return;running=true;try{const r=await agent<Report>('GET','/tracking/doctor');if(!disposed){setReport(r);setError('');}}catch{if(!disposed)setError('Tracking Doctor is offline. These are the last verified values.');}finally{running=false;}};
  void read();const timer=setInterval(()=>void read(),5000);return()=>{disposed=true;clearInterval(timer);};
 },[readonly]);
 async function access(p:Profile){setBusy(true);try{await agent('POST','/tracking/profiles/access',{extension_id:p.extension_id,profile_id:p.profile_id,authorized:!p.authorized});setReport(await agent<Report>('GET','/tracking/doctor'));setError('');}catch{setError('Could not change this profile connection.');}finally{setBusy(false);}}
 return <article className="surface padded advanced-card tracking-doctor" aria-label="Tracking Doctor">
  <div className="panel-header"><div><h3>Tracking Doctor</h3><p>Follow an observation from this browser profile to the dashboard.</p></div><button type="button" className="secondary-btn" disabled={readonly||busy} onClick={()=>void refresh()}>Check now</button></div>
  {report?.expected_companion_version&&<p className="form-hint">Agent {report.agent_version} · current companion {report.expected_companion_version} · protocol 1.0. Upgrade the companion in place to retain pairing and pending events.</p>}
  {readonly?<p className="form-hint">Live checks require the local Windows agent.</p>:!report?<p className="form-hint">{error||'Checking authorized browser profiles…'}</p>:null}
  {error&&report&&<p role="status" className="advanced-feedback">{error}</p>}
  {report?.profiles.map(p=>{const steps=[['Authorized',p.authorized],['Online',p.online],['Recording',p.recording_enabled===true],['Player ready',p.adapter==='ready'],['Play qualified',p.qualified_at_ms>0],['Agent accepted',p.accepted_at_ms>0],['Dashboard received',p.dashboard_received]] as const;
   const failed=steps.find(x=>!x[1])?.[0];
   const advice=!p.authorized?'Reconnect this profile below. Its pending observations remain in the companion.':!p.online?'Open this browser profile and use Connect & sync in its companion.':p.adapter==='private_session'?'A private session is active. Recording resumes at its declared expiry; browser restrictions continue.':report.expected_companion_version&&oldCompanion(p.companion_version,report.expected_companion_version)?'Update the existing companion folder in place and reload it to preserve its identity and queue.':p.recording_enabled===false?'Recording is off. Restrictions can still sync; enable browser recording only if you want observations.':p.recording_enabled===null?'This companion has not reported a recording preference. Update its existing folder in place and reload.':p.adapter!=='ready'?'Refresh an allowed YouTube or Instagram tab and open a supported video.':!p.qualified_at_ms?'Play a supported video visibly for at least two seconds; route visits do not qualify.':!p.accepted_at_ms?'Check the companion’s pending queue and retry status, then sync.':!p.dashboard_received?'Open Overview or Video Intelligence to render the latest accepted count.':'All observation stages have a confirmed signal. Check timestamps when testing a new play.';
   return <section className="doctor-profile" key={p.extension_id+':'+p.profile_id}><div className="panel-header"><div><strong>{p.browser.replace(/^./,x=>x.toUpperCase())} profile · {p.profile_id.slice(0,8)}</strong><small>Companion {p.companion_version||'version not reported'} · last connection {at(p.last_seen_ms)}</small></div><span className={'tag '+(p.online&&p.authorized?'tag-on':'')}>{!p.authorized?'Disconnected':p.online?'Online':'Offline'}</span></div>
    <ol className="doctor-stages">{steps.map(([name,ok])=><li className={ok?'confirmed':'pending'} key={name}><span className="doctor-dot"/>{name}<small>{ok?'Confirmed':name==='Recording'&&p.adapter==='private_session'?'Private session':name==='Recording'&&p.recording_enabled===false?'Disabled':'Not confirmed'}</small></li>)}</ol>
    <p className="form-hint"><strong>{failed?'Next stage: '+failed:'Observation path confirmed'}</strong> · {advice}</p>
    <div className="doctor-details"><span>Pending events <strong>{p.queued}</strong></span><span>Last qualified <strong>{at(p.qualified_at_ms)}</strong></span><span>Last accepted <strong>{at(p.accepted_at_ms)}</strong></span><span>Policy response <strong>{at(p.policy_ack_ms)}</strong></span></div>
    {p.loss_count>0&&<p className="advanced-feedback">{p.loss_count} events were rejected, expired or evicted in this profile. Review Delivery &amp; recovery in the companion.</p>}
    <button className="secondary-btn" type="button" disabled={readonly||busy} onClick={()=>void access(p)}>{p.authorized?'Disconnect this profile':'Reconnect this profile'}</button>
   </section>;
  })}
  {report&&!report.profiles.length&&<p className="muted-paragraph">No profile handshake yet. Complete browser setup above, then open the companion and sync. Existing extension authorization is retained independently of its online state.</p>}
  <p className="form-hint">{report?.native_attribution||'An authorized browser may be offline or have recording disabled.'} Imported device history is separate from local foreground observation.</p>
 </article>;
}
