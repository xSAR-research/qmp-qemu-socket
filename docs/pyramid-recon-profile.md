# Pyramid reconnaissance native profile

## Scope and evidence boundary

`src/pyramid_recon/vision.rs` implements a read-only, bounded classifier for an
explicit native calibration profile. It does not send input, capture frames,
infer cards by elimination, decide that a deal is restored or publish JSON.
Those responsibilities belong to the reconnaissance controller and worker.

**Native calibration is an evidence-backed candidate, not live acceptance.** Unit
tests distinguish synthetic mechanism patterns from the recovered native originals.
A missing or invalid profile refuses loading. A supported board is returned only
when all 28 tableau cards and both pile states are classified from actual frame
pixels. Test success is not native card-reader or live-input acceptance.

The existing [Pyramid calibration record](pyramid-calibration.md) records 20
original native PNG hashes and attachment URLs, the seven-row geometry and
toolbar positions. It explicitly does not calibrate the Undo All confirmation.
All twenty original PNGs have now been recovered, checked against those hashes
and retained in `tests/fixtures/pyramid-recon/originals/`. The adjacent manifest
records 46 manually labelled card observations: 30 tableau training observations
cover all 13 ranks and four suits, and 16 LEFT observations are held out. Native
tests exercise those real pixels through the production reader. Charlie then
supplied the actual 1920×1080 Undo All confirmation, SHA-256
`78da9587035ee2707ffacf4adaf493a2a55e1ed5c28712bd5765dca23b30a9e0`.
Its original pixels now supply a specific modal signature. It is one modal over
an arbitrary partially played board, not proof of exhaustion or restoration.
Native probe extracts and HALO locations are not substituted for glyph pixels.

Charlie also supplied a current start frame, SHA-256
`4aff43e30e8dfa3218467fbedd9169b6c224613f97a7aa976722b8d39d24ac35`,
and a current exhausted-LEFT frame, SHA-256
`b352d1645b15b2cb58d2a7a64e8b25a845ca91c955e3d66861cb76be1d352ef6`.
They show the same 28-card tableau, initial LEFT King of diamonds/RIGHT empty,
then LEFT empty/RIGHT Queen of diamonds. Their unordered existence does not
prove the intervening 24 advances, complete stock inventory or restoration.

The classifier supports only 1920×1080 captures, 100% guest display/text scaling
and CLASSIC BLUE artwork. Its profile declaration is not proof that those
conditions hold: the calibrated scene patches must discriminate them in pixels.

The first live attempts exposed partial-next-row text in two chart-signature
crops. The [Gate 3 correction record](pyramid-recon-gate3-correction.md) documents
the two retained original failure captures, narrower complete-row crops and
unchanged tolerances. The corrected profile ID is
`pyramid-native-1920-classic-blue-v2`; its JSON schema remains version 1.
Offline recognition of those captures is not live acquisition/restoration.

The subsequent live run completed the 28+24 inventory but refused the Undo All
modal. The [modal correction record](pyramid-recon-modal-correction.md) records
the previous profile, `pyramid-native-1920-classic-blue-v3`: exact unchanged modal
RGB/crops/masks with measured title/body/Cancel/OK tolerances 9/10/9/8. Panel and
both backdrops remain at 4. The previous uniform 4 was insufficient for those
actual text-rendering samples. This widens accepted channel ranges, with explicit
negative/boundary checks; it neither enables generic OK nor resumes a failed run.

The repeated modal refusal is addressed by the current
`pyramid-native-1920-classic-blue-v4`, described in the
[stable-text correction record](pyramid-recon-stable-text-correction.md).
Only four text masks and the ID change from v3. Each mask retains exact pure-ink
centres OR samples whose original 3x3 neighbourhood has every channel range
at most 16, including a one-pixel halo outside the ROI. RGB/crops/tolerances stay
unchanged. Excluded pixels are unconstrained: this deliberately widens matching
and requires retained foreground/negative-space tests, not a claim of full OCR.
All seven modal components, fresh second confirmation and restoration checks
remain mandatory. The mask rule uses the original modal only, not live failures.

