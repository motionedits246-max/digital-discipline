import {useCallback, useEffect, useRef, useState} from 'react';
import {agent} from './api';
import {CustomSelect} from './CustomSelect';

type Profile = {
  extension_id:string; profile_id:string; browser:string; online:boolean; authorized:boolean;
  recording_enabled:boolean|null; queued:number; adapter:string; qualified_at_ms:number; accepted_at_ms:number;
  rendered_ms:number; policy_ack_ms:number; companion_version:string; loss_count:number; last_seen_ms:number;
  dashboard_received:boolean; policy_acknowledged:boolean; accepted_events:number;
};
type Report = {
  profiles:Profile[]; checked_at_ms:number; play_threshold_ms:number; native_attribution:string;
  agent_version?:string; expected_companion_version?:string;
};

const keyFor = (profile:Profile) => profile.extension_id + ':' + profile.profile_id;
const nameFor = (profile:Profile) => profile.browser.replace(/^./, letter => letter.toUpperCase()) + ' · ' + profile.profile_id.slice(0, 8);
const at = (value:number) => value > 0 ? new Date(value).toLocaleTimeString() : '—';
const connection = (profile:Profile) => !profile.authorized ? 'Disconnected' : profile.online ? 'Online' : 'Offline';
const stages = (p:Profile) => [
  ['Authorized', p.authorized], ['Online', p.online], ['Recording', p.recording_enabled === true],
  ['Player ready', p.adapter === 'ready'], ['Play qualified', p.qualified_at_ms > 0],
  ['Agent accepted', p.accepted_at_ms > 0], ['Dashboard received', p.dashboard_received],
] as const;

function useBrowserHealth(readonly:boolean, interval:number) {
  const [report, setReport] = useState<Report | null>(null);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const generation = useRef(0);
  const running = useRef(false);
  const refresh = useCallback(async () => {
    if (readonly || running.current) return;
    running.current = true;
    const request = ++generation.current;
    setBusy(true);
    try {
      const data = await agent<Report>('GET', '/tracking/doctor');
      if (!Array.isArray(data?.profiles)) throw Error('Browser profiles unavailable.');
      if (request === generation.current) {setReport(data);setError('');}
    } catch {
      if (request === generation.current) setError('Browser profiles are unavailable from the local agent.');
    } finally {
      if (request === generation.current) {running.current = false;setBusy(false);}
    }
  }, [readonly]);
  useEffect(() => {
    if (readonly) return;
    void refresh();
    const timer = interval ? window.setInterval(() => void refresh(), interval) : undefined;
    return () => {window.clearInterval(timer);generation.current++;running.current = false;};
  }, [interval, readonly, refresh]);
  return {report, error, busy, refresh};
}

export function BrowserProfileConnections({readonly}: {readonly:boolean}) {
  const {report, error, busy, refresh} = useBrowserHealth(readonly, 0);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState('');
  async function access(profile:Profile) {
    setSaving(true);
    setMessage('');
    try {
      await agent('POST', '/tracking/profiles/access', {
        extension_id:profile.extension_id, profile_id:profile.profile_id, authorized:!profile.authorized,
      });
      await refresh();
      setMessage(profile.authorized ? 'Browser profile disconnected.' : 'Browser profile reconnected.');
    } catch {setMessage('Could not change this profile connection.');}
    finally {setSaving(false);}
  }
  return <section className="surface padded settings-card" aria-label="Browser profile access">
    <div className="panel-header"><div><h3>Browser profile access</h3><p>Manage access for an individual browser profile.</p></div>
      <button type="button" className="secondary-btn" disabled={readonly || busy || saving} onClick={() => void refresh()}>Refresh profiles</button></div>
    {report?.profiles.map(profile => <div key={keyFor(profile)} className="settings-row">
      <div><strong>{nameFor(profile)}</strong><small>{profile.authorized ? 'Authorized' : 'Disconnected'}</small></div>
      <button type="button" className="secondary-btn" disabled={readonly || busy || saving} onClick={() => void access(profile)}>
        {profile.authorized ? 'Disconnect this profile' : 'Reconnect this profile'}
      </button>
    </div>)}
    {(message || error) && <p className="form-hint" role="status">{message || error}</p>}
    {report && !report.profiles.length && <p className="form-hint">Browser profiles appear after their first connection.</p>}
  </section>;
}

function advice(profile:Profile, expected?:string) {
  if (!profile.authorized) return 'Reconnect this profile in Settings & Privacy → Browser.';
  if (!profile.online) return 'Open this profile and use Connect & sync in the companion.';
  if (profile.adapter === 'private_session') return 'Recording resumes when the private session ends.';
  if (expected && profile.companion_version !== expected) return 'Check the companion version and reload its folder after updating.';
  if (profile.recording_enabled !== true) return 'Enable recording in the companion to collect activity.';
  if (profile.adapter !== 'ready') return 'Open a supported YouTube or Instagram video in this profile.';
  if (!profile.qualified_at_ms) return 'Let a supported video play visibly for at least two seconds.';
  if (!profile.accepted_at_ms) return 'Check the pending queue and sync again.';
  if (!profile.dashboard_received) return 'Open Overview or Video Intelligence to render the latest accepted event.';
  return 'The dashboard received an observed play from this profile.';
}

