//! Versioned native glyph scoring with full-ROI paper-relative ink distance.
//!
//! Each accepted label has one translated RGB witness satisfying its score,
//! raw paper and exclusive strong-colour requirements together. Actual and
//! template paper references are independent lower medians of supported bright
//! pixels in that same ROI. Every pixel contributes its paper-relative ink;
//! paper brightness above the reference deliberately contributes zero ink.
//! Recognition never consults a deck or grants input authority.

use std::cmp::Ordering;

use serde_json::Value;
use xsar::image_matching::FrameView;

use super::{Pattern, PixelRect};


/// Fixed ratio scale used only for policy encoding and diagnostic presentation.
const SCORE_SCALE: u64 = 1_000_000;


/// Inclusive calibrated rank error bound in parts per million.
const RANK_MAX_ERROR_PPM: u32 = 220_000;


/// Inclusive calibrated suit error bound in parts per million.
const SUIT_MAX_ERROR_PPM: u32 = 120_000;


/// Inclusive minimum separation from the closest different label.
const MIN_LABEL_GAP_PPM: u32 = 80_000;


/// Maximum decoded glyph area; all pixels participate in every score.
const MAX_PIXELS: usize = 4096;


/// Immutable schema-2 policy; arbitrary operator threshold relaxation is refused.
#[derive(Clone, Copy, Debug)]
pub(super) struct Policy;


impl Policy {
    /// Parse exactly the frozen `paper_ink_l1_v2` object without extra keys.
    pub(super) fn parse(value: &Value) -> Result<Self, String> {
        let fields = super::object(
            value,
            &[
                "policy",
                "score_scale",
                "rank_max_error_ppm",
                "suit_max_error_ppm",
                "min_label_gap_ppm",
            ],
        )?;
        let exact = fields.get("policy").and_then(Value::as_str) == Some("paper_ink_l1_v2")
            && fields.get("score_scale").and_then(Value::as_u64) == Some(SCORE_SCALE)
            && fields.get("rank_max_error_ppm").and_then(Value::as_u64)
                == Some(u64::from(RANK_MAX_ERROR_PPM))
            && fields.get("suit_max_error_ppm").and_then(Value::as_u64)
                == Some(u64::from(SUIT_MAX_ERROR_PPM))
            && fields.get("min_label_gap_ppm").and_then(Value::as_u64)
                == Some(u64::from(MIN_LABEL_GAP_PPM));
        if !exact {
            return Err(
                "paper_ink_l1_v2 policy requires its exact frozen integer values".to_owned(),
            );
        }

        Ok(Self)
    }


    /// Return the frozen inclusive error bound for a rank or suit respectively.
    pub(super) fn max_error_ppm(self, rank: bool) -> u32 {
        if rank {
            RANK_MAX_ERROR_PPM
        } else {
            SUIT_MAX_ERROR_PPM
        }
    }


    /// Return the frozen inclusive different-label score separation in ppm.
    pub(super) fn min_gap_ppm(self) -> u32 {
        MIN_LABEL_GAP_PPM
    }
}


/// Exact non-zero-denominator distance; flooring is forbidden in decisions.
#[derive(Clone, Copy, Debug)]
struct Ratio {
    /// Sum of absolute differences between observed and template ink strengths.
    numerator: u64,
    /// Sum of the greater observed/template ink strength at every ROI pixel.
    denominator: u64,
}


impl Ratio {
    /// Order two bounded fractions by checked exact cross-products.
    fn compare(self, other: Self) -> Result<Ordering, String> {
        Ok(product(self.numerator, other.denominator)?
            .cmp(&product(other.numerator, self.denominator)?))
    }


    /// Apply an inclusive absolute quality bound without rounding the fraction.
    fn within(self, maximum_ppm: u32) -> Result<bool, String> {
        Ok(product(self.numerator, SCORE_SCALE)?
            <= product(u64::from(maximum_ppm), self.denominator)?)
    }


    /// Require a positive, inclusive minimum gap to an ordered competitor.
    fn separated_from(self, other: Self, minimum_ppm: u32) -> Result<bool, String> {
        let other_scaled = product(other.numerator, self.denominator)?;
        let self_scaled = product(self.numerator, other.denominator)?;
        let Some(gap) = other_scaled.checked_sub(self_scaled) else {
            return Err("graded glyph competitor precedes the selected label".to_owned());
        };
        Ok(gap != 0
            && product(gap, SCORE_SCALE)?
                >= product(
                    product(self.denominator, other.denominator)?,
                    u64::from(minimum_ppm),
                )?)
    }


