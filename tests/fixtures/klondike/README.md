# Klondike detector regression fixtures

These nineteen derived PNGs retain original RGBA pixels only in the rectangles
listed in `manifest.json`; other pixels are opaque black. A per-file rectangle
list overrides the default for K16-K19 so their toolbar overlap is retained. Their dimensions and
coordinates remain 1920 × 1080. This avoids retaining unrelated desktop pixels without
resizing, colour conversion or loss of pixels used by this detector's scene,
HALO and effect probes. They are test data, not replacements for the originals.
The manifest records original names and SHA-256 alongside each fixture hash.

K01-K11 correspond to the supplied evidence numbering. K12 is the original
`qmp-qemu-socket 260930 015843 Recycle-Stack.png` showing highlighted exhausted
stock. The five actual action pairs are K02/K03, K04/K05, K06/K07, K08/K09 and
K10/K11. K17/K18 adds Charlie's replayed Draw pair. Other adjacent timestamps
are not assumed to be a single input trace.

K13 is the supplied `qmp-qemu-socket 260930 034141 Solve-available.png`.
K14 is `qmp-qemu-socket 260930 034428 Stopped-Uncertain.png`; K15 is
`qmp-qemu-socket 260930 034701 Failed-to-recycle.png`. K14 and K15 show
result scenes only: no preceding failure-action frame was supplied. They
therefore establish current targets and card artwork, not the cause of the
reported failure or a successful before/after action pair.

K16 is `qmp-qemu-socket 260930 050143 Fail.png`; K17 is
`qmp-qemu-socket 260930 055422 1-step-before-fail.png`; K18 is
`qmp-qemu-socket 260930 055442 Step-where-it-failed.png`; K19 is
`qmp-qemu-socket 260930 055856 Failed-to-detect-big-HALO.png`.
K17/K18 shows Draw revealing 2-hearts at RIGHT fan offset 1. K16 and K18
have identical retained pixels; they do not show the subsequent RIGHT move
succeeding. Charlie reports that replaying through single steps worked, so a
live timing issue remains possible but these frames do not measure it.

K19 supplies the eight-card column-6 source from K-diamonds to 6-clubs. Its
paired exterior rails start at y390 and continue through y957; the 96-pixel
lower closing edge is visible at y951-952 beneath the translucent toolbar.
The toolbar first dims the card at y947. Candidate 1 incorrectly closed the
block at an internal card crossbar, returning rows390..779. Candidate 2
recognises rows390..953 and keeps its click at (1296,430), on the upper card.
The scan ends at y963, accepts the existing darker rail colour only in the
measured y947..962 overlap, and requires closed ends and terminating exterior
rails. The existing 600-pixel height limit remains. Clipped outlines, removed
closing edges, broken rails, toolbar-only pixels and dashed guides are refusal
cases. No result frame for this long-run move has yet been supplied.
Charlie subsequently confirms that Single Step moved the long run successfully;
the detector geometry defect does not by itself explain the continuous-run stop.

Source effect proof ends before y947; destination proof retains its earlier
y936 limit. Toolbar changes therefore cannot establish either side of a move.
The derived fixtures retain toolbar pixels through y1032 so this refusal is
exercised using the original layout rather than blacked-out control pixels.

The production `src/klondike-solve-control.rgb` contains 899 RGB8 samples from
K13, on a four-pixel lattice over the measured Solve control. The manifest
records its exact bounds and hash. Matching requires the control outline,
check-mark/letter artwork and colours, plus a separately empty stock and
calibrated gameplay scene. Solve is distinct from the lower Solver toolbar
control and is sent once before stopping for result review.

Two new tests construct synthetic RIGHT replacements using genuine K14/K15
card faces. An A-spades/J-clubs or 2-diamonds/2-clubs replacement can retain
mostly white paper and fail the old one-way positive-pixel gate. The focused
replacement test checks paper/ink changes in two opposed corners and still
requires an independent destination change. No card rank or suit is decoded.
Guide darkening, unchanged/source-only frames, one changed corner and cursor
changes remain refusal cases. Passing these synthetic cases is not proof of
the original live failure cause or of Beast acceptance.

The Rust tests load these files at runtime under `CARGO_MANIFEST_DIR`; they are
not linked into a release executable. Explicitly synthetic tests translate
real source-card pixels to each waste fan/foundation location and manipulate
mask/cursor/overlay pixels to exercise refusal paths. The synthetic recycle
after frame does not establish live recycle acceptance. Those action classes,
settings and input semantics are authorised by Charlie's 30 September 2026
instructions; new live effects still require observation on the Beast.

The profile remains bounded to the evidenced geometry, Draw 1, 100% guest
scaling and complete source outlines, including only K19's measured toolbar
overlap. Unknown, shifted or generally dimmed layouts stop. The evidenced
Solve-control scene is recognised, but no completion or restart detector is
included.
