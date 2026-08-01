<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Terminal from '$lib/components/Terminal.svelte';
  import StatsPanel from '$lib/components/StatsPanel.svelte';
  import FileManager from '$lib/components/FileManager.svelte';
  import ProcessMonitor from '$lib/components/ProcessMonitor.svelte';
  import HudBackground from '$lib/components/HudBackground.svelte';
  import { themes, currentTheme, setTheme, applyThemeCssVars } from '$lib/stores/theme';

  let now = new Date();
  let uptime = 0;
  let fullscreen = false;
  let booting = true;
  let bootText = '';
  let showThemePicker = false;
  let showFileManager = false;
  let showProcesses = false;
  let clockInterval: ReturnType<typeof setInterval>;
  let uptimeInterval: ReturnType<typeof setInterval>;

  let tabs: { id: string; label: string }[] = [{ id: 'main', label: 'SHELL 1' }];
  let activeTab = 'main';
  let tabCounter = 2;

  function addTab() {
    const id = `shell-${tabCounter}`;
    tabs = [...tabs, { id, label: `SHELL ${tabCounter}` }];
    activeTab = id;
    tabCounter++;
  }

  function closeTab(id: string) {
    if (tabs.length === 1) return;
    const idx = tabs.findIndex((t) => t.id === id);
    tabs = tabs.filter((t) => t.id !== id);
    activeTab = tabs[Math.max(0, idx - 1)].id;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === 't') { e.preventDefault(); addTab(); }
    if (e.ctrlKey && e.key === 'w') { e.preventDefault(); closeTab(activeTab); }
    if (e.ctrlKey && e.key === 'f') { e.preventDefault(); toggleFullscreen(); }
    if (e.ctrlKey && e.key === 'b') { e.preventDefault(); showFileManager = !showFileManager; }
    if (e.ctrlKey && e.key === 'p') { e.preventDefault(); showProcesses = !showProcesses; }
  }

  const bootLines = [
    '> INITIALIZING NEXTGEN-UI v1.1.0...',
    '> LOADING WEBGL RENDERER... OK',
    '> SPAWNING PTY SESSION... OK',
    '> CONNECTING STATS ENGINE... OK',
    '> VISUAL OVERHAUL ACTIVE.',
    '> ALL SYSTEMS NOMINAL.',
  ];

  async function runBoot() {
    for (const line of bootLines) {
      for (let i = 0; i <= line.length; i++) {
        bootText = line.slice(0, i);
        await sleep(14);
      }
      await sleep(80);
      bootText = '';
    }
    booting = false;
  }

  function sleep(ms: number) { return new Promise((r) => setTimeout(r, ms)); }

  function toggleFullscreen() {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen();
      fullscreen = true;
    } else {
      document.exitFullscreen();
      fullscreen = false;
    }
  }

  function formatTime(d: Date) { return d.toLocaleTimeString('en-GB', { hour12: false }); }
  function formatDate(d: Date) {
    return d.toLocaleDateString('en-GB', { day: '2-digit', month: 'short', year: 'numeric' });
  }
  function formatUptime(s: number) {
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    return `${String(h).padStart(2,'0')}:${String(m).padStart(2,'0')}:${String(sec).padStart(2,'0')}`;
  }

  let theme = $currentTheme;
  const unsubTheme = currentTheme.subscribe((t) => { theme = t; });

  onMount(() => {
    applyThemeCssVars($currentTheme);
    runBoot();
    clockInterval = setInterval(() => { now = new Date(); }, 1000);
    uptimeInterval = setInterval(() => { uptime++; }, 1000);
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    clearInterval(clockInterval);
    clearInterval(uptimeInterval);
    unsubTheme();
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<div class="hud">
  <!-- Animated WebGL background -->
  <HudBackground />

  <div class="scanlines"></div>
  <div class="corner tl"></div><div class="corner tr"></div>
  <div class="corner bl"></div><div class="corner br"></div>

  <header class="hud-header">
    <div class="hud-left">
      <span class="bracket">[</span>
      <span class="hud-title">NEXTGEN-UI</span>
      <span class="bracket">]</span>
      <span class="sep">//</span>
      <span class="tag">v1.1.0</span>
      <span class="tag highlight">VISUAL OVERHAUL</span>
    </div>
    <div class="hud-center">
      {#if booting}
        <span class="boot-text">{bootText}<span class="cursor">█</span></span>
      {:else}
        <span class="status-ok">● ALL SYSTEMS NOMINAL</span>
      {/if}
    </div>
    <div class="hud-right">
      <div class="theme-wrap">
        <button class="theme-btn" on:click={() => showThemePicker = !showThemePicker}>◈ {theme.name}</button>
        {#if showThemePicker}
          <div class="theme-dropdown">
            {#each Object.entries(themes) as [key, t]}
              <button class="theme-option" class:active={theme.name === t.name} style="color:{t.accent}"
                on:click={() => { setTheme(key); showThemePicker = false; }}>● {t.name}</button>
            {/each}
          </div>
        {/if}
      </div>
      <button class="icon-btn" on:click={() => showFileManager = !showFileManager}>
        {showFileManager ? '✕ FILES' : '◫ FILES'}
      </button>
      <button class="icon-btn" on:click={() => showProcesses = !showProcesses} title="Processes (Ctrl+P)">
        {showProcesses ? '✕ PROCS' : '⬡ PROCS'}
      </button>
      <span class="clock">{formatTime(now)}</span>
      <span class="date">{formatDate(now)}</span>
      <span class="uptime">UP {formatUptime(uptime)}</span>
      <button class="icon-btn" on:click={toggleFullscreen}>{fullscreen ? '⊡' : '⊞'}</button>
      <span class="dot-live"></span>
      <span class="tag live">LIVE</span>
    </div>
  </header>

  <main class="hud-body">
    {#if showFileManager}
      <aside class="fm-section">
        <FileManager />
      </aside>
    {/if}
    <section class="terminal-section">
      <div class="tab-bar">
        {#each tabs as tab}
          <button class="tab" class:active={activeTab === tab.id} on:click={() => activeTab = tab.id}>
            {tab.label}
            {#if tabs.length > 1}
              <span class="tab-close" on:click|stopPropagation={() => closeTab(tab.id)}>✕</span>
            {/if}
          </button>
        {/each}
        <button class="tab-add" on:click={addTab}>＋</button>
        <span class="tab-hint">Ctrl+T · Ctrl+W · Ctrl+B · Ctrl+F</span>
      </div>
      <div class="terminal-container">
        {#each tabs as tab}
          <div class="terminal-slot" class:hidden={activeTab !== tab.id}>
            <Terminal sessionId={tab.id} active={activeTab === tab.id} />
          </div>
        {/each}
      </div>
    </section>
    {#if showProcesses}
      <aside class="proc-section">
        <ProcessMonitor />
      </aside>
    {/if}

    <aside class="stats-section">
      <StatsPanel />
    </aside>
  </main>

  <footer class="hud-footer">
    <span class="footer-item">TAURI 2</span>
    <span class="sep">//</span>
    <span class="footer-item">RUST 1.96</span>
    <span class="sep">//</span>
    <span class="footer-item">SVELTE 5</span>
    <span class="sep">//</span>
    <span class="footer-item">WEBGL</span>
    <span class="sep">//</span>
    <span class="footer-item">PORTABLE-PTY</span>
    <span class="spacer"></span>
    <span class="footer-dim">nextgen-ui © 2026 — AGPL-3.0 — github.com/nextgen-ui</span>
  </footer>
</div>

<style>
  :global(*){box-sizing:border-box}
  :global(body){margin:0;padding:0;background:var(--bg,#0a0e1a);color:var(--fg,#c8d8f0);font-family:'JetBrains Mono','Fira Code',monospace;overflow:hidden}
  .scanlines{position:fixed;inset:0;pointer-events:none;z-index:9999;background:repeating-linear-gradient(to bottom,transparent 0px,transparent 3px,rgba(0,0,0,0.06) 3px,rgba(0,0,0,0.06) 4px)}
  .corner{position:fixed;width:20px;height:20px;pointer-events:none;z-index:9998;border-color:var(--accent,#00c8ff);border-style:solid;opacity:0.6;animation:cornerPulse 3s ease-in-out infinite}
  .tl{top:4px;left:4px;border-width:2px 0 0 2px}.tr{top:4px;right:4px;border-width:2px 2px 0 0;animation-delay:0.75s}
  .bl{bottom:4px;left:4px;border-width:0 0 2px 2px;animation-delay:1.5s}.br{bottom:4px;right:4px;border-width:0 2px 2px 0;animation-delay:2.25s}
  @keyframes cornerPulse{0%,100%{opacity:0.6;transform:scale(1)}50%{opacity:1;transform:scale(1.1)}}
  .hud{display:flex;flex-direction:column;height:100vh;width:100vw;padding:8px;gap:8px;background:transparent;position:relative}
  .hud-header{display:flex;justify-content:space-between;align-items:center;padding:7px 16px;border:1px solid var(--panel-border,#00c8ff44);border-radius:6px;background:var(--panel-bg,#0d1220ee);flex-shrink:0;box-shadow:0 0 20px var(--accent-glow,#00c8ff18),inset 0 0 30px var(--accent-glow,#00c8ff06);position:relative;z-index:1}
  .hud-left,.hud-right{display:flex;align-items:center;gap:10px;min-width:300px}
  .hud-right{justify-content:flex-end}
  .hud-center{flex:1;text-align:center;font-size:11px;color:var(--accent,#00c8ff);opacity:0.8}
  .hud-title{font-size:14px;font-weight:bold;letter-spacing:0.25em;color:var(--accent,#00c8ff);text-shadow:0 0 16px var(--accent,#00c8ff),0 0 32px var(--accent,#00c8ff44)}
  .bracket{color:var(--accent-dim,#00c8ff44);font-size:14px}.sep{color:var(--accent-dim,#00c8ff33);font-size:11px}
  .tag{font-size:9px;color:var(--fg-dim,#c8d8f044);letter-spacing:0.12em;border:1px solid var(--fg-dim,#c8d8f011);padding:1px 5px;border-radius:3px}
  .tag.live{color:var(--green,#00ffb3);border-color:var(--green,#00ffb322)}
  .tag.highlight{color:var(--accent,#00c8ff);border-color:var(--accent,#00c8ff33);animation:tagPulse 2s ease-in-out infinite}
  @keyframes tagPulse{0%,100%{opacity:0.7}50%{opacity:1}}
  .boot-text{font-size:10px;color:var(--accent,#00c8ff);opacity:0.9}
  .status-ok{font-size:10px;color:var(--green,#00ffb3);opacity:0.9}
  .cursor{animation:blink 0.7s step-end infinite}
  @keyframes blink{0%,100%{opacity:1}50%{opacity:0}}
  .clock{font-size:13px;color:var(--accent,#00c8ff);letter-spacing:.1em;text-shadow:0 0 10px var(--accent,#00c8ff)}
  .date{font-size:9px;color:var(--fg-dim,#c8d8f055)}.uptime{font-size:9px;color:var(--fg-dim,#c8d8f033)}
  .icon-btn{background:none;border:1px solid var(--accent-dim,#00c8ff22);color:var(--accent,#00c8ff);opacity:0.7;font-size:11px;font-family:inherit;letter-spacing:0.08em;cursor:pointer;padding:3px 8px;border-radius:3px;transition:all 0.2s}
  .icon-btn:hover{opacity:1;box-shadow:0 0 10px var(--accent-glow,#00c8ff33)}
  .dot-live{width:6px;height:6px;border-radius:50%;background:var(--green,#00ffb3);box-shadow:0 0 10px var(--green,#00ffb3);animation:pulse 2s infinite}
  @keyframes pulse{0%,100%{opacity:1;box-shadow:0 0 10px var(--green,#00ffb3)}50%{opacity:0.3;box-shadow:0 0 3px var(--green,#00ffb3)}}
  .theme-wrap{position:relative}
  .theme-btn{background:none;border:1px solid var(--panel-border,#00c8ff22);color:var(--accent,#00c8ff);font-size:9px;font-family:inherit;letter-spacing:0.1em;padding:3px 8px;border-radius:3px;cursor:pointer;transition:all 0.2s}
  .theme-btn:hover{box-shadow:0 0 8px var(--accent-glow,#00c8ff22)}
  .theme-dropdown{position:absolute;top:calc(100% + 6px);right:0;background:var(--panel-bg,#0d1220);border:1px solid var(--panel-border,#00c8ff33);border-radius:6px;overflow:hidden;z-index:100;min-width:130px;box-shadow:0 4px 20px rgba(0,0,0,0.5)}
  .theme-option{display:block;width:100%;background:none;border:none;font-family:inherit;font-size:10px;letter-spacing:0.1em;padding:8px 14px;text-align:left;cursor:pointer;transition:background 0.15s}
  .theme-option:hover{background:#ffffff08}.theme-option.active{background:#ffffff0f}
  .hud-body{display:flex;flex:1;gap:8px;overflow:hidden;min-height:0;position:relative;z-index:1}
  .fm-section{width:260px;flex-shrink:0}
  .terminal-section{flex:1;min-width:0;display:flex;flex-direction:column}
  .tab-bar{display:flex;align-items:center;gap:2px;padding:0 4px;background:var(--panel-bg,#0d1220ee);border:1px solid var(--panel-border,#00c8ff33);border-bottom:none;border-radius:6px 6px 0 0;flex-shrink:0}
  .tab{display:flex;align-items:center;gap:6px;background:none;border:none;border-bottom:2px solid transparent;color:var(--fg-dim,#c8d8f044);font-family:inherit;font-size:10px;letter-spacing:0.1em;padding:6px 12px;cursor:pointer;transition:all 0.15s}
  .tab:hover{color:var(--fg,#c8d8f0)}.tab.active{color:var(--accent,#00c8ff);border-bottom-color:var(--accent,#00c8ff)}
  .tab-close{font-size:9px;opacity:0.5;transition:opacity 0.15s}.tab-close:hover{opacity:1;color:var(--red,#ff4c6a)}
  .tab-add{background:none;border:none;color:var(--fg-dim,#c8d8f033);font-size:16px;cursor:pointer;padding:2px 8px;transition:color 0.15s}
  .tab-add:hover{color:var(--accent,#00c8ff)}
  .tab-hint{font-size:8px;color:var(--fg-dim,#c8d8f022);letter-spacing:0.05em;margin-left:auto;padding-right:6px}
  .terminal-container{flex:1;min-height:0}.terminal-slot{height:100%}.terminal-slot.hidden{display:none}
  .stats-section{width:230px;flex-shrink:0}
  .hud-footer{display:flex;align-items:center;gap:8px;padding:5px 16px;border:1px solid var(--panel-border,#00c8ff22);border-radius:6px;background:var(--panel-bg,#0d1220ee);flex-shrink:0;font-size:9px;letter-spacing:0.12em;position:relative;z-index:1}
  .spacer{flex:1}.footer-item{color:var(--fg-dim,#c8d8f044)}.footer-dim{color:var(--fg-dim,#c8d8f018)}
</style>
