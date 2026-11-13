# Free Cell playback correction — 3.1.17

Application 3.1.17 and xSAR 0.16.2 implement the approved playback correction. All 51 fresh paired host gates passed with 1,896 application, 381 library and 80 console tests. The full suites report zero failed, ignored, measured or filtered tests.

## Movement and timing

Let S be the freshly proved source and D the effective destination. PLAY uses its observed printed corner. CELL and SUIT use the canonical target slot centre.

| Input | Position or button |
| --- | --- |
| 1 | Source ABS S, acknowledged before continuing |
| 2 | LEFT down |
| 3 | `(S.x, S.y - 45)` |
| 4 | `(floor((S.x + D.x) / 2), floor((S.y + D.y) / 2))` |
| 5 | Exact destination D |
| 6 | LEFT up |

The midpoint widens each original native coordinate sum to i64 before division. All coordinates are validated and independently floor-mapped before operation I/O. Source y below 45 refuses. The final CELL/SUIT point must remain inside its target slot.

SINGLE STEP and MULTI STEP use a minimum 500 ms across each of the five gesture intervals. The fast SINGLE STEP uses a zero interval. Source acknowledgements, routing queries, socket work, cooperative control checks and logging contribute elapsed time. Zero interval removes deliberate gesture sleeps.

MULTI STEP repeats this same absolute gesture after each verified card play. The queued request freezes its absolute mode and half-second interval. SINGLE STEP returns after one explicit transfer and its recorded AUTO follow-ups are verified.

## Recorded AUTO animation

Each newly reached recorded `is_auto` edge incurs an additional 500 ms wait. This includes explicitly transferred AUTO records, observed readiness prefixes and recorded AUTO follow-ups. A resumed cursor receives waits only for newly reached records. The route checks STOP, cooperative pause, context and ownership before and after each wait; the native wait also checks them between bounded sleep slices before any further input. The fast gesture also incurs these AUTO animation waits.

Recorded AUTO observations require the existing exact physical endpoint evidence. Relevant pile proofs include every changed pile across the supplied recorded AUTO prefix. The waiting policy supplies no cursor credit or alternate route.

## Authority and delivery

The additive xSAR `diagnostic_drag_path_with_interval` accepts exactly three or four caller-supplied held points and a Duration interval. The existing public timing enum and methods retain their contracts. The application supplies three points for this correction.

The source matching acknowledgement and fresh selected-pointer verification precede DOWN. UP precedes collection of all five held-phase replies. Existing targeted release recovery retains the original failure and sends safety releases after possible DOWN. Source, plan, context, run and STOP checks govern every phase. Two successive fresh exact final endpoints within six observations govern accepted route progress.

The operator reports successful 3.1.16 plays with both buttons and observes that its first held source-up movement moves the pointer while the card stays at its source. That observation motivates removing the destination-up movement and applying the successful route continuously. It does not measure Windows button state. Fresh Windows acceptance of 3.1.17 remains pending.
