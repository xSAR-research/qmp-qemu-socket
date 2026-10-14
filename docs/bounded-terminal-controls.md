# Bounded terminal controls and search metrics

Candidate: qmp-qemu-socket 2.2.2-candidate.3, based on the live-confirmed Spider
checkpoint `7c6c6fb3e4fecc866067045a3f36a9649bdf333d`. It uses the existing xsar
helper from `5fa8a1a963a7d0591458a348cabd5c28099970bd` without changing that crate.

## Search contract

All five game types use one application adapter for expected OK, New Game and
Play controls. It calls `xsar::image_matching::find_black_run_bottom_up` with
a run length of five. Pixels must have RGB exactly `(0,0,0)`; alpha is ignored.
Rows run bottom-to-top and columns left-to-right. The first qualifying run's
leftmost full-frame coordinate is the click point. A miss returns `(0,0)`.

The crop is a bounded view over the existing image buffer, not a new allocation.
Native 1920×1080 geometry and valid storage are required before guest input.

| Game | Control | Inclusive X | Inclusive Y | Maximum comparisons |
| --- | --- | --- | --- | ---: |
| All five | OK | 930–954 | 700–850 | 3,775 |
| All five | New Game | 810–878 | 831–879 | 3,381 |
| Free Cell | Play | 675–694 | 740–790 | 1,020 |
| Spider / Klondike | Play | 675–694 | 840–890 | 1,020 |
| TriPeaks / Pyramid | Play | 675–694 | 885–940 | 1,120 |

The application configuration lives in `src/terminal_controls/profiles.rs`.
Each game has a named `GameControlProfile` constant containing `ok`, `new_game`
and `play` search specifications. Each specification supplies its rectangle and
run length. `for_game` selects the block once; the controller passes it through
searches, retries and diagnostics. The Params panel reads the same block.
Change a game's layout here rather than modifying the scanner or click logic.

OK has the wider vertical allowance for Level Up movement. New Game and Play
use tighter windows around their static positions within each game. The OK
stripe covers the O and excludes the pointer retained at `(960,834)` in
Klondike K53. Play uses lettering to the left of New Game's caption; New Game
uses its right-hand lettering. Movement outside a configured window requires
a profile adjustment. TriPeaks/Pyramid Play bounds are provisional, derived
from the previous static geometry pending native screenshots or live evidence.

The reusable xsar helper remains unchanged and receives only a frame view,
rectangle and minimum run length. It contains no game types, button identities,
application coordinates, retry policy or logging. The application adapter also
receives a numerical `SearchSpec`, while the controller owns game selection.

## Context and retries

The caller still establishes the expected game/menu stage. Five black pixels
do not independently declare a game win or identify arbitrary words. Existing
Congratulations, score, completed-board and fresh-game evidence retain their
separate roles. The selected point travels with the stage to the input boundary;
there is no subsequent replacement with a fixed button coordinate.

An absent expected control receives one fresh capture and repeat search. A
second miss stops with the latest captured image available as a diagnostic
preview. Optional OK/New Game branches may examine both rectangles on the same
frame. The shared TriPeaks/Pyramid controller retires an earlier menu stage after its
click is delivered; black runs in that earlier region cannot authorise another
click or reset the current missing-control allowance. New Game takes precedence
at the optional Level Up step unless independent TriPeaks foreground-modal
context proves that OK must be handled first. Recognised reward/gameplay
recovery remains distinct from missing-control observations. Uncertain QMP input is not
replayed by the search retry.

## Metrics

Every expected-control lookup logs its attempts and a final summary through
the normal worker event sink, reaching the session file and on-screen log.
Pure independent screen-classification probes are separate from these lookup
metrics.

| Field | Meaning |
| --- | --- |
| `searches` | Number of scanner calls in this control lookup, including an invalid-frame attempt |
| `recaptures` | Additional capture attempts made for this lookup; includes a failed capture attempt |
| `second_grab` | Whether at least one additional capture was attempted |
| `pixels`, `rows` | Work actually visited before early exit, summed in the summary |
| `scan_ms` | Monotonic elapsed time inside this attempt's scanner call |
| `scan_total_ms` | Sum of completed scan durations |
| `wall_ms` | Monotonic elapsed time from lookup construction to its final result/exit |
| `outcome` | Found, missing, error, cancellation or a recognised transition, as applicable |

Wall time includes retry waits and capture work performed during the lookup.
When the first frame is already available, its earlier acquisition is outside
that interval. The shared controller can create a lookup before acquiring its
initial frame; that acquisition is then included. Normal selected-control clicks
and subsequent settling occur after the lookup summary. Existing recovery work
performed while a lookup remains active contributes to its wall interval.

Two alternative controls checked on the same retry image each record that image
once. Their per-control recapture counts must not be added as though separate
physical screenshots had been taken. Cancellation during a wait does not count
a screenshot that was never attempted.

Example shape for an OK miss followed by the supplied Spider level-505 match;
timing placeholders are deliberately not benchmark claims:

```text
Spider OK search 1: X=930..=954, Y=700..=850 inclusive; bottom-to-top rows, left-to-right pixels; 5 consecutive RGB(0,0,0); result=(0, 0); pixels=3775; rows=151; scan_ms=<measured>.
Spider OK search 2: X=930..=954, Y=700..=850 inclusive; bottom-to-top rows, left-to-right pixels; 5 consecutive RGB(0,0,0); result=(944, 771); pixels=1994; rows=80; scan_ms=<measured>.
Spider OK search summary: searches=2; recaptures=1; second_grab=true; pixels=5769; rows=231; scan_total_ms=<measured>; wall_ms=<measured>; outcome=found.
```

## Verification scope

The shared scanner regression covers 29 native terminal captures against all
three controls, including positive/negative pairs and the K53 retained-pointer
case. Separate regressions cover exact visit counts, first/last legal runs,
missing or invalid images, measured input coordinates, retries and diagnostic
frame retention. The supplied native TriPeaks Play frame matches at (680,905). Native
TriPeaks/Pyramid New Game and Pyramid Play captures are absent; those profile
rectangles still require native-frame confirmation. The Pyramid log nevertheless
records a successful New Game match at (863,862). Live transition acceptance is
separate from these pixel measurements.

See the delivery package's `VERIFICATION.md` for checks actually executed.
Run its verifier with the repository's required nightly Rust toolchain before
live acceptance. Screenshot measurements are not proof of QMP input or animation
behaviour.

## Code Analysis

The scanner reads validated frame storage through safe Rust and checked bounds.
It creates no pixel copy and performs at most the rectangle's area in comparisons.
The configured stripes exclude the origin, so `(0,0)` remains an unambiguous miss.
Input boundaries refuse that sentinel. Visit counts are calculated from the
known traversal and first match, without a second image pass.

The deliberate recognition limit is positional context: any qualifying black
run inside the selected stripe matches. A cursor or unrelated dark content in
that stripe can therefore match too. Narrow bounds remove the observed K53
collision; this is not general word recognition. No new network or filesystem
surface is introduced by scanning or timing. Existing capture and log failures
remain visible through their normal error paths.
