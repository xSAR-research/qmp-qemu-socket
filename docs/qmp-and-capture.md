# QMP and capture design

This describes the current shared transport and mode-owned input policies in
v1.3.0 candidate 4. Free Cell uses approved native CELL/PLAY sources and its own
one-board terminal sequence; capture-only requests never send input.
Later dated correction notes retain historical evidence rather than current
card-effect requirements.


## Connection boundary

QMP is QEMU's newline-delimited JSON control protocol. The worker connects to
the user-owned Unix socket and negotiates capabilities. A read-only capture or
manual snapshot request uses one connection and drops it on return. A guarded
run retains its connection across actions, captures and post-game transitions,
then drops it when the run returns. STOP is cooperative; it is checked between
operations and during cancellable waits, while in-flight QMP work may first
complete or time out.

Before every guest input the worker requires:

1. a successful QMP probe;
2. VM state `running`;
3. the current pointer device to be absolute;
4. no pending STOP request;
5. one typed action from validated visual evidence.

## Capture

QMP `screendump` writes one full-display image to a filename in QEMU's host
namespace. It does not return image bytes through the QMP socket or accept a
crop rectangle. The worker:

1. reserves a unique mode-0600 file beside the QMP socket;
2. requests PNG explicitly with an absolute filename;
3. reads the original PNG bytes and decodes the complete primary display to RGBA;
4. attempts to remove the temporary file when its guard drops;
5. performs tight detector checks in memory.

Removal is best effort on normal returns, including ordinary error returns;
the destructor currently ignores removal errors. A process abort or forced
termination can leave a file behind. This pipeline is file-backed even when
the socket directory is on a memory filesystem. File-free automatic capture
is an open requirement, not an implemented or measured property.

Gameplay uses one initial planning frame and one result frame per ordinary
action. Slow score, deal, fireworks and unknown transitions use sparse captures.
The worker records time spent reserving, capturing, reading, decoding and
detecting; these timings describe the current pipeline.

The decoded frame must be exactly 1920x1080 with a valid four-channel layout.
No scaling, interpolation or coordinate guessing is permitted.

**Capture PNG** requests one separate read-only QMP screendump. The worker
delivers its original PNG bytes and decoded frame together to the dialog, where
the user can see the pending image before saving. Save reserves a private
output file and writes those bytes without another QMP command or guest input.
Recapture replaces the pending image and bytes; Cancel discards them. Only the
on-screen preview is scaled. The default socket filename is
`qmp-qemu-socket.sock`; `QMP_SOCKET_PATH` selects an existing QEMU socket.

## Host path contract

QEMU creates the listener at its configured `-qmp unix:` path. The application
must connect to that same socket. An absolute socket path is required for the
current capture flow because it derives the temporary PNG path from the socket
directory and rejects relative screendump output paths. Launching QEMU or the
application from `target/release` does not relocate an absolute path.

The Beast uses `/run/user/1000/qmp-qemu-socket.sock`; its temporary capture
PNGs consequently use `/run/user/1000`. If another deployment chooses a
relative QEMU listener filename, resolve its actual location to an absolute
application path first. QEMU's screendump protocol does not require the socket
directory: that location is this application's current choice.

The application and QEMU must both be able to access the temporary capture
directory in their host filesystem namespaces. The normal configuration uses
the user's runtime directory. The client does not copy files across hosts or
translate container/chroot paths.

