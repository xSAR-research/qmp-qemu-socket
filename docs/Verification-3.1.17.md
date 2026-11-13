# Verification — application 3.1.17 / xSAR 0.16.2

All 51 fresh paired host gates passed with 1,896 application, 381 library and 80 console tests. The full suites report zero failed, ignored, measured or filtered tests.

The approved [goal H](FREE_CELL_goal_20261111H.md) and [contract](FreeCell-Playback-3.1.17.md) govern the playback correction. The operator reports successful 3.1.16 play using both prior buttons and reports that the first held source-up moved the pointer while the card stayed at source. This is prior-version operator evidence; it does not qualify 3.1.17 or measure Windows held-button state. Fresh 3.1.17 Windows acceptance remains pending. No application launch, new guest input/capture, installation or promotion was performed during this host qualification.

The application supplies exactly three held ABS points: source-up45, the native floor midpoint of ORIGINAL source/effective destination, and exact destination. Source → DOWN → those three points → UP uses six input requests and five minimum intervals of 500 ms for SINGLE STEP and MULTI STEP, or zero inserted gesture delay for fast SINGLE STEP. MULTI STEP freezes ABS and the half-second profile and repeats only after the preceding card play has physical effect proof.

Every newly reached recorded AUTO edge adds 500 ms for animation, including an explicitly transferred AUTO record, an observed readiness prefix and recorded follow-ups. Resumed playback waits only for newly reached records. The route checks STOP, cooperative pause, context and exact ownership before and after each wait; the native wait also checks them between bounded sleep slices. Accepted physical cursor progress is retained if STOP then revokes further work. A wait supplies no route credit.

## Source, publication and toolchain binding

Capsule `Evidence/FreeCell/Validation/freecell-playback-317-20261111/` retains 1,057 baseline compiler inputs, 1,140 preimages and four starting repository Git states.

The prior 3.1.16 Destination Sentinel, using independently supplied closeout SHA-256 `f18f902514d33b69093dc3e4ed37502846e36c3210e123f258d93b6d2f6b2a5c`, returned exactly `HANDOFF REGRESSION: repository fields differ: /home/charlie/repo/RUST/qmp-qemu-socket`. This was the approved publication transition, not an old-sentinel pass. Both exact committed publication guards passed against local/GitHub commits: application `a3e09258c88005ab48ab55d72fe371ac8b5de041` and xSAR `0a59f841f5bb933494bf157f20805e7d0538480b`. All 12,869 prior sealed pins matched. The separate starting-state attestation is `HANDOFF VERIFIED (approved publication reconciled; scoped source/evidence; no cached build claim)`.

All 51 final gates bind the same 1,061 compiler inputs through seal `701a57f139232831ab8903a0293789e237f0463f1c5daf5625f417941d2a7329` at `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-playback-317-20261111/qualified-release-inputs.json`. The toolchain/configuration seal is `577f3299ff52c92186ab0281bc2b63d9021d281d99a19939e26c819915767a52` at `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-playback-317-20261111/toolchain-qualified-release-inputs.json`. Every command checks source, toolchain and Cargo configuration before and after execution. Exact arguments, working directories, environments, times and log hashes are retained in the gate receipts and `qualification-summary.json`.

```text
rustc 1.102.0-nightly (ac8310ba6 2026-11-09)
cargo 1.102.0-nightly (c07ac2c4d 2026-11-06)
rustfmt 1.12.0-nightly (ac8310ba64 2026-11-09)
```

The host target is `x86_64-unknown-linux-gnu`, edition 2024 and MSRV 1.101.0. Application and console use their unchanged xSAR path-only dependency requirements (`../xsar`); their lockfiles resolve xSAR 0.16.2. A local path is not a Git revision pin. The final compiler-input seal identifies the modified path-dependency content alongside the published baseline HEADs. Only approved package-version/owning-lock changes accompany the implementation; console 0.8.5 and its manifest are unchanged.

## Checks and retained first failures

The paired schedule covers scoped formatting, locked/offline all-target checks, strict private rustdoc and Clippy with warnings denied, full tests including ignored cases, release builds, xSAR defaults and all 16 isolated features, and the real console consumer.

The initial focused runs passed 15 transport tests and 8 application tests against their recorded earlier inputs. These are separate focused receipts, not the final full-suite qualification.

The first application Clippy run rejected two test-only `chunks_exact(6)` loops in `src/worker/freecell_route/tests/native.rs` under `clippy::chunks_exact_to_as_chunks`. The loops now use `as_chunks::<6>()`; runtime behaviour was unchanged. `clippy-1.json` and `clippy-1.log` retain the original failure.

The first full application suite passed 1,895 tests and failed the older `freecell_312_final_check_retains_accepted_cursor` phase assertion at `src/freecell_route/tests.rs:903`: actual `(6, "effect")`, expected `(9, "final-check")`. The accepted cursor, progress, four fresh observations, one transfer and exhausted supplied observations already passed their assertions. STOP now correctly pre-empts the new AUTO wait before the obsolete final-check phase. Only this old test was updated: it retains those proof/input assertions and checks that animation waits equal the already accepted root AUTO-prefix count, with no additional wait after STOP. `tests-1.json` and `tests-1.log` preserve the first failure. The focused `accepted-cursor-retry.json` passed 1 test; the final full suites below qualify the final sealed inputs.

Transport peers cover the three-point 500 ms and zero profiles, six inputs/five gaps, source ACK before DOWN, held ACKs withheld until UP, all preflight failures, three/four-point acceptance, independent integer mapping, preserved intermediate aliases, command/write failures, STOP boundaries and original-cause targeted release recovery. Application peers cover both single-step timings, PLAY/upper-slot targets, two verified continuous ABS transfers and per-record AUTO waits across readiness, explicit records and follow-ups, including STOP during animation.

