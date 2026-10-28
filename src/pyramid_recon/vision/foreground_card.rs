//! Native-size, paper-relative corner ink, independent of board artwork.
//!
//! Complete font families compete without rescaling or deck-based inference.
//! This private capability never changes serialized/custom comparison policies.

use super::{Card, FrameView, GlyphReader, Pattern, PixelRect, Rank, Suit};

/// Bounded native corner, including complete retro Q tails and suit stems.
const CORNER_WIDTH: usize = 31;


/// Last sampled row is card-relative52; body artwork has a separate role.
const CORNER_HEIGHT: usize = 49;


/// Exclusive label boundary, preserving measured suit antialias ink at card row50.
/// Complete components crossing into lower illustration rows are rejected, never clipped.
const SUIT_BOTTOM: u8 = 51;


/// Exactly one native-pixel translation in half-pixel steps; never scaling.
const SHIFTS: [i32; 5] = [0, -1, 1, -2, 2];


/// Complete-corner rank quality, distinct from frozen paper_ink_l1_v2.
const RANK_ERROR: u64 = 220_000;


/// Complete-corner suit quality; unlike v2 this includes full native strokes.
const SUIT_ERROR: u64 = 200_000;


/// A different rank must remain separated within each complete font family.
const RANK_GAP: u64 = 80_000;


/// Same-colour suits compete after identical half-pixel registration.
const SUIT_GAP: u64 = 40_000;


/// Two exclusive printed ink families, rather than a felt colour palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Colour {
    /// Neutral or sufficiently dark printed ink.
    Black,
    /// Positively red printed ink.
    Red,
}


/// One measured native stroke pixel and its independent strong-seed fact.
#[derive(Clone, Copy, Debug)]
struct Point {
    /// Card-relative horizontal coordinate.
    x: u8,
    /// Card-relative vertical coordinate.
    y: u8,
    /// Paper-relative ink strength, not an RGB template value.
    ink: u8,
    /// The pixel satisfies the original strong colour predicate.
    strong: bool,
}


/// An eight-connected same-colour component with at least two strong seeds.
#[derive(Clone, Debug)]
struct Component {
    /// Printed colour shared by every retained pixel.
    colour: Colour,
    /// Complete bounded component, including seed-supported antialias pixels.
    points: Vec<Point>,
}


/// Translation-normalised native raster; dimensions are never rescaled.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Glyph {
    /// Original complete stroke width.
    width: usize,
    /// Original complete stroke height.
    height: usize,
    /// Row-major ink including all internal negative space.
    ink: Vec<u8>,
    /// Exact total native ink mass, invariant under the bounded translation.
    mass: u64,
    /// Nonzero native pixels; negative space is accounted for algebraically.
    nonzero: Vec<(i32, i32, u64)>,
}


impl Glyph {
    /// Preserve complete measured bounds and their internal zero-ink pixels.
    fn from_points(points: &[Point]) -> Option<Self> {
        let left = points.iter().map(|point| point.x).min()?;
        let right = points.iter().map(|point| point.x).max()?;
        let top = points.iter().map(|point| point.y).min()?;
        let bottom = points.iter().map(|point| point.y).max()?;
        let width = usize::from(right - left) + 1;
        let height = usize::from(bottom - top) + 1;
        let mut ink = vec![0; width * height];
        for point in points {
            let index = usize::from(point.y - top) * width + usize::from(point.x - left);
            *ink.get_mut(index)? = point.ink;
        }
        let mass = ink.iter().copied().map(u64::from).sum();
        let nonzero = ink
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                (*value != 0).then_some((
                    (index % width) as i32,
                    (index / width) as i32,
                    u64::from(*value) * 4,
                ))
            })
            .collect();
        Some(Self {
            width,
            height,
            ink,
            mass,
            nonzero,
        })
    }


    /// Extend the raster by zero ink for checked native translation comparison.
    fn sample(&self, x: i32, y: i32) -> u64 {
        let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y)) else {
            return 0;
        };
        if x >= self.width || y >= self.height {
            return 0;
        }
        self.ink
            .get(y * self.width + x)
            .copied()
            .map_or(0, u64::from)
    }


    /// Sample one translated native raster with exact quarter-strength weights.
    #[cfg(test)]
    fn translated(&self, x: i32, y: i32, dx: i32, dy: i32) -> u64 {
        let x2 = 2 * x - dx;
        let y2 = 2 * y - dy;
        let left = x2.div_euclid(2);
        let top = y2.div_euclid(2);
        let fx = u64::try_from(x2.rem_euclid(2)).unwrap_or(0);
        let fy = u64::try_from(y2.rem_euclid(2)).unwrap_or(0);
        self.sample(left, top) * (2 - fx) * (2 - fy)
            + self.sample(left + 1, top) * fx * (2 - fy)
            + self.sample(left, top + 1) * (2 - fx) * fy
            + self.sample(left + 1, top + 1) * fx * fy
    }
}


