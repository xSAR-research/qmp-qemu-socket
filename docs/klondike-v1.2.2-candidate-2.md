# Klondike v1.2.2, candidate 2

Date: 30 September 2026, Australia/Brisbane.
Committed base: v1.2.1 at `1cc57c21e696759b5d61bbcee24b96a733e9c430`.
This corrects the uncommitted v1.2.2 candidate 1 against the same base.
The prior candidate notes remain historical evidence.

## Tall highlighted run

The supplied `055856 Failed-to-detect-big-HALO.png` shows the source in column 6:
K-diamonds, Q-spades, J-diamonds, 10-clubs, 9-diamonds, 8-clubs, 7-diamonds and
6-clubs. Its gold outline extends into the top of the bottom toolbar. The dashed
guide is in the empty column 5. The source is one connected actionable run.

A regression using those pixels establishes that candidate 1 incorrectly ends
the source at an internal gold crossbar, returning top 390 and bottom 779. The
closing edge is at bottom 953. Its old lower scan bound of 936 misses that edge;
the toolbar also dims the lower gold below the ordinary bright-edge threshold.
The screenshot establishes incorrect source geometry. Without that run's log,
it does not establish which live controller stage stopped.

Charlie subsequently reports that Single Step moved this long highlighted run
successfully. This confirms the existing one-click source-block contract for
the scene; incorrect source bounds alone do not explain the continuous-run
stop. No after-move PNG or log for that transfer has been supplied.

Candidate 2 searches the measured overlap, follows both exterior rails, and
rejects internal crossbars while the rails continue below them. The dimmed
closing edge is accepted only in the measured toolbar overlap and still needs
a continuous edge, connected side rails and bright card paper above it.
Clipped or otherwise incomplete blocks remain unsupported.

Visual source bounds and input/effect bounds have different purposes. The
complete outline identifies the run, but its click remains near the top of the
source and above the toolbar. Material source and destination comparisons also
exclude toolbar pixels, so Undo/Undo All/other control changes cannot prove a
card transfer. No destination click, drag or rank recognition is introduced.

## RIGHT replay and timing evidence

Charlie's v1.2.2 candidate 1 log establishes one successful recycle and two
successful Draw operations before a RIGHT action stopped with four observations
showing no qualifying source or destination change. These are filtered detector
counts: inset borders, gold, the cursor area and small channel differences are
excluded. Zero counts do not prove whole-frame byte identity or conclusively
identify a failed guest input.

The reconstructed `055422 1-step-before-fail.png` and
`055442 Step-where-it-failed.png` show a Draw from the stock, exposing the
highlighted 2-hearts in RIGHT. They are not a recorded RIGHT transfer pair.
Charlie reports successful Single Steps and a subsequent continuous replay.
That supports the existing one-click input contract while leaving an
intermittent input/readiness problem possible.

Single Step and continuous execution use the same click path: acknowledged
pointer movement, 100 ms pointer settle, 50 ms mouse hold, release, and QMP
response handling. The independent editable 750 ms action-settle interval
starts after that, before capture. Capture and classification add their own
time. Draw retains its 20 ms key hold. No replacement interval is established
by these still images or the successful replay.

Candidate 2 logs those existing intervals and measures input response handling
and observation time. Timings remain unchanged. For a controlled Beast
comparison, the action delay can be changed in Params from 750 to the previously
used 2000 ms; that is a diagnostic setting, not a demonstrated fix. Change one
setting at a time and retain any new failure log and original frame. An
unverified action still stops and is never automatically replayed.

## Candidate installation and verification

The candidate 2 installer recognises the exact committed base, the complete
known candidate 1 contents, or an already installed candidate 2. It rejects
unknown affected-file edits and mixed states. Applying over candidate 1 backs
up the actual candidate 1 files; rollback restores that pre-install state.
Candidate 2 remains uncommitted until Charlie accepts and promotes it.

The separate candidate review records compiler versions, all checks, limitations
and the entire numbered Beast sequence. Relevant regressions cover the new
original-pixel scenes, the reconstructed Draw pair, complete tall-run bounds,
internal crossbars, dimmed edges, toolbar-only changes and unsafe geometry.
Live acceptance must verify the full source prediction and transfer above the
toolbar. The intermittent RIGHT failure and Solve completion/restart remain
open; a successful offline regression does not establish those behaviours.

Continuous mode, 750/1000 ms editable action/re-observation defaults, bounded
recovery, STOP, diagnostic frame retention and the one-shot Solve review
endpoint carry forward. QMP, accepted TriPeaks/Pyramid policies, calibration,
nightly selection, dependency versions and exact-byte manual PNG saving remain
as in candidate 1. No formatter is run.
