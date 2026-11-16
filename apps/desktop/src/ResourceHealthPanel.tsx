import {useCallback, useEffect, useRef, useState} from 'react';
import {agent, lifecycleStatus, type LifecycleStatus, type ProcessResources} from './api';

type Report = {
  resources?: ProcessResources;
  sampled_at_ms: number;
  agent: {
    reachable: boolean; tracking_paused: boolean; version?: string; clock_rollbacks?: number;
    observation_gaps?: number; storage_degraded?: boolean; storage_error_count?: number;
  };
  browser: {last_message_at_ms:number; last_rule_ack_at_ms:number; recent_signal:boolean; recent_rule_ack:boolean};
  storage: {db_bytes:number|null; persisted_event_count:number; session_rows:number; last_accepted_event_at_ms:number|null};
  optional_workers: {auto_classification_enabled:boolean; lite_mode?:boolean};
};

type Sample = {at:number; agent?:ProcessResources; desktop?:ProcessResources};
type Metric = 'memory' | 'cpu';
const series = ['agent','desktop'] as const;
const time = (value?:number|null) => value ? new Date(value).toLocaleTimeString() : '—';
const mib = (bytes:number|null|undefined) => bytes == null ? '—' : (bytes / 1048576).toFixed(1) + ' MiB';

function valueAt(resources: ProcessResources | undefined, metric: Metric) {
  if (!resources?.available) return null;
  const value = metric === 'memory'
    ? resources.working_set_bytes == null ? null : resources.working_set_bytes / 1048576
    : resources.cpu_percent_one_core_basis;
  return typeof value === 'number' && Number.isFinite(value) && value >= 0 ? value : null;
}

function ResourceChart({samples, metric}: {samples:Sample[]; metric:Metric}) {
  const unit = metric === 'memory' ? 'MiB' : '%';
  const title = metric === 'memory' ? 'Working set memory' : 'CPU per core';
  const values = samples.flatMap(sample => series.map(key => valueAt(sample[key], metric)).filter(value => value != null));
  const quantum = metric === 'memory' ? 16 : 1;
  const maximum = Math.max(quantum, Math.ceil(Math.max(0, ...values) / quantum) * quantum);
  const first = samples.at(0)?.at ?? 0;
  const last = samples.at(-1)?.at ?? first;
  const width = 360;
  const height = 100;
  const x = (at:number) => 42 + (last === first ? width / 2 : ((at - first) / (last - first)) * width);
  const y = (value:number) => 10 + (1 - value / maximum) * height;
  const display = (value:number|null) => value == null ? '—' : value.toFixed(metric === 'memory' ? 1 : 2) + ' ' + unit;

  return <figure className="diagnostic-chart" aria-label={title}>
    <figcaption>
      <h4>{title}</h4>
      <div className="chart-legend">{series.map(key => <span key={key} className={'chart-key ' + key}>
        {key === 'agent' ? 'Agent' : 'Desktop'} <strong>{display(valueAt(samples.at(-1)?.[key], metric))}</strong>
      </span>)}</div>
    </figcaption>
    {values.length ? <>
      <svg viewBox="0 0 416 120" role="img" aria-label={title + ' over ' + samples.length + ' live samples'}>
        {[1,.5,0].map(fraction => <g key={fraction}>
          <line className="chart-gridline" x1="42" x2="402" y1={y(maximum * fraction)} y2={y(maximum * fraction)}/>
          <text x="35" y={y(maximum * fraction) + 4} textAnchor="end">{(maximum * fraction).toLocaleString(undefined,{maximumFractionDigits:1})}</text>
        </g>)}
        {series.map(key => {
          let drawing = false;
          const path = samples.map(sample => {
            const value = valueAt(sample[key], metric);
            if (value == null) { drawing = false; return ''; }
            const point = (drawing ? 'L' : 'M') + x(sample.at) + ' ' + y(value);
            drawing = true;
            return point;
          }).join(' ');
          const current = valueAt(samples.at(-1)?.[key], metric);
          return <g key={key} className={'chart-series ' + key}>
            <path d={path}/>
            {current != null && <circle cx={x(last)} cy={y(current)} r="3"/>}
          </g>;
        })}
      </svg>
      <div className="chart-times"><span>{time(first)}</span><span>{time(last)}</span></div>
    </> : <p className="chart-empty">{metric === 'cpu' ? 'Waiting for two process samples' : 'Waiting for a process sample'}</p>}
  </figure>;
}

