//! Shared game-control observers and foreground card recognition across Solitaire boards.
//!
//! All helpers operate on `1920×1080` native frames (and injected `OperationIo` when capturing
//! or waiting), never mutating guest state or issuing Continue/cleanup input.

use std::time::Duration;

use xsar::games::{
    cards::{Card, Rank, Suit},
    freecell::rules::card_id,
};

use crate::{
    capture::CapturedFrame,
    cards::{TABLEAU_ROW_LENGTHS, TABLEAU_ROW_STARTS},
    detector::pixel_rgb,
    freecell::{CELL_X, FOUNDATION_X, PLAY_X},
    game::{GameMode, is_classic_blue_back},
    game_operations::{CardRead, OperationError, OperationIo, validate_native_frame},
    geometry::PixelRect,
    parameters::{
        FACE_UP_WHITE_CHANNEL_MINIMUM, GAMEPLAY_FELT_GREEN_BLUE_DELTA_MINIMUM,
        GAMEPLAY_FELT_GREEN_MINIMUM, GAMEPLAY_FELT_GREEN_RED_DELTA_MINIMUM,
        GAMEPLAY_FELT_REQUIRED_FRACTION_PER_MILLE, STOCK_CARD_BOUNDS, TABLEAU_CARD_REGIONS,
        TARGET_STOCK, WASTE_CARD_BOUNDS,
    },
    pyramid_recon::vision::{
        CornerClassification, classify_foreground_corner, pyramid_tableau_origin,
        read_pyramid_draw_card,
    },
};

/// Native vertical origins for the initial seven rows of a Free Cell tableau cascade.
pub const FREECELL_ROW_Y: [u32; 7] = [331, 384, 437, 490, 543, 597, 650];

/// Maximum 1-based depth supported on a Free Cell `PLAY` cascade (`1..=19`).
pub const FREECELL_MAX_PLAY_ROW: u8 = 19;

/// Native vertical origin of Free Cell's upper `CELL` and `SUIT` card slots.
pub const FREECELL_UPPER_TOP: u32 = 112;

/// First native tableau top, independent of subsequent compressed fan spacing.
const FREECELL_TABLEAU_TOP: u32 = 331;

/// Latest possible printed-corner origin whose complete crop stays above toolbar947.
const FREECELL_LAST_CORNER_TOP: u32 = 898;

/// Maximum separation of neighbouring native ±1 recrops of one printed corner.
const FREECELL_RECROP_SEPARATION: u32 = 2;

/// Zero-tolerance pure white (`[255, 255, 255]`) lower stems of `ll` inside **Challenge** (`225` pixels).
pub const CHALLENGE_LL_RECTS: [PixelRect; 2] = [
    PixelRect::new(766, 204, 8, 15),
    PixelRect::new(785, 204, 7, 15),
];

/// Zero-tolerance pure black (`[0, 0, 0]`) lettering segments of **Continue** (`149` unique pixels).
pub const CONTINUE_BLACK_RECTS: [PixelRect; 8] = [
    PixelRect::new(952, 874, 9, 1),
    PixelRect::new(955, 870, 1, 19),
    PixelRect::new(966, 866, 3, 1),
    PixelRect::new(967, 874, 1, 15),
    PixelRect::new(976, 874, 2, 15),
    PixelRect::new(988, 875, 1, 15),
    PixelRect::new(997, 874, 2, 14),
    PixelRect::new(1009, 874, 2, 16),
];

/// Maximum number of fresh-frame `challenge_complete` probes in [`wait_for_challenge_complete`].
pub const CHALLENGE_COMPLETE_MAX_ATTEMPTS: usize = 5;

/// Cancellable pause between consecutive `challenge_complete` probes.
pub const CHALLENGE_COMPLETE_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// One-based board/pile location categories across supported games.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocationKind {
    /// Tableau/cascade position (`PLAY`).
    Play,
    /// Free Cell upper temporary cell (`CELL 1..=4`).
    Cell,
    /// Free Cell upper foundation slot (`SUIT 1..=4`, left-to-right physical order).
    Suit,
    /// Stock or waste/draw pile (`DRAW 1` = left/stock, `DRAW 2` = right/waste).
    Draw,
}

/// Checked 1-based physical card location request for [`read_card`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationRequest {
    /// Selected Solitaire game mode.
    pub game: GameMode,
    /// Location category within that game.
    pub kind: LocationKind,
    /// One-based vertical row (`1..=19` for Free Cell `PLAY`, `1..=7` for Pyramid `PLAY`, `1..=4` for TriPeaks `PLAY`, `1` for `CELL`/`SUIT`/`DRAW`).
    pub row: u8,
    /// One-based horizontal column or slot (`1..=8` for Free Cell `PLAY`, `1..=4` for `CELL`/`SUIT`, `1..=row` for Pyramid `PLAY`, `1..=row_length` for TriPeaks `PLAY`, `1..=2` for `DRAW`).
    pub column: u8,
}

impl LocationRequest {
    /// Build a 1-based Free Cell `PLAY` location request (`column` `1..=8`, `row` `1..=19`).
    pub const fn freecell_play(column: u8, row: u8) -> Self {
        Self {
            game: GameMode::FreeCell,
            kind: LocationKind::Play,
            row,
            column,
        }
    }

    /// Build a 1-based Free Cell `CELL` location request (`slot` `1..=4`, `row == 1`).
    pub const fn freecell_cell(slot: u8) -> Self {
        Self {
            game: GameMode::FreeCell,
            kind: LocationKind::Cell,
            row: 1,
            column: slot,
        }
    }

    /// Build a 1-based Free Cell `SUIT` location request (`slot` `1..=4`, physical left-to-right, `row == 1`).
    pub const fn freecell_suit(slot: u8) -> Self {
        Self {
            game: GameMode::FreeCell,
            kind: LocationKind::Suit,
            row: 1,
            column: slot,
        }
    }

    /// Build a 1-based Pyramid `PLAY` location request (`1 <= column <= row <= 7`).
    pub const fn pyramid_play(row: u8, column: u8) -> Self {
        Self {
            game: GameMode::Pyramid,
            kind: LocationKind::Play,
            row,
            column,
        }
    }

    /// Build a 1-based Pyramid `DRAW` location request (`1` = Left, `2` = Right, `row == 1`).
    pub const fn pyramid_draw(slot: u8) -> Self {
        Self {
            game: GameMode::Pyramid,
            kind: LocationKind::Draw,
            row: 1,
            column: slot,
        }
    }

