# Contributing to Verascope

Thanks for considering a contribution. Verascope is a small, focused tool —
keep changes aligned with the two non-negotiable principles in
[`CLAUDE.md`](CLAUDE.md) before anything else:

1. Never blur C2PA verification with AI-artifact heuristics — they stay
   structurally and visually separate.
2. Never use absolutist language ("this is AI-generated" / "this is
   authentic") in UI copy or docs.

See [`docs/ROADMAP.md`](docs/ROADMAP.md) for active priorities and
[`docs/PROJECT.md`](docs/PROJECT.md) for the full product rationale and
technical constraints.

## Setup

Requires Rust 1.88+, [Bun](https://bun.sh), and the
[Tauri v2 system dependencies](https://tauri.app/start/prerequisites/).

```bash
bun install
bun run tauri dev      # run the full app — the real integration check
```

The repository's `rust-toolchain.toml` selects stable Rust for rustup users.
If a distro-provided Rust toolchain is too old, install Rust through rustup.

## Where to help

Start with a small, reviewable contribution:

- Add a test or fixture for a known C2PA verification outcome.
- Improve the labelled-corpus calibration harness or its documentation.
- Reproduce and improve one accessibility or installation issue.
- Remove a specific CI/release maintenance burden without reducing checks.

The [roadmap](docs/ROADMAP.md) gives the current context. For a larger change,
open an issue first so the design can be discussed before implementation.

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

## Maintainers

The project is looking for long-term contributors in Rust, Tauri, C2PA test
coverage, accessibility, and release engineering. Read
[`GOVERNANCE.md`](GOVERNANCE.md) to understand how responsibilities are
shared.
