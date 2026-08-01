# Contributing to nextgen-ui

Thank you for your interest in contributing. This document explains how to get started, how the codebase is organized, and what standards we expect.

> This repository is currently private. This guide will be finalized before the public release at v1.0.0.

---

## Prerequisites

| Tool | Version | Purpose |
|---|---|---|
| Rust | 1.78+ | Backend |
| Node.js | 20+ | Frontend tooling |
| pnpm | 9+ | Package manager |
| Tauri CLI | 2.x | Build + dev server |
| cargo | latest | Rust build tool |

Install Tauri prerequisites for your OS: https://tauri.app/start/prerequisites/

---

## Dev environment setup

```bash
# Clone the repo
git clone https://github.com/nextgen-ui/nextgen-ui.git
cd nextgen-ui

# Install frontend dependencies
pnpm install

# Run in development mode (hot-reload)
pnpm tauri dev
```

---

## Branch workflow

```
main        ← stable, tagged releases only — never push directly
dev         ← active development — all PRs target this branch
feature/*   ← new features — branch from dev, merge back to dev
fix/*       ← bug fixes — branch from dev, merge back to dev
```

```bash
# Start a new feature
git checkout dev
git pull origin dev
git checkout -b feature/your-feature-name

# Start a bug fix
git checkout dev
git pull origin dev
git checkout -b fix/short-description
```

After your PR is merged, delete the branch.

---

## Code style

### Rust
- Format with `rustfmt` — run `cargo fmt` before committing
- Lint with `clippy` — run `cargo clippy -- -D warnings` before committing
- No `.unwrap()` in production paths — use `?` or explicit error handling
- No `unsafe` in business logic

### Svelte / TypeScript
- Format with Prettier — run `pnpm format` before committing
- Type-check with `svelte-check` — run `pnpm check` before committing
- All Tauri `invoke()` calls must go through `src/lib/ipc/index.ts` — no direct calls in components

---

## Commit message format

```
type(scope): short description

Optional longer description.
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

Examples:
```
feat(pty): add multi-tab session support
fix(stats): reduce CPU spike on rapid polling
docs(arch): update data flow diagram
```

---

## Pull request checklist

Before opening a PR:

- [ ] `cargo fmt` run
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `pnpm check` passes
- [ ] New functionality has tests
- [ ] `docs/` updated if architecture changed
- [ ] `CHANGELOG.md` updated under `[Unreleased]`
- [ ] PR targets `dev`, not `main`

---

## Testing

```bash
# Rust tests
cargo test

# Frontend type check
pnpm check

# Full build test
pnpm tauri build
```

---

## Reporting issues

Use the GitHub issue templates:
- **Bug report** — for crashes, incorrect behavior, performance regressions
- **Feature request** — for new functionality proposals

Search existing issues before opening a new one.

---

## Architecture decisions

Before making significant architectural changes, read the existing [Architecture Decision Records](decisions/) and open a GitHub issue for discussion. Major decisions should be documented as a new ADR in `docs/decisions/`.
