# Verification — application 2.6.114

Status: required host checks passed; final documentation/preservation audit passed.

## Scope and completion boundary

This approved correction adds literal native board-count observation and
observed-progress reconciliation for later Continuous support. The native policy
still supplies no standard-three-board identity; the UI keeps Continuous disabled.
A `/3` denominator cannot establish that identity. Specific standard-game terminal
and restart-control evidence must also be available and its contract validated.
The full goal remains incomplete; M10 remains Awaiting verification.

The session requires fresh paired board/counter facts after every positive route
outcome, including routes already reporting NextBoardReady or GameComplete.
Only the observed exact next count permits another acquisition. Route delivery,
action totals and elapsed time do not supply a displayed count. Within the serial
driver's supported standard-game contract, a supported terminal may authorise
restart only after the expected third clear. Restart must return observed
standard-game 0/3 on a full untouched board before acquisition. Unknown, stale,
backwards, skipped, nonstandard or conflicting facts stop.
Single board returns after its original one-board effect without these series
phases; ordinary Computer Vision restart behaviour is unchanged.

## Reader and evidence

The new reader returns a factual `BoardCount`, never input authority or a game-mode
classification. Its only calibrated counts are 0/3, 1/3 and 2/3. Unsupported
counts, challenge rank/score counters, terminal colours and damaged captions
remain unknown. Every selected native RGB patch pixel participates; no OCR,
resizing, partial masking or deck inference is used. Geometry and accesses use
checked xSAR frame views. Original sources and hashes are retained in the
[fixture](../tests/fixtures/pyramid-progress/README.md).

The current terminal-image audit examined 14 GAME WIN frames and identified them
as Klondike. They provide no Pyramid terminal or restart-control qualification.
The separate read-only record is
`/home/charlie/tmp/pyramid-current-terminal-audit-ypebcoc1/image-audit.json`.

Native-v14 card calibration and all historical card-profile semantics remain
unchanged. xSAR 0.6.102, console 0.8.4, all three Git indices/HEADs and protected
107–113 remain frozen. App114 earns one correction build after required checks;
it consumes no extra milestone minor. No new dependency, unsafe Rust, guest I/O,
GUI, installation, process replacement or promotion is part of host validation.

## Required verification

All commands below passed on `runs/0002`, using the selected nightly and native
`x86_64-unknown-linux-gnu`, locked and offline. The full suite passed
**1,132 tests, zero failed, zero ignored** (command 204.859 s; harness 201.74 s).
Check: 1.718 s; strict private rustdoc: 1.167 s; Clippy: 6.978 s; release: 14.016 s;
native and protected headless help: 0.002 s each. Scoped format checked three new
modules (0.032 s); isolated parent shadows checked three changed existing modules
(0.114 s), preserving standing spacing. No unchanged sibling suite was rerun.

```text
cargo +nightly check --locked --offline --target x86_64-unknown-linux-gnu --all-targets --all-features
cargo +nightly test --locked --offline --target x86_64-unknown-linux-gnu --all-features -- --include-ignored
cargo +nightly doc --locked --offline --target x86_64-unknown-linux-gnu --all-features --no-deps --document-private-items
cargo +nightly clippy --locked --offline --target x86_64-unknown-linux-gnu --all-targets --all-features -- -D warnings
cargo +nightly build --locked --offline --target x86_64-unknown-linux-gnu --all-features --release --message-format=json
/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket --help
```

Rustdoc used `RUSTDOCFLAGS=-D warnings -D missing_docs`. Toolchain:
`rustc 1.101.0-nightly (b6eeb93c9 2026-10-18)`; full commit
`b6eeb93c992027a03e1fd120b6652546396b142c`, LLVM 23.1.4;
`cargo 1.101.0-nightly (3960fa69b 2026-10-12)`.

Coverage includes 14 reader tests, ten actual native-adapter tests (eight private
AF_UNIX cases), and eight new serial-driver tests. These overlap the full suite.
They cover native counts, every required RGB pixel, inclusive nine/ten channel
boundaries, other objectives, 3/3/terminal rejection, malformed layout, padding
and alpha, paired captures, mismatched facts, bounded transition captures,
first-capture STOP/context revocation and evidence retention. They do not supply
standard-game identity or live restart acceptance.

