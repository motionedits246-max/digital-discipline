import {useState} from 'react';
import type {FormEvent} from 'react';
import {agent} from './api';
import {useCategoryLabels} from './categoryLabels';
import {CustomSelect} from './CustomSelect';
import type {Session} from './types';

function NewLabel({distraction, disabled, onAdd}: {
  distraction: boolean;
  disabled: boolean;
  onAdd: (name:string, distraction:boolean) => Promise<boolean>;
}) {
  const [name, setName] = useState('');
  async function submit(event:FormEvent) {
    event.preventDefault();
    if (name.trim() && await onAdd(name, distraction)) setName('');
  }
  return <form className="label-create" onSubmit={submit}>
    <label>{distraction ? 'New distraction label' : 'New category label'}
      <input value={name} required maxLength={40} disabled={disabled} onChange={event => setName(event.target.value)}
        placeholder={distraction ? 'e.g. Feed scrolling' : 'e.g. Client editing'}/>
    </label>
    <button className="primary-btn" disabled={disabled || !name.trim()}>{distraction ? 'Create distraction' : 'Create label'}</button>
  </form>;
}

export function CategorySettings({sessions, readonly, onUpdate}: {
  sessions: Session[];
  readonly: boolean;
  onUpdate: () => void;
}) {
  const {report, categories, error, reload} = useCategoryLabels(!readonly);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');
  const [kind, setKind] = useState<'app' | 'domain'>('app');
  const [target, setTarget] = useState('');
  const [category, setCategory] = useState('Productivity');
  const selected = report?.distraction_categories ?? [];
  const disabled = readonly || busy || !report || !!error;
  const recent = [...new Map(sessions.filter(s => !s.idle).map(s => [
    s.domain ? 'domain:' + s.domain : 'app:' + s.app, s,
  ])).entries()].slice(0, 25);

  async function change(action:()=>Promise<unknown>, success:string) {
    if (disabled) return false;
    setBusy(true);
    setMessage('');
    try {
      await action();
      await reload();
      setMessage(success);
      onUpdate();
      return true;
    } catch (error) {
      setMessage(String(error));
      return false;
    } finally {
      setBusy(false);
    }
  }

  async function add(name: string, distraction: boolean) {
    let created = '';
    const saved = await change(async () => {
      const result = await agent<{label:string}>('POST', '/categories/labels', {name, distraction});
      created = result.label;
    }, distraction ? 'Distraction label created and selected.' : 'Category label created.');
    if (saved && created) setCategory(created);
    return saved;
  }

  function chooseRecent(value: string) {
    const entry = recent.find(([key]) => key === value);
    if (!entry) return;
    const [key, session] = entry;
    setKind(key.startsWith('domain:') ? 'domain' : 'app');
    setTarget(key.slice(key.indexOf(':') + 1));
    if (categories.includes(session.category)) setCategory(session.category);
  }

  function toggleDistraction(name: string) {
    const next = selected.includes(name) ? selected.filter(label => label !== name) : [...selected, name];
    return change(() => agent('POST', '/settings', {distraction_categories: next}), 'Distraction choices saved.');
  }

  return <div className="settings-card-stack">
    {(message || error) && <p className="advanced-feedback" role="status">{message || error}</p>}
    {readonly && <p className="form-hint">Connect to the local agent to change labels.</p>}
    <section className="surface padded settings-card">
      <h3>Category labels</h3>
      <p className="form-hint">Use your own labels in detection rules and when correcting activity.</p>
      <NewLabel distraction={false} disabled={disabled} onAdd={add}/>
      {!!report?.custom_categories?.length && <ul className="dd-token-list" aria-label="Custom category labels">
        {report.custom_categories.map(name => <li key={name}>{name}</li>)}
      </ul>}
    </section>
    <section className="surface padded settings-card">
      <h3>App and site detection</h3>
      <form className="release-form" onSubmit={event => {
        event.preventDefault();
        void change(() => agent('POST', '/categories/override', {
          subject:kind + ':' + target.trim().toLowerCase(), category,
        }), 'Detection rule saved.');
      }}>
        {recent.length > 0 && <label>Pick recent activity<CustomSelect aria-label="Pick recent activity" value={kind + ':' + target} onChange={event => chooseRecent(event.target.value)}>
          <option value="">Choose an app or site</option>
          {recent.map(([key]) => <option key={key} value={key}>{key.slice(key.indexOf(':') + 1)}</option>)}
        </CustomSelect></label>}
        <label>Source<CustomSelect aria-label="Source" value={kind} onChange={event => {setKind(event.target.value as 'app' | 'domain');setTarget('');}}>
          <option value="app">Windows app</option><option value="domain">Website</option>
        </CustomSelect></label>
        <label>{kind === 'app' ? 'Executable name' : 'Domain'}
          <input value={target} onChange={event => setTarget(event.target.value)} required maxLength={253}
            placeholder={kind === 'app' ? 'editor.exe' : 'example.com'}/>
        </label>
        <label>Category label<CustomSelect aria-label="Category label" value={category} onChange={event => setCategory(event.target.value)}>
          {categories.map(name => <option key={name}>{name}</option>)}
        </CustomSelect></label>
        <div className="release-actions"><button className="primary-btn" disabled={disabled || !target.trim()}>Save detection rule</button></div>
      </form>
      {!!report?.overrides.length && <div className="release-list">
        {report.overrides.map(rule => <div className="release-row" key={rule.subject}>
          <div><strong>{rule.subject.slice(rule.subject.indexOf(':') + 1)}</strong><p>{rule.category}</p></div>
          <button type="button" className="text-action" disabled={disabled}
            aria-label={'Remove detection rule for ' + rule.subject.slice(rule.subject.indexOf(':') + 1)}
            onClick={() => void change(() => agent('DELETE', '/categories/override/' + rule.subject), 'Detection rule removed.')}>Remove</button>
        </div>)}
      </div>}
    </section>
    <section className="surface padded settings-card">
      <h3>Distraction labels</h3>
      <p className="form-hint">Select the categories that count toward your distraction time.</p>
      <div className="label-options" role="group" aria-label="Choose distraction categories">
        {categories.filter(name => selected.includes(name) || !['Unclassified','System'].includes(name)).map(name =>
          <button key={name} type="button" className="label-choice" aria-pressed={selected.includes(name)}
            disabled={disabled} onClick={() => void toggleDistraction(name)}>{name}</button>)}
      </div>
      <NewLabel distraction disabled={disabled} onAdd={add}/>
    </section>
  </div>;
}
