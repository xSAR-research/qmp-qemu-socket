# Klondike v1.2.6 candidate 5

This is a correction to delivered candidate 4, against the same committed
candidate-2 base `7d869c22a6ca98179cd3efc8f83552d2edd1ada7`. No remote
refetch, commit or push forms part of this correction. Charlie performs live
verification before promotion.

## Problem and resulting behaviour

After a tableau card moves to SUIT, the next recommendation can use the old
source as its landing position. The landing guide covers the newly revealed
or spreading card. The previous full-card proof can then be unavailable even
though the next source is correctly detected. Candidate 5 adds a separate
tableau-to-SUIT continuation witness; it does not weaken ordinary effect proof.

The witness requires a canonical single-card source, material source change,
a uniquely located exposed header with complete paper/gutter evidence, and
positive transitions in both cursor-free side strips. Neutral guide-paper
whitening does not count. It independently selects exactly one bright materially
changed fixed SUIT receiver, then requires newly received RGB ink matching the
old visible source. Retaining the old source ink population rejects the witness.
Old source pixels at or below row 947 are unavailable; no clipped lower corner
is reclassified as complete. A different fresh canonical recommendation is
still required. The controller records continuation separately from verified
effect or completion and never replays the old input.

The new witness alone permits a reproduced 191-pixel single-card outline,
as recorded in the historical log. Its accepting regression is explicitly
synthetic: genuine K72 border pixels move two rows below the proof boundary;
the detector must independently reproduce the new outline. This is not a
reconstruction of the historical worker frame.
Existing 180–190 eligibility, material thresholds, target detector, mouse/key
coordinates and timings remain unchanged. Candidate 4's SUIT-return, long-outline
and Pro Level Up corrections are retained. Solve and the one-board restart flow,
editable 750 ms action settle, Multi-Step 0, STOP and exact-byte saving remain.

## Evidence boundary

| Pair | Original manual frames | Established observations |
| --- | --- | --- |
| K70/K71 | 114942 predecessor; 114652 failure | Two-clubs appears in SUIT2; black Queen exposed in column7; the column4 run also appears as a landing preview in column7. |
| K72/K73 | 115310 predecessor; 115250 failure | Four-clubs appears in SUIT2; five-hearts spreads in column7 under the next guide; four-spades is the fresh column4 source. |

All four fixtures are byte-identical copies of the original uploaded PNGs.
The log `qmp-qemu-socket-1790991850384.log` records one column7 click at
rows406..597, followed by bounded observations selecting column4 rows390..961.
It stops while verifying the two-clubs move, before clicking the long run.
Its final source counts are 8,546 material, 37 positive and 458 dimmed-felt
pixels; independent destination change is 14,109.

K70's later manual source is rows407..597, and K71 has no selected HALO.
Candidate 4 already verifies that manual pair, with 2,464 source material and
13,719 destination pixels. The new route separately refuses its incomplete
continuous header-gutter evidence and absent fresh target. It is not a reproduction of the logged automatic
frames, counts or input sequence. K72/K73 reproduces the blocked source proof
under native classification; no separate log establishes its historical run.
Neither manual frame pair establishes live candidate-5 acceptance.

## Validation and Beast checks

The delivery review records actual compiler versions, checks, warnings and
installer rehearsals. Read-only fixture analysis is distinct from guest input.
No formatter, warning suppression or global toolchain change is authorised.

The guarded installer requires the exact base above and one complete affected
content state: committed base, delivered candidate 4, or candidate 5. Older
candidate-3-r2 contents and unknown mixtures refuse. Preview changes no repository
files. Apply retains verified private backups and verifies every replacement.
Rollback restores its actual before-state and refuses later unknown edits.
Keep local untracked AGENTS.md, calibration and unrelated work.

On the Beast, require startup identity `v1.2.6, candidate 5`. Exercise the
four-clubs source followed by the guide-covered five-hearts, and the logged
two-clubs source followed by the long run. Confirm one input per logical action,
finite action budgets, continuous STOP before the next input, no uncertain replay
and retained latest frames on failure. Recheck Solve, win/restart and accepted
TriPeaks/Pyramid behaviour before committing. Preserve the first failing log
and original before/result PNGs.

## Code Analysis

The threat surface is false visual authority from guides, stale frames, cursor
pixels or changed unrelated cards. Source headers and fixed receivers are
selected independently of artwork matching; both are mandatory. Checked frame
access, complete header patches and exclusive toolbar bounds keep pixel work
within the decoded RGBA buffer. No host input path or new input operation is
added. Unsupported geometry and ambiguous receivers withdraw continuation.
STOP, QMP probes, uncertain-input refusal and bounded read-only recovery remain
in the existing controller. Pinned archive bounds, exact content guards, verified
backups and actual-before-state rollback protect installation; individual file
replacement does not provide whole-filesystem atomicity.
