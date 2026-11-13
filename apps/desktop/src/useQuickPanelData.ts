import {useState} from 'react';

export type Todo = {id: string; title: string; completed: boolean};
export type Note = {id: string; title: string; text: string};
type Entries = {todos: Todo[]; notes: Note[]};
const key = 'dd:quick-panel-v1';
const empty: Entries = {todos: [], notes: []};

function readEntries(): Entries {
  const saved = localStorage.getItem(key);
  if (saved === null) return empty;
  const entries = JSON.parse(saved) as Partial<Entries> | null;
  if (
    !Array.isArray(entries?.todos) ||
    !entries.todos.every(todo => typeof todo?.id === 'string' &&
      typeof todo.title === 'string' && typeof todo.completed === 'boolean') ||
    !Array.isArray(entries.notes) ||
    !entries.notes.every(note => typeof note?.id === 'string' &&
      typeof note.title === 'string' && typeof note.text === 'string')
  ) {
    throw Error('Invalid quick panel data');
  }
  return {todos: entries.todos, notes: entries.notes};
}

export function useQuickPanelData() {
  const [state, setState] = useState(() => {
    try {
      return {entries: readEntries(), error: ''};
    } catch {
      return {entries: empty, error: 'Saved to-dos and notes could not be read.'};
    }
  });

  function save(change: (entries: Entries) => Entries) {
    try {
      // Read the latest saved state before applying a change from the panel.
      const entries = change(readEntries());
      localStorage.setItem(key, JSON.stringify(entries));
      setState({entries, error: ''});
      return true;
    } catch {
      setState(current => ({
        ...current,
        error: 'Could not save changes to your to-dos and notes.',
      }));
      return false;
    }
  }

  return {...state.entries, error: state.error, save};
}

export type QuickPanelData = ReturnType<typeof useQuickPanelData>;
