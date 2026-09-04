# Changelog

All notable changes to nextgen-ui will be documented in this file.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

---

## [Unreleased]

### In Progress
- Phase 6: hardening, signing, public release
- StatsPanel render cost — the largest remaining idle-CPU item

---

## [1.1.0] — 2026-09-04

### Added
- On-screen keyboard — full US-ANSI layout writing to the active PTY, with
  one-shot Shift/Ctrl/Alt, latching Caps, C0 control codes and Alt-prefixed
  escapes. Toggle with `◰ KEYS` or Ctrl+Shift+K.
- Keyboard mirrors the physical keyboard, keyed on `KeyboardEvent.code` so it
  follows physical position rather than host layout. Pressed keys fill.
- Keyboard suppresses all highlighting while a password is being typed, so the
  panel cannot display a secret to anyone watching the screen. Detects both
  canonical-mode reads (getpass, `read -s`) and raw-mode readers such as sudo
  with `pwfeedback`.
- World-view globe widget — rotating point cloud of real coastlines, from a
  ~2.3KB land mask rasterised offline from Natural Earth 110m (public domain).
  Toggle with `◍ GLOBE` or Ctrl+Shift+G. Costs 0.4% of a CPU core.
- Process monitor with kill and CPU/MEM sort (Ctrl+Shift+P).
- WebGL terminal renderer via `@xterm/addon-webgl`, with fallback to the DOM
  renderer when WebGL is unavailable or the context is lost.
- First tests: 4 Rust (pty termios detection) and 8 TypeScript (secure-input
  state machine), both run in CI. `pnpm test` uses node's built-in runner.

### Changed
- HUD shortcuts moved from Ctrl to the Ctrl+Shift tier, so plain Ctrl stays
  with the shell (Ctrl+W deletes a word, Ctrl+P walks history, Ctrl+K kills
  the line) — the convention terminal emulators use.
- Crate and package renamed from `nextgen-ui-scaffold` to `nextgen-ui`, and
  version aligned to 1.1.0 across package.json, Cargo.toml and tauri.conf.json.
- Dropped `three`, which was declared but never imported.

### Performance
- Idle CPU cut from ~39% to ~10% of a core (release build, 30s windows).
- Process walk is gated on the process panel being open, asks only for CPU and
  memory rather than a full per-process refresh, and allocates strings only for
  the rows actually shown instead of for every process on the system.
- `get_system_stats` and `kill_process` no longer build whole-system snapshots.
- HudBackground caches its gradients instead of rebuilding five per frame,
  replaces per-line rgba strings with `globalAlpha`, batches 58 grid strokes
  into 6, and runs at 30fps — 5fps unfocused, stopped when hidden.
- `.dot-live` animated `box-shadow`, forcing a full repaint every frame; that
  one 6px dot cost ~6% of a CPU core. Now animates opacity.

### Fixed
- HUD shortcuts never fired while the terminal had focus — xterm.js calls
  `stopPropagation()` on keys it handles, so the window listener never saw
  them. Terminal now declines HUD keys via `attachCustomKeyEventHandler`.
- Tab close control was a `<span>` click handler nested inside the tab
  `<button>` — keyboard-unreachable and invalid HTML; now sibling buttons.
- `package.json` declared MIT while LICENSE, the README badge and the app
  footer all state AGPL-3.0. Now AGPL-3.0-only.
- Cleared all 25 svelte-check errors, 13 warnings and 2 clippy lints; CI
  enforces clippy, type-check and tests again.
- CI's Linux dependency list could not have built the app: `libdbus-1-dev` was
  missing and `libappindicator3-dev` is only transitional.

---

## [0.5.0] — 2026-08

### Added
- File manager panel (Rust fs backend + Svelte UI)
- Directory navigation with up button
- File size display per entry
- Config store — TOML-based, persisted via OS app config dir
- Keyboard shortcuts: Ctrl+T (new tab), Ctrl+W (close tab), Ctrl+B (toggle files), Ctrl+F (fullscreen)
- Keyboard hint bar in tab strip

---

## [0.4.0] — 2026-08

### Added
- Theme engine — CSS custom properties driven by TypeScript theme store
- 4 built-in themes: Default (cyan), TRON (white), Matrix (green), Blood (red)
- Theme switcher dropdown in header
- Multi-tab terminal — open unlimited shell sessions
- Tab add / close UI
- Corner bracket decorations on HUD edges

---

## [0.3.0] — 2026-08

### Added
- Live clock and date in header bar
- Session uptime counter
- CRT scanline overlay (CSS, zero GPU cost)
- Glow effects on title and borders
- Boot animation — typewriter sequence on startup
- Fullscreen toggle button (⊞/⊡)
- Animated LIVE indicator

---

## [0.2.0] — 2026-08

### Added
- PowerShell as default shell on Windows (pwsh.exe → powershell.exe fallback)
- Network IN/OUT stats (per interface, aggregated)
- Disk read/write stats
- Per-core CPU bar chart in stats panel
- NET and DISK sections in StatsPanel

### Fixed
- Shell spawn on Windows — CMD replaced with PowerShell

---

## [0.1.0] — 2026-08

### Added
- Tauri 2 + SvelteKit + Svelte 5 project scaffold
- Rust PTY manager via portable-pty (spawn, read, write, resize, close)
- IPC bridge — typed invoke() commands + event emitter
- Async Tokio runtime
- System stats via sysinfo — CPU overall + per-core, RAM, SWAP
- Stats streamed to frontend every 1s via Tauri events
- Basic sci-fi HUD layout — header, terminal panel, stats panel, footer
- xterm.js terminal renderer with sci-fi color theme
- Dev workflow established — pnpm tauri dev with hot-reload
