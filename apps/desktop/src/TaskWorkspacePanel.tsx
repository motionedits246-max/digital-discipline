import {useEffect,useState} from 'react';
import type {FormEvent} from 'react';
import {agent} from './api';
import {CustomSelect} from './CustomSelect';

type Profile={id:string;name:string;apps:string[];domains:string[]};
type Task={id:string;profile_id:string|null;title:string;outcome:string;state:string;updated_ms:number};
type Data={profiles:Profile[];tasks:Task[];may_be_truncated:boolean};
export function TaskWorkspacePanel({readonly}:{readonly:boolean}){
 const [data,setData]=useState<Data|null>(null),[error,setError]=useState(''),[busy,setBusy]=useState(false);
 const [editing,setEditing]=useState('');
 const [profileName,setProfileName]=useState('Coding'),[apps,setApps]=useState('code.exe, cursor.exe'),[domains,setDomains]=useState('github.com, docs.rs');
 const [title,setTitle]=useState('Finish today\'s focused work'),[profileId,setProfileId]=useState('');
 const [outcomes,setOutcomes]=useState<Record<string,string>>({});
 async function refresh(){try{const result=await agent<Data>('GET','/workflows');setData(result);setError('');}catch(e){setError(String(e));}}
 useEffect(()=>{if(!readonly)void refresh();},[readonly]);
 const split=(s:string)=>[...new Set(s.split(/[\s,;]+/).map(x=>x.trim().toLowerCase()).filter(Boolean))];
 async function submit<T>(action:()=>Promise<T>){
  if(busy||readonly)return;setBusy(true);setError('');
  try{await action();await refresh();}catch(e){setError(String(e));}
  finally{setBusy(false);}
 }
 function createProfile(e:FormEvent){e.preventDefault();void submit(()=>agent('POST',editing?'/workflows/profiles/'+editing:'/workflows/profiles',{name:profileName,apps:split(apps),domains:split(domains)}).then(()=>setEditing('')));}
 function createTask(e:FormEvent){e.preventDefault();void submit(()=>agent('POST','/workflows/tasks',{title,profile_id:profileId||null}));}
 function transition(id:string,state:string){void submit(()=>agent('POST','/workflows/tasks/'+id,{state,outcome:outcomes[id]||null}));}
 const actions:Record<string,string[]>={draft:['active','abandoned'],active:['paused','completed','abandoned'],paused:['active','completed','abandoned']};
 return <section className="advanced-stack" aria-label="Task-based focus workflow">
  <div className="panel-header"><div><h3>Work profiles and task outcomes</h3>
    <p>Choose what is relevant to a task. Completing it is always your own report, never inferred from elapsed time.</p></div>
    <span className="tag">Local</span></div>
  <div className="double-grid">
   <article className="surface padded advanced-card"><h3>{editing?'Edit focus profile':'Create a focus profile'}</h3>
    <form className="rule-form" onSubmit={createProfile}>
     <label>Profile name<input value={profileName} maxLength={80} onChange={e=>setProfileName(e.target.value)} required/></label>
     <label>Related app executables<textarea value={apps} onChange={e=>setApps(e.target.value)} rows={2}/></label>
     <label>Related public domains<textarea value={domains} onChange={e=>setDomains(e.target.value)} rows={2}/></label>
     <p className="form-hint">Targets describe relevance for this profile only. They do not rewrite global categories or automatically add restrictions.</p>
     <button className="primary-btn" disabled={busy||readonly}>{editing?'Save profile':'Create profile'}</button>{editing&&<button className="secondary-btn" type="button" onClick={()=>setEditing('')}>Cancel edit</button>}
    </form>
   </article>
   <article className="surface padded advanced-card"><h3>Add a task</h3>
    <form className="rule-form" onSubmit={createTask}>
     <label>Intended outcome<input value={title} maxLength={160} onChange={e=>setTitle(e.target.value)} required/></label>
     <label>Related profile<CustomSelect value={profileId} onChange={e=>setProfileId(e.target.value)}>
      <option value="">None</option>{(data?.profiles||[]).map(p=><option value={p.id} key={p.id}>{p.name}</option>)}
     </CustomSelect></label>
     <p className="form-hint">Task state is self-reported and separate from measured activity or focus minutes.</p>
     <button className="primary-btn" disabled={busy||readonly}>Create task</button>
    </form>
   </article>
  </div>
  <article className="surface padded advanced-card"><div className="panel-header"><h3>Tasks</h3><button className="secondary-btn" type="button" onClick={()=>void refresh()}>Refresh</button></div>
   {(data?.tasks||[]).length?data!.tasks.map(t=><div key={t.id} className="settings-row">
    <div><strong>{t.title}</strong><small>{data?.profiles.find(p=>p.id===t.profile_id)?.name||'Unassigned'} · {t.state} · Self-reported outcome</small>
     {!['completed','abandoned'].includes(t.state)&&<input aria-label={'Optional outcome for '+t.title} placeholder="Optional result or reason" maxLength={1000} value={outcomes[t.id]||''} onChange={e=>setOutcomes(p=>({...p,[t.id]:e.target.value}))}/>}
     {t.outcome&&<small>Outcome: {t.outcome}</small>}
    </div><div className="coach-vault-actions">{(actions[t.state]||[]).map(next=><button className="secondary-btn" type="button" key={next} disabled={readonly||busy} onClick={()=>transition(t.id,next)}>{next}</button>)}</div>
   </div>):<p className="muted-paragraph">No tasks yet. Add one above; time spent is observed independently.</p>}
   {data?.may_be_truncated&&<p className="form-hint">Showing a limited recent list; some older items are omitted.</p>}
  </article>
  <article className="surface padded advanced-card"><div className="panel-header"><h3>Saved profiles</h3></div>
   {(data?.profiles||[]).map(p=><div className="settings-row" key={p.id}><div><strong>{p.name}</strong>
    <small>{p.apps.join(', ')||'No app targets'} · {p.domains.join(', ')||'No domain targets'}</small></div>
    <div className="coach-vault-actions"><button type="button" className="secondary-btn" disabled={readonly||busy} onClick={()=>{setEditing(p.id);setProfileName(p.name);setApps(p.apps.join(', '));setDomains(p.domains.join(', '));}}>Edit profile</button><button type="button" className="secondary-btn" disabled={readonly||busy} onClick={()=>void submit(()=>agent('DELETE','/workflows/profiles/'+p.id))}>Archive</button></div>
   </div>)}
  </article>
  {error&&<p role="alert" className="advanced-feedback">{error}</p>}
 </section>;
}
