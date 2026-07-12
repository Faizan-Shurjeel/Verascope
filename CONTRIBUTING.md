# Contributing to Verascope

Thanks for considering a contribution. Verascope is a small, focused tool —
keep changes aligned with the two non-negotiable principles in
[`CLAUDE.md`](CLAUDE.md) before anything else:

1. Never blur C2PA verification with AI-artifact heuristics — they stay
   structurally and visually separate.
2. Never use absolutist language ("this is AI-generated" / "this is
   authentic") in UI copy or docs.

See [`docs/PROJECT.md`](docs/PROJECT.md) for the full product spec, roadmap,
and phase checkpoints.

## Setup

Requires Rust, [Bun](https://bun.sh), and the
[Tauri v2 system dependencies](https://tauri.app/start/prerequisites/).

```bash
bun install
bun run tauri dev      # run the full app — the real integration check
```

## Before opening a PR

```bash
bunx tsc --noEmit                                             # frontend typecheck
cargo check --manifest-path src-tauri/Cargo.toml --no-default-features
cargo test --manifest-path src-tauri/Cargo.toml --no-default-features
bun run build                                                  # production build
```

A passing `cargo check`/`tsc` does not mean the wired app works — actually
run `bun run tauri dev` and exercise the change (drop in a real file) before
calling it done.

## Scope

Out of scope for now (see `docs/PROJECT.md` §7.2 for the full list): cloud
analysis, batch/API access, or anything that adds a network dependency to
core verification. Open an issue to discuss before building something in
this area — it'll likely be declined on principle, not on execution.

## Commit style

Small, focused commits. Explain *why* in the message when it isn't obvious
from the diff.
