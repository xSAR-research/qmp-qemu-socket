# Verification — app 2.8.119 / xSAR 0.8.104

The scoped Continuous/challenge implementation is host qualified: app **1260**, xSAR **289**, private console **80**; zero failed, ignored or filtered.
The six overlapping shared feature configurations passed **14/55/25/67/57/99**.
Locked native checks, strict private rustdoc, warnings-denied Clippy, both release
builds, 35 explicitly scoped Rust formatting checks and native/protected help passed.
Fresh Beast gameplay acceptance remains pending. M01-M09 remain accepted,
M10 is Awaiting verification and M11 is Planned.

## Delivered behaviour and limits

Route objectives are **Shortest route** (minimum winning MATCH/KING cost under
successful exact search) and **Maximum card matches** (seek a win first, otherwise
beneficial tableau removal with exhaustive-versus-bounded proof). MATCH and KING
cost one, including stock-only matches; MOVE/DRAW and RECYCLE cost zero in the
match objective. Total logical actions, draws and recycles are reported separately.
Stock-only removals do not increase the tableau-removal score. Maximum card matches
keeps the established logical-action cost for its win-first search and equal-score
fallback tie-break; it makes no minimum-card-match claim. Existing xSAR Pyramid
entry points retain their original unit-action policy.

Continuous has a separate stop choice: **Repeat normal games** retains actual
three-board progression, shared GAME WIN and Solver-free NewGame/fresh Recon;
**Stop at challenge target** binds actual Score, Boards or measured card-rank/count
requirements, retains monotonic progress across execution and board transitions,
and recognises completion at whole-action boundaries. The remaining route retires
only after that whole action is acknowledged. As Charlie clarified, XP skip,
optional Level Up OK, timing and mechanics reuse the existing shared GAME WIN
controller. Challenge mode stops on freshly recognised NewGame without clicking
NewGame, Play or Solver.

Native active rank-label evidence currently covers **Twos, Fours and Sevens**;
other active rank labels refuse rather than being synthesised. A common measured
Challenge Complete event may be bound to a previously qualified objective without
reading its final numeric header. The receipt/report retains the last actual count
and explicitly states when the final count was not read. A cold generic terminal
does not invent a requirement. Known contradictory literal terminal headers refuse.
A partial route requires exact independent replay and native endpoint verification,
then stops. DRAW-until-REDEAL continuation is a later milestone, not implemented.

## Native refusal correction and retained evidence

The actual protected-118 Grandmaster failure had a valid normal board-one header.
The mandatory New-footer reference failed at three pixels: maximum channel
difference 11 against tolerance 10. A second complete measured footer appearance
was retained; footer tolerance remains 10 and header tolerance remains 12.
No pixels were masked or generated. The native-v17 card/final-clear profile and
all older calibration bytes remain unchanged.

The [permanent objective corpus](../tests/fixtures/pyramid-objectives/README.md)
contains 12 unchanged native originals, complete RGB patches, 74 isolated native
counter glyph observations and three provenance manifests:
[originals](../tests/fixtures/pyramid-objectives/provenance.json),
[counter sources/crops](../tests/fixtures/pyramid-objectives/counter-provenance.json)
and [transition/terminal sources](../tests/fixtures/pyramid-objectives/transition-provenance.json).
The original Screenshots and temporary diagnostic directories are cleanable;
the required retained copies no longer depend on them. Existing permanent source
fixtures are referenced without duplication. These are exposed finite calibration
and regression data, not general recognition accuracy or fresh guest acceptance.

Actual Fours evidence includes automatic next-board dealing at 4/12 and 8/12,
and Challenge Complete at action 43 of a planned 46-action route. Boards 1/2 with
Perfect is retained as negative challenge-success evidence. Tests also cover
unknown/changed/rank-mismatched/regressed objectives, execution high-water
regression, STOP precedence, two fresh terminal observations, selection-phase
refusal, final/partial dispatch, shared XP/optional-OK handling and no NewGame.
Synthetic temporal repeats/header erasures/menu derivatives are labelled tests;
they do not add native or live observations.

## Required qualification

The full command, environment, log SHA-256 and compiler-input maps are in
/home/charlie/Documents/Pyramid/Validation/continuous-challenges-20261021/qualification.json and runs/0001/commands.json.

