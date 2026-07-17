# Project Document: Offline AI Content & Provenance Verifier
**Working title:** *Verascope* (placeholder — rename freely)
**Version:** 0.1 (Draft)
**Last updated:** July 9, 2026
**Status:** Concept / Pre-development
**License model:** Open-source (community-driven)

---

## 1. Executive Summary

Verascope is a cross-platform desktop application, built in **Rust** with **Tauri**, that lets an ordinary person check whether an image (and eventually video/audio) has verifiable provenance information — and, where that information doesn't exist, offers a clearly-labeled, secondary heuristic signal about likely AI-generation. All processing happens **locally on the user's machine**. No file is ever uploaded to a server.

The core engine is the **c2pa-rs** crate, which reads and validates **C2PA (Coalition for Content Provenance and Authenticity)** manifests — structured, cryptographically signed metadata that can travel embedded inside a media file, describing who created it, what tools touched it, and what edits were made.

The project exists to fill a specific, currently underserved niche: a free, offline, honest, non-alarmist tool that helps individuals — casual users and journalists/fact-checkers alike — understand what they can and cannot know about a piece of content's origin.

---

## 2. Problem Statement

### 2.1 Why this, why now
AI-generated and AI-edited images are now a normal part of everyday media consumption. At the same time, an industry standard for provenance (C2PA) exists and is increasingly adopted by camera manufacturers, generative AI tools, and editing software — but almost no consumer-facing tool makes that data easy to inspect, verify, and understand **offline** and **for free**.

