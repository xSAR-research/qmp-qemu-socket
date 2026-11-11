# Verification — application 3.1.9 / xSAR 0.14.1

Status: host qualification complete; operator live acceptance pending.

Approved [goal](FREE_CELL_goal_20261110A.md) and [contract](FreeCell-Auto-Followups-3.1.9.md). The prior 3.1.8 destination sentinel returned `HANDOFF VERIFIED` before edits using the independently supplied closeout SHA256 `bc151524847ccd650651c012c41ea71587c426bedbf5dffc246adc7d8bf9bf5e`. All 1,033 prior compiler/configuration/test inputs matched; source/document preimages and all four repository baselines were preserved.

## Fresh application qualification

| Gate | Result | Seconds |
| --- | --- | ---: |
| check | PASS | 12.555 |
| clippy | PASS | 13.367 |
| format | PASS | 0.772 |
| release | PASS | 60.747 |
| rustdoc | PASS | 2.415 |
| tests | PASS | 104.006 |

The full test scope passed **1,871 tests, zero failed/ignored/measured/filtered**. The focused `freecell_319 -- --include-ignored` retry passed all ten tests, including two real queued-worker local TCPv3/Unix-QMP regressions. Independent source and document reviews found no production blockers.

The exact argv, environment, full diagnostics and tested-content bindings are recorded in `Evidence/FreeCell/Validation/freecell-auto-followups-319-20261110/`. Every fresh gate used the same 1,033-input seal `42ca3d5829dbd455558cd67ee52e8e57dc2aceb420d8b9a5c935d194a13a2e5a` and checked source/configuration/toolchain before and after. Commands used Cargo nightly with `--locked --offline --package qmp-qemu-socket --all-features --target x86_64-unknown-linux-gnu`; full tests included ignored cases, private rustdoc denied warnings and Clippy used all targets with `-D warnings`. Only the six owning Rust files were formatted.

Toolchain: rustc 1.102.0-nightly `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`, Cargo 1.102.0-nightly `c07ac2c4d12237effc4b94f4543769b421790045`; edition 2024/MSRV 1.101.0 and existing native target/CPU/hardening settings preserved.

## Retained and fresh evidence

xSAR 0.14.1 and console 0.8.5 sources, manifests, locks, feature declarations and tracked diffs are unchanged. Their exact 93 compiler/test/configuration inputs and toolchain match prior qualification: **347 library and 80 console tests PASSED (Cached)**, with the previous feature/default matrix receipts retained. No fresh sibling suite or version is claimed. The fresh application release independently selects xSAR 0.14.1 with the same consumer feature/profile metadata. Its consumer-feature rlib is distinct from the standalone all-features library artifact.

The first focused attempt compiled and passed eight core regressions. Two new native fixtures failed prefix replay at step 3 before transport because moving 4S exposed AH without a required recorded AUTO. The fixture third move now uses 5D and independently proves closed replay; recognition/replay/production checks were unchanged. The first log is retained. A closeout-helper assertion initially compared consumer-feature and standalone all-features rlibs; the corrected helper compares their proper compilation contexts and retains the failed assertion record. All six qualification gates passed their first attempts.

## Executable and preservation

Actual Cargo JSON executable: `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, version 3.1.9, 16822792 bytes, mode 0755. SHA256 `9e9637e275578e18b4d2de44f8dcf2ded3093939eeaac05eab3b2cd1265a54da`. This is a mutable Cargo output; no protected installation was made. `artifact.json` and `paired-artifacts.json` bind the delivered app and its selected xSAR dependency.

All four Git HEADs, index bytes and staged diffs are unchanged. xSAR, console and capture-server tracked diffs are also unchanged. Only the owning application version/lock entry changed from 3.1.8 to 3.1.9; no dependency/API/schema change. Earlier evidence capsules remain untouched. The scoped pattern scans found zero credential/history matches; mandatory case-insensitive Charlie references and personal-path findings are listed privately for promotion review. No scanner was installed.

No agent application launch, guest input/capture, installation or Git promotion occurred. At final process inspection there were no running qmp-qemu-socket processes.

## Live acceptance and destination sentinel

Host/local-peer tests do not prove fresh Windows timing or the next column transfer. Restore the original deal or begin a fresh supported initial board, RECON/SOLVE, select **Observe recorded AUTO follow-ups** and ABS, then Single Step. Confirm one explicit AC drag plus the observed 2C follow-up stops at the verified combined cursor; the next press performs the next planned transfer. Never resume the retired changed-board 3.1.8 run.

The new closeout pins source, owned documents, immutable receipts and Git fields. Local tooling and mutable Cargo artifacts are excluded from default coverage. Its verifier requires an independently supplied closeout digest; `--check-artifacts --check-toolchain` are required for current cached-build claims. The closeout digest is delivered separately and must not be obtained as the expected value from the receipt under verification.

## Code Analysis

The change affects private sequencing and application policy copy. Candidate positions are exact recorded prefixes, each proof is bounded to six fresh observations, and two identical final combined witnesses are required after an explicit action. Initial identity, fresh parent, VM/pointer, STOP/context/source ownership, terminal/partial retirement and no-replay rules remain. A late AUTO in the fresh parent refuses before input. There is no new unsafe code, dependency, public library API or transport gesture. Missing or uncertain effects stop rather than manufacture progress.
