# Verification — 2.9.169 — 30 October 2026

2.9.169 changes the starting Pyramid joint-change coverage from strictly more
than 10.0 percent to strictly more than **5.0 percent**. The RGB tolerance stays
13: a pixel counts only when all three absolute R, G and B differences at the
same coordinate are strictly greater than 13; alpha is ignored. On the existing
31 by 49 corner, 75 pixels refuse and 76 of 1,519 pixels pass. Live PARAMS still
publishes one complete pair, frozen for the current action or paired-D batch;
an in-flight edit takes effect at the next established boundary.

The reported Recon counts of 102 (6.71 percent, standard-theme 9D to 5D as
clarified by Charlie) and 131 (8.62 percent, Coral) clear this pixel gate under
the new default. Synthetic regressions check both counts against 5 and 10
percent. Immediate runtime pre-batch pixel pairs were not retained, so these
tests do not reproduce those guest transitions or prove complete Recon.
Expected identities, stock order, full inventory, restoration, quiet settling,
STOP/context revocation and no-replay rules remain in force. Fresh live
standard/Coral Recon acceptance remains pending.

All six controller gates pass, including 1,670 tests with the required
include-ignored scope, locked checks, strict private rustdoc, Clippy, scoped
formatting and release. xSAR is unchanged at the modified, unreleased 0.9.1
path dependency; its 16-gate receipt (303 full tests, 14 default tests, feature
checks and two focused Miri checks) is PASSED (Cached). User Cargo profiles
and CPU configuration are preserved. No install or guest launch was performed.

## Authority, baseline and scope

Charlie approved the 10-to-5-percent default change and exact post-168
lifecycle reconciliation. The 168 sentinel discrepancy remains historical.
baseline-state.json, its current-memory.sqlite copy and prior-destination-state.json
preserve the agreed inputs and metadata. HEAD is dcb92241aac491d917431b2ddcb851284f71898a; parent is 58aa2882fb0c4ff3aa0d71ae49de0871545b86e3.
No branch, index, commit, push, install, capture or guest input was changed.
The four-repository snapshot and old evidence hashes are checked at closeout.

Only the production default coverage constant/documentation changes. The RGB
predicate, 13-byte default, live-pair freeze, input cadence and quiet settling
are unchanged. The cfg(test) guided helper now explicitly uses its historical
13/10 fixture policy; production still receives the selected frozen pair.
No comparator, dependency, global setting, CPU config or sibling source changed.

## Host qualification

| Gate | Result | Elapsed |
| --- | --- | --- |
| scoped-format | PASS | 0.20 s |
| check | PASS | 3.95 s |
| rustdoc | PASS | 1.89 s |
| clippy | PASS | 12.87 s |
| full-tests | PASS | 166.17 s |
| release | PASS | 55.13 s |

The full suite reports 1,670 passed, zero failed/ignored/measured/filtered,
149.53 seconds of test time. Cargo uses nightly, explicit package, locked,
offline, x86_64-unknown-linux-gnu and all features; full tests include ignored
tests with eight test threads. Source input seals cover all 920 paired inputs.
After these passes, only the controller's two manifest/lock version entries
are aligned; its locked check and release pass again, with 918 other inputs
unchanged. Protected --help returns before GUI/image-cleanup startup.
All 56 library inputs and the 16 cached library gate/log hashes match.

## Regressions and evidence limits

Focused guided 31, temporal 14 and live 12 tests pass. Default boundary checks
prove 75 refuses and 76 passes. Explicit old 10.0-percent 151/152 and 10.1-percent
153/154 boundaries, and explicit frozen26 material/stock refusals, remain tested.
The reported 102 and 131 synthetic counts pass only the approved new pixel
allowance; that does not establish stock order, complete inventory, restoration
or guest acceptance. Charlie clarified the standard Recon transition as 9D to
5D, exposing very similar suit-coloured paper underneath. The immediate prior
stock pixels are absent from the packet; no new capture or replay was taken.
The standard packet/native diagnostics are privately retained with original
names. Existing 168 Coral whole-position tests are offline tableau-removal
evidence; the native paired-D adapter regression is synthetic. Complete fresh
Coral/standard Recon, other-theme live acceptance and normal Continuous remain
pending. No new timing/focus explanation is claimed.

## Protected delivery

Executable: /home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-coverage-169-20261030-9ppdzni6/native/qmp-qemu-socket-2.9.169
SHA256: 5e6c414442ff7876691e4e944e0c43f0c5f9b9377354db154fb89ca0fda8d884
Bytes: 13389696; mode 0700; not installed.
Exact gates, logs, baseline reconciliation, alignment and final snapshot: /home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-coverage-169-20261030-9ppdzni6.

## Complete owned paths

- `/home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock`
- `/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml`
- `/home/charlie/repo/RUST/qmp-qemu-socket/README.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Acceptance-Before-3.0.0.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Short-Term-Goal.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.9.169.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/development.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/handoff.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/milestones.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/pyramid-execution.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/src/parameters.rs`
- `/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid.rs`
- `/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_effect.rs`
- `/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/removal/tests.rs`

## Code Analysis

The narrower required changed area admits more measured appearances, including
102/131-pixel synthetic changes; it is an occurrence signal within the existing
semantic and authority gates. It cannot alone prove the right card/action or
full Recon completion. Safe checked sampling, RGB byte extraction, alpha
exclusion, overflow-free strict fractional arithmetic and bounded comparisons
are unchanged. Whole settings pairs remain frozen per action/batch; STOP,
context invalidation, source permits and no-replay rules are preserved.
