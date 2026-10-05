//! Free Cell's independent, single-frame Solver source detector.
//!
//! Native FC01-FC13 evidence supplies four CELL and eight PLAY columns. Only a
//! solid source outline is actionable; dark dashed destination guides are not.
//! Every frame is classified independently, so stack compression, expansion and
//! repeated source positions do not require card recognition or effect proofs.
//! SUIT piles, Draw, Recycle and Solve are not Free Cell input targets.

use std::fmt;

use crate::{
    capture::CapturedFrame,
    detector::{HaloDetectionError, pixel_rgb},
    game::{
        ActionSpecification, ActionTarget, AnimationClass, GameProfile, GuidedAction,
        InputOperation, RepeatTargetPolicy, TargetSelectionPolicy,
    },
    geometry::{PixelPoint, PixelRect},
    tracker::{FrameAnalysis, PredictedAction},
};


/// Native card-face origins for the four temporary cells, left to right.
const CELL_X: [u32; 4] = [166, 358, 551, 743];


/// Native card-face origins for the eight play columns, left to right.
const PLAY_X: [u32; 8] = [218, 410, 603, 795, 988, 1_180, 1_373, 1_565];


/// Measured face width; the exterior outline is read only outside this width.
const CARD_WIDTH: u32 = 137;


/// Earliest source outline above the initial first play card at row 331.
const PLAY_SCAN_TOP: u32 = 319;


/// First native toolbar row; all source probes and source clicks stay above it.
pub const TOOLBAR_TOP: u32 = 947;


/// Centre of the supplied Solver toolbar control, used only for activation.
pub const SOLVER_CLICK: PixelPoint = PixelPoint::new(602, 977);


/// A fully visible single-card source, including its outline, exceeds this height.
const MINIMUM_SOURCE_HEIGHT: u32 = 180;


/// Maximum tolerated rail gap around rounded corners or a shared card boundary.
const MAXIMUM_RAIL_GAP: u32 = 10;


/// Independent profile: no fixed rows, consumed slots or three-board progress.
pub const PROFILE: GameProfile = GameProfile {
    label: "Free Cell",
    frame_width: 1_920,
    frame_height: 1_080,
    preview_targets: &[],
    gameplay_scene: None,
    game_progress: None,
    bottom_targets: &[],
    target_selection: TargetSelectionPolicy::FirstByPriority,
    tableau_cards: &[],
    tableau_rows: &[],
    tableau_row_count: 1,
    initial_active_rows: 1,
    tableau_click_offset: PixelPoint::new(0, 0),
    minimum_tableau_changed_pixels: 0,
    boards_per_game: 1,
};


/// Source identity derived from the current frame, never from card ranks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FreeCellTarget {
    /// One-based temporary CELL position.
    Cell {
        /// Native left-to-right cell number, in `1..=4`.
        column: u8,
    },
    /// One contiguous highlighted PLAY block, including a multi-card run.
    Play {
        /// Native left-to-right column number, in `1..=8`.
        column: u8,
        /// Inclusive upper outline row measured in the current frame.
        top: u32,
        /// Exclusive lower outline row, strictly above the game toolbar.
        bottom: u32,
    },
}


impl fmt::Display for FreeCellTarget {


    /// Preserve source geometry in predictions and diagnostic messages.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {


        match self {
            Self::Cell { column } => write!(formatter, "CELL {column}"),
            Self::Play { column, top, bottom } =>
                write!(formatter, "PLAY {column}, rows {top}..{bottom}"),
        }
    }
}


/// Reject malformed storage or captures outside the evidenced native geometry.
fn validate_frame(frame: &CapturedFrame) -> Result<(), HaloDetectionError> {


    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }


    if frame.width != PROFILE.frame_width || frame.height != PROFILE.frame_height {
        return Err(HaloDetectionError::BoundsOutsideFrame);
    }

    Ok(())
}


/// Recognise warm source gold by channel relationships, not exact artwork RGB.
fn is_gold([red, green, blue]: [u8; 3]) -> bool {
    red >= 125 && green >= 90
        && i16::from(red) - i16::from(blue) >= 50
        && i16::from(green) - i16::from(blue) >= 28
        && i16::from(red) - i16::from(green) >= -15
}


/// Ordinary undimmed felt excludes modal blue and animation-darkened overlays.
fn is_felt([red, green, blue]: [u8; 3]) -> bool {
    green >= 65 && i16::from(green) - i16::from(red) >= 25
        && i16::from(green) - i16::from(blue) >= 20
}


