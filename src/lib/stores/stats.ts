import { writable, derived } from 'svelte/store';
import type { SystemStats } from '$lib/ipc';

const HISTORY_LEN = 60; // 60 seconds of history

export const systemStats = writable<SystemStats>({
  cpu_usage: 0,
  cpu_per_core: [],
  mem_used: 0,
  mem_total: 1,
  swap_used: 0,
  swap_total: 1,
  net_in: 0,
  net_out: 0,
  disk_read: 0,
  disk_write: 0,
});

// History rings
export const cpuHistory   = writable<number[]>([]);
export const netInHistory  = writable<number[]>([]);
export const netOutHistory = writable<number[]>([]);
export const memHistory   = writable<number[]>([]);

function pushHistory(store: ReturnType<typeof writable<number[]>>, val: number) {
  store.update((arr) => {
    const next = [...arr, val];
    return next.length > HISTORY_LEN ? next.slice(next.length - HISTORY_LEN) : next;
  });
}

export function updateStats(s: SystemStats) {
  systemStats.set(s);
  pushHistory(cpuHistory,   s.cpu_usage);
  pushHistory(netInHistory,  s.net_in / 1024);   // KB/s
  pushHistory(netOutHistory, s.net_out / 1024);
  pushHistory(memHistory,   (s.mem_used / s.mem_total) * 100);
}

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
