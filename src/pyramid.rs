use std::{fmt, time::Duration};

use crate::{
    capture::CapturedFrame,
    detector::{HaloDetectionError, pixel_rgb},
    game::{
        ActionSpecification, ActionTarget, AnimationClass, GameMode, GameProfile,
        GameplaySceneProfile, GuidedAction, InputOperation, PreviewTarget, RepeatTargetPolicy,
        TargetSelectionPolicy,
    },
    geometry::{PixelPoint, PixelRect},
    parameters::{
        ACTION_CHANGE_CHANNEL_THRESHOLD, NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH,
        SHARED_SOLVER_PROGRESS_PROFILE,
    },
    tracker::{FrameAnalysis, PredictedAction, RowMask},
};

pub const TARGET_COUNT: usize = 31;
pub const TABLEAU_CARD_COUNT: usize = 28;

/// Semantic identity and scan priority for a Pyramid target.
/// Rows run from the apex (1) to the bottom (7); columns run left to right.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PyramidTargetKind {
    Move,
    Left,
    Right,
    Card { row: u8, column: u8 },
}

impl PyramidTargetKind {
    pub const fn is_card(self) -> bool {
        matches!(self, Self::Card { .. })
    }

    pub const fn slot_index(self) -> Option<usize> {
        match self {
            Self::Move => Some(0),
            Self::Left => Some(1),
            Self::Right => Some(2),
            Self::Card { row, column } if row >= 1 && row <= 7 && column >= 1 && column <= row => {
                let row = row as usize;
                Some(3 + TABLEAU_CARD_COUNT - row * (row + 1) / 2 + column as usize - 1)
            }
            Self::Card { .. } => None,
        }
    }
}

impl fmt::Display for PyramidTargetKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Move => formatter.write_str("Move/Recycle"),
            Self::Left => formatter.write_str("Left"),
            Self::Right => formatter.write_str("Right"),
            Self::Card { row, column } => {
                write!(formatter, "row {row}, column {column}")
            }
        }
    }
}

/// Per-board history; only a verified removal of the clicked card consumes a slot.
/// An unclicked pair partner is never inferred into this history.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PyramidBoardState {
    pub clicked_target_slots: [bool; TARGET_COUNT],
}

impl PyramidBoardState {
    pub const fn new() -> Self {
        Self {
            clicked_target_slots: [false; TARGET_COUNT],
        }
    }

    pub fn mark_removed(&mut self, kind: PyramidTargetKind) {
        if kind.is_card()
            && let Some(index) = kind.slot_index()
        {
            self.clicked_target_slots[index] = true;
        }
    }
}

impl Default for PyramidBoardState {
    fn default() -> Self {
        Self::new()
    }
}

/// Calibrated guest-pixel geometry. Geometry alone does not grant input authority.
/// Bounds are half-open visible-face envelopes, excluding shadows and halos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PyramidTargetSlot {
    pub kind: PyramidTargetKind,
    pub label: &'static str,
    pub bounds: PixelRect,
    pub click_point: PixelPoint,
}

const fn target(
    kind: PyramidTargetKind,
    label: &'static str,
    bounds: PixelRect,
    click_point: PixelPoint,
) -> PyramidTargetSlot {
    PyramidTargetSlot {
        kind,
        label,
        bounds,
        click_point,
    }
}

// Measured from Issue #1's 1920x1080 captures. The envelope includes the
// one-pixel variation in antialiased card edges. See docs/pyramid-calibration.md.
pub const CARD_FACE_WIDTH: u32 = 140;
pub const CARD_FACE_HEIGHT: u32 = 187;

const fn card(row: u8, column: u8, label: &'static str, x: u32, y: u32) -> PyramidTargetSlot {
    target(
        PyramidTargetKind::Card { row, column },
        label,
        PixelRect::new(x, y, CARD_FACE_WIDTH, CARD_FACE_HEIGHT),
        // The top strip remains visible while lower rows overlap the card.
        // This point is geometry only; it says nothing about whether a card
        // is uncovered, highlighted or legal to play.
        PixelPoint::new((x + 70) as i32, (y + 24) as i32),
    )
}

/// The same visual control moves a card or recycles the pile. Geometry alone
/// cannot distinguish the operation; only its fresh halo authorises a click.
pub const MOVE_TARGET: PyramidTargetSlot = target(
    PyramidTargetKind::Move,
    "PY Move/Recycle",
    PixelRect::new(920, 678, 80, 80),
    PixelPoint::new(960, 718),
);
pub const LEFT_TARGET: PyramidTargetSlot = target(
    PyramidTargetKind::Left,
    "PY Left",
    PixelRect::new(759, 678, 139, 187),
    PixelPoint::new(828, 771),
);
pub const RIGHT_TARGET: PyramidTargetSlot = target(
    PyramidTargetKind::Right,
    "PY Right",
    PixelRect::new(1_022, 678, 139, 187),
    PixelPoint::new(1_091, 771),
);

