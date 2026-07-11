## Starting a Rust and Tauri project

Last updated: 2026-07-11

---

### *User:*

Have A look,SHall we start the Project or Not?
I have rust & tauri installed along with both bun & npm\.
I also have git & gh cli installed so you can create & push to repo yourself\.



### *Claude:*

Good news on the diligence front: c2pa-rs is actively maintained, with releases as recent as July 2026 and the crate is currently at version 0.84+ with over 9 million downloads — it's already dual-licensed MIT/Apache-2.0, which matches what your doc leans toward. Phase 0's "is the ecosystem mature enough" checkpoint looks like a clear yes.

My honest take on the plan itself: it's unusually well-scoped for a v0.1 doc. The three-state verdict model (§6) is the right call — it protects you legally and ethically, and it's the single most important design decision in here, so I'm glad it's locked in early rather than left as an open question. The images-first, offline-first, C2PA-before-heuristics sequencing in the roadmap is sound too. I don't see red flags that should stop you from starting.

So — yes, let's start. Given what you have installed, I'd begin with a real Phase 1 scaffold rather than pure research, since Rust/Tauri/bun/npm are all ready:

1. `cargo create-tauri-app` for the shell
2. Wire up `c2pa` crate for manifest reading (skip validation/trust-list for the very first commit — get "load image → print manifest JSON" working first)
3. Basic three-state verdict UI stub
4. Push to a GitHub repo via `gh`

A couple of quick decisions before I scaffold:



### *User:*

Q: Frontend framework inside Tauri \(doc leaves this open\)?
A: React

Q: GitHub repo visibility?
A: Public

Q: Package manager for the frontend?
A: Bun



### *Claude:*

Quick correction before I go further: the sandbox I run commands in is a separate, ephemeral container — it's not your local machine. It doesn't have `cargo`, `rustc`, `bun`, or `gh` installed, and it has no access to your GitHub credentials. So I can't literally push to a repo under your account from here.

What I *can* do well: build the full project scaffold as real files (Cargo.toml, Rust backend, React frontend, configs), test-compile what's testable in this sandbox, package it up, and hand you exact copy-paste commands to `git init`, commit, and `gh repo create --push` from your own machine where everything's already installed and authenticated. That gets you to a working repo in under a minute on your end.

Let me try installing a Rust toolchain here via apt (it's allowlisted) so I can at least verify the backend compiles before handing it off.

Good context gathered — I now have the current `c2pa` crate API (0.84.x, `Reader`/`Builder`/`Context` model) rather than an outdated one from memory. Let me build the actual scaffold.

Now the frontend — React/TS with the three-state verdict UI:

Now let's sanity-check the frontend TypeScript compiles cleanly with npm (bun isn't available in this sandbox, but the code is bun/npm-agnostic):

Frontend type-checks cleanly. Now let's write the remaining scaffolding files — README, license, gitignore, CI, and CONTRIBUTING.

Now a minimal CI workflow and final repo checks:

Now let's package this for download and prep the exact hand-off commands:

**Reality check on the "push it yourself" part:** the sandbox I run commands in has no `cargo`/`rustc` ≥1.88, no `bun`, and no `gh` auth — it's not your machine. So instead I built the whole Phase 1 scaffold as real files, verified what I could (the TypeScript frontend type-checks clean against the actual `@tauri-apps` APIs), and packaged it above. On your machine, where everything's already installed:

```bash
tar -xzf verascope-scaffold.tar.gz && cd verascope
cargo check --manifest-path src-tauri/Cargo.toml   # do this first
bun install
bun tauri dev
gh repo create verascope --public --source=. --remote=origin --push
```

What's in it:
- **Rust backend** (`src-tauri/src/lib.rs`) — reads C2PA manifests via `c2pa-rs`'s current `Reader` API, maps the result onto your three-state verdict model exactly as spec'd in §6, keeps "no manifest" (`Error::JumbfNotFound`) explicitly distinct from "broken manifest"
- **React/TS frontend** — drag-drop or file-picker, verdict card (color-coded, ink-stamp motif, not a generic AI-app gradient), raw manifest JSON viewer, and a **visually separate, disabled** Phase 2 heuristic panel stub — because §2.3 is the rule most worth protecting structurally, not just in copy
- Dual MIT/Apache-2.0 licensing, placeholder icons, a CI skeleton, and your original doc copied into `docs/PROJECT.md`

One honest flag: `c2pa` is a 0.x crate that ships breaking changes across minor versions, and I couldn't compile against it here (this sandbox only had rustc 1.75 via apt; the crate needs 1.88+). I wrote it against the current documented API, but `cargo check` on your machine is the real test — if a method signature drifted, the errors will point straight at `lib.rs`.



### *User:*

