# nextgen-ui — agent handoff

Canonical context for any AI assistant or new contributor working on this
repo. `CLAUDE.md` and `GEMINI.md` point here; keep this file as the single
source of truth and update it as things change.

Last updated: 2026-09-04, at tag `v1.1.0`.

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
`v1.0.0` and was deliberately not advanced.

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
src/lib/
  components/         Terminal, StatsPanel, ProcessMonitor, FileManager,
                      HudBackground, GlobeWidget, Sparkline, OnScreenKeyboard
  secureInput.ts      Decides when typed input must not be displayed. Unit
                      tested — do not inline this logic back into the component
  shortcuts.ts        HUD shortcut predicate, shared by the page and Terminal
  data/landmask.ts    Packed Natural Earth land mask for the globe (~2.3 KB)
  ipc/index.ts        All Tauri invoke() calls, centralised
  stores/             theme, stats (history rings), config
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

All four gates are green at `v1.1.0` and are enforced in CI. Keep them that way:
CI previously had them disabled with `continue-on-error` because the tree was
dirty, which made the badge meaningless.

`cargo fmt` is deliberately **not** enforced. rustfmt wants a ~263-line reflow
that would undo the deliberately compact style used throughout `src-tauri`.
Settle that as a project decision before turning it on.

Tests: 4 Rust (pty termios detection) + 8 TypeScript (secure-input state
machine). `pnpm test` uses node's built-in runner with `--experimental-strip-types`,
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

### The single largest remaining item
**`StatsPanel` costs ~12.6 points of CPU** — more than everything fixed in the
canvas backdrop. Removing it dropped WebKit from 16.3% to 3.7% in a controlled
A/B. It re-renders 16 per-core bars plus 4 sparklines every second, and each
`Sparkline` rebuilds its polygon point strings from scratch on every tick. This
is the highest-value optimisation available and was deliberately deferred.

---

## 8. Open work

1. **`StatsPanel` render cost** (§7) — largest remaining perf win.
2. **Custom app icon** — still the stock Tauri icon, and a Phase 6 release item.
3. **SSH session manager** — needs an early decision: `russh` vs wrapping the
   system `ssh` binary.
4. Themes on disk: `themes/` has only `default/theme.json`; the other three live
   only in `src/lib/stores/theme.ts`, so the JSON theme system described in the
   ADRs is not actually loading from disk yet.
5. Feature ideas ranked by identity-per-hour, from a review of what eDEX-UI had
   and what a 2026 version could beat it at: sound design (key clicks — a large
   part of eDEX's identity and cheap to add), network detail (public IP, ping,
   geolocation plotted on the existing globe), CPU temperature / battery / load
   average / disk capacity (all already exposed by `sysinfo`), GPU monitoring,
   split panes, a settings GUI over the existing TOML config, command palette.
6. CI was red from August until 2026-09-04 and is now fixed; confirm the first
   green run. The causes are worth knowing: the Rust job never installed the
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