/// Retain the agreed semantic order: Move, Left, Right, then bottom to apex.
/// Every target has a separate fixed halo probe and a calibrated click point.
pub const PYRAMID_TARGETS: [PyramidTargetSlot; TARGET_COUNT] = [
    MOVE_TARGET,
    LEFT_TARGET,
    RIGHT_TARGET,
    card(7, 1, "PY r7c1", 288, 432),
    card(7, 2, "PY r7c2", 489, 432),
    card(7, 3, "PY r7c3", 690, 432),
    card(7, 4, "PY r7c4", 890, 432),
    card(7, 5, "PY r7c5", 1_091, 432),
    card(7, 6, "PY r7c6", 1_291, 432),
    card(7, 7, "PY r7c7", 1_492, 432),
    card(6, 1, "PY r6c1", 389, 378),
    card(6, 2, "PY r6c2", 589, 378),
    card(6, 3, "PY r6c3", 790, 378),
    card(6, 4, "PY r6c4", 990, 378),
    card(6, 5, "PY r6c5", 1_191, 378),
    card(6, 6, "PY r6c6", 1_392, 378),
    card(5, 1, "PY r5c1", 489, 325),
    card(5, 2, "PY r5c2", 690, 325),
    card(5, 3, "PY r5c3", 890, 325),
    card(5, 4, "PY r5c4", 1_091, 325),
    card(5, 5, "PY r5c5", 1_291, 325),
    card(4, 1, "PY r4c1", 589, 272),
    card(4, 2, "PY r4c2", 790, 272),
    card(4, 3, "PY r4c3", 990, 272),
    card(4, 4, "PY r4c4", 1_191, 272),
    card(3, 1, "PY r3c1", 690, 218),
    card(3, 2, "PY r3c2", 890, 218),
    card(3, 3, "PY r3c3", 1_091, 218),
    card(2, 1, "PY r2c1", 790, 165),
    card(2, 2, "PY r2c2", 990, 165),
    card(1, 1, "PY r1c1", 890, 112),
];

const fn preview_targets() -> [PreviewTarget; TARGET_COUNT] {
    let mut targets = [PreviewTarget {
        label: "",
        bounds: PixelRect::new(0, 0, 0, 0),
        colour: [80, 190, 255],
    }; TARGET_COUNT];
    let mut index = 0;
    while index < TARGET_COUNT {
        let slot = PYRAMID_TARGETS[index];
        targets[index] = PreviewTarget {
            label: slot.label,
            bounds: slot.bounds,
            colour: if slot.kind.is_card() {
                [80, 190, 255]
            } else {
                [255, 192, 48]
            },
        };
        index += 1;
    }
    targets
}

pub const PREVIEW_TARGETS: [PreviewTarget; TARGET_COUNT] = preview_targets();

/// Pyramid shares the progress and transition controller with TriPeaks. Its
/// target detector is separate because overlap topology and multi-halo priority
/// differ from the TriPeaks row scanner.
pub const PROFILE: GameProfile = GameProfile {
    mode: GameMode::Pyramid,
    label: "Pyramid",
    frame_width: NOMINAL_FRAME_WIDTH,
    frame_height: NOMINAL_FRAME_HEIGHT,
    preview_targets: &PREVIEW_TARGETS,
    gameplay_scene: Some(GameplaySceneProfile {
        probe_bounds: PixelRect::new(938, 635, 12, 3),
        green_minimum: 50,
        green_red_delta_minimum: 20,
        green_blue_delta_minimum: 15,
        required_fraction_per_mille: 1_000,
    }),
    game_progress: Some(SHARED_SOLVER_PROGRESS_PROFILE),
    bottom_targets: &[],
    target_selection: TargetSelectionPolicy::FirstByPriority,
    tableau_cards: &[],
    tableau_rows: &[],
    tableau_row_count: 7,
    initial_active_rows: 0b111_1111,
    tableau_click_offset: PixelPoint::new(0, 0),
    minimum_tableau_changed_pixels: 0,
    boards_per_game: 3,
};

// All six Pyramid legend separators are outside every input hotspot. The
// Move icon also has a stable signature, but can be obscured by the guest
// cursor immediately after clicking; it must not be a required scene probe.
const WHITE_SCENE_PROBES: [PixelRect; 6] = [
    PixelRect::new(1_816, 169, 68, 2),
    PixelRect::new(1_816, 207, 68, 2),
    PixelRect::new(1_816, 245, 68, 2),
    PixelRect::new(1_816, 283, 68, 2),
    PixelRect::new(1_816, 321, 68, 2),
    PixelRect::new(1_816, 359, 68, 2),
];
const FELT_SCENE_PROBES: [PixelRect; 4] = [
    PixelRect::new(938, 635, 12, 3),
    PixelRect::new(160, 600, 16, 16),
    PixelRect::new(1_720, 600, 16, 16),
    PixelRect::new(940, 816, 16, 16),
];
const PILE_CHANGE_MINIMUM: usize = 128;
const PILE_EFFECT_BOUNDS: PixelRect = PixelRect::new(759, 678, 402, 187);

/// Fixed 2x2 probes, not a search over card artwork or a long-line fallback.
pub const fn halo_probe(slot: PyramidTargetSlot) -> PixelRect {
    match slot.kind {
        PyramidTargetKind::Move => PixelRect::new(950, 753, 2, 2),
        PyramidTargetKind::Left => PixelRect::new(789, 871, 2, 2),
        PyramidTargetKind::Right => PixelRect::new(1_052, 871, 2, 2),
        PyramidTargetKind::Card { .. } => {
            PixelRect::new(slot.bounds.x + 40, slot.bounds.y + 192, 2, 2)
        }
    }
}

fn validate_frame(frame: &CapturedFrame) -> Result<(), HaloDetectionError> {
    if frame.width != NOMINAL_FRAME_WIDTH || frame.height != NOMINAL_FRAME_HEIGHT {
        return Err(HaloDetectionError::BoundsOutsideFrame);
    }
    let required = frame
        .stride
        .checked_mul(frame.height as usize)
        .ok_or(HaloDetectionError::InvalidFrameLayout)?;
    if frame.stride < NOMINAL_FRAME_WIDTH as usize * 4 || frame.pixels.len() < required {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }
    Ok(())
}

