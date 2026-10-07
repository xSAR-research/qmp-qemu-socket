//! Spider's single-frame, ten-column Solver source detector.
//!
//! A solid source outline authorises one source click or one Draw key press.
//! Overlapping card outlines form one run; dark dashed destinations, collapsed
//! suits and toolbar artwork are never input targets. Every capture supplies its
//! own source geometry, without card identities or before/after effect proofs.

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


/// Native face origins for the ten PLAY columns, left to right.
const PLAY_X: [u32; 10] = [121, 292, 463, 634, 805, 976, 1_147, 1_318, 1_489, 1_660];


/// Measured native face width, independent of changing stack spacing.
const CARD_WIDTH: u32 = 139;


/// Earliest supplied source outline above the first card at row 110.
const PLAY_SCAN_TOP: u32 = 102;


/// First game toolbar row; source probes and source clicks stop above it.
pub const TOOLBAR_TOP: u32 = 947;


/// Centre of the supplied Solver toolbar control, used only for activation.
pub const SOLVER_CLICK: PixelPoint = PixelPoint::new(602, 977);


/// Empty left felt keeps the captured pointer clear of outlines and scene probes.
pub const POINTER_PARK: PixelPoint = PixelPoint::new(20, 500);


/// Native stock face row shared with the collapsed-packet display.
const DRAW_TOP: u32 = 747;


/// Leftmost supplied top stock-packet origin.
const DRAW_X_START: u32 = 1_489;


/// Rightmost supplied top stock-packet origin with all deals remaining.
const DRAW_X_END: u32 = 1_531;


/// Complete single-card outlines exceed this measured minimum height.
const MINIMUM_SOURCE_HEIGHT: u32 = 180;


/// Click inside the bottom visible card, above a clipped toolbar boundary.
const SOURCE_CLICK_BOTTOM_INSET: u32 = 40;


/// Clipped sources need a visible top, side rails and a usable card interior.
const MINIMUM_CLIPPED_SOURCE_HEIGHT: u32 = 66;


/// Rounded corners and shared card boundaries may interrupt exterior rails.
const MAXIMUM_RAIL_GAP: u32 = 10;


/// Independent one-board profile with dynamic source geometry.
pub const PROFILE: GameProfile = GameProfile {
    label: "Spider",
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


/// A current Solver recommendation, without remembered card identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpiderTarget {
    /// Deal one card to every PLAY column using the confirmed D key.
    Draw,
    /// One highlighted PLAY source block, including overlapping run outlines.
    Play {
        /// Native one-based column number in `1..=10`.
        column: u8,
        /// Inclusive upper outline row in this capture.
        top: u32,
        /// Exclusive visible outline end, capped above the game toolbar.
        bottom: u32,
    },
}


impl fmt::Display for SpiderTarget {


    /// Include current source geometry in previews and logs.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {


        match self {
            Self::Draw => write!(formatter, "DRAW"),
            Self::Play { column, top, bottom } =>
                write!(formatter, "PLAY {column}, rows {top}..{bottom}"),
        }
    }
}


/// Validate native dimensions and decoded storage before any pixel access.
fn validate_frame(frame: &CapturedFrame) -> Result<(), HaloDetectionError> {


    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }


    if frame.width != PROFILE.frame_width || frame.height != PROFILE.frame_height {
        return Err(HaloDetectionError::BoundsOutsideFrame);
    }

    Ok(())
}


/// Warm source gold is recognised by channel relationships, not artwork RGB.
fn is_gold([red, green, blue]: [u8; 3]) -> bool {
    red >= 125 && green >= 90
        && i16::from(red) - i16::from(blue) >= 50
        && i16::from(green) - i16::from(blue) >= 28
        && i16::from(red) - i16::from(green) >= -15
}


/// Undimmed felt distinguishes open gameplay from terminal panels.
fn is_felt([red, green, blue]: [u8; 3]) -> bool {
    green >= 65 && i16::from(green) - i16::from(red) >= 25
        && i16::from(green) - i16::from(blue) >= 20
}


/// Neutral paper establishes a visible card margin, never its identity.
fn is_paper([red, green, blue]: [u8; 3]) -> bool {
    red >= 180 && green >= 180 && blue >= 180
}


