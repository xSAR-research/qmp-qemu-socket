# Current handoff — application 3.4.0, xSAR 0.16.3

Application `3.4.0` delivers the six-goal TriPeaks RECON, A* planning, multi-board execution, and plain operator UI pipeline while keeping `xSAR` untouched at `0.16.3` (commit `067ce26`):

- Added the shared partial-observability RECON coordinator (`src/recon.rs`), `CardRead::FaceDown` (`src/game_operations.rs`), and TriPeaks `game_controls::read_card` Stock contraction / Draw-23 empty-tray detection (`src/game_controls.rs`).
- Added Beast `1920×1080` TriPeaks captures, templates, and read-only HUD/modal observers (`tests/fixtures/tripeaks-calibration-20261116/`, `src/tripeaks_progress.rs`, shared `src/pyramid_progress/challenge/`).
- Built the pure `52`-slot TriPeaks model, replay verifier, admissible `xsar::search` A* & `BestEffortProblem` adapters, and `65,536`-byte atomic JSON persistence (`src/tripeaks_model.rs`, `src/tripeaks_model/{rules,replay,search,json}.rs`).
- Implemented `LiveTriPeaksDomain` (`src/worker/tripeaks_reconnaissance.rs`), Draw-23 `"No Moves Remaining"` `500 ms` hold-click modal back-out, confirmation-less `Undo All` restoration (`src/game_operations/{session,restoration}.rs`), and `CompletedDuringRecon` interception.
- Enabled `GameMode::TriPeaks` under `SolvingStrategy::ShortestPath` (`src/strategy.rs`) with `EscalatedClearBoard` multi-board planning (`src/worker/tripeaks_calculation.rs`), shared QMP route playback (`src/worker/tripeaks_route.rs`), `Continuous` / `Single board` / `Manual` session runner (`src/worker/tripeaks_session.rs`, `RestartPolicy::IndependentTriPeaks`, `0`-click `Continue`/`Redeal`, `--no-image`-aware `GAME WIN` capture), plain `Known / Unknown / Unreachable` `egui` controls (`src/tripeaks_workflow_model.rs`, `src/app/tripeaks_workflow.rs`), and qualified [tripeaks-master-spec.md](tripeaks-master-spec.md).

## Tier 2 host verification summary (Beast)

| Gate | Command | Result |
| --- | --- | --- |
| Format | `cargo fmt --package qmp-qemu-socket -- --check` | Pass |
| Lib/bin strict Clippy | `cargo clippy --workspace --lib --bins -- -D warnings -D clippy::unwrap_used -D clippy::dbg_macro -D clippy::todo` | Pass |
| All-targets Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| Default tests | `cargo test --workspace --offline --locked` | `1,818` passed, `0` failed, `185` ignored (`src/lib.rs`; `0` in `src/main.rs`, `0` doc-tests) |
| Full tests | `cargo test --locked --offline --package qmp-qemu-socket -- --include-ignored` | `2,003` passed, `0` failed, `0` ignored (`src/lib.rs`; `0` in `src/main.rs`, `0` doc-tests) |
| Private `cargo doc` | `RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline --package qmp-qemu-socket --no-deps --document-private-items` | Pass |
| Release build | `cargo build --locked --offline --release --package qmp-qemu-socket --target x86_64-unknown-linux-gnu` | Pass |

- **Release executable:** `target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket` (`17,493,464` bytes, mode `0755`, SHA-256 `b4723b40647461f2012423f61f7c0d2e0e6a7dc87ccc57b38f0ebf9126b72bd2`)
- **Live acceptance status:** Live Windows 11 guest verification on Beast covering TriPeaks `Continuous` (3-board Normal loop + multi-board Challenge `EscalatedClearBoard` + `0`-click `Continue` retirement), `Single board`, Draw-23 `"No Moves Remaining"` `500 ms` hold-click modal back-out + `Undo All`, `Manual` (`RECON / Export 52-slot deal`, `CALCULATE`, `SINGLE STEP`, `MULTI STEP`, `Stop after step`, `UNDO ALL`), unsolvable partial banking vs `0`-click `ClearBoard` dead ends, and default `GAME WIN` frame capture vs `--no-image` (alongside existing Free Cell and Pyramid regressions) remains pending operator launch:

```sh
cargo run --release
```

Use `cargo run --release -- --no-image` to disable `GAME WIN` sequence recording while keeping failure diagnostics.

## Continuity check

External `--expected-closeout` Python sentinel scripts are retired from `3.2.0` onwards per operator direction (the prior `3.1.21` sentinel requirement was waived). Continuity relies on repository state in `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `docs/milestones.md`, `git status --short`, and locked Cargo checks.

### Archival 3.1.21 and 3.1.20 handoff notes (historical reference only)

- `3.1.21` qualified the destination synchronisation package (`1,923` tests, `xSAR 0.16.3`) with `3♦` and `8♣` ambiguities noted as pending (resolved in `3.2.0`).
- Historical `3.1.21` destination receipt: `Evidence/FreeCell/Validation/sync-321-20261113-_1zupweo/qualification-summary.json`.
