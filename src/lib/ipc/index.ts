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
export interface TerminalInputState {
  /** Terminal is echoing typed characters itself. */
  echo: boolean;
  /** Canonical (line) mode — off for anything reading key by key. */
  canonical: boolean;
  /** The shell owns the terminal, i.e. the user is at a prompt. */
  at_shell_prompt: boolean;
  /** Foreground program name, when it can be determined. */
  foreground: string | null;
  /** Canonical-mode secret read (getpass, `read -s`). True for exactly as long
   *  as the read lasts, so it can gate display directly. */
  canonical_secret: boolean;
  /** A secret-reading program owns the terminal in raw mode (sudo pwfeedback).
   *  Stays true for the whole session after `sudo su` authenticates, so it marks
   *  that a read *started*, not that one is still in progress. */
  program_secret: boolean;
}

/** Inspect what the terminal is doing with typed input. */
export async function ptyInputState(sessionId: string) {
  return invoke<TerminalInputState>('pty_input_state', { sessionId });
}
export async function onPtyOutput(
  callback: (payload: { session_id: string; data: string }) => void
) {
  return listen<{ session_id: string; data: string }>('pty_output', (e) => callback(e.payload));
}

/** A shell exited on its own (`exit`, Ctrl+D). Not sent for pty_close. */
export async function onPtyExit(callback: (payload: { session_id: string }) => void) {
  return listen<{ session_id: string }>('pty_exit', (e) => callback(e.payload));
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

// ── Network & Geolocation ───────────────────────────────────────────────────

export interface LocalNetInfo {
  local_ip: string | null;
  interface_name: string | null;
  ping_ms: number | null;
}

export async function getLocalNetInfo() {
  return invoke<LocalNetInfo>('get_local_net_info');
}

export async function measurePing() {
  return invoke<number | null>('measure_ping');
}

