# ADR 002 — Svelte 5 over React

**Date:** 2026-06  
**Status:** Accepted  
**Deciders:** @gokulgk4616

---

## Context

The frontend needs a reactive UI framework for building the terminal panel, HUD components, stats dashboard, and file manager. The framework runs inside Tauri's WebView, so bundle size and runtime overhead directly affect startup time and memory usage.

---

## Options considered

### Option 1 — React 19
- Virtual DOM runtime (~45KB gzipped)
- Large ecosystem
- Requires additional state management (Zustand, Jotai, etc.)
- JSX compilation adds build complexity
- Common choice — many contributors will know it

### Option 2 — Svelte 5
- Compiles to vanilla JS — no runtime framework overhead
- Reactive primitives baked in (runes in v5) — no separate state library needed
- Smallest bundle output of any major framework
- Built-in transitions and animations
- Less widely known but growing rapidly

### Option 3 — Vue 3
- Moderate runtime size
- Good reactivity model (Composition API)
- Middle ground between React and Svelte

---

## Decision

**Svelte 5** is chosen as the UI framework.

---

## Rationale

Since Tauri's WebView has no Chromium to amortize startup cost across, every kilobyte of JavaScript increases parse time. Svelte compiles away the framework — the output is plain JavaScript that runs directly, with no virtual DOM diff pass on every update.

For a UI that updates many reactive values simultaneously (stats dashboard with 10+ live metrics, terminal scroll buffer, HUD animations), Svelte's compiler-driven reactivity avoids the reconciliation overhead that React's virtual DOM introduces.

Svelte 5's runes system provides fine-grained reactivity without needing Zustand, Jotai, or Redux — reducing dependency surface.

---

## Consequences

- Contributors unfamiliar with Svelte will have a learning curve (roughly 1–2 days to become productive)
- Svelte's TypeScript support is slightly less mature than React's, requiring occasional type workarounds
- Component library ecosystem is smaller — we write most UI components from scratch (acceptable given the custom sci-fi aesthetic)
- Build tooling: Vite + `@sveltejs/vite-plugin-svelte` — well-supported and fast
