# Verification — application 3.1.13 / xSAR 0.15.1

Status: implementation and all 51 required fresh host gates passed. Console remains 0.8.5. Fresh Windows acceptance remains separate.

Approved scope is the existing diagnostic ABS approach point four native pixels above destination, then exact destination, both retained timing profiles, truthful UI/telemetry and associated regressions. Public APIs, ordinary drag, REL, recognition/calibration, search, AUTO policy and cached manual UNDO ALL recovery retain their contracts.

## Continuity and tested content

The previous 3.1.12 scoped source/evidence sentinel passed before mutation using independent closeout SHA-256 `77a694a3cbd611e2f2be9716a3f99961ffa54c6834219c098edad6ac2e84bb13`. All 1,043 previous compiler inputs matched. Starting Git state, 1,082 exact preimages and original operator failure copies are retained under `Evidence/FreeCell/Validation/freecell-approach-313-20261111/`. No new 3.1.13 sentinel pass is claimed; its expected digest must come from the independent delivery record.

All final gates bind 1,046 compiler/configuration/test inputs in `qualified-inputs.json`, SHA-256 `477ad1bd0c9e396e6b0e84584228a3e7276a872e5e37196cc5fbb803ecdf1b57`. Each receipt verifies unchanged inputs and toolchain/configuration before and after execution. Selected tools are `rustc 1.102.0-nightly (ac8310ba6 2026-11-09)`, `cargo 1.102.0-nightly (c07ac2c4d 2026-11-06)` and `rustfmt 1.12.0-nightly (ac8310ba64 2026-11-09)`, target `x86_64-unknown-linux-gnu`. Full tool versions and inherited configuration are in `toolchain-qualified-inputs.json` and every gate receipt. Existing edition, MSRV, optimisation and dependencies are preserved.

The path dependencies remain unversioned `../xsar`; a path or lock entry is not a source revision pin. Exact dirty source is identified by the seal. Only approved package versions/resolutions changed in manifests and locks. HEAD, index and staged fields are preserved in all four repositories; unrelated server state and prior sealed content are unchanged. Working binary-diff fingerprints at qualification are recorded below; later documentation updates are separately sealed at closeout.

| Repository | HEAD | Unstaged binary-diff SHA-256 |
| --- | --- | --- |
| basic-route-planning | `98ac0a7bc02b799829dfddfa251298890c1c1b9f` | `9df15d7de05f50c0bb78e4a62e4d91b8a651cb411d36f8b6b82013a8d29e6d7e` |
| qemu-screen-grab-server | `3e4f76ed4f9c2e0dea4edd91646f6fd87dc78345` | `311639238a6ed6e118a68cae88259af4fdedb7c316869bd62e31d4fca7ea21f3` |
| qmp-qemu-socket | `c541e878440b19e3f7aaa289d7e3c92ed895fe2c` | `f151fc0882de51844f9059f40d1512335c543c11adde6cb5694c7afce285ab9e` |
| xsar | `59c5639eed1b57e5be51b97c3f35065f92c22b42` | `10fd67ae057bf4965734a5ba2b74a13a1a9135a9e5c38affe97be32e2827f2db` |

## Fresh checks

All 51 fresh paired host gates passed: 1,886 application, 354 library and 80 console tests, zero failed/ignored/filtered.

The schedule includes both scoped owning format checks, locked offline all-target checks, strict private rustdoc and Clippy with `-D warnings`, full tests including ignored, library defaults and all sixteen isolated features, actual application/console consumers and all three release builds. Every required test invocation has zero ignored/filtered tests. The default library run passed 14 cases. Isolated feature check and test gates passed:

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

Exact commands, environment, durations and log hashes are in `qualification-summary.json` and named receipts. Representative actual commands were:

```text
cargo +nightly check --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --all-targets
cargo +nightly doc --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --no-deps --document-private-items
cargo +nightly clippy --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --all-targets -- -D warnings
cargo +nightly test --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features -- --include-ignored
cargo +nightly test --locked --offline --package xsar --target x86_64-unknown-linux-gnu --all-features -- --include-ignored
cargo +nightly test --locked --offline --package basic-route-planning --target x86_64-unknown-linux-gnu --all-features -- --include-ignored
cargo +nightly build --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --release --message-format=json-render-diagnostics
```

Cargo used the application target directory, fixed host target, retained historical acceptance deal and capsule-local session directory. Formatting covered exactly the six approved Rust files, edition/style 2024, skip-children, lower blank-line bound 0 and upper bound 2.

## Focused evidence and reviews

The QMP-focused run passed 21 cases, including exact five-packet source/approach/destination sequences under both real timing profiles, source-ACK gating, UP before four held replies, native/mapped bounds and aliases, and retained STOP/command/write recovery matrices. The new `qmp_313_` cases include thirteen valid approach vectors and eighteen invalid vectors under both profiles. Both native application `freecell_3113_` tests passed over local TCP/QMP peers, checking exact native approach coordinates and all four 2-second or 500-millisecond waits. Focused runs filter unrelated tests; the final full suites filter none.

No focused or required qualification attempt failed in this capsule. Independent source/doc and helper-schema reviews found no blocking issue. Their source comparisons confirm unchanged public timing entry points, enum, normal drag, REL, QmpConfig, routing, waits and recovery bodies. Reviews are distinct from executed tests.

Original 3.1.12 failure files are copied unchanged in `operator-312-failure`, with hashes/provenance in `operator-evidence.json`. Operator reports of midpoint card travel and earlier ten-step success motivate the trial; game-side cause, Windows held-button state and one-update lag remain unproved. The possible destination-column-centre waypoint remains future work.

## Qualified artifacts and limits

Qualified mutable application executable: `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, version 3.1.13, 16835936 bytes, mode 0755, SHA-256 `f3f23273841e88131c8d9697492f2a02260ee8748471150d2cfec6a5dd7dd249`. `artifact.json` binds the actual Cargo release artifact/package ID to this tested content. No protected copy, installation or agent launch occurred. Rebuilding does not replace an older running GUI.

`paired-artifacts.json` separately identifies the standalone all-feature xSAR rlib, actual application-consumer feature-unified xSAR rlib and console binary. Different library contexts are not interchangeable evidence. Mutable Cargo artifacts and current toolchain require the sentinel's optional `--check-artifacts --check-toolchain`; default coverage is scoped source/evidence/Git only.

Final scoped credential-pattern scans and content/link checks are sealed in `documentation-checks.json` and `closeout.json`. No installed scanner was available; working/staged and five-commit pattern checks were used without installing tools. Existing identity/path references remain for promotion review.

Local peers and retained pixels prove host packet/timing, bounds, request fences and refusal behaviour. They do not prove Windows pickup/drop, target-region acceptance, reliable minimum pacing or the hypothesised cause. No guest input/new capture, process launch/replacement, driver change, installation, staging, commit, push or publication occurred. Operator Windows observation and milestone acceptance remain separate.

## Code Analysis

No new unsafe code, dependencies or public API is introduced. Checked native destination subtraction and existing floor mapping precede authority callbacks/I/O; y below four refuses without clamping/fallback. Endpoint alias refusal retains precedence and intermediate aliases remain allowed. Source ACK/fresh routing, immutable timing, STOP/context/source/run guards and captured-pointer release-only recovery remain. No uncertain DOWN/motion is replayed; ACK grants no cursor credit. Ordinary effects still require two consecutive fresh exact combined endpoint observations within six. Synchronous waits can delay STOP until the next boundary, and log durability remains best-effort.