/// All supplied Spider stock packets use this broad red-back family.
fn is_stock_back([red, green, blue]: [u8; 3]) -> bool {
    red >= 100 && i16::from(red) - i16::from(green) >= 30
        && i16::from(red) - i16::from(blue) >= 30
}


/// Coarse card or slot edges establish the ten-column header layout.
fn is_slot_edge(rgb: [u8; 3]) -> bool {
    let [red, green, blue] = rgb;
    is_paper(rgb) || is_gold(rgb)
        || (green >= 125 && i16::from(green) - i16::from(red) >= 25
            && i16::from(green) - i16::from(blue) >= 20)
}


/// Count a bounded rectangle with safe reads and widened percentage arithmetic.
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


/// Recognise open Spider by ten top edges and their narrow felt gutters.
/// Changing card heights, difficulty, ranks, toolbar icons and stock availability
/// are not gameplay-scene conditions.
pub fn is_gameplay_scene(frame: &CapturedFrame) -> Result<bool, HaloDetectionError> {
    validate_frame(frame)?;


    if ![PixelRect::new(70, 350, 24, 24), PixelRect::new(1_820, 350, 24, 24)]
        .into_iter().all(|bounds| fraction_at_least(frame, bounds, is_felt, 900)) {
        return Ok(false);
    }

    let edges = PLAY_X.into_iter().filter(|x|
        fraction_at_least(frame, PixelRect::new(*x + 20, 110, 98, 3), is_slot_edge, 700)
    ).count();
    let gutters = PLAY_X[..9].iter().filter(|x|
        fraction_at_least(frame, PixelRect::new(**x + 147, 120, 16, 16), is_felt, 900)
    ).count();

    Ok(edges >= 8 && gutters >= 8)
}


/// Solver activation uses only the stable top banner's golden boundaries.
pub fn solver_active(frame: &CapturedFrame) -> Result<bool, HaloDetectionError> {
    validate_frame(frame)?;
    Ok([36, 85].into_iter().all(|y|
        fraction_at_least(frame, PixelRect::new(828, y, 263, 1), is_gold, 900)
    ))
}


/// Both exterior rails must be present; jewellery stays inside the face width.
fn paired_rails(frame: &CapturedFrame, x: u32, y: u32) -> bool {
    (x - 10..x).any(|column| pixel_rgb(frame, column, y).is_some_and(is_gold))
        && (x + CARD_WIDTH..x + CARD_WIDTH + 11)
            .any(|column| pixel_rgb(frame, column, y).is_some_and(is_gold))
}


/// A near-full-width crossbar closes a solid source, unlike dashed guides.
fn crossbar(frame: &CapturedFrame, x: u32, y: u32) -> bool {
    (x + 18..x + CARD_WIDTH - 18)
        .filter(|column| pixel_rgb(frame, *column, y).is_some_and(is_gold)).count() >= 95
}


/// Visible paper at either side establishes a card without testing its art.
/// Queen artwork may fill the central click area and a corner suit may cover
/// one margin, so neither the click colour nor both margins are conditions.
fn source_card_is_visible(frame: &CapturedFrame, x: u32, bottom: u32) -> bool {
    let y = bottom - SOURCE_CLICK_BOTTOM_INSET;
    [x + 6, x + CARD_WIDTH - 15].into_iter().any(|left|
        fraction_at_least(frame, PixelRect::new(left, y - 8, 9, 17), is_paper, 500)
    )
}


/// Find the lowest connected solid source in one column from this frame.
/// Complete outlines can close directly against a visible boundary. Sources
/// clipped by the toolbar instead require their visible closed top, continuous
/// opposing rails and visible paper margins; no hidden bottom is invented.
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
        let top = (upper_rail.saturating_sub(10).max(scan_top)..=(upper_rail + 5).min(scan_bottom - 1))
            .find(|y| crossbar(frame, x, *y));


        if let Some(top) = top
            && scan_bottom == TOOLBAR_TOP
            && lower_rail == TOOLBAR_TOP - 1
            && TOOLBAR_TOP - top >= MINIMUM_CLIPPED_SOURCE_HEIGHT
            && matched_rows * 100 >= rail_height * 85
            && (TOOLBAR_TOP - 4..TOOLBAR_TOP).all(|y| paired_rails(frame, x, y))
            && source_card_is_visible(frame, x, TOOLBAR_TOP) {
            return Some(PixelRect::new(x, top, CARD_WIDTH, TOOLBAR_TOP - top));
        }


        if rail_height < MINIMUM_SOURCE_HEIGHT - 10
            || matched_rows * 100 < rail_height * 85 {
            continue;
        }

        let bottom = (lower_rail.saturating_sub(5)..(lower_rail + 11).min(scan_bottom))
            .rev().find(|y| crossbar(frame, x, *y));


        if let (Some(top), Some(last)) = (top, bottom)
            && last + 1 - top >= MINIMUM_SOURCE_HEIGHT {
            return Some(PixelRect::new(x, top, CARD_WIDTH, last + 1 - top));
        }
    }

    None
}