/// Broad white or pale-green slot edges identify the eight-column board layout.
fn is_slot_edge(rgb: [u8; 3]) -> bool {
    let [red, green, blue] = rgb;
    (red >= 180 && green >= 180 && blue >= 180)
        || (red >= 55 && green >= 125 && blue >= 105
            && i16::from(green) - i16::from(red) >= 25
            && i16::from(green) - i16::from(blue) >= 20)
        || is_gold(rgb)
}


/// Count a bounded rectangle with safe frame reads and widened percentage maths.
fn fraction_at_least(
    frame: &CapturedFrame,
    bounds: PixelRect,
    predicate: fn([u8; 3]) -> bool,
    per_mille: u32,
) -> bool {
    let mut matched = 0_u64;


    for y in bounds.y..bounds.bottom() {


        for x in bounds.x..bounds.right() {


            if pixel_rgb(frame, x, y).is_some_and(predicate) {
                matched += 1;
            }
        }
    }

    matched * 1_000 >= u64::from(bounds.width) * u64::from(bounds.height) * u64::from(per_mille)
}


/// Recognise open Free Cell gameplay using gutters and coarse board-slot rows.
/// No card identity, suit pile, rank, toolbar icon or level artwork is matched.
pub fn is_gameplay_scene(frame: &CapturedFrame) -> Result<bool, HaloDetectionError> {
    validate_frame(frame)?;
    let open_gutters = [PixelRect::new(80, 350, 24, 24), PixelRect::new(1_790, 350, 24, 24)];


    if !open_gutters.into_iter().all(|bounds| fraction_at_least(frame, bounds, is_felt, 900)) {
        return Ok(false);
    }

    let cells = CELL_X.into_iter().filter(|x|
        fraction_at_least(frame, PixelRect::new(*x + 20, 112, 96, 3), is_slot_edge, 700)
    ).count();
    let plays = PLAY_X.into_iter().filter(|x|
        fraction_at_least(frame, PixelRect::new(*x + 20, 331, 96, 3), is_slot_edge, 700)
    ).count();

    Ok(cells >= 3 && plays >= 6)
}


/// Recognise Solver activation by its stable top banner's golden boundaries.
pub fn solver_active(frame: &CapturedFrame) -> Result<bool, HaloDetectionError> {
    validate_frame(frame)?;
    Ok([36, 85].into_iter().all(|y|
        fraction_at_least(frame, PixelRect::new(828, y, 263, 1), is_gold, 900)
    ))
}


/// Both exterior rails must be present; card face jewellery stays inside them.
fn paired_rails(frame: &CapturedFrame, x: u32, y: u32) -> bool {
    (x - 10..x).any(|column| pixel_rgb(frame, column, y).is_some_and(is_gold))
        && (x + CARD_WIDTH..x + CARD_WIDTH + 11)
            .any(|column| pixel_rgb(frame, column, y).is_some_and(is_gold))
}


/// A near-full-width golden crossbar closes a solid source outline.
fn crossbar(frame: &CapturedFrame, x: u32, y: u32) -> bool {
    let matched = (x + 18..x + CARD_WIDTH - 18)
        .filter(|column| pixel_rgb(frame, *column, y).is_some_and(is_gold)).count();

    matched >= 95
}


/// Find the lowest complete solid source, grouping connected card rails as one.
/// Dashed destinations lack continuous rails and closed crossbars. A clipped
/// source remains unresolved until original toolbar-crossing FC14 is supplied.
fn find_source(frame: &CapturedFrame, x: u32, scan_top: u32, scan_bottom: u32) -> Option<PixelRect> {
    let mut row = scan_bottom;


    while row > scan_top {
        row -= 1;


        if !paired_rails(frame, x, row) {
            continue;
        }

        let lower_rail = row;
        let mut upper_rail = row;
        let mut matched_rows = 1_u32;
        let mut gap = 0_u32;


        while row > scan_top {
            row -= 1;


            if paired_rails(frame, x, row) {
                upper_rail = row;
                matched_rows += 1;
                gap = 0;
            } else {
                gap += 1;


                if gap > MAXIMUM_RAIL_GAP {
                    break;
                }
            }
        }

        let rail_height = lower_rail + 1 - upper_rail;


        if rail_height < MINIMUM_SOURCE_HEIGHT - 10
            || matched_rows * 100 < rail_height * 85
            || lower_rail >= scan_bottom - 8 {
            continue;
        }

        let top = (upper_rail.saturating_sub(10).max(scan_top)..=upper_rail + 5)
            .find(|y| crossbar(frame, x, *y));
        let bottom = (lower_rail.saturating_sub(5)..(lower_rail + 11).min(scan_bottom))
            .rev().find(|y| crossbar(frame, x, *y));


        if let (Some(top), Some(last)) = (top, bottom) {
            let bottom = last + 1;


            if bottom - top >= MINIMUM_SOURCE_HEIGHT && bottom < scan_bottom {
                return Some(PixelRect::new(x, top, CARD_WIDTH, bottom - top));
            }
        }
    }

    None
}


