//! Klondike Draw 1 Solver targets and conservative, mode-owned effect evidence.
//!
//! Geometry is measured from Charlie's 1920x1080 captures K01-K12. A source
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


/// Exclusive lower limit, above the guest toolbar and all input controls.
const TABLEAU_BOTTOM: u32 = 936;


/// Card fan advances by this many pixels, capped at two advances.
const WASTE_FAN_STEP: u32 = 28;


/// Full waste face envelope, including the two exposed older-card strips.
const WASTE_BOUNDS: PixelRect = PixelRect::new(558, UPPER_Y, 188, CARD_HEIGHT);


/// Material content changes required independently in source/destination ROIs.
const MINIMUM_CONTENT_CHANGE: usize = 512;


/// A click on this measured toolbar control requests a fresh Solver recommendation.
/// Only the worker may authorise its bounded, scene-validated recovery use.
pub const SOLVER_CLICK: PixelPoint = PixelPoint::new(600, 990);


/// Geometry-bearing target identity; no persistent card-rank state is implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KlondikeTarget {
    /// Highlighted non-empty stock, delivered with Charlie's confirmed D shortcut.
    Draw,
    /// Highlighted empty stock, delivered as one click on the recycle area.
    Recycle,
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
        label: "KL RIGHT",
        bounds: WASTE_BOUNDS,
        colour: [255, 192, 48],
    },
    PreviewTarget {
        label: "KL tableau: dynamic source blocks",
        bounds: PixelRect::new(390, TABLEAU_Y, 1_140, TABLEAU_BOTTOM - TABLEAU_Y),
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


/// Find a bright source card/run inside a dynamic vertical scan envelope.
///
/// This reusable primitive starts at the bottom, requires a 96-pixel continuous
/// edge for the 132-pixel cards, follows both exterior rails, and probes white
/// paper above the bottom. A black destination interior fails even if some gold
/// dashes align. Card-rank artwork is not required to be white at one exact pixel.
/// The returned rectangle groups all connected highlighted cards as one action.
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


    for y in (scan.y..scan.y + scan.height).rev() {


        if y < scan.y + 170
            || !pixel_rgb(frame, x + scan.width / 2, y).is_some_and(is_edge_gold)
        {
            continue;
        }


        if !(x + 18..right - 18).all(|xx| pixel_rgb(frame, xx, y).is_some_and(is_edge_gold)) {
            continue;
        }

        let mut top = y;
        let mut gap = 0;
        let mut matched_rows = 0;


        for row in (scan.y..y - 9).rev() {
            let left_rail = (x - 9..x).any(|xx| pixel_rgb(frame, xx, row).is_some_and(is_rail_gold));
            let right_rail = (right..right + 10).any(|xx| {
                pixel_rgb(frame, xx, row).is_some_and(is_rail_gold)
            });


            if left_rail && right_rail {
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


        if !(180..=600).contains(&height) || matched_rows * 100 < (height - 10) * 85 {
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
        KlondikeTarget::Waste { offset } if offset <= 2 => {
            let x = 558 + WASTE_FAN_STEP * u32::from(offset);
            (
                InputOperation::Click(PixelPoint::new((x + CARD_WIDTH / 2) as i32, 199)),
                PixelRect::new(x, UPPER_Y, CARD_WIDTH, CARD_HEIGHT),
            )
        }
        KlondikeTarget::Tableau { column, top, bottom }
            if (1..=7).contains(&column) && top >= 332
                && u32::from(bottom) <= TABLEAU_BOTTOM
                && bottom.checked_sub(top).is_some_and(|height| (180..=600).contains(&height)) =>
        {
            let x = FIRST_COLUMN_X + COLUMN_PITCH * u32::from(column - 1);
            (
                InputOperation::Click(PixelPoint::new((x + CARD_WIDTH / 2) as i32, i32::from(top) + 40)),
                PixelRect::new(x, u32::from(top), CARD_WIDTH, u32::from(bottom - top)),
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


/// Select DRAW, RIGHT, the lowest tableau source, then SUIT, from one frame.
/// Source run grouping never treats each outlined card as a separate action.
pub fn analyse(frame: &CapturedFrame) -> Result<FrameAnalysis, HaloDetectionError> {


    let target = if is_gameplay_scene(frame)? && has_solver_banner(frame) {
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


    if find_solid_outline(frame, upper_scan(390), false)?.is_some() {
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


    for offset in (0..=2_u8).rev() {


        if find_solid_card_source(frame, upper_scan(558 + WASTE_FAN_STEP * u32::from(offset)))?.is_some() {
            return Ok(Some(KlondikeTarget::Waste { offset }));
        }
    }

    let mut lowest = None;


    for column in 1..=7_u8 {
        let x = FIRST_COLUMN_X + COLUMN_PITCH * u32::from(column - 1);
        let scan = PixelRect::new(x, 332, CARD_WIDTH, TABLEAU_BOTTOM - 332);


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


/// Verify positive source and destination changes without copying another game's
/// repeated-HALO exception. Draw needs changed waste content; recycle needs the
/// stock back to reappear and the waste to empty. Unknown end screens fail closed.
pub fn verify_effect(
    before: &CapturedFrame,
    after: &CapturedFrame,
    action: GuidedAction,
) -> Result<bool, HaloDetectionError> {
    validate_frame(before)?;
    validate_frame(after)?;


    let ActionTarget::Klondike(target) = action.target else { return Ok(false); };


    if canonical_action(target) != Some(action)
        || analyse(before)?.prediction != PredictedAction::Action(action)
        || !is_gameplay_scene(after)?
    {
        return Ok(false);
    }


    if matches!(target, KlondikeTarget::Draw) {
        return Ok(content_changes(before, after, WASTE_BOUNDS, action, false) >= MINIMUM_CONTENT_CHANGE);
    }


    if matches!(target, KlondikeTarget::Recycle) {
        let stock_inside = PixelRect::new(406, 130, 100, 139);
        let waste_inside = PixelRect::new(574, 130, 156, 139);
        return Ok(
            fraction_at_least(before, stock_inside, is_felt, 900)
                && fraction_at_least(after, stock_inside, is_back, 500)
                && fraction_at_least(after, waste_inside, is_felt, 950)
                && content_changes(before, after, WASTE_BOUNDS, action, false) >= MINIMUM_CONTENT_CHANGE,
        );
    }

    let source = action.effect_bounds();


    if content_changes(before, after, source, action, true) < MINIMUM_CONTENT_CHANGE {
        return Ok(false);
    }

    let tableau = (0..7).map(|column| {
        PixelRect::new(FIRST_COLUMN_X + COLUMN_PITCH * column, TABLEAU_Y, CARD_WIDTH, TABLEAU_BOTTOM - TABLEAU_Y)
    });
    let foundations = (0..4).map(|column| {
        PixelRect::new(894 + COLUMN_PITCH * column, UPPER_Y, CARD_WIDTH, CARD_HEIGHT)
    });
    let destination_changed = tableau.chain(foundations).any(|bounds| {
        let same_source_column = bounds.x == source.x
            && ((bounds.y >= TABLEAU_Y && source.y >= 332) || bounds.y == source.y);
        !same_source_column
            && content_changes(before, after, bounds, action, false) >= MINIMUM_CONTENT_CHANGE
    });

    Ok(destination_changed)
}


#[cfg(test)]
mod tests {
    //! Original-pixel regressions and explicitly synthetic adverse-scene probes.

    use super::*;
    use crate::capture::decode_png;


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


    /// All twelve captures retain the evidenced board; K01 has no action while Solver is off.
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
    fn all_five_recorded_action_pairs_have_independent_effects() {


        for (before_number, after_number) in [(2, 3), (4, 5), (6, 7), (8, 9), (10, 11)] {
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
