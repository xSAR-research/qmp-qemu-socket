# Free Cell responsiveness correction — 3.1.18

Status: host-qualified. Application-only scope uses xSAR 0.16.2 and console 0.8.5.

## Approved outcome

SINGLE STEP keeps five 500 ms gesture intervals. SINGLE STEP fast uses five 100 ms intervals, and MULTI STEP freezes ABS with five 200 ms intervals. The six inputs are source → LEFT DOWN → source-up45 → native floor midpoint of ORIGINAL source/effective destination → exact destination → LEFT UP. Every newly reached recorded AUTO edge retains its 500 ms animation wait.

The first post-transfer wait is min(configured settle, 200 ms). After one eligible endpoint match, the confirmation wait is min(configured settle, 100 ms). Unmatched frames and intermediate AUTO endpoints use the configured retry gap. Two successive fresh exact endpoint matches are required within six captures. The sixth rejection ends immediately. At the default 750 ms configuration, an all-unmatched run reaches its six effect captures after 200, 950, 1700, 2450, 3200 and 3950 ms of intentional waiting. The prior sequence reached them after 750 through 4500 ms. The total observation window is therefore 550 ms shorter before capture/processing time, and the former final 750 ms rejection wait is removed.

During active MULTI playback its button becomes STOP AFTER STEP. A direct request is bound to the exact plan and run. The current transfer, two-frame effect proof, recorded AUTO follow-ups and animation waits finish before StepVerified retains the proved cursor for resumption. A request before the next transfer, including one received at its PAUSE checkpoint, prevents that transfer. PAUSE stays independent and requires CONTINUE to proceed. Existing immediate STOP remains available for immediate revocation and release-only recovery.

Successful playback dispatch releases completed RECON preview ownership for the same context. Each acquired playback frame already enters the bounded latest-frame slot before semantic interpretation, so the preview can show the newest acquired readiness, effect and failure pixels. Pending RECON and stale-context fences remain required. Frames coalesce in one slot. During a held gesture the preview displays the latest acquisition until the next existing capture, without inserting an in-flight capture or capture-only QMP connection.

## Retained failure and evidence

The retained 3.1.17 packet pyramid-error-1794394350164-1307502-10 shows 6S still in CELL 4 and 5S on the Spades foundation. Its first/latest PNG bytes match (SHA-256 `fb0c593a843e343bd6ba55790986b2224e88fe7aac04dcc100b17c0206a2a73a`). CELL 4 has zero qualifying outline pixels on each of its four required strokes. Gold tint also yields zero bright-paper pixels, so paper absence alone cannot establish an empty cell. The empty-slot refusal correctly prevents cursor 30→31 credit. Five attempted and acknowledged gestures establish delivery counts. The saved frames leave the input-side cause and in-flight Windows button state unresolved. The capsule retains the original PNG, error state/log and incident-assessment.json.

## Acceptance

All six fresh application gates passed with 1,905 tests and zero failed, ignored, measured or filtered tests. xSAR 0.16.2 and console 0.8.5 retain historical qualification evidence.

Fresh Windows acceptance of 3.1.18 remains pending. Host tests use retained/synthetic pixels and local TCP/Unix-socket peers. Relevant source/destination/AUTO identity proofs, fresh original-52 binding, source ACK, fresh VM/pointer preflight, queued timing and source/run/context/STOP fences govern input. QMP delivery and physical effect have separate evidence. Uncertain non-idempotent gestures are never replayed. No application launch, new guest capture/input, installation, commit, push or registry publication occurred in this correction.

See [goal](FREE_CELL_goal_20261111I.md), [contract](FreeCell-Responsiveness-3.1.18.md) and [verification](Verification-3.1.18.md).
