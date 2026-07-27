<script lang="ts">
  import Terminal from '$lib/components/Terminal.svelte';
  import StatsPanel from '$lib/components/StatsPanel.svelte';
</script>

<div class="hud">
  <!-- Top bar -->
  <header class="hud-header">
    <div class="hud-title">
      <span class="bracket">[</span>
      NEXTGEN-UI
      <span class="bracket">]</span>
    </div>
    <div class="hud-meta">
      <span class="tag">v0.1.0</span>
      <span class="tag">PHASE-1</span>
      <span class="dot-live" />
      <span class="tag live">LIVE</span>
    </div>
  </header>

  <!-- Main content -->
  <main class="hud-body">
    <section class="terminal-section">
      <Terminal sessionId="main" />
    </section>
    <aside class="stats-section">
      <StatsPanel />
    </aside>
  </main>

  <!-- Bottom bar -->
  <footer class="hud-footer">
    <span class="footer-item">TAURI 2</span>
    <span class="sep">//</span>
    <span class="footer-item">RUST</span>
    <span class="sep">//</span>
    <span class="footer-item">SVELTE 5</span>
    <span class="spacer" />
    <span class="footer-item dim">nextgen-ui © 2026 — AGPL-3.0</span>
  </footer>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: #0a0e1a;
    color: #c8d8f0;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    overflow: hidden;
  }

  .hud {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    box-sizing: border-box;
    padding: 8px;
    gap: 8px;
    background: #0a0e1a;
  }

  /* ── Header ── */
  .hud-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 14px;
    border: 1px solid #00c8ff33;
    border-radius: 6px;
    background: #0d1220cc;
    flex-shrink: 0;
  }

  .hud-title {
    font-size: 13px;
    letter-spacing: 0.2em;
    color: #00c8ff;
  }

  .bracket { color: #00c8ff66; }

  .hud-meta {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .tag {
    font-size: 10px;
    color: #c8d8f055;
    letter-spacing: 0.1em;
  }

  .tag.live { color: #00ffb3; }

  .dot-live {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #00ffb3;
    box-shadow: 0 0 6px #00ffb3;
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50%       { opacity: 0.4; }
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
    width: 220px;
    flex-shrink: 0;
  }

  /* ── Footer ── */
  .hud-footer {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 14px;
    border: 1px solid #00c8ff22;
    border-radius: 6px;
    background: #0d1220cc;
    flex-shrink: 0;
    font-size: 9px;
    letter-spacing: 0.12em;
    color: #c8d8f044;
  }

  .sep { color: #00c8ff33; }
  .spacer { flex: 1; }
  .dim { color: #c8d8f022; }
</style>