    /// Build a 1-based TriPeaks `PLAY` location request (`row` `1..=4`, `column` `1..=row_length`).
    pub const fn tripeaks_play(row: u8, column: u8) -> Self {
        Self {
            game: GameMode::TriPeaks,
            kind: LocationKind::Play,
            row,
            column,
        }
    }

    /// Build a 1-based TriPeaks `DRAW` location request (`1` = Stock, `2` = Waste, `row == 1`).
    pub const fn tripeaks_draw(slot: u8) -> Self {
        Self {
            game: GameMode::TriPeaks,
            kind: LocationKind::Draw,
            row: 1,
            column: slot,
        }
    }
}

/// Nearby recrops of one physical Free Cell cascade corner, including unreadable candidate evidence.
#[derive(Debug)]
pub(crate) struct PlayCorner {
    /// First candidate origin in this connected `±1` recrop cluster.
    pub(crate) first: u32,
    /// Last candidate origin in this cluster.
    pub(crate) last: u32,
    /// Last origin in this cluster that positively matched a printed card corner.
    pub(crate) matched_last: Option<u32>,
    /// Independently observed canonical card ID (`0..52`), if any crop passed matching.
    pub(crate) id: Option<u8>,
    /// Last failed candidate diagnostic, retained if no crop supplies an identity.
    pub(crate) failure: Option<String>,
}

impl PlayCorner {
    /// Merge one neighbouring crop, refusing contradictory identities in one corner.
    fn add(&mut self, y: u32, observed: Result<u8, String>) -> Result<(), String> {
        self.last = y;
        match observed {
            Ok(id) => {
                if self.id.is_some_and(|previous| previous != id) {
                    return Err("Ambiguous identities within one printed corner".to_owned());
                }
                self.id = Some(id);
                self.matched_last = Some(y);
            }
            Err(cause) => self.failure = Some(cause),
        }
        Ok(())
    }

    /// Locate the calibrated card top without drifting up into the foreground crop margin.
    pub(crate) fn top(&self) -> u32 {
        self.matched_last
            .unwrap_or_else(|| self.first + (self.last - self.first) / 2)
    }
}

/// Classify one `1920×1080` top-left card corner under the shared foreground reader.
pub(crate) fn classify_corner(
    frame: &CapturedFrame,
    origin: (u32, u32),
) -> Result<CornerClassification, OperationError> {
    validate_native_frame(frame)?;
    classify_foreground_corner(frame, origin).map_err(OperationError::Recognition)
}

/// Classify a Free Cell upper `CELL` or `SUIT` card corner across the `[x, x - 1]` native origins.
pub(crate) fn classify_freecell_upper_corner(
    frame: &CapturedFrame,
    x: u32,
) -> Result<CornerClassification, OperationError> {
    validate_native_frame(frame)?;
    let left_x = x
        .checked_sub(1)
        .ok_or_else(|| OperationError::Invalid("Free Cell upper slot x underflow".to_owned()))?;
    let primary = classify_foreground_corner(frame, (x, FREECELL_UPPER_TOP))
        .map_err(OperationError::Recognition)?;
    let shifted = classify_foreground_corner(frame, (left_x, FREECELL_UPPER_TOP))
        .map_err(OperationError::Recognition)?;

    Ok(match (primary, shifted) {
        (CornerClassification::Card(first), CornerClassification::Card(second)) => {
            if first == second {
                CornerClassification::Card(first)
            } else {
                CornerClassification::Ambiguous {
                    detail: format!(
                        "upper slot `[x, x-1]` disagreed on card identity: {first} vs {second}"
                    ),
                }
            }
        }
        (a @ CornerClassification::Ambiguous { .. }, _)
        | (_, a @ CornerClassification::Ambiguous { .. }) => a,
        (CornerClassification::Card(card), _) | (_, CornerClassification::Card(card)) => {
            CornerClassification::Card(card)
        }
        (c @ CornerClassification::ColourDisagreement { .. }, _)
        | (_, c @ CornerClassification::ColourDisagreement { .. }) => c,
        (p @ CornerClassification::PartialLabel { .. }, _)
        | (_, p @ CornerClassification::PartialLabel { .. }) => p,
        (unreadable, _) => unreadable,
    })
}

/// Prove a Free Cell upper `CELL` or `SUIT` slot empty via green outline coverage and absence of white card paper.
pub(crate) fn freecell_empty_top(frame: &CapturedFrame, x: u32) -> Result<bool, OperationError> {
    Ok(freecell_empty_slot_error(frame, x, FREECELL_UPPER_TOP)?.is_ok())
}

/// Check empty-slot absence of interior card paper (`y`) and four-edge green stroke (`>= 995‰`).
pub(crate) fn freecell_empty_slot_error(
    frame: &CapturedFrame,
    x: u32,
    y: u32,
) -> Result<Result<(), String>, OperationError> {
    validate_native_frame(frame)?;
    let view = frame
        .as_view()
        .map_err(|error| OperationError::Invalid(error.to_string()))?;
    let interior_top = if y == FREECELL_UPPER_TOP { 120 } else { 340 };
    let interior = PixelRect::new(x + 8, interior_top, 120, 160);
    view.checked_bounds(interior)
        .map_err(|error| OperationError::Invalid(format!("{error:?}")))?;
    for yy in interior.y..interior.y + interior.height {
        for xx in interior.x..interior.x + interior.width {
            let [r, g, b] = view.pixel_rgb(xx, yy).ok_or_else(|| {
                OperationError::Invalid("Empty interior pixel out of bounds".to_owned())
            })?;
            if r.min(g).min(b) >= 245 {
                return Ok(Err("Card paper interrupts empty-slot interior".to_owned()));
            }
        }
    }
    let green = |[r, g, b]: [u8; 3]| {
        r >= 45
            && g >= 140
            && b >= 100
            && i16::from(g) - i16::from(r) >= 50
            && i16::from(g) - i16::from(b) >= 20
    };
    for rect in [
        PixelRect::new(x + 20, y + 1, 96, 3),
        PixelRect::new(x + 20, y + 176, 96, 3),
        PixelRect::new(x + 1, y + 19, 3, 140),
        PixelRect::new(x + 130, y + 19, 3, 140),
    ] {
        if !coverage_per_mille(&view, rect, green, 995)? {
            return Ok(Err(
                "Positive native empty-slot outline unavailable".to_owned()
            ));
        }
    }
    Ok(Ok(()))
}

