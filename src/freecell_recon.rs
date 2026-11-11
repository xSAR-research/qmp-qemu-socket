//! Fixed-position Free Cell identity extraction and live initial-board binding.
//! Physical columns and depths are never sorted or inferred from the remaining deck.
use crate::{
    capture::{CapturedFrame, decode_png},
    freecell::{CARD_WIDTH, CELL_X, FOUNDATION_X, PLAY_X},
};
use xsar::games::{
    cards::{Card, Rank, Suit},
    freecell::{BoardState, CascadeStack, INITIAL_LENGTHS, InitialDeal, Metadata},
};

pub(crate) mod objective;
pub(crate) mod position;

/// Identity of this one measured, unscaled native profile.
pub(crate) const PROFILE: &str = "native-freecell-v1";

/// Typed first failure; no unreadable identity is inferred from inventory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FailureKind {
    /// Native storage, dimensions or crop are invalid.
    InvalidFrame,
    /// A required game control or excluded Solver state does not match this profile.
    UnsupportedScene,
    /// A top slot lacks its complete positive empty signature.
    TopSlotNotEmpty,
    /// No label meets absolute evidence quality.
    UnknownIdentity,
    /// Competing distinct labels lack required separation.
    AmbiguousIdentity,
    /// Rank and suit disagree on red/black print colour.
    ColourDisagreement,
    /// An actual printed identity occurs twice.
    DuplicateIdentity,
    /// The recognised physical inventory is not a complete initial deal.
    InvalidDeck,
    /// The reserved context or cancellation authority was revoked.
    Revoked,
    /// Bundled calibration is internally invalid.
    Calibration,
}

/// Physical location retained with the first error; indices here are zero-based.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Place {
    /// Whole-frame or calibration evidence.
    Scene,
    /// Original PLAY column and depth.
    Cascade {
        /// Original physical column, zero-based.
        column: usize,
        /// Original top-to-bottom depth, zero-based.
        depth: usize,
    },
    /// Physical CELL slot.
    Cell(usize),
    /// Physical SUIT slot, independent of canonical model suit ordering.
    Foundation(usize),
}

impl std::fmt::Display for Place {
    /// Render fixed physical locations one-based while retaining zero-based typed storage.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Scene => formatter.write_str("scene"),
            Self::Cascade { column, depth } => {
                write!(formatter, "PLAY {} depth {}", column + 1, depth + 1)
            }
            Self::Cell(slot) => write!(formatter, "CELL {}", slot + 1),
            Self::Foundation(slot) => write!(formatter, "SUIT {}", slot + 1),
        }
    }
}

/// A bounded matching region within the physical location.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Region {
    /// Native storage.
    Frame,
    /// Stable Solver-off header.
    Header,
    /// Variable objective pixels, retained separately from scene and card identity.
    Objective,
    /// Complete positive empty top slot.
    TopSlot,
    /// Complete rank crop.
    Rank,
    /// Complete suit crop.
    Suit,
    /// Initial-deck model.
    Deck,
    /// Bounded bundled reference bank.
    Calibration,
}

/// Structured failure formatted only at the worker/UI boundary.
#[derive(Debug, thiserror::Error)]
#[error("{profile} {place} {region:?} {bounds:?}: {kind:?}: {cause}")]
pub(crate) struct RecognitionError {
    /// Measured profile identity.
    pub profile: &'static str,
    /// Original physical location.
    pub place: Place,
    /// Relevant image region or model boundary.
    pub region: Region,
    /// Complete pixel bounds when the failure concerns an image rectangle.
    pub bounds: Option<crate::geometry::PixelRect>,
    /// Caller-action category.
    pub kind: FailureKind,
    /// The first underlying cause, with bounded native candidate diagnostics.
    pub cause: String,
}

