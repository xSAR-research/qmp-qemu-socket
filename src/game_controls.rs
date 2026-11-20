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
    capture::{CapturedFrame, PixelFormat},
    cards::{TABLEAU_ROW_LENGTHS, TABLEAU_ROW_STARTS},
    detector::pixel_rgb,
    freecell::{CELL_X, FOUNDATION_X, PLAY_X},
    game::{GameMode, is_classic_blue_back},
    game_operations::{CardRead, OperationError, OperationIo, validate_native_frame},
    geometry::PixelRect,
    parameters::{
        FACE_UP_WHITE_CHANNEL_MINIMUM, GAMEPLAY_FELT_GREEN_BLUE_DELTA_MINIMUM,
        GAMEPLAY_FELT_GREEN_MINIMUM, GAMEPLAY_FELT_GREEN_RED_DELTA_MINIMUM,
        GAMEPLAY_FELT_REQUIRED_FRACTION_PER_MILLE, NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH,
        STOCK_CARD_BOUNDS, TABLEAU_CARD_REGIONS, TARGET_STOCK, WASTE_CARD_BOUNDS,
    },
    pyramid_recon::vision::{
        CornerClassification, classify_foreground_corner, pyramid_tableau_origin,
        read_pyramid_draw_card,
    },
    tripeaks_progress::{UNDO_LAST_MOVE_COMPACT_CLICK_POINT, observe_no_moves_modal},
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

/// Settled `T = 0` left deck-edge white paper stack inside `TARGET_STOCK` (`x = 768..818`).
const TRIPEAKS_ROOT_STOCK_LEFT_STACK_BOUNDS: PixelRect = PixelRect::new(768, 698, 50, 150);

/// Minimum neutral bright paper per-mille fraction across the `T = 0` Stock deck stack.
const TRIPEAKS_ROOT_STOCK_STACK_WHITE_PER_MILLE: u64 = 950;

/// Minimum red channel on warm ivory Challenge target-rank card paper (`RGB ~ [252, 246, 230]`).
const TRIPEAKS_TARGET_PAPER_RED_MINIMUM: u8 = 242;

/// Minimum green channel on warm ivory Challenge target-rank card paper.
const TRIPEAKS_TARGET_PAPER_GREEN_MINIMUM: u8 = 235;

/// Minimum blue channel on warm ivory Challenge target-rank card paper (excluding gold glow borders).
const TRIPEAKS_TARGET_PAPER_BLUE_MINIMUM: u8 = 215;

/// Maximum red-minus-blue warmth spread on Challenge target-rank card paper.
const TRIPEAKS_TARGET_PAPER_MAX_WARMTH: u8 = 40;

