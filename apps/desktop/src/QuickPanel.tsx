import {useId, useState} from 'react';
import type {FormEvent, KeyboardEvent, PointerEventHandler, ReactNode} from 'react';
import {useConfirm} from './ConfirmDialog';
import {Icon} from './icons';
import type {Note, QuickPanelData} from './useQuickPanelData';

const tabs = [
  {id: 'activity', label: 'Activity', icon: 'activity'},
  {id: 'todo', label: 'To-do', icon: 'checklist'},
  {id: 'notes', label: 'Notes', icon: 'note'},
] as const;

function Todos({data}: {data: QuickPanelData}) {
  const [title, setTitle] = useState('');

  function add(event: FormEvent) {
    event.preventDefault();
    const name = title.trim();
    if (!name) return;
    if (data.save(current => ({
      ...current,
      todos: [{id: crypto.randomUUID(), title: name, completed: false}, ...current.todos],
    }))) setTitle('');
  }

  return <>
    <form className="quick-todo-form" onSubmit={add}>
      <input aria-label="New to-do" placeholder="Add a to-do" value={title}
        maxLength={160} required onChange={event => setTitle(event.target.value)} />
      <button className="primary-btn" type="submit" aria-label="Add to-do" disabled={!title.trim()}>
        <Icon name="plus" size={15} /> Add
      </button>
    </form>
    {data.error && <p className="quick-save-error" role="alert">{data.error}</p>}
    <div className="quick-entry-scroll">
      {data.todos.length ? <ul className="quick-entry-list" aria-label="To-dos">
        {data.todos.map(todo => <li className="quick-todo-row" key={todo.id}>
          <label className={todo.completed ? 'quick-todo-done' : undefined}>
            <input type="checkbox" checked={todo.completed} onChange={event => {
              const completed = event.target.checked;
              data.save(current => ({
                ...current,
                todos: current.todos.map(item => item.id === todo.id ? {...item, completed} : item),
              }));
            }} />
            <span>{todo.title}</span>
          </label>
          <button type="button" className="icon-btn quick-remove" title="Remove to-do"
            aria-label={`Remove to-do ${todo.title}`} onClick={() => data.save(current => ({
              ...current,
              todos: current.todos.filter(item => item.id !== todo.id),
            }))}>
            <Icon name="close" size={14} />
          </button>
        </li>)}
      </ul> : <p className="quick-empty">No to-dos yet.</p>}
    </div>
  </>;
}

function Notes({data}: {data: QuickPanelData}) {
  const confirm = useConfirm();
  const [draft, setDraft] = useState<Note | null>(null);

  function save(event: FormEvent) {
    event.preventDefault();
    if (!draft?.title.trim() || !draft.text.trim()) return;
    const title = draft.title.trim();
    if (data.save(current => ({
      ...current,
      notes: draft.id
        ? current.notes.map(note => note.id === draft.id ? {...draft, title} : note)
        : [{...draft, id: crypto.randomUUID(), title}, ...current.notes],
    }))) setDraft(null);
  }

  async function remove(note: Note) {
    const approved = await confirm({
      title: 'Delete note?',
      description: note.title,
      confirmLabel: 'Delete note',
      danger: true,
    });
    if (approved) data.save(current => ({
      ...current,
      notes: current.notes.filter(saved => saved.id !== note.id),
    }));
  }

  if (draft) return <form className="quick-note-form quick-entry-scroll" onSubmit={save}>
    <label>
      Heading
      <input value={draft.title} maxLength={160} required autoFocus
        onChange={event => setDraft({...draft, title: event.target.value})} />
    </label>
    <label className="quick-note-text-field">
      Note text
      <textarea value={draft.text} maxLength={12000} required
        onChange={event => setDraft({...draft, text: event.target.value})} />
    </label>
    {data.error && <p className="quick-save-error" role="alert">{data.error}</p>}
    <div className="quick-note-actions">
      <button className="secondary-btn" type="button" onClick={() => setDraft(null)}>Cancel</button>
      <button className="primary-btn" type="submit" disabled={!draft.title.trim() || !draft.text.trim()}>
        Save note
      </button>
    </div>
  </form>;

  return <>
    <div className="quick-notes-toolbar">
      <button className="secondary-btn" type="button" onClick={() => setDraft({id: '', title: '', text: ''})}>
        <Icon name="plus" size={15} /> New note
      </button>
    </div>
    {data.error && <p className="quick-save-error" role="alert">{data.error}</p>}
    <div className="quick-entry-scroll">
      {data.notes.length ? <ul className="quick-entry-list" aria-label="Saved notes">
        {data.notes.map(note => <li key={note.id}>
          <details className="quick-note">
            <summary><Icon name="note" size={16} /><span>{note.title}</span><Icon name="chevron" size={14} /></summary>
            <p>{note.text}</p>
            <div className="quick-note-actions">
              <button className="secondary-btn" type="button" onClick={() => setDraft(note)}>Edit</button>
              <button className="secondary-btn" type="button" onClick={() => void remove(note)}>Delete</button>
            </div>
          </details>
        </li>)}
      </ul> : <p className="quick-empty">No notes yet.</p>}
    </div>
  </>;
}

