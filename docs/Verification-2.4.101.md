# Native v7 repair 2.4.101 verification

Candidate checkpoint, 19 October 2026. Charlie approved the bounded v7 native
calibration repair. **Host verification, release build and preservation checks
passed**, with the documented pre-existing package-format failure retained.
The full suite passed 876 tests. No collector was running at the final process
check; the verified binary has not been launched as a GUI or live-accepted.

## Scope and version accounting

The candidate appends one native rank-2 variant and one spade variant to both
reader banks, with the original baseline, provenance and held-out regression
evidence retained. Existing variants, all ROIs/masks, caps 24/8, alignment 1,
colour/paper checks and scene/dialog signatures are unchanged. Recognition is
never inferred by elimination. Capture, input, STOP, controller policy and accepted
five-game behaviour remain outside this calibration-only change.

Verified application **2.4.101** is existing 2.2.100 plus the two already accepted
M01/M03 application minor credits, applied once, and **one** qualifying build
increment to 101. The successful full required test scope earns that increment;
the known package-format baseline limitation is reported separately below.
This does not reopen M01–M04 acceptance or allocate another milestone. Console
0.8.4 and xSAR 0.4.100 are unchanged. M05–M09 remain Awaiting acceptance;
M10/M11 remain Planned.

## Immutable refusal and crop evidence

| Artefact | Retained identity |
| --- | --- |
| New byte-original `native-v7/baseline.png` | SHA-256 `f00076f38a2906dc17d1dc5b7c54806b6285f363f936924e5c785785e9173edf` |
| v6 source profile | SHA-256 `2d28864f120b4fc47600721a78658fd1880e97473377a90929de8aebe4b775fd` |
| Rank-2 source | slot 7 / row 4, column 2; crop `(792,278,27,24)`; RGB SHA-256 `9250b21fe48cea0bb48b51673a7cbd122557f98a226e2f42202eded67a5cd1f7` |
| Spade source | slot 25 / row 7, column 5; crop `(1096,462,20,16)`; RGB SHA-256 `35415efed2a2501d217db7805338403d028c7d1ae10302ca8830c741f866a350` |

Source slots were already recognised by v6; they are distinct from the failures.
Held-out rank-2 slot 6 / row 4, column 1 measures v6 **28 > 24**, v7 **19**.
Held-out spade slot 23 / row 7, column 3 measures v6 **12 > 8**, v7 **7**.
These are retained diagnostic measurements, not new live acquisition results.
Per reader, totals change 34→35 rank variants and 36→37 suit variants; rank-2
3→4 of 8, spade 9→10 of 16. Profile schema stays 1.

The retained diagnostic `INCOMPLETE.json` has SHA-256
`bb231bb749bfbd60b3ebefa830efaf03e39401b3c9c4986ea84f125b1276966a`;
`provenance.json` has SHA-256
`e764a80d64a1370c404ee4369742dbd82a2450a1df0e9114bee7d2afe04b1b35`.
Its first error is:

```text
unknown rank/suit at tableau slot 6; accepted identities=0; ink<128; XOR rank<=24, suit<=8; alignment=1px; unique labels/paper/colour required
```

The first attempt recorded **one capture, zero attempted inputs and no attempted
output publication**. No Undo All/restoration was required for this no-input
refusal. The diagnostic's `restored=false` records that no restoration was
performed, not a failed restoration after input. Preserve the first failure rather
than replaying uncertain actions;
there were no such attempted actions in this run.

## Required verification and runtime boundaries

| Check/evidence | Current result |
| --- | --- |
| Exact profile identity | Verified profile: 806,442 bytes, SHA-256 `9517ce85382ec0c977e735c79871c1e2f8110b7265343dc1140ac45d506e3e08`. |
| Append-only structure and production regression checks | Passed: removing exactly the two appended variants restores the complete v6 profile. |
| Focused native-v7 regressions | 10 passed. |
| Full native frame classification and historical observations | Passed: all 28 new tableau cards and initial LEFT four spades/positive-empty RIGHT; 29 retained v6 visible cards and 104 historical labelled observations, each exercised in both banks. No complete new stock inventory is claimed. |
| Package-scoped formatting | Failed on pre-existing baseline differences; pristine HEAD and candidate produce identical normalised differences. No new package-format difference. |
| New test-module formatting | Standalone check passed. |
| Locked check | All targets/all features passed. |
| Full tests including ignored tests | 876 passed, 0 failed, 0 ignored; ignored socket tests included. |
| Strict private rustdoc | Passed with `-D warnings -D missing_docs`. |
| Clippy with warnings denied | All targets/all features passed with `-D warnings`. |
| Native release build | Passed; compiler-selected native release artefact. |
| Resulting executable version/hash | Cargo package 2.4.101; SHA-256 `ebd8d763b12eb2f4026441fb575378e50c7d809a80287d953fd0b1525810e17c`; `--help` passed without starting the GUI. |
| Source/dependency/evidence preservation | Passed: application inputs, xSAR tracked inputs, console compiler inputs and all three Git indices unchanged through verification; no unrelated lockfile resolution changed. |
| Active collector | None found at the final independent process scan. Verification recorded old PID 521945 absent both before and after; its exit time/cause are unknown. No collector was started by the checks. |
| Fresh v7 acquisition, complete stock survey and restoration | Waiting for an operator application launch and Charlie's fresh-board/form confirmation; existing images are now backed up. V7 profile compatibility itself requires no upgrade/restart. |
| M09 manual console/game trial | Waiting for Charlie; M05–M09 acceptance remains open. |

