# Current handoff — application 3.3.0, xSAR 0.16.3

Application `3.3.0` builds on `3.2.1` by adding worker-owned **Continuous normal games** and **Single board** session automation for independent A* Free Cell (`src/worker/freecell_session.rs`, `WorkerCommand::StartFreeCellSession`), reusing parameterised `post_game::RestartPolicy::IndependentFreeCell` (`GAME WIN` score skip preserving default win-sequence recording unless `--no-image` is passed -> optional `LEVEL-UP!` `OK` -> `NEW GAME` -> `PLAY` -> `verify_independent_start` 2-capture fresh deal proof), implementing Option 2A clean unsolvable exit (`RunTermination::Completed` with zero mid-game clicks), removing all superseded relative-coordinate (`REL` / `DragMode`) drag code to standardise exclusively on checked integer ABS dragging, and adding the Free Cell `Mode` (`Continuous normal games` default, `Single board`, `Manual`) + `Transfer policy` controls in `src/app/pyramid_workflow.rs` while keeping `Manual` 3-step controls intact. `xSAR` is aligned at `0.16.3` (commit `067ce26`).

## Tier 2 host verification summary (Beast)

| Gate | Command | Result |
| --- | --- | --- |
| Format | `cargo fmt --package qmp-qemu-socket -- --check` | Pass |
| Lib/bin strict Clippy | `cargo clippy --workspace --lib --bins -- -D warnings -D clippy::unwrap_used -D clippy::dbg_macro -D clippy::todo` | Pass |
| All-targets Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| Default tests | `cargo test --workspace` | `1,757` passed, `0` failed, `185` ignored (`src/lib.rs`; `0` in `src/main.rs`, `0` doc-tests) |
| Full tests | `cargo test --locked --offline --package qmp-qemu-socket -- --include-ignored` | `1,942` passed, `0` failed, `0` ignored (`src/lib.rs`; `0` in `src/main.rs`, `0` doc-tests) |
| Rustdoc | `RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline --package qmp-qemu-socket --no-deps --document-private-items` | Pass |
| Release build | `cargo build --locked --offline --release --package qmp-qemu-socket --target x86_64-unknown-linux-gnu` | Pass |

- **Release executable:** `target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket` (`16,922,304` bytes, mode `0755`, SHA-256 `3e8727fa13abd0d98a1c6a9768a6cf4c2a8f06e7152104d3349b4d4f72b92e97`)
- **Live acceptance status:** Live Windows 11 guest verification on Beast covering `Continuous normal games` multi-board transitions (`GAME WIN` -> optional `LEVEL-UP!` `OK` -> `NEW GAME` -> `PLAY` -> fresh 52-card deal recon), default `GAME WIN` frame capture vs `--no-image`, `Single board` / `Manual` Free Cell modes, ABS-only card dragging, and Option 2A unsolvable-deal handling remains pending operator launch:

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
