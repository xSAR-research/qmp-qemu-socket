# Free Cell drop anchor — 3.1.19

Status: host-qualified.

## Coordinate contract

A pickup at the recognised card corner plus 15 px horizontally and 20 px vertically now uses the same offset at an empty CELL, SUIT or PLAY destination. Occupied destinations retain their observed recognised-corner point. The route no longer replaces CELL/SUIT observations with slot centres. This aligns the planned cursor endpoints with the existing source anchor; it does not change calibration, recognition thresholds or physical-effect proof.

The corner recrop identifies printed identity; it does not assert a measured card-face edge. For the retained JC parent, target (758,132) preserves the same recognised 15 px, 20 px offset used at source (233,722). Corrected midpoint (495,427) is the floor of the ORIGINAL source plus effective destination divided by two on each native axis. The first held point remains source minus 45 px vertically.

## Gesture, single-card route and authority

The six diagnostic inputs remain source → LEFT DOWN → source minus 45 px vertically → native floor midpoint of the ORIGINAL source and effective destination → exact destination → LEFT UP. Standard SINGLE STEP retains five 500 ms gesture intervals, fast SINGLE STEP five 100 ms intervals, and MULTI STEP freezes ABS with five 200 ms intervals. Those intentional gesture waits total 2.5 s, 500 ms and 1 s respectively, excluding transport, scheduling, logging and capture costs. Every newly reached recorded AUTO edge retains its 500 ms animation wait.

Calculated A* routes retain one `RecordedMove` card identity per transfer. Rules and independent replay require an exposed source and remove/place exactly one card. The user confirmed individual single-card transfers suffice. No carried-stack feature or stack regression is added; legacy Solver/HALO handling remains separate.

Fresh supported selected-game evidence, original 52-card binding, VM-running and absolute-pointer probes, validated native coordinates, acknowledged source positioning and the selected source-ACK wait remain required before LEFT down. Source/run/context/STOP fences are rechecked at the existing boundaries. Two fresh semantic source/destination/AUTO endpoint proofs, bounded to six effect captures, determine effect. QMP acknowledgement proves delivery and never physical effect. Uncertain non-idempotent gestures are never replayed; failure and revocation preserve release-only recovery.

## Retained 3.1.18 playback behaviour

The first post-transfer wait remains min(configured settle, 200 ms); one eligible endpoint match uses min(configured settle, 100 ms) before confirmation. Unmatched frames and intermediate AUTO endpoints use the configured retry gap. Two successive fresh exact semantic endpoint matches are required within six captures; the sixth rejection ends immediately. At the default 750 ms configuration, all-unmatched captures follow 200, 950, 1,700, 2,450, 3,200 and 3,950 ms of intentional waiting, excluding acquisition/proof costs. This retains 3.1.18 cadence, including its 550 ms shorter all-unmatched window relative to the earlier sequence from 750 to 4,500 ms.

MULTI retains STOP AFTER STEP bound to the exact plan/run. The current transfer, two-frame proof, recorded AUTO follow-ups and animation waits finish before a proved cursor is retained for resumption. A request before the next transfer, including one received inside its PAUSE checkpoint, prevents that transfer. PAUSE remains independent and requires CONTINUE. Immediate STOP revokes further authority and permits only existing release-only recovery.

Playback retains the bounded latest-frame preview slot and releases completed same-context RECON preview ownership on successful dispatch. Every already acquired playback frame reaches that slot before semantic interpretation. Pending-RECON and stale-context fences remain. During a held gesture the preview retains the newest acquired frame until the next existing capture; there is no added in-flight capture or capture-only QMP connection.

## Original JC failure

The original app 3.1.18 run 61/request 1/source-generation 1 packet `pyramid-error-1794397492303-1326849-3` records JC PLAY 1→CELL 4 at transfer 40/87. The planned pointer endpoints were (233,722) and slot centre (811,203). CELL 4 face bounds are `[743,880)×[112,294)`; the recognised source origin is (218,702), giving a (15,20) pickup offset. The old destination uses (68,91), a +53 px,+71 px anchor mismatch. Under a retained-offset model this predicts a card origin (796,183), while a matching destination cursor is (758,132). These are model predictions, not measurements of a held card at release. The operator reported a drop too far right followed by return. The retained first/latest PNGs show empty CELL 4 and JC returning/returned near PLAY 1; they do not preserve the held destination state. The session log explicitly selected HalfSecond 500 ms and its six input timestamps have adjacent gaps 501, 500, 500, 501, 500 ms. This was not a fast-timing run. One attempted and acknowledged gesture proves delivery only; failed child proof earns no cursor advancement and no replay authority. The separate 3.1.17 6S CELL 4→SUIT Spades failure remains a distinct historical incident. Original packet/PNGs and exact source/log hashes/line anchors are retained in [incident-assessment.json](../Evidence/FreeCell/Validation/freecell-drop-anchor-319-20261111/incident-assessment.json).

## Verification and limits

All six fresh application gates passed with 1,908 tests and zero failed, ignored, measured or filtered tests. xSAR 0.16.2 and console 0.8.5 retain historical qualification evidence; no fresh standalone sibling gates are claimed.

Fresh Windows acceptance of 3.1.19 remains pending. Host regressions use retained/synthetic pixels and local TCP/Unix-socket peers. The retained JC frame demonstrates direct child-position refusal, and local peers verify the corrected outgoing packet sequence. This correction does not launch an app, take a new guest capture, send guest input, install, commit, push or publish. A rebuild does not replace an already running process; live acceptance must identify the executing host and capture-server builds.

See [goal](FREE_CELL_goal_20261111J.md), [contract](FreeCell-Drop-Anchor-3.1.19.md) and [verification](Verification-3.1.19.md).

## Code Analysis

Native anchor and path arithmetic remains checked and bounded before probes/input; calibration and image recognition remain authoritative. This application-private coordinate correction preserves sibling source/API/dependency versions and the single-card domain contract. Input authority remains exact-source/run/context owned. Rejected or stale pixels cannot earn cursor credit, and STOP or uncertainty cannot authorise replay. No unsafe Rust or new dependency is introduced. Transport acknowledgement, reconstructed geometry and local-peer tests leave the actual Windows held-card anchor and physical landing as live acceptance limits.