/// Surface pixel counters over a bounded native TriPeaks inspection region.
#[derive(Clone, Copy, Default)]
struct TriPeaksSurfaceStats {
    /// Total sampled pixels.
    total: u64,
    /// Pixels matching `CLASSIC BLUE` card-back thresholds.
    blue_back: u64,
    /// Pixels matching calibrated neutral bright or warm ivory card-face paper.
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

/// Check whether one RGB pixel matches calibrated neutral bright or warm ivory card-face paper,
/// excluding decorative gold Challenge glow borders.
fn is_tripeaks_white_paper_pixel([red, green, blue]: [u8; 3]) -> bool {
    (red >= FACE_UP_WHITE_CHANNEL_MINIMUM
        && green >= FACE_UP_WHITE_CHANNEL_MINIMUM
        && blue >= FACE_UP_WHITE_CHANNEL_MINIMUM)
        || (red >= TRIPEAKS_TARGET_PAPER_RED_MINIMUM
            && green >= TRIPEAKS_TARGET_PAPER_GREEN_MINIMUM
            && blue >= TRIPEAKS_TARGET_PAPER_BLUE_MINIMUM
            && red >= green
            && green >= blue
            && red.saturating_sub(blue) <= TRIPEAKS_TARGET_PAPER_MAX_WARMTH)
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

/// Positively confirm that a full `23`-card `T = 0` TriPeaks Stock pile is settled in place.
///
/// While Microsoft Solitaire rewinds draws during `Undo All`, `SHARED_UNDO_ALL_CONTROL` greys out
/// immediately even while Stock remains horizontally contracted (`x = 953..968` exposed green tray)
/// or flying cards expose `CLASSIC BLUE` stripes inside the white left deck-edge stack.
pub(crate) fn is_tripeaks_full_root_stock(frame: &CapturedFrame) -> Result<bool, OperationError> {
    validate_native_frame(frame)?;

    let interior_start = STOCK_CARD_BOUNDS.x + 4;
    let interior_end = STOCK_CARD_BOUNDS.right().saturating_sub(2);
    for x in interior_start..interior_end {
        let col_rect = PixelRect::new(x, TRIPEAKS_STOCK_SCAN_TOP, 1, TRIPEAKS_STOCK_SCAN_HEIGHT);
        let col_stats =
            sample_tripeaks_surface_with(frame, col_rect, is_tripeaks_stock_tray_green)?;
        if col_stats.blue_per_mille() < TRIPEAKS_STOCK_COLUMN_BLUE_PER_MILLE
            || col_stats.felt_per_mille() > TRIPEAKS_SURFACE_TRACE_LIMIT_PER_MILLE
        {
            return Ok(false);
        }
    }

    let overhang_width = TARGET_STOCK
        .right()
        .saturating_sub(STOCK_CARD_BOUNDS.right());
    if overhang_width > 0 {
        let overhang_rect = PixelRect::new(
            STOCK_CARD_BOUNDS.right(),
            TRIPEAKS_STOCK_SCAN_TOP,
            overhang_width,
            TRIPEAKS_STOCK_SCAN_HEIGHT,
        );
        let overhang_stats =
            sample_tripeaks_surface_with(frame, overhang_rect, is_tripeaks_stock_tray_green)?;
        if overhang_stats.blue_back > 0 {
            return Ok(false);
        }
    }

    let stack_stats = sample_tripeaks_surface_with(
        frame,
        TRIPEAKS_ROOT_STOCK_LEFT_STACK_BOUNDS,
        is_tripeaks_stock_tray_green,
    )?;
    Ok(
        stack_stats.white_per_mille() >= TRIPEAKS_ROOT_STOCK_STACK_WHITE_PER_MILLE
            && stack_stats.blue_back == 0
            && stack_stats.felt == 0,
    )
}

/// Minimum per-mille dimmed tray/outline coverage proving an empty Stock slot behind a modal.
const TRIPEAKS_DIMMED_EMPTY_PER_MILLE: u64 = 850;

/// Maximum per-mille stray dimmed paper or blue-back pixels tolerated inside an empty Stock tray.
const TRIPEAKS_DIMMED_NOISE_LIMIT_PER_MILLE: u64 = 20;

/// Minimum per-mille dimmed card-paper coverage required across unobstructed Waste paper strips.
const TRIPEAKS_DIMMED_WASTE_PAPER_PER_MILLE: u64 = 500;

/// Minimum per-mille unobstructed lower Waste paper required for tall Normal-modal rotated fallback.
const TRIPEAKS_DIMMED_LOWER_PAPER_PER_MILLE: u64 = 750;

/// Minimum per-channel brightness accepted as dimmed white/ivory card paper under a modal overlay.
const TRIPEAKS_DIMMED_PAPER_MIN_CHANNEL: u8 = 25;

/// Maximum per-channel brightness accepted as dimmed card paper (excludes un-dimmed bright white).
const TRIPEAKS_DIMMED_PAPER_MAX_CHANNEL: u8 = 145;

/// Maximum channel spread accepted for dimmed neutral white or warm-ivory card paper.
const TRIPEAKS_DIMMED_PAPER_MAX_SPREAD: u8 = 18;

/// Unobstructed Stock verification geometry paired with one modal layout (`compact` vs `tall`).
struct DimmedStockRegions<'a> {
    /// Unobstructed dark-green interior tray rectangle.
    interior: PixelRect,
    /// Unobstructed dimmed light-green empty-slot outline edges.
    outline_edges: &'a [PixelRect],
}

/// Compact Challenge modal leaves all four empty Stock edges and full interior unobstructed.
const COMPACT_DIMMED_STOCK_REGIONS: DimmedStockRegions<'static> = DimmedStockRegions {
    interior: PixelRect::new(776, 692, 116, 160),
    outline_edges: &[
        PixelRect::new(780, 682, 106, 4),
        PixelRect::new(780, 857, 106, 4),
        PixelRect::new(768, 692, 3, 158),
        PixelRect::new(898, 692, 3, 158),
    ],
};