/// Detect the supplied stock back independently of whether its HALO is active.
fn stock_is_present(frame: &CapturedFrame) -> bool {
    fraction_at_least(frame, PixelRect::new(1_535, 770, 90, 140), is_stock_back, 800)
}


/// A stock source needs both its solid outline and its coloured back interior.
/// An exhausted-stock tableau card or dashed destination is not a Draw source.
fn draw_is_highlighted(frame: &CapturedFrame) -> bool {
    stock_is_present(frame)
        && (DRAW_X_START..=DRAW_X_END).any(|x|
            find_source(frame, x, DRAW_TOP - 12, TOOLBAR_TOP).is_some()
                && fraction_at_least(frame, PixelRect::new(x + 15, 765, 109, 150), is_stock_back, 800)
        )
}


/// Bound only the currently occupied collapsed-suit display.
/// Its fixed first packet has a paper edge beside open felt and a white top.
/// No King, suit or artwork is recognised; absent packets do not reserve space.
fn collapsed_display(frame: &CapturedFrame) -> Option<PixelRect> {


    if !fraction_at_least(frame, PixelRect::new(237, 762, 4, 158), is_felt, 900)
        || !fraction_at_least(frame, PixelRect::new(262, DRAW_TOP, 19, 3), is_paper, 900) {
        return None;
    }

    let edge_rows = (762..920).filter(|y|
        (244..249).any(|x| pixel_rgb(frame, x, *y).is_some_and(is_paper))
    ).count();


    if edge_rows * 100 < 158 * 90 {
        return None;
    }

    let mut last = 262;
    let mut gap = 0;


    for x in 262..701 {


        if (DRAW_TOP..DRAW_TOP + 3).filter(|y|
            pixel_rgb(frame, x, *y).is_some_and(is_paper)
        ).count() >= 2 {
            last = x;
            gap = 0;
        } else {
            gap += 1;


            if gap > 5 {
                break;
            }
        }
    }

    Some(PixelRect::new(244, DRAW_TOP, last + 1 - 244, 186))
}


/// Occupied display areas shorten only the PLAY columns they overlap.
/// Exhausted stock and absent collapsed packets do not restrict those columns.
fn play_scan_bottom(x: u32, stock_present: bool, collapsed: Option<PixelRect>) -> u32 {
    let stock_overlaps = stock_present && x < 1_670 && x + CARD_WIDTH > DRAW_X_START;
    let collapsed_overlaps = collapsed.is_some_and(|bounds|
        x < bounds.right() && x + CARD_WIDTH > bounds.x
    );


    if stock_overlaps || collapsed_overlaps {
        DRAW_TOP
    } else {
        TOOLBAR_TOP
    }
}