/// A complete same-colour rank and suit under one bounded separation witness.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Corner {
    /// Rank raster, including both components of Ten.
    rank: Glyph,
    /// Suit raster, including its native stem/tail.
    suit: Glyph,
    /// Rank and suit must independently establish this same colour.
    colour: Colour,
    /// The separator does not cut any eligible connected component.
    whole_separator: bool,
}


/// Exact shape distance; no floating-point acceptance decisions are used.
#[derive(Clone, Copy, Debug)]
struct Distance {
    /// Sum of absolute quarter-strength ink differences.
    difference: u64,
    /// Sum of maximum quarter-strength ink, including negative-space conflicts.
    mass: u64,
}


impl Distance {
    /// Order rational distances without rounding away a narrow ambiguity.
    fn less(self, other: Self) -> bool {
        u128::from(self.difference) * u128::from(other.mass)
            < u128::from(other.difference) * u128::from(self.mass)
    }


    /// Apply an inclusive absolute quality limit to the exact fraction.
    fn within(self, ppm: u64) -> bool {
        self.mass != 0
            && u128::from(self.difference) * 1_000_000 <= u128::from(self.mass) * u128::from(ppm)
    }


    /// Require an exact different-label gap after all variants have competed.
    fn separated(self, second: Self, ppm: u64) -> bool {
        let first = u128::from(self.difference) * u128::from(second.mass);
        let next = u128::from(second.difference) * u128::from(self.mass);
        next >= first
            && (next - first) * 1_000_000
                >= u128::from(self.mass) * u128::from(second.mass) * u128::from(ppm)
    }
}


/// Measured printed ink used only with its complete native font family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InkPolicy {
    /// Existing neutral black, aged blue and red printed glyphs.
    Standard,
    /// Underwater brown Spades/Clubs and the existing red glyph predicate.
    Sepia,
}


/// Maximum brown-print channel in the retained Underwater strong-ink cores.
const UNDERWATER_BROWN_MAX: u8 = 160;


/// Blue-channel ceiling for the measured Underwater brown strong cores.
const UNDERWATER_BROWN_BLUE_MAX: u8 = 110;


/// Measured brown compatibility allows at most this red-minus-green difference.
const UNDERWATER_BROWN_RG_MAX: u8 = 64;


/// Minimum green-minus-blue separation from neutral print in this native family.
const UNDERWATER_BROWN_GB_MIN: u8 = 12;


/// Maximum green-minus-blue difference retaining the measured brown antialias fringe.
const UNDERWATER_BROWN_GB_MAX: u8 = 96;


/// The complete native rank ends before this measured Underwater suit boundary.
const UNDERWATER_SEPARATOR: u8 = 30;


/// Exclusive card-relative x bound retaining both measured Underwater Ten digits.
const UNDERWATER_RANK_RIGHT: u8 = 34;


/// Both complete independent label banks for one coherent font family.
#[derive(Clone, Debug)]
struct Family {
    /// All thirteen rank labels and measured raster variants.
    ranks: Vec<(Rank, Vec<Glyph>)>,
    /// All four suit labels and measured raster variants.
    suits: Vec<(Suit, Vec<Glyph>)>,
    /// Complete measured exemplars enable this family's explicit native-size bound.
    native_size: bool,
    /// Smaller Western labels use whole-component separation before matching.
    western: bool,
    /// Printed ink and native component extents belonging to this family alone.
    ink: InkPolicy,
    /// One measured Five-of-spades appearance requires its rank and an unsplit separator.
    separated_spade: Option<Glyph>,
}


/// Measured native Western antialias bounds vary by at most two pixels per side length.
const WESTERN_NATIVE_SIZE_MARGIN: usize = 2;


/// Native Underwater red/brown Eight envelopes differ by two rows in the originals.
const UNDERWATER_NATIVE_SIZE_MARGIN: usize = 2;


impl Family {
    /// Compare the measured alternate only when no candidate stroke was cut.
    fn suit_bank(&self, corner: &Corner, rank: Option<Rank>) -> Vec<(Suit, Vec<Glyph>)> {
        let mut suits: Vec<_> = self
            .suits
            .iter()
            .filter(|(suit, _)| red_suit(*suit) == (corner.colour == Colour::Red))
            .cloned()
            .collect();
        if rank == Some(Rank::Five)
            && corner.whole_separator
            && let Some(spade) = &self.separated_spade
            && let Some((_, variants)) = suits.iter_mut().find(|(suit, _)| *suit == Suit::Spades)
        {
            variants.push(spade.clone());
        }
        suits
    }


    /// Keep each calibrated family's complete native raster bounds after competition.
    fn native_tolerance(&self) -> Option<usize> {
        if self.ink == InkPolicy::Sepia {
            Some(UNDERWATER_NATIVE_SIZE_MARGIN)
        } else if self.western {
            Some(WESTERN_NATIVE_SIZE_MARGIN)
        } else {
            self.native_size.then_some(1)
        }
    }
}