/// Validate source coordinates and construct exactly one bottom-card click.
/// No changed-pixel effect threshold is attached to Free Cell actions.
pub fn canonical_action(target: FreeCellTarget) -> Option<GuidedAction> {
    let (x, click_y, bounds) = match target {
        FreeCellTarget::Cell { column } if (1..=4).contains(&column) => {
            let x = CELL_X[usize::from(column - 1)];
            (x, 203, PixelRect::new(x, 112, CARD_WIDTH, 182))
        }
        FreeCellTarget::Play { column, top, bottom }


            if (1..=8).contains(&column) && top >= PLAY_SCAN_TOP && bottom < TOOLBAR_TOP
                && bottom.checked_sub(top).is_some_and(|height| height >= MINIMUM_SOURCE_HEIGHT) => {
            let x = PLAY_X[usize::from(column - 1)];
            (x, bottom - 40, PixelRect::new(x, top, CARD_WIDTH, bottom - top))
        }
        _ => return None,
    };
    let anchor = PixelPoint::new((x + CARD_WIDTH / 2) as i32, click_y as i32);

    Some(GuidedAction {
        target: ActionTarget::FreeCell(target),
        anchor,
        specification: ActionSpecification {
            operation: InputOperation::Click(anchor),
            animation_class: AnimationClass::FreeCell,
            effect_bounds: bounds,
            minimum_changed_pixels: 0,
            exclude_cursor_from_effect: false,
            repeat_target: RepeatTargetPolicy::Allowed,
        },
    })
}


/// Check CELL1-4 first, then the lowest PLAY source with left-to-right tie-breaks.
/// Each invocation uses only this frame; SUIT positions are never searched.
pub fn analyse(frame: &CapturedFrame) -> Result<FrameAnalysis, HaloDetectionError> {
    let mut selected = None;


    if is_gameplay_scene(frame)? && solver_active(frame)? {


        for (index, x) in CELL_X.into_iter().enumerate() {


            if find_source(frame, x, 100, 310).is_some() {
                selected = Some(FreeCellTarget::Cell { column: index as u8 + 1 });
                break;
            }
        }


        if selected.is_none() {


            for (index, x) in PLAY_X.into_iter().enumerate() {


                if let Some(bounds) = find_source(frame, x, PLAY_SCAN_TOP, TOOLBAR_TOP) {
                    let candidate = FreeCellTarget::Play {
                        column: index as u8 + 1,
                        top: bounds.y,
                        bottom: bounds.bottom(),
                    };


                    if selected.is_none_or(|prior| matches!(prior,
                        FreeCellTarget::Play { bottom, .. } if bounds.bottom() > bottom)) {
                        selected = Some(candidate);
                    }
                }
            }
        }
    }

    Ok(FrameAnalysis {
        prediction: selected.and_then(canonical_action)
            .map_or(PredictedAction::NoHighlight, PredictedAction::Action),
        observed_rows: None,
        top_row_face_up_count: 0,
    })
}


#[cfg(test)]
mod tests {
    //! Focused native source, scene, priority and bounds regressions.

    use super::*;
    use crate::{capture::{PixelFormat, decode_png}, game::GameMode};


    /// Decode an unmodified native fixture by its stable evidence identifier.
    fn fixture(code: u8) -> CapturedFrame {
        let bytes: &[u8] = match code {
            1 => include_bytes!("../tests/fixtures/freecell-FC01.png"),
            2 => include_bytes!("../tests/fixtures/freecell-FC02.png"),
            3 => include_bytes!("../tests/fixtures/freecell-FC03.png"),
            4 => include_bytes!("../tests/fixtures/freecell-FC04.png"),
            5 => include_bytes!("../tests/fixtures/freecell-FC05.png"),
            6 => include_bytes!("../tests/fixtures/freecell-FC06.png"),
            7 => include_bytes!("../tests/fixtures/freecell-FC07.png"),
            8 => include_bytes!("../tests/fixtures/freecell-FC08.png"),
            9 => include_bytes!("../tests/fixtures/freecell-FC09.png"),
            10 => include_bytes!("../tests/fixtures/freecell-FC10.png"),
            11 => include_bytes!("../tests/fixtures/freecell-FC11.png"),
            12 => include_bytes!("../tests/fixtures/freecell-FC12.png"),
            13 => include_bytes!("../tests/fixtures/freecell-FC13.png"),
            _ => panic!("unknown native Free Cell fixture"),
        };

        decode_png(bytes).expect("native Free Cell fixture")
    }


