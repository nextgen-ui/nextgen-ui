<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { ptyWrite, ptyInputState, onPtyOutput } from '$lib/ipc';
  import { SecureInputTracker } from '$lib/secureInput';
  import { playKeySound } from '$lib/audio';

  export let sessionId: string;

  interface Key {
    code: string;    // KeyboardEvent.code — physical position, layout independent
    lo?: string;     // unshifted character
    hi?: string;     // shifted character, drawn as a superscript
    seq?: string;    // raw byte sequence, sent verbatim
    mod?: Mod;       // modifier key rather than an input key
    label?: string;  // display override
    w?: number;      // relative width
    boxed?: boolean; // drawn with an outline, as eDEX does for Enter/Space/arrows
  }
  type Mod = 'shift' | 'ctrl' | 'alt' | 'caps';

  const k = (code: string, lo: string, hi?: string): Key => ({ code, lo, hi });
  const row = (codes: string, chars: string) =>
    chars.split('').map((c, i) => k(codes.split(',')[i], c));

  const rows: Key[][] = [
    [
      { code: 'Escape', label: 'ESC', seq: '\x1b', w: 1.8 },
      k('Backquote', '`', '~'), k('Digit1', '1', '!'), k('Digit2', '2', '@'),
      k('Digit3', '3', '#'), k('Digit4', '4', '$'), k('Digit5', '5', '%'),
      k('Digit6', '6', '^'), k('Digit7', '7', '&'), k('Digit8', '8', '*'),
      k('Digit9', '9', '('), k('Digit0', '0', ')'),
      k('Minus', '-', '_'), k('Equal', '=', '+'),
      { code: 'Backspace', label: 'BACK', seq: '\x7f', w: 2.2 },
    ],
    [
      { code: 'Tab', label: 'TAB', seq: '\t', w: 1.8 },
      ...row('KeyQ,KeyW,KeyE,KeyR,KeyT,KeyY,KeyU,KeyI,KeyO,KeyP', 'qwertyuiop'),
      k('BracketLeft', '[', '{'), k('BracketRight', ']', '}'),
      k('Backslash', '\\', '|'),
    ],
    [
      { code: 'CapsLock', label: 'CAPS', mod: 'caps', w: 2 },
      ...row('KeyA,KeyS,KeyD,KeyF,KeyG,KeyH,KeyJ,KeyK,KeyL', 'asdfghjkl'),
      k('Semicolon', ';', ':'), k('Quote', "'", '"'),
      { code: 'Enter', label: 'ENTER', seq: '\r', w: 2.4, boxed: true },
    ],
    [
      { code: 'ShiftLeft', label: 'SHIFT', mod: 'shift', w: 2.6 },
      ...row('KeyZ,KeyX,KeyC,KeyV,KeyB,KeyN,KeyM', 'zxcvbnm'),
      k('Comma', ',', '<'), k('Period', '.', '>'), k('Slash', '/', '?'),
      { code: 'ShiftRight', label: 'SHIFT', mod: 'shift', w: 2.6 },
    ],
    [
      { code: 'ControlLeft', label: 'CTRL', mod: 'ctrl', w: 1.8 },
      { code: 'AltLeft', label: 'ALT', mod: 'alt', w: 1.5 },
      { code: 'Space', lo: ' ', label: '', w: 8, boxed: true },
      { code: 'AltRight', label: 'ALT GR', mod: 'alt', w: 1.5 },
      { code: 'ControlRight', label: 'CTRL', mod: 'ctrl', w: 1.8 },
      { code: 'ArrowLeft', label: '←', seq: '\x1b[D', boxed: true },
      { code: 'ArrowUp', label: '↑', seq: '\x1b[A', boxed: true },
      { code: 'ArrowDown', label: '↓', seq: '\x1b[B', boxed: true },
      { code: 'ArrowRight', label: '→', seq: '\x1b[C', boxed: true },
    ],
  ];

  let shift = false, ctrl = false, alt = false, caps = false;
  let lastError = '';

  // ── Secure input ───────────────────────────────────────────────────────────
  // Lighting up keys as they are typed would display a password to anyone
  // watching the screen. The decision logic lives in SecureInputTracker, where
  // it is unit tested — see src/lib/secureInput.ts for why the two backend
  // signals are treated differently.
  const POLL_MS = 150;
  const PROMPT_RE = /(password|passphrase|pin)\b[^\n]{0,40}:/i;

  const tracker = new SecureInputTracker();
  let secure = false;
  let echoTimer: ReturnType<typeof setInterval>;
  let unlistenOutput: (() => void) | null = null;

  /** Physical keys currently held down, by KeyboardEvent.code. */
  let downCodes = new Set<string>();

  function applySecure(next: boolean) {
    if (next === secure) return;
    secure = next;
    if (secure && downCodes.size) downCodes = new Set();
  }

  /** A secret read ends when the line is submitted. */
  function endSecretRead() {
    applySecure(tracker.submitted());
  }

  async function poll() {
    try {
      applySecure(tracker.sample(await ptyInputState(sessionId)));
    } catch {
      applySecure(tracker.unknown());
    }
  }

  // ── Physical key mirroring ─────────────────────────────────────────────────
  // Registered in the capture phase: xterm.js calls stopPropagation() on keys it
  // handles, so a bubble-phase listener would never see most keystrokes.
  function onPhysicalDown(e: KeyboardEvent) {
    if (e.code === 'Enter' || e.code === 'NumpadEnter') endSecretRead();
    if (secure || !e.code) return;
    if (downCodes.has(e.code)) return;
    downCodes.add(e.code);
    downCodes = downCodes;
  }
  function onPhysicalUp(e: KeyboardEvent) {
    if (!downCodes.delete(e.code)) return;
    downCodes = downCodes;
  }
  const clearHeld = () => { if (downCodes.size) downCodes = new Set(); };

  onMount(async () => {
    window.addEventListener('keydown', onPhysicalDown, true);
    window.addEventListener('keyup', onPhysicalUp, true);
    window.addEventListener('blur', clearHeld);

    await poll();
    echoTimer = setInterval(poll, POLL_MS);

    unlistenOutput = await onPtyOutput(({ session_id, data }) => {
      if (session_id !== sessionId) return;
      if (PROMPT_RE.test(data)) applySecure(tracker.promptSeen());
    });
  });

  onDestroy(() => {
    window.removeEventListener('keydown', onPhysicalDown, true);
    window.removeEventListener('keyup', onPhysicalUp, true);
    window.removeEventListener('blur', clearHeld);
    clearInterval(echoTimer);
    unlistenOutput?.();
  });

  // ── Input ──────────────────────────────────────────────────────────────────
  function isMod(m: Mod) {
    return m === 'shift' ? shift : m === 'ctrl' ? ctrl : m === 'alt' ? alt : caps;
  }

  function toggleMod(m: Mod) {
    if (m === 'shift') shift = !shift;
    else if (m === 'ctrl') ctrl = !ctrl;
    else if (m === 'alt') alt = !alt;
    else caps = !caps;
  }

  /** The character this key produces given the current shift/caps state. */
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
    playKeySound(key.code === 'Enter' ? 'Enter' : (key.lo || key.code));
    if (key.mod) { toggleMod(key.mod); return; }
    if (key.code === 'Enter') endSecretRead();

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

  /** Main glyph for a cap. Letters are always drawn uppercase, as eDEX does. */
  function capLabel(key: Key): string {
    if (key.label !== undefined) return key.label;
    return (key.lo ?? '').toUpperCase();
  }
