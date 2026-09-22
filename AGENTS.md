# nextgen-ui — agent handoff

Canonical context for any AI assistant or new contributor working on this
repo. `CLAUDE.md` and `GEMINI.md` point here; keep this file as the single
source of truth and update it as things change.

Last updated: 2026-09-22. Open branches: `fix/security-hardening`,
`feature/hardware-telemetry` (both local, unmerged, unpushed).
**Starting a new session? Read §2 and §11 first** — §11 is the running handoff
log and says exactly what is in flight and what to do next.

---

## 1. What this project is

A sci-fi terminal emulator, successor to [eDEX-UI](https://github.com/GitSquared/edex-ui)
(archived 2021, Electron). Tauri 2 + Rust + Svelte 5.

**The thesis: everything eDEX-UI was iconic for, but genuinely lightweight and
usable as a daily driver.** eDEX-UI was a showpiece people screenshotted rather
than a terminal they lived in, and it was abandoned partly to Electron
maintenance burden. Idle cost is therefore a *feature*, not an afterthought —
it is the main reason to prefer this over the original.

Measured on the same machine, both idle, both unfocused, September 2026:

| | eDEX-UI (Electron) | nextgen-ui (Tauri) |
|---|---|---|
| CPU | 36.4% of a core | ~10% |
| RSS | 1460 MB | ~450–590 MB |
| Processes | 15 | 3 |
| Binary | ~100 MB AppImage | 15.5 MB |

Do not let that regress. See §7.

---

## 2. Current status

**v1.1.0 is tagged and pushed.** `dev` is the active branch; `main` is still at
`v1.0.0` and was deliberately not advanced. Three features landed on `dev` after
the tag (sound, network/geo, command palette) and are unreleased. CI on `dev`
has been green on every push since 2026-09-04.

Built and working:

- PTY terminal (bash/PowerShell), multi-tab
- Live stats — CPU per-core, RAM, swap, NET, DISK, with sparklines
- Process monitor with kill and CPU/MEM sort
- File manager panel
- 4 themes with a live switcher
- Animated HUD backdrop (canvas 2D — *not* WebGL, despite the footer text)
- On-screen keyboard: eDEX-style transparent layout, mirrors the physical
  keyboard, suppresses all display while a password is typed
- World-view globe: rotating point cloud of real coastlines
- WebGL terminal renderer (`@xterm/addon-webgl`) with DOM fallback
- Sound FX (Web Audio, `src/lib/audio.ts`) — keypress/UI sounds, pitch
  variance, rate-throttled, zero idle cost
- Network detail — TCP ping, public/local IP, interface detection
  (`src-tauri/src/net`), plus a pulsing geolocation pin on the globe
- Command palette + settings GUI (Ctrl+Shift+C) — fuzzy search, keyboard nav,
  font/volume sliders, persisted to TOML

In flight (uncommitted, then committed to `feature/hardware-telemetry` — see
§11): hardware telemetry in the stats panel — CPU/GPU temperature, load
average, per-mount disk capacity, battery, system uptime.

Not built yet — see §8.

---

## 3. Repo map

```
src-tauri/src/
  lib.rs              Tauri builder, command registration, managed state
  pty/mod.rs          PTY manager; also termios + foreground-process
                      inspection used to detect password prompts (§6.4)
  stats/mod.rs        System stats + process list. The process walk is gated
                      on the panel being open — see §7.
  fs/mod.rs           Directory listing
  config/mod.rs       TOML config, persisted to the OS app config dir
  net/mod.rs          Ping, public/local IP, interface detection
src/lib/
  components/         Terminal, StatsPanel, ProcessMonitor, FileManager,
                      HudBackground, GlobeWidget, Sparkline, OnScreenKeyboard,
                      CommandPalette
  audio.ts            Web Audio sound FX engine
  secureInput.ts      Decides when typed input must not be displayed. Unit
                      tested — do not inline this logic back into the component
  shortcuts.ts        HUD shortcut predicate, shared by the page and Terminal
  data/landmask.ts    Packed Natural Earth land mask for the globe (~2.3 KB)
  ipc/index.ts        All Tauri invoke() calls, centralised
  stores/             theme, stats (history rings), config, network
src/routes/+page.svelte   The whole HUD layout and its CSS
tools/                Measurement and screenshot helpers (§6)
docs/                 ARCHITECTURE, ROADMAP, CONTRIBUTING, ADRs
```

---

## 4. Environment

This matters — two of these cost real time to discover.

- **node/pnpm are under nvm and not on the default PATH.** Every shell needs
  `export PATH="$HOME/.nvm/versions/node/v24.20.0/bin:$PATH"` or pnpm is simply
  "not found" while `node_modules/` sits there populated.
- **Linux build deps include `libdbus-1-dev`, which is missing from Tauri's own
  prerequisites page.** Full list:
  `libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev
  libgtk-3-dev librsvg2-dev libssl-dev libayatana-appindicator3-dev
  libdbus-1-dev`. Note `libappindicator3-dev` is only a transitional package;
  Tauri v2 wants the ayatana one.
- The dev machine is Linux Mint (Cinnamon, X11) on `DISPLAY=:1`.
- `sudo` requires a password; there is no passwordless sudo.

---

## 5. Commands and quality gates

```
pnpm tauri dev                  # dev server + hot reload
pnpm tauri build --no-bundle    # release build, no packaging (~20s incremental)
pnpm check                      # svelte-check — must be 0 errors AND 0 warnings
pnpm test                       # frontend tests (node built-in runner)
cargo test  --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

All four gates are green (last verified 2026-09-22) and are enforced in CI. Keep them that way:
CI previously had them disabled with `continue-on-error` because the tree was
dirty, which made the badge meaningless.

`cargo fmt` is deliberately **not** enforced. rustfmt wants a ~263-line reflow
that would undo the deliberately compact style used throughout `src-tauri`.
Settle that as a project decision before turning it on. (Exception:
`stats/mod.rs` was rustfmt'd wholesale in the hardware-telemetry work, so it no
longer matches the compact style of its neighbours.)

Tests, as of 2026-09-22: 9 Rust (pty termios detection ×4, config ×2, net ×2,
stats ×1) + 20 TypeScript across `secureInput`, `audio`, and the `config`,
`network` and `stats` stores. `pnpm test` uses node's built-in runner with `--experimental-strip-types`,
so there is no test framework dependency; that is why `tsconfig.json` sets
`allowImportingTsExtensions` and test files import with an explicit `.ts`.

---

## 6. How to verify a change

### 6.1 The frontend is embedded in the release binary
`pnpm tauri build` bakes `build/` into the binary. Changing a `.svelte` file and
re-running the old binary shows nothing. Either use `pnpm tauri dev` (HMR), or
rebuild. Incremental release rebuilds are ~20s, which is fast enough to A/B.

### 6.2 Screenshots
Use `python3 tools/grab_window.py out.png`. It reads the window's pixels
directly via Xlib and touches neither focus nor the pointer.

**Do not use `gnome-screenshot -w`** — it captures whatever is *focused*, which
during development is frequently the user's own window. And select the window by
WM_CLASS, not title: a file manager showing the project directory is also titled
"nextgen-ui".

### 6.3 Driving the UI
There is no `xdotool` installed; `python3-xlib` with XTEST is, and works. **The
user is usually at this machine.** Before synthesising any input, check that
`_NET_ACTIVE_WINDOW` is already the app's window and abort otherwise — never
steal focus to send keys, and never send input into whatever the user happens to
have open. Ask first.

### 6.4 Things that need a real password prompt
`pty_input_state` classifies password prompts. To exercise it without knowing
the user's password, launch the app with `SHELL=` pointed at a script that
reproduces the terminal state. `tools/` does not ship these because they are
throwaway, but the shape that works is a Python script that sets raw mode with
echo off, forks, `setpgid`s the child, `tcsetpgrp`s it into the foreground, and
execs a copy of `/bin/sleep` renamed `sudo`.

Two failed attempts at that reproduction, for the record: using `exec` makes the
shell process *become* sudo, so `at_shell_prompt` reads true; and `bash --rcfile`
did not run the file at all under the app.

---

## 7. Performance: the part that matters

### Methodology
`python3 tools/measure_idle.py --seconds 30 --label "..."`.

**Every measurement must record the window state, and comparisons are only valid
between like states.** There are three, and they differ enormously:

1. **Focused** — everything runs at full rate. This is the number that matters,
   because it is what the user experiences while using the app.
2. **Visible but unfocused** — WebKit throttles, and the app's own animations
   throttle themselves (`HudBackground` 30→5fps, globe 20→2fps).
3. **Occluded** — WebKit stops painting almost entirely. A reading near 1.5% is
   almost certainly this, not a real improvement.

Also check nothing else heavy is running. Several early measurements were taken
while eDEX-UI itself was running at 36% CPU.

### Recorded figures
Treat these as approximate and re-baseline before optimising:

| Condition | CPU | RSS |
|---|---|---|
| v1.0.x release, before any perf work | ~38.8% | ~446 MB |
| v1.1.0, focused | ~27.5% | ~455 MB |
| v1.1.0, unfocused | ~10% | ~450 MB |
| v1.1.0, occluded | ~1.5% | ~590 MB |

RSS is dominated by `WebKitWebProcess` (~250 MB) and is largely a floor you
cannot optimise from app code.

### StatsPanel render cost (Resolved)
`StatsPanel` previously cost ~12.6 points of CPU in WebKitWebProcess due to
SVG string building/parsing in `Sparkline` and layout reflows from `height`/`width`
CSS transitions on 16 core bars and main telemetry bars.
Resolved by:
1. Replacing SVG Sparkline with a Canvas 2D renderer (zero string allocations,
   unified stroke & fill path, DPR scaling, visibility throttling).
2. Transitioning GPU-composited `transform: scaleY()` and `transform: scaleX()`
   with `will-change: transform`, eliminating layout reflows.
3. Consolidating stats updates into an atomic `allStats` snapshot with in-place
   ring buffer management.

---

## 8. Open work

Done since v1.1.0 (on `dev`, unreleased): StatsPanel render cost (Canvas 2D +
compositor transforms), sound design, network detail + globe geolocation pin,
command palette + settings GUI. CI confirmed green on `dev` since 2026-09-04.

Remaining, roughly in priority order:

0. **Verify and merge `fix/security-hardening`** — see §11 checklist.
1. **Finish hardware telemetry** (`feature/hardware-telemetry`) — see §11 for
   the exact checklist. Mostly needs an idle-CPU measurement and polish.
2. **Release workflow is untested since the CI fix.** The `Release` run for tag
   `v1.1.0` failed (2026-09-04 09:33Z), *before* the CI fixes landed, so no
   v1.1.0 binaries were ever published. The workflow was patched afterwards but
   has not run since. Verify it before cutting v1.2.0.
3. **Cut v1.2.0**: merge `dev` → `main`, tag, confirm the release job builds on
   all three OSes. Bump version in package.json, Cargo.toml, tauri.conf.json.
4. **Custom app icon** — still the stock Tauri icon; a Phase 6 release item.
5. **Themes on disk**: `themes/` has only `default/theme.json`; the other three
   live only in `src/lib/stores/theme.ts`, so the JSON theme system described
   in the ADRs is not actually loading from disk yet.
6. **SSH session manager** — needs an early decision: `russh` vs wrapping the
   system `ssh` binary.
7. Further ideas ranked by identity-per-hour: GPU utilisation (not just temp),
   split panes.
8. Housekeeping: stale local branch `feature/on-screen-keyboard` and remote
   `feature/process-monitor` / `feature/visual-overhaul` are all merged — prune
   once confirmed. Decide on `cargo fmt` (§5).

Historical note on the CI fix, worth knowing: the Rust job never installed the
Linux system dependencies, so clippy and tests could not build Tauri at all,
and `cache: pnpm` in `actions/setup-node` failed on every runner. pnpm's
version now comes from `packageManager` in package.json rather than being
duplicated in YAML, and the store is cached explicitly.

---

## 9. Traps discovered the hard way

Each of these cost time and is not obvious from the code.

- **xterm.js calls `stopPropagation()` on every key it handles.** A
  window-level `keydown` listener therefore never fires while the terminal has
  focus — which is always, since `Terminal.svelte` focuses on mount. All five
  documented HUD shortcuts were silently dead for the entire life of the project.
  Fixed via `attachCustomKeyEventHandler`, which declines the key and, crucially,
  also stops xterm cancelling the event. The on-screen keyboard's key mirroring
  hits the same wall and registers its listeners in the **capture phase**.
- **Animating `box-shadow` forces a full repaint every frame.** One 6px pulsing
  "LIVE" dot cost ~6% of a CPU core. Animate `opacity` instead — it composites,
  and it fades the glow along with the element so it looks identical.
- **Creating canvas gradients inside the frame loop is the classic killer.**
  `HudBackground` was building 5 gradients and ~58 rgba strings per frame.
  Cache gradients; use `globalAlpha` over a solid style; batch strokes.
- **Password detection is subtler than it looks.** An interactive shell runs with
  `ECHO` *off* because readline echoes for itself, so "not echoing" does not mean
  "password". Canonical mode is the discriminator for getpass-style reads — but
  **sudo with `pwfeedback`** (default on Linux Mint, `/etc/sudoers.d/0pwfeedback`)
  reads key-by-key to print asterisks and so turns `ICANON` off too, becoming
  termios-identical to a shell prompt. That needs the *foreground process*.
  And `sudo` remains the process-group leader for the whole session it launched,
  so that signal cannot tell you when the read *ends* — see `secureInput.ts`.
  Also: `read -s` is a shell builtin, so the shell itself is the foreground
  process during it. All of this is encoded in tests; read them before changing.
- **The antimeridian.** Rasterising the land mask, Fiji's polygon spans +180 to
  −180 and, filled in flat pixel space, swept a solid band of fake land across
  every longitude at 16°S. Unwrap longitudes and draw each ring at ±360.
- **`pkill -f <pattern>` matches your own shell's command line** and will kill
  the shell running it. Use `pgrep -x <comm>`, or the `fakebi[n]` bracket trick.
- **`/proc/<pid>/comm` is truncated to 15 characters.** `WebKitWebProcess`
  appears as `WebKitWebProces`; exact-match filters silently return nothing.
- **A stale `vite` dev server on port 5173 will block `pnpm tauri dev`** with
  "Port 5173 is already in use". Check for an orphan before assuming a bug.

---

## 10. Conventions

- Branches: `main` = stable releases, `dev` = active development, `feature/*` =
  short-lived, merged into `dev` with a merge commit (`--no-ff`).
- Commit messages explain **why**, not just what, and name the cause when fixing
  a bug. Look at `git log` for the established tone.
- Git identity in this repo is set locally to the GitHub noreply address, to keep
  the maintainer's real email out of a repo intended to become public.
- Both quality gates pass before committing.
- Anything that runs every frame caches what it can, batches draw calls, caps its
  frame rate, and stops when the page is hidden.
- Security-relevant logic is unit tested rather than reasoned about. The
  password-suppression logic was got wrong twice by reasoning; the tests exist
  because of that.
- Fail closed on anything touching secrets: if state cannot be determined,
  suppress display rather than risk showing a password.

---

## 11. Session handoff log

Newest first. Every session appends an entry here before stopping: what was
done, what is left half-finished, and the next concrete step. Keep entries
short; move anything durable into the sections above.

### 2026-09-22 (later) — security review + fixes on `fix/security-hardening`

Internal code review of every IPC command, the Tauri config, CI and deps.
Findings and fixes, one commit each, all on `fix/security-hardening`
(branched from `dev`, **not merged, not pushed**):

| # | Finding | Fix |
|---|---|---|
| 1 | `csp: null` while the webview can drive a shell | Strict CSP in `tauri.conf.json` |
| 2 | Geo lookup automatic, HTTP fallback, IP/city on screen | Opt-in `geoLookup` (Settings → Privacy), HTTPS only, cache cleared when off |
| 3 | Sync `measure_ping` blocked main thread up to 4.5s offline | `net` + `fs_list_dir` async via `spawn_blocking` |
| 4 | Zombie shell per closed tab (verified: state `Z`) | Session owns `Child`; reaped on close and on shell exit |
| 5 | Terminal links used `window.open`, any scheme | `src/lib/links.ts`, opener plugin, http(s) only |
| 6 | 9 npm advisories (3 high), all build tooling | `pnpm update`; 1 low left (`cookie`, server-only) |
| 6b | `cargo audit`: 2 quick-xml DoS via plist | `cargo update -p plist`; only unmaintained/unsound *warnings* remain, transitive via Tauri's GTK stack |
| — | UTF-8 split across 4 KB reads → U+FFFD | `take_utf8` in `pty/mod.rs` |

Gates green: 12 Rust, 23 TS, `cargo audit` 0 vulnerabilities, check 0/0, clippy. Release build OK and embeds
the CSP.

**Not yet verified — do this before merging:**
- [ ] **Run the release build and exercise it with the CSP on**: terminal
      renders (WebGL + DOM fallback), themes/inline styles apply, sounds play,
      globe draws, geo toggle works both ways, a link click opens the browser.
      A CSP violation shows as a blank or unstyled panel. `devCsp` (for
      `pnpm tauri dev`) is also untested.
- [ ] Close a tab and confirm no `Z` bash is left (`ps -o pid,stat,comm
      --ppid $(pgrep -x nextgen-ui)`).
- [ ] Merge order: this branch and `feature/hardware-telemetry` both touch
      nothing in common except docs, so either order works.

**Residual, accepted for now:** `style-src 'unsafe-inline'`; `fs_list_dir`
and `kill_process` are unrestricted by design (user-level local app) — the
CSP is what protects them. The Settings "Shell Executable Path" field is
saved but **ignored** by the backend (`default_shell()` reads `$SHELL`);
either wire it up or remove it — if wired, it becomes a config-file →
exec path, so validate it.

### 2026-09-22 — resumed after an 18-day gap

**Found:** `dev` clean at `b7a88b4` and in sync with `origin/dev`, plus an
uncommitted hardware-telemetry feature dated 2026-09-05 (no commit, no notes).
It touched `stats/mod.rs`, `StatsPanel.svelte`, `ipc/index.ts`,
`stores/stats.ts` and added `stores/stats.test.ts`.

**Did:**
- Ran all four gates against the WIP — all green.
- Restored four "why" comments the WIP had silently deleted (process refresh
  kind, rank-then-allocate, single-pid kill refresh, core bar minimum).
- Moved the WIP onto `feature/hardware-telemetry` and committed it there.
  **Not merged into `dev`, not pushed.**
- Brought this file and CHANGELOG up to date.

**Hardware telemetry — what's left before merging to `dev`:**
- [ ] **Measure idle CPU** (§7), focused and unfocused, against `dev`. The
      emitter now also calls `Components::refresh` (hwmon sysfs reads), reads
      `/sys/class/power_supply` and collects disk capacity *every second*. If
      it costs anything measurable, move temps/disks/battery onto a slower
      cadence (e.g. every 5th tick) — they don't change at 1 Hz.
- [ ] Look at it running and screenshot it (§6.2); it has never been seen.
- [ ] `collect_thermals_and_temps` falls back to *the first sensor of any kind*
      as "CPU temp" when no CPU-looking label matches — that can be an NVMe or
      Wi-Fi sensor. Prefer showing nothing (fail honest, like §10).
- [ ] The "cpu" substring match is broad; check real labels on this machine
      (`sensors` / sysinfo labels) and tighten.
- [ ] `BatteryInfo.present` is always `true` when `Some` — redundant field.
- [ ] Battery/thermals are Linux-only (`cfg(target_os = "linux")`); on
      Windows/macOS battery is always `None`. Fine for now, but document it.
- [ ] Then: merge `--no-ff` into `dev`, update CHANGELOG, push (ask the
      maintainer first).

**Next after that:** §8 items 2–3 — verify the release workflow, cut v1.2.0.
