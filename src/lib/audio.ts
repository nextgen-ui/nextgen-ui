import { writable } from 'svelte/store';

export type SoundEffect =
  | 'stdin'    // Key typing
  | 'granted'  // Enter / command executed / toggle on
  | 'stdout'   // Terminal stdout data burst
  | 'panels'   // Panel open / close / toggle
  | 'folder'   // Tab switch / add / close
  | 'theme'    // Theme change
  | 'keyboard' // On-screen keyboard open
  | 'denied'   // Close / cancel / toggle off
  | 'error';   // Error alert

export interface SoundState {
  enabled: boolean;
  volume: number; // 0.0 to 1.0
}

const STORAGE_KEY = 'nextgen_ui_sound';

const AUDIO_FILES: Record<SoundEffect, string> = {
  stdin: '/audio/stdin.wav',
  granted: '/audio/granted.wav',
  stdout: '/audio/stdout.wav',
  panels: '/audio/panels.wav',
  folder: '/audio/folder.wav',
  theme: '/audio/theme.wav',
  keyboard: '/audio/keyboard.wav',
  denied: '/audio/denied.wav',
  error: '/audio/error.wav',
};

function loadInitialState(): SoundState {
  if (typeof localStorage !== 'undefined') {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        return {
          enabled: typeof parsed.enabled === 'boolean' ? parsed.enabled : true,
          volume: typeof parsed.volume === 'number' ? Math.max(0, Math.min(1, parsed.volume)) : 0.6,
        };
      }
    } catch {}
  }
  return { enabled: true, volume: 0.6 };
}

const initial = loadInitialState();
export const soundState = writable<SoundState>(initial);

let ctx: AudioContext | null = null;
let currentEnabled = initial.enabled;
let currentVolume = initial.volume;
const buffers = new Map<SoundEffect, AudioBuffer>();
const loading = new Set<SoundEffect>();

soundState.subscribe((s) => {
  currentEnabled = s.enabled;
  currentVolume = s.volume;
  if (typeof localStorage !== 'undefined') {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(s));
    } catch {}
  }
});

function getAudioContext(): AudioContext | null {
  if (typeof window === 'undefined') return null;
  if (!ctx) {
    const AudioCtx =
      window.AudioContext ||
      (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
    if (AudioCtx) {
      ctx = new AudioCtx();
    }
  }
  if (ctx && ctx.state === 'suspended') {
    ctx.resume().catch(() => {});
  }
  return ctx;
}

// Preload audio and unlock audio context on initial user gesture
if (typeof window !== 'undefined') {
  preloadAll();
  const unlock = () => {
    const c = getAudioContext();
    if (c && c.state === 'suspended') {
      c.resume().catch(() => {});
    }
  };
  window.addEventListener('keydown', unlock, { capture: true });
  window.addEventListener('pointerdown', unlock, { capture: true });
}

async function loadBuffer(effect: SoundEffect): Promise<AudioBuffer | null> {
  if (buffers.has(effect)) return buffers.get(effect)!;
  if (loading.has(effect)) return null;

  const c = getAudioContext();
  if (!c) return null;

  loading.add(effect);
  try {
    const path = AUDIO_FILES[effect];
    const res = await fetch(path);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const arrayBuffer = await res.arrayBuffer();
    const audioBuffer = await c.decodeAudioData(arrayBuffer);
    buffers.set(effect, audioBuffer);
    loading.delete(effect);
    return audioBuffer;
  } catch {
    loading.delete(effect);
    return null;
  }
}

export function preloadAll() {
  for (const key of Object.keys(AUDIO_FILES) as SoundEffect[]) {
    loadBuffer(key);
  }
}

export function setSoundEnabled(enabled: boolean) {
  soundState.update((s) => ({ ...s, enabled }));
}

export function toggleSound(): boolean {
  let next = !currentEnabled;
  soundState.update((s) => {
    next = !s.enabled;
    return { ...s, enabled: next };
  });
  if (next) {
    playSound('granted');
  } else {
    playSound('denied');
  }
  return next;
}

export function setVolume(volume: number) {
  const clamped = Math.max(0, Math.min(1, volume));
  soundState.update((s) => ({ ...s, volume: clamped }));
}

// Throttle for high-frequency stdout data to prevent audio flooding
let lastStdoutTime = 0;

export function playSound(
  effect: SoundEffect,
  options?: { pitchVariance?: boolean; volumeScale?: number }
) {
  if (!currentEnabled || currentVolume <= 0) return;

  // Throttle stdout bursts like eDEX-UI (at least 35ms between triggers)
  if (effect === 'stdout') {
    const now = typeof performance !== 'undefined' ? performance.now() : Date.now();
    if (now - lastStdoutTime < 35) return;
    lastStdoutTime = now;
  }

  const c = getAudioContext();
  if (!c) return;

  const buf = buffers.get(effect);
  if (!buf) {
    loadBuffer(effect).then((loaded) => {
      if (loaded) playBuffer(c, loaded, options);
    });
    return;
  }

  playBuffer(c, buf, options);
}

function playBuffer(
  c: AudioContext,
  buf: AudioBuffer,
  options?: { pitchVariance?: boolean; volumeScale?: number }
) {
  try {
    const source = c.createBufferSource();
    source.buffer = buf;

    if (options?.pitchVariance) {
      // Subtle pitch variation (+/- 4%) so rapid typing sounds natural and dynamic
      source.playbackRate.value = 0.96 + Math.random() * 0.08;
    }

    const gain = c.createGain();
    const scale = options?.volumeScale ?? 1.0;
    gain.gain.value = currentVolume * scale;

    source.connect(gain);
    gain.connect(c.destination);

    source.start(0);
  } catch {}
}

/**
 * High-performance typing sound handler.
 * Called on every keystroke in terminal and on-screen keyboard.
 */
export function playKeySound(key: string) {
  if (key === 'Enter' || key === '\r') {
    playSound('granted', { volumeScale: 0.8 });
  } else {
    playSound('stdin', { pitchVariance: true, volumeScale: 0.7 });
  }
}