/// Private complete-family reader enabled only by the bundled native constructor.
#[derive(Clone, Debug)]
pub(super) struct Reader {
    /// Families remain separate; a rank cannot borrow an unrelated font's suit.
    families: Vec<Family>,
}


impl Reader {
    /// Compile separate complete Classic, Retro, Western and Underwater families.
    pub(super) fn new(classic: &GlyphReader) -> Result<Self, String> {
        let ranks: Vec<(Rank, Vec<Glyph>)> = classic
            .ranks
            .iter()
            .map(|(label, variants)| {
                Ok((
                    *label,
                    variants
                        .iter()
                        .map(template_glyph)
                        .collect::<Result<_, String>>()?,
                ))
            })
            .collect::<Result<_, String>>()?;
        let suits: Vec<(Suit, Vec<Glyph>)> = classic
            .suits
            .iter()
            .map(|(label, variants)| {
                Ok((
                    *label,
                    variants
                        .iter()
                        .map(template_glyph)
                        .collect::<Result<_, String>>()?,
                ))
            })
            .collect::<Result<_, String>>()?;
        Ok(Self {
            families: vec![
                Family {
                    separated_spade: Some(measured_classic_spade(&ranks)?),
                    ranks,
                    suits,
                    native_size: false,
                    western: false,
                    ink: InkPolicy::Standard,
                },
                retro_family()?,
                wild_west_family()?,
                underwater_family()?,
            ],
        })
    }


    /// Read only these immutable pixels; disagreement never becomes input authority.
    pub(super) fn recognise(
        &self,
        view: FrameView<'_>,
        origin: (u32, u32),
    ) -> Result<Option<Card>, String> {
        let pixels = corner_pixels(view, origin)?;
        let mut recognised = None;
        for family in &self.families {
            for corner in corners_with_ink(&pixels, None, None, family.western, family.ink) {
                let Some(rank) = matching_label_with_tolerance(
                    &corner.rank,
                    &family.ranks,
                    RANK_ERROR,
                    RANK_GAP,
                    family.native_tolerance(),
                ) else {
                    continue;
                };
                let suit_bank = family.suit_bank(&corner, Some(rank));
                let suit = matching_label_with_tolerance(
                    &corner.suit,
                    &suit_bank,
                    SUIT_ERROR,
                    SUIT_GAP,
                    family.native_tolerance(),
                );
                let Some(suit) = suit else {
                    continue;
                };
                let card = Card::new(rank, suit);
                if recognised.is_some_and(|previous| previous != card) {
                    return Err("Native corner identity is ambiguous across complete font families/separators".to_owned());
                }
                recognised = Some(card);
            }
        }
        Ok(recognised)
    }


    /// Explain the same immutable corner without recapturing or authorising input.
    pub(super) fn failure_details(&self, view: FrameView<'_>, origin: (u32, u32)) -> String {
        let describe = || -> Result<String, String> {
            let pixels = corner_pixels(view, origin)?;
            let paper = paper_reference(&pixels);
            let components = components(&pixels, None);
            let candidates: Vec<_> = self
                .families
                .iter()
                .map(|family| corners_with_ink(&pixels, None, None, family.western, family.ink))
                .collect();
            let mut details = vec![format!(
                "{}; paper_reference={paper:?}; components={}; corner_candidates={}",
                Self::policy(),
                components.len(),
                candidates.iter().map(Vec::len).sum::<usize>()
            )];
            if candidates.iter().all(Vec::is_empty) {
                details.push(
                    "extraction rejected: no complete seed-supported same-colour rank/suit corner"
                        .to_owned(),
                );
            }
            for (family, candidates) in self.families.iter().zip(candidates) {
                for (candidate, corner) in candidates.iter().enumerate() {
                    let name = if family.ink == InkPolicy::Sepia {
                        "underwater"
                    } else if family.western {
                        "wild-west"
                    } else if family.native_size {
                        "retro"
                    } else {
                        "classic"
                    };
                    let rank = matching_label_with_tolerance(
                        &corner.rank,
                        &family.ranks,
                        RANK_ERROR,
                        RANK_GAP,
                        family.native_tolerance(),
                    );
                    let suits = family.suit_bank(corner, rank);
                    details.push(format!(
                        "candidate={candidate} family={name} colour={:?}; rank {}; suit {}",
                        corner.colour,
                        label_details(
                            &corner.rank,
                            &family.ranks,
                            RANK_ERROR,
                            RANK_GAP,
                            family.native_tolerance()
                        ),
                        label_details(
                            &corner.suit,
                            &suits,
                            SUIT_ERROR,
                            SUIT_GAP,
                            family.native_tolerance()
                        )
                    ));
                }
            }
            Ok(details.join(" | "))
        };
        describe()
            .unwrap_or_else(|error| format!("{}; extraction rejected: {error}", Self::policy()))
    }


