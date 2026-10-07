# Spider native evidence

These twenty-three supplied originals are copied byte-for-byte, with dimensions,
filenames, sizes and SHA-256 hashes in `spider-manifest.json`. Each is a
1920x1080 RGBA PNG containing the guest title bar and Windows taskbar.

| ID | Observation |
|---|---|
| SP01 | Fresh Grandmaster / four-suit board, Solver inactive. |
| SP02 | Solver active; solid PLAY 2 source and dashed PLAY 4 destination. |
| SP03 | PLAY 10 connected multi-card source; each rounded outline belongs to one recommended block. |
| SP04 | High solitary PLAY 5 source. |
| SP05 | Empty PLAY 5; source is PLAY 3, not the vacant column. |
| SP06 | PLAY 4 source directed into empty PLAY 5. |
| SP07 | PLAY 1 source reaches the toolbar cutoff. |
| SP08 | DRAW PILE solid source; D deals one card to each column. |
| SP09 | Settled result of SP08's one draw, after Charlie's observed 1–2 second wait; current source PLAY 8. |
| SP10 | Long PLAY 3 source extends into the toolbar. |
| SP11 | A completed hearts run has packed automatically; first collapsed packet, current source PLAY 4. |
| SP12 | Two collapsed packets; source PLAY 10. |
| SP13 | Heavy compaction in PLAY 1; current source PLAY 5. |
| SP14 | Compressed PLAY 1 source ends above the collapsed packets. |
| SP15 | Congratulations with score-counting skip caption; Easy deck solved. |
| SP17 | Congratulations with New Game control. |
| SP18 | Spider setup / Grandmaster four suits; Play is 100 px lower than Free Cell. |
| SP19 | Fresh dealt Grandmaster board with Solver inactive. |
| SP20 | Standard Level Up at Spider Grandmaster level 105; OK caption is 19 px above FC10, with the existing click still inside its button. |
| SP21 | Queen before exposing the Jack; solid PLAY 6 source rows 145–349, click (1045,309); actual large cursor at (21,921). |
| SP22 | Reconstructed Jack state; solid PLAY 6 source rows 123–327, with the cursor moved to left felt. |
| SP-Lower-Toolbar | Easy board; PLAY 5 source continues behind toolbar icons. |
| SP-no-DRAW-PILE | Easy board; DRAW absent, source PLAY 7; lower PLAY 10 box is a dashed destination guide. |

SP16 was not captured because no LEVEL UP occurred in the recorded end sequence.
SP20 now supplies the original Spider Level Up frame. Tests cover its local OK
and the accepted Free Cell OK fixture used for the original Spider sequence.
The controller checks either evidenced local OK location after a known win;
level, rank, banner colour and medal artwork do not gate the click. SP20's
existing click point remains inside the gold button, so input coordinates and
timings do not change. No missing Spider frame is fabricated. SP20's supplied attachment is 1,732,335
bytes; the session log reports a saved capture of 1,109,762 bytes. The fixture
is byte-identical to the supplied attachment; identity to that logged saved
PNG is unverified.
Easy and Grandmaster captures establish the observed geometry and controls,
not a contiguous same-difficulty game. The controller does not change difficulty.

PLAY is numbered 1–10 from left to right. COLLAPSED SUITS is display-only;
completed runs pack automatically. Stock absence is not win evidence. Source
scans and clicks stay above Y947, using visible complete or clipped outlines.
The dashed destination grants no input. Connected source outlines form one
logical action, followed by editable settle and a fresh capture. No card ranks,
recipient identities or before/after pixel effect differences are checked.

The DRAW contract and 1–2 second timing come from Charlie's direct confirmation.
The initial 2000 ms DRAW wait is editable independently of card moves, input-free
observations and new-game dealing. Pixel regressions cannot establish QMP
input delivery, animation duration, all unseen layouts or live completion.
The delivery review distinguishes the executed Python rehearsal from native
Rust tests and Beast gameplay.

SP21 and SP22 are byte-identical supplied originals; their original names,
byte counts and SHA-256 provenance are recorded in `spider-manifest.json`.
The exact failed runtime PNG was not captured. Charlie used Undo to reconstruct
the same Queen-to-Jack board four times, so these frames establish the source
geometry and a reproducible cursor diagnostic, not identity to the failed frame.

The focused regression copies only measured neutral black/white pixels from
SP21's cursor rectangle (20,920,34,47), with hotspot (21,921), into decoded SP22
test memory at the preceding Queen click (1045,309). This lowers gold coverage
on Jack bottom-bar rows 322–326 from 103 pixels to 91, 90, 89, 88 and 87,
below the unchanged 95-pixel crossbar requirement. The surviving inner bars
at rows 316/317 lie outside the bottom search window 318–333. The source rails,
scene and Solver checks remain present. The original clear Jack frame still
recognises PLAY 6 rows 123–327. No diagnostic PNG is generated or edited.
The native Rust regressions were added but could not be executed in this
environment; the measured decoded-pixel rehearsal is separate from a live
Beast gameplay verification.

The pointer park is (20,500), outside the existing scene and source probes.
Its full 33x46 cursor bounds are clear felt on all nineteen current gameplay
fixtures, including SP21/SP22. A second in-memory cursor-overlay regression
requires parking to preserve scene recognition, Solver state and the original
source prediction on every one of those frames. The centre of the existing
left scene probe, (82,362), is unsuitable: the actual neutral cursor pixels
cover 64 of its 576 pixels, lowering felt coverage to 88.89% below the unchanged
90% gameplay requirement.
