<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Terminal from '$lib/components/Terminal.svelte';
  import StatsPanel from '$lib/components/StatsPanel.svelte';

  let now = new Date();
  let uptime = 0;
  let fullscreen = false;
  let booting = true;
  let bootText = '';
  let clockInterval: ReturnType<typeof setInterval>;
  let uptimeInterval: ReturnType<typeof setInterval>;

  const bootLines = [
    '> INITIALIZING NEXTGEN-UI v0.1.0...',
    '> LOADING TAURI RUNTIME... OK',
    '> SPAWNING PTY SESSION... OK',
    '> CONNECTING STATS ENGINE... OK',
    '> ALL SYSTEMS NOMINAL.',
  ];

  async function runBoot() {
    for (const line of bootLines) {
      for (let i = 0; i <= line.length; i++) {
        bootText = line.slice(0, i);
        await sleep(18);
      }
      await sleep(120);
      bootText = '';
    }
    booting = false;
  }

  function sleep(ms: number) {
    return new Promise((r) => setTimeout(r, ms));
  }

  function toggleFullscreen() {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen();
      fullscreen = true;
    } else {
      document.exitFullscreen();
      fullscreen = false;
    }
  }

  function formatTime(d: Date) {
    return d.toLocaleTimeString('en-GB', { hour12: false });
  }

  function formatDate(d: Date) {
    return d.toLocaleDateString('en-GB', { day: '2-digit', month: 'short', year: 'numeric' });
  }

  function formatUptime(s: number) {
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    return `${String(h).padStart(2,'0')}:${String(m).padStart(2,'0')}:${String(sec).padStart(2,'0')}`;
  }

  onMount(() => {
    runBoot();
    clockInterval = setInterval(() => { now = new Date(); }, 1000);
    uptimeInterval = setInterval(() => { uptime++; }, 1000);
  });

  onDestroy(() => {
    clearInterval(clockInterval);
    clearInterval(uptimeInterval);
  });
</script>

