import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { AnalysisResult, Verdict } from "./types";
import "./App.css";

// Kept in sync with mime_for_path() in src-tauri/src/lib.rs.
const IMAGE_EXTENSIONS = [
  "jpg",
  "jpeg",
  "png",
  "webp",
  "gif",
  "tif",
  "tiff",
  "heic",
  "heif",
  "avif",
];

// Presentation for each of the three verdict states (docs/PROJECT.md §6).
// The label/symbol are deliberately calibrated, never a binary real/fake.
const VERDICT_META: Record<
  Verdict,
  { label: string; symbol: string; className: string }
> = {
  verified: {
    label: "Verified Provenance",
    symbol: "✓",
    className: "verdict--verified",
  },
  untrusted_or_broken: {
    label: "Provenance Present — Untrusted or Broken",
    symbol: "!",
    className: "verdict--untrusted",
  },
  no_provenance: {
    label: "No Provenance Found",
    symbol: "?",
    className: "verdict--none",
  },
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
      filters: [{ name: "Images", extensions: IMAGE_EXTENSIONS }],
    });
    if (typeof selected === "string") {
      await analyze(selected);
    }
  }, [analyze]);

  // Native Tauri drag-drop — unlike HTML5 drag-drop, this hands us real
  // filesystem paths that the Rust backend can open.
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

  const verdictMeta = result ? VERDICT_META[result.verdict] : null;

  return (
    <main className="app">
      <header className="app__header">
        <h1 className="app__wordmark">Verascope</h1>
        <p className="app__tagline">
          Offline content provenance &amp; authenticity — checked locally, on
          your device.
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
          {loading
            ? "Analyzing…"
            : "Drop an image here, or click to choose a file"}
        </p>
        <p className="dropzone__hint">
          {IMAGE_EXTENSIONS.map((e) => e.toUpperCase()).join(" · ")}
        </p>
        {fileName && !loading && (
          <p className="dropzone__file">{fileName}</p>
        )}
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

          {/* Phase 2 stub. Kept structurally and visually separate from the
              provenance verdict above — a heuristic guess must never be
              presented as, or blended into, cryptographic provenance
              (docs/PROJECT.md §2.3). */}
          <section className="heuristic" aria-disabled="true">
            <div className="heuristic__header">
              <span className="heuristic__icon">🔍</span>
              <h3 className="heuristic__title">AI-Artifact Signal</h3>
              <span className="heuristic__tag">
                heuristic · non-authoritative
              </span>
            </div>
            <p className="heuristic__body">
              Pixel-pattern estimation of likely AI-generation is planned for a
              later release (Phase 2). When available, it will be a best-effort
              statistical guess shown here, separate from the provenance result
              above — never a verified fact, and never evidence on its own.
            </p>
          </section>
        </>
      )}

      <footer className="app__footer">
        Everything runs locally. No file ever leaves your device. Absence of
        provenance data is not evidence that a file is fake or AI-generated.
      </footer>
    </main>
  );
}

export default App;
