import {FileDropzone} from './FileDropzone';
import {CustomSelect} from './CustomSelect';
import {sealBundle,openBundle} from "./secureBundle.mjs";
import {useEffect,useMemo,useRef,useState} from 'react';
import type {FormEvent,KeyboardEvent,ReactNode} from 'react';
import {agent,autostartStatus,setAutostart,isTauri} from './api';
import {BrowserSetupWizard} from './BrowserSetupWizard';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import type {Focus,Goal,Insights,PrivacySettings,Rule,Session,Video} from './types';

const cats=['Development','Design','Productivity','Communication','Education','Entertainment','Gaming','Social','News','Utility','System','Browser','Unclassified'];
const pretty=(n:number)=>`${Math.floor(Math.max(0,n)/3600000)}h ${String(Math.floor(Math.max(0,n)%3600000/60000)).padStart(2,'0')}m`;
const split=(str:string)=>[...new Set(str.split(/[\s,;]+/g).map(s=>s.trim().toLowerCase()).filter(Boolean))];

function Card({title,children}:{title:string;children:ReactNode}) {return <article className="surface padded advanced-card"><h3>{title}</h3>{children}</article>;}
function Feedback({text}:{text:string}) {return text?<p className="advanced-feedback" role="status">{text}</p>:null;}
function useRemote<T>(path:string,active:boolean,refreshKey:number):{data:T|null;error:string;refresh:()=>void} {
 const [data,setData]=useState<T|null>(null);const [error,setError]=useState('');const [refreshNo,setRefreshNo]=useState(0);
 useEffect(()=>{if(!active){setData(null);return;}let keep=true;agent<T>('GET',path).then(v=>{if(keep){setData(v);setError('');}}).catch(e=>{if(keep){setError(String(e));setData(null);}});return()=>{keep=false;};},[path,active,refreshKey,refreshNo]);
 return {data,error,refresh:()=>setRefreshNo(n=>n+1)};
}
function useAction(onUpdate:()=>void){
 const [message,setMessage]=useState('');const [busy,setBusy]=useState(false);
 async function perform(fn:()=>Promise<unknown>,success:string){setBusy(true);setMessage('');try{await fn();setMessage(success);onUpdate();}catch(e){setMessage(String(e));}finally{setBusy(false);}}
 return {message,busy,perform};
}