| Scope | Result |
| --- | --- |
| Application full all-feature tests, including ignored | 1260 passed; 0 failed/ignored/filtered |
| xSAR full all-feature tests, including ignored | 289 passed; 0 failed/ignored/filtered |
| Private unchanged-source console compatibility | 80 passed; 0 failed/ignored/filtered |
| Shared default/search/execution/combined/rules/search matrix | 14/55/25/67/57/99; overlapping rows |
| Four neutral dependency trees | std-only |
| Both locked native checks / strict private docs / Clippy / release | Passed |
| Explicit Rust formatting | 35 files; passed |
| Native and protected executable help | Passed; no GUI or guest connection |

All Cargo commands force native x86_64-unknown-linux-gnu, locked offline resolution
and the selected nightly toolchain. Full tests use --include-ignored and eight
test threads. Private rustdoc denies warnings and missing docs; Clippy denies
warnings. The actual toolchain and per-command durations are in the receipt.
Private console source/manifest are unchanged. Only its private xSAR lock entry
is changed to 0.8.104; the real console 0.8.4 and historical lock remain frozen.

The first complete application attempt is retained: **1,257 passed / 3 failed /
0 ignored / 0 filtered**, 343.206 seconds for the command (329.17 test report).
The obsolete assertions expected the old Continuous wording, old search labels
and a 49-action route under the previous all-action metric. The match-cost route
had 66 total logical actions; its legality/clear endpoint is independently replayed.
The next full application attempt retained **1,259 passed / 1 failed**, zero
ignored/filtered. Its remaining test exposed that using free DRAW/RECYCLE costs
for the default weighted policy reached two million histories with only 21/28
tableau removals on a known winnable deal (50 actions, 16 matches, 33 draws,
one recycle). A diagnostic focused rerun retained that exact report. Maximum card
matches now keeps the established logical-action policy for its win-first search
and fallback tie-break; Shortest route alone uses the match-cost policy. The same
real weighted planning/independent-clear regression then passed in 0.83 test
seconds, without changing search bounds or weakening its winning-route assertion.
The corrected complete application scope passed; unchanged successful shared
workloads were sealed and retained rather than repeated. Earlier focused
compile/lint/assertion failures remain recorded in the session/checkpoints.
No recognition or input guard was relaxed to make tests pass.

The first command receipts accidentally labelled post-command timestamps as
started_utc. The untouched commands-before-timestamp-correction.json is retained;
those values are now explicitly recorded_after_command_utc. Resumed commands
record real start and finish separately. Command durations are unchanged.

## Protected executable

- Path: /home/charlie/Documents/Pyramid/Validation/continuous-challenges-20261021/release/qmp-qemu-socket-2.8.119
- Size: **13,412,104 bytes**; mode **0555**.
- SHA-256: **72637b8248ae83ca43617a982c688898c7c92cd2086e5432eedcd77b29471df5**.
- Native Cargo artifact bytes are identical.
- protected-artifact.json records artifact selection, help outputs, read-only
  running-process identity and preservation checks. A build does not replace a
  running process.

## Preservation and review scope

The final documentation/preservation audit passed: **11 documents, 291 local
links and 86 review paths**; tested input maps and native asset hashes match.
Its record is /home/charlie/Documents/Pyramid/Validation/continuous-challenges-20261021/final-audit.json.
Scoped git diff --check passed in both owning repositories. This documentation
update does not change compiled inputs or repeat successful Cargo workloads.

All three baseline HEADs/branches/indices, unrelated dirty bytes, 328 pre-existing
app fixture files, nine shared fixture files, frozen console files and protected
107-118 releases are preserved. The authoritative pre-edit baseline is
/home/charlie/Documents/Pyramid/Validation/continuous-challenges-20261021/baseline.json. The literal review scope has **86 paths**:
76 app, 10 shared and zero console; see review-paths.json. It includes existing
uncommitted work and new files; base commits alone do not contain this delivery.
No dependency, unsafe block, install, guest trial, controller replacement, Git
promotion, publication, upload, thread transfer or archival was performed.

The original attached goal remains open until fresh Beast acceptance. Follow the
[live brief](Live-Test-2.8.119.md) and [current handoff](Game-Win-Handoff.md).