/// Construct one source action, without any card-pixel effect requirement.
pub fn canonical_action(target: SpiderTarget) -> Option<GuidedAction> {
    let (anchor, specification) = match target {
        SpiderTarget::Draw => (
            PixelPoint::new(1_590, 839),
            ActionSpecification {
                operation: InputOperation::PressDrawKey,
                animation_class: AnimationClass::SpiderDraw,
                effect_bounds: PixelRect::new(DRAW_X_START - 10, DRAW_TOP - 12, 201, 212),
                minimum_changed_pixels: 0,
                exclude_cursor_from_effect: false,
                repeat_target: RepeatTargetPolicy::Allowed,
            },
        ),
        SpiderTarget::Play { column, top, bottom }


            if (1..=10).contains(&column) && top >= PLAY_SCAN_TOP && bottom <= TOOLBAR_TOP
                && bottom.checked_sub(top).is_some_and(|height|
                    height >= if bottom == TOOLBAR_TOP { MINIMUM_CLIPPED_SOURCE_HEIGHT }
                        else { MINIMUM_SOURCE_HEIGHT }) => {
            let x = PLAY_X[usize::from(column - 1)];
            let point = PixelPoint::new((x + CARD_WIDTH / 2) as i32,
                (bottom - SOURCE_CLICK_BOTTOM_INSET) as i32);
            (
                point,
                ActionSpecification {
                    operation: InputOperation::Click(point),
                    animation_class: AnimationClass::Spider,
                    effect_bounds: PixelRect::new(x, top, CARD_WIDTH, bottom - top),
                    minimum_changed_pixels: 0,
                    exclude_cursor_from_effect: false,
                    repeat_target: RepeatTargetPolicy::Allowed,
                },
            )
        }
        _ => return None,
    };

    Some(GuidedAction { target: ActionTarget::Spider(target), anchor, specification })
}