/// Build one first-cause record without replacing earlier evidence.
fn failure(
    place: Place,
    region: Region,
    bounds: Option<crate::geometry::PixelRect>,
    kind: FailureKind,
    cause: impl Into<String>,
) -> RecognitionError {
    crate::single_step_telemetry::record(|| {
        serde_json::json!({
            "event":"recognition-refusal", "profile":PROFILE, "place":place.to_string(),
            "region":format!("{region:?}"), "bounds":bounds.map(|r|[r.x,r.y,r.width,r.height]),
            "kind":format!("{kind:?}"),
        })
    });
    RecognitionError {
        profile: PROFILE,
        place,
        region,
        bounds,
        kind,
        cause: cause.into(),
    }
}

/// Yield to cancellation between bounded recognition units.
fn checkpoint(
    check: &mut impl FnMut() -> Result<(), String>,
    place: Place,
    region: Region,
) -> Result<(), RecognitionError> {
    crate::single_step_telemetry::record(
        || serde_json::json!({"event":"recognition-stage", "place":place.to_string(), "region":format!("{region:?}")}),
    );
    check().map_err(|cause| failure(place, region, None, FailureKind::Revoked, cause))
}

/// Preserve absolute-quality and competing-label refusals as distinct outcomes.
#[derive(Debug, thiserror::Error)]
#[error("{kind:?}: best {best}, next {next}")]
struct MatchFailure {
    /// Typed quality/separation refusal.
    kind: FailureKind,
    /// Best native pixel difference.
    best: usize,
    /// Nearest distinct-label difference.
    next: usize,
}

/// Independently labelled native calibration appearance; no capture is performed here.
const REFERENCE: &[u8] = include_bytes!("../tests/fixtures/freecell-FC13.png");


/// Native6D corner at (1373,597) from the retained normal-board3.1.9 refusal.
/// This explicitly cropped30×48 derivative supplies one additional rank appearance only.
const SIX_RANK_CORNER: &[u8] =
    include_bytes!("../tests/fixtures/freecell-normal-six-rank-corner-20261110.png");


/// Complete Menu and Back glyph bounds from the retained native game controls.
/// Their printed shape supplies scene evidence; arbitrary felt, objective and score pixels do not.
const NATIVE_CONTROL_REGIONS: [crate::geometry::PixelRect; 2] = [
    crate::geometry::PixelRect::new(20, 46, 24, 22),
    crate::geometry::PixelRect::new(90, 46, 27, 25),
];


/// Measured top-border origins, including native pixel rounding.
const ROW_Y: [u32; 7] = [331, 384, 437, 490, 543, 597, 650];
/// Manually read calibration labels in physical column/depth order.
const LABELS: [&[&str]; 8] = [
    &["JD", "QC", "KD", "4C", "AH", "4S", "AC"],
    &["2D", "AD", "6S", "TD", "TC", "8H", "9H"],
    &["TH", "9C", "QH", "KH", "QD", "JC", "5C"],
    &["2S", "2C", "5S", "6D", "TS", "5D", "QS"],
    &["4D", "KC", "7D", "AS", "2H", "3C"],
    &["JS", "6C", "3S", "6H", "8D", "3D"],
    &["9D", "7C", "5H", "7H", "4H", "8S"],
    &["JH", "7S", "8C", "9S", "3H", "KS"],
];
/// Complete selected rank or suit ink, with independently observed print colour.
struct Glyph {
    /// Complete bounded region's thresholded ink, including all background samples.
    ink: Vec<bool>,
    /// Independently observed red print; false denotes black print.
    red: bool,
}


/// Paper evidence permitted at a glyph crop; this never grants live input authority.
#[derive(Clone, Copy)]
enum PaperPolicy {
    /// Preserve calibrated native white paper for independently labelled reference banks.
    StrictWhite,
    /// Use local contrast, coherent ink and unchanged shape acceptance for observed identities.
    ReadOnlyRecon,
}


