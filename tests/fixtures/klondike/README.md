# Klondike detector regression fixtures

These forty derived PNGs retain original RGBA pixels only in the rectangles
listed in `manifest.json`; other pixels are opaque black. A per-file rectangle
list overrides the default for K16-K25 so their toolbar overlap is retained,
K31-K40 use that same toolbar rectangle, and K26-K30 retain the complete
measured terminal/control region. Their dimensions and
coordinates remain 1920 × 1080. This avoids retaining unrelated desktop pixels without
resizing, colour conversion or loss of pixels used by this detector's scene,
HALO and effect probes. They are test data, not replacements for the originals.
The manifest records original names and SHA-256 alongside each fixture hash.

K01-K11 correspond to the supplied evidence numbering. K12 is the original
`qmp-qemu-socket 260930 015843 Recycle-Stack.png` showing highlighted exhausted
stock. The five actual action pairs are K02/K03, K04/K05, K06/K07, K08/K09 and
K10/K11. K17/K18 adds Charlie's replayed Draw pair. Other adjacent timestamps
are not assumed to be a single input trace.

K13 is the supplied `qmp-qemu-socket 260930 034141 Solve-available.png`.
K14 is `qmp-qemu-socket 260930 034428 Stopped-Uncertain.png`; K15 is
`qmp-qemu-socket 260930 034701 Failed-to-recycle.png`. K14 and K15 show
result scenes only: no preceding failure-action frame was supplied. They
therefore establish current targets and card artwork, not the cause of the
reported failure or a successful before/after action pair.

K16 is `qmp-qemu-socket 260930 050143 Fail.png`; K17 is
`qmp-qemu-socket 260930 055422 1-step-before-fail.png`; K18 is
`qmp-qemu-socket 260930 055442 Step-where-it-failed.png`; K19 is
`qmp-qemu-socket 260930 055856 Failed-to-detect-big-HALO.png`.
K17/K18 shows Draw revealing 2-hearts at RIGHT fan offset 1. K16 and K18
have identical retained pixels; they do not show the subsequent RIGHT move
succeeding. Charlie reports that replaying through single steps worked, so a
live timing issue remains possible but these frames do not measure it.

K19 supplies the eight-card column-6 source from K-diamonds to 6-clubs. Its
paired exterior rails start at y390 and continue through y957; the 96-pixel
lower closing edge is visible at y951-952 beneath the translucent toolbar.
The toolbar first dims the card at y947. Candidate 1 incorrectly closed the
block at an internal card crossbar, returning rows390..779. Candidate 2
recognises rows390..953 and keeps its click at (1296,430), on the upper card.
The scan ends at y963, accepts the existing darker rail colour only in the
measured y947..962 overlap, and requires closed ends and terminating exterior
rails. The existing 600-pixel height limit remains. Clipped outlines, removed
closing edges, broken rails, toolbar-only pixels and dashed guides are refusal
cases. No result frame for this long-run move has yet been supplied.
Charlie subsequently confirms that Single Step moved the long run successfully;
the detector geometry defect does not by itself explain the continuous-run stop.

Source effect proof ends before y947; destination proof retains its earlier
y936 limit. Toolbar changes therefore cannot establish either side of a move.
The derived fixtures retain toolbar pixels through y1032 so this refusal is
exercised using the original layout rather than blacked-out control pixels.

K20-K23 retain Charlie's 30 September evening failure evidence. K20 is
`qmp-qemu-socket 260930 225837 Two-moves-before-fail.png`; K21 is
`qmp-qemu-socket 260930 225821 Move-before-fail.png`; K22 is
`qmp-qemu-socket 260930 225614 Move-fail.png`; K23 is
`qmp-qemu-socket 260930 230232 Second-Fail.png`. These timestamps reflect
Undo reconstruction and must not be sorted into an assumed input sequence.
K20/K21 reconstructs the RIGHT 2-spades transfer to foundation 2. K21/K22
reconstructs the next column-5 3-spades transfer to the same foundation.

