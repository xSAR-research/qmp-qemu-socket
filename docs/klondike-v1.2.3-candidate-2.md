# Klondike v1.2.3, candidate 2

Committed base: v1.2.2 candidate 2 at
`bcce7515979e2c4520b02f39e9c4d4d1bb019d7f`.
Prepared: 1 October 2026, Australia/Brisbane. Beast verification is pending.

## Two different stops

The 02:53 session stopped during fresh initial validation. Its displayed
prediction was RIGHT, but the fresh frame identified tableau column 5. The log
explicitly records `guest input attempted=false`: that run did not test the
candidate 1 source-removal correction. Exact initial approval remains required.
Candidate 2 publishes the valid refreshed prediction for review without input;
another explicit Step request freshly validates that prediction. An approved
card changing to no-HALO also cannot silently authorise a Solver click.

The later log supplied in `Pasted text(4).txt` contains a genuine effect refusal.
The column 7 source at `425..615` was clicked once and acknowledged. Four fresh
observations reported 3984 materially changed source pixels, only 428 positive
replacement pixels, 9 dimmed-felt pixels and 17151 destination changes. The next
source at `407..597` was detected. Extra observations could not remedy that
source predicate.

The reconstructed Undo pair independently shows 2-clubs transferred from column
7 to below 3-diamonds in column 3, automatically revealing King-spades in column
7. The former source remains mostly white card paper, with different artwork.
Both opposed source corners contain ink-to-paper and paper-to-ink changes,
outside the cursor and gold masks. This is an additional source-face replacement
proof, not a reduction of the existing 512-pixel source/destination thresholds.
It is restricted to the evidenced single-card block. Uniform darkening, uniform
whitening, cursor changes, gold changes and source-only changes remain refusals.
No card rank or suit reader is added.

## Completion and controls

Detection uses one immutable frame in this order: DRAW/recycle, RIGHT HALO,
RIGHT Solve, tableau bottom upwards, SUIT sources, then completion evidence.
Card/draw/recycle settling defaults to 750 ms, Solve animation settling separately
defaults to 750 ms, and re-observation defaults to 1000 ms. All are editable.
These retain existing authorised timing values; still images do not measure
animation duration. Actions per Multi-Step defaults to 0.

One Klondike board is one game. Completion needs two consecutive fresh positive
observations. The progress route requires an intact active Solver banner, no
eligible action, every one of the calibrated 76 right-interior pixels gold and
zero black. No black on blue dialogs or the ordinary green background is not a
win. The supplied pre-Solve frame still has 56 black pixels and is a refusal.
The completed-game score, Level Up and Congratulations artwork supplies a
separate independent route when overlays obscure the progress bar.

Solve receives one guarded click and its separate animation wait. The first
result is captured, followed by at most three delayed read-only observations.
An unresolved result stops without another Solve or Solver click. Finite runs,
including Single Step, stop at a confirmed win before terminal inputs.

Only continuous Multi-Step 0 may progress through the supplied terminal sequence:

| Recognised scene | One authorised control | Fresh result required |
|---|---|---|
| Congratulations reward counting | Click inside the score panel to skip | Klondike Level Up |
| Klondike Level Up | OK | Congratulations with New Game |
| Congratulations result | New Game | Klondike Standard Draw 1 selector |
| Draw 1 selector retaining guest settings | Play | Fresh seven-column deal with Solver off |
| Fresh deal with Solver off | Solver | Canonical actionable new Klondike board |

The first supplied win panel says “Click anywhere to skip”; it has no OK button.
Klondike's Play button is higher than the shared three-board calibration, so its
measured click remains mode-owned. Existing post-game holds and settle intervals
are reused. Each control needs a new QMP/tablet probe and STOP check. Unknown or
unchanged stages receive bounded input-free observations; unexpected recognised
stages stop. No terminal input is retried. The completed-game notification and
continuous resume occur only after the next actionable Solver board is verified.

## Evidence and limits

K24/K25 cover the changed-preview and pre-Solve scenes. K26–K30 cover the five
terminal stages. K31/K32 retain the actual reconstructed replacement pair. The
fixture manifest records original attachment names, hashes, native dimensions,
retained rectangles and fixture hashes. Derived fixtures retain unmodified RGBA
probe pixels and are re-encoded PNGs; they are not byte-identical original files.
Reconstructed-pair pixel counts differ slightly from runtime screenshots, so
tests verify the evidenced result rather than asserting literal runtime counts.

The full-gold progress endpoint is additionally tested with a labelled synthetic
frame; no original PERFECT/full-bar screenshot was supplied. Terminal artwork
has real positive examples. Unsupported themes, shifted layouts, other draw
settings, ambiguous effects and unknown terminal stages stop for inspection.
Recognition fixtures and deterministic worker tests do not prove live input or
new-game timing. Charlie's candidate 1 Beast build passed all 226 tests; candidate
2's actual checks and tool versions are recorded in its separate review.

## Beast verification

Close the running app before installation. The guarded installer accepts exact
committed-base, installed candidate 1 or candidate 2 contents, stops on unknown
affected edits or staged changes, and backs up the actual starting state. An
upgrade rollback restores candidate 1, including its previously created files.
Untracked AGENTS.md, calibration, ignore files and unrelated workspace edits are
outside the payload.

Replay the 2-clubs transfer in Single Step, then continuous mode. Check both
source-corner measurements and the independent destination count. Recheck the
candidate 1 dimmed-felt removal and the tall run. Confirm changed previews send
zero input and become reviewable. Confirm visible Multi-Step 0 on startup and
editable independent card, Solve and re-observation settings. Verify finite
Solve stops before terminal inputs and continuous mode completes exactly one
board per game, advances through all five recognised stages and resumes.

Check STOP during gameplay, Solve observations and every terminal wait; unchanged
or unsupported dialogs must stop without repeating the last click. Recheck
TriPeaks/Pyramid, mode/socket invalidation, exact-byte manual PNG saving and
warning-free Beast checks. Retain the first result PNG and full session log
before Undo or manual play if another refusal occurs. No promotion or GitHub
write is authorised by this candidate delivery.
