# Free Cell native Solver evidence

These thirteen PNG files are complete, byte-identical originals from Charlie's
5 October 2026 captures, taken through the application's manual QMP PNG capture
and exact-byte save. They are not crops, sparse fixtures, OCR products or resized
images. Each frame is 1920×1080 RGBA, including the title bar, guest game toolbar
and Windows taskbar. `freecell-manifest.json` records original filenames and
SHA-256 digests.

## Observed frames

| Evidence | Visible source / scene | Detector result |
|---|---|---|
| FC01 | Initial eight-column deal, Solver off | Gameplay; no source action |
| FC02 | Ace of hearts in PLAY 7; dashed SUIT destination | PLAY 7 source; SUIT guide ignored |
| FC03 | Queen of clubs in PLAY 5; dashed CELL 1 destination | PLAY 5 source; CELL guide ignored |
| FC04 | Ten of hearts in PLAY 5; dashed CELL 2 destination | PLAY 5 source; CELL guide ignored |
| FC05 | Ten of hearts in CELL 2; dashed PLAY 2 destination | CELL 2 source |
| FC06 | Jack of spades / ten of hearts in PLAY 2 | One connected PLAY 2 source |
| FC07 | Same two-card PLAY 2 recommendation as FC06 | One connected source, not two moves |
| FC08 | Six of spades in PLAY 2; dashed PLAY 5 destination | PLAY 2 source |
| FC09 | Congratulations / XP score counting | Terminal panel, not gameplay |
| FC10 | Level Up, OK visible | Expected terminal OK stage |
| FC11 | Congratulations, New Game visible | Expected terminal New Game stage |
| FC12 | New Game chooser, Play visible | Expected terminal Play stage |
| FC13 | Fresh deal, Solver off | Gameplay; activate Solver before source play |

FC05's source is CELL 2, not a SUIT card. FC06 and FC07 show the same highlighted
run; they do not form a before/after proof of transfer. The frames' source
positions establish one-click source detection, not card-effect verification.

## Geometry and decision boundary

CELL positions are numbered 1–4 and PLAY columns 1–8, left to right. CELL sources
have priority. PLAY scanning starts from the lowest solid source, with
left-to-right tie-breaking. A run's exterior rails are grouped into one source;
its inner card outlines are not separate candidate moves. The click is inside
the bottom highlighted card, above the game toolbar.

The game toolbar begins at native row 947. Gameplay source scanning excludes
that row and everything below it. FC14, the requested original showing a run
crossing the toolbar, was not supplied. Candidate 2 requires a complete visible
source outline above that boundary and does not infer a clipped bottom card.
An unresolved clipped source receives input-free recaptures and then a guarded
stop with its latest frame and log.

SUIT cards are not sources in this guest Free Cell contract. There is no Draw,
Recycle or Solve action. Eligible cards can automatically move to SUIT; absent
HALOs therefore require bounded fresh observations, never speculative input.

The detector uses broad warm-gold relationships and outline geometry to separate
solid sources from dark dashed destinations. It does not compare consecutive
card pixels, recognise ranks, track consumed slots or prove a previous effect.
Terminal progression is tested separately, using the expected button regions
instead of level-dependent decoration or whole-screen artwork matching.

## Evidence limits

The original PNGs provide positive frame observations, not runtime input tests
or settle-time measurements. Automatic transfers, complete game-win progression,
new-game Solver activation and STOP/socket invalidation still require Charlie's
Beast checks. Native FC14 remains a separate outstanding evidence case.