/// Tall Normal modal overlaps `y = 682..795`, leaving the lower tray and three edges unobstructed.
const TALL_DIMMED_STOCK_REGIONS: DimmedStockRegions<'static> = DimmedStockRegions {
    interior: PixelRect::new(776, 800, 116, 52),
    outline_edges: &[
        PixelRect::new(780, 857, 106, 4),
        PixelRect::new(768, 800, 3, 50),
        PixelRect::new(898, 800, 3, 50),
    ],
};

/// Pixel predicate matching darkened empty Stock tray interior behind a modal dim overlay.
fn is_dimmed_stock_tray_pixel([red, green, blue]: [u8; 3]) -> bool {
    red <= 18 && (8..=25).contains(&green) && green > red && green >= blue
}

/// Pixel predicate matching the darkened light-green empty Stock slot outline behind a modal overlay.
fn is_dimmed_stock_outline_pixel([red, green, blue]: [u8; 3]) -> bool {
    green >= 26 && green.saturating_sub(red) >= 14 && green.saturating_sub(blue) >= 5
}

/// Pixel predicate matching a darkened `CLASSIC BLUE` card back behind a modal dim overlay.
fn is_dimmed_blue_back_pixel([red, green, blue]: [u8; 3]) -> bool {
    blue >= 24 && blue >= green.saturating_add(8) && blue >= red.saturating_add(16)
}

/// Pixel predicate matching darkened neutral white or warm-ivory card paper behind a modal overlay.
fn is_dimmed_paper_pixel([red, green, blue]: [u8; 3]) -> bool {
    let min_c = red.min(green).min(blue);
    let max_c = red.max(green).max(blue);
    min_c >= TRIPEAKS_DIMMED_PAPER_MIN_CHANNEL
        && max_c <= TRIPEAKS_DIMMED_PAPER_MAX_CHANNEL
        && max_c.saturating_sub(min_c) <= TRIPEAKS_DIMMED_PAPER_MAX_SPREAD
}

/// Internal Stock verification status behind an active `"No Moves Remaining"` modal.
#[derive(Clone, Copy, Eq, PartialEq)]
enum DimmedStockStatus {
    /// Positive dimmed empty-Stock tray and outline verified.
    Empty,
    /// Conflicting dimmed blue card back observed inside Stock.
    BlueConflict,
    /// Neither empty-Stock outline/tray proof nor clear conflict succeeded.
    Unverified,
}

/// Verify unobstructed dimmed empty-Stock tray and outline pixels for the active modal layout.
fn verify_dimmed_empty_stock(
    view: &xsar::image_matching::FrameView<'_>,
    compact_modal: bool,
) -> Result<DimmedStockStatus, OperationError> {
    let regions = if compact_modal {
        &COMPACT_DIMMED_STOCK_REGIONS
    } else {
        &TALL_DIMMED_STOCK_REGIONS
    };
    view.checked_bounds(regions.interior)
        .map_err(|error| OperationError::Invalid(format!("{error:?}")))?;
    let mut total = 0_u64;
    let mut tray = 0_u64;
    let mut blue = 0_u64;
    let mut paper = 0_u64;
    for y in regions.interior.y..regions.interior.bottom() {
        for x in regions.interior.x..regions.interior.right() {
            let rgb = view.pixel_rgb(x, y).ok_or_else(|| {
                OperationError::Invalid("Dimmed Stock interior pixel out of bounds".to_owned())
            })?;
            total += 1;
            if is_dimmed_stock_tray_pixel(rgb) {
                tray += 1;
            }
            if is_dimmed_blue_back_pixel(rgb) {
                blue += 1;
            }
            if is_dimmed_paper_pixel(rgb) {
                paper += 1;
            }
        }
    }
    if ratio_per_mille(blue, total) >= TRIPEAKS_CONFLICT_PER_MILLE {
        return Ok(DimmedStockStatus::BlueConflict);
    }
    if ratio_per_mille(tray, total) < TRIPEAKS_DIMMED_EMPTY_PER_MILLE
        || ratio_per_mille(blue, total) > TRIPEAKS_DIMMED_NOISE_LIMIT_PER_MILLE
        || ratio_per_mille(paper, total) > TRIPEAKS_DIMMED_NOISE_LIMIT_PER_MILLE
    {
        return Ok(DimmedStockStatus::Unverified);
    }
    for &edge in regions.outline_edges {
        if !coverage_per_mille(
            view,
            edge,
            is_dimmed_stock_outline_pixel,
            TRIPEAKS_DIMMED_EMPTY_PER_MILLE,
        )? {
            return Ok(DimmedStockStatus::Unverified);
        }
    }
    Ok(DimmedStockStatus::Empty)
}

