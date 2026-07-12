## What & why

## Checks run
- [ ] `bunx tsc --noEmit`
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml --no-default-features`
- [ ] `bun run tauri dev` — actually exercised the change, not just typechecked it

## Scope
- [ ] Doesn't blend C2PA verification with the heuristic layer (if UI touched)
- [ ] No absolutist language in any new/changed copy (see `CLAUDE.md`)
