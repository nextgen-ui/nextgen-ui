# Changelog

All notable changes to nextgen-ui will be documented in this file.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

---

## [Unreleased]

### In Progress
- Phase 6: hardening, signing, public release

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
