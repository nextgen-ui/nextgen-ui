<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { systemStats, cpuHistory, netInHistory, netOutHistory, memHistory, updateStats, formatBytes, formatPercent } from '$lib/stores/stats';
  import { onSystemStats } from '$lib/ipc';
  import { currentTheme } from '$lib/stores/theme';
  import Sparkline from './Sparkline.svelte';

  let unlisten: (() => void) | null = null;
  let stats = $systemStats;
  let theme = $currentTheme;

  const unsubStats = systemStats.subscribe((s) => (stats = s));
  const unsubTheme = currentTheme.subscribe((t) => (theme = t));

  let cpuHist: number[] = [];
  let netInHist: number[] = [];
  let netOutHist: number[] = [];
  let memHist: number[] = [];

  const unsubCpu    = cpuHistory.subscribe((v) => (cpuHist = v));
  const unsubNetIn  = netInHistory.subscribe((v) => (netInHist = v));
  const unsubNetOut = netOutHistory.subscribe((v) => (netOutHist = v));
  const unsubMem    = memHistory.subscribe((v) => (memHist = v));

  onMount(async () => {
    unlisten = await onSystemStats((s) => updateStats(s));
  });

  onDestroy(() => {
    unlisten?.();
    unsubStats(); unsubTheme();
    unsubCpu(); unsubNetIn(); unsubNetOut(); unsubMem();
  });

  function barWidth(used: number, total: number): string {
    if (total === 0) return '0%';
    return `${Math.min((used / total) * 100, 100).toFixed(1)}%`;
  }

  function cpuColor(val: number): string {
    if (val > 80) return theme.red;
    if (val > 50) return theme.yellow;
    return theme.accent;
  }

  $: maxNetIn  = Math.max(...netInHist,  1);
  $: maxNetOut = Math.max(...netOutHist, 1);
</script>

<div class="stats-panel">
  <div class="panel-header">⬡ SYS MONITOR</div>

  <!-- CPU -->
  <div class="stat-section">
    <div class="stat-row">
      <span class="label">CPU</span>
      <span class="value" style="color:{cpuColor(stats.cpu_usage)}">{formatPercent(stats.cpu_usage)}</span>
    </div>
    <div class="bar-track">
      <div class="bar-fill" style="width:{barWidth(stats.cpu_usage, 100)};background:{cpuColor(stats.cpu_usage)}"></div>
    </div>
    <div class="sparkline-wrap">
      <Sparkline data={cpuHist} max={100} color={cpuColor(stats.cpu_usage)} height={28} width={180} />
    </div>
  </div>

  <!-- Per-core -->
  {#if stats.cpu_per_core.length > 0}
    <div class="core-grid">
      {#each stats.cpu_per_core as core, i}
        <div class="core-item">
          <div class="core-bar-track">
            <div class="core-bar-fill" style="height:{barWidth(core, 100)};background:{cpuColor(core)}"></div>
          </div>
          <div class="core-val">{core.toFixed(0)}</div>
        </div>
      {/each}
    </div>
  {/if}

  <div class="divider"></div>

  <!-- RAM -->
  <div class="stat-section">
    <div class="stat-row">
      <span class="label">RAM</span>
      <span class="value">{formatBytes(stats.mem_used)} / {formatBytes(stats.mem_total)}</span>
    </div>
    <div class="bar-track">
      <div class="bar-fill" style="width:{barWidth(stats.mem_used, stats.mem_total)};background:{theme.green}"></div>
    </div>
    <div class="sparkline-wrap">
      <Sparkline data={memHist} max={100} color={theme.green} height={28} width={180} />
    </div>
  </div>

  {#if stats.swap_total > 0}
    <div class="stat-section">
      <div class="stat-row">
        <span class="label">SWAP</span>
        <span class="value">{formatBytes(stats.swap_used)} / {formatBytes(stats.swap_total)}</span>
      </div>
      <div class="bar-track">
        <div class="bar-fill" style="width:{barWidth(stats.swap_used, stats.swap_total)};background:{theme.purple}"></div>
      </div>
    </div>
  {/if}

  <div class="divider"></div>

  <!-- Network -->
  <div class="stat-section">
    <div class="stat-row">
      <span class="label">NET ↓</span>
      <span class="value">{formatBytes(stats.net_in)}/s</span>
    </div>
    <div class="sparkline-wrap">
      <Sparkline data={netInHist} max={maxNetIn} color={theme.accent} height={22} width={180} />
    </div>
    <div class="stat-row" style="margin-top:6px">
      <span class="label">NET ↑</span>
      <span class="value">{formatBytes(stats.net_out)}/s</span>
    </div>
    <div class="sparkline-wrap">
      <Sparkline data={netOutHist} max={maxNetOut} color={theme.green} height={22} width={180} />
    </div>
  </div>

  <div class="divider"></div>

  <!-- Disk -->
  <div class="stat-section">
    <div class="stat-row">
      <span class="label">DISK R</span>
      <span class="value">{formatBytes(stats.disk_read)}/s</span>
    </div>
    <div class="stat-row">
      <span class="label">DISK W</span>
      <span class="value">{formatBytes(stats.disk_write)}/s</span>
    </div>
  </div>
</div>

<style>
  .stats-panel {
    display: flex; flex-direction: column; gap: 10px;
    padding: 12px; height: 100%;
    background: var(--panel-bg, #0d1220cc);
    border: 1px solid var(--panel-border, #00c8ff33);
    border-radius: 6px;
    font-family: 'JetBrains Mono', monospace;
    overflow-y: auto;
    box-sizing: border-box;
    position: relative; z-index: 1;
  }
  .stats-panel::-webkit-scrollbar { width: 3px; }
  .stats-panel::-webkit-scrollbar-thumb { background: var(--panel-border, #00c8ff22); border-radius: 2px; }

  .panel-header { font-size: 10px; letter-spacing: 0.15em; color: var(--accent, #00c8ff); border-bottom: 1px solid var(--panel-border, #00c8ff22); padding-bottom: 8px; }

  .stat-section { display: flex; flex-direction: column; gap: 4px; }
  .stat-row { display: flex; justify-content: space-between; align-items: center; }
  .label { font-size: 10px; color: var(--fg-dim, #c8d8f077); letter-spacing: 0.08em; }
  .value { font-size: 10px; color: var(--fg, #c8d8f0); }

  .bar-track { height: 3px; background: #ffffff0f; border-radius: 2px; overflow: hidden; }
  .bar-fill { height: 100%; border-radius: 2px; transition: width 0.5s ease; }

  .sparkline-wrap { margin-top: 2px; opacity: 0.9; }

  .core-grid { display: flex; gap: 3px; align-items: flex-end; height: 36px; }
  .core-item { display: flex; flex-direction: column; align-items: center; flex: 1; gap: 2px; }
  .core-bar-track { width: 100%; height: 28px; background: #ffffff08; border-radius: 2px; display: flex; align-items: flex-end; overflow: hidden; }
  .core-bar-fill { width: 100%; border-radius: 2px; transition: height 0.5s ease; min-height: 2px; }
  .core-val { font-size: 7px; color: var(--fg-dim, #c8d8f033); }

  .divider { height: 1px; background: var(--panel-border, #00c8ff11); flex-shrink: 0; }
</style>