</script>

<div class="osk" class:secure>
  <div class="osk-head">
    <span class="osk-title">KEYBOARD</span>
    <span class="osk-target">{sessionId.toUpperCase()}</span>
    <span class="osk-mods">
      {#if ctrl}<span class="mod-live">CTRL</span>{/if}
      {#if alt}<span class="mod-live">ALT</span>{/if}
      {#if shift}<span class="mod-live">SHIFT</span>{/if}
      {#if caps}<span class="mod-live caps">CAPS</span>{/if}
    </span>
    {#if secure}
      <span class="osk-secure">◆ SECURE INPUT — DISPLAY SUPPRESSED</span>
    {:else if lastError}
      <span class="osk-err">{lastError}</span>
    {/if}
  </div>

  {#each rows as r}
    <div class="osk-row">
      {#each r as key}
        <button
          class="cap"
          class:mod={!!key.mod}
          class:on={key.mod ? isMod(key.mod) : false}
          class:down={downCodes.has(key.code)}
          class:boxed={key.boxed}
          class:glyph={key.label === undefined}
          style="flex:{key.w ?? 1}"
          aria-label={key.label || key.lo || key.code}
          aria-pressed={key.mod ? isMod(key.mod) : undefined}
          on:mousedown|preventDefault
          on:click={() => press(key)}
        >
          <span class="pair">
            {#if key.hi}<span class="hi">{key.hi}</span>{/if}
            <span class="lo">{capLabel(key)}</span>
          </span>
        </button>
      {/each}
    </div>
  {/each}
</div>

<style>
  /* eDEX-style: glyphs on bare background, outlines only where a key needs a
     shape of its own (Enter, Space, arrows). */
  .osk{display:flex;flex-direction:column;gap:7px;padding:10px 14px 12px;margin-top:6px;flex-shrink:0;background:transparent}
  .osk-head{display:flex;align-items:center;gap:10px;font-size:8px;letter-spacing:0.22em;color:var(--fg-dim,#c8d8f044);padding-bottom:2px}
  .osk-title{color:var(--accent-dim,#00c8ff88)}
  .osk-mods{display:flex;gap:5px;margin-left:auto}
  .mod-live{color:var(--accent,#00c8ff)}
  .mod-live.caps{color:var(--yellow,#ffcc00)}
  .osk-secure{color:var(--yellow,#ffcc00);letter-spacing:0.18em}
  .osk-err{color:var(--red,#ff4c6a)}

  .osk-row{display:flex;gap:6px;align-items:stretch}

  .cap{position:relative;flex:1;min-width:0;height:32px;display:flex;align-items:center;justify-content:center;
       background:none;border:none;border-radius:3px;padding:0;cursor:pointer;
       color:var(--fg-dim,#c8d8f066);font-family:inherit;font-size:14px;letter-spacing:0.1em;
       transition:color 0.09s,text-shadow 0.09s,border-color 0.09s}
  .cap.mod{font-size:9px;letter-spacing:0.16em}
  .cap.boxed{border:1px solid var(--panel-border,#00c8ff22)}
  .pair{position:relative;display:inline-block;line-height:1}
  .hi{position:absolute;right:100%;bottom:55%;margin-right:3px;font-size:8px;opacity:0.5;letter-spacing:0}

  .cap:hover{color:var(--fg,#c8d8f0);background:var(--accent-glow,#00c8ff18)}
  /* Pressed — physically or by click. The whole key fills so it reads at a
     glance; static, and only on the few keys actually down, so this never
     becomes a per-frame repaint. */
  .cap.down,.cap:active{background:var(--accent,#00c8ff);color:var(--bg,#0a0e1a);text-shadow:none;
                        box-shadow:0 0 14px var(--accent-glow,#00c8ff33)}
  .cap.down .hi,.cap:active .hi{opacity:0.75}
  .cap.boxed.down,.cap.boxed:active{border-color:var(--accent,#00c8ff)}
  .cap.mod.on{background:var(--accent,#00c8ff);color:var(--bg,#0a0e1a)}

  /* While a secret is being typed, nothing lights up at all. */
  .osk.secure .cap.down,
  .osk.secure .cap:active,
  .osk.secure .cap:hover{background:none;color:var(--fg-dim,#c8d8f066);text-shadow:none;box-shadow:none}
  .osk.secure .cap.boxed.down,
  .osk.secure .cap.boxed:active{border-color:var(--panel-border,#00c8ff22)}
</style>