<div class="hud">
  <!-- Scanline overlay -->
  <div class="scanlines" />

  <!-- Header -->
  <header class="hud-header">
    <div class="hud-left">
      <span class="bracket">[</span>
      <span class="hud-title">NEXTGEN-UI</span>
      <span class="bracket">]</span>
      <span class="sep">//</span>
      <span class="tag">v0.1.0</span>
      <span class="tag">PHASE-2</span>
    </div>

    <div class="hud-center">
      {#if booting}
        <span class="boot-text">{bootText}<span class="cursor">█</span></span>
      {:else}
        <span class="status-ok">● ALL SYSTEMS NOMINAL</span>
      {/if}
    </div>

    <div class="hud-right">
      <span class="clock">{formatTime(now)}</span>
      <span class="date">{formatDate(now)}</span>
      <span class="uptime">UP {formatUptime(uptime)}</span>
      <button class="fs-btn" on:click={toggleFullscreen} title="Toggle fullscreen">
        {fullscreen ? '⊡' : '⊞'}
      </button>
      <span class="dot-live" />
      <span class="tag live">LIVE</span>
    </div>
  </header>

  <!-- Body -->
  <main class="hud-body">
    <section class="terminal-section">
      <Terminal sessionId="main" />
    </section>
    <aside class="stats-section">
      <StatsPanel />
    </aside>
  </main>

  <!-- Footer -->
  <footer class="hud-footer">
    <span class="footer-item">TAURI 2</span>
    <span class="sep">//</span>
    <span class="footer-item">RUST 1.96</span>
    <span class="sep">//</span>
    <span class="footer-item">SVELTE 5</span>
    <span class="sep">//</span>
    <span class="footer-item">PORTABLE-PTY</span>
    <span class="spacer" />
    <span class="footer-item dim">nextgen-ui © 2026 — AGPL-3.0 — github.com/nextgen-ui</span>
  </footer>
</div>

<style>
  :global(*) { box-sizing: border-box; }
  :global(body) {
    margin: 0; padding: 0;
    background: #0a0e1a;
    color: #c8d8f0;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    overflow: hidden;
  }

  /* ── Scanlines ── */
  .scanlines {
    position: fixed;
    inset: 0;
    pointer-events: none;
    z-index: 9999;
    background: repeating-linear-gradient(
      to bottom,
      transparent 0px,
      transparent 3px,
      rgba(0,0,0,0.08) 3px,
      rgba(0,0,0,0.08) 4px
    );
  }

  /* ── HUD shell ── */
  .hud {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    padding: 8px;
    gap: 8px;
    background: #0a0e1a;
    position: relative;
  }

  /* ── Header ── */
  .hud-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 7px 16px;
    border: 1px solid #00c8ff44;
    border-radius: 6px;
    background: #0d1220ee;
    flex-shrink: 0;
    box-shadow: 0 0 12px #00c8ff18, inset 0 0 20px #00c8ff06;
  }

  .hud-left, .hud-right {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 280px;
  }

  .hud-right { justify-content: flex-end; }

  .hud-center {
    flex: 1;
    text-align: center;
    font-size: 11px;
    color: #00c8ff99;
    letter-spacing: 0.08em;
  }

  .hud-title {
    font-size: 14px;
    font-weight: bold;
    letter-spacing: 0.25em;
    color: #00c8ff;
    text-shadow: 0 0 10px #00c8ff88;
  }

  .bracket { color: #00c8ff44; font-size: 14px; }
  .sep { color: #00c8ff33; font-size: 11px; }

  .tag {
    font-size: 9px;
    color: #c8d8f044;
    letter-spacing: 0.12em;
    border: 1px solid #c8d8f011;
    padding: 1px 5px;
    border-radius: 3px;
  }

  .tag.live {
    color: #00ffb3;
    border-color: #00ffb322;
  }

  .boot-text {
    font-size: 10px;
    color: #00c8ffaa;
    letter-spacing: 0.05em;
  }

  .cursor {
    animation: blink 0.7s step-end infinite;
    color: #00c8ff;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0; }
  }

  .status-ok {
    font-size: 10px;
    color: #00ffb3aa;
    letter-spacing: 0.1em;
  }

  .clock {
    font-size: 13px;
    color: #00c8ff;
    letter-spacing: 0.1em;
    text-shadow: 0 0 8px #00c8ff66;
  }

  .date {
    font-size: 9px;
    color: #c8d8f055;
    letter-spacing: 0.08em;
  }

  .uptime {
    font-size: 9px;
    color: #c8d8f033;
    letter-spacing: 0.08em;
  }

  .fs-btn {
    background: none;
    border: 1px solid #00c8ff22;
    color: #00c8ff88;
    font-size: 14px;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 3px;
    line-height: 1;
    transition: all 0.2s;
  }
  .fs-btn:hover {
    border-color: #00c8ff66;
    color: #00c8ff;
    box-shadow: 0 0 8px #00c8ff33;
  }

  .dot-live {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #00ffb3;
    box-shadow: 0 0 8px #00ffb3;
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; box-shadow: 0 0 8px #00ffb3; }
    50%       { opacity: 0.4; box-shadow: 0 0 3px #00ffb3; }
  }

  /* ── Body ── */
  .hud-body {
    display: flex;
    flex: 1;
    gap: 8px;
    overflow: hidden;
    min-height: 0;
  }

  .terminal-section {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .stats-section {
    width: 230px;
    flex-shrink: 0;
  }

  /* ── Footer ── */
  .hud-footer {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 16px;
    border: 1px solid #00c8ff22;
    border-radius: 6px;
    background: #0d1220ee;
    flex-shrink: 0;
    font-size: 9px;
    letter-spacing: 0.12em;
    color: #c8d8f044;
    box-shadow: 0 0 8px #00c8ff0a;
  }

  .spacer { flex: 1; }
  .dim { color: #c8d8f018; }
  .footer-item { color: #c8d8f044; }
</style>
