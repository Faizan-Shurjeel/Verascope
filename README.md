# Verascope

**Offline C2PA provenance verifier.** A cross-platform desktop app that reads and validates [C2PA](https://c2pa.org) provenance manifests in images, video, and audio files — fully locally. No file ever leaves your device; no network calls.

Built with Tauri v2, React 19, and a Rust backend (`c2pa-rs`).

## The verdict model

Verascope never gives a binary "real/fake" answer. Every file resolves to one of three states:

| State | Meaning |
|---|---|
| ✅ **Verified** | Manifest present, signature valid, and it chains to a trusted authority in the bundled trust list. |
| ⚠️ **Untrusted or Broken** | A manifest exists but failed validation (bad signature, tampered content, or an untrusted signer). |
| ❔ **No Provenance** | No manifest found. This is **not** evidence of anything — most genuine photos have no manifest. |

A secondary, clearly separated heuristic panel (Error Level Analysis — recompression-artifact detection, images only) shows a non-authoritative signal alongside the verdict. Cryptographic provenance and heuristic guessing are two different problems and are never blended.

## Develop

Requires Rust, [Bun](https://bun.sh), and the [Tauri v2 system deps](https://tauri.app/start/prerequisites/).

```bash
bun install
bun run tauri dev      # run the full app (primary dev loop)
bun run build          # typecheck + build frontend
bun run tauri build    # native installers
```

## Trust list

The official C2PA trust list is embedded at build time (`src-tauri/trust-list/`), so validation is fully offline. It goes stale over time — the app shows its bundled date and flags staleness. Updating means replacing the bundled PEM and rebuilding.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Status

Phase 1 (C2PA verification), Phase 2 (heuristic panel), and Phase 3 (video/audio) are functional. See [`docs/PROJECT.md`](docs/PROJECT.md) for full scope and roadmap.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
