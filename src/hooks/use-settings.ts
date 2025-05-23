import { LazyStore } from '@tauri-apps/plugin-store';

export interface Settings {
  port?: number;
}

const SETTINGS_FILE_NAME = 'settings.json';

const store = new LazyStore(SETTINGS_FILE_NAME, { autoSave: true });

export function useSettings() {
  const getSettings = async <T extends keyof Settings>(key: T) => {
    const data = await store.get<Settings[T]>(key);
    return data;
  };

  const setSettings = async <T extends keyof Settings>(
    key: T,
    value: Settings[T],
  ) => {
    await store.set(key, value);
  };

  return {
    getSettings,
    setSettings,
  };
}
