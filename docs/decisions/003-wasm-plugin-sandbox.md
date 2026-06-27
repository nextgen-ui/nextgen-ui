# ADR 003 — WASM sandbox for plugin system

**Date:** 2026-06  
**Status:** Accepted  
**Deciders:** @gokulgk4616

---

## Context

nextgen-ui needs a plugin system that allows third-party developers to add panels, commands, and UI extensions. The plugin system must be:

- Safe — a malicious or buggy plugin must not be able to crash the host app or access unauthorized resources
- Cross-platform — plugins ship once, run on Linux/Windows/macOS
- Language-agnostic — plugin authors should not be locked into one language

---

## Options considered

### Option 1 — Node.js child processes
- Plugins run as separate Node.js processes
- Requires Node.js runtime on the user's machine (conflicts with our no-Node-runtime goal)
- IPC via stdin/stdout or sockets — high overhead
- OS process isolation — crash in plugin doesn't crash host
- No language choice for plugin authors (JS/TS only)

### Option 2 — Shared library (.so / .dll) loading
- Plugins compiled as native shared libraries, loaded via `dlopen`
- Maximum performance, direct Rust ABI
- Zero isolation — a crash or memory error in a plugin crashes the host process
- Platform-specific compilation required per OS
- Unacceptable security risk for user-installed plugins

### Option 3 — WebAssembly (WASM) via wasmtime
- Plugins compiled to WASM — runs on any platform without recompilation
- Full memory isolation — plugin runs in its own linear memory space
- CPU and memory limits enforceable at the host level
- Crash in plugin is caught by the WASM runtime, host continues
- Language-agnostic — Rust, Go, C, AssemblyScript all compile to WASM
- Explicit capability grants — plugin can only access what the host explicitly provides via the guest API

---

## Decision

**WebAssembly via `wasmtime`** is chosen as the plugin execution environment.

---

## Rationale

Security is non-negotiable for user-installable plugins. A plugin that can crash the host or access the filesystem without permission would make the plugin system a liability rather than a feature.

WASM's memory isolation model solves this cleanly — each plugin runs in its own linear memory space with no access to host memory. The `wasmtime` runtime enforces this at the hardware level via memory bounds checking.

The capability grant model (host explicitly imports functions into the WASM module) means plugins can only call what we allow — file access, network calls, and system commands require explicit opt-in by the user at install time.

Cross-platform compatibility without per-OS builds is an operational advantage — plugin authors compile to WASM once, and it runs identically on Linux, Windows, and macOS.

---

## Consequences

- Plugin performance is slightly lower than native code — acceptable for UI panels and command extensions, not suitable for compute-intensive tasks (we will document this limit)
- WASM guest API must be designed carefully — it is a stable ABI that we cannot easily break after v1.0.0
- Plugin authors need a WASM-capable toolchain — Rust is the primary supported language, with a provided SDK (`plugins/sdk/`)
- `wasmtime` adds ~8MB to the binary size
- Debugging WASM plugins is harder than native code — we will invest in plugin dev tooling post-v1.0.0
