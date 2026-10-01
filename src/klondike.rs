//! Klondike Draw 1 Solver targets and conservative, mode-owned effect evidence.
//!
//! Geometry is measured from Charlie's 1920x1080 captures K01-K40. A source
//! needs a continuous lower gold edge, matching exterior rails and a bright
//! card interior. Dashed dark destinations never grant click authority. The
//! outline primitive groups a connected run into one target. No card ranks,
//! legal moves or restart behaviour are inferred here. Completion uses a separate
//! Klondike one-board progress observation, never the shared three-board threshold.

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


/// Klondike-only terminal artwork, one-board win evidence and guarded controls.
#[path = "klondike_terminal.rs"]
pub(crate) mod terminal;


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


/// Exclusive scan limit including K38's lower border and subsequent clear rows.
/// K38's exterior rails end at row987 and its lower border ends at row990.
/// This remains above the Windows taskbar and never extends input/proof bounds.
const TABLEAU_OUTLINE_BOTTOM: u32 = 994;


/// Measured Undo All icon/shadow over K38's three lower source-border rows.
/// Only these 28 by 3 pixels may be unavailable to lower-edge recognition;
/// every remaining edge pixel, closed top and exterior rail is still required.
const TOOLBAR_EDGE_OCCLUSION: PixelRect = PixelRect::new(1_308, 988, 28, 3);


/// Red Undo All artwork independently visible over K38's border occlusion.
/// K38 supplies 68 matching pixels here; missing control artwork removes the mask.
const TOOLBAR_ICON_SUPPORT: PixelRect = PixelRect::new(1_314, 980, 14, 11);


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


/// Interior right progress-bar probe, measured at the same native coordinates
/// in K24/K25. Its 38 by 2 pixels are checked individually, never by a majority.
const COMPLETION_PROGRESS_BOUNDS: PixelRect = PixelRect::new(1_050, 87, 38, 2);


/// Number of original progress-probe pixels; no absent/unknown pixel is gold.
const COMPLETION_PROGRESS_PIXELS: usize = 76;


/// Largest channel value classifying the measured unfilled interior as black.
const COMPLETION_BLACK_MAXIMUM: u8 = 32;


/// Required paper/ink changes in each of two opposed RIGHT-card corner patches.
/// Both corners must retain white paper; a dark guide cannot prove replacement.
const MINIMUM_CORNER_CHANGE: usize = 48;


/// Retained neutral paper required by the occupied-foundation replacement routes.
/// K33/K34 retain 12,483 of 14,300 inset pixels; unknown artwork fails closed.
const MINIMUM_STABLE_PAPER_PERMILLE: usize = 800;


/// A guide's existing paper and gold rails must not shift by this channel delta.
/// This is a stability guard, not a reduced ordinary material-change threshold.
const GUIDE_STABILITY_DELTA: u8 = 8;


/// Geometry-bearing target identity; no persistent card-rank state is implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KlondikeTarget {
    /// Highlighted non-empty stock, delivered with Charlie's confirmed D shortcut.
    Draw,
    /// Highlighted empty stock, delivered as one click on the recycle area.
    Recycle,
    /// Evidenced Solve control; one click requests completion, then read-only checks.
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
            Self::Solve => formatter.write_str("Solve control (one-board completion request)"),
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


/// Klondike owns its dynamic detector and independent one-board completion policy.
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


/// Green felt under the next translucent destination guide in K22/K23.
/// Retained chroma distinguishes it from neutral dimmed paper and opaque black;
/// this darker range is separate from the ordinary bright-felt predicate.
fn is_dimmed_felt([red, green, blue]: [u8; 3]) -> bool {
    (15..65).contains(&green) && i16::from(green) - i16::from(red) >= 8
        && i16::from(green) - i16::from(blue) >= 6
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


/// Positive red Undo All artwork; neutral/gold source edges cannot supply it.
fn is_toolbar_undo_red([red, green, blue]: [u8; 3]) -> bool {
    red >= 100 && green <= 90 && blue <= 80 && i16::from(red) - i16::from(green) >= 40
}


/// Positive warm-gold interior fill measured in the partial progress of K25.
/// Bright white, green felt, neutral grey and merely non-black pixels fail.
fn is_progress_gold([red, green, blue]: [u8; 3]) -> bool {
    red >= 190 && green >= 150 && (80..=170).contains(&blue)
        && i16::from(red) - i16::from(green) >= 20
        && i16::from(green) - i16::from(blue) >= 55
}


/// Independent Klondike one-board completion observation, not input authority.
/// A worker must obtain repeated fresh evidence before recording game completion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompletionEvidence {
    /// True for independently validated completed Klondike terminal artwork,
    /// or intact native scene/banner with no action, zero black and all 76 gold.
    /// The worker still requires repeated fresh observations before accepting it.
    pub complete_candidate: bool,
    /// Probe pixels whose red, green and blue channels are each at most 32.
    pub black_pixels: usize,
    /// Probe pixels matching the measured warm-gold interior fill relation.
    pub gold_pixels: usize,
    /// Independent authority used, or why neither terminal artwork nor full bar
    /// currently establishes a positive completion candidate.
    pub reason: &'static str,
}


impl fmt::Display for CompletionEvidence {


    /// Log raw black/fill counts without promoting absence into a positive result.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter,
            "complete candidate={}; BLACK={}/{COMPLETION_PROGRESS_PIXELS}, positive gold={}/{COMPLETION_PROGRESS_PIXELS}; {}",
            self.complete_candidate, self.black_pixels, self.gold_pixels, self.reason)
    }
}


/// Measure the literal no-black one-board policy authorised by Charlie.
///
/// Counts remain diagnostic on a valid but unknown scene. Invalid storage or
/// geometry is an error; blank/dimmed/partial bars, absent Solver rails and any
/// recognised action (including Solve) cannot establish progress-bar completion.
/// Independently validated K26-K28 terminal artwork over a completed board also
/// demonstrates the one-board win; its dimmed bar counts stay diagnostic. K25
/// remains a real unfinished negative. No transient Perfect frame was supplied.
pub fn completion_evidence(
    frame: &CapturedFrame,
) -> Result<CompletionEvidence, HaloDetectionError> {
    validate_frame(frame)?;
    validate_bounds(frame, COMPLETION_PROGRESS_BOUNDS)?;
    let black_pixels = count_pixels(frame, COMPLETION_PROGRESS_BOUNDS, |rgb| {
        rgb.into_iter().all(|channel| channel <= COMPLETION_BLACK_MAXIMUM)
    }) as usize;
    let gold_pixels = count_pixels(frame, COMPLETION_PROGRESS_BOUNDS, is_progress_gold) as usize;
    let terminal_completion = terminal::classify_terminal(frame)?
        .is_some_and(terminal::TerminalStage::demonstrates_game_win);
    let positive_progress = black_pixels == 0 && gold_pixels == COMPLETION_PROGRESS_PIXELS;
    let progress_completion = positive_progress && is_gameplay_scene(frame)?
        && has_solver_banner(frame) && select_target(frame)?.is_none();
    let complete_candidate = terminal_completion || progress_completion;


    let reason = if terminal_completion {
        "independent completed Klondike terminal artwork and completed-board background"
    } else if progress_completion {
        "literal full-gold progress with intact Klondike scene/Solver banner and no higher-priority action"
    } else {
        "neither a validated completed terminal scene nor full-gold/no-action progress is present"
    };

    Ok(CompletionEvidence { complete_candidate, black_pixels, gold_pixels, reason })
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
/// This primitive starts at the bottom, normally requires a continuous 96-pixel
/// lower edge, follows both exterior rails and probes paper above the toolbar.
/// Only K38's measured column-6 icon overlap may obscure 28 lower-edge pixels;
/// positive icon artwork and all remaining 68 gold pixels are then mandatory.
/// A black destination interior fails even if some gold
/// dashes align. Card-rank artwork is not required to be white at one exact pixel.
/// The returned rectangle groups all connected highlighted cards as one action.
/// An interior crossbar cannot close a block while its exterior rails continue.
/// K19/K38 permit darker closed edges in the measured toolbar overlap strip.
/// K38's known icon occlusion cannot provide an edge, top or rail of its own.
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
    // K19's last rail row is958; K38's is987. Subsequent clear rows are observed.
    if (x - 9..x).any(|xx| pixel_rgb(frame, xx, bottom - 1).is_some_and(is_rail_gold))
        || (right..right + 10).any(|xx| pixel_rgb(frame, xx, bottom - 1).is_some_and(is_rail_gold))
    {
        return Ok(None);
    }

    let last_rail_row = (scan.y..bottom).rev().find(|&row| paired_rails(row));


    for y in (scan.y..bottom).rev() {
        let toolbar_overlap = require_card_face && (TOOLBAR_TOP..TABLEAU_OUTLINE_BOTTOM).contains(&y);
        let known_icon_occlusion = toolbar_overlap && x == 1_230 && scan.width == CARD_WIDTH
            && bottom >= TABLEAU_OUTLINE_BOTTOM
            && (TOOLBAR_EDGE_OCCLUSION.y..TOOLBAR_EDGE_OCCLUSION.bottom()).contains(&y)
            && count_pixels(frame, TOOLBAR_ICON_SUPPORT, is_toolbar_undo_red) as usize >= MINIMUM_CORNER_CHANGE;
        let lower_edge = |rgb| is_edge_gold(rgb)
            || (toolbar_overlap && is_rail_gold(rgb));


        if y < scan.y + 170
            || !pixel_rgb(frame, x + scan.width / 2, y).is_some_and(lower_edge)
            || last_rail_row.is_some_and(|row| row >= y + 10)
            || (toolbar_overlap && last_rail_row.is_none_or(|row| row.abs_diff(y) > 10))
        {
            continue;
        }


        let mut visible_edge_pixels = 0;
        let closed_lower_edge = (x + 18..right - 18).all(|xx| {


            if known_icon_occlusion && TOOLBAR_EDGE_OCCLUSION.contains(PixelPoint::new(xx as i32, y as i32)) {
                return true;
            }

            visible_edge_pixels += 1;
            pixel_rgb(frame, xx, y).is_some_and(lower_edge)
        });


        if !closed_lower_edge || visible_edge_pixels < 48
            || (known_icon_occlusion && visible_edge_pixels != 68)
        {
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


        let face_bottom = if require_card_face { y.min(TOOLBAR_TOP) } else { y };
        let face_probe = PixelRect::new(x + 18, face_bottom - 36, scan.width - 36, 12);


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
            animation_class: if target == KlondikeTarget::Solve {
                AnimationClass::KlondikeSolve
            } else {
                AnimationClass::Klondike
            },
            effect_bounds: bounds,
            minimum_changed_pixels: MINIMUM_CONTENT_CHANGE,
            exclude_cursor_from_effect: true,
            repeat_target: RepeatTargetPolicy::Allowed,
        },
    })
}


