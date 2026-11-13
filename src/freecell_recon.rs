//! Fixed-position Free Cell identity extraction and live initial-board binding.
//! Physical columns and depths are never sorted or inferred from the remaining deck.
use crate::{
    capture::{CapturedFrame, decode_png},
    freecell::{CARD_WIDTH, CELL_X, FOUNDATION_X, PLAY_X},
    game_controls::FREECELL_ROW_Y as ROW_Y,
    pyramid_recon::vision::CornerClassification,
};
#[cfg(test)]
use xsar::games::cards::{Card, Rank, Suit};
use xsar::games::freecell::{BoardState, CascadeStack, INITIAL_LENGTHS, InitialDeal, Metadata};

pub(crate) mod objective;
pub(crate) mod position;

/// Identity of this one measured, unscaled native profile.
pub const PROFILE: &str = "native-freecell-v1";

/// Typed first failure; no unreadable identity is inferred from inventory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureKind {
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
pub enum Place {
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
pub enum Region {
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
pub struct RecognitionError {
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

/// Independently labelled native calibration appearance; no capture is performed here.
const REFERENCE: &[u8] = include_bytes!("../tests/fixtures/freecell-FC13.png");


/// Complete Menu and Back glyph bounds from the retained native game controls.
/// Their printed shape supplies scene evidence; arbitrary felt, objective and score pixels do not.
const NATIVE_CONTROL_REGIONS: [crate::geometry::PixelRect; 2] = [
    crate::geometry::PixelRect::new(20, 46, 24, 22),
    crate::geometry::PixelRect::new(90, 46, 27, 25),
];


/// Manually read calibration labels for FC13 in physical column/depth order (tests only).
#[cfg(test)]
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


/// Parse test expectation card shorthand (`"6D"`, `"TC"`, etc.); never used in production recognition.
#[cfg(test)]
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


/// Require one initial upper `CELL` or `SUIT` slot to read positively `Empty` via `game_controls`.
fn check_initial_empty_top(
    frame: &CapturedFrame,
    request: crate::game_controls::LocationRequest,
    place: Place,
    x: u32,
) -> Result<(), RecognitionError> {
    use crate::game_operations::CardRead;
    let rect = Some(crate::geometry::PixelRect::new(x, 112, CARD_WIDTH, 182));
    match crate::game_controls::read_card(frame, request) {
        Ok(CardRead::Empty) => Ok(()),
        Ok(_) => {
            let cause = position::empty_top(frame, x).err().unwrap_or_else(|| {
                "Top slot contains printed card or ambiguous evidence".to_owned()
            });
            Err(failure(
                place,
                Region::TopSlot,
                rect,
                FailureKind::TopSlotNotEmpty,
                cause,
            ))
        }
        Err(error) => Err(failure(
            place,
            Region::TopSlot,
            rect,
            FailureKind::InvalidFrame,
            error.to_string(),
        )),
    }
}


/// Require native game controls, Solver exclusion and all eight initially empty top slots.
/// Printed tableau identities are read separately without decorative card geometry checks.
fn scene(
    frame: &CapturedFrame,
    reference: &CapturedFrame,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<objective::Objective, RecognitionError> {
    let objective = supported_scene(frame, reference, check)?;
    for (slot, x) in CELL_X.into_iter().enumerate() {
        let place = Place::Cell(slot);
        checkpoint(check, place, Region::TopSlot)?;
        let slot_1 = u8::try_from(slot + 1).map_err(|_| calibration("Invalid cell slot".into()))?;
        check_initial_empty_top(
            frame,
            crate::game_controls::LocationRequest::freecell_cell(slot_1),
            place,
            x,
        )?;
    }
    for (slot, x) in FOUNDATION_X.into_iter().enumerate() {
        let place = Place::Foundation(slot);
        checkpoint(check, place, Region::TopSlot)?;
        let slot_1 =
            u8::try_from(slot + 1).map_err(|_| calibration("Invalid foundation slot".into()))?;
        check_initial_empty_top(
            frame,
            crate::game_controls::LocationRequest::freecell_suit(slot_1),
            place,
            x,
        )?;
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
/// Verify positive native Free Cell controls and Solver exclusion on a live frame without I/O.
pub fn verify_gameplay_scene(
    frame: &CapturedFrame,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<(), String> {
    let reference = decode_png(REFERENCE).map_err(|e| e.to_string())?;
    supported_scene(frame, &reference, check)
        .map(|_| ())
        .map_err(|e| e.to_string())
}


/// Read the objective ROI and all 52 fixed printed identities via `game_controls` (checked 52-card deal).
///
/// No surrounding scene, slot, border, rail, below-stack or Solver evidence is inspected.
/// Foreground classification uses the shared native corner reader; shape thresholds and unique
/// inventory stay strict. The callback never captures or supplies identities.
///
/// # Errors
/// Returns the first structured frame/crop, calibration, unreadable or ambiguous glyph,
/// ink-colour, unique-inventory or cancellation failure. Objective content is retained honestly.
pub fn recognise_checked(
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
    read_identities(frame, metadata, objective, check)
}


/// Bind an initial native board using fresh scene evidence and all 52 independently read cards.
///
/// Positive game controls, Solver exclusion and empty top slots establish the initial scene.
/// Every fixed printed identity uses the shared foreground corner reader (`game_controls`) with
/// shape competition and unique inventory checks. Card paper, decoration, borders, rails and
/// final-face artwork do not decide identity or add a separate gameplay veto. This inspects
/// only the supplied pixels and performs no I/O; the caller compares the frozen physical deal.
///
/// # Errors
/// Returns the original scene, slot, glyph, inventory, calibration or cancellation refusal.
/// Fresh context, input probes and subsequent effect proof remain the caller's responsibility.
pub fn recognise_playback_initial_checked(
    frame: &CapturedFrame,
    metadata: Metadata,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<InitialDeal, RecognitionError> {
    checkpoint(check, Place::Scene, Region::Frame)?;
    let reference = decode_png(REFERENCE).map_err(|e| calibration(e.to_string()))?;
    let objective = scene(frame, &reference, check)?;
    read_identities(frame, metadata, objective, check)
}


/// Independently read every fixed physical rank/suit crop via `game_controls` and require a unique 52-card deal.
fn read_identities(
    frame: &CapturedFrame,
    metadata: Metadata,
    objective: objective::Objective,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<InitialDeal, RecognitionError> {
    use crate::geometry::PixelRect;
    let metadata = objective::metadata(&metadata, objective).map_err(calibration)?;
    let mut stacks: [CascadeStack; 8] = std::array::from_fn(|_| CascadeStack::default());
    for (column, (&x, &n)) in PLAY_X.iter().zip(INITIAL_LENGTHS.iter()).enumerate() {
        for depth in 0..n {
            let place = Place::Cascade { column, depth };
            let y = *ROW_Y
                .get(depth)
                .ok_or_else(|| calibration("Invalid physical depth".into()))?;
            checkpoint(check, place, Region::Rank)?;
            let map_op_err = |e: crate::game_operations::OperationError| {
                let kind = match e {
                    crate::game_operations::OperationError::Invalid(_) => FailureKind::InvalidFrame,
                    _ => FailureKind::Calibration,
                };
                failure(
                    place,
                    Region::Rank,
                    Some(PixelRect::new(x + 3, y + 4, 31, 49)),
                    kind,
                    e.to_string(),
                )
            };
            let card =
                match crate::game_controls::classify_corner(frame, (x, y)).map_err(map_op_err)? {
                    CornerClassification::Card(card) => {
                        checkpoint(check, place, Region::Suit)?;
                        card
                    }
                    CornerClassification::Ambiguous { detail } => {
                        return Err(failure(
                            place,
                            Region::Rank,
                            Some(PixelRect::new(x + 2, y + 7, 26, 23)),
                            FailureKind::AmbiguousIdentity,
                            detail,
                        ));
                    }
                    CornerClassification::ColourDisagreement { detail } => {
                        checkpoint(check, place, Region::Suit)?;
                        return Err(failure(
                            place,
                            Region::Suit,
                            Some(PixelRect::new(x + 5, y + 30, 18, 16)),
                            FailureKind::ColourDisagreement,
                            detail,
                        ));
                    }
                    CornerClassification::PartialLabel {
                        rank: Some(_),
                        suit: None,
                        detail,
                    } => {
                        checkpoint(check, place, Region::Suit)?;
                        return Err(failure(
                            place,
                            Region::Suit,
                            Some(PixelRect::new(x + 5, y + 30, 18, 16)),
                            FailureKind::UnknownIdentity,
                            detail,
                        ));
                    }
                    CornerClassification::PartialLabel { detail, .. }
                    | CornerClassification::Unreadable { detail } => {
                        return Err(failure(
                            place,
                            Region::Rank,
                            Some(PixelRect::new(x + 2, y + 7, 26, 23)),
                            FailureKind::UnknownIdentity,
                            detail,
                        ));
                    }
                };
            crate::single_step_telemetry::record(
                || serde_json::json!({"event":"recognition-identity", "place":place.to_string(), "rank":card.rank().as_str(), "suit":card.suit().to_string(), "stage":"passed"}),
            );
            stacks
                .get_mut(column)
                .ok_or_else(|| calibration("Invalid cascade".into()))?
                .push(card)
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
