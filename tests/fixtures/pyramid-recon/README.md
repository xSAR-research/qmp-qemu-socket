# Native Pyramid reconnaissance fixtures

## Evidence boundary

The 29 files in `originals/` are unchanged 1920 x 1080 RGB PNG captures:
20 historical public [Issue 1](https://github.com/xSAR-research/qmp-qemu-socket/issues/1)
attachments indexed in [the calibration record](../../../docs/pyramid-calibration.md),
three originals supplied by Charlie on 11 October 2026, and two original worker
captures from that day's initial failed Gate 3 attempts, followed by the live
stock-scan baseline, refused Undo All modal and later operator modal screenshot,
then the repeated scan's refused Undo All modal.
Exact SHA-256 values,
source URLs or operator-supplied filenames, dimensions and byte lengths are in
`manifest.json`. No PNG was resized, recompressed, annotated or reconstructed
from sparse probes. The repository licence does not grant new rights over the
game's artwork.

These fixtures establish native recognition evidence, not live reconnaissance
acceptance. Unordered captures do not prove 24 stock transitions, a complete
stock inventory/order, acknowledged guest input, or restoration. Neither known
tableau is a complete planning-input document.

**Actual Undo All confirmation is now supplied.** One original modal provides
the full panel/title/body/Cancel/OK and dimmed non-card backdrop signature.
The panel sample is the measured interior `(535,400,64,64)`, excluding the
rounded corner and its variable backdrop pixels. It is not three
board-state-specific dialogs, and its background does not prove
an initial, exhausted or restored board. The checkbox is unchecked and is not
an input target. A generic gold button is insufficient.

## Files

- `originals/issue1-01.png` through `originals/issue1-20.png`: exact historical
  attachment bytes matching all 20 hashes in the calibration record.
- `originals/current-start.png`, `originals/current-exhausted.png` and
  `originals/undo-all-confirmation.png`: exact newly supplied capture bytes.
- `originals/gate3-start-refusal.png` and `originals/gate3-exhausted-refusal.png`:
  original failed-attempt captures, each acquired before any guest input.
- `originals/gate3-undo-baseline.png`, `originals/gate3-undo-modal.png` and
  `originals/gate3-undo-modal-operator.png`: baseline and two observations from
  the live scan that reached a complete inventory but refused the Undo All modal.
- `gate3-undo-incomplete.json`: the exact diagnostic inventory from that failure,
  explicitly not restored planning input; the shared route codec rejects it.
- `originals/gate3-undo-modal-repeat.png`: the unchanged final capture of the
  repeated scan, which loaded v3 and stopped before OK with the same inventory.
- `manifest.json`: provenance for all 29 originals; the original 46 labelled
  observations; 58 additional endpoint observations; both 28-slot tableau
  arrangements; positive scene evidence, corrections and remaining gaps.
- `factorised-glyphs.json`: original-pixel extraction ledger for 33 rank and
  35 suit variants, including source observations, bounds, RGB and explicitly
  derived binary occupancy.
- `native-reader.json`: the strict seven-field factorised reader component.
  This file alone is not an operational root profile.
- `native-profile.json`: the complete loadable native runtime root profile,
  with both card readers, board/Solver-off/pile/Undo signatures and actual modal.
  It embeds samples and contains no host-specific filesystem paths.
- `profile-provenance.json`: all 14 scene/control/modal patch derivations,
  measured tolerances, calibration corrections, file hashes and validation scope.
- `native-v6/baseline.png`, `native-v6/profile.json` and
  `native-v6/provenance.json`: the original refused Grandmaster frame, the
  additive v6 runtime profile and its separate extraction/identity record.
  See [the v6 fixture record](native-v6/README.md). These files do not replace
  the historical `originals/` inventory or its manifest.

The retained bundled v4 runtime profile is
`tests/fixtures/pyramid-recon/native-profile.json`, relative to the application
checkout. The new calibration is separately named
`tests/fixtures/pyramid-recon/native-v6/profile.json`; the operator's v6 copy is
`/home/charlie/tmp/pyramid-native-1920-classic-blue-v6.json`. Runtime freshness,
STOP, selected context, input acknowledgement,
inventory validation and restoration remain controller responsibilities.

## Historical initial-looking board

Image 20 supplies all 28 manually read tableau labels in logical top-to-bottom,
left-to-right order. LEFT is Q clubs, RIGHT is an empty outlined slot, Solver is
off, score is zero and Undo All is disabled. This is native initial-looking
evidence, not proof of unseen preceding actions. C, D, H and S denote clubs,
diamonds, hearts and spades:

```text
row 0: 5S
row 1: 2C 9S
row 2: JC 6C 3S
row 3: 8S 9C 6H 6S
row 4: AD 10S 4D 2S 7C
row 5: 5H 3H 2H 4H 5C 4C
row 6: 10D JD 8D 3D AH 8H 9D
LEFT ordinal 0: QC
RIGHT: empty
```

All 29 observed identities are distinct. Stock ordinals 1-23 remain unobserved.

**Undo All correction:** historical images 01-18 are enabled; images 19 and 20
are disabled. Image 19 was initially mislabelled. Independent visual inspection
and exact RGB comparison show that its entire `(1301,959,38,38)` Undo All patch
is byte-identical to disabled image 20. The enabled image-01 patch differs in
3,804 RGB bytes, with maximum channel delta 173. Solver is on in image 19;
a disabled Undo All control does not establish Solver-off state.

## Current start and exhausted endpoints

Both newly supplied endpoint captures have Solver off and this same complete
28-card tableau:

```text
row 0: 2D
row 1: 9S AC
row 2: QC 4C 5H
row 3: KC 10H 2H KH
row 4: 8C KS 10D 6C 4H
row 5: 3D QH 9H 6H 3C 8S
row 6: 6S AS 2C QS 8D 4D 4S
start: LEFT KD at ordinal 0; RIGHT empty; Undo All disabled
exhausted: LEFT positively empty; RIGHT QD; Undo All enabled
```

The full 140 x 187 native envelopes at all 28 tableau positions are
byte-identical between the two originals; the manifest retains their RGB
hashes. This verifies unchanged visible tableau pixels between endpoints,
**not** the intervening stock sequence or restoration.

## Factorised native calibration

The initial bank used all 28 image-20 tableau corners, Q diamonds from image 01
r4c3 and K spades from image 04 r5c4. It covers all 13 rank labels and all four
suit labels without synthesising any of the 52 complete card identities.

The newly supplied endpoints exposed genuine missing rasterisation variants.
Nine additional components were extracted unchanged from current-start:
ranks 10H, 10D and QC; suits AC, KC, 10H, 3D, 2C and LEFT KD.
The manifest identifies each training observation and component. The sixteen
historical LEFT observations remain excluded from training; they cover eight
ranks (A, 3, 7, 8, 10, J, Q, K) and three suits. LEFT diamonds is now observed
as KD, but its suit component is training evidence, not a held-out success.

Across the 104 labelled observations, 38 contribute a training component and
66 do not. Repeated observations and pixel-identical endpoint tableau regions
are not independent statistical samples; these are deterministic fixture
checks, not a claimed population accuracy estimate.

| Component | Card-relative ROI (x,y,width,height) | Labels / retained variants | Maximum correct XOR | Minimum wrong-label XOR | Cap |
| --- | --- | --- | --- | --- | --- |
| Rank | (2,6,27,24) | 13 / 33 | 24 | 44 | 24 |
| Suit | (5,30,20,16) | 4 / 35 | 8 | 19 | 8 |

All measurements use the final bank across all 104 observations. Occupancy is
`min(R,G,B) < 128`; XOR compares every sampled pixel, including negative space,
at nine integer alignments (`dx,dy` each -1, 0 or 1). No raw RGB is recoloured.
Multiple same-label variants are allowed; zero labels is unknown and multiple
labels is ambiguous. Neither permits nearest-label fallback. Actual paper and
red/black ink checks remain separate and require both glyphs to agree with
the recognised suit colour.

The original suit ROI height 18 included dark King-hand artwork at relative
rows 46-47 in the actual LEFT KD capture. The corrected height 16 excludes that
art rather than weakening mixed-colour rejection. It samples a discriminating
glyph region, not necessarily every outline pixel: some club/spade stem pixels
extend to row 46. All positive and adverse cases were rechecked after this
change. Rank/suit caps remain 24/8; alignment remains one pixel.

Scene patches retain exact native RGB. Two rule-chart patches differ between
historical image 20 and current-start by maximum channel deltas 6 and 8;
only those scene patches use measured tolerances 6 and 8. Other board/control
patch tolerances are 4. These scene tolerances do not change glyph XOR limits.

The initial Gate 3 attempts exposed a crop-boundary error in those two chart
patches: their bottoms included fragments of text from the next row. Runtime
profile `pyramid-native-1920-classic-blue-v2` retains the complete K entry and
separator in `[1810,132,78,40]`, and the complete 9/4 entry and separator in
`[1815,281,67,42]`. RGB samples remain exact crops from `current-start`; every
retained pixel is selected and tolerances remain 6/8. Maximum deltas for both
new captures are 6/7. Spatial coverage is deliberately narrower: the excluded
next-row fragments are no longer constraints. Independent K, 9 and 4 erasure,
selected-edge channel limits, excluded-strip mutations and the other mandatory
scene/control signatures are tested with the actual bundled profile. See the
[Gate 3 correction record](../../../docs/pyramid-recon-gate3-correction.md).

The next live scan recognised all 28 tableau and 24 ordered stock identities,
then refused the Undo All dialog before OK. Its two modal observations have
selected RGB differences up to 9/10/9/8 in title/body/Cancel/OK compared with the
original modal, exceeding the former uniform tolerance 4. Profile
`pyramid-native-1920-classic-blue-v3` changes only those four modal tolerances
and the profile ID, retaining every crop, RGB sample, mask pixel and target.
Panel/backdrop tolerances remain 4. Wider bounds apply to every selected pixel,
not only observed text edges; exact boundary, missing-component, derived text
substitution and existing other-game native-scene negatives test that trade-off.
No native near-identical non-Undo confirmation has been supplied. See the
[modal correction record](../../../docs/pyramid-recon-modal-correction.md).

The repeated scan still refused ten Cancel/OK pixels exceeding v3 limits by one.
Bundled `pyramid-native-1920-classic-blue-v4` instead changes only the four text
masks and ID. Its source-only selection retains pure-ink centres OR samples with
every RGB channel's original 3x3 range at most 16, including a one-pixel halo
outside the ROI. RGB, ROIs and tolerances remain unchanged. All seven components
remain required, but excluded samples are unconstrained: this is a documented
acceptance widening, not full OCR. Fixed rule/count/hash, retained ink, negative
space and selected-channel boundary tests accompany the change. See the
[stable-text correction record](../../../docs/pyramid-recon-stable-text-correction.md).

## v6 native glyph variants

The v6 calibration starts from the exact accepted local v5 profile, SHA-256
`b48e0b1a20aec2439422a1b967de1468d5553cc9a12077784356c3b68bcfd3e7`.
It retains v5's chart-mask correction and every existing glyph variant, scene,
modal signature, ROI and threshold. The profile ID becomes
`pyramid-native-1920-classic-blue-v6`; schema 1 is unchanged. The historical
v4/v5 metadata remains separate and unchanged.

The original Grandmaster frame is 1920×1080, SHA-256
`02b1f0f98c8a4c5b5d57b4075fceb2829d1175d8b80ef2455d94fc81fff1c988`.
The failed v5 observation first refused logical tableau slot 13: its Queen
rank's best occupancy XOR distance was 28, above the unchanged limit 24.
Independent diagnostics also found King of spades at slot 16 with suit distance
10 and Jack of spades at slot 19 with distance 9, above the unchanged limit 8.
The spade at slot 24 was already correctly recognised at distance 6 of 8 and
supplies the new suit sample. All slot numbers are zero based.

Two unchanged RGB crops from that frame are appended to **both** readers:

| Component | Native bounds (x,y,width,height) | Source slot | Variants per label |
| --- | --- | --- | --- |
| Queen rank | `(1093,331,27,24)` | 13 | Q: 2 → 3, limit 8 |
| Spade suit | `(895,462,20,16)` | 24 | spades: 8 → 9, limit 16 |

Each reader therefore has 34 rank and 36 suit variants. XOR limits remain
24/8, alignment remains ±1 pixel and paper/ink/colour checks are unchanged.
This is additional native rasterisation evidence; no pixel is recoloured,
rescaled or combined into an unobserved card. Its 29 visible identities are
calibration evidence, not a stock scan, full 52-card inventory or new
restoration proof.

All eight v6 regressions pass through the production Rust parser and
recognisers. Both reader banks preserve all 104 historical observations,
including the 16 held-out LEFT observations. The full application suite passes
858 tests with none failed or ignored; locked check, strict private rustdoc,
all-target Clippy and native release build also pass. Fresh live reconnaissance
remains unverified. A successful v6 classifier test alone cannot establish
capture freshness, stock order or original-board restoration.

## Validation and remaining proof

The preceding delivery checked all 23 original hashes/dimensions, all 68 retained glyph samples and their
binary occupancy, and all 14 runtime signature samples are checked against
their named originals. No crop is mislabelled as an original. All three supplied
originals were visually inspected, as were the historical labelled corners.

The independent Python reference check passes 104 native positives and
1,255 negative cases: erased glyphs, five-pixel displacement, inconsistent ink
colour, solid unknown symbols, missing labels, ambiguous labels and seven
native empty-pile observations. Mutations exist only as derived adverse test
inputs; they are not native captures.

The isolated Rust harness loads the concrete runtime file through a snapshot
of the actual production reader and real xSAR PNG decoding. It verifies the
historical initial board, both current 28-card endpoints and pile states,
Solver-on refusal, enabled/disabled Undo All, actual modal recognition,
refusal after erasing any modal component, gold-button-only refusal and
independence from variable background card identities. It also exercises
native glyph unknown/ambiguity and colour failures. The final corrected-profile
run passes six tests, zero failed and zero ignored; adverse glyph checks cover
all 104 observations, with seven native empty-pile cases. Detailed counts and
source qualification are in `profile-provenance.json`; logs and retained first
failures are under `/tmp/pyramid-recon-native/`. This focused evidence does not
substitute for the parent's whole-application validation or live acceptance.

Still missing: retained individual LEFT image fixtures for ranks 2, 4, 5, 6 and 9;
exhaustive native proof of all 52 card identities/layouts; and positive live
restoration of the original tableau/LEFT/RIGHT followed by complete JSON output.
The new live diagnostic records all 24 ordered stock identities, but only its
baseline/latest images were retained, not an image for each intermediate card.
Historical images 06-09 provide empty-LEFT artwork
with Solver on and partial tableaux, not a successful Solver-off reconnaissance.

The fixtures grant no guest-input authority, do not infer missing identities
by elimination and do not weaken STOP or context revocation.
