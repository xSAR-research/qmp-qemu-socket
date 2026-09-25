# QMP and capture design

## Connection boundary

QMP is QEMU's newline-delimited JSON control protocol. The worker connects to
the user-owned Unix socket for a transaction, negotiates capabilities, performs
the required operation and drops the connection. No long-lived QMP connection
is retained between captures or after STOP.

Before every guest input the worker requires:

1. a successful QMP probe;
2. VM state `running`;
3. the current pointer device to be absolute;
4. no pending STOP request;
5. one typed action from validated visual evidence.

## Capture

QMP `screendump` writes one full-display PNG in QEMU's host namespace. It does
not accept a crop rectangle. The worker therefore:

1. reserves a unique mode-0600 file beside the QMP socket;
2. requests PNG explicitly;
3. reads and decodes the complete primary display;
4. removes the temporary file on every return path;
5. performs tight detector checks in memory.

A screenshot briefly stalls QEMU, so gameplay uses one initial planning frame
and one result frame per ordinary action. Slow score, deal, fireworks and
unknown transitions use deliberately sparse captures rather than rapid
polling.

The decoded frame must be exactly 1920x1080 with a valid four-channel layout.
No scaling, interpolation or coordinate guessing is permitted.

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

## Read-only Pyramid calibration

Selecting Pyramid changes the preview profile but not input authority. Capture
Frame still obtains the full display, Track reports guest pixels on a preview
left-click (with no guest input), and Draw
Targets shows broad positioning envelopes. The worker returns explicit
calibration-only metadata and rejects Pyramid execution before connecting to
QMP.

Future Pyramid calibration must establish exact probes and click points for
`Move`, `Left`, `Right`, and all 28 card slots, plus action effects, Kings,
pair behaviour, stock/recycle transitions and post-game stages.