fn count_matching(
    frame: &CapturedFrame,
    bounds: PixelRect,
    predicate: fn([u8; 3]) -> bool,
) -> Result<u32, HaloDetectionError> {
    let right = bounds
        .x
        .checked_add(bounds.width)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
    let bottom = bounds
        .y
        .checked_add(bounds.height)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
    if bounds.is_empty() {
        return Err(HaloDetectionError::EmptyBounds);
    }
    if right > frame.width || bottom > frame.height {
        return Err(HaloDetectionError::BoundsOutsideFrame);
    }
    let mut count = 0;
    for y in bounds.y..bottom {
        for x in bounds.x..right {
            let rgb = pixel_rgb(frame, x, y).ok_or(HaloDetectionError::InvalidFrameLayout)?;
            count += u32::from(predicate(rgb));
        }
    }
    Ok(count)
}

fn is_white(rgb: [u8; 3]) -> bool {
    rgb.into_iter().all(|channel| channel >= 245)
}

fn is_felt([red, green, blue]: [u8; 3]) -> bool {
    green >= 50
        && red <= 100
        && blue <= 140
        && i16::from(green) - i16::from(red) >= 20
        && i16::from(green) - i16::from(blue) >= 15
}

fn is_halo_gold([red, green, blue]: [u8; 3]) -> bool {
    let red_green = i16::from(red) - i16::from(green);
    red >= 180
        && green >= 140
        && blue <= 170
        && (15..=72).contains(&red_green)
        && i16::from(green) - i16::from(blue) >= 40
}

