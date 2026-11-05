import {useEffect,useState} from 'react';
import {agent} from './api';
type Report={sampled_at_ms:number;agent:{reachable:boolean;tracking_paused:boolean};
 browser:{last_message_at_ms:number;last_rule_ack_at_ms:number;recent_signal:boolean;recent_rule_ack:boolean};
 storage:{db_bytes:number|null;persisted_event_count:number;session_rows:number;last_accepted_event_at_ms:number|null};
 optional_workers:{auto_classification_enabled:boolean};redacted:boolean};
export function ResourceHealthPanel({readonly}:{readonly:boolean}){
 const [report,setReport]=useState<Report|null>(null),[error,setError]=useState('');
 const refresh=()=>void agent<Report>('GET','/health/details').then(r=>{setReport(r);setError('');}).catch(()=>setError('Local agent unavailable or health endpoint unsupported.'));
 useEffect(()=>{if(readonly)return;refresh();const timer=setInterval(refresh,20000);return()=>clearInterval(timer);},[readonly]);
 const at=(v:number|null|undefined)=>v?new Date(v).toLocaleTimeString():'Never observed';
 return <article className="surface padded advanced-card" aria-label="Resource and connection health">
  <div className="panel-header"><div><h3>Resource &amp; connection health</h3>
   <p>Local observation freshness and storage diagnostics</p></div><button className="secondary-btn" onClick={refresh} disabled={readonly}>Refresh</button></div>
  {report?<div className="health-metric-grid">
   <div><small>Windows collection</small><strong>{report.agent.tracking_paused?'Paused':'Agent reachable'}</strong></div>
   <div><small>Browser transport</small><strong>{report.browser.recent_signal?'Recent signal':'Not confirmed'}</strong><small>Last event: {at(report.browser.last_message_at_ms)}</small></div>
   <div><small>Browser restrictions</small><strong>{report.browser.recent_rule_ack?'Recent acknowledgment':'Not confirmed'}</strong><small>Last rule response: {at(report.browser.last_rule_ack_at_ms)}</small></div>
   <div><small>Event persistence</small><strong>{report.storage.persisted_event_count.toLocaleString()} events</strong><small>Latest: {at(report.storage.last_accepted_event_at_ms)}</small></div>
   <div><small>Local database</small><strong>{report.storage.db_bytes===null?'Unavailable':(report.storage.db_bytes/1024/1024).toFixed(2)+' MiB'}</strong><small>{report.storage.session_rows.toLocaleString()} session rows</small></div>
   <div><small>Optional AI worker</small><strong>{report.optional_workers.auto_classification_enabled?'Enabled':'Disabled'}</strong></div>
  </div>:<p className="muted-paragraph">{error||'Loading live component status…'}</p>}
  <p className="form-hint">An authorized browser is not necessarily online or recording. This report cannot inspect recording consent inside every browser profile. Check the Brave companion for its recording switch, observed plays and local event queue.</p>
 </article>;
}
