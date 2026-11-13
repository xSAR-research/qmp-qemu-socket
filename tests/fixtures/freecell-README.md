# Free Cell native Solver evidence

## Current independent identity and live binding boundary

RECON, live initial binding and exposed-pile position proofs now share the
foreground card-reading engine (`pyramid_recon::vision::foreground_card` via
`game_controls`), replacing the retired 47-template 1-bit matcher while keeping
local contrast, coherent ink, competing-label quality/separation and
unique-inventory checks without expected-card shortcuts or relaxed tolerances.
Live binding additionally requires a valid native frame, positive Menu/Back
controls, Solver exclusion and actual initially empty top slots. The caller
separately compares the freshly read physical deal with the frozen source; a
cached deal does not authenticate UNDO ALL or a new board.

Paper colour, gold decoration, complete card borders/rails and lower face
artwork no longer add gameplay vetoes. Historical edge-envelope measurements
below remain provenance for retired checks. Tests now require readable identities
to survive those decorative differences and still refuse damaged, ambiguous,
mixed-colour or duplicated identities, invalid frames, unsupported controls,
Solver/real overlays and cancellation. Saved-frame results establish host
recognition; they do not establish guest input delivery or live acceptance.

## Native sixes incident regression

`freecell-sixes-decorative-20261111.png` is the complete byte-identical
`last-observed.png` from
`/home/charlie/tmp/pyramid-error-1794339214682-1138131-2/`. It is3,623,060 bytes,
1920×1080 RGBA, SHA256
`3621b103590b6d5b1c5c52fc7d2b9fba48ff6c1f0800f277e9d8336c01bd4d61`.
The retained image shows “Sixes cleared0/3”, a highlighted6H at PLAY3 depth1,
and the original initial tableau. Its52 identities were read independently
from the saved image, in physical top-to-bottom order:

| PLAY | Printed cards |
|---|---|
| 1 | 8S,3C,TD,7D,7C,KC,4H |
| 2 | 8C,9S,5C,JD,9H,KD,4C |
| 3 | 6H,9D,8H,AD,JS,QD,QH |
| 4 | 3H,KS,7H,6D,3S,4S,5S |
| 5 | 9C,6C,5H,TS,TH,TC |
| 6 | JC,3D,AH,2H,8D,JH |
| 7 | AS,7S,2D,6S,QC,2S |
| 8 | QS,2C,AC,5D,KH,4D |

The authentic frame must acquire and freshly bind the same complete physical
inventory without a new glyph calibration or relaxed identity threshold.
Explicit synthetic in-memory variants also change a PLAY3 outer decorative
rail pixel to red237 and238; both retain the same identities and exposed-card
positions. The original packet remains unchanged.

The additional `freecell-challenge-20261107.png` is a byte-for-byte retained later
operator screenshot (SHA256
`5398456b9f949e548241b4f555fafca3f5fcf356d57bd73a7c726a58c8816059`). It shows a
red “Clear the 9♦” banner and a highlighted card border. It is **not** an original
failed RECON frame and establishes no cause for those missing original pixels.
The3.1.3 tests used its objective pixels as unresolved evidence; its original
highlighted card failed the now-retired strict card-edge geometry check. A separately
labelled in-memory derivative places only those objective pixels on native FC01
to verify inventory acquisition and objective retention without claiming captured
gameplay, recognised challenge semantics or guest acceptance.

## Read-only RECON regressions in 3.1.4

`freecell-challenge-highlight-20261109.png` is the complete byte-identical acquired
frame from
`/home/charlie/tmp/pyramid-error-1794150459483-380772-0/first-unmatched.png`.
It is 3,615,634 bytes, 1920×1080 RGBA, SHA256
`35dbe33e2e9f05ab593dc0334094cd5acae026e1c5d619d8ec46f3b65adf5636`.
The original 3.1.3 packet records RECON request 1, TCP protocol 3, session 11,
transport request 1, frame 1706 and display generation 1. Its first refusal was
the gold top border of PLAY 2 depth 1. The original packet remains unchanged.

The frame shows a “Clear the 9♦” objective and a highlighted 9♦. These manually
read identities are fixed physical columns, each listed from top to bottom:

| PLAY | Printed cards |
|---|---|
| 1 | TD, 9C, JC, 2S, 9S, KD, KH |
| 2 | 9D, 2H, 2D, 6H, TH, 8S, 3H |
| 3 | 5S, JS, JD, KS, JH, 6D, 7C |
| 4 | QS, 7H, AS, 7S, TC, TS, 7D |
| 5 | AD, KC, QD, 4C, 5D, AC |
| 6 | 3D, 8H, 5H, 9H, QH, AH |
| 7 | 4H, 5C, 2C, 8C, QC, 8D |
| 8 | 6S, 4S, 4D, 3C, 6C, 3S |

The native highlighted rank and suit crops have a 90th-percentile minimum RGB
channel value of 233; the dominant bright rank paper is RGB (255,250,233).
With the existing relative ink contrast and ±1 recrops, the best rank differences
are 9 = 7 and 5 = 59; the best suit differences are diamonds = 2 and hearts = 53.
These measurements explain RECON's use of local glyph contrast without a separate
paper-whiteness veto; rank/suit quality and separation limits and the reference
glyph bank remain unchanged. Fresh live initial binding now uses that same
printed identity evidence without a separate card-border veto. An acquired
challenge retains its objective honestly; acquisition alone grants no input
authority or recognised challenge semantics.

`freecell-ambiguous-rank-20261109.png` is an explicitly cropped native evidence
derivative, **not** a complete acquired frame. It retains the 28×25 RGBA rectangle
at x=604, y=549 from
`/home/charlie/tmp/pyramid-error-1794149415703-375927-0/first-unmatched.png`,
whose original SHA256 is
`98c956b33c957a0a08d75a7c2ec8fb16837185798dddf9d99287f71c95dfd24e`.
The derivative is 1,374 bytes, SHA256
`faef5a47ac4f2185d9415b1c145713a2466511e80cd644c7c3465463ff7cb908`.
It includes every ±1 rank recrop around that original PLAY 3 depth 5 corner.
Before the 3.1.21 supplemental native 9 calibration, an explicitly synthetic
in-memory placement onto FC01 scored 9 = 34 versus 5 = 50 and correctly refused
the separation of 16. With the augmented bank it scores 9 = 4 versus 5 = 50.
It is now positive when placed on FC01's actual 9D, a duplicate-identity
negative on its JD and a colour-disagreement negative on its TC. Its filename
records its historical role; the retained PNG bytes remain unchanged. The
separate synthetic 6/5 tie below preserves a real ambiguity regression against
the current bank without using inventory elimination.

Other 3.1.4 derivatives are made only in memory and are explicitly synthetic:
changing all pixels outside the objective and the complete rank/suit recrop
unions must preserve the same acquired deal. Altered top slots, felt, final-face
borders, below-stack backgrounds and Solver decoration cannot veto read-only
RECON. Current live binding additionally checks actual scene controls, Solver
exclusion and initial top-slot occupancy. Readable non-white rank crops and
decorative border/face changes are positive live-binding and position tests.
Damage to required glyphs, insufficient contrast, duplicate identities,
malformed frames and cancellation remain refused. These fixtures qualify host
recognition and acquisition behaviour, not fresh guest input or live acceptance.

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

Version 2.9.179 also checks the four foundation stacks after CELL and PLAY.
Only a complete closed solid source outline authorises one foundation click;
dashed SUIT destination guides remain ignored. Measured foundation origins are
1041, 1233, 1425 and 1618 at native Y112, with centre clicks at Y203.
There is no Draw, Recycle or Solve action. Eligible cards can automatically move to SUIT; absent
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

## Foundation source coverage in 2.9.179

FC01/FC02/FC05/FC08 establish native foundation geometry. All-four source and
controller tests translate the unchanged FC05 CELL2 solid-HALO patch into those
positions in memory. These are explicitly synthetic source appearances; none of
FC01–FC23 records a genuine Free Cell foundation-source HALO. Native dashed SUIT
guides remain negative evidence. The controller tests cover foundation-to-PLAY
continuation, STOP and uncertain delivery without adding card-effect gates. Fresh
live acceptance of the rare foundation recommendation remains pending.


## Native sevens challenge regression in 3.1.8

`freecell-sevens-halo-20261109.png` is the complete byte-identical saved
`last-observed.png` from the operator's 3.1.7 execution failure packet
`/home/charlie/tmp/pyramid-error-1794229477346-672085-5/`.
It is 3,637,114 bytes, 1920×1080 RGBA, SHA256
`fef4cdd0a498b4cefb968b07980535d6f7dc6fd5b77052211332dfa196b8face`.
The failure was before input: attempted 0, acknowledged 0, capture serial 1,
initial physical binding false. The old `BelowStack` rectangle at
x=218, y=835, width=137, height=100 intersects the genuine final 7C's glow.
That rectangle's equality to bundled felt was not card-identity evidence.