    /// State comparison bounds for saved diagnostics without implying probability.
    pub(super) fn policy() -> &'static str {
        "native_corner_ink_v1; paper-relative seeded components; complete coherent font families; native half-pixel translation/no scaling; rank<=220000ppm/gap>=80000; suit<=200000ppm/same-colour gap>=40000; unique family/cut identity"
    }
}


/// Read exactly one checked native corner for both decisions and refusal diagnostics.
fn corner_pixels(view: FrameView<'_>, origin: (u32, u32)) -> Result<Vec<[u8; 3]>, String> {
    let left = origin
        .0
        .checked_add(3)
        .ok_or_else(|| "Corner x overflow".to_owned())?;
    let top = origin
        .1
        .checked_add(4)
        .ok_or_else(|| "Corner y overflow".to_owned())?;
    let bounds = PixelRect::new(left, top, CORNER_WIDTH as u32, CORNER_HEIGHT as u32);
    view.checked_bounds(bounds)
        .map_err(|error| format!("Card corner: {error:?}"))?;
    let mut pixels = Vec::with_capacity(CORNER_WIDTH * CORNER_HEIGHT);
    for y in top..top + CORNER_HEIGHT as u32 {
        for x in left..left + CORNER_WIDTH as u32 {
            pixels.push(
                view.pixel_rgb(x, y)
                    .ok_or_else(|| "Card corner pixel exceeds the checked frame".to_owned())?,
            );
        }
    }
    Ok(pixels)
}


/// Establish the printed red family independently of rank value.
fn red_suit(suit: Suit) -> bool {
    matches!(suit, Suit::Diamonds | Suit::Hearts)
}


/// Measure all translated pixels including complete native negative space.
fn distance(actual: &Glyph, expected: &Glyph) -> Distance {
    if actual.width == expected.width
        && actual.height == expected.height
        && actual.ink == expected.ink
    {
        return Distance {
            difference: 0,
            mass: actual.mass * 4,
        };
    }
    let total = (actual.mass + expected.mass) * 4;
    let lower = Distance {
        difference: actual.mass.abs_diff(expected.mass),
        mass: actual.mass.max(expected.mass),
    };
    let mut best = Distance {
        difference: 1,
        mass: 1,
    };
    for dy in SHIFTS {
        for dx in SHIFTS {
            let ox = (-dx).div_euclid(2);
            let oy = (-dy).div_euclid(2);
            let fx = (-dx).rem_euclid(2);
            let fy = (-dy).rem_euclid(2);
            let mut overlap = 0;
            for &(x, y, ink) in &actual.nonzero {
                let x = x + ox;
                let y = y + oy;
                let template = match (fx, fy) {
                    (0, 0) => expected.sample(x, y) * 4,
                    (1, 0) => (expected.sample(x, y) + expected.sample(x + 1, y)) * 2,
                    (0, 1) => (expected.sample(x, y) + expected.sample(x, y + 1)) * 2,
                    _ => {
                        expected.sample(x, y)
                            + expected.sample(x + 1, y)
                            + expected.sample(x, y + 1)
                            + expected.sample(x + 1, y + 1)
                    }
                };
                overlap += ink.min(template);
            }
            // Sum|a-b|=sum(a)+sum(b)-2sum(min); sum(max)=sum(a)+sum(b)-sum(min).
            // Complete padding makes both ink masses translation-invariant.
            let Some(difference) = total.checked_sub(2 * overlap) else {
                return Distance {
                    difference: 1,
                    mass: 1,
                };
            };
            let score = Distance {
                difference,
                mass: total - overlap,
            };
            if score.mass != 0 && score.less(best) {
                best = score;
            }
            if !lower.less(best) {
                return best;
            }
        }
    }
    best
}


/// Find labels before applying geometry; competitors must never be size-filtered away.
#[cfg(test)]
fn matching_label<T: Copy + Eq>(
    actual: &Glyph,
    bank: &[(T, Vec<Glyph>)],
    max_error: u64,
    gap: u64,
    native_size: bool,
) -> Option<T> {
    matching_label_with_tolerance(actual, bank, max_error, gap, native_size.then_some(1))
}


/// Apply family-measured native raster bounds only after full label competition.
fn matching_label_with_tolerance<T: Copy>(
    actual: &Glyph,
    bank: &[(T, Vec<Glyph>)],
    max_error: u64,
    gap: u64,
    tolerance: Option<usize>,
) -> Option<T> {
    let scores = ranked_labels(actual, bank);
    let (label, best, template) = *scores.first()?;
    let (_, second, _) = *scores.get(1)?;
    if !best.within(max_error) || !best.separated(second, gap) {
        return None;
    }
    if tolerance.is_some_and(|limit| {
        actual.width.abs_diff(template.width) > limit
            || actual.height.abs_diff(template.height) > limit
    }) {
        return None;
    }
    Some(label)
}