    /// Return a floored ppm value for explanation only; never use it to decide.
    fn diagnostic_ppm(self) -> u64 {
        // Both terms are bounded by MAX_PIXELS * 255 before construction.
        self.numerator * SCORE_SCALE / self.denominator
    }
}


/// Complete observed evidence from one template and translated ROI.
#[derive(Clone, Copy, Debug)]
struct Witness<Label> {
    /// Rank or suit named by the measured template.
    label: Label,
    /// Exact full-ROI shape error at this position.
    score: Ratio,
    /// Actual integer translation in pixels relative to the declared glyph ROI.
    shift: (i32, i32),
    /// Zero-based measured template variant, retained only for diagnostics.
    variant: usize,
    /// Number of actual pixels with every RGB channel at least 180.
    paper: usize,
    /// Lower-median minimum RGB value of supported actual paper pixels.
    actual_paper_reference: u8,
    /// Independent lower-median minimum RGB value of template paper pixels.
    template_paper_reference: u8,
    /// Number of actual strongly red pixels under the existing raw RGB predicate.
    red: usize,
    /// Number of actual strongly black pixels under the existing raw predicate.
    black: usize,
}


impl<Label: Copy> Witness<Label> {
    /// Return exclusive ink colour only when eight strong pixels establish it.
    fn exclusive_red(self) -> Option<bool> {
        if self.red >= 8 && self.black == 0 {
            Some(true)
        } else if self.black >= 8 && self.red == 0 {
            Some(false)
        } else {
            None
        }
    }
}


/// A translated ROI either supplies one complete witness or a distinct refusal.
#[derive(Clone, Copy, Debug)]
enum Observation<Label> {
    /// Shape, raw colour and paper reference from the same translated pixels.
    Witness(Witness<Label>),
    /// Actual pixels cannot establish the required supported paper reference.
    PaperRefused,
    /// Both full-ROI normalised ink masses are zero despite supported paper.
    ZeroMass,
}


/// Bounded diagnostic counts; failures never become fallback recognition.
#[derive(Clone, Copy, Debug, Default)]
struct Counts {
    /// Number of translated template observations inspected.
    observed: usize,
    /// Observations with no ink mass in either template or actual pixels.
    zero_mass: usize,
    /// Observations below the unchanged full-ROI paper requirement.
    insufficient_paper: usize,
    /// Observations lacking exactly one valid strong ink colour.
    invalid_colour: usize,
}


/// Positively accepted label and its co-located colour/translation witness.
#[derive(Clone, Copy, Debug)]
pub(super) struct Hit<Label> {
    /// Accepted rank or suit; competing labels passed an exact separation check.
    pub(super) label: Label,
    /// True for exclusive strong red ink, false for exclusive strong black ink.
    pub(super) red: bool,
}


/// Complete decision evidence retained even when recognition refuses a label.
#[derive(Debug)]
pub(super) struct Evaluation<Label> {
    /// Best eligible observation for any label, including scores above the cap.
    winner: Option<Witness<Label>>,
    /// Best eligible observation belonging to a different label.
    competitor: Option<Witness<Label>>,
    /// Closest scored observation even when paper or exclusive colour refused.
    nearest: Option<Witness<Label>>,
    /// Positive result only after absolute quality and different-label separation.
    hit: Option<Hit<Label>>,
    /// Human-readable fixed refusal/acceptance reason without inferred identity.
    reason: &'static str,
    /// Actual bounded guard rejection counts.
    counts: Counts,
}


impl<Label: Copy> Evaluation<Label> {
    /// Return only a fully accepted, co-located label and colour witness.
    pub(super) fn accepted(&self) -> Option<Hit<Label>> {
        self.hit
    }


