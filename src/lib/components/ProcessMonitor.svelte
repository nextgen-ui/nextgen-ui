<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { onProcessList, killProcess } from '$lib/ipc';
  import type { ProcessInfo } from '$lib/ipc';
  import { currentTheme } from '$lib/stores/theme';

  let processes: ProcessInfo[] = [];
  let unlisten: (() => void) | null = null;
  let sortBy: 'cpu' | 'mem' = 'cpu';
  let confirmKill: number | null = null;
  let theme = $currentTheme;
  const unsubTheme = currentTheme.subscribe((t) => (theme = t));

  $: sorted = [...processes].sort((a, b) =>
    sortBy === 'cpu' ? b.cpu - a.cpu : b.mem - a.mem
  );

  onMount(async () => {
    unlisten = await onProcessList((procs) => { processes = procs; });
  });

  onDestroy(() => { unlisten?.(); unsubTheme(); });

  function formatMem(bytes: number): string {
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function cpuColor(cpu: number): string {
    if (cpu > 50) return theme.red;
    if (cpu > 20) return theme.yellow;
    return theme.fg;
  }

  async function handleKill(pid: number) {
    if (confirmKill === pid) {
      try { await killProcess(pid); } catch {}
      confirmKill = null;
    } else {
      confirmKill = pid;
      setTimeout(() => { confirmKill = null; }, 2000);
    }
  }
</script>

<div class="proc-panel">
  <div class="panel-header">
    <span>⬡ PROCESSES</span>
    <div class="sort-btns">
      <button class="sort-btn" class:active={sortBy === 'cpu'} on:click={() => sortBy = 'cpu'}>CPU</button>
      <button class="sort-btn" class:active={sortBy === 'mem'} on:click={() => sortBy = 'mem'}>MEM</button>
    </div>
  </div>

  <div class="proc-list-header">
    <span class="col-pid">PID</span>
    <span class="col-name">NAME</span>
    <span class="col-cpu">CPU%</span>
    <span class="col-mem">MEM</span>
    <span class="col-act"></span>
  </div>

  <div class="proc-list">
    {#each sorted as proc (proc.pid)}
      <div class="proc-row" class:highlight={proc.cpu > 50}>
        <span class="col-pid dim">{proc.pid}</span>
        <span class="col-name">{proc.name.slice(0, 16)}</span>
        <span class="col-cpu" style="color:{cpuColor(proc.cpu)}">{proc.cpu.toFixed(1)}</span>
        <span class="col-mem dim">{formatMem(proc.mem)}</span>
        <button
          class="kill-btn"
          class:confirm={confirmKill === proc.pid}
          on:click={() => handleKill(proc.pid)}
          title="Kill process"
        >
          {confirmKill === proc.pid ? '!' : '✕'}
        </button>
      </div>
    {/each}
  </div>
</div>

<style>
  .proc-panel {
    display: flex; flex-direction: column;
    height: 100%;
    background: var(--panel-bg, #0d1220cc);
    border: 1px solid var(--panel-border, #00c8ff33);
    border-radius: 6px;
    font-family: 'JetBrains Mono', monospace;
    overflow: hidden;
  }

  .panel-header {
    display: flex; justify-content: space-between; align-items: center;
    padding: 8px 12px;
    border-bottom: 1px solid var(--panel-border, #00c8ff22);
    font-size: 10px; letter-spacing: 0.15em;
    color: var(--accent, #00c8ff);
    flex-shrink: 0;
  }

  .sort-btns { display: flex; gap: 4px; }
  .sort-btn {
    background: none;
    border: 1px solid var(--panel-border, #00c8ff22);
    color: var(--fg-dim, #c8d8f044);
    font-family: inherit; font-size: 8px;
    letter-spacing: 0.1em; padding: 2px 6px;
    border-radius: 3px; cursor: pointer;
    transition: all 0.15s;
  }
  .sort-btn.active { color: var(--accent, #00c8ff); border-color: var(--accent, #00c8ff44); }

  .proc-list-header {
    display: flex; align-items: center;
    padding: 4px 12px;
    font-size: 8px; letter-spacing: 0.1em;
    color: var(--fg-dim, #c8d8f033);
    border-bottom: 1px solid var(--panel-border, #00c8ff11);
    flex-shrink: 0;
  }

  .proc-list {
    flex: 1; overflow-y: auto;
  }
  .proc-list::-webkit-scrollbar { width: 3px; }
  .proc-list::-webkit-scrollbar-thumb { background: var(--panel-border, #00c8ff22); border-radius: 2px; }

  .proc-row {
    display: flex; align-items: center;
    padding: 4px 12px;
    transition: background 0.1s;
    border-bottom: 1px solid var(--panel-border, #00c8ff08);
  }
  .proc-row:hover { background: #ffffff05; }
  .proc-row.highlight { background: rgba(255, 76, 106, 0.04); }

  .col-pid  { width: 44px; font-size: 9px; }
  .col-name { flex: 1; font-size: 10px; color: var(--fg, #c8d8f0); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .col-cpu  { width: 38px; font-size: 10px; text-align: right; }
  .col-mem  { width: 58px; font-size: 9px; text-align: right; }
  .col-act  { width: 24px; }
  .dim { color: var(--fg-dim, #c8d8f044); }

  .kill-btn {
    background: none;
    border: 1px solid transparent;
    color: var(--fg-dim, #c8d8f033);
    font-size: 9px; cursor: pointer;
    padding: 1px 4px; border-radius: 3px;
    transition: all 0.15s; margin-left: 4px;
  }
  .kill-btn:hover { color: var(--red, #ff4c6a); border-color: var(--red, #ff4c6a44); }
  .kill-btn.confirm { color: var(--red, #ff4c6a); border-color: var(--red, #ff4c6a); animation: flashRed 0.3s ease; }
  @keyframes flashRed { 0%,100%{opacity:1} 50%{opacity:0.4} }
</style>
