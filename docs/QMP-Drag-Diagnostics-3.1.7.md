# QMP drag diagnostics 3.1.7

Status: application 3.1.7 / xSAR 0.14.0 host qualification passed. Charlie approved the paired REL library build and consumer lockfile alignment on 9 November 2026.

## Approved experiment

A* Free Cell SINGLE STEP offers ABS and REL diagnostic drag choices. Each invocation freezes the selected mode. It executes one route transfer only, with two seconds between gesture phases. Neither mode performs a standalone source or destination click, and the application never retries the other method automatically.

The operator observed the source card visibly raised, then dropped before pointer movement in the earlier 3.1.6 trial. The later target-card relocation was the separately requested visibility click, not a second drag defect. Outgoing JSON and acknowledgements alone cannot identify the guest's held-button state or establish a QEMU routing defect.

## ABS sequence

1. Capture and prove the physical source and intended destination; freshly probe the running VM and absolute pointer.
2. Move to the proved source using checked absolute coordinates.
3. Wait two seconds, then send LEFT down.
4. Wait two seconds, then send the absolute destination movement with no intervening release.
5. Wait two seconds, then send LEFT up.
6. Collect delivery acknowledgements, settle and require the existing fresh semantic endpoint proofs. No extra click follows.

## REL sequence

The library method queries the actual pointer inventory, positions the source absolutely, selects and verifies a unique relative mouse, waits two seconds and freshly verifies that relative selection before LEFT down. It then waits two seconds before signed relative motion and another two seconds before LEFT up. The signed counts derive from the native source/destination delta but are relative device units: guest acceleration can change their visible distance. They are not scaled to the absolute 0..32767 range, and reaching the intended column is not assumed.

After confirmed release, the original pointer can be restored only while authority remains valid. STOP prevents ordinary input and restoration. The approved recovery exception may reselect the captured relative pointer solely to release LEFT on that handler on the old and fresh recovery connection, without repeating a down or movement. A failure or STOP can leave the relative pointer selected; ordinary restoration is not authorised after revocation. Routing is not pinned against concurrent pointer activation, and the application still requires the normal fresh absolute preflight before beginning either trial.

## Packet display

Every outgoing packet is shown as its complete compact JSON on its own readable line in the existing visible LOG and private session log. Connection, request, source context, timestamps and write-attempt/flushed/failed provenance remain separate. Logging neither opens a second QMP connection nor emits extra QMP commands. The existing disabled/disk-failure/abrupt-exit limitations remain.

## Qualification

All 51 required owning/consumer gates passed; see [verification](Verification-3.1.7.md) for exact evidence and limits. No fresh guest trial, input or configuration change has been performed by the agent. Charlie will choose and observe each trial using the qualified executable.

## Code Analysis

Relative movement is an explicitly bounded diagnostic, not calibrated endpoint authority. Fresh frame recognition, exact physical source binding, STOP/context/request ownership, no replay after uncertainty and two consecutive endpoint proofs within six observations remain mandatory. New pointer-selection traffic must be distinguished from button/motion events and logged exactly. All prior evidence is retained separately.