Root independently compared all 12 provenance sources/copies and all six raw RGB
crops with original PNG pixels; exact parity passed. The four positive header
observations have maximum channel drift nine for the left word and three for the
right word. The 11-file fixture manifest includes its frozen README. Work root:
`/home/charlie/tmp/pyramid-series-progress-validation-_jj7_a_o`. Exact logs, source snapshots and
`progress-source-byte-audit.json` retain the evidence.

Protected executable: `/home/charlie/tmp/pyramid-series-progress-validation-_jj7_a_o/release/qmp-qemu-socket-2.6.114`

- Size: 12,762,736 bytes; mode 0555; headless help exit zero.
- SHA-256: `57cd216d4af59cf45075eaadc00ca56c53c5bb19f8a7994f8444e75bf8600c5a`.
- Exact parity with Cargo-selected native release; created in this attempt.
- App114 earns one correction build, no additional milestone minor.

All required commands returned zero before attempt0002's preservation guard
stopped on changed automatic trigger-tree read telemetry. The successful command
seal and first guard result are retained. Exact individual telemetry hashes were
reviewed and audit-only resumption0001 passed; no Cargo, formatter, test or build
was repeated for that operational refresh. All three HEADs/indices, frozen
siblings, earlier card profiles/core/paired tests and protected107–113 passed
preservation checks. Final documentation/preservation audit passed.

Read-only preflight at 2026-10-19T16:27:04.888462+00:00 found PID45759 still running
protected111 with SHA-256
`32638935cae71f240a6c6e733168dbcd1217bad916af0bd3fa10c26aac97b55b`;
`/home/charlie/tmp/images` held 14 files/30,813,884 bytes at that instant.
The fourteen originals were privately preserved and classified as Klondike,
not Pyramid. No controller launch/replacement, guest input or installation occurred.

## First required-check attempt

Attempt `runs/0001` passed the check, strict private documentation and all 1,132
tests (zero failed/ignored). Clippy then refused four constant-size
`chunks_exact`/`chunks_exact_mut` uses in the new observer and its test helpers.
The exact diagnostics and first source snapshot remain under
`/home/charlie/tmp/pyramid-series-progress-validation-_jj7_a_o/first-clippy-failure/`.
The four sites now use the checked `as_chunks`/`as_chunks_mut` API; explicit RGB
length and remainder checks remain. Required checks were rerun and passed;
no second build increment is earned by a failed candidate.

## Review paths and promotion

The following 26 literal correction paths are frozen for host verification.
No staging, commit or push is authorised.

- `/home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock`
- `/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml`
- `/home/charlie/repo/RUST/qmp-qemu-socket/README.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/Codex-Current-Status.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/Live-Test-2.6.114.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Evidence-Index.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.6.114.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/src/main.rs`
- `/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_progress.rs`
- `/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_progress/tests.rs`
- `/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_session.rs`
- `/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_session/native_tests.rs`
- `/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_session/tests.rs`
- `/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-progress/README.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-progress/fraction-one.rgb`
- `/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-progress/fraction-two.rgb`
- `/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-progress/fraction-zero.rgb`
- `/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-progress/label-left.rgb`
- `/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-progress/label-right.rgb`
- `/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-progress/negative-aces-complete.png`
- `/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-progress/negative-three-of-three-header.rgb`
- `/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-progress/negative-three-of-three.png`
- `/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-progress/provenance.json`
- `/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-progress/zero-of-three.png`
Proposed title: `fix(pyramid): reconcile observed series progress`.

Proposed body:

- Read retained literal board-count headers with checked native RGB evidence.
- Require observed next progress and post-restart zero before continuing.
- Preserve native Continuous gating, card profiles and separate mode authority.

## Code Analysis

The reader is bounded and input-free; its counts alone cannot authorise gameplay.
The driver preserves the same worker reservation and monotonic cancellation.
Failed or contradictory observation stops without replay or cleanup input.
All required host checks passed. Full acceptance remains unproved until native
standard terminal/restart evidence and live acceptance establish the whole path.
