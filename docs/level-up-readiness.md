# Level Up OK readiness — candidate.10

The first post-score capture follows the existing 1000 ms intentional wait.
All five controllers use one borrowed-pixel readiness helper before their
single OK click. The fading “You've Reached” caption remains **diagnostic
only**. A 200 ms fresh confirmation wait applies when the current settled-size
button and exact-black central glyph need only geometry agreement. Candidate.10
adds one final OK-only confirmation if that evidence first becomes eligible on
the third normal observation.

## Required evidence

Each fresh control observation measures the button body. The current body must
have settled dimensions and agree with the immediately preceding observation
within one pixel in position, width and height. The current frame must also
supply five consecutive exact RGB(0,0,0) glyph pixels inside the measured
button's central caption band. Separate completion evidence continues to
establish GAME WIN; this readiness helper does not declare a win.

A missing black glyph can seed geometry evidence but cannot authorise input.
Missing body geometry clears consecutive evidence. A fading or brightened
fixed caption changes diagnostics only. The caption check does not inspect the
level number, rank, medal or fireworks art.

The initial frame and two input-free recaptures form three normal observations.
Only a third frame with settled dimensions and a current exact-black central
glyph awaiting geometry agreement may consume one final fresh confirmation.
The fourth frame must qualify independently; it cannot renew the reserve or
authorise another control. There is no fifth observation or extra click budget.
Ordinary New Game and Play keep three observations and their existing coordinate
readiness. A visible pending OK prevents their underlay
from bypassing the foreground modal, even when its black glyph is washed out.

## Conditional confirmation wait

`OkReadiness::observe` reports `confirmation_pending` only when the current
body has settled dimensions, a valid exact-black central glyph is present and
consecutive geometry has not yet agreed. This is scheduling evidence, never
permission to click. `ControlLookup::retry_delay(normal)` selects
`LEVEL_UP_CONFIRMATION_DELAY` (200 ms) for that observation across Free Cell,
Spider, Klondike, Pyramid and TriPeaks. Other observations keep the caller's
normal retry interval. This is the intentional wait before recapture; transport,
PNG encoding and processing add elapsed time. Earlier confirmation waits consume
the normal three-observation allowance. At exhaustion,
`ControlLookup::reserve_final_ok_confirmation(3)` permits one additional OK-only
frame if the current evidence qualifies. Consuming the reserve is permanent
for that lookup, including across resets and later outcomes.

The next capture is checked from its own pixels. A missing or tinted glyph
still withholds input, even after a short wait. A new search, consumed
recapture, explicit reset or error clears the scheduling flag; missing body
geometry also resets consecutive geometry. Acknowledged or uncertain input
cannot be replayed by this policy. The 1000 ms first post-score wait remains.

## Recorded Free Cell late confirmation

The retained sequence in `tests/fixtures/freecell-confirmation-20261016` matches
`qmp-qemu-socket-1792105494041.log`:

| Capture | Candidate.9 observation | Candidate.10 consequence |
| --- | --- | --- |
| 4612.310 s, entering.png | Raw black point (947,723), no recognised body | Normal spacing; no input |
| 4613.501 s, tinted.png | No exact-black run or recognised body | Normal spacing; no input |
| 4614.697 s, clean.png | Body (839,723,237,71), current glyph (942,770), no preceding matching geometry | One final fresh confirmation after 200 ms |

Candidate.9 stopped with `confirmation_pending=true` on observation three. The
glyph search had succeeded; the remaining geometry confirmation had no capture
budget. Candidate.10 reserves that single additional read-only observation.
A simulated fourth clean response can establish eligibility; a missing or
tinted response still stops without OK. No recorded fourth frame was supplied,
so those mock responses test policy rather than predicting live timing.

TriPeaks has separately passed the existing 200 ms path live in session
`1792116927181`: the 89.308 s frame seeded geometry, the 89.680 s fresh frame
qualified at (944,770), and the ordered restart was verified at 96.687 s. That
successful three-observation path is retained.

## Native calibration

Calibration lives in `src/terminal_controls/ok_readiness.rs`, separately from
the per-game raw search rectangles. The existing 13 retained positive layouts
cover Free Cell, Spider, Klondike and TriPeaks. Six further byte-original
captures now cover native Pyramid and TriPeaks entering, clean and tinted
observations. Their clean frames both measure the same shared settled modal.
Pixel eligibility and controlled mock sequences still require live acceptance.

| Check | Native bounds or rule |
| --- | --- |
| Body candidate | Gold strip at X=1050, Y=700–879; height 55–85; horizontal span measured at top+15 within X=810–1109; width 180–270 |
| Gold predicate | R≥145, G≥85, R≥G, G≥B+15 |
| Settled body | Width 235–239, height 68–73; retained positives measure width 237 and height 70 or 71 |
| Stable geometry | X, Y, width and height each differ by at most one pixel from the preceding observation |
| Fixed caption, diagnostic only | 1434 packed white-core samples, translated over Y origins 280–380; every channel ≥235 for at least 90% |
| Dark caption gaps, diagnostic only | At least 18 of 20 fixed holes/gaps have every channel ≤195; a uniform white patch fails the diagnostic |
| Final glyph scan | X=930–954; 35 rows centred on the measured body; five consecutive exact RGB(0,0,0), bottom-up and left-to-right |

