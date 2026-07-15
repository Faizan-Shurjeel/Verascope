# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Verascope is an **offline, cross-platform desktop app** (Tauri v2 + React 19 + TypeScript, Rust backend) that reads and validates **C2PA provenance manifests** in media files, and — as a later phase — offers a clearly-separated heuristic AI-generation signal. All processing is local; no file is ever uploaded. The full product spec lives in `docs/PROJECT.md` and is the source of truth for scope, roadmap, and UX principles.

## Two non-negotiable design principles

These come from `docs/PROJECT.md` §2.3, §6, §11 and must be preserved in any code or copy you write:

1. **Never blur C2PA verification with AI detection.** They are two separate problems. C2PA is cryptographic fact; heuristic AI detection is a probabilistic guess. "No manifest found" is *not* evidence of anything — never let the UI imply that absence of provenance means "AI" or "fake". The heuristic layer (Phase 2, not yet built) must always be visually separated and labeled non-authoritative.
2. **Never use absolutist language.** No "this is AI-generated" / "this is authentic". Use calibrated phrasing ("no verifiable provenance was found…", "has a verified provenance chain from [signer]…"). This is a liability requirement, not a style preference.

The app resolves every file into a **three-state verdict** (`src-tauri/src/lib.rs`, `Verdict` enum): `Verified`, `UntrustedOrBroken`, `NoProvenance` — never a binary real/fake.

## Commands

Frontend package manager is **Bun**. Run all commands from the repo root.

```bash
bun install                                       # install JS deps (needed before any tauri CLI works)
bun run tauri dev                                 # run the full desktop app (Rust + webview) — primary dev loop
bun run dev                                       # vite only, browser at localhost:1420 (no Rust backend / invoke)
bun run build                                     # tsc typecheck + vite production build
bunx tsc --noEmit                                 # frontend typecheck only
cargo check --manifest-path src-tauri/Cargo.toml  # Rust backend check (slow on first run — c2pa has 100+ deps)
bun run tauri build                               # produce native installers
```

There are no tests yet. `bun run tauri dev` is the real integration check — a passing `cargo check` + `tsc --noEmit` does not mean the wired app works.

Note: the maintainer sometimes runs cargo via an `rtk` wrapper. If a `cargo` result returns implausibly fast ("0 crates compiled in 0.6s"), it's a cached/no-op — re-run plain `cargo check` to actually verify.

## Architecture

**IPC boundary is the key structural fact.** The Rust backend exposes Tauri commands; the React frontend calls them via `invoke(...)` from `@tauri-apps/api/core`. Commands are registered in the `invoke_handler` in `src-tauri/src/lib.rs` (`run()`), and the entrypoint is `src-tauri/src/main.rs` → `verascope_lib::run()`.

- **Backend — `src-tauri/src/lib.rs`**: the entire Phase 1 logic. `analyze_media(path)` opens a local image, calls `c2pa::Reader::from_stream`, and maps the result onto the three-state verdict:
  - `Err(c2pa::Error::JumbfNotFound)` → `NoProvenance` (a normal outcome, **never** an error).
  - `ValidationState::Trusted` → `Verified`; `Valid`/`Invalid` → `UntrustedOrBroken`.
  - Signer/claim-generator are extracted from the manifest's **JSON output** (`reader.json()`), *not* typed struct getters — deliberate, because c2pa-rs's typed API breaks across minor versions while the JSON shape is spec-stable. Keep new field extraction JSON-based for the same reason.
- **Frontend — `src/App.tsx`**: the UI. Should render the three-state verdict card plus a visually-separate, disabled Phase 2 heuristic stub.

### Current state (important)

The project is **mid-wiring**: `lib.rs` already implements `analyze_media` and has removed the scaffold's `greet` command, but `src/App.tsx` is still the default Tauri+React template that calls the now-nonexistent `greet`. The immediate work is replacing `App.tsx` to call `analyze_media` (via file-picker/drag-drop using the `dialog`/`fs` plugins) and render the verdict. Expect `greet`-related calls to fail until this is done.

## Offline / dependency constraints

- `c2pa` is pinned with `default-features = false, features = ["file_io", "rust_native_crypto"]` in `src-tauri/Cargo.toml`. This is intentional: `rust_native_crypto` avoids a system OpenSSL dependency, and disabling defaults drops the HTTP resolver backends. **This app makes no network calls by design** — do not add features or crates that fetch remotely for core verification.
- Tauri plugins in use: `opener`, `dialog`. Permissions are granted in `src-tauri/capabilities/default.json` — adding a new plugin capability requires updating that file. (`fs` was intentionally removed: `analyze_media` opens files via `std::fs` inside the Rust command, so no filesystem plugin/permission is needed.)
- `security.csp` in `tauri.conf.json` is deliberately `null`. Normally a Tauri anti-pattern, but acceptable here: the app is fully offline, loads no remote content, and renders no untrusted HTML — there is no injection surface a CSP would protect. Revisit if any remote resource or user-supplied markup is ever introduced.
- `c2pa` is a 0.x crate that ships breaking changes across minor versions; when touching backend C2PA code, verify against the resolved version (currently 0.89) rather than memory.

## Roadmap context

Phase 1 (current) = images-only C2PA verification, no heuristic layer. Phase 2 adds the local AI-artifact heuristic (via `ort`/`tract` ONNX inference) as a separate panel. Phase 3 = video/audio. Trust-list management (bundled, versioned, staleness-indicated) is a Phase 1+ concern. See `docs/PROJECT.md` §14 for the full phased plan and checkpoints.
