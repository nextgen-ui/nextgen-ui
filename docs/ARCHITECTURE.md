# Architecture

**Document:** ARCH-001  
**Version:** 0.1  
**Status:** Active  
**Last updated:** 2026-06

---

## Overview

nextgen-ui is a fullscreen terminal emulator and system monitor built on Tauri 2 (Rust backend) and Svelte 5 (frontend). The architecture is designed around three core principles:

1. **Performance first** — no Electron, no Node.js runtime, no unnecessary overhead
2. **Security by default** — Tauri capability sandboxing, WASM plugin isolation, OS keyring for secrets
3. **Extensibility** — JSON-driven themes, WASM plugin host, typed IPC API

---

## Layer diagram

```
┌─────────────────────────────────────────────────────────────┐
│              UI Layer (Svelte 5 + WebGPU/WebGL)             │
│  HUD Panels │ Terminal View │ File Manager │ Stats Dashboard │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│         Theme Engine (JSON schema · WGSL shaders)           │
│   Theme Loader │ Shader Pipeline │ Animation Throttler       │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│              Tauri 2 IPC Bridge                             │
│   invoke() handlers │ Event emitter │ Capability sandbox    │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                   Rust Core (Tokio async)                   │
│  PTY Manager │ Stats Collector │ Plugin Host │ FS Manager   │
│  Config Store │ Secrets Vault │ Auto-updater │ Crash Report │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                 OS / Platform Layer                         │
│   Linux (X11/Wayland) │ Windows (ConPTY) │ macOS (Metal)   │
└─────────────────────────────────────────────────────────────┘
```

---

## Layer breakdown

### UI layer — Svelte 5 + WebGPU/WebGL

The frontend is written in Svelte 5. It is responsible for all visual rendering, user interaction, and state management. It never calls OS APIs directly — everything goes through the IPC bridge.

**Key components:**

| Component | Responsibility |
|---|---|
| `HUD.svelte` | Outer sci-fi frame, panel layout, global keyboard shortcuts |
| `Terminal.svelte` | Terminal panel, tab management, input handling |
| `StatsPanel.svelte` | Real-time CPU, RAM, NET, DISK, GPU display |
| `FileManager.svelte` | File tree navigation, preview pane |
| `Keyboard.svelte` | On-screen keyboard for touch/display usage |

**Rendering pipeline:**
- Text glyphs are rendered via WebGPU (fallback: WebGL) for GPU-accelerated terminal output
- All animations are capped at 60fps via `requestAnimationFrame` budget
- When the window is minimized or hidden, the render loop pauses entirely (0% GPU/CPU)

---

### Theme engine

Themes are JSON files validated against `themes/schema.json`. No raw CSS injection. The theme engine:

- Loads and validates a theme JSON on startup
- Hot-reloads when the file changes on disk (dev mode)
- Applies color tokens, font choices, shader parameters, and panel layout via CSS custom properties
- WGSL shader files are referenced by path from the theme JSON and compiled at runtime

This approach replaces eDEX-UI's fragile CSS injection with a structured, type-safe system.

---

### Tauri 2 IPC bridge

All communication between the Svelte frontend and the Rust backend goes through Tauri's IPC system.

**Commands (frontend → backend):** Called via `invoke('command_name', { args })`. Each command is a Rust async function decorated with `#[tauri::command]`. Commands are registered in `main.rs` and scoped via capability files.

**Events (backend → frontend):** The Rust backend emits typed events via `app.emit()`. The frontend subscribes with `listen('event_name', handler)`. Used for streaming data — stats updates, PTY output, file system changes.

**Capability sandboxing:** Tauri 2's capability model restricts which IPC commands each window can call. No window has unrestricted access to the Rust backend. This is configured in `src-tauri/capabilities/`.

---

### Rust core

The Rust backend runs on the Tokio async runtime. All I/O is non-blocking. Modules:

| Module | Crate(s) | Responsibility |
|---|---|---|
| PTY manager | `portable-pty` | Spawn and manage terminal sessions cross-platform |
| Stats collector | `sysinfo` | Poll CPU, RAM, NET, DISK at configurable interval |
| Plugin host | `wasmtime` | Load and execute WASM plugins in isolation |
| FS manager | `tokio::fs` | Sandboxed file system operations for file manager |
| Config store | `toml`, `notify` | Read/write TOML config, watch for changes |
| Secrets vault | `keyring` | Store sensitive data in OS keyring |
| Auto-updater | Tauri updater | Check for and apply signed updates |
| Crash reporter | `sentry` | Opt-in crash reporting |

**Design rules:**
- Zero `unsafe` blocks in business logic — only in FFI boundaries if unavoidable
- All public functions are async
- All errors are typed — no `.unwrap()` in production paths

---

### OS / platform layer

| Platform | PTY | GPU | Package format |
|---|---|---|---|
| Linux | `openpty` (POSIX) | Vulkan via WebGPU | `.deb`, `.AppImage` |
| Windows | ConPTY (Windows 10+) | DirectX 12 via WebGPU | `.exe` (NSIS), `.msi` |
| macOS | `openpty` (POSIX) | Metal via WebGPU | `.dmg`, notarized |

---

## Data flow — keypress to terminal output

```
User presses key
    → Terminal.svelte captures keydown event
    → invoke('pty_write', { session_id, data })
    → Rust PTY manager writes to PTY master fd
    → Shell process receives input, produces output
    → Rust reads PTY master fd output
    → app.emit('pty_output', { session_id, data })
    → Terminal.svelte receives event
    → GPU renderer draws new glyphs to canvas
```

---

## Key optimizations vs eDEX-UI

| Metric | eDEX-UI | nextgen-ui |
|---|---|---|
| RAM baseline | ~150MB (Electron) | ~15MB (Tauri) |
| Startup time | ~60 seconds | < 1 second |
| Idle CPU | 5–15% | ~0% (render paused) |
| Binary size | ~200MB | ~10MB |
| Node.js dependency | Required | None |
| Security model | Electron (permissive) | Tauri capability sandbox |

---

## Related documents

- [ROADMAP.md](ROADMAP.md)
- [decisions/001-tauri-over-electron.md](decisions/001-tauri-over-electron.md)
- [decisions/002-svelte-over-react.md](decisions/002-svelte-over-react.md)
- [decisions/003-wasm-plugin-sandbox.md](decisions/003-wasm-plugin-sandbox.md)