    /// Explain closest/competing scores, exact shifts and actual guard refusals.
    pub(super) fn describe(&self, name: fn(Label) -> &'static str) -> String {
        let describe_witness = |witness: Witness<Label>| {
            format!(
                "{} error={}/{} ({}ppm), shift=({},{}), variant={}, paper={}, actual-paper-reference={}, template-paper-reference={}, red={}, black={}",
                name(witness.label),
                witness.score.numerator,
                witness.score.denominator,
                witness.score.diagnostic_ppm(),
                witness.shift.0,
                witness.shift.1,
                witness.variant,
                witness.paper,
                witness.actual_paper_reference,
                witness.template_paper_reference,
                witness.red,
                witness.black,
            )
        };
        let closest = self
            .winner
            .or(self.nearest)
            .map(describe_witness)
            .unwrap_or_else(|| "none".to_owned());
        let competitor = self
            .competitor
            .map(describe_witness)
            .unwrap_or_else(|| "none".to_owned());
        format!(
            "paper_ink_l1_v2 {}; closest=[{}]; other-label=[{}]; observations={}, zero-mass={}, paper-refusals={}, colour-refusals={}",
            self.reason,
            closest,
            competitor,
            self.counts.observed,
            self.counts.zero_mass,
            self.counts.insufficient_paper,
            self.counts.invalid_colour,
        )
    }
}


/// Evaluate bounded measured labels without deck knowledge or nearest-only fallback.
///
/// Every full-ROI pixel contributes `abs(actual-template) / max(actual,template)`
/// through sums, using strength `floor(255*max(paper-min(R,G,B),0)/paper)`.
/// Actual and template paper references are independent supported lower medians.
/// Each label contributes its best colour-eligible translated observation,
/// including competitors above the quality cap. Missing actual paper is refused.
/// Invalid geometry, excluded pixels or checked-arithmetic failures return errors.
pub(super) fn evaluate<Label: Copy + PartialEq>(
    bank: &[(Label, Vec<Pattern>)],
    view: FrameView<'_>,
    origin: (u32, u32),
    maximum_error_ppm: u32,
    min_gap_ppm: u32,
    alignment: i32,
) -> Result<Evaluation<Label>, String> {
    if bank.is_empty()
        || bank.len() > 13
        || !(0..=1).contains(&alignment)
        || u64::from(maximum_error_ppm) > SCORE_SCALE
        || u64::from(min_gap_ppm) > SCORE_SCALE
    {
        return Err("graded glyph bank, alignment or ratio bound is invalid".to_owned());
    }
    let mut labels: Vec<Witness<Label>> = Vec::with_capacity(bank.len());
    let mut nearest = None;
    let mut counts = Counts::default();
    for (label, variants) in bank {
        if variants.is_empty() || variants.len() > 16 {
            return Err("graded glyph variant count is outside 1..=16".to_owned());
        }
        for (variant, pattern) in variants.iter().enumerate() {
            validate_pattern(pattern)?;
            for shift_y in -alignment..=alignment {
                for shift_x in -alignment..=alignment {
                    counts.observed += 1;
                    let witness = match observe(
                        *label,
                        variant,
                        pattern,
                        view,
                        origin,
                        (shift_x, shift_y),
                    )? {
                        Observation::Witness(witness) => witness,
                        Observation::PaperRefused => {
                            counts.insufficient_paper += 1;
                            continue;
                        }
                        Observation::ZeroMass => {
                            counts.zero_mass += 1;
                            continue;
                        }
                    };
                    replace_if_better(&mut nearest, witness)?;
                    let colour_ok = witness.exclusive_red().is_some();
                    counts.invalid_colour += usize::from(!colour_ok);
                    if !colour_ok {
                        continue;
                    }
                    if let Some(previous) = labels.iter_mut().find(|item| item.label == *label) {
                        if witness.score.compare(previous.score)? == Ordering::Less {
                            *previous = witness;
                        }
                    } else {
                        labels.push(witness);
                    }
                }
            }
        }
    }
    let mut winner = None;
    for witness in &labels {
        replace_if_better(&mut winner, *witness)?;
    }
    let mut competitor = None;
    if let Some(best) = winner {
        for witness in &labels {
            if witness.label != best.label {
                replace_if_better(&mut competitor, *witness)?;
            }
        }
    }
    let mut hit = None;
    let reason = if let Some(best) = winner {
        if !best.score.within(maximum_error_ppm)? {
            "refused: absolute shape error"
        } else if let Some(other) = competitor
            && !best.score.separated_from(other.score, min_gap_ppm)?
        {
            "refused: competing-label ambiguity"
        } else if let Some(red) = best.exclusive_red() {
            hit = Some(Hit {
                label: best.label,
                red,
            });
            "accepted"
        } else {
            return Err("graded glyph lost its co-located colour witness".to_owned());
        }
    } else {
        "refused: no paper/colour-eligible non-zero-mass observation"
    };

    Ok(Evaluation {
        winner,
        competitor,
        nearest,
        hit,
        reason,
        counts,
    })
}


