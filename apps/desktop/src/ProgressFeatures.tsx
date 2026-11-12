import { useCallback, useEffect, useState } from 'react';
import type { FormEvent } from 'react';
import { agent } from './api';
import { CustomSelect } from './CustomSelect';
import { DatePicker } from './DatePicker';
import { TimePicker } from './TimePicker';
import type { Session } from './types';

const toTime = (minute: number) => [Math.floor(minute / 60), minute % 60]
  .map(value => String(value).padStart(2, '0')).join(':');
const toMinute = (time: string) => {
  const [hour, minute] = time.split(':').map(Number);
  return hour * 60 + minute;
};
const minutes = (ms: number) => Math.floor(ms / 60_000);
const split = (input: string) => [...new Set(input.split(',').map(value => value.trim().toLowerCase()).filter(Boolean))];

export function useAgentPanel<T>(path: string, disabled: boolean) {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const reload = useCallback(async () => {
    if (disabled) return;
    try {
      setData(await agent<T>('GET', path));
      setError('');
    } catch (error) { setError(String(error).replaceAll('_', ' ')); }
  }, [disabled, path]);
  useEffect(() => { void reload(); }, [reload]);
  async function change(action: () => Promise<unknown>) {
    setBusy(true);
    setError('');
    try { await action(); await reload(); return true; }
    catch (error) { setError(String(error).replaceAll('_', ' ')); return false; }
    finally { setBusy(false); }
  }
  return {data, error, busy, reload, change};
}

type Plan = {
  id: string; revision: number; day: string; start_min: number; minutes: number;
  kind: string; title: string; profile_id: string | null; status: string; missed: boolean; overlaps: boolean;
};
type Target = {
  id: string; revision: number; title: string; metric: string; target: number; days: number;
  progress: number; remaining: number; remaining_days: number; per_day: number | null;
};
type PlanData = {
  today: string; blocks: Plan[]; may_be_truncated: boolean;
  report: {
    start: string; through: string; observed_ms: number; days_observed: number;
    previous_days_observed: number; comparison_ms: number | null; completed_tasks: number;
    focus_sessions_ended: number; rule_activations: number; temporary_exceptions: number; targets: Target[];
    reading_ms?: number; regular_video_ms?: number;
  };
};
type PlanDraft = Pick<Plan, 'day'|'start_min'|'minutes'|'kind'|'title'|'profile_id'> & Partial<Pick<Plan, 'id'|'revision'>>;
type TargetDraft = Pick<Target, 'title'|'metric'|'target'|'days'> & Partial<Pick<Target, 'id'|'revision'>>;
const newPlan = (day: string): PlanDraft => ({day, start_min:540, minutes:25, kind:'focus', title:'', profile_id:null});
const newTarget: TargetDraft = {title:'', metric:'observed_minutes', target:300, days:31};

