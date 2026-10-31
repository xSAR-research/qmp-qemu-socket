# Verification - 2.9.165 Coral recognition

Charlie approved 165 in m01659 and confirms correct Route objective selection.
The 164 Destination Sentinel returned HANDOFF VERIFIED before edits; baseline
qualification is PASSED (Cached). Root owned edits; one independent read-only
review completed with NO REMAINING FINDINGS. No guest input/capture occurred.

## Qualified change

Request2 originally failed original 26 (7|6); Request 13 failed 21 (7|1), then whole-
position testing exposed 22 (7|2). Exact complete crops now make both original
pending-after states verify, including every other identity and both piles.
Three fixed 144 by 34 patterns compare every one of 14,688 RGB bytes. The third
Request13 crop SHA256 is bdd2d3dea7a691eb063dff9316010f9b82748fe84bfaee8227cf83951536dc45.
See [fixture provenance](../tests/fixtures/pyramid-recognition-165/README.md).
All old structural limits/material-role masks and old exact proofs are unchanged.
The new positives remain after Present refusal and before generic aged refusal.

## Actual host gates

| Gate | Result | Elapsed |
| --- | --- | --- |
| scoped-format | PASS | 0.09 s |
| check | PASS | 11.46 s |
| rustdoc | PASS | 2.32 s |
| clippy | PASS | 12.85 s |
| full-tests | PASS | 599.88 s |
| release | PASS | 16.38 s |

Full tests: 1,643 passed; 0 failed, ignored, measured or filtered. All ignored tests were
included with --include-ignored; 8 test threads. All Cargo commands use nightly,
--package qmp-qemu-socket --locked --offline --target x86_64-unknown-linux-gnu
--all-features. Rustdoc uses --document-private-items --no-deps with -D warnings
-D missing_docs; Clippy covers all targets with -D warnings. Native release uses
Cargo JSON. 888 inputs sealed; 52 unchanged xSAR inputs retain cached 296 tests / 3 Miri.
Ten new focused regressions and 34 retained 162/163/164 regressions pass.

Both original red failures and the next-slot red failure are retained. The first
focused attempt had 9 pass / 1 fail; its full-position test correctly found 7|2.
No test expectation was weakened; the complete missing fixed footprint was added.
Only own Cargo.toml/Cargo.lock then changed 164 to 165; other 886 inputs identical.
Aligned locked check, native release and protected help pass. Help returns before
image cleanup or GUI creation (src/main.rs52-60). All4 histories,indices,siblings,
unrelated edits and old receipts are preserved; closeout-check.json has details.

## Protected executable and evidence

Executable: `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-coral-165-20261029-zoj98pfk/native/qmp-qemu-socket-2.9.165`. SHA256 `c5fa1292a84ec4b7fca89c3b0cece056c234b6c4fa2e1a4c7abd7665e8eccde3`. Mode0700, not installed.
Private case: `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-coral-165-20261029-zoj98pfk`. qualification.json, aligned-inputs.json,
focused-results.json, complete gate logs, original files, owned-paths.txt and
closeout-check.json retain reproducible host evidence. No staging/commit/push.

Charlie confirms Route objective selection; Request 13's actual log selected
Maximum / Weighted A* 2 for Card 0/12. The failure form's differing Exact A* field
is a separate diagnostic issue. Policy code is unchanged from 164.
Fresh Coral live acceptance, rendering cause and future appearance coverage are
unproven. These three patterns are calibration data; generic card material is
not thereby classified as background. The Continuous retraction and wider
[acceptance checklist](Pyramid-Acceptance-Before-3.0.0.md) remain.

## Code Analysis

No new unsafe code, dependencies, allocation, concurrency or input path changes.
Fixed rectangles and checked frame access bound every RGB traversal; fixed-size
arrays and checked reference indexing prevent truncated proof. Malformed frames,
wrong scenes, other locations, altered channels, surviving/partial card material
and wrong identities/piles retain refusal tests. Conservative exact calibration
can still refuse unseen artwork. STOP/context/target/no-replay rules are unchanged.

## Complete owned paths

These 23 paths are owned relative to the 164 checkpoint, without staging authority:

- `Cargo.toml`
- `Cargo.lock`
- `src/pyramid_recon/vision/native_colour.rs`
- `src/pyramid_recon/vision/foreground_card/tests.rs`
- `src/pyramid_recon/vision/foreground_card/tests/effect_165.rs`
- `README.md`
- `CHANGELOG.md`
- `docs/milestones.md`
- `docs/handoff.md`
- `docs/Verification-2.9.165.md`
- `docs/Pyramid-Short-Term-Goal.md`
- `docs/Pyramid-Acceptance-Before-3.0.0.md`
- `tests/fixtures/pyramid-recognition-165/README.md`
- `tests/fixtures/pyramid-recognition-165/calibration.json`
- `tests/fixtures/pyramid-recognition-165/request13/clearance-seven-two.rgb`
- `tests/fixtures/pyramid-recognition-165/request13/clearance.rgb`
- `tests/fixtures/pyramid-recognition-165/request13/deal.json`
- `tests/fixtures/pyramid-recognition-165/request13/first-unmatched.png`
- `tests/fixtures/pyramid-recognition-165/request13/last-observed.png`
- `tests/fixtures/pyramid-recognition-165/request2/clearance.rgb`
- `tests/fixtures/pyramid-recognition-165/request2/deal.json`
- `tests/fixtures/pyramid-recognition-165/request2/first-unmatched.png`
- `tests/fixtures/pyramid-recognition-165/request2/last-observed.png`