The completed local record is `pyramid-v7-validation-r5ua33pj/verification.json`,
status `passed-required-checks`; command results are in `commands.json` and the
named logs. Its SHA-256 is
`32cd389c917d88b0410d8c0901b5074c135d62aba6eafc104b6d1d6de6015562`.
Host classifier success cannot establish live input delivery,
complete stock inventory, original-board restoration or a game win.

Toolchain: `rustc 1.101.0-nightly (8006091eb 2026-10-17)` and
`cargo 1.101.0-nightly (3960fa69b 2026-10-12)`.

The built binary is
`target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`. Application 2.4.101 is
an uncommitted local candidate over the 2.2.100 baseline at HEAD `842646e`. No
install, deployment, GUI launch or guest input was performed by verification.
xSAR remains clean at `00986b8d621803fc98da85616444fc76c14638a9`, version 0.4.100.
The local **uncommitted console 0.8.4 candidate** sits over committed HEAD
`22f52d7ec5fc867b542dd7ca4c20fe2b9fbe2e84` (0.3.2); its compiler inputs remained
unchanged throughout this v7 repair and verification. Neither HEAD identifies
all uncommitted candidate bytes; the tested input hashes below identify this
application candidate. A relative path dependency is not a Git pin.

## Executed host commands

These are the completed audit commands from `commands.json`, run in the
application checkout with `CARGO_TERM_COLOR=never`. Absolute checkout paths
are rendered relative to that root below. Rustdoc additionally used
`RUSTDOCFLAGS="-D warnings -D missing_docs"`. These results do not request a rerun.

| Check | Executed command | Exit |
| --- | --- | --- |
| toolchain | `rustc +nightly -vV` | 0 |
| cargo | `cargo +nightly --version` | 0 |
| scoped-format | `rustfmt +nightly --edition 2024 --config-path rustfmt.toml --config skip_children=true --check src/pyramid_recon/vision/native_v7_tests.rs` | 0 |
| focused-v7 | `cargo +nightly test --locked --offline --target x86_64-unknown-linux-gnu --bin qmp-qemu-socket native_v7 -- --include-ignored` | 0 |
| check | `cargo +nightly check --locked --offline --target x86_64-unknown-linux-gnu --all-targets --all-features` | 0 |
| tests | `cargo +nightly test --locked --offline --target x86_64-unknown-linux-gnu --all-targets --all-features -- --include-ignored` | 0 |
| rustdoc | `cargo +nightly doc --locked --offline --target x86_64-unknown-linux-gnu --no-deps --document-private-items` | 0 |
| clippy | `cargo +nightly clippy --locked --offline --target x86_64-unknown-linux-gnu --all-targets --all-features -- -D warnings` | 0 |
| release | `cargo +nightly build --locked --offline --target x86_64-unknown-linux-gnu --release --message-format=json` | 0 |
| release-help | `target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket --help` | 0 |

The separate package-format preflight failed with exit 1 as described below;
its identical pristine-HEAD failure is retained rather than counted as a pass.

## Tested input identities

The complete application compiler/fixture snapshot is retained in the local
`verification.json`. Review-critical SHA-256 values from that snapshot are:

| Repository-relative input | SHA-256 |
| --- | --- |
| `Cargo.toml` | `89de20c4b9f130d30c490095767d5a1c9fd800de624a3cd6e2cf34615c80fcb0` |
| `Cargo.lock` | `c29be9006f48784d580bf41b4b0607cbd479606086975e9770c1c6953beac89c` |
| `rust-toolchain.toml` | `facb8e24288b93da5b8a0bb9a16fb0b67689ec0752d9b30d03165c30e1c87c8c` |
| `rustfmt.toml` | `f0d3774f75c7bebb65b7d4173cddf5c1ea57c5c272a5e12e873fcfdc012ac3b4` |
| `src/pyramid_recon/vision.rs` | `af60b1326d1159649788d25ab76ea0ce981ea142a01b44bca9dac27dd4ebc7bd` |
| `src/pyramid_recon/vision/native_v7_tests.rs` | `8e342810225584736ec298e496fc365ea6eaee1d0ae3fd15d3f74db586a1f2f7` |
| `tests/fixtures/pyramid-recon/native-v7/profile.json` | `9517ce85382ec0c977e735c79871c1e2f8110b7265343dc1140ac45d506e3e08` |
| `tests/fixtures/pyramid-recon/native-v7/baseline.png` | `f00076f38a2906dc17d1dc5b7c54806b6285f363f936924e5c785785e9173edf` |
| `tests/fixtures/pyramid-recon/native-v7/INCOMPLETE.json` | `bb231bb749bfbd60b3ebefa830efaf03e39401b3c9c4986ea84f125b1276966a` |
| `tests/fixtures/pyramid-recon/native-v7/provenance.json` | `e764a80d64a1370c404ee4369742dbd82a2450a1df0e9114bee7d2afe04b1b35` |

