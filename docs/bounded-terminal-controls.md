# Bounded terminal controls and search metrics

Candidate: qmp-qemu-socket 2.2.2-candidate.10, installed over the exact candidate.9
source on TCP-enabled checkpoint `40975060deeb2fa703874c309b6a8ddfc958f095`.
It uses the existing xsar
helper from `6765e0a89eddf610cbad8cb302db98a31465cdff` without changing that crate.

## Search contract

All five game types use one application adapter for expected OK, New Game and
Play controls. It calls `xsar::image_matching::find_black_run_bottom_up` with
a run length of five. Pixels must have RGB exactly `(0,0,0)`; alpha is ignored.
Rows run bottom-to-top and columns left-to-right. The first qualifying run's
leftmost full-frame coordinate is a candidate point. A miss returns `(0,0)`.
For OK, visual readiness narrows the final exact-black scan to the measured
settled glyph band; only that latest validated point may reach input.

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

An absent control or unsettled OK receives three normal observations: the
initial frame and at most two input-free recaptures. If observation three has
a settled-size OK body and a current exact-black central glyph awaiting only
geometry agreement, a lookup may consume one final 200 ms OK-only confirmation.
That fourth frame must independently pass readiness. No fifth observation is
permitted. Missing, tinted, entering or pointer-only third frames receive no
reserve. New Game and Play retain three observations. Unresolved searches keep
the latest frame as a diagnostic preview.

Optional OK/New Game branches may examine both rectangles on the first three
frames. The reserved fourth frame cannot authorise a different control, score
skip or Solver recovery. The shared TriPeaks/Pyramid controller retires an earlier menu stage after its
click is delivered; black runs in that earlier region cannot authorise another
click or reset the current missing-control allowance. New Game takes precedence
at the optional Level Up step unless a measured foreground OK button or
independent TriPeaks foreground-modal context requires OK first. A pending OK
button also blocks a speculative score-skip budget renewal. Recognised reward/gameplay
recovery remains distinct from missing-control observations. Uncertain QMP input is not
replayed by the search retry.

`TERMINAL_CONTROL_SEARCH_ATTEMPTS` owns the three normal observations across all
five game types. `TERMINAL_OK_FINAL_CONFIRMATIONS` supplies one conditional
reserve, consumed by `ControlLookup::reserve_final_ok_confirmation` at normal
observation three. That reserve belongs to the lookup and cannot be renewed by
a geometry reset, another search, recapture or outcome change. Recognised reward
or gameplay recovery remains a separate path with its existing observation and
input budgets; those transitions can reset the consecutive-miss count in the
shared TriPeaks/Pyramid controller. Therefore three is a limit on consecutive
unresolved control frames, not a universal cap on every recovery episode.

`LEVEL_UP_APPEAR_DELAY` is now 1000 ms after an acknowledged score-skip click,
reduced from 3000 ms so the first OK search starts before the reported later
fireworks. Capture and recognition add elapsed time beyond this intentional
wait. `LEVEL_UP_CONFIRMATION_DELAY` is 200 ms only when the latest frame has a
settled-size OK body and current exact-black central glyph, awaiting geometry
agreement with a fresh frame. `ControlLookup::retry_delay` selects that wait
across all five games and logs `chosen_ms`, `normal_ms` and
`confirmation_pending`. It never authorises input. The fresh observation must
still pass all readiness checks. A clean candidate first appearing on normal
observation three can use the one final confirmation; that scheduling flag
never supplies click authority from the earlier frame.

Other missing-control spacing remains 2000 ms in the shared TriPeaks/Pyramid
controller, and the existing editable reobserve interval in Free Cell, Spider
and Klondike. Entering, missing, tinted and pointer-only evidence cannot select
the short confirmation wait. A new search, consumed recapture, reset or error
clears that eligibility. Score, gameplay, Solver and click budgets remain
independent.

Capture transport is unchanged from the TCP-enabled base. Production capture
defaults to `QMP_CAPTURE_BACKEND=tcp` at `127.0.0.1:9999`; explicit `qmp` retains
the legacy private-file PNG route. TCP errors stop without silent fallback.
The matching xSAR client's protocol v2, validated `u16` stride and 14-byte header
are preserved. The verifier checks that sibling commit and runs its focused TCP
protocol integration tests before application checks. QMP input and probes are
unchanged by this candidate.

`--image` adds scoped GAME WIN evidence without requesting extra captures or
changing the recording boundary. See [GAME WIN image diagnostics](game-win-images.md)
for directory reset, capture/log correlation and the recording boundary.

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
frame retention. Candidate.4 extends menu coverage to success on the third
attempt, a third final miss, cancellation before the final recapture and
input-free synthetic OK washout in an otherwise native Level Up panel. These
are regression definitions; see the delivery record for execution status.
The supplied native TriPeaks Play frame matches at (680,905). Native
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

Raw exact-black matches remain positional candidates and may include a
cursor. OK additionally requires settled measured geometry agreeing across
consecutive fresh observations and a current exact-black glyph in the central
band. The fixed caption remains diagnostic only (`caption_gate=false`). This
rejects the supplied Free Cell pointer-outline point. New Game and Play retain
their established bounded positional policy. See [Level Up readiness](level-up-readiness.md)
for calibration, native evidence and limits. No new network or filesystem
surface is introduced by scanning or timing. Existing capture and log failures
remain visible through their normal error paths.