type QuickPanelProps = {
  data: QuickPanelData;
  presentationPrivacy: boolean;
  onOpenWorkspace: () => void;
  onCollapse: () => void;
  onDrag: PointerEventHandler<HTMLDivElement>;
  children: ReactNode;
};

export function QuickPanel({data, presentationPrivacy, onOpenWorkspace, onCollapse, onDrag, children}: QuickPanelProps) {
  const id = useId();
  const [selected, setSelected] = useState<typeof tabs[number]['id']>('activity');
  const active = presentationPrivacy ? 'activity' : selected;

  function navigateTabs(event: KeyboardEvent<HTMLButtonElement>, index: number) {
    const key = event.key;
    if (key !== 'ArrowLeft' && key !== 'ArrowRight' && key !== 'Home' && key !== 'End') return;
    const next = {
      ArrowLeft: (index + tabs.length - 1) % tabs.length,
      ArrowRight: (index + 1) % tabs.length,
      Home: 0,
      End: tabs.length - 1,
    }[key];
    event.preventDefault();
    setSelected(tabs[next].id);
    event.currentTarget.parentElement?.querySelectorAll('button')[next]?.focus();
  }

  return <div className="quick-surface" onPointerDownCapture={onDrag}>
    <div className="quick-head">
      <div className="quick-brand"><span className="brand-symbol">◐</span><span>Digital Discipline</span></div>
      <div className="quick-window-controls">
        <button type="button" className="icon-btn window-control" title="Maximize workspace"
          aria-label="Maximize workspace" onClick={onOpenWorkspace}><Icon name="maximize" /></button>
        <button type="button" className="icon-btn window-control window-control-close"
          title="Collapse to pill" aria-label="Collapse to pill" onClick={onCollapse}><Icon name="close" /></button>
      </div>
    </div>
    {!presentationPrivacy && <div className="quick-tabs" role="tablist" aria-label="Quick panel">
      {tabs.map((tab, index) => <button key={tab.id} type="button" role="tab"
        id={`${id}-tab-${tab.id}`} aria-controls={`${id}-panel-${tab.id}`}
        aria-selected={active === tab.id} tabIndex={active === tab.id ? 0 : -1}
        onClick={() => setSelected(tab.id)} onKeyDown={event => navigateTabs(event, index)}>
        <Icon name={tab.icon} size={15} /> {tab.label}
      </button>)}
    </div>}
    <div className="quick-tab-panel quick-activity" id={`${id}-panel-activity`}
      role={presentationPrivacy ? undefined : 'tabpanel'}
      aria-labelledby={presentationPrivacy ? undefined : `${id}-tab-activity`}
      hidden={active !== 'activity'}>
      {active === 'activity' && children}
    </div>
    {!presentationPrivacy && <>
      <div className="quick-tab-panel quick-tools" id={`${id}-panel-todo`} role="tabpanel"
        aria-labelledby={`${id}-tab-todo`} hidden={active !== 'todo'} data-quick-panel-tools>
        <Todos data={data} />
      </div>
      <div className="quick-tab-panel quick-tools" id={`${id}-panel-notes`} role="tabpanel"
        aria-labelledby={`${id}-tab-notes`} hidden={active !== 'notes'} data-quick-panel-tools>
        <Notes data={data} />
      </div>
    </>}
  </div>;
}
