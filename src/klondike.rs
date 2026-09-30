//! Klondike Draw 1 Solver targets and conservative, mode-owned effect evidence.
//!
//! Geometry is measured from Charlie's 1920x1080 captures K01-K19. A source
//! needs a continuous lower gold edge, matching exterior rails and a bright
//! card interior. Dashed dark destinations never grant click authority. The
//! outline primitive groups a connected run into one target. No card ranks,
//! legal moves, game completion or restart behaviour are inferred here.

use std::fmt;

use crate::{
    capture::CapturedFrame,
    detector::{HaloDetectionError, pixel_rgb},
    game::{
        ActionSpecification, ActionTarget, AnimationClass, GameProfile, GuidedAction,
        InputOperation, PreviewTarget, RepeatTargetPolicy, TargetSelectionPolicy,
    },
    geometry::{PixelPoint, PixelRect},
    tracker::{FrameAnalysis, PredictedAction},
};


/// Measured card-face width; changing fan overlap does not change card width.
const CARD_WIDTH: u32 = 132;


/// Measured card-face height, without the exterior source highlight.
const CARD_HEIGHT: u32 = 175;


/// First column's card-face X origin in the calibrated guest frame.
const FIRST_COLUMN_X: u32 = 390;


/// Separation between adjacent tableau/foundation columns in guest pixels.
const COLUMN_PITCH: u32 = 168;


/// Top card-face row shared by stock, waste and foundations.
const UPPER_Y: u32 = 112;


/// Initial tableau origin; all later cards remain below this row.
const TABLEAU_Y: u32 = 342;


/// Exclusive destination-effect bound retained above all guest toolbar pixels.
const TABLEAU_EFFECT_BOTTOM: u32 = 936;


/// First toolbar-dimmed card row measured in K19; source proof excludes this row.
const TOOLBAR_TOP: u32 = 947;


/// Exclusive outline scan limit covering K19's closed source border under the
/// translucent toolbar. This extends recognition, never the input/proof region.
const TABLEAU_OUTLINE_BOTTOM: u32 = 963;


/// Card fan advances by this many pixels, capped at two advances.
const WASTE_FAN_STEP: u32 = 28;


/// Full waste face envelope, including the two exposed older-card strips.
const WASTE_BOUNDS: PixelRect = PixelRect::new(558, UPPER_Y, 188, CARD_HEIGHT);


/// Material content changes required independently in source/destination ROIs.
const MINIMUM_CONTENT_CHANGE: usize = 512;


/// A click on this measured toolbar control requests a fresh Solver recommendation.
/// Only the worker may authorise its bounded, scene-validated recovery use.
pub const SOLVER_CLICK: PixelPoint = PixelPoint::new(600, 990);


/// The separate completion-request button measured in K13; not the Solver icon.
const SOLVE_BOUNDS: PixelRect = PixelRect::new(562, 143, 124, 113);


/// Original RGB8 samples on a four-pixel lattice over the entire Solve button.
/// The fixture manifest records provenance; no image is decoded during detection.
const SOLVE_TEMPLATE: &[u8; 2_697] = include_bytes!("klondike-solve-control.rgb");


/// Required paper/ink changes in each of two opposed RIGHT-card corner patches.
/// Both corners must retain white paper; a dark guide cannot prove replacement.
const MINIMUM_CORNER_CHANGE: usize = 48;


/// Geometry-bearing target identity; no persistent card-rank state is implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KlondikeTarget {
    /// Highlighted non-empty stock, delivered with Charlie's confirmed D shortcut.
    Draw,
    /// Highlighted empty stock, delivered as one click on the recycle area.
    Recycle,
    /// Evidenced Solve control; one click requests completion and requires review.
    Solve,
    /// Highlighted top waste card; offset is zero, one or two fan advances.
    Waste {
        /// Number of 28-pixel advances from the first waste-card origin.
        offset: u8,
    },
    /// One connected highlighted tableau block, including a complete card run.
    Tableau {
        /// One-based column, from left 1 through right 7.
        column: u8,
        /// First row where both exterior source rails are present.
        top: u16,
        /// Exclusive lower row after the source's continuous gold outline.
        bottom: u16,
    },
    /// One highlighted card returning from a foundation to the tableau.
    Foundation {
        /// One-based foundation position from left 1 through right 4.
        column: u8,
    },
}


impl fmt::Display for KlondikeTarget {


    /// Describe target geometry without pretending to know a card's rank or suit.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {


        match self {
            Self::Draw => formatter.write_str("Draw 1"),
            Self::Recycle => formatter.write_str("stock recycle"),
            Self::Solve => formatter.write_str("Solve control (result requires review)"),
            Self::Waste { offset } => write!(formatter, "RIGHT (fan offset {offset})"),
            Self::Tableau { column, top, bottom } => {
                write!(formatter, "tableau column {column}, rows {top}..{bottom}")
            }
            Self::Foundation { column } => write!(formatter, "SUIT {column}"),
        }
    }
}


/// Preview envelopes only; actual tableau clicks derive from fresh source bounds.
const PREVIEW_TARGETS: [PreviewTarget; 3] = [
    PreviewTarget {
        label: "KL DRAW / recycle",
        bounds: PixelRect::new(390, 112, CARD_WIDTH, CARD_HEIGHT),
        colour: [255, 192, 48],
    },
    PreviewTarget {
        label: "KL RIGHT / Solve",
        bounds: WASTE_BOUNDS,
        colour: [255, 192, 48],
    },
    PreviewTarget {
        label: "KL tableau: dynamic source blocks",
        bounds: PixelRect::new(390, TABLEAU_Y, 1_140, TABLEAU_OUTLINE_BOTTOM - TABLEAU_Y),
        colour: [80, 190, 255],
    },
];


/// Klondike owns its dynamic detector and has no established completion detector.
/// The single logical row is an unused shared-interface hint, not a card layout.
pub const PROFILE: GameProfile = GameProfile {
    label: "Klondike",
    frame_width: 1_920,
    frame_height: 1_080,
    preview_targets: &PREVIEW_TARGETS,
    gameplay_scene: None,
    game_progress: None,
    bottom_targets: &[],
    target_selection: TargetSelectionPolicy::FirstByPriority,
    tableau_cards: &[],
    tableau_rows: &[],
    tableau_row_count: 1,
    initial_active_rows: 1,
    tableau_click_offset: PixelPoint::new(0, 0),
    minimum_tableau_changed_pixels: MINIMUM_CONTENT_CHANGE,
    boards_per_game: 1,
};


/// Reject malformed storage and every frame outside the evidenced native geometry.
fn validate_frame(frame: &CapturedFrame) -> Result<(), HaloDetectionError> {


    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }


    if frame.width != PROFILE.frame_width || frame.height != PROFILE.frame_height {
        return Err(HaloDetectionError::BoundsOutsideFrame);
    }

    Ok(())
}