/// Select DRAW, RIGHT HALO, RIGHT Solve, lowest tableau source, then SUIT.
/// Completion evidence is independent and evaluated only after these priorities.
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


    if solver_active {


        for offset in (0..=2_u8).rev() {


            if find_solid_card_source(frame, upper_scan(558 + WASTE_FAN_STEP * u32::from(offset)))?.is_some() {
                return Ok(Some(KlondikeTarget::Waste { offset }));
            }
        }
    }


    if has_solve_control(frame) {
        return Ok(Some(KlondikeTarget::Solve));
    }


    if !solver_active {
        return Ok(None);
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


/// Count formerly white tableau paper exposing chromatic dimmed felt in the
/// same inset, gold/cursor mask and material-change bounds as ordinary proof.
/// The next dashed guide may darken exposed felt after a card moves; darkening
/// retained white paper alone cannot satisfy this colour transition.
fn tableau_dimmed_felt_changes(
    before: &CapturedFrame,
    after: &CapturedFrame,
    bounds: PixelRect,
    action: GuidedAction,
) -> usize {


    let InputOperation::Click(point) = action.operation() else { return 0; };
    let mut count = 0;


    for y in bounds.y + 16..bounds.y + bounds.height - 16 {


        for x in bounds.x + 16..bounds.x + bounds.width - 16 {


            if (i64::from(x) - i64::from(point.x)).abs() < 48
                && (i64::from(y) - i64::from(point.y)).abs() < 48
            {
                continue;
            }


            let Some(first) = pixel_rgb(before, x, y) else { continue; };


            let Some(second) = pixel_rgb(after, x, y) else { continue; };


            if is_rail_gold(first) || is_rail_gold(second) {
                continue;
            }


            if is_white(first) && is_dimmed_felt(second)
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


/// Paper/ink identity replacement in RIGHT, SUIT or one complete tableau card
/// source. Opposed patches include detail lost by the ordinary 16-pixel inset.
/// Both retain white paper before/after; a dark guide or pointer change fails.
/// Tableau is limited to the measured 180..190-pixel single-card outline, uses
/// its fresh source top, excludes gold and additionally needs 512 materially
/// changed source pixels. Each tableau corner needs at least 48 paper-to-ink
/// and 48 ink-to-paper pixels, rejecting one-way dimming or whitening. SUIT uses
/// the same strict policy in its fixed upper-card geometry. RIGHT's
/// established total-corner policy remains. A separated destination is mandatory.
fn card_identity_changes(
    before: &CapturedFrame,
    after: &CapturedFrame,
    source: PixelRect,
    action: GuidedAction,
) -> [[usize; 2]; 2] {


    let (top, tableau) = match action.target {
        ActionTarget::Klondike(KlondikeTarget::Waste { .. }) => (UPPER_Y, false),
        ActionTarget::Klondike(KlondikeTarget::Foundation { .. }) => (UPPER_Y, true),
        ActionTarget::Klondike(KlondikeTarget::Tableau { top, bottom, .. })


            if bottom.checked_sub(top).is_some_and(|height| (180..=190).contains(&height)) =>
        {
            (u32::from(top), true)
        }
        _ => return [[0, 0], [0, 0]],
    };
    let patches = [
        PixelRect::new(source.x + 5, top + 5, 28, 44),
        PixelRect::new(source.x + 99, top + 126, 28, 44),
    ];


    // A recognised outline may overlap the toolbar, but its proof ROI never
    // does. A partly clipped corner is unavailable, not a smaller trusted patch.
    if patches.into_iter().any(|bounds| bounds.x < source.x || bounds.y < source.y
        || bounds.x + bounds.width > source.x + source.width
        || bounds.y + bounds.height > source.y + source.height
        || (tableau && bounds.y + bounds.height > TOOLBAR_TOP))
    {
        return [[0, 0], [0, 0]];
    }


    let InputOperation::Click(point) = action.operation() else { return [[0, 0], [0, 0]]; };

    patches.map(|bounds| {


        if !fraction_at_least(before, bounds, is_white, 500)
            || !fraction_at_least(after, bounds, is_white, 500)
        {
            return [0, 0];
        }

        let mut paper_to_ink = 0;
        let mut ink_to_paper = 0;


        for y in bounds.y..bounds.y + bounds.height {


            for x in bounds.x..bounds.x + bounds.width {


                if (i64::from(x) - i64::from(point.x)).abs() < 48
                    && (i64::from(y) - i64::from(point.y)).abs() < 48
                {
                    continue;
                }


                let Some(first) = pixel_rgb(before, x, y) else { continue; };


                let Some(second) = pixel_rgb(after, x, y) else { continue; };


                if tableau && (is_rail_gold(first) || is_rail_gold(second)) {
                    continue;
                }


                if first.into_iter().zip(second).any(|(a, b)| a.abs_diff(b) >= 48) {
                    paper_to_ink += usize::from(is_white(first) && is_card_ink(second));
                    ink_to_paper += usize::from(is_card_ink(first) && is_white(second));
                }
            }
        }

        [paper_to_ink, ink_to_paper]
    })
}


/// Two complete visible print patches for a single tableau card whose ordinary
/// lower identity corner crosses the toolbar. The existing corner function still
/// refuses a clipped patch. This separate K39/K40 policy moves the entire lower
/// 28 by 44 patch up to the proof boundary, never truncating it or reading y947.
/// Both opposed patches retain 50% white paper and both print directions use the
/// existing 48-pixel bound. The caller additionally requires 512 material source
/// pixels. This route supports fresh-HALO continuation only; it cannot establish
/// a verified recipient, complete input effect or game completion.
fn tableau_visible_identity_changes(
    before: &CapturedFrame,
    after: &CapturedFrame,
    source: PixelRect,
    action: GuidedAction,
) -> [[usize; 2]; 2] {


    let ActionTarget::Klondike(target @ KlondikeTarget::Tableau { top, bottom, .. }) = action.target else {
        return [[0, 0], [0, 0]];
    };


    if canonical_action(target) != Some(action)
        || !bottom.checked_sub(top).is_some_and(|height| (180..=190).contains(&height))
        || u32::from(bottom) <= TOOLBAR_TOP
        || u32::from(top) + 126 + 44 <= TOOLBAR_TOP
        || source.y != u32::from(top)
        || source.width != CARD_WIDTH
        || source.bottom() != TOOLBAR_TOP
    {
        return [[0, 0], [0, 0]];
    }

    let patches = [
        PixelRect::new(source.x + 5, source.y + 5, 28, 44),
        PixelRect::new(source.x + 99, TOOLBAR_TOP - 44, 28, 44),
    ];


    if patches[0].bottom() > patches[1].y
        || patches.into_iter().any(|bounds| bounds.x < source.x || bounds.y < source.y
            || bounds.right() > source.right() || bounds.bottom() > source.bottom()
            || validate_bounds(before, bounds).is_err() || validate_bounds(after, bounds).is_err())
    {
        return [[0, 0], [0, 0]];
    }


    let InputOperation::Click(point) = action.operation() else { return [[0, 0], [0, 0]]; };

    patches.map(|bounds| {


        if !fraction_at_least(before, bounds, is_white, 500)
            || !fraction_at_least(after, bounds, is_white, 500)
        {
            return [0, 0];
        }

        let mut directions = [0, 0];


        for y in bounds.y..bounds.bottom() {


            for x in bounds.x..bounds.right() {


                if (i64::from(x) - i64::from(point.x)).abs() < 48
                    && (i64::from(y) - i64::from(point.y)).abs() < 48
                {
                    continue;
                }


                let Some(first) = pixel_rgb(before, x, y) else { continue; };


                let Some(second) = pixel_rgb(after, x, y) else { continue; };


                if is_rail_gold(first) || is_rail_gold(second) {
                    continue;
                }


                if first.into_iter().zip(second).any(|(a, b)| a.abs_diff(b) >= 48) {
                    directions[0] += usize::from(is_white(first) && is_card_ink(second));
                    directions[1] += usize::from(is_card_ink(first) && is_white(second));
                }
            }
        }

        directions
    })
}


/// Neutral paper under the unchanged dark guide measured in K33/K34.
/// Ordinary white cards, opaque black and chromatic felt cannot satisfy it.
fn is_dimmed_paper(rgb: [u8; 3]) -> bool {
    let minimum = rgb.into_iter().min().unwrap_or(0);
    let maximum = rgb.into_iter().max().unwrap_or(255);
    minimum >= 16 && maximum <= 100 && maximum - minimum <= 6
}


/// Dimmed red printed detail, measured from the newly received middle heart.
/// Black/white cursor pixels and neutral guide shading do not qualify. A dark
/// black-pip replacement remains unsupported by this specialised fallback.
fn is_dimmed_red_ink([red, green, blue]: [u8; 3]) -> bool {
    (20..=100).contains(&red) && i16::from(red) - i16::from(green) >= 15
        && i16::from(red) - i16::from(blue) >= 15
        && u16::from(green) * 2 < u16::from(red)
        && u16::from(blue) * 2 < u16::from(red)
}


/// Bright neutral card paper; chromatic tint and the dark guide are excluded.
fn is_bright_neutral_paper(rgb: [u8; 3]) -> bool {
    is_white(rgb) && rgb.into_iter().max().unwrap_or(255)
        - rgb.into_iter().min().unwrap_or(0) <= 6
}


/// Inspect an unprinted paper field rather than antialiased glyph edges.
/// Callers use the sixteen-pixel card inset, so this 3 by 3 neighborhood stays
/// within the validated face and cannot reach an outline or toolbar pixel.
fn retained_bright_paper_field(
    before: &CapturedFrame,
    after: &CapturedFrame,
    x: u32,
    y: u32,
) -> bool {
    (y - 1..=y + 1).all(|yy| (x - 1..=x + 1).all(|xx| {
        pixel_rgb(before, xx, yy).is_some_and(is_bright_neutral_paper)
            && pixel_rgb(after, xx, yy).is_some_and(is_bright_neutral_paper)
    }))
}


/// SUIT source replacement evidence in the fixed bright upper-card face.
/// It needs independent material source and tableau recipient change; printed
/// detail alone cannot establish input effect or game completion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FoundationSourceEvidence {
    /// Newly printed red/black detail outside the commanded cursor mask.
    pub new_ink: usize,
    /// Previously printed detail becoming bright neutral paper.
    pub cleared_ink: usize,
    /// Bright neutral paper retained at the same positions before and after.
    pub stable_paper: usize,
    /// Unprinted 3 by 3 paper fields changing by eight or more per channel.
    pub changed_paper_fields: usize,
    /// At least 80% occupied neutral paper before, after and in common.
    pub paper_supported: bool,
    /// Material source, both print directions and independent paper guards pass.
    pub verified: bool,
}


/// Recognise printed replacement in an occupied SUIT source whose background
/// remains bright and neutral. The source's material 512-pixel gate is retained.
/// Both red/black print directions use the existing 48-pixel detail bound and
/// exclude gold and the commanded cursor. Neutral-field stability rejects
/// guide fading/recolouring without counting antialiased ink edges as paper.
fn foundation_source_print_changes(
    before: &CapturedFrame,
    after: &CapturedFrame,
    bounds: PixelRect,
    action: GuidedAction,
    material_changed: usize,
) -> FoundationSourceEvidence {
    let mut evidence = FoundationSourceEvidence::default();


    let InputOperation::Click(point) = action.operation() else { return evidence; };
    let mut before_paper = 0;
    let mut after_paper = 0;
    let mut before_ink = 0;
    let mut after_ink = 0;
    let mut eligible = 0;


    for y in bounds.y + 16..bounds.y + bounds.height - 16 {


        for x in bounds.x + 16..bounds.x + bounds.width - 16 {


            if (i64::from(x) - i64::from(point.x)).abs() < 48
                && (i64::from(y) - i64::from(point.y)).abs() < 48
            {
                continue;
            }


            let Some(first) = pixel_rgb(before, x, y) else { continue; };


            let Some(second) = pixel_rgb(after, x, y) else { continue; };


            if is_rail_gold(first) || is_rail_gold(second) {
                continue;
            }

            eligible += 1;
            let first_paper = is_bright_neutral_paper(first);
            let second_paper = is_bright_neutral_paper(second);
            let first_ink = is_card_ink(first);
            let second_ink = is_card_ink(second);
            before_paper += usize::from(first_paper);
            after_paper += usize::from(second_paper);
            before_ink += usize::from(first_ink);
            after_ink += usize::from(second_ink);
            evidence.stable_paper += usize::from(first_paper && second_paper);


            if retained_bright_paper_field(before, after, x, y) {
                evidence.changed_paper_fields += usize::from(first.into_iter().zip(second)
                    .any(|(a, b)| a.abs_diff(b) >= GUIDE_STABILITY_DELTA));
            }


            if first.into_iter().zip(second).any(|(a, b)| a.abs_diff(b) >= 48) {
                evidence.new_ink += usize::from(first_paper && second_ink);
                evidence.cleared_ink += usize::from(first_ink && second_paper);
            }
        }
    }

    evidence.paper_supported = eligible != 0 && [before_paper, after_paper, evidence.stable_paper]
        .into_iter().all(|count| count * 1_000 >= eligible * MINIMUM_STABLE_PAPER_PERMILLE)
        && before_ink >= MINIMUM_CORNER_CHANGE && after_ink >= MINIMUM_CORNER_CHANGE;
    evidence.verified = material_changed >= MINIMUM_CONTENT_CHANGE && evidence.paper_supported
        && evidence.changed_paper_fields == 0
        && evidence.new_ink >= MINIMUM_CORNER_CHANGE
        && evidence.cleared_ink >= MINIMUM_CORNER_CHANGE;
    evidence
}


/// Evidence for an occupied foundation retaining its existing dark guide while
/// new red print appears. This never supplies source or completion authority.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FoundationPrintEvidence {
    /// One-based foundation; zero denotes no eligible recipient observation.
    pub column: u8,
    /// Newly printed red pixels in the left and right inset halves, separately.
    pub new_ink_halves: [usize; 2],
    /// Neutral dimmed paper present at the same positions before and after.
    pub stable_paper: usize,
    /// Retained neutral paper changing by at least the guide-stability delta.
    pub changed_paper: usize,
    /// Both images retain occupied dimmed paper and printed red detail.
    pub paper_supported: bool,
    /// All four established guide rails retain their gold pattern and colour.
    pub guide_stable: bool,
    /// Both print halves and every independent paper/guide guard have passed.
    pub verified: bool,
}


/// Require the existing dashed guide on all four edges, with no newly gold,
/// missing gold or materially recoloured gold pixels. This accepts only the
/// fixed 132 by 175 occupied-foundation geometry supplied by the caller.
fn foundation_guide_stable(
    before: &CapturedFrame,
    after: &CapturedFrame,
    bounds: PixelRect,
) -> bool {
    let rails = [
        PixelRect::new(bounds.x + 3, bounds.y + 3, bounds.width - 6, 10),
        PixelRect::new(bounds.x + 3, bounds.y + bounds.height - 13, bounds.width - 6, 10),
        PixelRect::new(bounds.x + 3, bounds.y + 13, 10, bounds.height - 26),
        PixelRect::new(bounds.x + bounds.width - 13, bounds.y + 13, 10, bounds.height - 26),
    ];


    rails.into_iter().all(|rail| {
        let mut existing_gold = 0;


        for y in rail.y..rail.y + rail.height {


            for x in rail.x..rail.x + rail.width {


                let Some(first) = pixel_rgb(before, x, y) else { return false; };


                let Some(second) = pixel_rgb(after, x, y) else { return false; };
                let first_gold = is_rail_gold(first);
                let second_gold = is_rail_gold(second);


                if first_gold != second_gold || (first_gold && first.into_iter().zip(second)
                    .any(|(a, b)| a.abs_diff(b) >= GUIDE_STABILITY_DELTA))
                {
                    return false;
                }

                existing_gold += usize::from(first_gold);
            }
        }

        existing_gold >= MINIMUM_CORNER_CHANGE
    })
}


/// Inspect the original material-proof inset for new red print under an
/// unchanged occupied-foundation guide. It keeps the 48-channel material
/// delta, gold/cursor masks and measured guide/paper geometry. Each half needs
/// the existing 48-pixel printed-detail bound. At least 80% of paper must stay
/// neutral and dimmed, with no retained-paper change of eight or more. K33/K34
/// supplies the 360-pixel heart; no card rank or legal move is recognised.
fn foundation_print_changes(
    before: &CapturedFrame,
    after: &CapturedFrame,
    bounds: PixelRect,
    column: u8,
    action: GuidedAction,
) -> FoundationPrintEvidence {
    let mut evidence = FoundationPrintEvidence {
        column,
        guide_stable: foundation_guide_stable(before, after, bounds),
        ..FoundationPrintEvidence::default()
    };
    let mut before_paper = 0;
    let mut after_paper = 0;
    let mut before_ink = 0;
    let mut after_ink = 0;
    let mut eligible = 0;


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

            eligible += 1;
            let first_paper = is_dimmed_paper(first);
            let second_paper = is_dimmed_paper(second);
            let first_ink = is_dimmed_red_ink(first);
            let second_ink = is_dimmed_red_ink(second);
            before_paper += usize::from(first_paper);
            after_paper += usize::from(second_paper);
            before_ink += usize::from(first_ink);
            after_ink += usize::from(second_ink);


            if first_paper && second_paper {
                evidence.stable_paper += 1;
                evidence.changed_paper += usize::from(first.into_iter().zip(second)
                    .any(|(a, b)| a.abs_diff(b) >= GUIDE_STABILITY_DELTA));
            }


            if first_paper && second_ink && first.into_iter().zip(second)
                .any(|(a, b)| a.abs_diff(b) >= 48)
            {
                let half = usize::from(x >= bounds.x + bounds.width / 2);
                evidence.new_ink_halves[half] += 1;
            }
        }
    }

    evidence.paper_supported = eligible != 0 && [before_paper, after_paper, evidence.stable_paper]
        .into_iter().all(|count| count * 1_000 >= eligible * MINIMUM_STABLE_PAPER_PERMILLE)
        && before_ink >= MINIMUM_CORNER_CHANGE && after_ink >= MINIMUM_CORNER_CHANGE;
    evidence.verified = evidence.paper_supported && evidence.guide_stable
        && evidence.changed_paper == 0
        && evidence.new_ink_halves.into_iter().all(|count| count >= MINIMUM_CORNER_CHANGE);
    evidence
}


