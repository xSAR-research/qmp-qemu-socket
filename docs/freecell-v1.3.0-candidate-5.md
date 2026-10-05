# Free Cell v1.3.0, candidate 5

## Result and exact base

FC18 and FC19 show complete three-card source outlines whose lower gold edge
ends immediately above the toolbar. Candidate 4 excluded them from its complete
source branch and could not classify them as clipped sources. Candidate 5
accepts the observed closed bottom, retaining the toolbar exclusion and existing
gold relationships. It also sets the editable game-start default to 3000ms,
as confirmed by Charlie's live deal timing.

The exact source base is pushed candidate 4 at
`2345898e8a59e37f065d93ebda51726af0a8887f`. Cargo stays version 1.3.0;
the runtime label becomes **v1.3.0, candidate 5**. The fresh checkout was clean
and its committed candidate-4 files matched the delivered content before edits.

## Closed outline at the toolbar boundary

The first toolbar row remains Y947. Source detection never reads that row or
anything below it. Both new source groups start at Y641 and have a complete
lower crossbar through row946, producing exclusive bottom947.

| Fixture | One source group | Last paired rail | Exclusive bottom | Click |
|---|---|---|---|---|
| FC18 | PLAY 1: six of spades / five of diamonds / four of clubs | Y944 | Y947 | (286,907) |
| FC19 | PLAY 3: five of clubs / four of diamonds / three of clubs | Y945 | Y947 | (671,907) |

Candidate 4 rejected a complete source when its last paired rail occupied the
last eight scan rows. Its clipped branch instead required the rails to reach
row946. The rounded closed corners in FC18/FC19 fall between those policies:
their side rails stop earlier and their bottom crossbar is fully visible.

For a PLAY outline near the cutoff, candidate 5 accepts the complete end only
when the paired rails terminate before row946 and a closing crossbar lies below
the last paired rail. The existing closed top, 180px minimum complete-source
height and 85% rail coverage remain. The exclusive bottom may equal Y947;
every authoritative pixel is still above the toolbar. The existing bottom-40
click gives Y907.

Rails continuing through row946 still use candidate 4's clipped-source path,
including its visible card-paper probe. A failed clipped source cannot fall
through to the new closed-bottom allowance. CELL policy is unchanged. Interior
crossbars, absent tops and dashed guides do not establish a closing bottom.

FC19 has card ink at the click probe; its fully closed outline supplies authority
without reducing the clipped paper threshold. There is no rank matching,
recipient matching, previous-move verification or changed-pixel effect proof.
Each connected source group receives one click.

## Game-start default

**After Free Cell game start** now defaults to **3000ms**. Its PARMS range stays
0–5000ms. The UI, Restore defaults and snapshotted run settings use the same
constant. Action/automatic-transfer settle remains 750ms; repeat observation
remains 1000ms. The supplied candidate-4 log already shows Charlie selected
3000ms manually; this candidate makes that value the startup default.

Worker flow is unchanged. Inactive Solver with no source gets one game-start
wait and fresh capture before activation. An active no-HALO board gets one
delayed input-free capture before a single refresh if still needed. Independent
GAME WIN entry is checked first. The optional LEVEL UP, New Game and Play
sequence remains unchanged, as do STOP, mode/socket invalidation and uncertain
input handling. Only continuous Multi-Step 0 restarts games.

## Evidence and validation boundary

FC18/FC19 are complete supplied 1920×1080 RGBA PNGs copied without modification.
Their hashes and filenames are in the fixture manifest. The attached log shows
three acknowledged source actions, then supported gameplay and active Solver
without a detected HALO. It sent one Solver refresh and continued observations
until Charlie pressed STOP. It did not report a QMP connection failure or apply
a card-effect pixel threshold.

FC18's supplied attachment is 3,203,290 bytes; its log records a saved PNG of
2,544,532 bytes. Fixture equality is verified against the supplied attachment.
Equality to that earlier saved encoding and the reason for the size difference
are unverified. The native pixels are inspected directly, and shared capture/
original-byte saving code remains unchanged.

One existing worker regression now checks the ordered wait sequence instead of
counting equal durations: both game-start and LEVEL UP delays are 3000ms, so
duration equality no longer identifies a stage. Production worker code is unchanged.

Focused Rust regressions cover both original boundary sources, toolbar-pixel
independence, near-edge closing authority, failed clipped fallthrough and
unchanged CELL behaviour. Existing FC01–FC17 tests remain. The delivery review
records actual workspace checks separately from Rust compilation and live input.
No formatter or warning suppression is used.

On the Beast, resume each supplied boundary source, then complete a game and
verify the new deal uses the 3000ms default before fresh Solver activation.
Also retain the previously accepted clipped-source and optional LEVEL UP cases.
Preserve an original PNG and full log before Undo if another failure occurs.
This candidate stops at Gate 2 for Charlie's verification.

## Code Analysis

- **Threats:** only the current supported source outline authorises input.
  The new near-edge path requires the last rails to terminate before a visible
  closing crossbar; interior lines and toolbar icons cannot substitute for it.
- **Memory safety:** safe Rust and existing validated RGBA reads remain.
  Crossbar scans stop before the exclusive cutoff; no unsafe code or frame queue
  is introduced.
- **Bounds:** source reads and clicks stay at Y<947. Complete-source height,
  rail coverage, CELL policy and the clipped paper path are preserved. Timings
  remain independently bounded and snapshotted; the controller's action and
  observation limits are unchanged.
- **Failures:** missing or incomplete outlines still lead to bounded recovery
  and a stop. Transport/capture/probe/classification errors and uncertain input
  retain evidence and do not replay input. Known terminal flow remains as in
  candidate 4; unknown reward controls remain outside that observed sequence.
