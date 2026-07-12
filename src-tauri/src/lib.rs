// Verascope — Rust backend
//
// Read + validate a C2PA manifest for a single local media file (image,
// video, or audio — Phase 3, PROJECT.md §14) and map the result onto the
// three-state verdict model from the project doc (§6):
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
//
// Trust list (PROJECT.md §10): the app is offline-first, so the C2PA
// trust list is embedded directly into the compiled binary at build time
// (see TRUST_LIST_PEM below) rather than fetched at runtime or shipped as
// a loose Tauri "resource" file. Updating the trust list means replacing
// trust-list/C2PA-TRUST-LIST.pem and trust-list/meta.json and rebuilding
// — an explicit, versioned, auditable act, matching the "never silent"
// requirement in §10. No network call happens as part of this.
//
// Phase 2 (PROJECT.md §2.3): compute_heuristic_signal() below is a
// separate, non-authoritative Error Level Analysis signal. It is never
// merged into the C2PA verdict above — kept as its own Option field so
// the UI is structurally forced to show it separately.

use c2pa::{Context, Reader, Settings, ValidationState};
use image::{codecs::jpeg::JpegEncoder, ExtendedColorType, ImageEncoder};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

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
    /// Secondary, non-authoritative heuristic signal (Phase 2). Always a
    /// separate field from the C2PA verdict above — never blended.
    pub heuristic: Option<HeuristicSignal>,
}

/// Secondary, non-authoritative signal — recompression-error variance,
/// NOT an AI-generation classifier. Deliberately its own type so it can
/// never be conflated with the C2PA verdict (PROJECT.md §2.3).
#[derive(Serialize, Debug)]
pub struct HeuristicSignal {
    /// 0.0–1.0. Relative recompression-artifact magnitude, not a
    /// probability of anything.
    pub score: f32,
    pub summary: String,
}

/// Metadata about the bundled C2PA trust list, surfaced to the UI so it
/// can show the list's age and flag staleness rather than presenting
/// trust validation as silently absolute and current (PROJECT.md §10).
#[derive(Serialize, Debug)]
pub struct TrustListInfo {
    pub bundled_date: String,
    pub source_url: String,
    pub cert_count: usize,
    pub is_stale: bool,
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
        // Phase 3 (PROJECT.md §14): video/audio. c2pa-rs reads these through
        // the same Reader::from_stream(mime, ...) call as images — no new
        // parsing path needed, just widening the accepted MIME set.
        Some("mp4") => Some("video/mp4"),
        Some("mov") => Some("video/quicktime"),
        Some("m4a") => Some("audio/mp4"),
        Some("mp3") => Some("audio/mpeg"),
        Some("wav") => Some("audio/wav"),
        _ => None,
    }
}

// ---------------------------------------------------------------------
// Trust list (PROJECT.md §10)
// ---------------------------------------------------------------------

/// Official C2PA trust list (X.509 root/subordinate CAs recognized by the
/// C2PA conformance program), embedded at compile time so validation
/// works fully offline. Source + bundled date live in trust-list/meta.json.
/// https://github.com/c2pa-org/conformance-public/blob/main/trust-list/C2PA-TRUST-LIST.pem
const TRUST_LIST_PEM: &str = include_str!("../trust-list/C2PA-TRUST-LIST.pem");
const TRUST_LIST_META: &str = include_str!("../trust-list/meta.json");

/// How old the bundled trust list can get before the UI must flag it as
/// stale (PROJECT.md §10). Placeholder value — revisit per Open Question #4.
const TRUST_LIST_STALENESS_DAYS: i64 = 180;

/// Context shared across every `analyze_media` call, built once with the
/// bundled trust list wired in. `Context` is `Send + Sync`; sharing it via
/// `Arc` (rather than rebuilding it per call) is the pattern the c2pa-rs
/// docs recommend for exactly this situation.
static SHARED_CONTEXT: OnceLock<Arc<Context>> = OnceLock::new();

fn shared_context() -> &'static Arc<Context> {
    SHARED_CONTEXT.get_or_init(|| {
        let settings = Settings::new()
            .with_value("trust.trust_anchors", TRUST_LIST_PEM.to_string())
            .expect("bundled trust list PEM is malformed");
        Context::new()
            .with_settings(settings)
            .expect("failed to build C2PA context with bundled trust list")
            .into_shared()
    })
}

/// Days since the Unix epoch (1970-01-01) for a proleptic-Gregorian civil
/// date. Standard algorithm (Howard Hinnant); used instead of pulling in
/// a date/time crate for one staleness calculation.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Parses a "YYYY-MM-DD" string into days-since-epoch. The bundled
/// meta.json is a build-time asset, not user data — a parse failure here
/// means the packaging is broken, so this panics rather than propagating
/// a runtime error.
fn parse_date_to_epoch_days(date: &str) -> i64 {
    let parts: Vec<i64> = date
        .split('-')
        .map(|p| p.parse().expect("bundled trust list date is malformed"))
        .collect();
    days_from_civil(parts[0], parts[1], parts[2])
}

