# ADR 001 — Tauri 2 over Electron

**Date:** 2026-06  
**Status:** Accepted  
**Deciders:** @gokulgk4616

---

## Context

nextgen-ui requires a cross-platform desktop application shell that can:
- Render a WebGPU/WebGL-based UI
- Communicate with native OS APIs (PTY, system stats, keyring)
- Package and distribute on Linux, Windows, macOS
- Run without a Node.js runtime on the end user's machine

The primary reference implementation (eDEX-UI) used Electron. We evaluated whether to continue with Electron or switch to an alternative.

---

## Options considered

### Option 1 — Electron
- Ships Chromium + Node.js (~150MB baseline RAM)
- Startup time: 10–60 seconds depending on system
- Large binary (~150–200MB)
- Full Node.js access in renderer (security risk)
- Mature ecosystem, well-documented
- eDEX-UI's existing architecture is Electron-based

### Option 2 — Tauri 2
- Uses the OS's native WebView (no bundled Chromium)
- RAM baseline: ~15MB
- Startup time: sub-1 second
- Binary size: ~5–15MB
- Rust backend — memory safe, zero-cost abstractions
- Capability-based sandboxing model
- WebGPU supported via the native WebView on all 3 platforms
- Actively maintained (v2 released 2024)

### Option 3 — Native Rust UI (egui / iced)
- Purely native, no web tech
- Smallest footprint
- Very difficult to achieve the sci-fi aesthetic without massive custom work
- Would lose WebGPU shader flexibility

---

## Decision

**Tauri 2** is chosen as the application shell.

---

## Rationale

The defining requirement of nextgen-ui is that it must not repeat eDEX-UI's performance problems. Electron's Chromium bundle is the single largest source of eDEX-UI's RAM and startup time issues. Tauri 2 eliminates this entirely by using the OS's native WebView.

The Rust backend gives us memory safety, async I/O via Tokio, and strong typing across the IPC boundary. The capability sandbox model is strictly more secure than Electron's renderer model.

Tauri 2's WebView supports WebGPU on all three target platforms (Linux via Vulkan, Windows via DirectX 12, macOS via Metal), which is required for the GPU-accelerated terminal renderer.

---

## Consequences

- Developers need Rust installed (in addition to Node.js)
- WebView differences between OSes may cause minor rendering inconsistencies — must be tested per platform
- Some Electron-era npm packages for terminal emulation cannot be used directly — we use `xterm.js` (WebGL addon) as the terminal renderer, which works in any WebView
- Tauri 2's ecosystem is smaller than Electron's — some integrations require writing Rust code directly
