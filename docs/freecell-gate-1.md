# FreeCell Gate 1: native evidence intake

Prepared 1 October 2026, Australia/Brisbane. This is read-only intake alongside
Klondike v1.2.3 candidate 3. No FreeCell mode or runtime input is enabled.
Implementation follows acceptance of the Klondike correction and Charlie's
promotion/new-cycle authorisation. No remote freshness polling was performed.

All twelve supplied PNGs were inspected visually as native 1920×1080 RGBA frames.
The current theme, guest scaling and toolbar are retained. Expert is visible.
The timestamps do not establish animation durations or one action per frame.

## Observed sequence

| ID | Pixel observation |
|---|---|
| FC01 | Initial eight-column deal; four empty free cells and foundations; Solver off. |
| FC02 | A-hearts source in tableau column 6, guided to foundation 1. |
| FC03 | A-hearts in foundation 1; 6-diamonds source in column 5, guided to free cell 1. |
| FC04 | 6-diamonds in free cell 1; 7-diamonds source in column 1, guided below 8-spades in column 4. |
| FC05 | 5-clubs source in free cell 3, guided to foundation 2. |
| FC06 | 5-clubs in foundation 2; 10-spades source in column 3, guided to empty free cell 3. |
| FC07 | K-hearts/Q-spades run in column 1, guided to empty tableau column 3. |
| FC08 | Run now in column 3; 7-hearts source in free cell 1, guided to foundation 1. |
| FC09 | Completed-game Congratulations reward counting, with Click anywhere to skip. |
| FC10 | FreeCell Level Up with its low OK control. |
| FC11 | Congratulations with New Game/Home; a taskbar thumbnail overlaps part of the lower panel. |
| FC12 | FreeCell difficulty selector retaining Expert, with Play/Cancel. |

The FC06 prose comparison to FC04 describes a free-cell transfer in the pixels,
matching FC03's action class. FC07's individually outlined adjacent cards form
one actionable run, confirmed by Charlie's one-click source contract; drag and
destination clicks are unnecessary. Distinct independent source blocks remain
uncalibrated and must stop rather than choose speculatively.

## Geometry and controls

Tableau card X origins are approximately 218, 410, 603, 795, 988, 1180, 1372 and
1565, with initial row 331 and approximately 53/54-pixel overlaps. Free-cell X
origins are 165, 357, 550 and 742; foundation origins are 1040, 1233, 1425 and
1618. The common upper card row is 112. Full faces are approximately 137×183
pixels, distinct from the exterior source rails and Klondike's calibration.
These are intake measurements, not a completed runtime detector.

FC07's lower gold edge spans about rows 678–682; its internal card boundary lies
near 484–485. A detector must group the whole run while rejecting dashed landing
guides. Do not copy Klondike's seven-column geometry or consumed-slot assumptions.

| Control | Evidenced interior / existing point |
|---|---|
| Solver | Existing 602,977 lies inside the visible toolbar control. |
| Level Up OK | Gold interior about x840–1075, y776–845; existing 960,795 is inside. |
| New Game | Interior about x632–937, y815–887; existing 792,840 is unobscured. |
| Play | Interior about x603–812, y726–793; representative 709,761 is inside. |

FreeCell Play is much higher than Klondike's 705,860. That coordinate must remain
mode-owned. FC11's taskbar thumbnail occupies approximately x808–1025, y863–1020;
do not use its contaminated pixels as dialog artwork or complete-game proof.

## Authorised first action slice and limits

Charlie confirms one click on the highlighted source transfers a card or whole
run. Evidence supports tableau-to-foundation, tableau-to-free-cell,
tableau-to-tableau single/run and free-cell-to-foundation. Free-cell-to-tableau
and foundation-to-tableau have no example in this set; obtain those incrementally
before enabling them. There is no stock, Draw shortcut or Solve button to inherit.

Automatic foundation transfers may follow a gameplay move and can pause for a
further highlighted move. FreeCell therefore needs resultant-board interpretation
and bounded input-free observation that permits more than two changed regions.
Neither QMP acknowledgement nor an arbitrary fresh HALO proves completion.

FC09–FC12 provide positive single-board terminal/restart scenes. The captured run
levels up, but does not establish that every future win must level up. Positive
FreeCell completed-board and dialog context should own completion; black absence
alone is insufficient. Timings need independent editable action/automatic-transfer
and recapture settings; no duration is inferred from still images.

## Next implementation gate

After Klondike acceptance/promotion, establish the newly pushed base once and agree
FreeCell candidate numbering. Add a typed mode/source target, canonical one-click
actions, a mode-owned detector/result/completion/terminal policy and a focused
worker branch. Reuse QMP, capture, cancellation, logging and exact-byte PNG saving.
Avoid a universal solver framework or independent rank/search implementation.

No blocking input ambiguity remains for the evidenced first action slice. Unknown
scenes, unsupported source classes, toolbar-crossing runs and uncertain input stop
with the newest usable frame and logs. Live calibration, automatic-transfer timing,
effect verification, terminal controls and STOP remain Beast validation work.
