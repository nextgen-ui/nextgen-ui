import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// ── PTY ──────────────────────────────────────────────────────────────────────

export async function ptySpawn(sessionId: string, cols: number, rows: number) {
  return invoke<void>('pty_spawn', { sessionId, cols, rows });
}
export async function ptyWrite(sessionId: string, data: string) {
  return invoke<void>('pty_write', { sessionId, data });
}
export async function ptyResize(sessionId: string, cols: number, rows: number) {
  return invoke<void>('pty_resize', { sessionId, cols, rows });
}
export async function ptyClose(sessionId: string) {
  return invoke<void>('pty_close', { sessionId });
}
export async function onPtyOutput(
  callback: (payload: { session_id: string; data: string }) => void
) {
  return listen<{ session_id: string; data: string }>('pty_output', (e) => callback(e.payload));
}

// ── Stats ─────────────────────────────────────────────────────────────────────

export interface SystemStats {
  cpu_usage: number;
  cpu_per_core: number[];
  mem_used: number;
  mem_total: number;
  swap_used: number;
  swap_total: number;
  net_in: number;
  net_out: number;
  disk_read: number;
  disk_write: number;
}
export async function getSystemStats() {
  return invoke<SystemStats>('get_system_stats');
}
export async function onSystemStats(callback: (stats: SystemStats) => void) {
  return listen<SystemStats>('system_stats', (e) => callback(e.payload));
}

// ── Processes ─────────────────────────────────────────────────────────────────

export interface ProcessInfo {
  pid: number;
  name: string;
  cpu: number;
  mem: number;
  status: string;
}
export async function onProcessList(callback: (procs: ProcessInfo[]) => void) {
  return listen<ProcessInfo[]>('process_list', (e) => callback(e.payload));
}
export async function killProcess(pid: number) {
  return invoke<void>('kill_process', { pid });
}
/**
 * Tell the backend whether anything is displaying the process list. Walking
 * every process dominates the sampler's cost, so it is skipped while off.
 */
export async function setProcessStream(enabled: boolean) {
  return invoke<void>('set_process_stream', { enabled });
}