/// Returns the bundled trust list's date, source, and whether it has
/// crossed the staleness threshold, so the UI can show this rather than
/// presenting validation as silently absolute and current (PROJECT.md
/// §10 — never automatic/silent about trust list age).
#[tauri::command]
fn get_trust_list_info() -> Result<TrustListInfo, String> {
    let meta: Value = serde_json::from_str(TRUST_LIST_META)
        .map_err(|e| format!("Bundled trust list metadata is malformed: {e}"))?;

    let bundled_date = meta
        .get("bundled_date")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let source_url = meta
        .get("source_url")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let cert_count = TRUST_LIST_PEM.matches("BEGIN CERTIFICATE").count();

    let bundled_days = parse_date_to_epoch_days(&bundled_date);
    let now_days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64 / 86_400)
        .unwrap_or(bundled_days);
    let is_stale = (now_days - bundled_days) > TRUST_LIST_STALENESS_DAYS;

    Ok(TrustListInfo {
        bundled_date,
        source_url,
        cert_count,
        is_stale,
    })
}

/// Reads and validates any C2PA manifest embedded in the file at `path`,
/// returning a three-state verdict. Never returns an error for "no
/// manifest" — that's a normal, expected outcome (state 3), not a failure.
/// The heuristic signal (Phase 2) is computed once, here, regardless of
/// which verdict branch was hit — it's an independent pixel-level check.
#[tauri::command]
fn analyze_media(path: String) -> Result<AnalysisResult, String> {
    let p = Path::new(&path);

    if !p.exists() {
        return Err(format!("File not found: {path}"));
    }

    let mime = mime_for_path(p)
        .ok_or_else(|| format!("Unsupported or unrecognized file type: {path}"))?;

    let file = std::fs::File::open(p).map_err(|e| format!("Could not open file: {e}"))?;

    let mut result = match Reader::from_shared_context(shared_context()).with_stream(mime, file) {
        Ok(reader) => build_result_from_reader(&reader),

        // No embedded (or sidecar) manifest at all. This is state 3 and is
        // explicitly NOT evidence of anything about the file.
        Err(c2pa::Error::JumbfNotFound) => AnalysisResult {
            verdict: Verdict::NoProvenance,
            summary: "No verifiable provenance data was found in this file. This is common, \
                      even for genuine, unedited photos, videos, or audio recordings — it is \
                      not evidence that the file is inauthentic or AI-generated."
                .to_string(),
            manifest_json: None,
            signer: None,
            claim_generator: None,
            notes: vec![],
            heuristic: None,
        },

        // Any other error (malformed JUMBF, unreadable stream, etc.) —
        // treat conservatively as "present but broken" rather than
        // silently reporting "no provenance", since something was there.
        Err(e) => AnalysisResult {
            verdict: Verdict::UntrustedOrBroken,
            summary: "Provenance data was found but could not be read or validated correctly."
                .to_string(),
            manifest_json: None,
            signer: None,
            claim_generator: None,
            notes: vec![e.to_string()],
            heuristic: None,
        },
    };

    result.heuristic = compute_heuristic_signal(p);
    Ok(result)
}

/// Error Level Analysis: re-save at a fixed JPEG quality, diff against the
/// original. Uneven error levels can indicate localized edits/splicing.
/// NOT specific to AI-generation — a rough, honestly-scoped signal only.
/// Still-frame/pixel analysis only — deliberately not attempted for video
/// or audio (PROJECT.md §14 leaves that an open research question, not a
/// quick add). Returns None (not an error) for anything the `image` crate
/// can't decode — HEIC, or any Phase 3 video/audio file; the C2PA verdict
/// still stands on its own either way.
fn compute_heuristic_signal(path: &Path) -> Option<HeuristicSignal> {
    let original = image::open(path).ok()?.to_rgb8();
    let (w, h) = original.dimensions();

    let mut recompressed_bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut recompressed_bytes, 90)
        .write_image(original.as_raw(), w, h, ExtendedColorType::Rgb8)
        .ok()?;
    let recompressed = image::load_from_memory(&recompressed_bytes).ok()?.to_rgb8();

    let total_diff: u64 = original
        .pixels()
        .zip(recompressed.pixels())
        .map(|(a, b)| {
            (0..3)
                .map(|c| (a[c] as i32 - b[c] as i32).unsigned_abs() as u64)
                .sum::<u64>()
        })
        .sum();
    let mean_diff = total_diff as f32 / (w as u64 * h as u64 * 3) as f32;
    let score = (mean_diff / 30.0).min(1.0);

    let summary = if score > 0.6 {
        "Recompression-error analysis found unusually uneven error levels across the image \
         — sometimes seen in edited or spliced photos. This is a rough, non-authoritative \
         signal, not evidence of AI-generation."
            .to_string()
    } else {
        "Recompression-error analysis found no unusual patterns. This does not confirm the \
         image is untouched."
            .to_string()
    };

    Some(HeuristicSignal { score, summary })
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
            heuristic: None,
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
            heuristic: None,
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
            heuristic: None,
        },
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![analyze_media, get_trust_list_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_for_path_accepts_phase3_video_and_audio() {
        assert_eq!(mime_for_path(Path::new("clip.mp4")), Some("video/mp4"));
        assert_eq!(mime_for_path(Path::new("clip.MOV")), Some("video/quicktime"));
        assert_eq!(mime_for_path(Path::new("voice.m4a")), Some("audio/mp4"));
        assert_eq!(mime_for_path(Path::new("song.mp3")), Some("audio/mpeg"));
        assert_eq!(mime_for_path(Path::new("clip.wav")), Some("audio/wav"));
        assert_eq!(mime_for_path(Path::new("doc.txt")), None);
    }
}