/// Check half-open probe bounds before any coordinate iteration or subtraction.
fn validate_bounds(frame: &CapturedFrame, bounds: PixelRect) -> Result<(), HaloDetectionError> {


    if bounds.width == 0 || bounds.height == 0 {
        return Err(HaloDetectionError::EmptyBounds);
    }

    let right = bounds.x.checked_add(bounds.width);
    let bottom = bounds.y.checked_add(bounds.height);


    if right.is_none_or(|value| value > frame.width)
        || bottom.is_none_or(|value| value > frame.height)
    {
        return Err(HaloDetectionError::BoundsOutsideFrame);
    }

    Ok(())
}


/// Bright continuous source edge; ordinary green felt cannot satisfy this relation.
fn is_edge_gold([red, green, blue]: [u8; 3]) -> bool {
    red >= 165 && green >= 115 && blue <= 205
        && i16::from(red) - i16::from(green) >= 7
        && i16::from(green) - i16::from(blue) >= 28
}


/// Darker gradient on the source side rails; used only outside a known card width.
fn is_rail_gold([red, green, blue]: [u8; 3]) -> bool {
    red >= 100 && green >= 70 && blue <= 210
        && i16::from(red) - i16::from(green) >= 7
        && i16::from(green) - i16::from(blue) >= 20
}


/// White card paper, distinct from the black/translucent destination guide.
fn is_white([red, green, blue]: [u8; 3]) -> bool {
    red >= 210 && green >= 210 && blue >= 210
}


/// The blue card-back design shown in all evidenced non-empty stock captures.
fn is_back([red, green, blue]: [u8; 3]) -> bool {
    blue >= 100 && i16::from(blue) - i16::from(red) >= 40
        && i16::from(blue) - i16::from(green) >= 15
}


/// Undimmed green felt or the green recycle glyph; neither is blue card artwork.
fn is_felt([red, green, blue]: [u8; 3]) -> bool {
    green >= 65 && i16::from(green) - i16::from(red) >= 25
        && i16::from(green) - i16::from(blue) >= 20
}


/// Recognise an occupied or outlined slot's measured horizontal top strip.
fn is_slot_edge(rgb: [u8; 3]) -> bool {
    let [red, green, blue] = rgb;
    (red >= 190 && green >= 190 && blue >= 190)
        || (blue >= 105 && i16::from(blue) - i16::from(red) >= 30)
        || (red >= 40 && green >= 145
            && i16::from(green) - i16::from(red) >= 40
            && i16::from(green) - i16::from(blue) >= 25)
}


/// Count qualifying pixels inside previously validated small immutable probes.
fn count_pixels(frame: &CapturedFrame, bounds: PixelRect, predicate: fn([u8; 3]) -> bool) -> u32 {
    let mut count = 0;


    for y in bounds.y..bounds.y + bounds.height {


        for x in bounds.x..bounds.x + bounds.width {


            if pixel_rgb(frame, x, y).is_some_and(predicate) {
                count += 1;
            }
        }
    }

    count
}


/// Test a fractional count with widened arithmetic and an inclusive threshold.
fn fraction_at_least(
    frame: &CapturedFrame,
    bounds: PixelRect,
    predicate: fn([u8; 3]) -> bool,
    per_mille: u32,
) -> bool {
    u64::from(count_pixels(frame, bounds, predicate)) * 1_000
        >= u64::from(bounds.width) * u64::from(bounds.height) * u64::from(per_mille)
}


/// Require the measured seven-column/four-foundation layout and open felt gutters.
/// Solver may be off: this supports the worker's explicitly authorised activation.
/// Sparse/unknown endgames and overlays fail closed; no completion is inferred.
pub fn is_gameplay_scene(frame: &CapturedFrame) -> Result<bool, HaloDetectionError> {
    validate_frame(frame)?;
    let gutters = [
        PixelRect::new(160, 310, 16, 16),
        PixelRect::new(1_730, 310, 16, 16),
        PixelRect::new(868, 710, 14, 14),
        PixelRect::new(1_202, 840, 14, 14),
    ];


    if !gutters.into_iter().all(|bounds| fraction_at_least(frame, bounds, is_felt, 950)) {
        return Ok(false);
    }

    let tableau_edges = (0..7).filter(|column| {
        fraction_at_least(
            frame,
            PixelRect::new(FIRST_COLUMN_X + COLUMN_PITCH * column + 20, TABLEAU_Y, 92, 6),
            is_slot_edge,
            700,
        )
    }).count();
    let foundation_edges = (0..4).filter(|column| {
        fraction_at_least(
            frame,
            PixelRect::new(894 + COLUMN_PITCH * column + 20, UPPER_Y, 92, 6),
            is_slot_edge,
            700,
        )
    }).count();

    Ok(tableau_edges >= 5 && foundation_edges >= 3)
}


/// A source halo is actionable only while the measured Solver banner is present.
fn has_solver_banner(frame: &CapturedFrame) -> bool {
    [36, 85].into_iter().all(|y| {
        fraction_at_least(frame, PixelRect::new(828, y, 263, 1), is_rail_gold, 950)
    })
}


/// Match K13's control shape, check mark, lettering and colours at native scale.
/// Every four-pixel sample is compared; require 98% within 24 RGB levels both
/// overall and separately in the glyph area. Empty stock is a separate guard.
fn has_solve_control(frame: &CapturedFrame) -> bool {
    let stock_inside = PixelRect::new(406, 130, 100, 139);


    if !fraction_at_least(frame, stock_inside, is_felt, 900) {
        return false;
    }

    let mut matched = 0_u32;
    let mut glyph_matched = 0_u32;
    let mut glyph_samples = 0_u32;
    let mut sample = 0;


    for y in (SOLVE_BOUNDS.y..SOLVE_BOUNDS.y + SOLVE_BOUNDS.height).step_by(4) {


        for x in (SOLVE_BOUNDS.x..SOLVE_BOUNDS.x + SOLVE_BOUNDS.width).step_by(4) {
            let expected = &SOLVE_TEMPLATE[sample..sample + 3];
            sample += 3;
            let agrees = pixel_rgb(frame, x, y).is_some_and(|actual| {
                actual.into_iter().zip(expected).all(|(a, b)| a.abs_diff(*b) <= 24)
            });
            matched += u32::from(agrees);


            if (590..660).contains(&x) && (160..236).contains(&y) {
                glyph_samples += 1;
                glyph_matched += u32::from(agrees);
            }
        }
    }

    matched * 100 >= 899 * 98 && glyph_matched * 100 >= glyph_samples * 98
}