The native frame shows “Sevens cleared 0/2” and four highlighted sevens.
These identities are independently read from the saved pixels, in physical
column order from top to bottom, without completing inventory by elimination:

| PLAY | Printed cards |
|---|---|
| 1 | 4S, KH, 5S, 3H, 7H, 9D, 7C |
| 2 | 3C, AS, KD, AD, KC, 8C, TH |
| 3 | 7S, 6C, 7D, TD, JC, 6S, AC |
| 4 | 5D, 5C, 2H, 4C, 2S, TS, 2C |
| 5 | JH, JS, 4D, QC, QH, 5H |
| 6 | AH, 8S, 2D, 3D, 9C, QD |
| 7 | 4H, TC, QS, 6D, 3S, JD |
| 8 | 8H, 8D, 9S, KS, 9H, 6H |

The four native gold faces begin at (218,543), (218,650), (603,331) and
(603,437). The former geometry policy measured their top edges and exposed
rails, including the final7C lower edge at y=832. Those gold envelopes are
historical evidence; current binding and position matching use printed glyphs
without a decorative border or rail veto.

The incident's ordinary ten of hearts has one genuine neutral rail pixel at
(410,667), RGB (200,218,221), whose channel spread is 21. Complete exposed
neutral-rail measurements cover 5,386 pixels in this frame; their maximum is
21, compared with maxima 16 on FC01, 15 on FC13, and 20 on the earlier
highlighted challenge. The retired rail-only spread bound was21. Current
positive derivatives establish that a spread22 rail pixel and interrupted
lower borders cannot veto intact printed rank/suit evidence. Glyph identity
quality and separation thresholds remain unchanged.

Synthetic derivatives are constructed only in memory. Repainting advisory
canvas, top-slot interior texture and below-stack areas preserves live binding
when native control glyphs, actual empty slots and exact printed identities
remain present. Damaged game controls, Solver, occupied initial slots, extra
exposed cards and unknown or mixed identities still refuse at their respective
boundaries. Interrupted native card edges are positive decorative contrasts. Off-card decoration supplies
no input authority and is not a universal modal detector. The supported native
card artwork, objective policy, STOP and fresh effect checks remain separate
requirements. This saved-frame regression establishes host recognition, not a
fresh guest drag or full-route acceptance.


## Native normal6D and highlighted5H regressions in 3.1.10

The original packets were moved into `/home/charlie/tmp/_HOLDER_/`; the retained
filenames below preserve their captured bytes and independently supplied digests.
Neither original packet is changed.

| Fixture | Captured source | Bytes | SHA256 |
|---|---|---:|---|
| `freecell-normal-six-20261110.png` | `pyramid-error-1794300141350-832580-2/first-unmatched.png` | 3,649,494 | `b5fab09b41f9b6fe5e1bd6b1034390741958c5dd439a476f28f99c3b05fff7c1` |
| `freecell-gold-five-heart-20261110.png` | `pyramid-error-1794302014997-832580-6/last-observed.png` | 3,698,608 | `858731b6214a7049f1ec310a2e90f001eeb872966e761f9dbd89c65874d6262e` |

Both are complete 1920×1080 RGBA native frames. The first shows an ordinary
Expert board; the second shows the “Clear the 5♥” challenge and its highlighted
5H at PLAY1 depth2. Printed identities were read independently from the saved
pixels, in original top-to-bottom column order:

| PLAY | Normal board | Highlighted5H challenge |
|---|---|---|
| 1 | QH, TD, 9D, JC, 2H, JH, 5S | 5D, 5H, 7H, 8H, KS, 3H, KC |
| 2 | 7S, AS, 8C, KH, AH, QS, QC | 6D, TC, TS, AD, 6C, 6H, QH |
| 3 | 4H, KC, 3S, 9S, 5H, 7H, 4C | QC, TH, 5C, 7S, 4D, JD, QS |
| 4 | 5C, 9C, 4D, TS, 4S, 7C, KD | 7D, 9C, 8S, 2C, 4S, AS, JS |
| 5 | TC, 6C, KS, 8S, JD, AD | 3S, 9H, QD, 5S, 2D, JH |
| 6 | 2S, 9H, 3D, 8D, 2C, 7D | 7C, 9D, 8C, KD, KH, 2S |
| 7 | TH, AC, 6S, QD, 8H, 6D | JC, 9S, 6S, 4H, 3D, 8D |
| 8 | 3C, 3H, 6H, 5D, JS, 2D | AH, 2H, 4C, 3C, TD, AC |