/// Positive scene fingerprint measured across all twenty calibration captures.
/// Missing probes or unknown overlays supply no Pyramid input authority.
pub fn is_gameplay_scene(frame: &CapturedFrame) -> Result<bool, HaloDetectionError> {
    validate_frame(frame)?;
    for bounds in WHITE_SCENE_PROBES {
        if count_matching(frame, bounds, is_white)? != bounds.width * bounds.height {
            return Ok(false);
        }
    }
    for bounds in FELT_SCENE_PROBES {
        if count_matching(frame, bounds, is_felt)? != bounds.width * bounds.height {
            return Ok(false);
        }
    }
    Ok(true)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FaceEvidence {
    Present,
    Absent,
    Blocked,
    Unknown,
}

const fn lower_face_probe(slot: PyramidTargetSlot) -> PixelRect {
    PixelRect::new(slot.bounds.x + 40, slot.bounds.y + 178, 60, 4)
}

fn face_evidence(
    frame: &CapturedFrame,
    slot: PyramidTargetSlot,
) -> Result<FaceEvidence, HaloDetectionError> {
    let bounds = lower_face_probe(slot);
    let total = bounds.width * bounds.height;
    if count_matching(frame, bounds, is_white)? * 100 >= total * 90 {
        Ok(FaceEvidence::Present)
    } else if count_matching(frame, bounds, is_felt)? * 100 >= total * 95 {
        Ok(FaceEvidence::Absent)
    } else {
        Ok(FaceEvidence::Unknown)
    }
}

/// Lower-face pixels can be interpreted only after both children are absent.
/// A top-strip white test would mistake exposed ancestor cards for this card.
fn tableau_evidence(
    frame: &CapturedFrame,
) -> Result<[FaceEvidence; TARGET_COUNT], HaloDetectionError> {
    let mut evidence = [FaceEvidence::Unknown; TARGET_COUNT];
    for (index, slot) in PYRAMID_TARGETS.iter().copied().enumerate() {
        let PyramidTargetKind::Card { row, column } = slot.kind else {
            continue;
        };
        if row < 7 {
            let left = PyramidTargetKind::Card {
                row: row + 1,
                column,
            }
            .slot_index()
            .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
            let right = PyramidTargetKind::Card {
                row: row + 1,
                column: column + 1,
            }
            .slot_index()
            .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
            if evidence[left] != FaceEvidence::Absent || evidence[right] != FaceEvidence::Absent {
                evidence[index] = FaceEvidence::Blocked;
                continue;
            }
        }
        evidence[index] = face_evidence(frame, slot)?;
    }
    Ok(evidence)
}

fn guided_action(slot: PyramidTargetSlot) -> GuidedAction {
    let probe = halo_probe(slot);
    GuidedAction {
        target: ActionTarget::Pyramid(slot.kind),
        anchor: PixelPoint::new(probe.x as i32, probe.y as i32),
        specification: ActionSpecification {
            operation: InputOperation::Click(slot.click_point),
            animation_class: AnimationClass::Fixed(Duration::from_millis(500)),
            effect_bounds: if slot.kind.is_card() {
                slot.bounds
            } else {
                PILE_EFFECT_BOUNDS
            },
            minimum_changed_pixels: PILE_CHANGE_MINIMUM,
            exclude_cursor_from_effect: true,
            repeat_target: if slot.kind.is_card() {
                RepeatTargetPolicy::MustClear
            } else {
                RepeatTargetPolicy::Allowed
            },
        },
    }
}

pub fn action_for_kind(kind: PyramidTargetKind) -> Option<GuidedAction> {
    kind.slot_index()
        .map(|index| guided_action(PYRAMID_TARGETS[index]))
}

/// Choose the first eligible halo in semantic priority order. Multiple halos
/// are valid; they never become a batch of clicks or consume an unclicked card.
pub fn analyse(
    frame: &CapturedFrame,
    state: &PyramidBoardState,
) -> Result<FrameAnalysis, HaloDetectionError> {
    if !is_gameplay_scene(frame)? {
        return Ok(FrameAnalysis {
            prediction: PredictedAction::NoHighlight,
            observed_rows: Some(RowMask::all_for(7)),
            top_row_face_up_count: 0,
        });
    }
    let evidence = tableau_evidence(frame)?;
    let mut rows = RowMask::EMPTY;
    let mut top_row_face_up_count = 0;
    for (index, slot) in PYRAMID_TARGETS.iter().copied().enumerate() {
        if let PyramidTargetKind::Card { row, .. } = slot.kind {
            if evidence[index] != FaceEvidence::Absent {
                rows.insert(row);
            }
            if row == 1 && evidence[index] == FaceEvidence::Present {
                top_row_face_up_count += 1;
            }
        }
    }
    let mut prediction = PredictedAction::NoHighlight;
    for (index, slot) in PYRAMID_TARGETS.iter().copied().enumerate() {
        let eligible = match slot.kind {
            PyramidTargetKind::Move => true,
            PyramidTargetKind::Left | PyramidTargetKind::Right => {
                face_evidence(frame, slot)? == FaceEvidence::Present
            }
            PyramidTargetKind::Card { .. } => {
                !state.clicked_target_slots[index] && evidence[index] == FaceEvidence::Present
            }
        };
        if eligible && count_matching(frame, halo_probe(slot), is_halo_gold)? == 4 {
            prediction = PredictedAction::Action(guided_action(slot));
            break;
        }
    }
    Ok(FrameAnalysis {
        prediction,
        observed_rows: (!rows.is_empty()).then_some(rows),
        top_row_face_up_count,
    })
}

pub fn board_is_empty(frame: &CapturedFrame) -> Result<bool, HaloDetectionError> {
    if !is_gameplay_scene(frame)? {
        return Ok(false);
    }
    let evidence = tableau_evidence(frame)?;
    Ok(PYRAMID_TARGETS
        .iter()
        .enumerate()
        .filter(|(_, slot)| slot.kind.is_card())
        .all(|(index, _)| evidence[index] == FaceEvidence::Absent))
}

/// Redeal evidence after the controller has already verified an empty board.
/// Unknown/blocked occupancy does not establish that a new card has arrived.
pub fn has_visible_tableau_card(frame: &CapturedFrame) -> Result<bool, HaloDetectionError> {
    if !is_gameplay_scene(frame)? {
        return Ok(false);
    }
    let evidence = tableau_evidence(frame)?;
    Ok(PYRAMID_TARGETS
        .iter()
        .enumerate()
        .filter(|(_, slot)| slot.kind.is_card())
        .any(|(index, _)| evidence[index] == FaceEvidence::Present))
}

/// A narrow pre-action check for the controller's independently recognised
/// LEVEL UP fallback when the game skips an observable empty-tableau frame.
/// This is not removal evidence and cannot authorise a transition by itself.
pub fn final_tableau_action(
    frame: &CapturedFrame,
    kind: PyramidTargetKind,
) -> Result<bool, HaloDetectionError> {
    if kind == PyramidTargetKind::Move || !is_gameplay_scene(frame)? {
        return Ok(false);
    }
    let Some(index) = kind.slot_index() else {
        return Ok(false);
    };
    let apex_index = PyramidTargetKind::Card { row: 1, column: 1 }
        .slot_index()
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
    let apex = PYRAMID_TARGETS[apex_index];
    if kind.is_card() && kind != apex.kind {
        return Ok(false);
    }
    let evidence = tableau_evidence(frame)?;
    if evidence[apex_index] != FaceEvidence::Present
        || PYRAMID_TARGETS
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.kind.is_card() && slot.kind != apex.kind)
            .any(|(index, _)| evidence[index] != FaceEvidence::Absent)
        || count_matching(frame, halo_probe(apex), is_halo_gold)? != 4
    {
        return Ok(false);
    }
    let clicked = PYRAMID_TARGETS[index];
    Ok(
        (kind.is_card() || face_evidence(frame, clicked)? == FaceEvidence::Present)
            && count_matching(frame, halo_probe(clicked), is_halo_gold)? == 4,
    )
}

/// Recognise a redeal that completed before an empty-board capture. The
/// before-frame must prove the final apex action; the after-frame must restore
/// at least two distinct bottom cards that were positively absent beforehand.
/// The controller additionally requires the shared AnotherBoard progress state.
pub fn redeal_after_final_tableau_action(
    before: &CapturedFrame,
    after: &CapturedFrame,
    kind: PyramidTargetKind,
) -> Result<bool, HaloDetectionError> {
    if !final_tableau_action(before, kind)? || !is_gameplay_scene(after)? {
        return Ok(false);
    }
    let evidence = tableau_evidence(after)?;
    Ok(PYRAMID_TARGETS
        .iter()
        .enumerate()
        .filter(|(index, slot)| {
            matches!(slot.kind, PyramidTargetKind::Card { row: 7, .. })
                && evidence[*index] == FaceEvidence::Present
        })
        .count()
        >= 2)
}