/// Find a bright source card/run inside a dynamic vertical scan envelope.
///
/// This reusable primitive starts at the bottom, requires a 96-pixel continuous
/// edge for the 132-pixel cards, follows both exterior rails, and probes white
/// paper above the bottom. A black destination interior fails even if some gold
/// dashes align. Card-rank artwork is not required to be white at one exact pixel.
/// The returned rectangle groups all connected highlighted cards as one action.
/// An interior crossbar cannot close a block while its exterior rails continue.
/// K19 permits a darker closed edge only in the measured toolbar overlap strip.
pub fn find_solid_card_source(
    frame: &CapturedFrame,
    scan: PixelRect,
) -> Result<Option<PixelRect>, HaloDetectionError> {
    find_solid_outline(frame, scan, true)
}


/// Shared outline geometry; stock uses its own positive back/recycle interior test.
fn find_solid_outline(
    frame: &CapturedFrame,
    scan: PixelRect,
    require_card_face: bool,
) -> Result<Option<PixelRect>, HaloDetectionError> {
    validate_frame(frame)?;
    validate_bounds(frame, scan)?;


    if scan.width < 96 || scan.x < 10 || scan.x + scan.width + 10 > frame.width {
        return Err(HaloDetectionError::BoundsOutsideFrame);
    }

    let x = scan.x;
    let right = x + scan.width;
    let bottom = scan.y + scan.height;
    let paired_rails = |row| {
        (x - 9..x).any(|xx| pixel_rgb(frame, xx, row).is_some_and(is_rail_gold))
            && (right..right + 10).any(|xx| pixel_rgb(frame, xx, row).is_some_and(is_rail_gold))
    };


    // The last scanned row must show both rails ending. Otherwise an internal
    // crossbar near a clipped scan boundary could masquerade as the lower edge.
    // K19's right rail ends at y958; its first wholly clear row is y959.
    if (x - 9..x).any(|xx| pixel_rgb(frame, xx, bottom - 1).is_some_and(is_rail_gold))
        || (right..right + 10).any(|xx| pixel_rgb(frame, xx, bottom - 1).is_some_and(is_rail_gold))
    {
        return Ok(None);
    }

    let last_rail_row = (scan.y..bottom).rev().find(|&row| paired_rails(row));


    for y in (scan.y..bottom).rev() {
        let lower_edge = |rgb| is_edge_gold(rgb)
            || (require_card_face && (TOOLBAR_TOP..TABLEAU_OUTLINE_BOTTOM).contains(&y)
                && is_rail_gold(rgb));


        if y < scan.y + 170
            || !pixel_rgb(frame, x + scan.width / 2, y).is_some_and(lower_edge)
            || last_rail_row.is_some_and(|row| row >= y + 10)
        {
            continue;
        }


        if !(x + 18..right - 18).all(|xx| pixel_rgb(frame, xx, y).is_some_and(lower_edge)) {
            continue;
        }

        let mut top = y;
        let mut gap = 0;
        let mut matched_rows = 0;


        for row in (scan.y..y - 9).rev() {


            if paired_rails(row) {
                top = row;
                gap = 0;
                matched_rows += 1;
            } else {
                gap += 1;


                if gap > 10 {
                    break;
                }
            }
        }

        let height = y + 1 - top;


        if !(180..=600).contains(&height) || top == scan.y
            || matched_rows * 100 < (height - 10) * 85 {
            continue;
        }

        // A clipped rail segment or an interruption cannot invent a new source top.
        let closed_top = (top.saturating_sub(10).max(scan.y)..=top).any(|row| {
            (x + 18..right - 18).all(|xx| pixel_rgb(frame, xx, row).is_some_and(is_edge_gold))
        });


        if !closed_top {
            continue;
        }

        let face_probe = PixelRect::new(x + 18, y - 36, scan.width - 36, 12);


        if require_card_face && !fraction_at_least(frame, face_probe, is_white, 350) {
            continue;
        }

        return Ok(Some(PixelRect::new(x, top, scan.width, height)));
    }

    Ok(None)
}


/// Build the immutable operation for a valid target; reject forged dynamic geometry.
pub fn canonical_action(target: KlondikeTarget) -> Option<GuidedAction> {


    let (operation, bounds) = match target {
        KlondikeTarget::Draw => (InputOperation::PressDrawKey, WASTE_BOUNDS),
        KlondikeTarget::Recycle => (
            InputOperation::Click(PixelPoint::new(456, 199)),
            PixelRect::new(390, UPPER_Y, 356, CARD_HEIGHT),
        ),
        KlondikeTarget::Solve => (
            InputOperation::Click(PixelPoint::new(624, 199)),
            SOLVE_BOUNDS,
        ),
        KlondikeTarget::Waste { offset } if offset <= 2 => {
            let x = 558 + WASTE_FAN_STEP * u32::from(offset);
            (
                InputOperation::Click(PixelPoint::new((x + CARD_WIDTH / 2) as i32, 199)),
                PixelRect::new(x, UPPER_Y, CARD_WIDTH, CARD_HEIGHT),
            )
        }
        KlondikeTarget::Tableau { column, top, bottom }
            if (1..=7).contains(&column) && top >= 332
                && u32::from(bottom) <= TABLEAU_OUTLINE_BOTTOM
                && u32::from(top) + 40 < TOOLBAR_TOP
                && bottom.checked_sub(top).is_some_and(|height| (180..=600).contains(&height)) =>
        {
            let x = FIRST_COLUMN_X + COLUMN_PITCH * u32::from(column - 1);
            (
                InputOperation::Click(PixelPoint::new((x + CARD_WIDTH / 2) as i32, i32::from(top) + 40)),
                PixelRect::new(x, u32::from(top), CARD_WIDTH, u32::from(bottom).min(TOOLBAR_TOP) - u32::from(top)),
            )
        }
        KlondikeTarget::Foundation { column } if (1..=4).contains(&column) => {
            let x = 894 + COLUMN_PITCH * u32::from(column - 1);
            (
                InputOperation::Click(PixelPoint::new((x + CARD_WIDTH / 2) as i32, 199)),
                PixelRect::new(x, UPPER_Y, CARD_WIDTH, CARD_HEIGHT),
            )
        }
        _ => return None,
    };


    let anchor = match operation {
        InputOperation::Click(point) => point,
        InputOperation::PressDrawKey => PixelPoint::new(456, 295),
    };

    Some(GuidedAction {
        target: ActionTarget::Klondike(target),
        anchor,
        specification: ActionSpecification {
            operation,
            animation_class: AnimationClass::Klondike,
            effect_bounds: bounds,
            minimum_changed_pixels: MINIMUM_CONTENT_CHANGE,
            exclude_cursor_from_effect: true,
            repeat_target: RepeatTargetPolicy::Allowed,
        },
    })
}


