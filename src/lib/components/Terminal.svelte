<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { WebLinksAddon } from '@xterm/addon-web-links';
  import { ptySpawn, ptyWrite, ptyResize, ptyClose, onPtyOutput } from '$lib/ipc';
  import '@xterm/xterm/css/xterm.css';

  export let sessionId: string = 'main';

  let container: HTMLDivElement;
  let term: Terminal;
  let fitAddon: FitAddon;
  let unlisten: (() => void) | null = null;
  let resizeObserver: ResizeObserver;

  onMount(async () => {
    // Init xterm
    term = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: '"JetBrains Mono", "Fira Code", "Cascadia Code", monospace',
      theme: {
        background: '#060910',
        foreground: '#c8d8f0',
        cursor:     '#00c8ff',
        selectionBackground: '#00c8ff33',
        black:   '#1a1e2e',
        red:     '#ff4c6a',
        green:   '#00ffb3',
        yellow:  '#ffcc00',
        blue:    '#00c8ff',
        magenta: '#b06fff',
        cyan:    '#00e5cc',
        white:   '#c8d8f0',
        brightBlack:   '#2a2e3e',
        brightRed:     '#ff6b84',
        brightGreen:   '#33ffbf',
        brightYellow:  '#ffd633',
        brightBlue:    '#33d4ff',
        brightMagenta: '#bf8fff',
        brightCyan:    '#33eedd',
        brightWhite:   '#ddeeff',
      },
      allowProposedApi: true,
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.loadAddon(new WebLinksAddon());
    term.open(container);
    fitAddon.fit();

    const { cols, rows } = term;

    // Spawn PTY session
    try {
      await ptySpawn(sessionId, cols, rows);
    } catch (e) {
      term.writeln(`\r\n\x1b[31m[nextgen-ui] Failed to spawn shell: ${e}\x1b[0m\r\n`);
      return;
    }

    // Frontend → backend: send keystrokes
    term.onData(async (data) => {
      await ptyWrite(sessionId, data);
    });

    // Backend → frontend: receive output
    unlisten = await onPtyOutput(({ session_id, data }) => {
      if (session_id === sessionId) {
        term.write(data);
      }
    });

    // Resize PTY when terminal panel resizes
    resizeObserver = new ResizeObserver(() => {
      fitAddon.fit();
      const { cols, rows } = term;
      ptyResize(sessionId, cols, rows).catch(() => {});
    });
    resizeObserver.observe(container);

    term.focus();
  });

  onDestroy(async () => {
    unlisten?.();
    resizeObserver?.disconnect();
    term?.dispose();
    await ptyClose(sessionId).catch(() => {});
  });
</script>

<div class="terminal-wrap">
  <div class="terminal-header">
    <span class="dot red" />
    <span class="dot yellow" />
    <span class="dot green" />
    <span class="session-label">TERMINAL // {sessionId.toUpperCase()}</span>
  </div>
  <div class="terminal-body" bind:this={container} />
</div>

<style>
  .terminal-wrap {
    display: flex;
    flex-direction: column;
    height: 100%;
    border: 1px solid #00c8ff33;
    border-radius: 6px;
    overflow: hidden;
    background: #060910;
  }

  .terminal-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: #0a0e1acc;
    border-bottom: 1px solid #00c8ff22;
    flex-shrink: 0;
  }

  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }

  .dot.red    { background: #ff4c6a; }
  .dot.yellow { background: #ffcc00; }
  .dot.green  { background: #00ffb3; }

  .session-label {
    margin-left: 8px;
    font-size: 10px;
    font-family: 'JetBrains Mono', monospace;
    color: #00c8ff88;
    letter-spacing: 0.1em;
  }

  .terminal-body {
    flex: 1;
    overflow: hidden;
    padding: 4px;
  }
</style>
