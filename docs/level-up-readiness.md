# Level Up OK readiness — candidate.8

The first post-score capture follows the existing 1000 ms intentional wait.
All five controllers use one borrowed-pixel readiness helper before their
single OK click. Candidate.8 makes the fading “You've Reached” caption
**diagnostic only**; it cannot veto sufficient current button evidence.

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

The initial frame and at most two input-free recaptures remain the entire
control budget. Readiness does not create another capture loop or click attempt.
A first button appearing only on observation three stops without a click if
there was no preceding matching body. Ordinary New Game and Play keep their
existing coordinate readiness. A visible pending OK prevents their underlay
from bypassing the foreground modal, even when its black glyph is washed out.

## Native calibration

Calibration lives in `src/terminal_controls/ok_readiness.rs`, separately from
the per-game raw search rectangles. The existing 13 retained positive layouts
cover Free Cell, Spider, Klondike and TriPeaks. Pyramid uses the same shared
modal calibration and still requires live acceptance; no native Pyramid
settled-frame positive is claimed by this package. New shared-controller tests
use the supplied Spider modal as an explicit fixture after independent GAME WIN
context, rather than labelling it as a native Pyramid capture.

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
presence/origin, `caption_gate=false`, geometry stability, glyph bounds,
decision, reason, additional pixel comparisons and elapsed readiness time.
The successful reason is `settled-button-and-glyph`. This work is separate from
the original black-scanner `pixels`, `rows` and `scan_ms` fields. It contributes
to the encompassing lookup's wall time.

Readiness borrows validated native 1920×1080 pixels, allocates no image copy,
and makes at most 150,000 RGB comparisons per observation. Independent Python
replay checked 71 observations and measured a maximum of 147,239 comparisons
on the uniform-brightness diagnostic case. These are pixel-work counts, not
Rust execution or speed benchmarks. Unknown scale, changed button geometry or
an occluded glyph can withhold readiness; the three-observation budget is not
extended. Changed header typography affects the caption diagnostic only.

No screendump, mouse movement or other guest input is sent by the helper.
After readiness the existing fresh QMP probe and STOP checks still precede the
one click. Once OK is acknowledged, later missing controls cannot replay it.
Uncertain delivery also stops without replay. GAME WIN PNG recording includes
the confirmation captures and continues through verified next-game readiness.

## Code Analysis

All pixel reads use a checked frame view and bounded native coordinates. State
holds only the previous measured rectangle, rather than a frame or click point.
The central glyph band rejects the pointer match observed in the supplied Free
Cell sequence. Caption samples remain bounded diagnostics and no longer block
input. This is calibrated visual readiness, not a query of the guest's enabled
state. Capture failures, cancellation and uncertain input retain their existing
stop paths. Dependencies, xSAR source and application capture transport are
unchanged from the matching published bases.
