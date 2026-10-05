# FreeCell Gate 1: native evidence intake

## Current preparation — 4 October 2026

Version 1.3.0 candidate 1 is based on accepted and pushed v1.2.6 candidate 11
at `146fe173f6e123a92e2eadf518523715d042cd66`. It adds Free Cell as a read-only
calibration mode with shared capture and independent future Params values.
No HALO detector, source click, Solver activation or win/restart automation is
enabled. The three attached PNGs from this date were available locally and
visually inspected for intake: the initial Solver-off board, Solver-on PLAY-to-SUIT
recommendation and PLAY-to-CELL recommendation. They are observations, not a
calibrated runtime detector. The fresh FC01–FC13 sequence below has not yet been
received. This preparation does not replace missing frames with inferred pixels.

The agreed board names are **CELL 1–4**, upper left; **PLAY 1–8**, tableau;
and **SUIT 1–4**, upper right. Number each area from left to right. The intended
action is a single click on the guest Solver's highlighted CELL or PLAY source,
including a whole run. Automatic transfers into SUIT can follow and can pause
for another HALO. A dashed landing guide is not a source. There is no Draw,
Recycle or Solve operation to copy from another game type.

Future independent settle/re-observation settings start at **750/1000 ms**,
editable from 0 to 5000 ms. Actions per Multi-Step starts at **0** for continuous
operation when execution is later enabled; finite settings are bounded at
1–10000. These are initial settings rather than measured animation durations.
They cannot activate input in this calibration-only candidate.


## Fresh FC01–FC13 capture sequence

Start a clean new Free Cell game in the existing full guest layout, retaining
1920-by-1080 resolution and 100% display/text scaling. Operate the guest manually;
the preparation candidate captures but does not play. For each requested state,
use the application's **Capture PNG**, inspect its preview, then **Save PNG**.
Attach those original full-frame PNGs. Save does not make a second screendump.
Avoid cropping, preview exports or a host screenshot for native calibration.

Use the labels below in the saved filenames. Each fresh file's timestamp
distinguishes it from the historical FC01–FC12 sequence. Preserve a before/action/
after order for the paired cases; do not insert another gameplay move between
each pair. Note automatic SUIT transfers rather than assuming each image
differs by exactly one card. No delay is inferred from filename timestamps.

| Fresh ID | Capture state |
| --- | --- |
| FC01 | Initial new board, guest Solver off. |
| FC02 | Solver on; highlighted PLAY source recommended to SUIT, before its source click. |
| FC03 | Highlighted PLAY source recommended to CELL, before its source click. |
| FC04 | Settled result of FC03; source card has arrived in CELL, including any automatic SUIT transfer. |
| FC05 | Highlighted CELL source before clicking it; prefer a recommendation back to PLAY. |
| FC06 | Settled result of FC05, without an intervening manual gameplay move. |
| FC07 | Highlighted multi-card PLAY run before clicking any card in that source block. |
| FC08 | Settled result of FC07, including automatic transfers if present. |
| FC09 | GAME WIN / XP counting screen before manually speeding up the count. |
| FC10 | LEVEL UP screen with OK ready. |
| FC11 | Congratulations screen with New Game ready. |
| FC12 | New Game selection screen with Play ready. |
| FC13 | Fresh board after Play, Solver still off. |
| FC14, optional | A PLAY source HALO reaching or continuing behind the bottom game toolbar. |

If automatic play bypasses FC02's source or a terminal stage does not appear,
record that observation; do not force a move or fabricate a missing frame.
Use the next available recommended PLAY-to-SUIT source for FC02. Record the
selected difficulty and any automatic transfers with the images. An optional
unsupported-source example can be obtained later if the Solver offers a class
not in this sequence; unsupported actions stay disabled until evidenced.

These captures will establish native source geometry, solid-source versus
dashed-destination discrimination, automatic-transfer observations and the
one-board win/restart controls. The future PLAY detector will exclude toolbar
pixels, with the boundary measured from the fresh Free Cell frames. No card-rank
recognition, changed-pixel move proof or large fixture quota is required.
Charlie will approve the evidenced action slice before runtime input is enabled.


## Historical intake — 1 October 2026

The following original intake records an earlier screenshot series. Its pixel
measurements and former next-gate limits are historical, not proof that a current
detector or runtime action is implemented. Those earlier raw files were not
available for reinspection in this cycle. Fresh FC labels above are a new series.

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
