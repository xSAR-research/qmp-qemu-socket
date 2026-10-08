# Free Cell native Solver evidence

These twenty-one complete PNG files were copied byte-for-byte from Charlie's
supplied 5–7 October 2026 attachments. He captured them through the application.
They are not crops, sparse fixtures, OCR products or resized images. Each frame is 1920×1080 RGBA, including the title bar, guest game toolbar
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
| FC14 | PLAY 3 single five of spades source crosses toolbar; SUIT 4 guide | Visible clipped PLAY 3 source |
| FC15 | Completed game, 2000 XP; New Game visible without LEVEL UP | New Game ready; OK absent |
| FC16 | Four-card PLAY 3 source crosses toolbar; PLAY 6 guide | One visible clipped PLAY 3 run |
| FC17 | Four-card PLAY 1 source crosses toolbar; empty PLAY 5 guide | One visible clipped PLAY 1 run |
| FC18 | Three-card PLAY 1 source closes at toolbar boundary; PLAY 8 guide | One complete PLAY 1 run ending at Y947 |
| FC19 | Three-card PLAY 3 source closes at toolbar boundary; PLAY 1 guide | One complete PLAY 3 run ending at Y947 |
| FC20 | Five-card PLAY 6 source crosses toolbar; eight of diamonds at bottom; PLAY 7 guide | One visible clipped PLAY 6 run |
| FC21 | PLAY 7 eight of diamonds source crosses toolbar; SUIT 3 guide | Visible clipped PLAY 7 source |

FC05's source is CELL 2, not a SUIT card. FC06 and FC07 show the same highlighted
run; they do not form a before/after proof of transfer. The frames' source
positions establish one-click source detection, not card-effect verification.

## Geometry and decision boundary

CELL positions are numbered 1–4 and PLAY columns 1–8, left to right. CELL sources
have priority. PLAY scanning starts from the lowest solid source, with
left-to-right tie-breaking. A run's exterior rails are grouped into one source;
its inner card outlines are not separate candidate moves. The click is inside
the visible bottom highlighted card, above the game toolbar.

The game toolbar begins at native row 947. Gameplay source scanning excludes
that row and everything below it. FC14/FC16/FC17 now supply native single-card
and run overlap cases. A clipped PLAY source needs a closed visible top,
connected opposing rails reaching the cutoff and visible paper at both card
side margins beside its click.
Its exclusive bottom is Y=947, documenting the visible cutoff rather than an
inferred hidden card edge. Source scans do not read the overlaid toolbar. FC18/FC19 supply fully closed
source bottoms whose last crossbar is row946, giving exclusive end947. Rounded
rails terminate before that crossbar; the complete outline supplies authority
without depending on the clipped paper probe.

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
new-game Solver activation and STOP/socket invalidation require live Beast checks.
Charlie reported successful candidate-2 gameplay and win entry; FC15 and its log
expose the previously unsupported skipped-LEVEL-UP branch. Candidate 3 requires
verification both with and without a level change. Candidate 4 also requires live
checks of all three clipped sources, game-start delay and bounded Solver refresh.
The attached 49-line log covers the first toolbar stall; the later restart excerpt
was pasted separately. Still images do not measure deal or transfer durations.

Candidate 5 adds the two boundary captures and a 3000ms game-start default from
Charlie's live timing observation. Its attached log shows supported gameplay
and active Solver throughout the no-HALO wait; Charlie's STOP ended the run.
FC18's supplied PNG is 3,203,290 bytes while its log records a 2,544,532-byte
saved PNG. The fixture hash proves equality to the supplied attachment, not
to that earlier PNG encoding. Pixels are inspected directly at native resolution;
the reason for the encoding difference is unverified. Original-byte capture
and saving code are unchanged.

Candidate 3 of v1.4.0 adds FC20/FC21. Both clips have complete visible source
tops and opposing rails through row946. The previous central 17×17 paper probe
has only 139 of 289 paper pixels because of the eight of diamonds' central
pips; both symmetric side-margin probes have 289 of 289. The clipped presence
check now uses these margins at the same row907, keeping the existing 50%
threshold, source geometry, click and toolbar cutoff. This establishes visible
card presence only; it does not recognise the card or verify a prior move.
Spider LEVEL UP remains unchanged pending its separate log and frame.

FC20/FC21's supplied attachments contain 3,331,493/3,457,697 bytes; their logs
record saved PNGs of 2,645,645/2,743,438 bytes. Fixture hashes prove equality to
the supplied attachments, not those earlier encodings. The reason for these
encoding differences is unknown; exact-byte capture and saving code are unchanged.


## v2.0.0 candidate 4: shifted local Level Up OK

FC22 is the supplied level-102 PNG with OK 19 pixels higher than FC10; FC23
is the supplied level-80 PNG with OK 12 pixels lower. Both have identical
horizontal lettering bounds and keep the existing (959, 812) click inside
the gold button. The fixture files are complete byte-identical attachments.
Only the expected local OK word and body are aligned together; level, medal,
fireworks, modal frame and completed cards do not gate readiness. An isolated
OK still does not establish a game win. Spider retains its offsets 0/-19.
These captures establish local readiness, not live terminal-click success.
