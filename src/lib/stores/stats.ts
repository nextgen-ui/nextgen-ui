import { writable, derived } from 'svelte/store';
import type { SystemStats } from '$lib/ipc';

export const HISTORY_LEN = 60; // 60 seconds of history

export interface StatsSnapshot {
  stats: SystemStats;
  cpuHistory: number[];
  netInHistory: number[];
  netOutHistory: number[];
  memHistory: number[];
}

const initialStats: SystemStats = {
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
};

// Internal mutable buffers to avoid GC pressure
const cpuRing: number[] = [];
const netInRing: number[] = [];
const netOutRing: number[] = [];
const memRing: number[] = [];

function pushToRing(ring: number[], val: number): number[] {
  if (ring.length < HISTORY_LEN) {
    ring.push(val);
  } else {
    ring.copyWithin(0, 1);
    ring[HISTORY_LEN - 1] = val;
  }
  return [...ring];
}

// Single consolidated store for atomic updates
export const allStats = writable<StatsSnapshot>({
  stats: initialStats,
  cpuHistory: [],
  netInHistory: [],
  netOutHistory: [],
  memHistory: [],
});

// Derived stores for backward compatibility
export const systemStats = derived(allStats, ($s) => $s.stats);
export const cpuHistory = derived(allStats, ($s) => $s.cpuHistory);
export const netInHistory = derived(allStats, ($s) => $s.netInHistory);
export const netOutHistory = derived(allStats, ($s) => $s.netOutHistory);
export const memHistory = derived(allStats, ($s) => $s.memHistory);

export function updateStats(s: SystemStats) {
  const cpuHist = pushToRing(cpuRing, s.cpu_usage);
  const netInHist = pushToRing(netInRing, s.net_in / 1024);   // KB/s
  const netOutHist = pushToRing(netOutRing, s.net_out / 1024);
  const memHist = pushToRing(memRing, s.mem_total > 0 ? (s.mem_used / s.mem_total) * 100 : 0);

  allStats.set({
    stats: s,
    cpuHistory: cpuHist,
    netInHistory: netInHist,
    netOutHistory: netOutHist,
    memHistory: memHist,
  });
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
