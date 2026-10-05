# Spider v1.4.0, candidate 1

## Result and exact base

This candidate adds the approved Solver-led Spider slice to accepted Free Cell
candidate 5. The fresh main checkout was clean at
`4298fe96079f9283159a0b1200876f97de047d7f`, with package/lockfile version 1.3.0.
GitHub's read-only intake matched Charlie's announced push and found no open
issues. That snapshot is the only source base; no later remote poll is used.
Cargo and its lockfile now declare 1.4.0; the runtime label is candidate 1.
Dependencies, minimum Rust 1.101.0, nightly selection and formatting settings
remain. No formatter is run. The Beast's root AGENTS.md remains untracked and
excluded from installation. No local companion AGENTS.md was available in this
intake; the pasted handoff and current session constraints govern.

## Fresh Solver-led actions

Spider owns its ten dynamic PLAY columns and DRAW source. Each run captures
fresh evidence; the initial preview is advisory. DRAW takes priority and sends
one D key press, dealing one card to every column. Otherwise the solid PLAY
source is selected bottom-up and left-to-right, with connected overlapping
outlines grouped as one card/run click. Only visible source geometry supplies
coordinates; a dark dashed destination guide never grants input authority.

Card movement, automatic reveal, run packing, compression and expansion are
observed anew. There is no independent rank recognition, search solver,
source/recipient identity or changed-pixel effect verification. Acknowledged
input consumes one finite budget slot and is logged without claiming its guest
card effect. A repeated fresh HALO may legitimately authorise another action.
COLLAPSED SUITS is display-only and completed runs pack automatically. There is
no Recycle, SUIT-return or Solve-button operation.

Source reads and clicks end before the measured toolbar row Y947. Complete
source edges immediately above it and sources continuing behind icons are
handled using visible outlines/card area. Hidden bottom edges are not inferred.
Stock presence and collapsed packets affect only their observed occupied
regions; absent stock does not establish completion or retain a permanent
exclusion against longer tableau columns. Full-frame capture remains unchanged.

## Editable timing and budgets

| PARAM | Initial value | Basis |
|---|---|---|
| After Spider card / run action | 750 ms | Accepted Free Cell starting value, pending Spider Beast tuning. |
| After Spider DRAW / deal to all columns | 2000 ms | Charlie observed 1–2 seconds from SP08 DRAW to SP09's settled HALO. |
| Spider repeat observation settle | 1000 ms | Initial bounded re-observation interval. |
| After Spider game start | 3000 ms | Accepted Free Cell starting value, pending Spider Beast tuning. |
| Actions per Multi-Step | 0 | Continuous; finite range 1–10000. |
| Delayed observations per unresolved stage | 20 | Editable 1–40; recovery phases remain bounded. |

All timing fields accept 0–5000 ms and are snapshotted for each run. Card and
DRAW delays begin after acknowledged gameplay input. The game-start delay is
applied directly after Play, before a new capture, without a second start wait
before Solver activation. A cold inactive-Solver board also waits once and
captures freshly before activation. Single Step sends at most one logical
card/run or DRAW action; Solver and terminal controls do not consume the
ordinary gameplay budget.

## Missing HALO and one-board completion

Automatic deals and packing can temporarily remove the source HALO. Each fresh
context checks independent win entry and supported source evidence. Supported
active-Solver gameplay first receives an input-free delayed capture; if still
without a source or win, it can receive one Solver refresh. Inactive Solver
receives the start wait and fresh capture before one activation. Activation and
refresh share one reserve per unresolved context. Further recaptures are
bounded. Unknown scenes remain input-free; uncertain delivery is never replayed.

One Spider board is one game. Positive Congratulations plus score-skip or
New Game caption establishes win entry; absent DRAW, absent HALO and isolated
OK/Play buttons do not. Finite runs stop at a win. Continuous 0 follows:
score skip, optional LEVEL UP OK, New Game, Spider Play, deal wait, fresh board,
Solver activation if needed, then fresh source evidence.

After score skip, each fresh frame checks local OK and New Game. Exactly one
ready control selects the branch; both-ready ambiguity stops and neither-ready
gets bounded input-free observations. Subsequent controls are checked in order
and clicked once. These checks use local word contrast and button body, not
levels, medals, fireworks, panel decoration or fixed artwork RGB. The accepted
Free Cell score/OK/New Game helpers are reused; Spider's Play uses its measured
100 px lower position at (709,861). No Spider LEVEL UP capture was supplied;
Charlie confirms the existing OK control, and its accepted fixture is reused.

## Evidence and validation boundary

Twenty originals are byte-identical to the supplied attachments, including
SP01–SP15, SP17–SP19 and the two extra lower-bound/empty-stock examples. All are
1920x1080 RGBA. The fixture README/manifest records observations and hashes.
The evidence includes Easy and Grandmaster boards, without asserting a single
unchanged-difficulty sequence or changing the guest's selection.

Focused native regressions cover the supplied source classes, toolbar exclusion,
DRAW input/delay, mode-local action authority, bounded recovery, optional LEVEL
UP, measured Play offset, finite runs, STOP and uncertain delivery. Actual
executed pixel/installer checks and tool versions are recorded in the delivery
review and validation archive. They are not substituted for compiled Rust or
live QMP execution. Rust/cargo/rustup are absent in the delivery environment;
Charlie must run the complete Beast sequence before acceptance. No new warning
suppression is introduced.

## Beast checks

Confirm the title/log says v1.4.0, candidate 1. Select Spider and verify Capture
Frame/Capture PNG are read-only. Single Step on a card/run should send one click
above the toolbar; Single Step on DRAW should send one D and wait the independent
2000 ms default before the fresh frame. Test SP07/SP10/SP14 and both extra
lower-bound layouts, automatic packing and repeated positions.

Check continuous 0 and a small finite budget, edit each PARAM independently,
and exercise STOP/mode/socket invalidation. Complete games with and without
LEVEL UP. The critical live acceptance is score skip, optional OK, New Game,
Spider's lower Play control, the deal wait, Solver and resumed fresh-HALO play.
Preserve an original PNG and the full log before Undo if anything fails.
This delivery stops at Gate 2 for Charlie's verification; Charlie commits/pushes
only after acceptance.

## Code Analysis

- **Threats:** fresh supported board/source evidence and a canonical typed action
  gate gameplay; dashed guides and display-only packets do not authorise input.
  Terminal input follows independent win entry and expected local controls.
- **Memory safety:** safe Rust, validated native RGBA storage and checked pixel
  reads remain. No unsafe code, effect-comparison frame history or new preview
  queue is added. Shared capture and exact-byte snapshot facilities are reused.
- **Bounds:** source pixels/clicks stay above Y947; dynamic source and Play-offset
  arithmetic is checked. Timing, finite actions and delayed observations are
  bounded and immutable per run. Installer paths, sizes, base/content hashes,
  modes and protected files remain guarded.
- **Failures:** missing targets receive bounded input-free observations/recovery;
  unknown scenes, cancellation, transport/capture/probe errors and uncertain
  input stop with latest evidence. No non-idempotent input is replayed. A fresh
  repeated source may receive another action without effect proof, as requested.
  Native compilation, unobserved layouts and live GAME WIN remain Beast limits.
