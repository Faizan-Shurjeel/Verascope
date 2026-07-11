export type Verdict = "verified" | "untrusted_or_broken" | "no_provenance";

export interface HeuristicSignal {
  score: number;
  summary: string;
}

export interface AnalysisResult {
  verdict: Verdict;
  summary: string;
  manifest_json: string | null;
  signer: string | null;
  claim_generator: string | null;
  notes: string[];
  heuristic: HeuristicSignal | null;
}

export interface TrustListInfo {
  bundled_date: string;
  source_url: string;
  cert_count: number;
  is_stale: boolean;
}