    /// Construct native storage without any positive board or source evidence.
    fn blank_frame() -> CapturedFrame {
        CapturedFrame {
            width: PROFILE.frame_width,
            height: PROFILE.frame_height,
            stride: PROFILE.frame_width as usize * 4,
            format: PixelFormat::Rgba8,
            pixels: vec![0; PROFILE.frame_width as usize * PROFILE.frame_height as usize * 4],
        }
    }


    /// Paint only an in-memory fixture copy to isolate an outline decision.
    fn paint(frame: &mut CapturedFrame, bounds: PixelRect, rgb: [u8; 3]) {


        for y in bounds.y..bounds.bottom() {


            for x in bounds.x..bounds.right() {
                let offset = y as usize * frame.stride + x as usize * 4;
                frame.pixels[offset..offset + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
        }
    }


    /// Draw a simple solid source on a fixture copy, not a new PNG artifact.
    fn paint_source(frame: &mut CapturedFrame, x: u32, top: u32, bottom: u32) {
        let gold = [235, 195, 90];
        paint(frame, PixelRect::new(x - 5, top + 3, 3, bottom - top - 6), gold);
        paint(frame, PixelRect::new(x + CARD_WIDTH + 2, top + 3, 3, bottom - top - 6), gold);
        paint(frame, PixelRect::new(x + 18, top, CARD_WIDTH - 36, 1), gold);
        paint(frame, PixelRect::new(x + 18, bottom - 1, CARD_WIDTH - 36, 1), gold);
    }


    /// Separate gameplay and terminal panels without whole-screen RGB matching.
    #[test]
    fn native_scenes_and_solver_activation_are_independent() {


        for code in 1..=13 {
            let frame = fixture(code);
            assert_eq!(is_gameplay_scene(&frame).unwrap(), code <= 8 || code == 13, "FC{code:02}");
            assert_eq!(solver_active(&frame).unwrap(), (2..=8).contains(&code), "FC{code:02}");
        }

        assert_eq!(analyse(&fixture(1)).unwrap().prediction, PredictedAction::NoHighlight);
        assert_eq!(analyse(&fixture(13)).unwrap().prediction, PredictedAction::NoHighlight);
        assert_eq!(GameMode::FreeCell.profile().boards_per_game, 1);
        assert!(GameMode::FreeCell.profile().game_progress.is_none());
        let foreign = decode_png(include_bytes!("../tests/fixtures/klondike/K95.png")).unwrap();
        assert!(!is_gameplay_scene(&foreign).unwrap(), "a native Klondike board is not Free Cell");
    }


    /// Solid sources win over dark dashed destinations in the evidenced moves.
    #[test]
    fn native_moves_select_one_source_and_click_its_bottom_card() {
        let expected = [
            (2, FreeCellTarget::Play { column: 7, top: 587, bottom: 789 }),
            (3, FreeCellTarget::Play { column: 5, top: 534, bottom: 736 }),
            (4, FreeCellTarget::Play { column: 5, top: 481, bottom: 683 }),
            (5, FreeCellTarget::Cell { column: 2 }),
            (6, FreeCellTarget::Play { column: 2, top: 481, bottom: 736 }),
            (7, FreeCellTarget::Play { column: 2, top: 481, bottom: 736 }),
            (8, FreeCellTarget::Play { column: 2, top: 428, bottom: 630 }),
        ];


        for (code, target) in expected {
            let action = canonical_action(target).unwrap();
            assert_eq!(analyse(&fixture(code)).unwrap().prediction, PredictedAction::Action(action), "FC{code:02}");
            assert_eq!(action.minimum_changed_pixels(), 0);
            assert_eq!(action.repeat_target_policy(), RepeatTargetPolicy::Allowed);
            let InputOperation::Click(click) = action.operation() else {
                panic!("Free Cell sources only click");
            };

            assert!(click.y < TOOLBAR_TOP as i32);


            if let FreeCellTarget::Play { bottom, .. } = target {
                assert_eq!(click.y as u32, bottom - 40);
            }
        }
    }


    /// The connected two-card source remains one action at the repeated position.
    #[test]
    fn a_run_and_its_same_position_next_frame_remain_one_recommendation() {
        let first = analyse(&fixture(6)).unwrap().prediction;
        assert_eq!(first, analyse(&fixture(7)).unwrap().prediction);
        let PredictedAction::Action(action) = first else { panic!("native run source"); };
        assert_eq!(action.target, ActionTarget::FreeCell(FreeCellTarget::Play {
            column: 2, top: 481, bottom: 736,
        }));
        assert_eq!(action.operation(), InputOperation::Click(PixelPoint::new(478, 696)));
    }


    /// CELL priority and bottom-up PLAY order are independent of card contents.
    #[test]
    fn source_priority_is_cells_then_lowest_play_with_left_ties() {
        let mut frame = fixture(2);
        paint_source(&mut frame, PLAY_X[0], 500, 850);
        paint_source(&mut frame, PLAY_X[1], 500, 850);
        assert_eq!(analyse(&frame).unwrap().prediction,
            PredictedAction::Action(canonical_action(FreeCellTarget::Play {
                column: 1, top: 500, bottom: 850,
            }).unwrap()));
        paint_source(&mut frame, CELL_X[1], 102, 305);
        assert_eq!(analyse(&frame).unwrap().prediction,
            PredictedAction::Action(canonical_action(FreeCellTarget::Cell { column: 2 }).unwrap()));
        paint_source(&mut frame, CELL_X[0], 102, 305);
        assert_eq!(analyse(&frame).unwrap().prediction,
            PredictedAction::Action(canonical_action(FreeCellTarget::Cell { column: 1 }).unwrap()));
    }


    /// Destinations, SUIT outlines and toolbar pixels never become source input.
    #[test]
    fn destination_guides_and_clipped_toolbar_sources_are_not_actions() {


        for (code, x, top) in [(2, PLAY_X[6], 580), (3, PLAY_X[4], 524)] {
            let mut frame = fixture(code);
            paint(&mut frame, PixelRect::new(x - 10, top, CARD_WIDTH + 21, 218), [20, 120, 75]);
            assert!(is_gameplay_scene(&frame).unwrap());
            assert_eq!(analyse(&frame).unwrap().prediction, PredictedAction::NoHighlight);
        }

        let mut frame = fixture(2);
        paint(&mut frame, PixelRect::new(PLAY_X[6] - 10, 580, CARD_WIDTH + 21, 218), [20, 120, 75]);
        paint_source(&mut frame, 1_041, 102, 305);
        assert_eq!(analyse(&frame).unwrap().prediction, PredictedAction::NoHighlight,
            "SUIT has no Free Cell source scan");
        paint_source(&mut frame, PLAY_X[1], 800, 1_001);
        assert_eq!(analyse(&frame).unwrap().prediction, PredictedAction::NoHighlight,
            "FC14 is absent; a truncated source cannot invent a bottom-card click");
        paint(&mut frame, PixelRect::new(0, TOOLBAR_TOP, 1_920, 1_080 - TOOLBAR_TOP), [255, 255, 255]);
        assert_eq!(analyse(&frame).unwrap().prediction, PredictedAction::NoHighlight,
            "toolbar pixels are outside source authority");
    }


    /// Bright pixels and malformed frames cannot become source input authority.
    #[test]
    fn rejects_invalid_frames_and_forged_source_coordinates() {
        let mut frame = blank_frame();
        assert!(!is_gameplay_scene(&frame).unwrap());
        paint(&mut frame, PixelRect::new(0, 0, 1_920, 1_080), [20, 150, 90]);
        assert!(!is_gameplay_scene(&frame).unwrap(), "plain felt lacks Free Cell slot rows");
        frame.pixels.fill(255);
        assert_eq!(analyse(&frame).unwrap().prediction, PredictedAction::NoHighlight);
        frame.pixels.truncate(8);
        assert_eq!(analyse(&frame), Err(HaloDetectionError::InvalidFrameLayout));
        let mut frame = blank_frame();
        frame.width = 1_919;
        assert_eq!(solver_active(&frame), Err(HaloDetectionError::BoundsOutsideFrame));


        for target in [
            FreeCellTarget::Cell { column: 0 },
            FreeCellTarget::Cell { column: 5 },
            FreeCellTarget::Play { column: 0, top: 400, bottom: 600 },
            FreeCellTarget::Play { column: 9, top: 400, bottom: 600 },
            FreeCellTarget::Play { column: 1, top: 300, bottom: 600 },
            FreeCellTarget::Play { column: 1, top: 400, bottom: 947 },
            FreeCellTarget::Play { column: 1, top: 600, bottom: 400 },
            FreeCellTarget::Play { column: 1, top: u32::MAX, bottom: u32::MAX },
        ] {
            assert!(canonical_action(target).is_none(), "{target}");
        }
    }
}
