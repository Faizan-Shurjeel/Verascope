# Verascope Roadmap

Verascope v0.1.0 is released for Windows, macOS, and Linux. It reads and
validates C2PA provenance in images, video, and audio entirely on the local
device. The secondary image heuristic is deliberately non-authoritative and
never changes a provenance verdict.

This roadmap describes active priorities, not delivery promises. The full
product rationale and technical constraints live in [PROJECT.md](PROJECT.md).

## Now

### Improve confidence in the shipped verifier

- Add C2PA fixtures covering verified, untrusted/broken, and no-provenance
  outcomes across supported media types.
- Expand Rust and frontend test coverage around verdict mapping and result
  rendering.
- Exercise packaged Linux builds and document reproducible local smoke tests.

### Make the heuristic defensible

- Build the labelled-corpus calibration harness described in PROJECT.md.
- Publish its threshold sweep, confusion matrix, ROC/AUC, score distributions,
  and known failure cases.
- Keep the panel experimental until evaluation supports a calibrated user-facing
  interpretation.

### Sustain the offline trust model

- Make bundled trust-list update work easier to audit and review.
- Keep trust-list freshness visible in the app and release notes.
- Preserve the no-network guarantee for core verification.

## Next

- Reduce CI and release-workflow duplication while retaining equivalent checks.
- Improve accessibility, including keyboard navigation and screen-reader
  coverage.
- Add contributor-facing documentation, issue triage, and release smoke-test
  procedures.
- Evaluate signed builds when certificate management is sustainable.

## Potential contributor projects

These are intentionally bounded places to start:

| Area | First useful contribution |
| --- | --- |
| Tests | Add a regression fixture and assertion for one documented C2PA outcome. |
| Calibration | Help specify corpus metadata and CSV output for the offline evaluation harness. |
| CI | Propose a small change that removes setup drift while retaining test coverage. |
| Accessibility | Audit one workflow and submit a focused, reproducible improvement. |
| Documentation | Improve installation or troubleshooting notes using a real supported OS. |

Before starting a larger change, open an issue or discussion so the approach
can be agreed before implementation.

## Boundaries

Verascope is not a binary real/fake detector. A missing manifest is only an
absence of provenance data, not evidence about a file's origin. Cloud analysis,
accounts, and network-backed core verification remain out of scope by design.