`freecell-normal-six-rank-corner-20261110.png` is an explicitly cropped native
calibration derivative, not a complete captured frame or a held-out rank test.
Its exact source rectangle is x=1373, y=597, width=30, height=48 in the normal
fixture above. It is 2,422 bytes, SHA256
`6c938d555b40d529f38a951e0cd1d54bf82dc36fffb5b4c3ebf8915f46e81c46`.
The complete rank rectangle inside it is x=2, y=7, width=26, height=23, and is
independently labelled6 from the printed6D. Only this rank is added to the bank.
Its supplementary template repeats the original FC13's6D suit bitmap exactly;
the derivative's suit supplies no additional suit calibration.

The original44-strip bank scores the actual normal6D as6=21 versus5=40, a gap
of19 below the unchanged20 requirement. The added native appearance scores6=0
versus5=40. It competes with every distinct same-colour rank; it grants no
expected-card shortcut or inventory inference. At that revision, the earlier9D
crop remained9=34 versus5=50 and correctly refused. The3.1.21 bank below now
recognises it independently. Rank48/20 and suit16/12 bounds, relative
contrast, colour coherence, complete crops, ±1 recrops and unique52 inventory
remain unchanged. The normal frame's6D is calibration evidence. The separate
5H challenge and retained FC05/FC08/FC14/FC18/FC20/FC21 native faces exercise
the augmented bank independently of that new rank sample; these are host
recognition checks, not guest input evidence.

For the native5H at (218,384), all121 required top pixels measure
R243..253/G206..209/B8..60. Its37 required left-rail pixels measure
R249..252/G209..210/B7..32, and its37 right-rail pixels measure
R219..253/G201..208/B12..124. Exactly19 right-rail pixels have red219; the
first is (354,406), RGB (219,201,122). The retired right-role red minimum
changed from220 to219. Those numbers explain the earlier geometry calibration;
current acceptance does not depend on these decorative edge envelopes.
Red218 and interrupted top/left/right derivatives are now positive contrasts
when printed glyphs and actual slot evidence remain readable.

Synthetic derivatives are made only in memory and labelled by their tests:
erased/unknown/mixed glyphs, a native black suit beside an unchanged red rank,
duplicated printed identities, occluded identity corners and synthetic ambiguous
rank evidence remain negative cases. Decorative5H top/rail changes no longer
veto printed identities. Complete acquisition and fresh initial binding still
check all52 printed cards using local contrast; an objective or HALO never
supplies identity or input authority. These saved-frame
regressions do not establish fresh live gameplay, drag delivery or route success.

## Native 8D and 9D rank calibration in 3.1.21

These complete native RGBA images were copied byte-for-byte from the retained
RECON failures. Both are 1920×1080 and include the complete game, toolbar and
Windows taskbar. The source packets remain unchanged.

| Fixture | Source | Bytes | SHA256 |
|---|---|---:|---|
| `freecell-native-eight-20261112.png` | `/home/charlie/tmp/pyramid-error-1794454780365-1451120-1/first-unmatched.png` | 3,667,090 | `83f04c34ce70b2e3c1c215d22119d1207f0f8094fa4761022ed333826bec889c` |
| `freecell-native-nine-20261112.png` | `/home/charlie/tmp/pyramid-error-1794456196473-1451120-3/first-unmatched.png` | 3,700,950 | `7b78640222a30000ed02d51a61650fc8a9802d10b32c5c5956f2f1f34e0c5dd9` |

The first image shows “Solve the deck”; the second shows Game #146692. All 52
printed identities in each frame were transcribed directly, in physical
top-to-bottom order, without completing cards from the remaining inventory:

| PLAY | Native 8D frame | Native 9D frame |
|---|---|---|
| 1 | TD, 6C, QD, 6S, KS, 7C, 5C | JC, 8C, 9S, TC, 2C, 5C, 5D |
| 2 | 9D, 2C, 3S, TC, 8H, 3H, 4C | KC, 5H, 4D, 6C, 4C, 4H, KH |
| 3 | 6D, AD, AC, 5D, 2H, TS, 9S | 9C, 6D, AC, QH, AD, TS, JH |
| 4 | 8D, TH, 4D, KC, 3D, 5S, QC | 5S, QC, QS, 7H, TD, 8H, 3C |
| 5 | 8C, JC, AS, QH, AH, 9H | 2D, 8S, 6S, 7S, 8D, KD |
| 6 | QS, 4S, 6H, 9C, 2D, 7H | 6H, JS, 2H, 9H, 9D, JD |
| 7 | 3C, 8S, 7S, 7D, KD, KH | 7C, TH, 3D, AS, 7D, 2S |
| 8 | 5H, 4H, JS, JD, 2S, JH | 4S, 3S, 3H, AH, KS, QD |