/// Threshold one complete glyph region against its own 90th-percentile background.
fn local_ink<const N: usize>(frame: &CapturedFrame, rect: PixelRect) -> Result<[bool; N], String> {
    let count = usize::try_from(rect.width)
        .ok()
        .and_then(|width| {
            usize::try_from(rect.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or("Glyph support area overflow")?;
    if count != N {
        return Err("Glyph support dimensions differ from native crop".to_owned());
    }
    let view = frame.as_view().map_err(|cause| cause.to_string())?;
    view.checked_bounds(rect)
        .map_err(|cause| cause.to_string())?;
    let mut samples = [0_u8; N];
    for (sample, (x, y)) in samples.iter_mut().zip(
        (rect.y..rect.y + rect.height)
            .flat_map(|y| (rect.x..rect.x + rect.width).map(move |x| (x, y))),
    ) {
        let [r, g, b] = view
            .pixel_rgb(x, y)
            .ok_or("Missing local glyph support pixel")?;
        *sample = r.min(g).min(b);
    }
    let mut background = samples;
    background.sort_unstable();
    let percentile = N.checked_mul(9).ok_or("Glyph percentile overflow")? / 10;
    let paper = u32::from(
        *background
            .get(percentile)
            .ok_or("Missing local glyph background")?,
    );
    Ok(samples.map(|value| 100 * paper.saturating_sub(u32::from(value)) > 45 * paper))
}

/// Locate printed-corner support cheaply before allocating full foreground recrops.
pub(crate) fn corner_candidate(frame: &CapturedFrame, x: u32, y: u32) -> Result<bool, String> {
    let view = frame.as_view().map_err(|e| e.to_string())?;
    view.checked_bounds(PixelRect::new(x + 2, y + 3, 26, 43))
        .map_err(|e| e.to_string())?;
    let minimum = |xx, yy| -> Result<u8, String> {
        let [r, g, b] = view.pixel_rgb(xx, yy).ok_or("Missing corner pixel")?;
        Ok(r.min(g).min(b))
    };
    let mut background = [0_u8; 80];
    for (slot, (xx, yy)) in background
        .iter_mut()
        .zip((y + 3..y + 7).flat_map(|yy| (x + 5..x + 25).map(move |xx| (xx, yy))))
    {
        *slot = minimum(xx, yy)?;
    }
    background.sort_unstable();
    let paper = u32::from(*background.get(72).ok_or("Missing corner background")?);
    let ink = |xx, yy| -> Result<bool, String> {
        Ok(100 * paper.saturating_sub(u32::from(minimum(xx, yy)?)) > 45 * paper)
    };
    let mut gutter_ink = 0;
    for yy in y + 3..y + 7 {
        for xx in x + 5..x + 25 {
            gutter_ink += usize::from(ink(xx, yy)?);
        }
    }
    if gutter_ink > 8 {
        return Ok(false);
    }
    let ranks = local_ink::<{ 26 * 23 }>(frame, PixelRect::new(x + 2, y + 7, 26, 23))?;
    let rank_ink = ranks.iter().filter(|ink| **ink).count();
    let rank_top = ranks.iter().take(26 * 5).filter(|ink| **ink).count();
    let rank_bottom = ranks
        .iter()
        .skip(26 * 17)
        .take(26 * 3)
        .filter(|ink| **ink)
        .count();
    let suits = local_ink::<{ 18 * 16 }>(frame, PixelRect::new(x + 5, y + 30, 18, 16))?;
    let suit_ink = suits.iter().filter(|ink| **ink).count();
    let rank = rank_ink >= 30 && rank_top >= 2 && rank_bottom >= 2;
    Ok(rank || suit_ink >= 20)
}

/// Retain a partial corner or conflicting/ambiguous label when printed strokes establish it.
pub(crate) fn corner_identity(frame: &CapturedFrame, x: u32, y: u32) -> Option<Result<u8, String>> {
    let classified = match classify_corner(frame, (x, y)) {
        Ok(classified) => classified,
        Err(error) => return Some(Err(error.to_string())),
    };
    match classified {
        CornerClassification::Card(card) => Some(Ok(card_id(card))),
        CornerClassification::Ambiguous { detail } => {
            Some(Err(format!("AmbiguousIdentity: {detail}")))
        }
        CornerClassification::ColourDisagreement { detail } => Some(Err(format!(
            "Visible rank/suit colour disagreement: {detail}"
        ))),
        CornerClassification::PartialLabel { rank, suit, detail } => {
            Some(Err(match (rank, suit) {
                (None, Some(_)) => format!("Unreadable printed rank: {detail}"),
                (Some(_), None) => format!("Unreadable printed suit: {detail}"),
                _ => format!("Unreadable printed labels: {detail}"),
            }))
        }
        CornerClassification::Unreadable { .. } => None,
    }
}

/// Scan bounded native origins top-to-bottom along one Free Cell `PLAY` cascade column.
pub(crate) fn scan_freecell_play_corners(
    frame: &CapturedFrame,
    x: u32,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<Vec<PlayCorner>, String> {
    let mut result: Vec<PlayCorner> = Vec::new();
    for y in FREECELL_TABLEAU_TOP..=FREECELL_LAST_CORNER_TOP {
        if (y - FREECELL_TABLEAU_TOP).is_multiple_of(8) {
            check()?;
        }
        if !corner_candidate(frame, x, y)? {
            continue;
        }
        let Some(observed) = corner_identity(frame, x, y) else {
            continue;
        };
        if let Some(previous) = result.last_mut()
            && y - previous.last <= FREECELL_RECROP_SEPARATION
        {
            previous.add(y, observed)?;
        } else {
            let mut corner = PlayCorner {
                first: y,
                last: y,
                matched_last: None,
                id: None,
                failure: None,
            };
            corner.add(y, observed)?;
            result.push(corner);
        }
    }
    check()?;
    Ok(result)
}

/// Convert a canonical Free Cell card ordinal (`0..52`) back into [`Card`].
fn card_from_id(id: u8) -> Result<Card, OperationError> {
    let rank = Rank::ALL
        .get(usize::from(id % 13))
        .copied()
        .ok_or_else(|| OperationError::Invalid(format!("Invalid Free Cell card rank id {id}")))?;
    let suit = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades]
        .get(usize::from(id / 13))
        .copied()
        .ok_or_else(|| OperationError::Invalid(format!("Invalid Free Cell card suit id {id}")))?;
    Ok(Card::new(rank, suit))
}

/// Check whether white card paper extends below a bottommost card face in column `x`.
fn below_cascade_has_paper(
    frame: &CapturedFrame,
    x: u32,
    last_top: u32,
) -> Result<bool, OperationError> {
    let start_y = last_top.saturating_add(185);
    let end_y = last_top.saturating_add(225).min(940);
    if start_y >= end_y {
        return Ok(false);
    }
    let view = frame
        .as_view()
        .map_err(|error| OperationError::Invalid(error.to_string()))?;
    let strip = PixelRect::new(x + 12, start_y, 108, end_y - start_y);
    view.checked_bounds(strip)
        .map_err(|error| OperationError::Invalid(format!("{error:?}")))?;
    for yy in strip.y..strip.y + strip.height {
        for xx in strip.x..strip.x + strip.width {
            let [r, g, b] = view.pixel_rgb(xx, yy).ok_or_else(|| {
                OperationError::Invalid("Cascade lower paper pixel outside bounds".to_owned())
            })?;
            if r.min(g).min(b) >= 245 {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Combine a slot corner classification with its positive empty-slot state.
fn resolve_slot_read(classified: CornerClassification, empty: bool) -> CardRead {
    match (classified, empty) {
        (CornerClassification::Card(card), false) => CardRead::Card(card),
        (CornerClassification::Ambiguous { .. }, _) | (CornerClassification::Card(_), true) => {
            CardRead::Ambiguous
        }
        (_, true) => CardRead::Empty,
        (
            CornerClassification::ColourDisagreement { .. }
            | CornerClassification::PartialLabel { .. }
            | CornerClassification::Unreadable { .. },
            false,
        ) => CardRead::Unreadable,
    }
}

/// Read one physical card location on a borrowed `1920×1080` frame without scene gates or I/O.
pub fn read_card(
    frame: &CapturedFrame,
    request: LocationRequest,
) -> Result<CardRead, OperationError> {
    validate_native_frame(frame)?;
    match (request.game, request.kind) {
        (GameMode::FreeCell, LocationKind::Play) => {
            let column = usize::from(request.column.checked_sub(1).ok_or_else(|| {
                OperationError::Invalid("Free Cell PLAY column must be 1..=8".to_owned())
            })?);
            let x = PLAY_X.get(column).copied().ok_or_else(|| {
                OperationError::Invalid(format!(
                    "Invalid 1-based Free Cell PLAY column {}",
                    request.column
                ))
            })?;
            if !(1..=FREECELL_MAX_PLAY_ROW).contains(&request.row) {
                return Err(OperationError::Invalid(format!(
                    "Invalid 1-based Free Cell PLAY row {} (expected 1..={FREECELL_MAX_PLAY_ROW})",
                    request.row
                )));
            }
            if request.row == 1 {
                let classified = classify_corner(frame, (x, FREECELL_TABLEAU_TOP))?;
                let empty = freecell_empty_slot_error(frame, x, FREECELL_TABLEAU_TOP)?.is_ok()
                    && scan_freecell_play_corners(frame, x, &mut || Ok(()))
                        .map_err(OperationError::Recognition)?
                        .is_empty();
                return Ok(resolve_slot_read(classified, empty));
            }
            let corners = scan_freecell_play_corners(frame, x, &mut || Ok(()))
                .map_err(OperationError::Recognition)?;
            if corners.is_empty() {
                return Ok(
                    if freecell_empty_slot_error(frame, x, FREECELL_TABLEAU_TOP)?.is_ok() {
                        CardRead::Empty
                    } else {
                        CardRead::Unreadable
                    },
                );
            }
            if corners[0].first > FREECELL_TABLEAU_TOP + 4 {
                return Ok(CardRead::Unreadable);
            }
            let target_idx = usize::from(request.row - 1);
            for idx in 1..corners.len().min(target_idx + 1) {
                if corners[idx].top().saturating_sub(corners[idx - 1].top()) > 58 {
                    return Ok(CardRead::Unreadable);
                }
            }
            if let Some(target) = corners.get(target_idx) {
                return if let Some(id) = target.id {
                    Ok(CardRead::Card(card_from_id(id)?))
                } else if target
                    .failure
                    .as_deref()
                    .is_some_and(|detail| detail.starts_with("Ambiguous"))
                {
                    Ok(CardRead::Ambiguous)
                } else {
                    Ok(CardRead::Unreadable)
                };
            }
            let last_top = corners.last().map_or(FREECELL_TABLEAU_TOP, PlayCorner::top);
            if below_cascade_has_paper(frame, x, last_top)? {
                Ok(CardRead::Unreadable)
            } else {
                Ok(CardRead::Empty)
            }
        }
        (GameMode::FreeCell, LocationKind::Cell | LocationKind::Suit) => {
            if request.row != 1 {
                return Err(OperationError::Invalid(format!(
                    "Free Cell {:?} request requires row 1 (got {})",
                    request.kind, request.row
                )));
            }
            let slot = usize::from(request.column.checked_sub(1).ok_or_else(|| {
                OperationError::Invalid("Free Cell upper slot must be 1..=4".to_owned())
            })?);
            let coords = match request.kind {
                LocationKind::Cell => CELL_X,
                LocationKind::Suit => FOUNDATION_X,
                _ => unreachable!(),
            };
            let x = coords.get(slot).copied().ok_or_else(|| {
                OperationError::Invalid(format!(
                    "Invalid 1-based Free Cell upper slot {}",
                    request.column
                ))
            })?;
            let classified = classify_freecell_upper_corner(frame, x)?;
            let empty = freecell_empty_top(frame, x)?;
            Ok(resolve_slot_read(classified, empty))
        }
        (GameMode::Pyramid, LocationKind::Play) => {
            let origin = pyramid_tableau_origin(request.row, request.column).ok_or_else(|| {
                OperationError::Invalid(format!(
                    "Invalid 1-based Pyramid PLAY coordinate {}|{}",
                    request.row, request.column
                ))
            })?;
            Ok(read_from_classification(classify_corner(frame, origin)?))
        }
        (GameMode::Pyramid, LocationKind::Draw) => {
            if request.row != 1 {
                return Err(OperationError::Invalid(format!(
                    "Pyramid DRAW request requires row 1 (got {})",
                    request.row
                )));
            }
            let right = match request.column {
                1 => false,
                2 => true,
                slot => {
                    return Err(OperationError::Invalid(format!(
                        "Invalid 1-based Pyramid DRAW slot {slot} (expected 1 or 2)"
                    )));
                }
            };
            read_pyramid_draw_card(frame, right).map_err(OperationError::Recognition)
        }
        (GameMode::TriPeaks, LocationKind::Play) => {
            read_tripeaks_play_card(frame, request.row, request.column)
        }
        (GameMode::TriPeaks, LocationKind::Draw) => {
            if request.row != 1 {
                return Err(OperationError::Invalid(format!(
                    "TriPeaks DRAW requires row 1, got row {}",
                    request.row
                )));
            }
            read_tripeaks_draw_card(frame, request.column)
        }
        (GameMode::FreeCell, LocationKind::Draw)
        | (GameMode::Pyramid | GameMode::TriPeaks, LocationKind::Cell | LocationKind::Suit)
        | (GameMode::Klondike | GameMode::Spider, _) => Err(OperationError::Unsupported(format!(
            "{:?} does not support {:?} in game_controls::read_card",
            request.game, request.kind
        ))),
    }
}

/// Vertical offset from `card_bounds.y` into each TriPeaks row's non-overlapping scan band.
const TRIPEAKS_BAND_Y_OFFSET: u32 = 85;

/// Vertical height of the non-overlapping TriPeaks tableau scan band (clear of child halo).
const TRIPEAKS_BAND_HEIGHT: u32 = 8;

/// Horizontal margin from each card edge into the bright white border strip (`x + 5..x + 15`).
const TRIPEAKS_BORDER_INSET_X: u32 = 5;

/// Width of each vertical card-paper border strip sampled in the non-overlapping band.
const TRIPEAKS_SIDE_STRIP_WIDTH: u32 = 10;

/// Corner box inset used to confirm unobstructed `CLASSIC BLUE` back coverage.
const TRIPEAKS_CORNER_BACK_INSET: u32 = 6;

/// Corner box width and height kept inside `31×49` foreground inspection bounds.
const TRIPEAKS_CORNER_BACK_WIDTH: u32 = 28;

/// Corner box height kept inside `31×49` foreground inspection bounds.
const TRIPEAKS_CORNER_BACK_HEIGHT: u32 = 44;

/// Top pixel coordinate inside `TARGET_STOCK` vertical interior scan.
const TRIPEAKS_STOCK_SCAN_TOP: u32 = 690;

/// Vertical pixel span sampled per Stock column (`y = 690..856`, avoiding bottom gold halo).
const TRIPEAKS_STOCK_SCAN_HEIGHT: u32 = 166;

/// Minimum contiguous `CLASSIC BLUE` column run required to confirm a non-empty TriPeaks Stock card.
const TRIPEAKS_STOCK_MIN_BLUE_COLUMNS: u32 = 100;

/// Minimum `CLASSIC BLUE` per-mille fraction in one `Stock` vertical column.
const TRIPEAKS_STOCK_COLUMN_BLUE_PER_MILLE: u64 = 750;

/// Minimum gameplay-felt per-mille fraction in one `Stock` vertical column.
const TRIPEAKS_STOCK_COLUMN_FELT_PER_MILLE: u64 = 750;

/// Minimum white-paper per-mille fraction marking one `Stock` card-edge column.
const TRIPEAKS_STOCK_COLUMN_WHITE_PER_MILLE: u64 = 500;

/// Minimum gameplay-felt column count (`80%` of `280` `TARGET_STOCK` columns) proving empty Stock.
const TRIPEAKS_STOCK_EMPTY_FELT_COLUMNS: u32 = 224;

/// Maximum white-paper columns tolerated when `TARGET_STOCK` is empty.
const TRIPEAKS_STOCK_MAX_EMPTY_WHITE_COLUMNS: u32 = 8;

/// Minimum `Stock` blue columns considered non-noise when checking felt/back conflict.
const TRIPEAKS_STOCK_CONFLICT_BLUE_COLUMNS: u32 = 25;

/// Minimum `Stock` felt columns considered non-noise when checking felt/back conflict.
const TRIPEAKS_STOCK_CONFLICT_FELT_COLUMNS: u32 = 200;

/// Minimum dominant surface ratio matching calibrated `GAMEPLAY_FELT_REQUIRED_FRACTION_PER_MILLE`.
const TRIPEAKS_SURFACE_DOMINANT_PER_MILLE: u64 = GAMEPLAY_FELT_REQUIRED_FRACTION_PER_MILLE as u64;

/// Maximum allowed trace pixel ratio from opposing card-back/felt/paper classes.
const TRIPEAKS_SURFACE_TRACE_LIMIT_PER_MILLE: u64 = 50;

/// Minimum per-mille coverage for conflicting simultaneous surface evidence.
const TRIPEAKS_CONFLICT_PER_MILLE: u64 = 200;

/// Minimum bright-white side-border coverage confirming a face-up TriPeaks card.
const TRIPEAKS_FACE_UP_SIDE_PAPER_PER_MILLE: u64 = 500;

/// Minimum green channel matching the darkened TriPeaks Stock tray background.
const TRIPEAKS_STOCK_TRAY_GREEN_MINIMUM: u8 = 55;

/// Minimum green-minus-blue delta inside the darkened TriPeaks Stock tray.
const TRIPEAKS_STOCK_TRAY_GREEN_BLUE_DELTA_MINIMUM: u8 = 20;

/// Surface pixel counters over a bounded native TriPeaks inspection region.
#[derive(Clone, Copy, Default)]
struct TriPeaksSurfaceStats {
    /// Total sampled pixels.
    total: u64,
    /// Pixels matching `CLASSIC BLUE` card-back thresholds.
    blue_back: u64,
    /// Pixels matching neutral bright card-paper minimum channels.
    white_paper: u64,
    /// Pixels matching gameplay green felt thresholds.
    felt: u64,
}

impl TriPeaksSurfaceStats {
    /// Accumulate pixel counts across `rect` in `frame` using a caller-selected felt classifier.
    fn accumulate_with(
        &mut self,
        frame: &CapturedFrame,
        rect: PixelRect,
        is_felt: fn([u8; 3]) -> bool,
    ) -> Result<(), OperationError> {
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                let rgb = pixel_rgb(frame, x, y).ok_or_else(|| {
                    OperationError::Invalid(format!(
                        "TriPeaks inspection coordinate ({x}, {y}) out of native bounds"
                    ))
                })?;
                self.total = self.total.saturating_add(1);
                if is_classic_blue_back(rgb) {
                    self.blue_back = self.blue_back.saturating_add(1);
                }
                if is_tripeaks_white_paper_pixel(rgb) {
                    self.white_paper = self.white_paper.saturating_add(1);
                }
                if is_felt(rgb) {
                    self.felt = self.felt.saturating_add(1);
                }
            }
        }
        Ok(())
    }

    /// Accumulate pixel counts across `rect` using standard bright gameplay felt thresholds.
    fn accumulate(&mut self, frame: &CapturedFrame, rect: PixelRect) -> Result<(), OperationError> {
        self.accumulate_with(frame, rect, is_tripeaks_felt_pixel)
    }

    /// Return `CLASSIC BLUE` card-back ratio in per mille (`0..=1000`).
    fn blue_per_mille(self) -> u64 {
        ratio_per_mille(self.blue_back, self.total)
    }

    /// Return bright card-paper ratio in per mille (`0..=1000`).
    fn white_per_mille(self) -> u64 {
        ratio_per_mille(self.white_paper, self.total)
    }

    /// Return gameplay green felt ratio in per mille (`0..=1000`).
    fn felt_per_mille(self) -> u64 {
        ratio_per_mille(self.felt, self.total)
    }
}

/// Convert `count / total` to rounded-down integer per mille (`0..=1000`).
fn ratio_per_mille(count: u64, total: u64) -> u64 {
    count.saturating_mul(1_000).checked_div(total).unwrap_or(0)
}

/// Sample one rectangular region into `TriPeaksSurfaceStats` using `is_felt`.
fn sample_tripeaks_surface_with(
    frame: &CapturedFrame,
    rect: PixelRect,
    is_felt: fn([u8; 3]) -> bool,
) -> Result<TriPeaksSurfaceStats, OperationError> {
    let mut stats = TriPeaksSurfaceStats::default();
    stats.accumulate_with(frame, rect, is_felt)?;
    Ok(stats)
}

/// Sample one rectangular region into `TriPeaksSurfaceStats` using standard gameplay felt.
fn sample_tripeaks_surface(
    frame: &CapturedFrame,
    rect: PixelRect,
) -> Result<TriPeaksSurfaceStats, OperationError> {
    sample_tripeaks_surface_with(frame, rect, is_tripeaks_felt_pixel)
}

/// Check whether one RGB pixel matches TriPeaks green gameplay felt.
fn is_tripeaks_felt_pixel([red, green, blue]: [u8; 3]) -> bool {
    green >= GAMEPLAY_FELT_GREEN_MINIMUM
        && green.saturating_sub(red) >= GAMEPLAY_FELT_GREEN_RED_DELTA_MINIMUM
        && green.saturating_sub(blue) >= GAMEPLAY_FELT_GREEN_BLUE_DELTA_MINIMUM
}

/// Check whether one RGB pixel matches the darkened TriPeaks Stock tray felt or empty outline.
fn is_tripeaks_stock_tray_green([red, green, blue]: [u8; 3]) -> bool {
    green >= TRIPEAKS_STOCK_TRAY_GREEN_MINIMUM
        && green.saturating_sub(red) >= GAMEPLAY_FELT_GREEN_RED_DELTA_MINIMUM
        && green.saturating_sub(blue) >= TRIPEAKS_STOCK_TRAY_GREEN_BLUE_DELTA_MINIMUM
}

/// Check whether one RGB pixel matches calibrated bright card-face paper.
fn is_tripeaks_white_paper_pixel(rgb: [u8; 3]) -> bool {
    rgb.into_iter()
        .all(|channel| channel >= FACE_UP_WHITE_CHANNEL_MINIMUM)
}

/// Read one 1-based TriPeaks tableau position (`row` `1..=4`, `column` `1..=row_length`).
fn read_tripeaks_play_card(
    frame: &CapturedFrame,
    row: u8,
    column: u8,
) -> Result<CardRead, OperationError> {
    if !(1..=4).contains(&row) {
        return Err(OperationError::Invalid(format!(
            "TriPeaks PLAY row must be 1..=4, got {row}"
        )));
    }
    let row_index = usize::from(row - 1);
    let row_length = TABLEAU_ROW_LENGTHS[row_index];
    if !(1..=row_length).contains(&column) {
        return Err(OperationError::Invalid(format!(
            "TriPeaks PLAY row {row} column must be 1..={row_length}, got {column}"
        )));
    }
    let slot_index = TABLEAU_ROW_STARTS[row_index] + usize::from(column - 1);
    read_tripeaks_card_bounds(frame, TABLEAU_CARD_REGIONS[slot_index].card_bounds, true)
}

/// Read one 1-based TriPeaks draw pile (`1` = Stock across `TARGET_STOCK`, `2` = Waste).
fn read_tripeaks_draw_card(frame: &CapturedFrame, slot: u8) -> Result<CardRead, OperationError> {
    match slot {
        1 => read_tripeaks_stock_card(frame),
        2 => read_tripeaks_card_bounds(frame, WASTE_CARD_BOUNDS, false),
        _ => Err(OperationError::Invalid(format!(
            "TriPeaks DRAW slot must be 1 (Stock) or 2 (Waste), got {slot}"
        ))),
    }
}

/// Read one TriPeaks card rectangle (`allow_face_down` distinguishes covered Tableau from Waste).
fn read_tripeaks_card_bounds(
    frame: &CapturedFrame,
    bounds: PixelRect,
    allow_face_down: bool,
) -> Result<CardRead, OperationError> {
    let classified = classify_corner(frame, (bounds.x, bounds.y))?;
    let band = PixelRect::new(
        bounds.x + TRIPEAKS_CORNER_BACK_INSET,
        bounds.y + TRIPEAKS_BAND_Y_OFFSET,
        bounds.width - TRIPEAKS_CORNER_BACK_INSET * 2,
        TRIPEAKS_BAND_HEIGHT,
    );
    let band_stats = sample_tripeaks_surface(frame, band)?;

    let mut side_stats = TriPeaksSurfaceStats::default();
    let left_strip = PixelRect::new(
        bounds.x + TRIPEAKS_BORDER_INSET_X,
        bounds.y + TRIPEAKS_BAND_Y_OFFSET,
        TRIPEAKS_SIDE_STRIP_WIDTH,
        TRIPEAKS_BAND_HEIGHT,
    );
    let right_strip = PixelRect::new(
        bounds.right() - TRIPEAKS_BORDER_INSET_X - TRIPEAKS_SIDE_STRIP_WIDTH,
        bounds.y + TRIPEAKS_BAND_Y_OFFSET,
        TRIPEAKS_SIDE_STRIP_WIDTH,
        TRIPEAKS_BAND_HEIGHT,
    );
    side_stats.accumulate(frame, left_strip)?;
    side_stats.accumulate(frame, right_strip)?;

    let corner_back_box = PixelRect::new(
        bounds.x + TRIPEAKS_CORNER_BACK_INSET,
        bounds.y + TRIPEAKS_CORNER_BACK_INSET,
        TRIPEAKS_CORNER_BACK_WIDTH,
        TRIPEAKS_CORNER_BACK_HEIGHT,
    );
    let corner_back_stats = sample_tripeaks_surface(frame, corner_back_box)?;

    let band_felt_empty = band_stats.felt_per_mille() >= TRIPEAKS_SURFACE_DOMINANT_PER_MILLE
        && band_stats.blue_per_mille() <= TRIPEAKS_SURFACE_TRACE_LIMIT_PER_MILLE
        && band_stats.white_per_mille() <= TRIPEAKS_SURFACE_TRACE_LIMIT_PER_MILLE;
    let band_blue_back = band_stats.blue_per_mille() >= TRIPEAKS_SURFACE_DOMINANT_PER_MILLE
        && band_stats.felt_per_mille() <= TRIPEAKS_SURFACE_TRACE_LIMIT_PER_MILLE
        && side_stats.white_per_mille() <= TRIPEAKS_SURFACE_TRACE_LIMIT_PER_MILLE;
    let corner_blue_back = corner_back_stats.blue_per_mille()
        >= TRIPEAKS_SURFACE_DOMINANT_PER_MILLE
        && corner_back_stats.felt_per_mille() <= TRIPEAKS_SURFACE_TRACE_LIMIT_PER_MILLE;
    let band_mixed_conflict = (band_stats.blue_per_mille() >= TRIPEAKS_CONFLICT_PER_MILLE
        && band_stats.felt_per_mille() >= TRIPEAKS_CONFLICT_PER_MILLE)
        || (side_stats.white_per_mille() >= TRIPEAKS_CONFLICT_PER_MILLE
            && band_stats.felt_per_mille() >= TRIPEAKS_CONFLICT_PER_MILLE);
    let face_up_side_paper = side_stats.white_per_mille() >= TRIPEAKS_FACE_UP_SIDE_PAPER_PER_MILLE
        && band_stats.felt_per_mille() <= TRIPEAKS_SURFACE_TRACE_LIMIT_PER_MILLE
        && side_stats.blue_per_mille() <= TRIPEAKS_SURFACE_TRACE_LIMIT_PER_MILLE;

    Ok(match classified {
        CornerClassification::Ambiguous { .. } => CardRead::Ambiguous,
        CornerClassification::Card(card) => {
            if band_felt_empty
                || band_blue_back
                || band_mixed_conflict
                || band_stats.felt_per_mille() >= TRIPEAKS_CONFLICT_PER_MILLE
                || side_stats.blue_per_mille() >= TRIPEAKS_CONFLICT_PER_MILLE
            {
                CardRead::Ambiguous
            } else if face_up_side_paper {
                CardRead::Card(card)
            } else {
                CardRead::Unreadable
            }
        }
        CornerClassification::ColourDisagreement { .. }
        | CornerClassification::PartialLabel { .. }
        | CornerClassification::Unreadable { .. } => {
            if band_mixed_conflict {
                CardRead::Ambiguous
            } else if band_felt_empty {
                CardRead::Empty
            } else if allow_face_down
                && matches!(classified, CornerClassification::Unreadable { .. })
                && corner_blue_back
                && band_blue_back
            {
                CardRead::FaceDown
            } else {
                CardRead::Unreadable
            }
        }
    })
}

/// Read TriPeaks Stock (`DRAW 1`) across the horizontally contracting `TARGET_STOCK` lane.
fn read_tripeaks_stock_card(frame: &CapturedFrame) -> Result<CardRead, OperationError> {
    let mut total_blue_cols = 0_u32;
    let mut felt_cols = 0_u32;
    let mut white_cols = 0_u32;
    let mut current_run = 0_u32;
    let mut best_run_len = 0_u32;
    let mut current_start = TARGET_STOCK.x;
    let mut best_run_start = TARGET_STOCK.x;

    for x in TARGET_STOCK.x..TARGET_STOCK.right() {
        let col_rect = PixelRect::new(x, TRIPEAKS_STOCK_SCAN_TOP, 1, TRIPEAKS_STOCK_SCAN_HEIGHT);
        let col_stats =
            sample_tripeaks_surface_with(frame, col_rect, is_tripeaks_stock_tray_green)?;
        if col_stats.blue_per_mille() >= TRIPEAKS_STOCK_COLUMN_BLUE_PER_MILLE {
            total_blue_cols = total_blue_cols.saturating_add(1);
            if current_run == 0 {
                current_start = x;
            }
            current_run = current_run.saturating_add(1);
            if current_run > best_run_len {
                best_run_len = current_run;
                best_run_start = current_start;
            }
        } else {
            current_run = 0;
        }
        if col_stats.felt_per_mille() >= TRIPEAKS_STOCK_COLUMN_FELT_PER_MILLE {
            felt_cols = felt_cols.saturating_add(1);
        }
        if col_stats.white_per_mille() >= TRIPEAKS_STOCK_COLUMN_WHITE_PER_MILLE {
            white_cols = white_cols.saturating_add(1);
        }
    }

    let stock_box = PixelRect::new(
        STOCK_CARD_BOUNDS.x + 4,
        TRIPEAKS_STOCK_SCAN_TOP,
        STOCK_CARD_BOUNDS.width - 8,
        TRIPEAKS_STOCK_SCAN_HEIGHT,
    );
    let stock_box_stats =
        sample_tripeaks_surface_with(frame, stock_box, is_tripeaks_stock_tray_green)?;

    if best_run_len >= TRIPEAKS_STOCK_MIN_BLUE_COLUMNS {
        let top_card_rect = PixelRect::new(
            best_run_start,
            TRIPEAKS_STOCK_SCAN_TOP,
            best_run_len,
            TRIPEAKS_STOCK_SCAN_HEIGHT,
        );
        let top_card_stats =
            sample_tripeaks_surface_with(frame, top_card_rect, is_tripeaks_stock_tray_green)?;
        let detected_corner = classify_corner(
            frame,
            (best_run_start.saturating_sub(4), STOCK_CARD_BOUNDS.y),
        )?;
        let default_corner = classify_corner(frame, (STOCK_CARD_BOUNDS.x, STOCK_CARD_BOUNDS.y))?;
        let has_conflicting_face = matches!(
            detected_corner,
            CornerClassification::Card(_) | CornerClassification::Ambiguous { .. }
        ) || matches!(
            default_corner,
            CornerClassification::Card(_) | CornerClassification::Ambiguous { .. }
        );
        return Ok(
            if has_conflicting_face
                || top_card_stats.felt_per_mille() > TRIPEAKS_SURFACE_TRACE_LIMIT_PER_MILLE
            {
                CardRead::Ambiguous
            } else {
                CardRead::FaceDown
            },
        );
    }

    if (total_blue_cols >= TRIPEAKS_STOCK_CONFLICT_BLUE_COLUMNS
        && felt_cols >= TRIPEAKS_STOCK_CONFLICT_FELT_COLUMNS)
        || (stock_box_stats.blue_per_mille() >= TRIPEAKS_CONFLICT_PER_MILLE
            && stock_box_stats.felt_per_mille() >= TRIPEAKS_CONFLICT_PER_MILLE)
    {
        return Ok(CardRead::Ambiguous);
    }

    if total_blue_cols == 0
        && white_cols <= TRIPEAKS_STOCK_MAX_EMPTY_WHITE_COLUMNS
        && felt_cols >= TRIPEAKS_STOCK_EMPTY_FELT_COLUMNS
    {
        let corner = classify_corner(frame, (STOCK_CARD_BOUNDS.x, STOCK_CARD_BOUNDS.y))?;
        return Ok(
            if matches!(
                corner,
                CornerClassification::Card(_) | CornerClassification::Ambiguous { .. }
            ) {
                CardRead::Ambiguous
            } else {
                CardRead::Empty
            },
        );
    }

    Ok(CardRead::Unreadable)
}

/// Check whether one native `1920×1080` frame proves both **Challenge Complete** (`ll`) and **Continue** (`[0, 0, 0]`).
pub fn is_challenge_complete_frame(frame: &CapturedFrame) -> Result<bool, OperationError> {
    validate_native_frame(frame)?;
    let view = frame
        .as_view()
        .map_err(|error| OperationError::Invalid(error.to_string()))?;
    for rect in CHALLENGE_LL_RECTS {
        if !all_exact_rgb(&view, rect, [255, 255, 255])? {
            return Ok(false);
        }
    }
    for rect in CONTINUE_BLACK_RECTS {
        if !all_exact_rgb(&view, rect, [0, 0, 0])? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Request one fresh `1920×1080` frame and test for **Challenge Complete** with Continue visible; sends no input.
pub fn challenge_complete(io: &mut impl OperationIo) -> Result<bool, OperationError> {
    io.check_active()?;
    let frame = io.capture()?;
    io.check_active()?;
    is_challenge_complete_frame(&frame)
}

/// Wait up to 5 new-frame observations (at most 4 one-second cancellable waits) for Challenge Complete.
///
/// Propagates cancellation, transport, wait, and invalid-frame errors immediately and sends zero input.
pub fn wait_for_challenge_complete(io: &mut impl OperationIo) -> Result<bool, OperationError> {
    for attempt in 0..CHALLENGE_COMPLETE_MAX_ATTEMPTS {
        if challenge_complete(io)? {
            return Ok(true);
        }
        if attempt + 1 < CHALLENGE_COMPLETE_MAX_ATTEMPTS {
            io.wait(CHALLENGE_COMPLETE_POLL_INTERVAL)?;
        }
    }
    Ok(false)
}

/// Convert a corner classification into `CardRead` for public location queries.
fn read_from_classification(classification: CornerClassification) -> CardRead {
    match classification {
        CornerClassification::Card(card) => CardRead::Card(card),
        CornerClassification::Ambiguous { .. } => CardRead::Ambiguous,
        CornerClassification::ColourDisagreement { .. }
        | CornerClassification::PartialLabel { .. }
        | CornerClassification::Unreadable { .. } => CardRead::Unreadable,
    }
}

/// Verify every pixel inside `rect` matches `expected` RGB with zero tolerance.
fn all_exact_rgb(
    view: &xsar::image_matching::FrameView<'_>,
    rect: PixelRect,
    expected: [u8; 3],
) -> Result<bool, OperationError> {
    view.checked_bounds(rect)
        .map_err(|error| OperationError::Invalid(format!("{error:?}")))?;
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            let rgb = view.pixel_rgb(x, y).ok_or_else(|| {
                OperationError::Invalid("Challenge Complete pixel outside frame bounds".to_owned())
            })?;
            if rgb != expected {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// Verify at least `per_mille` thousandths of `rect` satisfy `predicate`.
fn coverage_per_mille(
    view: &xsar::image_matching::FrameView<'_>,
    rect: PixelRect,
    predicate: impl Fn([u8; 3]) -> bool,
    per_mille: u64,
) -> Result<bool, OperationError> {
    view.checked_bounds(rect)
        .map_err(|error| OperationError::Invalid(format!("{error:?}")))?;
    let mut matching = 0_u64;
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            let rgb = view.pixel_rgb(x, y).ok_or_else(|| {
                OperationError::Invalid("Coverage pixel outside frame bounds".to_owned())
            })?;
            if predicate(rgb) {
                matching += 1;
            }
        }
    }
    Ok(matching * 1000 >= u64::from(rect.width) * u64::from(rect.height) * per_mille)
}


#[cfg(test)]
mod tests;