/// Rank all different labels before geometry, shared by decisions and diagnostics.
fn ranked_labels<'a, T: Copy>(
    actual: &Glyph,
    bank: &'a [(T, Vec<Glyph>)],
) -> Vec<(T, Distance, &'a Glyph)> {
    let mut scores = Vec::with_capacity(bank.len());
    for (label, variants) in bank {
        let mut best: Option<(Distance, &Glyph)> = None;
        for template in variants {
            let lower = Distance {
                difference: actual.mass.abs_diff(template.mass),
                mass: actual.mass.max(template.mass),
            };
            if best.is_some_and(|(previous, _)| !lower.less(previous)) {
                continue;
            }
            let score = distance(actual, template);
            if best.is_none_or(|(previous, _)| score.less(previous)) {
                best = Some((score, template));
            }
            if score.difference == 0 {
                break;
            }
        }
        if let Some((score, template)) = best {
            scores.push((*label, score, template));
        }
    }
    scores.sort_by(|left, right| {
        if left.1.less(right.1) {
            std::cmp::Ordering::Less
        } else if right.1.less(left.1) {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });
    scores
}


/// Report exact-distance quality, different-label gap and winning native-size guards.
fn label_details<T: Copy + Eq + std::fmt::Debug>(
    actual: &Glyph,
    bank: &[(T, Vec<Glyph>)],
    error: u64,
    gap: u64,
    tolerance: Option<usize>,
) -> String {
    let scores = ranked_labels(actual, bank);
    let (Some((label, best, template)), Some((other, next, _))) = (scores.first(), scores.get(1))
    else {
        return "no complete label competition".to_owned();
    };
    let size = tolerance.is_none_or(|limit| {
        actual.width.abs_diff(template.width) <= limit
            && actual.height.abs_diff(template.height) <= limit
    });
    format!(
        "best={label:?} error={}/{} other={other:?} error={}/{}; quality={} limit={error}ppm; gap={} required={gap}ppm; native_size={size} actual={}x{} reference={}x{}",
        best.difference,
        best.mass,
        next.difference,
        next.mass,
        best.within(error),
        best.separated(*next, gap),
        actual.width,
        actual.height,
        template.width,
        template.height
    )
}


/// Estimate card paper from a bounded lower median, independently for each face.
fn paper_reference(pixels: &[[u8; 3]]) -> Option<u8> {
    let mut paper: Vec<_> = pixels
        .iter()
        .map(|rgb| *rgb.iter().min().unwrap_or(&0))
        .filter(|value| *value >= 180)
        .collect();
    if paper.len() < pixels.len() / 3 || paper.is_empty() {
        return super::card_paper::aged_reference(pixels);
    }
    paper.sort_unstable();
    paper.get((paper.len() - 1) / 2).copied()
}


/// Identify neutral or existing aged-blue support for a complete black component.
/// Connected dark chromatic contours remain intact when neutral seeds anchor the glyph.
fn black_compatible([r, g, b]: [u8; 3], aged: bool) -> bool {
    r.max(g).max(b) - r.min(g).min(b) <= 40
        || (aged && b >= g && g >= r && b.saturating_sub(r) <= 96 && g.saturating_sub(r) <= 64)
}


/// Extract seed-supported same-colour ink, excluding disconnected single-pixel art.
fn components(pixels: &[[u8; 3]], paper_override: Option<u8>) -> Vec<Component> {
    components_with_ink(pixels, paper_override, InkPolicy::Standard)
}


/// Identify the measured brown print without admitting orange/red glyph cores.
fn brown_compatible([r, g, b]: [u8; 3]) -> bool {
    r >= g
        && g >= b
        && r - g <= UNDERWATER_BROWN_RG_MAX
        && (UNDERWATER_BROWN_GB_MIN..=UNDERWATER_BROWN_GB_MAX).contains(&(g - b))
}


/// Grow bounded whole components using the selected family's printed ink.
fn components_with_ink(
    pixels: &[[u8; 3]],
    paper_override: Option<u8>,
    policy: InkPolicy,
) -> Vec<Component> {
    let Some(paper) = paper_override.or_else(|| paper_reference(pixels)) else {
        return Vec::new();
    };
    if pixels.len() != CORNER_WIDTH * CORNER_HEIGHT || paper == 0 {
        return Vec::new();
    }
    let aged = paper < 180;
    let background = if aged {
        super::card_paper::aged_channels(pixels)
    } else {
        None
    };
    let [red_bias_green, red_bias_blue] = background.map_or([0, 0], |[r, g, b]| {
        [r.saturating_sub(g), r.saturating_sub(b)]
    });
    let ink: Vec<_> = pixels
        .iter()
        .map(|rgb| {
            let minimum = rgb.iter().min().copied().unwrap_or(255);
            (u16::from(paper.saturating_sub(minimum)) * 255 / u16::from(paper)) as u8
        })
        .collect();
    let mut result = Vec::new();
    for colour in [Colour::Black, Colour::Red] {
        let strong: Vec<_> = pixels
            .iter()
            .map(|[r, g, b]| match colour {
                Colour::Black => match policy {
                    InkPolicy::Standard => (*r).max(*g).max(*b) <= 100,
                    InkPolicy::Sepia => *r <= UNDERWATER_BROWN_MAX
                        && *b <= UNDERWATER_BROWN_BLUE_MAX && brown_compatible([*r, *g, *b]),
                },
                Colour::Red => {
                    (*r >= 160 && r.saturating_sub(*g) >= 80 && r.saturating_sub(*b) >= 80)
                        || (aged
                            && *r >= 96
                            && r.saturating_sub(*g) >= 64
                            && r.saturating_sub(*b) >= 64)
                }
            })
            .collect();
        let mut eligible = vec![false; pixels.len()];
        for (index, [r, g, b]) in pixels.iter().enumerate() {
            let compatible = match colour {
                Colour::Black => match policy {
                    InkPolicy::Standard => black_compatible([*r, *g, *b], aged),
                    InkPolicy::Sepia => brown_compatible([*r, *g, *b]),
                },
                Colour::Red => {
                    *r >= 120
                        && r.saturating_sub(*g) >= red_bias_green.saturating_add(24)
                        && r.saturating_sub(*b) >= red_bias_blue.saturating_add(24)
                }
            };
            if !strong[index] && !(ink[index] >= 32 && compatible) {
                continue;
            }
            let x = index % CORNER_WIDTH;
            let y = index / CORNER_WIDTH;
            eligible[index] = (y.saturating_sub(2)..=(y + 2).min(CORNER_HEIGHT - 1)).any(|ny| {
                (x.saturating_sub(2)..=(x + 2).min(CORNER_WIDTH - 1))
                    .any(|nx| strong[ny * CORNER_WIDTH + nx])
            });
        }
        let mut seen = vec![false; pixels.len()];
        for start in 0..pixels.len() {
            if seen[start] || !eligible[start] {
                continue;
            }
            let mut queue = vec![start];
            seen[start] = true;
            let mut head = 0;
            let mut points = Vec::new();
            let mut supported_strong = 0;
            while let Some(index) = queue.get(head).copied() {
                head += 1;
                let x = index % CORNER_WIDTH;
                let y = index / CORNER_WIDTH;
                supported_strong += usize::from(
                    strong[index]
                        && (colour != Colour::Black || match policy {
                            InkPolicy::Standard => black_compatible(pixels[index], aged),
                            InkPolicy::Sepia => brown_compatible(pixels[index]),
                        }),
                );
                points.push(Point {
                    x: (x + 3) as u8,
                    y: (y + 4) as u8,
                    ink: ink[index],
                    strong: strong[index],
                });
                for ny in y.saturating_sub(1)..=(y + 1).min(CORNER_HEIGHT - 1) {
                    for nx in x.saturating_sub(1)..=(x + 1).min(CORNER_WIDTH - 1) {
                        let next = ny * CORNER_WIDTH + nx;
                        if eligible[next] && !seen[next] {
                            seen[next] = true;
                            queue.push(next);
                        }
                    }
                }
            }
            // Preserve the two-seed minimum and every pixel of supported components.
            if supported_strong >= 2 {
                result.push(Component { colour, points });
            }
        }
    }
    result
}


/// Keep complete bounded glyph groups and every qualifying separator; never clip to fit.
#[cfg(test)]
fn corners(
    pixels: &[[u8; 3]],
    paper_override: Option<u8>,
    reference_cut: Option<u8>,
) -> Vec<Corner> {
    corners_with_layout(pixels, paper_override, reference_cut, false)
}


/// Separate complete Western components; retain the original Classic/Retro layout.
fn corners_with_layout(
    pixels: &[[u8; 3]],
    paper_override: Option<u8>,
    reference_cut: Option<u8>,
    western: bool,
) -> Vec<Corner> {
    corners_with_ink(
        pixels,
        paper_override,
        reference_cut,
        western,
        InkPolicy::Standard,
    )
}


/// Extract complete native glyphs at the measured family separator without resizing.
fn corners_with_ink(
    pixels: &[[u8; 3]],
    paper_override: Option<u8>,
    reference_cut: Option<u8>,
    western: bool,
    policy: InkPolicy,
) -> Vec<Corner> {
    let eligible: Vec<_> = components_with_ink(pixels, paper_override, policy)
        .into_iter()
        .filter(|component| {
            let Some(left) = component.points.iter().map(|point| point.x).min() else {
                return false;
            };
            let right = component
                .points
                .iter()
                .map(|point| point.x)
                .max()
                .map_or(0, |x| x + 1);
            let top = component
                .points
                .iter()
                .map(|point| point.y)
                .min()
                .unwrap_or(0);
            let bottom = component
                .points
                .iter()
                .map(|point| point.y)
                .max()
                .map_or(0, |y| y + 1);
            if policy == InkPolicy::Sepia {
                // Native "10" reaches x32; both digits must remain inside this x34 crop.
                (left >= 3 && right <= UNDERWATER_RANK_RIGHT && top >= 4 && bottom <= 33)
                    || (left >= 4 && right <= 26 && top >= 29 && bottom <= SUIT_BOTTOM)
                    || (left >= 4 && right <= 26 && top >= 4 && bottom <= SUIT_BOTTOM)
            } else if western {
                // Measured complete label extents over both saved Western boards.
                (left >= 3 && right <= 24 && top >= 4 && bottom <= 26)
                    || (left >= 3 && right <= 19 && top >= 27 && bottom <= 45)
            } else {
                (left >= 3 && right <= 29 && top >= 4 && bottom <= 33)
                    || (left >= 4 && right <= 26 && top >= 29 && bottom <= SUIT_BOTTOM)
                    || (left >= 4 && right <= 26 && top >= 4 && bottom <= SUIT_BOTTOM)
            }
        })
        .collect();
    let mut candidates = Vec::new();
    let separators = if western {
        26..=27
    } else if policy == InkPolicy::Sepia {
        UNDERWATER_SEPARATOR..=UNDERWATER_SEPARATOR
    } else {
        30..=33
    };
    for cut in separators {
        if reference_cut.is_some_and(|required| required != cut) {
            continue;
        }
        if western
            && eligible.iter().any(|component| {
                component.points.iter().any(|point| point.y < cut)
                    && component.points.iter().any(|point| point.y >= cut)
            })
        {
            continue;
        }
        let mut rank = Vec::new();
        let mut suit = Vec::new();
        let mut rank_colour = None;
        let mut suit_colour = None;
        let mut mixed = false;
        for component in &eligible {
            for point in &component.points {
                let (target, colour) = if point.y < cut {
                    (&mut rank, &mut rank_colour)
                } else {
                    if point.x >= 26 {
                        mixed = true;
                    }
                    (&mut suit, &mut suit_colour)
                };
                if colour.is_some_and(|previous| previous != component.colour) {
                    mixed = true;
                }
                *colour = Some(component.colour);
                target.push(*point);
            }
        }
        if mixed
            || rank_colour.is_none()
            || rank_colour != suit_colour
            || rank.iter().filter(|point| point.strong).count() < 8
            || suit.iter().filter(|point| point.strong).count() < 8
        {
            continue;
        }
        if let (Some(rank), Some(suit), Some(colour)) = (
            Glyph::from_points(&rank),
            Glyph::from_points(&suit),
            rank_colour,
        ) {
            let whole_separator = !eligible.iter().any(|component| {
                component.points.iter().any(|point| point.y < cut)
                    && component.points.iter().any(|point| point.y >= cut)
            });
            let corner = Corner { rank, suit, colour, whole_separator };
            if !candidates.contains(&corner) {
                candidates.push(corner);
            }
        }
    }
    candidates
}


/// Compile one complete measured Classic spade with its independently recognised Five.
/// The original full corner is retained: no stroke clipping, scaling or changed limits.
fn measured_classic_spade(ranks: &[(Rank, Vec<Glyph>)]) -> Result<Glyph, String> {
    const RGB: &[u8; 4_557] = include_bytes!("foreground_card/classic-spade-158.rgb");
    let pixels: Vec<_> = RGB.as_chunks::<3>().0.to_vec();
    let mut candidates = corners_with_layout(&pixels, None, Some(30), false).into_iter();
    let corner = candidates
        .next()
        .ok_or("Measured Classic spade lacks a complete corner")?;
    if candidates.next().is_some()
        || corner.colour != Colour::Black
        || matching_label_with_tolerance(&corner.rank, ranks, RANK_ERROR, RANK_GAP, None)
            != Some(Rank::Five)
    {
        return Err("Measured Classic spade lacks its coherent Five/black witness".to_owned());
    }
    Ok(corner.suit)
}


/// Convert an unchanged native RGB pattern with its own measured paper reference.
fn template_glyph(pattern: &Pattern) -> Result<Glyph, String> {
    let width = pattern.bounds.width as usize;
    let height = pattern.bounds.height as usize;
    if width > CORNER_WIDTH || height > CORNER_HEIGHT || pattern.pixels.len() != width * height {
        return Err("Native foreground template exceeds its bounded corner".to_owned());
    }
    let raw: Vec<_> = pattern
        .pixels
        .iter()
        .map(|rgb| rgb.unwrap_or([255; 3]))
        .collect();
    let paper = paper_reference(&raw).ok_or_else(|| "Native template lacks paper".to_owned())?;
    let mut pixels = vec![[255; 3]; CORNER_WIDTH * CORNER_HEIGHT];
    for y in 0..height {
        for x in 0..width {
            pixels[y * CORNER_WIDTH + x] = raw[y * width + x];
        }
    }
    let points: Vec<_> = components(&pixels, Some(paper))
        .into_iter()
        .flat_map(|component| component.points)
        .collect();
    Glyph::from_points(&points)
        .ok_or_else(|| "Native foreground template lacks seeded ink".to_owned())
}


/// Load exactly thirteen independently labelled unchanged RGB calibration corners.
fn retro_family() -> Result<Family, String> {
    complete_family(
        include_str!("../../../assets/pyramid-glyphs/retro-corners-v1.json"),
        "Retro",
        false,
    )
}


/// Load complete measured rank/ink-colour variants; thirty-three faces stay held out.
fn wild_west_family() -> Result<Family, String> {
    complete_family(
        include_str!("../../../assets/pyramid-glyphs/wild-west-corners-v1.json"),
        "Wild West",
        true,
    )
}


/// Validate a complete labelled family before any candidate can be recognised.
fn complete_family(text: &str, family: &str, western: bool) -> Result<Family, String> {
    complete_family_with_ink(text, family, western, InkPolicy::Standard)
}


/// Load all thirteen native Underwater ranks and all four measured suits.
fn underwater_family() -> Result<Family, String> {
    complete_family_with_ink(
        include_str!("../../../assets/pyramid-glyphs/underwater-corners-v1.json"),
        "Underwater",
        false,
        InkPolicy::Sepia,
    )
}


/// Compile a complete bounded family using only its own colour/layout policy.
fn complete_family_with_ink(
    text: &str,
    family: &str,
    western: bool,
    ink: InkPolicy,
) -> Result<Family, String> {
    let data: serde_json::Value = serde_json::from_str(text).map_err(|error| error.to_string())?;
    let entries = data["corners"]
        .as_array()
        .ok_or_else(|| format!("{family} corners must be an array"))?;
    if data["schema"] != 1
        || data["corner_size"] != serde_json::json!([31, 49])
        || entries.len() != if western { 25 } else { 13 }
    {
        return Err(format!(
            "{family} family requires its complete bounded native corners"
        ));
    }
    let mut ranks: Vec<(Rank, Vec<Glyph>)> = Rank::ALL
        .into_iter()
        .map(|rank| (rank, Vec::new()))
        .collect();
    let mut suits: Vec<(Suit, Vec<Glyph>)> = Suit::ALL
        .into_iter()
        .map(|suit| (suit, Vec::new()))
        .collect();
    for entry in entries {
        let rank = Rank::ALL
            .into_iter()
            .find(|rank| entry["rank"] == rank.as_str())
            .ok_or_else(|| format!("Unknown {family} rank"))?;
        let suit = Suit::ALL
            .into_iter()
            .find(|suit| entry["suit"] == suit.as_str())
            .ok_or_else(|| format!("Unknown {family} suit"))?;
        let text = entry["rgb"]
            .as_str()
            .ok_or_else(|| format!("{family} RGB corner missing"))?;
        if text.len() != CORNER_WIDTH * CORNER_HEIGHT * 6 {
            return Err(format!("{family} RGB corner size mismatch"));
        }
        let bytes: Vec<u8> = text
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| Ok(super::hex(pair[0])? * 16 + super::hex(pair[1])?))
            .collect::<Result<_, String>>()?;
        let pixels = bytes.as_chunks::<3>().0.to_vec();
        let reference_cut = if ink == InkPolicy::Sepia { Some(UNDERWATER_SEPARATOR) } else { (!western).then_some(32) };
        let corner = corners_with_ink(&pixels, None, reference_cut, western, ink)
            .into_iter()
            .next()
            .ok_or_else(|| format!("{family} {rank:?}/{suit:?} corner lacks complete same-colour glyphs: paper={:?}; components={:?}", paper_reference(&pixels), components(&pixels, None).iter().map(|c| (c.colour, c.points.iter().map(|p|p.x).min(), c.points.iter().map(|p|p.x).max(), c.points.iter().map(|p|p.y).min(), c.points.iter().map(|p|p.y).max(), c.points.iter().filter(|p|p.strong).count())).collect::<Vec<_>>()))?;
        if red_suit(suit) != (corner.colour == Colour::Red) {
            return Err(format!("{family} exemplar colour contradicts its suit"));
        }
        let rank_glyph = corner.rank;
        let suit_glyph = corner.suit;
        let (_, variants) = ranks
            .iter_mut()
            .find(|(label, _)| *label == rank)
            .ok_or_else(|| format!("{family} rank bank incomplete"))?;
        if !western && !variants.is_empty() {
            return Err(format!("Duplicate {family} rank exemplar"));
        }
        if !variants.contains(&rank_glyph) {
            variants.push(rank_glyph);
        }
        let (_, variants) = suits
            .iter_mut()
            .find(|(label, _)| *label == suit)
            .ok_or_else(|| format!("{family} suit bank incomplete"))?;
        if !variants.contains(&suit_glyph) {
            variants.push(suit_glyph);
        }
    }
    if ranks.iter().any(|(_, variants)| variants.is_empty())
        || suits.iter().any(|(_, variants)| variants.is_empty())
    {
        return Err(format!("{family} family must cover every rank and suit"));
    }
    Ok(Family {
        ranks,
        suits,
        native_size: true,
        western,
        ink,
        separated_spade: None,
    })
}


#[cfg(test)]
mod tests;
