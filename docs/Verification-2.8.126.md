# Verification — app 2.8.126

App **2.8.126 / native-v22** passed **1318 tests, 0 failed, 0 ignored, 0 filtered** in the complete native include-ignored scope (360.435s command time). Locked all-target/all-feature check, strict private rustdoc, Clippy with warnings denied, native release, scoped formatting of 19 Rust files and native/protected headless help passed. xSAR **0.8.104** and console **0.8.4** are unchanged: 18 shared/feature/private-consumer gates are retained by exact input/log hashes (289 shared tests, 80 console tests; feature rows 14/55/25/67/57/99), not rerun.

## Code Analysis

The active Card reader identifies the complete top-centre **cleared** word and
the actual `current/target` counter. `ChallengeKind::Cards` contains no card rank.
All thirteen possible rank names use this one path; no per-rank runtime label
bank is required. Boards and Score retain their nine original literal labels,
their separate units and the unchanged numeric-reader policy.

The shared word uses the full 84×32 measured blue-minus-6 intensity template,
exact L1/max ratio ≤210000 ppm, one strong-white core in each of seven letters,
two blank columns at both boundaries, and unchanged tolerance-9 native banner
borders. Search is horizontal within the existing 224×32 objective band, without
resizing or rotating. A separate measured Boards word is a veto only; it grants
no Boards authority and tolerates one damaged core to prevent a corrupted whole
Boards label from becoming a Card challenge. No card-glyph/profile, scene,
modal, empty-position, final-effect, numeric tolerance or timing was relaxed.

Counter kind, target, monotonic progress and session context still bind the
lifetime. An unmet target needs a positively verified board effect and witnessed
fresh board before automatic new Recon/restoration/planning/execution. A numeric
target alone cannot finish the challenge. Independent real completion can stop
remaining actions at the existing guarded action boundary; Star Club Continue
remains operator-owned. Manual, Single board, normal Continuous/restart/mid-game
entry, both route objectives and STOP/context/no-uncertain-retry guards remain.

Native Sixes 0/12 and Nines 0/15 now pass the actual paired worker entry adapter.
Synthetic prefix and missing-letter/boundary/banner/counter cases are explicitly
host regressions. They do not establish fresh guest acceptance of 126.

The model migration retains wrong-requirement tests as wrong-kind, wrong-target
and regressed-counter negatives. Native terminal references remain exact originals
with Card units, without identifying their rank. The original nine Boards/Score
label patches and the complete numeric reader remain byte-for-byte unchanged
apart from the visibility of three reused numeric-reader constants. The native-v22
card profile and all previously retained fixtures are unchanged.

## Actual qualification

| Gate | Retained log | Exit | Seconds |
| --- | --- | --- | --- |
| app-check | `/home/charlie/Documents/Pyramid/Validation/card-cleared-20261022/runs/0001/005-app-check.log` | 0 | 2.029 |
| app-rustdoc | `/home/charlie/Documents/Pyramid/Validation/card-cleared-20261022/runs/0001/006-app-rustdoc.log` | 0 | 1.492 |
| app-clippy | `/home/charlie/Documents/Pyramid/Validation/card-cleared-20261022/runs/0001/007-app-clippy.log` | 0 | 2.421 |
| app-tests | `/home/charlie/Documents/Pyramid/Validation/card-cleared-20261022/runs/0001/008-app-tests.log` | 0 | 360.435 |
| app-release | `/home/charlie/Documents/Pyramid/Validation/card-cleared-20261022/runs/0001/009-app-release.log` | 0 | 15.834 |
| scoped-format | `/home/charlie/Documents/Pyramid/Validation/card-cleared-20261022/runs/0001/010-scoped-format.log` | 0 | 0.326 |

Actual argv, UTC, log SHA-256 and compiler inputs are sealed in
`/home/charlie/Documents/Pyramid/Validation/card-cleared-20261022/qualification.json`. There is no new shared/console run: the 18
retained logs and shared inputs were independently rehashed before reuse.
The first narrow compile found dangling label-array delimiters from the migration;
they were corrected before tests. An intermediate check found two unused imports,
since removed. Focused new cases passed 9/0, and progress-reader cases 54/0. The
first complete run finished 1317 passed / 1 failed / 0 ignored / 0 filtered. Its
retained native-v22 JD regression still expected the Sixes header to be unknown;
the actual new result was Card 0/12. That single expectation was corrected to the
new required generic Card result while retaining every original direct-card/JD
assertion. The failed full log and its exact compiler inputs remain sealed in
the same command record. The corrected full run above is the required passing
scope; no recognition policy or threshold was changed to resolve the assertion.
Neither filtered run is represented as the required full test pass.

## Protected executable and preservation

Protected executable: `/home/charlie/Documents/Pyramid/Validation/card-cleared-20261022/release/qmp-qemu-socket-2.8.126`

SHA-256: `b4a4ffb7a70a106ac9d74d46aa035589d008400e5a4f569a8342de6b458e95cf`; 13685152 bytes; mode 0555. Byte-identical to the Cargo-selected native release. The dated process receipt is an observation, not proof of a later running identity.

**The selected generic Card goal is blocked pending fresh Beast verification
and remains incomplete.** Fresh 126 Beast
verification starts with the available Sixes challenge. It must establish real
unmet-target board continuation and verified target stopping, with preserved
modes/guards checked at their appropriate evidence layer. The old missing-125-
trial blocker is historical: actual 125 Single-board Sixes acquisitions and
planning were observed, but did not prove Continuous completion. Its recorded
Continuous refusal used the normal-repeat stop choice. Select **Stop at challenge
target** for this goal. M14 DRAW/REDEAL, variable backgrounds and idle CPU stay
outside this goal. M01–M09 remain accepted; M10 Awaiting verification, M11 Planned,
M12 In progress and M13/M14 Planned. No milestone acceptance is inferred here.

`/home/charlie/Documents/Pyramid/Validation/card-cleared-20261022/baseline.json` pins all three repositories' original HEAD, branch,
index and file hashes/modes/symlinks. The explicit review scope is
`/home/charlie/Documents/Pyramid/Validation/card-cleared-20261022/review-paths.json`. The final audit verifies unchanged siblings,
unrelated dirty work, all original fixtures and protected releases, literal
Boards/Score patches and numeric policy, new native crop provenance, manifest/
lock version-only changes and every local link. It also verifies all sealed
compiler inputs and command logs. No guest test or process replacement is inferred
from host success. [Live brief](Live-Test-2.8.126.md).
