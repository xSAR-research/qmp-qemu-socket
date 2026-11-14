# Current handoff — application 3.3.2, xSAR 0.16.3

Application `3.3.2` builds on `3.3.1` with three approved updates while keeping `xSAR` untouched at `0.16.3` (commit `067ce26`):
- Restored `FREECELL_ROUTE_ABS_GESTURE_INTERVAL` (`src/parameters.rs`) from `100 ms` to `200 ms` across all five inter-event waits of six-event held ABS Free Cell drags (`SINGLE STEP` and `MULTI STEP`) so guest compositor frame stalls up to `163.9 ms` do not coalesce held motion into Tap-to-Move clicks.
- Displayed captured board images immediately on `START` during `Single board` / session `RECON` and `Planning` via an immediate `on_frame` hook in `freecell_reconnaissance::run_checked_with_deal` + `press_freecell_session` releasing prior completed `Manual` `FreeCellReconState` preview locks + `FreeCellWorkflow::retain_after_detach`.
- Consolidated live A* search counters (`Paused`, active time, expanded/generated/retained/peak frontier, best foundations `X/52`) and final determined `Report` statistics (`outcome_summary_message`, `Final`, `Paused`, `Frozen source`) across `Manual` Calculation and `Single board` / session modes via shared helpers in `src/worker/freecell_calculation.rs`, `src/app/freecell_calculation.rs`, `src/pyramid_workflow_model.rs`, `src/worker/freecell_session.rs`, and `src/app/pyramid_workflow.rs`.

## Tier 2 host verification summary (Beast)

| Gate | Command | Result |
| --- | --- | --- |
| Format | `cargo fmt --package qmp-qemu-socket -- --check` | Pass |
| Lib/bin strict Clippy | `cargo clippy --workspace --lib --bins -- -D warnings -D clippy::unwrap_used -D clippy::dbg_macro -D clippy::todo` | Pass |
| All-targets Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| Default tests | `cargo test --workspace --offline --locked` | `1,761` passed, `0` failed, `185` ignored (`src/lib.rs`; `0` in `src/main.rs`, `0` doc-tests) |
| Full tests | `cargo test --locked --offline --package qmp-qemu-socket -- --include-ignored` | `1,946` passed, `0` failed, `0` ignored (`src/lib.rs`; `0` in `src/main.rs`, `0` doc-tests) |
| Rustdoc | `RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline --package qmp-qemu-socket --no-deps --document-private-items` | Pass |
| Release build | `cargo build --locked --offline --release --package qmp-qemu-socket --target x86_64-unknown-linux-gnu` | Pass |

- **Release executable:** `target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket` (`16,943,152` bytes, mode `0755`, SHA-256 `cc1792528ccce6d32f2a7639776d5c8def615f1adefb578987c54f0723412cdc`)
- **Live acceptance status:** Live Windows 11 guest verification on Beast covering `Single board` live board preview from `START` through `RECON`/`ROUTE`/execution, matching live & determined `Manual`-style calculation statistics, `200 ms` held ABS drag reliability under `MULTI STEP` (including `8♦ PLAY 1 -> PLAY 4` challenge boards), `Continuous normal games`, Option 2A unsolvable-deal reporting, and default `GAME WIN` frame capture vs `--no-image` remains pending operator launch:

```sh
cargo run --release
```

Use `cargo run --release -- --no-image` to disable `GAME WIN` sequence recording while keeping failure diagnostics.

## Continuity check

External `--expected-closeout` Python sentinel scripts are retired from `3.2.0` onwards per operator direction (the prior `3.1.21` sentinel requirement was waived). Continuity relies on repository state in `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `docs/milestones.md`, `git status --short`, and locked Cargo checks.

<details>
<summary>Archival 3.1.21 and 3.1.20 handoff notes (historical reference only)</summary>

- `3.1.21` qualified the destination synchronisation package (`1,923` tests, `xSAR 0.16.3`) with `3♦` and `8♣` ambiguities noted as pending (resolved in `3.2.0`).
- Historical `3.1.21` destination receipt: `Evidence/FreeCell/Validation/sync-321-20261113-_1zupweo/qualification-summary.json`.

</details>
