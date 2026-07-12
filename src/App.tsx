import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { AnalysisResult, TrustListInfo, Verdict } from "./types";
import "./App.css";

// Kept in sync with mime_for_path() in src-tauri/src/lib.rs.
const MEDIA_EXTENSIONS = [
  "jpg", "jpeg", "png", "webp", "gif", "tif", "tiff", "heic", "heif", "avif",
  // Phase 3 (PROJECT.md §14): video/audio.
  "mp4", "mov", "m4a", "mp3", "wav",
];

const VERDICT_META: Record<Verdict, { label: string; symbol: string; className: string }> = {
  verified: { label: "Verified Provenance", symbol: "✓", className: "verdict--verified" },
  untrusted_or_broken: { label: "Provenance Present — Untrusted or Broken", symbol: "!", className: "verdict--untrusted" },
  no_provenance: { label: "No Provenance Found", symbol: "?", className: "verdict--none" },
};

function basename(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

function App() {
  const [result, setResult] = useState<AnalysisResult | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dragging, setDragging] = useState(false);
  const [showRaw, setShowRaw] = useState(false);
  const [trustList, setTrustList] = useState<TrustListInfo | null>(null);

  const analyze = useCallback(async (path: string) => {
    setLoading(true);
    setError(null);
    setResult(null);
    setShowRaw(false);
    setFileName(basename(path));
    try {
      const res = await invoke<AnalysisResult>("analyze_media", { path });
      setResult(res);
    } catch (e) {
      setError(typeof e === "string" ? e : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const pickFile = useCallback(async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Media", extensions: MEDIA_EXTENSIONS }],
    });
    if (typeof selected === "string") {
      await analyze(selected);
    }
  }, [analyze]);

  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      const payload = event.payload;
      if (payload.type === "enter" || payload.type === "over") {
        setDragging(true);
      } else if (payload.type === "leave") {
        setDragging(false);
      } else if (payload.type === "drop") {
        setDragging(false);
        if (payload.paths.length > 0) {
          void analyze(payload.paths[0]);
        }
      }
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, [analyze]);

  useEffect(() => {
    invoke<TrustListInfo>("get_trust_list_info")
      .then(setTrustList)
      .catch(() => setTrustList(null));
  }, []);

  const verdictMeta = result ? VERDICT_META[result.verdict] : null;

  return (
    <main className="app">
      <header className="app__header">
        <h1 className="app__wordmark">Verascope</h1>
        <p className="app__tagline">
          Offline content provenance & authenticity — checked locally, on your device.
        </p>
      </header>

      <section
        className={`dropzone${dragging ? " dropzone--active" : ""}`}
        onClick={pickFile}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            void pickFile();
          }
        }}
      >
        <p className="dropzone__primary">
          {loading ? "Analyzing…" : "Drop an image, video, or audio file here, or click to choose one"}
        </p>
        <p className="dropzone__hint">
          {MEDIA_EXTENSIONS.map((e) => e.toUpperCase()).join(" · ")}
        </p>
        {fileName && !loading && <p className="dropzone__file">{fileName}</p>}
      </section>

      {error && (
        <section className="notice notice--error">
          <strong>Could not analyze this file.</strong>
          <span>{error}</span>
        </section>
      )}

      {result && verdictMeta && (
        <>
          <section className={`verdict ${verdictMeta.className}`}>
            <div className="verdict__stamp" aria-hidden="true">
              {verdictMeta.symbol}
            </div>
            <div className="verdict__body">
              <h2 className="verdict__label">{verdictMeta.label}</h2>
              <p className="verdict__summary">{result.summary}</p>

              {(result.signer || result.claim_generator) && (
                <dl className="verdict__meta">
                  {result.signer && (
                    <div className="verdict__meta-row">
                      <dt>Signed by</dt>
                      <dd>{result.signer}</dd>
                    </div>
                  )}
                  {result.claim_generator && (
                    <div className="verdict__meta-row">
                      <dt>Produced with</dt>
                      <dd>{result.claim_generator}</dd>
                    </div>
                  )}
                </dl>
              )}

              {result.notes.length > 0 && (
                <ul className="verdict__notes">
                  {result.notes.map((note, i) => (
                    <li key={i}>{note}</li>
                  ))}
                </ul>
              )}

              {result.manifest_json && (
                <div className="raw">
                  <button
                    type="button"
                    className="raw__toggle"
                    onClick={() => setShowRaw((v) => !v)}
                  >
                    {showRaw ? "Hide" : "Show"} raw manifest
                  </button>
                  {showRaw && (
                    <pre className="raw__json">{result.manifest_json}</pre>
                  )}
                </div>
              )}
            </div>
          </section>

          <section className="heuristic">
            <div className="heuristic__header">
              <span className="heuristic__icon">🔍</span>
              <h3 className="heuristic__title">AI-Artifact Signal</h3>
              <span className="heuristic__tag">heuristic · non-authoritative</span>
            </div>
            {result.heuristic ? (
              <>
                <p className="heuristic__body">{result.heuristic.summary}</p>
                <div className="heuristic__bar" aria-hidden="true">
                  <div
                    className="heuristic__bar-fill"
                    style={{ width: `${Math.round(result.heuristic.score * 100)}%` }}
                  />
                </div>
                <p className="heuristic__caption">
                  Recompression-error magnitude — not a probability of AI-generation.
                </p>
              </>
            ) : (
              <p className="heuristic__body">
                No heuristic signal available for this file — it's either a format the pixel
                analysis can't decode, or a video/audio file (not yet supported by this signal).
                The provenance result above is unaffected either way.
              </p>
            )}
          </section>
        </>
      )}

      <footer className="app__footer">
        Everything runs locally. No file ever leaves your device. Absence of provenance data is not evidence that a file is fake or AI-generated.
      </footer>

      {trustList && (
        <div className={`trustlist${trustList.is_stale ? " trustlist--stale" : ""}`}>
          <span className="trustlist__dot" aria-hidden="true" />
          <span className="trustlist__text">
            Trust list: {trustList.cert_count} certificate{trustList.cert_count === 1 ? "" : "s"}, bundled {trustList.bundled_date}.
            {trustList.is_stale
              ? " This list may be out of date — a valid signature from a newer authority could show as untrusted until you update it."
              : ""}
          </span>
        </div>
      )}
    </main>
  );
}

export default App;