can't you just gimme bash commands that can do everyhting here with latest & stable everything:
faizy@faizy:\~/Desktop/Verascope$ ls
Documentation\.md
faizy@faizy:\~/Desktop/Verascope$

\-\-\-

i can provide you output where needed



### *Claude:*

Fair — you have the real toolchain, I don't. Here's a script that scaffolds with whatever's *actually* latest on your machine (via the official generators, not my guesses), rather than the versions I hand-wrote earlier. Run this from `~/Desktop/Verascope`:

```bash
# 0. Sanity check your toolchain first
rustc --version && cargo --version && bun --version && node --version && gh --version

# 1. Keep the doc safe, scaffold Tauri fresh into this same folder
mkdir -p /tmp/verascope-stage
mv Documentation.md /tmp/verascope-stage/

# 2. Official scaffolder — pick "." when it asks for project name (uses cwd),
#    then: TypeScript / React / TypeScript (for the UI flavor) / bun
bun create tauri-app@latest

# 3. Put the doc back, under docs/
mkdir -p docs
mv /tmp/verascope-stage/Documentation.md docs/PROJECT.md

# 4. Add the two Tauri plugins the app needs (dialog for file picker, fs for reading)
#    — this auto-wires Cargo.toml, capabilities/*.json, and the JS bindings for you
bun tauri add dialog
bun tauri add fs

# 5. Add c2pa-rs — let cargo resolve whatever's actually latest right now
cd src-tauri
cargo add c2pa --features file_io
cd ..

# 6. Baseline check — nothing custom yet, just confirming the scaffold itself builds
cd src-tauri && cargo check 2>&1 | tail -40 && cd ..
bunx tsc --noEmit
```

A few things worth knowing before you run it:

- Step 2 is interactive — when it asks for the project name, type `.` so it scaffolds into the current folder instead of making a subfolder.
- Step 6's `cargo check` will be slow the first time (pulling and compiling the full dependency tree, `c2pa` is a big crate).

Paste me back:
1. The version output from step 0
2. Whether step 2's prompts matched what I guessed (TS/React/TS/bun)
3. The tail of `cargo check` and the `tsc --noEmit` output from step 6

Once I see that, I'll give you the exact `lib.rs`, `App.tsx`, and `App.css` to drop in — written against whatever `c2pa` version actually resolved for you, not the one I guessed at earlier.



### *User:*

Hmm



### *Claude:*

Okay — a few things happened here, let me untangle it:

1. **Your paste got scrambled** (the terminal shows commands merged together, e.g. `bunx tsc --noEmitargo check`), which is why `bun create tauri-app` hit an interrupted read. But it recovered — you answered the prompts manually and it worked: fresh scaffold created (`.` / `verascope` / `com.faizy.verascope` / React+TS+bun). Good.
2. **The actual build error** (`glib-sys` / `glib-2.0 was not found`) is just missing Linux system packages for Tauri — nothing wrong with the code, everyone hits this on a fresh Ubuntu box.
3. `tauri: command not found` and the 34 TS errors are both because `bun install` was never run yet — no `node_modules`, so nothing's on `PATH` and nothing has types. Not a real problem, just ordering.
4. Good news buried in there: `cargo add c2pa` resolved **v0.89.2** as current-latest — newer than what I referenced, and it pulled in `openssl` + four separate HTTP backends by default, which is both unnecessary for an offline-only app and *is* the thing that just broke your build (openssl-sys wants system dev headers too). I'll have you trim that.