/// Sample dimmed Waste card paper across `rects` and return median `[R, G, B]` if coverage holds.
fn sample_dimmed_waste_paper(
    view: &xsar::image_matching::FrameView<'_>,
    rects: &[PixelRect],
    min_paper_per_mille: u64,
) -> Result<Option<[u8; 3]>, OperationError> {
    let mut total = 0_u64;
    let mut tray_or_blue = 0_u64;
    let mut reds = Vec::new();
    let mut greens = Vec::new();
    let mut blues = Vec::new();
    for &rect in rects {
        view.checked_bounds(rect)
            .map_err(|error| OperationError::Invalid(format!("{error:?}")))?;
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                let rgb = view.pixel_rgb(x, y).ok_or_else(|| {
                    OperationError::Invalid("Dimmed Waste paper pixel out of bounds".to_owned())
                })?;
                total += 1;
                if is_dimmed_paper_pixel(rgb) {
                    reds.push(rgb[0]);
                    greens.push(rgb[1]);
                    blues.push(rgb[2]);
                } else if is_dimmed_stock_tray_pixel(rgb) || is_dimmed_blue_back_pixel(rgb) {
                    tray_or_blue += 1;
                }
            }
        }
    }
    let paper_count = u64::try_from(reds.len()).unwrap_or(0);
    if ratio_per_mille(paper_count, total) < min_paper_per_mille
        || ratio_per_mille(tray_or_blue, total) > TRIPEAKS_SURFACE_TRACE_LIMIT_PER_MILLE
    {
        return Ok(None);
    }
    reds.sort_unstable();
    greens.sort_unstable();
    blues.sort_unstable();
    let mid = reds.len() / 2;
    Ok(Some([reds[mid], greens[mid], blues[mid]]))
}

/// Contrast-normalise one dimmed RGB pixel relative to measured dimmed `paper` RGB.
fn normalise_dimmed_pixel(rgb: [u8; 3], paper: [u8; 3]) -> [u8; 3] {
    let mut out = [0_u8; 3];
    for ch in 0..3 {
        let denom = u32::from(paper[ch].max(1));
        out[ch] = ((u32::from(rgb[ch]) * 255) / denom).min(255) as u8;
    }
    out
}

/// Compact Challenge-mode `"No Moves Remaining"` panel envelope plus outer drop-shadow margin.
const COMPACT_MODAL_OCCLUSION_BOUNDS: PixelRect = PixelRect::new(525, 387, 870, 290);

/// Tall Normal-mode `"No Moves Remaining"` panel envelope plus outer drop-shadow margin.
const TALL_MODAL_OCCLUSION_BOUNDS: PixelRect = PixelRect::new(566, 262, 788, 540);

/// Return `true` when two axis-aligned pixel rectangles intersect.
const fn rects_overlap(a: PixelRect, b: PixelRect) -> bool {
    a.x < b.right() && b.x < a.right() && a.y < b.bottom() && b.y < a.bottom()
}

/// Normalise `copy_bounds` relative to `paper` onto an empty native buffer and return it.
fn normalise_dimmed_region(
    view: &xsar::image_matching::FrameView<'_>,
    copy_bounds: PixelRect,
    paper: [u8; 3],
) -> Result<CapturedFrame, OperationError> {
    let mut normalised = blank_native_frame();
    for y in copy_bounds.y..copy_bounds.bottom() {
        for x in copy_bounds.x..copy_bounds.right() {
            let rgb = view.pixel_rgb(x, y).ok_or_else(|| {
                OperationError::Invalid("Dimmed card region pixel out of bounds".to_owned())
            })?;
            let scaled = normalise_dimmed_pixel(rgb, paper);
            let off = y as usize * normalised.stride + x as usize * 4;
            normalised.pixels[off..off + 3].copy_from_slice(&scaled);
        }
    }
    Ok(normalised)
}

