# Free Cell ABS destination approach — 3.1.13 / xSAR 0.15.1

## Approved sequence

Replace the midpoint with an approach at `(destination.x, destination.y - 4)` in native pixels, followed by the exact destination. For the reported QS PLAY1 to PLAY5 operation, source `(286,730)` leads to approach `(1056,673)` then destination `(1056,677)`. No additional click is added.

| Input | TwoSeconds minimum schedule | HalfSecond minimum schedule |
| --- | ---: | ---: |
| Source ABS, acknowledged | 0 s | 0 s |
| LEFT down | 2 s | 0.5 s |
| Approach ABS, 4 native pixels above destination | 4 s | 1 s |
| Exact destination ABS | 6 s | 1.5 s |
| LEFT up | 8 s | 2 s |

These are four minimum waits; source acknowledgement, fresh routing, socket operations and synchronous logging may add time. Held requests are sent before collecting all four corresponding replies, after UP. The default method and slow button select TwoSeconds; the fast button selects HalfSecond and refuses REL before reservation. Timing remains frozen in the queued command.

## Bounds and compatibility

xSAR computes checked destination.y minus the named four-pixel native offset, then uses the existing checked `pixel * 32767 / extent` mapping. Both endpoint validations and mapped-endpoint alias refusal retain their precedence. A destination y below four refuses before authority callbacks or operation I/O, with no clamp/fallback. An intermediate mapped alias is permitted, preserving the previous diagnostic contract at very large extents. The application supplies a freshly proved physical destination; no new generic column target-region API or pixel recognition policy is introduced.

This is a correction to existing diagnostic behaviour, with unchanged public signatures, DiagnosticDragTiming variants, QmpConfig, feature defaults, dependencies and deal/route schemas. It is intentionally not the previous midpoint trajectory. Host-qualified application 3.1.13 and library 0.15.1 identify this revision; console stays 0.8.5. Path-only consumer dependency requirements remain unchanged and lock resolutions align. Passing tests alone does not establish compatibility.

## Input and evidence rules

Fresh original 52-card binding, exact current parent, source/run/context/STOP checks and VM/absolute-pointer probes remain prerequisites. Source ACK and fresh selected-pointer routing precede DOWN. Every permitted phase uses the existing guards; any uncertain post-DOWN outcome uses captured-pointer old/fresh release-only recovery, never DOWN/motion replay. Ordinary restoration requires confirmed UP and continuing authority. Waits are synchronous and STOP is observed at boundaries, not guaranteed instantaneously.

Transport ACK grants no effect credit. Ordinary route advancement still requires two consecutive fresh exact combined endpoint observations within six; recognition/calibration, recorded AUTO follow-ups, route cache and manual UNDO ALL restart rules stay unchanged. REL remains its existing direction-only movement probe; remaining-route playback keeps ordinary drag.

The application records planned approach coordinates distinctly from actual wire events; source timestamps identify phases actually reached. Existing logger durability limitations apply. Original operator failure files are preserved separately in the private qualification capsule without obtaining new frames or replaying input.

## Operator observation and future experiment

The operator reports that both 3.1.12 timing profiles visibly moved the dragged card to the midpoint, and that prior ten-step slow drags succeeded. A one-update lag is consistent with those reports but its mechanism is unproved. Current failure details record attempted1/acknowledged1 with six exact endpoint refusals, not a proved move.

The operator also confirmed that a drop anywhere inside the valid destination column region can succeed. If this approach trial succeeds, a waypoint at the destination column centre then final destination may be considered, with bounds for full and empty columns. That future trajectory and reduced delays are not implemented or qualified here. No claim about game internals, Windows held-button state, reliable minimum pacing or fresh guest acceptance follows from host tests.

See [verification](Verification-3.1.13.md).
