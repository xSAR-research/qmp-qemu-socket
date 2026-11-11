# Free Cell midpoint — application 3.1.12 / xSAR 0.15.0

## Input sequence

Only ABS Single Step changes. Both buttons use the same five separate `input-send-event` requests:

| Input | SINGLE STEP, nominal time | SINGLE STEP 0.5 s, nominal time |
| --- | --- | --- |
| Source ABS; await matching acknowledgement | 0 s | 0 s |
| LEFT down, after fresh routing check | 2 s | 0.5 s |
| Native midpoint ABS | 4 s | 1 s |
| Exact destination ABS | 6 s | 1.5 s |
| LEFT up | 8 s | 2 s |

These are minimum input intervals. Source acknowledgement, queries, guards, logging and socket work add elapsed time. The source pause follows its acknowledgement; each later pause precedes the next held input. No additional pause or click follows UP. All four held-phase acknowledgements are collected after transmitting UP.

For each validated nonnegative native axis, midpoint is `(i64(source) + i64(destination)) / 2`, checked back to the coordinate type. Compute it before QMP mapping: map each native point with `pixel * 32767 / extent`, using checked bounds and floor division. Endpoint aliases remain refused; a midpoint that maps to an endpoint on a short path is permitted. Do not interpolate already scaled QMP values.

The additional button is available only with ABS selected and the same idle, source and plan prerequisites as SINGLE STEP. Freeze its typed timing in the queued command and telemetry. Changing the displayed selector cannot change an in-flight gesture. Existing REL probe timing and normal remaining-route drag retain their contracts.

## API and safety

xSAR adds `DiagnosticDragTiming::{TwoSeconds, HalfSecond}` and `diagnostic_drag_with_timing`. The existing `diagnostic_drag` signature remains available and selects TwoSeconds. Existing configuration/default features/dependencies and other public signatures remain unchanged. The pre-1.0 0.14.3 to 0.15.0 minor transition is explicitly approved; both consumers use unversioned local path dependencies and receive matching lock resolutions. A path and lock resolution do not pin the xSAR source revision.

Retain fresh complete physical source binding, exact parent proof, VM/absolute-pointer probing and STOP/context/source/run checks. Waits check authority before and after sleeping; cancellation is cooperative. Routing is not pinned across the held interval. After possible DOWN, failures retain their original cause and use only existing captured-handler old/fresh safety UP. No DOWN or motion is replayed. Restore the original pointer only after confirmed UP and continuing authority; STOP can leave a diagnostic pointer selected.

Acknowledgement establishes transport delivery. The application still requires two successive fresh exact final combined endpoints within six observations before accepting effect or advancing its cursor. Recorded AUTO policy, terminal retirement and cached manual UNDO ALL recovery remain unchanged. Gold challenge outlines give no route instruction. Recognition is outside this change.

## Protocol commands

`query-mice` lists available emulated pointer handlers, their current status and absolute/relative type; it sends no gesture. `human-monitor-command` carries a textual HMP command. `mouse_set N` selects the handler whose queried index is N; N is not a permanent tablet identity. The application queries and verifies selection before pressing. LEFT down has no coordinates and uses the selected handler's last pointer position. See the [QEMU monitor command reference](https://www.qemu.org/docs/master/system/monitor.html) and [tagged QMP UI schema](https://github.com/qemu/qemu/blob/v11.1.3/qapi/ui.json).

## Operator observation

Launch the qualified executable after closing an earlier process. For an eligible cached retry, manually UNDO ALL to the original board, then request a new step; otherwise acquire and calculate a fresh supported deal. Select ABS. Compare one slow step with one separately requested 0.5-second step, retaining any original failure evidence. These are timing trials, not a claim that either interval is a reliable minimum or a proven Windows drag correction.
