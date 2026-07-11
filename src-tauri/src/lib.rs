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

use c2pa::{Context, Reader, ValidationState};
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
fn analyze_media(path: String) -> Result<AnalysisResult, String> {
    let p = Path::new(&path);

    if !p.exists() {
        return Err(format!("File not found: {path}"));
    }

    let mime = mime_for_path(p)
        .ok_or_else(|| format!("Unsupported or unrecognized file type: {path}"))?;

    let file = std::fs::File::open(p).map_err(|e| format!("Could not open file: {e}"))?;

    match Reader::from_context(Context::new()).with_stream(mime, file) {
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