The two 30×48 corner PNGs are explicit native calibration derivatives, not
complete frames or held-out rank tests. Each retains the full 26×23 rank at
local (2,7), while its suit supplies no additional calibration.

| Derivative | Source rectangle | Bytes | SHA256 |
|---|---|---:|---|
| `freecell-native-eight-rank-corner-20261112.png` | (795,331), 30×48 in the native 8D frame | 2,448 | `0a795d516edd9719ece0e3a1f095ebfb36be1c84d1b117ef845fbdb81724c8ad` |
| `freecell-native-nine-rank-corner-20261112.png` | (1180,543), 30×48 in the native 9D frame | 2,405 | `3be894c09766a919238e96d42d6c87eea7a8dcf424de5030b4e42a258ff91f6e` |

The existing 44 original strips plus native 6 sample score the new 8D as
8 = 26 versus 6 = 44, and the new 9D as 9 = 32 versus 5 = 50. Both separations
of 18 correctly refuse the unchanged required gap of 20. Adding these two rank
appearances gives 8 = 0 versus 6 = 44 and 9 = 0 versus 5 = 50. The shared
47-template bank serves acquisition, fresh initial binding and exposed position
proofs. Each supplemental 6D, 8D and 9D template repeats its original FC13 suit
bitmap and colour exactly; no suit is retrained. Every distinct same-colour
label still competes under rank 48/gap 20 and suit 16/gap 12, including the same
integer ±1 recrops. No expected identity or inventory elimination supplies a
rank.

The new ambiguity regression is explicitly synthetic and generated in memory.
It starts with the native 6 rank mask and changes the first 31 raster-order
pixels that differ from FC13's red 5D at (795,597) to that 5 mask. White paper
and coherent red ink render the complete rank and ±1 union. Against the
augmented bank, competition across all nine recrops produces best 31 / next 31, which must
return `AmbiguousIdentity`. Acquisition and fresh binding reject it; position
tests also place it below a valid matching upper card and require refusal
instead of skipping the unreadable lower corner. Damaged ranks, damaged suits,
duplicate identities and colour disagreement remain negative cases for the
new calibration appearances.

These regressions qualify saved-frame recognition only when their host checks
pass. The two source rank corners are calibration evidence; the remaining
cards and retained earlier native faces provide independent comparisons. They
do not establish fresh guest input delivery, gameplay or human acceptance.

## Retained JC to CELL4 drop incident in 3.1.19

`freecell-jc-cell4-returned-20261111.png` is the complete byte-identical
`/home/charlie/tmp/pyramid-error-1794397492303-1326849-3/last-observed.png`.
It is 3,676,267 bytes, 1920×1080 RGBA, SHA256
`bc2e29d7d37830482c12f94b69bb64401415c577ca3c78e84a86affbea916d53`.
The original 3.1.18 failure packet and session log remain unchanged.

The frame follows run61/request1/source-generation1, route cursor39
(transfer40 of87): JC was picked up from PLAY1, sent towards empty CELL4 and
returned to PLAY1. The regression independently recognises the authentic initial
Sixes fixture above, then legally replays the 29 deliberate and ten AUTO edges
recorded before this transfer. The retained frame proves the requested exposed
JC and positively empty CELL4; replay owns the complete hidden inventory.

The source's recognised corner anchor is (233,722) under the legacy 1-bit bank
and (233,724) under `3.2.0` `matched_last` foreground anchoring, with the
existing native pickup inset (+15,+20). The corrected destination retains that
inset at (758,132) for CELL4's fixed origin (743,112). The former centre
(811,203) changed the inset by (+53,+71). The local peer regression requires
the unchanged six-event sequence with lift (233,679), original-endpoint midpoint
(495,428) and exact destination (758,132) under `3.2.0`. Its acknowledged packets
establish host transport delivery, not a measured physical card edge, live
landing or guest acceptance. Other in-memory empty PLAY compositions are
explicitly synthetic.

