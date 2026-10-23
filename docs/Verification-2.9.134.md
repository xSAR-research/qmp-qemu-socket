# Timing-constants verification — 2.9.134

The four-repository timing refactor passed required host verification at **2026-10-23T12:32:41.880991+00:00**. It moves fixed policies into each `src/parameters.rs` without tuning their values. The [catalogue](Timing-Constants.md) lists 66 timing declarations, their roles and tuning boundaries. Source edits require rebuilding and restarting; they do not change a running process.

| Repository | Version | Full host tests | Failed / ignored / filtered |
| --- | --- | ---: | --- |
| xsar | 0.8.105 | 289 | 0 / 0 / 0 |
| basic-route-planning | 0.8.5 | 80 | 0 / 0 / 0 |
| qmp-qemu-socket | 2.9.134 | 1359 | 0 / 0 / 0 |
| qemu-screen-grab-server | 0.1.4 | 19 | 0 / 0 / 0 |

## Verification

All four owning packages passed locked/offline all-target/all-feature Cargo check, strict private rustdoc (`-D warnings -D missing_docs`), all-target/all-feature Clippy (`-D warnings`), complete include-ignored tests and native release compilation. Target: `x86_64-unknown-linux-gnu`; Rust 1.101.0-nightly (b6eeb93c9). Scoped formatting covered the 17 changed Rust files; workspace `--all` was not used.

The app suite took **439.424 seconds** of command time. xSAR feature rows passed **14 / 55 / 25 / 67 / 57 / 99** tests (overlapping). Isolated QMP/TCP checks and dependency-free default/search/execution/combined trees passed. Actual consumers used sibling xSAR without temporary overrides or new dependencies.

There are **672 sealed compiler/config/fixture inputs**. Successful checks are retained only when the owning package and its actual sibling dependency inputs match. Failed attempts remain recorded. The private validation bundle `timing-constants-20261023` retains `qualification.json`, `commands.json`, `baseline.json` and `final-audit.json`, including argv, environment, UTC bounds, hashes and elapsed times. These local evidence files are not part of the published repositories.

## Retained corrections

The first app Clippy attempt found unused `std::time::Duration` imports in `src/worker/computed_route/continuation.rs` and `src/worker/pyramid_session.rs` after timings moved. Removing the unused local import and gating the test-only import fixed this. Clippy then passed and the full app suite ran once on final source.

An initial validation-helper resume incorrectly read the last appended log instead of the retained dependency-tree record. Its assertion stopped before new Cargo work. Retention was corrected to use the returned record; unchanged successful checks were retained. Neither issue was hidden by weakening checks or excluding tests.

## Behaviour and boundaries

- Shared xSAR QMP defaults remain caller-overridable; game/input/STOP policy stays in the app.
- Recon/remaining-stock DRAW retains 500 ms. Ordinary planned MOVE/RECYCLE remains 1 s and card actions 2 s; final settle remains 200 ms within the existing two-second ACK-relative budget.
- Existing UI clamps/immutable settings, exact effect proofs, clocks and no-uncertain-input-replay rules remain. Partial-board REDEAL stays 1.5 s; normal board REDEAL stays 4 s.
- Repaint/reporting/poll intervals are responsiveness or logging controls. Application search remains operator-controlled, while xSAR/console defaults retain 120 s and caller overrides.
- Caller/runtime durations, measured elapsed conversions and independent test fixtures retain their original owners.

No Windows target/GDI verification or fresh Beast trial occurred. Linux server tests cover its portable runtime; its Linux executable retains unsupported-platform behaviour. No installation, GUI launch, process replacement, guest input, commit/push or registry publication occurred. The protected 2.9.133 executable is preserved. M10 and NoChallenge acceptance remains; M11/full target-aware completion and M14 acceptance remain open. This refactor does not correct the separately reported Fours caption refusal.

## Changed scope

**xsar**

- `xsar/CHANGELOG.md`
- `xsar/Cargo.lock`
- `xsar/Cargo.toml`
- `xsar/README.md`
- `xsar/src/lib.rs`
- `xsar/src/parameters.rs`
- `xsar/src/qmp.rs`
- `xsar/src/search.rs`
- `xsar/src/tcp/mod.rs`

**basic-route-planning**

- `basic-route-planning/CHANGELOG.md`
- `basic-route-planning/Cargo.lock`
- `basic-route-planning/Cargo.toml`
- `basic-route-planning/README.md`
- `basic-route-planning/src/parameters.rs`

**qmp-qemu-socket**

- `qmp-qemu-socket/CHANGELOG.md`
- `qmp-qemu-socket/Cargo.lock`
- `qmp-qemu-socket/Cargo.toml`
- `qmp-qemu-socket/README.md`
- `qmp-qemu-socket/src/app.rs`
- `qmp-qemu-socket/src/parameters.rs`
- `qmp-qemu-socket/src/pyramid_recon.rs`
- `qmp-qemu-socket/src/worker/computed_route.rs`
- `qmp-qemu-socket/src/worker/computed_route/continuation.rs`
- `qmp-qemu-socket/src/worker/computed_route/search_log.rs`
- `qmp-qemu-socket/src/worker/pyramid_session.rs`
- `qmp-qemu-socket/src/worker/tcp_capture.rs`
- `qmp-qemu-socket/docs/Timing-Constants.md`
- `qmp-qemu-socket/docs/Verification-2.9.134.md`

**qemu-screen-grab-server**

- `qemu-screen-grab-server/CHANGELOG.md`
- `qemu-screen-grab-server/Cargo.lock`
- `qemu-screen-grab-server/Cargo.toml`
- `qemu-screen-grab-server/README.md`
- `qemu-screen-grab-server/src/main.rs`
- `qemu-screen-grab-server/src/parameters.rs`
- `qemu-screen-grab-server/src/server.rs`

## Code Analysis

Existing bounded I/O, queues, checked durations/clocks, immutable snapshots and cooperative STOP checks remain. Polls still bound cancellation/shutdown responsiveness; waiting never grants input authority or retries uncertain gestures. Rust ownership and capture-thread/GDI boundaries are unchanged, with no new dependency or unsafe code. Shorter future waits need measured native verification and a reliable margin; source constants are not live configuration.
