import {useId, useState} from 'react';

export function NumberField({label, value, min, max, onChange}: {
  label: string;
  value: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
}) {
  const id = useId();
  const [draft, setDraft] = useState<string | null>(null);
  function step(delta: number) {
    setDraft(null);
    onChange(Math.min(max, Math.max(min, value + delta)));
  }
  return <div className="dd-field">
    <label htmlFor={id}>{label}</label>
    <div className="dd-field-control dd-stepper">
      <input id={id} type="number" inputMode={min < 0 ? 'text' : 'numeric'}
        min={min} max={max} step={1} required value={draft ?? value}
        onChange={event => {
          const text = event.currentTarget.value;
          setDraft(text);
          if (text && Number.isFinite(event.currentTarget.valueAsNumber)) {
            onChange(event.currentTarget.valueAsNumber);
          }
        }}
        onBlur={() => setDraft(null)}/>
      <div className="dd-stepper-buttons">
        <button type="button" aria-label={'Decrease ' + label} disabled={value <= min} onClick={() => step(-1)}>−</button>
        <button type="button" aria-label={'Increase ' + label} disabled={value >= max} onClick={() => step(1)}>+</button>
      </div>
    </div>
  </div>;
}

export function TokenField({label, values, onChange, placeholder}: {
  label: string;
  values: string[];
  onChange: (values: string[]) => void;
  placeholder: string;
}) {
  const id = useId();
  const [draft, setDraft] = useState('');
  function add() {
    const additions = draft.split(/[\s,;]+/).map(value => value.trim().toLowerCase()).filter(Boolean);
    if (additions.length) {
      onChange([...new Set([...values, ...additions])]);
      setDraft('');
    }
  }
  return <div className="dd-field">
    <label htmlFor={id}>{label}</label>
    <div className="dd-field-control dd-token-entry">
      <input id={id} value={draft} placeholder={placeholder} onChange={event => setDraft(event.target.value)}
        onBlur={add} onKeyDown={event => {
          if (event.key === 'Enter' || event.key === ',') {
            event.preventDefault();
            add();
          }
        }}/>
      <button className="text-action" type="button" aria-label={'Add to ' + label} disabled={!draft.trim()} onClick={add}>Add</button>
    </div>
    {values.length > 0 && <ul className="dd-token-list" aria-label={label + ' saved values'}>
      {values.map(value => <li key={value}><span>{value}</span>
        <button type="button" aria-label={'Remove ' + value} onClick={() => onChange(values.filter(item => item !== value))}>×</button>
      </li>)}
    </ul>}
  </div>;
}
