import {useEffect,useState} from 'react';
import {agent,lifecycleStatus,type LifecycleStatus,type ProcessResources} from './api';
type Report={resources?:ProcessResources;sampled_at_ms:number;agent:{reachable:boolean;tracking_paused:boolean;version?:string;clock_rollbacks?:number;observation_gaps?:number;storage_degraded?:boolean;storage_last_error_at_ms?:number;storage_error_count?:number};
 browser:{last_message_at_ms:number;last_rule_ack_at_ms:number;recent_signal:boolean;recent_rule_ack:boolean};
 storage:{db_bytes:number|null;persisted_event_count:number;session_rows:number;last_accepted_event_at_ms:number|null};
 optional_workers:{auto_classification_enabled:boolean;lite_mode?:boolean};redacted:boolean};
function measured(resources:ProcessResources|undefined){
 return resources?.available?{
   working_set_bytes:resources.working_set_bytes,
   cpu_percent_one_core_basis:resources.cpu_percent_one_core_basis,
   sampled_at_ms:resources.sampled_at_ms,
 }:null;
}
function ResourceSample({label,resources}:{label:string;resources?:ProcessResources}){
 const data=measured(resources);
 return <div><small>{label}</small><strong>{data?((data.working_set_bytes??0)/1048576).toFixed(1)+' MiB':'Unavailable'}</strong>
  {data&&<small>{data.cpu_percent_one_core_basis==null?'CPU needs two samples':data.cpu_percent_one_core_basis.toFixed(2)+'% CPU per core'}</small>}</div>;
}
export function ResourceHealthPanel({readonly}:{readonly:boolean}){
 const [report,setReport]=useState<Report|null>(null),[supervisor,setSupervisor]=useState<LifecycleStatus|null>(null),[error,setError]=useState('');
 const refresh=()=>{void lifecycleStatus().then(setSupervisor).catch(()=>setSupervisor(null));void agent<Report>('GET','/health/details').then(r=>{setReport(r);setError('');}).catch(()=>setError('Local agent unavailable or health endpoint unsupported.'));};
 useEffect(()=>{if(readonly)return;refresh();const timer=setInterval(refresh,20000);return()=>clearInterval(timer);},[readonly]);
 const at=(v:number|null|undefined)=>v?new Date(v).toLocaleTimeString():'Never observed';
 const exportDiagnostics=()=>{
  if(!report||readonly)return;
  const diagnostics={
   schema:1, sampled_at_ms:report.sampled_at_ms,
   version:report.agent.version, tracking_paused:report.agent.tracking_paused,
   observation_gaps:report.agent.observation_gaps, clock_rollbacks:report.agent.clock_rollbacks,
   storage_degraded:report.agent.storage_degraded, storage_error_count:report.agent.storage_error_count,
   recent_browser_signal:report.browser.recent_signal, recent_rule_ack:report.browser.recent_rule_ack,
   database_bytes:report.storage.db_bytes, persisted_event_count:report.storage.persisted_event_count,
   session_rows:report.storage.session_rows, agent_restarts:supervisor?.restart_count,
   agent_process:measured(report.resources), desktop_host_process:measured(supervisor?.resources),
  };
  const url=URL.createObjectURL(new Blob([JSON.stringify(diagnostics,null,2)],{type:'application/json'}));
  const link=document.createElement('a');
  link.href=url;
  link.download='digital-discipline-diagnostics.json';
  link.click();
  window.setTimeout(()=>URL.revokeObjectURL(url),3000);
 };
 return <article className="surface padded advanced-card" aria-label="Resource and connection health">
  <div className="panel-header"><div><h3>Resource &amp; connection health</h3>
   <p>Collection freshness, recovery and storage diagnostics</p></div><div className="release-actions"><button className="secondary-btn" onClick={refresh} disabled={readonly}>Refresh</button><button className="secondary-btn" onClick={exportDiagnostics} disabled={readonly||!report}>Export diagnostics</button></div></div>
  {supervisor&&<p className="form-hint">Desktop recovery: {supervisor.state.replaceAll('_',' ')}{supervisor.restart_count!==undefined?` · ${supervisor.restart_count} agent launches this session`:''}</p>}
  {report?<div className="health-metric-grid">
   <div><small>Windows collection</small><strong>{report.agent.storage_degraded?'Storage degraded':report.agent.tracking_paused?'Paused':'Agent reachable'}</strong>
    {report.agent.storage_degraded&&<small>Recent SQLite write failed. Review disk space and file permissions; missing time will not be backfilled.</small>}</div>
   <div><small>Browser transport</small><strong>{report.browser.recent_signal?'Recent signal':'Not confirmed'}</strong><small>Last event: {at(report.browser.last_message_at_ms)}</small></div>
   <div><small>Browser restrictions</small><strong>{report.browser.recent_rule_ack?'Recent acknowledgment':'Not confirmed'}</strong><small>Last rule response: {at(report.browser.last_rule_ack_at_ms)}</small></div>
   <div><small>Event persistence</small><strong>{report.storage.persisted_event_count.toLocaleString()} events</strong><small>Latest: {at(report.storage.last_accepted_event_at_ms)}</small></div>
   <div><small>Local database</small><strong>{report.storage.db_bytes===null?'Unavailable':(report.storage.db_bytes/1024/1024).toFixed(2)+' MiB'}</strong><small>{report.storage.session_rows.toLocaleString()} session rows</small></div>
   <div><small>Observation gaps</small><strong>{report.agent.observation_gaps??0} gaps · {report.agent.clock_rollbacks??0} clock corrections</strong><small>Unknown or repeated clock intervals are excluded from tracked time.</small></div>
   <ResourceSample label="Agent process" resources={report.resources}/>
   <ResourceSample label="Desktop host process" resources={supervisor?.resources}/>
   <div><small>Optional AI worker</small><strong>{report.optional_workers.auto_classification_enabled?'Enabled':'Disabled'}</strong><small>{report.optional_workers.lite_mode?'Lite scheduling: every 3 minutes':'Standard scheduling'}</small></div>
  </div>:<p className="muted-paragraph">{error||'Loading live component status…'}</p>}
  {report&&error&&<p role="status">{error} Showing the last sample.</p>}
  <p className="form-hint">Resource readings cover the agent and desktop host processes. The diagnostics file contains aggregate health metrics.</p>
 </article>;
}
