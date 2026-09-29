# Pyramid calibration and guarded execution

## Source and environment

Issue [#1](https://github.com/xSAR-research/qmp-qemu-socket/issues/1) supplied the
1920×1080 guest captures used to establish Pyramid geometry. The operator
verified the target overlays in version 1.0.1. Issue
[#4](https://github.com/xSAR-research/qmp-qemu-socket/issues/4) enables guarded
Solver-guided input against that accepted geometry.

The guest display is 1920×1080 at 100% display and text scaling. Coordinates
are guest physical pixels from the top-left; rectangles are half-open
`(x, y, width, height)` regions. Preview scaling and host desktop coordinates
do not alter these positions.

The operator confirmed that one click on either highlighted card removes a
Solver pair, a highlighted King takes one click, and Pyramid uses the same
progress bar and post-game controls as TriPeaks. These are operational
confirmations, not a claim that the new candidate has passed live testing.

## Ordered targets

The fixed 31-slot array contains Move, Left, Right, then every card left to
right from row 7 upward to the row-1 apex. The first eligible 2×2 gold probe
wins. Multiple halos are expected and do not create a two-click batch.

| Target | Face/control bounds | Click | 2×2 halo probe origin |
|---|---|---|---|
| Move / Recycle | `(920, 678, 80, 80)` | `(960, 718)` | `(950, 753)` |
| Left | `(759, 678, 139, 187)` | `(828, 771)` | `(789, 871)` |
| Right | `(1022, 678, 139, 187)` | `(1091, 771)` | `(1052, 871)` |

Card face envelopes use width 140 and height 187. A card whose origin is
`(x, y)` has click point `(x + 70, y + 24)` and a 2×2 halo probe at
`(x + 40, y + 192)`. The click lies in the exposed top strip; the halo probe
lies below the face. Card appearance and removal checks are separate from the
halo test.

| Row | Face y | Face x positions, left to right |
|---|---:|---|
| 7 | 432 | 288, 489, 690, 890, 1091, 1291, 1492 |
| 6 | 378 | 389, 589, 790, 990, 1191, 1392 |
| 5 | 325 | 489, 690, 890, 1091, 1291 |
| 4 | 272 | 589, 790, 990, 1191 |
| 3 | 218 | 690, 890, 1091 |
| 2 | 165 | 790, 990 |
| 1 | 112 | 890 |

The uniform face envelope covers one-pixel rasterisation differences in the
source captures. The apex's lower edge was occluded in the supplied stills;
its full height is inferred from the shared card geometry. Static geometry
alone does not prove a card is currently uncovered or legal to click.

## Effect and board state

A normal action uses a validated frame, one QMP mouse click, the configured
settling delay (defaults: Move 1000 ms, card/pile 2000 ms), and a fresh result frame.
The verified result is reused for the next scan.
There is no extra screenshot immediately before each subsequent click.

A clicked card is marked consumed only when its removal is positively
verified. Halo disappearance, QMP acknowledgement, cursor motion, selection
and arbitrary pixel change cannot set that mark. The unclicked partner is
not marked. Move, Left and Right remain repeatable after a verified relevant
effect; the fresh-pair exception below also permits identical Left/Right pairs.
Additional observations are bounded and do not resend the click.
For Left/Right, a second effect path checks the planning frame for exactly one
other eligible highlighted card. If that partner is a tableau card which was
positively present and is now positively absent, it proves the pair removal
even when the clicked pile's inset pixels change by fewer than 128 pixels.
If the unique partner is the opposite pile, both piles' cursor-excluded
interior changes are summed and compared with the same 128-pixel minimum;
the partner's positive Present-to-Absent transition is also sufficient.
This changes the measured region to the known pair, allowing two similar
replacement faces to provide evidence together. It does not add unrelated
or unhighlighted pile changes to a card operation's verification.
The clicked pile must have a positive face and halo before input and known face
evidence afterward. An opposite-pile partner must also have known face evidence
afterward. An unknown or blocked partner, multiple possible partners,
halo disappearance alone, or a new target alone cannot provide this proof.
The unclicked partner is still not added to consumed-card history. Move and
tableau-card verification retain their existing rules. Effect logs report
before/after face evidence and cursor-excluded changes for both piles, the
identified partner, any qualifying combined count and any positively removed
partner, separately from the broader effect-pixel profile count. The other pile
is logged even when it is ineligible to verify the action. None of these paths
replays input or treats the progress bar alone as proof of a completed board.

Repeated Left/Right pairs may have identical ranks, suits and HALOs. If the
effect cannot be proven, the configured card settle and one fresh QMP capture
can still authorise the next operation when **both** planning and result frames
show the same unique eligible Left/Right pair, with positive faces and all four
HALO pixels on each pile. MOVE must not be highlighted, a tableau card must
remain visible, and no final-apex, board-completed or redeal phase may be pending.
No pixel-difference minimum applies to this continuation route. It uses a new
plan from the fresh frame, without changing consumed history or completion
counters. Step Once ends after its one click; Multi-Step may execute the new
plan. The logs and run counters distinguish `continued from fresh halo` from
effect-verified operations. This deliberately trusts the fresh HALOs and cannot
distinguish identical successor cards from a delivered click the game ignored;
STOP and the configured operation limit remain available. QMP delivery errors
still stop before this result path and are never replayed.

Each halo-free result waits the configured repeat interval (default 1000 ms)
before a new QMP capture. The
ordinary action path does not activate Solver to chase a late halo; only a
positively verified redeal with three settled halo-free observations allows
one Solver activation on that new board. If the effect cannot be verified or
the halo does not appear within the observation limit, the run stops and the
latest unverified capture appears for diagnosis, with no target authorised.

Params exposes Move/Recycle, Card/Left/Right and repeat-observation milliseconds
separately, bounded 0–5000. The worker snapshots these values per run and logs
them; edits take effect on the next run. Restore execution defaults resets them
to 1000/2000/1000. Settings are session-only and reset when the app restarts.
Double-click a number to enter milliseconds or drag it. Controls are locked
during a run/capture or snapshot dialog; STOP and close the dialog first.
The card/Left/Right delay applies after those clicks, including when the next
halo is MOVE. The defaults reflect Charlie's candidate 3 timing experiment;
they do not guarantee that every animation will finish before the first capture.
TriPeaks timings and the shared post-game stage waits are separate.

When only the highlighted apex remained before the click, two fresh frames
showing multiple restored bottom cards and `AnotherBoard` progress prove a
redeal even if the action effect was verified earlier. A temporarily full
progress probe causes input-free re-observation, not an immediate error. Its
black-pixel count is logged. Conflicting/missing evidence resets confirmation;
20 observations remain the bound for a recognised final-apex transition.
After a positively empty board, new cards with `AnotherBoard` also need two
consecutive captures. Stale transition halos cannot become new input after a
final-apex action. Per-board consumed state is cleared before rescanning a
confirmed redeal. Three subsequent settled halo-free new-board captures permit
at most one Solver recovery click. Persistent conflicts stop with diagnostic
pixels and no additional input. Game Win keeps the shared post-game controller.

A confirmed new board/game, explicit progress reset, or mode/socket change
clears consumed state. No halo alone does not mean the board is complete or
that Move should be clicked. Positive completion evidence is required before
the shared progress discriminator and transition controller are used.

If the transition was missed and an old consumed-card record suppresses every
eligible target, read-only Capture Frame may recover: a positively present
card with a fresh Solver halo at one of those previously verified removed slots
proves that the old record no longer fits the current board. The worker clears
the old record, resets the advisory board count to unknown, and reuses the
already captured frame. Absence of a halo, an unknown scene, or a lower-control
halo cannot trigger this recovery. The automatic transition path re-observes
before the first score-skip click and re-observes an ambiguous Level Up control.

## Shared controls and transitions

| Control | Bounds | Click |
|---|---|---|
| Solver | `(585, 959, 35, 36)` | `(602, 977)` |
| Undo All | `(1301, 959, 38, 38)` | `(1320, 978)` |
| Undo | `(1661, 960, 37, 36)` | `(1680, 978)` |

Undo All and Undo are shown as overlay references; this change does not add
an automated Undo All confirmation or a replay solver.

After verified completion, the shared progress probe at `(1050, 87, 38, 2)`
distinguishes the mostly black unfinished progress bar from a completed one.
At least 65 of its 76 pixels with all RGB channels at most 32 means another
board; otherwise the shared completion path is selected. The advisory board
counter alone cannot authorise this transition. Post-game processing uses the
existing guarded score-skip, Level Up OK, New Game, Play and Solver stages.

The native new-board capture from 2026-09-28 showed the previous y=84..85
probe on the gold border (0/76 black), while the corrected y=87..88 interior
was 76/76 black. `tests/fixtures/solver-progress-new-board.json` retains exact
RGB samples and the source PNG hash. A filled-bar regression uses gold pixels
from that same bar's filled left third; it is synthetic completion evidence.
Live Game Win and TriPeaks still require Beast regression checks.

## Evidence limits and Beast acceptance

Still screenshots establish positions and colour samples. They do not prove
input delivery, settling under VM load, or the complete action and transition
sequence. Source review, synthetic detector tests and offline image replay
are distinct from live QMP acceptance; the candidate review records which
checks actually ran.

Beast verification must cover a King, tableau pair, pile/tableau pair,
repeated pile targets, Move/recycle, all three boards and the shared post-game
sequence. It must also exercise STOP, mode/socket invalidation, explicit
progress reset and a TriPeaks regression run. Saved Capture PNG evidence must
remain free of preview overlays. Any unresolved or unknown scene must stop
with diagnostics rather than infer a click.

Capture still uses a temporary QMP PNG, RGBA decoding and normal best-effort
file removal. File-free automatic capture, rank/OCR reconstruction, independent
search and Undo All replay remain separate requirements.
