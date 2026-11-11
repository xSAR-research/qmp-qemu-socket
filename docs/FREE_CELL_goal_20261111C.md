# Free Cell midpoint and timing trial — 11 November 2026

Charlie approved application 3.1.12 and xSAR 0.15.0. This is a host implementation and qualification task; Windows observation and milestone acceptance remain separate.

For A* Free Cell ABS SINGLE STEP, replace the duplicated destination with the native coordinate midpoint followed by the exact destination. Preserve five input requests: source, LEFT down, midpoint, destination, LEFT up. SINGLE STEP uses two seconds between consecutive input commands; an additional SINGLE STEP 0.5 s button uses 500 milliseconds at each of the same four boundaries. Both requests freeze their timing before queueing. The faster button is ABS-only; REL retains its existing diagnostic probe.

Precompute and validate both endpoints and the widened integer native midpoint before I/O. Map each point through the existing checked floor mapping. Retain source acknowledgement before DOWN, fresh routing, STOP/context/source/run fences, release before waiting for held acknowledgements, targeted release-only recovery and exact physical effect proof. No extra click, uncertain replay, route credit from acknowledgement or automatic guest test is authorised.

Preserve ordinary remaining-route drag, REL, recognition/calibration, search, AUTO policy, cached manual UNDO ALL retry, dependencies and target configuration. Add an additive xSAR timing API while preserving the existing diagnostic method signature with the two-second default. Update the application and both path-consumer lock resolutions; console stays 0.8.5.

Acceptance requires scoped formatting, locked offline all-target checks, strict private rustdoc, Clippy, complete tests including ignored, defaults and isolated library features, real consumer integration and release artifacts bound to their actual input/toolchain seals. Preserve first failures, prior evidence, all four Git HEADs/indexes/staged changes and unrelated edits. No installation, process launch/replacement or promotion is included.

See [contract](FreeCell-Midpoint-3.1.12.md) and [verification](Verification-3.1.12.md).