## Public interface

```rust
pub struct ReconVision;

impl ReconVision {
    pub fn load(path: &Path) -> Result<Self, String>;
    pub fn profile_id(&self) -> &str;
    pub fn observe(&self, frame: &CapturedFrame) -> Result<ReconScene, String>;
}

pub enum ReconScene {
    Board(BoardObservation),
    UndoConfirmation { ok: PixelPoint },
    Unknown,
}

pub struct BoardObservation {
    pub tableau: [Card; 28],
    pub left: StockFace,
    pub right: StockFace,
    pub undo_all: Option<PixelPoint>,
}

pub enum StockFace {
    Card(Card),
    Empty,
}
```

These are interface declarations, not complete source definitions. `Card` is
`xsar::games::cards::Card`; frame and geometry types are the application's xSAR
reexports. Observation types implement `Clone`, `Debug`, `PartialEq` and `Eq`;
`StockFace` also implements `Copy`. The validated calibration is private and
cannot be constructed by supplying unchecked templates through this API.
The retained ASCII `profile_id` is available for worker logging alongside the
selected path. It is not a digest or authenticated source identity; retained
profile SHA-256 and release evidence provide the separate exact-byte record.

Load one profile before the operation. Each call to `observe` checks dimensions,
storage and stride through xSAR's checked `FrameView`. RGB matching ignores
alpha and row padding. No capture is retained or rescaled. Factorised glyphs allow
at most nine integer alignments within one native pixel; scene signatures and
whole-identity RGB templates have no alignment search. Coordinates remain native.

`Unknown` means neither complete scene signature matched. Invalid frame layout,
contradictory board/dialog signatures, unknown or ambiguous card/pile evidence
on a supported board, or duplicate visible identities return a bounded error.
They never silently supply a card, an empty pile or an input target.

### Undo All readiness

Positive Solver-OFF and scene evidence establish the supported board independently
of Undo All availability. On an untouched board, disabled or unmatched Undo All
produces `undo_all: None`; it does not prevent card classification. `Some` is
returned only when the current frame matches the profile's **enabled** Undo All
signature. Its point is fixed at `(1320, 978)`.

The controller must require `Some` on the freshly observed exhausted board before
clicking Undo All. It must not reuse an earlier point across frames or contexts.
Historical images **01–18** match the enabled icon. Images **19 and 20** match
the disabled icon byte-for-byte in its calibrated bounds; the earlier evidence
inventory's “01–19 enabled” label was incorrect. Image 19 remains Solver ON, so
Solver state and Undo availability must not be conflated.

## Strict profile schema 1

All listed fields are required. Additional, missing or duplicate fields at every
object level refuse; escaped aliases of duplicate keys also refuse. Arrays must
have the exact lengths or explicit bounds below. No schema migration, defaults,
numeric-string conversion, null substitution or unknown-field compatibility is
implemented.

### Root object

| Field | Required value or shape |
| --- | --- |
| `schema_version` | Integer `1` |
| `profile_id` | 1–64 ASCII letters, digits, `-` or `_` |
| `game` | `"pyramid"` |
| `artwork` | `"CLASSIC BLUE"` |
| `dimensions` | Exactly `[1920, 1080]` |
| `tableau_reader` | Complete card-reader object |
| `stock_reader` | Complete card-reader object |
| `board` | Board-signature object |
| `dialog` | Undo All confirmation-signature object |

All tableau positions use one reader; LEFT and RIGHT use the other. A reader can
use independently labelled rank/suit glyphs, or a complete 52-identity RGB bank.
Do not mix the two object shapes. Native evidence currently uses **factorised
glyphs**, not 52 synthetic combinations. Sharing a glyph bank across the two
settings does not claim that every possible stock identity has been observed.

### Factorised card-reader object

