//! Strict read-only recognition of measured native Free Cell initial deals.
//! Physical columns and depths are never sorted or inferred from the remaining deck.
use crate::{
    capture::{CapturedFrame, decode_png},
    freecell::{CARD_WIDTH, CELL_X, FOUNDATION_X, PLAY_X},
};
use xsar::games::{
    cards::{Card, Rank, Suit},
    freecell::{BoardState, CascadeStack, INITIAL_LENGTHS, InitialDeal, Metadata},
};

pub(crate) mod position;

/// Identity of this one measured, unscaled native profile.
pub(crate) const PROFILE: &str = "native-freecell-v1";

/// Typed first failure; no unreadable identity is inferred from inventory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FailureKind {
    /// Native storage, dimensions or crop are invalid.
    InvalidFrame,
    /// Header or excluded scene material does not match this profile.
    UnsupportedScene,
    /// A required top, bottom or side boundary is absent.
    InitialGeometry,
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
    /// Complete positive empty top slot.
    TopSlot,
    /// Required exposed-card top edge.
    TopEdge,
    /// Required complete final-face bottom edge.
    BottomEdge,
    /// Continuous outer final-face rails.
    SideEdges,
    /// Absence of extra cards below the expected stack.
    BelowStack,
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
/// Labelled same-print native template; only exposed strips train this bank.
struct Template {
    /// Manually labelled identity of the original native reference strip.
    card: Card,
    /// Rank evidence extracted from that same complete strip.
    rank: Glyph,
    /// Suit evidence extracted from that same complete strip.
    suit: Glyph,
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
    if p < 245 || red.max(black) < 70 || red.min(black) > 10 {
        return Err("Unsupported paper, missing or mixed-colour Free Cell glyph".into());
    }
    Ok(Glyph {
        ink: pixels
            .into_iter()
            .map(|v| 100 * u32::from(p.saturating_sub(v)) > 45 * u32::from(p))
            .collect(),
        red: red > black,
    })
}
/// Re-crop complete RGB rectangles at bounded native offsets; never pad ink masks.
fn glyph_variants(frame: &CapturedFrame, x: u32, y: u32, rank: bool) -> Result<Vec<Glyph>, String> {
    let centre = glyph(frame, x, y, rank)?;
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
            if let Ok(candidate) = glyph(frame, xx, yy, rank)
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
/// Require one complete native scene rectangle; alpha never supplies evidence.
fn same_region(
    frame: &CapturedFrame,
    reference: &CapturedFrame,
    rect: crate::geometry::PixelRect,
    place: Place,
    region: Region,
    kind: FailureKind,
) -> Result<(), RecognitionError> {
    let bad = |cause: String| failure(place, region, Some(rect), FailureKind::InvalidFrame, cause);
    let view = frame.as_view().map_err(|e| bad(e.to_string()))?;
    let expected = reference.as_view().map_err(|e| bad(e.to_string()))?;
    view.checked_bounds(rect).map_err(|e| bad(e.to_string()))?;
    expected
        .checked_bounds(rect)
        .map_err(|e| bad(e.to_string()))?;
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            if view.pixel_rgb(x, y) != expected.pixel_rgb(x, y) {
                return Err(failure(
                    place,
                    region,
                    Some(rect),
                    kind,
                    "Native scene signature differs",
                ));
            }
        }
    }
    Ok(())
}