/// Measured effect evidence, including rejected counts for private session logs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectEvidence {
    /// True only when all target-specific source/destination conditions pass.
    pub verified: bool,
    /// The previous card source independently changed by an existing replacement
    /// rule. This never establishes a win. The visible toolbar-overlap route
    /// supports fresh-HALO continuation only, leaving complete effect unverified.
    /// Stock/Recycle/Solve/noncanonical/unsupported observations leave it false.
    pub source_replaced: bool,
    /// Interior source pixels changing materially outside gold and cursor masks.
    pub source_changed: usize,
    /// Source pixels positively revealing white paper or green felt.
    pub source_positive: usize,
    /// Formerly white tableau pixels exposing green felt beneath the next guide.
    pub source_dimmed_felt: usize,
    /// Total paper/ink changes in retained-white RIGHT/SUIT/single-card corners.
    pub source_identity_changed: usize,
    /// Per-corner paper/ink counts; both must pass the independent corner bound.
    pub source_identity_corners: [usize; 2],
    /// Per-corner [paper-to-ink, ink-to-paper] counts. A tableau replacement
    /// or SUIT replacement requires both directions, unlike one-way guides.
    pub source_identity_directions: [[usize; 2]; 2],
    /// Two full visible patches above the toolbar; zero outside complete
    /// single-card tableau overlap. With material source proof these counts
    /// support continuation only, never complete source/recipient verification.
    pub source_visible_identity_directions: [[usize; 2]; 2],
    /// Fixed bright SUIT face replacement; zero for all other source classes.
    pub source_foundation_print: FoundationSourceEvidence,
    /// Largest eligible destination interior change; waste count for stock input.
    pub destination_changed: usize,
    /// Strongest separately guarded occupied-foundation printed-detail candidate.
    pub destination_print: FoundationPrintEvidence,
    /// Explicit acceptance/refusal reason; unknown scenes never imply completion.
    pub reason: &'static str,
}


impl fmt::Display for EffectEvidence {


    /// Keep all authority-bearing measurements on one readable log line.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "verified={}; source replaced={}; source changed={}, positive={}, dimmed felt={} (each source proof requires {}), card corner ink/paper={:?} (each required {}), corner [paper-to-ink, ink-to-paper]={:?} (tableau/SUIT each required {}), visible-tableau full-patch directions={:?} (each required {} plus source 512; continuation only), bright-SUIT print={:?}, destination changed={} (required {}), dimmed-foundation print={:?}; {}",
            self.verified, self.source_replaced, self.source_changed, self.source_positive,
            self.source_dimmed_felt, MINIMUM_CONTENT_CHANGE, self.source_identity_corners,
            MINIMUM_CORNER_CHANGE, self.source_identity_directions, MINIMUM_CORNER_CHANGE,
            self.source_visible_identity_directions, MINIMUM_CORNER_CHANGE,
            self.source_foundation_print,
            self.destination_changed, MINIMUM_CONTENT_CHANGE, self.destination_print,
            self.reason,
        )
    }
}


