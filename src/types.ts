// Frontend mirror of the Rust backend's serialized types in
// src-tauri/src/lib.rs. Keep these in sync with that file — the string
// values below match serde's `rename_all = "snake_case"` on `Verdict` and
// the snake_case field names on `AnalysisResult`.

/**
 * The three-state verdict model (docs/PROJECT.md §6). Deliberately NOT a
 * binary real/fake. `no_provenance` is a normal, expected outcome and is
 * never evidence that a file is inauthentic or AI-generated.
 */
export type Verdict = "verified" | "untrusted_or_broken" | "no_provenance";

export interface AnalysisResult {
  verdict: Verdict;
  /** Calibrated, non-absolutist human-readable summary. */
  summary: string;
  /** Raw manifest JSON when a manifest was found (present even when broken). */
  manifest_json: string | null;
  /** Best-effort signer/issuer name from the active manifest. */
  signer: string | null;
  /** The tool that produced the manifest, if present. */
  claim_generator: string | null;
  /** Non-fatal validation notes (e.g. why untrusted/broken). */
  notes: string[];
}

/**
 * Metadata about the bundled C2PA trust list (docs/PROJECT.md §10).
 * Surfaced so the UI can show the list's age and flag staleness rather
 * than presenting trust validation as silently absolute and current.
 * Mirrors the Rust `TrustListInfo` struct in src-tauri/src/lib.rs.
 */
export interface TrustListInfo {
  /** ISO date (YYYY-MM-DD) the bundled trust list was captured. */
  bundled_date: string;
  /** Where the bundled list came from, for auditability. */
  source_url: string;
  /** Number of certificates (trust anchors) in the bundled list. */
  cert_count: number;
  /** True once the list has crossed the staleness threshold. */
  is_stale: boolean;
}
