# Free Cell destination approach trial — 11 November 2026

Charlie approved application 3.1.13 and xSAR 0.15.1. Replace the ABS diagnostic midpoint with a point four native pixels above the exact destination, then send the exact destination. Retain five input requests: source, LEFT down, upward approach, destination, LEFT up. The existing SINGLE STEP uses four two-second waits; SINGLE STEP 0.5 s uses four 500-millisecond waits and remains ABS-only. Freeze timing before queueing.

Validate source, destination and checked destination.y minus four before authority callbacks or operation I/O. Refuse an unavailable upward approach without clamping or fallback; preserve mapped endpoint alias refusal and permit intermediate mapped aliases. Retain source acknowledgement, fresh routing, STOP/context/source/run fences, UP before held acknowledgement collection, captured-handler release-only recovery, no replay and two successive fresh exact physical endpoint proofs within six observations.

Preserve ordinary route drag, REL, recognition/calibration, search, AUTO policy, cached manual UNDO ALL retry, dependencies and toolchain/target settings. Existing public signatures and timing enum stay unchanged. Console remains 0.8.5; only its path-library lock resolution changes.

Qualification requires scoped formatting, locked offline checks, strict private rustdoc, Clippy, full tests including ignored, default and isolated feature tests, actual consumer integration and release artifacts bound to their input/toolchain seals. Preserve prior sealed evidence, original failures, all four Git histories/indices and unrelated edits. Host implementation does not authorise guest input/capture, launch, installation or promotion.

Operator reports that both midpoint timings visibly dragged the card to the intermediate point. This motivates the approach experiment; the one-update lag and game-side cause remain hypotheses. The operator confirmed a valid column drop region need not be a precise pixel. A centre-of-column waypoint is a future experiment contingent on this trial succeeding, with full/empty target bounds; it is not part of this release.

See [contract](FreeCell-Approach-3.1.13.md) and [verification](Verification-3.1.13.md).
