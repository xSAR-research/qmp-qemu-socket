# Verification — 2.9.143 mixed legend scene correction

Host qualification passed: **1432 full application tests**, zero failed/ignored/filtered, **726 sealed inputs**, all six required gates and native release/help. Fresh corrected guest behaviour and Charlie acceptance remain pending.

## Actual trigger and scope

The unchanged native Xbox frame under `tests/fixtures/pyramid-mixed-legend-143/score-entry.png` came from `/home/charlie/tmp/pyramid-error-1792854151812-1031088-1/last-observed.png`. Original checksummed evidence is under `/home/charlie/Documents/Pyramid/Validation/xbox-boards-entry-142-20261025`. Its SHA-256 is `c72972b5f77dec5194fa31bf4c60e70ed74bb0f29f697d91ebd78187e47f84e2`.

Score 775/2200 was positively read; the first blocker was the scene guard, before cards/piles. Six complete 68×2 strips had strong-white counts [0,0,0,136,136,0]. Four dim strips had maximum channel spread 13 and minimum per-channel lift 72–73 against both y−3/y+4 rows. Retained modal strips have lift 33–40: neutral ink and contrast 32 alone would admit those negatives. The cause of the partially dimmed UI state is unestablished. No Recon JSON was created because entry never authorised acquisition.

Only the application changes. xSAR remains 0.9.0; dependencies, APIs, concurrency, timing, capture budgets and Computer Vision/HALO retain their existing contracts.

## Bounded correction

`white_legend` retains the original all-six-white body. A native-v23 positive alternative requires exactly two adjacent completely white strips. Every other strip must have zero strong-white pixels, neutral ink with RGB spread at most 32, and per-channel lift at least 64 over both complete neighbouring rows. Every strip pixel is checked. The lift has at least eight native levels below the actual minimum and exceeds retained modal maxima. Partial bright strips, stray white pixels, nonadjacent groups, missing/coloured/low-contrast strips and fully dimmed frames refuse.

The new positive alternative supports scene, gameplay-veto and Undo shape preconditions. `dialog_matches` deliberately retains the original `white_legend` exclusion; broadening that inverse predicate could suppress a real modal. Specific original modal proof remains first, and its five-component fallback remains available for mixed scenes. Independent identities, exclusive stock, controls, legal state, fresh probes, STOP/context, acknowledgement/effect separation and no-replay rules remain.

## Tests and review

The actual-frame positive first failed against the retained 142 runtime. Four focused tests then passed: actual full board and Score/LEFT 3S/empty RIGHT; strip and independent scene negatives; a labelled synthetic mixed-legend/modal composition preserving UndoConfirmation and missing-component refusal; and production `observed_entry` over isolated AF_UNIX with two actual frames plus first-capture STOP/context revocation and no guest commands.

An initial session test used its helper's legacy-v17 profile and refused. The test now explicitly selects bundled-v23 through a separate test operation; runtime recognition was not widened to satisfy that fixture. All first failures are retained. The review is read-only source/fixture analysis; it does not establish live 143 success or arbitrary themes.

## Host qualification

Complete selected-nightly qualification passed: 1432 tests, zero failed/ignored/filtered, 726 unchanged sealed inputs. Owning-package Cargo gates use locked/offline mode, all features and explicit x86_64-unknown-linux-gnu; check/Clippy include all targets. Private rustdoc denies warnings/missing docs; Clippy denies warnings. Native release/help passed. Unchanged xSAR/paused-clock/UI evidence is retained by source hash, not rerun or presented as fresh 143 evidence.

| Gate | Result | Seconds |
| --- | --- | --- |
| scoped-format | exit 0 | 0.164 |
| check | exit 0 | 9.483 |
| rustdoc | exit 0 | 1.767 |
| clippy | exit 0 | 10.235 |
| full-tests | exit 0 | 444.799 |
| release | exit 0 | 14.750 |

Protected executable: `/home/charlie/Documents/Pyramid/Validation/pyramid-mixed-legend-20261025/native/qmp-qemu-socket-2.9.143`.

SHA-256: `74e05609a68c031404c1de46649fb7bbc4c9af2fd211390e330acd31cb5ea2b7`. The protected bundle retains logs, source snapshots, first failures, review and trigger provenance. This qualification snapshot preceded the launch recorded below.

## Remaining milestones

M01–M10 and Continuous — No challenge are accepted. M11 and M14 await fresh live acceptance; M12 pacing is in progress; M13 naming/API audit is planned and explicitly later than the approved solver goal. Current acceptance criteria are reconciled to temporal REDEAL; no status or dated evidence is promoted by this correction. See the [ordered master schedule](Pyramid-Implementation-Schedule.md), [matrix](Pyramid-Testing-Matrix.md) and [manual checks](Live-Test-2.9.143.md).

## Launch for Charlie

The protected qualified executable was launched at UTC 2026-10-24T15:40:59.906099+00:00 after confirming the old controller had exited. `/proc/1038788/exe` resolved to the protected 143 path and its SHA-256 matched the qualified artifact.

Process: `1038788`. SHA-256: `74e05609a68c031404c1de46649fb7bbc4c9af2fd211390e330acd31cb5ea2b7`. Session log: `/home/charlie/tmp/qmp-qemu-socket-1792856459954.log`. Exact first line: `[+0000.047s] qmp-qemu-socket v2.9.143 started; waiting for a solving strategy choice.`.

The agent did not press START or send guest input. Startup-cleanup evidence was checked; 0 image files required preservation. Fresh corrected gameplay and Charlie acceptance remain pending. The bundle `launch.json` records the launch; `candidate.json` remains the immutable prelaunch qualification snapshot.

## Code Analysis

False board recognition and suppressed dialogs are the relevant input-authority threats. Full strip/contrast proof, unchanged all-white modal exclusion and independent scene/card/control guards bound the alternative. Checked frame/rectangle/pixel access returns errors; no unsafe code, production panic, allocation-heavy search or new input action is introduced. Unsupported evidence stays unknown and stops without replay. Offline regressions do not replace fresh guest acceptance.