The [official QMP screendump contract](https://www.qemu.org/docs/master/interop/qemu-qmp-ref.html#command-screendump)
was checked on 2026-09-26: the command accepts a filename and optional display,
head and format arguments. This application sets `format` to `png` and omits
display/head, selecting the primary display and head 0. The absolute-path
restriction and temporary-file lifecycle are application choices documented
from `qmp.rs` and `worker.rs`.

## Absolute pointer conversion

For a valid guest pixel and content extent, each axis uses widened integer
floor arithmetic:

```text
qmp_axis = guest_axis * 32767 / extent
```

The implementation rejects out-of-range pixels. It does not round, divide by
`extent - 1` or clamp invalid input.

## Input delivery

A click is delivered as three QMP commands:

1. absolute pointer movement;
2. left-button down;
3. left-button up after the configured hold.

TriPeaks Draw, Pyramid's highlighted MOVE/Recycle and confirmed Klondike Draw 1
use qcode `d` down and up with the pointer unchanged.
Klondike recycle uses its recognised stock-area click; `d` is not assumed to recycle.
Pyramid pile and card targets retain their one-click operation. The
removed blank-felt primer click must not be reintroduced without evidence;
the proven Solver click at each board also establishes guest focus.

Down and up are deliberately separate. Once a down command has been flushed,
the release timing must not wait indefinitely for a delayed acknowledgement.
If delivery becomes uncertain, the action is never sent again automatically.

## Mode-owned visual policy

QMP acknowledgement proves only that QEMU accepted a command. It does not
prove that Solitaire acted on it. After the configured settle, the retained
TriPeaks/Pyramid effect-verification path requires a fresh screenshot showing:

- the selected profile's effect evidence inside its calibrated region; and
- a valid resulting gameplay or transition state.

TriPeaks uses material pixel change with cursor exclusion. Pyramid card actions
add positive card-removal evidence; lower controls require a relevant pile
change or positive removal of its unique highlighted partner. A selection-only response or uncertain animation does not consume a
card slot. Pyramid uses editable settle defaults of 1000 ms after Move and
2000 ms after card or pile clicks, then uses bounded additional
result observations when necessary, without repeating uncertain input.

Pyramid additionally allows an identical successor Left/Right pair to continue
from fresh HALOs without proving the previous effect. Both planning and result
must identify that unique eligible pair with known faces; MOVE cannot be
highlighted, tableau must remain visible and no final-card/redeal phase may be
pending. The log distinguishes this route from effect verification. It never
marks a card consumed or advances a completion counter. Step Once still sends
one operation; Multi-Step plans the next operation from the accepted frame.

An ordinary Pyramid result without a halo receives its configured repeat
pause (default 1000 ms) followed
by another fresh QMP screendump and scan. A missing halo does not request
Solver activation; only a positively verified redeal to a new board can
authorise one Solver activation after repeated settled observations. The
latest unverified post-action frame is displayed for diagnosis and cannot
approve the next click. Capture Frame obtains new authority for a later run.

TriPeaks ordinary no-HALO recovery uses up to three delayed input-free captures
at its editable interval before one scene-validated Solver refresh, then up to
three further delayed captures. A late stock HALO authorises D without an extra
Solver click. This recovery does not alter its action-effect or redeal policies.

The accepted result frame is immutable and becomes the next action's planning
frame. Klondike instead follows a fresh valid Solver target without card-pixel
effect proof, as described below. A fresh QMP health/pointer probe remains
mandatory immediately before each active input, including Free Cell source,
Solver activation and expected terminal clicks.

For the shared TriPeaks/Pyramid terminal controller, a fresh pre-score capture
after a completion signal can recover an already
actionable board before any score-skip click. Two consecutive non-gameplay
captures are required before the first score-skip click. A gameplay transition
without a halo is observed again without input. After every score-panel click, including
bounded retries, the worker waits
three seconds before classifying Level Up OK. After an unknown first frame,
it waits and captures again without input before retrying score skip. A click
requires a recognised gold control; if both Level Up layouts match, a strong
gold bridge and button interior must prove one tall button; otherwise bounded
read-only recaptures apply. If a fresh frame instead shows New Game, another
fresh frame
must also recognise New Game before that stage's guarded click. Other post-game
stages retain a one-second inter-stage wait. Challenge Complete Continue is
calibrated for future use but does not authorise automatic input.

## Pyramid profile

Pyramid reuses the shared QMP, Solver activation, progress probe and post-game
controller. Its 31 target slots carry semantic `Move`, `Left`, `Right` and
`Card { row, column }` identities. Fixed 2×2 halo probes select the first
eligible target in that priority order, with cards scanned bottom row upward.
Multiple halos are valid; one click removes a Solver pair or a King.

Only the clicked card's verified removal sets its per-board consumed flag.
The unclicked highlighted partner is not flagged, and lower controls remain
repeatable. The card state resets on a confirmed new board/game, explicit
progress reset, or socket/mode context change. Stale preview predictions cannot
authorise input for another mode or socket.

Missing halos trigger scene classification and bounded recovery, not an
unconditional Move click. Positive completion evidence enters the same
progress-bar and score-skip → Level Up OK → New Game → Play → Solver sequence
used by TriPeaks. The user confirmed this UI and the progress probe position
are shared; live execution acceptance is still required on the Beast.

The Draw Targets overlay and prediction marks are painted only on the preview.
The saved manual PNG always contains the original capture bytes. Undo All
confirmation remains uncalibrated and is not clicked automatically. See
`pyramid-execution.md` for the profile measurements and evidence limits.

## Current Klondike Solver-led policy

Charlie authorised Klondike-specific Solver recovery on 30 September 2026:
recognised gameplay with no eligible HALO can receive one Solver activation,
then a capture immediately after acknowledged button release, without an added
settle delay. Pointer positioning and button hold remain part of input delivery.
Recovery is bounded, STOP-aware and never repeats an uncertain operation.
Later unresolved observations use the editable Klondike recapture interval.
This policy does not alter Pyramid or TriPeaks recovery rules.

Klondike source detection consumes one immutable RGBA frame in priority order:
DRAW/RECYCLE, RIGHT HALO, RIGHT Solve, tableau bottom upwards and SUIT.
It does not take a separate screenshot for each target class. The preview is
advisory; each fresh supported scene and canonical target authorises one logical
operation, followed by editable settle and fresh capture. Card ranks, source or
recipient matching and changed-pixel effect proof do not gate play. Delivery
is acknowledged while its previous effect remains explicitly unproven.

Live tableau recognition checks only Y<947. Source blocks continuing behind the
toolbar use their visible closed top, connected opposing rails and card paper;
their reported exclusive bottom is 947. No hidden lower edge is inferred and
the single upper source click stays above the toolbar. The captured frame itself
remains full size. Upper piles, Solver and terminal controls have separate areas.

Ordinary missing HALOs receive at most three delayed input-free observations,
then independent completion review. At most one Solver refresh is allowed per
unresolved context on a positively recognised gameplay scene, followed by an
immediate capture and at most three further delayed observations. Unknown scenes
or uncertain delivery stop without input replay. Single Step sends at most one
gameplay operation; finite budgets count acknowledged operations. Continuous
Multi-Step 0 follows only each new valid target and remains STOP-cancellable.

A separately recognised upper Solve button permits one auto-finish request,
its own editable animation delay and up to 20 delayed read-only completion
captures at the editable Klondike re-observation interval. The bound is inherited
from the existing post-game observation limit, independent of card recovery.
It never triggers Solver recovery or an input retry after that click. One board
is one Klondike game; two fresh positive observations independently establish
completion. The active Solver banner requires every calibrated right-interior
pixel gold and none black; completed-game artwork provides a separate positive
proof. Background and overlays cannot satisfy completion through black absence.

Only continuous authority permits the independently recognised Klondike
score-skip → Level Up OK → New Game → Draw 1 Play → fresh-board Solver sequence.
Once that win context is established, readiness checks only the expected local
gold OK, New Game or Play button and its printed-caption contrast. Rank, medal,
title, surrounding frame and fireworks colours do not gate these ordered clicks.
Each control receives a QMP health/tablet probe and STOP check, one input,
existing calibrated hold/settle timings, then fresh observation. Missing readiness
receives up to 20 delayed input-free observations; uncertain or unchanged input
is not repeated. The fresh deal still requires positive mode-owned evidence
before Solver activation and bounded HALO recovery. The Klondike Play point
remains independent from the shared three-board calibration.

Capture/decode failures stop. Later manual PNGs remain separate evidence and
cannot be assumed byte-identical to an earlier automatic result frame.


## Free Cell input boundary

Free Cell reuses full-frame capture and original-byte snapshot saving. Only a
fresh supported board and canonical solid CELL/PLAY source authorise one
bottom-card click. There is no card-rank or changed-pixel effect proof, SUIT
source, stock key, Recycle or Solve input. Source reads/clicks remain above Y=947.
Native FC14/FC16/FC17 support clipped sources with a closed visible top, connected
rails reaching the boundary and visible paper around the click. The boundary is
an exclusive visible cutoff, not an estimated hidden card edge.

One acknowledged action settles for the editable initial 750 ms and captures
anew. Automatic SUIT transfers can leave no HALO: check independent win entry,
otherwise wait the editable initial 1000 ms and recapture without input before
one Solver refresh on a supported active board. An inactive supported board
waits the separate game-start interval (initially 1000 ms), then captures freshly
before activation. Each context permits at most one Solver click and an editable
input-free allowance, initially 20, bounded 1–100. Every input has a fresh
VM/tablet probe and STOP check; uncertain input is not replayed.

One board is one game. A positively recognised score/skip or completed-game
New Game panel permits continuous mode to follow score skip, optional OK,
New Game and Play controls, each once. After score counting, each fresh frame
checks both local OK and New Game readiness. A skipped LEVEL UP proceeds
directly to New Game; ambiguous simultaneous controls stop. Finite runs do not
restart. The next
control must be freshly ready; unrelated artwork is not checked. Existing
score/terminal hold and settle timings are reused, followed by bounded input-free
observations. A fresh board, Solver activation if needed, and a new source frame
are required before play resumes. Capture stays full-frame and saving stays
byte-identical. See [Free Cell Gate 1](freecell-gate-1.md).


## Historical Klondike correction notes

The following notes preserve earlier candidate measurements and verification
limits. Their card-effect witnesses, initial-preview equality and artwork-wide
terminal checks do not override the current policy above.

## v1.2.5 Klondike diagnostics and initial recovery

Each guarded Klondike observation reports the stable Solve interior and glyph
counts separately from outer artwork, together with the selected target.
Terminal evidence identifies recognised stages and scene/artwork guard results.
These are read-only classifications of the same fresh captured frame.

An exact no-HALO initial validation can enable Solver once through its own scene
and input guards. Immediate capture and bounded delayed captures remain. A
continuous request resumes from a freshly recognised target; finite requests
return the preview for explicit approval. No missing HALO authorises a Draw or
card click, and a QMP acknowledgement does not establish the previous game effect.
Capture and exact-byte manual saving are unchanged.

## v1.2.6 pending Solve observations

Pending Solve uses the existing editable Klondike re-observation delay for at
most three fresh input-free captures before lower-priority tableau/SUIT input.
The weaker diagnostic predicate can only request observation; the existing full
Solve predicate is required before a click. Persistent or unsupported evidence
stops without another gameplay, Solver or terminal event. Previous accepted
action effect and finite action counts are not recomputed by this planning wait.
Capture architecture and original-byte manual saving are unchanged.


Candidate 2 installation checks distinguish download integrity, exact-base
installation and executable identity. `--verify-installed` checks complete
candidate source before Cargo/launch commands and refuses old or mixed files.
The application still reports its Cargo package and candidate label at startup.
Capture/QMP policy is unchanged. The hearts Undo attachment contains a transient
duplicated source card; it cannot substitute for a settled fresh pre-click PNG.

## Klondike candidate 3 source evidence

Fresh captured source geometry may change when a compressed stack expands. The
bounded clipped-card comparison maps both images to relative card positions and
excludes the commanded cursor neighbourhood in both sampled coordinate spaces.
It supplies qualified continuation only; it does not replay input or establish
complete effect. QMP/capture, editable waits and original-byte saving are unchanged.

The later Draw result failed scene classification before any stack click. Its
measured replacement lower probe is a 7-by-28 rectangle with the same 196 pixels
and 95% felt support, rather than a wider scan. The King transfer retains separate
shade diagnostics and accepts their disjoint positive-felt union only under the
existing source, non-source, cursor, gold and geometry guards. Full effect proof
and the shared execution controller remain unchanged.


## Klondike candidate 4 SUIT-return evidence

The matched SUIT return keeps the original cursor exclusion and ordinary effect
thresholds. Positive source-corner replacement and newly changed recipient print
are measured from the same two decoded observations; no extra capture, input,
rank reader or destination click is introduced. Receiving geometry and transferred
artwork have no click authority. A passing witness authorises fresh-target
continuation only and is logged separately from complete action verification.
Unsupported or ambiguous evidence follows the existing bounded read-only stop.


K67's Pro Level Up layout is recognised from the same fresh decoded terminal
frame. The preceding score click had been acknowledged; no OK click was attempted
before the reported recognition stop. Separate measured title/label/frame/button
evidence permits the existing LevelUp action without replaying prior input.


K68/K69 calibrate source-outline recognition from newly supplied originals.
The four-pixel scan extension and bounded column-7 shadow-gold rail support
change detection only. No capture timing, input retry, click location or effect
bound is introduced. The separate Strange-fail screenshot is recognised by
the previous detector; its matching log is still required to explain the stop.


Candidate 5 adds no QMP command, input coordinate or delay. Its tableau-to-SUIT
continuation witness uses the retained before/result RGBA frames after one
acknowledged source click. A new landing guide can cover the old source; an
independent exposed header and newly received old artwork are required before
continuing from the fresh target. QMP acknowledgement alone never supplies this
proof, and uncertain delivery stops before recovery or another gameplay input.


## Candidate 6 source and restart observations

The column4 Hint-shadow correction concerns read-only outline recognition only;
its positive gold samples do not extend card-effect proof into the toolbar.
Fresh initial-deal paper margins replace a rank-sensitive area test after Play.
The existing acknowledged Solver activation still runs once, then captures fresh
frames under bounded observation and cancellation gates. No capture path, PNG
identity contract, mouse/key operation or retry policy changes.


The current candidate-6 Klondike policy omits live card-effect comparison. QMP
acknowledgement still establishes delivery only; the next fresh valid Solver
recommendation authorises the next action. Independent completion is unchanged.
A finite run captures once after its last action and returns; continuous mode
uses bounded input-free no-HALO observations before one scene-validated Solver
refresh. STOP, native input bounds and uncertain-input non-replay remain.
