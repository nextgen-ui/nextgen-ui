import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface AppConfig {
  theme: string;
  shell: string;
  fontSize: number;
  statsWidth: number;
}

const defaults: AppConfig = { theme: 'default', shell: '', fontSize: 14, statsWidth: 230 };
export const appConfig = writable<AppConfig>(defaults);

export async function loadConfig() {
  try {
    const cfg = await invoke<AppConfig>('get_config');
    appConfig.set(cfg);
    return cfg;
  } catch { return defaults; }
}

export async function saveConfig(cfg: AppConfig) {
  try { await invoke('save_config', { config: cfg }); } catch {}
}
