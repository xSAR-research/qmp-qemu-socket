# TriPeaks redeal HALO geometry recovery — candidate.10

Candidate.10 preserves the calibrated detector and click coordinates. A
TriPeaks post-action frame rejected specifically for `ClickOffsetMismatch`
receives at most three delayed input-free recaptures. A fresh frame must pass
the unchanged geometry check before a target becomes actionable.

## Observed failures

| Session log | Transition | Rejected derived centre | Canonical centre | X difference |
| --- | --- | --- | --- | --- |
| `1792114992624` at 89.145 s | Board 1 to 2 | (379,533) | (366,533) | +13 px |
| `1792116638392` at 73.356 s | Board 2 to 3 | (365,533) | (366,533) | -1 px |

Both concern row 4, column 2, near the left edge of the tableau. The detector
had found an exact-colour HALO run. The tracker rejected the derived click
centre and returned a typed error; capture then stopped the action immediately.
This was not an exhausted X search or absence of every HALO match.

Restarting the controller without changing either game board resumed play.
Session `1792116927181` also reached the next board with a DRAW-first target
after the existing Solver recovery path. That comparison supports allowing a
bounded fresh observation of rejected transition geometry. It does not justify
shifting the canonical click or accepting an arbitrary offset.

The retained native fixtures in `tests/fixtures/tripeaks-redeal-20261016` show
the settled left-side 8 of clubs, left-side ace of clubs and DRAW-first board.
They are later manual captures. Tests label shifted in-memory derivatives as
controlled inputs; the original PNG bytes remain unchanged.

## Recovery contract

1. Capture and publish the native diagnostic frame through the existing route.
2. Classify that immutable frame with typed tracker errors intact.
3. Only a TriPeaks post-action `ClickOffsetMismatch` may use the geometry
   recovery budget. Initial planning, manual captures and other game modes
   retain their existing error behaviour.
4. Wait the configured TriPeaks reobserve interval, checking STOP, then request
   a new frame. The loop has no card, DRAW, Solver or terminal input method.
5. A passing fresh classification returns to ordinary post-action verification.
   Persistent mismatch stops after the original rejected frame and at most
   three additional captures. The budget belongs to the whole action, so an
   intervening missing-HALO frame cannot renew it.

Transport, decoding, invalid layout and unrelated detector failures stop
immediately. Rejected geometry never becomes a predicted action. No constant
pixel tolerance is added and the canonical-centre check remains exact. Existing
effect proof, independent board/game completion and fresh input probes remain
separate requirements after recovery.

The latest rejected frame is retained for diagnostic preview. The existing
`--image` boundary is unchanged: board-transition gameplay captures are not
written into the GAME WIN image directory. If the controller is already within
GAME WIN, normal recording retains its current scope.

## Verification and limits

Focused regressions exercise typed +13 px and -1 px rejection, fresh canonical
recovery, a persistent mismatch, the whole-action retry cap, STOP, unrelated
errors and DRAW-first handling. Source, pixel and delivery evidence are distinct
from executing Rust tests. The supplied host verifier runs the application
checks, all tests, strict documentation, Clippy and release build on Beast.

Live acceptance must cover both board transitions with a first recommendation
in the lower-left tableau and the complete GAME WIN sequence. The sampled
failures establish the error path; the cause of the transient pixel offset
remains an inference until further runtime evidence is collected.

## Code Analysis

The change adds a typed error route and a finite read-only retry budget. It
introduces no network endpoint, dependency, pixel threshold relaxation or
alternative click coordinate. Native frame layout and detector bounds retain
their existing checks. Recovery uses the newest frame and does not accumulate
a capture history. STOP and capture errors end recovery; an unresolved geometry
error stops with diagnostic pixels and no replay of the acknowledged action.
