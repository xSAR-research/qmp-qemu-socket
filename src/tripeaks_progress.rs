#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! TriPeaks Challenge/Normal progress, deals-left tray caption, and modal observers.
//!
//! Every public observer inspects supplied `1920×1080` native frames read-only without
//! performing QMP I/O, mutating guest state, or granting input authority.

use std::cmp::Ordering;

use xsar::{games::cards::Rank, image_matching::FrameView};

use crate::{
    capture::CapturedFrame,
    game_operations::{OperationError, validate_native_frame},
    geometry::{PixelPoint, PixelRect},
    pyramid_progress::challenge::{ChallengeKind, decimal, digits},
    recon::Objective,
};

pub use crate::pyramid_progress::BoardCount;

/// Calibrated native bounds for `"Undo last move"` on the tall Normal-mode `"No Moves Remaining"` modal.
pub const UNDO_LAST_MOVE_BUTTON_BOUNDS: PixelRect = PixelRect::new(976, 692, 305, 72);

/// Calibrated native click centre for `"Undo last move"` on the tall Normal-mode `"No Moves Remaining"` modal.
pub const UNDO_LAST_MOVE_CLICK_POINT: PixelPoint = PixelPoint::new(1_128, 728);

/// Calibrated native bounds for `"Undo last move"` on the compact Challenge-mode `"No Moves Remaining"` modal.
pub const UNDO_LAST_MOVE_COMPACT_BUTTON_BOUNDS: PixelRect = PixelRect::new(977, 568, 312, 74);

/// Calibrated native click centre for `"Undo last move"` on the compact Challenge-mode `"No Moves Remaining"` modal.
pub const UNDO_LAST_MOVE_COMPACT_CLICK_POINT: PixelPoint = PixelPoint::new(1_133, 605);

/// Top-HUD Challenge ribbon band matching primary/secondary banner words (`32` px tall).
const BANNER_WORD_BAND: PixelRect = PixelRect::new(820, 48, 282, 32);

/// Inner counter region on `Clear <rank>` and `Boards cleared` ribbons (`< 1101` left of rail).
const COUNTER_BOUNDS_SHORT: PixelRect = PixelRect::new(1_036, 48, 65, 32);

/// Inner counter region on `Score <cur>/<target>` ribbons (`<= 142` max `digits` width).
const COUNTER_BOUNDS_SCORE: PixelRect = PixelRect::new(959, 48, 142, 32);

/// Normal top-HUD region scanned for the centred `"Score:"` label.
const NORMAL_SCORE_LABEL_BAND: PixelRect = PixelRect::new(1_435, 46, 125, 32);

/// Rightmost exclusive native column sampled for Normal top-HUD `Score` digits.
const NORMAL_SCORE_DIGIT_RIGHT: u32 = 1_625;

/// Tray caption band below Stock/Waste checked for `"0 deals left"` / `"1 deal left"` / `"2 deals left"`.
const DEALS_LEFT_BAND: PixelRect = PixelRect::new(882, 872, 156, 26);

/// Blue modal title band checked for `"No Moves Remaining"` on the tall Normal-mode modal.
const NO_MOVES_TITLE_BAND: PixelRect = PixelRect::new(740, 299, 445, 48);

/// Blue modal title band checked for `"No Moves Remaining"` on the compact Challenge-mode modal.
const NO_MOVES_TITLE_BAND_COMPACT: PixelRect = PixelRect::new(714, 457, 496, 48);

/// Height of Challenge banner and Normal top-HUD word strips.
const WORD_HEIGHT_HUD: usize = 32;

/// Height of tray deals-left caption templates.
const WORD_HEIGHT_DEALS: usize = 26;

/// Height of `"No Moves Remaining"` modal title templates.
const WORD_HEIGHT_MODAL: usize = 48;

/// Width of `"the"` prefix inside `the-deck.ink` (`97 × 32`).
const THE_PREFIX_WIDTH: usize = 38;

/// Height-preserving width of `the-deck.ink`.
const THE_DECK_WIDTH: usize = 97;

/// Largest ink band capacity (`445 × 48 = 21,360`, rounded to cover `520 × 48`).
const MAX_BAND_PIXELS: usize = 520 * 48;

/// Maximum horizontal ink span capacity per band.
const MAX_SPANS: usize = 16;

/// Maximum normalised L1 distance (`DIFF / TOTAL`) accepted for word matching, in ppm.
const WORD_DISTANCE_LIMIT: u64 = 210_000;

/// Minimum required separation (`runner_up - winner`) between distinct word classes, in ppm.
const WORD_MARGIN: u64 = 100_000;

/// Integer fixed-point denominator (`1,000,000` parts per million).
const SCALE: u64 = 1_000_000;

/// Banner border-consistency tolerance for ribbon interior background samples.
const BANNER_INTERIOR_TOLERANCE: u8 = 25;

/// Channel-consistency tolerance for DirectWrite ClearType white-blend ink extraction.
const CHANNEL_INK_TOLERANCE: u8 = 40;

/// Minimum peak ink column intensity treated as foreground glyph stroke (filters felt noise).
const SPAN_INK_THRESHOLD: u8 = 60;

/// Native white-ink seed threshold matching `digits::CORE_MINIMUM` semantics on arbitrary ribbons.
const RIBBON_DIGIT_SEED_MIN: u8 = 145;

/// Normal green top-HUD nominal background measured at `(1520, 48)` for `h = 17` digit templates.
const HUD_NOMINAL_BG: [u8; 3] = [17, 80, 51];

