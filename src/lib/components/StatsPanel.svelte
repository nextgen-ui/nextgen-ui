<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { systemStats, formatBytes, formatPercent } from '$lib/stores/stats';
  import { onSystemStats } from '$lib/ipc';

  let unlisten: (() => void) | null = null;
  let stats = $systemStats;
  const unsubscribe = systemStats.subscribe((s) => (stats = s));

  onMount(async () => {
    unlisten = await onSystemStats((s) => systemStats.set(s));
  });
  onDestroy(() => { unlisten?.(); unsubscribe(); });

  function barWidth(used: number, total: number): string {
    if (total === 0) return '0%';
    return `${Math.min((used / total) * 100, 100).toFixed(1)}%`;
  }
  function cpuColor(val: number): string {
    if (val > 80) return '#ff4c6a';
    if (val > 50) return '#ffcc00';
    return '#00c8ff';
  }
</script>

<div class="stats-panel">
  <div class="panel-header">SYS MONITOR</div>

  <div class="stat-block">
    <div class="stat-label">CPU <span class="stat-value">{formatPercent(stats.cpu_usage)}</span></div>
    <div class="bar-track">
      <div class="bar-fill" style="width:{barWidth(stats.cpu_usage, 100)}; background:{cpuColor(stats.cpu_usage)}" />
    </div>
  </div>

  {#if stats.cpu_per_core.length > 0}
    <div class="core-grid">
      {#each stats.cpu_per_core as core, i}
        <div class="core-item">
          <div class="core-label">C{i}</div>
          <div class="core-bar-track">
            <div class="core-bar-fill" style="height:{barWidth(core, 100)}; background:{cpuColor(core)}" />
          </div>
          <div class="core-val">{core.toFixed(0)}%</div>
        </div>
      {/each}
    </div>
  {/if}

  <div class="stat-block">
    <div class="stat-label">RAM <span class="stat-value">{formatBytes(stats.mem_used)} / {formatBytes(stats.mem_total)}</span></div>
    <div class="bar-track">
      <div class="bar-fill" style="width:{barWidth(stats.mem_used, stats.mem_total)}; background:#00ffb3" />
    </div>
  </div>

  {#if stats.swap_total > 0}
    <div class="stat-block">
      <div class="stat-label">SWAP <span class="stat-value">{formatBytes(stats.swap_used)} / {formatBytes(stats.swap_total)}</span></div>
      <div class="bar-track">
        <div class="bar-fill" style="width:{barWidth(stats.swap_used, stats.swap_total)}; background:#b06fff" />
      </div>
    </div>
  {/if}

  <div class="divider" />

  <div class="stat-block">
    <div class="stat-label">NET ↓ <span class="stat-value">{formatBytes(stats.net_in)}/s</span></div>
    <div class="stat-label">NET ↑ <span class="stat-value">{formatBytes(stats.net_out)}/s</span></div>
  </div>

  <div class="stat-block">
    <div class="stat-label">DISK R <span class="stat-value">{formatBytes(stats.disk_read)}/s</span></div>
    <div class="stat-label">DISK W <span class="stat-value">{formatBytes(stats.disk_write)}/s</span></div>
  </div>
</div>

<style>
  .stats-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px;
    background: #0d1220cc;
    border: 1px solid #00c8ff33;
    border-radius: 6px;
    height: 100%;
    box-sizing: border-box;
    font-family: 'JetBrains Mono', monospace;
    overflow-y: auto;
  }
  .panel-header {
    font-size: 10px;
    letter-spacing: 0.15em;
    color: #00c8ff;
    border-bottom: 1px solid #00c8ff22;
    padding-bottom: 8px;
  }
  .stat-block { display: flex; flex-direction: column; gap: 4px; }
  .stat-label {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: #c8d8f088;
  }
  .stat-value { color: #c8d8f0; }
  .bar-track { height: 4px; background: #ffffff11; border-radius: 2px; overflow: hidden; }
  .bar-fill { height: 100%; border-radius: 2px; transition: width 0.4s ease; }
  .core-grid { display: flex; gap: 4px; align-items: flex-end; height: 48px; }
  .core-item { display: flex; flex-direction: column; align-items: center; flex: 1; gap: 2px; }
  .core-label { font-size: 8px; color: #c8d8f044; }
  .core-bar-track { width: 100%; height: 32px; background: #ffffff0a; border-radius: 2px; display: flex; align-items: flex-end; overflow: hidden; }
  .core-bar-fill { width: 100%; border-radius: 2px; transition: height 0.4s ease; min-height: 2px; }
  .core-val { font-size: 7px; color: #c8d8f055; }
  .divider { height: 1px; background: #00c8ff11; }
</style>
