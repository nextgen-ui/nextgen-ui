import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface AppConfig {
  theme: string;
  shell: string;
  fontSize: number;
  statsWidth: number;
  audioEnabled: boolean;
  audioVolume: number;
}

export const defaults: AppConfig = {
  theme: 'default',
  shell: '',
  fontSize: 14,
  statsWidth: 230,
  audioEnabled: true,
  audioVolume: 0.6,
};

export const appConfig = writable<AppConfig>(defaults);

export async function loadConfig(): Promise<AppConfig> {
  if (typeof window === 'undefined') return defaults;
  try {
    const cfg = await invoke<AppConfig>('get_config');
    const merged: AppConfig = { ...defaults, ...cfg };
    appConfig.set(merged);
    return merged;
  } catch {
    return defaults;
  }
}

export async function saveConfig(cfg: AppConfig): Promise<boolean> {
  if (typeof window === 'undefined') return false;
  try {
    await invoke('save_config', { config: cfg });
    appConfig.set(cfg);
    return true;
  } catch {
    return false;
  }
}