fn changed_pile_interior(
    before: &CapturedFrame,
    after: &CapturedFrame,
    pile: PyramidTargetSlot,
) -> Result<usize, HaloDetectionError> {
    // The inset removes border/halo changes. Exclude the clicked cursor region
    // explicitly; pointer movement is not evidence of a delivered game action.
    let bounds = PixelRect::new(
        pile.bounds.x + 12,
        pile.bounds.y + 12,
        pile.bounds.width - 24,
        pile.bounds.height - 24,
    );
    // The previous verified frame may retain the cursor over either pile.
    // Exclude both old and new possible pile hotspots, not just the new click.
    let exclusions = [MOVE_TARGET, LEFT_TARGET, RIGHT_TARGET]
        .map(|target| guided_action(target).effect_exclusion_bounds());
    let mut changed = 0;
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            let point = PixelPoint::new(x as i32, y as i32);
            if exclusions
                .iter()
                .any(|excluded| excluded.is_some_and(|bounds| bounds.contains(point)))
            {
                continue;
            }
            let old = pixel_rgb(before, x, y).ok_or(HaloDetectionError::InvalidFrameLayout)?;
            let new = pixel_rgb(after, x, y).ok_or(HaloDetectionError::InvalidFrameLayout)?;
            if old
                .into_iter()
                .zip(new)
                .any(|(old, new)| old.abs_diff(new) >= ACTION_CHANGE_CHANNEL_THRESHOLD)
            {
                changed += 1;
            }
        }
    }
    Ok(changed)
}

