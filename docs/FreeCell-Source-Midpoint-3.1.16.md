# Free Cell source-up and midpoint diagnostic — 3.1.16

Application 3.1.16 implements the approved held path using xSAR 0.16.1. All 51 fresh paired host gates passed with 1,893 application, 366 library and 80 console tests. The full suites report zero failed, ignored, measured or filtered tests.

## Native path

Let S be the freshly proved source and D the effective destination. PLAY uses its observed printed-corner destination. CELL and SUIT use their canonical slot centre. The source retains its observed printed-corner position.

| Input | Position or button |
| --- | --- |
| 1 | Source ABS S, with matching acknowledgement |
| 2 | LEFT down |
| 3 | `(S.x, S.y - 45)` |
| 4 | `(floor((S.x + D.x) / 2), floor((S.y + D.y) / 2))` |
| 5 | `(D.x, D.y - 45)` |
| 6 | Exact destination D |
| 7 | LEFT up |

The midpoint uses the original source and effective destination in native pixels. Each sum widens to i64 before division. The 45-pixel displacement is the integer quarter of the existing 182-pixel native slot height. Source, observed destination and all four held points must fit the native frame. Source or destination y below 45 refuses before input. The final two CELL/SUIT points must lie inside the same target slot.

Both buttons use the selected minimum delay across all six consecutive input boundaries. TwoSeconds gives at least 12 seconds source-to-UP. HalfSecond gives at least 3 seconds. Source acknowledgement, routing queries, socket work and logging can add time.

## Authority and failure handling

The application owns the native path. The existing xSAR bounded four-point API validates and maps it before authority callbacks and operation I/O. Its source acknowledgement and fresh selected-pointer verification precede DOWN. UP is sent before collecting the six held-phase replies. Uncertain post-DOWN failure retains the original cause and uses the existing captured-pointer release-only recovery. Original pointer restoration requires confirmed UP and continuing authority.

Fresh original 52-card binding and relevant source, target and recorded AUTO proofs govern each invocation. Ordinary effects require two consecutive fresh exact final recorded endpoints within six observations. STOP, context, source, plan and run ownership checks apply throughout. QMP acknowledgements establish command delivery. Windows cursor/button state and the game's handling of the gesture require operator evidence.

The public xSAR API has the same signatures and implementation. Version 0.16.1 updates its caller-supplied path regression tests. Ordinary route gestures, REL, recognition and search retain their existing source contracts.

## Approval and evidence

Charlie explicitly approved this trial and both patch versions. Standing approval also permits PATCH version increases for qmp-qemu-socket and xsar while debugging DRAG and trialling authorised movement processes. Each trial still records its own scope and verification. Major/minor version transitions and Git publication retain their applicable approval boundaries.

Saved native fixtures and local TCP/QMP peers exercise coordinates, timing, input ordering and failure handling. Fresh Windows gameplay acceptance is pending operator trial. Publication commands are prepared separately for Charlie to run.