## Known pre-existing formatting limitation

The package `cargo +nightly fmt --package qmp-qemu-socket -- --check --color never`
preflight returned exit 1. The pristine HEAD source copy also returned exit 1
under the repository's formatter configuration. Normalising the source-root paths
proved their **entire diffs identical**, recorded as
`same_baseline_differences=true` in `format-results.json`.

The new native-v7 test module was formatted independently with exit 0. Its
standalone check passed. No legacy application or sibling file was
reformatted. The candidate has no new normalised package-format differences;
this does **not** make the full package-format check pass. Retained local run
identifier: `pyramid-v7-validation-r5ua33pj`, including `format-baseline.log`,
`format-package.log`, `format-new-module.log` and `format-results.json`.

The locked check, 10 focused v7 regressions, complete 876-test suite, strict
private docs, Clippy, release build and final preservation checks passed.
Unrelated mass formatting remains outside the repair.

## Prepared fresh run — not executed

Proposed deal ID: `pyramid-recon-20261019-v7-01`. Proposed unused output filename:
`pyramid-recon-20261019-v7-01.json` in the existing local temporary-output
directory. Its absence and the absence of a same-name sibling Evidence file
were checked while preparing the run; no export or fresh live acquisition has
occurred. Recheck non-existence when the existing form starts its guarded run.
No collector is currently active at this checkpoint. Existing GAME WIN images
were copied before preparing manual startup instructions: **355 files,
818,763,843 bytes**, retained locally under
`pyramid-v7-validation-r5ua33pj/retained-game-win-images`. The local
`game-win-backup.json` records every SHA-256; destination equals source and the
original source manifest is unchanged. The manifest SHA-256 is
`55e693b42e09ab56b2e31dc8fa9cd5d60aa1f7135c8b88832eada651da810035`.
This private backup is outside promotion.
Application startup removes the original image directory even without `--image`.
After launch, record the actual executable/process identity, selected v7 profile hash,
Charlie's confirmation of the same untouched supported board and the actual
export producer. Do not substitute the historical PID 521945 for fresh evidence.

## Review and proposed future promotion

Complete approved review scope, relative to this repository:

```text
Cargo.toml
Cargo.lock
README.md
CHANGELOG.md
docs/pyramid-recon-profile.md
docs/Pyramid-Implementation-Schedule.md
docs/Codex-Current-Status.md
docs/Pyramid-Evidence-Index.md
docs/Verification-2.4.101.md
src/pyramid_recon/vision.rs
src/pyramid_recon/vision/native_v7_tests.rs
tests/fixtures/pyramid-recon/native-v7/README.md
tests/fixtures/pyramid-recon/native-v7/profile.json
tests/fixtures/pyramid-recon/native-v7/baseline.png
tests/fixtures/pyramid-recon/native-v7/INCOMPLETE.json
tests/fixtures/pyramid-recon/native-v7/provenance.json
```

The fixture profile is the stable operator selection; no temporary runtime copy
is required. The compatible 2.2.100 collector loaded the selected profile afresh
for each reconnaissance request. V7 uses the same schema and unchanged caps;
profile selection itself therefore requires no installation or restart. However,
that process has exited and no collector is active at this checkpoint. An
operator launch, after the completed image preservation, is required because
the process is absent. No production Rust logic changed: the source addition
is test-only.
For a fresh run, record its actual process identity, selected v7 profile hash
and export producer. Do not label an export from 2.2.100 as produced by 2.4.101
or treat the built candidate as installed/live-accepted.

This review list authorises no staging,
commit, push, deployment, restart or guest input. Keep local AGENTS, private logs,
ODS files, unrelated changes and the existing handoff packet outside promotion.

Proposed Conventional Commit title:
`fix(pyramid): add retained native v7 glyph variants`

Proposed body: Extend the native profile with one rank-2 and one spade variant
from independently recognised source slots. Preserve the byte-original refusal,
crop provenance and held-out failures without widening geometry, masks or matching
caps. Apply previously accepted M01/M03 minor credits once with qualifying build
101 as verified 2.4.101, retaining the identical baseline formatter failure.
Keep fresh live acquisition/restoration,
M09 human acceptance and the built executable's identity separate. The compatible
external profile does not require candidate installation or a restart.

See [the profile contract](pyramid-recon-profile.md),
[fixture record](../tests/fixtures/pyramid-recon/native-v7/README.md),
[evidence index](Pyramid-Evidence-Index.md) and
[master schedule](Pyramid-Implementation-Schedule.md).
