import {useState} from 'react';

type Props = {
  value: string;
  onChange: (event: {target: {value: string}}) => void;
  'aria-label'?: string;
  required?: boolean;
};

function normalize(value: string) {
  const parts = value.match(/^(\d{1,2}):(\d{1,2})$/);
  if (!parts || +parts[1] >= 24 || +parts[2] >= 60) return null;
  return parts[1].padStart(2, '0') + ':' + parts[2].padStart(2, '0');
}

export function TimePicker({value, onChange, 'aria-label':label='Time', required=false}: Props) {
  const [draft, setDraft] = useState<string | null>(null);
  const shown = draft ?? value;
  const emit = (next: string) => onChange({target: {value: next}});
  function step(delta: number) {
    const [hour, minute] = (normalize(shown) ?? normalize(value) ?? '00:00').split(':').map(Number);
    const next = (hour * 60 + minute + delta + 1440) % 1440;
    setDraft(null);
    emit(String(Math.floor(next / 60)).padStart(2, '0') + ':' + String(next % 60).padStart(2, '0'));
  }
  return <span className="dd-time dd-field-control">
    <input type="text" inputMode="text" maxLength={5} pattern="([01][0-9]|2[0-3]):[0-5][0-9]"
      placeholder="HH:MM" aria-label={label} autoComplete="off" required={required} value={shown}
      onChange={event => {
        const text = event.currentTarget.value;
        setDraft(text);
        if (text === '' && !required) emit('');
        if (text.length === 5) {
          const time = normalize(text);
          if (time) emit(time);
        }
      }}
      onBlur={() => {
        const time = normalize(shown);
        if (time && time !== value) emit(time);
        setDraft(null);
      }}
      onKeyDown={event => {
        if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
          event.preventDefault();
          step(event.key === 'ArrowUp' ? 5 : -5);
        }
      }}/>
    <span className="dd-stepper-buttons">
      <button type="button" aria-label={'Earlier ' + label} title="Five minutes earlier" onClick={() => step(-5)}>−</button>
      <button type="button" aria-label={'Later ' + label} title="Five minutes later" onClick={() => step(5)}>+</button>
      {!required && shown && <button type="button" aria-label={'Clear ' + label} onClick={() => {setDraft(null);emit('');}}>×</button>}
    </span>
  </span>;
}
