# nextgen-ui

> A next-generation, GPU-accelerated terminal emulator and system monitor with a sci-fi interface — built with Tauri 2 + Rust + Svelte 5.

![Status](https://img.shields.io/badge/status-active%20development-orange)
![License](https://img.shields.io/badge/license-AGPL--3.0-blue)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey)
![Phase](https://img.shields.io/badge/phase-6%20of%206-green)

---

## What is nextgen-ui?

nextgen-ui is a spiritual successor to [eDEX-UI](https://github.com/GitSquared/edex-ui), fixing every major problem of the original while adding new features.

| Problem in eDEX-UI | nextgen-ui solution |
|---|---|
| ~150MB+ RAM (Electron) | ~15MB baseline (Tauri 2) |
| High CPU even when idle | Render loop pauses on idle |
| ~60 second startup | Sub-1 second startup |
| Abandoned since 2021 | Actively maintained |
| CMD-only on Windows | PowerShell (pwsh / powershell.exe) |
| No plugin system | WASM plugin host (planned v1.1) |
| Raw CSS theming (fragile) | JSON + CSS var theme engine |
| Single terminal session | Multi-tab terminal |
| No file manager | Built-in sci-fi file manager |
| No keyboard shortcuts | Ctrl+Shift+T/W/B/P/K/F shortcuts |

---

## Features

### Terminal
- Full PTY terminal (PowerShell on Windows, bash/zsh on Linux/macOS)
- Multi-tab sessions — unlimited shells, Ctrl+Shift+T to open, Ctrl+Shift+W to close
- xterm.js renderer with sci-fi color themes
- Resize-aware — PTY cols/rows sync on window resize

### System Monitor
- Live CPU usage — overall + per-core bar chart
- RAM and SWAP usage with progress bars
- Network IN/OUT (KB/s per second)
- Disk read/write (KB/s per second)
- Updates every 1 second via Rust event emitter

### File Manager
- Browse filesystem in a sci-fi tree panel
- Click directories to navigate, `↑` to go up
- File sizes displayed per entry
- Toggle with `◫ FILES` button or Ctrl+Shift+B

### Sci-fi HUD
- Live clock, date, session uptime in header
- Boot animation typewriter sequence on startup
- CRT scanline overlay
- Glow effects on borders and title
- Corner bracket decorations
- Animated LIVE indicator
- Fullscreen toggle (Ctrl+Shift+F)

### Themes
- **DEFAULT** — cyan on dark blue
- **TRON** — white on black
- **MATRIX** — green on near-black
- **BLOOD** — red on dark crimson
- Hot-swap themes at runtime via dropdown

### Keyboard Shortcuts
| Shortcut | Action |
|---|---|
| Ctrl+Shift+T | New terminal tab |
| Ctrl+Shift+W | Close current tab |
| Ctrl+Shift+B | Toggle file manager |
| Ctrl+Shift+P | Toggle process monitor |
| Ctrl+Shift+K | Toggle on-screen keyboard |
| Ctrl+Shift+F | Toggle fullscreen |

Shortcuts use the Ctrl+Shift tier so plain Ctrl stays with the shell — Ctrl+W
still deletes a word, Ctrl+P still walks history, Ctrl+K still kills the line.

---

## Tech stack

| Layer | Technology |
|---|---|
| App shell | Tauri 2 |
| Backend | Rust + Tokio (async) |
| Frontend | Svelte 5 + SvelteKit |
| Terminal | xterm.js + portable-pty |
| Stats | sysinfo crate |
| Config | TOML |
| Themes | CSS custom properties |

---

## Getting started

### Prerequisites
- [Rust](https://rustup.rs) 1.78+
- [Node.js](https://nodejs.org) 20+
- [pnpm](https://pnpm.io) 9+
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS

### Dev mode
```bash
git clone https://github.com/nextgen-ui/nextgen-ui.git
cd nextgen-ui
pnpm install
pnpm tauri dev
```

### Build
```bash
pnpm tauri build
```

---

## Project status

| Phase | Description | Status |
|---|---|---|
| 1 | Tauri + Svelte scaffold, PTY, basic stats | ✅ Done |
| 2 | PowerShell, NET/DISK stats | ✅ Done |
| 3 | Sci-fi HUD, clock, scanlines, animations | ✅ Done |
| 4 | Themes, multi-tab terminal | ✅ Done |
| 5 | File manager, config, keyboard shortcuts | ✅ Done |
| 6 | Hardening, signing, public release | 🔲 In progress |

See [ROADMAP.md](docs/ROADMAP.md) for full detail.

---

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Contributing](docs/CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)
- [ADR: Tauri over Electron](docs/decisions/001-tauri-over-electron.md)
- [ADR: Svelte over React](docs/decisions/002-svelte-over-react.md)
- [ADR: WASM plugin sandbox](docs/decisions/003-wasm-plugin-sandbox.md)

---

## License

[AGPL-3.0](LICENSE) — Any derivative work, including forks run as a service, must remain open source.

---

## Roadmap highlights (post-v1.0.0)

- GPU-accelerated WebGPU terminal renderer
- SSH session manager
- WASM plugin system
- Process list panel
- AI command assistant (local, opt-in)
