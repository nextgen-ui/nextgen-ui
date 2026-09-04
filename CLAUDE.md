# nextgen-ui

Sci-fi terminal emulator — successor to eDEX-UI. Built with Tauri 2 + Rust + Svelte 5.

Goal: everything eDEX-UI was iconic for, but genuinely lightweight and usable as
a daily driver. Idle cost is a feature — measure before and after any change
that runs continuously.

## Stack
- Backend: Rust + Tokio in src-tauri/
- Frontend: Svelte 5 + SvelteKit in src/
- Terminal: xterm.js (WebGL renderer) + portable-pty
- Stats: sysinfo crate
- Themes: CSS custom properties + JSON schema

## Modules
- src-tauri/src/pty/mod.rs — PTY manager; also termios/foreground-process
  inspection used to detect password prompts
- src-tauri/src/stats/mod.rs — system stats + process list (process walk is
  gated on the panel being open)
- src-tauri/src/fs/mod.rs — file system commands
- src-tauri/src/config/mod.rs — TOML config
- src/lib/components/Terminal.svelte — xterm.js terminal
- src/lib/components/StatsPanel.svelte — live stats with sparklines
- src/lib/components/ProcessMonitor.svelte — process list with kill
- src/lib/components/FileManager.svelte — file browser
- src/lib/components/HudBackground.svelte — animated canvas-2D backdrop
- src/lib/components/GlobeWidget.svelte — rotating point-cloud world view
- src/lib/components/Sparkline.svelte — history graph component
- src/lib/components/OnScreenKeyboard.svelte — on-screen keyboard, writes to
  active PTY, mirrors physical keys
- src/lib/secureInput.ts — decides when typed input must not be displayed
- src/lib/shortcuts.ts — HUD shortcut predicate, shared by page + Terminal
- src/lib/data/landmask.ts — packed Natural Earth land mask for the globe
- src/lib/stores/theme.ts — 4 themes: default, tron, matrix, blood
- src/lib/stores/stats.ts — stats store with history rings
- src/lib/ipc/index.ts — all Tauri invoke() calls centralized

## Dev commands
- pnpm tauri dev — start dev server with hot reload
- pnpm tauri build --no-bundle — release build without packaging (~20s incremental)
- cargo build --manifest-path src-tauri/Cargo.toml — build Rust only
- pnpm check — Svelte type check (must be 0 errors, 0 warnings)
- pnpm test — frontend tests (node built-in runner, type stripping)
- cargo test --manifest-path src-tauri/Cargo.toml — Rust tests
- cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings

Note: node/pnpm are under nvm and are not on the default PATH —
`export PATH="$HOME/.nvm/versions/node/v24.20.0/bin:$PATH"`.

Linux build deps include `libdbus-1-dev`, which is absent from Tauri's own
prerequisites page.

## Branch strategy
- main — stable releases (v1.0.0)
- dev — active development
- feature/* — short-lived feature branches, merged to dev with a merge commit

## Current version: v1.1.0
Already built:
- PTY terminal (PowerShell/bash)
- Live stats — CPU/RAM/NET/DISK with sparklines
- Process monitor with kill + CPU/MEM sort
- File manager panel
- 4 themes with live switcher
- Multi-tab terminal
- Animated HUD backdrop
- On-screen keyboard with physical-key mirroring and password suppression
- World-view globe widget
- Keyboard shortcuts: Ctrl+Shift+T/W/B/P/K/G/F (Shift tier avoids shadowing
  readline — plain Ctrl+W/P/B/F/K belong to the shell)

## Next features to build
1. StatsPanel render cost — largest remaining idle-CPU item (~12.6% of a core)
2. Custom app icon (still the stock Tauri icon)
3. SSH session manager

## Conventions
- Both quality gates must pass before committing: `pnpm check` at 0/0, and
  `cargo clippy -- -D warnings` clean.
- Never animate `box-shadow` or any property that forces repaint on an element
  that is always on screen; it costs whole percent of a CPU core.
- Anything drawn every frame caches what it can, batches draw calls, caps its
  frame rate, and stops when the page is hidden.
- Security-relevant logic (password detection) is unit tested rather than
  reasoned about.
