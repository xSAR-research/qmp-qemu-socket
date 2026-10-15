# Level Up OK readiness — candidate.7

The first post-score capture still follows a 1000 ms intentional wait. An
exact-black OK match is now a candidate, rather than permission to click during
the modal's entrance animation. All five controllers use the same borrowed-pixel
readiness helper before the single OK click.

## Required evidence

Each fresh control observation measures the button body. The current body must
have settled dimensions and agree with the immediately preceding observation
within one pixel in position, width and height. The current frame must also show
the invariant “You've Reached” caption fully and supply a current exact-black
glyph run inside the measured button's central caption band.

A fading caption or missing black glyph can seed geometry evidence. Neither
permits input. Missing body geometry clears consecutive evidence. The fixed
caption check does not inspect the level number, rank, medal or fireworks art.
Separate completion evidence continues to establish GAME WIN.

The initial frame and at most two input-free recaptures remain the entire
control budget. Readiness does not create another capture loop or click attempt.
A first button appearing only on observation three stops without a click if
there was no preceding matching body. Ordinary New Game and Play keep their
existing coordinate readiness. A visible pending OK prevents their underlay
from bypassing the foreground modal, even when its black glyph is washed out.

## Native calibration

Calibration lives in `src/terminal_controls/ok_readiness.rs`, separately from
the per-game raw search rectangles. It covers 13 retained Level Up captures
from Free Cell, Spider, Klondike and TriPeaks. Pyramid uses the same shared modal
calibration and still requires live acceptance; no native Pyramid settled-frame
positive is claimed by this package.

| Check | Native bounds or rule |
| --- | --- |
| Body candidate | Gold strip at X=1050, Y=700–879; height 55–85; horizontal span measured at top+15 within X=810–1109; width 180–270 |
| Gold predicate | R≥145, G≥85, R≥G, G≥B+15 |
| Settled body | Width 235–239, height 68–73; all retained positives measure width 237 and height 70 or 71 |
| Stable geometry | Each edge representation differs by at most one pixel from the preceding observation |
| Fixed caption | 1434 packed white-core samples, translated over Y origins 280–380; every channel ≥235 for at least 90% |
| Dark caption gaps | At least 18 of 20 fixed holes/gaps have every channel ≤195; a uniform white patch fails |
| Final glyph scan | X=930–954; 35 rows centred on the measured body; five consecutive exact RGB(0,0,0), bottom-up and left-to-right |

The caption moves vertically between native layouts; the body also moves with
rank-dependent layout. The rule therefore measures their permitted translations
instead of fixing the OK Y coordinate. The latest raw point is retained when
inside the glyph band; otherwise the unchanged xSAR helper scans that narrower
band. A missing current glyph returns `(0,0)`, never a previous coordinate.

## Supplied Free Cell failure

The fixture files are unchanged copies of the supplied native PNGs. Their
elapsed acquisition times correlate with log `qmp-qemu-socket-1791979335295.log`.

| Capture | Raw black match | Measured body (X,Y,W,H) | Fixed caption | Readiness |
| --- | --- | --- | --- | --- |
| 340.507 s / early.png | (945,753) | (848,709,219,65) | Not fully shown | Entrance size; no click |
| 342.395 s / fading.png | (945,792), pointer outline | (839,723,237,71) | Fading | Geometry changed; no click |
| 343.670 s / settled.png | (945,792), pointer outline | (839,723,237,71) | Fully shown at Y=350 | Stable; select current glyph (941,770) |

The historical early point was inside both visible button shapes. These pixels
support premature animation input as the leading explanation; they do not
reveal the game's internal enabled state. Pixel replay establishes the new
visual decisions, not that Windows accepted a QMP click. Live acceptance remains
required across all five games.

## Diagnostics and limits

Each `OK readiness` log records raw and selected points, measured body, caption
presence/origin, geometry stability, glyph bounds, decision, reason, additional
pixel comparisons and elapsed readiness time. This work is separate from the
original black-scanner `pixels`, `rows` and `scan_ms` fields. It contributes to
the encompassing lookup's wall time.

Readiness borrows validated native 1920×1080 pixels, allocates no image copy,
and makes at most 150,000 RGB comparisons per observation. Independent replay
checked 62 observations and measured a maximum of 147,239 comparisons on the
uniform-brightness negative case. Those are pixel-work counts, not Rust speed
benchmarks. Unknown scale, changed typography or occluded evidence can safely
withhold readiness; the three-observation budget is not extended to compensate.

No screendump, mouse movement or other guest input is sent by the helper.
After readiness the existing fresh QMP probe and STOP checks still precede the
one click. Once OK is acknowledged, later missing controls cannot replay it.
GAME WIN PNG recording includes the confirmation captures and continues only
through verified next-game readiness.

## Code Analysis

All pixel reads use a checked frame view and bounded native coordinates. State
holds only the previous measured rectangle, rather than a frame or click point.
Fixed white and dark caption samples resist a simple brightness washout; the
central glyph band rejects the pointer match observed in the supplied sequence.
This is calibrated visual readiness, not a query of the guest's enabled state.
Capture failures, cancellation and uncertain input retain their existing
stop paths. Dependencies, xSAR and capture transport are unchanged.
