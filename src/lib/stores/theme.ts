import { writable } from 'svelte/store';

export interface Theme {
  name: string;
  accent: string;
  accentDim: string;
  accentGlow: string;
  bg: string;
  panelBg: string;
  panelBorder: string;
  fg: string;
  fgDim: string;
  green: string;
  red: string;
  yellow: string;
  purple: string;
  termBg: string;
  termFg: string;
  termCursor: string;
}

export const themes: Record<string, Theme> = {
  default: {
    name: 'DEFAULT',
    accent:      '#00c8ff',
    accentDim:   '#00c8ff44',
    accentGlow:  '#00c8ff33',
    bg:          '#0a0e1a',
    panelBg:     '#0d1220ee',
    panelBorder: '#00c8ff44',
    fg:          '#c8d8f0',
    fgDim:       '#c8d8f055',
    green:       '#00ffb3',
    red:         '#ff4c6a',
    yellow:      '#ffcc00',
    purple:      '#b06fff',
    termBg:      '#060910',
    termFg:      '#c8d8f0',
    termCursor:  '#00c8ff',
  },
  tron: {
    name: 'TRON',
    accent:      '#f8f8f8',
    accentDim:   '#f8f8f844',
    accentGlow:  '#f8f8f822',
    bg:          '#000000',
    panelBg:     '#080808ee',
    panelBorder: '#f8f8f833',
    fg:          '#e8e8e8',
    fgDim:       '#e8e8e855',
    green:       '#f8f8f8',
    red:         '#ff3300',
    yellow:      '#ffaa00',
    purple:      '#cc88ff',
    termBg:      '#000000',
    termFg:      '#e8e8e8',
    termCursor:  '#ffffff',
  },
  matrix: {
    name: 'MATRIX',
    accent:      '#00ff41',
    accentDim:   '#00ff4144',
    accentGlow:  '#00ff4122',
    bg:          '#000300',
    panelBg:     '#010a01ee',
    panelBorder: '#00ff4133',
    fg:          '#00cc33',
    fgDim:       '#00cc3355',
    green:       '#00ff41',
    red:         '#ff0000',
    yellow:      '#aaff00',
    purple:      '#00ffaa',
    termBg:      '#000300',
    termFg:      '#00cc33',
    termCursor:  '#00ff41',
  },
  blood: {
    name: 'BLOOD',
    accent:      '#ff2244',
    accentDim:   '#ff224444',
    accentGlow:  '#ff224422',
    bg:          '#0d0005',
    panelBg:     '#140008ee',
    panelBorder: '#ff224433',
    fg:          '#ffaaaa',
    fgDim:       '#ffaaaa55',
    green:       '#ff6644',
    red:         '#ff2244',
    yellow:      '#ff8800',
    purple:      '#ff44aa',
    termBg:      '#0a0003',
    termFg:      '#ffaaaa',
    termCursor:  '#ff2244',
  },
};

export const currentTheme = writable<Theme>(themes.default);
export const currentThemeName = writable<string>('default');

export function setTheme(name: string) {
  if (themes[name]) {
    currentTheme.set(themes[name]);
    currentThemeName.set(name);
    applyThemeCssVars(themes[name]);
  }
}

export function applyThemeCssVars(t: Theme) {
  const root = document.documentElement;
  root.style.setProperty('--accent',       t.accent);
  root.style.setProperty('--accent-dim',   t.accentDim);
  root.style.setProperty('--accent-glow',  t.accentGlow);
  root.style.setProperty('--bg',           t.bg);
  root.style.setProperty('--panel-bg',     t.panelBg);
  root.style.setProperty('--panel-border', t.panelBorder);
  root.style.setProperty('--fg',           t.fg);
  root.style.setProperty('--fg-dim',       t.fgDim);
  root.style.setProperty('--green',        t.green);
  root.style.setProperty('--red',          t.red);
  root.style.setProperty('--yellow',       t.yellow);
  root.style.setProperty('--purple',       t.purple);
}