/// Read an unobstructed dimmed face-up card inside `bounds` given measured `modal_box`.
fn read_unobstructed_dimmed_card(
    view: &xsar::image_matching::FrameView<'_>,
    bounds: PixelRect,
    modal_box: PixelRect,
) -> Result<CardRead, OperationError> {
    let corner_crop = PixelRect::new(bounds.x + 3, bounds.y + 4, 31, 49);
    if rects_overlap(corner_crop, modal_box) {
        return Ok(CardRead::Unreadable);
    }

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
    if !rects_overlap(left_strip, modal_box) && !rects_overlap(right_strip, modal_box) {
        let Some(paper) = sample_dimmed_waste_paper(
            view,
            &[left_strip, right_strip],
            TRIPEAKS_DIMMED_WASTE_PAPER_PER_MILLE,
        )?
        else {
            return Ok(CardRead::Unreadable);
        };
        let normalised = normalise_dimmed_region(view, bounds, paper)?;
        return read_tripeaks_card_bounds(&normalised, bounds, false);
    }

    // When the mid-card band (`y + 85`) sits behind the modal while the top corner (`y + 4..53`)
    // remains clear (e.g. Row 3 middle slots above the compact Challenge modal), sample the upper
    // white paper borders clear of court-card portrait frames (`dy = 4..10`, `dx = 121..130`).
    let top_strip = PixelRect::new(bounds.x + 40, bounds.y + 4, 65, 6);
    let right_margin_strip = PixelRect::new(bounds.right() - 15, bounds.y + 15, 9, 30);
    if rects_overlap(top_strip, modal_box) || rects_overlap(right_margin_strip, modal_box) {
        return Ok(CardRead::Unreadable);
    }
    let Some(paper) = sample_dimmed_waste_paper(
        view,
        &[top_strip, right_margin_strip],
        TRIPEAKS_DIMMED_LOWER_PAPER_PER_MILLE,
    )?
    else {
        return Ok(CardRead::Unreadable);
    };
    let normalised = normalise_dimmed_region(view, corner_crop, paper)?;
    Ok(read_from_classification(classify_corner(
        &normalised,
        (bounds.x, bounds.y),
    )?))
}

/// Allocate an empty packed `1920×1080` `CapturedFrame` scratch surface for contrast normalisation.
fn blank_native_frame() -> CapturedFrame {
    let width = NOMINAL_FRAME_WIDTH as usize;
    let height = NOMINAL_FRAME_HEIGHT as usize;
    let stride = width * 4;
    let mut pixels = vec![0_u8; stride * height];
    let (chunks, _) = pixels.as_chunks_mut::<4>();
    for chunk in chunks {
        chunk[3] = 255;
    }
    CapturedFrame {
        width: NOMINAL_FRAME_WIDTH,
        height: NOMINAL_FRAME_HEIGHT,
        stride,
        format: PixelFormat::Rgba8,
        pixels,
    }
}

/// Positively verify the darkened empty Stock tray while a `"No Moves Remaining"` modal is active.
///
/// Strictly gated on [`observe_no_moves_modal`]; returns `Ok(false)` on un-dimmed frames or when
/// no calibrated modal overlay is present.
///
/// # Errors
///
/// Returns [`OperationError::Invalid`] if `frame` is not packed `1920×1080` RGBA8.
pub fn is_tripeaks_dimmed_empty_stock(frame: &CapturedFrame) -> Result<bool, OperationError> {
    validate_native_frame(frame)?;
    let Some(modal_click) = observe_no_moves_modal(frame)? else {
        return Ok(false);
    };
    let view = frame
        .as_view()
        .map_err(|error| OperationError::Invalid(error.to_string()))?;
    Ok(
        verify_dimmed_empty_stock(&view, modal_click == UNDO_LAST_MOVE_COMPACT_CLICK_POINT)?
            == DimmedStockStatus::Empty,
    )
}

