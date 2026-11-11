# 3.1.3 — Free Cell acquisition, errors and telemetry

This application-only correction implements [the seven-item goal](FREE_CELL_goal_20261109A.md). Application version is 3.1.3; unchanged xSAR is 0.11.1 at `59c5639eed1b57e5be51b97c3f35065f92c22b42`. The application base is the accepted, published `c541e878440b19e3f7aaa289d7e3c92ed895fe2c`. There are no dependency, xSAR API, codec schema, calibration threshold or transport policy changes.

The current goal approves this patch build, superseding the recovery document's historical pending-version wording. The accepted published baseline supersedes using the pre-publication 3.1.2 sentinel as a current Git-state check. Its original receipt and server discrepancy remain preserved. A separate starting-state record covers all four repositories before this correction.

## Requirement evidence

| Requirement | Implementation and evidence boundary |
| --- | --- |
| 1. FAILURE DETAILS | Immutable RECON failures retain original cause, request, stage, recognition region, acquisition identity and frame metadata. Matching reducers install the exact frame and failure; browser tokens reject stale/duplicate same-directory failures. Local Free Cell dispatch, calculation, capture, snapshot and control errors use populated presentation. Persistence errors remain secondary. |
| 2. Challenge text | Four exact scene rectangles outside the objective area replace blank-banner scene evidence. Solver detection, empty top slots, geometry, complete 52 identities and ambiguity refusal remain independent. The retained later screenshot and labelled synthetic header derivatives exercise this distinction; its highlighted border remains unsupported. |
| 3. Default game | Entering A* selects Free Cell. Ordered five-game selection, unavailable capabilities and saved Computer Vision selection remain. |
| 4. Alignment | Acquisition, source/calculation and playback occupy separate full-width groups with wrapping controls and bounded editors. Headless egui checks use the production container at 640, 840 and 1920 pixel widths. Broader styling remains separate. |
| 5. Fresh preview | Each RECON acquires one fresh TCPv3 response before recognition. Completion owns those exact pixels rather than referring to the generic latest-frame slot. Only its accepted request installs that preview, on success or failure. Evidence encoding uses the same frame without another capture. |
| 6. Telemetry scope | `FREE_CELL` records cover UI intent and serial worker commands only for A* / Free Cell. Other games and Computer Vision retain their existing tracing policy. CALCULATE remains offline. |
| 7. Telemetry detail | Correlated command/source/request/run, capture identity/layout, RECON regions/results, search stages, candidate refusals/verified cursor, QMP qcodes/endpoints/button phases and evidence write paths/results. ACKs remain delivery evidence. Logging adds no capture, transfer, retry or authority. |

## Objective compatibility

The strict shared schema is unchanged. Application-owned `Metadata.source` records `single-frame-tcp-v3;objective=full-board` or `single-frame-tcp-v3;objective=unresolved`. An unresolved objective retains its acquired pixels in a labelled private acquisition packet, with a bounded source reference. A write error is separate from a successfully published deal and does not trigger another publication or capture.

CALCULATE and playback require explicit full-board provenance. Challenge text is not OCR, an inferred target or permission to clear the board. Legacy/unmarked saved deals still decode, but now require RECON and re-export before planning. This deliberate application compatibility restriction prevents missing objective context from silently becoming a full-board objective. Marked deals must still pass fresh original-board and live-position checks before input.

## Qualification

Fresh host qualification passed all six owning-package gates on Beast. The commands used `--locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features`; check and Clippy also used `--all-targets`. The 24 owning Rust files passed the selected nightly formatter with `skip_children=true`.

| Gate | Command / result | Receipt |
| --- | --- | --- |
| Format | `rustup run nightly rustfmt --edition 2024 --config skip_children=true --check <24 owning paths>` — pass | `format-2.json` |
| Check | `cargo +nightly check <common flags> --all-targets` — pass | `check-2.json` |
| Private rustdoc | `cargo +nightly doc <common flags> --no-deps --document-private-items`, both rustdoc flag variables set to `-D warnings` — pass | `rustdoc-2.json` |
| Clippy | `cargo +nightly clippy <common flags> --all-targets -- -D warnings` — pass | `clippy-2.json` |
| Full tests | `cargo +nightly test <common flags> -- --include-ignored` — **1,803 passed, zero failed/ignored/filtered** | `tests-1.json` |
| Release | `cargo +nightly build <common flags> --release --message-format=json-render-diagnostics` — pass | `release-1.json` |

