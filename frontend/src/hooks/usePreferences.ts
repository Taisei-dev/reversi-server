import { useCallback, useState } from 'react';
import type { GamePreferences } from '../types';

const STORAGE_KEY = 'reversi:preferences';

export const DEFAULT_PREFERENCES: GamePreferences = {
  playerName: 'Player_Web',
  color: 'black',
  aiType: 'egaroucid',
  aiLevel: 7,
  aiUseBook: true,
};

// 保存値は信用できないため、フィールドごとに検証してデフォルトで補完する
const parsePreferences = (raw: unknown): GamePreferences => {
  if (typeof raw !== 'object' || raw === null) return DEFAULT_PREFERENCES;
  const v = raw as Record<string, unknown>;
  return {
    playerName:
      typeof v.playerName === 'string' && v.playerName !== ''
        ? v.playerName
        : DEFAULT_PREFERENCES.playerName,
    color: v.color === 'white' ? 'white' : 'black',
    aiType: v.aiType === 'random' ? 'random' : 'egaroucid',
    aiLevel:
      typeof v.aiLevel === 'number' && Number.isInteger(v.aiLevel) && v.aiLevel >= 0 && v.aiLevel <= 20
        ? v.aiLevel
        : DEFAULT_PREFERENCES.aiLevel,
    aiUseBook: typeof v.aiUseBook === 'boolean' ? v.aiUseBook : DEFAULT_PREFERENCES.aiUseBook,
  };
};

const loadPreferences = (): GamePreferences => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored === null ? DEFAULT_PREFERENCES : parsePreferences(JSON.parse(stored));
  } catch {
    return DEFAULT_PREFERENCES;
  }
};

const savePreferences = (prefs: GamePreferences) => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
  } catch {
    // 保存できない環境ではメモリ上の設定だけで動作させる
  }
};

export const usePreferences = () => {
  const [prefs, setPrefs] = useState(loadPreferences);

  const update = useCallback((patch: Partial<GamePreferences>) => {
    setPrefs((prev) => {
      const next = { ...prev, ...patch };
      savePreferences(next);
      return next;
    });
  }, []);

  return { prefs, update };
};