export function PlannerPanel({disabled, onFocus}: {disabled: boolean; onFocus: () => void}) {
  const panel = useAgentPanel<PlanData>('/planner', disabled);
  const [profiles, setProfiles] = useState<{id: string; name: string}[]>([]);
  const [draft, setDraft] = useState<PlanDraft | null>(null);
  const [target, setTarget] = useState<TargetDraft>(newTarget);
  useEffect(() => {
    if (!disabled) void agent<{profiles: typeof profiles}>('GET','/workflows')
      .then(result => setProfiles(result.profiles ?? [])).catch(() => {});
  }, [disabled]);

  if (disabled) return <div className="surface padded">Connect to the local agent to edit your plan.</div>;
  if (!panel.data?.today || !Array.isArray(panel.data.blocks) || !Array.isArray(panel.data.report?.targets)) return <div className="surface padded" role="status">{panel.error || (panel.data?'Planner data is unavailable.':'Loading planner…')}</div>;
  const {blocks, today, report} = panel.data;
  const plan = draft ?? newPlan(today);
  const patch = (value: Partial<PlanDraft>) => setDraft({...plan, ...value});
  async function save(event: FormEvent) {
    event.preventDefault();
    if (await panel.change(() => agent('POST','/planner',plan))) setDraft(null);
  }
  async function action(block: Plan, action: string) {
    const ok = await panel.change(() => agent('POST','/planner/action',{id:block.id, revision:block.revision, action}));
    if (ok && action === 'start') onFocus();
  }
  async function saveTarget(event: FormEvent) {
    event.preventDefault();
    if (await panel.change(() => agent('POST','/planner/weekly-target',target))) setTarget(newTarget);
  }
  return <div className="release-stack">
    {panel.error && <p role="alert" className="form-hint">{panel.error}</p>}
    <section className="surface padded">
      <div className="panel-header"><div><h3>Time blocks</h3><p>Set a plan for today or a later day. Start a focus block when you’re ready.</p></div></div>
      <form className="release-form" onSubmit={save}>
        <label>Block name<input required maxLength={120} value={plan.title} onChange={e=>patch({title:e.target.value})}/></label>
        <label>Day<DatePicker value={plan.day} onChange={day=>patch({day})} aria-label="Plan day"/></label>
        <label>Start time<TimePicker value={toTime(plan.start_min)} onChange={e=>patch({start_min:toMinute(e.target.value)})} aria-label="Plan start time"/></label>
        <label>Minutes<input type="number" min={1} max={240} required value={plan.minutes} onChange={e=>patch({minutes:Number(e.target.value)})}/></label>
        <label>Block type<CustomSelect value={plan.kind} onChange={e=>patch({kind:e.target.value, profile_id:null})} aria-label="Block type">
          <option value="focus">Focus</option><option value="task">Task</option><option value="break">Break</option><option value="free">Free time</option>
        </CustomSelect></label>
        {plan.kind==='focus' && <label>Focus profile<CustomSelect value={plan.profile_id??''} onChange={e=>patch({profile_id:e.target.value||null})} aria-label="Planner focus profile">
          <option value="">General focus</option>{profiles.map(profile=><option key={profile.id} value={profile.id}>{profile.name}</option>)}
        </CustomSelect></label>}
        <div className="release-actions"><button className="primary-btn" disabled={panel.busy}>{plan.id?'Save changes':'Add block'}</button>
          {plan.id && <button type="button" className="secondary-btn" onClick={()=>setDraft(null)}>Cancel edit</button>}</div>
      </form>
      <div className="release-list">{blocks.map(block=><div className="release-row" key={block.id}>
        <div><strong>{block.title}</strong><p>{block.day} · {toTime(block.start_min)} · {block.minutes} min · {block.kind}</p>
          <p>{block.status==='started'?'Focus started':block.status==='skipped'?'Skipped':block.missed?'Missed · reschedule or skip':'Planned'}{block.overlaps && block.status==='planned'?' · Overlaps another plan':''}</p></div>
        {block.status==='planned' && <div className="release-actions">
          {block.kind==='focus' && <button className="secondary-btn" disabled={panel.busy} onClick={()=>void action(block,'start')}>Start focus</button>}
          <button className="text-action" onClick={()=>setDraft({id:block.id,revision:block.revision,day:block.day,start_min:block.start_min,minutes:block.minutes,kind:block.kind,title:block.title,profile_id:block.profile_id})}>Reschedule</button>
          <button className="text-action" disabled={panel.busy} onClick={()=>void action(block,'skip')}>Skip</button>
        </div>}
        {block.status==='skipped' && <button className="text-action" disabled={panel.busy} onClick={()=>void action(block,'remove')}>Remove</button>}
      </div>)}</div>
      {!blocks.length && <p className="form-hint">Your time blocks will appear here.</p>}
      {panel.data.may_be_truncated && <p className="form-hint">Showing the first 500 blocks in this date range.</p>}
    </section>
    <section className="surface padded">
      <div className="panel-header"><div><h3>Weekly review</h3><p>{report.start} through {report.through} · {report.days_observed} observed days</p></div>
        <button className="text-action" onClick={()=>void panel.reload()}>Refresh</button></div>
      <dl className="release-metrics">
        <div><dt>Observed active time</dt><dd>{minutes(report.observed_ms)} min</dd></div>
        <div><dt>Tasks you completed</dt><dd>{report.completed_tasks}</dd></div>
        <div><dt>Focus sessions ended</dt><dd>{report.focus_sessions_ended}</dd></div>
        <div><dt>Rule activations</dt><dd>{report.rule_activations}</dd></div>
        <div><dt>Temporary exceptions</dt><dd>{report.temporary_exceptions}</dd></div>
        {report.reading_ms!==undefined && <div><dt>Confirmed reading</dt><dd>{minutes(report.reading_ms)} min</dd></div>}
        {report.regular_video_ms!==undefined && <div><dt>Visible regular video</dt><dd>{minutes(report.regular_video_ms)} min</dd></div>}
      </dl>
      <p className="form-hint">{report.comparison_ms===null
        ? `Last week had ${report.previous_days_observed} observed days in the matching period. Comparable coverage is unavailable.`
        : `Same weekdays and time last week · ${Math.round(report.comparison_ms/60_000)} min difference in observed time.`}</p>
      {report.temporary_exceptions>0 && <p className="form-hint">Review the rules you often unlock before planning next week.</p>}
    </section>
    <section className="surface padded">
      <div className="panel-header"><div><h3>Weekly targets</h3><p>Choose the days you want to work toward each target.</p></div></div>
      <form className="release-form" onSubmit={saveTarget}>
        <label>Target name<input required maxLength={120} value={target.title} onChange={e=>setTarget({...target,title:e.target.value})}/></label>
        <label>Measure<CustomSelect value={target.metric} onChange={e=>setTarget({...target,metric:e.target.value})} aria-label="Weekly target measure">
          <option value="observed_minutes">Observed active minutes</option><option value="completed_tasks">Owner-completed tasks</option>
          {report.reading_ms!==undefined && <option value="reading_minutes">Confirmed reading minutes</option>}
        </CustomSelect></label>
        <label>Weekly total<input type="number" min={1} max={target.metric==='completed_tasks'?1000:10080} value={target.target} onChange={e=>setTarget({...target,target:Number(e.target.value)})}/></label>
        <fieldset className="release-days"><legend>Target days</legend>
          {['Mon','Tue','Wed','Thu','Fri','Sat','Sun'].map((day,i)=><label key={day}><input type="checkbox" checked={!!(target.days&(1<<i))} onChange={()=>setTarget({...target,days:target.days^(1<<i)})}/>{day}</label>)}
        </fieldset>
        <div className="release-actions"><button className="primary-btn" disabled={panel.busy||target.days===0}>{target.id?'Save target':'Add target'}</button>
          {target.id && <button type="button" className="secondary-btn" onClick={()=>setTarget(newTarget)}>Cancel edit</button>}</div>
      </form>
      <div className="release-list">{report.targets.map(item=><div className="release-row" key={item.id}>
        <div><strong>{item.title}</strong><p>{item.progress} of {item.target} {item.metric==='completed_tasks'?'tasks':'minutes'} this week</p>
          <p>{item.remaining===0?'Target reached':item.per_day===null?'Choose another target day to rebalance the remaining work.':`${item.per_day} per remaining target day · ${item.remaining_days} days left`}</p></div>
        <div className="release-actions"><button className="text-action" onClick={()=>setTarget({id:item.id,revision:item.revision,title:item.title,metric:item.metric,target:item.target,days:item.days})}>Rebalance</button>
          <button className="text-action" disabled={panel.busy} onClick={()=>void panel.change(()=>agent('POST','/planner/weekly-target',{id:item.id,revision:item.revision,title:item.title,metric:item.metric,target:item.target,days:item.days,remove:true}))}>Remove</button></div>
      </div>)}</div>
    </section>
  </div>;
}

