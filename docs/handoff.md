# Current handoff — application 3.4.1, xSAR 0.16.3

Application `3.4.1` builds on `3.4.0` with the TriPeaks `Undo All` full-Stock rewind settlement fix while keeping `xSAR` untouched at `0.16.3` (commit `067ce26`):

- Added `game_controls::is_tripeaks_full_root_stock` (`src/game_controls.rs`) checking full `23`-card `T = 0` Stock settlement across `x = 836..966`, zero right-overhang transit blue (`x = 968..980`), and settled left deck-edge white paper (`768..818`) so mid-rewind captures where Microsoft Solitaire greys out `Undo All` before all `23` cards land return `FrameClassification::Settling`.
- Required `MIN_CONFIRMED_DRIFT_STREAK = 4` consecutive identical settled non-root frames in `DriverUndoAllIo::observe` and raised `TriPeaksReconConfig.max_restore_observations` from `16` to `24` (`src/worker/tripeaks_reconnaissance.rs`), covered by `full_root_stock_accepts_settled_t0_and_rejects_contracted_or_mid_rewind` (`src/game_controls/tests.rs`) and `undo_all_waits_through_mid_rewind_disabled_toolbar_and_compositor_stall` (`src/worker/tripeaks_reconnaissance/tests.rs`).

## Tier 2 host verification summary (Beast)

| Gate | Command | Result |
| --- | --- | --- |
| Format | `cargo fmt --package qmp-qemu-socket -- --check` | Pass |
| Lib/bin strict Clippy | `cargo clippy --workspace --lib --bins -- -D warnings -D clippy::unwrap_used -D clippy::dbg_macro -D clippy::todo` | Pass |
| All-targets Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| Default tests | `cargo test --workspace --offline --locked` | `1,820` passed, `0` failed, `185` ignored (`src/lib.rs`; `0` in `src/main.rs`, `0` doc-tests) |
| Full tests | `cargo test --locked --offline --package qmp-qemu-socket -- --include-ignored` | `2,005` passed, `0` failed, `0` ignored (`src/lib.rs`; `0` in `src/main.rs`, `0` doc-tests) |
| Private `cargo doc` | `RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline --package qmp-qemu-socket --no-deps --document-private-items` | Pass |
| Release build | `cargo build --locked --offline --release --package qmp-qemu-socket --target x86_64-unknown-linux-gnu` | Pass |

- **Release executable:** `target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket` (`17,494,760` bytes, mode `0755`, SHA-256 `946b28bc029fdcac21d3cbed75024a0df448f3e1e211fc4850ded38dbb6388ed`)
- **Live acceptance status:** Live Windows 11 guest verification on Beast covering `v3.4.1` `Undo All` full-Stock rewind restoration on the waiting Challenge board, TriPeaks `Continuous` (3-board Normal loop + multi-board Challenge `EscalatedClearBoard` + `0`-click `Continue` retirement), `Single board`, Draw-23 `"No Moves Remaining"` `500 ms` hold-click modal back-out, `Manual` (`RECON / Export 52-slot deal`, `CALCULATE`, `SINGLE STEP`, `MULTI STEP`, `Stop after step`, `UNDO ALL`), unsolvable partial banking vs `0`-click `ClearBoard` dead ends, and default `GAME WIN` frame capture vs `--no-image` (alongside existing Free Cell and Pyramid regressions) remains pending operator relaunch:

```sh
cargo run --release
```

Use `cargo run --release -- --no-image` to disable `GAME WIN` sequence recording while keeping failure diagnostics.

## Continuity check

External `--expected-closeout` Python sentinel scripts are retired from `3.2.0` onwards per operator direction (the prior `3.1.21` sentinel requirement was waived). Continuity relies on repository state in `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `docs/milestones.md`, `git status --short`, and locked Cargo checks.

### Archival 3.1.21 and 3.1.20 handoff notes (historical reference only)

- `3.1.21` qualified the destination synchronisation package (`1,923` tests, `xSAR 0.16.3`) with `3♦` and `8♣` ambiguities noted as pending (resolved in `3.2.0`).
- Historical `3.1.21` destination receipt: `Evidence/FreeCell/Validation/sync-321-20261113-_1zupweo/qualification-summary.json`.
