<script lang="ts">
  import { ptyWrite } from '$lib/ipc';

  export let sessionId: string;

  interface Key {
    lo?: string;    // unshifted character
    hi?: string;    // shifted character
    seq?: string;   // raw byte sequence, sent verbatim
    mod?: Mod;      // modifier key rather than an input key
    label?: string; // display override
    w?: number;     // relative width
  }
  type Mod = 'shift' | 'ctrl' | 'alt' | 'caps';

  const k = (lo: string, hi?: string): Key => ({ lo, hi });
  const letters = (s: string) => s.split('').map((c) => k(c));

  const rows: Key[][] = [
    [
      { label: 'ESC', seq: '\x1b', w: 1.5 },
      k('`', '~'), k('1', '!'), k('2', '@'), k('3', '#'), k('4', '$'), k('5', '%'),
      k('6', '^'), k('7', '&'), k('8', '*'), k('9', '('), k('0', ')'),
      k('-', '_'), k('=', '+'),
      { label: '⌫', seq: '\x7f', w: 2 },
    ],
    [
      { label: 'TAB', seq: '\t', w: 1.5 },
      ...letters('qwertyuiop'),
      k('[', '{'), k(']', '}'), k('\\', '|'),
    ],
    [
      { label: 'CAPS', mod: 'caps', w: 1.75 },
      ...letters('asdfghjkl'),
      k(';', ':'), k("'", '"'),
      { label: 'ENTER', seq: '\r', w: 2.25 },
    ],
    [
      { label: 'SHIFT', mod: 'shift', w: 2.25 },
      ...letters('zxcvbnm'),
      k(',', '<'), k('.', '>'), k('/', '?'),
      { label: 'SHIFT', mod: 'shift', w: 2.25 },
    ],
    [
      { label: 'CTRL', mod: 'ctrl', w: 1.5 },
      { label: 'ALT', mod: 'alt', w: 1.25 },
      { label: 'SPACE', lo: ' ', w: 7 },
      { label: 'ALT', mod: 'alt', w: 1.25 },
      { label: 'CTRL', mod: 'ctrl', w: 1.5 },
      { label: '←', seq: '\x1b[D' }, { label: '↑', seq: '\x1b[A' },
      { label: '↓', seq: '\x1b[B' }, { label: '→', seq: '\x1b[C' },
    ],
  ];

  let shift = false, ctrl = false, alt = false, caps = false;
  let lastError = '';

  function isMod(m: Mod) {
    return m === 'shift' ? shift : m === 'ctrl' ? ctrl : m === 'alt' ? alt : caps;
  }

  function toggleMod(m: Mod) {
    if (m === 'shift') shift = !shift;
    else if (m === 'ctrl') ctrl = !ctrl;
    else if (m === 'alt') alt = !alt;
    else caps = !caps;
  }

  /** Label shown on the cap, reflecting the live shift/caps state. */
  function capLabel(key: Key): string {
    if (key.label) return key.label;
    const base = key.lo ?? '';
    if (/^[a-z]$/.test(base)) return caps !== shift ? base.toUpperCase() : base;
    return shift && key.hi ? key.hi : base;
  }

  /** Resolve a printable key to the character shift/caps say it produces. */
  function resolve(key: Key): string {
    const base = key.lo ?? '';
    if (/^[a-z]$/.test(base)) return caps !== shift ? base.toUpperCase() : base;
    return shift && key.hi ? key.hi : base;
  }

  /** Apply Ctrl (C0 control codes) and Alt (ESC prefix) to a character. */
  function encode(ch: string): string {
    let out = ch;
    if (ctrl) {
      const c = ch.toLowerCase();
      if (c >= 'a' && c <= 'z') out = String.fromCharCode(c.charCodeAt(0) - 96);
      else if (c === '[') out = '\x1b';
      else if (c === '\\') out = '\x1c';
      else if (c === ']') out = '\x1d';
      else if (c === ' ') out = '\x00';
    }
    if (alt) out = '\x1b' + out;
    return out;
  }

  async function press(key: Key) {
    if (key.mod) { toggleMod(key.mod); return; }

    let data: string;
    if (key.seq) {
      // Alt-prefix plain sequences (Alt+Backspace = delete word) but never
      // double-escape one that is already an escape sequence.
      data = alt && !key.seq.startsWith('\x1b') ? '\x1b' + key.seq : key.seq;
    } else {
      data = encode(resolve(key));
    }

    try {
      await ptyWrite(sessionId, data);
      lastError = '';
    } catch (e) {
      lastError = String(e);
    }

    // Shift/Ctrl/Alt are one-shot; Caps Lock latches.
    shift = false; ctrl = false; alt = false;
  }