const categories = ['Development','Design','Productivity','Communication','Education','Entertainment','Gaming','Social','News','Utility','System','Browser','Unclassified'];
type HistoryData = {
  corrections: {session_id:string;app:string;category:string|null;hidden:boolean;can_undo:boolean}[];
  manual: {id:string;start_ms:number;end_ms:number;label:string;category:string}[];
};
export function HistoryCorrections({day, sessions, disabled, onRefresh}: {
  day:string; sessions:Session[]; disabled:boolean; onRefresh:()=>void;
}) {
  const panel=useAgentPanel<HistoryData>('/history/'+day,disabled);
  const [query,setQuery]=useState('');
  const [id,setId]=useState('');
  const [category,setCategory]=useState('Productivity');
  const [time,setTime]=useState('09:00');
  const [duration,setDuration]=useState(30);
  const [label,setLabel]=useState('');
  if (disabled) return null;
  const matches=sessions.filter(s=>!s.idle&&s.end_ms<Date.now()-10000&&
    (s.app+' '+(s.domain??'')).toLowerCase().includes(query.trim().toLowerCase())).slice(0,100);
  const chosen=matches.find(s=>s.id===id);
  async function correct(session_id:string,action:string) {
    if (await panel.change(()=>agent('POST','/history/correct',{session_id,action,category}))) {
      setId(''); onRefresh();
    }
  }
  async function addManual(event:FormEvent) {
    event.preventDefault();
    if (await panel.change(()=>agent('POST','/history/manual',{start_ms:new Date(day+'T'+time+':00').getTime(),minutes:duration,label,category}))) setLabel('');
  }
  return <section className="surface padded">
    <div className="panel-header"><div><h3>Review and correct history</h3><p>Corrections apply to one observation. Offline entries are shown separately.</p></div></div>
    {panel.error && <p role="alert">{panel.error}</p>}
    <div className="release-form">
      <label>Find an app or site<input value={query} onChange={e=>{setQuery(e.target.value);setId('');}} placeholder="Search this day"/></label>
      <label>Recorded interval<CustomSelect value={id} onChange={e=>setId(e.target.value)} aria-label="Recorded interval">
        <option value="">Choose an interval</option>{matches.map(s=><option key={s.id} value={s.id}>{new Date(s.start_ms).toLocaleTimeString([], {hour:'2-digit',minute:'2-digit'})} · {s.domain||s.app} · {s.category}</option>)}
      </CustomSelect></label>
      <label>Category<CustomSelect value={category} onChange={e=>setCategory(e.target.value)} aria-label="Correction category">{categories.map(c=><option key={c}>{c}</option>)}</CustomSelect></label>
      <div className="release-actions"><button className="secondary-btn" disabled={!chosen||panel.busy} onClick={()=>void correct(id,'category')}>Apply category</button>
        <button className="secondary-btn" disabled={!chosen||panel.busy} onClick={()=>void correct(id,'hide')}>Hide false observation</button></div>
    </div>
    {panel.data?.corrections?.map(item=><div className="release-row" key={item.session_id}>
      <span>{item.app} · {item.hidden?'Hidden observation':item.category}</span>
      <button className="text-action" disabled={!item.can_undo||panel.busy} onClick={()=>void correct(item.session_id,'undo')}>{item.can_undo?'Undo correction':'Undo period ended'}</button>
    </div>)}
    <form className="release-form release-subsection" onSubmit={addManual}>
      <label>Offline work label<input maxLength={120} value={label} required onChange={e=>setLabel(e.target.value)}/></label>
      <label>Start<TimePicker value={time} onChange={e=>setTime(e.target.value)} aria-label="Offline work start"/></label>
      <label>Minutes<input type="number" min={1} max={480} value={duration} onChange={e=>setDuration(Number(e.target.value))}/></label>
      <div className="release-actions"><button className="secondary-btn" disabled={panel.busy}>Add self-reported time</button></div>
    </form>
    {panel.data?.manual?.map(item=><div className="release-row" key={item.id}><span>{item.label} · {item.category} · {minutes(item.end_ms-item.start_ms)} min · Self-reported</span>
      <button className="text-action" disabled={panel.busy} onClick={()=>void panel.change(()=>agent('POST','/history/manual',{id:item.id}))}>Remove</button></div>)}
  </section>;
}

