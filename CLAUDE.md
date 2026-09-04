# nextgen-ui

Sci-fi terminal emulator — successor to eDEX-UI. Built with Tauri 2 + Rust + Svelte 5.

## Stack
- Backend: Rust + Tokio in src-tauri/
- Frontend: Svelte 5 + SvelteKit in src/
- Terminal: xterm.js + portable-pty
- Stats: sysinfo crate
- Themes: CSS custom properties + JSON schema

## Modules
- src-tauri/src/pty/mod.rs — PTY manager
- src-tauri/src/stats/mod.rs — system stats + process list
- src-tauri/src/fs/mod.rs — file system commands
- src-tauri/src/config/mod.rs — TOML config
- src/lib/components/Terminal.svelte — xterm.js terminal
- src/lib/components/StatsPanel.svelte — live stats with sparklines
- src/lib/components/ProcessMonitor.svelte — process list with kill
- src/lib/components/FileManager.svelte — file browser
- src/lib/components/HudBackground.svelte — WebGL animated background
- src/lib/components/Sparkline.svelte — history graph component
- src/lib/components/OnScreenKeyboard.svelte — on-screen keyboard, writes to active PTY
- src/lib/shortcuts.ts — HUD shortcut predicate, shared by page + Terminal
- src/lib/stores/theme.ts — 4 themes: default, tron, matrix, blood
- src/lib/stores/stats.ts — stats store with history rings
- src/lib/ipc/index.ts — all Tauri invoke() calls centralized

## Dev commands
- pnpm tauri dev — start dev server with hot reload
- cargo build --manifest-path src-tauri/Cargo.toml — build Rust only
- pnpm check — Svelte type check

## Branch strategy
- main — stable releases (v1.0.0 current)
- dev — active development (v1.1.0 in progress)
- feature/* — short-lived feature branches

## Current version: v1.1.0 (dev)
Already built:
- PTY terminal (PowerShell/bash)
- Live stats — CPU/RAM/NET/DISK with sparklines
- Process monitor with kill + CPU/MEM sort
- File manager panel
- 4 themes with live switcher
- Multi-tab terminal
- WebGL animated background
- On-screen keyboard (Ctrl+Shift+K)
- Keyboard shortcuts: Ctrl+Shift+T/W/B/P/K/F (Shift tier avoids shadowing readline)

## Next features to build
1. Custom app icon
2. SSH session manager