/// Check DRAW first, then the lowest PLAY source, with left-to-right ties.
/// Collapsed suits and dashed destinations are display-only. Selection consumes
/// this single fresh frame and does not infer the success of a previous move.
pub fn analyse(frame: &CapturedFrame) -> Result<FrameAnalysis, HaloDetectionError> {
    let mut selected = None;


    if is_gameplay_scene(frame)? && solver_active(frame)? {


        if draw_is_highlighted(frame) {
            selected = Some(SpiderTarget::Draw);
        } else {
            let stock_present = stock_is_present(frame);
            let collapsed = collapsed_display(frame);


            for (index, x) in PLAY_X.into_iter().enumerate() {
                let scan_bottom = play_scan_bottom(x, stock_present, collapsed);


                if let Some(bounds) = find_source(frame, x, PLAY_SCAN_TOP, scan_bottom)
                    && source_card_is_visible(frame, x, bounds.bottom()) {
                    let candidate = SpiderTarget::Play {
                        column: index as u8 + 1,
                        top: bounds.y,
                        bottom: bounds.bottom(),
                    };


                    if selected.is_none_or(|prior| matches!(prior,
                        SpiderTarget::Play { bottom, .. } if bounds.bottom() > bottom)) {
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
    //! Native evidence and focused boundary/negative regressions.

    use super::*;
    use crate::capture::decode_png;


    /// Decode original native evidence, preserving the production capture path.
    fn fixture(code: u8) -> CapturedFrame {
        let bytes: &[u8] = match code {
            1 => include_bytes!("../tests/fixtures/spider-SP01.png"),
            2 => include_bytes!("../tests/fixtures/spider-SP02.png"),
            3 => include_bytes!("../tests/fixtures/spider-SP03.png"),
            4 => include_bytes!("../tests/fixtures/spider-SP04.png"),
            5 => include_bytes!("../tests/fixtures/spider-SP05.png"),
            6 => include_bytes!("../tests/fixtures/spider-SP06.png"),
            7 => include_bytes!("../tests/fixtures/spider-SP07.png"),
            8 => include_bytes!("../tests/fixtures/spider-SP08.png"),
            9 => include_bytes!("../tests/fixtures/spider-SP09.png"),
            10 => include_bytes!("../tests/fixtures/spider-SP10.png"),
            11 => include_bytes!("../tests/fixtures/spider-SP11.png"),
            12 => include_bytes!("../tests/fixtures/spider-SP12.png"),
            13 => include_bytes!("../tests/fixtures/spider-SP13.png"),
            14 => include_bytes!("../tests/fixtures/spider-SP14.png"),
            15 => include_bytes!("../tests/fixtures/spider-SP15.png"),
            17 => include_bytes!("../tests/fixtures/spider-SP17.png"),
            18 => include_bytes!("../tests/fixtures/spider-SP18.png"),
            19 => include_bytes!("../tests/fixtures/spider-SP19.png"),
            20 => include_bytes!("../tests/fixtures/spider-lower-toolbar.png"),
            21 => include_bytes!("../tests/fixtures/spider-no-draw-pile.png"),
            22 => include_bytes!("../tests/fixtures/spider-SP21.png"),
            23 => include_bytes!("../tests/fixtures/spider-SP22.png"),
            _ => panic!("unknown native Spider fixture"),
        };

        decode_png(bytes).expect("native Spider PNG")
    }


    /// Paint a bounded in-memory fixture copy; no new screenshot is generated.
    fn paint(frame: &mut CapturedFrame, bounds: PixelRect, rgb: [u8; 3]) {


        for y in bounds.y..bounds.bottom() {


            for x in bounds.x..bounds.right() {
                let offset = y as usize * frame.stride + x as usize * 4;
                frame.pixels[offset..offset + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
        }
    }


    /// Add exterior source rails and crossbars to a fixture copy.
    fn paint_source(frame: &mut CapturedFrame, x: u32, top: u32, bottom: u32) {
        let gold = [235, 195, 90];
        paint(frame, PixelRect::new(x - 5, top + 3, 3, bottom - top - 6), gold);
        paint(frame, PixelRect::new(x + CARD_WIDTH + 2, top + 3, 3, bottom - top - 6), gold);
        paint(frame, PixelRect::new(x + 18, top, CARD_WIDTH - 36, 1), gold);
        paint(frame, PixelRect::new(x + 18, bottom - 1, CARD_WIDTH - 36, 1), gold);
    }


    /// Copy only measured neutral cursor pixels into decoded test memory.
    /// The supplied original PNGs are never modified or re-encoded.
    fn copy_actual_cursor(source: &CapturedFrame, frame: &mut CapturedFrame, hotspot: PixelPoint) -> usize {
        let bounds = PixelRect::new(20, 920, 34, 47);
        let mut copied = 0;


        for y in bounds.y..bounds.bottom() {


            for x in bounds.x..bounds.right() {
                let rgb = pixel_rgb(source, x, y).expect("measured cursor pixel inside native frame");
                let minimum = rgb.into_iter().min().unwrap();
                let maximum = rgb.into_iter().max().unwrap();


                if maximum - minimum > 12 || !(minimum >= 160 || maximum <= 65) {
                    continue;
                }

                let target_x = u32::try_from(hotspot.x + x as i32 - 21).expect("cursor x is nonnegative");
                let target_y = u32::try_from(hotspot.y + y as i32 - 921).expect("cursor y is nonnegative");
                assert!(target_x < frame.width && target_y < frame.height);
                let offset = target_y as usize * frame.stride + target_x as usize * 4;
                frame.pixels.get_mut(offset..offset + 3)
                    .expect("cursor destination inside decoded native frame").copy_from_slice(&rgb);
                copied += 1;
            }
        }

        copied
    }


    /// The previous source click can hide the newly exposed Jack's bottom bar.
    /// These are supplied reconstructed boards, not an exact failed runtime PNG.
    #[test]
    fn actual_cursor_at_previous_queen_click_hides_jack_source() {
        let queen = fixture(22);
        let queen_action = canonical_action(SpiderTarget::Play {
            column: 6, top: 145, bottom: 349,
        }).unwrap();
        assert_eq!(analyse(&queen).unwrap().prediction, PredictedAction::Action(queen_action));
        let queen_click = PixelPoint::new(1_045, 309);
        assert_eq!(queen_action.operation(), InputOperation::Click(queen_click));

        let clear_jack = fixture(23);
        let jack_action = canonical_action(SpiderTarget::Play {
            column: 6, top: 123, bottom: 327,
        }).unwrap();
        assert_eq!(analyse(&clear_jack).unwrap().prediction, PredictedAction::Action(jack_action));

        let mut covered_jack = fixture(23);
        assert!(copy_actual_cursor(&queen, &mut covered_jack, queen_click) > 0);
        assert!(is_gameplay_scene(&covered_jack).unwrap());
        assert!(solver_active(&covered_jack).unwrap());
        assert_eq!(analyse(&covered_jack).unwrap().prediction, PredictedAction::NoHighlight,
            "actual cursor pixels cover the next source's required bottom crossbar");
        assert_eq!(analyse(&clear_jack).unwrap().prediction, PredictedAction::Action(jack_action),
            "a fresh clear source frame retains its original source geometry");
    }


    /// Parking must preserve gameplay gates and recommendations with a visible cursor.
    #[test]
    fn actual_cursor_park_preserves_all_native_gameplay_sources() {
        let cursor_source = fixture(22);
        let cursor_bounds = PixelRect::new(POINTER_PARK.x as u32, POINTER_PARK.y as u32, 33, 46);


        for code in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 19, 20, 21, 22, 23] {
            let original = fixture(code);
            assert!(is_gameplay_scene(&original).unwrap(), "native gameplay fixture {code}");
            assert!(fraction_at_least(&original, cursor_bounds, is_felt, 1_000),
                "entire parked cursor bounds are clear felt in fixture {code}");
            let prediction = analyse(&original).unwrap().prediction;
            let solver = solver_active(&original).unwrap();
            let mut parked = original.clone();
            assert!(copy_actual_cursor(&cursor_source, &mut parked, POINTER_PARK) > 0);
            assert!(is_gameplay_scene(&parked).unwrap(), "parked gameplay fixture {code}");
            assert_eq!(solver_active(&parked).unwrap(), solver, "parked Solver fixture {code}");
            assert_eq!(analyse(&parked).unwrap().prediction, prediction,
                "parked cursor preserves the source prediction in fixture {code}");
        }
    }


    /// Native solid sources remain one action despite per-card rounded outlines.
    #[test]
    fn native_sources_and_dynamic_geometry() {
        let expected = [
            (2, 2, 212, 416), (3, 10, 249, 714), (4, 5, 104, 305),
            (5, 3, 190, 394), (6, 4, 145, 349), (7, 1, 477, 947),
            (9, 8, 154, 359), (10, 3, 167, 947), (11, 4, 154, 359),
            (12, 10, 231, 542), (13, 5, 208, 466), (14, 1, 468, 724),
            (20, 5, 783, 947), (21, 7, 370, 574),
        ];


        for (code, column, top, bottom) in expected {
            let target = SpiderTarget::Play { column, top, bottom };
            let action = canonical_action(target).unwrap();
            assert_eq!(analyse(&fixture(code)).unwrap().prediction,
                PredictedAction::Action(action), "SP fixture {code}");
            assert_eq!(action.minimum_changed_pixels(), 0);
            assert_eq!(action.repeat_target_policy(), RepeatTargetPolicy::Allowed);
            let InputOperation::Click(point) = action.operation() else {
                panic!("Spider PLAY must use one click");
            };
            assert!(point.y < TOOLBAR_TOP as i32);
        }

        let draw = canonical_action(SpiderTarget::Draw).unwrap();
        assert_eq!(analyse(&fixture(8)).unwrap().prediction, PredictedAction::Action(draw));
        assert_eq!(draw.operation(), InputOperation::PressDrawKey);
        assert_eq!(draw.specification.animation_class, AnimationClass::SpiderDraw);
    }


    /// Off-Solver boards and terminal panels cannot authorise gameplay input.
    #[test]
    fn native_scene_and_solver_controls_are_separate() {


        for code in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, 18, 19, 20, 21, 22, 23] {
            let frame = fixture(code);
            let gameplay = ![15, 17, 18].contains(&code);
            assert_eq!(is_gameplay_scene(&frame).unwrap(), gameplay, "SP fixture {code}");
            assert_eq!(solver_active(&frame).unwrap(), gameplay && ![1, 19].contains(&code),
                "SP fixture {code}");


            if !gameplay || [1, 19].contains(&code) {
                assert_eq!(analyse(&frame).unwrap().prediction, PredictedAction::NoHighlight);
            }
        }


        for bytes in [
            include_bytes!("../tests/fixtures/freecell-FC02.png").as_slice(),
            include_bytes!("../tests/fixtures/klondike/K95.png").as_slice(),
        ] {
            assert!(!is_gameplay_scene(&decode_png(bytes).unwrap()).unwrap());
        }
    }


    /// Toolbar pixels cannot change a clipped source's authority or click.
    #[test]
    fn clipped_sources_ignore_every_toolbar_pixel_and_card_artwork() {


        for code in [7, 10, 20] {
            let original = fixture(code);
            let prediction = analyse(&original).unwrap().prediction;


            for rgb in [[0, 0, 0], [255, 255, 255], [235, 195, 90], [20, 120, 75]] {
                let mut altered = fixture(code);
                paint(&mut altered, PixelRect::new(0, TOOLBAR_TOP, 1_920, 1_080 - TOOLBAR_TOP), rgb);
                assert_eq!(analyse(&altered).unwrap().prediction, prediction, "SP fixture {code}");
            }
        }

        let mut queen = fixture(20);
        paint(&mut queen, PixelRect::new(840, 820, 70, 110), [0, 0, 0]);
        assert_eq!(analyse(&queen).unwrap().prediction, analyse(&fixture(20)).unwrap().prediction,
            "central artwork is not clipped-card authority");
    }


    /// Dashed destinations and an exhausted Draw area never become Draw input.
    #[test]
    fn destination_and_display_areas_are_not_sources() {
        let mut no_stock = fixture(21);
        paint(&mut no_stock, PixelRect::new(PLAY_X[6] - 10, 360, CARD_WIDTH + 21, 224), [20, 120, 75]);
        assert_eq!(analyse(&no_stock).unwrap().prediction, PredictedAction::NoHighlight,
            "the remaining low dashed destination is not a source");
        assert!(!draw_is_highlighted(&fixture(21)));
        assert_eq!(play_scan_bottom(PLAY_X[8], false, None), TOOLBAR_TOP);
        assert_eq!(play_scan_bottom(PLAY_X[8], true, None), DRAW_TOP);

        let display = collapsed_display(&fixture(14)).expect("native collapsed packets");
        assert_eq!(play_scan_bottom(PLAY_X[1], false, Some(display)), DRAW_TOP);
        assert_eq!(play_scan_bottom(PLAY_X[7], false, Some(display)), TOOLBAR_TOP);
        assert!(collapsed_display(&fixture(10)).is_none(),
            "a long PLAY3 source cannot reserve absent collapsed packets");

        let mut draw = fixture(8);
        paint(&mut draw, PixelRect::new(1_535, 770, 90, 140), [255, 255, 255]);
        assert!(!draw_is_highlighted(&draw), "a white card face is not stock-back evidence");

        let mut draw = fixture(8);
        paint_source(&mut draw, PLAY_X[0], 300, 505);
        assert_eq!(analyse(&draw).unwrap().prediction,
            PredictedAction::Action(canonical_action(SpiderTarget::Draw).unwrap()),
            "DRAW retains priority over a simultaneous PLAY source");

        let mut empty_stock_area = fixture(21);
        paint(&mut empty_stock_area, PixelRect::new(PLAY_X[6] - 10, 360, CARD_WIDTH + 21, 224), [20, 120, 75]);
        paint(&mut empty_stock_area, PixelRect::new(PLAY_X[8], 740, CARD_WIDTH, TOOLBAR_TOP - 740), [255, 255, 255]);
        paint_source(&mut empty_stock_area, PLAY_X[8], 737, 1_010);
        assert_eq!(analyse(&empty_stock_area).unwrap().prediction,
            PredictedAction::Action(canonical_action(SpiderTarget::Play {
                column: 9, top: 737, bottom: TOOLBAR_TOP,
            }).unwrap()), "PLAY9 can extend through the exhausted-stock area");
        assert!(!draw_is_highlighted(&empty_stock_area));
    }


    /// Reject invalid storage, dimensions, source numbers and geometric ranges.
    #[test]
    fn malformed_frames_and_noncanonical_geometry_are_rejected() {
        let mut frame = fixture(1);
        frame.pixels.pop();
        assert_eq!(analyse(&frame), Err(HaloDetectionError::InvalidFrameLayout));
        let mut frame = fixture(1);
        frame.width -= 1;
        assert_eq!(analyse(&frame), Err(HaloDetectionError::BoundsOutsideFrame));


        for target in [
            SpiderTarget::Play { column: 0, top: 200, bottom: 500 },
            SpiderTarget::Play { column: 11, top: 200, bottom: 500 },
            SpiderTarget::Play { column: 1, top: 101, bottom: 500 },
            SpiderTarget::Play { column: 1, top: 500, bottom: 400 },
            SpiderTarget::Play { column: 1, top: 200, bottom: 948 },
            SpiderTarget::Play { column: 1, top: 882, bottom: 947 },
        ] {
            assert!(canonical_action(target).is_none());
        }
    }
}
