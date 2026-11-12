import {useCallback, useEffect, useRef, useState} from 'react';
import {agent} from './api';

export const DEFAULT_CATEGORIES = [
  'Development', 'Design', 'Productivity', 'Communication', 'Education', 'Entertainment',
  'Gaming', 'Social', 'News', 'Utility', 'System', 'Browser', 'Unclassified',
];

type CategoryReport = {
  categories: string[];
  custom_categories?: string[];
  distraction_categories?: string[];
  overrides: {subject:string;category:string}[];
};

export function useCategoryLabels(enabled: boolean) {
  const [report, setReport] = useState<CategoryReport | null>(null);
  const [error, setError] = useState('');
  const generation = useRef(0);
  const reload = useCallback(async () => {
    if (!enabled) return;
    const current = ++generation.current;
    try {
      const data = await agent<CategoryReport>('GET', '/categories');
      if (!Array.isArray(data.categories) || !Array.isArray(data.overrides)) throw Error('Category settings are unavailable.');
      if (current === generation.current) {
        setReport(data);
        setError('');
      }
    } catch (error) {
      if (current === generation.current) setError(String(error));
    }
  }, [enabled]);
  useEffect(() => {
    void reload();
    return () => { generation.current++; };
  }, [reload]);

  return {report, categories: report?.categories ?? DEFAULT_CATEGORIES, error, reload};
}
