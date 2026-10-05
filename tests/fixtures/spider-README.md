# Spider native evidence

These twenty supplied originals are copied byte-for-byte, with dimensions,
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
| SP-Lower-Toolbar | Easy board; PLAY 5 source continues behind toolbar icons. |
| SP-no-DRAW-PILE | Easy board; DRAW absent, source PLAY 7; lower PLAY 10 box is a dashed destination guide. |

SP16 was not captured because no LEVEL UP occurred in the recorded end sequence.
Charlie confirms its optional OK control is the accepted local control. Tests
reuse the accepted Free Cell OK fixture; no missing Spider frame is fabricated.
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