The body moves with rank-dependent layout. The helper measures its permitted
translations instead of fixing the OK Y coordinate. The latest raw point is
retained when inside the glyph band; otherwise the unchanged xSAR helper scans
that narrower band. A missing current glyph returns `(0,0)`, never a previous
coordinate.

## Supplied Pyramid and TriPeaks timing failures

The retained originals are in `tests/fixtures/ok-confirmation-20261016`.
These sequences show why keeping the normal 2000 ms wait after a clean but
unconfirmed button can miss its exact-black window.

| Game and session log | Entering capture | Clean capture awaiting geometry | Next capture after normal wait |
| --- | --- | --- | --- |
| TriPeaks, `1792071010302` | 256.627 s: no measured body | 258.781 s: body (839,723,237,71), glyph (943,770), exact RGB(0,0,0) | 261.007 s: same geometry; those glyph pixels are RGB(2,1,0), no exact-black glyph |
| Pyramid, `1792073624308` | 3535.370 s: no measured body | 3537.539 s: body (839,723,237,71), glyph (944,771), exact RGB(0,0,0) | 3539.679 s: those glyph pixels are RGB(2,1,8); tint also breaks the measured gold strip |

The clean second frame was withheld because the first frame supplied no
matching geometry. Candidate.9 schedules its remaining fresh observation after
200 ms. The recorded third frames remain misses: the exact-black requirement
and geometry checks are unchanged. Pyramid's tint is visible in measured
pixels; attributing it specifically to fireworks is not established.

There is no recorded frame from the proposed 200 ms interval. Mock regressions
repeat a clean frame as a controlled fresh-capture response to test the wait,
three-observation cap and one-click policy. They do not demonstrate that the
real game will still be clean then. Native clean→tinted mock sequences also
prove that choosing 200 ms cannot authorise a click from the earlier pixels.

## Supplied Spider failure

Four new fixtures are byte-original copies of the supplied native PNGs, paired
with log `qmp-qemu-socket-1792020298048.log`. The entry score frame remains
separate GAME WIN evidence and has no OK button.

| Capture | Raw black match | Measured body (X,Y,W,H) | Caption diagnostic | Candidate.8 decision |
| --- | --- | --- | --- | --- |
| 748.899 s / entering.png | None | None in the configured body region | False | No click |
| 750.180 s / near-black.png | None; glyph RGB(1,1,1) | (839,723,237,71) | False | Seed geometry; no click |
| 751.461 s / grey-caption.png | (943,770), exact RGB(0,0,0) | (839,723,237,71) | False | Stable body and current glyph; ready on observation three |

Candidate.7 rejected the last frame solely because the fixed caption was not
fully white. Candidate.8 preserves that diagnostic result but permits the
single OK click. Exact-black matching is unchanged: RGB(1,1,1) still fails.
The later manual screenshot is not substituted for the failed sequence's
actual pixels. These decisions establish visual eligibility; Windows accepting
the click still requires live verification.

## Previous Free Cell failure

The earlier native fixtures remain unchanged and correlate with log
`qmp-qemu-socket-1791979335295.log`.

| Capture | Raw black match | Measured body (X,Y,W,H) | Candidate.8 decision |
| --- | --- | --- | --- |
| 340.507 s / early.png | (945,753) | (848,709,219,65) | Entrance size; no click |
| 342.395 s / fading.png | (945,792), pointer outline | (839,723,237,71) | Geometry changed and central glyph absent; no click |
| 343.670 s / settled.png | (945,792), pointer outline | (839,723,237,71) | Stable; rescan current glyph at (941,770) |

Removing the caption veto does not authorise the pointer match or reuse an
old coordinate. The existing entrance and glyph checks still reject these
premature observations.

## Diagnostics and limits

Each `OK readiness` log records raw and selected points, measured body, caption
presence/origin, `caption_gate=false`, geometry stability, `confirmation_pending`, glyph bounds,
decision, reason, additional pixel comparisons and elapsed readiness time.
The successful reason is `settled-button-and-glyph`. This work is separate from
the original black-scanner `pixels`, `rows` and `scan_ms` fields. It contributes
to the encompassing lookup's wall time.

Readiness borrows validated native 1920×1080 pixels, allocates no image copy,
and makes at most 150,000 RGB comparisons per observation. Independent Python
candidate.10 replay checked 98 observations, including native captures and
explicitly simulated fresh confirmations, and measured a maximum of 147,239
comparisons on the uniform-brightness diagnostic case. These are pixel-work
counts, not Rust execution or speed benchmarks. Unknown scale, changed button
geometry or an occluded glyph can withhold readiness. The bounded
final-confirmation checks are recorded in the delivery evidence.
Changed header typography affects the caption diagnostic only.

No screendump, mouse movement or other guest input is sent by the helper.
After readiness the existing fresh QMP probe and STOP checks still precede the
one click. Once OK is acknowledged, later missing controls cannot replay it.
Uncertain delivery also stops without replay. GAME WIN PNG recording includes
the confirmation captures and continues through verified next-game readiness.

## Code Analysis

All pixel reads use a checked frame view and bounded native coordinates. State
holds the previous measured rectangle, a scheduling Boolean and a consumed
reserve Boolean, with no retained click point or additional frame.
The central glyph band rejects the pointer match observed in the supplied Free
Cell sequence. Caption samples remain bounded diagnostics and no longer block
input. This is calibrated visual readiness, not a query of the guest's enabled
state. Capture failures, cancellation and uncertain input retain their existing
stop paths. Dependencies, xSAR source and application capture transport are
unchanged from the matching published bases.