## Shared foreground card reader and Challenge Complete in 3.2.0

Five complete 1920×1080 RGBA frames are retained verbatim from saved operator
diagnostics to cover the shared foreground Free Cell card reader and input-free
Challenge Complete detection:

| Fixture | Scope | Bytes | SHA256 |
|---|---|---:|---|
| `freecell-sixes-three-diamond-20261113.png` | Sixes `0/3` native tableau with exposed-stack `3♦` (`PLAY 7` row 6) | 3,606,073 | `bee877905098ff91411b8958d3c005cc86384e2a70f2373e9707945a46d07864` |
| `freecell-solve-eight-club-20261113.png` | Solve the Deal `0/1` native tableau with `8♣` (`PLAY 1` row 1) | 3,604,913 | `3ab9ff1e0be11fe62ecf06d2201124b44ef7ef606714ea86287525029db293be` |
| `freecell-challenge-complete-xp-20261113.png` | Live Free Cell crimson-ribbon XP-counting `Challenge Complete!` overlay | 1,805,464 | `aca1db86d94c5a7a5f1c598c6a13d8fe26f3cbd22602097b73ba53efa9433037` |
| `freecell-empty-play1-fours-20261113.png` | Live Fours `0/3` mid-game (`last-observed.png` from packet `pyramid-error-1794528785307-252895-4`): empty `PLAY 1` after `3♥` → `SUIT Hearts`, `CELL 1..4` occupied | 3,543,657 | `beb7f800b282e14ad2936185b74fb9138c84ccd5a662e9592f66a713f1cf9c23` |
| `freecell-challenge-complete-continue-20261113.png` | Settled live Free Cell crimson-ribbon `Challenge Complete!` overlay with **Continue** button visible | 1,841,128 | `5759759c666c1dd7e6dfa5303365ebe8ab688a0bbc4d429e2efcbaf390bb8590` |

- `freecell-sixes-three-diamond-20261113.png` resolves the legacy 1-bit `3♦`
  ambiguity across full 52-card RECON, fresh live initial binding and
  exposed-pile position proof (`PLAY 7` row 6). Mid-card court artwork scanned
  during vertical cascade sweeps returns `CornerClassification::Unreadable` so
  exposed-pile scanning continues to the true corner, while `Ambiguous`,
  `ColourDisagreement` and `PartialLabel` corners still fail.
- `freecell-solve-eight-club-20261113.png` resolves the legacy 1-bit `8♣`
  ambiguity across full 52-card RECON, fresh live initial binding and
  exposed-pile position proofs.
- `freecell-challenge-complete-xp-20261113.png` captures the active XP-counting
  crimson-ribbon overlay before **Continue** appears (`is_challenge_complete_frame`
  returns `false`, while Option 1A `freecell_terminal::classify_challenge_handoff`
  identifies `ChallengeTerminalHandoff::XpCounting` via white `ll` stems,
  non-white ribbon gap, and **Click anywhere to skip** so fault-gated route
  polling retires cleanly after 5 frames with zero input).
- `freecell-challenge-complete-continue-20261113.png` verifies zero-tolerance
  `Challenge Complete!` detection (`is_challenge_complete_frame` returns `true`)
  using both `ll` lower limbs in **Challenge** (`(766, 204, 8×15)` and
  `(785, 204, 7×15)`, RGB `[255, 255, 255]`) and eight black `[0, 0, 0]` segments
  in **Continue**, independent of ribbon or panel colour and without clicking
  **Continue** or authorising any QMP input.
- `freecell-empty-play1-fours-20261113.png` is the complete byte-identical
  `last-observed.png` from `pyramid-error-1794528785307-252895-4` after 19 legal
  transfers emptied `PLAY 1`. Two felt-tinted pixels `(277, 508)` and
  `(279, 508)` on `PLAY 1`'s bottom green outline stroke measure
  `RGB = [49, 151, 110]`, previously leaving `286 / 288 = 993‰ < 995‰` under
  `r >= 50`. Lowering the native empty-slot stroke red-channel floor from `50`
  to `45` (matching measured `46..81` across all 188 native empty slots while
  keeping bare felt `<= 22‰`) achieves `1000‰` across all four strokes. Checking
  interior card paper before the four outline strokes ensures occupied slots
  such as `CELL 1` (`9♦`) report `"Card paper interrupts empty-slot interior"`
  instead of `"Positive native empty-slot outline unavailable"`.