/// Select DRAW, Solve/RIGHT, the lowest tableau source, then SUIT, from one frame.
/// Source run grouping never treats each outlined card as a separate action.
pub fn analyse(frame: &CapturedFrame) -> Result<FrameAnalysis, HaloDetectionError> {


    let target = if is_gameplay_scene(frame)? {
        select_target(frame)?
    } else {
        None
    };
    let prediction = target.and_then(canonical_action)
        .map_or(PredictedAction::NoHighlight, PredictedAction::Action);

    Ok(FrameAnalysis { prediction, observed_rows: None, top_row_face_up_count: 0 })
}


/// Apply the approved priority without granting authority on an unclassified stock.
fn select_target(frame: &CapturedFrame) -> Result<Option<KlondikeTarget>, HaloDetectionError> {
    let upper_scan = |x| PixelRect::new(x, 100, CARD_WIDTH, 204);
    let solver_active = has_solver_banner(frame);


    if solver_active && find_solid_outline(frame, upper_scan(390), false)?.is_some() {
        let inside = PixelRect::new(406, 130, 100, 139);


        if fraction_at_least(frame, inside, is_back, 500) {
            return Ok(Some(KlondikeTarget::Draw));
        }


        if fraction_at_least(frame, inside, is_felt, 900) {
            return Ok(Some(KlondikeTarget::Recycle));
        }

        return Err(HaloDetectionError::MissingProfileCalibration {
            target: "Klondike highlighted stock interior",
        });
    }


    if has_solve_control(frame) {
        return Ok(Some(KlondikeTarget::Solve));
    }


    if !solver_active {
        return Ok(None);
    }


    for offset in (0..=2_u8).rev() {


        if find_solid_card_source(frame, upper_scan(558 + WASTE_FAN_STEP * u32::from(offset)))?.is_some() {
            return Ok(Some(KlondikeTarget::Waste { offset }));
        }
    }

    let mut lowest = None;


    for column in 1..=7_u8 {
        let x = FIRST_COLUMN_X + COLUMN_PITCH * u32::from(column - 1);
        let scan = PixelRect::new(x, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332);


        if let Some(bounds) = find_solid_card_source(frame, scan)? {
            let target = KlondikeTarget::Tableau {
                column,
                top: bounds.y as u16,
                bottom: (bounds.y + bounds.height) as u16,
            };


            if lowest.is_none_or(|(_, prior_bottom)| bounds.y + bounds.height > prior_bottom) {
                lowest = Some((target, bounds.y + bounds.height));
            }
        }
    }


    if let Some((target, _)) = lowest {
        return Ok(Some(target));
    }


    for column in 1..=4_u8 {
        let x = 894 + COLUMN_PITCH * u32::from(column - 1);


        if find_solid_card_source(frame, upper_scan(x))?.is_some() {
            return Ok(Some(KlondikeTarget::Foundation { column }));
        }
    }

    Ok(None)
}


/// Count inner content changes, excluding card edges, gold and a pointer-sized
/// neighbourhood of the commanded click. A halo change alone has zero authority.
fn content_changes(
    before: &CapturedFrame,
    after: &CapturedFrame,
    bounds: PixelRect,
    action: GuidedAction,
    require_positive_replacement: bool,
) -> usize {
    let mut count = 0;


    let click = match action.operation() {
        InputOperation::Click(point) => Some(point),
        InputOperation::PressDrawKey => None,
    };


    for y in bounds.y + 16..bounds.y + bounds.height - 16 {


        for x in bounds.x + 16..bounds.x + bounds.width - 16 {


            if click.is_some_and(|point| {
                (i64::from(x) - i64::from(point.x)).abs() < 48
                    && (i64::from(y) - i64::from(point.y)).abs() < 48
            }) {
                continue;
            }


            let Some(first) = pixel_rgb(before, x, y) else { continue; };


            let Some(second) = pixel_rgb(after, x, y) else { continue; };


            if is_rail_gold(first) || is_rail_gold(second) {
                continue;
            }


            let replacement = (is_felt(second) && !is_felt(first))
                || (is_white(second) && !is_white(first));


            if (!require_positive_replacement || replacement)
                && first.into_iter().zip(second).any(|(a, b)| a.abs_diff(b) >= 48)
            {
                count += 1;
            }
        }
    }

    count
}


/// Black or red printed detail; this does not identify a card rank or suit.
fn is_card_ink([red, green, blue]: [u8; 3]) -> bool {
    (red <= 180 && green <= 180 && blue <= 180)
        || (red >= 140 && green <= 130 && blue <= 130)
}


/// Evidence that a RIGHT card was replaced by another card at the same fan slot.
/// Opposed corner probes include detail lost by the ordinary 16-pixel inset.
/// Both sides must retain white paper before and after; guide dimming fails.
/// Excluding the commanded cursor area and requiring both corners rejects a
/// pointer-only change. The separate destination change is still mandatory.
fn waste_identity_changes(
    before: &CapturedFrame,
    after: &CapturedFrame,
    source: PixelRect,
    action: GuidedAction,
) -> [usize; 2] {
    let patches = [
        PixelRect::new(source.x + 5, UPPER_Y + 5, 28, 44),
        PixelRect::new(source.x + 99, UPPER_Y + 126, 28, 44),
    ];


    let InputOperation::Click(point) = action.operation() else { return [0, 0]; };

    patches.map(|bounds| {


        if !fraction_at_least(before, bounds, is_white, 500)
            || !fraction_at_least(after, bounds, is_white, 500)
        {
            return 0;
        }

        let mut changed = 0;


        for y in bounds.y..bounds.y + bounds.height {


            for x in bounds.x..bounds.x + bounds.width {


                if (i64::from(x) - i64::from(point.x)).abs() < 48
                    && (i64::from(y) - i64::from(point.y)).abs() < 48
                {
                    continue;
                }


                let Some(first) = pixel_rgb(before, x, y) else { continue; };


                let Some(second) = pixel_rgb(after, x, y) else { continue; };


                if ((is_white(first) && is_card_ink(second))
                    || (is_card_ink(first) && is_white(second)))
                    && first.into_iter().zip(second).any(|(a, b)| a.abs_diff(b) >= 48)
                {
                    changed += 1;
                }
            }
        }

        changed
    })
}


/// Measured effect evidence, including rejected counts for private session logs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectEvidence {
    /// True only when all target-specific source/destination conditions pass.
    pub verified: bool,
    /// Interior source pixels changing materially outside gold and cursor masks.
    pub source_changed: usize,
    /// Source pixels positively revealing white paper or green felt.
    pub source_positive: usize,
    /// Total paper/ink changes in the two retained-white RIGHT corner patches.
    pub source_identity_changed: usize,
    /// Per-corner paper/ink counts; both must pass the independent corner bound.
    pub source_identity_corners: [usize; 2],
    /// Largest eligible destination interior change; waste count for stock input.
    pub destination_changed: usize,
    /// Explicit acceptance/refusal reason; unknown scenes never imply completion.
    pub reason: &'static str,
}


impl fmt::Display for EffectEvidence {