/// Validate a full bounded template and its raw-paper support before use.
///
/// Schema-2 construction and runtime evaluation share this check. An invalid
/// template is an error rather than an actual-frame unknown observation.
pub(super) fn validate_pattern(pattern: &Pattern) -> Result<(), String> {
    let area = pattern
        .bounds
        .width
        .checked_mul(pattern.bounds.height)
        .and_then(|area| usize::try_from(area).ok())
        .ok_or_else(|| "graded glyph area overflow".to_owned())?;
    if !(1..=MAX_PIXELS).contains(&area)
        || pattern.pixels.len() != area
        || pattern.tolerance != 0
        || pattern.pixels.iter().any(Option::is_none)
    {
        return Err("graded glyph requires a full bounded RGB mask with zero tolerance".to_owned());
    }
    let paper = pattern
        .pixels
        .iter()
        .flatten()
        .filter(|rgb| rgb.iter().all(|channel| *channel >= 180))
        .count();
    if paper == 0 || paper < area / 3 {
        return Err("graded glyph template lacks its supported paper reference".to_owned());
    }

    Ok(())
}


/// Measure one translated ROI using independently supported paper references.
///
/// Temporary minima and histograms are bounded by `MAX_PIXELS` and 256 bins.
/// Missing actual paper and zero total ink mass remain distinct unknown results;
/// a missing template reference or invalid geometry returns an error. The caller
/// first validates the template with `validate_pattern`.
fn observe<Label: Copy>(
    label: Label,
    variant: usize,
    pattern: &Pattern,
    view: FrameView<'_>,
    origin: (u32, u32),
    shift: (i32, i32),
) -> Result<Observation<Label>, String> {
    let column = i64::from(origin.0) + i64::from(pattern.bounds.x) + i64::from(shift.0);
    let row = i64::from(origin.1) + i64::from(pattern.bounds.y) + i64::from(shift.1);
    let column = u32::try_from(column).map_err(|_| "graded glyph x overflow".to_owned())?;
    let row = u32::try_from(row).map_err(|_| "graded glyph y overflow".to_owned())?;
    let bounds = PixelRect::new(column, row, pattern.bounds.width, pattern.bounds.height);
    view.checked_bounds(bounds)
        .map_err(|_| "graded glyph is outside the frame".to_owned())?;
    let mut actual_minima = [0_u8; MAX_PIXELS];
    let mut template_minima = [0_u8; MAX_PIXELS];
    let mut actual_paper_histogram = [0_usize; 256];
    let mut template_paper_histogram = [0_usize; 256];
    let mut paper = 0;
    let mut template_paper = 0;
    let mut red = 0;
    let mut black = 0;
    for (index, expected) in pattern.pixels.iter().enumerate() {
        let expected = expected.ok_or_else(|| "graded glyph excludes negative space".to_owned())?;
        let offset = u32::try_from(index).map_err(|_| "graded glyph offset overflow".to_owned())?;
        let x = column
            .checked_add(offset % bounds.width)
            .ok_or_else(|| "graded glyph pixel x overflow".to_owned())?;
        let y = row
            .checked_add(offset / bounds.width)
            .ok_or_else(|| "graded glyph pixel y overflow".to_owned())?;
        let actual = view
            .pixel_rgb(x, y)
            .ok_or_else(|| "graded glyph pixel is outside the frame".to_owned())?;
        let actual_minimum = minimum_channel(actual);
        let template_minimum = minimum_channel(expected);
        *actual_minima
            .get_mut(index)
            .ok_or_else(|| "graded glyph actual buffer bound exceeded".to_owned())? =
            actual_minimum;
        *template_minima
            .get_mut(index)
            .ok_or_else(|| "graded glyph template buffer bound exceeded".to_owned())? =
            template_minimum;
        if actual_minimum >= 180 {
            paper += 1;
            actual_paper_histogram[usize::from(actual_minimum)] += 1;
        }
        if template_minimum >= 180 {
            template_paper += 1;
            template_paper_histogram[usize::from(template_minimum)] += 1;
        }
        let [actual_red, actual_green, actual_blue] = actual;
        red += usize::from(
            actual_red >= 160
                && actual_red.saturating_sub(actual_green) >= 80
                && actual_red.saturating_sub(actual_blue) >= 80,
        );
        black += usize::from(actual.iter().all(|channel| *channel <= 100));
    }
    if paper == 0 || paper < pattern.pixels.len() / 3 {
        return Ok(Observation::PaperRefused);
    }
    let Some(actual_paper_reference) = lower_paper_reference(&actual_paper_histogram, paper) else {
        return Ok(Observation::PaperRefused);
    };
    if template_paper == 0 || template_paper < pattern.pixels.len() / 3 {
        return Err("graded glyph template lacks its supported paper reference".to_owned());
    }
    let template_paper_reference = lower_paper_reference(&template_paper_histogram, template_paper)
        .ok_or_else(|| "graded glyph template lacks its supported paper reference".to_owned())?;
    let mut numerator = 0_u64;
    let mut denominator = 0_u64;
    for index in 0..pattern.pixels.len() {
        let actual_strength = strength(actual_minima[index], actual_paper_reference)?;
        let template_strength = strength(template_minima[index], template_paper_reference)?;
        numerator = numerator
            .checked_add(u64::from(actual_strength.abs_diff(template_strength)))
            .ok_or_else(|| "graded glyph difference overflow".to_owned())?;
        denominator = denominator
            .checked_add(u64::from(actual_strength.max(template_strength)))
            .ok_or_else(|| "graded glyph mass overflow".to_owned())?;
    }
    if denominator == 0 {
        return Ok(Observation::ZeroMass);
    }

    Ok(Observation::Witness(Witness {
        label,
        score: Ratio {
            numerator,
            denominator,
        },
        shift,
        variant,
        paper,
        actual_paper_reference,
        template_paper_reference,
        red,
        black,
    }))
}