Run these **one command at a time** (not pasted as one block — that's likely what scrambled the terminal earlier):

```bash
sudo apt update
```

```bash
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

```bash
bun install
```

```bash
bun run tauri add dialog
```

```bash
bun run tauri add fs
```

Now open `src-tauri/Cargo.toml`, find the `c2pa = "0.89.2"` line `cargo add` inserted, and replace it with:

```toml
c2pa = { version = "0.89", default-features = false, features = ["file_io", "rust_native_crypto"] }
```

That drops `openssl` (switches to pure-Rust crypto — no system OpenSSL needed) and drops the four HTTP resolver backends (`reqwest`/`ureq`/`wasi`/`wstd`), which you don't want anyway — this app never fetches anything remotely by design.

Then:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Paste me the tail of that output. If it's clean, I'll give you the real `lib.rs` and `App.tsx`/`App.css` to drop in next.



### *User:*

also i already have \`rtk\` installed,
It's CLean: faizy@faizy:\~/Desktop/Verascope$ rtk cargo check \-\-manifest\-path src\-tauri/Cargo\.toml
cargo build \(0 crates compiled\)
Finished \`dev\` profile \[unoptimized \+ debuginfo\] target\(s\) in 0\.64s
faizy@faizy:\~/Desktop/Verascope$



### *Claude:*

That "0 crates compiled in 0.64s" is the tell — a first-time check with `c2pa` in the tree (100+ transitive dependencies) takes minutes, not 0.64s. That's a cached/no-op result, not a real verification of your edit.

Two questions:

1. What is `rtk`? If it's some caching or task-runner wrapper around cargo, it may be reading a stale target dir or a different manifest than you think.
2. Can you confirm the edit actually saved? Run:

```bash
grep -A3 "^c2pa" src-tauri/Cargo.toml
```

That should show the trimmed `default-features = false, features = ["file_io", "rust_native_crypto"]` line. Paste that back.

Then, to get a real check (bypass `rtk` for now so we know exactly what ran):

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Let it run to completion — this is the one that should take a couple of minutes and show a long list of `Compiling ...` lines. Paste the tail of that output (last ~40 lines, or the whole thing if it errors).



### *User:*

rtk readme attached,
faizy@faizy:\~/Desktop/Verascope$ grep \-A3 "^c2pa" src\-tauri/Cargo\.toml
c2pa = \{ version = "0\.89", default\-features = false, features = \["file\_io", "rust\_native\_crypto"\] \}
tauri\-plugin\-dialog = "2"
tauri\-plugin\-fs = "2"
faizy@faizy:\~/Desktop/Verascope$



### *Claude:*





### *User:*

rtk readme attached,
faizy@faizy:\~/Desktop/Verascope$ grep \-A3 "^c2pa" src\-tauri/Cargo\.toml
c2pa = \{ version = "0\.89", default\-features = false, features = \["file\_io", "rust\_native\_crypto"\] \}
tauri\-plugin\-dialog = "2"
tauri\-plugin\-fs = "2"
faizy@faizy:\~/Desktop/Verascope$
It did take a lot of time, i then reran it multiple times, so not a big deal



### *Claude:*

Good — that confirms `c2pa` (with the trimmed features) compiles clean on your machine. I also dug into the exact current `Manifest` API on docs.rs (0.80–0.89 range) and found two things I had wrong in my first draft: `claim_generator_info` is a **struct field**, not a method, and there's a simpler `.issuer()` method directly on `Manifest` instead of going through `signature_info()`. I've rewritten `lib.rs` to use the JSON output for those two fields instead of chasing exact struct/field names — it's more resilient to the crate's frequent breaking changes than binding tightly to typed getters.

Two things to fix first, **one command at a time**:

```bash
bun install
```

```bash
bunx tauri add dialog
```

```bash
bunx tauri add fs
```

(Your earlier `tauri add dialog` failed with "command not found" because it ran before `bun install` populated `node_modules/.bin`, and without the `bunx`/`bun run` prefix. This time it should work and will auto-wire `Cargo.toml`, `capabilities/default.json`, and the JS bindings for you.)

Now replace `src-tauri/src/lib.rs` entirely:

```bash
cat > src-tauri/src/lib.rs << 'RUST_EOF'
// Verascope — Rust backend
//
// Phase 1 scope only: read + validate a C2PA manifest for a single local
// image file and map the result onto the three-state verdict model from
// the project doc (§6):
//
//   1. Verified          — manifest present, signature valid, chains to a
//                           trusted issuer.
//   2. UntrustedOrBroken  — manifest present but validation failed.
//   3. NoProvenance       — no manifest found. This is NOT evidence of
//                           anything; the UI must say so explicitly.
//
// Note on approach: claim_generator_info / issuer extraction below reads
// from the manifest's own JSON output (reader.json()) rather than typed
// Rust struct getters. c2pa-rs's typed API has shifted across minor
// versions; the JSON manifest shape is spec-documented and much more
// stable, so this is the safer thing to depend on here.

use c2pa::{Reader, ValidationState};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Verified,
    UntrustedOrBroken,
    NoProvenance,
}

#[derive(Serialize, Debug)]
pub struct AnalysisResult {
    pub verdict: Verdict,
    /// Short, calibrated human-readable summary. Never absolutist
    /// ("this is AI-generated" / "this is authentic") per §11 of the
    /// project doc.
    pub summary: String,
    /// Raw manifest JSON, if any manifest was found (present even for
    /// UntrustedOrBroken, so the UI can show *why* it's untrusted).
    pub manifest_json: Option<String>,
    /// Best-effort signer/issuer name extracted from the active manifest,
    /// if present.
    pub signer: Option<String>,
    /// Claim generator (the tool that produced the manifest), if present.
    pub claim_generator: Option<String>,
    /// Non-fatal validation notes (e.g. why UntrustedOrBroken).
    pub notes: Vec<String>,
}