    /// Keep all authority-bearing measurements on one readable log line.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "verified={}; source changed={}, positive={} (required {}), RIGHT corner ink/paper={:?} (each required {}), destination changed={} (required {}); {}",
            self.verified, self.source_changed, self.source_positive,
            MINIMUM_CONTENT_CHANGE, self.source_identity_corners,
            MINIMUM_CORNER_CHANGE, self.destination_changed, MINIMUM_CONTENT_CHANGE,
            self.reason,
        )
    }
}


/// Verify positive source and destination changes without copying another game's
/// repeated-HALO exception. Draw needs changed waste content; recycle needs the
/// stock back to reappear and the waste to empty. RIGHT can also show a changed
/// printed identity in two white-paper corners. Solve results require review.
pub fn inspect_effect(
    before: &CapturedFrame,
    after: &CapturedFrame,
    action: GuidedAction,
) -> Result<EffectEvidence, HaloDetectionError> {
    validate_frame(before)?;
    validate_frame(after)?;
    let mut evidence = EffectEvidence {
        verified: false,
        source_changed: 0,
        source_positive: 0,
        source_identity_changed: 0,
        source_identity_corners: [0, 0],
        destination_changed: 0,
        reason: "target is not a Klondike action",
    };


    let ActionTarget::Klondike(target) = action.target else { return Ok(evidence); };


    if canonical_action(target) != Some(action)
        || analyse(before)?.prediction != PredictedAction::Action(action)
    {
        evidence.reason = "before frame does not reproduce the canonical action";
        return Ok(evidence);
    }


    if matches!(target, KlondikeTarget::Solve) {
        evidence.reason = "Solve result requires review; no completion detector is established";
        return Ok(evidence);
    }


    if !is_gameplay_scene(after)? {
        evidence.reason = "result scene does not match the calibrated gameplay layout";
        return Ok(evidence);
    }


    if matches!(target, KlondikeTarget::Draw) {
        evidence.destination_changed = content_changes(before, after, WASTE_BOUNDS, action, false);
        evidence.verified = evidence.destination_changed >= MINIMUM_CONTENT_CHANGE;
        evidence.reason = "Draw requires material waste-content change";
        return Ok(evidence);
    }


    if matches!(target, KlondikeTarget::Recycle) {
        let stock_inside = PixelRect::new(406, 130, 100, 139);
        let waste_inside = PixelRect::new(574, 130, 156, 139);
        evidence.destination_changed = content_changes(before, after, WASTE_BOUNDS, action, false);
        evidence.verified = fraction_at_least(before, stock_inside, is_felt, 900)
            && fraction_at_least(after, stock_inside, is_back, 500)
            && fraction_at_least(after, waste_inside, is_felt, 950)
            && evidence.destination_changed >= MINIMUM_CONTENT_CHANGE;
        evidence.reason = "Recycle requires blue stock, empty waste and material waste change";
        return Ok(evidence);
    }

    let source = action.effect_bounds();
    evidence.source_changed = content_changes(before, after, source, action, false);
    evidence.source_positive = content_changes(before, after, source, action, true);


    if matches!(target, KlondikeTarget::Waste { .. }) {
        evidence.source_identity_corners = waste_identity_changes(before, after, source, action);
        evidence.source_identity_changed = evidence.source_identity_corners.into_iter().sum();
    }

    let tableau = (0..7).map(|column| {
        PixelRect::new(FIRST_COLUMN_X + COLUMN_PITCH * column, TABLEAU_Y, CARD_WIDTH, TABLEAU_EFFECT_BOTTOM - TABLEAU_Y)
    });
    let foundations = (0..4).map(|column| {
        PixelRect::new(894 + COLUMN_PITCH * column, UPPER_Y, CARD_WIDTH, CARD_HEIGHT)
    });
    evidence.destination_changed = tableau.chain(foundations).map(|bounds| {
        let same_source_column = bounds.x == source.x
            && ((bounds.y >= TABLEAU_Y && source.y >= 332) || bounds.y == source.y);


        if same_source_column { 0 } else { content_changes(before, after, bounds, action, false) }
    }).max().unwrap_or(0);
    let source_verified = evidence.source_positive >= MINIMUM_CONTENT_CHANGE
        || evidence.source_identity_corners.into_iter().all(|count| count >= MINIMUM_CORNER_CHANGE);
    evidence.verified = source_verified && evidence.destination_changed >= MINIMUM_CONTENT_CHANGE;


    evidence.reason = match (source_verified, evidence.destination_changed >= MINIMUM_CONTENT_CHANGE) {
        (false, _) => "source removal/replacement not established; fresh HALO is insufficient",
        (true, false) => "source changed, but no independent destination change was established",
        (true, true) => "independent source and destination changes established",
    };

    Ok(evidence)
}


#[cfg(test)]
mod tests {
    //! Original-pixel regressions and explicitly synthetic adverse-scene probes.

    use super::*;
    use crate::capture::decode_png;


    /// Retain concise boolean assertions while production logs structured evidence.
    fn verify_effect(
        before: &CapturedFrame,
        after: &CapturedFrame,
        action: GuidedAction,
    ) -> Result<bool, HaloDetectionError> {
        inspect_effect(before, after, action).map(|evidence| evidence.verified)
    }


    /// Read a sparse fixture whose retained pixels are exact original screenshot bytes.
    fn fixture(number: u8) -> CapturedFrame {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/klondike/K{number:02}.png"));
        decode_png(&std::fs::read(path).expect("read Klondike evidence fixture"))
            .expect("decode Klondike evidence fixture")
    }


    /// Extract the sole prediction from a capture which is expected to be actionable.
    fn action(frame: &CapturedFrame) -> GuidedAction {


        let PredictedAction::Action(action) = analyse(frame).expect("analyse evidence") .prediction else {
            panic!("expected one evidenced Klondike action");
        };
        action
    }


    /// Copy genuine pixels at their original or deliberately translated coordinates.
    fn copy_region(
        source: &CapturedFrame,
        destination: &mut CapturedFrame,
        bounds: PixelRect,
        point: PixelPoint,
    ) {
        validate_bounds(source, bounds).unwrap();
        assert!(point.x >= 0 && point.y >= 0);
        validate_bounds(destination, PixelRect::new(point.x as u32, point.y as u32, bounds.width, bounds.height)).unwrap();


        for row in 0..bounds.height {
            let source_offset = (bounds.y + row) as usize * source.stride + bounds.x as usize * 4;
            let destination_offset = (point.y as usize + row as usize) * destination.stride + point.x as usize * 4;
            let length = bounds.width as usize * 4;
            destination.pixels[destination_offset..destination_offset + length]
                .copy_from_slice(&source.pixels[source_offset..source_offset + length]);
        }
    }