| Field | Required value or shape |
| --- | --- |
| `rank_roi` | Card-relative `[x,y,width,height]` |
| `suit_roi` | Separate, non-overlapping card-relative ROI |
| `ranks` | Exactly 13 objects, each with `label` and `variants` |
| `suits` | Exactly four objects, each with `label` and `variants` |
| `rank_max_distance` | Integer 0–24, inclusive occupancy XOR count |
| `suit_max_distance` | Integer 0–8, inclusive occupancy XOR count |
| `alignment` | Integer 0 or 1, bounding both native coordinate translations |

Each `label` is a canonical rank or suit string, with no omissions or duplicates.
Each rank has 1–8 actual native variants; each suit has 1–16. A variant is an RGB
sample object, with tolerance **zero** and an all-`1` mask: negative space cannot
be excluded to make a weak glyph match. RGB samples remain unchanged native
pixels with their original capture hash. Normalisation is a comparison operation,
not fabricated red/black artwork or composition of unobserved complete cards.

The current measured native configuration uses rank ROI `[2,6,27,24]`, suit ROI
`[5,30,20,16]`, distances 24 and 8, and alignment 1. The historical 18-high suit
crop included six dark pixels from the adjacent King illustration in the new
LEFT King-of-diamonds capture. The 16-high crop retains all 90 strong red diamond
pixels and excludes that artwork; the mixed-colour guard remains unchanged.
Occupancy is
`min(R,G,B) < 128`. Compare the complete ROI, including white negative space,
at each allowed integer translation. Accept **one qualifying label**, not merely
the nearest candidate. Several variants of the same label are allowed; multiple
qualifying labels are an explicit ambiguity error. Identical normalised templates
with different labels refuse loading; overlapping tolerance regions still refuse
when they produce actual runtime ambiguity.

Require at least one third of the observed ROI to be paper (all RGB channels at
least 180) and at least eight strong ink pixels. After independently matching rank
and suit, both actual glyphs must agree with that suit's colour: red has R at
least 160 and R−G/R−B at least 80; black has all channels at most 100. Require at
least eight expected-colour pixels and no opposite strong-colour pixels in each
qualifying glyph. Anti-aliased uncertain colours are not themselves colour proof.

The historical 46-observation study trained on 30 tableau rank variants and 29
tableau suit variants. Its sixteen held-out LEFT observations had worst correct
distances 22 (rank) and 6 (suit); nearest wrong labels were at least 44 and 22
respectively across that set. These are historical evidence-set measurements,
not universal guarantees or probabilistic confidence. That historical set lacked
LEFT ranks 2/4/5/6/9 and LEFT diamonds.

The newly supplied start and exhausted originals add 58 labelled observations,
including **LEFT King of diamonds** and **RIGHT Queen of diamonds**. LEFT diamonds
are therefore no longer absent from the retained evidence. Actual rasterisation
variants from the current start frame extend the native bank to 33 rank and 35
suit variants without increasing the XOR limits or alignment bound. Any such
start-frame crop used for calibration,
including the King-of-diamonds suit, is training evidence, not held out. The
sixteen historical LEFT observations and exhausted RIGHT Queen of diamonds remain
held out of template extraction. LEFT ranks 2/4/5/6/9 remain unobserved in this
retained set; recognising one LEFT diamond does not establish every LEFT identity.

The retained `factorised-glyphs.json` is labelled **not a runtime profile**. It
contains native component evidence, not input authority. Tests adapt only its
labelled RGB samples into the strict component schema. A complete runtime profile
must separately include all native board and modal signatures; component evidence
alone never bypasses `ReconVision::load` validation.

### Whole-identity RGB card-reader object

| Field | Required value or shape |
| --- | --- |
| `roi` | `[x, y, width, height]`, relative to each native card origin |
| `templates` | Exactly 52 template objects, one for every standard identity |

Tableau ROIs must fit the common exposed `140×53` top strip; stock ROIs must fit
the `139×187` pile face. A crop must contain enough **actual rank and suit**
evidence to discriminate all 52 identities. If the necessary glyph is occluded,
do not claim that this schema can recognise it. Wider native investigation or a
reviewed crop/profile extension is then needed.