fn mime_for_path(path: &Path) -> Option<&'static str> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => Some("image/jpeg"),
        Some("png") => Some("image/png"),
        Some("webp") => Some("image/webp"),
        Some("gif") => Some("image/gif"),
        Some("tif") | Some("tiff") => Some("image/tiff"),
        Some("heic") | Some("heif") => Some("image/heic"),
        Some("avif") => Some("image/avif"),
        _ => None,
    }
}

/// Reads and validates any C2PA manifest embedded in the file at `path`,
/// returning a three-state verdict. Never returns an error for "no
/// manifest" — that's a normal, expected outcome (state 3), not a failure.
#[tauri::command]
pub fn analyze_media(path: String) -> Result<AnalysisResult, String> {
    let p = Path::new(&path);

    if !p.exists() {
        return Err(format!("File not found: {path}"));
    }

    let mime = mime_for_path(p)
        .ok_or_else(|| format!("Unsupported or unrecognized file type: {path}"))?;

    let file = std::fs::File::open(p).map_err(|e| format!("Could not open file: {e}"))?;

    match Reader::from_stream(mime, file) {
        Ok(reader) => Ok(build_result_from_reader(&reader)),

        // No embedded (or sidecar) manifest at all. This is state 3 and is
        // explicitly NOT evidence of anything about the file.
        Err(c2pa::Error::JumbfNotFound) => Ok(AnalysisResult {
            verdict: Verdict::NoProvenance,
            summary: "No verifiable provenance data was found in this file. This is common, \
                      even for genuine, unedited photos — it is not evidence that the file \
                      is inauthentic or AI-generated."
                .to_string(),
            manifest_json: None,
            signer: None,
            claim_generator: None,
            notes: vec![],
        }),

        // Any other error (malformed JUMBF, unreadable stream, etc.) —
        // treat conservatively as "present but broken" rather than
        // silently reporting "no provenance", since something was there.
        Err(e) => Ok(AnalysisResult {
            verdict: Verdict::UntrustedOrBroken,
            summary: "Provenance data was found but could not be read or validated correctly."
                .to_string(),
            manifest_json: None,
            signer: None,
            claim_generator: None,
            notes: vec![e.to_string()],
        }),
    }
}

/// Pulls claim_generator_info[0].name and signature_info.issuer out of the
/// manifest JSON for the active manifest, if present. Deliberately JSON-
/// based rather than typed-API-based; see module doc comment.
fn extract_generator_and_signer(manifest_json: &str) -> (Option<String>, Option<String>) {
    let parsed: Value = match serde_json::from_str(manifest_json) {
        Ok(v) => v,
        Err(_) => return (None, None),
    };

    let active_label = parsed.get("active_manifest").and_then(Value::as_str);
    let active = active_label
        .and_then(|label| parsed.get("manifests").and_then(|m| m.get(label)));

    let claim_generator = active
        .and_then(|m| m.get("claim_generator_info"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("name"))
        .and_then(Value::as_str)
        .map(String::from);

    let signer = active
        .and_then(|m| m.get("signature_info"))
        .and_then(|s| s.get("issuer"))
        .and_then(Value::as_str)
        .map(String::from);

    (claim_generator, signer)
}

fn build_result_from_reader(reader: &Reader) -> AnalysisResult {
    let manifest_json = reader.json();
    let (claim_generator, signer) = extract_generator_and_signer(&manifest_json);

    match reader.validation_state() {
        ValidationState::Trusted => AnalysisResult {
            verdict: Verdict::Verified,
            summary: format!(
                "This file has a verified provenance chain from {}.",
                signer.clone().unwrap_or_else(|| "an identified signer".to_string())
            ),
            manifest_json: Some(manifest_json),
            signer,
            claim_generator,
            notes: vec![],
        },
        ValidationState::Valid => AnalysisResult {
            verdict: Verdict::UntrustedOrBroken,
            summary: "This file has a structurally valid provenance manifest, but its \
                      signing certificate does not chain to a source in the local trust \
                      list, so it cannot be marked as fully verified."
                .to_string(),
            manifest_json: Some(manifest_json),
            signer,
            claim_generator,
            notes: vec!["Signature is valid but the issuer is not in the trust list.".into()],
        },
        ValidationState::Invalid => AnalysisResult {
            verdict: Verdict::UntrustedOrBroken,
            summary: "This file has a provenance manifest, but it failed validation \
                      (broken signature, tampered content, or a broken edit chain). \
                      See the raw manifest below for details."
                .to_string(),
            manifest_json: Some(manifest_json),
            signer,
            claim_generator,
            notes: vec!["Manifest validation reported errors — see raw JSON.".into()],
        },
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![analyze_media])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
RUST_EOF
echo "lib.rs written"
```

Now the frontend. Replace `src/types.ts`, `src/App.tsx`, `src/App.css`, `src/main.tsx`:
