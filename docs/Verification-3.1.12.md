# Verification — application 3.1.12 / xSAR 0.15.0

Status: implementation and all 51 required fresh host gates passed. Console remains 0.8.5. Fresh Windows observation and milestone acceptance remain separate.

Approved scope is the ABS native midpoint, two-second SINGLE STEP and additional 500-millisecond ABS step, additive timing API, frozen request plumbing, related UI/telemetry and regressions. Ordinary route drag, REL, recognition/calibration, search, AUTO policy and cached recovery retain their contracts.

## Continuity and tested content

The previous 3.1.11 scoped sentinel passed exactly before mutation using independently supplied closeout SHA-256 `6c5a1333233cb4029edf3fb462a17a201e245942673a6b7290cdf092825ad47c`. All 1,040 previous compiler inputs matched. The starting state, four Git states and 1,076 exact preimages are retained under `Evidence/FreeCell/Validation/freecell-midpoint-312-20261111/`. No new 3.1.12 sentinel pass is claimed; its expected digest must come from the independent delivery record.

All final gates bind 1,043 compiler/configuration/test inputs in `qualified-final-inputs.json`, SHA-256 `928e92f12bc4399a5906f1d1c527db02fadfd862c2bb6bcaab5476a7db9d22ba`. Each receipt records unchanged input and toolchain/configuration seals before and after execution. Earlier receipts under the first seal are retained and excluded from final qualification.

Selected toolchain: `rustc 1.102.0-nightly (ac8310ba6 2026-11-09)`, `cargo 1.102.0-nightly (c07ac2c4d 2026-11-06)`, `rustfmt 1.12.0-nightly (ac8310ba64 2026-11-09)`, host `x86_64-unknown-linux-gnu`. Full versions and inherited configuration/environment fingerprints are in `toolchain-qualified-final-inputs.json` and every gate receipt. Existing edition, MSRV, target, optimisation and dependencies are preserved.

The path dependencies remain unversioned `../xsar`; a path or lock entry is not a Git source pin. The exact dirty source is identified by the input seal. Application/xSAR/console manifests and locks contain only approved owning version/resolution changes; console remains 0.8.5. HEAD, index and staged changes are preserved in all four repositories, and unrelated server state is unchanged. Recorded HEAD and tracked working-diff fingerprints at qualification are:

| Repository | HEAD | Unstaged binary-diff SHA-256 |
| --- | --- | --- |
| basic-route-planning | `98ac0a7bc02b799829dfddfa251298890c1c1b9f` | `767895bc16395fc019a1c52e681fa24d4c85892be54587bdb1c09af38f0e1c5c` |
| qemu-screen-grab-server | `3e4f76ed4f9c2e0dea4edd91646f6fd87dc78345` | `311639238a6ed6e118a68cae88259af4fdedb7c316869bd62e31d4fca7ea21f3` |
| qmp-qemu-socket | `c541e878440b19e3f7aaa289d7e3c92ed895fe2c` | `a282e07c568245fd22d0ee3cc110ae8cafedcbe584d894d53ff15ad2ff4f95a9` |
| xsar | `59c5639eed1b57e5be51b97c3f35065f92c22b42` | `9c6f4bd4677d27d62c9be32fa3f8713372c40f806c42f4781af0e8bd61529a46` |

## Fresh checks

All 51 fresh paired host gates passed: 1,886 application, 354 library and 80 console tests, zero failed/ignored/filtered.

The 51-command schedule includes both owning scoped format checks, locked offline all-target checks, strict private rustdoc with `-D warnings`, all-target Clippy with `-D warnings`, full tests including ignored, library defaults, all sixteen isolated library features, actual console integration and all three release builds. Every required test invocation has zero ignored and zero filtered tests. The default library test run passed 14 cases. Isolated feature check and test gates all passed:

| Isolated feature | Tests passed |
| --- | ---: |
| cards | 16 |
| execution | 25 |
| freecell | 19 |
| freecell_json | 34 |
| freecell_rules | 22 |
| freecell_search | 84 |
| image_matching | 44 |
| pixel_format | 28 |
| png | 47 |
| pyramid | 41 |
| pyramid_json | 122 |
| pyramid_rules | 57 |
| pyramid_search | 115 |
| qmp | 48 |
| search | 71 |
| tcp_capture | 59 |

Exact commands, flags, environment, attempt numbers, durations and log hashes are in `qualification-summary.json` and the individually named gate receipts. Representative actual application/paired commands were:

```text
cargo +nightly check --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --all-targets
cargo +nightly doc --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --no-deps --document-private-items
cargo +nightly clippy --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --all-targets -- -D warnings
cargo +nightly test --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features -- --include-ignored
cargo +nightly test --locked --offline --package xsar --target x86_64-unknown-linux-gnu --all-features -- --include-ignored
cargo +nightly test --locked --offline --package basic-route-planning --target x86_64-unknown-linux-gnu --all-features -- --include-ignored
cargo +nightly build --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --release --message-format=json-render-diagnostics
```

Cargo commands ran with `CARGO_TARGET_DIR` set to the application `target`, host build target fixed, `FREECELL_ACCEPTANCE_DEAL` bound to the retained historical deal and `QMP_SESSION_LOG_DIR` scoped to this capsule's `session` directory. Strict rustdoc flags are recorded in its receipt. Formatting uses the approved twelve owning Rust paths, edition/style 2024, `skip_children=true`, lower blank-line bound 0 and upper bound 2.

## Focused evidence and first failure

The focused library QMP filter passed 21 cases, including both real timing profiles, exact source/native-midpoint/exact-destination JSON, source-ACK gating, all held replies after UP, widened midpoint/alias/bounds preflight and existing STOP/command/write recovery matrices. The correctly named new application filter `freecell_3112_` passed six cases: actual widget/disabled states, frozen queued timing, both real native-worker timing profiles and both STOP recovery paths. An earlier `freecell_312_` filter passed thirteen retained 3.1.2 regressions; it is not new midpoint coverage. These focused runs precede the lint-only correction; the final full suites exercise the final sealed source.

The initial application Clippy attempt failed with `unnecessary closure used with bool::then` at `src/qmp.rs:183:47` (`clippy::unnecessary_lazy_evaluations`). Its exit-101 receipt and `clippy-1.log` are preserved. The sole correction changes `.then(|| [...])` to `.then_some([...])` for already widened midpoint telemetry arithmetic. All 51 final gates were rerun against the new final seal and passed. No failure receipt was overwritten.

Independent source/doc and helper-schema reviews found no blocking issue. Normal drag, REL runtime, QmpConfig, pointer recovery, recognition and solution/cursor rules were compared against preserved source. These reviews are separate from executed tests.

## Qualified artifacts and delivery limits

Qualified mutable application executable: `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, version 3.1.12, 16835616 bytes, mode 0755, SHA-256 `82d07ddc68831052318376c5860d91c3c434d1c41804cb7b8ac905b94fcdf9d1`. `artifact.json` binds it to the actual release log and Cargo package ID. No protected copy, installation or agent launch occurred.

`paired-artifacts.json` separately identifies the standalone all-feature xSAR rlib, the actual application-consumer feature-unified xSAR rlib and console binary. Different library contexts are not interchangeable receipts. These are mutable Cargo outputs; the default destination sentinel covers source/evidence/Git, while `--check-artifacts --check-toolchain` are required for current cached-build claims.

Final scoped credential-pattern scans and content/link checks are sealed in `documentation-checks.json` and `closeout.json`. No installed scanner was available; working/staged and five-commit pattern checks are used without installing tools. Existing identity/path references remain for promotion review.

Local peers and retained pixels verify packet order, selected minimum delays, typed request fences and effect/refusal handling. They do not establish Windows cursor/button state, card pickup/drop, the necessity of a midpoint, or a reliable minimum delay. No guest input or new guest capture, process launch/replacement, driver change, install, staging, commit, push or publication was performed. The operator must launch the intended build and test the two ABS buttons separately.

## Code Analysis

No new unsafe code or dependencies are introduced. Endpoints and midpoint use checked native bounds, widened integer sums and existing QMP floor mapping before I/O. The timing enum is closed and immutable in each queued run; fast REL is refused before reservation. STOP/source/context/run checks, source ACK and fresh routing remain in place. Post-DOWN uncertainty permits only captured-pointer release recovery, never replay; successful ACKs grant no cursor credit. Ordinary card effects still require two consecutive fresh exact physical endpoint observations within six. Synchronous waits can delay STOP observation until the next boundary. External QMP/Windows acceptance and best-effort log durability remain explicit limits.