In K21/K22, the card has reached its foundation and the next dashed destination
guide covers both the newly exposed 4-diamonds and the felt below it. The old
source's measured rows571..761 change by 8434 retained pixels, yet zero pixels
meet the existing bright white/felt replacement test. Outside the same gold,
cursor and edge masks, 1050 formerly white pixels become chromatic green felt
under the translucent guide. The added tableau-only proof recognises this
measured green residue; neutral dimmed paper and opaque black remain refusals.
It retains the independent 512-pixel source and destination requirements.

K23 is a result-only capture. The log identifies the preceding source as
column 6, rows337..527; the image now shows that column empty under a dark
next-destination guide. Its test constructs a before frame using genuine card
pixels from K21 and ordinary column-4 pixels from K20. That explicitly synthetic
before frame does not establish the preceding card's identity, original source
pixels, destination or a live successful action pair. It tests the same colour
boundary against K23's original darkened empty-column pixels. Guide-only
darkening, unchanged frames, source-only changes, cursor pixels, gold and
bottom-toolbar pixels cannot establish a completed transfer.

The production `src/klondike-solve-control.rgb` contains 899 RGB 8 samples from
K13, on a four-pixel lattice over the measured Solve control. The manifest
records its exact bounds and hash. Matching requires the control outline,
check-mark/letter artwork and colours, plus a separately empty stock and
calibrated gameplay scene. Solve is distinct from the lower Solver toolbar
control and is sent once before bounded, input-free one-board completion observations.

Two new tests construct synthetic RIGHT replacements using genuine K14/K15
card faces. An A-spades/J-clubs or 2-diamonds/2-clubs replacement can retain
mostly white paper and fail the old one-way positive-pixel gate. The focused
replacement test checks paper/ink changes in two opposed corners and still
requires an independent destination change. No card rank or suit is decoded.
Guide darkening, unchanged/source-only frames, one changed corner and cursor
changes remain refusal cases. Passing these synthetic cases is not proof of
the original live failure cause or of Beast acceptance.

The Rust tests load these files at runtime under `CARGO_MANIFEST_DIR`; they are
not linked into a release executable. Explicitly synthetic tests translate
real source-card pixels to each waste fan/foundation location and manipulate
mask/cursor/overlay pixels to exercise refusal paths. The synthetic recycle
after frame does not establish live recycle acceptance. Those action classes,
settings and input semantics are authorised by Charlie's 30 September 2026
instructions; new live effects still require observation on the Beast.

K24 is `qmp-qemu-socket 261001 025337 Repeat-Fail-after-undo.png`. Its
highlighted column-5 3-spades reproduces the fresh tableau recommendation.
The accompanying log stopped before guest input because the previous approved
preview was RIGHT and the fresh target had changed after Undo. It is not a
repeated failure of the source/destination effect check and is not a transfer
before/after pair. Unknown changes to an approved preview still refuse input.

K25 is `qmp-qemu-socket 261001 030153 SOLVE-Icon-available-to-complete-game.png`.
Solve is available alongside a highlighted tableau 10-spades; the specified
priority selects Solve before that tableau source. The exact right progress
probe is `(1050,87,38,2)`, containing 76 native pixels. K25 has 56 black and 20
measured warm-gold pixels, so it is explicitly unfinished. Sharing the other
modes' 85% black threshold would misclassify this actual capture.

Klondike's independent one-board completion observation requires literally
zero pixels with all RGB channels at most 32 and all 76 pixels positively
matching the measured warm-gold fill. It also needs the original Solver
banner rails, native game scene and no higher-priority Draw, recycle, RIGHT,
Solve, tableau or foundation target. Absence of black alone cannot establish
completion. The worker confirms repeated fresh observations without guest
input and does not infer a three-board progression or send restart input.

No unoccluded completed-bar or transient Perfect image was supplied. The positive
bar test is explicitly synthetic: it paints K25's measured gold colour into
the entire right probe on an existing active scene without a HALO. It tests
the authorised bar-classifier boundary, not live game-win acceptance. Negative
probes retain one black pixel, arbitrary non-black/blank colours, broken
Solver rails, blank/covered scenes and each higher-priority action. All actual
K01-K25 captures remain negative completion evidence.

K26-K30 are Charlie's original 1 October terminal-sequence captures:

- K26: `1 - GAME-WIN.png`, Congratulations and Click anywhere to skip.
- K27: `2 - LEVEL-UP.png`, Klondike Level-Up and OK.
- K28: `3 - NEW-GAME.png`, completed Congratulations result and New Game/Home.
- K29: `4 - GAME-PLAY.png`, Standard (Draw 1) setup with Grandmaster selected
  and Play/Cancel controls.
