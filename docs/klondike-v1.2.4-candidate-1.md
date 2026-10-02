# Klondike v1.2.4 candidate 1

Base: `d9ad2cdf536165bb99ae4e02dabc49f7a9fa7227`, whose source and runtime
identify v1.2.3 candidate 5 despite its v1.2.2 commit message. The new pushed
commit changes only seven blank lines in CHANGELOG relative to `27e5016`.
This candidate requires Beast verification before promotion.

## Evidence and correction

K41 is the supplied `261002 225934 FAIL-Out-of-bounds` capture. The preceding
column 3 move was verified; subsequent captures could not recognise the next
source. The ten-card column 6 run has paired rails from row 390 through 990,
with its closing edge at rows 991–993 partly hidden by Undo All. Its complete
height is 604 pixels. The old 600-pixel cap and overlap mask ending before
row 991 independently refused it. The scan limit already included the outline.

The correction accepts this measured height and extends only the existing
column 6 Undo All overlap to rows 988–993. Both exterior rails, the closed top,
the unoccluded lower edge, positive red icon and white card face are still
required. The click is `(1296, 430)` and the effect rectangle ends at row 947.
No click or effect proof is permitted in the toolbar. The existing shorter
K19 and K38 cases remain covered, alongside adverse cropped/broken outlines.

K42 is the supplied `261002 230407 SOLVE-appeared-but-not-detected` capture.
Its stable interior and glyphs pass the existing tolerance, but 198 outer-border
samples exceed the original red tolerance by one or two levels. A later live
validation recognised Solve and requested fresh authorisation before clicking.
The correction permits positive red warming through 26 levels only in the
outer border. Green/blue tolerance, darker-red tolerance, interior, glyph,
empty-stock and gameplay-scene requirements remain unchanged. Selection remains
DRAW/RECYCLE, RIGHT HALO, Solve, tableau, then SUIT.

K43 is the supplied `261002 230440 GAME-WIN-not-detected` capture, showing
Congratulations, level 46 and click-anywhere-to-skip. Its existing terminal
signature passes: maximum RGB difference is 5 against an artwork tolerance of
18; completed-board context matches exactly against its tolerance of 8.
The recogniser is unchanged.

The live log acknowledges Solve at +117.702 seconds, then reports completion
verdicts false, true, false, false. The short budget ends at +123.343 seconds,
5.641 seconds after acknowledgement. K43 is acquired later at +127.913 seconds,
10.211 seconds after acknowledgement. The supplied PNG does not establish
exactly when the intermediate animation became a settled recognised dialog.

After an acknowledged Solve, the worker now uses the existing post-game limit
of 20 delayed observations, in addition to its first capture. It uses the
unchanged editable Solve settle and Klondike re-observation interval. Capture
time is additional: this is not a fixed 20-second timeout. Two consecutive
fresh positive observations remain necessary; a negative resets that count
but never resets the total budget. No card, Solver, Solve or terminal input
is sent while completion remains unconfirmed.

## Preserved behaviour

- Ordinary card recovery and ordinary completion-candidate confirmation keep
  their existing three-delayed-observation bounds.
- One Klondike board is one game. Only continuous Multi-Step 0 navigates the
  independently recognised score, Level Up, New Game, Play and Solver controls.
- The terminal-control sequence, coordinates and transition delays are unchanged.
- Single Step and finite runs stop at confirmed completion before terminal input.
- STOP, uncertain-input refusal, mode/socket invalidation and latest-frame
  retention remain in place. No failed or uncertain Solve is retried.
- TriPeaks/Pyramid source and gameplay policies are unchanged. Charlie's separate
  Pyramid Solver-refresh report awaits its own clean runtime log.
- Capture remains temporary QMP PNG, read, RGBA8 decode and normal cleanup.
  Manual PNG saving retains the originally previewed bytes without recapture.
- Nightly, Rust minimum 1.101.0, dependencies, formatting settings, timing
  defaults and calibration files are preserved. No formatter is run.

## Verification and live limits

K41–K43 derivatives retain native pixels within the manifest's declared
rectangles; other pixels are opaque black. Originals and derivatives have
separate SHA-256 records. Negative tests remove required outline/control
evidence and exercise the precise bounds. Worker tests cover a delayed
false/true/false/false transition followed by consecutive positives, bounded
intermittent positives, uncertainty, STOP and continuous terminal navigation.

The delivery review and validation bundle record actual compiler, unit-test,
Clippy, documentation, release-build and installer rehearsal results. Fixture
and scripted-controller success does not establish live guest input acceptance.
The Beast must verify the long stack, early Solve appearance, animation through
win recognition and the full new-game cycle at the current editable timings.

## Code Analysis

Threat surfaces are guest-controlled frame bytes, QMP input/output, local socket
and capture paths, and the downloaded installer. The correction introduces no
new network endpoint, external command or input class. Calibrated full-frame
geometry, storage and region bounds are checked before pixel reads. Recognition
uses safe Rust and existing owned frame buffers; no unsafe code is introduced.

The taller outline grants no input below the toolbar. The expanded overlap is
specific to the measured Undo All area and requires independent surrounding
evidence. Solve's extra tolerance is directional and limited to its outer border.
Unknown or corrupted scenes continue to refuse input.

Completion requires fresh positive evidence after the one-shot Solve request.
The longer wait is finite, cancellable and input-free; isolated positives cannot
keep it alive indefinitely. Socket errors and uncertain delivery stop without
retrying a non-idempotent action, retaining the latest available frame and log.
Unseen animation durations or terminal layouts can still exhaust observation
bounds and require new Beast evidence rather than speculative clicks.