</script>

<div class="osk">
  <div class="osk-head">
    <span class="osk-title">◰ KEYBOARD</span>
    <span class="osk-target">→ {sessionId.toUpperCase()}</span>
    <span class="osk-mods">
      {#if ctrl}<span class="mod-live">CTRL</span>{/if}
      {#if alt}<span class="mod-live">ALT</span>{/if}
      {#if shift}<span class="mod-live">SHIFT</span>{/if}
      {#if caps}<span class="mod-live caps">CAPS</span>{/if}
    </span>
    {#if lastError}<span class="osk-err">{lastError}</span>{/if}
  </div>

  {#each rows as row}
    <div class="osk-row">
      {#each row as key}
        <button
          class="cap"
          class:mod={!!key.mod}
          class:on={key.mod ? isMod(key.mod) : false}
          class:wide={(key.w ?? 1) > 1.4}
          style="flex:{key.w ?? 1}"
          aria-pressed={key.mod ? isMod(key.mod) : undefined}
          on:mousedown|preventDefault
          on:click={() => press(key)}
        >{capLabel(key)}</button>
      {/each}
    </div>
  {/each}
</div>

<style>
  .osk{display:flex;flex-direction:column;gap:3px;padding:6px;margin-top:6px;background:var(--panel-bg,#0d1220ee);border:1px solid var(--panel-border,#00c8ff33);border-radius:6px;flex-shrink:0}
  .osk-head{display:flex;align-items:center;gap:8px;padding:0 2px 4px;font-size:9px;letter-spacing:0.12em}
  .osk-title{color:var(--accent,#00c8ff)}
  .osk-target{color:var(--fg-dim,#c8d8f044)}
  .osk-mods{display:flex;gap:4px;margin-left:auto}
  .mod-live{color:var(--accent,#00c8ff);border:1px solid var(--accent-dim,#00c8ff44);border-radius:3px;padding:0 4px}
  .mod-live.caps{color:var(--yellow,#ffcc00);border-color:var(--yellow,#ffcc00)}
  .osk-err{color:var(--red,#ff4c6a);margin-left:8px}
  .osk-row{display:flex;gap:3px}
  .cap{flex:1;min-width:0;height:26px;background:#ffffff06;border:1px solid var(--panel-border,#00c8ff22);border-radius:3px;color:var(--fg,#c8d8f0);font-family:inherit;font-size:10px;letter-spacing:0.05em;cursor:pointer;transition:background 0.1s,color 0.1s,border-color 0.1s;padding:0 2px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .cap:hover{background:var(--accent-glow,#00c8ff33);color:var(--accent,#00c8ff);border-color:var(--accent-dim,#00c8ff44)}
  .cap:active{background:var(--accent,#00c8ff);color:var(--bg,#0a0e1a)}
  .cap.wide{font-size:8px;color:var(--fg-dim,#c8d8f055)}
  .cap.mod{color:var(--fg-dim,#c8d8f055);font-size:8px}
  .cap.mod.on{background:var(--accent,#00c8ff);border-color:var(--accent,#00c8ff);color:var(--bg,#0a0e1a)}
</style>