/// Read the darkened Draw-23 Waste card and confirm the darkened empty Stock tray while a
/// `"No Moves Remaining"` modal dim overlay is positively active.
///
/// Strictly gated on [`observe_no_moves_modal`] so un-dimmed [`read_card`] thresholds remain
/// untouched. Contrast-normalises the unobstructed dimmed Waste paper/ink (`WASTE_CARD_BOUNDS`
/// directly on the compact Challenge modal, or the unobstructed 180°-rotated bottom-right corner
/// fallback when the tall Normal modal overlaps `(1017, 682)`) and runs the full foreground rank
/// and suit classification margins.
///
/// # Errors
///
/// Returns [`OperationError`] on invalid native frame layout or underlying recognition errors.
pub fn read_tripeaks_dimmed_modal_waste(frame: &CapturedFrame) -> Result<CardRead, OperationError> {
    validate_native_frame(frame)?;
    let Some(modal_click) = observe_no_moves_modal(frame)? else {
        return Ok(CardRead::Unreadable);
    };
    let compact_modal = modal_click == UNDO_LAST_MOVE_COMPACT_CLICK_POINT;
    let view = frame
        .as_view()
        .map_err(|error| OperationError::Invalid(error.to_string()))?;

    match verify_dimmed_empty_stock(&view, compact_modal)? {
        DimmedStockStatus::Empty => {}
        DimmedStockStatus::BlueConflict => return Ok(CardRead::Ambiguous),
        DimmedStockStatus::Unverified => return Ok(CardRead::Unreadable),
    }

    if compact_modal {
        return read_unobstructed_dimmed_card(
            &view,
            WASTE_CARD_BOUNDS,
            COMPACT_MODAL_OCCLUSION_BOUNDS,
        );
    }

    // Tall Normal-mode modal overlaps `(1017, 682)` through `y = 795`; fall back to the
    // unobstructed lower paper strips and 180°-rotated bottom-right corner (`y >= 807`).
    let lower_left_strip = PixelRect::new(
        WASTE_CARD_BOUNDS.x + TRIPEAKS_BORDER_INSET_X,
        848,
        TRIPEAKS_SIDE_STRIP_WIDTH,
        8,
    );
    let lower_bottom_strip = PixelRect::new(WASTE_CARD_BOUNDS.x + 10, 853, 80, 6);
    let Some(paper) = sample_dimmed_waste_paper(
        &view,
        &[lower_left_strip, lower_bottom_strip],
        TRIPEAKS_DIMMED_LOWER_PAPER_PER_MILLE,
    )?
    else {
        return Ok(CardRead::Unreadable);
    };

    let mut normalised = blank_native_frame();
    let max_x = WASTE_CARD_BOUNDS.x + 134;
    let max_y = WASTE_CARD_BOUNDS.y + 181;
    for dy in 0..57_u32 {
        for dx in 0..38_u32 {
            let sx = max_x - dx;
            let sy = max_y - dy;
            let rgb = view.pixel_rgb(sx, sy).ok_or_else(|| {
                OperationError::Invalid(
                    "Dimmed rotated Waste corner pixel out of bounds".to_owned(),
                )
            })?;
            let scaled = normalise_dimmed_pixel(rgb, paper);
            let off = (WASTE_CARD_BOUNDS.y + dy) as usize * normalised.stride
                + (WASTE_CARD_BOUNDS.x + dx) as usize * 4;
            normalised.pixels[off..off + 3].copy_from_slice(&scaled);
        }
    }
    Ok(read_from_classification(classify_corner(
        &normalised,
        (WASTE_CARD_BOUNDS.x, WASTE_CARD_BOUNDS.y),
    )?))
}

/// Read an unobstructed dimmed TriPeaks tableau slot (`0..28`) while a `"No Moves Remaining"`
/// modal dim overlay is positively active (used by mid-RECON probe recovery).
///
/// Strictly gated on [`observe_no_moves_modal`]; returns `Ok(CardRead::Unreadable)` when no modal
/// is active or when the slot corner/paper strips overlap the modal panel.
///
/// # Errors
///
/// Returns [`OperationError`] on out-of-range `slot_index` or invalid frame layout.
pub(crate) fn read_tripeaks_dimmed_modal_tableau_slot(
    frame: &CapturedFrame,
    slot_index: usize,
) -> Result<CardRead, OperationError> {
    validate_native_frame(frame)?;
    let region = TABLEAU_CARD_REGIONS.get(slot_index).ok_or_else(|| {
        OperationError::Invalid(format!(
            "Invalid TriPeaks tableau slot index {slot_index} (expected 0..{})",
            TABLEAU_CARD_REGIONS.len()
        ))
    })?;
    let Some(modal_click) = observe_no_moves_modal(frame)? else {
        return Ok(CardRead::Unreadable);
    };
    let modal_box = if modal_click == UNDO_LAST_MOVE_COMPACT_CLICK_POINT {
        COMPACT_MODAL_OCCLUSION_BOUNDS
    } else {
        TALL_MODAL_OCCLUSION_BOUNDS
    };
    let view = frame
        .as_view()
        .map_err(|error| OperationError::Invalid(error.to_string()))?;
    read_unobstructed_dimmed_card(&view, region.card_bounds, modal_box)
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