| Gate | Exit | Seconds | Receipt |
| --- | ---: | ---: | --- |
| `xsar-format` | 0 | 0.724 | `xsar-format-3.json` |
| `xsar-check` | 0 | 0.735 | `xsar-check-3.json` |
| `xsar-rustdoc` | 0 | 0.704 | `xsar-rustdoc-3.json` |
| `xsar-clippy` | 0 | 0.746 | `xsar-clippy-3.json` |
| `format` | 0 | 0.792 | `format-3.json` |
| `check` | 0 | 3.504 | `check-3.json` |
| `rustdoc` | 0 | 2.6 | `rustdoc-3.json` |
| `clippy` | 0 | 4.167 | `clippy-3.json` |
| `console-check` | 0 | 0.736 | `console-check-2.json` |
| `console-rustdoc` | 0 | 0.695 | `console-rustdoc-2.json` |
| `console-clippy` | 0 | 0.753 | `console-clippy-2.json` |
| `xsar-default-check` | 0 | 0.728 | `xsar-default-check-2.json` |
| `xsar-default-tests` | 0 | 0.732 | `xsar-default-tests-2.json` |
| `xsar-feature-cards-check` | 0 | 0.697 | `xsar-feature-cards-check-2.json` |
| `xsar-feature-cards-tests` | 0 | 0.732 | `xsar-feature-cards-tests-2.json` |
| `xsar-feature-execution-check` | 0 | 0.697 | `xsar-feature-execution-check-2.json` |
| `xsar-feature-execution-tests` | 0 | 0.744 | `xsar-feature-execution-tests-2.json` |
| `xsar-feature-freecell-check` | 0 | 0.705 | `xsar-feature-freecell-check-2.json` |
| `xsar-feature-freecell-tests` | 0 | 0.772 | `xsar-feature-freecell-tests-2.json` |
| `xsar-feature-freecell_json-check` | 0 | 0.691 | `xsar-feature-freecell_json-check-2.json` |
| `xsar-feature-freecell_json-tests` | 0 | 0.766 | `xsar-feature-freecell_json-tests-2.json` |
| `xsar-feature-freecell_rules-check` | 0 | 0.706 | `xsar-feature-freecell_rules-check-2.json` |
| `xsar-feature-freecell_rules-tests` | 0 | 10.873 | `xsar-feature-freecell_rules-tests-2.json` |
| `xsar-feature-freecell_search-check` | 0 | 0.696 | `xsar-feature-freecell_search-check-2.json` |
| `xsar-feature-freecell_search-tests` | 0 | 11.31 | `xsar-feature-freecell_search-tests-2.json` |
| `xsar-feature-image_matching-check` | 0 | 0.693 | `xsar-feature-image_matching-check-2.json` |
| `xsar-feature-image_matching-tests` | 0 | 0.858 | `xsar-feature-image_matching-tests-2.json` |
| `xsar-feature-pixel_format-check` | 0 | 0.694 | `xsar-feature-pixel_format-check-2.json` |
| `xsar-feature-pixel_format-tests` | 0 | 0.845 | `xsar-feature-pixel_format-tests-2.json` |
| `xsar-feature-png-check` | 0 | 0.71 | `xsar-feature-png-check-2.json` |
| `xsar-feature-png-tests` | 0 | 0.911 | `xsar-feature-png-tests-2.json` |
| `xsar-feature-pyramid-check` | 0 | 0.743 | `xsar-feature-pyramid-check-2.json` |
| `xsar-feature-pyramid-tests` | 0 | 0.774 | `xsar-feature-pyramid-tests-2.json` |
| `xsar-feature-pyramid_json-check` | 0 | 0.717 | `xsar-feature-pyramid_json-check-2.json` |
| `xsar-feature-pyramid_json-tests` | 0 | 1.621 | `xsar-feature-pyramid_json-tests-2.json` |
| `xsar-feature-pyramid_rules-check` | 0 | 0.701 | `xsar-feature-pyramid_rules-check-2.json` |
| `xsar-feature-pyramid_rules-tests` | 0 | 0.754 | `xsar-feature-pyramid_rules-tests-2.json` |
| `xsar-feature-pyramid_search-check` | 0 | 0.688 | `xsar-feature-pyramid_search-check-2.json` |
| `xsar-feature-pyramid_search-tests` | 0 | 0.758 | `xsar-feature-pyramid_search-tests-2.json` |
| `xsar-feature-qmp-check` | 0 | 0.701 | `xsar-feature-qmp-check-2.json` |
| `xsar-feature-qmp-tests` | 0 | 24.792 | `xsar-feature-qmp-tests-2.json` |
| `xsar-feature-search-check` | 0 | 0.686 | `xsar-feature-search-check-2.json` |
| `xsar-feature-search-tests` | 0 | 0.76 | `xsar-feature-search-tests-2.json` |
| `xsar-feature-tcp_capture-check` | 0 | 0.701 | `xsar-feature-tcp_capture-check-2.json` |
| `xsar-feature-tcp_capture-tests` | 0 | 1.429 | `xsar-feature-tcp_capture-tests-2.json` |
| `xsar-tests` | 0 | 39.295 | `xsar-tests-2.json` |
| `tests` | 0 | 98.506 | `tests-2.json` |
| `console-tests` | 0 | 4.869 | `console-tests-1.json` |
| `xsar-release` | 0 | 2.314 | `xsar-release-1.json` |
| `console-release` | 0 | 1.903 | `console-release-1.json` |
| `release` | 0 | 57.766 | `release-1.json` |

## Executable and compilation contexts