/// Labelled native template; the supplementary full-face rank retains an original strip's suit.
struct Template {
    /// Manually labelled identity of this native reference appearance.
    card: Card,
    /// Complete printed rank from a labelled native strip or supplementary corner.
    rank: Glyph,
    /// Complete original-strip suit evidence, unchanged by the additional rank appearance.
    suit: Glyph,
}


/// Add one independently labelled native6 rank while preserving the original suit competition.
/// The small suit bitmap is copied exactly from the existing6D calibration, never retrained.
fn append_native_six_rank(bank: &mut Vec<Template>) -> Result<(), String> {
    let card = label("6D")?;
    let suit = bank
        .iter()
        .find(|template| template.card == card)
        .map(|template| Glyph {
            ink: template.suit.ink.clone(),
            red: template.suit.red,
        })
        .ok_or("Original6D suit calibration unavailable")?;
    let corner = decode_png(SIX_RANK_CORNER).map_err(|e| e.to_string())?;
    bank.push(Template {
        card,
        rank: glyph(&corner, 0, 0, true)?,
        suit,
    });
    Ok(())
}


/// Parse bounded bundled provenance; this is never used to infer an observed identity.
fn label(code: &str) -> Result<Card, String> {
    let (rank, suit) = code.split_at(code.len().checked_sub(1).ok_or("Empty calibration label")?);
    let rank = Rank::ALL
        .into_iter()
        .find(|r| r.as_str() == if rank == "T" { "10" } else { rank })
        .ok_or("Invalid calibration rank")?;
    let suit = match suit {
        "C" => Suit::Clubs,
        "D" => Suit::Diamonds,
        "H" => Suit::Hearts,
        "S" => Suit::Spades,
        _ => return Err("Invalid calibration suit".into()),
    };
    Ok(Card::new(rank, suit))
}
/// Read one complete measured glyph and require native paper and coherent red/black ink.
fn glyph(frame: &CapturedFrame, x: u32, y: u32, rank: bool) -> Result<Glyph, String> {
    glyph_with_policy(frame, x, y, rank, PaperPolicy::StrictWhite)
}


/// Read one bounded glyph using measured local contrast and coherent ink for observed cards.
fn glyph_with_policy(
    frame: &CapturedFrame,
    x: u32,
    y: u32,
    rank: bool,
    policy: PaperPolicy,
) -> Result<Glyph, String> {
    let view = frame.as_view().map_err(|e| e.to_string())?;
    let (dx, dy, w, h) = if rank {
        (2, 7, 26, 23)
    } else {
        (5, 30, 18, 16)
    };
    let bounds = crate::geometry::PixelRect::new(x + dx, y + dy, w, h);
    view.checked_bounds(bounds).map_err(|e| e.to_string())?;
    let mut pixels = Vec::with_capacity((w * h) as usize);
    let mut paper = Vec::with_capacity((w * h) as usize);
    let (mut red, mut black) = (0, 0);
    for yy in y + dy..y + dy + h {
        for xx in x + dx..x + dx + w {
            let rgb = view.pixel_rgb(xx, yy).ok_or("Glyph pixel out of bounds")?;
            let [r, g, b] = rgb;
            let minimum = r.min(g).min(b);
            paper.push(minimum);
            pixels.push(minimum);
            if r > 140 && 5 * u32::from(r) > 8 * u32::from(g) && 5 * u32::from(r) > 8 * u32::from(b)
            {
                red += 1;
            }
            if r.max(g).max(b) < 160 {
                black += 1;
            }
        }
    }
    paper.sort_unstable();
    let p = *paper.get(paper.len() * 9 / 10).ok_or("Empty glyph")?;
    if (matches!(policy, PaperPolicy::StrictWhite) && p < 245)
        || red.max(black) < 70
        || red.min(black) > 10
    {
        return Err(match policy {
            PaperPolicy::StrictWhite => {
                "Unsupported paper, missing or mixed-colour Free Cell glyph".into()
            }
            PaperPolicy::ReadOnlyRecon => "Missing or mixed-colour Free Cell glyph".into(),
        });
    }
    Ok(Glyph {
        ink: pixels
            .into_iter()
            .map(|v| 100 * u32::from(p.saturating_sub(v)) > 45 * u32::from(p))
            .collect(),
        red: red > black,
    })
}