/// Require stable header, eight positive empty slots and exclusion of extra rows.
fn scene(
    frame: &CapturedFrame,
    reference: &CapturedFrame,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<(), RecognitionError> {
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
    checkpoint(check, Place::Scene, Region::Header)?;
    same_region(
        frame,
        reference,
        PixelRect::new(825, 36, 270, 50),
        Place::Scene,
        Region::Header,
        FailureKind::UnsupportedScene,
    )?;
    for (slot, x) in CELL_X.into_iter().enumerate() {
        let place = Place::Cell(slot);
        checkpoint(check, place, Region::TopSlot)?;
        same_region(
            frame,
            reference,
            PixelRect::new(x, 112, CARD_WIDTH, 182),
            place,
            Region::TopSlot,
            FailureKind::TopSlotNotEmpty,
        )?;
    }
    for (slot, x) in FOUNDATION_X.into_iter().enumerate() {
        let place = Place::Foundation(slot);
        checkpoint(check, place, Region::TopSlot)?;
        same_region(
            frame,
            reference,
            PixelRect::new(x, 112, CARD_WIDTH, 182),
            place,
            Region::TopSlot,
            FailureKind::TopSlotNotEmpty,
        )?;
    }
    for (column, (x, n)) in PLAY_X.into_iter().zip(INITIAL_LENGTHS).enumerate() {
        let y = if n == 7 { 835 } else { 782 };
        let place = Place::Cascade {
            column,
            depth: n - 1,
        };
        checkpoint(check, place, Region::BelowStack)?;
        same_region(
            frame,
            reference,
            PixelRect::new(x, y, CARD_WIDTH, 935 - y),
            place,
            Region::BelowStack,
            FailureKind::InitialGeometry,
        )?;
    }
    Ok(())
}

/// Prove the final face's lower border and continuous outer rails without comparing artwork.
fn final_face(
    frame: &CapturedFrame,
    x: u32,
    y: u32,
    column: usize,
    depth: usize,
) -> Result<(), RecognitionError> {
    use crate::geometry::PixelRect;
    let place = Place::Cascade { column, depth };
    let bottom = if depth == 6 { 832 } else { 779 };
    let view = frame.as_view().map_err(|e| {
        failure(
            place,
            Region::SideEdges,
            None,
            FailureKind::InvalidFrame,
            e.to_string(),
        )
    })?;
    let rect = PixelRect::new(x + 8, bottom, 121, 1);
    view.checked_bounds(rect).map_err(|e| {
        failure(
            place,
            Region::BottomEdge,
            Some(rect),
            FailureKind::InvalidFrame,
            e.to_string(),
        )
    })?;
    for xx in x + 8..x + 129 {
        let [r, g, b] = view.pixel_rgb(xx, bottom).ok_or_else(|| {
            failure(
                place,
                Region::BottomEdge,
                Some(rect),
                FailureKind::InvalidFrame,
                "Missing bottom pixel",
            )
        })?;
        if r.min(g).min(b) < 100 || r.max(g).max(b) > 240 || r.max(g).max(b) - r.min(g).min(b) > 10
        {
            return Err(failure(
                place,
                Region::BottomEdge,
                Some(rect),
                FailureKind::InitialGeometry,
                "Complete native lower border is absent",
            ));
        }
    }
    let rails = PixelRect::new(x, y + 8, CARD_WIDTH, bottom - y - 15);
    view.checked_bounds(rails).map_err(|e| {
        failure(
            place,
            Region::SideEdges,
            Some(rails),
            FailureKind::InvalidFrame,
            e.to_string(),
        )
    })?;
    for yy in y + 8..=bottom - 8 {
        for xx in [x, x + CARD_WIDTH - 1] {
            let [r, g, b] = view.pixel_rgb(xx, yy).ok_or_else(|| {
                failure(
                    place,
                    Region::SideEdges,
                    Some(rails),
                    FailureKind::InvalidFrame,
                    "Missing outer rail",
                )
            })?;
            if r.min(g).min(b) < 100
                || r.max(g).max(b) > 240
                || r.max(g).max(b) - r.min(g).min(b) > 20
            {
                return Err(failure(
                    place,
                    Region::SideEdges,
                    Some(rails),
                    FailureKind::InitialGeometry,
                    "Complete native outer rail is absent",
                ));
            }
        }
    }
    Ok(())
}

/// Recognise supplied pixels without a cancellation owner (offline evidence/tests).
#[cfg(test)]
pub(crate) fn recognise(
    frame: &CapturedFrame,
    metadata: Metadata,
) -> Result<InitialDeal, RecognitionError> {
    recognise_checked(frame, metadata, &mut || Ok(()))
}

/// Recognise exactly52 printed identities with cancellation between bounded units.
/// The callback never captures or supplies identities; no input, I/O or deck elimination occurs.
pub(crate) fn recognise_checked(
    frame: &CapturedFrame,
    metadata: Metadata,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<InitialDeal, RecognitionError> {
    use crate::geometry::PixelRect;
    let calibration = |cause: String| {
        failure(
            Place::Scene,
            Region::Calibration,
            None,
            FailureKind::Calibration,
            cause,
        )
    };
    checkpoint(check, Place::Scene, Region::Frame)?;
    let reference = decode_png(REFERENCE).map_err(|e| calibration(e.to_string()))?;
    scene(frame, &reference, check)?;
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
                rank: glyph(&reference, x, y, true).map_err(calibration)?,
                suit: glyph(&reference, x, y, false).map_err(calibration)?,
            });
        }
    }
    let mut stacks: [CascadeStack; 8] = std::array::from_fn(|_| CascadeStack::default());
    for (column, (&x, &n)) in PLAY_X.iter().zip(INITIAL_LENGTHS.iter()).enumerate() {
        for depth in 0..n {
            let place = Place::Cascade { column, depth };
            checkpoint(check, place, Region::TopEdge)?;
            let y = *ROW_Y
                .get(depth)
                .ok_or_else(|| calibration("Invalid physical depth".into()))?;
            let rect = PixelRect::new(x + 8, y, 121, 1);
            let view = frame.as_view().map_err(|e| {
                failure(
                    place,
                    Region::Frame,
                    None,
                    FailureKind::InvalidFrame,
                    e.to_string(),
                )
            })?;
            for xx in x + 8..x + 129 {
                let [r, g, b] = view.pixel_rgb(xx, y).ok_or_else(|| {
                    failure(
                        place,
                        Region::TopEdge,
                        Some(rect),
                        FailureKind::InvalidFrame,
                        "Missing card border",
                    )
                })?;
                if r.min(g).min(b) < 180
                    || r.max(g).max(b) > 240
                    || r.max(g).max(b) - r.min(g).min(b) > 10
                {
                    return Err(failure(
                        place,
                        Region::TopEdge,
                        Some(rect),
                        FailureKind::InitialGeometry,
                        "Required native card top border is absent",
                    ));
                }
            }
            if depth + 1 == n {
                final_face(frame, x, y, column, depth)?;
            }
            checkpoint(check, place, Region::Rank)?;
            let ranks = glyph_variants(frame, x, y, true).map_err(|e| {
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
            let suits = glyph_variants(frame, x, y, false).map_err(|e| {
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