export function ResourceHealthPanel({readonly, lite=false}: {readonly:boolean; lite?:boolean}) {
  const [report, setReport] = useState<Report | null>(null);
  const [supervisor, setSupervisor] = useState<LifecycleStatus | null>(null);
  const [samples, setSamples] = useState<Sample[]>([]);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const request = useRef(0);
  const running = useRef(false);
  const refresh = useCallback(async () => {
    if (readonly || running.current) return;
    running.current = true;
    const generation = ++request.current;
    setBusy(true);
    try {
      const [health, desktop] = await Promise.allSettled([
        agent<Report>('GET', '/health/details'),
        lifecycleStatus(),
      ]);
      if (generation !== request.current) return;
      const next = health.status === 'fulfilled' && health.value?.agent && health.value.storage && health.value.browser && health.value.optional_workers
        ? health.value : null;
      const host = desktop.status === 'fulfilled' ? desktop.value : null;
      if (next) setReport(next);
      setSupervisor(host);
      setError(next ? '' : 'The local agent did not return a health report.');
      const at = next?.sampled_at_ms ?? host?.resources?.sampled_at_ms ?? Date.now();
      setSamples(previous => {
        const last = previous.at(-1)?.at;
        if (last === at) return previous;
        const history = last != null && at < last ? [] : previous.slice(-47);
        return [...history, {at, agent:next?.resources, desktop:host?.resources}];
      });
    } finally {
      if (generation === request.current) {
        running.current = false;
        setBusy(false);
      }
    }
  }, [readonly]);
  useEffect(() => {
    if (readonly) return;
    void refresh();
    const timer = window.setInterval(() => void refresh(), lite ? 20000 : 5000);
    return () => {window.clearInterval(timer); request.current++; running.current = false;};
  }, [refresh, readonly, lite]);

  function exportDiagnostics() {
    if (!report || readonly) return;
    const diagnostics = {
      schema:1, sampled_at_ms:report.sampled_at_ms,
      version:report.agent.version, tracking_paused:report.agent.tracking_paused,
      observation_gaps:report.agent.observation_gaps, clock_rollbacks:report.agent.clock_rollbacks,
      storage_degraded:report.agent.storage_degraded, storage_error_count:report.agent.storage_error_count,
      recent_browser_signal:report.browser.recent_signal, recent_rule_ack:report.browser.recent_rule_ack,
      database_bytes:report.storage.db_bytes, persisted_event_count:report.storage.persisted_event_count,
      session_rows:report.storage.session_rows, agent_restarts:supervisor?.restart_count,
      agent_process:report.resources, desktop_host_process:supervisor?.resources,
      process_samples:samples,
    };
    const url = URL.createObjectURL(new Blob([JSON.stringify(diagnostics, null, 2)], {type:'application/json'}));
    const link = document.createElement('a');
    link.href = url;
    link.download = 'digital-discipline-diagnostics.json';
    link.click();
    window.setTimeout(() => URL.revokeObjectURL(url), 3000);
  }

  const rows = report ? [
    ['Windows collection', report.agent.storage_degraded ? 'Storage degraded' : report.agent.tracking_paused ? 'Paused' : 'Agent reachable', 'Agent ' + (report.agent.version ?? '—')],
    ['Browser transport', report.browser.recent_signal ? 'Recent signal' : 'Awaiting recent signal', time(report.browser.last_message_at_ms)],
    ['Restriction sync', report.browser.recent_rule_ack ? 'Recent acknowledgment' : 'Awaiting acknowledgment', time(report.browser.last_rule_ack_at_ms)],
    ['Event storage', report.storage.persisted_event_count.toLocaleString() + ' events', time(report.storage.last_accepted_event_at_ms)],
    ['Local database', mib(report.storage.db_bytes), report.storage.session_rows.toLocaleString() + ' session rows'],
    ['Observation gaps', String(report.agent.observation_gaps ?? 0), (report.agent.clock_rollbacks ?? 0) + ' clock corrections'],
    ['Recovery', supervisor?.state.replaceAll('_',' ') ?? 'Unavailable', (supervisor?.restart_count ?? 0) + ' agent launches this session'],
    ['AI worker', report.optional_workers.auto_classification_enabled ? 'Enabled' : 'Disabled', report.optional_workers.lite_mode ? 'Lite scheduling' : 'Standard scheduling'],
  ] : [];

  return <article className="surface padded diagnostics-card" aria-label="Resource and connection health">
    <div className="panel-header">
      <div><h3>Resource &amp; connection health</h3>
        <p>{readonly ? 'Connect to the local Windows agent for live measurements.' : 'Live samples while Diagnostics is open.'}</p></div>
      <div className="release-actions">
        <button type="button" className="secondary-btn" onClick={() => void refresh()} disabled={readonly || busy}>Refresh</button>
        <button type="button" className="secondary-btn" onClick={exportDiagnostics} disabled={readonly || !report}>Export diagnostics</button>
      </div>
    </div>
    {error && <p className="advanced-feedback" role="status">{error}{report ? ' Showing the last agent report.' : ''}</p>}
    <div className="diagnostic-charts"><ResourceChart samples={samples} metric="memory"/><ResourceChart samples={samples} metric="cpu"/></div>
    {rows.length > 0 && <div className="diagnostic-table-wrap">
      <table className="diagnostic-table">
        <caption className="visually-hidden">Local component health</caption>
        <thead><tr><th scope="col">Component</th><th scope="col">Status</th><th scope="col">Details</th></tr></thead>
        <tbody>{rows.map(([label,status,detail]) => <tr key={label}><th scope="row">{label}</th><td>{status}</td><td>{detail}</td></tr>)}</tbody>
      </table>
    </div>}
  </article>;
}