/// Re-crop complete reference RGB rectangles at bounded native offsets; never pad ink masks.
#[cfg(test)]
fn glyph_variants(frame: &CapturedFrame, x: u32, y: u32, rank: bool) -> Result<Vec<Glyph>, String> {
    glyph_variants_with_policy(frame, x, y, rank, PaperPolicy::StrictWhite)
}


/// Preserve native integer offset variants under an explicit crop paper policy.
fn glyph_variants_with_policy(
    frame: &CapturedFrame,
    x: u32,
    y: u32,
    rank: bool,
    paper: PaperPolicy,
) -> Result<Vec<Glyph>, String> {
    let centre = glyph_with_policy(frame, x, y, rank, paper)?;
    let red = centre.red;
    let mut variants = vec![centre];
    for dy in -1_i32..=1 {
        for dx in -1_i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let Some(xx) = x.checked_add_signed(dx) else {
                continue;
            };
            let Some(yy) = y.checked_add_signed(dy) else {
                continue;
            };
            if let Ok(candidate) = glyph_with_policy(frame, xx, yy, rank, paper)
                && candidate.red == red
            {
                variants.push(candidate);
            }
        }
    }
    Ok(variants)
}


/// Compare every actual pixel of two equally sized native glyph rectangles.
fn distance(actual: &Glyph, reference: &Glyph) -> usize {
    actual
        .ink
        .iter()
        .zip(&reference.ink)
        .filter(|(a, b)| a != b)
        .count()
}
/// Compete all distinct same-colour labels before accepting absolute quality and separation.
fn identify<T: Copy + Eq>(
    actual: &[Glyph],
    bank: &[Template],
    select: impl Fn(&Template) -> (T, &Glyph),
    limit: usize,
    gap: usize,
) -> Result<T, MatchFailure> {
    let mut scores: Vec<(T, usize)> = Vec::new();
    for t in bank {
        let (label, g) = select(t);
        if actual.first().is_none_or(|a| g.red != a.red) {
            continue;
        }
        let score = actual
            .iter()
            .map(|a| distance(a, g))
            .min()
            .unwrap_or(usize::MAX);
        if let Some((_, old)) = scores.iter_mut().find(|(l, _)| *l == label) {
            *old = (*old).min(score);
        } else {
            scores.push((label, score));
        }
    }
    scores.sort_by_key(|(_, score)| *score);
    let no_bank = || MatchFailure {
        kind: FailureKind::Calibration,
        best: usize::MAX,
        next: usize::MAX,
    };
    let best = scores.first().ok_or_else(no_bank)?;
    let second = scores.get(1).ok_or_else(no_bank)?;
    if best.1 > limit {
        return Err(MatchFailure {
            kind: FailureKind::UnknownIdentity,
            best: best.1,
            next: second.1,
        });
    }
    if second.1.saturating_sub(best.1) < gap {
        return Err(MatchFailure {
            kind: FailureKind::AmbiguousIdentity,
            best: best.1,
            next: second.1,
        });
    }
    Ok(best.0)
}
/// Recognise a native control's complete bright-ink shape, independent of its dark background RGB.
/// A filled rectangle or missing/occluded glyph cannot substitute for the retained control.
fn control_signature(
    frame: &CapturedFrame,
    reference: &CapturedFrame,
    rect: crate::geometry::PixelRect,
) -> Result<(), RecognitionError> {
    let place = Place::Scene;
    let region = Region::Header;
    crate::single_step_telemetry::record(
        || serde_json::json!({"event":"recognition-region", "place":place.to_string(), "region":format!("{region:?}"), "bounds":[rect.x,rect.y,rect.width,rect.height], "stage":"begin"}),
    );
    let bad = |cause: String| failure(place, region, Some(rect), FailureKind::InvalidFrame, cause);
    let view = frame.as_view().map_err(|e| bad(e.to_string()))?;
    let expected = reference.as_view().map_err(|e| bad(e.to_string()))?;
    view.checked_bounds(rect).map_err(|e| bad(e.to_string()))?;
    expected
        .checked_bounds(rect)
        .map_err(|e| bad(e.to_string()))?;
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            let ink =
                |pixel: Option<[u8; 3]>| pixel.is_some_and(|[r, g, b]| r.min(g).min(b) >= 245);
            if ink(view.pixel_rgb(x, y)) != ink(expected.pixel_rgb(x, y)) {
                return Err(failure(
                    place,
                    region,
                    Some(rect),
                    FailureKind::UnsupportedScene,
                    "Native game-control shape differs",
                ));
            }
        }
    }
    crate::single_step_telemetry::record(
        || serde_json::json!({"event":"recognition-region", "place":place.to_string(), "region":format!("{region:?}"), "bounds":[rect.x,rect.y,rect.width,rect.height], "stage":"passed"}),
    );
    Ok(())
}


