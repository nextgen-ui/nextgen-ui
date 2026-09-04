# Roadmap

**Project:** nextgen-ui  
**Last updated:** 2026-08

---

## Release philosophy

- All development happens on `dev`
- `main` receives merges only at milestone completions
- Each milestone is a tagged release (`v0.1.0`, `v0.2.0`, etc.)
- Public release at `v1.0.0` — repository visibility changes from private to public

---

## Phase 1 — Skeleton `v0.1.0` ✅ DONE

| Task | Status |
|---|---|
| Tauri 2 project scaffold | ✅ |
| Svelte 5 frontend scaffold | ✅ |
| IPC bridge — invoke() + events wired | ✅ |
| PTY manager (portable-pty) | ✅ |
| Basic terminal panel | ✅ |
| Tokio async runtime setup | ✅ |
| Basic system stats (CPU, RAM) via sysinfo | ✅ |
| Stats streamed to frontend via events | ✅ |
| TOML config file read/write | ✅ |
| CI workflow (build + test on push) | ✅ |

---

## Phase 2 — PTY + Stats Engine `v0.2.0` ✅ DONE

| Task | Status |
|---|---|
| PTY manager with PowerShell support | ✅ |
| PTY read/write/resize/close commands | ✅ |
| CPU per-core usage | ✅ |
| RAM used / total / swap | ✅ |
| Network IN/OUT per interface | ✅ |
| Disk read/write | ✅ |
| Stats history streamed via events | ✅ |

---

## Phase 3 — Sci-fi HUD `v0.3.0` ✅ DONE

| Task | Status |
|---|---|
| Live clock + date in header | ✅ |
| Session uptime counter | ✅ |
| CRT scanline overlay | ✅ |
| Glow effects on borders + title | ✅ |
| Boot animation typewriter sequence | ✅ |
| Fullscreen toggle button | ✅ |
| Corner bracket decorations | ✅ |
| Footer with stack info | ✅ |

---

## Phase 4 — Themes + Multi-tab `v0.4.0` ✅ DONE

| Task | Status |
|---|---|
| JSON theme system (CSS vars) | ✅ |
| Default (cyan) theme | ✅ |
| TRON (white) theme | ✅ |
| Matrix (green) theme | ✅ |
| Blood (red) theme | ✅ |
| Theme switcher dropdown | ✅ |
| Multi-tab terminal | ✅ |
| Tab add / close | ✅ |

---

## Phase 5 — File Manager + Shortcuts `v0.5.0` ✅ DONE

| Task | Status |
|---|---|
| File manager panel (Rust + Svelte) | ✅ |
| Directory navigation | ✅ |
| File size display | ✅ |
| Toggle file manager (Ctrl+B) | ✅ |
| Ctrl+T — new tab | ✅ |
| Ctrl+W — close tab | ✅ |
| Ctrl+F — fullscreen | ✅ |
| Config store (TOML, Rust) | ✅ |

---

## Phase 6 — Hardening + Public Release `v1.0.0` 🔲 IN PROGRESS

| Task | Status |
|---|---|
| App icon (custom nextgen-ui icon) | 🔲 |
| README.md finalized with screenshots | 🔲 |
| CHANGELOG.md updated | ✅ |
| All ADRs reviewed | ✅ |
| Merge dev → main | 🔲 |
| Tag v1.0.0 | ✅ |
| Repository made public | 🔲 |

---

## Phase 7 — Keyboard, Globe + Performance `v1.1.0` ✅ DONE

| Task | Status |
|---|---|
| Process monitor (top N by CPU, with kill) | ✅ |
| On-screen keyboard | ✅ |
| Physical key mirroring on the on-screen keyboard | ✅ |
| Password-prompt detection, display suppressed | ✅ |
| World-view globe (Natural Earth land mask) | ✅ |
| GPU terminal renderer (xterm WebGL addon) | ✅ |
| Shortcuts moved to the Ctrl+Shift tier | ✅ |
| Idle CPU ~39% → ~10% of a core | ✅ |
| First test suites, Rust + TypeScript, wired into CI | ✅ |

---

## Performance budget

Idle cost is a feature, and the main reason to prefer this over eDEX-UI.
Measured on the same machine, both idle:

| | eDEX-UI (Electron) | nextgen-ui (Tauri) |
|---|---|---|
| CPU | 36.4% | ~10% |
| RSS | 1460 MB | 452 MB |
| Processes | 15 | 3 |
| Binary | ~100 MB | 15.5 MB |

Largest remaining item: StatsPanel (~12.6% of a core) — ✅ Optimized via Canvas 2D and compositor transforms.

---

## Future (post-v1.1.0)

- SSH session manager
- Custom app icon
- Sound design (key clicks, panel tones)
- Split panes
- Network detail — public IP, ping, geolocation on the globe
- CPU temperature, battery, load average, disk capacity
- GPU monitoring (nvidia/amd)
- Settings GUI over the existing TOML config
- Command palette
- Container/Docker widget
- Plugin system (WASM)
- Mobile companion stats viewer
- AI command assistant (local, opt-in)
