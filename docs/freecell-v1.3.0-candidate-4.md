# Free Cell v1.3.0, candidate 4

## Result and base

Candidate 3 rejected the supplied Free Cell sources because their lower HALO
edge was covered by the game toolbar. Its new-game Solver activation could also
arrive while the cards were still being dealt. Candidate 4 addresses these two
boundaries and restores Charlie's requested no-HALO Solver refresh.

The exact source base is pushed candidate 3 at
`676afb477495a166a67132a5735b2347b343585d`. Cargo remains version 1.3.0;
the runtime label is **v1.3.0, candidate 4**. This is a complete-file candidate
for Beast verification, not an accepted gameplay result.

## Toolbar-clipped sources

FC14, FC16 and FC17 are original 1920×1080 RGBA PNGs, copied without modification.
They supply a clipped single card and two clipped runs. The source detector
continues to examine CELL first, then PLAY bottom upwards. It does not scan SUIT.

The first toolbar row is Y=947. A clipped PLAY source can end at that exclusive
visible cutoff when it has a closed solid top, connected opposing exterior
rails reaching row 946, and visible card paper around its click above the
toolbar. All four final rail rows must be present; the existing connected-rail
coverage requirement remains 85%. The small paper probe requires at least half
of its 17×17 pixels to be bright card paper. It establishes a visible card,
without recognising its rank, suit or effect.

The existing 40px bottom click inset is applied to the visible cutoff, giving
Y=907. The minimum 66px visible span encloses that inset, the 8px paper-probe
radius and 18px clearance below the closed top. The detector neither estimates
the hidden lower edge nor reads toolbar pixels for source authority.

| Fixture | One source | Visible bounds | Click |
|---|---|---|---|
| FC14 | PLAY 3, single card | Y=786..947 | (671, 907) |
| FC16 | PLAY 3, four-card run | Y=627..947 | (671, 907) |
| FC17 | PLAY 1, four-card run | Y=627..947 | (286, 907) |

Complete-source geometry and its bottom-card click remain as before. Dashed
destination guides, absent tops, disconnected rails and insufficiently visible
cards do not become actionable. Compression, expansion, identical source
positions and automatic SUIT transfers are observed afresh; card-pixel effect
proof is not part of Free Cell execution.

## Game start and missing HALOs

PARMS adds **After Free Cell game start**, default 1000ms as confirmed by Charlie.
It is independent of the existing action/automatic-transfer settle of 750ms
and repeat-observation delay of 1000ms. Its editable range is 0–5000ms; edits are
snapshotted for the next run and Restore defaults restores all three separately.

Each unresolved frame follows this process:

1. Check independent one-board GAME WIN entry.
2. On a supported board, act on a fresh source HALO if present.
3. With inactive Solver and no HALO, wait the game-start interval once and
   capture again before deciding whether activation is still required.
4. With active Solver and no HALO, wait one repeat-observation interval and
   capture again, allowing automatic card transfers to finish.
5. If the fresh supported board still has no source, click Solver once, settle
   for the action interval, and capture again.
6. Continue bounded input-free observations if necessary. Activation and refresh
   share one reserve in that unresolved context; neither is repeatedly clicked.

A source or win appearing during either pre-Solver wait supersedes the old
decision. An unsupported scene receives only bounded observations. After an
acknowledged Solver click, the editable observation allowance starts again;
the default remains 20 delayed captures, bounded to 1–100. Logs now describe the
observed active/inactive/unavailable state and whether input is activation or
refresh. A log label is not treated as proof that Solver took effect.

After Play, the existing terminal transition settles and captures. The new
game-start interval is then applied if that fresh board is still Solver-off
with no source. This provides Charlie's additional delay without changing
terminal click timings.

## Preserved behaviour and checks

Candidate 3's optional LEVEL UP branch is retained unchanged: after score skip,
local OK alone selects LEVEL UP, local New Game alone skips it, neither receives
bounded observations, and both stop as uncertain. Continuous Multi-Step 0 may
restart through New Game and Play. Single Step and finite runs do not restart.
There is no Free Cell Solve, Draw, Recycle or SUIT-return operation.

The shared QMP/capture/cancellation facilities and TriPeaks/Pyramid/Klondike
runtime modules are unchanged. Manual Capture PNG still saves the original
previewed bytes without a second capture. Capture remains temporary QMP PNG →
read → RGBA8 decode → normal cleanup.

Focused Rust regressions cover the three native clipped sources, independent
toolbar pixels, missing outline/card evidence, the fresh start-delay decision,
one active-Solver refresh, win priority, STOP during waits and uncertain input.
They must be compiled and run on the Beast. The delivery review records actual
workspace checks separately from Rust and live gameplay validation.

Beast verification should resume each supplied clipped source, complete games
with and without LEVEL UP, and verify the restart log shows the game-start wait
and fresh capture before Solver activation. Also check that a temporary no-HALO
automatic-transfer gap can resume directly, and a sustained supported no-HALO
board receives at most one Solver refresh. Preserve the first failed original
PNG and complete log before using Undo.

## Code Analysis

- **Threats:** only a fresh supported source or recognised terminal control
  permits its corresponding input. Toolbar icons and dashed destination guides
  are excluded from source authority. Solver recovery requires a supported
  no-HALO board and a fresh decision after the relevant wait.
- **Memory safety:** safe Rust and the existing validated RGBA reads are used.
  The clipped probe is bounded within the visible source. No unsafe code,
  card recognition, independent search or new frame queue is introduced.
- **Bounds:** source reads and clicks stay at Y<947. Complete sources retain
  their geometry; clipped coordinates are capped at the exclusive cutoff.
  Timings and observation budgets are bounded and snapshotted per run. The
  latest-frame slot and shared cancellation/input checks remain in place.
- **Failures:** STOP, probe/transport/capture/classification errors and uncertain
  input stop without replay and retain the latest available evidence. Missing
  or occluded source evidence can still exhaust the observation allowance.
  A single Solver attempt cannot establish that activation succeeded; only
  subsequent observations permit source input. Unknown reward dialogs remain
  outside the observed terminal sequence.
