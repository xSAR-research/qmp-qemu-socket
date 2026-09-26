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

With a relative QEMU listener filename, the socket location depends on QEMU's
launch directory. Resolve that actual location before setting the application's
path. For a listener named `qmp-qemu-socket.sock` created from the Beast's
`/home/charlie/repo/RUST/qmp-qemu-socket/target/release`, use that directory plus
the filename as the absolute application path. Temporary PNGs will then also be
placed there by the worker. QEMU's screendump protocol does not require this
directory: putting PNGs beside the socket is this application's current choice.

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

DRAW is delivered as qcode `D` down and up with the pointer unchanged. The
removed blank-felt primer click must not be reintroduced without evidence;
the proven Solver click at each board also establishes guest focus.

Down and up are deliberately separate. Once a down command has been flushed,
the release timing must not wait indefinitely for a delayed acknowledgement.
If delivery becomes uncertain, the action is never sent again automatically.

## Visual verification

QMP acknowledgement proves only that QEMU accepted a command. It does not
prove that Solitaire acted on it. After the configured settle, the worker
requires a fresh screenshot showing:

- material pixel change inside the action's effect region; and
- a valid resulting gameplay or transition state.

The verified result frame is immutable and becomes the next action's planning
frame. A fresh QMP health/pointer probe remains mandatory immediately before
the next input.

After every score-panel click, including bounded retries, the worker waits
three seconds before classifying Level Up OK. A click requires the recognised
gold control. If a fresh frame instead shows New Game, a second fresh frame
must also recognise New Game before that stage's guarded click. Other post-game
stages retain a one-second inter-stage wait. Challenge Complete Continue is
calibrated for future use but does not authorise automatic input.

## Read-only Pyramid calibration

Selecting Pyramid changes the preview profile but not input authority. Capture
Frame still obtains the full display, Track reports guest pixels on a preview
left-click (with no guest input), and Draw Targets shows the 28 card bounds,
Left/Move/Right control bounds and the shared bottom toolbar. The worker returns explicit
calibration-only metadata and rejects Pyramid execution before connecting to
QMP.

Pyramid target centres remain read-only metadata. Future execution work must
validate HALO probes and hit-point behaviour for `Move`, `Left`, `Right`, and all
28 card slots, plus action effects, Kings, pair behaviour, stock/recycle
transitions and post-game stages. An Undo All confirmation position has not
been established from the supplied images. See `pyramid-calibration.md` for
the preview calibration and the checks still required on the Beast.
