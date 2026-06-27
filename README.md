# nextgen-ui

> A next-generation, GPU-accelerated terminal emulator and system monitor with a sci-fi interface — built with Tauri 2 + Rust + Svelte.

![Status](https://img.shields.io/badge/status-in%20development-orange)
![License](https://img.shields.io/badge/license-AGPL--3.0-blue)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey)

---

## What is nextgen-ui?

nextgen-ui is a spiritual successor to [eDEX-UI](https://github.com/GitSquared/edex-ui), addressing every major limitation of the original:

| Problem in eDEX-UI | nextgen-ui solution |
|---|---|
| ~150MB+ RAM (Electron) | ~15MB baseline (Tauri 2) |
| High CPU even when idle | Render loop pauses on idle |
| ~60 second startup time | Sub-1 second startup |
| Abandoned since 2021 | Actively maintained |
| No plugin system | WASM sandboxed plugin host |
| Raw CSS theming (fragile) | JSON schema-driven themes |
| Old Electron, security holes | Tauri 2 capability sandboxing |
| Single-threaded boot | Async Tokio parallel boot |

---

## Features (planned)

- GPU-accelerated terminal rendering (WebGPU / WebGL)
- Real-time system stats — CPU, RAM, GPU, NET, DISK
- Modular sci-fi HUD panels
- Integrated file manager
- On-screen keyboard
- WASM plugin system
- JSON theme engine with hot-reload
- Cross-platform — Linux, Windows, macOS
- Multi-tab terminal sessions
- OS keyring integration for secrets

---

## Tech stack

| Layer | Technology |
|---|---|
| App shell | Tauri 2 |
| Backend | Rust + Tokio (async) |
| Frontend | Svelte 5 |
| Rendering | WebGPU / WebGL fallback |
| Plugins | WebAssembly (WASM) sandbox |
| Config | TOML + hot-reload |
| Secrets | OS keyring + AES-GCM |

---

## Project status

Currently in **Phase 1 — Skeleton** (see [ROADMAP.md](docs/ROADMAP.md)).

This repository is private during development. It will be made public at **v1.0.0**.

---

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Contributing](docs/CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)
- [Decisions](docs/decisions/)

---

## License

[AGPL-3.0](LICENSE) — Any derivative work, including forks run as a service, must remain open source.
