# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Web UI for the sw-cor24-x-tinyc Tiny C compiler. Live browser demos of COR24 compilation using Rust, Yew, and WebAssembly.

## Related Projects

- `~/github/sw-embed/sw-cor24-x-tinyc` — Tiny C compiler for COR24 (Rust) — embedded as WASM
- `~/github/sw-embed/sw-cor24-emulator` — COR24 assembler and emulator (Rust)
- `~/github/sw-embed/sw-cor24-project` — COR24 ecosystem hub

## Build

```bash
./scripts/serve.sh              # dev server with hot reload on port 9101
./scripts/build-pages.sh        # release build to pages/ for GitHub Pages
cargo clippy -- -D warnings     # lint
```

Edition 2024. Never suppress warnings.

## Deployment

Live demo: https://sw-embed.github.io/web-sw-cor24-tinyc/

`trunk build --release` outputs to `dist/` (gitignored). `./scripts/build-pages.sh` builds to `dist/` then rsyncs to `pages/` (tracked). `pages/.nojekyll` is committed once and never regenerated. GitHub Actions (`.github/workflows/pages.yml`) deploys `pages/` on push to main.

## Adding New Demos

Demos live in `sw-cor24-x-tinyc/demos/`. To add new demos to the web UI:
1. Add entries to the `DEMOS` array in `src/main.rs` — tuple of `(filename, description)`
2. Run `cargo clippy -- -D warnings`
3. Run `./scripts/build-pages.sh`
4. Commit `src/main.rs` and `pages/` on a `feat/<slug>` branch; rename to `pr/<slug>` when ready (see Workflow below).

## Workflow (devgroup)

This repo is hosted on a devgroup workstation. Run `onboarding` (in `$PATH`) for current state and helpers; full policy at `/disk1/github/softwarewrighter/devgroup/docs/branching-pr-strategy.md`.

- **Never push.** No `git push`, no `gh`. Signaling readiness is done by branch rename only — coordinator (mike) relays `pr/*` into `dev` and pushes.
- **Branches:** `feat/<slug>` for work in progress, based on `origin/dev` (not `origin/main`). Rename to `pr/<slug>` when ready to merge. `fix/<slug>` is the bug-fix flavor of `feat/`.
- **Helpers** (in `$PATH`): `dg-new-feature <slug>`, `dg-new-fix <slug>`, `dg-mark-pr`, `dg-list-pr`, `dg-reap`.
- **No history rewrites** on `dev` or `main`. Rebase is OK on your own `feat/*`.
- **After merge:** `git fetch origin --prune && git switch dev && git branch -D pr/<slug>`.