The qualified Cargo executable is `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, version 3.1.17, 16833008 bytes, mode 0755, SHA-256 `1e482651f7fc5814f989f3ebd286ede438d5bb43380887afeef85a4859c4eb93`. Cargo may overwrite this artifact on rebuild. Rebuilding does not replace an already running process; fresh Windows acceptance must launch this qualified version deliberately.

Cargo JSON selected each release artifact below. `paired-artifacts.json` retains package IDs, profiles, features, exact paths, bytes, modes, hashes and qualifying command logs. Standalone xSAR and the application-consumer feature-unification context are separate compilation results.

| Context | Version | Path | Bytes | SHA-256 |
| --- | --- | --- | ---: | --- |
| application all-features release | 3.1.17 | `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket` | 16833008 | `1e482651f7fc5814f989f3ebd286ede438d5bb43380887afeef85a4859c4eb93` |
| console all-features release | 0.8.5 | `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/basic-route-planning` | 784016 | `55314c9806304ab4cb8a5a5b60275191fd7b3ccbc33b77d96b52d60e220f4b02` |
| standalone xsar all-features release | 0.16.2 | `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/build/xsar/9b8fd2301be74db2/out/libxsar-9b8fd2301be74db2.rlib` | 1319192 | `f9cedea2d7cbbbdf22d7bc27cdbb03c523012c550fd82b2abfbacee22ff39dc1` |
| application dependency feature-unification release | 0.16.2 | `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/build/xsar/e53e1a2d9827965e/out/libxsar-e53e1a2d9827965e.rlib` | 1519108 | `d6cc09078b74204334c798f595b59be0fe3b01937c3711a30f7a028d6c4dc1af` |

## Preservation

| Repository | HEAD | Index and staged changes |
| --- | --- | --- |
| basic-route-planning | `98ac0a7bc02b799829dfddfa251298890c1c1b9f` | Preserved |
| qemu-screen-grab-server | `3e4f76ed4f9c2e0dea4edd91646f6fd87dc78345` | Preserved |
| qmp-qemu-socket | `a3e09258c88005ab48ab55d72fe371ac8b5de041` | Preserved |
| xsar | `0a59f841f5bb933494bf157f20805e7d0538480b` | Preserved |

All four histories, index bytes and staged diffs match the starting records. Approved source and version/lock differences are checked against preimages. The capture-server tracked diff and unrelated material preserve their prior bytes. The eight current release prefixes preserve their exact preimage historical bodies; prior capsules and sealed evidence remain unchanged.

The new 3.1.17 Destination Sentinel awaits its independently supplied expected closeout digest after final documentation, scans and closeout. No new sentinel pass is claimed. Default verification covers source, evidence and Git state. `--check-artifacts --check-toolchain` additionally compare mutable Cargo outputs and current tools. An expected digest must come from Charlie’s approved independent delivery record, never from the closeout being verified.

## Code Analysis

The application computes source-up45 with checked native bounds and widens original source/destination sums to i64 for the floor midpoint. PLAY uses the observed destination; CELL/SUIT use canonical slot centres. Source y below 45 refuses, and the final upper-slot destination must fit its target. Every point is independently mapped with integer pixel × 32767 / extent before operation input. Intermediate mapped aliases are retained; a final mapped alias of the source refuses.

The additive neutral `QmpClient::diagnostic_drag_path_with_interval` accepts a slice of exactly three or four held points and a `Duration`. Complete count/coordinate/frame/final-alias preflight precedes authority callbacks and I/O. It delegates to the existing release-safe held-input implementation. Public `DiagnosticDragTiming::{TwoSeconds,HalfSecond}`, prior method signatures and four-point behaviour remain unchanged. Application three-point delivery collects all five held-phase replies after UP, while matching source acknowledgement and fresh pointer routing precede DOWN.

Possible post-DOWN failures retain their original cause and use only the established captured-pointer safety release path; failed recovery preserves both causes. A safety release can affect the guest. Original-pointer restoration requires acknowledged UP and continuing authority; a final-UP STOP suppresses restoration. Uncertain non-idempotent input is never replayed.

Every newly reached recorded AUTO edge adds 500 ms for animation, including an explicitly transferred AUTO record, an observed readiness prefix and recorded follow-ups. Resumed playback waits only for newly reached records. The route checks STOP, cooperative pause, context and exact ownership before and after each wait; the native wait also checks them between bounded sleep slices. Accepted physical cursor progress is retained if STOP then revokes further work. A wait supplies no route credit.

Fresh original-52 binding, relevant printed identity proofs, source/plan/run/context ownership, VM-running and absolute-pointer checks remain input conditions. Two consecutive fresh exact recorded endpoints within six observations govern progress. Recognition rules, the single-threaded search, normal drag and REL behaviour retain their contracts. Rust checked arithmetic and bounded path counts add no unsafe memory operations. Pointer routing can change concurrently, socket/scheduling work adds elapsed time, synchronous gesture waits have finite cancellation latency, and session-log persistence remains best effort. Local peers prove outgoing requests and supplied responses; fresh Windows button state and game handling require operator evidence.
