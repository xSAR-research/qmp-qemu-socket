# Free Cell responsiveness — 3.1.18

Status: host-qualified.

## Gesture and request ownership

SINGLE STEP keeps five 500 ms gesture intervals. SINGLE STEP fast uses five 100 ms intervals, and MULTI STEP freezes ABS with five 200 ms intervals. The six inputs are source → LEFT DOWN → source-up45 → native floor midpoint of ORIGINAL source/effective destination → exact destination → LEFT UP. Every newly reached recorded AUTO edge retains its 500 ms animation wait.

Standard SINGLE STEP remains 2.5 seconds of minimum gesture waiting, fast totals 500 ms, and MULTI totals one second, excluding transport, logging, scheduling, captures and AUTO delays. MULTI normalises ABS and 200 ms before queueing and again at worker dispatch. REL remains the separately selected bounded SINGLE STEP motion probe.

## Capture cadence and preview

The first post-transfer wait is min(configured settle, 200 ms). After one eligible endpoint match, the confirmation wait is min(configured settle, 100 ms). Unmatched frames and intermediate AUTO endpoints use the configured retry gap. Two successive fresh exact endpoint matches are required within six captures. The sixth rejection ends immediately. At the default 750 ms configuration, an all-unmatched run reaches its six effect captures after 200, 950, 1700, 2450, 3200 and 3950 ms of intentional waiting. The prior sequence reached them after 750 through 4500 ms. The total observation window is therefore 550 ms shorter before capture/processing time, and the former final 750 ms rejection wait is removed.

Retained 3.1.17 host logs contain 241 acquisition-plus-proof observations with median 141 ms and 95th percentile 202 ms. There were 136 inter-observation gaps of 750–752 ms and 77 post-delivery gaps of 750–751 ms, including the configured settle wait and adjacent host overhead. These observations motivated the caps. Configurations below the caps, including zero, are honoured. The source digest, timestamp pairs, calculation and limitations are retained in the [timing assessment](../Evidence/FreeCell/Validation/freecell-responsiveness-318-20261111/timing-assessment.json). Fresh Windows acceptance remains pending.

Successful playback dispatch releases completed RECON preview ownership for the same context. Each acquired playback frame already enters the bounded latest-frame slot before semantic interpretation, so the preview can show the newest acquired readiness, effect and failure pixels. Pending RECON and stale-context fences remain required. Frames coalesce in one slot. During a held gesture the preview displays the latest acquisition until the next existing capture, without inserting an in-flight capture or capture-only QMP connection.

## Graceful and immediate controls

During active MULTI playback its button becomes STOP AFTER STEP. A direct request is bound to the exact plan and run. The current transfer, two-frame effect proof, recorded AUTO follow-ups and animation waits finish before StepVerified retains the proved cursor for resumption. A request before the next transfer, including one received at its PAUSE checkpoint, prevents that transfer. PAUSE stays independent and requires CONTINUE to proceed. Existing immediate STOP remains available for immediate revocation and release-only recovery.

Proof failure during the current step still ends the live invocation with its original error. Graceful intent never manufactures endpoint credit. An ordinary failure can retain the immutable offline report for separately requested manual UNDO ALL and fresh-52 rebinding. Terminal/probe completion and explicit source/context/STOP revocation retain their retirement policy.

## QMP queries

`query-status` checks whether the VM is running. The five `query-mice` calls on the ABS diagnostic path are the app preflight, the transport inventory, the availability check before selection, the verification after selection, and the fresh routing check immediately before LEFT down. `human-monitor-command` carries HMP `mouse_set <queried ID>`. `mouse_set 3` selects handler ID 3, historically the QEMU HID Tablet in the retained configuration. IDs come from the current inventory. The helper currently reselects even an already-current handler. These replies describe QEMU handlers and routing, while Windows cursor/button state requires separate observation. See [query schema](https://github.com/qemu/qemu/blob/v11.1.3/qapi/ui.json#L767-L813) and [selection implementation](https://github.com/qemu/qemu/blob/v11.1.3/ui/input.c#L540-L587).

## Saved CELL 4 refusal

The retained 3.1.17 packet pyramid-error-1794394350164-1307502-10 shows 6S still in CELL 4 and 5S on the Spades foundation. Its first/latest PNG bytes match (SHA-256 `fb0c593a843e343bd6ba55790986b2224e88fe7aac04dcc100b17c0206a2a73a`). CELL 4 has zero qualifying outline pixels on each of its four required strokes. Gold tint also yields zero bright-paper pixels, so paper absence alone cannot establish an empty cell. The empty-slot refusal correctly prevents cursor 30→31 credit. Five attempted and acknowledged gestures establish delivery counts. The saved frames leave the input-side cause and in-flight Windows button state unresolved. The capsule retains the original PNG, error state/log and incident-assessment.json.

## Verification and limits

All six fresh application gates passed with 1,905 tests and zero failed, ignored, measured or filtered tests. xSAR 0.16.2 and console 0.8.5 retain historical qualification evidence.

Fresh Windows acceptance of 3.1.18 remains pending. Host tests use retained/synthetic pixels and local TCP/Unix-socket peers. Relevant source/destination/AUTO identity proofs, fresh original-52 binding, source ACK, fresh VM/pointer preflight, queued timing and source/run/context/STOP fences govern input. QMP delivery and physical effect have separate evidence. Uncertain non-idempotent gestures are never replayed. No application launch, new guest capture/input, installation, commit, push or registry publication occurred in this correction.

See [goal](FREE_CELL_goal_20261111I.md), [contract](FreeCell-Responsiveness-3.1.18.md) and [verification](Verification-3.1.18.md).

## Code Analysis

Arithmetic remains checked and all native path points are validated before probes/input. The private timing enum uses the existing xSAR Duration API, preserving sibling source/API/dependency versions. STOP AFTER STEP adds exact-run state under the existing route mutex and bypasses the serial worker command queue. A fresh request resets the flag, stale requests fail, and normal proof/cursor ownership remains authoritative. Short waits change cadence rather than the two-frame matching condition. Cancellation is checked between bounded native sleep slices and at semantic/input boundaries. Best-effort preview/log delivery, finite synchronous gesture cancellation latency and concurrent QEMU routing changes retain their practical limits.