/// Return the minimum raw RGB channel without changing source pixels.
fn minimum_channel([red, green, blue]: [u8; 3]) -> u8 {
    red.min(green).min(blue)
}


/// Return the supported lower median from the 180..=255 paper bins only.
///
/// `paper` is the counted raw-paper population of this exact ROI. Empty or
/// inconsistent histograms yield no reference; no white-paper fallback exists.
fn lower_paper_reference(histogram: &[usize; 256], paper: usize) -> Option<u8> {
    if paper == 0 || paper > MAX_PIXELS {
        return None;
    }
    let population = histogram[180..]
        .iter()
        .try_fold(0_usize, |sum, count| sum.checked_add(*count))?;
    if population != paper {
        return None;
    }
    let target = (paper - 1) / 2;
    let mut cumulative = 0_usize;
    for reference in 180_u8..=255 {
        cumulative = cumulative.checked_add(histogram[usize::from(reference)])?;
        if cumulative > target {
            return Some(reference);
        }
    }

    None
}


/// Return floored paper-relative ink in `0..=255`, retaining all ROI pixels.
///
/// The supported reference is 180..=255. Pixels at or above it have zero ink;
/// the checked `u32` product is at most 65,025 and no divide-by-zero is possible.
fn strength(minimum: u8, paper_reference: u8) -> Result<u8, String> {
    if paper_reference < 180 {
        return Err("graded glyph paper reference is outside 180..=255".to_owned());
    }
    let difference = paper_reference.saturating_sub(minimum);
    let scaled = u32::from(difference)
        .checked_mul(255)
        .ok_or_else(|| "graded glyph strength overflow".to_owned())?;
    u8::try_from(scaled / u32::from(paper_reference))
        .map_err(|_| "graded glyph strength exceeds 255".to_owned())
}


/// Retain the closest complete witness; equal scores keep deterministic bank order.
fn replace_if_better<Label: Copy>(
    previous: &mut Option<Witness<Label>>,
    next: Witness<Label>,
) -> Result<(), String> {
    if let Some(current) = previous
        && next.score.compare(current.score)? != Ordering::Less
    {
        return Ok(());
    }
    *previous = Some(next);
    Ok(())
}


/// Multiply bounded ratio terms explicitly; arithmetic failure never grants a match.
fn product(first: u64, second: u64) -> Result<u64, String> {
    first
        .checked_mul(second)
        .ok_or_else(|| "graded glyph ratio overflow".to_owned())
}


#[cfg(test)]
#[path = "graded_glyph_tests.rs"]
mod tests;
