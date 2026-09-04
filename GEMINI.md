# nextgen-ui

**Read [AGENTS.md](AGENTS.md) first — it is the canonical context for this repo
and is kept up to date. This file is a pointer, so that Claude, Codex, Gemini
and anyone else work from the same brief.**

Three things worth knowing before your first command:

1. node/pnpm live under nvm and are **not** on the default PATH:
   `export PATH="$HOME/.nvm/versions/node/v24.20.0/bin:$PATH"`
2. Idle CPU is this project's whole thesis. Measure before and after any change
   to code that runs every frame or every tick — `tools/measure_idle.py`, and
   read AGENTS.md §7 first, because focused, unfocused and occluded windows give
   wildly different numbers and are not comparable.
3. Quality gates, all of which must stay green:
   `pnpm check` (0 errors **and** 0 warnings), `pnpm test`,
   `cargo test`, `cargo clippy --all-targets -- -D warnings`.

AGENTS.md §9 lists the traps that already cost time here — xterm swallowing
keyboard shortcuts, `box-shadow` animation costing whole percent of a CPU core,
and how password prompts are actually detected. Skim it before debugging
anything in those areas.