/// Native Challenge plural rank template bytes (`aces`..`kings`, with spaced/compact `threes`).
const TEMPLATE_ACES: &[u8; 55 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/aces.ink");
/// Native `"Twos"` banner template (`59 × 32`).
const TEMPLATE_TWOS: &[u8; 59 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/twos.ink");
/// Calibrated-synthetic `"Threes"` spaced-kerning banner template (`75 × 32`).
const TEMPLATE_THREES_SPACED: &[u8; 75 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/threes-spaced.ink");
/// Calibrated-synthetic `"Threes"` compact `r->e` banner template (`75 × 32`).
const TEMPLATE_THREES_COMPACT: &[u8; 75 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/threes-compact.ink");
/// Native `"Fours"` banner template (`62 × 32`).
const TEMPLATE_FOURS: &[u8; 62 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/fours.ink");
/// Native `"Fives"` banner template (`55 × 32`).
const TEMPLATE_FIVES: &[u8; 55 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/fives.ink");
/// Native `"Sixes"` banner template (`56 × 32`).
const TEMPLATE_SIXES: &[u8; 56 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/sixes.ink");
/// Native `"Sevens"` banner template (`78 × 32`).
const TEMPLATE_SEVENS: &[u8; 78 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/sevens.ink");
/// Native `"Eights"` banner template (`68 × 32`).
const TEMPLATE_EIGHTS: &[u8; 68 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/eights.ink");
/// Native `"Nines"` banner template (`64 × 32`).
const TEMPLATE_NINES: &[u8; 64 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/nines.ink");
/// Native `"Tens"` banner template (`51 × 32`).
const TEMPLATE_TENS: &[u8; 51 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/tens.ink");
/// Native `"Jacks"` banner template (`59 × 32`).
const TEMPLATE_JACKS: &[u8; 59 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/jacks.ink");
/// Native `"Queens"` banner template (`85 × 32`).
const TEMPLATE_QUEENS: &[u8; 85 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/queens.ink");
/// Native `"Kings"` banner template (`61 × 32`).
const TEMPLATE_KINGS: &[u8; 61 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/kings.ink");
/// Native `"Boards"` banner template (`77 × 32`).
const TEMPLATE_BOARDS: &[u8; 77 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/boards.ink");
/// Native `"cleared"` secondary banner template (`81 × 32`).
const TEMPLATE_CLEARED: &[u8; 81 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/cleared.ink");
/// Native `"Score"` banner template (`63 × 32`).
const TEMPLATE_SCORE: &[u8; 63 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/score.ink");
/// Native `"Solve"` banner template (`61 × 32`).
const TEMPLATE_SOLVE: &[u8; 61 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/solve.ink");
/// Native `"the deck"` secondary banner template (`97 × 32`).
const TEMPLATE_THE_DECK: &[u8; THE_DECK_WIDTH * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/the-deck.ink");
/// Native `"Clear"` banner template (`59 × 32`).
const TEMPLATE_CLEAR_WORD: &[u8; 59 * 32] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/clear-word.ink");

/// Native Normal HUD `"Score:"` primary label template (`69 × 32`).
const TEMPLATE_NORMAL_SCORE_LABEL: &[u8; 69 * 32] = include_bytes!(
    "../tests/fixtures/tripeaks-calibration-20261116/templates/normal-score-label.ink"
);
/// Native Normal HUD `"Score:"` alternate fractional-hinting template (`68 × 32`).
const TEMPLATE_NORMAL_SCORE_LABEL_ALT: &[u8; 68 * 32] = include_bytes!(
    "../tests/fixtures/tripeaks-calibration-20261116/templates/normal-score-label-alt.ink"
);
/// Native `h = 17` Normal HUD digit/comma templates (`28 × 520` bytes).
const NORMAL_DIGITS_GLYPHS: &[u8; 28 * 520] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/normal-digits.ink");
/// ASCII symbol labels matching `NORMAL_DIGITS_GLYPHS`.
const NORMAL_DIGITS_LABELS: &[u8; 28] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/normal-labels.txt");

/// Native `"0 deals left"` tray caption template (`136 × 26`).
const TEMPLATE_DEALS_ZERO: &[u8; 136 * 26] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/deals-zero.ink");
/// Native `"1 deal left"` tray caption template (`124 × 26`).
const TEMPLATE_DEALS_ONE: &[u8; 124 * 26] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/deals-one.ink");
/// Native `"2 deals left"` tray caption template (`140 × 26`).
const TEMPLATE_DEALS_TWO: &[u8; 140 * 26] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/deals-two.ink");

/// Native `"No Moves Remaining"` modal title template for the tall Normal-mode layout (`416 × 48`).
const TEMPLATE_NO_MOVES_TITLE: &[u8; 416 * 48] =
    include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/templates/no-moves-title.ink");

/// Native `"No Moves Remaining"` modal title template for the compact Challenge-mode layout (`470 × 48`).
const TEMPLATE_NO_MOVES_TITLE_COMPACT: &[u8; 470 * 48] = include_bytes!(
    "../tests/fixtures/tripeaks-calibration-20261116/templates/no-moves-title-compact.ink"
);

/// Classified Challenge header objective observed from the native top ribbon.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TriPeaksChallenge {
    /// Clear `target` cards of `rank` (`Aces`..`Kings`).
    ClearRank {
        /// Target standard rank (`Ace`..`King`).
        rank: Rank,
        /// Observed cleared count so far.
        current: u32,
        /// Observed target count.
        target: u32,
    },
    /// Clear `target` boards (`Boards cleared current/target`).
    Boards {
        /// Observed completed boards.
        current: u32,
        /// Observed board target.
        target: u32,
    },
    /// Reach `target` cumulative Challenge score (`Score current/target`).
    Score {
        /// Observed Challenge score so far.
        current: u32,
        /// Observed score target.
        target: u32,
    },
    /// Solve the entire deal (`Solve the deck`).
    SolveDeck,
    /// Single-card Clear objective (`Clear the <rank+suit>`); unsupported by TriPeaks RECON.
    UnsupportedSingleCard,
}

impl TriPeaksChallenge {
    /// Map a supported Challenge objective into a generic `recon::Objective`.
    ///
    /// Returns `None` for [`TriPeaksChallenge::UnsupportedSingleCard`] or if a rank target quota
    /// cannot fit within `u8`.
    pub fn to_recon_objective(self) -> Option<Objective> {
        match self {
            Self::SolveDeck | Self::Boards { .. } => Some(Objective::ClearBoard),
            Self::ClearRank {
                rank,
                current,
                target,
            } => Some(Objective::ClearRankQuota {
                rank: rank.value().checked_sub(1)?,
                remaining: u8::try_from(target.saturating_sub(current)).ok()?,
            }),
            Self::Score { current, target } => Some(Objective::ReachScoreDelta {
                points_needed: target.saturating_sub(current),
            }),
            Self::UnsupportedSingleCard => None,
        }
    }
}

/// Primary banner word classes distinguished before secondary/counter parsing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PrimaryClass {
    /// One of the 13 plural card rank objectives (`Aces`..`Kings`).
    Rank(Rank),
    /// `Boards cleared X/Y`.
    Boards,
    /// `Score X/Y`.
    Score,
    /// `Solve the deck`.
    Solve,
    /// `Clear the <single card>` (unsupported by TriPeaks RECON).
    ClearWord,
}

/// Descriptor pairing a static grayscale template slice with its width and height.
#[derive(Clone, Copy)]
struct WordTemplate<'a> {
    /// Template width in native pixels.
    width: usize,
    /// Template height in native pixels.
    height: usize,
    /// Row-major `0..=255` white-ink bytes (`len == width * height`).
    bytes: &'a [u8],
}

/// Primary Challenge banner word catalogue (`18` templates across `17` distinct classes).
const PRIMARY_WORD_CATALOGUE: &[(PrimaryClass, WordTemplate<'static>)] = &[
    (
        PrimaryClass::Rank(Rank::Ace),
        WordTemplate {
            width: 55,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_ACES,
        },
    ),
    (
        PrimaryClass::Rank(Rank::Two),
        WordTemplate {
            width: 59,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_TWOS,
        },
    ),
    (
        PrimaryClass::Rank(Rank::Three),
        WordTemplate {
            width: 75,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_THREES_SPACED,
        },
    ),
    (
        PrimaryClass::Rank(Rank::Three),
        WordTemplate {
            width: 75,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_THREES_COMPACT,
        },
    ),
    (
        PrimaryClass::Rank(Rank::Four),
        WordTemplate {
            width: 62,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_FOURS,
        },
    ),
    (
        PrimaryClass::Rank(Rank::Five),
        WordTemplate {
            width: 55,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_FIVES,
        },
    ),
    (
        PrimaryClass::Rank(Rank::Six),
        WordTemplate {
            width: 56,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_SIXES,
        },
    ),
    (
        PrimaryClass::Rank(Rank::Seven),
        WordTemplate {
            width: 78,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_SEVENS,
        },
    ),
    (
        PrimaryClass::Rank(Rank::Eight),
        WordTemplate {
            width: 68,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_EIGHTS,
        },
    ),
    (
        PrimaryClass::Rank(Rank::Nine),
        WordTemplate {
            width: 64,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_NINES,
        },
    ),
    (
        PrimaryClass::Rank(Rank::Ten),
        WordTemplate {
            width: 51,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_TENS,
        },
    ),
    (
        PrimaryClass::Rank(Rank::Jack),
        WordTemplate {
            width: 59,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_JACKS,
        },
    ),
    (
        PrimaryClass::Rank(Rank::Queen),
        WordTemplate {
            width: 85,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_QUEENS,
        },
    ),
    (
        PrimaryClass::Rank(Rank::King),
        WordTemplate {
            width: 61,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_KINGS,
        },
    ),
    (
        PrimaryClass::Boards,
        WordTemplate {
            width: 77,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_BOARDS,
        },
    ),
    (
        PrimaryClass::Score,
        WordTemplate {
            width: 63,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_SCORE,
        },
    ),
    (
        PrimaryClass::Solve,
        WordTemplate {
            width: 61,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_SOLVE,
        },
    ),
    (
        PrimaryClass::ClearWord,
        WordTemplate {
            width: 59,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_CLEAR_WORD,
        },
    ),
];

/// Exact rational normalised intensity distance (`difference / total`, non-zero denominator).
#[derive(Clone, Copy, Eq, PartialEq)]
struct RatioDistance {
    /// Sum of absolute intensity differences over the aligned union window.
    difference: u64,
    /// Sum of per-pixel maxima over the same window (`> 0`).
    total: u64,
}

impl Ord for RatioDistance {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.difference * other.total).cmp(&(other.difference * self.total))
    }
}

impl PartialOrd for RatioDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl RatioDistance {
    /// Return true when `self <= limit_ppm / SCALE` in exact arithmetic.
    fn within_ppm(self, limit_ppm: u64) -> bool {
        self.difference * SCALE <= limit_ppm * self.total
    }

    /// Return true when `(other - self) >= margin_ppm / SCALE` in exact arithmetic.
    fn margin_before(self, other: Self, margin_ppm: u64) -> bool {
        let Some(numerator) =
            (other.difference * self.total).checked_sub(self.difference * other.total)
        else {
            return false;
        };
        numerator * SCALE >= margin_ppm * other.total * self.total
    }
}

/// Horizontal half-open ink interval inside a [`WhiteInkBand`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InkSpan {
    /// Inclusive starting column relative to band left edge.
    start: usize,
    /// Exclusive ending column relative to band left edge.
    end: usize,
}

/// Bounded row-major white-ink strip extracted from a native frame region.
struct WhiteInkBand {
    /// Row-major `0..=255` ink pixels (`width * height <= MAX_BAND_PIXELS`).
    pixels: Vec<u8>,
    /// Band width in pixels.
    width: usize,
    /// Band height in pixels.
    height: usize,
}

impl WhiteInkBand {
    /// Extract white-blend foreground ink across `bounds` relative to background `bg`.
    fn extract(
        view: FrameView<'_>,
        bounds: PixelRect,
        bg: [u8; 3],
        channel_tolerance: u8,
    ) -> Option<Self> {
        view.checked_bounds(bounds).ok()?;
        let width = usize::try_from(bounds.width).ok()?;
        let height = usize::try_from(bounds.height).ok()?;
        let length = width.checked_mul(height)?;
        if width == 0 || height == 0 || length > MAX_BAND_PIXELS {
            return None;
        }
        let anchor_channel = min_channel_index(bg);
        let denom = u16::from(255_u8.saturating_sub(bg[anchor_channel]));
        if denom == 0 {
            return None;
        }
        let mut pixels = Vec::with_capacity(length);
        for dy in 0..height {
            for dx in 0..width {
                let rgb = view.pixel_rgb(bounds.x + dx as u32, bounds.y + dy as u32)?;
                pixels.push(white_blend_ink(
                    rgb,
                    bg,
                    anchor_channel,
                    denom,
                    channel_tolerance,
                ));
            }
        }
        Some(Self {
            pixels,
            width,
            height,
        })
    }

    /// Segment contiguous word spans (`>= SPAN_INK_THRESHOLD`, bridging internal gaps `<= 4` px).
    fn spans(&self) -> Vec<InkSpan> {
        let mut active_cols = Vec::with_capacity(self.width);
        for x in 0..self.width {
            let active =
                (0..self.height).any(|y| self.pixels[y * self.width + x] >= SPAN_INK_THRESHOLD);
            active_cols.push(active);
        }
        let mut spans = Vec::new();
        let mut x = 0;
        while x < self.width && spans.len() < MAX_SPANS {
            if !active_cols[x] {
                x += 1;
                continue;
            }
            let start = x;
            let mut end = x;
            while end + 1 < self.width {
                if active_cols[end + 1] {
                    end += 1;
                } else if let Some(next) =
                    (end + 2..self.width.min(end + 6)).find(|&probe| active_cols[probe])
                {
                    end = next;
                } else {
                    break;
                }
            }
            if end + 1 - start >= 4 {
                spans.push(InkSpan {
                    start,
                    end: end + 1,
                });
            }
            x = end + 1;
        }
        spans
    }

    /// Best normalised L1 distance of `template` aligned near `span` across bounded shifts.
    fn match_at_span(&self, span: InkSpan, template: WordTemplate<'_>) -> Option<RatioDistance> {
        if template.height != self.height
            || template.bytes.len() != template.width.checked_mul(template.height)?
            || self.width < template.width
        {
            return None;
        }
        let sx_min = span.start.saturating_sub(5);
        let sx_max = span
            .start
            .saturating_add(2)
            .min(self.width.saturating_sub(template.width));
        let mut best = None::<RatioDistance>;
        for sx in sx_min..=sx_max {
            let win_start = span.start.min(sx);
            let win_end = span.end.max(sx + template.width).min(self.width);
            for dy in -1_i32..=1 {
                let mut difference = 0_u64;
                let mut total = 0_u64;
                for y in 0..self.height {
                    let ty = i32::try_from(y).ok()? - dy;
                    let row_ok = (0..self.height as i32).contains(&ty);
                    for x in win_start..win_end {
                        let actual = self.pixels[y * self.width + x];
                        let expected = if row_ok && x >= sx && x - sx < template.width {
                            template.bytes[ty as usize * template.width + (x - sx)]
                        } else {
                            0
                        };
                        difference =
                            difference.checked_add(u64::from(actual.abs_diff(expected)))?;
                        total = total.checked_add(u64::from(actual.max(expected)))?;
                    }
                }
                if total == 0 {
                    continue;
                }
                let candidate = RatioDistance { difference, total };
                if best.is_none_or(|prev| candidate < prev) {
                    best = Some(candidate);
                }
            }
        }
        best
    }

    /// Match a word span against a single expected template within `WORD_DISTANCE_LIMIT`.
    fn matches_single(&self, span: InkSpan, template: WordTemplate<'_>) -> bool {
        self.match_at_span(span, template)
            .is_some_and(|dist| dist.within_ppm(WORD_DISTANCE_LIMIT))
    }

    /// Match `span` exclusively against `candidates` (`(class, template)` pairs), grouping equal classes.
    fn classify_exclusive<T: Copy + Eq>(
        &self,
        span: InkSpan,
        candidates: &[(T, WordTemplate<'_>)],
    ) -> Option<T> {
        let mut per_class: Vec<(T, RatioDistance)> = Vec::new();
        for &(class, template) in candidates {
            let dist = self.match_at_span(span, template)?;
            if let Some((_, best_dist)) = per_class
                .iter_mut()
                .find(|(existing, _)| *existing == class)
            {
                if dist < *best_dist {
                    *best_dist = dist;
                }
            } else {
                per_class.push((class, dist));
            }
        }
        if per_class.len() < 2 {
            return None;
        }
        per_class.sort_unstable_by_key(|item| item.1);
        let (winner_class, winner_dist) = per_class[0];
        let runner_dist = per_class[1].1;
        (winner_dist.within_ppm(WORD_DISTANCE_LIMIT)
            && winner_dist.margin_before(runner_dist, WORD_MARGIN))
        .then_some(winner_class)
    }
}

/// Index (`0..=2`) of the smallest RGB component used as the linear white-mix anchor.
fn min_channel_index(rgb: [u8; 3]) -> usize {
    let mut best_idx = 0;
    if rgb[1] < rgb[best_idx] {
        best_idx = 1;
    }
    if rgb[2] < rgb[best_idx] {
        best_idx = 2;
    }
    best_idx
}

/// Recover linear white-blend intensity `(0..=255)` relative to background `bg`.
fn white_blend_ink(
    rgb: [u8; 3],
    bg: [u8; 3],
    anchor_channel: usize,
    denom: u16,
    channel_tolerance: u8,
) -> u8 {
    let raw = rgb[anchor_channel].saturating_sub(bg[anchor_channel]);
    if raw == 0 {
        return 0;
    }
    let ink = ((u16::from(raw) * 255) / denom).min(255) as u8;
    for channel in 0..3 {
        let expected = u16::from(bg[channel])
            + (u16::from(ink) * u16::from(255_u8.saturating_sub(bg[channel]))) / 255;
        if rgb[channel].abs_diff(expected as u8) > channel_tolerance {
            return 0;
        }
    }
    ink
}

/// Sample 5 native coordinates and return their per-channel median when they stay consistent.
fn median_consistent_rgb(
    view: FrameView<'_>,
    points: [(u32, u32); 5],
    max_spread: u8,
) -> Option<[u8; 3]> {
    let mut samples = [[0_u8; 3]; 5];
    for (dst, &(x, y)) in samples.iter_mut().zip(&points) {
        *dst = view.pixel_rgb(x, y)?;
    }
    let mut median = [0_u8; 3];
    for channel in 0..3 {
        let mut vals = [
            samples[0][channel],
            samples[1][channel],
            samples[2][channel],
            samples[3][channel],
            samples[4][channel],
        ];
        vals.sort_unstable();
        if vals[4].saturating_sub(vals[0]) > max_spread {
            return None;
        }
        median[channel] = vals[2];
    }
    Some(median)
}

/// Verify the Challenge header ribbon border/outer contrast invariant and return interior `bg`.
fn verify_challenge_ribbon_bg(view: FrameView<'_>) -> Option<[u8; 3]> {
    let bg = median_consistent_rgb(
        view,
        [(825, 45), (840, 45), (825, 81), (900, 45), (900, 81)],
        BANNER_INTERIOR_TOLERANCE,
    )?;
    if bg[0].min(bg[1]).min(bg[2]) > 170 {
        return None;
    }
    let rail_pts = [(960, 42), (960, 85), (815, 64), (1104, 64)];
    let outer_pts = [(960, 36), (960, 91), (808, 64), (1111, 64)];
    for (&(rx, ry), &(ox, oy)) in rail_pts.iter().zip(&outer_pts) {
        let rail = view.pixel_rgb(rx, ry)?;
        let outer = view.pixel_rgb(ox, oy)?;
        for channel in 0..3 {
            if rail[channel] < bg[channel].saturating_add(12) {
                return None;
            }
        }
        let has_outer_drop = (0..3).any(|c| rail[c].saturating_sub(outer[c]) >= 40);
        if !has_outer_drop {
            return None;
        }
    }
    Some(bg)
}

/// Verify Normal/Challenge top green HUD strip near the top-right `Score` region.
fn verify_green_top_hud_bg(view: FrameView<'_>) -> Option<[u8; 3]> {
    let bg = median_consistent_rgb(
        view,
        [(1440, 48), (1500, 48), (1560, 48), (1600, 48), (1615, 76)],
        35,
    )?;
    (bg[0] <= 65 && bg[1] >= bg[0].saturating_add(45) && bg[1] >= bg[2].saturating_add(15))
        .then_some(bg)
}

/// Extract a compile-time crop prefix of `TEMPLATE_THE_DECK` matching `"the"` (`38 × 32`).
const fn the_prefix_bytes() -> [u8; THE_PREFIX_WIDTH * WORD_HEIGHT_HUD] {
    let mut out = [0_u8; THE_PREFIX_WIDTH * WORD_HEIGHT_HUD];
    let mut y = 0;
    while y < WORD_HEIGHT_HUD {
        let mut x = 0;
        while x < THE_PREFIX_WIDTH {
            out[y * THE_PREFIX_WIDTH + x] = TEMPLATE_THE_DECK[y * THE_DECK_WIDTH + x];
            x += 1;
        }
        y += 1;
    }
    out
}

/// Static `"the"` word bytes (`38 × 32`) extracted from native `TEMPLATE_THE_DECK`.
const TEMPLATE_THE_PREFIX: [u8; THE_PREFIX_WIDTH * WORD_HEIGHT_HUD] = the_prefix_bytes();

/// Observe a colour-independent Challenge banner objective on a `1920×1080` frame.
///
/// Supports all 13 plural rank words (`Aces`..`Kings`), `Boards cleared X/Y`,
/// `Score X/Y`, `Solve the deck`, and `Clear the <single card>` without granting
/// input authority or performing QMP I/O. Returns `Ok(None)` for Normal games,
/// dimmed overlays, or unsupported banners.
///
/// # Errors
///
/// Returns [`OperationError::Invalid`] if the frame is not packed `1920×1080` RGBA8.
pub fn observe_challenge_header(
    frame: &CapturedFrame,
) -> Result<Option<TriPeaksChallenge>, OperationError> {
    validate_native_frame(frame)?;
    let Ok(view) = frame.as_view() else {
        return Ok(None);
    };
    let Some(bg) = verify_challenge_ribbon_bg(view) else {
        return Ok(None);
    };
    let Some(band) = WhiteInkBand::extract(view, BANNER_WORD_BAND, bg, CHANNEL_INK_TOLERANCE)
    else {
        return Ok(None);
    };
    let spans = band.spans();
    let Some(&first_span) = spans.first() else {
        return Ok(None);
    };
    let Some(primary) = band.classify_exclusive(first_span, PRIMARY_WORD_CATALOGUE) else {
        return Ok(None);
    };

    let anchor_channel = min_channel_index(bg);
    let denom = u16::from(255_u8.saturating_sub(bg[anchor_channel]));
    let is_ribbon_seed = |rgb: [u8; 3]| {
        white_blend_ink(rgb, bg, anchor_channel, denom, CHANNEL_INK_TOLERANCE)
            >= RIBBON_DIGIT_SEED_MIN
    };
    let ribbon_ink =
        |rgb: [u8; 3]| white_blend_ink(rgb, bg, anchor_channel, denom, CHANNEL_INK_TOLERANCE);

    match primary {
        PrimaryClass::Rank(rank) => {
            let Some(&second) = spans.get(1) else {
                return Ok(None);
            };
            if !band.matches_single(
                second,
                WordTemplate {
                    width: 81,
                    height: WORD_HEIGHT_HUD,
                    bytes: TEMPLATE_CLEARED,
                },
            ) {
                return Ok(None);
            }
            let Some(symbols) =
                digits::read_with_extractor(view, COUNTER_BOUNDS_SHORT, is_ribbon_seed, ribbon_ink)
            else {
                return Ok(None);
            };
            let Some(progress) = decimal::parse(ChallengeKind::Cards, symbols.as_slice()) else {
                return Ok(None);
            };
            Ok(Some(TriPeaksChallenge::ClearRank {
                rank,
                current: progress.current,
                target: progress.target.get(),
            }))
        }
        PrimaryClass::Boards => {
            let Some(&second) = spans.get(1) else {
                return Ok(None);
            };
            if !band.matches_single(
                second,
                WordTemplate {
                    width: 81,
                    height: WORD_HEIGHT_HUD,
                    bytes: TEMPLATE_CLEARED,
                },
            ) {
                return Ok(None);
            }
            let Some(symbols) =
                digits::read_with_extractor(view, COUNTER_BOUNDS_SHORT, is_ribbon_seed, ribbon_ink)
            else {
                return Ok(None);
            };
            let Some(progress) = decimal::parse(ChallengeKind::Boards, symbols.as_slice()) else {
                return Ok(None);
            };
            Ok(Some(TriPeaksChallenge::Boards {
                current: progress.current,
                target: progress.target.get(),
            }))
        }
        PrimaryClass::Score => {
            let Some(symbols) =
                digits::read_with_extractor(view, COUNTER_BOUNDS_SCORE, is_ribbon_seed, ribbon_ink)
            else {
                return Ok(None);
            };
            let Some(progress) = decimal::parse(ChallengeKind::Score, symbols.as_slice()) else {
                return Ok(None);
            };
            Ok(Some(TriPeaksChallenge::Score {
                current: progress.current,
                target: progress.target.get(),
            }))
        }
        PrimaryClass::Solve => {
            let (Some(&w1), Some(&w2)) = (spans.get(1), spans.get(2)) else {
                return Ok(None);
            };
            let phrase_span = InkSpan {
                start: w1.start,
                end: w2.end,
            };
            let matched = band.matches_single(
                phrase_span,
                WordTemplate {
                    width: THE_DECK_WIDTH,
                    height: WORD_HEIGHT_HUD,
                    bytes: TEMPLATE_THE_DECK,
                },
            );
            Ok(matched.then_some(TriPeaksChallenge::SolveDeck))
        }
        PrimaryClass::ClearWord => {
            let Some(&second) = spans.get(1) else {
                return Ok(None);
            };
            let matched = band.matches_single(
                second,
                WordTemplate {
                    width: THE_PREFIX_WIDTH,
                    height: WORD_HEIGHT_HUD,
                    bytes: &TEMPLATE_THE_PREFIX,
                },
            );
            Ok(matched.then_some(TriPeaksChallenge::UnsupportedSingleCard))
        }
    }
}

/// Observe complete Normal series `Boards: <X> of 3` header and `New` footer.
///
/// Delegates to `crate::pyramid_progress::observe_normal` after validating
/// the packed `1920×1080` frame.
///
/// # Errors
///
/// Returns [`OperationError::Invalid`] for bad frame dimensions or
/// [`OperationError::Recognition`] if reference patch calibration fails.
pub fn observe_normal_boards(frame: &CapturedFrame) -> Result<Option<BoardCount>, OperationError> {
    validate_native_frame(frame)?;
    crate::pyramid_progress::observe_normal(frame).map_err(OperationError::Recognition)
}

/// Observe complete Normal series `Boards: <X> of 3` header (without requiring `New` footer).
///
/// # Errors
///
/// Returns [`OperationError::Invalid`] for bad frame dimensions or
/// [`OperationError::Recognition`] if reference patch calibration fails.
pub fn observe_normal_boards_header(
    frame: &CapturedFrame,
) -> Result<Option<BoardCount>, OperationError> {
    validate_native_frame(frame)?;
    crate::pyramid_progress::observe_normal_header(frame).map_err(OperationError::Recognition)
}

/// Observe the live Normal top-HUD `Score: <N>` value (`h = 17` digits) on `1920×1080`.
///
/// Matches the centred `"Score:"` label across native subpixel phases, reads native
/// `0..=9` and grouped thousands `,` glyphs to its right, and returns `Ok(None)` when
/// the top bar is dimmed, in a Challenge view, or shows a different label (`Time:`/`Moves:`).
///
/// # Errors
///
/// Returns [`OperationError::Invalid`] if the frame is not packed `1920×1080` RGBA8.
pub fn observe_normal_score(frame: &CapturedFrame) -> Result<Option<u32>, OperationError> {
    validate_native_frame(frame)?;
    let Ok(view) = frame.as_view() else {
        return Ok(None);
    };
    let Some(bg) = verify_green_top_hud_bg(view) else {
        return Ok(None);
    };
    let Some(band) =
        WhiteInkBand::extract(view, NORMAL_SCORE_LABEL_BAND, bg, CHANNEL_INK_TOLERANCE)
    else {
        return Ok(None);
    };
    let spans = band.spans();
    let Some(&label_span) = spans.first() else {
        return Ok(None);
    };
    let label_matches = [
        WordTemplate {
            width: 69,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_NORMAL_SCORE_LABEL,
        },
        WordTemplate {
            width: 68,
            height: WORD_HEIGHT_HUD,
            bytes: TEMPLATE_NORMAL_SCORE_LABEL_ALT,
        },
    ]
    .into_iter()
    .any(|template| band.matches_single(label_span, template));
    if !label_matches {
        return Ok(None);
    }

    let digit_x = NORMAL_SCORE_LABEL_BAND.x + label_span.end as u32 + 1;
    let Some(digit_width) = NORMAL_SCORE_DIGIT_RIGHT.checked_sub(digit_x) else {
        return Ok(None);
    };
    let digit_bounds = PixelRect::new(digit_x, NORMAL_SCORE_LABEL_BAND.y, digit_width, 32);

    let is_seed_hud = |rgb: [u8; 3]| {
        let min_c = rgb[0].min(rgb[1]).min(rgb[2]);
        let max_c = rgb[0].max(rgb[1]).max(rgb[2]);
        min_c >= 135 && max_c.saturating_sub(min_c) <= 95
    };
    let intensity_hud = |rgb: [u8; 3]| {
        let raw = rgb[0].saturating_sub(HUD_NOMINAL_BG[0]);
        if raw == 0 {
            return 0;
        }
        let denom = u16::from(255 - HUD_NOMINAL_BG[0]);
        let ink = ((u16::from(raw) * 255) / denom).min(255) as u8;
        for channel in 1..3 {
            let expected = u16::from(HUD_NOMINAL_BG[channel])
                + (u16::from(ink) * u16::from(255 - HUD_NOMINAL_BG[channel])) / 255;
            if rgb[channel].abs_diff(expected as u8) > CHANNEL_INK_TOLERANCE {
                return 0;
            }
        }
        ink
    };

    let Some(symbols) = digits::read_with_glyph_bank(
        view,
        digit_bounds,
        NORMAL_DIGITS_LABELS,
        NORMAL_DIGITS_GLYPHS,
        is_seed_hud,
        intensity_hud,
    ) else {
        return Ok(None);
    };
    Ok(decimal::integer(symbols.as_slice(), true))
}

/// Observe the Challenge tray caption (`0 deals left`, `1 deal left`, or `2 deals left`).
///
/// Returns `Ok(None)` when no caption is displayed (e.g. Normal TriPeaks) or when
/// the tray is dimmed behind an active modal.
///
/// # Errors
///
/// Returns [`OperationError::Invalid`] if the frame is not packed `1920×1080` RGBA8.
pub fn observe_deals_left_caption(frame: &CapturedFrame) -> Result<Option<u8>, OperationError> {
    validate_native_frame(frame)?;
    let Ok(view) = frame.as_view() else {
        return Ok(None);
    };
    let Some(bg) = median_consistent_rgb(
        view,
        [(880, 885), (885, 885), (960, 900), (1035, 885), (1042, 885)],
        35,
    ) else {
        return Ok(None);
    };
    if !(bg[0] <= 55 && bg[1] >= bg[0].saturating_add(40) && bg[1] >= bg[2].saturating_add(15)) {
        return Ok(None);
    }
    let Some(band) = WhiteInkBand::extract(view, DEALS_LEFT_BAND, bg, CHANNEL_INK_TOLERANCE) else {
        return Ok(None);
    };
    let spans = band.spans();
    let (Some(&first), Some(&last)) = (spans.first(), spans.last()) else {
        return Ok(None);
    };
    let full_span = InkSpan {
        start: first.start,
        end: last.end,
    };
    let candidates = [
        (
            0_u8,
            WordTemplate {
                width: 136,
                height: WORD_HEIGHT_DEALS,
                bytes: TEMPLATE_DEALS_ZERO,
            },
        ),
        (
            1_u8,
            WordTemplate {
                width: 124,
                height: WORD_HEIGHT_DEALS,
                bytes: TEMPLATE_DEALS_ONE,
            },
        ),
        (
            2_u8,
            WordTemplate {
                width: 140,
                height: WORD_HEIGHT_DEALS,
                bytes: TEMPLATE_DEALS_TWO,
            },
        ),
    ];
    Ok(band.classify_exclusive(full_span, &candidates))
}

/// Calibration profile for one `"No Moves Remaining"` modal variant (tall Normal vs compact Challenge).
struct NoMovesModalLayout {
    /// Five modal background probes flanking the title to tolerate subtle suit watermarks.
    bg_probes: [(u32, u32); 5],
    /// Title white-ink search band.
    title_band: PixelRect,
    /// Calibrated `"No Moves Remaining"` title template for this layout.
    title_template: WordTemplate<'static>,
    /// Active blue `"Undo last move"` button envelope.
    button_bounds: PixelRect,
    /// Click centre inside the `"Undo last move"` button (never `"End game"`).
    click_point: PixelPoint,
    /// Three interior blue-button probes clear of white label strokes.
    button_probes: [(u32, u32); 3],
}

/// Supported `"No Moves Remaining"` modal layouts: tall Normal-mode and compact Challenge-mode.
const NO_MOVES_MODAL_LAYOUTS: [NoMovesModalLayout; 2] = [
    NoMovesModalLayout {
        bg_probes: [
            (640, 314),
            (680, 314),
            (710, 314),
            (1_210, 314),
            (1_260, 314),
        ],
        title_band: NO_MOVES_TITLE_BAND,
        title_template: WordTemplate {
            width: 416,
            height: WORD_HEIGHT_MODAL,
            bytes: TEMPLATE_NO_MOVES_TITLE,
        },
        button_bounds: UNDO_LAST_MOVE_BUTTON_BOUNDS,
        click_point: UNDO_LAST_MOVE_CLICK_POINT,
        button_probes: [(1_128, 745), (982, 728), (1_275, 728)],
    },
    NoMovesModalLayout {
        bg_probes: [
            (640, 480),
            (680, 480),
            (710, 480),
            (1_215, 480),
            (1_260, 480),
        ],
        title_band: NO_MOVES_TITLE_BAND_COMPACT,
        title_template: WordTemplate {
            width: 470,
            height: WORD_HEIGHT_MODAL,
            bytes: TEMPLATE_NO_MOVES_TITLE_COMPACT,
        },
        button_bounds: UNDO_LAST_MOVE_COMPACT_BUTTON_BOUNDS,
        click_point: UNDO_LAST_MOVE_COMPACT_CLICK_POINT,
        button_probes: [(1_133, 622), (983, 605), (1_283, 605)],
    },
];

/// Return `true` when `view` positively matches one calibrated `"No Moves Remaining"` layout.
fn matches_no_moves_modal_layout(view: FrameView<'_>, layout: &NoMovesModalLayout) -> bool {
    let Some(bg) = median_consistent_rgb(view, layout.bg_probes, 24) else {
        return false;
    };
    if bg[2] < 100 || bg[2] < bg[0].saturating_add(60) {
        return false;
    }
    let Some(band) = WhiteInkBand::extract(view, layout.title_band, bg, CHANNEL_INK_TOLERANCE)
    else {
        return false;
    };
    let spans = band.spans();
    let (Some(&first), Some(&last)) = (spans.first(), spans.last()) else {
        return false;
    };
    let full_span = InkSpan {
        start: first.start,
        end: last.end,
    };
    if !band.matches_single(full_span, layout.title_template) {
        return false;
    }

    let Ok(click_x) = u32::try_from(layout.click_point.x) else {
        return false;
    };
    let Ok(click_y) = u32::try_from(layout.click_point.y) else {
        return false;
    };
    if click_x < layout.button_bounds.x
        || click_x >= layout.button_bounds.right()
        || click_y < layout.button_bounds.y
        || click_y >= layout.button_bounds.bottom()
    {
        return false;
    }
    let Some(top_rail) = view.pixel_rgb(click_x, layout.button_bounds.y) else {
        return false;
    };
    if top_rail[2] < 180 {
        return false;
    }
    for (bx, by) in layout.button_probes {
        let Some(btn_rgb) = view.pixel_rgb(bx, by) else {
            return false;
        };
        if btn_rgb[2] < 140 || btn_rgb[2] < btn_rgb[0].saturating_add(70) {
            return false;
        }
    }
    true
}

/// Detect the Draw-23 zero-match `"No Moves Remaining"` modal (tall Normal-mode or compact
/// Challenge-mode) and return the layout-specific click target for its active blue
/// `"Undo last move"` button ([`UNDO_LAST_MOVE_CLICK_POINT`] or
/// [`UNDO_LAST_MOVE_COMPACT_CLICK_POINT`]).
///
/// Never returns the adjacent gold `"End game"` button. Requires:
/// - Dimmed outer board backdrop (`max(rgb) <= 85` at `(200, 500)`, `(1700, 500)`,
///   and `SHARED_UNDO_ALL_CONTROL` `(1320, 978)`),
/// - Bright blue modal header matching `"No Moves Remaining"` (`no-moves-title.ink` or
///   `no-moves-title-compact.ink`), and
/// - Active blue `"Undo last move"` control within the matched layout's button bounds.
///
/// Returns `Ok(None)` with zero side effects and zero QMP I/O when absent or ambiguous.
///
/// # Errors
///
/// Returns [`OperationError::Invalid`] if the frame is not packed `1920×1080` RGBA8.
pub fn observe_no_moves_modal(frame: &CapturedFrame) -> Result<Option<PixelPoint>, OperationError> {
    validate_native_frame(frame)?;
    let Ok(view) = frame.as_view() else {
        return Ok(None);
    };
    for (x, y) in [(200, 500), (1_700, 500), (1_320, 978)] {
        let Some(rgb) = view.pixel_rgb(x, y) else {
            return Ok(None);
        };
        if rgb[0].max(rgb[1]).max(rgb[2]) > 85 {
            return Ok(None);
        }
    }
    let mut matched = None;
    for layout in &NO_MOVES_MODAL_LAYOUTS {
        if matches_no_moves_modal_layout(view, layout) {
            if matched.is_some() {
                return Ok(None);
            }
            matched = Some(layout.click_point);
        }
    }
    Ok(matched)
}

#[cfg(test)]
mod tests {
    //! Native fixture, synthetic-colour `Threes`, Normal `Score`, deals-left, and modal regressions.

    use super::*;
    use crate::capture::decode_png;

    /// Decode a committed native PNG fixture into a packed `CapturedFrame`.
    fn decode(bytes: &[u8]) -> CapturedFrame {
        decode_png(bytes).expect("valid native fixture PNG")
    }

    /// Render a `WordTemplate` linearly onto a solid uniform ribbon background on a `1920×1080` copy.
    fn render_synthetic_banner(template: WordTemplate, bg: [u8; 3]) -> CapturedFrame {
        let mut frame = decode(include_bytes!(
            "../tests/fixtures/tripeaks-calibration-20261116/stock-last-card.png"
        ));
        let rail = [
            bg[0].saturating_add((255 - bg[0]) / 4),
            bg[1].saturating_add((255 - bg[1]) / 4),
            bg[2].saturating_add((255 - bg[2]) / 4),
        ];
        // Fill outer ribbon rectangle `(814, 41, 292, 46)` with lighter border rails and interior `bg`.
        for y in 41_u32..87 {
            for x in 814_u32..1_106 {
                let is_rail = x <= 816 || x >= 1_103 || y <= 43 || y >= 84;
                let rgb = if is_rail { rail } else { bg };
                let off = y as usize * frame.stride + x as usize * 4;
                frame.pixels[off..off + 3].copy_from_slice(&rgb);
            }
        }
        // Stamp `template` + `TEMPLATE_CLEARED` + `0/3` from `fours` ribbon colour delta.
        let stamp =
            |f: &mut CapturedFrame, x0: usize, y0: usize, bytes: &[u8], w: usize, h: usize| {
                for dy in 0..h {
                    for dx in 0..w {
                        let ink = u16::from(bytes[dy * w + dx]);
                        if ink == 0 {
                            continue;
                        }
                        let rgb = [
                            (u16::from(bg[0]) + (ink * u16::from(255 - bg[0])) / 255) as u8,
                            (u16::from(bg[1]) + (ink * u16::from(255 - bg[1])) / 255) as u8,
                            (u16::from(bg[2]) + (ink * u16::from(255 - bg[2])) / 255) as u8,
                        ];
                        let off = (y0 + dy) * f.stride + (x0 + dx) * 4;
                        f.pixels[off..off + 3].copy_from_slice(&rgb);
                    }
                }
            };
        stamp(
            &mut frame,
            845,
            48,
            template.bytes,
            template.width,
            template.height,
        );
        stamp(&mut frame, 922, 48, TEMPLATE_CLEARED, 81, WORD_HEIGHT_HUD);
        // Copy counter ink from native `freecell-empty-play1-fours-20261113.png` (`0/3`) at `x=1044..1090`.
        let src_fours = decode(include_bytes!(
            "../tests/fixtures/freecell-empty-play1-fours-20261113.png"
        ));
        let fours_bg = [125_u8, 39, 39];
        let c = min_channel_index(fours_bg);
        let denom = u16::from(255 - fours_bg[c]);
        for y in 48_usize..80 {
            for x in 1_042_usize..1_096 {
                let so = y * src_fours.stride + x * 4;
                let rgb_pixel = [
                    src_fours.pixels[so],
                    src_fours.pixels[so + 1],
                    src_fours.pixels[so + 2],
                ];
                let ink = u16::from(white_blend_ink(
                    rgb_pixel,
                    fours_bg,
                    c,
                    denom,
                    CHANNEL_INK_TOLERANCE,
                ));
                if ink > 0 {
                    let rgb = [
                        (u16::from(bg[0]) + (ink * u16::from(255 - bg[0])) / 255) as u8,
                        (u16::from(bg[1]) + (ink * u16::from(255 - bg[1])) / 255) as u8,
                        (u16::from(bg[2]) + (ink * u16::from(255 - bg[2])) / 255) as u8,
                    ];
                    let doff = y * frame.stride + x * 4;
                    frame.pixels[doff..doff + 3].copy_from_slice(&rgb);
                }
            }
        }
        frame
    }

    #[test]
    fn challenge_header_recognises_twelve_native_ranks_boards_score_solve_and_clear() {
        let cases: &[(&[u8], TriPeaksChallenge)] = &[
            (
                include_bytes!("../tests/fixtures/pyramid-recon/native-v13/recon-seven.png"),
                TriPeaksChallenge::ClearRank {
                    rank: Rank::Ace,
                    current: 0,
                    target: 12,
                },
            ),
            (
                include_bytes!(
                    "../tests/fixtures/pyramid-colour-independence/twos-exhausted-green.png"
                ),
                TriPeaksChallenge::ClearRank {
                    rank: Rank::Two,
                    current: 12,
                    target: 15,
                },
            ),
            (
                include_bytes!("../tests/fixtures/freecell-empty-play1-fours-20261113.png"),
                TriPeaksChallenge::ClearRank {
                    rank: Rank::Four,
                    current: 0,
                    target: 3,
                },
            ),
            (
                include_bytes!("../tests/fixtures/pyramid-recognition-158/five-spades.png"),
                TriPeaksChallenge::ClearRank {
                    rank: Rank::Five,
                    current: 8,
                    target: 10,
                },
            ),
            (
                include_bytes!("../tests/fixtures/pyramid-card-cleared/sixes.png"),
                TriPeaksChallenge::ClearRank {
                    rank: Rank::Six,
                    current: 0,
                    target: 12,
                },
            ),
            (
                include_bytes!("../tests/fixtures/freecell-sixes-three-diamond-20261113.png"),
                TriPeaksChallenge::ClearRank {
                    rank: Rank::Six,
                    current: 0,
                    target: 2,
                },
            ),
            (
                include_bytes!("../tests/fixtures/freecell-sevens-halo-20261109.png"),
                TriPeaksChallenge::ClearRank {
                    rank: Rank::Seven,
                    current: 0,
                    target: 2,
                },
            ),
            (
                include_bytes!("../tests/fixtures/pyramid-challenge-redeal/eights.png"),
                TriPeaksChallenge::ClearRank {
                    rank: Rank::Eight,
                    current: 3,
                    target: 12,
                },
            ),
            (
                include_bytes!("../tests/fixtures/pyramid-card-cleared/nines.png"),
                TriPeaksChallenge::ClearRank {
                    rank: Rank::Nine,
                    current: 0,
                    target: 15,
                },
            ),
            (
                include_bytes!("../tests/fixtures/pyramid-stock-recycle/first-unmatched.png"),
                TriPeaksChallenge::ClearRank {
                    rank: Rank::Ten,
                    current: 8,
                    target: 12,
                },
            ),
            (
                include_bytes!("../tests/fixtures/pyramid-challenge-redeal/jacks.png"),
                TriPeaksChallenge::ClearRank {
                    rank: Rank::Jack,
                    current: 4,
                    target: 12,
                },
            ),
            (
                include_bytes!("../tests/fixtures/pyramid-recon/native-v14/baseline.png"),
                TriPeaksChallenge::ClearRank {
                    rank: Rank::Queen,
                    current: 8,
                    target: 15,
                },
            ),
            (
                include_bytes!("../tests/fixtures/pyramid-kings-default/kings-one.png"),
                TriPeaksChallenge::ClearRank {
                    rank: Rank::King,
                    current: 0,
                    target: 18,
                },
            ),
            (
                include_bytes!("../tests/fixtures/pyramid-boards-perfect-150/last-observed.png"),
                TriPeaksChallenge::Boards {
                    current: 2,
                    target: 3,
                },
            ),
            (
                include_bytes!("../tests/fixtures/pyramid-challenge-redeal/score.png"),
                TriPeaksChallenge::Score {
                    current: 200,
                    target: 2_400,
                },
            ),
            (
                include_bytes!("../tests/fixtures/freecell-native-eight-20261112.png"),
                TriPeaksChallenge::SolveDeck,
            ),
            (
                include_bytes!("../tests/fixtures/freecell-challenge-20261107.png"),
                TriPeaksChallenge::UnsupportedSingleCard,
            ),
        ];

        for &(png, expected) in cases {
            let frame = decode(png);
            let observed = observe_challenge_header(&frame).expect("valid frame");
            assert_eq!(observed, Some(expected));
        }
    }

    #[test]
    fn threes_recognised_across_six_ribbon_colours_and_two_kerning_variants() {
        let colours = [
            [149, 82, 0],  // amber
            [125, 39, 39], // ruby
            [24, 98, 60],  // emerald
            [86, 44, 128], // purple
            [28, 76, 146], // cobalt
            [58, 72, 86],  // slate
        ];
        for bytes in [TEMPLATE_THREES_SPACED, TEMPLATE_THREES_COMPACT] {
            let template = WordTemplate {
                width: 75,
                height: WORD_HEIGHT_HUD,
                bytes,
            };
            for bg in colours {
                let frame = render_synthetic_banner(template, bg);
                assert_eq!(
                    observe_challenge_header(&frame).expect("valid synthetic banner"),
                    Some(TriPeaksChallenge::ClearRank {
                        rank: Rank::Three,
                        current: 0,
                        target: 3,
                    })
                );
            }
        }
    }

    #[test]
    fn challenge_to_recon_objective_mapping_matches_contract() {
        assert_eq!(
            TriPeaksChallenge::SolveDeck.to_recon_objective(),
            Some(Objective::ClearBoard)
        );
        assert_eq!(
            TriPeaksChallenge::Boards {
                current: 1,
                target: 3,
            }
            .to_recon_objective(),
            Some(Objective::ClearBoard)
        );
        assert_eq!(
            TriPeaksChallenge::ClearRank {
                rank: Rank::Three,
                current: 4,
                target: 12,
            }
            .to_recon_objective(),
            Some(Objective::ClearRankQuota {
                rank: 2,
                remaining: 8,
            })
        );
        assert_eq!(
            TriPeaksChallenge::Score {
                current: 1_200,
                target: 3_500,
            }
            .to_recon_objective(),
            Some(Objective::ReachScoreDelta {
                points_needed: 2_300,
            })
        );
        assert_eq!(
            TriPeaksChallenge::UnsupportedSingleCard.to_recon_objective(),
            None
        );
    }

    #[test]
    fn normal_boards_and_normal_score_match_retained_native_captures() {
        let board_1 = decode(include_bytes!(
            "../tests/fixtures/pyramid-standard-series/normal-1.png"
        ));
        let board_2 = decode(include_bytes!(
            "../tests/fixtures/pyramid-standard-series/normal-2.png"
        ));
        let board_3 = decode(include_bytes!(
            "../tests/fixtures/pyramid-standard-series/normal-3.png"
        ));
        for (completed, frame) in [(0, &board_1), (1, &board_2), (2, &board_3)] {
            assert_eq!(
                observe_normal_boards(frame).expect("valid frame"),
                Some(BoardCount {
                    completed,
                    target: 3,
                })
            );
        }
        // Damaging the `New` footer rejects full `observe_normal_boards` while header still matches.
        let mut header_only = board_2;
        let footer_off = 1_010 * header_only.stride + 225 * 4;
        header_only.pixels[footer_off..footer_off + 3].copy_from_slice(&[0, 0, 0]);
        assert_eq!(
            observe_normal_boards(&header_only).expect("valid frame"),
            None
        );
        assert_eq!(
            observe_normal_boards_header(&header_only).expect("valid frame"),
            Some(BoardCount {
                completed: 1,
                target: 3,
            })
        );

        let scores: &[(&[u8], Option<u32>)] = &[
            (
                include_bytes!(
                    "../tests/fixtures/tripeaks-calibration-20261116/stock-last-card.png"
                ),
                Some(0),
            ),
            (
                include_bytes!("../tests/fixtures/tripeaks-calibration-20261116/stock-empty.png"),
                Some(0),
            ),
            (
                include_bytes!(
                    "../tests/fixtures/tripeaks-calibration-20261116/tableau-row2-uncovered.png"
                ),
                Some(4_600),
            ),
            (
                include_bytes!(
                    "../tests/fixtures/tripeaks-calibration-20261116/peak1-uncovered.png"
                ),
                Some(5_000),
            ),
            (
                include_bytes!("../tests/fixtures/pyramid-standard-series/normal-2.png"),
                Some(1_280),
            ),
            (
                include_bytes!("../tests/fixtures/pyramid-standard-series/normal-3.png"),
                Some(2_550),
            ),
            (
                include_bytes!("../tests/fixtures/pyramid-normal-final-152/last-observed.png"),
                Some(3_780),
            ),
            (
                include_bytes!(
                    "../tests/fixtures/pyramid-testing-156/normal-final-king-spades.png"
                ),
                Some(3_770),
            ),
            (
                include_bytes!(
                    "../tests/fixtures/tripeaks-calibration-20261116/no-moves-remaining.png"
                ),
                None,
            ),
            (include_bytes!("../tests/fixtures/freecell-FC01.png"), None),
            (include_bytes!("../tests/fixtures/spider-SP01.png"), None),
            (
                include_bytes!("../tests/fixtures/pyramid-challenge-redeal/score.png"),
                None,
            ),
        ];

        for &(png, expected) in scores {
            let frame = decode(png);
            assert_eq!(observe_normal_score(&frame).expect("valid frame"), expected);
            if expected.is_some() {
                assert_eq!(observe_challenge_header(&frame).expect("valid frame"), None);
            }
        }
    }

    #[test]
    fn deals_left_caption_separates_zero_one_two_from_blank_and_dimmed() {
        let zero = decode(include_bytes!(
            "../tests/fixtures/tripeaks-redeal-20261016/draw-first.png"
        ));
        let one = decode(include_bytes!(
            "../tests/fixtures/pyramid-challenge-redeal/jacks.png"
        ));
        let two = decode(include_bytes!(
            "../tests/fixtures/pyramid-redeal-appearance/count-two.png"
        ));
        let blank_normal = decode(include_bytes!(
            "../tests/fixtures/tripeaks-calibration-20261116/stock-empty.png"
        ));
        let dimmed = decode(include_bytes!(
            "../tests/fixtures/tripeaks-calibration-20261116/no-moves-remaining.png"
        ));

        assert_eq!(
            observe_deals_left_caption(&zero).expect("valid frame"),
            Some(0)
        );
        assert_eq!(
            observe_deals_left_caption(&one).expect("valid frame"),
            Some(1)
        );
        assert_eq!(
            observe_deals_left_caption(&two).expect("valid frame"),
            Some(2)
        );
        assert_eq!(
            observe_deals_left_caption(&blank_normal).expect("valid frame"),
            None
        );
        assert_eq!(
            observe_deals_left_caption(&dimmed).expect("valid frame"),
            None
        );
    }

    #[test]
    fn no_moves_modal_detects_live_capture_and_rejects_normal_and_corrupted_controls() {
        let modal = decode(include_bytes!(
            "../tests/fixtures/tripeaks-calibration-20261116/no-moves-remaining.png"
        ));
        let compact_modal = decode(include_bytes!(
            "../tests/fixtures/tripeaks-calibration-20261116/no-moves-remaining-compact.png"
        ));
        let stock_empty = decode(include_bytes!(
            "../tests/fixtures/tripeaks-calibration-20261116/stock-empty.png"
        ));
        let stock_last = decode(include_bytes!(
            "../tests/fixtures/tripeaks-calibration-20261116/stock-last-card.png"
        ));
        let tp01 = decode(include_bytes!("../tests/fixtures/tripeaks-TP01.png"));
        let tp02 = decode(include_bytes!("../tests/fixtures/tripeaks-TP02.png"));
        let tp03 = decode(include_bytes!("../tests/fixtures/tripeaks-TP03.png"));

        assert_eq!(
            observe_no_moves_modal(&modal).expect("valid frame"),
            Some(UNDO_LAST_MOVE_CLICK_POINT)
        );
        assert_eq!(
            observe_no_moves_modal(&compact_modal).expect("valid compact modal frame"),
            Some(UNDO_LAST_MOVE_COMPACT_CLICK_POINT)
        );
        assert_ne!(
            UNDO_LAST_MOVE_COMPACT_CLICK_POINT,
            UNDO_LAST_MOVE_CLICK_POINT
        );
        for non_modal in [&stock_empty, &stock_last, &tp01, &tp02, &tp03] {
            assert_eq!(
                observe_no_moves_modal(non_modal).expect("valid frame"),
                None
            );
        }

        // Corrupting the active blue `"Undo last move"` button must fail closed for both layouts.
        let mut corrupted_btn = modal;
        let off = 745 * corrupted_btn.stride + 1_128 * 4;
        corrupted_btn.pixels[off..off + 3].copy_from_slice(&[20, 20, 20]);
        assert_eq!(
            observe_no_moves_modal(&corrupted_btn).expect("valid frame"),
            None
        );

        let mut corrupted_compact_btn = compact_modal.clone();
        let compact_off = 622 * corrupted_compact_btn.stride + 1_133 * 4;
        corrupted_compact_btn.pixels[compact_off..compact_off + 3].copy_from_slice(&[20, 20, 20]);
        assert_eq!(
            observe_no_moves_modal(&corrupted_compact_btn).expect("valid frame"),
            None
        );

        // Subtle suit-watermark spread (`17` Blue units) flanking the compact title must still match.
        let mut watermarked = compact_modal.clone();
        let left_wm = 480 * watermarked.stride + 640 * 4;
        watermarked.pixels[left_wm..left_wm + 3].copy_from_slice(&[32, 92, 167]);
        let right_wm = 480 * watermarked.stride + 1_260 * 4;
        watermarked.pixels[right_wm..right_wm + 3].copy_from_slice(&[20, 78, 150]);
        assert_eq!(
            observe_no_moves_modal(&watermarked).expect("watermarked compact modal"),
            Some(UNDO_LAST_MOVE_COMPACT_CLICK_POINT)
        );

        // Corrupting the compact modal title text must also fail closed.
        let mut corrupted_compact_title = compact_modal;
        for y in 460..500_usize {
            for x in 735..1_180_usize {
                let title_off = y * corrupted_compact_title.stride + x * 4;
                corrupted_compact_title.pixels[title_off..title_off + 3]
                    .copy_from_slice(&[24, 80, 154]);
            }
        }
        assert_eq!(
            observe_no_moves_modal(&corrupted_compact_title).expect("valid frame"),
            None
        );
    }

    #[test]
    fn pairwise_primary_word_separation_exceeds_margin() {
        for (idx_a, &(class_a, template_a)) in PRIMARY_WORD_CATALOGUE.iter().enumerate() {
            let mut pixels = vec![0_u8; 282 * WORD_HEIGHT_HUD];
            for y in 0..template_a.height {
                for x in 0..template_a.width {
                    pixels[y * 282 + (20 + x)] = template_a.bytes[y * template_a.width + x];
                }
            }
            let band = WhiteInkBand {
                pixels,
                width: 282,
                height: WORD_HEIGHT_HUD,
            };
            let span = InkSpan {
                start: 22,
                end: 20 + template_a.width - 2,
            };
            let zero = RatioDistance {
                difference: 0,
                total: 1,
            };
            for (idx_b, &(class_b, template_b)) in PRIMARY_WORD_CATALOGUE.iter().enumerate() {
                if idx_a == idx_b || class_a == class_b {
                    continue;
                }
                let dist = band
                    .match_at_span(span, template_b)
                    .expect("non-empty word distance");
                assert!(
                    zero.margin_before(dist, WORD_MARGIN),
                    "primary templates {class_a:?} vs {class_b:?} separated by < {WORD_MARGIN} ppm ({}/{})",
                    dist.difference,
                    dist.total
                );
            }
        }
    }
}