/// Verify positive source and destination changes without copying another game's
/// repeated-HALO exception. Draw needs changed waste content; recycle needs the
/// stock back to reappear and the waste to empty. RIGHT can also show a changed
/// printed identity in two white-paper corners. SUIT and a single-card tableau
/// replacement may use the same corners plus 512 changed source pixels; removal
/// can expose chromatic green felt beneath a dark guide. An occupied foundation
/// with unchanged guide/paper may supply the separate evidenced red-print proof;
/// SUIT-return sources still require the ordinary 512-pixel recipient proof.
/// Solve has separate one-board completion observations and cannot use ordinary
/// card effect proof.
/// A complete single card crossing the toolbar may use two full visible print
/// patches above it plus 512 changed source pixels for separate continuation
/// evidence only. It never changes ordinary complete-effect verification.
pub fn inspect_effect(
    before: &CapturedFrame,
    after: &CapturedFrame,
    action: GuidedAction,
) -> Result<EffectEvidence, HaloDetectionError> {
    validate_frame(before)?;
    validate_frame(after)?;
    let mut evidence = EffectEvidence {
        verified: false,
        source_replaced: false,
        source_changed: 0,
        source_positive: 0,
        source_dimmed_felt: 0,
        source_identity_changed: 0,
        source_identity_corners: [0, 0],
        source_identity_directions: [[0, 0], [0, 0]],
        source_visible_identity_directions: [[0, 0], [0, 0]],
        source_foundation_print: FoundationSourceEvidence::default(),
        destination_changed: 0,
        destination_print: FoundationPrintEvidence::default(),
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
        evidence.reason = "Solve uses separate one-board completion observations";
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


    if matches!(target, KlondikeTarget::Tableau { .. }) {
        evidence.source_dimmed_felt = tableau_dimmed_felt_changes(before, after, source, action);
        evidence.source_visible_identity_directions = tableau_visible_identity_changes(before, after, source, action);
    }


    if matches!(target, KlondikeTarget::Foundation { .. }) {
        evidence.source_foundation_print = foundation_source_print_changes(
            before, after, source, action, evidence.source_changed,
        );
    }


    if matches!(target, KlondikeTarget::Waste { .. } | KlondikeTarget::Tableau { .. }
        | KlondikeTarget::Foundation { .. })
    {
        evidence.source_identity_directions = card_identity_changes(before, after, source, action);
        evidence.source_identity_corners = evidence.source_identity_directions
            .map(|directions| directions.into_iter().sum());
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


    if matches!(target, KlondikeTarget::Waste { .. } | KlondikeTarget::Tableau { .. }) {
        evidence.destination_print = (1..=4_u8).map(|column| {
            let bounds = PixelRect::new(894 + COLUMN_PITCH * u32::from(column - 1), UPPER_Y, CARD_WIDTH, CARD_HEIGHT);
            foundation_print_changes(before, after, bounds, column, action)
        }).max_by_key(|print| (print.verified, print.new_ink_halves.into_iter().sum::<usize>()))
            .unwrap_or_default();
    }

    let identity_verified = evidence.source_identity_corners.into_iter()
        .all(|count| count >= MINIMUM_CORNER_CHANGE)
        && (!matches!(target, KlondikeTarget::Tableau { .. } | KlondikeTarget::Foundation { .. })
            || (evidence.source_changed >= MINIMUM_CONTENT_CHANGE
                && evidence.source_identity_directions.into_iter().flatten()
                    .all(|count| count >= MINIMUM_CORNER_CHANGE)));
    let visible_source_replaced = evidence.source_changed >= MINIMUM_CONTENT_CHANGE
        && evidence.source_visible_identity_directions.into_iter().flatten()
            .all(|count| count >= MINIMUM_CORNER_CHANGE);
    let source_verified = evidence.source_positive >= MINIMUM_CONTENT_CHANGE
        || evidence.source_dimmed_felt >= MINIMUM_CONTENT_CHANGE || identity_verified
        || evidence.source_foundation_print.verified;
    evidence.source_replaced = source_verified || visible_source_replaced;
    let destination_verified = evidence.destination_changed >= MINIMUM_CONTENT_CHANGE
        || (evidence.destination_print.verified && evidence.source_changed >= MINIMUM_CONTENT_CHANGE);
    evidence.verified = source_verified && destination_verified;


    evidence.reason = match (source_verified, destination_verified) {
        (false, _) if visible_source_replaced =>
            "two full visible patches establish source replacement for fresh-HALO continuation only; recipient and complete effect unverified",
        (false, _) => "source removal/replacement not established; fresh HALO is insufficient",
        (true, false) => "source changed, but no independent destination change was established",
        (true, true) if evidence.destination_changed < MINIMUM_CONTENT_CHANGE =>
            "independent source replacement and new red print under unchanged occupied-foundation guide",
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
        assert_eq!(planned.specification.animation_class, AnimationClass::KlondikeSolve);
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


    /// All twenty-five original action captures plus K31/K32 retain the board.
    /// K26-K30 are tested separately as evidenced terminal/setup/new-deal scenes.
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
            Some(KlondikeTarget::Waste { offset: 1 }),
            Some(KlondikeTarget::Tableau { column: 5, top: 571, bottom: 761 }),
            Some(KlondikeTarget::Tableau { column: 6, top: 407, bottom: 597 }),
            Some(KlondikeTarget::Tableau { column: 4, top: 372, bottom: 562 }),
            Some(KlondikeTarget::Tableau { column: 5, top: 571, bottom: 761 }),
            Some(KlondikeTarget::Solve),
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


    /// Each actual/reconstructed before/after pair proves an effect; timestamps
    /// from Undo reconstruction must not be sorted into a supposed input trace.
    #[test]
    fn all_nine_recorded_action_pairs_have_independent_effects() {


        for (before_number, after_number) in [(2, 3), (4, 5), (6, 7), (8, 9), (10, 11), (17, 18), (20, 21), (21, 22), (31, 32)] {
            let before = fixture(before_number);
            let after = fixture(after_number);
            assert!(verify_effect(&before, &after, action(&before)).unwrap(), "K{before_number:02}->K{after_number:02}");
            assert!(!verify_effect(&before, &before, action(&before)).unwrap(), "unchanged K{before_number:02}");
        }
    }


    /// Charlie's Undo reconstruction contains a completed tableau-to-foundation
    /// move even though the next dark destination guide dims the exposed felt.
    #[test]
    fn recorded_tableau_removal_under_the_next_destination_guide_is_verified() {
        let before = fixture(21);
        let after = fixture(22);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau { column: 5, top: 571, bottom: 761 }));
        let evidence = inspect_effect(&before, &after, planned).unwrap();
        assert_eq!(evidence.source_positive, 0, "ordinary bright-pixel proof must reproduce the stop: {evidence}");
        assert_eq!(evidence.source_dimmed_felt, 1_050, "{evidence}");
        assert!(evidence.source_changed >= MINIMUM_CONTENT_CHANGE, "{evidence}");
        assert!(evidence.destination_changed >= MINIMUM_CONTENT_CHANGE, "{evidence}");
        assert!(evidence.verified, "completed recorded move must pass: {evidence}");
        assert!(!verify_effect(&before, &before, planned).unwrap());
    }


    /// Darkened exposed felt needs an independent destination change; neither
    /// a changed source alone nor an unchanged original frame is a transfer.
    #[test]
    fn dimmed_felt_source_only_has_no_destination_authority() {
        let before = fixture(21);
        let completed = fixture(22);
        let planned = action(&before);
        let mut source_only = before.clone();
        let source = planned.effect_bounds();
        copy_region(&completed, &mut source_only, source, PixelPoint::new(source.x as i32, source.y as i32));
        let evidence = inspect_effect(&before, &source_only, planned).unwrap();
        assert_eq!(evidence.source_dimmed_felt, 1_050, "{evidence}");
        assert_eq!(evidence.destination_changed, 0, "{evidence}");
        assert!(!evidence.verified, "{evidence}");
    }


    /// Neutral guide darkening, black, gold, cursor and toolbar pixels cannot
    /// prove removal even with the genuine first-pair foundation change.
    #[test]
    fn dimmed_felt_proof_rejects_guide_only_and_masked_changes() {
        let before = fixture(21);
        let completed = fixture(22);
        let planned = action(&before);
        let mut destination_only = before.clone();
        copy_region(&completed, &mut destination_only, PixelRect::new(1_062, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(1_062, UPPER_Y as i32));
        let source = planned.effect_bounds();
        let InputOperation::Click(point) = planned.operation() else { panic!("expected click"); };


        for (label, bounds, rgb) in [
            ("neutral guide", source, [31, 31, 31]),
            ("opaque black", source, [0, 0, 0]),
            ("gold", source, [240, 190, 70]),
            ("cursor", PixelRect::new(point.x as u32 - 20, point.y as u32 - 20, 40, 40), [4, 17, 11]),
            ("toolbar", PixelRect::new(source.x, TOOLBAR_TOP, CARD_WIDTH, 86), [4, 17, 11]),
        ] {
            let mut after = destination_only.clone();
            paint(&mut after, bounds, rgb);
            let evidence = inspect_effect(&before, &after, planned).unwrap();
            assert_eq!(evidence.source_positive, 0, "{label}: {evidence}");
            assert_eq!(evidence.source_dimmed_felt, 0, "{label}: {evidence}");
            assert!(evidence.destination_changed >= MINIMUM_CONTENT_CHANGE, "{label}: {evidence}");
            assert!(!evidence.verified, "{label}: {evidence}");
        }
    }


    /// K23 supplies only an empty-column result. Genuine K21 card pixels and
    /// ordinary K20 column-4 pixels construct a synthetic prior frame; this is
    /// a colour-boundary regression, not the missing live before/after pair.
    #[test]
    fn synthetic_last_card_removal_exposes_k23_dimmed_empty_column() {
        let after = fixture(23);
        let mut before = after.clone();
        copy_region(&fixture(20), &mut before, PixelRect::new(884, 332, 154, 420), PixelPoint::new(884, 332));
        copy_region(&fixture(21), &mut before, PixelRect::new(1_052, 561, 154, 212), PixelPoint::new(1_220, 327));
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau { column: 6, top: 337, bottom: 527 }));
        let evidence = inspect_effect(&before, &after, planned).unwrap();
        assert_eq!(evidence.source_positive, 0, "{evidence}");
        assert!(evidence.source_dimmed_felt >= MINIMUM_CONTENT_CHANGE, "{evidence}");
        assert!(evidence.destination_changed >= MINIMUM_CONTENT_CHANGE, "{evidence}");
        assert!(evidence.verified, "{evidence}");
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


    /// An explicitly synthetic full bar preserves the original active banner
    /// and scene without adding a gameplay action. It is not a live win image.
    fn synthetic_full_progress_without_action() -> CapturedFrame {
        let mut frame = active_without_highlight();
        paint(&mut frame, COMPLETION_PROGRESS_BOUNDS, [255, 226, 138]);
        frame
    }


    /// The actual Solve-available frame has 56 black of 76 probe pixels. A
    /// shared 85% discriminator would be unsafe here; Klondike stays unfinished.
    #[test]
    fn recorded_solve_available_progress_remains_unfinished() {
        let frame = fixture(25);
        assert_eq!(action(&frame).target, ActionTarget::Klondike(KlondikeTarget::Solve));
        let evidence = completion_evidence(&frame).unwrap();
        assert_eq!(evidence.black_pixels, 56, "{evidence}");
        assert_eq!(evidence.gold_pixels, 20, "{evidence}");
        assert!(!evidence.complete_candidate, "{evidence}");


        for number in 1..=25 {
            assert!(!completion_evidence(&fixture(number)).unwrap().complete_candidate,
                "K{number:02} is not a supplied positive game-win capture");
        }
    }


    /// No-black means literally zero; all 76 samples also need positively
    /// measured fill, so a last black pixel or an arbitrary colour is refusal.
    #[test]
    fn completion_requires_every_progress_pixel_to_be_gold_and_none_black() {
        let full = synthetic_full_progress_without_action();
        assert_eq!(completion_evidence(&full).unwrap(), CompletionEvidence {
            complete_candidate: true, black_pixels: 0, gold_pixels: 76,
            reason: "literal full-gold progress with intact Klondike scene/Solver banner and no higher-priority action",
        });


        for rgb in [[0, 0, 0], [32, 32, 32], [33, 33, 33], [255, 255, 255], [12, 82, 45]] {
            let mut corrupted = full.clone();
            paint(&mut corrupted, PixelRect::new(1_087, 88, 1, 1), rgb);
            let evidence = completion_evidence(&corrupted).unwrap();
            assert!(!evidence.complete_candidate, "one unfilled pixel {rgb:?}: {evidence}");
            assert_eq!(evidence.gold_pixels, 75, "{evidence}");
            assert_eq!(evidence.black_pixels, usize::from(rgb.into_iter().all(|channel| channel <= COMPLETION_BLACK_MAXIMUM)));
        }


        for rgb in [[0, 0, 0], [255, 255, 255], [12, 82, 45], [128, 128, 128]] {
            let mut blank_bar = full.clone();
            paint(&mut blank_bar, COMPLETION_PROGRESS_BOUNDS, rgb);
            let evidence = completion_evidence(&blank_bar).unwrap();
            assert_eq!(evidence.gold_pixels, 0, "{evidence}");
            assert!(!evidence.complete_candidate, "arbitrary non-black is not gold fill: {evidence}");
        }
    }


    /// The positive synthetic bar still needs the original Solver rails and
    /// native game context. A blank/covered screen cannot become completion.
    #[test]
    fn full_gold_progress_without_intact_scene_and_solver_banner_is_refused() {
        let full = synthetic_full_progress_without_action();


        for bounds in [
            PixelRect::new(828, 36, 263, 1),
            PixelRect::new(828, 85, 263, 1),
            PixelRect::new(824, 34, 272, 52),
            PixelRect::new(160, 310, 16, 16),
            PixelRect::new(820, 600, 400, 200),
        ] {
            let mut unsupported = full.clone();
            paint(&mut unsupported, bounds, [0, 0, 0]);
            let evidence = completion_evidence(&unsupported).unwrap();
            assert_eq!(evidence.gold_pixels, 76, "{evidence}");
            assert!(!evidence.complete_candidate, "damaged context {bounds:?}: {evidence}");
        }

        let mut blank = full.clone();
        paint(&mut blank, PixelRect::new(0, 0, PROFILE.frame_width, PROFILE.frame_height), [0, 0, 0]);
        copy_region(&full, &mut blank, PixelRect::new(824, 34, 272, 58), PixelPoint::new(824, 34));
        let evidence = completion_evidence(&blank).unwrap();
        assert_eq!(evidence.gold_pixels, 76, "{evidence}");
        assert!(!evidence.complete_candidate, "a banner on a blank frame is not a game: {evidence}");
    }


    /// Every evidenced actionable frame wins priority over a synthetic full
    /// progress bar, including Draw, recycle, RIGHT, Solve and tableau blocks.
    #[test]
    fn full_progress_never_overrides_a_higher_priority_target() {


        for number in 2..=25 {
            let mut frame = fixture(number);
            let selected = action(&frame);
            paint(&mut frame, COMPLETION_PROGRESS_BOUNDS, [255, 226, 138]);
            assert_eq!(action(&frame), selected);
            let evidence = completion_evidence(&frame).unwrap();
            assert_eq!(evidence.gold_pixels, 76, "{evidence}");
            assert!(!evidence.complete_candidate, "K{number:02} still has a priority action: {evidence}");
        }

        let mut foundation = synthetic_full_progress_without_action();
        add_upper_source(&mut foundation, 894);
        assert_eq!(action(&foundation).target, ActionTarget::Klondike(KlondikeTarget::Foundation { column: 1 }));
        assert!(!completion_evidence(&foundation).unwrap().complete_candidate);
    }


    /// Undo changed the approved preview's RIGHT source into a tableau source
    /// before any input. This capture supplies fresh selection, not move effect.
    #[test]
    fn undo_replay_capture_selects_the_tableau_source_and_not_an_old_right_target() {
        let replayed = fixture(24);
        let selected = action(&replayed);
        assert_eq!(selected.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 5, top: 571, bottom: 761,
        }));
        assert_eq!(selected.operation(), InputOperation::Click(PixelPoint::new(1_128, 611)));
        assert!(!inspect_effect(&replayed, &replayed, selected).unwrap().verified);
    }


    /// Completion observations preserve checked-layout and native extent errors.
    #[test]
    fn completion_rejects_malformed_storage_and_wrong_native_geometry() {
        let mut malformed = synthetic_full_progress_without_action();
        malformed.pixels.truncate(4);
        assert_eq!(completion_evidence(&malformed), Err(HaloDetectionError::InvalidFrameLayout));
        let mut wrong_size = synthetic_full_progress_without_action();
        wrong_size.width = 1_919;
        assert_eq!(completion_evidence(&wrong_size), Err(HaloDetectionError::BoundsOutsideFrame));
    }


    /// Actual completed score-counting, Level-Up and New Game scenes provide
    /// independent positive win evidence despite their dimmed/non-gold top bar.
    #[test]
    fn recorded_completed_terminal_scenes_are_independent_one_board_win_evidence() {


        for number in 26..=28 {
            let frame = fixture(number);
            let evidence = completion_evidence(&frame).unwrap();
            assert!(evidence.complete_candidate, "K{number:02}: {evidence}");
            assert_eq!(evidence.gold_pixels, 0, "the actual terminal bar is dimmed: {evidence}");
            assert_eq!(evidence.reason,
                "independent completed Klondike terminal artwork and completed-board background");
        }


        for number in [29, 30] {
            let evidence = completion_evidence(&fixture(number)).unwrap();
            assert!(!evidence.complete_candidate, "new game/setup cannot supply previous game-win authority: K{number:02}: {evidence}");
        }
    }


    /// Reconstruct only one direction of paper/ink change in each corner, plus
    /// a separate material grey source change and the real destination pixels.
    /// This explicitly synthetic scene exercises rejection of guide-like changes.
    fn synthetic_one_way_tableau_corners(paper_to_ink: bool) -> CapturedFrame {
        let before = fixture(31);
        let mut after = before.clone();
        copy_region(&fixture(32), &mut after,
            PixelRect::new(726, TABLEAU_Y, CARD_WIDTH, TABLEAU_EFFECT_BOTTOM - TABLEAU_Y),
            PixelPoint::new(726, TABLEAU_Y as i32));
        paint(&mut after, PixelRect::new(1_428, 515, 50, 25), [64, 64, 64]);
        let point = PixelPoint::new(1_464, 465);


        for bounds in [PixelRect::new(1_403, 430, 28, 44), PixelRect::new(1_497, 551, 28, 44)] {
            let mut changed = 0;


            for y in bounds.y..bounds.y + bounds.height {


                for x in bounds.x..bounds.x + bounds.width {
                    let first = pixel_rgb(&before, x, y).unwrap();
                    let excluded_cursor = (i64::from(x) - i64::from(point.x)).abs() < 48
                        && (i64::from(y) - i64::from(point.y)).abs() < 48;
                    let eligible = (paper_to_ink && is_white(first))
                        || (!paper_to_ink && is_card_ink(first));


                    if changed == 80 || excluded_cursor || is_rail_gold(first) || !eligible {
                        continue;
                    }

                    let offset = y as usize * after.stride + x as usize * 4;


                    let rgb = if paper_to_ink { [0, 0, 0] } else { [255, 255, 255] };
                    after.pixels[offset..offset + 3].copy_from_slice(&rgb);
                    changed += 1;
                }
            }

            assert_eq!(changed, 80, "controlled one-way corner {bounds:?}");
        }

        after
    }


    /// K31/K32 shows a real reconstructed single-card transfer followed by
    /// automatic reveal. Mostly white replacement needs two-sided printed
    /// detail at each retained-white corner, plus material source/destination.
    #[test]
    fn recorded_single_card_tableau_replacement_has_bidirectional_corner_proof() {
        let before = fixture(31);
        let after = fixture(32);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 7, top: 425, bottom: 615,
        }));
        assert_eq!(action(&after).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 7, top: 407, bottom: 597,
        }));
        let evidence = inspect_effect(&before, &after, planned).unwrap();
        assert!(evidence.source_positive < MINIMUM_CONTENT_CHANGE, "old bright proof refuses: {evidence}");
        assert!(evidence.source_dimmed_felt < MINIMUM_CONTENT_CHANGE, "old dimmed-felt proof refuses: {evidence}");
        assert!(evidence.source_changed >= MINIMUM_CONTENT_CHANGE, "{evidence}");
        assert!(evidence.destination_changed >= MINIMUM_CONTENT_CHANGE, "{evidence}");
        assert!(evidence.source_identity_directions.into_iter().flatten()
            .all(|count| count >= MINIMUM_CORNER_CHANGE), "{evidence}");
        assert!(evidence.verified, "the evidenced replacement must pass: {evidence}");
        assert!(!verify_effect(&before, &before, planned).unwrap());
        assert!(!completion_evidence(&before).unwrap().complete_candidate);
        assert!(!completion_evidence(&after).unwrap().complete_candidate);
    }


    /// Printed replacement still needs both corners, material source change
    /// and an independent destination. Cursor/gold/black darkening has no proof.
    #[test]
    fn tableau_corner_replacement_keeps_source_destination_and_mask_guards() {
        let before = fixture(31);
        let completed = fixture(32);
        let planned = action(&before);
        let source = planned.effect_bounds();
        let mut source_only = before.clone();
        copy_region(&completed, &mut source_only, source, PixelPoint::new(source.x as i32, source.y as i32));
        let report = inspect_effect(&before, &source_only, planned).unwrap();
        assert_eq!(report.destination_changed, 0, "{report}");
        assert!(!report.verified, "{report}");
        let mut destination_only = before.clone();
        copy_region(&completed, &mut destination_only,
            PixelRect::new(726, TABLEAU_Y, CARD_WIDTH, TABLEAU_EFFECT_BOTTOM - TABLEAU_Y),
            PixelPoint::new(726, TABLEAU_Y as i32));
        assert!(!inspect_effect(&before, &destination_only, planned).unwrap().verified);


        for (bounds, rgb) in [
            (source, [0, 0, 0]),
            (source, [170, 120, 75]),
            (PixelRect::new(1_444, 445, 40, 40), [0, 0, 0]),
            (PixelRect::new(378, TOOLBAR_TOP, 1_162, 1_033 - TOOLBAR_TOP), [255, 255, 255]),
        ] {
            let mut damaged = destination_only.clone();
            paint(&mut damaged, bounds, rgb);
            assert!(!inspect_effect(&before, &damaged, planned).unwrap().verified, "masked/dark source {bounds:?}");
        }

        let mut one_corner = completed.clone();
        copy_region(&before, &mut one_corner, PixelRect::new(1_497, 551, 28, 44), PixelPoint::new(1_497, 551));
        let evidence = inspect_effect(&before, &one_corner, planned).unwrap();
        assert_eq!(evidence.source_identity_corners[1], 0, "{evidence}");
        assert!(!evidence.verified, "one good corner is insufficient: {evidence}");

        let mut insufficient_material = destination_only;


        for bounds in [PixelRect::new(1_403, 430, 28, 44), PixelRect::new(1_497, 551, 28, 44)] {
            copy_region(&completed, &mut insufficient_material, bounds, PixelPoint::new(bounds.x as i32, bounds.y as i32));
        }

        let evidence = inspect_effect(&before, &insufficient_material, planned).unwrap();
        assert!(evidence.source_identity_directions.into_iter().flatten()
            .all(|count| count >= MINIMUM_CORNER_CHANGE), "{evidence}");
        assert!(evidence.source_changed < MINIMUM_CONTENT_CHANGE, "material source guard stays independent: {evidence}");
        assert!(!evidence.verified, "{evidence}");

        let run = fixture(19);
        let run_action = action(&run);
        assert_eq!(card_identity_changes(&run, &run, run_action.effect_bounds(), run_action), [[0, 0], [0, 0]],
            "printed-corner replacement is limited to evidenced complete single-card geometry");
    }


    /// Uniformly dimming paper or clearing print changes one direction only.
    /// Both directions in both corners are mandatory for new tableau authority.
    #[test]
    fn tableau_corner_replacement_rejects_one_way_paper_or_ink_changes() {
        let before = fixture(31);
        let planned = action(&before);


        for paper_to_ink in [true, false] {
            let after = synthetic_one_way_tableau_corners(paper_to_ink);
            let evidence = inspect_effect(&before, &after, planned).unwrap();
            assert!(evidence.source_changed >= MINIMUM_CONTENT_CHANGE, "{evidence}");
            assert!(evidence.destination_changed >= MINIMUM_CONTENT_CHANGE, "{evidence}");
            assert!(evidence.source_positive < MINIMUM_CONTENT_CHANGE, "{evidence}");
            assert!(evidence.source_identity_corners.into_iter()
                .all(|count| count >= MINIMUM_CORNER_CHANGE), "one-way totals alone would be insufficient: {evidence}");
            assert!(!evidence.verified, "one-way guide-like changes cannot prove replacement: {evidence}");
        }
    }


    /// A synthetic single-card outline may be recognised beneath the toolbar,
    /// but a paper/ink corner extending outside its clipped proof ROI is absent.
    #[test]
    fn clipped_single_card_identity_corner_cannot_use_toolbar_pixels() {
        let planned = canonical_action(KlondikeTarget::Tableau {
            column: 7, top: 783, bottom: 963,
        }).unwrap();
        let source = planned.effect_bounds();
        assert_eq!(source.y + source.height, TOOLBAR_TOP);
        let mut before = fixture(31);
        let mut after = before.clone();


        for bounds in [PixelRect::new(1_403, 788, 28, 44), PixelRect::new(1_497, 909, 28, 44)] {
            paint(&mut before, bounds, [255, 255, 255]);
            paint(&mut after, bounds, [0, 0, 0]);
            paint(&mut before, PixelRect::new(bounds.x, bounds.y, 14, 44), [0, 0, 0]);
            paint(&mut after, PixelRect::new(bounds.x, bounds.y, 14, 44), [255, 255, 255]);
        }

        assert_eq!(card_identity_changes(&before, &after, source, planned), [[0, 0], [0, 0]],
            "even synthetic opposite paper/ink changes cannot borrow toolbar rows947..952");
    }


    /// The Undo-reconstructed 3-hearts transfer exposes 4-hearts at the same
    /// RIGHT coordinates. The occupied recipient remains under the next guide;
    /// its new middle heart supplies only 360 ordinary material pixels.
    #[test]
    fn recorded_right_transfer_under_an_unchanged_foundation_guide_is_verified() {
        let before = fixture(33);
        let after = fixture(34);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Waste { offset: 2 }));
        assert_eq!(action(&after), planned, "success can retain the same source geometry");
        let evidence = inspect_effect(&before, &after, planned).unwrap();
        assert!(evidence.source_positive >= MINIMUM_CONTENT_CHANGE, "source already passed: {evidence}");
        assert_eq!(evidence.destination_changed, 360, "recipient reproduces the runtime count: {evidence}");
        assert_eq!(evidence.destination_print.column, 1);
        assert_eq!(evidence.destination_print.new_ink_halves, [176, 184]);
        assert_eq!(evidence.destination_print.stable_paper, 12_483);
        assert_eq!(evidence.destination_print.changed_paper, 0);
        assert!(evidence.destination_print.paper_supported && evidence.destination_print.guide_stable);
        assert!(evidence.destination_print.verified && evidence.verified, "{evidence}");
        assert!(!inspect_effect(&before, &before, planned).unwrap().verified);
        assert!(!completion_evidence(&after).unwrap().complete_candidate);
    }


    /// Neither new recipient print nor changed RIGHT content supplies the other
    /// half of effect authority. A repeated source HALO is never enough.
    #[test]
    fn occupied_foundation_print_requires_independent_source_replacement() {
        let before = fixture(33);
        let completed = fixture(34);
        let planned = action(&before);
        let source = planned.effect_bounds();
        let foundation = PixelRect::new(894, UPPER_Y, CARD_WIDTH, CARD_HEIGHT);
        let mut source_only = before.clone();
        copy_region(&completed, &mut source_only, source, PixelPoint::new(source.x as i32, source.y as i32));
        let report = inspect_effect(&before, &source_only, planned).unwrap();
        assert!(report.source_positive >= MINIMUM_CONTENT_CHANGE, "{report}");
        assert_eq!(report.destination_changed, 0);
        assert!(!report.destination_print.verified && !report.verified, "{report}");
        let mut recipient_only = before.clone();
        copy_region(&completed, &mut recipient_only, foundation, PixelPoint::new(894, UPPER_Y as i32));
        let report = inspect_effect(&before, &recipient_only, planned).unwrap();
        assert!(report.destination_print.verified, "recipient alone is real: {report}");
        assert_eq!(report.source_changed, 0);
        assert!(!report.verified, "recipient alone has no source authority: {report}");
    }


    /// An introduced/removed/recoloured guide or changed neutral paper cannot
    /// use the specialised print route. The ordinary material gate is separate.
    #[test]
    fn occupied_foundation_print_rejects_guide_and_paper_changes() {
        let before = fixture(33);
        let completed = fixture(34);
        let planned = action(&before);
        let foundation = PixelRect::new(894, UPPER_Y, CARD_WIDTH, CARD_HEIGHT);


        for rgb in [[255, 255, 255], [50, 50, 50], [0, 0, 0], [40, 95, 55]] {
            let mut different = completed.clone();
            paint(&mut different, foundation, rgb);
            let print = foundation_print_changes(&before, &different, foundation, 1, planned);
            assert!(!print.verified, "a blank/white/felt/opaque recipient is unavailable: {rgb:?}: {print:?}");
            assert!(!foundation_print_changes(&different, &completed, foundation, 1, planned).verified,
                "introducing a guide is also unavailable: {rgb:?}");
        }

        let mut paper_faded = completed.clone();
        let mut paper_recoloured = completed.clone();
        let mut rail_recoloured = completed;


        for y in foundation.y..foundation.y + foundation.height {


            for x in foundation.x..foundation.x + foundation.width {
                let rgb = pixel_rgb(&paper_faded, x, y).unwrap();
                let offset = y as usize * paper_faded.stride + x as usize * 4;


                if is_dimmed_paper(rgb) {
                    paper_faded.pixels[offset..offset + 3]
                        .copy_from_slice(&rgb.map(|channel| channel.saturating_sub(GUIDE_STABILITY_DELTA)));
                    let mut tint = rgb;
                    tint[0] = tint[0].saturating_add(16);
                    paper_recoloured.pixels[offset..offset + 3].copy_from_slice(&tint);
                }


                if is_rail_gold(rgb) {
                    rail_recoloured.pixels[offset..offset + 3].copy_from_slice(&[170, 120, 75]);
                }
            }
        }


        for different in [paper_faded, paper_recoloured, rail_recoloured] {
            let report = inspect_effect(&before, &different, planned).unwrap();
            assert!(report.destination_changed < MINIMUM_CONTENT_CHANGE, "ordinary route stays insufficient: {report}");
            assert!(!report.destination_print.verified && !report.verified, "{report}");
        }
    }


    /// A cursor arriving/leaving, one-sided or tiny red changes, gold and
    /// toolbar noise do not supply the evidenced two-half red print transition.
    #[test]
    fn occupied_foundation_print_rejects_cursor_gold_tiny_and_toolbar_changes() {
        let before = fixture(33);
        let completed = fixture(34);
        let planned = action(&before);
        let source = planned.effect_bounds();
        let mut source_only = before.clone();
        copy_region(&completed, &mut source_only, source, PixelPoint::new(source.x as i32, source.y as i32));


        for (bounds, rgb) in [
            (PixelRect::new(952, 189, 16, 16), [0, 0, 0]),
            (PixelRect::new(952, 189, 16, 16), [255, 255, 255]),
            (PixelRect::new(956, 189, 8, 5), [73, 0, 0]),
            (PixelRect::new(940, 189, 10, 10), [73, 0, 0]),
            (PixelRect::new(952, 189, 16, 16), [170, 120, 75]),
            (PixelRect::new(378, TOOLBAR_TOP, 1_162, 1_033 - TOOLBAR_TOP), [255, 255, 255]),
        ] {
            let mut after = source_only.clone();
            paint(&mut after, bounds, rgb);
            let report = inspect_effect(&before, &after, planned).unwrap();
            assert!(report.destination_changed < MINIMUM_CONTENT_CHANGE, "{bounds:?}: {report}");
            assert!(!report.destination_print.verified && !report.verified, "{bounds:?}: {report}");
        }

        let mut pointer_before = before.clone();
        paint(&mut pointer_before, PixelRect::new(952, 189, 16, 16), [0, 0, 0]);
        let pointer_plan = action(&pointer_before);
        let report = inspect_effect(&pointer_before, &source_only, pointer_plan).unwrap();
        assert!(!report.destination_print.verified && !report.verified, "cursor departure is not new red print: {report}");
    }


    /// Construct an explicitly synthetic upper SUIT replacement from real
    /// A-spades/2-clubs pixels, with a separately recorded tableau recipient.
    /// This exercises image policy, not card legality or the missing rare PNG.
    fn synthetic_foundation_replacement() -> (CapturedFrame, CapturedFrame) {
        let mut before = active_without_highlight();
        add_upper_source(&mut before, 1_230);
        copy_region(&fixture(15), &mut before, PixelRect::new(1_398, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(1_230, UPPER_Y as i32));
        let destination = PixelRect::new(726, TABLEAU_Y, CARD_WIDTH, TABLEAU_EFFECT_BOTTOM - TABLEAU_Y);
        copy_region(&fixture(31), &mut before, destination, PixelPoint::new(726, TABLEAU_Y as i32));
        let mut after = before.clone();
        copy_region(&fixture(14), &mut after, PixelRect::new(614, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(1_230, UPPER_Y as i32));
        copy_region(&fixture(32), &mut after, destination, PixelPoint::new(726, TABLEAU_Y as i32));
        (before, after)
    }


    /// Opposed upper-card corners now include SUIT sources, preserving strict
    /// bidirectional detail, material source and generic destination bounds.
    #[test]
    fn synthetic_foundation_replacement_uses_strict_upper_card_proofs() {
        let (before, after) = synthetic_foundation_replacement();
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Foundation { column: 3 }));
        let report = inspect_effect(&before, &after, planned).unwrap();
        assert_eq!(report.source_changed, 772, "{report}");
        assert_eq!(report.source_positive, 70, "old positive gate is insufficient: {report}");
        assert_eq!(report.source_identity_directions, [[67, 88], [67, 83]]);
        assert_eq!(report.destination_changed, 17_151);
        assert_eq!(report.destination_print, FoundationPrintEvidence::default(), "SUIT source cannot use specialised foundation recipients");
        assert!(report.verified, "{report}");
        assert!(!inspect_effect(&before, &before, planned).unwrap().verified);
    }


    /// The global SUIT print route handles sparse corner detail while retaining
    /// both print directions, stable paper and independent source/destination.
    /// Restoring eight outside-inset corner columns isolates this synthetic gate.
    #[test]
    fn synthetic_foundation_print_replacement_requires_source_and_destination() {
        let (before, mut completed) = synthetic_foundation_replacement();
        let planned = action(&before);
        let source = planned.effect_bounds();
        copy_region(&before, &mut completed, PixelRect::new(source.x + 5, UPPER_Y + 5, 8, 44), PixelPoint::new(source.x as i32 + 5, UPPER_Y as i32 + 5));
        let report = inspect_effect(&before, &completed, planned).unwrap();
        assert!(report.source_identity_directions[0][0] < MINIMUM_CORNER_CHANGE, "corner route is insufficient: {report}");
        assert_eq!(report.source_changed, 772);
        assert_eq!(report.source_foundation_print.new_ink, 643);
        assert_eq!(report.source_foundation_print.cleared_ink, 65);
        assert_eq!(report.source_foundation_print.changed_paper_fields, 0);
        assert!(report.source_foundation_print.verified && report.verified, "{report}");
        let destination = PixelRect::new(726, TABLEAU_Y, CARD_WIDTH, TABLEAU_EFFECT_BOTTOM - TABLEAU_Y);
        let mut source_only = completed.clone();
        copy_region(&before, &mut source_only, destination, PixelPoint::new(726, TABLEAU_Y as i32));
        let report = inspect_effect(&before, &source_only, planned).unwrap();
        assert!(report.source_foundation_print.verified && !report.verified, "no recipient: {report}");
        assert_eq!(report.destination_changed, 0);
        let mut recipient_only = before.clone();
        copy_region(&completed, &mut recipient_only, destination, PixelPoint::new(726, TABLEAU_Y as i32));
        let report = inspect_effect(&before, &recipient_only, planned).unwrap();
        assert_eq!(report.source_changed, 0);
        assert!(!report.source_foundation_print.verified && !report.verified, "no source: {report}");
    }


    /// Gold/pointer/toolbar changes cannot supply material source proof, and
    /// fading/recolouring stable paper or one-way guide changes withdraw it.
    #[test]
    fn synthetic_foundation_source_print_rejects_masks_and_guide_fading() {
        let (before, mut completed) = synthetic_foundation_replacement();
        let planned = action(&before);
        let source = planned.effect_bounds();
        copy_region(&before, &mut completed, PixelRect::new(source.x + 5, UPPER_Y + 5, 8, 44), PixelPoint::new(source.x as i32 + 5, UPPER_Y as i32 + 5));
        let mut faded = completed.clone();
        let mut tinted = completed.clone();


        for y in source.y..source.y + source.height {


            for x in source.x..source.x + source.width {
                let rgb = pixel_rgb(&completed, x, y).unwrap();


                if is_bright_neutral_paper(rgb) {
                    let offset = y as usize * completed.stride + x as usize * 4;
                    faded.pixels[offset..offset + 3].copy_from_slice(&rgb.map(|channel| channel.saturating_sub(16)));
                    tinted.pixels[offset..offset + 3].copy_from_slice(&[255, 240, 240]);
                }
            }
        }


        for after in [faded, tinted] {
            let report = inspect_effect(&before, &after, planned).unwrap();
            assert!(report.source_changed >= MINIMUM_CONTENT_CHANGE && report.destination_changed >= MINIMUM_CONTENT_CHANGE, "material alone remains insufficient: {report}");
            assert!(!report.source_foundation_print.verified && !report.verified, "{report}");
        }

        let destination = PixelRect::new(726, TABLEAU_Y, CARD_WIDTH, TABLEAU_EFFECT_BOTTOM - TABLEAU_Y);
        let mut recipient_only = before.clone();
        copy_region(&completed, &mut recipient_only, destination, PixelPoint::new(726, TABLEAU_Y as i32));


        for (bounds, rgb) in [
            (PixelRect::new(1_276, 179, 40, 40), [0, 0, 0]),
            (source, [170, 120, 75]),
            (source, [160, 160, 160]),
            (PixelRect::new(source.x + 24, source.y + 16, 40, 20), [0, 0, 0]),
            (PixelRect::new(378, TOOLBAR_TOP, 1_162, 1_033 - TOOLBAR_TOP), [255, 255, 255]),
        ] {
            let mut after = recipient_only.clone();
            paint(&mut after, bounds, rgb);
            let report = inspect_effect(&before, &after, planned).unwrap();
            assert!(!report.source_foundation_print.verified && !report.verified, "{bounds:?}: {report}");
        }
    }


    /// Malformed/wrong-sized captures stop before either new private proof can
    /// inspect card pixels; unsupported scene storage never grants authority.
    #[test]
    fn occupied_foundation_and_source_proofs_reject_malformed_frames() {
        let before = fixture(33);
        let planned = action(&before);
        let mut broken = fixture(34);
        broken.pixels.truncate(4);
        assert_eq!(inspect_effect(&before, &broken, planned), Err(HaloDetectionError::InvalidFrameLayout));
        let mut wrong_size = fixture(34);
        wrong_size.width = 1_919;
        assert_eq!(inspect_effect(&before, &wrong_size, planned), Err(HaloDetectionError::BoundsOutsideFrame));
    }


    /// The recorded black-pip recipient has only 251 ordinary change pixels.
    /// Source replacement remains separate evidence for the worker's authorised
    /// fresh-HALO continuation; it does not promote the old effect to verified.
    #[test]
    fn recorded_right_progression_exposes_source_replacement_without_recipient_proof() {
        let before = fixture(35);
        let after = fixture(36);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Waste { offset: 1 }));
        assert_eq!(action(&after).target, ActionTarget::Klondike(KlondikeTarget::Waste { offset: 0 }));
        let report = inspect_effect(&before, &after, planned).unwrap();
        assert!(report.source_replaced, "independent source evidence survives sparse recipient artwork: {report}");
        assert!(report.source_positive >= MINIMUM_CONTENT_CHANGE, "{report}");
        assert_eq!(report.destination_changed, 251, "the reconstructed recipient reproduces the log: {report}");
        assert!(!report.destination_print.verified && !report.verified, "no black-pip special case is invented: {report}");
        assert!(!inspect_effect(&before, &before, planned).unwrap().source_replaced);
    }


    /// The exposed source verdict never borrows recipient, HALO or stock/Solve
    /// authority. It continues to use the existing card replacement predicates.
    #[test]
    fn source_replacement_verdict_keeps_target_and_separation_guards() {
        let before = fixture(35);
        let completed = fixture(36);
        let planned = action(&before);
        let source = planned.effect_bounds();
        let mut source_only = before.clone();
        copy_region(&completed, &mut source_only, source, PixelPoint::new(source.x as i32, source.y as i32));
        let report = inspect_effect(&before, &source_only, planned).unwrap();
        assert!(report.source_replaced && !report.verified, "source is not complete effect authority: {report}");
        let mut recipient_only = before.clone();
        copy_region(&completed, &mut recipient_only, PixelRect::new(1_398, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(1_398, UPPER_Y as i32));
        let report = inspect_effect(&before, &recipient_only, planned).unwrap();
        assert!(!report.source_replaced && !report.verified, "{report}");


        for (first, second) in [(6, 7), (12, 1), (13, 13)] {
            let frame = fixture(first);
            let report = inspect_effect(&frame, &fixture(second), action(&frame)).unwrap();
            assert!(!report.source_replaced, "stock/Recycle/Solve are not card-source continuation: K{first:02}: {report}");
        }
    }


    /// The supplied Solve frame already passes the existing complete artwork
    /// detector. Its highlighted tableau Jack is lower priority than the control.
    /// The earlier block is worker effect policy, not missing button recognition.
    #[test]
    fn recorded_solve_control_has_priority_over_a_remaining_tableau_halo() {
        let frame = fixture(37);
        assert!(has_solve_control(&frame));
        let solve = action(&frame);
        assert_eq!(solve.target, ActionTarget::Klondike(KlondikeTarget::Solve));
        assert_eq!(solve.operation(), InputOperation::Click(PixelPoint::new(624, 199)));
        assert!(find_solid_card_source(&frame, PixelRect::new(390, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap().is_some());
        let mut preceding = frame.clone();
        paint(&mut preceding, SOLVE_BOUNDS, [12, 82, 45]);
        let card = action(&preceding);
        assert!(matches!(card.target, ActionTarget::Klondike(KlondikeTarget::Tableau { column: 1, .. })));
        let report = inspect_effect(&preceding, &frame, card).unwrap();
        assert_eq!(report.source_changed, 0);
        assert!(!report.source_replaced && !report.verified, "Solve availability is separate authority: {report}");
    }


    /// K38's nine-card source ends beneath Undo All. Only the measured icon
    /// occlusion is unavailable; the intact remaining border groups one run.
    /// Its click and effect pixels remain above the toolbar.
    #[test]
    fn recorded_nine_card_source_under_toolbar_is_one_safe_upper_action() {
        let frame = fixture(38);
        let planned = action(&frame);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 6, top: 407, bottom: 990,
        }));
        assert_eq!(planned.operation(), InputOperation::Click(PixelPoint::new(1_296, 447)));
        assert_eq!(planned.effect_bounds(), PixelRect::new(1_230, 407, CARD_WIDTH, 540));
        assert_eq!(planned.effect_bounds().bottom(), TOOLBAR_TOP);
        assert_eq!(find_solid_card_source(&frame, PixelRect::new(1_230, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap(),
            Some(PixelRect::new(1_230, 407, CARD_WIDTH, 583)));
        assert_eq!(count_pixels(&frame, TOOLBAR_ICON_SUPPORT, is_toolbar_undo_red), 68);
        assert!(!completion_evidence(&frame).unwrap().complete_candidate);
    }


    /// A cropped scan, one missing visible edge pixel, absent icon evidence,
    /// broken rail/top or dimmed face cannot invent K38's complete run.
    #[test]
    fn nine_card_toolbar_source_requires_closed_border_clear_rows_and_positive_icon() {
        let frame = fixture(38);


        for bottom in 930..TABLEAU_OUTLINE_BOTTOM {
            assert_eq!(find_solid_card_source(&frame, PixelRect::new(1_230, 332, CARD_WIDTH, bottom - 332)).unwrap(), None,
                "the known icon mask cannot complete a scan cropped at row{bottom}");
        }


        for corruption in [
            PixelRect::new(1_248, 988, 1, 3),
            TOOLBAR_ICON_SUPPORT,
            PixelRect::new(1_218, 975, 156, TABLEAU_OUTLINE_BOTTOM - 975),
            PixelRect::new(1_218, 398, 156, 31),
            PixelRect::new(1_218, 620, 12, 24),
            PixelRect::new(1_248, 911, 96, 12),
        ] {
            let mut damaged = frame.clone();
            paint(&mut damaged, corruption, [12, 82, 45]);
            assert_eq!(analyse(&damaged).unwrap().prediction, PredictedAction::NoHighlight, "{corruption:?}");
        }
    }


    /// A floating toolbar border, genuine dark dashed recipient or cursor
    /// cannot substitute for the complete connected nine-card source.
    #[test]
    fn nine_card_toolbar_source_rejects_floating_edges_dark_guides_and_cursors() {
        let frame = fixture(38);
        let mut empty = frame.clone();
        paint(&mut empty, PixelRect::new(1_218, 398, 156, TABLEAU_OUTLINE_BOTTOM - 398), [12, 82, 45]);
        let mut floating_edge = empty.clone();
        paint(&mut floating_edge, PixelRect::new(1_248, 989, 96, 1), [170, 120, 75]);
        copy_region(&frame, &mut floating_edge, TOOLBAR_ICON_SUPPORT, PixelPoint::new(TOOLBAR_ICON_SUPPORT.x as i32, TOOLBAR_ICON_SUPPORT.y as i32));
        assert_eq!(analyse(&floating_edge).unwrap().prediction, PredictedAction::NoHighlight);
        let mut dashed = empty.clone();
        copy_region(&frame, &mut dashed, PixelRect::new(726, 398, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(1_230, 407));
        assert_eq!(analyse(&dashed).unwrap().prediction, PredictedAction::NoHighlight);


        for rgb in [[0, 0, 0], [255, 255, 255]] {
            let mut cursor = empty.clone();
            paint(&mut cursor, PixelRect::new(1_276, 427, 40, 60), rgb);
            assert_eq!(analyse(&cursor).unwrap().prediction, PredictedAction::NoHighlight);
        }

        let mut internal_edge = frame;
        paint(&mut internal_edge, PixelRect::new(1_218, 975, 156, TABLEAU_OUTLINE_BOTTOM - 975), [12, 82, 45]);
        paint(&mut internal_edge, PixelRect::new(1_248, 944, 96, 1), [170, 120, 75]);
        assert_eq!(analyse(&internal_edge).unwrap().prediction, PredictedAction::NoHighlight,
            "continuing exterior rails invalidate a fabricated internal crossbar");
    }


    /// The measured lower toolbar adds recognition pixels only. It still cannot
    /// supply source replacement, recipient change, click or completion proof.
    #[test]
    fn nine_card_toolbar_source_proof_and_canonical_bounds_stay_above_toolbar() {
        let before = fixture(38);
        let planned = action(&before);
        let mut toolbar_changed = before.clone();
        paint(&mut toolbar_changed, PixelRect::new(378, TOOLBAR_TOP, 1_162, 1_033 - TOOLBAR_TOP), [255, 255, 255]);
        let report = inspect_effect(&before, &toolbar_changed, planned).unwrap();
        assert_eq!(report.source_changed, 0);
        assert_eq!(report.destination_changed, 0);
        assert!(!report.source_replaced && !report.verified, "{report}");
        assert!(canonical_action(KlondikeTarget::Tableau { column: 6, top: 407, bottom: TABLEAU_OUTLINE_BOTTOM as u16 + 1 }).is_none());
        assert!(canonical_action(KlondikeTarget::Tableau { column: 6, top: 390, bottom: 994 }).is_none(),
            "the unchanged 600-pixel maximum rejects uncalibrated taller runs");
        let mut malformed = before;
        malformed.pixels.truncate(4);
        assert_eq!(find_solid_card_source(&malformed, PixelRect::new(1_230, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)),
            Err(HaloDetectionError::InvalidFrameLayout));
    }


    /// Native 4-clubs removal exposes a bright 5-hearts above the toolbar. The
    /// ordinary lower corner remains unavailable; both alternate full patches
    /// establish only source replacement, keeping the complete effect unverified.
    #[test]
    fn recorded_bottom_single_card_replacement_uses_two_full_visible_patches() {
        let before = fixture(39);
        let after = fixture(40);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 3, top: 799, bottom: 989,
        }));
        assert_eq!(planned.operation(), InputOperation::Click(PixelPoint::new(792, 839)));
        assert_eq!(planned.effect_bounds().bottom(), TOOLBAR_TOP);
        assert_eq!(action(&after).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 6, top: 372, bottom: 562,
        }));
        let report = inspect_effect(&before, &after, planned).unwrap();
        assert_eq!(report.source_changed, 824, "{report}");
        assert_eq!(report.source_positive, 61, "{report}");
        assert_eq!(report.source_dimmed_felt, 0, "{report}");
        assert_eq!(report.source_identity_directions, [[0, 0], [0, 0]], "ordinary clipped proof stays refused");
        assert_eq!(report.source_visible_identity_directions, [[68, 204], [228, 56]], "{report}");
        assert_eq!(report.destination_changed, 1_298, "{report}");
        assert!(report.source_replaced && !report.verified, "{report}");
        assert!(!inspect_effect(&before, &before, planned).unwrap().verified);
    }


    /// Visible patches independently establish source replacement, while the
    /// actual 318-pixel receiving-foundation change alone establishes neither
    /// source replacement nor complete effect. Material source remains required.
    #[test]
    fn visible_bottom_card_patches_separate_source_replacement_from_recipient() {
        let before = fixture(39);
        let completed = fixture(40);
        let planned = action(&before);
        let source = planned.effect_bounds();
        let recipient = PixelRect::new(1_398, UPPER_Y, CARD_WIDTH, CARD_HEIGHT);
        let mut source_only = before.clone();
        copy_region(&completed, &mut source_only, source, PixelPoint::new(source.x as i32, source.y as i32));
        let report = inspect_effect(&before, &source_only, planned).unwrap();
        assert_eq!(report.source_visible_identity_directions, [[68, 204], [228, 56]], "{report}");
        assert!(report.source_changed >= MINIMUM_CONTENT_CHANGE, "{report}");
        assert_eq!(report.destination_changed, 0, "{report}");
        assert!(report.source_replaced && !report.verified, "{report}");
        let mut recipient_only = before.clone();
        copy_region(&completed, &mut recipient_only, recipient, PixelPoint::new(recipient.x as i32, recipient.y as i32));
        let report = inspect_effect(&before, &recipient_only, planned).unwrap();
        assert_eq!(report.destination_changed, 318, "actual received foundation has no next-source HALO decoration: {report}");
        assert!(!report.source_replaced && !report.verified, "{report}");
        let mut halo_only = before.clone();
        copy_region(&completed, &mut halo_only,
            PixelRect::new(1_230, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332), PixelPoint::new(1_230, 332));
        let report = inspect_effect(&before, &halo_only, planned).unwrap();
        assert_eq!(report.destination_changed, 1_298, "broad change diagnostic includes the next source's HALO shadow: {report}");
        assert_eq!(report.source_changed, 0, "{report}");
        assert!(!report.source_replaced && !report.verified, "HALO decoration alone establishes neither source nor complete effect: {report}");

        let mut tiny_source = recipient_only;


        for bounds in [PixelRect::new(731, 804, 28, 44), PixelRect::new(825, 903, 28, 44)] {
            copy_region(&completed, &mut tiny_source, bounds, PixelPoint::new(bounds.x as i32, bounds.y as i32));
        }

        let report = inspect_effect(&before, &tiny_source, planned).unwrap();
        assert_eq!(report.source_visible_identity_directions, [[68, 204], [228, 56]], "{report}");
        assert!(report.source_changed < MINIMUM_CONTENT_CHANGE, "independent material source remains required: {report}");
        assert!(!report.source_replaced && !report.verified, "{report}");
    }


    /// Restoring either opposed patch refuses the new source route, even when
    /// unrelated neutral shading leaves material source change above 512.
    #[test]
    fn visible_bottom_card_patches_require_both_complete_opposed_patches() {
        let before = fixture(39);
        let completed = fixture(40);
        let planned = action(&before);
        let source = planned.effect_bounds();
        let mut source_and_foundation = before.clone();
        copy_region(&completed, &mut source_and_foundation, source, PixelPoint::new(source.x as i32, source.y as i32));
        copy_region(&completed, &mut source_and_foundation, PixelRect::new(1_398, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(1_398, UPPER_Y as i32));


        for (index, bounds) in [PixelRect::new(731, 804, 28, 44), PixelRect::new(825, 903, 28, 44)]
            .into_iter().enumerate()
        {
            let mut one_patch = source_and_foundation.clone();
            copy_region(&before, &mut one_patch, bounds, PixelPoint::new(bounds.x as i32, bounds.y as i32));
            paint(&mut one_patch, PixelRect::new(760, 891, 64, 10), [190, 190, 190]);
            let report = inspect_effect(&before, &one_patch, planned).unwrap();
            assert_eq!(report.source_visible_identity_directions[index], [0, 0], "{report}");
            assert!(report.source_changed >= MINIMUM_CONTENT_CHANGE, "{report}");
            assert_eq!(report.destination_changed, 318, "{report}");
            assert!(report.source_positive < MINIMUM_CONTENT_CHANGE, "{report}");
            assert!(!report.source_replaced && !report.verified, "one complete patch is insufficient: {report}");
        }
    }


    /// One-way guide-like dimming or whitening cannot supply both print
    /// directions, regardless of unrelated material source shading and the
    /// independently observed actual 318-pixel receiving-foundation change.
    #[test]
    fn visible_bottom_card_patches_reject_one_way_print_changes() {
        let original = fixture(39);
        let completed = fixture(40);
        let planned = action(&original);


        for paper_to_ink in [true, false] {
            let mut before = original.clone();
            let mut after = original.clone();
            copy_region(&completed, &mut after, PixelRect::new(1_398, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(1_398, UPPER_Y as i32));


            for bounds in [PixelRect::new(731, 804, 28, 44), PixelRect::new(825, 903, 28, 44)] {
                let mut changed = 0;


                for y in bounds.y..bounds.bottom() {


                    for x in bounds.x..bounds.right() {


                        if changed == 80
                            || ((i64::from(x) - 792).abs() < 48 && (i64::from(y) - 839).abs() < 48)
                            || !pixel_rgb(&original, x, y).is_some_and(is_white)
                        {
                            continue;
                        }

                        paint(&mut before, PixelRect::new(x, y, 1, 1), if paper_to_ink { [255, 255, 255] } else { [0, 0, 0] });
                        paint(&mut after, PixelRect::new(x, y, 1, 1), if paper_to_ink { [0, 0, 0] } else { [255, 255, 255] });
                        changed += 1;
                    }
                }

                assert_eq!(changed, 80);
            }

            paint(&mut after, PixelRect::new(760, 891, 64, 10), [190, 190, 190]);
            assert_eq!(action(&before), planned, "synthetic print preserves the independently recognised source");
            let report = inspect_effect(&before, &after, planned).unwrap();
            assert!(report.source_changed >= MINIMUM_CONTENT_CHANGE, "{report}");
            assert_eq!(report.destination_changed, 318, "{report}");
            assert!(report.source_positive < MINIMUM_CONTENT_CHANGE, "{report}");
            assert!(report.source_visible_identity_directions.into_iter().all(|counts| counts[usize::from(paper_to_ink)] == 0), "{report}");
            assert!(!report.source_replaced && !report.verified, "both directions stay mandatory: {report}");
        }
    }


    /// Pointer, gold, neutral/dark guides and lower-toolbar changes cannot
    /// establish replacement even alongside the genuine received foundation.
    #[test]
    fn visible_bottom_card_patches_reject_masks_guides_and_toolbar_pixels() {
        let before = fixture(39);
        let completed = fixture(40);
        let planned = action(&before);
        let mut recipient_only = before.clone();
        copy_region(&completed, &mut recipient_only, PixelRect::new(1_398, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(1_398, UPPER_Y as i32));


        for (name, bounds, rgb) in [
            ("pointer", PixelRect::new(745, 792, 94, 94), [0, 0, 0]),
            ("white pointer", PixelRect::new(745, 792, 94, 94), [255, 255, 255]),
            ("gold", PixelRect::new(726, 799, CARD_WIDTH, TOOLBAR_TOP - 799), [230, 185, 70]),
            ("dark guide", PixelRect::new(726, 799, CARD_WIDTH, TOOLBAR_TOP - 799), [32, 32, 32]),
            ("neutral shading", PixelRect::new(726, 799, CARD_WIDTH, TOOLBAR_TOP - 799), [190, 190, 190]),
            ("toolbar", PixelRect::new(378, TOOLBAR_TOP, 1_162, 1_033 - TOOLBAR_TOP), [255, 255, 255]),
        ] {
            let mut after = recipient_only.clone();
            paint(&mut after, bounds, rgb);
            let report = inspect_effect(&before, &after, planned).unwrap();
            assert_eq!(report.destination_changed, 318, "{name}: actual recipient pixels are retained without next-source decoration: {report}");
            assert!(!report.source_replaced && !report.verified, "{name}: {report}");
        }
    }


    /// The alternate patch is full-sized and wholly above the toolbar. Shifted
    /// noncanonical actions, runs, invalid height and malformed storage refuse it.
    #[test]
    fn visible_bottom_card_patches_keep_geometry_and_storage_guards() {
        let before = fixture(39);
        let after = fixture(40);
        let planned = action(&before);
        let mut toolbar_before = before.clone();
        let mut toolbar_after = after.clone();
        paint(&mut toolbar_before, PixelRect::new(726, TOOLBAR_TOP, CARD_WIDTH, 86), [0, 0, 0]);
        paint(&mut toolbar_after, PixelRect::new(726, TOOLBAR_TOP, CARD_WIDTH, 86), [255, 255, 255]);
        assert_eq!(tableau_visible_identity_changes(&toolbar_before, &toolbar_after, planned.effect_bounds(), planned), [[68, 204], [228, 56]],
            "no pixel at or below947 enters a complete alternative patch");
        assert_eq!(card_identity_changes(&before, &after, planned.effect_bounds(), planned), [[0, 0], [0, 0]],
            "ordinary clipped-corner rejection is preserved");


        for target in [
            KlondikeTarget::Tableau { column: 3, top: 799, bottom: 990 },
            KlondikeTarget::Tableau { column: 3, top: 799, bottom: 995 },
            KlondikeTarget::Tableau { column: 3, top: 446, bottom: 636 },
            KlondikeTarget::Tableau { column: 3, top: 768, bottom: 958 },
            KlondikeTarget::Tableau { column: 3, top: 777, bottom: 967 },
            KlondikeTarget::Tableau { column: 6, top: 407, bottom: 990 },
        ] {


            if let Some(other) = canonical_action(target) {
                assert_eq!(tableau_visible_identity_changes(&before, &after, other.effect_bounds(), other), [[0, 0], [0, 0]], "{target:?}");
            } else {
                assert_eq!(canonical_action(target), None);
            }
        }

        let mut shifted = planned;
        shifted.target = ActionTarget::Klondike(KlondikeTarget::Tableau { column: 4, top: 799, bottom: 989 });
        assert_eq!(tableau_visible_identity_changes(&before, &after, planned.effect_bounds(), shifted), [[0, 0], [0, 0]]);
        assert!(canonical_action(KlondikeTarget::Tableau { column: 3, top: 799, bottom: 995 }).is_none());
        let mut malformed = after;
        malformed.pixels.truncate(4);
        assert_eq!(inspect_effect(&before, &malformed, planned), Err(HaloDetectionError::InvalidFrameLayout));
    }
}