export function TrackingDoctor({readonly, lite=false}: {readonly:boolean;lite?:boolean}) {
  const {report, error, busy, refresh} = useBrowserHealth(readonly, lite ? 15000 : 5000);
  const [selected, setSelected] = useState('');
  const profiles = report?.profiles ?? [];
  const profile = profiles.find(p => keyFor(p) === selected) ?? profiles[0];
  const largestQueue = Math.max(1, ...profiles.map(p => p.queued));
  const charts = profiles.slice(0, 8);

  return <article className="surface padded diagnostics-card tracking-doctor" id="tracking-doctor" aria-label="Tracking Doctor">
    <div className="panel-header">
      <div><h3>Tracking Doctor</h3><p>Browser companion delivery, recording and restriction acknowledgments.</p></div>
      <button type="button" className="secondary-btn" disabled={readonly || busy} onClick={() => void refresh()}>Check now</button>
    </div>
    {report?.expected_companion_version && <p className="form-hint">
      Agent {report.agent_version ?? '—'} · companion {report.expected_companion_version} · checked {at(report.checked_at_ms)}
    </p>}
    {error && <p className="advanced-feedback" role="status">{error}{report ? ' Showing the last profile report.' : ''}</p>}
    {!error && !profiles.length && <p className="form-hint">{readonly ? 'Live checks require the local Windows agent.' : report ? 'No browser profiles have connected yet.' : 'Checking browser profiles…'}</p>}
    {profiles.length > 0 && <>
      <div className="diagnostic-charts">
        <figure className="diagnostic-chart">
          <figcaption><h4>Confirmed observation stages</h4></figcaption>
          <ul className="profile-bars">{charts.map(p => {
            const completed = stages(p).filter(([, confirmed]) => confirmed).length;
            return <li key={keyFor(p)}><span>{nameFor(p)}</span><meter min={0} max={7} value={completed} aria-label={nameFor(p) + ' confirmed stages'}/><strong>{completed}/7</strong></li>;
          })}</ul>
        </figure>
        <figure className="diagnostic-chart">
          <figcaption><h4>Pending events by profile</h4></figcaption>
          <ul className="profile-bars">{charts.map(p => <li key={keyFor(p)}>
            <span>{nameFor(p)}</span><meter min={0} max={largestQueue} value={p.queued} aria-label={nameFor(p) + ' pending events'}/><strong>{p.queued.toLocaleString()}</strong>
          </li>)}</ul>
        </figure>
      </div>
      {profiles.length > charts.length && <p className="form-hint">Charts show the first {charts.length} profiles. All profiles appear in the table.</p>}
      <div className="diagnostic-table-wrap">
        <table className="diagnostic-table profile-table">
          <caption className="visually-hidden">Browser companion response by profile</caption>
          <thead><tr>{['Profile','Connection','Recording','Companion','Pending','Accepted','Lost','Last response','Rule acknowledgment'].map(label => <th scope="col" key={label}>{label}</th>)}</tr></thead>
          <tbody>{profiles.map(p => <tr key={keyFor(p)}>
            <th scope="row">{nameFor(p)}</th><td><span className="status-text" data-online={p.online && p.authorized}>{connection(p)}</span></td>
            <td>{p.recording_enabled == null ? 'Unknown' : p.recording_enabled ? 'On' : 'Off'}</td>
            <td>{p.companion_version || '—'}</td><td>{p.queued.toLocaleString()}</td><td>{p.accepted_events.toLocaleString()}</td>
            <td>{p.loss_count.toLocaleString()}</td><td>{at(p.last_seen_ms)}</td><td>{at(p.policy_ack_ms)}</td>
          </tr>)}</tbody>
        </table>
      </div>
      {profile && <section className="doctor-profile">
        <div className="panel-header"><h4>Profile observation details</h4>
          <CustomSelect aria-label="Inspect browser profile" value={keyFor(profile)} onChange={event => setSelected(event.target.value)}>
            {profiles.map(p => <option key={keyFor(p)} value={keyFor(p)}>{nameFor(p)}</option>)}
          </CustomSelect>
        </div>
        <ol className="doctor-stages">{stages(profile).map(([name, confirmed]) => <li className={confirmed ? 'confirmed' : 'pending'} key={name}>
          <span className="doctor-dot"/>{name}<small>{confirmed ? 'Confirmed' : 'Awaiting signal'}</small>
        </li>)}</ol>
        <p className="form-hint">{advice(profile, report?.expected_companion_version)}</p>
        <div className="doctor-details"><span>Last qualified play <strong>{at(profile.qualified_at_ms)}</strong></span>
          <span>Last accepted event <strong>{at(profile.accepted_at_ms)}</strong></span>
          <span>Last dashboard receipt <strong>{at(profile.rendered_ms)}</strong></span></div>
      </section>}
    </>}
    {report?.native_attribution && <p className="form-hint">{report.native_attribution}</p>}
  </article>;
}