### 2.2 The gap this project fills
Most existing "AI image detector" tools:
- Run entirely in the cloud (privacy concern — user's image is uploaded to a third party)
- Rely purely on statistical/heuristic guessing with no transparency about confidence
- Do not touch C2PA data at all, even when it's present and would give a definitive answer
- Present a binary "AI or not AI" verdict, which is often simply *wrong* or misleading

### 2.3 The core insight (and the most important thing to get right)
**C2PA verification and AI-content detection are two different problems, and this project must never blur them together.**

| | C2PA Verification | AI-Artifact Heuristic Detection |
|---|---|---|
| What it checks | A cryptographically signed manifest embedded in the file | Statistical/pixel-level patterns typical of generative models |
| Answer when data exists | Strong, verifiable | Probabilistic, best-effort |
| Answer when data is absent | "No provenance data found" — **not** the same as "not AI" | Still possible to give a heuristic signal |
| Can be stripped? | Yes — most social platforms strip metadata on upload/re-upload | No — works on pixels directly, survives re-encoding somewhat |
| Legal/reputational weight | High (this is closer to fact) | Lower — must be labeled as a guess |

A huge share of AI content circulating online has **no manifest at all**, either because the generating tool never embedded one, or because a platform stripped it. A tool that only does C2PA reading will frequently and unhelpfully report "no data" on exactly the images people are most suspicious of. This project addresses that by pairing genuine provenance verification with a clearly separated, honestly-labeled secondary detection layer — and by never letting the UI imply that "no manifest" means "authentic," or that a heuristic guess is a fact.

---

## 3. What This Is / What This Is Not

**This is:**
- A provenance *reader and validator* first, and an AI-artifact *heuristic estimator* second
- Fully offline — no network calls required for core verification functionality
- Transparent about certainty: every verdict clearly states which category it falls into and why
- A tool that shows its work — full ingredient/edit chain, signer identity, timestamps, trust status

**This is not:**
- A definitive "real vs fake" detector — no such tool can exist, and Verascope will never claim to be one
- A content moderation or bulk-processing pipeline (out of scope for now, see §7.2)
- A replacement for platform-level provenance systems — it's a *reader* of standards that already exist, not a new standard
- A cloud service — there is intentionally no "upload for analysis" mode planned

---

## 4. Target Audience & Personas

Per current scope, the two target personas are:

**Persona A — "The Curious Individual"**
A general consumer who received or found a suspicious-looking image (a screenshot, a forwarded photo, a social post) and wants a quick, trustworthy, private way to check it — without installing spyware-adjacent "AI detector" browser extensions or uploading personal photos to unknown servers.

**Persona B — "The Fact-Checker / Journalist"**
Someone doing due diligence on source material before publishing or reporting — needs more detail (full ingredient chain, signer certificate info, edit history) than a casual user, but is still using the tool as an individual, not as part of an automated pipeline.

**Explicitly out of scope for now:** platform/moderator bulk use, API access, integration into third-party moderation pipelines. This may be revisited later (see Phase 5) but is not a design constraint today.

---

## 5. Core Concepts & Terminology

| Term | Meaning |
|---|---|
| **C2PA** | Coalition for Content Provenance and Authenticity — the open technical standard this project reads |
| **Manifest** | The structured, signed metadata block embedded in a media file describing its provenance |
| **Claim** | A signed assertion within a manifest (e.g., "this was edited by Tool X at Time Y") |
| **Ingredient** | A prior asset referenced by a manifest — enables reconstructing an edit history/chain |
| **Trust List** | A bundled list of certificate authorities/issuers considered trustworthy for signature validation |
| **Signer/Issuer** | The entity (software, camera, service) that cryptographically signed the manifest |
| **Heuristic Detection** | Verascope's secondary, non-C2PA layer that estimates AI-generation likelihood from pixel data alone |
| **Three-State Verdict** | The UI model described in §6 — never collapses to a single real/fake answer |

---

## 6. The Verdict Model (Critical UX Principle)

Verascope will **never** present a single binary "real/fake" badge. Every analyzed file resolves into one of these states, always shown separately from any heuristic signal:

1. **✅ Verified Provenance** — A manifest is present, its signature is cryptographically valid, and the issuing certificate chains to a trusted authority in the local trust list.
2. **⚠️ Provenance Present but Untrusted/Broken** — A manifest exists but fails validation (invalid signature, broken ingredient chain, expired/revoked/untrusted certificate, or manifest is stale relative to the bundled trust list).
3. **❔ No Provenance Found** — No manifest is present. The UI must make explicit that this is **not** evidence of anything — it is simply an absence of data, common even for genuine, unedited photos.

Independently of the above, and always visually separated, a **secondary heuristic panel** may show:

4. **🔍 AI-Artifact Signal (heuristic, non-authoritative)** — e.g., "Pixel-pattern analysis suggests possible AI-generation (confidence: moderate)." This panel is always labeled as a best-effort statistical estimate, never as a verified fact, and is most useful precisely in state 3 above, where C2PA has nothing to say.

All UI copy will avoid absolutist language ("this is AI-generated") in favor of calibrated language ("signals suggest…", "no verifiable provenance data was found…"). This is both an ethical commitment and a liability mitigation (see §11).

---

## 7. Goals & Non-Goals

### 7.1 Goals
- Accurate, offline, transparent C2PA manifest reading and validation
- A secondary, clearly-labeled local AI-artifact heuristic detector
- Full ingredient/edit-chain visualization
- Honest, calibrated language throughout the UI
- A genuinely lightweight, fast, native-feeling cross-platform desktop app
- A healthy, welcoming open-source project that others can contribute to and trust

### 7.2 Explicit Non-Goals (for now — see roadmap for revisit points)
- Bulk/batch processing or API access for platforms/moderators
- Cloud-based analysis or account systems
- Acting as a new provenance *standard* — Verascope only reads/validates the existing C2PA standard
- Legal certification services ("certificate of authenticity" documents) — at least not in early phases

---

## 8. Key Differentiators

- **Actually reads provenance data**, not just pixel-guessing — most "AI detector" tools in the market skip this entirely
- **Fully offline** — genuine privacy, no image ever leaves the user's device
- **Honest three-state model** instead of a false-confidence binary badge
- **Native performance** — Rust + Tauri means a small install size and low resource use compared to Electron-based alternatives
- **Open source** — auditable, and trust in a *trust-verification tool* benefits enormously from being inspectable by anyone

---

## 9. System Architecture (Overview)

```
┌─────────────────────────────────────────────┐
│                Tauri Shell                   │
│  (native window, OS integration, file I/O)   │
├───────────────────────┬───────────────────────┤
│   Frontend (WebView)  │     Rust Backend       │
│  UI, verdict display, │  - c2pa-rs integration │
│  ingredient chain     │  - local trust list    │
│  visualization        │  - heuristic AI model  │
│                       │    runner (ort/tract)  │
│                       │  - file parsing/IO      │
└───────────────────────┴───────────────────────┘
```

**Key components:**
- **Tauri** — application shell, native OS integration, small binary size, secure IPC bridge between frontend and Rust core
- **c2pa-rs** — manifest parsing, signature validation, ingredient chain traversal
- **Bundled trust list** — a maintained, versioned list of trusted issuing certificates, shipped with the app and periodically updatable (see §10)
- **Local heuristic model runner** — likely `ort` (ONNX Runtime bindings) or `tract` (pure-Rust inference) to run a lightweight AI-artifact classifier fully on-device
- **Frontend** — a web-based UI (framework TBD — see Open Questions) rendered inside Tauri's native webview

Frontend framework, exact heuristic model choice, and packaging details are intentionally left open for Phase 1 research rather than fixed here (see §15).

---

## 10. Trust List & Certificate Management (Offline Realities)

Because the app has no live network dependency, it ships with a **bundled, versioned trust list** used to validate signer certificates. This introduces a real tradeoff that must be surfaced honestly to the user rather than hidden:

- The bundled list will go stale over time as new issuers are added or old ones are revoked.
- The UI must show the trust list's version/date and flag when it is old, rather than silently presenting validation as absolute and current.
- An **optional, explicit, user-initiated** "check for trust list updates" action can be offered without breaking the "offline-first" principle — the app works fully offline by default, but the user may choose to fetch an updated list when connected. This must never be silent or automatic without consent.

---

## 11. Legal, Ethical & Language Guidelines

Because verdicts here can influence real decisions (what someone believes, shares, or publishes), calibrated language is a first-class design requirement, not an afterthought:

- Never say "this image is AI-generated" — say "no verifiable provenance was found, and heuristic signals suggest possible AI-generation."
- Never say "this image is authentic/real" — say "this image has a verified provenance chain from [signer]."
- Always disclose the heuristic detector's non-authoritative nature adjacent to any confidence score it shows.
- Maintain a visible, dated note on trust list staleness (§10).
- Consider an in-app "how to read this result" explainer, always one click away from any verdict screen.

---

## 12. Success Metrics (Prospective)

These are early hypotheses, expected to evolve once real usage data exists:

- Time-to-verdict for a typical image (target: near-instant, offline)
- Correct three-state classification rate on a curated test set of known-provenance images
- Heuristic detector precision/recall on a held-out AI-vs-camera-original dataset (tracked and published openly, including failure cases — false confidence here is the main reputational risk)
- Community engagement: contributors, issues, PRs (open-source health)
- User-reported clarity of verdict language (does it avoid overstating certainty?)

---

## 13. Open Questions / Risks Log

This section is meant to be actively maintained — add to it as decisions get made or new unknowns appear.

| # | Question | Status |
|---|---|---|
| 1 | Which frontend framework inside Tauri (Svelte/React/vanilla)? | Open |
| 2 | Which heuristic AI-detection model/architecture, and where to source/train it? | Open |
| 3 | Exact open-source license (MIT vs Apache-2.0 vs dual) | Open — Apache-2.0 or MIT/Apache dual is common in the Rust ecosystem and worth defaulting toward |
| 4 | How to source and maintain the bundled trust list responsibly | Open |
| 5 | Video/audio support timing — bundled into MVP or deferred? | Leaning: deferred (see roadmap) |
| 6 | Governance model for accepting community contributions | Open |
| 7 | Update/distribution mechanism (installer, package managers, auto-update) | Open |

---

## 14. Roadmap

**This roadmap is intentionally phased and prospective — not a fixed contract.** Each phase ends with a checkpoint to reassess scope, priorities, and whether the next phase still makes sense as planned.

### Phase 0 — Research & Validation *(foundation-laying, no code commitment yet)*
- Deep-dive c2pa-rs API surface, current maturity, and known limitations
- Survey existing C2PA-embedding tools/cameras/AI generators to understand real-world manifest availability
- Decide frontend framework and confirm Tauri version/capabilities fit
- Scope and source a candidate heuristic AI-detection approach (existing open model vs custom-trained)
- Draft initial UI/UX wireframes for the three-state verdict model
- **Checkpoint:** Is the C2PA ecosystem mature enough right now to build on confidently? Adjust timeline if not.

### Phase 1 — MVP: Core Provenance Verification (Images Only)
- Rust backend: load image, extract and parse C2PA manifest via c2pa-rs
- Implement signature validation against bundled trust list
- Implement the three-state verdict UI (§6), no heuristic layer yet
- Basic ingredient-chain display (even if minimal at first)
- Local-only, single-file drag-and-drop workflow
- Trust list staleness indicator
- **Rationale for images-only first:** C2PA image support and tooling is currently the most mature part of the spec/ecosystem, and keeps the MVP scope realistic.
- **Checkpoint:** Does manifest validation work reliably across a broad enough sample of real-world images (camera photos, edited photos, AI-tool exports, social-media re-uploads)?

### Phase 2 — Heuristic AI-Artifact Detection Layer
- Integrate local inference runtime (`ort` or `tract`)
- Add secondary, clearly-separated heuristic panel to the UI
- Publish detector methodology and known limitations openly (transparency as a trust mechanism)
- Begin structured internal testing of false-positive/false-negative rates
- Build a calibration/evaluation harness for the heuristic: run the scorer over a small labelled corpus (AI-generated, camera-original, edited/re-saved images), sweep the verdict threshold, and produce confusion matrix + ROC/AUC + score-distribution plots — so the current eyeballed constants (normalizer 30.0, threshold 0.6 in `compute_heuristic_signal`) become defensible, published numbers instead of magic values. (Methodology borrowed from the maintainer's earlier MalwareRizz project's `malware_evaluate.m` — same labelled-corpus → score → threshold-sweep pattern, different domain.)
- Refine calibrated-language guidelines based on real test feedback
- **Checkpoint:** Is the heuristic signal accurate/honest enough to ship, or does it need more work/should it stay clearly experimental for longer?

### Phase 3 — Expanded Media Support (Video / Audio)
- Extend manifest parsing to video and audio assets (c2pa-rs supports this)
- Adapt the ingredient-chain UI for time-based media
- Evaluate whether heuristic detection extends meaningfully to video/audio or should remain image-focused
- **Checkpoint:** Reassess demand — was this actually needed by real users yet, or should effort go elsewhere first?

### Phase 4 — Open-Source Launch & Distribution
- Finalize license choice (§13, item 3)
- Public repository launch: contribution guidelines, code of conduct, issue templates
- Cross-platform packaging (Windows/macOS/Linux installers)
- Establish an update mechanism for both the app and the bundled trust list
- Public documentation site / in-app help content
- **Checkpoint:** Community health check — are contributions coming in, is triage sustainable?

### Phase 5 — Ecosystem Growth *(exploratory, order not fixed)*
- Possible plugin/extension system for additional heuristic models
- Possible community-maintained/expanded trust lists
- Revisit previously out-of-scope items if demand emerges (e.g., API access, batch tooling) — to be evaluated fresh, not assumed
- Localization/internationalization of UI copy
- Accessibility audit and improvements

---

## 15. Contribution & Community Model

- **License:** Open-source, community-driven (specific license TBD — see Open Questions; MIT or Apache-2.0 are natural defaults given the Rust ecosystem).
- **Governance:** To be defined before Phase 4 launch — even a lightweight "maintainer + CONTRIBUTING.md" model is fine early on, formalize only as needed.
- **Transparency as a core value:** Given this is fundamentally a *trust* tool, the project's own trustworthiness (open code, open methodology, open failure-rate reporting) is as important as any single feature.

---

## 16. Glossary (Quick Reference)

- **C2PA** — Coalition for Content Provenance and Authenticity
- **c2pa-rs** — The official Rust implementation/crate for reading and validating C2PA manifests
- **Manifest** — Embedded, signed provenance metadata
- **Ingredient** — A referenced prior asset in an edit/creation chain
- **Trust List** — Bundled list of certificate issuers considered valid
- **Heuristic Detector** — Non-C2PA, pixel-based AI-generation likelihood estimator
- **Tauri** — Rust-based framework for building lightweight native desktop apps with web frontends
- **ort / tract** — Rust libraries for running ML model inference locally

---

## 17. References & Resources *(to review during Phase 0)*

- C2PA official specification and technical documentation
- c2pa-rs crate documentation and repository
- Tauri official documentation
- `ort` (ONNX Runtime for Rust) and `tract` (pure-Rust inference) documentation

---

*This document is a living artifact. Update the Open Questions log and roadmap checkpoints as decisions are made — treat every phase boundary as a natural point to revisit assumptions rather than a deadline to hit at all costs.*