Every passing receipt binds the same 1,022 compiler inputs before and after execution: `qualified-inputs-3.json`, SHA256 `fc3687822599e4f8af2e1c0f5d7c6f82759bfe9c8a920289138c2fde1b90da3a`. Toolchain is rustc `1.101.0-nightly (c980177d8 2026-11-06)`, Cargo `1.101.0-nightly (111443f3d 2026-11-05)`, LLVM 23.1.5, host/target `x86_64-unknown-linux-gnu`; the repository target CPU/linker configuration is included in the input pins. `QMP_SESSION_LOG_DIR` was isolated under the new evidence directory, retaining private diagnostic packets. xSAR 0.11.1 source and its dependency-free defaults are unchanged; no new library/console suite is claimed.

All fourteen new regression tests passed in the full suite. They cover original-cause/request/duplicate fencing, exact preview ownership and two distinct local TCP acquisitions with disk-pixel equality while GAME WIN recording is disabled, successful synthetic challenge acquisition/export evidence, unresolved/unmarked objective refusal before search or any route I/O, independent native/Solver/geometry refusals, production UI container geometry at three widths, and UI/worker/transport telemetry isolation with unchanged action results. The earlier focused `freecell_3` family passed 56 tests before the final browser-detail and test-iterator corrections; the full suite covers the final source. The browser dispatch identity correction also passed independent source review.

First attempts remain retained: the TCP result type inference error, transient cross-agent objective-field compile error, test-only alpha/reference-first-cause assertions, and Clippy's constant chunk iterator warning. The failed 75 MB assertion log is retained losslessly as `focused-app-first.log.gz` with its original/compressed digest receipt. Fixes preserve recognition strictness and do not suppress lints or reduce checks.

The release artifact is the normal mutable Cargo target `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, version **3.1.3**, **16709944 bytes**, mode `0755`, SHA256 `9f11a807ac283ef367a1214435345ac043be8cb09e9c1cca882f8ed3b8f4f5e8`. `artifact.json` selects it from Cargo's release compiler-artifact record (fresh build, release opt-level 3), rather than inferring a path or testing a different executable. No application process was launched or replaced.

All four HEADs, index bytes and staged patches match `starting-state.json`; xSAR, console and server unstaged patches are also unchanged. In particular, the server's independent staged and unstaged work is preserved byte-for-byte against this correction's start, without claiming agreement with the old 3.1.2 destination snapshot. The goal file and restricted records are preserved. All command receipts, toolchain details, original attempts and preservation metadata are under `Evidence/FreeCell/Validation/freecell-correction-313-20261109A/`.


## Limits and acceptance

The later screenshot is not authenticated as either missing original failed RECON frame. Its banner mismatch and separate highlighted-border refusal are the supported conclusions. Native support remains 1920×1080, 100% scale, CLASSIC BLUE, Solver off and supported geometry. Objective interpretation/completion, highlighted geometry, toolbar-clipped columns, partial optimisation and win/restart loops remain separate work.

Host tests, retained pixels and local peers do not establish fresh guest drag effects, autoplay timing or full-route acceptance. This correction performs no guest input, GUI launch, installation, Git promotion or server modification. Failure evidence remains independent of GAME WIN recording and `--no-image`. Startup resets `/home/charlie/tmp/images`; preserve incident material before any separately authorised live test.

## Code analysis

Strict bounded JSON, checked image/coordinate/inventory boundaries and private exclusive publication remain. Persistence errors preserve the primary result. Workers own acquired frames and immutable reports; borrowed previews cannot become authoritative failure evidence. STOP/context checks and release-only recovery retain their authority. Unrecognised objectives refuse planning/playback, and independent scene and geometry checks still refuse unsupported boards. Passive scoped telemetry grants no input authority.
