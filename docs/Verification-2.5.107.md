# Verification — qmp-qemu-socket 2.5.107

Date: 19 October 2026, Australia/Brisbane.

**Host verified; fresh Beast live acceptance remains pending.**
This is the approved King repair first. The required full scope passed, earning
exactly one correction build/no minor. xSAR 0.5.101 and accepted console
0.8.4 remain fully frozen. M01–M09 are accepted; **M10 Awaiting verification
and M11 Planned**. Historical [2.5.106](Verification-2.5.106.md) and older
verification/live records remain unchanged.

## Correction and evidence

The original `/home/charlie/tmp/.pyramid-recon-710918-3` refusal retained one
native capture, zero input attempts and no export. K♠ at fixed one-based `3|2`
measured old best rank XOR 28 against cap 24. Suit distance 2 passes cap 8;
the other 27 tableau cards and DL 4♣ pass. Native-v9 appends one exact labelled
King rank crop to both tableau and stock reader banks, preserving every old
variant and all native-v8 bytes. Rank/suit caps 24/8, alignment ±1, masks,
scene/modal signatures, geometry and paper/ink/colour checks remain unchanged.

The bundled default becomes native-v9. Literal custom File selection retains
its strict loading/validation contract. Profile declaration and a recognised
single frame do not establish complete stock order, restoration or live authority.
See [profile details](pyramid-recon-profile.md) and
[fixture provenance](../tests/fixtures/pyramid-recon/native-v9/README.md).

The existing three explicit UI steps, automatic evidence names, exact restored
receipt/bytes, independent replay, search policy, per-action effect guards,
STOP/context revocation, no resume and private error packets remain unchanged.
The application has no imposed memory byte cutoff; separate 2M history/expansion,
120-second, cancellation and allocation boundaries remain. Receipt consumption
and current Daily Challenge completion/progress predicates are unchanged.

The native-v9 profile is 803,364 bytes, SHA-256
`b6ffea89b108f222f1d1aee80a951e024588b465465200fc7daa612c20e21865`.
The original 3,744,974-byte baseline has SHA-256
`0b2f53ff00b32233bdbad78659f2c18304972338ae6d5af7a7a1640ece6b1310`.
The exact untransformed RGB crop is `[892,224,27,24]`, SHA-256
`12967fd5fbe85e756a890160803fdb8b6a7616bb81516e46a7a7ac674a79055f`.

## Baseline and actual checks

Work root: `/home/charlie/tmp/pyramid-v9-validation-8s961bw8`.
`baseline.json` records repository/index/file maps before correction writes.
It already contains earlier uncommitted M10 work. Original diagnostics, v8 and
older fixtures, all three indices, every xSAR/console file and unrelated
source/evidence were preserved. Closed compiler/fixture inputs match the checker.
Only automatic `.trigger-tree/history.jsonl` and
`.trigger-tree/sessions/b1065bcab784f174a496f2aa5538322e.json` telemetry differs;
these operational changes were classified and left untouched. Actual commands
and logs are retained under this work root; `verification.json` reports
`passed-required-checks`.

| Check | Actual result |
| --- | --- |
| Focused native-v9 production-reader/negative/provenance checks | 9 passed, 0 failed, 0 ignored; 968 filtered; 8.42 s test time. Overlaps the full suite. |
| Scoped formatting and parent formatting checks | Five modules passed in 0.064 s; four parent shadows in 0.215 s. Existing blank-line-only exceptions preserved in `src/app.rs`, `src/pyramid_recon.rs`, `src/worker/pyramid_reconnaissance.rs`; nonempty changed lines match the formatter. |
| Locked Cargo check | Passed; all targets/all features, 5.324 s |
| Full tests including ignored tests | 977 passed, 0 failed, 0 ignored; 139.941 s command / 139.75 s test report |
| Strict private rustdoc | Passed, 1.066 s; all features, private items, warnings/missing docs denied |
| Clippy | Passed; all targets/all features, warnings denied, 5.574 s |
| Native release and headless help | Passed; release 12.512 s, native help 0.002 s; stable-copy help exit zero |
| Compiler/fixture/index/sibling/unrelated-evidence preservation | Passed; frozen siblings unchanged, all three indices unchanged, only classified automatic telemetry exception |

The initial focused compile failed before test execution:

```text
error[E0689]: can't call method `is_multiple_of` on ambiguous numeric type `{integer}`
```

The error was at `src/pyramid_recon/vision/native_v9_tests.rs:370:35`. Explicit `u32`
checkerboard-coordinate ranges corrected only the test helper, with no
production change. Original source, log and result remain under
`first-focused-failure/`; the subsequent focused run passed all nine cases.