/// Require positive native game controls and Solver exclusion, independently of canvas RGB.
fn supported_scene(
    frame: &CapturedFrame,
    reference: &CapturedFrame,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<objective::Objective, RecognitionError> {
    use crate::geometry::PixelRect;
    crate::game_operations::validate_native_frame(frame).map_err(|e| {
        failure(
            Place::Scene,
            Region::Frame,
            None,
            FailureKind::InvalidFrame,
            e.to_string(),
        )
    })?;
    for rect in NATIVE_CONTROL_REGIONS {
        checkpoint(check, Place::Scene, Region::Header)?;
        control_signature(frame, reference, rect)?;
    }
    checkpoint(check, Place::Scene, Region::Header)?;
    let solver = crate::freecell::solver_active(frame).map_err(|e| {
        failure(
            Place::Scene,
            Region::Header,
            None,
            FailureKind::InvalidFrame,
            e.to_string(),
        )
    })?;
    crate::single_step_telemetry::record(
        || serde_json::json!({"event":"recognition-region", "region":"Solver banner", "active":solver, "stage":if solver {"failed"} else {"passed"}}),
    );
    if solver {
        return Err(failure(
            Place::Scene,
            Region::Header,
            Some(PixelRect::new(825, 36, 270, 50)),
            FailureKind::UnsupportedScene,
            "Solver overlay is active; live Free Cell binding requires Solver off",
        ));
    }
    objective::observe(frame, reference)
}


/// Require native game controls, Solver exclusion and all eight initially empty top slots.
/// Printed tableau identities are read separately without decorative card geometry checks.
fn scene(
    frame: &CapturedFrame,
    reference: &CapturedFrame,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<objective::Objective, RecognitionError> {
    use crate::geometry::PixelRect;
    let objective = supported_scene(frame, reference, check)?;
    for (slot, x) in CELL_X.into_iter().enumerate() {
        let place = Place::Cell(slot);
        checkpoint(check, place, Region::TopSlot)?;
        position::empty_top(frame, x).map_err(|cause| {
            failure(
                place,
                Region::TopSlot,
                Some(PixelRect::new(x, 112, CARD_WIDTH, 182)),
                FailureKind::TopSlotNotEmpty,
                cause,
            )
        })?;
    }
    for (slot, x) in FOUNDATION_X.into_iter().enumerate() {
        let place = Place::Foundation(slot);
        checkpoint(check, place, Region::TopSlot)?;
        position::empty_top(frame, x).map_err(|cause| {
            failure(
                place,
                Region::TopSlot,
                Some(PixelRect::new(x, 112, CARD_WIDTH, 182)),
                FailureKind::TopSlotNotEmpty,
                cause,
            )
        })?;
    }
    Ok(objective)
}


/// Recognise supplied pixels without a cancellation owner (offline evidence/tests).
#[cfg(test)]
pub(crate) fn recognise(
    frame: &CapturedFrame,
    metadata: Metadata,
) -> Result<InitialDeal, RecognitionError> {
    recognise_checked(frame, metadata, &mut || Ok(()))
}


/// Attribute an invalid bundled calibration or metadata contract without changing its cause.
fn calibration(cause: String) -> RecognitionError {
    failure(
        Place::Scene,
        Region::Calibration,
        None,
        FailureKind::Calibration,
        cause,
    )
}


/// Read exactly 52 fixed printed identities and the objective ROI from supplied native pixels.
///
/// No surrounding scene, slot, border, rail, below-stack or Solver evidence is inspected.
/// Crop paper uses local contrast and coherent ink; shape thresholds and inventory stay strict.
/// The callback never captures or supplies identities; no input, I/O or deck elimination occurs.
///
/// # Errors
/// Returns the first structured frame/crop, calibration, unreadable or ambiguous glyph,
/// ink-colour, unique-inventory or cancellation failure. Objective content is retained honestly.
pub(crate) fn recognise_checked(
    frame: &CapturedFrame,
    metadata: Metadata,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<InitialDeal, RecognitionError> {
    checkpoint(check, Place::Scene, Region::Frame)?;
    crate::game_operations::validate_native_frame(frame).map_err(|e| {
        failure(
            Place::Scene,
            Region::Frame,
            None,
            FailureKind::InvalidFrame,
            e.to_string(),
        )
    })?;
    let reference = decode_png(REFERENCE).map_err(|e| calibration(e.to_string()))?;
    checkpoint(check, Place::Scene, Region::Objective)?;
    let objective = objective::observe(frame, &reference)?;
    read_identities(frame, &reference, metadata, objective, check)
}


/// Bind an initial native board using fresh scene evidence and all52 independently read cards.
///
/// Positive game controls, Solver exclusion and empty top slots establish the initial scene.
/// Every fixed printed identity uses the same local contrast, coherent ink, shape competition
/// and unique inventory checks as read-only RECON. Card paper, decoration, borders, rails and
/// final-face artwork do not decide identity or add a separate gameplay veto. This inspects
/// only the supplied pixels and performs no I/O; the caller compares the frozen physical deal.
///
/// # Errors
/// Returns the original scene, slot, glyph, inventory, calibration or cancellation refusal.
/// Fresh context, input probes and subsequent effect proof remain the caller's responsibility.
pub(crate) fn recognise_playback_initial_checked(
    frame: &CapturedFrame,
    metadata: Metadata,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<InitialDeal, RecognitionError> {
    checkpoint(check, Place::Scene, Region::Frame)?;
    let reference = decode_png(REFERENCE).map_err(|e| calibration(e.to_string()))?;
    let objective = scene(frame, &reference, check)?;
    read_identities(frame, &reference, metadata, objective, check)
}


/// Independently read every fixed physical rank/suit crop and require an exact unique inventory.
fn read_identities(
    frame: &CapturedFrame,
    reference: &CapturedFrame,
    metadata: Metadata,
    objective: objective::Objective,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<InitialDeal, RecognitionError> {
    use crate::geometry::PixelRect;
    let metadata = objective::metadata(&metadata, objective).map_err(calibration)?;
    let mut bank = Vec::new();
    for (column, (&x, labels)) in PLAY_X.iter().zip(LABELS).enumerate() {
        for (depth, code) in labels.iter().enumerate().take(
            INITIAL_LENGTHS
                .get(column)
                .copied()
                .ok_or_else(|| calibration("Invalid calibration column".into()))?
                - 1,
        ) {
            checkpoint(check, Place::Scene, Region::Calibration)?;
            let y = *ROW_Y
                .get(depth)
                .ok_or_else(|| calibration("Invalid calibration depth".into()))?;
            bank.push(Template {
                card: label(code).map_err(calibration)?,
                rank: glyph(reference, x, y, true).map_err(calibration)?,
                suit: glyph(reference, x, y, false).map_err(calibration)?,
            });
        }
    }
    append_native_six_rank(&mut bank).map_err(calibration)?;
    let mut stacks: [CascadeStack; 8] = std::array::from_fn(|_| CascadeStack::default());
    for (column, (&x, &n)) in PLAY_X.iter().zip(INITIAL_LENGTHS.iter()).enumerate() {
        for depth in 0..n {
            let place = Place::Cascade { column, depth };
            let y = *ROW_Y
                .get(depth)
                .ok_or_else(|| calibration("Invalid physical depth".into()))?;
            let variants =
                |rank| glyph_variants_with_policy(frame, x, y, rank, PaperPolicy::ReadOnlyRecon);
            checkpoint(check, place, Region::Rank)?;
            let ranks = variants(true).map_err(|e| {
                failure(
                    place,
                    Region::Rank,
                    Some(PixelRect::new(x + 2, y + 7, 26, 23)),
                    FailureKind::UnknownIdentity,
                    e,
                )
            })?;
            let rank =
                identify(&ranks, &bank, |t| (t.card.rank(), &t.rank), 48, 20).map_err(|e| {
                    failure(
                        place,
                        Region::Rank,
                        Some(PixelRect::new(x + 2, y + 7, 26, 23)),
                        e.kind,
                        e.to_string(),
                    )
                })?;
            checkpoint(check, place, Region::Suit)?;
            let suits = variants(false).map_err(|e| {
                failure(
                    place,
                    Region::Suit,
                    Some(PixelRect::new(x + 5, y + 30, 18, 16)),
                    FailureKind::UnknownIdentity,
                    e,
                )
            })?;
            if ranks.first().map(|g| g.red) != suits.first().map(|g| g.red) {
                return Err(failure(
                    place,
                    Region::Suit,
                    Some(PixelRect::new(x + 5, y + 30, 18, 16)),
                    FailureKind::ColourDisagreement,
                    "Rank and suit print colours disagree",
                ));
            }
            let suit =
                identify(&suits, &bank, |t| (t.card.suit(), &t.suit), 16, 12).map_err(|e| {
                    failure(
                        place,
                        Region::Suit,
                        Some(PixelRect::new(x + 5, y + 30, 18, 16)),
                        e.kind,
                        e.to_string(),
                    )
                })?;
            crate::single_step_telemetry::record(
                || serde_json::json!({"event":"recognition-identity", "place":place.to_string(), "rank":rank.as_str(), "suit":suit.to_string(), "stage":"passed"}),
            );
            stacks
                .get_mut(column)
                .ok_or_else(|| calibration("Invalid cascade".into()))?
                .push(Card::new(rank, suit))
                .map_err(|e| {
                    failure(
                        place,
                        Region::Deck,
                        None,
                        FailureKind::InvalidDeck,
                        e.to_string(),
                    )
                })?;
        }
    }
    let board = BoardState::new(stacks, [None; 4], [0; 4]).map_err(|e| {
        failure(
            Place::Scene,
            Region::Deck,
            None,
            FailureKind::InvalidDeck,
            e.to_string(),
        )
    })?;
    InitialDeal::new(board, metadata).map_err(|e| {
        let (kind, place) = if let xsar::games::freecell::ModelError::Duplicate {
            second: xsar::games::freecell::Location::Cascade(location),
            ..
        } = &e
        {
            (
                FailureKind::DuplicateIdentity,
                Place::Cascade {
                    column: location.column(),
                    depth: location.depth(),
                },
            )
        } else {
            (FailureKind::InvalidDeck, Place::Scene)
        };
        failure(place, Region::Deck, None, kind, e.to_string())
    })
}
#[cfg(test)]
mod tests;