Each template object has exactly:

| Field | Required value or shape |
| --- | --- |
| `rank` | `A`, `2`–`10`, `J`, `Q` or `K`, as a string |
| `suit` | `clubs`, `diamonds`, `hearts` or `spades` |
| `samples` | RGB sample object described below |

This alternate format is supported for genuinely complete native identity banks;
it is not used to fabricate combinations from the recovered component evidence.
Template order has no identity meaning. All identities must be explicitly
labelled once; a missing suit or rank is not filled by elimination. Each pair of
templates must have at least one **jointly selected** channel whose reference
values differ by more than the sum of their tolerances. This proves that their
accepted RGB ranges cannot overlap. Two different labels with indistinguishable
pixels, or masks with no separating common evidence, are rejected at load time.
Runtime matching still rejects more than one candidate defensively.

### RGB sample object

| Field | Required value or shape |
| --- | --- |
| `rgb` | Lower-case hexadecimal, six digits per row-major pixel, RGB order |
| `mask` | One `0` or `1` per row-major pixel; `1` selects that RGB constraint |
| `tolerance` | Integer 0–24, inclusive per-channel absolute difference |
| `source_sha256` | Exactly 64 lower-case hexadecimal digits |

The RGB string includes excluded pixels too; its length is exactly
`width × height × 6`, and mask length is exactly `width × height`. Alpha is not
encoded. Masks must select at least 32 pixels and have at least 64 levels of
selected mean-RGB contrast. A card mask additionally needs at least eight paper
pixels (all channels at least 180) and eight ink pixels (any channel at most
100). For absolute patches and whole-identity RGB matching, every selected pixel
must match within channel tolerance. Factorised readers instead use the separately
bounded occupancy XOR rule described above; they do not interpret `tolerance`
as a mismatch allowance. Neither mode uses a nearest-candidate fallback or an
invented confidence percentage.

`source_sha256` records the claimed original capture. The loader checks its
syntax, **not its truth**: it does not open attachments or verify hashes against
external image bytes. Keep the originals, label review and extraction record
with calibration evidence. The profile is a trusted operational configuration;
someone able to replace it can relabel templates or assert unrelated signatures.
Schema validation cannot prove that an image depicts the declared card or dialog.

### Absolute patch object

Every scene, control and empty-pile patch has exactly:

| Field | Required value or shape |
| --- | --- |
| `bounds` | Positive half-open native `[x, y, width, height]` |
| `samples` | RGB sample object |

Each patch contains 32–4096 pixels before masking. Coordinates and exclusive
edges are checked before matching; empty, overflowing and out-of-frame bounds
refuse. All profiles use the same mask/tolerance requirements as card templates,
apart from the additional card paper/ink requirement and the deliberately
domain-specific **16-level** contrast floor for the two dimmed modal backdrop
patches. The captured modal dims white to RGB levels around 51; requiring 64
levels there would reject real evidence. All other patches keep the 64-level
contrast floor. Do not alter native pixels to manufacture contrast.

### Board-signature object

| Field | Required value or shape |
| --- | --- |
| `scene` | 3–8 non-overlapping absolute supported-scene patches |
| `solver_off` | Positive inactive-Solver patch inside `(585,959,35,36)` |
| `undo_all` | Positive **enabled** Undo All patch inside `(1301,959,38,38)` |
| `left_empty` | Positive empty-LEFT patch inside `(759,678,139,187)` |
| `right_empty` | Positive empty-RIGHT patch inside `(1022,678,139,187)` |

The enabled Undo All patch must geometrically cover `(1320,978)`. Its match is
not included in the initial board-signature decision. Scene patches must be
chosen to discriminate the supported Pyramid board and artwork, not changing
card identities, Solver-on HALOs or generic felt/gold alone. Do not include the
enabled Undo All icon inside the mandatory scene patches: the untouched board
may correctly have it disabled.