- K30: `5 - READY-FOR-SOLVER.png`, newly dealt board, Solver off.

These retain the original RGBA pixels in the half-open central rectangle
`(260,34)..(1660,1033)` and the two measured outer context patches. The manifest
records their hashes, source names, dimensions and rectangle list. The
Klondike-only terminal classifier matches independently measured title/control
artwork and completed-board background. K26-K28 supply actual positive win
observations despite their dimmed/non-gold progress bar; K29/K30 are negative
completion observations. No rank or level number is recognised independently.

The sequence and its click semantics are explicitly authorised by Charlie:
skip score counting, Level-Up OK when present, New Game, the unchanged Draw 1
setup Play control, then Solver on the newly dealt board. Fresh stage evidence
is required before each logical action. Continuous mode alone may restart;
finite/Single Step completes its authorised action budget and stops at the
verified game result. Unknown controls, inconsistent stages, uncertain inputs
and STOP cannot authorise another click. The local fixture tests do not measure
actual animation/transition durations; the editable existing delays remain
subject to Beast acceptance.

K31/K32 adds the supplied new successful-move refusal:
`qmp-qemu-socket 261001 033735 before-fail.png` and
`qmp-qemu-socket 261001 033314 Fail-card-was-at-the-same-position.png`.
The later before timestamp is an Undo reconstruction, not an automatic
chronological input pair. Column-7 2-clubs moved below column-3 3-diamonds;
King-spades was then automatically revealed in column 7. The log reports
3984 changed source pixels, 428 ordinary positive pixels, nine dimmed-felt
pixels and 17151 changed destination pixels. Reconstructed original pixels
produce slightly different counts; the test preserves the verdict rather than
asserting byte identity with the logged initial frame.

White paper replacing mostly white paper defeats the one-way positive source
proof. The new identity proof is restricted to complete 180..190-pixel
single-card tableau outlines. It compares the established opposed 28 by 44
corner patches at offsets `(5,5)` and `(99,126)` from the fresh source top.
Each patch retains at least 50% white paper before and after, excludes gold
and the commanded cursor mask, and needs at least 48 paper-to-ink and 48
ink-to-paper transitions. The actual pair has directional counts 101/179
and 242/85. It also needs at least 512 independently changed source pixels
and 512 separated destination pixels. Thresholds are retained, and the
existing RIGHT total-corner semantics are unchanged.

Negative tests retain unchanged and source-only/destination-only frames,
one changed corner, insufficient material source change, cursor/gold/black
changes and one-way dimming/whitening in both corners. Runs taller than the
measured single-card outline cannot use this new alternative proof. Neither
a fresh HALO nor a rank inference establishes the preceding effect.

The profile remains bounded to the evidenced geometry, Draw 1, 100% guest
scaling and complete source outlines, including K19 and K38's measured toolbar
overlaps. Unknown, shifted or generally dimmed layouts stop. The new actual
terminal artwork establishes classifier evidence; automatic live transitions,
editable delays and repeated-game operation remain to be verified on the Beast.

K33/K34 adds the successful RIGHT transfer refused by candidate 2:
`qmp-qemu-socket 261001 045349 Move-before-Fail.png` and
`qmp-qemu-socket 261001 045324 Fail-unnecessary-card-detection-stall.png`.
K33 is Charlie's later Undo reconstruction. It shows a highlighted 3-hearts
at RIGHT fan offset 2 and an occupied 2-hearts foundation under a dashed guide.
K34 shows 3-hearts received by that foundation and 4-hearts highlighted at the
same RIGHT coordinates. The next recommendation preserves the recipient's
dark guide. The existing inset counts only 360 materially changed pixels,
all from the new middle heart; this matches the runtime recipient count.
Reconstructed source counts are not asserted as the original runtime count.

