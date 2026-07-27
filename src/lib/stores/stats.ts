import { writable } from 'svelte/store';
import type { SystemStats } from '$lib/ipc';

export const systemStats = writable<SystemStats>({
  cpu_usage: 0,
  cpu_per_core: [],
  mem_used: 0,
  mem_total: 1,
  swap_used: 0,
  swap_total: 1
});

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

export function formatPercent(value: number): string {
  return `${value.toFixed(1)}%`;
}