    /// Change a test rectangle without interpreting the synthetic pixels as live evidence.
    fn paint(frame: &mut CapturedFrame, bounds: PixelRect, rgb: [u8; 3]) {
        validate_bounds(frame, bounds).unwrap();


        for y in bounds.y..bounds.y + bounds.height {


            for x in bounds.x..bounds.x + bounds.width {
                let offset = y as usize * frame.stride + x as usize * 4;
                frame.pixels[offset..offset + 3].copy_from_slice(&rgb);
            }
        }
    }


    /// Build a no-HALO board with the genuine active Solver banner for synthetic tests.
    fn active_without_highlight() -> CapturedFrame {
        let mut frame = fixture(1);
        copy_region(&fixture(2), &mut frame, PixelRect::new(824, 34, 272, 58), PixelPoint::new(824, 34));
        frame
    }


    /// Translate the original A-clubs source highlight to a controlled upper-pile slot.
    fn add_upper_source(frame: &mut CapturedFrame, x: u32) {
        copy_region(&fixture(2), frame, PixelRect::new(1_388, 436, 154, 202), PixelPoint::new(x as i32 - 10, 100));
    }


    /// Arrange genuine card pixels into an explicitly synthetic RIGHT transfer.
    /// K14 supplies 2-diamonds/2-clubs; K15 supplies A-spades/J-clubs. Neither
    /// screenshot has the preceding live frame, so this is a controlled repro.
    fn synthetic_right_replacement(number: u8, foundation_x: u32) -> (CapturedFrame, CapturedFrame) {
        let observed = fixture(number);
        let mut before = active_without_highlight();
        add_upper_source(&mut before, 614);
        copy_region(
            &observed, &mut before,
            PixelRect::new(foundation_x, UPPER_Y, CARD_WIDTH, CARD_HEIGHT),
            PixelPoint::new(614, UPPER_Y as i32),
        );
        let mut after = before.clone();
        copy_region(
            &observed, &mut after,
            PixelRect::new(614, UPPER_Y, CARD_WIDTH, CARD_HEIGHT),
            PixelPoint::new(614, UPPER_Y as i32),
        );
        copy_region(
            &observed, &mut after,
            PixelRect::new(foundation_x, UPPER_Y, CARD_WIDTH, CARD_HEIGHT),
            PixelPoint::new(foundation_x as i32, UPPER_Y as i32),
        );
        (before, after)
    }


    /// Card-to-card replacement can have few newly white pixels outside the
    /// cursor mask. Printed detail in both corners supplies independent proof.
    #[test]
    fn right_replacements_use_two_white_paper_corners_and_a_destination() {


        for (number, foundation_x) in [(14, 894), (15, 1_398)] {
            let (before, after) = synthetic_right_replacement(number, foundation_x);
            let planned = action(&before);
            assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Waste { offset: 2 }));
            let evidence = inspect_effect(&before, &after, planned).unwrap();
            assert!(evidence.source_positive < MINIMUM_CONTENT_CHANGE, "old source gate should reject: {evidence}");
            assert!(evidence.source_identity_corners.into_iter().all(|count| count >= MINIMUM_CORNER_CHANGE), "{evidence}");
            assert!(evidence.destination_changed >= MINIMUM_CONTENT_CHANGE, "{evidence}");
            assert!(evidence.verified, "{evidence}");

