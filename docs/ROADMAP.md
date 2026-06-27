# Roadmap

**Project:** nextgen-ui  
**Last updated:** 2026-06

---

## Release philosophy

- All development happens on `dev`
- `main` receives merges only at milestone completions
- Each milestone is a tagged release (`v0.1.0`, `v0.2.0`, etc.)
- Public release at `v1.0.0` — repository visibility changes from private to public

---

## Phase 1 — Skeleton `v0.1.0`

**Goal:** A working terminal. Nothing fancy, everything functional.

| Task | Status |
|---|---|
| Tauri 2 project scaffold | 🔲 |
| Svelte 5 frontend scaffold | 🔲 |
| IPC bridge — invoke() + events wired | 🔲 |
| PTY manager (portable-pty) | 🔲 |
| Basic terminal panel (no GPU yet) | 🔲 |
| Tokio async runtime setup | 🔲 |
| Basic system stats (CPU, RAM) via sysinfo | 🔲 |
| Stats streamed to frontend via events | 🔲 |
| TOML config file read/write | 🔲 |
| CI workflow (build + test on push) | 🔲 |

**Done when:** You can open nextgen-ui, see a working terminal, type commands, and see live CPU/RAM stats.

---

## Phase 2 — GPU Renderer `v0.2.0`

**Goal:** Terminal output rendered on the GPU. Smooth, fast, no jank.

| Task | Status |
|---|---|
| WebGPU glyph atlas built | 🔲 |
| Terminal renderer ported to WebGPU | 🔲 |
| WebGL fallback renderer | 🔲 |
| Frame rate cap (60fps) with RAF budget | 🔲 |
| Render loop pause on window hide/minimize | 🔲 |
| Font rendering quality pass | 🔲 |
| Benchmark vs Phase 1 (CPU/RAM delta) | 🔲 |

**Done when:** Terminal renders at 60fps with GPU, idles at ~0% CPU when minimized.

---

## Phase 3 — Stats Engine `v0.3.0`

**Goal:** Full real-time system monitoring — all metrics, all platforms.

| Task | Status |
|---|---|
| CPU per-core usage | 🔲 |
| RAM used / total / swap | 🔲 |
| Network IN/OUT per interface | 🔲 |
| Disk read/write per device | 🔲 |
| GPU usage + VRAM (where available) | 🔲 |
| Process list (top N by CPU) | 🔲 |
| Configurable poll interval (TOML) | 🔲 |
| Stats history ring buffer (for graphs) | 🔲 |
| Frontend sparkline graphs per metric | 🔲 |

**Done when:** All stats visible in real time, matching htop/Task Manager accuracy within 2%.

---

## Phase 4 — Sci-fi UI + Themes `v0.4.0`

**Goal:** The eDEX-UI aesthetic, done properly and performantly.

| Task | Status |
|---|---|
| JSON theme schema defined + validator | 🔲 |
| Default dark sci-fi theme | 🔲 |
| TRON-inspired theme | 🔲 |
| Matrix theme | 🔲 |
| Theme hot-reload (dev mode) | 🔲 |
| WGSL shader pipeline for HUD effects | 🔲 |
| Modular HUD panel layout system | 🔲 |
| On-screen keyboard component | 🔲 |
| Panel drag-to-resize | 🔲 |
| Animation throttler (idle detection) | 🔲 |

**Done when:** nextgen-ui looks unmistakably sci-fi, themes are swappable at runtime, animations don't spike CPU.

---

## Phase 5 — File Manager + Plugin Host `v0.5.0`

**Goal:** Usable file manager and a working plugin system.

| Task | Status |
|---|---|
| File tree component (Svelte) | 🔲 |
| File preview pane | 🔲 |
| Sandboxed FS operations (Rust) | 🔲 |
| WASM plugin host (wasmtime) | 🔲 |
| Plugin SDK (guest API) | 🔲 |
| Example plugin (hello-world) | 🔲 |
| Plugin manifest format (JSON) | 🔲 |
| Plugin loading from config dir | 🔲 |
| OS keyring integration (secrets vault) | 🔲 |

**Done when:** File manager works, one example plugin loads and renders a panel.

---

## Phase 6 — Hardening + Public Release `v1.0.0`

**Goal:** Production quality. Signed, auto-updating, crash-reported, documented.

| Task | Status |
|---|---|
| Code signing — Linux, Windows, macOS | 🔲 |
| Auto-updater (Tauri updater, signed) | 🔲 |
| Crash reporter (Sentry, opt-in) | 🔲 |
| Full CONTRIBUTING.md written | 🔲 |
| README.md finalized with screenshots | 🔲 |
| All ADRs written and reviewed | 🔲 |
| Security audit pass | 🔲 |
| Performance benchmark suite | 🔲 |
| Release workflow (.deb, .exe, .dmg) | 🔲 |
| Repository made public | 🔲 |

**Done when:** v1.0.0 tagged, binaries attached to GitHub Release, repo public.

---

## Future (post-v1.0.0)

- SSH session manager
- Multi-pane terminal layouts
- Plugin marketplace
- Mobile companion app (stats viewer)
- AI command assistant (local, opt-in)
- Remote stats monitoring