export function GoalsPanel({goals,focus,readonly,onUpdate}:{goals:Goal[];focus:Focus|null|undefined;readonly:boolean;onUpdate:()=>void}){
 const {message,busy,perform}=useAction(onUpdate);
 const [name,setName]=useState('Deep work');const [minutes,setMinutes]=useState(90);const [apps,setApps]=useState('code.exe, cursor.exe, antigravity.exe');const [domains,setDomains]=useState('docs.rs, github.com');
 const [goalId,setGoalId]=useState('');const [focusMinutes,setFocusMinutes]=useState(45);const [difficulty,setDifficulty]=useState('Balanced');const [blocked,setBlocked]=useState('www.youtube.com, www.instagram.com');
 const active=focus&&focus.end_ms>Date.now();
 function create(e:FormEvent){e.preventDefault();void perform(()=>agent('POST','/goals',{name,target_minutes:minutes,apps:split(apps),domains:split(domains)}),'Goal created. Activity is matched locally without AI.');}
 function start(e:FormEvent){e.preventDefault();void perform(()=>agent('POST','/focus/start',{goal_id:goalId||null,minutes:focusMinutes,difficulty,block_domains:split(blocked)}),'Focus session started. Extension restrictions sync within one minute.');}
 return <div className="advanced-stack"><div className="double-grid"><Card title="Create a measurable goal"><form onSubmit={create} className="rule-form"><label>Goal title<input required maxLength={80} value={name} onChange={e=>setName(e.target.value)}/></label><label>Target minutes per day<input type="number" required min={1} max={1440} value={minutes} onChange={e=>setMinutes(+e.target.value)}/></label><label>Qualifying applications (comma-separated)<textarea value={apps} onChange={e=>setApps(e.target.value)} rows={2}/></label><label>Qualifying documentation sites (comma-separated)<textarea value={domains} onChange={e=>setDomains(e.target.value)} rows={2}/></label><p className="form-hint">Every editor can qualify using its executable name, including new IDEs. No window titles or document names are needed.</p><button disabled={readonly||busy} className="primary-btn">Save goal</button></form></Card>
 <Card title="Focus commitment">{active?<><span className="tag tag-on">FOCUS ACTIVE</span><h2>{focus?.difficulty} focus until {new Date(focus!.end_ms).toLocaleTimeString()}</h2><p>Browser blocks are reversible. You always have the extension's emergency recovery option.</p><button disabled={readonly||busy} className="secondary-btn" onClick={()=>void perform(()=>agent('POST','/focus/end',{}),'Focus session ended.')}>Finish focus</button></>:<form className="rule-form" onSubmit={start}><label>Attached goal<CustomSelect value={goalId} onChange={e=>setGoalId(e.target.value)}><option value="">No attached goal</option>{goals.filter(g=>g.enabled).map(g=><option key={g.id} value={g.id}>{g.name}</option>)}</CustomSelect></label><label>Duration, minutes<input type="number" min={1} max={240} value={focusMinutes} onChange={e=>setFocusMinutes(+e.target.value)}/></label><label>Focus difficulty<CustomSelect value={difficulty} onChange={e=>setDifficulty(e.target.value)}><option>Gentle</option><option>Balanced</option><option>Strict</option></CustomSelect></label><label>Temporarily block domains (optional)<textarea rows={2} value={blocked} onChange={e=>setBlocked(e.target.value)}/></label><p className="form-hint">Gentle: timer only. Balanced: hides selected site feeds, or sends opt-in notifications elsewhere. Strict: temporary browser navigation blocks with emergency recovery; never tamper-proof.</p><button className="primary-btn" disabled={readonly||busy}>Start focus</button></form>}</Card></div>
 <Card title="Saved goals"><div className="advanced-list">{goals.filter(x=>x.enabled).length?goals.filter(x=>x.enabled).map(g=><div key={g.id} className="settings-row"><div><strong>{g.name}</strong><small>{pretty(g.observed_ms_today||0)} observed / {g.target_minutes}m target · {g.apps.length} apps · {g.domains.length} domains</small><progress max={Math.max(1,g.target_minutes*60000)} value={Math.min(g.observed_ms_today||0,g.target_minutes*60000)} aria-label={`${g.name} progress`} /></div><button className="secondary-btn" disabled={readonly||busy} onClick={()=>void perform(()=>agent('DELETE',`/goals/${g.id}`),'Goal archived.')}>Archive</button></div>):<p>No goals yet. Create one to plan your work.</p>}</div></Card><Feedback text={message}/></div>;
}

export function AutomationsPanel({readonly,rules,onUpdate}:{readonly:boolean;rules:Rule[];onUpdate:()=>void}){
 const {message,busy,perform}=useAction(onUpdate);
 const [name,setName]=useState('Shorts daily cap');const [domain,setDomain]=useState('www.youtube.com');const [path,setPath]=useState('/shorts/');
 const [trigger,setTrigger]=useState('plays');const [threshold,setThreshold]=useState(15);const [group,setGroup]=useState('www.instagram.com');const [replays,setReplays]=useState(true);
 const [start,setStart]=useState('');const [end,setEnd]=useState('');const [days,setDays]=useState<number[]>([]);const [action,setAction]=useState('block');
 const [duration,setDuration]=useState(30);
 const [groupApps,setGroupApps]=useState('');const [groupCats,setGroupCats]=useState<string[]>([]);
 const [excludeDomains,setExcludeDomains]=useState('');const [excludeApps,setExcludeApps]=useState('');const [gapSeconds,setGapSeconds]=useState(120);
 const [preview,setPreview]=useState<{would_activate_now:boolean;schedule_active:boolean;observed_eligible_minutes_today:number;qualified_plays_today:number;rolling_streak:number;signal:string}|null>(null);
 const [previewError,setPreviewError]=useState('');
 const {data:audit,refresh:refreshAudit}=useRemote<{events:{detail:{rule_name:string;triggered:boolean};occurred_ms:number}[]}>('/rules/audit',!readonly,0);
 function payload(){const config={action,group_domains:trigger==='shared'?split(group):[],group_apps:trigger==='shared'?split(groupApps):[],
   group_categories:trigger==='shared'?groupCats:[],excluded_domains:split(excludeDomains),excluded_apps:split(excludeApps),
   streak_gap_seconds:trigger==='streak'?gapSeconds:null,qualified_plays:trigger==='plays'?threshold:null,
   streak_plays:trigger==='streak'?threshold:null,streak_window_minutes:trigger==='streak'?15:null,
   include_replays:replays,weekdays:days,start_time:start||null,end_time:end||null};
   return {name,domain:domain.trim().toLowerCase(),path_prefix:path.trim()||null,
   minutes:trigger==='timed'?duration:null,budget_minutes:['minutes','shared'].includes(trigger)?threshold:null,config};}
 function submit(e:FormEvent){e.preventDefault();void perform(()=>agent('POST','/rules',payload()),'Automation armed. Policy evaluation continues in the background agent.');}
 async function previewRule(){setPreviewError('');try{setPreview(await agent('POST','/rules/preview',payload()));}catch(e){setPreviewError(String(e));}}
 return <div className="advanced-stack"><div className="automation-flow"><div className="flow-step"><span>IF</span><strong>{trigger==='streak'?'Doomscroll streak in 15m':trigger==='plays'?'Qualifying short-form count':trigger==='minutes'?'Daily usage budget':trigger==='shared'?'Shared app/site/category budget':trigger==='timed'?'Time period':'Manual condition'}</strong><small>{threshold} {trigger==='plays'?'plays':'minutes'}</small></div><span>→</span><div className="flow-step"><span>AND</span><strong>{start&&end?`${start}–${end}`:'Any time'}</strong><small>{days.length?`${days.length} weekdays selected`:'All weekdays'}</small></div><span>→</span><div className="flow-step"><span>THEN</span><strong>{action==='block'?'Block route':action==='hide_feed'?'Hide feed':'Disable autoplay'}</strong><small>Paired browser only · reversible</small></div></div>
 <div className="double-grid"><Card title="IF / AND / THEN builder"><form className="rule-form" onSubmit={submit}><div className="form-grid"><label>Policy name<input required value={name} maxLength={80} onChange={e=>setName(e.target.value)}/></label><label>Target domain<input required value={domain} onChange={e=>setDomain(e.target.value)}/></label><label>Optional path<input value={path} onChange={e=>setPath(e.target.value)}/></label><label>IF condition<CustomSelect value={trigger} onChange={e=>setTrigger(e.target.value)}><option value="plays">Qualified plays today</option><option value="streak">Doomscroll streak in 15 minutes</option><option value="minutes">Active minutes today</option><option value="shared">Shared minutes across apps/sites/categories</option><option value="timed">Temporary block</option><option value="manual">Indefinite manual block</option></CustomSelect></label>{trigger!=='manual'&&<label>{trigger==='timed'?'Block duration (min)':'Threshold'}<input type="number" min={1} max={['plays','streak'].includes(trigger)?10000:1440} value={trigger==='timed'?duration:threshold} onChange={e=>trigger==='timed'?setDuration(+e.target.value):setThreshold(+e.target.value)}/></label>}
 <label>THEN action<CustomSelect value={action} onChange={e=>setAction(e.target.value)}><option value="block">Block navigation</option><option value="hide_feed">Hide feed (YouTube/Instagram)</option><option value="stop_autoplay">Stop autoplay (YouTube/Instagram)</option><option value="notify">Notification only (opt-in)</option></CustomSelect></label></div>
 {trigger==='shared'&&<><label>Additional domains in shared budget<textarea value={group} rows={2} onChange={e=>setGroup(e.target.value)}/></label><label>Native apps in shared budget<textarea value={groupApps} rows={2} onChange={e=>setGroupApps(e.target.value)} placeholder="cursor.exe, code.exe"/></label><fieldset className="category-choices"><legend>Also match categories</legend>{cats.map(c=><label key={c} className="check-line"><input type="checkbox" checked={groupCats.includes(c)} onChange={e=>setGroupCats(v=>e.target.checked?[...v,c]:v.filter(x=>x!==c))}/>{c}</label>)}</fieldset></>}
 {trigger==='streak'&&<label>Maximum gap between qualified plays (seconds)<input type="number" min={10} max={600} value={gapSeconds} onChange={e=>setGapSeconds(+e.target.value)}/></label>}
 {trigger==='plays'&&<label className="check-line"><input type="checkbox" checked={replays} onChange={e=>setReplays(e.target.checked)}/> Count replay instances (otherwise unique video IDs)</label>}
 <div className="form-grid"><label>Exclude domains from usage accounting<textarea rows={2} value={excludeDomains} onChange={e=>setExcludeDomains(e.target.value)} placeholder="bank.example.com"/></label><label>Exclude apps from usage accounting<textarea rows={2} value={excludeApps} onChange={e=>setExcludeApps(e.target.value)} placeholder="private.exe"/></label></div>
 <div className="form-grid"><label>Scheduled start (optional)<input type="time" value={start} onChange={e=>setStart(e.target.value)}/></label><label>Scheduled end (optional)<input type="time" value={end} onChange={e=>setEnd(e.target.value)}/></label></div><label>Days for schedule</label><div className="day-buttons">{['Mon','Tue','Wed','Thu','Fri','Sat','Sun'].map((d,i)=><button key={d} type="button" className={days.includes(i+1)?'day-on':''} aria-pressed={days.includes(i+1)} onClick={()=>setDays(xs=>xs.includes(i+1)?xs.filter(x=>x!==i+1):[...xs,i+1])}>{d}</button>)}</div>
 <p className="form-hint">Schedules use your Windows local clock (including overnight ranges). Thresholds reset at local midnight. Protection requires the companion extension. Blocks use short, renewed leases to prevent stale lockouts after a crash.</p><div className="device-actions"><button type="button" className="secondary-btn" disabled={readonly||busy||Boolean(start)!==Boolean(end)} onClick={()=>void previewRule()}>Preview without saving</button><button className="primary-btn" disabled={readonly||busy||Boolean(start)!==Boolean(end)}>Save automation</button></div></form></Card>
 <Card title="Policy preview and current state"><h4>Target: {domain}{path}</h4>{preview&&<div className="preview-brief"><strong>{preview.would_activate_now?'Would activate now':'Would remain armed'}</strong><p>{preview.schedule_active?'Inside scheduled window':'Outside scheduled window'}</p><p>Observed eligible time today: {preview.observed_eligible_minutes_today.toFixed(1)}m · qualifying plays: {preview.qualified_plays_today} · rolling streak: {preview.rolling_streak}</p><small>{preview.signal}</small></div>}<Feedback text={previewError}/><p className="muted-paragraph">An automation evaluates local observed time and videos. Browser navigation isn't closed in advance, and manual emergency recovery remains possible.</p><div className="advanced-list">{rules.map(r=><div key={r.id} className="settings-row"><div><strong>{r.name}</strong><small>{r.domain}{r.path_prefix} · {r.config?.action||'block'}{r.config?.qualified_plays?` · ${r.config.qualified_plays} plays`:''}</small></div><span className={`tag ${r.enforced?'tag-on':''}`}>{!r.enabled?'Disabled':r.enforced?'Triggered':'Armed'}</span></div>)}</div><div className="small-divider"/><div className="panel-header"><h4>Trigger / clear audit history</h4><button className="secondary-btn" type="button" onClick={refreshAudit}>Refresh trace</button></div><div className="advanced-list">{(audit?.events||[]).slice(0,25).map((ev,i)=><div key={i} className="settings-row"><span>{ev.detail?.rule_name||'Rule'} · {ev.detail?.triggered?'Activated':'Cleared'}</span><small>{new Date(ev.occurred_ms).toLocaleString()}</small></div>)}</div></Card></div><Feedback text={message}/></div>;
}

export function VideoHistoryPanel({active,readonly,refreshKey}:{active:boolean;readonly:boolean;refreshKey:number}) {
 const {data,error}=useRemote<{videos:Video[];visibility_disabled?:boolean}>('/videos',active&&!readonly,refreshKey);
 const [search,setSearch]=useState('');const filtered=(data?.videos||[]).filter(v=>(v.video_id+' '+v.platform).toLowerCase().includes(search.toLowerCase()));
 return <Card title="Observed video history · optional metadata"><label>Filter by platform or video ID<input value={search} maxLength={128} onChange={e=>setSearch(e.target.value)} placeholder="Search locally; nothing is sent online"/></label>{error&&<p>{error}</p>}{data?.visibility_disabled?<p>Video gallery is hidden by your privacy setting; existing local counts remain available.</p>:data?.videos.length?<div className="video-grid">{filtered.slice(0,100).map(v=><div className="video-mini" key={v.platform+':'+v.video_id}><small>{v.platform==='youtube_shorts'?'YOUTUBE SHORTS':v.platform==='youtube_watch'?'YOUTUBE VIDEOS':'INSTAGRAM REELS'}</small><strong>{v.video_id}</strong><span>{v.qualified_plays} qualifying plays · {pretty(v.watched_ms)}</span><small>{new Date(v.last_seen_ms).toLocaleString()}</small></div>)}</div>:<p>No observed video history yet. Enable browser collection in the extension to record qualifying playback. Titles and transcripts are not stored.</p>}</Card>;
}

export function InsightsPanel({active,readonly,refreshKey}:{active:boolean;readonly:boolean;refreshKey:number}) {
 const {data,error}=useRemote<Insights>('/insights',active&&!readonly,refreshKey);const d=data;
 const bars=useMemo(()=>d?.hourly_distribution_ms||[],[d]);const max=Math.max(1,...bars);
 return <div className="advanced-stack">{error&&<Feedback text={error}/>}<div className="metric-grid"><div className="metric"><span>Week observed</span><strong>{pretty(d?.week.current_ms||0)}</strong><small>Incomplete coverage caveat</small></div><div className="metric"><span>Previous week</span><strong>{pretty(d?.week.previous_ms||0)}</strong><small>Compare cautiously</small></div><div className="metric"><span>Attention switches</span><strong>{d?.attention_switches_this_week??'—'}</strong><small>Foreground app changes this week</small></div><div className="metric"><span>Longest short-form streak</span><strong>{d?.shorts_longest_streak_90d??'—'}</strong><small>Qualified plays with ≤2m gaps</small></div></div>
 <div className="double-grid"><Card title="Hour-of-day activity · past 90 days"><div className="hour-chart" aria-label="Hourly observed activity">{bars.map((n,i)=><div key={i} className="hour-col" title={`${i}:00 · ${pretty(n)}`}><div style={{height:`${Math.max(2,n/max*100)}%`}}/><small>{i%4===0?i:''}</small></div>)}</div><p className="form-hint">Observed foreground intervals are split at local hour boundaries. Missing sessions are not reconstructed.</p></Card><Card title="Browser and category comparison"><h4>Browsers · measured active time</h4>{Object.entries(d?.browser_comparison||{}).map(([name,v])=><div key={name} className="settings-row"><span>{name}</span><strong>{pretty(v)}</strong></div>)}<h4>Categories · all observed time</h4>{Object.entries(d?.categories||{}).sort((a,b)=>b[1]-a[1]).slice(0,8).map(([name,v])=><div className="settings-row" key={name}><span>{name}</span><strong>{pretty(v)}</strong></div>)}</Card></div><Card title="Opt-in, evidence-gated hour-of-day forecast">{d?.prediction.available?<><h2>{Math.round((d.prediction.probability||0)*100)}% historical likelihood</h2><p className="muted-paragraph">For {d.prediction.hour_local}:00 in your local timezone · {d.prediction.confidence} confidence · 95% interval {Math.round((d.prediction.interval_low||0)*100)}–{Math.round((d.prediction.interval_high||0)*100)}%.</p><p className="form-hint">{d.prediction.signal} Based on {d.prediction.hour_samples} observed hours across {d.prediction.observed_days} days. This is uncertain, not a statement about future behavior or objectively wasted time.</p></>:<p className="muted-paragraph">{d?.prediction.reason||'Requires opt-in and enough verified observed history.'}</p>}</Card></div>;
}

export function CategoryPanel({sessions,readonly,onUpdate}:{sessions:Session[];readonly:boolean;onUpdate:()=>void}) {
 const {message,busy,perform}=useAction(onUpdate);const [choices,setChoices]=useState<Record<string,string>>({});
 const known=[...new Map(sessions.map(s=>[(s.domain?'domain:'+s.domain:'app:'+s.app),s])).entries()].slice(0,30);
 return <Card title="App/domain categorization · manual overrides"><p className="muted-paragraph">Unknown applications remain counted and unclassified until you assign one. User decisions always override curated app guesses and are applied to historical totals.</p><div className="advanced-list">{known.map(([key,s])=><div className="settings-row" key={key}><div><strong>{key.slice(key.indexOf(':')+1)}</strong><small>{s.category} · {key.startsWith('domain:')?'Website':'Application'}</small></div><CustomSelect aria-label={'Category for '+key} value={choices[key]||s.category} onChange={e=>setChoices(v=>({...v,[key]:e.target.value}))}>{cats.map(c=><option key={c}>{c}</option>)}</CustomSelect><button className="secondary-btn" disabled={readonly||busy} onClick={()=>void perform(()=>agent('POST','/categories/override',{subject:key,category:choices[key]||s.category}),'Override saved; previous sessions will recalculate.')}>Save</button></div>)}</div><Feedback text={message}/></Card>;
}

export function ClassificationsPanel({readonly,onUpdate}:{readonly:boolean;onUpdate:()=>void}) {
 const {data,refresh,error}=useRemote<{jobs:{subject:string;state:string;category:string;reason:string;updated_ms:number}[]}>('/classifications',!readonly,0);
 const {message,busy,perform}=useAction(()=>{refresh();onUpdate();});
 const [subject,setSubject]=useState('');const [provider,setProvider]=useState('openai');const [model,setModel]=useState('gpt-4.1-mini');const [key,setKey]=useState('');const [consent,setConsent]=useState(false);
 useEffect(()=>{if(readonly)return;const t=setInterval(refresh,15000);return()=>clearInterval(t);},[readonly]);
 function submit(e:FormEvent){e.preventDefault();const secret=key;setKey('');void perform(()=>agent('POST','/classifications/research',{subject,provider,model,api_key:secret,explicit_consent:consent}),'Classification queued outside the tracking loop. Check status below.');}
 const jobs=data?.jobs||[];
 return <Card title="Nonblocking app classification · optional BYOK research"><p className="muted-paragraph">Unknown app identities are queued locally every minute. No external service is contacted automatically. You can ask a selected model to classify a single known executable name, or manually override its category above.</p>
 <form className="rule-form" onSubmit={submit}><label>Unknown observed app<CustomSelect value={subject} onChange={e=>setSubject(e.target.value)} required><option value="">Select observed app</option>{jobs.filter(j=>j.state!=='succeeded').map(j=><option key={j.subject} value={j.subject}>{j.subject} · {j.state}</option>)}</CustomSelect></label><div className="form-grid"><label>Provider<CustomSelect value={provider} onChange={e=>{setProvider(e.target.value);setModel(e.target.value==='ollama'?'llama3.2':e.target.value==='anthropic'?'claude-sonnet-4-5':e.target.value==='gemini'?'gemini-2.5-flash':e.target.value==='openrouter'?'openai/gpt-4.1-mini':'gpt-4.1-mini');}}>{['openai','anthropic','gemini','groq','openrouter','ollama'].map(x=><option key={x}>{x}</option>)}</CustomSelect></label><label>Model ID<input required value={model} onChange={e=>setModel(e.target.value)}/></label></div>{provider!=='ollama'&&<label>One-use API key<input type="password" autoComplete="off" value={key} onChange={e=>setKey(e.target.value)} required minLength={8}/></label>}<label className="check-line"><input type="checkbox" checked={consent} onChange={e=>setConsent(e.target.checked)}/> Send only this app executable basename to the provider</label><button disabled={readonly||busy||!subject||!consent} className="primary-btn">Classify asynchronously</button></form><Feedback text={message||error}/>
 <div className="advanced-list">{jobs.map(j=><div key={j.subject} className="settings-row"><div><strong>{j.subject.slice(4)}</strong><small>{j.state} · {j.category} · {j.reason}</small></div><span className="tag">{j.state}</span></div>)}</div><p className="form-hint">Model output is strict-schema validated; name-only confidence is capped, low-confidence answers need review, and AI output cannot issue blocking commands. This provider key is discarded after one call.</p></Card>;
}

type CoachMessage={role:'user'|'assistant';content:string;truncated?:boolean;error?:boolean};

export function CoachPanel({readonly}:{readonly:boolean}) {
 const [provider,setProvider]=useState('openai'),[model,setModel]=useState('gpt-4.1-mini');
 const [key,setKey]=useState(''),[question,setQuestion]=useState('');
 const [consent,setConsent]=useState(false),[messages,setMessages]=useState<CoachMessage[]>([]);
 const [busy,setBusy]=useState(false),[status,setStatus]=useState(''),[shared,setShared]=useState('');
 const [savedProviders,setSavedProviders]=useState<string[]>([]),[credentialMode,setCredentialMode]=useState<'saved'|'one-time'>('one-time');
 const [saving,setSaving]=useState(false),[vaultLoading,setVaultLoading]=useState(true);
 const endOfChat=useRef<HTMLDivElement|null>(null),textareaRef=useRef<HTMLTextAreaElement|null>(null);
 const modelDefaults:Record<string,string>={openai:'gpt-4.1-mini',anthropic:'claude-sonnet-4-5',gemini:'gemini-2.5-flash',groq:'llama-3.3-70b-versatile',openrouter:'openai/gpt-4.1-mini',ollama:'llama3.2'};
 const hasSaved=savedProviders.includes(provider),useSaved=provider!=='ollama'&&hasSaved&&credentialMode==='saved';
 useEffect(()=>{if(readonly){setVaultLoading(false);return;}let active=true;
   void agent<{providers:string[]}>('GET','/coach/credentials').then(r=>{
     if(!active)return;
     const providers=r.providers.filter(p=>p in modelDefaults);
     setSavedProviders(providers);
     if(providers.length){
       const chosen=providers.includes(provider)?provider:providers[0];
       setProvider(chosen);setModel(modelDefaults[chosen]);setCredentialMode('saved');
     }else setCredentialMode('one-time');
   }).catch(e=>{if(active)setStatus('Could not load encrypted keys: '+String(e));}).finally(()=>{if(active)setVaultLoading(false);});
   return()=>{active=false;};
 // Initial vault lookup only. Changing provider is handled explicitly by chooseProvider.
 // eslint-disable-next-line react-hooks/exhaustive-deps
 },[readonly]);
 useEffect(()=>{endOfChat.current?.scrollIntoView({behavior:'smooth',block:'end'});},[messages,busy]);
 const prompts=[
  {label:'📊 Find focus patterns',value:'Based on what is actually observed today, what focus patterns stand out?'},
  {label:'🎯 Make a focus plan',value:'Build a realistic 30-minute focus plan from my activity data.'},
  {label:'💡 Review my day',value:'What can I learn from my activity today, and what can’t this data tell me?'}
 ];
 function chooseProvider(p:string){setProvider(p);setModel(modelDefaults[p]||'');setCredentialMode(savedProviders.includes(p)?'saved':'one-time');setKey('');setStatus('');setMessages([]);setShared('');}
 async function saveKey(){
   if(readonly||provider==='ollama'||key.length<8)return;
   setSaving(true);setStatus('');
   try{
     await agent('POST','/coach/credentials',{provider,api_key:key});
     setSavedProviders(previous=>[...new Set([...previous,provider])]);setKey('');setCredentialMode('saved');
     setStatus('Key encrypted for the current Windows account. Saved-key mode is now active.');
   }catch(e){setStatus('Could not save key: '+String(e));}
   finally{setSaving(false);}
 }
 async function deleteKey(){
   if(readonly||!hasSaved)return;
   setSaving(true);setStatus('');
   try{
     await agent('DELETE','/coach/credentials/'+provider);
     setSavedProviders(previous=>previous.filter(p=>p!==provider));setCredentialMode('one-time');
     setStatus('Encrypted key deleted. A one-time key can still be used.');
   }catch(e){setStatus('Could not delete key: '+String(e));}
   finally{setSaving(false);}
 }
 async function sendMessage(text:string){
   const prompt=text.trim();
   if(!prompt||busy||readonly||!consent||prompt.length>1000||(provider!=='ollama'&&!useSaved&&key.length<8))return;
   const history=messages.filter(m=>!m.error).slice(-10).map(m=>({role:m.role,content:m.content.slice(-1800)}));
   const oneTimeKey=useSaved?null:key;
   setBusy(true);setStatus('');setQuestion('');setMessages(previous=>[...previous,{role:'user',content:prompt}]);
   try{
     const reply=await agent<{answer:string;shared:unknown;truncated?:boolean}>('POST','/coach',{
       provider,model,api_key:oneTimeKey,use_saved_key:useSaved,question:prompt,history,share_aggregates:true
     });
     setShared(JSON.stringify(reply.shared,null,2));
     setMessages(previous=>[...previous,{role:'assistant',content:reply.answer,truncated:!!reply.truncated}]);
   }catch(e){
     const error=String(e);
     setMessages(previous=>[...previous,{role:'assistant',content:'The request failed. '+error,error:true}]);
     if(error.includes('saved_key_missing_or_unavailable'))setStatus('Saved key could not be read. Check that the encrypted key belongs to this Windows user or save it again.');
   }finally{setBusy(false);}
 }
 function submit(e:FormEvent){e.preventDefault();void sendMessage(question);}
 function composerKeys(e:KeyboardEvent<HTMLTextAreaElement>){
   if(e.key==='Enter'&&!e.shiftKey&&!e.nativeEvent.isComposing){
     e.preventDefault();void sendMessage(question);
   }
 }
 function continueReply(){void sendMessage('Continue your last response from the point it stopped. Avoid repeating text already provided.');}
 function clearConversation(){setMessages([]);setShared('');setQuestion('');setStatus('');textareaRef.current?.focus();}
 return <section className="coach-chat-layout">
  <div className="coach-chat-main surface">
    <header className="coach-chat-header">
     <div className="coach-chat-brand"><span className="coach-chat-mark" aria-hidden="true">✦</span><div><strong>Digital Discipline Coach</strong><small>Private-by-default • Chat about your activity</small></div></div>
     <button type="button" className="coach-icon-button" title="New conversation" onClick={clearConversation} disabled={busy}>＋ <span>New chat</span></button>
    </header>
    <div className="coach-chat-scroller" role="log" aria-live="polite" aria-label="AI coaching conversation">
     {messages.length===0&&<div className="coach-chat-empty">
      <div className="coach-chat-emblem">✧</div><h2>What would you like to explore?</h2>
      <p>Ask about your tracked activity, focus goals, or how to plan your day. The coach only receives aggregate totals when you approve a request.</p>
      <div className="coach-chat-suggestions">{prompts.map(p=><button type="button" key={p.label} onClick={()=>{setQuestion(p.value);textareaRef.current?.focus();}}>{p.label} <span>↗</span></button>)}</div>
     </div>}
     <div className="coach-chat-messages">
     {messages.map((m,index)=><div key={index} className={'coach-chat-row '+(m.role==='user'?'coach-chat-user':'coach-chat-assistant')}>
       <div className="coach-chat-avatar" aria-hidden="true">{m.role==='assistant'?'✦':'You'}</div>
       <div className={'coach-chat-bubble '+(m.error?'coach-chat-error':'')}>
        {m.role==='assistant'?<div className="coach-markdown"><ReactMarkdown remarkPlugins={[remarkGfm]} skipHtml>{m.content}</ReactMarkdown></div>:<p>{m.content}</p>}
        {m.truncated&&<div className="coach-chat-truncated"><span>Response reached the model's output limit.</span><button type="button" disabled={busy} onClick={continueReply}>Continue response →</button></div>}
       </div>
      </div>)}
     {busy&&<div className="coach-chat-row coach-chat-assistant"><div className="coach-chat-avatar" aria-hidden="true">✦</div><div className="coach-chat-bubble coach-chat-thinking" aria-label="AI is generating"><i/><i/><i/></div></div>}
     <div ref={endOfChat}/>
     </div>
    </div>
    <form className="coach-composer" onSubmit={submit}>
     <label className="coach-consent-row"><input type="checkbox" checked={consent} onChange={e=>setConsent(e.target.checked)}/> Share today's aggregate activity totals with the selected AI model for each message</label>
     <div className="coach-composer-input"><textarea ref={textareaRef} aria-label="Message Digital Discipline Coach" placeholder={consent?'Message Digital Discipline Coach…':'Enable aggregate-data consent to send a message'} rows={2} maxLength={1000} value={question} onChange={e=>setQuestion(e.target.value)} onKeyDown={composerKeys} disabled={busy||readonly}/><button className="coach-send" aria-label="Send message" title="Send message" type="submit" disabled={busy||readonly||!consent||!question.trim()||(provider!=='ollama'&&!useSaved&&key.length<8)}>↑</button></div>
     <div className="coach-composer-foot"><small>Enter to send · Shift+Enter for newline</small><small>{question.length}/1000</small></div>
    </form>
  </div>
  <aside className="coach-chat-settings">
   <div className="surface padded coach-settings-panel"><h3>Model &amp; credentials</h3>
    <div className="coach-settings-fields">
     <label>AI provider<CustomSelect value={provider} onChange={e=>chooseProvider(e.target.value)}>{['openai','anthropic','gemini','groq','openrouter','ollama'].map(p=><option key={p} value={p}>{p}</option>)}</CustomSelect></label>
     <label>Model ID<input value={model} onChange={e=>setModel(e.target.value)} maxLength={100}/></label>
     {provider!=='ollama'&&<div className="coach-vault">
      <div className="coach-vault-header"><div><strong>🔐 API key vault</strong><small>{vaultLoading?'Checking saved credentials…':hasSaved?'Encrypted key is available':'No saved key for this provider'}</small></div><span className={hasSaved?'coach-key-ready':'coach-key-empty'}>{hasSaved?'Saved':'One-time'}</span></div>
      {hasSaved&&<div className="coach-key-mode" role="group" aria-label="API key selection">
       <button type="button" aria-pressed={credentialMode==='saved'} className={credentialMode==='saved'?'active':''} onClick={()=>{setCredentialMode('saved');setKey('');}}>Use saved key</button>
       <button type="button" aria-pressed={credentialMode==='one-time'} className={credentialMode==='one-time'?'active':''} onClick={()=>setCredentialMode('one-time')}>One-time key</button>
      </div>}
      {!useSaved&&<label>API key<input type="password" autoComplete="off" spellCheck={false} value={key} onChange={e=>setKey(e.target.value)} placeholder="Paste API key" maxLength={512}/></label>}
      <div className="coach-vault-actions">
       {!useSaved&&<button className="secondary-btn" type="button" disabled={readonly||saving||key.length<8} onClick={()=>void saveKey()}>Save encrypted</button>}
       {hasSaved&&<button className="danger-btn" type="button" disabled={readonly||saving} onClick={()=>void deleteKey()}>Delete key</button>}
      </div>
      <p className="form-hint">Encrypted by Windows DPAPI for this user. Never shown again or saved to browser storage. An RDP machine with a temporary profile may lose the encrypted key.</p>
     </div>}
     <div className="coach-data-status"><span className="health-dot on"/> {provider==='ollama'?'Local model selected':useSaved?'Encrypted saved key selected':provider==='ollama'||key.length>=8?'One-time key ready':'Enter your API key to chat'}</div>
     <details className="coach-privacy-details"><summary>What is shared with AI?</summary><p>Today's aggregate activity totals and the message history shown in this chat. No individual browsing URLs, video IDs or window titles are sent by the coach.</p>{shared&&<pre className="json-preview">{shared}</pre>}</details>
     <Feedback text={status}/>
    </div>
   </div>
  </aside>
 </section>;
}

export function PrivacyPanel({readonly,refreshKey,onUpdate}:{readonly:boolean;refreshKey:number;onUpdate:()=>void}) {
 const {data,error,refresh}=useRemote<{settings:PrivacySettings}>('/settings',!readonly,refreshKey);const {message,busy,perform}=useAction(()=>{refresh();onUpdate();});const setting=data?.settings;
 const [predictionEnabled,setPredictionEnabled]=useState(false);
 const [retention,setRetention]=useState(90);const [idle,setIdle]=useState(300000);const [excludedApps,setApps]=useState('');const [excludedDomains,setDomains]=useState('');const [video,setVideo]=useState(true);const [distractors,setDistractors]=useState<string[]>([]);
 useEffect(()=>{if(!setting)return;setRetention(setting.retention_days);setIdle(setting.idle_threshold_ms);setApps(setting.excluded_apps.join(', '));setDomains(setting.excluded_domains.join(', '));setVideo(setting.video_history_visible);setDistractors(setting.distraction_categories||[]);setPredictionEnabled(setting.prediction_opt_in===true);},[setting]);
 function save(e:FormEvent){e.preventDefault();void perform(()=>agent('POST','/settings',{retention_days:retention,idle_threshold_ms:idle,excluded_apps:split(excludedApps),excluded_domains:split(excludedDomains),video_history_visible:video,distraction_categories:distractors,prediction_opt_in:predictionEnabled}),'Privacy preferences saved. Retention cleanup runs at least hourly.');}
 return <Card title="Data minimization & retention"><form className="rule-form" onSubmit={save}><div className="form-grid"><label>History retention<CustomSelect value={retention} onChange={e=>setRetention(+e.target.value)}><option value={1}>1 day</option><option value={7}>7 days</option><option value={30}>30 days</option><option value={90}>90 days</option><option value={365}>1 year</option><option value={3650}>10 years</option></CustomSelect></label><label>Idle threshold<CustomSelect value={idle} onChange={e=>setIdle(+e.target.value)}><option value={60000}>1 minute</option><option value={180000}>3 minutes</option><option value={300000}>5 minutes</option><option value={600000}>10 minutes</option><option value={900000}>15 minutes</option></CustomSelect></label></div><label>Excluded application executable names<input value={excludedApps} placeholder="privateapp.exe, banking.exe" onChange={e=>setApps(e.target.value)}/></label><label>Excluded domains<input value={excludedDomains} placeholder="bank.example.com" onChange={e=>setDomains(e.target.value)}/></label><fieldset className="category-choices"><legend>Categories YOU choose to label as distraction</legend><p className="form-hint">Optional. Never automatically equate entertainment or social time with wasted time.</p>{cats.filter(c=>!['Unclassified','System'].includes(c)).map(c=><label className="check-line" key={c}><input type="checkbox" checked={distractors.includes(c)} onChange={e=>setDistractors(prev=>e.target.checked?[...prev,c]:prev.filter(x=>x!==c))}/>{c}</label>)}</fieldset><label className="check-line"><input type="checkbox" checked={video} onChange={e=>setVideo(e.target.checked)}/> Show saved video IDs in history gallery</label><label className="check-line"><input type="checkbox" checked={predictionEnabled} onChange={e=>setPredictionEnabled(e.target.checked)}/> Opt in to local-only, historical risk forecasting</label><p className="form-hint">Prediction requires ≥21 observed days, enough coverage for the same hour, at least one category you choose, and a held-out calibration check. Unreliable forecasts stay hidden. No cloud or AI provider is used.</p><p className="form-hint">Exclusions stop new agent-side storage for matching foreground applications/domains. Previously collected data must be deleted separately. The extension still collects only where you enable it in each browser.</p><button className="primary-btn" disabled={readonly||busy}>Save privacy settings</button></form>{error&&<Feedback text={error}/>}<Feedback text={message}/></Card>;
}

export function BrowserPairPanel(){return <BrowserSetupWizard/>;}

export function StartupPanel(){
 const [enabled,setEnabled]=useState<boolean|null>(null);
 const [message,setMessage]=useState('');const [busy,setBusy]=useState(false);
 useEffect(()=>{if(isTauri)void autostartStatus().then(setEnabled).catch(e=>setMessage(String(e)));},[]);
 async function change(){setBusy(true);try{const next=!enabled;setMessage(await setAutostart(next));setEnabled(next);}catch(e){setMessage(String(e));}finally{setBusy(false);}}
 return <Card title="Independent agent · Windows sign-in"><p className="muted-paragraph">The native collector runs independently of the overlay. By default it is started when you open Digital Discipline. You can explicitly enable background startup when signing into Windows; no administrator permission required.</p><button className="secondary-btn" disabled={!isTauri||enabled===null||busy} onClick={()=>void change()}>{enabled?'Disable agent startup':'Enable agent startup'}</button><Feedback text={message}/></Card>;
}

export function DevicesPanel({readonly}:{readonly:boolean}){
 const {data,refresh,error}=useRemote<{own_device_id:string;devices:{device_id:string;display_name:string;last_import_ms:number}[];observed_union_ms_7d:number;per_device_observed_ms_7d:Record<string,number>}>('/devices',!readonly,0);
 const [secret,setSecret]=useState('');const [bundle,setBundle]=useState('');
 const [message,setMessage]=useState('');const [busy,setBusy]=useState(false);
 async function makeExport(){if(secret.length<12)return setMessage('Choose a passphrase with at least 12 characters.');setBusy(true);setMessage('');
  try{const original=await agent<{schema:string;source_device_id:string;source_device_name:string;records:unknown[];truncated:boolean}>('GET','/devices/export');
   const ciphertext=await sealBundle(original,secret);setBundle(ciphertext);setSecret('');
   const blob=new Blob([ciphertext],{type:'application/json'});const url=URL.createObjectURL(blob);
   const link=document.createElement('a');link.href=url;link.download='digital-discipline-encrypted-device.json';link.click();setTimeout(()=>URL.revokeObjectURL(url),3000);
   setMessage('Encrypted export created. Transfer only the encrypted file. '+(original.truncated?'The most recent 1,000 sessions were included; export is truncated.':'The export includes only observed local sessions.'));
  }catch(e){setMessage(String(e));}finally{setBusy(false);}}
 async function importBundle(){if(secret.length<12)return setMessage('Enter the bundle passphrase (12+ characters).');setBusy(true);setMessage('');
  try{const decrypted=await openBundle(bundle,secret);const {schema,source_device_id,source_device_name,records}=decrypted;
   const reply=await agent<{imported:number;duplicates_or_excluded:number}>('POST','/devices/import',{schema,source_device_id,source_device_name,records});
   setSecret('');setBundle('');setMessage(`Imported ${reply.imported} sessions. ${reply.duplicates_or_excluded} duplicate or excluded records skipped. Remote records never affect local blocking.`);refresh();
  }catch(e){setMessage(String(e));}finally{setBusy(false);}}
 async function readFile(file:File|undefined){if(!file)return;try{const text=await file.text();if(text.length>650000)throw Error('Encrypted file too large');setBundle(text);setMessage('Encrypted file loaded in memory. Enter its passphrase to import.');}catch(e){setMessage(String(e));}}
 async function forget(id:string){if(!window.confirm('Forget this device and delete all its imported sessions?'))return;setBusy(true);try{await agent('DELETE','/devices/'+id);refresh();setMessage('Linked device data deleted locally.');}catch(e){setMessage(String(e));}finally{setBusy(false);}}
 return <div className="advanced-stack"><div className="double-grid"><Card title="Encrypted device exchange · manual"><p className="muted-paragraph">Exchange activity between two Digital Discipline installations using an AES-256-GCM encrypted file. No cloud account or local-network server. Choose a unique 12+ character passphrase and deliver it securely, separately from the file.</p><label>Temporary passphrase<input type="password" autoComplete="new-password" minLength={12} maxLength={300} value={secret} onChange={e=>setSecret(e.target.value)} placeholder="Never saved"/></label><div className="device-actions"><button className="primary-btn" disabled={readonly||busy||secret.length<12} onClick={()=>void makeExport()}>Encrypt and export this device</button><button className="secondary-btn" disabled={!bundle||busy||!secret} onClick={()=>void importBundle()}>Decrypt and import bundle</button></div><FileDropzone disabled={busy||readonly} onFile={readFile}/><details><summary>Show or paste encrypted bundle text</summary><textarea className="device-bundle" value={bundle} onChange={e=>setBundle(e.target.value)} maxLength={650000} rows={3}/></details><Feedback text={message}/><p className="form-hint">Explicit offline exchange, not continuous automatic sync. Every imported session has a device-scoped ID. Duplicate imports are safe; activity overlap is unioned. If a session wasn't observed on another device, it cannot be reconstructed.</p></Card><Card title="Your devices · last 7 days"><p className="muted-paragraph">Own device ID: <code>{data?.own_device_id||'—'}</code></p><h2>{pretty(data?.observed_union_ms_7d||0)} observed union</h2><p className="form-hint">Across available local and imported device sessions. Concurrent sessions are not double-counted.</p><div className="advanced-list">{Object.entries(data?.per_device_observed_ms_7d||{}).map(([id,ms])=><div className="settings-row" key={id}><span>{id==='local'?'This computer':data?.devices.find(d=>d.device_id===id)?.display_name||'Imported device'}</span><strong>{pretty(ms)}</strong></div>)}</div><div className="small-divider"/><h4>Imported devices</h4>{data?.devices.length?data.devices.map(d=><div className="settings-row" key={d.device_id}><div><strong>{d.display_name}</strong><small>Last import {new Date(d.last_import_ms).toLocaleString()}</small></div><button className="secondary-btn" disabled={readonly||busy} onClick={()=>void forget(d.device_id)}>Forget</button></div>):<p className="muted-paragraph">No linked-device records yet.</p>}{error&&<Feedback text={error}/>}</Card></div></div>;
}
