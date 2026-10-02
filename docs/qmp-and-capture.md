# QMP and capture design

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

TriPeaks and confirmed Klondike Draw 1 use qcode `d` down and up with the pointer unchanged.
Klondike recycle uses its recognised stock-area click; `d` is not assumed to recycle.
Pyramid Move, pile and card targets are mouse clicks, never the draw key. The
removed blank-felt primer click must not be reintroduced without evidence;
the proven Solver click at each board also establishes guest focus.

Down and up are deliberately separate. Once a down command has been flushed,
the release timing must not wait indefinitely for a delayed acknowledgement.
If delivery becomes uncertain, the action is never sent again automatically.

## Visual verification

QMP acknowledgement proves only that QEMU accepted a command. It does not
prove that Solitaire acted on it. After the configured settle, the normal
effect-verification path requires a fresh screenshot showing:

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

The verified result frame is immutable and becomes the next action's planning
frame. A fresh QMP health/pointer probe remains mandatory immediately before
the next input.

After a completion signal, a fresh pre-score capture can recover an already
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

## Klondike exception to missing-HALO handling

Charlie authorised Klondike-specific Solver recovery on 30 September 2026:
recognised gameplay with no eligible HALO can receive one Solver activation,
then a capture immediately after acknowledged button release, without an added
settle delay. Pointer positioning and button hold remain part of input delivery.
Recovery is bounded, STOP-aware and never repeats an uncertain operation.
Later unresolved observations use the editable Klondike recapture interval.
This policy does not alter Pyramid or TriPeaks recovery rules.

Klondike source detection consumes one immutable RGBA frame in priority order.
It does not take a separate screenshot for each target class. Each accepted
result is reused as the next planning frame. Initial validation must reproduce
the approved target before any gameplay or no-HALO Solver recovery input. A
changed target publishes the fresh prediction for another explicit review and
Step request, with zero guest input.

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
Each control receives a QMP health/tablet probe and STOP check, one input,
existing calibrated hold/settle timings, then fresh observation. The Klondike
Play button is higher than the shared three-board calibration and remains
mode-specific. Missing or unchanged stages receive at most three delayed
input-free observations; no uncertain or unchanged control is clicked again.
Gameplay resumes only from the freshly recognised actionable next Solver board.

Candidate2 adds input-free result recapture for a temporarily unsupported
Klondike scene. Capture/decode failures still stop immediately; this does not
retry QMP delivery. Exact stock/artwork/glyph Solve measurements are logged from
run observations. Later manual PNGs remain separate evidence and cannot be
assumed byte-identical to an earlier automatic result frame.