            let mut source_only = after;
            copy_region(
                &before, &mut source_only,
                PixelRect::new(foundation_x, UPPER_Y, CARD_WIDTH, CARD_HEIGHT),
                PixelPoint::new(foundation_x as i32, UPPER_Y as i32),
            );
            let incomplete = inspect_effect(&before, &source_only, planned).unwrap();
            assert!(!incomplete.verified, "{incomplete}");
            assert_eq!(incomplete.destination_changed, 0);
        }
    }


    /// Centre/corner pointer changes, one changed corner and dark guide changes
    /// never become proof, even when the destination independently changes.
    #[test]
    fn right_corner_proof_rejects_cursor_single_corner_and_dark_guides() {
        let (before, completed) = synthetic_right_replacement(14, 894);
        let planned = action(&before);
        let mut destination_only = before.clone();
        copy_region(&completed, &mut destination_only, PixelRect::new(894, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(894, UPPER_Y as i32));
        assert!(!verify_effect(&before, &destination_only, planned).unwrap());


        for patch in [
            PixelRect::new(660, 179, 40, 40),
            PixelRect::new(619, 117, 24, 40),
            PixelRect::new(713, 238, 24, 40),
        ] {
            let mut cursor = destination_only.clone();
            paint(&mut cursor, patch, [0, 0, 0]);
            assert!(!verify_effect(&before, &cursor, planned).unwrap());
        }

        let mut one_corner = destination_only.clone();
        copy_region(&completed, &mut one_corner, PixelRect::new(619, 117, 28, 44), PixelPoint::new(619, 117));
        assert!(!verify_effect(&before, &one_corner, planned).unwrap());

        let mut dark_guide = destination_only;
        paint(&mut dark_guide, PixelRect::new(614, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), [0, 0, 0]);
        let report = inspect_effect(&before, &dark_guide, planned).unwrap();
        assert_eq!(report.source_identity_corners, [0, 0]);
        assert!(!report.verified);
    }


    /// Solve is a separately evidenced control; the toolbar Solver is not one.
    /// Shape/glyph corruption, a changed stock or an overlay withdraw authority.
    #[test]
    fn solve_control_requires_its_measured_artwork_empty_stock_and_scene() {
        let frame = fixture(13);
        let planned = action(&frame);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Solve));
        assert_eq!(planned.operation(), InputOperation::Click(PixelPoint::new(624, 199)));
        assert!(!inspect_effect(&frame, &frame, planned).unwrap().verified);
        let mut solver_off = frame.clone();
        paint(&mut solver_off, PixelRect::new(824, 34, 272, 58), [0, 0, 0]);
        assert_eq!(action(&solver_off), planned);


        for corruption in [
            PixelRect::new(590, 212, 70, 24),
            PixelRect::new(598, 160, 58, 42),
            PixelRect::new(562, 143, 10, 113),
        ] {
            let mut damaged = frame.clone();
            paint(&mut damaged, corruption, [12, 62, 40]);
            assert_ne!(analyse(&damaged).unwrap().prediction, PredictedAction::Action(planned));
        }

        let mut occupied = frame.clone();
        copy_region(&fixture(1), &mut occupied, PixelRect::new(390, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(390, UPPER_Y as i32));
        assert_ne!(analyse(&occupied).unwrap().prediction, PredictedAction::Action(planned));
        let mut overlay = frame.clone();
        paint(&mut overlay, PixelRect::new(820, 600, 400, 200), [20, 20, 20]);
        assert_eq!(analyse(&overlay).unwrap().prediction, PredictedAction::NoHighlight);

        let mut draw = frame;
        copy_region(&fixture(3), &mut draw, PixelRect::new(378, 100, 164, 204), PixelPoint::new(378, 100));
        assert_eq!(action(&draw).target, ActionTarget::Klondike(KlondikeTarget::Draw));
    }


    /// K19's eight-card source extends under the translucent toolbar. Its whole
    /// connected run is one target and its click remains on the unobscured King.
    #[test]
    fn long_source_below_toolbar_remains_one_safe_upper_click() {
        let frame = fixture(19);
        let planned = action(&frame);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 6, top: 390, bottom: 953,
        }));
        assert_eq!(planned.operation(), InputOperation::Click(PixelPoint::new(1_296, 430)));
        assert_eq!(planned.effect_bounds(), PixelRect::new(1_230, 390, CARD_WIDTH, 557));
        assert!(planned.effect_bounds().y + planned.effect_bounds().height <= TOOLBAR_TOP);
        assert_eq!(find_solid_card_source(
            &frame, PixelRect::new(1_230, 332, CARD_WIDTH, 936 - 332),
        ).unwrap(), None, "an internal crossbar must not close a truncated scan");
        assert!(canonical_action(KlondikeTarget::Tableau {
            column: 6, top: 390, bottom: TABLEAU_OUTLINE_BOTTOM as u16 + 1,
        }).is_none());
        assert!(canonical_action(KlondikeTarget::Tableau {
            column: 6, top: 350, bottom: 953,
        }).is_none(), "unobserved runs exceeding the existing 600-pixel bound stay unsupported");
    }


    /// A scan ending on an internal crossbar still clips the exterior rails.
    /// K19 has at least one rail on every row390..958 and neither from959;
    /// require observed termination even when a crossbar is near the scan end.
    #[test]
    fn internal_crossbars_near_scan_end_cannot_close_clipped_rails() {
        let frame = fixture(19);


        for bottom in 570..=959 {
            assert_eq!(find_solid_card_source(
                &frame, PixelRect::new(1_230, 332, CARD_WIDTH, bottom - 332),
            ).unwrap(), None, "truncated scan ending at row{bottom}");
        }


        for bottom in 960..=TABLEAU_OUTLINE_BOTTOM {
            assert_eq!(find_solid_card_source(
                &frame, PixelRect::new(1_230, 332, CARD_WIDTH, bottom - 332),
            ).unwrap(), Some(PixelRect::new(1_230, 390, CARD_WIDTH, 563)),
                "closed source with visible rail termination at row{bottom}");
        }
    }


    /// Clipping either end, removing a closing edge or breaking a rail cannot
    /// turn an internal card crossbar into the complete eight-card source.
    #[test]
    fn long_source_requires_both_closed_ends_and_connected_rails() {
        let frame = fixture(19);


        for corruption in [
            PixelRect::new(1_218, 940, 156, 23),
            PixelRect::new(1_218, 380, 156, 31),
            PixelRect::new(1_296, TOOLBAR_TOP, 1, TABLEAU_OUTLINE_BOTTOM - TOOLBAR_TOP),
            PixelRect::new(1_218, 620, 12, 24),
        ] {
            let mut damaged = frame.clone();
            paint(&mut damaged, corruption, [12, 82, 45]);
            assert_eq!(analyse(&damaged).unwrap().prediction, PredictedAction::NoHighlight, "{corruption:?}");
        }

        let mut toolbar_only = frame;
        paint(&mut toolbar_only, PixelRect::new(1_218, 380, 156, TOOLBAR_TOP - 380), [12, 82, 45]);
        assert_eq!(analyse(&toolbar_only).unwrap().prediction, PredictedAction::NoHighlight);
    }


    /// The overlap strip supplies outline recognition only. Changes to the
    /// toolbar cannot prove either side of a move, including this long source.
    #[test]
    fn toolbar_changes_do_not_prove_long_source_or_destination_effects() {
        let before = fixture(19);
        let planned = action(&before);
        let mut after = before.clone();
        paint(&mut after, PixelRect::new(378, TOOLBAR_TOP, 1_162, 1_033 - TOOLBAR_TOP), [255, 255, 255]);
        let evidence = inspect_effect(&before, &after, planned).unwrap();
        assert!(!evidence.verified, "{evidence}");
        assert_eq!(evidence.source_changed, 0);
        assert_eq!(evidence.source_positive, 0);
        assert_eq!(evidence.destination_changed, 0);
    }


    /// Charlie's replay returns to the identical retained board/toolbar pixels.
    /// It verifies the preceding Draw, not a successful RIGHT-card transfer.
    #[test]
    fn replay_reproduces_right_target_without_proving_its_transfer() {
        let stopped = fixture(16);
        let replayed = fixture(18);
        assert_eq!(stopped.pixels, replayed.pixels);
        let planned = action(&replayed);
        assert_eq!(planned.operation(), InputOperation::Click(PixelPoint::new(652, 199)));
        assert!(!inspect_effect(&stopped, &replayed, planned).unwrap().verified);
    }


    /// All nineteen captures retain the board; K01 has no action while Solver is off.
    #[test]
    fn all_original_pixel_fixtures_match_expected_targets() {
        let expected = [
            None,
            Some(KlondikeTarget::Tableau { column: 7, top: 443, bottom: 632 }),
            Some(KlondikeTarget::Draw),
            Some(KlondikeTarget::Tableau { column: 5, top: 407, bottom: 597 }),
            Some(KlondikeTarget::Tableau { column: 3, top: 372, bottom: 562 }),
            Some(KlondikeTarget::Draw),
            Some(KlondikeTarget::Draw),
            Some(KlondikeTarget::Draw),
            Some(KlondikeTarget::Draw),
            Some(KlondikeTarget::Tableau { column: 5, top: 372, bottom: 671 }),
            Some(KlondikeTarget::Draw),
            Some(KlondikeTarget::Recycle),
            Some(KlondikeTarget::Solve),
            Some(KlondikeTarget::Draw),
            Some(KlondikeTarget::Recycle),
            Some(KlondikeTarget::Waste { offset: 1 }),
            Some(KlondikeTarget::Draw),
            Some(KlondikeTarget::Waste { offset: 1 }),
            Some(KlondikeTarget::Tableau { column: 6, top: 390, bottom: 953 }),
        ];


        for (index, target) in expected.into_iter().enumerate() {
            let frame = fixture(index as u8 + 1);
            assert!(is_gameplay_scene(&frame).unwrap(), "K{:02} scene", index + 1);
            assert_eq!(
                analyse(&frame).unwrap().prediction,
                target.and_then(canonical_action).map_or(PredictedAction::NoHighlight, PredictedAction::Action),
                "K{:02} action", index + 1,
            );
        }
    }


    /// Each actual before/after pair proves an effect; intermediate timestamps are not paired.
    #[test]
    fn all_six_recorded_action_pairs_have_independent_effects() {


        for (before_number, after_number) in [(2, 3), (4, 5), (6, 7), (8, 9), (10, 11), (17, 18)] {
            let before = fixture(before_number);
            let after = fixture(after_number);
            assert!(verify_effect(&before, &after, action(&before)).unwrap(), "K{before_number:02}->K{after_number:02}");
            assert!(!verify_effect(&before, &before, action(&before)).unwrap(), "unchanged K{before_number:02}");
        }
    }


    /// Repeated Draw recommendations need an independently changed waste face every time.
    #[test]
    fn unchanged_draw_halo_never_proves_an_effect() {
        let before = fixture(8);
        assert_eq!(action(&before).target, ActionTarget::Klondike(KlondikeTarget::Draw));
        assert_eq!(action(&fixture(9)).target, action(&before).target);
        assert!(verify_effect(&before, &fixture(9), action(&before)).unwrap());
        assert!(!verify_effect(&before, &before, action(&before)).unwrap());
    }


    /// Removing a genuine source leaves only the genuine dark dashed destination, which fails.
    #[test]
    fn dashed_destinations_and_solid_dark_guides_are_not_sources() {
        let mut frame = fixture(4);
        copy_region(&fixture(3), &mut frame, PixelRect::new(1_050, 390, 156, 220), PixelPoint::new(1_050, 390));
        assert_eq!(analyse(&frame).unwrap().prediction, PredictedAction::NoHighlight);

        let mut dark_source = fixture(2);
        paint(&mut dark_source, PixelRect::new(1_398, 448, 132, 175), [0, 0, 0]);
        assert_eq!(analyse(&dark_source).unwrap().prediction, PredictedAction::NoHighlight);
    }


    /// An old source becoming a dark destination while the previous guide clears is no move.
    #[test]
    fn recommendation_only_mask_changes_never_prove_a_card_transfer() {
        let before = fixture(2);
        let mut after = before.clone();
        paint(&mut after, PixelRect::new(1_398, 448, 132, 175), [0, 0, 0]);
        copy_region(&fixture(3), &mut after, PixelRect::new(894, 112, 132, 175), PixelPoint::new(894, 112));
        assert!(is_gameplay_scene(&after).unwrap());
        assert!(!verify_effect(&before, &after, action(&before)).unwrap());
    }


    /// Neither a moved pointer nor source-only change supplies the two-sided transfer proof.
    #[test]
    fn cursor_changes_and_missing_destination_effect_are_rejected() {
        let before = fixture(2);
        let planned = action(&before);


        let InputOperation::Click(point) = planned.operation() else { panic!("expected click"); };
        let mut cursor = before.clone();
        paint(&mut cursor, PixelRect::new(point.x as u32 - 20, point.y as u32 - 20, 40, 40), [0, 0, 0]);
        assert!(!verify_effect(&before, &cursor, planned).unwrap());

        let mut source_only = before.clone();
        copy_region(&fixture(3), &mut source_only, PixelRect::new(1_388, 425, 154, 213), PixelPoint::new(1_388, 425));
        assert!(!verify_effect(&before, &source_only, planned).unwrap());
    }


    /// Stock wins over tableau; the complete three-card source remains one logical target.
    #[test]
    fn draw_priority_and_connected_run_identity_are_preserved() {
        let run = fixture(10);
        let selected = action(&run);
        assert!(matches!(selected.target, ActionTarget::Klondike(KlondikeTarget::Tableau { column: 5, top: 372, bottom: 671 })));
        assert_eq!(selected.operation(), InputOperation::Click(PixelPoint::new(1_128, 412)));
        let mut draw_and_run = run;
        copy_region(&fixture(3), &mut draw_and_run, PixelRect::new(378, 100, 164, 204), PixelPoint::new(378, 100));
        assert_eq!(action(&draw_and_run).target, ActionTarget::Klondike(KlondikeTarget::Draw));
    }


    /// Synthetic translated genuine halos cover the three fan positions and four foundations.
    #[test]
    fn right_fan_geometry_and_last_priority_foundations_are_bounded() {


        for offset in 0..=2 {
            let mut frame = active_without_highlight();
            add_upper_source(&mut frame, 558 + WASTE_FAN_STEP * u32::from(offset));
            assert_eq!(action(&frame).target, ActionTarget::Klondike(KlondikeTarget::Waste { offset }));
        }


        for column in 1..=4 {
            let mut frame = active_without_highlight();
            add_upper_source(&mut frame, 894 + COLUMN_PITCH * u32::from(column - 1));
            assert_eq!(action(&frame).target, ActionTarget::Klondike(KlondikeTarget::Foundation { column }));
            add_upper_source(&mut frame, 614);
            assert_eq!(action(&frame).target, ActionTarget::Klondike(KlondikeTarget::Waste { offset: 2 }));
        }
    }


    /// Recycle uses one click and needs positive blue-stock/empty-waste result evidence.
    /// The after scene here is synthetic because K12 supplies only the real before scene.
    #[test]
    fn recycle_requires_both_stock_return_and_empty_waste() {
        let before = fixture(12);
        let selected = action(&before);
        assert_eq!(selected.operation(), InputOperation::Click(PixelPoint::new(456, 199)));
        let mut after = before.clone();
        copy_region(&fixture(3), &mut after, PixelRect::new(378, 100, 164, 204), PixelPoint::new(378, 100));
        assert!(!verify_effect(&before, &after, selected).unwrap());
        copy_region(&fixture(6), &mut after, PixelRect::new(548, 100, 210, 204), PixelPoint::new(548, 100));
        assert!(verify_effect(&before, &after, selected).unwrap());
    }


    /// Reject malformed storage, changed capture geometry, overlays and forged targets.
    #[test]
    fn invalid_frames_scenes_and_dynamic_targets_fail_closed() {
        let mut malformed = fixture(2);
        malformed.pixels.truncate(4);
        assert_eq!(analyse(&malformed), Err(HaloDetectionError::InvalidFrameLayout));
        let mut wrong_size = fixture(2);
        wrong_size.width = 1_919;
        assert_eq!(analyse(&wrong_size), Err(HaloDetectionError::BoundsOutsideFrame));
        let mut overlay = fixture(2);
        paint(&mut overlay, PixelRect::new(820, 600, 400, 200), [20, 20, 20]);
        assert!(!is_gameplay_scene(&overlay).unwrap());
        assert_eq!(analyse(&overlay).unwrap().prediction, PredictedAction::NoHighlight);
        assert!(canonical_action(KlondikeTarget::Waste { offset: 3 }).is_none());
        assert!(canonical_action(KlondikeTarget::Foundation { column: 5 }).is_none());
        assert!(canonical_action(KlondikeTarget::Tableau { column: 1, top: 1, bottom: u16::MAX }).is_none());
        assert!(find_solid_card_source(&fixture(2), PixelRect::new(0, 100, 132, 200)).is_err());
    }
}