Cargo validation/build commands used explicit `+nightly`, `--locked`, `--offline` and
`--target x86_64-unknown-linux-gnu`. Check/Clippy used all targets/all features;
tests and private rustdoc used all features. Release used package defaults and
Cargo JSON artifact selection. Compiler: `rustc 1.101.0-nightly (8006091eb
2026-10-17)`, full commit `8006091eb71ae7d9e9bccb66c6122c9d68af4466`,
LLVM 23.1.4; Cargo `1.101.0-nightly (3960fa69b 2026-10-12)`. Focused cases
overlap the full suite. Following the retained focused test-helper compile fix,
the complete required sequence passed without a further failure. No shared
xSAR/console suite was rerun or claimed; every file there remains frozen.
Passing host tests do not establish a fresh guest result. No guest input or
capture is part of these host checks.

## Exact stable artifact and runtime

The Cargo-selected native artifact is
`/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`.
A stable copy was created at
`/home/charlie/tmp/pyramid-v9-validation-8s961bw8/release/qmp-qemu-socket-2.5.107`.
Both have SHA-256 `b2460b0dc15327346349425cb45a9e9f285fc840a75593d42c6949f12b1cf6cb`.
The stable copy is 12,563,592 bytes, mode 0555, and headless help exits zero;
`stable-artifact.json` records the verified identity. Its private stable pathname
prevents later broader-work builds from replacing this delivered candidate.

Read-only preflight at `2026-10-19T03:02:29.336844+00:00` still observed old
PID 710918 running `target/release/qmp-qemu-socket --image`, executable
`/home/charlie/repo/RUST/qmp-qemu-socket/target/release/qmp-qemu-socket`, SHA-256
`2b88916a010aa371629f6216fee8d7f5314935b4390ae811a8f359b93c87cb9c`.
Images were zero files/bytes at that instant only. No install, GUI launch, process
replacement, QMP connection or guest operation occurred for this correction.
The [operator brief](Live-Test-2.5.107.md) uses the verified stable path. Startup
clears `/home/charlie/tmp/images`; preserve evidence and recheck at launch.

## Review paths and proposed commit

These 24 explicit paths are the correction delta from `review-paths.json`,
not the full accumulated M10 promotion set. They comprise 11 source/manifest,
four fixture-data and nine documentation paths. No stage/commit/push/publication
is authorised. The final nine-document/147-link/24-path preservation audit passed at
2026-10-19T03:05:37.579251+00:00. Both verified binary copies, compiler inputs,
all three HEADs/indices, frozen siblings and unrelated evidence remain preserved.

```text
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock
/home/charlie/repo/RUST/qmp-qemu-socket/src/app.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/app/pyramid_workflow.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/app/pyramid_workflow_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/native_v8_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/daily_challenge_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/native_v9_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_reconnaissance.rs
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v9/profile.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v9/baseline.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v9/INCOMPLETE.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v9/provenance.json
/home/charlie/repo/RUST/qmp-qemu-socket/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Codex-Current-Status.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Evidence-Index.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/pyramid-recon-profile.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.5.107.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Live-Test-2.5.107.md
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v9/README.md
```

Proposed Conventional Commit title, for review only:

```text
fix(pyramid): recognise the retained native King variant
```

Proposed body:

```text
Append the exact retained King rank sample to both native reader banks and
select the separately named v9 profile. Preserve v8, all old variants, 24/8
XOR caps, one-pixel alignment and existing scene/modal/input guards.

Retain the original one-frame, zero-input refusal and exact fixture provenance.
Record required host checks separately from fresh Beast acquisition/restoration
and live route acceptance. Keep the larger approved automation work separate.
```

## Deferred approved work

Paired 500 ms RECON observations, final two-second completion observation,
manual/one-board/continuous modes and faster search/statistics are planned for
app 2.6.108/xSAR 0.6.102. None is implemented or verified by this repair. A
two-board challenge or changed completion scene can still encounter the existing
completion/progress boundary. This repair grants no retry or resume authority.

## Code Analysis

This adds calibrated native evidence rather than widening the XOR/alignment
rules or inferring unknown identity by elimination. Shared scene, modal, colour,
layout, uniqueness and stock-completeness checks remain authoritative. The added
variant intentionally recognises another supported rendering and therefore needs
production-reader positives and confusable-rank negatives in both banks.

No new unsafe Rust, transport or guest operation is introduced by the calibration
change. An old profile or unsupported render may still refuse safely. A retained
PNG, profile ID or host test cannot recreate a same-session restored receipt.
First-failure evidence is retained; failures stop without uncertain input replay
or cleanup. Fresh observed acquisition/restoration and live effects are required
for Beast acceptance.