/// A card succeeds only on a positive Present -> Absent transition. Halo loss,
/// selection tint, unrelated changes, input acknowledgement and unknown pixels
/// are insufficient. Bottom targets remain repeatable after a pile effect.
pub fn verify_effect(
    before: &CapturedFrame,
    after: &CapturedFrame,
    kind: PyramidTargetKind,
) -> Result<bool, HaloDetectionError> {
    if !is_gameplay_scene(before)? || !is_gameplay_scene(after)? {
        return Ok(false);
    }
    let Some(index) = kind.slot_index() else {
        return Ok(false);
    };
    let slot = PYRAMID_TARGETS[index];
    match kind {
        PyramidTargetKind::Card { .. } => Ok(tableau_evidence(before)?[index]
            == FaceEvidence::Present
            && tableau_evidence(after)?[index] == FaceEvidence::Absent),
        PyramidTargetKind::Move => {
            for pile in [LEFT_TARGET, RIGHT_TARGET] {
                if !matches!(
                    face_evidence(before, pile)?,
                    FaceEvidence::Present | FaceEvidence::Absent
                ) || !matches!(
                    face_evidence(after, pile)?,
                    FaceEvidence::Present | FaceEvidence::Absent
                ) {
                    return Ok(false);
                }
            }
            Ok(changed_pile_interior(before, after, LEFT_TARGET)?
                + changed_pile_interior(before, after, RIGHT_TARGET)?
                >= PILE_CHANGE_MINIMUM)
        }
        PyramidTargetKind::Left | PyramidTargetKind::Right => Ok(face_evidence(before, slot)?
            == FaceEvidence::Present
            && matches!(
                face_evidence(after, slot)?,
                FaceEvidence::Present | FaceEvidence::Absent
            )
            && changed_pile_interior(before, after, slot)? >= PILE_CHANGE_MINIMUM),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        capture::PixelFormat,
        geometry::{pixel_point_to_qmp, pixel_rect_to_qmp},
        parameters::AnimationSettleDelays,
    };

    const FELT: [u8; 4] = [20, 110, 60, 255];
    const WHITE: [u8; 4] = [255, 255, 255, 255];
    const GOLD: [u8; 4] = [237, 207, 109, 255];

    fn paint(frame: &mut CapturedFrame, bounds: PixelRect, rgba: [u8; 4]) {
        for y in bounds.y..bounds.bottom() {
            for x in bounds.x..bounds.right() {
                let offset = y as usize * frame.stride + x as usize * 4;
                frame.pixels[offset..offset + 4].copy_from_slice(&rgba);
            }
        }
    }

    fn empty_board() -> CapturedFrame {
        let mut frame = CapturedFrame {
            width: NOMINAL_FRAME_WIDTH,
            height: NOMINAL_FRAME_HEIGHT,
            stride: NOMINAL_FRAME_WIDTH as usize * 4,
            format: PixelFormat::Rgba8,
            pixels: FELT.repeat((NOMINAL_FRAME_WIDTH * NOMINAL_FRAME_HEIGHT) as usize),
            cursor: None,
        };
        for bounds in WHITE_SCENE_PROBES {
            paint(&mut frame, bounds, WHITE);
        }
        frame
    }

    fn add_card(frame: &mut CapturedFrame, kind: PyramidTargetKind, highlighted: bool) {
        let slot = PYRAMID_TARGETS[kind.slot_index().expect("valid test target")];
        paint(frame, lower_face_probe(slot), WHITE);
        if highlighted {
            paint(frame, halo_probe(slot), GOLD);
        }
    }

    fn predicted_target(
        frame: &CapturedFrame,
        state: &PyramidBoardState,
    ) -> Option<PyramidTargetKind> {
        match analyse(frame, state).expect("valid fixture").prediction {
            PredictedAction::Action(GuidedAction {
                target: ActionTarget::Pyramid(kind),
                ..
            }) => Some(kind),
            PredictedAction::NoHighlight => None,
            other => panic!("unexpected prediction {other:?}"),
        }
    }

    #[test]
    fn semantic_order_covers_every_card_once_from_bottom_to_apex() {
        assert_eq!(
            PYRAMID_TARGETS[..3],
            [MOVE_TARGET, LEFT_TARGET, RIGHT_TARGET]
        );
        let mut index = 3;
        for row in (1_u8..=7).rev() {
            for column in 1..=row {
                assert_eq!(
                    PYRAMID_TARGETS[index].kind,
                    PyramidTargetKind::Card { row, column }
                );
                assert_eq!(PYRAMID_TARGETS[index].kind.slot_index(), Some(index));
                index += 1;
            }
        }
        assert_eq!(index, TARGET_COUNT);
        assert_eq!(index - 3, TABLEAU_CARD_COUNT);
    }

    #[test]
    fn every_bound_and_hit_point_is_inside_the_frame_and_qmp_mappable() {
        for target in PYRAMID_TARGETS {
            assert!(
                target.bounds.contains(target.click_point),
                "{}",
                target.label
            );
            assert!(
                pixel_rect_to_qmp(target.bounds, NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT).is_ok()
            );
            assert!(
                pixel_point_to_qmp(
                    target.click_point,
                    NOMINAL_FRAME_WIDTH,
                    NOMINAL_FRAME_HEIGHT
                )
                .is_ok()
            );
        }
    }

    #[test]
    fn tableau_hit_points_avoid_overlapping_lower_rows() {
        for target in PYRAMID_TARGETS {
            let PyramidTargetKind::Card { row, .. } = target.kind else {
                continue;
            };
            for other in PYRAMID_TARGETS {
                if let PyramidTargetKind::Card { row: other_row, .. } = other.kind {
                    if other_row > row {
                        assert!(
                            !other.bounds.contains(target.click_point),
                            "{} overlaps {}",
                            target.label,
                            other.label
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn profile_uses_separate_detector_and_shared_progress_calibration() {
        assert!(PROFILE.bottom_targets.is_empty());
        assert!(PROFILE.tableau_cards.is_empty());
        assert!(PROFILE.tableau_rows.is_empty());
        assert_eq!(PROFILE.game_progress, Some(SHARED_SOLVER_PROGRESS_PROFILE));
        assert_eq!(PREVIEW_TARGETS.len(), TARGET_COUNT);
        for (preview, target) in PREVIEW_TARGETS.iter().zip(PYRAMID_TARGETS) {
            assert_eq!(preview.bounds, target.bounds);
            assert_eq!(preview.label, target.label);
        }
    }

    #[test]
    fn invalid_semantic_targets_never_index_or_consume_history() {
        let mut state = PyramidBoardState::new();
        for kind in [
            PyramidTargetKind::Card { row: 0, column: 1 },
            PyramidTargetKind::Card { row: 8, column: 1 },
            PyramidTargetKind::Card { row: 4, column: 0 },
            PyramidTargetKind::Card { row: 4, column: 5 },
        ] {
            assert_eq!(kind.slot_index(), None);
            assert_eq!(action_for_kind(kind), None);
            state.mark_removed(kind);
        }
        assert_eq!(state, PyramidBoardState::new());
    }

    #[test]
    fn multiple_halos_use_move_left_right_then_bottom_to_apex_priority() {
        let mut frame = empty_board();
        let bottom = PyramidTargetKind::Card { row: 7, column: 1 };
        add_card(&mut frame, bottom, true);
        add_card(&mut frame, PyramidTargetKind::Right, true);
        add_card(&mut frame, PyramidTargetKind::Left, true);
        paint(&mut frame, halo_probe(MOVE_TARGET), GOLD);
        let state = PyramidBoardState::new();
        for slot in [MOVE_TARGET, LEFT_TARGET, RIGHT_TARGET] {
            assert_eq!(predicted_target(&frame, &state), Some(slot.kind));
            paint(&mut frame, halo_probe(slot), FELT);
        }
        assert_eq!(predicted_target(&frame, &state), Some(bottom));
    }

    #[test]
    fn covered_or_absent_card_artwork_cannot_authorise_a_click() {
        let mut frame = empty_board();
        let parent = PyramidTargetKind::Card { row: 6, column: 1 };
        add_card(&mut frame, parent, true);
        add_card(
            &mut frame,
            PyramidTargetKind::Card { row: 7, column: 1 },
            false,
        );
        assert_eq!(predicted_target(&frame, &PyramidBoardState::new()), None);
        let mut absent = empty_board();
        paint(&mut absent, halo_probe(PYRAMID_TARGETS[3]), GOLD);
        assert_eq!(predicted_target(&absent, &PyramidBoardState::new()), None);
    }

    #[test]
    fn consumed_history_skips_only_clicked_card_and_never_bottom_targets() {
        let mut state = PyramidBoardState::new();
        let first = PyramidTargetKind::Card { row: 7, column: 1 };
        let partner = PyramidTargetKind::Card { row: 7, column: 2 };
        state.mark_removed(first);
        for kind in [
            PyramidTargetKind::Move,
            PyramidTargetKind::Left,
            PyramidTargetKind::Right,
        ] {
            state.mark_removed(kind);
        }
        assert_eq!(
            state
                .clicked_target_slots
                .iter()
                .filter(|used| **used)
                .count(),
            1
        );
        let mut frame = empty_board();
        add_card(&mut frame, first, true);
        add_card(&mut frame, partner, true);
        assert_eq!(predicted_target(&frame, &state), Some(partner));
        paint(&mut frame, halo_probe(MOVE_TARGET), GOLD);
        assert_eq!(
            predicted_target(&frame, &state),
            Some(PyramidTargetKind::Move)
        );
        assert_eq!(
            PyramidBoardState::new().clicked_target_slots,
            [false; TARGET_COUNT]
        );
    }

    #[test]
    fn halo_loss_selection_tint_and_unrelated_change_are_not_card_removal() {
        let mut before = empty_board();
        let kind = PyramidTargetKind::Card { row: 7, column: 3 };
        let slot = PYRAMID_TARGETS[kind.slot_index().unwrap()];
        add_card(&mut before, kind, true);
        let mut after = before.clone();
        paint(&mut after, halo_probe(slot), FELT);
        paint(&mut after, PixelRect::new(50, 50, 100, 100), WHITE);
        assert!(!verify_effect(&before, &after, kind).unwrap());
        paint(&mut after, lower_face_probe(slot), [220, 220, 220, 255]);
        assert!(!verify_effect(&before, &after, kind).unwrap());
        assert!(!board_is_empty(&after).unwrap());
        assert!(
            analyse(&after, &PyramidBoardState::new())
                .unwrap()
                .observed_rows
                .is_some()
        );
    }

    #[test]
    fn king_and_pair_removal_require_positive_lower_face_to_felt_transition() {
        let mut before = empty_board();
        let king = PyramidTargetKind::Card { row: 7, column: 3 };
        add_card(&mut before, king, true);
        let mut after = empty_board();
        // An ancestor can supply white at the removed card's old top strip.
        let slot = PYRAMID_TARGETS[king.slot_index().unwrap()];
        paint(
            &mut after,
            PixelRect::new(slot.bounds.x + 40, slot.bounds.y + 10, 60, 4),
            WHITE,
        );
        assert!(verify_effect(&before, &after, king).unwrap());
        let partner = PyramidTargetKind::Card { row: 7, column: 4 };
        add_card(&mut before, partner, true);
        assert!(verify_effect(&before, &after, king).unwrap());
        assert!(verify_effect(&before, &after, partner).unwrap());
    }

    #[test]
    fn empty_board_requires_every_slot_positively_absent_in_a_known_scene() {
        let frame = empty_board();
        assert!(board_is_empty(&frame).unwrap());
        assert_eq!(
            analyse(&frame, &PyramidBoardState::new())
                .unwrap()
                .observed_rows,
            None
        );
        let mut unknown = frame.clone();
        paint(
            &mut unknown,
            lower_face_probe(PYRAMID_TARGETS[3]),
            [0, 0, 0, 255],
        );
        assert!(!board_is_empty(&unknown).unwrap());
        let mut modal = frame;
        paint(&mut modal, WHITE_SCENE_PROBES[0], [0, 0, 0, 255]);
        assert!(!is_gameplay_scene(&modal).unwrap());
        assert!(!board_is_empty(&modal).unwrap());
        assert!(
            analyse(&modal, &PyramidBoardState::new())
                .unwrap()
                .observed_rows
                .is_some()
        );
    }

    #[test]
    fn fixed_probe_requires_all_four_gold_pixels_without_searching_nearby() {
        let mut frame = empty_board();
        let probe = halo_probe(MOVE_TARGET);
        paint(&mut frame, PixelRect::new(probe.x + 2, probe.y, 2, 2), GOLD);
        assert_eq!(predicted_target(&frame, &PyramidBoardState::new()), None);
        paint(&mut frame, probe, GOLD);
        paint(&mut frame, PixelRect::new(probe.x, probe.y, 1, 1), FELT);
        assert_eq!(predicted_target(&frame, &PyramidBoardState::new()), None);
    }

    #[test]
    fn bottom_verification_ignores_halo_and_cursor_but_accepts_pile_material_change() {
        let before = empty_board();
        let mut after = before.clone();
        paint(&mut after, halo_probe(LEFT_TARGET), GOLD);
        paint(&mut after, PixelRect::new(960, 758, 4, 10), WHITE);
        assert!(!verify_effect(&before, &after, PyramidTargetKind::Move).unwrap());
        paint(&mut after, PixelRect::new(775, 695, 20, 20), WHITE);
        assert!(verify_effect(&before, &after, PyramidTargetKind::Move).unwrap());

        let mut left_before = empty_board();
        add_card(&mut left_before, PyramidTargetKind::Left, true);
        let mut cursor_only = left_before.clone();
        paint(&mut cursor_only, PixelRect::new(820, 760, 20, 20), WHITE);
        assert!(!verify_effect(&left_before, &cursor_only, PyramidTargetKind::Left).unwrap());

        let mut old_cursor = before.clone();
        paint(&mut old_cursor, PixelRect::new(820, 760, 20, 20), WHITE);
        assert!(!verify_effect(&old_cursor, &before, PyramidTargetKind::Move).unwrap());

        let mut tinted = left_before.clone();
        paint(&mut tinted, LEFT_TARGET.bounds, [220, 220, 220, 255]);
        assert!(!verify_effect(&left_before, &tinted, PyramidTargetKind::Left).unwrap());
    }

    #[test]
    fn every_action_is_one_mouse_click_with_fixed_settle_and_typed_repeat_policy() {
        for slot in PYRAMID_TARGETS {
            let action = action_for_kind(slot.kind).unwrap();
            assert_eq!(action.operation(), InputOperation::Click(slot.click_point));
            assert_eq!(
                action.animation_settle_delay(AnimationSettleDelays::from_millis(1, 2, 3)),
                Duration::from_millis(500)
            );
            assert_eq!(
                action.repeat_target_policy(),
                if slot.kind.is_card() {
                    RepeatTargetPolicy::MustClear
                } else {
                    RepeatTargetPolicy::Allowed
                }
            );
        }
    }

    #[test]
    fn malformed_dimensions_stride_and_buffer_are_rejected_without_sampling() {
        let mut frame = empty_board();
        frame.width = 1_919;
        assert_eq!(
            is_gameplay_scene(&frame),
            Err(HaloDetectionError::BoundsOutsideFrame)
        );
        frame.width = NOMINAL_FRAME_WIDTH;
        frame.stride = usize::MAX;
        assert_eq!(
            is_gameplay_scene(&frame),
            Err(HaloDetectionError::InvalidFrameLayout)
        );
        frame.stride = NOMINAL_FRAME_WIDTH as usize * 4;
        frame.pixels.truncate(32);
        assert_eq!(
            is_gameplay_scene(&frame),
            Err(HaloDetectionError::InvalidFrameLayout)
        );
    }

    #[test]
    fn rgba_bgra_and_padded_stride_have_identical_detection() {
        let mut rgba = empty_board();
        add_card(&mut rgba, PyramidTargetKind::Right, true);
        let mut bgra = rgba.clone();
        for pixel in bgra.pixels.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        bgra.format = PixelFormat::Bgra8;
        let mut padded = rgba.clone();
        padded.stride += 16;
        padded.pixels = vec![0; padded.stride * padded.height as usize];
        for y in 0..rgba.height as usize {
            padded.pixels[y * padded.stride..y * padded.stride + rgba.stride]
                .copy_from_slice(&rgba.pixels[y * rgba.stride..(y + 1) * rgba.stride]);
        }
        let state = PyramidBoardState::new();
        assert_eq!(
            predicted_target(&rgba, &state),
            Some(PyramidTargetKind::Right)
        );
        assert_eq!(analyse(&rgba, &state), analyse(&bgra, &state));
        assert_eq!(analyse(&rgba, &state), analyse(&padded, &state));
    }

    #[test]
    fn final_transition_precondition_requires_only_apex_and_a_valid_clicked_halo() {
        let mut frame = empty_board();
        let apex = PyramidTargetKind::Card { row: 1, column: 1 };
        add_card(&mut frame, apex, true);
        assert!(final_tableau_action(&frame, apex).unwrap());
        assert!(!final_tableau_action(&frame, PyramidTargetKind::Move).unwrap());
        assert!(!final_tableau_action(&frame, PyramidTargetKind::Left).unwrap());
        add_card(&mut frame, PyramidTargetKind::Left, true);
        assert!(final_tableau_action(&frame, PyramidTargetKind::Left).unwrap());
        add_card(
            &mut frame,
            PyramidTargetKind::Card { row: 7, column: 7 },
            false,
        );
        assert!(!final_tableau_action(&frame, apex).unwrap());
    }

    #[test]
    fn redeal_requires_proven_final_action_and_multiple_restored_bottom_cards() {
        let mut before = empty_board();
        let apex = PyramidTargetKind::Card { row: 1, column: 1 };
        add_card(&mut before, apex, true);
        let mut after = empty_board();
        assert!(!has_visible_tableau_card(&after).unwrap());
        add_card(
            &mut after,
            PyramidTargetKind::Card { row: 7, column: 1 },
            false,
        );
        assert!(has_visible_tableau_card(&after).unwrap());
        assert!(!redeal_after_final_tableau_action(&before, &after, apex).unwrap());
        add_card(
            &mut after,
            PyramidTargetKind::Card { row: 7, column: 7 },
            false,
        );
        assert!(redeal_after_final_tableau_action(&before, &after, apex).unwrap());
        assert!(
            !redeal_after_final_tableau_action(&before, &after, PyramidTargetKind::Move).unwrap()
        );
        paint(
            &mut before,
            halo_probe(PYRAMID_TARGETS[TARGET_COUNT - 1]),
            FELT,
        );
        assert!(!redeal_after_final_tableau_action(&before, &after, apex).unwrap());
    }

    #[test]
    fn calibrated_screenshot_patches_select_the_recorded_targets() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/pyramid-probes.json"))
                .expect("valid measured screenshot fixture");
        assert_eq!(fixture["schema"], 1);
        let sources = fixture["sources"].as_array().expect("source captures");
        assert_eq!(sources.len(), 20);
        for source in sources {
            let name = source["name"].as_str().unwrap();
            assert_eq!(source["sha256"].as_str().unwrap().len(), 64);
            let mut frame = empty_board();
            frame.pixels.fill(0);
            for patch in source["patches"].as_array().unwrap() {
                let bounds = patch["bounds"].as_array().unwrap();
                let bounds = PixelRect::new(
                    bounds[0].as_u64().unwrap() as u32,
                    bounds[1].as_u64().unwrap() as u32,
                    bounds[2].as_u64().unwrap() as u32,
                    bounds[3].as_u64().unwrap() as u32,
                );
                let hex = patch["rgb_hex"].as_str().unwrap();
                assert_eq!(hex.len(), (bounds.width * bounds.height * 6) as usize);
                let mut offset = 0;
                for y in bounds.y..bounds.bottom() {
                    for x in bounds.x..bounds.right() {
                        let rgb = [0, 2, 4].map(|channel| {
                            u8::from_str_radix(&hex[offset + channel..offset + channel + 2], 16)
                                .unwrap()
                        });
                        paint(
                            &mut frame,
                            PixelRect::new(x, y, 1, 1),
                            [rgb[0], rgb[1], rgb[2], 255],
                        );
                        offset += 6;
                    }
                }
            }
            assert!(is_gameplay_scene(&frame).unwrap(), "{name}: known scene");
            let expected = source["expected_slot"]
                .as_u64()
                .map(|index| PYRAMID_TARGETS[index as usize].kind);
            assert_eq!(
                predicted_target(&frame, &PyramidBoardState::new()),
                expected,
                "{name}: recorded halo priority"
            );
            assert!(!board_is_empty(&frame).unwrap(), "{name}: occupied board");
        }
    }
}