type Terms = {enabled:boolean;work_apps:string[];work_domains:string[];spend_domains:string[];
  work_minutes:number;reward_minutes:number;daily_cap_minutes:number;rollover:boolean};
type RewardData = {terms:Terms;balance_ms:number;earned_ms:number;spent_ms:number;pending_ms:number;
  entries:{id:number;day:string;at_ms:number;kind:string;amount_ms:number}[]};
export function EarnedTimePanel({disabled}: {disabled:boolean}) {
  const panel=useAgentPanel<RewardData>('/rewards',disabled);
  const [draft,setDraft]=useState<(Omit<Terms,'work_apps'|'work_domains'|'spend_domains'>&{work_apps:string;work_domains:string;spend_domains:string})|null>(null);
  const [adjust,setAdjust]=useState(1);
  if (disabled) return null;
  if (!panel.data?.terms || !Array.isArray(panel.data.entries)) return <div className="surface padded" role="status">{panel.error||(panel.data?'Earned time data is unavailable.':'Loading earned time…')}</div>;
  const {terms, balance_ms, earned_ms, spent_ms, pending_ms, entries}=panel.data;
  const form=draft??{...terms,work_apps:terms.work_apps.join(', '),work_domains:terms.work_domains.join(', '),spend_domains:terms.spend_domains.join(', ')};
  async function save(event:FormEvent) {
    event.preventDefault();
    if (await panel.change(()=>agent('POST','/rewards',{...form,work_apps:split(form.work_apps),work_domains:split(form.work_domains),spend_domains:split(form.spend_domains)}))) setDraft(null);
  }
  return <section className="surface padded">
    <div className="panel-header"><div><h3>Earned entertainment</h3><p>Give selected leisure sites a budget earned through observed foreground work.</p></div>
      <button className="text-action" onClick={()=>void panel.reload()}>Refresh balance</button></div>
    {panel.error && <p role="alert">{panel.error}</p>}
    <dl className="release-metrics"><div><dt>Available</dt><dd>{minutes(balance_ms)} min</dd></div><div><dt>Earned today</dt><dd>{minutes(earned_ms)} min</dd></div>
      <div><dt>Spent today</dt><dd>{minutes(spent_ms)} min</dd></div><div><dt>Work toward next reward</dt><dd>{minutes(pending_ms)} min</dd></div></dl>
    <form className="release-form" onSubmit={save}>
      <label className="release-check"><input type="checkbox" checked={form.enabled} onChange={e=>setDraft({...form,enabled:e.target.checked})}/> Enable earned browser limits</label>
      <label>Work apps<input value={form.work_apps} placeholder="code.exe, blender.exe" onChange={e=>setDraft({...form,work_apps:e.target.value})}/></label>
      <label>Work sites<input value={form.work_domains} placeholder="docs.python.org" onChange={e=>setDraft({...form,work_domains:e.target.value})}/></label>
      <label>Leisure sites<input value={form.spend_domains} placeholder="youtube.com" onChange={e=>setDraft({...form,spend_domains:e.target.value})}/></label>
      <label>Work minutes per reward<input type="number" min={1} max={240} value={form.work_minutes} onChange={e=>setDraft({...form,work_minutes:Number(e.target.value)})}/></label>
      <label>Reward minutes<input type="number" min={1} max={120} value={form.reward_minutes} onChange={e=>setDraft({...form,reward_minutes:Number(e.target.value)})}/></label>
      <label>Daily earn cap<input type="number" min={1} max={240} value={form.daily_cap_minutes} onChange={e=>setDraft({...form,daily_cap_minutes:Number(e.target.value)})}/></label>
      <label className="release-check"><input type="checkbox" checked={form.rollover} onChange={e=>setDraft({...form,rollover:e.target.checked})}/> Carry unused time to tomorrow</label>
      <div className="release-actions"><button className="primary-btn" disabled={panel.busy}>Save earned time rule</button></div>
    </form>
    <p className="form-hint">Separate entries with commas. Saving an enabled rule applies a block to the selected sites when your balance runs out. Site spending requires browser recording.</p>
    {terms.enabled && <form className="release-form release-subsection" onSubmit={e=>{e.preventDefault();void panel.change(()=>agent('POST','/rewards/adjust',{minutes:adjust}));}}>
      <label>Balance adjustment in minutes<input type="number" min={-240} max={240} value={adjust} onChange={e=>setAdjust(Number(e.target.value))}/></label>
      <div className="release-actions"><button className="secondary-btn" disabled={panel.busy||adjust===0}>Adjust balance</button></div>
    </form>}
    {entries.length>0 && <details className="release-subsection"><summary>Balance history</summary><div className="release-list">
      {entries.map(entry=><div className="release-row" key={entry.id}><span>{entry.day} · {entry.kind.replaceAll('_',' ')}</span><span>{(entry.amount_ms/60000).toFixed(1)} min</span></div>)}
    </div></details>}
  </section>;
}
