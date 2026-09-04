<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { WebLinksAddon } from '@xterm/addon-web-links';
  import { ptySpawn, ptyWrite, ptyResize, ptyClose, onPtyOutput } from '$lib/ipc';
  import { currentTheme } from '$lib/stores/theme';
  import '@xterm/xterm/css/xterm.css';

  export let sessionId: string = 'main';
  export let active: boolean = true;

  let container: HTMLDivElement;
  let term: Terminal;
  let fitAddon: FitAddon;
  let unlisten: (() => void) | null = null;
  let resizeObserver: ResizeObserver;
  let theme = $currentTheme;

  const unsubTheme = currentTheme.subscribe((t) => {
    theme = t;
    if (term) {
      term.options.theme = buildXtermTheme(t);
    }
  });

  function buildXtermTheme(t: typeof theme) {
    return {
      background:       t.termBg,
      foreground:       t.termFg,
      cursor:           t.termCursor,
      selectionBackground: t.accent + '33',
      black:   '#1a1e2e', red:     t.red,
      green:   t.green,   yellow:  t.yellow,
      blue:    t.accent,  magenta: t.purple,
      cyan:    t.green,   white:   t.fg,
      brightBlack: '#2a2e3e', brightRed: t.red,
      brightGreen: t.green,  brightYellow: t.yellow,
      brightBlue: t.accent,  brightMagenta: t.purple,
      brightCyan: t.green,   brightWhite: t.fg,
    };
  }

  onMount(async () => {
    term = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: '"JetBrains Mono", "Fira Code", monospace',
      theme: buildXtermTheme(theme),
      allowProposedApi: true,
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.loadAddon(new WebLinksAddon());
    term.open(container);
    fitAddon.fit();

    const { cols, rows } = term;
    try {
      await ptySpawn(sessionId, cols, rows);
    } catch (e) {
      term.writeln(`\r\n\x1b[31m[nextgen-ui] Failed to spawn shell: ${e}\x1b[0m\r\n`);
      return;
    }

    term.onData(async (data) => { await ptyWrite(sessionId, data); });

    unlisten = await onPtyOutput(({ session_id, data }) => {
      if (session_id === sessionId) term.write(data);
    });

    resizeObserver = new ResizeObserver(() => {
      fitAddon.fit();
      ptyResize(sessionId, term.cols, term.rows).catch(() => {});
    });
    resizeObserver.observe(container);
    term.focus();
  });

  onDestroy(async () => {
    unlisten?.();
    resizeObserver?.disconnect();
    unsubTheme();
    term?.dispose();
    await ptyClose(sessionId).catch(() => {});
  });

  export function focus() { term?.focus(); }
</script>

<div class="terminal-wrap" class:hidden={!active}>
  <div class="terminal-header">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="session-label">TERMINAL // {sessionId.toUpperCase()}</span>
  </div>
  <div class="terminal-body" bind:this={container}></div>
</div>

<style>
  .terminal-wrap {
    display: flex;
    flex-direction: column;
    height: 100%;
    border: 1px solid var(--panel-border, #00c8ff33);
    border-radius: 6px;
    overflow: hidden;
    background: var(--term-bg, #060910);
  }
  .hidden { display: none; }
  .terminal-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: var(--panel-bg, #0a0e1acc);
    border-bottom: 1px solid var(--panel-border, #00c8ff22);
    flex-shrink: 0;
  }
  .dot { width: 10px; height: 10px; border-radius: 50%; }
  .dot.red    { background: #ff4c6a; }
  .dot.yellow { background: #ffcc00; }
  .dot.green  { background: #00ffb3; }
  .session-label {
    margin-left: 8px;
    font-size: 10px;
    font-family: 'JetBrains Mono', monospace;
    color: var(--accent-dim, #00c8ff88);
    letter-spacing: 0.1em;
  }
  .terminal-body { flex: 1; overflow: hidden; padding: 4px; }
</style>