Candidate 3 retains ordinary source/destination bounds of 512 pixels. Its
specialised occupied-foundation recipient proof retains the original
16-pixel inset and channel difference of at least 48. Both inset halves need
at least 48 new red-ink pixels: K33/K34 provides 176 and 184. Before and after
retain 12,483 common neutral dimmed paper pixels of 14,300; none changes by
eight or more per channel. At least 80% common paper, red print before/after,
and unchanged gold patterns and colours on all four guide rails are required.
Ordinary bright paper, opaque black, felt, newly introduced/removed guides,
guide recolouring/fading and black/white cursors cannot use this route. A
source still needs independent removal/replacement proof and at least 512
materially changed pixels. The specialised recipient route applies only to
RIGHT/tableau sources, never to SUIT sources. Dimmed black-pip replacement is
not calibrated by this fallback. No rank is read and no guide is clicked.

SUIT source replacement uses its existing canonical upper-card geometry and
confirmed return-to-tableau action class. Its opposed bright-paper corners
now use the strict tableau policy: both print directions in both corners,
material source change of at least 512, and independent ordinary recipient
change of at least 512. A separate bright-paper SUIT source route also needs
512 material source pixels, 80% retained neutral paper, at least 48 new and
48 cleared red/black print pixels, and unchanged unprinted paper fields.
The paper-field check uses a 3-by-3 neighbourhood within the original inset,
excluding antialiased glyph boundaries while rejecting neutral paper fading.
Gold and the commanded pointer neighbourhood remain excluded.

The rare `qmp-qemu-socket 261001 050009
Rare-case-where-a-card-is-moved-back-from-SUIT.png` was named in Charlie's
message but was not supplied in the available attachments. SUIT positives
therefore use explicitly synthetic arrangements of real K14/K15 card artwork
with K31/K32 recipient pixels. They are classifier tests, not a captured legal
return or live-input acceptance. The upper-face composition yields 772
material source pixels, 70 ordinary positive pixels and corner directions
67/88 and 67/83. Restoring eight outside-inset columns of one corner isolates
the new bright-paper route: 643 new and 65 cleared print pixels remain, with
zero changed unprinted paper fields. These counts do not claim to reproduce
the missing rare-case image or its reported 618/228 runtime source counts.

Negative tests cover unchanged/source-only/recipient-only frames, guide
appearance/removal, colour and paper shifts, cursor arrival/departure, gold,
small and one-sided red patches, toolbar noise, one-way source shading and
malformed geometry/storage. The checks report their individual measurements
in the private session log; a fresh repeated HALO remains insufficient alone.

K35/K36 adds the next successful RIGHT transfer refused by candidate 3:
`qmp-qemu-socket 261001 062949 Move-before-fail.png` and
`qmp-qemu-socket 261001 062853 Fail.png`. K35 is a later Undo reconstruction,
showing a highlighted RIGHT 8-spades at fan offset 1 and a dimmed 7-spades
foundation under the destination guide. K36 shows the received 8-spades and
the next RIGHT 9-spades source at offset 0. The recipient has 251 materially
changed inset pixels, matching the runtime count. The existing specialised
red-print route correctly refuses this black-pip change. No threshold or
rank-specific recipient exception is added.

Candidate 4 separately reports `source_replaced` using the established source
removal/replacement proof. The complete effect still requires both source and
recipient evidence; K35/K36 therefore retains `verified=false`. This source
verdict is not set for Draw, recycle or Solve. The worker can separately record
continuation from a fresh, canonical HALO after acknowledged card input and
independent source replacement, without claiming the complete prior transfer
was proven. Unchanged or recipient-only scenes do not obtain source authority.
These reconstructed images do not assert source byte identity with the original
runtime planning frame or establish a new input retry permission.

K37 is `qmp-qemu-socket 261001 063331 No-detecting-Solve.png`. The existing
Solve recogniser passes its artwork, scene, stock and Solver-banner guards and
selects Solve ahead of the still-highlighted column-1 Jack-hearts. The supplied
extended log likewise recognised Solve as the fresh next target; the previous
unchanged source effect blocked its execution. A test paints out only the Solve
control to construct a synthetic preceding scene, leaving the actual highlighted
tableau source unchanged. It demonstrates independent current Solve availability
without falsely establishing that prior card's effect. The actual later Solve
click was acknowledged once, but completion was not established by its bounded
observations. This fixture makes no live completion or restart claim.

K38 is `qmp-qemu-socket 261001 064037
HALO-goes-into-the-lower-bounds-of-the-screen.png`. Its column-6 source is the
nine-card run Queen-clubs, Jack-diamonds, 10-spades, 9-hearts, 8-spades,
7-hearts, 6-clubs, 5-hearts and 4-clubs. The paired exterior rails run from
y407 through y987, with a closed top and observed lower border through y990.
The exclusive outline scan bound is now y994, leaving a measured clear row
below the lower edge. The source is rows407..990; the canonical click remains
on its upper card at (1296,447). Its effect region is (1230,407,132,540), ending
before the toolbar at y947. The white-face probe is clipped to that same upper
boundary and uses the original 1152 white pixels at y911..923.

The Undo All icon obscures the lower border in the exact rectangle
`(1308,988,28,3)`. Only column 6 with the complete y994 scan can use this
exception. It needs positive red icon support in `(1314,980,14,11)` with at
least 48 matching pixels; K38 has 68 of 154. All 68 remaining lower-edge
pixels outside the occlusion must match the gold rail colour. The existing
closed top, paired-rail continuity, clear exterior row, face, safe click and
600-pixel height guards remain. There is no acceptance of a clipped bottom or
an arbitrary missing edge. K19 retains rows390..953 and its upper-card click.

Additional negatives remove the visible border or icon support, truncate the
scan, break the upper edge or rails, remove the safe white probe, introduce
floating or internal gold lines, copy a genuine dark guide, add black/white
cursor pixels, change toolbar-only pixels and supply malformed frame storage.
The original attachment names, dimensions, hashes, retained rectangles and
fixture hashes are recorded in the manifest. These are pixel and controller
regressions; live input and continuous-game acceptance remain Beast checks.

K39/K40 adds `qmp-qemu-socket 261001 075229 Move-before.png` and
`qmp-qemu-socket 261001 075722 Fail.png`. The before frame shows column-3
4-clubs highlighted in a complete single-card outline rows799..989. Foundation
4 contains 3-clubs under the dark dashed destination guide. The result shows
4-clubs received there, the column-3 stack expanded to expose 5-hearts at its
bottom, and the next source is column-6 5-clubs rows372..562. The actual pixels
produce the same 824 material source pixels, 61 positive replacement pixels and
broad 1298-pixel separated change diagnostic as the supplied log. The attachment bytes are not asserted
to be identical to the runtime planning/result frames. All four logged settled
observations had unchanged failed proof counts after one acknowledged input;
they establish no QMP delivery failure or timing instability.

The ordinary opposed-corner proof still rejects the lower-right patch
`(825,925,28,44)` because it crosses the toolbar at y947. Candidate 5 adds a
separate full-visible-patch policy only for a canonical complete 180..190-pixel
single-card tableau source whose ordinary lower patch actually crosses that
boundary. Its full upper-left patch `(731,804,28,44)` is unchanged. The entire
lower-right 28-by-44 patch moves up to end at the proof boundary, giving
`(825,903,28,44)`; no patch is truncated and no toolbar pixel is inspected.
The two patches remain spatially separated, retain the original gold/pointer
masks and need 50% white paper before and after.

Native upper-left paper counts are 826/1033 of 1232, with 68 paper-to-ink and
204 ink-to-paper transitions. The full lower-right patch has paper counts
1098/884 and directional counts 228/56. Every print direction retains the
existing minimum of 48. The new route also requires at least 512 materially
changed source pixels. It supplies source replacement for the existing guarded
C4 fresh-HALO continuation policy only; it never grants complete-effect
verification or completion authority. The ordinary source/recipient verification
formula and ordinary clipped-corner refusal remain unchanged. The worker still
requires acknowledged input, a fresh canonical source recommendation, STOP and
invalidation guards, and no uncertain input retry.

Native recipient isolation is significant here: the actual receiving foundation
changes by only 318 ordinary inset pixels. The broad 1298 count instead comes
from the next column-6 source's HALO decoration: 982 blue-back shading pixels at
y364..377, 230 lower-shadow pixels at y538..557 and 86 at y558..565. Its central
printed face interior y394..537 changes by zero pixels. Neither the broad count
nor a newly visible HALO proves the receiving-card effect. K39/K40 therefore
reports `source_replaced=true` and `verified=false`, and any continuation is
logged separately without claiming this complete transfer was proven.

Six focused tests cover the actual pair, unchanged/source-only/recipient-only
scenes, next-HALO decoration alone, insufficient material source change, either restored patch despite
unrelated material shading, one-way print changes, gold/neutral/dark guides,
commanded pointer pixels and toolbar changes. Geometry negatives include
ordinary lower corners that still fit above y947 despite a crossing outline,
unsafe height, a tall run, noncanonical shifted actions and malformed frame
storage. The recorded diagnostic reports both visible directional counts even
when the independent source-material gate refuses continuation evidence. Source-only
replacement passes source evidence but remains unverified as a complete effect;
the actual 318-pixel recipient alone establishes neither source nor effect. No rank or
suit is decoded, and no new action, timing or completion behaviour is enabled.

## v1.2.4 candidate 1 additions

- K41: ten-card column 6 source from rows390..994; closing gold at991..993 is
  partially hidden by Undo All. Complete source height604, safe click1296,430;
  effect evidence stops at947. Prior col3 move was verified in the supplied log.
- K42: Solve is visible alongside a tableau J-spades HALO. The stable interior
  and glyph pass existing tolerance; outer-border positive red warming reaches
 26 levels rather than24. This is a recorded appearance variant, not a priority
  change or evidence of a failed guest click.
- K43: settled Congratulations level46 with click-anywhere-to-skip. All existing
  terminal signatures pass. This PNG was captured10.211s after Solve ACK, after
  the old observation budget had ended at5.641s; earlier transient frames were
  not supplied. It does not establish the exact time the dialog became stable.

Each derivative retains the same native-coordinate rectangles as its corresponding
prior fixture class. The manifest records original and derived SHA-256 hashes.
No image is resized; originals remain separate from these sparse test fixtures.

## v1.2.4 candidate 2 additions

K44–K49 retain their original native pixels in the manifest's rectangles.
K44's complete column1 source needs the measured14-pixel right scan; K47's
settled column4 run already passes. Their logs rejected earlier unsupported
result scenes that were not saved. K45 is an earlier RIGHT Queen-clubs Undo
reconstruction, not the immediate predecessor of K46's bottom-card action.
The native K45→K44 transfer is reconstructed evidence only. The separate
contracted-source regression explicitly synthesises the prior4-diamonds face,
retains K46 result artwork and tests the log-derived print/geometry guards.

K48's existing Solve recogniser matches all899 artwork/342 glyph samples and
13,900 stockfelt pixels; further live refusals need the new same-observation
measurements. K49 Level Up exposes both a variable textshadow and later warm
fireworks. A controlled derivative removes only fireworks context to isolate
shadow refusal; mixed title/label/button/foundation guards remain required.
No derivative is presented as an original successful QMP input trace.

## v1.2.5 candidate 1 additions

K50 is the Undo reconstruction of the pre-transfer recommendation, captured
after the stopped run. K51 shows 4-clubs on SUIT 4, 5-diamonds exposed in column 1
and a new 5-clubs source in column 3. Their attached original bytes are retained
separately; the fixtures preserve native RGBA pixels in the listed rectangles.
They are not the exact original worker capture pair. No code recognises ranks.

K52 records the centred KLONDIKE label shifted two pixels at level 51. Stable
button geometry is unchanged. The other log records a recognised OK click and
later unrecognised frames that were not supplied; K52 does not establish those
frames or prove that click failed. Solve animation tests use explicitly
controlled derivatives of previously supplied artwork, not invented live frames.

## v1.2.6 candidate 1 addition

K53 is the later read-only Congratulations/New Game save at level 53. Log
1790964873320 records completed scene and both button labels passing while one
Home body colour probe failed on four earlier observations. No New Game input
was sent. The later native frame shows the last OK pointer at (960,795), whose
dark edge covers the old (990,827) sample. Replacement (1020,827) is identical
in K28 and K53. Original and fixture hashes/native retained regions are recorded.
The attached bytes differ from logged worker PNG size, so exact earlier frame
identity is not claimed. No original was edited or resized.

Pending Solve regressions use labelled controlled derivatives of original
button artwork, matching the logged 459/483 interior and 342/342 lettering
counts. No early Solve screenshot was provided in this cycle.