A pile is empty only when its positive-empty patch matches **and no card
template matches**. A matching card and matching empty patch are ambiguous.
Neither match is unknown, not empty. The initial RIGHT-empty requirement and
subsequent `RIGHT == previous LEFT` rule are controller policy, not inferred here.

### Undo All confirmation-signature object

| Field | Required value or shape |
| --- | --- |
| `bounds` | Native dialog rectangle inside `(300,100,1320,880)` |
| `panel` | Measured panel/frame patch inside that rectangle |
| `title` | Distinctive Undo All title patch inside that rectangle |
| `body` | Distinctive confirmation text/content patch inside that rectangle |
| `cancel` | Positive Cancel-control patch inside that rectangle |
| `ok` | Positive OK-control patch inside that rectangle |
| `ok_point` | `[x,y]`, strictly inside the matched OK patch's half-open bounds |
| `backdrop` | Exactly two native dimmed-background patches outside the dialog |

All seven component rectangles must be pairwise non-overlapping. All seven
patterns must match the **same current frame** before `UndoConfirmation` is
returned. A generic gold rectangle, OK caption alone, incomplete panel or stale
point does not pass. The body/title labels express required calibration meaning;
their actual semantic discrimination must be established from positive and
negative native fixtures. No numerical mask check can prove that meaning.

### Supplied native modal calibration

The verified modal occupies `[531,367,858,330]`; its confirmed OK point is
`[792,630]`. It has an unchecked “Don't show this again” checkbox, which is not
an input target and is never clicked. The actual bundled profile uses:

| Component | Native bounds |
| --- | --- |
| Panel/artwork | `[535,400,64,64]` |
| Undo All title | `[729,417,178,22]` |
| Reset-current-board text | `[852,474,144,28]` |
| Cancel caption | `[1075,615,119,31]` |
| OK caption/interior | `[763,611,65,38]` |
| Dimmed fixed Menu control | `[8,41,49,49]` |
| Dimmed fixed fullscreen control | `[1868,43,41,40]` |

These are exact integer-coordinate crops, not rescaled or synthesised templates.
Every patch fits the 4096-pixel cap. The panel/artwork crop is deliberately inside
the panel: a rounded-corner crop can include varying cards behind the modal and
must not be used as invariant dialog evidence. The two backdrop patches avoid
card identities, score, XP, taskbar, cursor and the desktop overlay. Native
regressions replace variable tableau/stock background pixels and require the
same dialog result; removing any required component must refuse confirmation.

## Logical geometry

Tableau positions are exported **top to bottom, left to right**, rows 0–6.
The origin table is explicit and separate from the legacy CV priority ordering:

| Logical row | Native y | Native x origins, left to right |
| --- | --- | --- |
| 0 | 112 | 890 |
| 1 | 165 | 790, 990 |
| 2 | 218 | 690, 890, 1091 |
| 3 | 272 | 589, 790, 990, 1191 |
| 4 | 325 | 489, 690, 890, 1091, 1291 |
| 5 | 378 | 389, 589, 790, 990, 1191, 1392 |
| 6 | 432 | 288, 489, 690, 890, 1091, 1291, 1492 |

LEFT and RIGHT origins are `(759,678)` and `(1022,678)`. Legacy Move/pile target
indices and bottom-to-top CV scan indices are never used as logical deal slots.
No old controller, geometry constant, transport or input policy is changed.

## Bounds and diagnostics

1. Only regular-file profiles are accepted. Path metadata is checked before
   opening, and the opened handle is checked again. This rejects ordinary
   directory/FIFO/device mistakes, not adversarial pathname races. Symlinks to
   regular files are allowed. Filesystem operations have no hard time deadline.
2. The encoded profile cap is **2,097,152 bytes**. Loading reads at most one byte
   beyond that cap to detect growth/oversize before allocating a JSON value tree.
3. Structural preflight permits at most 16 nesting levels, 16,384 values, 16
   fields per object, 64 entries per array and 24,576 encoded string-content
   bytes. Decoded keys are at most 32 bytes. Duplicate keys, including escaped
   aliases, are rejected before `serde_json::Value` could discard earlier values.
   Serde then validates the complete JSON grammar and UTF-8.
