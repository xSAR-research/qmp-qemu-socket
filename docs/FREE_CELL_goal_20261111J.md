# Free Cell drop-anchor correction — 3.1.19

Status: host-qualified. Application-only correction with xSAR 0.16.2 and console 0.8.5.

## Approved outcome

A pickup at the recognised card corner plus 15 px horizontally and 20 px vertically now uses the same offset at an empty CELL, SUIT or PLAY destination. Occupied destinations retain their observed recognised-corner point. The route no longer replaces CELL/SUIT observations with slot centres. This aligns the planned cursor endpoints with the existing source anchor; it does not change calibration, recognition thresholds or physical-effect proof.

The six diagnostic inputs remain source → LEFT DOWN → source minus 45 px vertically → native floor midpoint of the ORIGINAL source and effective destination → exact destination → LEFT UP. Standard SINGLE STEP retains five 500 ms gesture intervals, fast SINGLE STEP five 100 ms intervals, and MULTI STEP freezes ABS with five 200 ms intervals. Those intentional gesture waits total 2.5 s, 500 ms and 1 s respectively, excluding transport, scheduling, logging and capture costs. Every newly reached recorded AUTO edge retains its 500 ms animation wait.

Calculated A* routes retain one `RecordedMove` card identity per transfer. Rules and independent replay require an exposed source and remove/place exactly one card. The user confirmed individual single-card transfers suffice. No carried-stack feature or stack regression is added; legacy Solver/HALO handling remains separate.

## Original failure and retained evidence

The original app 3.1.18 run 61/request 1/source-generation 1 packet `pyramid-error-1794397492303-1326849-3` records JC PLAY 1→CELL 4 at transfer 40/87. The planned pointer endpoints were (233,722) and slot centre (811,203). CELL 4 face bounds are `[743,880)×[112,294)`; the recognised source origin is (218,702), giving a (15,20) pickup offset. The old destination uses (68,91), a +53 px,+71 px anchor mismatch. Under a retained-offset model this predicts a card origin (796,183), while a matching destination cursor is (758,132). These are model predictions, not measurements of a held card at release. The operator reported a drop too far right followed by return. The retained first/latest PNGs show empty CELL 4 and JC returning/returned near PLAY 1; they do not preserve the held destination state. The session log explicitly selected HalfSecond 500 ms and its six input timestamps have adjacent gaps 501, 500, 500, 501, 500 ms. This was not a fast-timing run. One attempted and acknowledged gesture proves delivery only; failed child proof earns no cursor advancement and no replay authority. The separate 3.1.17 6S CELL 4→SUIT Spades failure remains a distinct historical incident. Original packet/PNGs and exact source/log hashes/line anchors are retained in [incident-assessment.json](../Evidence/FreeCell/Validation/freecell-drop-anchor-319-20261111/incident-assessment.json).

## Acceptance

All six fresh application gates passed with 1,908 tests and zero failed, ignored, measured or filtered tests. xSAR 0.16.2 and console 0.8.5 retain historical qualification evidence; no fresh standalone sibling gates are claimed.

Fresh supported selected-game evidence, original 52-card binding, VM-running and absolute-pointer probes, validated native coordinates, acknowledged source positioning and the selected source-ACK wait remain required before LEFT down. Source/run/context/STOP fences are rechecked at the existing boundaries. Two fresh semantic source/destination/AUTO endpoint proofs, bounded to six effect captures, determine effect. QMP acknowledgement proves delivery and never physical effect. Uncertain non-idempotent gestures are never replayed; failure and revocation preserve release-only recovery.

Fresh Windows acceptance of 3.1.19 remains pending. Host regressions use retained/synthetic pixels and local TCP/Unix-socket peers. The retained JC frame demonstrates direct child-position refusal, and local peers verify the corrected outgoing packet sequence. This correction does not launch an app, take a new guest capture, send guest input, install, commit, push or publish. A rebuild does not replace an already running process; live acceptance must identify the executing host and capture-server builds.

See [goal](FREE_CELL_goal_20261111J.md), [contract](FreeCell-Drop-Anchor-3.1.19.md) and [verification](Verification-3.1.19.md).