4. Each whole-identity bank has exactly 52 templates. Each factorised bank has
   at most `13×8 + 4×16 = 168` variants. All patterns have at most 4096 samples;
   factorised glyphs are further constrained to the exposed native face and
   disjoint rank/suit ROIs. There are at most 19 absolute signature patches.
   Pairwise template validation is bounded by these fixed bank sizes.
5. A whole-identity observation compares at most `30×52×4096` sample positions.
   A factorised observation has at most nine alignments per variant, plus colour
   confirmation on at most eight rank and sixteen suit variants for the selected
   labels. A conservative upper bound is `30×(168+24)×9×4096` pixel visits, plus
   bounded scene/pile/control patches. The actual retained native crops/banks are
   much smaller. Early mismatch exits do not replace these worst-case bounds.
6. Byte/collection caps bound input-driven work and allocations; they are not an
   exact RSS guarantee or a wall-clock deadline. The controller's cooperative
   timeout cannot interrupt an already-running classifier or filesystem call.
7. Errors identify a logical tableau index or LEFT/RIGHT, report zero accepted
   identities or multiple candidate labels as applicable, and identify the
   deterministic RGB/XOR acceptance policy. They omit raw JSON, paths, OS error
   chains and arbitrary attacker-controlled text. Acceptance is a rule decision,
   not a probabilistic confidence claim.

The finite controller input/capture/time budgets belong in its own contract,
not in the pixel classifier. This module cannot establish capture freshness,
VM identity, absolute-pointer capability, successful delivery or restoration.

## Required acceptance

The inline unit tests cover strict loading/schema bounds, duplicate keys,
complete bank coverage, card ambiguity/discrimination, fixed logical ordering,
RGB tolerance boundaries, alpha/padded-stride handling, malformed frames,
positive-empty exclusivity, duplicate visible identities, optional Undo readiness,
complete dialog signatures and contradictory scenes. These are synthetic
mechanism regressions only. Separate native regressions exercise all 46 labelled
observations, the exact image-20 28-card tableau plus LEFT Queen of clubs, RIGHT
empty and disabled Undo All; Solver-ON refusal; enabled Undo distinction;
empty-pile appearance; and erased/mixed-colour glyph negatives. Original PNGs are
not modified by these tests. Native integration uses the actual supplied modal,
not a synthetic dialog stub. Separate regressions refuse missing dialog data,
erase each modal component, and change variable background card identities.

Before live use, retain and independently review:

1. Native originals covering every rank and suit label in each component bank,
   with exact original SHA-256, labels, crop settings and extraction provenance.
2. Untouched Solver-OFF boards; enabled and disabled Undo All; initial RIGHT
   empty; actual end-of-stock LEFT empty; and successive LEFT/RIGHT observations.
3. The complete Undo All confirmation, both control targets and dimmed backdrop.
   Include unrelated gold/OK dialogs, Solver-on boards, altered artwork,
   animation, occlusion/cursor interference and missing-card negatives.
4. Expected 28-slot logical identities and visible pile identities for each
   positive frame. Check exact identities, not only counts or plausible ranks.
5. Fresh-frame, STOP/context and full restoration acceptance through the actual
   controller. Missing evidence leaves live operation unavailable; do not
   substitute synthetic test patterns or assumptions to bypass that boundary.

## Code Analysis

The profile file is the configuration trust boundary; capture bytes are the
observation boundary. Both are checked, but label/provenance truth still requires
native evidence review. The module uses safe Rust and checked xSAR frame/ROI
access, with no input transport, unsafe code, image resizing or stateful replay.
Explicit allocation/work caps prevent arbitrary-sized calibration or matching
searches. Strict false-negative behaviour is intentional: an unrecognised font,
cursor, rasterisation variant, scene or dialog stops rather than selecting a
nearest identity or granting fallback input authority.
