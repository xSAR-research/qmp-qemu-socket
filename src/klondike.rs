//! Klondike Draw 1 Solver targets and mode-owned observation policy.
//! Historical pixel-effect proofs remain test-only diagnostic regressions.
//!
//! Geometry is measured from Charlie's 1920x1080 captures K01-K81. A source
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
#[cfg(test)]
const TABLEAU_EFFECT_BOTTOM: u32 = 936;


/// First toolbar-dimmed card row measured in K19; source proof excludes this row.
const TOOLBAR_TOP: u32 = 947;


/// Exclusive scan limit including K68/K69's complete row997 lower border.
/// K38's paired rails end at row987; K41's at row990, and K68/K69's at row994.
/// This remains above the Windows taskbar and never extends input/proof bounds.
const TABLEAU_OUTLINE_BOTTOM: u32 = 998;


/// K68's column-7 toolbar shade ends before ordinary rail row966. The two
/// ordinary paired rows immediately above/below this measured band must remain
/// positive before its existing darker shadow-gold predicate may bridge it.
const COLUMN_SEVEN_TOOLBAR_SHADOW_BOTTOM: u32 = 966;


/// K75's native column-4 rails retain positive shadow gold in rows952..962.
/// Ordinary paired rows949/950 and963/964 must bracket this eleven-row band.
/// Row951 stays unclassified, using the existing ten-row gap policy unchanged.
const COLUMN_FOUR_TOOLBAR_SHADOW_TOP: u32 = 952;


/// Exclusive end of K75's measured positive column-4 toolbar rail band.
const COLUMN_FOUR_TOOLBAR_SHADOW_BOTTOM: u32 = 963;


/// K75's two warm lower-edge samples under the Hint stem at row995.
/// RGB `[98,88,48]`/`[98,87,44]` pass the existing positive shadow-gold predicate.
/// Neither is omitted: all other94 pixels and the ordinary centre must pass.
const COLUMN_FOUR_HINT_EDGE_TAIL: PixelRect = PixelRect::new(963, 995, 2, 1);


/// Measured Undo All icon/shadow over K38/K41's lower source-border rows.
/// Only these 28 by 6 pixels may be unavailable to lower-edge recognition;
/// every remaining edge pixel, closed top and exterior rail is still required.
const TOOLBAR_EDGE_OCCLUSION: PixelRect = PixelRect::new(1_308, 988, 28, 6);


/// Two required border columns immediately right of the measured Undo All mask.
/// K54 supplies warm `[91,78,40]`/`[99,84,43]` at row988 beneath the icon shadow.
/// Only the existing mask rows/context can use their darker positive-gold test;
/// these samples remain required and do not enlarge the omitted 28-pixel mask.
const TOOLBAR_SHADOW_EDGE_TAIL: PixelRect = PixelRect::new(1_336, 988, 2, 6);


/// Existing tableau scan starts ten pixels above the first card-face row.
/// The exterior highlight must remain inside this native recognition envelope.
const TABLEAU_SCAN_TOP: u32 = TABLEAU_Y - 10;


/// Capacity of the unchanged native tableau scan, rather than an observed run.
/// K80's complete620-pixel source exceeds K41's former604-pixel maximum.
/// Closed edges, connected rails and paper still establish each detected block;
/// canonical clicks and legacy diagnostic content stay above the toolbar.
const MAXIMUM_SOURCE_HEIGHT: u32 = TABLEAU_OUTLINE_BOTTOM - TABLEAU_SCAN_TOP;


/// K44's complete tableau source face starts fourteen pixels right of its slot.
/// This second scan retains the same card width and all closed-outline guards;
/// the canonical column-centre click remains inside its first highlighted card.
const TABLEAU_SOURCE_RIGHT_OFFSET: u32 = 14;


/// K46's replacement single-card source is twenty-six pixels higher after spread.
/// This exact measured contraction can support continuation only, never effect
/// verification, and only with independent material and opposed print evidence.
#[cfg(test)]
const VISIBLE_SOURCE_CONTRACTION: u16 = 26;


/// Red Undo All artwork independently visible over K38's border occlusion.
/// K38/K41 each supply 68 matching pixels; missing artwork removes the mask.
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


/// The 483 measured button-face samples retain the check mark, lettering and
/// dark background while excluding the animated gold frame. K13/K25/K37/K42/K48
/// match every sample within the original symmetric 24-level RGB tolerance.
const SOLVE_STABLE_INTERIOR: PixelRect = PixelRect::new(578, 159, 92, 81);


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
#[cfg(test)]
const MINIMUM_STABLE_PAPER_PERMILLE: usize = 800;


/// A guide's existing paper and gold rails must not shift by this channel delta.
/// This is a stability guard, not a reduced ordinary material-change threshold.
#[cfg(test)]
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


/// Positive warm source detail under K54/K68's measured toolbar shadows.
/// Preserve the rail chroma/green/blue guards; red alone reaches the measured 90s.
/// Authority is limited to K54's tail with positive icon evidence or K68's
/// separately bracketed column-7 rail band. Ordinary rails, closed tops and
/// unmasked lower-edge pixels keep their original colour predicates.
fn is_toolbar_shadow_gold([red, green, blue]: [u8; 3]) -> bool {
    red >= 90 && green >= 70 && blue <= 210
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
#[cfg(test)]
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
/// The lower inter-column probe uses the clear seven-pixel core measured in K62,
/// retaining 196 pixels and 95% felt support. Wider sampling catches K62's
/// column-3 shadow or K47's opposing column-4 shadow. No source outline grants
/// an exception to this independent scene guard.
/// Solver may be off: this supports the worker's explicitly authorised activation.
/// Sparse/unknown endgames and overlays fail closed; no completion is inferred.
pub fn is_gameplay_scene(frame: &CapturedFrame) -> Result<bool, HaloDetectionError> {
    validate_frame(frame)?;
    let gutters = [
        PixelRect::new(160, 310, 16, 16),
        PixelRect::new(1_730, 310, 16, 16),
        PixelRect::new(874, 710, 7, 28),
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


/// Read-only Solve-artwork and empty-stock measurements, separate from scene
/// validation, priority, fresh action validation and input authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SolveEvidence {
    /// Stable button-face, glyph and empty-stock guards pass; scene is separate.
    pub available: bool,
    /// Empty-stock interior pixels satisfying the existing green-felt predicate.
    pub stock_felt_pixels: u32,
    /// All 13,900 pixels in the complete calibrated empty-stock probe.
    pub stock_probe_pixels: u32,
    /// Whole-control samples matching the original settled artwork; diagnostic only.
    pub artwork_matched: u32,
    /// All 899 samples over the measured complete Solve control.
    pub artwork_samples: u32,
    /// Matching samples in the stable button face, excluding its animated frame.
    pub interior_matched: u32,
    /// All 483 samples in the measured stable interior.
    pub interior_samples: u32,
    /// Matching samples inside the separately guarded check mark/lettering area.
    pub glyph_matched: u32,
    /// All 342 samples in that original glyph area.
    pub glyph_samples: u32,
}


impl SolveEvidence {


    /// A nearly matching face requests read-only settling, never a Solve click.
    /// The native run retained 459/483 face samples and every glyph sample for
    /// three observations before the original 98% face guard passed. Preserve
    /// that authoritative guard, requiring the same empty-stock and glyph
    /// support plus at least 95% face support only to defer lower-priority input.
    /// Missing probe samples withdraw even this diagnostic recovery request.
    pub fn awaiting_settle(&self) -> bool {
        !self.available
            && self.stock_probe_pixels != 0 && self.interior_samples != 0 && self.glyph_samples != 0
            && u64::from(self.stock_felt_pixels) * 100 >= u64::from(self.stock_probe_pixels) * 90
            && u64::from(self.interior_matched) * 100 >= u64::from(self.interior_samples) * 95
            && u64::from(self.glyph_matched) * 100 >= u64::from(self.glyph_samples) * 98
    }
}


impl fmt::Display for SolveEvidence {


    /// Preserve the exact classification counts needed to diagnose live refusal.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter,
            "Solve candidate={}; awaiting read-only settle={}; empty-stock felt={}/{} (required 90%); stable interior={}/{} and glyph={}/{} (each required 98% for input; 95% face only requests recapture); full artwork={}/{} (diagnostic only, animated frame excluded from authority); scene and target priority remain separate",
            self.available, self.awaiting_settle(), self.stock_felt_pixels, self.stock_probe_pixels,
            self.interior_matched, self.interior_samples, self.glyph_matched, self.glyph_samples,
            self.artwork_matched, self.artwork_samples,
        )
    }
}


/// Measure K13's stable button face and empty stock without granting input.
/// Require 98% within the original 24 RGB levels in both the 483-sample interior
/// and the separate 342-sample check mark/lettering area. The full animated gold
/// frame remains diagnostic only: runtime observations repeatedly retain every
/// glyph sample while its outer artwork changes. All samples are measured even
/// when stock support fails. Scene, priority and fresh-input checks remain with
/// the caller. Malformed storage or non-native geometry is a detection error.
pub fn inspect_solve_control(frame: &CapturedFrame) -> Result<SolveEvidence, HaloDetectionError> {
    validate_frame(frame)?;
    let stock_inside = PixelRect::new(406, 130, 100, 139);
    let stock_felt_pixels = count_pixels(frame, stock_inside, is_felt);
    let stock_probe_pixels = stock_inside.width * stock_inside.height;


    let mut matched = 0_u32;
    let mut interior_matched = 0_u32;
    let mut interior_samples = 0_u32;
    let mut glyph_matched = 0_u32;
    let mut glyph_samples = 0_u32;
    let mut sample = 0;


    for y in (SOLVE_BOUNDS.y..SOLVE_BOUNDS.y + SOLVE_BOUNDS.height).step_by(4) {


        for x in (SOLVE_BOUNDS.x..SOLVE_BOUNDS.x + SOLVE_BOUNDS.width).step_by(4) {
            let expected = &SOLVE_TEMPLATE[sample..sample + 3];
            sample += 3;
            let interior = SOLVE_STABLE_INTERIOR.contains(PixelPoint::new(x as i32, y as i32));
            let actual = pixel_rgb(frame, x, y);
            let agrees = actual.is_some_and(|actual| {
                let ordinary_match = actual.into_iter().zip(expected).all(|(a, b)| a.abs_diff(*b) <= 24);
                let border_warming = !interior
                    && (25..=26).contains(&(i16::from(actual[0]) - i16::from(expected[0])))
                    && actual[1].abs_diff(expected[1]) <= 24 && actual[2].abs_diff(expected[2]) <= 24;
                ordinary_match || border_warming
            });
            matched += u32::from(agrees);


            if interior {
                interior_samples += 1;
                interior_matched += u32::from(agrees);
            }


            if (590..660).contains(&x) && (160..236).contains(&y) {
                glyph_samples += 1;
                glyph_matched += u32::from(agrees);
            }
        }
    }

    let available = stock_felt_pixels * 1_000 >= stock_probe_pixels * 900
        && interior_matched * 100 >= interior_samples * 98
        && glyph_matched * 100 >= glyph_samples * 98;
    Ok(SolveEvidence {
        available, stock_felt_pixels, stock_probe_pixels,
        artwork_matched: matched, artwork_samples: 899, interior_matched, interior_samples,
        glyph_matched, glyph_samples,
    })
}



/// Reuse the measured artwork result after the caller has validated its scene.
fn has_solve_control(frame: &CapturedFrame) -> bool {
    inspect_solve_control(frame).is_ok_and(|evidence| evidence.available)
}


/// Find a bright source card/run inside a dynamic vertical scan envelope.
///
/// This primitive starts at the bottom, normally requires a continuous 96-pixel
/// lower edge, follows both exterior rails and probes paper above the toolbar.
/// Only K38/K41's measured column-6 icon overlap may obscure 28 lower-edge pixels;
/// positive icon artwork and all remaining 68 gold pixels are then mandatory.
/// K54's two adjacent shadowed columns still require positive warm gold; they
/// cannot enlarge that mask or alter the ordinary edge/rail colour predicates.
/// A black destination interior fails even if some gold
/// dashes align. Card-rank artwork is not required to be white at one exact pixel.
/// The returned rectangle groups all connected highlighted cards as one action.
/// An interior crossbar cannot close a block while its exterior rails continue.
/// K19/K38/K41 permit darker closed edges in the measured toolbar overlap strip.
/// The known icon occlusion cannot provide an edge, top or rail of its own.
/// K68's separate column-7 rail band retains positive shadow chroma and two
/// ordinary paired rail rows on each side; no shadow pixel may close an edge.
/// K75's separately bracketed column-4 band also retains positive shadow rails;
/// exactly two measured warm Hint-stem samples may complete its lower edge.
/// No Hint pixel is omitted and the ordinary centre remains independently gold.
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
    let ordinary_paired_rails = |row| {
        (x - 9..x).any(|xx| pixel_rgb(frame, xx, row).is_some_and(is_rail_gold))
            && (right..right + 10).any(|xx| pixel_rgb(frame, xx, row).is_some_and(is_rail_gold))
    };
    let bracketed_column_seven_shadow = require_card_face
        && x == FIRST_COLUMN_X + COLUMN_PITCH * 6 && scan.width == CARD_WIDTH
        && bottom >= TABLEAU_OUTLINE_BOTTOM
        && (TOOLBAR_TOP - 2..TOOLBAR_TOP).all(ordinary_paired_rails)
        && (COLUMN_SEVEN_TOOLBAR_SHADOW_BOTTOM..COLUMN_SEVEN_TOOLBAR_SHADOW_BOTTOM + 2)
            .all(ordinary_paired_rails);
    let bracketed_column_four_shadow = require_card_face
        && x == FIRST_COLUMN_X + COLUMN_PITCH * 3 && scan.width == CARD_WIDTH
        && bottom >= TABLEAU_OUTLINE_BOTTOM
        && (COLUMN_FOUR_TOOLBAR_SHADOW_TOP - 3..COLUMN_FOUR_TOOLBAR_SHADOW_TOP - 1)
            .all(ordinary_paired_rails)
        && (COLUMN_FOUR_TOOLBAR_SHADOW_BOTTOM..COLUMN_FOUR_TOOLBAR_SHADOW_BOTTOM + 2)
            .all(ordinary_paired_rails);
    let paired_rails = |row| {
        ordinary_paired_rails(row)
            || (((bracketed_column_seven_shadow
                    && (TOOLBAR_TOP..COLUMN_SEVEN_TOOLBAR_SHADOW_BOTTOM).contains(&row))
                    || (bracketed_column_four_shadow
                        && (COLUMN_FOUR_TOOLBAR_SHADOW_TOP..COLUMN_FOUR_TOOLBAR_SHADOW_BOTTOM).contains(&row)))
                && (x - 9..x).any(|xx| pixel_rgb(frame, xx, row).is_some_and(is_toolbar_shadow_gold))
                && (right..right + 10).any(|xx| pixel_rgb(frame, xx, row).is_some_and(is_toolbar_shadow_gold)))
    };


    // The last scanned row must show both rails ending. Otherwise an internal
    // crossbar near a clipped scan boundary could masquerade as the lower edge.
    // K19's last rail row is958; K38's is987; K41's is990; K68/K69's is994.
    if (x - 9..x).any(|xx| pixel_rgb(frame, xx, bottom - 1).is_some_and(is_rail_gold))
        || (right..right + 10).any(|xx| pixel_rgb(frame, xx, bottom - 1).is_some_and(is_rail_gold))
    {
        return Ok(None);
    }

    let last_rail_row = (scan.y..bottom).rev().find(|&row| paired_rails(row));


    for y in (scan.y..bottom).rev() {
        let toolbar_overlap = require_card_face && (TOOLBAR_TOP..TABLEAU_OUTLINE_BOTTOM).contains(&y);
        let known_icon_occlusion = toolbar_overlap && x == 1_230 && scan.width == CARD_WIDTH
            && bottom >= TOOLBAR_EDGE_OCCLUSION.bottom()
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
            pixel_rgb(frame, xx, y).is_some_and(|rgb| lower_edge(rgb)
                || (known_icon_occlusion
                    && TOOLBAR_SHADOW_EDGE_TAIL.contains(PixelPoint::new(xx as i32, y as i32))
                    && is_toolbar_shadow_gold(rgb))
                || (bracketed_column_four_shadow
                    && COLUMN_FOUR_HINT_EDGE_TAIL.contains(PixelPoint::new(xx as i32, y as i32))
                    && is_toolbar_shadow_gold(rgb)))
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


        if !(180..=MAXIMUM_SOURCE_HEIGHT).contains(&height) || top == scan.y
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


/// Recognise the nominal tableau outline or K44's measured horizontal offset.
/// Upper piles never use this fallback. Both scans keep the complete top/lower
/// edges, paired rails, paper and toolbar bounds required by the same primitive.
fn find_tableau_source(
    frame: &CapturedFrame,
    scan: PixelRect,
) -> Result<Option<PixelRect>, HaloDetectionError> {


    if let Some(bounds) = find_solid_card_source(frame, scan)? {
        return Ok(Some(bounds));
    }

    find_solid_card_source(frame, PixelRect::new(
        scan.x + TABLEAU_SOURCE_RIGHT_OFFSET, scan.y, scan.width, scan.height,
    ))
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


            if (1..=7).contains(&column) && u32::from(top) >= TABLEAU_SCAN_TOP
                && u32::from(bottom) <= TABLEAU_OUTLINE_BOTTOM
                && u32::from(top) + 40 < TOOLBAR_TOP
                && bottom.checked_sub(top).is_some_and(|height| (180..=MAXIMUM_SOURCE_HEIGHT).contains(&u32::from(height))) =>
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
        let scan = PixelRect::new(x, TABLEAU_SCAN_TOP, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - TABLEAU_SCAN_TOP);


        if let Some(bounds) = find_tableau_source(frame, scan)? {
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
#[cfg(test)]
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
#[cfg(test)]
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
#[cfg(test)]
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
#[cfg(test)]
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
#[cfg(test)]
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
#[cfg(test)]
fn is_dimmed_paper(rgb: [u8; 3]) -> bool {
    let minimum = rgb.into_iter().min().unwrap_or(0);
    let maximum = rgb.into_iter().max().unwrap_or(255);
    minimum >= 16 && maximum <= 100 && maximum - minimum <= 6
}


/// Dimmed red printed detail, measured from the newly received middle heart.
/// Black/white cursor pixels and neutral guide shading do not qualify. A dark
/// black-pip replacement remains unsupported by this specialised fallback.
#[cfg(test)]
fn is_dimmed_red_ink([red, green, blue]: [u8; 3]) -> bool {
    (20..=100).contains(&red) && i16::from(red) - i16::from(green) >= 15
        && i16::from(red) - i16::from(blue) >= 15
        && u16::from(green) * 2 < u16::from(red)
        && u16::from(blue) * 2 < u16::from(red)
}


/// Bright neutral card paper; chromatic tint and the dark guide are excluded.
#[cfg(test)]
fn is_bright_neutral_paper(rgb: [u8; 3]) -> bool {
    is_white(rgb) && rgb.into_iter().max().unwrap_or(255)
        - rgb.into_iter().min().unwrap_or(0) <= 6
}


/// Inspect an unprinted paper field rather than antialiased glyph edges.
/// Callers use validated complete faces with either the sixteen-pixel inset or
/// complete five-pixel-margin corners. This 3 by 3 neighbourhood stays within
/// those faces and cannot reach an outline or toolbar pixel.
#[cfg(test)]
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


/// Whether two fully visible opposed patches establish balanced printed change.
/// Material source and exact 190-pixel toolbar-overlap geometry remain required.
/// Both directions must occur in each patch, each patch must supply 96 print
/// changes, and the opposed pair must supply 96 changes in each direction.
/// This predicate alone does not establish a recipient or complete effect.
#[cfg(test)]
fn visible_opposed_print_supported(
    action: GuidedAction,
    material: usize,
    directions: [[usize; 2]; 2],
) -> bool {


    let ActionTarget::Klondike(target @ KlondikeTarget::Tableau { top, bottom, .. }) = action.target else {
        return false;
    };

    canonical_action(target) == Some(action)
        && material >= MINIMUM_CONTENT_CHANGE
        && bottom.checked_sub(top) == Some(190)
        && u32::from(bottom) > TOOLBAR_TOP
        && directions.into_iter().all(|[new_ink, cleared_ink]| {
            new_ink > 0 && cleared_ink > 0
                && new_ink + cleared_ink >= MINIMUM_CORNER_CHANGE * 2
        })
        && directions[0][0] + directions[1][0] >= MINIMUM_CORNER_CHANGE * 2
        && directions[0][1] + directions[1][1] >= MINIMUM_CORNER_CHANGE * 2
}


/// A contracted single-card outline can support the existing fresh-HALO path.
/// K46 retains two complete paper-supported patches, but printed detail is not
/// evenly divided between their corners. Both print directions must occur in
/// each patch, each patch must supply 96 changed print pixels, and each direction
/// must supply 96 across the opposed pair. The original 512 material-pixel guard
/// remains. Only the measured equal 26-pixel shift of a same-column 190-pixel
/// outline qualifies; it cannot prove a recipient or a verified complete effect.
#[cfg(test)]
fn visible_contracted_source_replaced(
    after: &CapturedFrame,
    action: GuidedAction,
    material: usize,
    directions: [[usize; 2]; 2],
) -> Result<bool, HaloDetectionError> {
    let ActionTarget::Klondike(KlondikeTarget::Tableau { column, top, bottom }) = action.target else {
        return Ok(false);
    };


    if !visible_opposed_print_supported(action, material, directions) {
        return Ok(false);
    }


    let PredictedAction::Action(next) = analyse(after)?.prediction else { return Ok(false); };


    let ActionTarget::Klondike(KlondikeTarget::Tableau {
        column: next_column, top: next_top, bottom: next_bottom,
    }) = next.target else { return Ok(false); };

    Ok(column == next_column
        && top.checked_sub(next_top) == Some(VISIBLE_SOURCE_CONTRACTION)
        && bottom.checked_sub(next_bottom) == Some(VISIBLE_SOURCE_CONTRACTION))
}


/// Bright neutral source replacement for fixed SUIT and constrained bottom faces.
/// This record supplies source proof only. Complete effect still needs separate
/// recipient proof; printed detail alone establishes neither effect nor completion.
#[cfg(test)]
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


/// Recognise printed replacement in an occupied bright neutral source interior.
/// SUIT uses its fixed upper-card face. The separately constrained visible-bottom
/// route uses only its complete inset above the toolbar and receives continuation
/// authority, never full effect verification. The 512-pixel gate is retained.
/// Both red/black print directions use the existing 48-pixel detail bound and
/// exclude gold and the commanded cursor. Neutral-field stability rejects
/// guide fading/recolouring without counting antialiased ink edges as paper.
#[cfg(test)]
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
#[cfg(test)]
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
#[cfg(test)]
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
#[cfg(test)]
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


/// Printed replacement after aligning two clipped single-card source faces.
/// This record proves source replacement only, never a recipient or full effect.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AlignedSourceEvidence {
    /// A canonical clipped source and independently bounded replacement geometry.
    pub geometry_supported: bool,
    /// Ordinary replacement geometry supplies continuation evidence only.
    pub ordinary_face_geometry: bool,
    /// Independently measured ordinary nominal face top; zero for the HALO route.
    pub ordinary_face_top: u16,
    /// Relative nominal face displacement, positive upward and at most 26.
    pub upward_shift: u16,
    /// Common visible source rows before the conservative face/core insets.
    pub visible_rows: u32,
    /// Materially changed aligned face pixels outside both cursor positions/gold.
    pub material_changed: usize,
    /// Bright neutral source paper replaced by printed red/black detail.
    pub new_ink: usize,
    /// Printed red/black source detail replaced by bright neutral paper.
    pub cleared_ink: usize,
    /// Eligible aligned face pixels after both cursor and gold exclusions.
    pub eligible: usize,
    /// Bright neutral paper in the original expanded face.
    pub before_paper: usize,
    /// Bright neutral paper in the aligned replacement expanded face.
    pub after_paper: usize,
    /// Bright neutral paper retained at corresponding expanded-face positions.
    pub stable_paper: usize,
    /// Eligible aligned core pixels within the unchanged sixteen-pixel inset.
    pub core_eligible: usize,
    /// Bright neutral original paper in the sixteen-pixel core.
    pub core_before_paper: usize,
    /// Bright neutral replacement paper in the aligned sixteen-pixel core.
    pub core_after_paper: usize,
    /// Bright neutral paper retained at corresponding core positions.
    pub core_stable_paper: usize,
    /// Retained neutral 3 by 3 core fields changing by eight or more per channel.
    pub changed_paper_fields: usize,
    /// At least eighty percent neutral paper in both faces/cores and in common.
    pub paper_supported: bool,
    /// Source geometry, material, opposed print and stable core all pass.
    pub verified: bool,
}


/// Compare a clipped source with a bounded, same-column replacement face.
/// K55/K56 supplies a five-pixel upward spread; the existing 26-pixel limit
/// bounds all eligible shifts without introducing a five-pixel special case.
/// Corresponding face coordinates reject a pure translation of unchanged print.
/// The full common visible face keeps five-pixel side/bottom and ten-pixel top
/// insets, while paper-field stability retains the original sixteen-pixel core.
/// Both sample positions retain the 96 by 96 commanded-pointer mask; gold,
/// toolbar pixels, noncanonical sources and unsupported scenes grant no proof.
/// The caller separately requires material non-source change and a fresh target
/// before using this source evidence for continuation, never full effect proof.
#[cfg(test)]
fn aligned_source_print_changes(
    before: &CapturedFrame,
    after: &CapturedFrame,
    action: GuidedAction,
) -> Result<AlignedSourceEvidence, HaloDetectionError> {
    validate_frame(before)?;
    validate_frame(after)?;
    let evidence = AlignedSourceEvidence::default();


    let ActionTarget::Klondike(target @ KlondikeTarget::Tableau { column, top, bottom }) = action.target else {
        return Ok(evidence);
    };


    if canonical_action(target) != Some(action)
        || analyse(before)?.prediction != PredictedAction::Action(action)
        || u32::from(bottom) <= TOOLBAR_TOP
        || !bottom.checked_sub(top).is_some_and(|height| (180..=190).contains(&height))
    {
        return Ok(evidence);
    }


    let PredictedAction::Action(next) = analyse(after)?.prediction else { return Ok(evidence); };


    let ActionTarget::Klondike(next_target @ KlondikeTarget::Tableau {
        column: next_column, top: next_top, bottom: next_bottom,
    }) = next.target else { return Ok(evidence); };


    let Some(shift) = top.checked_sub(next_top) else { return Ok(evidence); };


    if canonical_action(next_target) != Some(next) || column != next_column
        || bottom.checked_sub(top) != next_bottom.checked_sub(next_top)
        || !(1..=VISIBLE_SOURCE_CONTRACTION).contains(&shift)
    {
        return Ok(evidence);
    }

    let source = action.effect_bounds();
    let replacement = next.effect_bounds();
    let visible_rows = source.height.min(replacement.height);


    if source.x != replacement.x || source.width != CARD_WIDTH
        || replacement.width != CARD_WIDTH || visible_rows <= 32
    {
        return Ok(evidence);
    }

    measure_aligned_source_print_changes(before, after, source, visible_rows, shift, action)
}


/// Compare corresponding clipped face pixels after independently established
/// geometry. Both callers retain the same expanded face, sixteen-pixel core,
/// cursor/gold exclusions and existing material/printed-detail bounds.
#[cfg(test)]
fn measure_aligned_source_print_changes(
    before: &CapturedFrame,
    after: &CapturedFrame,
    source: PixelRect,
    visible_rows: u32,
    shift: u16,
    action: GuidedAction,
) -> Result<AlignedSourceEvidence, HaloDetectionError> {
    let mut evidence = AlignedSourceEvidence::default();
    let face = PixelRect::new(source.x + 5, source.y + 10, CARD_WIDTH - 10, visible_rows - 15);
    let core = PixelRect::new(source.x + 16, source.y + 16, CARD_WIDTH - 32, visible_rows - 32);
    let mapped_face = PixelRect::new(face.x, face.y - u32::from(shift), face.width, face.height);
    let mapped_core = PixelRect::new(core.x, core.y - u32::from(shift), core.width, core.height);
    validate_bounds(before, face)?;
    validate_bounds(before, core)?;
    validate_bounds(after, mapped_face)?;
    validate_bounds(after, mapped_core)?;


    let InputOperation::Click(point) = action.operation() else { return Ok(evidence); };
    let cursor_excluded = |x: u32, y: u32, mapped_y: u32| {
        (i64::from(x) - i64::from(point.x)).abs() < 48
            && ((i64::from(y) - i64::from(point.y)).abs() < 48
                || (i64::from(mapped_y) - i64::from(point.y)).abs() < 48)
    };
    evidence.geometry_supported = true;
    evidence.upward_shift = shift;
    evidence.visible_rows = visible_rows;


    for y in face.y..face.bottom() {
        let mapped_y = y - u32::from(shift);


        for x in face.x..face.right() {


            if cursor_excluded(x, y, mapped_y) { continue; }


            let Some(first) = pixel_rgb(before, x, y) else { return Ok(AlignedSourceEvidence::default()); };


            let Some(second) = pixel_rgb(after, x, mapped_y) else { return Ok(AlignedSourceEvidence::default()); };


            if is_rail_gold(first) || is_rail_gold(second) { continue; }

            let first_paper = is_bright_neutral_paper(first);
            let second_paper = is_bright_neutral_paper(second);
            evidence.eligible += 1;
            evidence.before_paper += usize::from(first_paper);
            evidence.after_paper += usize::from(second_paper);
            evidence.stable_paper += usize::from(first_paper && second_paper);


            if first.into_iter().zip(second).any(|(a, b)| a.abs_diff(b) >= 48) {
                evidence.material_changed += 1;
                evidence.new_ink += usize::from(first_paper && is_card_ink(second));
                evidence.cleared_ink += usize::from(is_card_ink(first) && second_paper);
            }
        }
    }


    for y in core.y..core.bottom() {
        let mapped_y = y - u32::from(shift);


        for x in core.x..core.right() {


            if cursor_excluded(x, y, mapped_y) { continue; }


            let Some(first) = pixel_rgb(before, x, y) else { return Ok(AlignedSourceEvidence::default()); };


            let Some(second) = pixel_rgb(after, x, mapped_y) else { return Ok(AlignedSourceEvidence::default()); };


            if is_rail_gold(first) || is_rail_gold(second) { continue; }

            let first_paper = is_bright_neutral_paper(first);
            let second_paper = is_bright_neutral_paper(second);
            evidence.core_eligible += 1;
            evidence.core_before_paper += usize::from(first_paper);
            evidence.core_after_paper += usize::from(second_paper);
            evidence.core_stable_paper += usize::from(first_paper && second_paper);
            let retained_field = (y - 1..=y + 1).all(|yy| (x - 1..=x + 1).all(|xx| {
                pixel_rgb(before, xx, yy).is_some_and(is_bright_neutral_paper)
                    && pixel_rgb(after, xx, yy - u32::from(shift)).is_some_and(is_bright_neutral_paper)
            }));


            if retained_field {
                evidence.changed_paper_fields += usize::from(first.into_iter().zip(second)
                    .any(|(a, b)| a.abs_diff(b) >= GUIDE_STABILITY_DELTA));
            }
        }
    }

    evidence.paper_supported = evidence.eligible != 0 && evidence.core_eligible != 0
        && [evidence.before_paper, evidence.after_paper, evidence.stable_paper]
            .into_iter().all(|count| count * 1_000 >= evidence.eligible * MINIMUM_STABLE_PAPER_PERMILLE)
        && [evidence.core_before_paper, evidence.core_after_paper, evidence.core_stable_paper]
            .into_iter().all(|count| count * 1_000 >= evidence.core_eligible * MINIMUM_STABLE_PAPER_PERMILLE);
    evidence.verified = evidence.paper_supported && evidence.changed_paper_fields == 0
        && evidence.material_changed >= MINIMUM_CONTENT_CHANGE
        && evidence.new_ink >= MINIMUM_CORNER_CHANGE
        && evidence.cleared_ink >= MINIMUM_CORNER_CHANGE;
    Ok(evidence)
}


/// K57/K58's independently unchanged four-heart artwork calibrates the nominal
/// card top five rows below its canonical paired-rail top. Ordinary gray seams
/// and highlighted paper start at different rows; paper-start displacement alone
/// cannot choose an alignment based on its printed-change result.
#[cfg(test)]
const HIGHLIGHTED_NOMINAL_FACE_OFFSET: u16 = 5;


/// A nominal-width ordinary face in the same clipped column. Geometry alone
/// selects its unique top before any printed-change measurement. A closed
/// neutral gray seam has bright neutral rows above/below; two occupied paper
/// side stripes and two chromatic felt gutters continue to the proof boundary.
/// The lower edge is hidden by the toolbar and contributes no pixel authority.
#[cfg(test)]
fn ordinary_source_face_top(
    after: &CapturedFrame,
    source: PixelRect,
    old_nominal_top: u16,
) -> Option<u16> {


    if source.width != CARD_WIDTH || validate_bounds(after, source).is_err()
        || source.bottom() != TOOLBAR_TOP || source.x < 9
        || source.right().checked_add(9).is_none_or(|right| right >= after.width)
    {
        return None;
    }
    let bright_row = |row: u32| (source.x + 18..source.right() - 18).all(|x| {
        pixel_rgb(after, x, row).is_some_and(|rgb| {
            is_bright_neutral_paper(rgb) && rgb.into_iter().all(|value| value >= 240)
        })
    });
    let gray_row = |row: u32| (source.x + 18..source.right() - 18).all(|x| {
        pixel_rgb(after, x, row).is_some_and(|rgb| {
            is_bright_neutral_paper(rgb) && rgb.into_iter().all(|value| value <= 220)
        })
    });
    let mut found = None;


    for shift in 1..=VISIBLE_SOURCE_CONTRACTION {
        let Some(top) = old_nominal_top.checked_sub(shift) else { continue; };
        let row = u32::from(top);


        if row == 0 || row + 1 >= TOOLBAR_TOP
            || !gray_row(row) || !bright_row(row - 1) || !bright_row(row + 1)
        {
            continue;
        }
        let complete_visible_sides = (row + 1..TOOLBAR_TOP).all(|y| {
            pixel_rgb(after, source.x + 5, y).is_some_and(is_bright_neutral_paper)
                && pixel_rgb(after, source.right() - 5, y).is_some_and(is_bright_neutral_paper)
                && pixel_rgb(after, source.x - 9, y).is_some_and(is_felt)
                && pixel_rgb(after, source.right() + 9, y).is_some_and(is_felt)
        });


        if !complete_visible_sides { continue; }


        if found.is_some() { return None; }
        found = Some(top);
    }
    found
}


/// Independently aligned ordinary replacement in a clipped single-card column.
/// Its top descriptor never creates an action or input target. The fresh action
/// remains the calibrated analyser's canonical result in another column/target.
/// The caller separately requires verified receiving-foundation print and 512
/// non-source pixels; neither this descriptor nor print supplies full effect proof.
#[cfg(test)]
fn ordinary_source_print_changes(
    before: &CapturedFrame,
    after: &CapturedFrame,
    action: GuidedAction,
) -> Result<AlignedSourceEvidence, HaloDetectionError> {
    validate_frame(before)?;
    validate_frame(after)?;
    let unsupported = AlignedSourceEvidence::default();
    let ActionTarget::Klondike(target @ KlondikeTarget::Tableau { column, top, bottom }) = action.target else {
        return Ok(unsupported);
    };


    if canonical_action(target) != Some(action)
        || analyse(before)?.prediction != PredictedAction::Action(action)
        || u32::from(bottom) <= TOOLBAR_TOP
        || !bottom.checked_sub(top).is_some_and(|height| (180..=190).contains(&height))
    {
        return Ok(unsupported);
    }
    let PredictedAction::Action(next) = analyse(after)?.prediction else { return Ok(unsupported); };
    let ActionTarget::Klondike(next_target) = next.target else { return Ok(unsupported); };


    if canonical_action(next_target) != Some(next) || next == action
        || matches!(next_target, KlondikeTarget::Tableau { column: next_column, .. } if next_column == column)
    {
        return Ok(unsupported);
    }
    let source = action.effect_bounds();
    let Some(old_nominal_top) = top.checked_add(HIGHLIGHTED_NOMINAL_FACE_OFFSET) else { return Ok(unsupported); };
    let Some(replacement_top) = ordinary_source_face_top(after, source, old_nominal_top) else { return Ok(unsupported); };
    let shift = old_nominal_top - replacement_top;
    let mut evidence = measure_aligned_source_print_changes(before, after, source, source.height, shift, action)?;
    evidence.ordinary_face_geometry = true;
    evidence.ordinary_face_top = replacement_top;
    Ok(evidence)
}


/// Formerly bright neutral source paper exposing shaded green felt.
/// This is separate continuation evidence, never complete effect verification.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeepFeltSourceEvidence {
    /// Canonical single-card source above the toolbar and a different fresh column.
    pub context_supported: bool,
    /// Lower-half neutral-paper to deep chromatic felt transitions.
    pub paper_to_deep_felt: usize,
    /// Deep-range positive transitions in the separate left and right inset halves.
    pub halves: [usize; 2],
    /// Neutral paper exposing the existing, disjoint 15..64 dimmed-felt range.
    pub ordinary_dimmed_felt: usize,
    /// Existing-range positive transitions in the separate inset halves.
    pub ordinary_halves: [usize; 2],
    /// Disjoint deep and existing-range positive transitions, counted once.
    pub positive_felt: usize,
    /// Combined positive transitions in the separate left and right inset halves.
    pub positive_halves: [usize; 2],
    /// Material source change, 512 combined pixels and both spatial halves pass.
    pub verified: bool,
}


/// K59/K60 exposes green felt beneath a dark destination guide after one card
/// leaves a fully visible source. The guide's deep green range is distinct from
/// the existing K22/K23 predicate: green 10..14 with retained 5/3 chroma. K63/K64
/// spans this deep range and the unchanged 15..64 K22/K23 range. Both ranges use
/// the same formerly neutral paper guard and are disjoint, counted once. Require
/// formerly bright neutral paper, the existing 16-pixel inset and gold/cursor
/// masks, 48-channel material change, 512 positive lower-half transitions and
/// 48 in each opposed half. Achromatic shading of retained neutral paper cannot
/// supply that chroma. A fresh canonical source in a different column is context
/// only; separate non-source change is checked by the caller. Runs and toolbar
/// overlap are ineligible, and this record never grants an input target.
#[cfg(test)]
fn deep_felt_source_changes(
    before: &CapturedFrame,
    after: &CapturedFrame,
    action: GuidedAction,
    material_changed: usize,
) -> Result<DeepFeltSourceEvidence, HaloDetectionError> {
    validate_frame(before)?;
    validate_frame(after)?;
    let mut evidence = DeepFeltSourceEvidence::default();
    let ActionTarget::Klondike(target @ KlondikeTarget::Tableau { column, top, bottom }) = action.target else {
        return Ok(evidence);
    };


    if canonical_action(target) != Some(action)
        || analyse(before)?.prediction != PredictedAction::Action(action)
        || !bottom.checked_sub(top).is_some_and(|height| (180..=190).contains(&height))
        || u32::from(bottom) > TOOLBAR_TOP
    {
        return Ok(evidence);
    }
    let PredictedAction::Action(next) = analyse(after)?.prediction else { return Ok(evidence); };
    let ActionTarget::Klondike(next_target @ KlondikeTarget::Tableau { column: next_column, .. }) = next.target else {
        return Ok(evidence);
    };


    if canonical_action(next_target) != Some(next) || column == next_column {
        return Ok(evidence);
    }
    let InputOperation::Click(point) = action.operation() else { return Ok(evidence); };
    let source = action.effect_bounds();
    evidence.context_supported = true;


    for y in source.y + source.height / 2..source.bottom() - 16 {


        for x in source.x + 16..source.x + source.width - 16 {


            if (i64::from(x) - i64::from(point.x)).abs() < 48
                && (i64::from(y) - i64::from(point.y)).abs() < 48
            {
                continue;
            }
            let Some(first) = pixel_rgb(before, x, y) else { continue; };
            let Some(second @ [red, green, blue]) = pixel_rgb(after, x, y) else { continue; };


            if !is_bright_neutral_paper(first) || is_rail_gold(first) || is_rail_gold(second)
                || !first.into_iter().zip(second).any(|(a, b)| a.abs_diff(b) >= 48)
            {
                continue;
            }
            let half = usize::from(x >= source.x + source.width / 2);
            let deep = (10..=14).contains(&green)
                && i16::from(green) - i16::from(red) >= 5
                && i16::from(green) - i16::from(blue) >= 3;


            if deep {
                evidence.paper_to_deep_felt += 1;
                evidence.halves[half] += 1;
            } else if is_dimmed_felt(second) {
                evidence.ordinary_dimmed_felt += 1;
                evidence.ordinary_halves[half] += 1;
            } else {
                continue;
            }
            evidence.positive_felt += 1;
            evidence.positive_halves[half] += 1;
        }
    }
    evidence.verified = material_changed >= MINIMUM_CONTENT_CHANGE
        && evidence.positive_felt >= MINIMUM_CONTENT_CHANGE
        && evidence.positive_halves.into_iter().all(|count| count >= MINIMUM_CORNER_CHANGE);
    Ok(evidence)
}


/// Fixed SUIT replacement plus an independently located receiving tableau face.
/// This is continuation evidence only: its source corners do not meet the
/// ordinary 512-pixel complete-effect rule. No card rank or suit is decoded.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FoundationTransferEvidence {
    /// Total material white/ink changes in each complete fixed source corner.
    pub corner_changes: [usize; 2],
    /// Antipodal same-direction [new ink, cleared ink] changes, counted once.
    pub paired_changes: [usize; 2],
    /// Retained unprinted neutral 3 by 3 fields outside gold and the cursor.
    pub stable_paper_fields: [usize; 2],
    /// Retained neutral fields changing by the existing guide-stability delta.
    pub changed_paper_fields: [usize; 2],
    /// Both complete source patches retain the established 50% white-paper bound.
    pub source_paper_supported: bool,
    /// A geometrically fitting common shift retains 80% of old print in both corners.
    pub translated_ink_retained: bool,
    /// Independently qualified new receiver seams before any artwork comparison.
    pub receiver_count: usize,
    /// One-based unique receiver column, or zero without a unique descriptor.
    pub receiver_column: u8,
    /// Unique neutral seam row selected independently of source artwork.
    pub receiver_top: u16,
    /// Materially changed pixels in the complete receiving upper corner.
    pub receiver_material_changed: usize,
    /// Newly received ink matching the old source print within the stability delta.
    pub matched_transfer_ink: usize,
    /// Every separate source, geometry, receiving-print and fresh-target guard passes.
    pub verified: bool,
}


/// Retain the established 96 by 96 cursor exclusion at every compared coordinate.
#[cfg(test)]
fn foundation_transfer_cursor_excluded(point: PixelPoint, x: u32, y: u32) -> bool {
    (i64::from(x) - i64::from(point.x)).abs() < 48
        && (i64::from(y) - i64::from(point.y)).abs() < 48
}


/// Reject old printed artwork merely moving within the fixed source face.
/// The offset range is derived from fitting both complete patches in that face;
/// no shifted pixel may leave it. Each patch needs 48 eligible old ink pixels
/// and the existing 80% retained-population bound at the same common offset.
#[cfg(test)]
fn foundation_transfer_retains_shifted_print(
    before: &CapturedFrame,
    after: &CapturedFrame,
    source: PixelRect,
    patches: [PixelRect; 2],
    point: PixelPoint,
) -> bool {
    let minimum_x = patches.into_iter().map(|patch| i64::from(source.x) - i64::from(patch.x)).max().unwrap_or(0);
    let maximum_x = patches.into_iter().map(|patch| i64::from(source.right()) - i64::from(patch.right())).min().unwrap_or(-1);
    let minimum_y = patches.into_iter().map(|patch| i64::from(source.y) - i64::from(patch.y)).max().unwrap_or(0);
    let maximum_y = patches.into_iter().map(|patch| i64::from(source.bottom()) - i64::from(patch.bottom())).min().unwrap_or(-1);


    for dy in minimum_y..=maximum_y {


        for dx in minimum_x..=maximum_x {
            let retained = patches.map(|patch| {
                let mut eligible_ink = 0;
                let mut retained_ink = 0;


                for y in patch.y..patch.bottom() {


                    for x in patch.x..patch.right() {
                        let mapped_x = (i64::from(x) + dx) as u32;
                        let mapped_y = (i64::from(y) + dy) as u32;


                        if foundation_transfer_cursor_excluded(point, x, y)
                            || foundation_transfer_cursor_excluded(point, mapped_x, mapped_y)
                        {
                            continue;
                        }


                        let Some(first) = pixel_rgb(before, x, y) else { continue; };


                        let Some(second) = pixel_rgb(after, mapped_x, mapped_y) else { continue; };


                        if is_rail_gold(first) || is_rail_gold(second) || !is_card_ink(first) {
                            continue;
                        }

                        eligible_ink += 1;
                        retained_ink += usize::from(is_card_ink(second));
                    }
                }

                eligible_ink >= MINIMUM_CORNER_CHANGE
                    && retained_ink * 1_000 >= eligible_ink * MINIMUM_STABLE_PAPER_PERMILLE
            });


            if retained.into_iter().all(|supported| supported) { return true; }
        }
    }

    false
}


/// A neutral ordinary-card seam is independent of printed artwork and HALOs.
/// The nominal 96-pixel row is gray, with bright neutral rows directly adjacent.
#[cfg(test)]
fn foundation_transfer_receiver_seam(frame: &CapturedFrame, x: u32, row: u32) -> bool {
    let bright = |y| (x + 18..x + CARD_WIDTH - 18).all(|xx| {
        pixel_rgb(frame, xx, y).is_some_and(|rgb| {
            is_bright_neutral_paper(rgb) && rgb.into_iter().all(|value| value >= 240)
        })
    });
    let gray = (x + 18..x + CARD_WIDTH - 18).all(|xx| {
        pixel_rgb(frame, xx, row).is_some_and(|rgb| {
            is_bright_neutral_paper(rgb) && rgb.into_iter().all(|value| value <= 220)
        })
    });
    gray && bright(row - 1) && bright(row + 1)
}


/// Recognise K65/K66's fixed SUIT source and independently received artwork.
/// Complete source corners use their established 50% paper and 48-detail bounds;
/// paired changes must contain both directions and at least 48 matching pixels.
/// A common translated old glyph is rejected, as is neutral paper-field fading.
/// Exactly one newly appeared neutral tableau seam must independently supply a
/// complete bright receiving corner and 512 material pixels. Only then may 48
/// newly changed printed pixels match old source RGB within the existing eight-
/// channel stability bound. Cursor, gold, old recipient print and the toolbar
/// supply no authority. A different canonical fresh target remains mandatory.
#[cfg(test)]
fn foundation_transfer_source_changes(
    before: &CapturedFrame,
    after: &CapturedFrame,
    action: GuidedAction,
    directions: [[usize; 2]; 2],
) -> Result<FoundationTransferEvidence, HaloDetectionError> {
    validate_frame(before)?;
    validate_frame(after)?;
    let mut evidence = FoundationTransferEvidence::default();
    let ActionTarget::Klondike(target @ KlondikeTarget::Foundation { .. }) = action.target else {
        return Ok(evidence);
    };


    if canonical_action(target) != Some(action) { return Ok(evidence); }


    let PredictedAction::Action(next) = analyse(after)?.prediction else { return Ok(evidence); };


    let ActionTarget::Klondike(next_target) = next.target else { return Ok(evidence); };


    if next == action || canonical_action(next_target) != Some(next) { return Ok(evidence); }

    let source = action.effect_bounds();
    let InputOperation::Click(point) = action.operation() else { return Ok(evidence); };
    let patches = [
        PixelRect::new(source.x + 5, source.y + 5, 28, 44),
        PixelRect::new(source.x + 99, source.y + 126, 28, 44),
    ];
    evidence.corner_changes = directions.map(|counts| counts.into_iter().sum());
    evidence.source_paper_supported = patches.into_iter().all(|patch| {
        fraction_at_least(before, patch, is_white, 500)
            && fraction_at_least(after, patch, is_white, 500)
    });


    if !evidence.source_paper_supported
        || evidence.corner_changes.into_iter().any(|count| count < MINIMUM_CORNER_CHANGE)
    {
        return Ok(evidence);
    }


    for (index, patch) in patches.into_iter().enumerate() {


        for y in patch.y..patch.bottom() {


            for x in patch.x..patch.right() {


                if foundation_transfer_cursor_excluded(point, x, y) { continue; }


                let Some(first) = pixel_rgb(before, x, y) else { continue; };


                let Some(second) = pixel_rgb(after, x, y) else { continue; };


                if is_rail_gold(first) || is_rail_gold(second) { continue; }


                if retained_bright_paper_field(before, after, x, y) {
                    evidence.stable_paper_fields[index] += 1;
                    evidence.changed_paper_fields[index] += usize::from(first.into_iter().zip(second)
                        .any(|(a, b)| a.abs_diff(b) >= GUIDE_STABILITY_DELTA));
                }


                if index != 0 { continue; }

                let mirrored_x = patches[1].right() - 1 - (x - patch.x);
                let mirrored_y = patches[1].bottom() - 1 - (y - patch.y);


                if foundation_transfer_cursor_excluded(point, mirrored_x, mirrored_y) { continue; }


                let Some(other_first) = pixel_rgb(before, mirrored_x, mirrored_y) else { continue; };


                let Some(other_second) = pixel_rgb(after, mirrored_x, mirrored_y) else { continue; };


                if is_rail_gold(other_first) || is_rail_gold(other_second)
                    || !first.into_iter().zip(second).any(|(a, b)| a.abs_diff(b) >= 48)
                    || !other_first.into_iter().zip(other_second).any(|(a, b)| a.abs_diff(b) >= 48)
                {
                    continue;
                }

                evidence.paired_changes[0] += usize::from(is_white(first) && is_card_ink(second)
                    && is_white(other_first) && is_card_ink(other_second));
                evidence.paired_changes[1] += usize::from(is_card_ink(first) && is_white(second)
                    && is_card_ink(other_first) && is_white(other_second));
            }
        }
    }


    if evidence.paired_changes.into_iter().any(|count| count == 0)
        || evidence.paired_changes.into_iter().sum::<usize>() < MINIMUM_CORNER_CHANGE
        || evidence.stable_paper_fields.into_iter().any(|count| count < MINIMUM_CORNER_CHANGE)
        || evidence.changed_paper_fields.into_iter().any(|count| count != 0)
    {
        return Ok(evidence);
    }
    evidence.translated_ink_retained = foundation_transfer_retains_shifted_print(
        before, after, source, patches, point,
    );


    if evidence.translated_ink_retained { return Ok(evidence); }

    let mut receiver = None;


    for column in 1..=7_u8 {
        let x = FIRST_COLUMN_X + COLUMN_PITCH * u32::from(column - 1);


        for top in 342..=TABLEAU_EFFECT_BOTTOM - 11 {


            if top + 5 + patches[0].height > TOOLBAR_TOP
                || !foundation_transfer_receiver_seam(after, x, top)
                || foundation_transfer_receiver_seam(before, x, top)
            {
                continue;
            }

            let patch = PixelRect::new(x + 5, top + 5, patches[0].width, patches[0].height);


            if !fraction_at_least(after, patch, is_white, 500) { continue; }

            let mut material = 0;


            for y in patch.y..patch.bottom() {


                for xx in patch.x..patch.right() {


                    if foundation_transfer_cursor_excluded(point, xx, y) { continue; }


                    let Some(first) = pixel_rgb(before, xx, y) else { continue; };


                    let Some(second) = pixel_rgb(after, xx, y) else { continue; };


                    if !is_rail_gold(first) && !is_rail_gold(second)
                        && first.into_iter().zip(second).any(|(a, b)| a.abs_diff(b) >= 48)
                    {
                        material += 1;
                    }
                }
            }


            if material < MINIMUM_CONTENT_CHANGE { continue; }

            evidence.receiver_count += 1;
            receiver = Some((column, top, patch, material));
        }
    }


    if evidence.receiver_count != 1 { return Ok(evidence); }


    let Some((column, top, receiving_patch, material)) = receiver else { return Ok(evidence); };
    evidence.receiver_column = column;
    evidence.receiver_top = top as u16;
    evidence.receiver_material_changed = material;


    for dy in 0..patches[0].height {


        for dx in 0..patches[0].width {
            let old_x = patches[0].x + dx;
            let old_y = patches[0].y + dy;
            let receiving_x = receiving_patch.x + dx;
            let receiving_y = receiving_patch.y + dy;


            if foundation_transfer_cursor_excluded(point, old_x, old_y)
                || foundation_transfer_cursor_excluded(point, receiving_x, receiving_y)
            {
                continue;
            }


            let Some(old_source) = pixel_rgb(before, old_x, old_y) else { continue; };


            let Some(old_recipient) = pixel_rgb(before, receiving_x, receiving_y) else { continue; };


            let Some(new_recipient) = pixel_rgb(after, receiving_x, receiving_y) else { continue; };


            if is_rail_gold(old_source) || is_rail_gold(old_recipient) || is_rail_gold(new_recipient)
                || !is_card_ink(old_source) || !is_card_ink(new_recipient)
                || !old_recipient.into_iter().zip(new_recipient).any(|(a, b)| a.abs_diff(b) >= 48)
            {
                continue;
            }
            let matches_source = |rgb: [u8; 3]| old_source.into_iter().zip(rgb)
                .all(|(a, b)| a.abs_diff(b) < GUIDE_STABILITY_DELTA);


            if matches_source(new_recipient)
                && !(is_card_ink(old_recipient) && matches_source(old_recipient))
            {
                evidence.matched_transfer_ink += 1;
            }
        }
    }

    evidence.verified = evidence.matched_transfer_ink >= MINIMUM_CORNER_CHANGE;
    Ok(evidence)
}


/// An exposed ordinary source header and newly received old tableau artwork.
/// Only visible ink above the toolbar is compared; a clipped lower corner is
/// never described as complete. This record supports continuation, not a win
/// or the ordinary complete source/destination effect verdict.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TableauFoundationTransferEvidence {
    /// Reproduced canonical single-card source and existing 512 material bound.
    pub context_supported: bool,
    /// A canonical different action is independently visible in the fresh frame.
    pub fresh_target_supported: bool,
    /// Old nominal face top, derived from the established five-row rail offset.
    pub old_face_top: u16,
    /// Visible old face population height, bounded by 175 and the toolbar.
    pub visible_height: u16,
    /// Independently qualified new ordinary source-header seams.
    pub header_count: usize,
    /// Unique neutral header seam, selected without any printed matching score.
    pub header_top: u16,
    /// Old back/true ink becoming neutral bright paper in both opposed strips.
    pub header_positive: [usize; 2],
    /// Visible old printed pixels eligible for the retained-print rejection.
    pub old_source_ink: usize,
    /// Old printed pixels remaining at the same coordinates within eight RGB.
    pub retained_source_ink: usize,
    /// The old printed population meets the existing 80% retention refusal.
    pub old_print_retained: bool,
    /// Independently bright and materially changed fixed receiving SUIT faces.
    pub receiver_count: usize,
    /// One-based unique receiving SUIT, or zero without unique geometry.
    pub receiver_column: u8,
    /// Independent receiving-face interior material change.
    pub receiver_material_changed: usize,
    /// Newly changed recipient ink matching the visible old source artwork.
    pub matched_transfer_ink: usize,
    /// Source geometry, positive exposure, unique receipt and fresh target pass.
    pub verified: bool,
}


/// Printed black/chromatic detail and blue backs can positively expose paper.
/// Neutral shaded paper with every channel at least sixteen is excluded, using
/// the existing guide-paper floor and six-channel neutral tolerance. Opaque
/// black and chromatic printed detail remain eligible; no rank is decoded.
#[cfg(test)]
fn is_exposure_ink(rgb: [u8; 3]) -> bool {
    is_card_ink(rgb) && (rgb.into_iter().min().unwrap_or(255) < 16
        || rgb.into_iter().max().unwrap_or(0) - rgb.into_iter().min().unwrap_or(255) > 6)
}


/// A complete neutral seam may follow either an ordinary card or a blue back.
/// Two full bright rows below it and complete chromatic felt gutters establish
/// an ordinary exposed header independently of print, guide or HALO intensity.
#[cfg(test)]
fn tableau_transfer_header_seam(frame: &CapturedFrame, source: PixelRect, row: u32) -> bool {
    let gray = (source.x + 18..source.right() - 18).all(|x| {
        pixel_rgb(frame, x, row).is_some_and(|rgb| {
            is_bright_neutral_paper(rgb) && rgb.into_iter().all(|value| value <= 220)
        })
    });
    let bright_below = (row + 1..=row + 2).all(|y| {
        (source.x + 18..source.right() - 18).all(|x| {
            pixel_rgb(frame, x, y).is_some_and(|rgb| {
                is_bright_neutral_paper(rgb) && rgb.into_iter().all(|value| value >= 240)
            })
        })
    });
    let gutters = (row + 5..row + 49).all(|y| {
        pixel_rgb(frame, source.x - 9, y).is_some_and(is_felt)
            && pixel_rgb(frame, source.right() + 9, y).is_some_and(is_felt)
    });
    gray && bright_below && gutters
}


/// An ordinary newly exposed header plus a unique newly occupied bright SUIT
/// face can support continuation when the old source becomes a landing guide.
/// Only this route accepts a reproduced 180..191-pixel single-card outline; the
/// older identity and complete-effect bounds remain unchanged. Header geometry
/// is selected before artwork and cannot be a guide-print replacement witness.
/// Newly received old ink must match within eight RGB, change by at least 48,
/// and not already match the old receiver. Neutral paper fading, old source
/// print retained at 80%, ambiguous receivers and absent fresh targets refuse.
#[cfg(test)]
fn tableau_foundation_transfer_source_changes(
    before: &CapturedFrame,
    after: &CapturedFrame,
    action: GuidedAction,
    source_changed: usize,
) -> Result<TableauFoundationTransferEvidence, HaloDetectionError> {
    validate_frame(before)?;
    validate_frame(after)?;
    let mut evidence = TableauFoundationTransferEvidence::default();
    let ActionTarget::Klondike(target @ KlondikeTarget::Tableau { top, bottom, .. }) = action.target else {
        return Ok(evidence);
    };


    if canonical_action(target) != Some(action)
        || analyse(before)?.prediction != PredictedAction::Action(action)
        || !bottom.checked_sub(top).is_some_and(|height| (180..=191).contains(&height))
        || source_changed < MINIMUM_CONTENT_CHANGE || !is_gameplay_scene(after)?
    {
        return Ok(evidence);
    }
    let source = action.effect_bounds();
    let Some(old_face_top) = top.checked_add(HIGHLIGHTED_NOMINAL_FACE_OFFSET) else { return Ok(evidence); };
    let visible_height = CARD_HEIGHT.min(TOOLBAR_TOP.saturating_sub(u32::from(old_face_top)));
    let visible = PixelRect::new(source.x, u32::from(old_face_top), CARD_WIDTH, visible_height);


    if source.x < 9 || source.right().checked_add(9).is_none_or(|right| right >= after.width)
        || visible_height == 0 || validate_bounds(before, visible).is_err()
        || !fraction_at_least(before, visible, is_white, MINIMUM_STABLE_PAPER_PERMILLE as u32)
    {
        return Ok(evidence);
    }
    evidence.context_supported = true;
    evidence.old_face_top = old_face_top;
    evidence.visible_height = visible_height as u16;
    evidence.fresh_target_supported = matches!(analyse(after)?.prediction,
        PredictedAction::Action(next) if next != action
            && matches!(next.target, ActionTarget::Klondike(next_target)
                if canonical_action(next_target) == Some(next)));
    let InputOperation::Click(point) = action.operation() else { return Ok(evidence); };
    let mut header = None;
    let first_row = u32::from(top.saturating_sub(VISIBLE_SOURCE_CONTRACTION));


    for row in first_row..=u32::from(old_face_top) {


        if row + 49 > TOOLBAR_TOP || !tableau_transfer_header_seam(after, source, row) {
            continue;
        }
        let old_gray = (source.x + 18..source.right() - 18).all(|x| {
            pixel_rgb(before, x, row).is_some_and(|rgb| {
                is_bright_neutral_paper(rgb) && rgb.into_iter().all(|value| value <= 220)
            })
        });
        let patches = [
            PixelRect::new(source.x + 5, row + 5, 28, 44),
            PixelRect::new(source.x + 99, row + 5, 28, 44),
        ];


        if old_gray || patches.into_iter().any(|patch| !fraction_at_least(after, patch, is_white, 500)) {
            continue;
        }
        evidence.header_count += 1;
        header = Some(row);
    }


    if evidence.header_count != 1 { return Ok(evidence); }


    let Some(header_top) = header else { return Ok(evidence); };
    evidence.header_top = header_top as u16;
    let strips = [
        PixelRect::new(source.x + 5, header_top + 5, 14, 44),
        PixelRect::new(source.x + 114, header_top + 5, 13, 44),
    ];


    for (index, strip) in strips.into_iter().enumerate() {


        for y in strip.y..strip.bottom() {


            for x in strip.x..strip.right() {


                if foundation_transfer_cursor_excluded(point, x, y) { continue; }


                let Some(first) = pixel_rgb(before, x, y) else { continue; };


                let Some(second) = pixel_rgb(after, x, y) else { continue; };


                if !is_rail_gold(first) && !is_rail_gold(second)
                    && (is_back(first) || is_exposure_ink(first)) && is_bright_neutral_paper(second)
                    && first.into_iter().zip(second).any(|(a, b)| a.abs_diff(b) >= 48)
                {
                    evidence.header_positive[index] += 1;
                }
            }
        }
    }


    for y in visible.y..visible.bottom() {


        for x in visible.x..visible.right() {


            if foundation_transfer_cursor_excluded(point, x, y) { continue; }


            let Some(first) = pixel_rgb(before, x, y) else { continue; };


            let Some(second) = pixel_rgb(after, x, y) else { continue; };


            if is_rail_gold(first) || is_rail_gold(second) || !is_card_ink(first) { continue; }
            evidence.old_source_ink += 1;
            evidence.retained_source_ink += usize::from(is_card_ink(second)
                && first.into_iter().zip(second).all(|(a, b)| a.abs_diff(b) < GUIDE_STABILITY_DELTA));
        }
    }
    evidence.old_print_retained = evidence.old_source_ink >= MINIMUM_CORNER_CHANGE
        && evidence.retained_source_ink * 1_000 >= evidence.old_source_ink * MINIMUM_STABLE_PAPER_PERMILLE;


    if evidence.header_positive.into_iter().any(|count| count < MINIMUM_CORNER_CHANGE)
        || evidence.old_source_ink < MINIMUM_CORNER_CHANGE || evidence.old_print_retained
    {
        return Ok(evidence);
    }
    let mut receiver = None;


    for column in 1..=4_u8 {
        let bounds = PixelRect::new(894 + COLUMN_PITCH * u32::from(column - 1), UPPER_Y, CARD_WIDTH, CARD_HEIGHT);
        let inside = PixelRect::new(bounds.x + 16, bounds.y + 16, bounds.width - 32, bounds.height - 32);


        if !fraction_at_least(after, inside, is_white, MINIMUM_STABLE_PAPER_PERMILLE as u32) { continue; }
        let material = content_changes(before, after, bounds, action, false);


        if material < MINIMUM_CONTENT_CHANGE { continue; }
        evidence.receiver_count += 1;
        receiver = Some((column, bounds, material));
    }


    if evidence.receiver_count != 1 { return Ok(evidence); }


    let Some((column, receiving, material)) = receiver else { return Ok(evidence); };
    evidence.receiver_column = column;
    evidence.receiver_material_changed = material;


    for dy in 0..visible.height {


        for dx in 0..visible.width {
            let old_x = visible.x + dx;
            let old_y = visible.y + dy;
            let receiving_x = receiving.x + dx;
            let receiving_y = receiving.y + dy;


            if foundation_transfer_cursor_excluded(point, old_x, old_y)
                || foundation_transfer_cursor_excluded(point, receiving_x, receiving_y)
            {
                continue;
            }


            let Some(old_source) = pixel_rgb(before, old_x, old_y) else { continue; };


            let Some(old_recipient) = pixel_rgb(before, receiving_x, receiving_y) else { continue; };


            let Some(new_recipient) = pixel_rgb(after, receiving_x, receiving_y) else { continue; };


            if is_rail_gold(old_source) || is_rail_gold(old_recipient) || is_rail_gold(new_recipient)
                || !is_card_ink(old_source) || !is_card_ink(new_recipient)
                || !old_recipient.into_iter().zip(new_recipient).any(|(a, b)| a.abs_diff(b) >= 48)
            {
                continue;
            }
            let matches_source = |rgb: [u8; 3]| old_source.into_iter().zip(rgb)
                .all(|(a, b)| a.abs_diff(b) < GUIDE_STABILITY_DELTA);


            if matches_source(new_recipient)
                && !(is_card_ink(old_recipient) && matches_source(old_recipient))
            {
                evidence.matched_transfer_ink += 1;
            }
        }
    }
    evidence.verified = evidence.fresh_target_supported
        && evidence.matched_transfer_ink >= MINIMUM_CORNER_CHANGE;
    Ok(evidence)
}


/// Measured effect evidence, including rejected counts for private session logs.
#[cfg(test)]
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
    /// Stable bright-paper printed replacement in the complete above-toolbar
    /// source inset. Only exact 190-pixel single-card overlap is eligible.
    /// Combined with opposed patches and a fresh target, this supports only
    /// continuation; material non-source change is not recipient proof.
    pub source_visible_print: FoundationSourceEvidence,
    /// Fixed bright SUIT face replacement; zero for all other source classes.
    pub source_foundation_print: FoundationSourceEvidence,
    /// Paired SUIT replacement and newly received old print; continuation only.
    pub source_foundation_transfer: FoundationTransferEvidence,
    /// Exposed tableau header and new receipt of visible old ink; continuation only.
    pub source_tableau_foundation_transfer: TableauFoundationTransferEvidence,
    /// Relative-coordinate printed replacement of a clipped single-card source.
    /// A bounded new outline and non-source change permit continuation only;
    /// this field is excluded from complete effect verification.
    pub source_aligned_print: AlignedSourceEvidence,
    /// Positive deep-guide felt exposed by a fully visible single-card source.
    /// Combined with a different fresh target and material non-source change,
    /// this supports continuation only; complete effect remains unverified.
    pub source_deep_felt: DeepFeltSourceEvidence,
    /// Largest eligible destination interior change; waste count for stock input.
    pub destination_changed: usize,
    /// Strongest separately guarded occupied-foundation printed-detail candidate.
    pub destination_print: FoundationPrintEvidence,
    /// Explicit acceptance/refusal reason; unknown scenes never imply completion.
    pub reason: &'static str,
}


#[cfg(test)]
impl fmt::Display for EffectEvidence {


    /// Keep all authority-bearing measurements on one readable log line.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "verified={}; source replaced={}; source changed={}, positive={}, dimmed felt={} (each source proof requires {}), card corner ink/paper={:?} (each required {}), corner [paper-to-ink, ink-to-paper]={:?} (tableau/SUIT each required {}), visible-tableau full-patch directions={:?} (ordinary each {}, contracted-outline opposed totals {}; source 512; continuation only), visible-bottom stable print={:?}, bright-SUIT print={:?}, SUIT-to-tableau transfer={:?} (48 per complete corner, 48 paired signed pixels, unique new seam, 512 receiver material, 48 newly matched old ink; continuation only), tableau-to-SUIT transfer={:?} (unique exposed header, 48 positive each side, unique bright receiver, 512 receiver material, 48 newly matched visible old ink; continuation only), aligned clipped-source print={:?} (512 material, 48 each print direction, 80% face/core paper; ordinary geometry additionally requires independent receiving print; continuation only), deep-guide source felt={:?} (512 positive lower-half pixels, 48 each opposed half; continuation only), destination changed={} (required {}), dimmed-foundation print={:?}; {}",
            self.verified, self.source_replaced, self.source_changed, self.source_positive,
            self.source_dimmed_felt, MINIMUM_CONTENT_CHANGE, self.source_identity_corners,
            MINIMUM_CORNER_CHANGE, self.source_identity_directions, MINIMUM_CORNER_CHANGE,
            self.source_visible_identity_directions, MINIMUM_CORNER_CHANGE, MINIMUM_CORNER_CHANGE * 2,
            self.source_visible_print, self.source_foundation_print, self.source_foundation_transfer,
            self.source_tableau_foundation_transfer, self.source_aligned_print, self.source_deep_felt,
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
/// evidence only. K50/K51 additionally support uneven opposed patches through
/// independent stable-paper printed replacement, material change outside the
/// source and a valid fresh target. Non-source change may include a new HALO
/// shadow, so this fallback remains continuation only. It never changes ordinary
/// complete-effect verification.
#[cfg(test)]
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
        source_visible_print: FoundationSourceEvidence::default(),
        source_foundation_print: FoundationSourceEvidence::default(),
        source_foundation_transfer: FoundationTransferEvidence::default(),
        source_tableau_foundation_transfer: TableauFoundationTransferEvidence::default(),
        source_aligned_print: AlignedSourceEvidence::default(),
        source_deep_felt: DeepFeltSourceEvidence::default(),
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


        if visible_opposed_print_supported(action, evidence.source_changed,
            evidence.source_visible_identity_directions)
        {
            evidence.source_visible_print = foundation_source_print_changes(
                before, after, source, action, evidence.source_changed,
            );
        }
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
    evidence.source_foundation_transfer = foundation_transfer_source_changes(
        before, after, action, evidence.source_identity_directions,
    )?;
    evidence.source_tableau_foundation_transfer = tableau_foundation_transfer_source_changes(
        before, after, action, evidence.source_changed,
    )?;
    let source_verified = evidence.source_positive >= MINIMUM_CONTENT_CHANGE
        || evidence.source_dimmed_felt >= MINIMUM_CONTENT_CHANGE || identity_verified
        || evidence.source_foundation_print.verified;
    let contracted_source_replaced = visible_contracted_source_replaced(
        after, action, evidence.source_changed, evidence.source_visible_identity_directions,
    )?;
    let stable_visible_source_replaced = evidence.source_visible_print.verified
        && evidence.destination_changed >= MINIMUM_CONTENT_CHANGE
        && matches!(analyse(after)?.prediction, PredictedAction::Action(_));
    evidence.source_aligned_print = aligned_source_print_changes(before, after, action)?;


    if !evidence.source_aligned_print.geometry_supported {
        evidence.source_aligned_print = ordinary_source_print_changes(before, after, action)?;
    }
    let aligned_source_replaced = evidence.source_aligned_print.verified
        && evidence.destination_changed >= MINIMUM_CONTENT_CHANGE
        && (!evidence.source_aligned_print.ordinary_face_geometry || evidence.destination_print.verified);
    evidence.source_deep_felt = deep_felt_source_changes(before, after, action, evidence.source_changed)?;
    let deep_felt_source_replaced = evidence.source_deep_felt.verified
        && evidence.destination_changed >= MINIMUM_CONTENT_CHANGE;
    evidence.source_replaced = source_verified || visible_source_replaced
        || contracted_source_replaced || stable_visible_source_replaced || aligned_source_replaced
        || deep_felt_source_replaced || evidence.source_foundation_transfer.verified
        || evidence.source_tableau_foundation_transfer.verified;
    let destination_verified = evidence.destination_changed >= MINIMUM_CONTENT_CHANGE
        || (evidence.destination_print.verified && evidence.source_changed >= MINIMUM_CONTENT_CHANGE);
    evidence.verified = source_verified && destination_verified;


    evidence.reason = match (source_verified, destination_verified) {
        (false, _) if evidence.source_foundation_transfer.verified =>
            "paired fixed-SUIT replacement and independently located newly received old artwork support fresh-target continuation only; complete effect unverified",
        (false, _) if evidence.source_tableau_foundation_transfer.verified =>
            "independently exposed ordinary tableau header and unique bright-SUIT receipt of visible old artwork support fresh-target continuation only; complete effect unverified",
        (false, _) if contracted_source_replaced =>
            "opposed print and a 26-pixel contracted same-column outline support fresh-HALO continuation only; complete effect unverified",
        (false, _) if visible_source_replaced =>
            "two full visible patches establish source replacement for fresh-HALO continuation only; recipient and complete effect unverified",
        (false, _) if stable_visible_source_replaced =>
            "stable bright-paper replacement and opposed visible print support fresh-HALO continuation; material non-source change is not recipient proof; complete effect unverified",
        (false, _) if aligned_source_replaced && evidence.source_aligned_print.ordinary_face_geometry =>
            "independently aligned ordinary clipped-source print and separately verified receiving print support fresh-target continuation only; complete effect unverified",
        (false, _) if aligned_source_replaced =>
            "aligned clipped-source print and a bounded fresh same-column source support continuation only; non-source change is not recipient proof; complete effect unverified",
        (false, _) if deep_felt_source_replaced =>
            "neutral source paper exposing deep chromatic guide-shaded felt supports a different fresh-target continuation only; non-source change is not recipient proof; complete effect unverified",
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


    /// Read original screenshot pixels, either complete or retained in a sparse fixture.
    fn fixture(number: u8) -> CapturedFrame {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/klondike/K{number:02}.png"));
        decode_png(&std::fs::read(path).expect("read Klondike evidence fixture"))
            .expect("decode Klondike evidence fixture")
    }


    /// K74/K75 are two native phases of the same known tableau recommendation.
    /// The thin phase keeps all96 edge pixels; its two darker warm samples and
    /// eleven-row rail bridge never change input, proof or scan bounds.
    #[test]
    fn hint_shadow_keeps_complete_native_column_four_sources_actionable() {


        for (number, top) in [(74, 807), (75, 808)] {
            let frame = fixture(number);
            let bounds = find_solid_card_source(&frame, PixelRect::new(894, 332, 132, 666)).unwrap();
            assert_eq!(bounds, Some(PixelRect::new(894, top, 132, 996 - top)), "K{number}");
            let planned = action(&frame);
            assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
                column: 4, top: top as u16, bottom: 996,
            }));
            assert_eq!(planned.operation(), InputOperation::Click(PixelPoint::new(960, top as i32 + 40)));
            assert_eq!(planned.effect_bounds().bottom(), TOOLBAR_TOP);
            assert!(!inspect_effect(&frame, &frame, planned).unwrap().source_replaced,
                "recognition alone is not previous-effect authority");
        }
        let thin = fixture(75);
        assert_eq!(pixel_rgb(&thin, 963, 995), Some([98, 88, 48]));
        assert_eq!(pixel_rgb(&thin, 964, 995), Some([98, 87, 44]));
        assert_eq!(count_pixels(&thin, PixelRect::new(912, 995, 96, 1), is_rail_gold), 94);
        assert_eq!(count_pixels(&thin, PixelRect::new(912, 995, 96, 1), is_toolbar_shadow_gold), 96);
        assert!(pixel_rgb(&thin, 960, 995).is_some_and(is_rail_gold));
        let duplicate = fixture(76);
        assert_eq!(analyse(&duplicate).unwrap().prediction, PredictedAction::NoHighlight,
            "a displaced duplicate over SUIT4 does not authorise an uncalibrated input");
    }


    /// The two calibrated edge coordinates still need positive warm chroma;
    /// absent, neutral, cool, too-dark or white pixels are never omitted.
    #[test]
    fn hint_shadow_requires_both_positive_lower_edge_samples_and_ordinary_closure() {
        let native = fixture(75);
        let scan = PixelRect::new(894, 332, 132, 666);


        for rgb in [[0, 0, 0], [128, 128, 128], [70, 140, 70], [60, 80, 170], [89, 70, 30], [255, 255, 255]] {


            for x in [963, 964] {
                let mut changed = native.clone();
                paint(&mut changed, PixelRect::new(x, 995, 1, 1), rgb);
                assert_eq!(find_solid_card_source(&changed, scan).unwrap(), None, "x{x}: {rgb:?}");
                assert_eq!(analyse(&changed).unwrap().prediction, PredictedAction::NoHighlight, "x{x}: {rgb:?}");
            }
        }


        for x in [935, 960] {
            let mut changed = native.clone();
            paint(&mut changed, PixelRect::new(x, 995, 1, 1), [255, 255, 255]);
            assert_eq!(find_solid_card_source(&changed, scan).unwrap(), None,
                "ordinary edge/centre x{x} remains independently required");
        }
    }


    /// Every ordinary bracket remains required. The existing shadow predicate
    /// bridges only952..962; an absent/overdark rail or a new twelve-row weak
    /// band elsewhere still breaks the unchanged ten-row gap policy.
    #[test]
    fn hint_shadow_rail_bridge_requires_measured_band_and_ordinary_brackets() {
        let native = fixture(75);
        let scan = PixelRect::new(894, 332, 132, 666);


        for y in [949, 950, 963, 964] {


            for (x, width) in [(885, 9), (1026, 10)] {
                let mut changed = native.clone();
                paint(&mut changed, PixelRect::new(x, y, width, 1), [0, 0, 0]);
                assert_eq!(find_solid_card_source(&changed, scan).unwrap(), None,
                    "ordinary bracket row{y}, side{x} cannot be borrowed");
            }
        }


        for rgb in [[0, 0, 0], [89, 70, 30], [128, 128, 128], [60, 80, 170]] {
            let mut changed = native.clone();
            paint(&mut changed, PixelRect::new(885, 952, 9, 11), rgb);
            assert_eq!(find_solid_card_source(&changed, scan).unwrap(), None,
                "the measured band still needs a positive left rail: {rgb:?}");
        }
        let mut elsewhere = native.clone();
        paint(&mut elsewhere, PixelRect::new(885, 937, 9, 12), [95, 74, 31]);
        paint(&mut elsewhere, PixelRect::new(1026, 937, 10, 12), [95, 74, 31]);
        assert_eq!(find_solid_card_source(&elsewhere, scan).unwrap(), None,
            "out-of-band shadow gold cannot repair a twelve-row rail gap");
    }


    /// The correction is fixed to one complete native column and one lower
    /// edge row. A shifted copy is explicitly synthetic and supplies no new
    /// coordinates, colour bounds, input classes or toolbar proof authority.
    #[test]
    fn hint_shadow_does_not_follow_shifted_columns_rows_or_incomplete_sources() {
        let native = fixture(75);
        let scan = PixelRect::new(894, 332, 132, 666);
        let mut other_column = native.clone();
        copy_region(&native, &mut other_column, PixelRect::new(884, 798, 152, 200), PixelPoint::new(716, 798));
        assert_eq!(find_solid_card_source(&other_column, PixelRect::new(726, 332, 132, 666)).unwrap(), None,
            "the measured Hint samples never follow source artwork to column3");
        let mut other_row = native.clone();
        paint(&mut other_row, PixelRect::new(884, 788, 152, 210), [30, 110, 70]);
        copy_region(&native, &mut other_row, PixelRect::new(884, 798, 152, 200), PixelPoint::new(884, 797));
        assert_eq!(find_solid_card_source(&other_row, scan).unwrap(), None,
            "the special samples do not follow an edge moved one row up");
        let mut no_top = native.clone();
        paint(&mut no_top, PixelRect::new(912, 798, 96, 11), [255, 255, 255]);
        assert_eq!(find_solid_card_source(&no_top, scan).unwrap(), None, "closed top remains required");
        let mut no_face = native.clone();
        paint(&mut no_face, PixelRect::new(912, 911, 96, 12), [0, 0, 0]);
        assert_eq!(find_solid_card_source(&no_face, scan).unwrap(), None, "complete face support remains required");
        assert_eq!(find_solid_card_source(&native, PixelRect::new(894, 332, 132, 664)).unwrap(), None,
            "a shorter scan cannot claim the complete calibrated band context");
        let mut continuing = native.clone();
        paint(&mut continuing, PixelRect::new(885, 997, 9, 1), [220, 180, 100]);
        assert_eq!(find_solid_card_source(&continuing, scan).unwrap(), None, "last-row termination is unchanged");
        assert!(find_solid_card_source(&native, PixelRect::new(894, 1_070, 132, 20)).is_err());
        let mut malformed = native;
        malformed.pixels.pop();
        assert!(find_solid_card_source(&malformed, scan).is_err());
    }


    /// The first manual pair preserves the already accepted ordinary effect.
    /// Its preview rendering supplies no fresh canonical action and is not the
    /// historical logged 191-pixel result frame. Its expanded preview interrupts
    /// the stronger complete gutter guard; neither that nor the missing fresh
    /// action changes the already accepted ordinary complete-effect verdict.
    #[test]
    fn tableau_suit_transfer_preserves_first_manual_pairs_ordinary_effect() {
        let before = fixture(70);
        let after = fixture(71);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 7, top: 407, bottom: 597,
        }));
        assert_eq!(analyse(&after).unwrap().prediction, PredictedAction::NoHighlight);
        let report = inspect_effect(&before, &after, planned).unwrap();
        let transfer = report.source_tableau_foundation_transfer;
        assert!(report.verified && report.source_replaced, "ordinary result is preserved: {report}");
        assert!(transfer.context_supported && !transfer.fresh_target_supported, "{report}");
        assert_eq!(transfer.header_count, 0, "the expanded preview interrupts complete gutters: {report}");
        assert_eq!(transfer.visible_height, 175, "{report}");
        assert_eq!(transfer.matched_transfer_ink, 0, "rejected geometry is never selected by matching artwork: {report}");
        assert!(!transfer.verified, "no fresh canonical target means no new continuation authority: {report}");
    }


    /// K72/K73's old lower corner crosses the toolbar. A bounded visible ink
    /// population and independent positive exposure support continuation only.
    #[test]
    fn tableau_suit_transfer_matches_visible_old_print_after_exposed_header() {
        let before = fixture(72);
        let after = fixture(73);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 7, top: 800, bottom: 989,
        }));
        assert_eq!(action(&after).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 4, top: 336, bottom: 527,
        }));
        let report = inspect_effect(&before, &after, planned).unwrap();
        let transfer = report.source_tableau_foundation_transfer;
        assert!(transfer.context_supported && transfer.fresh_target_supported, "{report}");
        assert_eq!(transfer.old_face_top, 805, "{report}");
        assert_eq!(transfer.visible_height, 142, "a clipped population is not a complete corner: {report}");
        assert_eq!(transfer.header_count, 1, "{report}");
        assert_eq!(transfer.header_top, 777, "{report}");
        assert_eq!(transfer.header_positive, [66, 54], "{report}");
        assert!(!transfer.old_print_retained, "{report}");
        assert_eq!(transfer.receiver_count, 1, "{report}");
        assert_eq!(transfer.receiver_column, 2, "{report}");
        assert_eq!(transfer.receiver_material_changed, 13_256, "{report}");
        assert_eq!(transfer.matched_transfer_ink, 78, "{report}");
        assert!(transfer.verified && report.source_replaced && !report.verified,
            "source continuation stays separate from complete effect authority: {report}");
        assert!(report.reason.contains("unique bright-SUIT receipt"), "{report}");
    }


    /// Explicitly synthetic: move genuine bottom-edge pixels by two rows below
    /// the proof boundary. The detector must freshly reproduce the resulting
    /// 191-pixel outline; no old log frame or clipped corner is manufactured.
    #[test]
    fn tableau_suit_transfer_accepts_reproduced_synthetic_191_geometry_only_in_its_route() {
        let mut before = fixture(72);
        let source = before.clone();
        copy_region(&source, &mut before, PixelRect::new(1398, 988, 132, 1), PixelPoint::new(1398, 990));
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 7, top: 800, bottom: 991,
        }));
        let report = inspect_effect(&before, &fixture(73), planned).unwrap();
        assert!(report.source_tableau_foundation_transfer.verified && report.source_replaced && !report.verified, "{report}");
        assert_eq!(report.source_identity_corners, [0, 0], "ordinary 180..190 identity policy is unchanged: {report}");
    }


    /// Neither the new ordinary header alone nor the receiving SUIT alone
    /// supplies transfer authority; retain the genuine fresh next action.
    #[test]
    fn tableau_suit_transfer_requires_exposed_source_and_independent_new_receipt() {
        let before = fixture(72);
        let after = fixture(73);
        let planned = action(&before);
        let mut source_only = after.clone();
        copy_region(&before, &mut source_only, PixelRect::new(894, 112, 636, 175), PixelPoint::new(894, 112));
        assert_eq!(action(&source_only), action(&after));
        let report = inspect_effect(&before, &source_only, planned).unwrap();
        assert_eq!(report.source_tableau_foundation_transfer.receiver_count, 0, "{report}");
        assert!(!report.source_tableau_foundation_transfer.verified && !report.source_replaced && !report.verified, "{report}");
        let mut recipient_only = after.clone();
        copy_region(&before, &mut recipient_only, PixelRect::new(1388, 742, 152, 256), PixelPoint::new(1388, 742));
        let report = inspect_effect(&before, &recipient_only, planned).unwrap();
        assert!(!report.source_tableau_foundation_transfer.verified && !report.source_replaced && !report.verified, "{report}");
        let mut existing_receipt = before.clone();
        copy_region(&after, &mut existing_receipt, PixelRect::new(1062, 112, 132, 175), PixelPoint::new(1062, 112));
        assert_eq!(action(&existing_receipt), planned);
        let report = inspect_effect(&existing_receipt, &after, planned).unwrap();
        assert_eq!(report.source_tableau_foundation_transfer.receiver_count, 0, "{report}");
        assert!(!report.source_tableau_foundation_transfer.verified && !report.source_replaced && !report.verified, "old matching receipt is not a new transfer: {report}");
    }


    /// A material recipient change cannot borrow an unchanged old source print.
    /// Restoration preserves the new seam above it and the genuine next HALO.
    #[test]
    fn tableau_suit_transfer_rejects_retained_old_print_and_neutral_paper_whitening() {
        let before = fixture(72);
        let after = fixture(73);
        let planned = action(&before);
        let mut retained = after.clone();
        let InputOperation::Click(point) = planned.operation() else { panic!("expected one click"); };


        for y in 805..947 {


            for x in 1398..1530 {
                let old = pixel_rgb(&before, x, y).unwrap();
                let current = pixel_rgb(&retained, x, y).unwrap();
                let exposed_header_strip = y < 826
                    && ((1403..1417).contains(&x) || (1512..1525).contains(&x));


                if foundation_transfer_cursor_excluded(point, x, y)
                    || is_rail_gold(old) || is_rail_gold(current) || !is_card_ink(old)
                    || exposed_header_strip
                {
                    continue;
                }
                let offset = y as usize * retained.stride + x as usize * 4;
                retained.pixels[offset..offset + 3].copy_from_slice(&old);
            }
        }
        assert_eq!(action(&before), planned);
        assert_eq!(action(&retained), action(&after));
        let report = inspect_effect(&before, &retained, planned).unwrap();
        assert!(report.source_changed >= MINIMUM_CONTENT_CHANGE, "the retained ink cannot borrow remaining guide/material change: {report}");
        assert!(report.source_tableau_foundation_transfer.header_positive.into_iter()
            .all(|count| count >= MINIMUM_CORNER_CHANGE), "positive exposure cannot borrow retained old print: {report}");
        assert!(report.source_tableau_foundation_transfer.old_print_retained, "{report}");
        assert!(!report.source_tableau_foundation_transfer.verified && !report.source_replaced && !report.verified, "retained source plus recipient is insufficient: {report}");
        let mut neutral = before.clone();


        for strip in [PixelRect::new(1403, 782, 14, 44), PixelRect::new(1512, 782, 13, 44)] {


            for y in strip.y..strip.bottom() {


                for x in strip.x..strip.right() {
                    let offset = y as usize * neutral.stride + x as usize * 4;
                    let rgb = pixel_rgb(&neutral, x, y).unwrap();


                    if !is_rail_gold(rgb) { neutral.pixels[offset..offset + 3].copy_from_slice(&[164, 164, 164]); }
                }
            }
        }
        assert_eq!(action(&neutral), planned);
        let report = inspect_effect(&neutral, &after, planned).unwrap();
        assert_eq!(report.source_tableau_foundation_transfer.header_positive, [0, 0], "neutral shaded paper is not true ink: {report}");
        assert!(!report.source_tableau_foundation_transfer.verified, "{report}");
    }


    /// Geometry precedes print correlation: absent or ambiguous seams, a broken
    /// chromatic gutter and a dark header corner all refuse the transfer route.
    #[test]
    fn tableau_suit_transfer_requires_one_complete_ordinary_exposed_header() {
        let before = fixture(72);
        let after = fixture(73);
        let planned = action(&before);
        let mut absent = after.clone();
        paint(&mut absent, PixelRect::new(1416, 777, 96, 1), [255, 255, 255]);
        let mut gutter = after.clone();
        paint(&mut gutter, PixelRect::new(1389, 782, 1, 44), [255, 255, 255]);
        let mut corner = after.clone();
        paint(&mut corner, PixelRect::new(1403, 782, 28, 44), [0, 0, 0]);


        for (name, changed) in [("missing seam", absent), ("missing felt gutter", gutter), ("dark complete header corner", corner)] {
            assert_eq!(action(&changed), action(&after), "{name}");
            let report = inspect_effect(&before, &changed, planned).unwrap();
            assert_eq!(report.source_tableau_foundation_transfer.header_count, 0, "{name}: {report}");
            assert!(!report.source_tableau_foundation_transfer.verified, "{name}: {report}");
        }
        let mut ambiguous = after.clone();
        copy_region(&after, &mut ambiguous, PixelRect::new(1416, 777, 96, 3), PixelPoint::new(1416, 790));
        let report = inspect_effect(&before, &ambiguous, planned).unwrap();
        assert_eq!(report.source_tableau_foundation_transfer.header_count, 2, "two geometry candidates refuse before matching: {report}");
        assert!(!report.source_tableau_foundation_transfer.verified, "{report}");
    }


    /// No unchanged/cursor-only scene, forged action, run or toolbar pixels can
    /// grant the new continuation. A second independently qualified fixed SUIT
    /// must refuse before selecting any old-artwork correlation.
    #[test]
    fn tableau_suit_transfer_preserves_action_scene_and_visible_population_bounds() {
        let before = fixture(72);
        let after = fixture(73);
        let planned = action(&before);
        assert!(!inspect_effect(&before, &before, planned).unwrap().source_replaced);
        let mut cursor = before.clone();
        paint(&mut cursor, PixelRect::new(1416, 792, 96, 96), [255, 255, 255]);
        assert!(!inspect_effect(&before, &cursor, planned).unwrap().source_tableau_foundation_transfer.verified);
        let mut toolbar = after.clone();
        paint(&mut toolbar, PixelRect::new(1398, 947, 132, 51), [0, 0, 0]);
        let report = inspect_effect(&before, &toolbar, planned).unwrap();
        assert!(report.source_tableau_foundation_transfer.verified && !report.verified, "{report}");
        assert_eq!(report.source_tableau_foundation_transfer.matched_transfer_ink, 78, "toolbar pixels supply no receipt: {report}");
        let mut duplicate_before = before.clone();
        let mut duplicate_after = after.clone();
        copy_region(&before, &mut duplicate_before, PixelRect::new(1078, 128, 100, 143), PixelPoint::new(910, 128));
        copy_region(&after, &mut duplicate_after, PixelRect::new(1078, 128, 100, 143), PixelPoint::new(910, 128));
        assert_eq!(action(&duplicate_before), planned, "fixed outer foundation geometry remains genuine");
        assert_eq!(action(&duplicate_after), action(&after));
        let report = inspect_effect(&duplicate_before, &duplicate_after, planned).unwrap();
        assert_eq!(report.source_tableau_foundation_transfer.receiver_count, 2, "geometry must be unique before comparing ink: {report}");
        assert!(!report.source_tableau_foundation_transfer.verified, "{report}");
        let run_before = fixture(10);
        let run_action = action(&run_before);
        assert!(!tableau_foundation_transfer_source_changes(&run_before, &fixture(11), run_action, MINIMUM_CONTENT_CHANGE).unwrap().context_supported);
        let mut forged = planned;
        forged.anchor = PixelPoint::new(1465, 840);
        assert!(!tableau_foundation_transfer_source_changes(&before, &after, forged, MINIMUM_CONTENT_CHANGE).unwrap().context_supported);
        let mut unknown = after.clone();
        paint(&mut unknown, PixelRect::new(824, 34, 272, 58), [0, 0, 0]);
        assert!(!inspect_effect(&before, &unknown, planned).unwrap().source_tableau_foundation_transfer.verified);
        let mut malformed = before.clone();
        malformed.pixels.pop();
        assert!(tableau_foundation_transfer_source_changes(&malformed, &after, planned, MINIMUM_CONTENT_CHANGE).is_err());
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
    /// Face/glyph corruption, a changed stock or an overlay withdraw authority.
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


        for (corruption, rgb) in [
            (PixelRect::new(590, 212, 70, 24), [12, 62, 40]),
            (PixelRect::new(598, 160, 58, 42), [12, 62, 40]),
            (PixelRect::new(578, 159, 8, 81), [255, 255, 255]),
        ] {
            let mut damaged = frame.clone();
            paint(&mut damaged, corruption, rgb);
            assert_ne!(analyse(&damaged).unwrap().prediction, PredictedAction::Action(planned), "{corruption:?}");
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
            column: 6, top: 331, bottom: 953,
        }).is_none(), "source geometry above the unchanged scan stays unsupported");
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


        for bottom in 930..TOOLBAR_EDGE_OCCLUSION.bottom() {
            assert_eq!(find_solid_card_source(&frame, PixelRect::new(1_230, 332, CARD_WIDTH, bottom - 332)).unwrap(), None,
                "the known icon mask cannot complete a scan cropped at row{bottom}");
        }


        assert!(find_solid_card_source(&frame,
            PixelRect::new(1_230, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap().is_some());


        for bottom in TOOLBAR_EDGE_OCCLUSION.bottom()..=TABLEAU_OUTLINE_BOTTOM {
            assert_eq!(find_solid_card_source(&frame,
                PixelRect::new(1_230, 332, CARD_WIDTH, bottom - 332)).unwrap(),
                find_solid_card_source(&frame,
                    PixelRect::new(1_230, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap(),
                "complete observed column-6 mask remains valid at row{bottom}");
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
        assert!(canonical_action(KlondikeTarget::Tableau { column: 6, top: 331, bottom: 994 }).is_none(),
            "source geometry above the unchanged scan stays unsupported");
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
        assert!(canonical_action(KlondikeTarget::Tableau { column: 3, top: 799, bottom: TABLEAU_OUTLINE_BOTTOM as u16 + 1 }).is_none());
        let mut malformed = after;
        malformed.pixels.truncate(4);
        assert_eq!(inspect_effect(&before, &malformed, planned), Err(HaloDetectionError::InvalidFrameLayout));
    }


    /// K41's ten-card block adds three lower-border rows and is 604 pixels tall.
    /// The measured source supplies one safe click; the toolbar supplies no effect.
    #[test]
    fn recorded_ten_card_source_keeps_one_upper_click_and_excludes_toolbar_effects() {
        let before = fixture(41);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 6, top: 390, bottom: 994,
        }));
        assert_eq!(planned.operation(), InputOperation::Click(PixelPoint::new(1_296, 430)));
        assert_eq!(planned.effect_bounds(), PixelRect::new(1_230, 390, CARD_WIDTH, 557));
        assert_eq!(count_pixels(&before, TOOLBAR_ICON_SUPPORT, is_toolbar_undo_red), 68);
        assert_eq!(find_solid_card_source(&before, PixelRect::new(1_230, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap(),
            Some(PixelRect::new(1_230, 390, CARD_WIDTH, 604)));
        assert!(!completion_evidence(&before).unwrap().complete_candidate);
        let mut toolbar_changed = before.clone();
        paint(&mut toolbar_changed, PixelRect::new(378, TOOLBAR_TOP, 1_162, 1_033 - TOOLBAR_TOP), [255, 255, 255]);
        let report = inspect_effect(&before, &toolbar_changed, planned).unwrap();
        assert_eq!(report.source_changed, 0);
        assert_eq!(report.destination_changed, 0);
        assert!(!report.source_replaced && !report.verified, "{report}");
        assert!(canonical_action(KlondikeTarget::Tableau { column: 6, top: 331, bottom: 994 }).is_none());
        assert!(canonical_action(KlondikeTarget::Tableau { column: 6, top: 390, bottom: 999 }).is_none());
    }


    /// K41's mask only covers the measured Undo All overlap. Cropping, broken
    /// visible edges, absent icon, interrupted rails/top and dark paper still fail.
    #[test]
    fn ten_card_source_requires_complete_measured_outline_and_visible_paper() {
        let frame = fixture(41);


        for bottom in 930..TOOLBAR_EDGE_OCCLUSION.bottom() {
            assert_eq!(find_solid_card_source(&frame, PixelRect::new(1_230, 332, CARD_WIDTH, bottom - 332)).unwrap(), None,
                "the icon mask cannot complete a cropped scan ending at row{bottom}");
        }


        assert!(find_solid_card_source(&frame,
            PixelRect::new(1_230, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap().is_some());


        for bottom in TOOLBAR_EDGE_OCCLUSION.bottom()..=TABLEAU_OUTLINE_BOTTOM {
            assert_eq!(find_solid_card_source(&frame,
                PixelRect::new(1_230, 332, CARD_WIDTH, bottom - 332)).unwrap(),
                find_solid_card_source(&frame,
                    PixelRect::new(1_230, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap(),
                "complete observed column-6 mask remains valid at row{bottom}");
        }


        for corruption in [
            PixelRect::new(1_248, 988, 1, 6),
            PixelRect::new(1_307, 988, 1, 6),
            TOOLBAR_ICON_SUPPORT,
            PixelRect::new(1_218, 975, 156, TABLEAU_OUTLINE_BOTTOM - 975),
            PixelRect::new(1_218, 380, 156, 31),
            PixelRect::new(1_218, 620, 12, 24),
            PixelRect::new(1_248, 911, 96, 12),
        ] {
            let mut damaged = frame.clone();
            paint(&mut damaged, corruption, [12, 82, 45]);
            assert_eq!(analyse(&damaged).unwrap().prediction, PredictedAction::NoHighlight, "{corruption:?}");
        }

        let mut malformed = frame;
        malformed.pixels.truncate(4);
        assert_eq!(find_solid_card_source(&malformed, PixelRect::new(1_230, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)),
            Err(HaloDetectionError::InvalidFrameLayout));
    }


    /// Increasing the calibrated run height cannot let decoration, a recipient
    /// guide or an internal crossbar substitute for the connected source outline.
    #[test]
    fn ten_card_source_rejects_floating_gold_dashed_guides_and_internal_edges() {
        let frame = fixture(41);
        let mut empty = frame.clone();
        paint(&mut empty, PixelRect::new(1_218, 380, 156, TABLEAU_OUTLINE_BOTTOM - 380), [12, 82, 45]);
        let mut floating = empty.clone();
        paint(&mut floating, PixelRect::new(1_248, 993, 96, 1), [170, 120, 75]);
        copy_region(&frame, &mut floating, TOOLBAR_ICON_SUPPORT, PixelPoint::new(TOOLBAR_ICON_SUPPORT.x as i32, TOOLBAR_ICON_SUPPORT.y as i32));
        assert_eq!(analyse(&floating).unwrap().prediction, PredictedAction::NoHighlight);
        let mut dashed = empty.clone();
        copy_region(&frame, &mut dashed, PixelRect::new(726, 342, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(1_230, 390));
        assert_eq!(analyse(&dashed).unwrap().prediction, PredictedAction::NoHighlight);


        for rgb in [[0, 0, 0], [255, 255, 255]] {
            let mut cursor = empty.clone();
            paint(&mut cursor, PixelRect::new(1_276, 410, 40, 60), rgb);
            assert_eq!(analyse(&cursor).unwrap().prediction, PredictedAction::NoHighlight);
        }

        let mut internal_edge = frame;
        paint(&mut internal_edge, PixelRect::new(1_218, 975, 156, TABLEAU_OUTLINE_BOTTOM - 975), [12, 82, 45]);
        paint(&mut internal_edge, PixelRect::new(1_248, 944, 96, 1), [170, 120, 75]);
        assert_eq!(analyse(&internal_edge).unwrap().prediction, PredictedAction::NoHighlight);
    }


    /// K54's nine-card source retains its complete outline beneath Undo All.
    /// Its two warm shadow-tail pixels remain mandatory positive evidence;
    /// neither the click nor the effect ROI reaches the translucent toolbar.
    #[test]
    fn recorded_shadow_tail_source_keeps_one_safe_upper_action() {
        let frame = fixture(54);
        let planned = action(&frame);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 6, top: 390, bottom: 989,
        }));
        assert_eq!(planned.operation(), InputOperation::Click(PixelPoint::new(1_296, 430)));
        assert_eq!(planned.effect_bounds(), PixelRect::new(1_230, 390, CARD_WIDTH, 557));
        assert_eq!(find_solid_card_source(&frame,
            PixelRect::new(1_230, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap(),
            Some(PixelRect::new(1_230, 390, CARD_WIDTH, 599)));
        assert_eq!(count_pixels(&frame, TOOLBAR_ICON_SUPPORT, is_toolbar_undo_red), 69);


        for (x, rgb) in [(1_336, [91, 78, 40]), (1_337, [99, 84, 43])] {
            assert_eq!(pixel_rgb(&frame, x, 988), Some(rgb));
            assert!(!is_rail_gold(rgb), "ordinary rail colour must remain strict");
            assert!(is_toolbar_shadow_gold(rgb), "recorded tail requires positive warm gold");
        }
        assert!(!completion_evidence(&frame).unwrap().complete_candidate);
    }


    /// The new tail is not a larger hole in the lower edge. Either tail pixel or
    /// an adjacent required border sample still refuses felt, neutral/cool fill,
    /// white paper, red artwork and warmth below the calibrated shadow support.
    #[test]
    fn shadow_tail_and_adjacent_visible_edge_samples_remain_required() {
        let frame = fixture(54);


        for x in [1_248, 1_307, 1_336, 1_337, 1_338] {


            for rgb in [[12, 82, 45], [0, 0, 0], [255, 255, 255], [150, 20, 20], [89, 78, 40]] {
                let mut damaged = frame.clone();
                paint(&mut damaged, PixelRect::new(x, 988, 1, 1), rgb);
                assert_eq!(analyse(&damaged).unwrap().prediction, PredictedAction::NoHighlight,
                    "required lower-edge point ({x},988) with {rgb:?}");
            }
        }
    }


    /// K54 still needs observed termination, both closed ends, connected rails,
    /// positive Undo All artwork and paper above the toolbar. Shadow support
    /// cannot repair a clipped source or a dimmed recipient-like card interior.
    #[test]
    fn shadow_tail_source_preserves_complete_outline_icon_and_paper_guards() {
        let frame = fixture(54);


        for bottom in 930..TOOLBAR_EDGE_OCCLUSION.bottom() {
            assert_eq!(find_solid_card_source(&frame,
                PixelRect::new(1_230, 332, CARD_WIDTH, bottom - 332)).unwrap(), None,
                "shadow support cannot complete a scan cropped at row{bottom}");
        }


        assert!(find_solid_card_source(&frame,
            PixelRect::new(1_230, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap().is_some());


        for bottom in TOOLBAR_EDGE_OCCLUSION.bottom()..=TABLEAU_OUTLINE_BOTTOM {
            assert_eq!(find_solid_card_source(&frame,
                PixelRect::new(1_230, 332, CARD_WIDTH, bottom - 332)).unwrap(),
                find_solid_card_source(&frame,
                    PixelRect::new(1_230, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap(),
                "complete observed column-6 mask remains valid at row{bottom}");
        }


        for corruption in [TOOLBAR_ICON_SUPPORT, PixelRect::new(1_218, 975, 156, 19),
            PixelRect::new(1_218, 380, 156, 31), PixelRect::new(1_218, 850, 12, 24)]
        {
            let mut damaged = frame.clone();
            paint(&mut damaged, corruption, [12, 82, 45]);
            assert_eq!(analyse(&damaged).unwrap().prediction, PredictedAction::NoHighlight,
                "missing source evidence: {corruption:?}");
        }
        let mut dimmed_face = frame;
        paint(&mut dimmed_face, PixelRect::new(1_248, 911, 96, 12), [128, 128, 128]);
        assert_eq!(analyse(&dimmed_face).unwrap().prediction, PredictedAction::NoHighlight);
    }


    /// Translated source pixels are a labelled adverse derivative, not another
    /// calibrated column. A real dashed guide, floating gold and cursor-like
    /// fill also cannot become a complete source merely beside Undo All artwork.
    #[test]
    fn shadow_tail_context_rejects_other_columns_guides_and_floating_artwork() {
        let frame = fixture(54);
        let envelope = PixelRect::new(1_218, 380, 156, TABLEAU_OUTLINE_BOTTOM - 380);
        let mut empty = frame.clone();
        paint(&mut empty, envelope, [12, 82, 45]);
        copy_region(&frame, &mut empty, TOOLBAR_ICON_SUPPORT,
            PixelPoint::new(TOOLBAR_ICON_SUPPORT.x as i32, TOOLBAR_ICON_SUPPORT.y as i32));
        let mut translated = empty.clone();
        copy_region(&frame, &mut translated, envelope, PixelPoint::new(1_386, 380));
        assert_eq!(count_pixels(&translated, TOOLBAR_ICON_SUPPORT, is_toolbar_undo_red), 69);
        assert_eq!(analyse(&translated).unwrap().prediction, PredictedAction::NoHighlight);
        let mut dashed = empty.clone();
        copy_region(&frame, &mut dashed, PixelRect::new(558, 390, CARD_WIDTH, 200),
            PixelPoint::new(1_230, 390));
        assert_eq!(analyse(&dashed).unwrap().prediction, PredictedAction::NoHighlight);
        let mut floating = empty.clone();
        paint(&mut floating, PixelRect::new(1_248, 988, 96, 1), [170, 120, 75]);
        assert_eq!(analyse(&floating).unwrap().prediction, PredictedAction::NoHighlight);


        for rgb in [[0, 0, 0], [255, 255, 255]] {
            let mut cursor = empty.clone();
            paint(&mut cursor, PixelRect::new(1_276, 410, 40, 60), rgb);
            assert_eq!(analyse(&cursor).unwrap().prediction, PredictedAction::NoHighlight);
        }
    }


    /// Recognition under the icon shadow is independent of source-effect proof.
    /// An unchanged source and toolbar-only changes cannot verify a transfer.
    #[test]
    fn shadow_tail_recognition_never_supplies_unchanged_or_toolbar_effect_proof() {
        let before = fixture(54);
        let planned = action(&before);
        let unchanged = inspect_effect(&before, &before, planned).unwrap();
        assert_eq!((unchanged.source_changed, unchanged.destination_changed), (0, 0));
        assert!(!unchanged.source_replaced && !unchanged.verified);
        let mut toolbar_changed = before.clone();
        paint(&mut toolbar_changed,
            PixelRect::new(378, TOOLBAR_TOP, 1_162, 1_033 - TOOLBAR_TOP), [255, 255, 255]);
        let report = inspect_effect(&before, &toolbar_changed, planned).unwrap();
        assert_eq!((report.source_changed, report.destination_changed), (0, 0));
        assert!(!report.source_replaced && !report.verified);
    }


    /// Alter only the template sample's red channel for a bounded adverse probe.
    /// Unrepresentable values are left untouched rather than clamped into evidence.
    fn shift_solve_sample_red(frame: &mut CapturedFrame, border: bool, delta: i16) {
        let mut sample = 0;


        for y in (SOLVE_BOUNDS.y..SOLVE_BOUNDS.bottom()).step_by(4) {


            for x in (SOLVE_BOUNDS.x..SOLVE_BOUNDS.right()).step_by(4) {
                let expected = &SOLVE_TEMPLATE[sample..sample + 3];
                sample += 3;


                if SOLVE_STABLE_INTERIOR.contains(PixelPoint::new(x as i32, y as i32)) == border {
                    continue;
                }


                if let Ok(red) = u8::try_from(i16::from(expected[0]) + delta) {
                    paint(frame, PixelRect::new(x, y, 1, 1), [red, expected[1], expected[2]]);
                }
            }
        }
    }


    /// The settled warmer frame retains the stable face and glyphs. Outer-frame
    /// colour no longer grants authority; the interior tolerance stays unchanged.
    #[test]
    fn solve_animation_frame_does_not_change_interior_colour_tolerance() {
        let frame = fixture(42);
        assert!(has_solve_control(&frame));
        assert_eq!(action(&frame).target, ActionTarget::Klondike(KlondikeTarget::Solve));
        assert!(find_solid_card_source(&frame, PixelRect::new(558, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap().is_some());


        for delta in [27, -25] {
            let mut damaged = frame.clone();
            shift_solve_sample_red(&mut damaged, true, delta);
            assert!(has_solve_control(&damaged), "outer-frame red delta={delta}");
            assert_eq!(action(&damaged).target, ActionTarget::Klondike(KlondikeTarget::Solve));
        }

        let mut damaged = frame;
        shift_solve_sample_red(&mut damaged, false, 25);
        assert!(!has_solve_control(&damaged));
        assert_ne!(action(&damaged).target, ActionTarget::Klondike(KlondikeTarget::Solve));
    }


    /// A stable face still needs the check mark, word and independent empty stock.
    /// An overlay or cursor withdrawing the face never authorises a Solve click.
    #[test]
    fn solve_control_still_requires_interior_glyph_stock_and_scene() {
        let frame = fixture(42);


        for (corruption, rgb) in [
            (PixelRect::new(590, 212, 70, 24), [12, 62, 40]),
            (PixelRect::new(598, 160, 58, 42), [12, 62, 40]),
            (PixelRect::new(578, 159, 8, 81), [255, 255, 255]),
        ] {
            let mut damaged = frame.clone();
            paint(&mut damaged, corruption, rgb);
            let report = inspect_solve_control(&damaged).unwrap();
            assert!(!report.available, "{corruption:?}: {report}");


            if rgb == [255, 255, 255] {
                // The word/check mark still match; independently missing dark
                // face samples must withdraw authority despite intact glyphs.
                assert_eq!((report.interior_matched, report.glyph_matched), (441, 342));
            }
        }


        for rgb in [[0, 0, 0], [255, 255, 255]] {
            let mut cursor = frame.clone();
            paint(&mut cursor, PixelRect::new(590, 160, 70, 76), rgb);
            assert!(!has_solve_control(&cursor));
        }

        let mut occupied = frame.clone();
        copy_region(&fixture(1), &mut occupied, PixelRect::new(390, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(390, UPPER_Y as i32));
        assert!(!has_solve_control(&occupied));
        let mut overlay = frame;
        paint(&mut overlay, PixelRect::new(820, 600, 400, 200), [20, 20, 20]);
        assert_eq!(analyse(&overlay).unwrap().prediction, PredictedAction::NoHighlight);
    }


    /// K44 displaces one complete source fourteen pixels right of its slot.
    /// The fixed primitive still refuses it; only the bounded tableau scan adds
    /// this evidenced offset and keeps the canonical click inside the source.
    #[test]
    fn recorded_right_shifted_run_uses_one_safe_column_centre_click() {
        let frame = fixture(44);
        assert!(is_gameplay_scene(&frame).unwrap());
        let scan = PixelRect::new(390, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332);
        assert_eq!(find_solid_card_source(&frame, scan).unwrap(), None);
        assert_eq!(find_tableau_source(&frame, scan).unwrap(),
            Some(PixelRect::new(404, 442, CARD_WIDTH, 464)));
        let planned = action(&frame);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 1, top: 442, bottom: 906,
        }));
        assert_eq!(planned.operation(), InputOperation::Click(PixelPoint::new(456, 482)));
        assert_eq!(planned.effect_bounds(), PixelRect::new(390, 442, CARD_WIDTH, 464));
        let report = inspect_effect(&frame, &frame, planned).unwrap();
        assert!(!report.source_replaced && !report.verified, "unchanged source: {report}");
    }


    /// K45 is the earlier RIGHT Queen recommendation leading to K44's shifted
    /// six-card source. It is not the missing pre-action image for K46's later
    /// bottom-card transfer. This native pair proves its own separate move.
    #[test]
    fn recorded_queen_transfer_leads_to_the_right_shifted_six_card_source() {
        let before = fixture(45);
        let after = fixture(44);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Waste { offset: 2 }));
        let report = inspect_effect(&before, &after, planned).unwrap();
        assert!(report.source_replaced && report.verified, "{report}");
        assert!(report.destination_changed >= MINIMUM_CONTENT_CHANGE, "{report}");
        assert_eq!(action(&after).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 1, top: 442, bottom: 906,
        }));
    }


    /// The offset cannot supply its own missing edge, rail, paper or scene.
    #[test]
    fn shifted_tableau_run_keeps_closed_geometry_paper_and_scene_guards() {
        let frame = fixture(44);


        for bounds in [
            PixelRect::new(422, 897, 96, 9),
            PixelRect::new(395, 540, 9, 24),
            PixelRect::new(422, 432, 96, 12),
            PixelRect::new(422, 868, 96, 12),
        ] {
            let mut damaged = frame.clone();
            paint(&mut damaged, bounds, [12, 82, 45]);
            assert_eq!(analyse(&damaged).unwrap().prediction, PredictedAction::NoHighlight, "{bounds:?}");
        }
        let mut unsupported = frame.clone();
        paint(&mut unsupported, PixelRect::new(160, 310, 16, 16), [0, 0, 0]);
        assert_eq!(analyse(&unsupported).unwrap().prediction, PredictedAction::NoHighlight);
        let mut shifted_farther = frame.clone();
        paint(&mut shifted_farther, PixelRect::new(378, 430, 184, 486), [12, 82, 45]);
        copy_region(&frame, &mut shifted_farther, PixelRect::new(395, 432, 154, 480), PixelPoint::new(425, 432));
        assert_eq!(analyse(&shifted_farther).unwrap().prediction, PredictedAction::NoHighlight);
    }


    /// The later manual K47 capture already has a canonical connected run;
    /// the earlier unsupported worker frame was not saved as this PNG.
    #[test]
    fn recorded_third_stop_capture_already_has_one_complete_four_card_source() {
        let frame = fixture(47);
        assert!(is_gameplay_scene(&frame).unwrap());
        assert_eq!(find_solid_card_source(&frame, PixelRect::new(894, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap(),
            Some(PixelRect::new(894, 555, CARD_WIDTH, 354)));
        assert_eq!(action(&frame).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 4, top: 555, bottom: 909,
        }));
    }


    /// K48 matches the stable face without wider colour tolerances, and keeps
    /// Solve ahead of the still-valid remaining tableau source.
    #[test]
    fn recorded_later_solve_capture_is_already_recognised_before_tableau() {
        let frame = fixture(48);
        assert!(is_gameplay_scene(&frame).unwrap());
        assert!(has_solve_control(&frame));
        assert!(find_solid_card_source(&frame, PixelRect::new(390, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap().is_some());
        assert_eq!(action(&frame).target, ActionTarget::Klondike(KlondikeTarget::Solve));
    }


    /// Exercise the logged counts against the actual later K46 outline. These
    /// counts came from unsaved worker frames; this is not a native move pair.
    #[test]
    fn logged_bottom_print_counts_need_material_opposed_print_and_exact_contraction() {
        let frame = fixture(46);
        let planned = canonical_action(KlondikeTarget::Tableau { column: 1, top: 799, bottom: 989 }).unwrap();
        assert_eq!(action(&frame).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 1, top: 773, bottom: 963,
        }));
        assert!(visible_contracted_source_replaced(&frame, planned, 687, [[36, 207], [142, 58]]).unwrap());


        for counts in [
            [[0, 207], [142, 58]], [[36, 0], [142, 58]],
            [[36, 207], [0, 58]], [[36, 207], [142, 0]],
            [[1, 94], [142, 58]], [[36, 207], [58, 58]],
            [[36, 37], [142, 58]], [[36, 207], [47, 48]],
        ] {
            assert!(!visible_contracted_source_replaced(&frame, planned, 687, counts).unwrap(), "{counts:?}");
        }
        assert!(!visible_contracted_source_replaced(&frame, planned, 511, [[36, 207], [142, 58]]).unwrap());


        for target in [
            KlondikeTarget::Tableau { column: 1, top: 798, bottom: 988 },
            KlondikeTarget::Tableau { column: 1, top: 800, bottom: 990 },
            KlondikeTarget::Tableau { column: 2, top: 799, bottom: 989 },
            KlondikeTarget::Tableau { column: 1, top: 773, bottom: 963 },
            KlondikeTarget::Waste { offset: 0 }, KlondikeTarget::Foundation { column: 1 },
        ] {
            let different = canonical_action(target).unwrap();
            assert!(!visible_contracted_source_replaced(&frame, different, 687, [[36, 207], [142, 58]]).unwrap(), "{target:?}");
        }
        let mut malformed = frame.clone();
        malformed.pixels.truncate(1);
        assert!(visible_contracted_source_replaced(&malformed, planned, 687, [[36, 207], [142, 58]]).is_err());
    }


    /// Reconstruct a deliberately synthetic previous four-diamond source from
    /// native K39 rails and K44 card artwork over K46's later board. The true
    /// pre-action frame was not captured, so no live effect is inferred here.
    fn synthetic_bottom_contraction() -> (CapturedFrame, CapturedFrame, GuidedAction) {
        let after = fixture(46);
        let mut before = after.clone();
        paint(&mut before, PixelRect::new(378, 760, 156, 234), [12, 82, 45]);
        copy_region(&fixture(39), &mut before, PixelRect::new(716, 799, 154, 190), PixelPoint::new(380, 799));
        copy_region(&fixture(44), &mut before, PixelRect::new(614, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(390, 805));
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 1, top: 799, bottom: 989,
        }));
        (before, after, planned)
    }


    /// Exercise production pixel exclusions before the contracted-outline route;
    /// no independent recipient or complete effect is invented by this pair.
    #[test]
    fn synthetic_bottom_contraction_supports_continuation_only() {
        let (before, after, planned) = synthetic_bottom_contraction();
        let report = inspect_effect(&before, &after, planned).unwrap();
        println!("synthetic contraction: {report}");
        assert!(report.source_replaced && !report.verified, "{report}");
        assert!(report.source_changed >= MINIMUM_CONTENT_CHANGE, "{report}");
        assert!(report.source_visible_identity_directions.into_iter().flatten().any(|count| count < MINIMUM_CORNER_CHANGE), "old per-corner route must remain insufficient: {report}");
        assert!(!inspect_effect(&before, &before, planned).unwrap().source_replaced);
    }


    /// The translated recogniser retains a conservative fixed source ROI. It
    /// must not borrow a receiver or decoration to establish source replacement.
    #[test]
    fn shifted_source_effect_roi_keeps_removal_recipient_and_mask_separation() {
        let before = fixture(44);
        let planned = action(&before);
        let mut source_only = before.clone();
        paint(&mut source_only, PixelRect::new(395, 432, 154, 480), [12, 82, 45]);
        let report = inspect_effect(&before, &source_only, planned).unwrap();
        assert!(report.source_replaced && !report.verified, "{report}");
        assert!(report.source_positive >= MINIMUM_CONTENT_CHANGE, "{report}");
        let mut receiver_only = before.clone();
        paint(&mut receiver_only, PixelRect::new(1_078, 130, 100, 139), [245, 245, 245]);
        let report = inspect_effect(&before, &receiver_only, planned).unwrap();
        assert!(report.destination_changed >= MINIMUM_CONTENT_CHANGE, "{report}");
        assert!(!report.source_replaced && !report.verified, "{report}");


        for (bounds, rgb) in [
            (PixelRect::new(440, 466, 32, 32), [0, 0, 0]),
            (PixelRect::new(406, 470, 96, 32), [170, 120, 75]),
            (PixelRect::new(390, TOOLBAR_TOP, 132, 44), [0, 0, 0]),
        ] {
            let mut masked = before.clone();
            paint(&mut masked, bounds, rgb);
            let report = inspect_effect(&before, &masked, planned).unwrap();
            assert!(!report.source_replaced && !report.verified, "{bounds:?}: {report}");
        }
    }


    /// Contracted-source continuation still needs two independent full patches;
    /// erasing one patch or recolouring either paper support fails the pixel path.
    #[test]
    fn synthetic_contraction_keeps_opposed_patch_paper_and_cursor_guards() {
        let (before, after, planned) = synthetic_bottom_contraction();


        for patch in [PixelRect::new(395, 804, 28, 44), PixelRect::new(489, 903, 28, 44)] {
            let mut restored = after.clone();
            copy_region(&before, &mut restored, patch, PixelPoint::new(patch.x as i32, patch.y as i32));
            paint(&mut restored, PixelRect::new(430, 891, 40, 10), [190, 190, 190]);
            let report = inspect_effect(&before, &restored, planned).unwrap();
            assert!(report.source_changed >= MINIMUM_CONTENT_CHANGE, "{report}");
            assert!(!report.source_replaced && !report.verified, "restored {patch:?}: {report}");
            let mut dark = after.clone();
            paint(&mut dark, patch, [170, 120, 75]);
            let report = inspect_effect(&before, &dark, planned).unwrap();
            assert!(!report.source_replaced && !report.verified, "dark {patch:?}: {report}");
        }
        let mut cursor_only = before.clone();
        paint(&mut cursor_only, PixelRect::new(440, 823, 32, 32), [0, 0, 0]);
        let report = inspect_effect(&before, &cursor_only, planned).unwrap();
        assert!(!report.source_replaced && !report.verified, "{report}");
        let mut malformed = after;
        malformed.pixels.truncate(1);
        assert!(inspect_effect(&before, &malformed, planned).is_err());
    }


    /// Diagnostics retain the whole-frame count beside the stable authoritative
    /// interior even when stock support fails; they never grant scene/input rights.
    #[test]
    fn solve_diagnostic_reports_full_artwork_and_independent_empty_stock_guard() {
        let frame = fixture(48);
        assert_eq!(inspect_solve_control(&frame).unwrap(), SolveEvidence {
            available: true, stock_felt_pixels: 13_900, stock_probe_pixels: 13_900,
            artwork_matched: 899, artwork_samples: 899, interior_matched: 483, interior_samples: 483,
            glyph_matched: 342, glyph_samples: 342,
        });
        let mut stock_occupied = frame.clone();
        copy_region(&fixture(1), &mut stock_occupied, PixelRect::new(390, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(390, UPPER_Y as i32));
        let report = inspect_solve_control(&stock_occupied).unwrap();
        assert!(!report.available && report.stock_felt_pixels < 12_510, "{report}");
        assert_eq!((report.artwork_matched, report.glyph_matched), (899, 342));
        let mut malformed = frame;
        malformed.pixels.truncate(1);
        assert!(inspect_solve_control(&malformed).is_err());
    }


    /// Remove only the exterior of the settled K48 control. This is a controlled
    /// derivative, not a captured animation phase; its unchanged native interior
    /// must select Solve before the genuine remaining tableau HALO.
    #[test]
    fn solve_control_without_gold_animation_frame_preserves_tableau_priority() {
        let recorded = fixture(48);


        for rgb in [[12, 82, 45], [0, 0, 0], [255, 255, 255]] {
            let mut derivative = recorded.clone();
            paint(&mut derivative, SOLVE_BOUNDS, rgb);
            copy_region(&recorded, &mut derivative, SOLVE_STABLE_INTERIOR,
                PixelPoint::new(SOLVE_STABLE_INTERIOR.x as i32, SOLVE_STABLE_INTERIOR.y as i32));
            let report = inspect_solve_control(&derivative).unwrap();
            assert!(report.available, "{report}");
            assert!(report.artwork_matched * 100 < report.artwork_samples * 98, "{report}");
            assert_eq!((report.interior_matched, report.glyph_matched), (483, 342));
            assert!(find_solid_card_source(&derivative,
                PixelRect::new(390, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap().is_some());
            assert_eq!(action(&derivative).target, ActionTarget::Klondike(KlondikeTarget::Solve));
        }
    }


    /// All native recorded controls match; every other supplied frame lacks the
    /// stable face. This includes card/waste scenes, terminal decoration and empty
    /// outlines, without generalising gold colour into a completion-request button.
    #[test]
    fn stable_solve_face_is_specific_across_the_original_recorded_corpus() {


        for number in 1..=49 {
            let report = inspect_solve_control(&fixture(number)).unwrap();
            assert_eq!(report.available, [13, 25, 37, 42, 48].contains(&number), "K{number:02}: {report}");
        }
    }


    /// Withdraw selected non-glyph face samples from the recorded settled K48.
    /// This controlled derivative reproduces diagnostic counts only; the native
    /// early frames were not retained and their changed pixel locations are unknown.
    fn solve_with_face_sample_losses(losses: usize) -> CapturedFrame {
        let mut frame = fixture(48);
        let mut changed = 0;


        for y in (SOLVE_BOUNDS.y..SOLVE_BOUNDS.bottom()).step_by(4) {


            for x in (SOLVE_BOUNDS.x..SOLVE_BOUNDS.right()).step_by(4) {


                if changed < losses
                    && SOLVE_STABLE_INTERIOR.contains(PixelPoint::new(x as i32, y as i32))
                    && !((590..660).contains(&x) && (160..236).contains(&y))
                {
                    paint(&mut frame, PixelRect::new(x, y, 1, 1), [255, 255, 255]);
                    changed += 1;
                }
            }
        }
        assert_eq!(changed, losses, "requested losses exceed the non-glyph face");
        frame
    }


    /// Logged 459/483 face and 342/342 glyph counts request observation only.
    /// The valid lower tableau remains a prediction until the worker's bounded
    /// priority guard settles the face; partial artwork cannot authorise Solve.
    #[test]
    fn solve_nearly_matching_face_requests_settling_without_click_authority() {
        let derivative = solve_with_face_sample_losses(24);
        let report = inspect_solve_control(&derivative).unwrap();
        assert_eq!((report.interior_matched, report.interior_samples), (459, 483));
        assert_eq!((report.glyph_matched, report.glyph_samples), (342, 342));
        assert_eq!((report.stock_felt_pixels, report.stock_probe_pixels), (13_900, 13_900));
        assert!(report.awaiting_settle(), "{report}");
        assert!(!report.available && !has_solve_control(&derivative), "{report}");
        assert!(matches!(action(&derivative).target, ActionTarget::Klondike(KlondikeTarget::Tableau { .. })));
        let settled = inspect_solve_control(&fixture(48)).unwrap();
        assert!(settled.available && !settled.awaiting_settle(), "{settled}");
    }


    /// The 95% face boundary is diagnostic only and stock/glyph support remains
    /// independently mandatory; zero-length or damaged probes cannot request it.
    #[test]
    fn solve_pending_settle_preserves_exact_face_glyph_and_stock_bounds() {
        let supported = inspect_solve_control(&solve_with_face_sample_losses(24)).unwrap();
        assert!(supported.awaiting_settle());
        let insufficient_face = inspect_solve_control(&solve_with_face_sample_losses(25)).unwrap();
        assert_eq!(insufficient_face.interior_matched, 458);
        assert!(!insufficient_face.available && !insufficient_face.awaiting_settle());


        for (field, accepted, refused) in [("glyph", 336, 335), ("stock", 12_510, 12_509)] {
            let mut boundary = supported;


            match field {
                "glyph" => boundary.glyph_matched = accepted,
                "stock" => boundary.stock_felt_pixels = accepted,
                _ => unreachable!("test field is bounded"),
            }
            assert!(boundary.awaiting_settle(), "{field} minimum: {boundary}");


            match field {
                "glyph" => boundary.glyph_matched = refused,
                "stock" => boundary.stock_felt_pixels = refused,
                _ => unreachable!("test field is bounded"),
            }
            assert!(!boundary.awaiting_settle(), "{field} below minimum: {boundary}");
        }


        for missing in ["stock", "interior", "glyph"] {
            let mut boundary = supported;


            match missing {
                "stock" => boundary.stock_probe_pixels = 0,
                "interior" => boundary.interior_samples = 0,
                "glyph" => boundary.glyph_samples = 0,
                _ => unreachable!("test field is bounded"),
            }
            assert!(!boundary.awaiting_settle(), "missing {missing} probe: {boundary}");
        }
        let mut occupied = solve_with_face_sample_losses(24);
        copy_region(&fixture(1), &mut occupied, PixelRect::new(390, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(390, UPPER_Y as i32));
        assert!(!inspect_solve_control(&occupied).unwrap().awaiting_settle());
        let mut missing_glyph = solve_with_face_sample_losses(24);
        paint(&mut missing_glyph, PixelRect::new(590, 212, 70, 24), [12, 62, 40]);
        assert!(!inspect_solve_control(&missing_glyph).unwrap().awaiting_settle());
        let mut malformed = occupied;
        malformed.pixels.truncate(1);
        assert!(inspect_solve_control(&malformed).is_err());
    }


    /// Native card, terminal and complete Solve scenes never request this new
    /// partial-face wait; early live face pixels are covered only by derivatives.
    #[test]
    fn solve_pending_settle_is_absent_from_all_recorded_settled_scenes() {


        for number in 1..=52 {
            let report = inspect_solve_control(&fixture(number)).unwrap();
            assert!(!report.awaiting_settle(), "K{number:02}: {report}");
        }
    }


    /// Charlie reconstructed the preceding 4-clubs source using Undo after the
    /// stopped run. Native K50/K51 pixels show 5-diamonds exposed in column 1
    /// and the next 5-clubs source in column 3. They are not the saved worker
    /// frames, whose corner counts differ slightly from this later attachment.
    #[test]
    fn recorded_bottom_club_removal_continues_to_a_different_tableau_column() {
        let before = fixture(50);
        let after = fixture(51);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 1, top: 799, bottom: 989,
        }));
        assert_eq!(action(&after).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 3, top: 610, bottom: 800,
        }));
        let report = inspect_effect(&before, &after, planned).unwrap();
        assert_eq!(report.source_changed, 687, "{report}");
        assert_eq!(report.source_positive, 61, "{report}");
        assert_eq!(report.source_identity_directions, [[0, 0], [0, 0]], "clipped ordinary corners stay unavailable");
        assert_eq!(report.source_visible_identity_directions, [[35, 209], [141, 58]], "{report}");
        assert_eq!(report.source_visible_print.new_ink, 550, "{report}");
        assert_eq!(report.source_visible_print.cleared_ink, 54, "{report}");
        assert_eq!(report.source_visible_print.stable_paper, 3_925, "{report}");
        assert_eq!(report.source_visible_print.changed_paper_fields, 0, "{report}");
        assert!(report.source_visible_print.paper_supported && report.source_visible_print.verified, "{report}");
        assert_eq!(report.destination_changed, 1_244, "next HALO shadow supplies the broad diagnostic: {report}");
        assert_eq!(content_changes(&before, &after, PixelRect::new(1_398, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), planned, false),
            318, "the actual receiving foundation does not meet the general recipient bound");
        assert!(report.source_replaced && !report.verified, "continuation remains separate from complete effect: {report}");
        assert!(!inspect_effect(&before, &before, planned).unwrap().source_replaced);
    }


    /// Material outside the source is only an extra restriction. A new HALO
    /// shadow or receiving foundation without source proof cannot grant either
    /// continuation or complete effect. Stable source print without that fresh
    /// non-source context also leaves the narrower fallback unavailable.
    #[test]
    fn stable_bottom_print_fallback_requires_independent_source_and_fresh_context() {
        let before = fixture(50);
        let completed = fixture(51);
        let planned = action(&before);
        let source = planned.effect_bounds();
        let mut source_only = before.clone();
        copy_region(&completed, &mut source_only, source, PixelPoint::new(source.x as i32, source.y as i32));
        let report = inspect_effect(&before, &source_only, planned).unwrap();
        assert!(report.source_visible_print.verified, "{report}");
        assert_eq!(report.destination_changed, 0, "{report}");
        assert!(!report.source_replaced && !report.verified, "{report}");
        let mut halo_only = before.clone();
        copy_region(&completed, &mut halo_only, PixelRect::new(716, 600, 154, 212), PixelPoint::new(716, 600));
        let report = inspect_effect(&before, &halo_only, planned).unwrap();
        assert_eq!(report.source_changed, 0, "{report}");
        assert_eq!(report.destination_changed, 1_244, "{report}");
        assert!(!report.source_replaced && !report.verified, "{report}");
        let mut recipient_and_halo = halo_only;
        copy_region(&completed, &mut recipient_and_halo,
            PixelRect::new(1_398, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(1_398, UPPER_Y as i32));
        let report = inspect_effect(&before, &recipient_and_halo, planned).unwrap();
        assert_eq!(report.source_changed, 0, "{report}");
        assert!(!report.source_replaced && !report.verified, "{report}");
    }


    /// Both visible patches and unchanged neutral paper fields remain mandatory.
    /// A neutral bright-paper fade is rejected even though it keeps the native
    /// opposed print counts, 512 material source pixels and the fresh target.
    #[test]
    fn stable_bottom_print_fallback_rejects_missing_patches_and_paper_fading() {
        let before = fixture(50);
        let completed = fixture(51);
        let planned = action(&before);


        for patch in [PixelRect::new(395, 804, 28, 44), PixelRect::new(489, 903, 28, 44)] {
            let mut one_patch = completed.clone();
            copy_region(&before, &mut one_patch, patch, PixelPoint::new(patch.x as i32, patch.y as i32));
            paint(&mut one_patch, PixelRect::new(425, 888, 40, 20), [190, 190, 190]);
            let report = inspect_effect(&before, &one_patch, planned).unwrap();
            assert!(report.source_changed >= MINIMUM_CONTENT_CHANGE, "{report}");
            assert!(!report.source_replaced && !report.verified, "restored patch {patch:?}: {report}");
        }
        let mut faded = completed.clone();
        paint(&mut faded, PixelRect::new(425, 888, 20, 6), [230, 230, 230]);
        let report = inspect_effect(&before, &faded, planned).unwrap();
        assert_eq!(report.source_visible_identity_directions, [[35, 209], [141, 58]], "print itself is retained: {report}");
        assert!(report.source_visible_print.changed_paper_fields > 0, "{report}");
        assert!(!report.source_replaced && !report.verified, "paper-field fading is not replacement: {report}");
    }


    /// The fallback uses neither gold, cursor nor toolbar change. Neutral guide
    /// darkening and whitening have one-way print changes and remain refused,
    /// even when the valid later HALO and actual receiving foundation are retained.
    #[test]
    fn stable_bottom_print_fallback_refuses_cursor_guides_and_toolbar_changes() {
        let before = fixture(50);
        let completed = fixture(51);
        let planned = action(&before);
        let mut outside_source = before.clone();
        copy_region(&completed, &mut outside_source, PixelRect::new(716, 600, 154, 212), PixelPoint::new(716, 600));
        copy_region(&completed, &mut outside_source,
            PixelRect::new(1_398, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(1_398, UPPER_Y as i32));


        for (name, bounds, rgb) in [
            ("cursor", PixelRect::new(409, 792, 94, 94), [0, 0, 0]),
            ("white cursor", PixelRect::new(409, 792, 94, 94), [255, 255, 255]),
            ("gold", planned.effect_bounds(), [230, 185, 70]),
            ("neutral guide", planned.effect_bounds(), [32, 32, 32]),
            ("neutral fading", planned.effect_bounds(), [190, 190, 190]),
            ("whitening", planned.effect_bounds(), [255, 255, 255]),
            ("toolbar", PixelRect::new(390, TOOLBAR_TOP, CARD_WIDTH, 86), [255, 255, 255]),
        ] {
            let mut after = outside_source.clone();
            paint(&mut after, bounds, rgb);
            let report = inspect_effect(&before, &after, planned).unwrap();
            assert!(!report.source_replaced && !report.verified, "{name}: {report}");
        }
        let mut malformed = completed;
        malformed.pixels.truncate(1);
        assert!(inspect_effect(&before, &malformed, planned).is_err());
    }


    /// K55 is a settled Undo reconstruction; K56 is a later result capture.
    /// They preserve the logged geometry, not the unsaved worker PNG bytes.
    #[test]
    fn recorded_clipped_source_spread_uses_aligned_print_for_continuation_only() {
        let before = fixture(55);
        let after = fixture(56);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 2, top: 804, bottom: 993,
        }));
        assert_eq!(action(&after).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 2, top: 799, bottom: 988,
        }));
        let report = inspect_effect(&before, &after, planned).unwrap();
        assert_eq!((report.source_changed, report.source_positive), (373, 310));
        let print = report.source_aligned_print;
        assert!(print.geometry_supported && print.verified, "{report}");
        assert_eq!((print.upward_shift, print.visible_rows), (5, 143));
        assert_eq!((print.material_changed, print.new_ink, print.cleared_ink), (564, 62, 235));
        assert_eq!((print.stable_paper, print.eligible), (7104, 7731));
        assert_eq!((print.core_stable_paper, print.core_eligible, print.changed_paper_fields), (3493, 3785, 0));
        assert!(report.destination_changed >= MINIMUM_CONTENT_CHANGE, "{report}");
        assert!(report.source_replaced && !report.verified, "source continuation is not complete effect: {report}");
        assert!(!completion_evidence(&after).unwrap().complete_candidate);
    }


    /// Restore identical old face pixels at the new relative source position.
    /// Outline displacement and real recipient artwork are deliberately retained.
    fn translated_unchanged_clipped_source() -> (CapturedFrame, CapturedFrame, GuidedAction) {
        let before = fixture(55);
        let mut after = fixture(56);
        copy_region(&before, &mut after, PixelRect::new(563, 814, 122, 128), PixelPoint::new(563, 809));
        let planned = action(&before);
        assert_eq!(action(&after).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 2, top: 799, bottom: 988,
        }));
        (before, after, planned)
    }


    /// Printed identity must change after alignment: a translated same card,
    /// its new outline or a pointer/gold overlay cannot supply that change.
    #[test]
    fn aligned_clipped_source_rejects_pure_translation_cursor_and_gold() {
        let (before, translated, planned) = translated_unchanged_clipped_source();
        let unchanged = aligned_source_print_changes(&before, &before, planned).unwrap();
        assert!(!unchanged.geometry_supported && !unchanged.verified);
        let print = aligned_source_print_changes(&before, &translated, planned).unwrap();
        assert!(print.geometry_supported, "{print:?}");
        assert_eq!((print.material_changed, print.new_ink, print.cleared_ink), (0, 0, 0));
        assert!(!print.verified, "pure translation is not replacement: {print:?}");


        for (bounds, rgb) in [
            (PixelRect::new(624, 844, 24, 32), [0, 0, 0]),
            (PixelRect::new(624, 844, 24, 32), [255, 255, 255]),
            (PixelRect::new(568, 820, 4, 32), [230, 185, 70]),
        ] {
            let mut different = translated.clone();
            paint(&mut different, bounds, rgb);
            let print = aligned_source_print_changes(&before, &different, planned).unwrap();
            assert!(!print.verified, "{bounds:?}, {rgb:?}: {print:?}");
            assert_eq!(print.material_changed, 0, "excluded overlay supplies no material proof: {print:?}");
        }
    }


    /// A fresh valid source and a real recipient cannot replace either of the
    /// independently required paper-to-ink and ink-to-paper source directions.
    #[test]
    fn aligned_clipped_source_rejects_each_one_way_print_transition() {
        let before = fixture(55);
        let completed = fixture(56);
        let planned = action(&before);


        for remove_new in [true, false] {
            let mut different = completed.clone();


            for y in 814..942 {


                for x in 563..685 {
                    let first = pixel_rgb(&before, x, y).unwrap();
                    let second = pixel_rgb(&completed, x, y - 5).unwrap();


                    let remove = if remove_new {
                        is_bright_neutral_paper(first) && is_card_ink(second)
                    } else {
                        is_card_ink(first) && is_bright_neutral_paper(second)
                    };


                    if remove {
                        let offset = (y - 5) as usize * different.stride + x as usize * 4;
                        different.pixels[offset..offset + 3].copy_from_slice(&first);
                    }
                }
            }
            let print = aligned_source_print_changes(&before, &different, planned).unwrap();
            assert!(print.geometry_supported, "the other source guards remain applicable: {print:?}");
            assert_eq!(if remove_new { print.new_ink } else { print.cleared_ink }, 0, "{print:?}");
            assert!(!print.verified, "both printed directions remain mandatory: {print:?}");
        }
    }


    /// Neutral source fading retains paper occupancy but fails stable core fields.
    /// Dark/chromatic source overlays cannot substitute for a bright printed card.
    #[test]
    fn aligned_clipped_source_rejects_core_fade_and_unsupported_paper() {
        let before = fixture(55);
        let completed = fixture(56);
        let planned = action(&before);
        let mut faded = completed.clone();


        for y in 815..926 {


            for x in 574..674 {
                let rgb = pixel_rgb(&completed, x, y).unwrap();


                if is_bright_neutral_paper(rgb) {
                    let offset = y as usize * faded.stride + x as usize * 4;
                    faded.pixels[offset..offset + 3].copy_from_slice(&rgb.map(|value| value.saturating_sub(12)));
                }
            }
        }
        let print = aligned_source_print_changes(&before, &faded, planned).unwrap();
        assert!(print.geometry_supported && print.paper_supported, "{print:?}");
        assert!(print.changed_paper_fields > 0 && !print.verified, "retained paper fading is not printed replacement: {print:?}");


        for rgb in [[32, 32, 32], [60, 110, 75], [230, 185, 70]] {
            let mut different = completed.clone();
            paint(&mut different, PixelRect::new(563, 809, 122, 128), rgb);
            let print = aligned_source_print_changes(&before, &different, planned).unwrap();
            assert!(!print.verified, "unsupported source paper {rgb:?}: {print:?}");
        }
    }


    /// Source replacement and material non-source change remain independent.
    /// Neither a receiving foundation alone nor a source-only replacement can
    /// authorise this bounded continuation, even when its visible print is real.
    #[test]
    fn aligned_clipped_source_keeps_non_source_and_fresh_target_requirements() {
        let before = fixture(55);
        let completed = fixture(56);
        let planned = action(&before);
        let mut recipient_only = before.clone();
        copy_region(&completed, &mut recipient_only, PixelRect::new(1398, UPPER_Y, CARD_WIDTH, CARD_HEIGHT), PixelPoint::new(1398, UPPER_Y as i32));
        let report = inspect_effect(&before, &recipient_only, planned).unwrap();
        assert!(report.destination_changed >= MINIMUM_CONTENT_CHANGE, "{report}");
        assert!(!report.source_aligned_print.verified && !report.source_replaced && !report.verified, "recipient alone: {report}");
        let mut source_only = before.clone();
        copy_region(&completed, &mut source_only, PixelRect::new(548, 332, 152, 662), PixelPoint::new(548, 332));
        let report = inspect_effect(&before, &source_only, planned).unwrap();
        assert!(report.source_aligned_print.verified, "source print is independently real: {report}");
        assert_eq!(report.destination_changed, 0, "source column changes are not recipient proof: {report}");
        assert!(!report.source_replaced && !report.verified, "no non-source change: {report}");
        let mut missing_source = completed;
        paint(&mut missing_source, PixelRect::new(690, 799, 10, 189), [12, 82, 45]);
        assert_eq!(analyse(&missing_source).unwrap().prediction, PredictedAction::NoHighlight);
        assert!(!aligned_source_print_changes(&before, &missing_source, planned).unwrap().geometry_supported);
    }


    /// Both source identities must retain canonical equal-height single-card
    /// geometry in the same column, within the existing upward contraction bound.
    #[test]
    fn aligned_clipped_source_rejects_other_columns_heights_and_excessive_shift() {
        let before = fixture(55);
        let completed = fixture(56);
        let planned = action(&before);


        for (column, point) in [(3, PixelPoint::new(716, 799)), (2, PixelPoint::new(548, 777))] {
            let mut different = completed.clone();
            paint(&mut different, PixelRect::new(548, 799, 152, 195), [12, 82, 45]);
            copy_region(&completed, &mut different, PixelRect::new(548, 799, 152, 189), point);
            let expected_top = point.y as u16;
            assert_eq!(action(&different).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
                column, top: expected_top, bottom: expected_top + 189,
            }));
            let print = aligned_source_print_changes(&before, &different, planned).unwrap();
            assert!(!print.geometry_supported && !print.verified, "different source geometry: {print:?}");
        }
        let mut different_height = before.clone();
        copy_region(&before, &mut different_height, PixelRect::new(548, 804, 152, 143), PixelPoint::new(548, 803));
        let different_plan = action(&different_height);
        assert_eq!(different_plan.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 2, top: 803, bottom: 993,
        }));
        let print = aligned_source_print_changes(&different_height, &completed, different_plan).unwrap();
        assert!(!print.geometry_supported && !print.verified, "unequal source heights: {print:?}");


        for target in [
            KlondikeTarget::Tableau { column: 2, top: 803, bottom: 993 },
            KlondikeTarget::Tableau { column: 2, top: 804, bottom: 992 },
            KlondikeTarget::Tableau { column: 1, top: 501, bottom: 691 },
            KlondikeTarget::Waste { offset: 2 }, KlondikeTarget::Foundation { column: 4 },
        ] {
            let other = canonical_action(target).unwrap();
            assert!(!aligned_source_print_changes(&before, &completed, other).unwrap().geometry_supported, "{target:?}");
        }
        let mut malformed = completed.clone();
        malformed.pixels.truncate(1);
        assert_eq!(aligned_source_print_changes(&before, &malformed, planned), Err(HaloDetectionError::InvalidFrameLayout));
        let mut wrong_dimensions = completed;
        wrong_dimensions.width -= 1;
        assert!(aligned_source_print_changes(&before, &wrong_dimensions, planned).is_err());
    }


    /// K57 is the settled Undo reconstruction and K58 is the attached result.
    /// Their retained PNG pixels calibrate this ordinary exposed-card route;
    /// they are not the unsaved worker frames with source counts 339/284.
    #[test]
    fn recorded_ordinary_clipped_source_continues_to_a_different_column() {
        let before = fixture(57);
        let after = fixture(58);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 3, top: 803, bottom: 992,
        }));
        assert_eq!(action(&after).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 7, top: 425, bottom: 615,
        }));
        assert_eq!(ordinary_source_face_top(&after, planned.effect_bounds(), 808), Some(803));
        assert!(!aligned_source_print_changes(&before, &after, planned).unwrap().geometry_supported,
            "the existing same-column HALO alignment is unavailable");
        let report = inspect_effect(&before, &after, planned).unwrap();
        assert_eq!((report.source_changed, report.source_positive), (343, 288), "{report}");
        let print = report.source_aligned_print;
        assert!(print.geometry_supported && print.ordinary_face_geometry && print.paper_supported && print.verified,
            "{report}");
        assert_eq!((print.ordinary_face_top, print.upward_shift, print.visible_rows), (803, 5, 144));
        assert_eq!((print.material_changed, print.new_ink, print.cleared_ink), (553, 82, 189));
        assert_eq!((print.stable_paper, print.eligible), (7_159, 7_853));
        assert_eq!((print.core_stable_paper, print.core_eligible, print.changed_paper_fields), (3_608, 3_885, 0));
        assert!(report.destination_changed >= MINIMUM_CONTENT_CHANGE && report.destination_print.verified,
            "the ordinary source needs both independent receiving restrictions: {report}");
        assert_eq!(report.destination_print.column, 1);
        assert_eq!(report.destination_print.new_ink_halves, [176, 184]);
        assert_eq!((report.destination_print.stable_paper, report.destination_print.changed_paper), (12_483, 0));
        assert!(report.destination_print.paper_supported && report.destination_print.guide_stable, "{report}");
        assert!(report.source_replaced && !report.verified,
            "ordinary source evidence permits continuation only: {report}");
        assert!(!completion_evidence(&after).unwrap().complete_candidate);
        let unchanged = inspect_effect(&before, &before, planned).unwrap();
        assert!(!unchanged.source_replaced && !unchanged.verified, "{unchanged}");
    }


    /// The unchanged column-7 print independently calibrates the five-row
    /// ordinary seam/paired-outline offset without consulting replacement counts.
    #[test]
    fn ordinary_face_offset_is_calibrated_by_unchanged_column_seven_print() {
        let before = fixture(57);
        let after = fixture(58);
        let ActionTarget::Klondike(KlondikeTarget::Tableau { column, top, bottom }) = action(&after).target else {
            panic!("expected the recorded column-7 source");
        };
        assert_eq!((column, top, bottom), (7, 425, 615));
        assert_eq!(top + HIGHLIGHTED_NOMINAL_FACE_OFFSET, 430);


        for x in 1_416..1_512 {
            assert_eq!(pixel_rgb(&before, x, 430), Some([212, 212, 212]), "gray seam x={x}");
        }
        let unchanged_print_pixels = (435..475).flat_map(|y| (1_403..1_431).map(move |x| (x, y)))
            .filter(|&(x, y)| pixel_rgb(&before, x, y) == pixel_rgb(&after, x, y)).count();
        assert_eq!(unchanged_print_pixels, 1_120, "the complete 28 by 40 printed patch is identical");
    }


    /// A unique closed neutral seam, bright adjacent rows, occupied paper sides
    /// and chromatic felt gutters establish geometry before any print is read.
    #[test]
    fn ordinary_clipped_source_rejects_ambiguous_seams_missing_paper_and_missing_felt() {
        let before = fixture(57);
        let completed = fixture(58);
        let planned = action(&before);
        let source = planned.effect_bounds();


        for (name, bounds, rgb) in [
            ("duplicate seam", PixelRect::new(744, 806, 96, 1), [212, 212, 212]),
            ("missing seam", PixelRect::new(744, 803, 96, 1), [253, 253, 253]),
            ("unsupported seam shade", PixelRect::new(744, 803, 96, 1), [209, 209, 209]),
            ("gold seam", PixelRect::new(744, 803, 96, 1), [230, 185, 70]),
            ("missing bright row above", PixelRect::new(744, 802, 96, 1), [230, 230, 230]),
            ("missing bright row below", PixelRect::new(744, 804, 96, 1), [230, 230, 230]),
            ("missing left paper", PixelRect::new(731, 804, 1, 143), [12, 82, 45]),
            ("missing right paper", PixelRect::new(853, 804, 1, 143), [12, 82, 45]),
            ("missing left felt", PixelRect::new(717, 804, 1, 143), [253, 253, 253]),
            ("missing right felt", PixelRect::new(867, 804, 1, 143), [253, 253, 253]),
        ] {
            let mut different = completed.clone();
            paint(&mut different, bounds, rgb);
            assert_eq!(action(&different).target, action(&completed).target, "{name}: fresh source is retained");
            assert_eq!(ordinary_source_face_top(&different, source, 808), None, "{name}");
            let print = ordinary_source_print_changes(&before, &different, planned).unwrap();
            assert!(!print.geometry_supported && !print.verified, "{name}: {print:?}");
            let report = inspect_effect(&before, &different, planned).unwrap();
            assert!(!report.source_replaced && !report.verified, "{name}: {report}");
        }


        for (name, top) in [("zero shift", 808), ("excessive shift", 781)] {
            let mut different = completed.clone();
            paint(&mut different, PixelRect::new(744, 803, 96, 1), [253, 253, 253]);
            paint(&mut different, PixelRect::new(744, top - 1, 96, 1), [253, 253, 253]);
            paint(&mut different, PixelRect::new(744, top, 96, 1), [212, 212, 212]);
            paint(&mut different, PixelRect::new(744, top + 1, 96, 1), [253, 253, 253]);
            assert_eq!(ordinary_source_face_top(&different, source, 808), None, "{name}");
            assert!(!ordinary_source_print_changes(&before, &different, planned).unwrap().geometry_supported,
                "{name}");
        }
    }


    /// Translate the original print into the ordinary result while preserving
    /// its independently measured seam, paper sides, receiving print and target.
    fn translated_unchanged_ordinary_clipped_source() -> (CapturedFrame, CapturedFrame, GuidedAction) {
        let before = fixture(57);
        let completed = fixture(58);
        let mut after = completed.clone();
        copy_region(&before, &mut after, PixelRect::new(731, 813, 122, 129), PixelPoint::new(731, 808));
        // Seven original antialiased side pixels are chromatic, but differ by
        // less than the material bound; retain the actual ordinary paper side.
        copy_region(&completed, &mut after, PixelRect::new(731, 804, 1, 143), PixelPoint::new(731, 804));
        let planned = action(&before);
        assert_eq!(action(&after).target, action(&completed).target);
        assert_eq!(ordinary_source_face_top(&after, planned.effect_bounds(), 808), Some(803));
        (before, after, planned)
    }


    /// A pure translation and masked cursor/gold pixels cannot establish new
    /// printed identity, even with genuine ordinary geometry and receiving print.
    #[test]
    fn ordinary_clipped_source_rejects_transplanted_unchanged_print_cursor_and_gold() {
        let (before, translated, planned) = translated_unchanged_ordinary_clipped_source();
        let print = ordinary_source_print_changes(&before, &translated, planned).unwrap();
        assert!(print.geometry_supported && print.ordinary_face_geometry, "{print:?}");
        assert_eq!((print.material_changed, print.new_ink, print.cleared_ink), (0, 0, 0));
        assert!(!print.verified, "translated original print cannot prove replacement: {print:?}");
        let report = inspect_effect(&before, &translated, planned).unwrap();
        assert!(report.destination_print.verified && report.destination_changed >= MINIMUM_CONTENT_CHANGE,
            "recipient and fresh target are retained: {report}");
        assert!(!report.source_replaced && !report.verified, "{report}");


        for (name, bounds, rgb) in [
            ("black cursor", PixelRect::new(780, 824, 24, 32), [0, 0, 0]),
            ("white cursor", PixelRect::new(780, 824, 24, 32), [255, 255, 255]),
            ("gold", PixelRect::new(734, 820, 4, 32), [230, 185, 70]),
        ] {
            let mut different = translated.clone();
            paint(&mut different, bounds, rgb);
            let print = ordinary_source_print_changes(&before, &different, planned).unwrap();
            assert!(print.geometry_supported && print.ordinary_face_geometry, "{name}: {print:?}");
            assert_eq!((print.material_changed, print.new_ink, print.cleared_ink), (0, 0, 0), "{name}: {print:?}");
            let report = inspect_effect(&before, &different, planned).unwrap();
            assert!(!report.source_replaced && !report.verified, "{name}: {report}");
        }
    }


    /// Opposed source print remains mandatory after independent alignment.
    /// Neither one-way darkening nor whitening proves an exposed different card.
    #[test]
    fn ordinary_clipped_source_rejects_each_one_way_print_transition() {
        let before = fixture(57);
        let completed = fixture(58);
        let planned = action(&before);


        for remove_new in [true, false] {
            let mut different = completed.clone();


            for y in 813..942 {


                for x in 732..853 {
                    let first = pixel_rgb(&before, x, y).unwrap();
                    let second = pixel_rgb(&completed, x, y - 5).unwrap();


                    let remove = if remove_new {
                        is_bright_neutral_paper(first) && is_card_ink(second)
                    } else {
                        is_card_ink(first) && is_bright_neutral_paper(second)
                    };


                    if remove {
                        let offset = (y - 5) as usize * different.stride + x as usize * 4;
                        different.pixels[offset..offset + 3].copy_from_slice(&first);
                    }
                }
            }
            let print = ordinary_source_print_changes(&before, &different, planned).unwrap();
            assert!(print.geometry_supported && print.ordinary_face_geometry, "{print:?}");
            assert_eq!(if remove_new { print.new_ink } else { print.cleared_ink }, 0, "{print:?}");
            assert!(!print.verified, "both source print directions are mandatory: {print:?}");
            let report = inspect_effect(&before, &different, planned).unwrap();
            assert!(!report.source_replaced && !report.verified, "{report}");
        }
    }


    /// Retained neutral paper that fades is not replacement. The seam, two
    /// ordinary paper sides, felt gutters and fresh recipient remain intact.
    #[test]
    fn ordinary_clipped_source_rejects_stable_paper_field_fading() {
        let before = fixture(57);
        let completed = fixture(58);
        let planned = action(&before);
        let mut faded = completed.clone();


        for y in 814..926 {


            for x in 742..842 {
                let rgb = pixel_rgb(&completed, x, y).unwrap();


                if is_bright_neutral_paper(rgb) {
                    let offset = y as usize * faded.stride + x as usize * 4;
                    faded.pixels[offset..offset + 3].copy_from_slice(&rgb.map(|value| value.saturating_sub(12)));
                }
            }
        }
        let print = ordinary_source_print_changes(&before, &faded, planned).unwrap();
        assert!(print.geometry_supported && print.ordinary_face_geometry && print.paper_supported, "{print:?}");
        assert!(print.changed_paper_fields > 0 && !print.verified, "paper fading is not printed replacement: {print:?}");
        let report = inspect_effect(&before, &faded, planned).unwrap();
        assert!(!report.source_replaced && !report.verified, "{report}");
    }


    /// Source-only print, recipient-only print and a fresh HALO without verified
    /// receiving print each fail a separate ordinary continuation requirement.
    #[test]
    fn ordinary_clipped_source_requires_fresh_target_non_source_change_and_receiving_print() {
        let before = fixture(57);
        let completed = fixture(58);
        let planned = action(&before);
        let source = planned.effect_bounds();
        let recipient = PixelRect::new(894, UPPER_Y, CARD_WIDTH, CARD_HEIGHT);
        let mut source_only = before.clone();
        copy_region(&completed, &mut source_only, PixelRect::new(716, 332, 154, 662), PixelPoint::new(716, 332));
        assert_eq!(ordinary_source_face_top(&source_only, source, 808), Some(803));
        let print = measure_aligned_source_print_changes(&before, &source_only, source, source.height, 5, planned).unwrap();
        assert!(print.verified, "source print is independently real: {print:?}");
        assert_eq!(analyse(&source_only).unwrap().prediction, PredictedAction::NoHighlight);
        let report = inspect_effect(&before, &source_only, planned).unwrap();
        assert_eq!(report.destination_changed, 0, "source column does not establish a recipient: {report}");
        assert!(!report.source_aligned_print.geometry_supported && !report.source_replaced && !report.verified,
            "no fresh action: {report}");
        let mut recipient_only = before.clone();
        copy_region(&completed, &mut recipient_only, recipient, PixelPoint::new(894, UPPER_Y as i32));
        let report = inspect_effect(&before, &recipient_only, planned).unwrap();
        assert!(report.destination_print.verified, "real receiving print is retained: {report}");
        assert_eq!(report.source_changed, 0, "{report}");
        assert!(!report.source_aligned_print.geometry_supported && !report.source_replaced && !report.verified,
            "receiving print alone: {report}");
        let mut no_receiving_print = completed.clone();
        copy_region(&before, &mut no_receiving_print, recipient, PixelPoint::new(894, UPPER_Y as i32));
        let report = inspect_effect(&before, &no_receiving_print, planned).unwrap();
        assert!(report.source_aligned_print.verified && report.destination_changed >= MINIMUM_CONTENT_CHANGE,
            "real source and fresh HALO remain: {report}");
        assert!(!report.destination_print.verified && !report.source_replaced && !report.verified,
            "non-source HALO change is insufficient without receiving print: {report}");
    }


    /// K59/K60 retains native manual-capture pixels: the former Q-spades source
    /// exposes dark green felt beneath the next guide. Q-clubs is a different
    /// fresh source, rather than a replacement queen at the previous position.
    #[test]
    fn recorded_deep_guide_felt_removal_supports_continuation_only() {
        let before = fixture(59);
        let after = fixture(60);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 7, top: 391, bottom: 581,
        }));
        assert_eq!(action(&after).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 3, top: 372, bottom: 562,
        }));
        let report = inspect_effect(&before, &after, planned).unwrap();
        assert_eq!((report.source_changed, report.source_positive, report.source_dimmed_felt), (6960, 0, 320));
        assert_eq!(report.source_deep_felt.paper_to_deep_felt, 990);
        assert_eq!(report.source_deep_felt.halves, [467, 523]);
        assert!(report.source_deep_felt.context_supported && report.source_deep_felt.verified, "{report}");
        assert_eq!(report.destination_changed, 10565);
        assert!(report.source_replaced && !report.verified, "continuation is not complete effect: {report}");
        assert!(!completion_evidence(&after).unwrap().complete_candidate);
    }


    /// A real fresh target and receiving foundation remain in these controls.
    /// Darkening the retained native Q-spades through an achromatic guide,
    /// including a 0.11 multiplier, cannot create exposed green felt.
    #[test]
    fn deep_guide_felt_rejects_unchanged_card_and_achromatic_guide_fading() {
        let before = fixture(59);
        let completed = fixture(60);
        let planned = action(&before);
        assert!(!inspect_effect(&before, &before, planned).unwrap().source_replaced);


        for percent in [2_u16, 5, 10, 11, 15, 20, 25, 30, 100] {
            let mut retained = completed.clone();


            for y in 391..581 {


                for x in 1398..1530 {
                    let rgb = pixel_rgb(&before, x, y).unwrap();
                    let offset = y as usize * retained.stride + x as usize * 4;
                    retained.pixels[offset..offset + 3].copy_from_slice(&rgb.map(|value| {
                        ((u16::from(value) * percent + 50) / 100) as u8
                    }));
                }
            }
            let report = inspect_effect(&before, &retained, planned).unwrap();
            assert_eq!(report.source_deep_felt.paper_to_deep_felt, 0, "{percent}%: {report}");
            assert!(!report.source_replaced && !report.verified, "retained card and guide are insufficient: {percent}%: {report}");
        }
    }


    /// Positive source and non-source observations remain independent. A real
    /// receiving foundation alone cannot replace a source; source-only change
    /// with a fresh outline already present in both images lacks recipient input.
    #[test]
    fn deep_guide_felt_keeps_source_and_non_source_changes_independent() {
        let before = fixture(59);
        let completed = fixture(60);
        let planned = action(&before);
        let mut recipient_only = completed.clone();
        copy_region(&before, &mut recipient_only, PixelRect::new(1414, 407, 100, 158), PixelPoint::new(1414, 407));
        let report = inspect_effect(&before, &recipient_only, planned).unwrap();
        assert_eq!(report.source_deep_felt.paper_to_deep_felt, 0);
        assert!(!report.source_replaced && !report.verified, "recipient alone: {report}");

        let mut controlled_before = before.clone();
        copy_region(&completed, &mut controlled_before, PixelRect::new(714, 360, 156, 214), PixelPoint::new(714, 360));
        assert_eq!(action(&controlled_before), planned);
        let mut source_only = controlled_before.clone();
        copy_region(&completed, &mut source_only, PixelRect::new(1390, 332, 148, 249), PixelPoint::new(1390, 332));
        assert_eq!(action(&source_only).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 3, top: 372, bottom: 562,
        }));
        let report = inspect_effect(&controlled_before, &source_only, planned).unwrap();
        assert!(report.source_deep_felt.verified, "source colour is independently real: {report}");
        assert_eq!(report.destination_changed, 0);
        assert!(!report.source_replaced && !report.verified, "source alone: {report}");
    }


    /// Gold, neutral dimming, pointer pixels and toolbar-only colour cannot
    /// supply positive lower-half green evidence. The receiving result survives.
    #[test]
    fn deep_guide_felt_rejects_excluded_and_nonchromatic_source_changes() {
        let before = fixture(59);
        let completed = fixture(60);
        let planned = action(&before);


        for rgb in [[32, 32, 32], [190, 190, 190], [230, 185, 70], [14, 14, 14], [5, 9, 6]] {
            let mut different = completed.clone();
            paint(&mut different, PixelRect::new(1414, 407, 100, 158), rgb);
            let report = inspect_effect(&before, &different, planned).unwrap();
            assert_eq!(report.source_deep_felt.paper_to_deep_felt, 0, "{rgb:?}: {report}");
            assert!(!report.source_replaced && !report.verified, "{rgb:?}: {report}");
        }
        let mut excluded = completed.clone();
        copy_region(&before, &mut excluded, PixelRect::new(1414, 407, 100, 158), PixelPoint::new(1414, 407));
        paint(&mut excluded, PixelRect::new(1417, 432, 94, 46), [2, 12, 7]);
        paint(&mut excluded, PixelRect::new(1398, TOOLBAR_TOP, CARD_WIDTH, 86), [2, 12, 7]);
        let report = inspect_effect(&before, &excluded, planned).unwrap();
        assert_eq!(report.source_deep_felt.paper_to_deep_felt, 0, "{report}");
        assert!(!report.source_replaced && !report.verified, "cursor and toolbar are excluded: {report}");
    }


    /// Each opposed half needs positive colour evidence, while the original
    /// 512 material-source bound is independent of a qualifying colour count.
    #[test]
    fn deep_guide_felt_keeps_material_and_opposed_half_guards() {
        let before = fixture(59);
        let completed = fixture(60);
        let planned = action(&before);
        let evidence = deep_felt_source_changes(&before, &completed, planned, 511).unwrap();
        assert_eq!(evidence.paper_to_deep_felt, 990);
        assert!(!evidence.verified, "material threshold is independent: {evidence:?}");

        let mut one_half = completed.clone();
        copy_region(&before, &mut one_half, PixelRect::new(1414, 486, 50, 79), PixelPoint::new(1414, 486));
        let report = inspect_effect(&before, &one_half, planned).unwrap();
        assert_eq!(report.source_deep_felt.halves, [0, 523]);
        assert!(!report.source_deep_felt.verified && !report.source_replaced && !report.verified, "one half exceeds512 but remains insufficient: {report}");
    }


    /// This route cannot expand source geometry, accept another before action,
    /// repair forged input metadata or read a malformed frame layout.
    #[test]
    fn deep_guide_felt_rejects_noncanonical_geometry_and_malformed_frames() {
        let before = fixture(59);
        let completed = fixture(60);
        let planned = action(&before);


        for target in [
            KlondikeTarget::Tableau { column: 7, top: 390, bottom: 581 },
            KlondikeTarget::Tableau { column: 7, top: 391, bottom: 580 },
            KlondikeTarget::Tableau { column: 7, top: 803, bottom: 993 },
            KlondikeTarget::Tableau { column: 3, top: 372, bottom: 562 },
            KlondikeTarget::Waste { offset: 0 },
            KlondikeTarget::Foundation { column: 2 },
        ] {
            let other = canonical_action(target).unwrap();
            let evidence = deep_felt_source_changes(&before, &completed, other, 6960).unwrap();
            assert!(!evidence.context_supported && !evidence.verified, "{target:?}: {evidence:?}");
        }
        let mut forged = planned;
        forged.anchor.x += 1;
        assert!(!deep_felt_source_changes(&before, &completed, forged, 6960).unwrap().context_supported);
        let mut malformed = completed;
        malformed.pixels.truncate(1);
        assert_eq!(deep_felt_source_changes(&before, &malformed, planned, 6960), Err(HaloDetectionError::InvalidFrameLayout));
    }


    /// Genuine long and clipped source captures do not borrow this full-card
    /// route. A removed result HALO leaves real source/recipient changes intact
    /// but cannot authorise continuation from an unrecognised next target.
    #[test]
    fn deep_guide_felt_requires_full_single_card_and_a_fresh_target() {
        let completed = fixture(60);


        for number in [10, 55] {
            let before = fixture(number);
            let planned = action(&before);
            let evidence = deep_felt_source_changes(&before, &completed, planned, 6960).unwrap();
            assert!(!evidence.context_supported && !evidence.verified, "K{number}: {evidence:?}");
        }
        let before = fixture(59);
        let planned = action(&before);
        let mut no_target = completed;
        copy_region(&before, &mut no_target, PixelRect::new(714, 360, 156, 214), PixelPoint::new(714, 360));
        assert_eq!(analyse(&no_target).unwrap().prediction, PredictedAction::NoHighlight);
        let report = inspect_effect(&before, &no_target, planned).unwrap();
        assert!(report.source_changed >= MINIMUM_CONTENT_CHANGE && report.destination_changed >= MINIMUM_CONTENT_CHANGE, "material changes survive: {report}");
        assert!(!report.source_deep_felt.context_supported && !report.source_replaced && !report.verified, "no fresh target: {report}");
    }


    /// K63/K64 records one K-diamonds source moving to column 1. The exposed
    /// J-diamonds has a dark guide, while the next long source is column 4.
    /// Two disjoint positive felt ranges share the original 512-pixel threshold.
    #[test]
    fn recorded_single_king_removal_combines_disjoint_positive_felt_ranges() {
        let before = fixture(63);
        let after = fixture(64);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 3, top: 354, bottom: 544,
        }));
        assert_eq!(action(&after).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 4, top: 372, bottom: 835,
        }));
        let report = inspect_effect(&before, &after, planned).unwrap();
        assert_eq!((report.source_changed, report.source_positive, report.source_dimmed_felt), (5012, 32, 417));
        let felt = report.source_deep_felt;
        assert_eq!((felt.paper_to_deep_felt, felt.ordinary_dimmed_felt, felt.positive_felt), (275, 407, 682));
        assert_eq!((felt.halves, felt.ordinary_halves, felt.positive_halves), ([162, 113], [224, 183], [386, 296]));
        assert!(felt.context_supported && felt.verified, "{report}");
        assert_eq!(report.destination_changed, 9666);
        assert!(report.source_replaced && !report.verified, "continuation remains separate from complete effect: {report}");
        assert!(!completion_evidence(&after).unwrap().complete_candidate);
    }


    /// Retained native artwork under an achromatic black guide cannot gain the
    /// required independent chroma in either disjoint range. A real recipient
    /// and fresh source survive these controls, including a 0.11 multiplier.
    #[test]
    fn disjoint_felt_union_rejects_native_retained_king_guide_fading() {
        let before = fixture(63);
        let completed = fixture(64);
        let planned = action(&before);


        for percent in [2_u16, 5, 10, 11, 15, 20, 25, 30, 100] {
            let mut retained = completed.clone();


            for y in 354..544 {


                for x in 726..858 {
                    let rgb = pixel_rgb(&before, x, y).unwrap();
                    let offset = y as usize * retained.stride + x as usize * 4;
                    retained.pixels[offset..offset + 3].copy_from_slice(&rgb.map(|value| {
                        ((u16::from(value) * percent + 50) / 100) as u8
                    }));
                }
            }
            let report = inspect_effect(&before, &retained, planned).unwrap();
            assert_eq!((report.source_deep_felt.paper_to_deep_felt, report.source_deep_felt.ordinary_dimmed_felt,
                report.source_deep_felt.positive_felt), (0, 0, 0), "{percent}%: {report}");
            assert!(!report.source_replaced && !report.verified, "retained card plus guide: {percent}%: {report}");
        }
    }


    /// Neither the 407 existing-range transitions nor the 275 deep-range
    /// transitions independently pass 512. Restore the other real transitions
    /// while retaining recipient artwork and the next recognised run.
    #[test]
    fn disjoint_felt_union_keeps_each_component_and_material_bounds() {
        let before = fixture(63);
        let completed = fixture(64);
        let planned = action(&before);


        for remove_deep in [true, false] {
            let mut different = completed.clone();


            for y in 449..528 {


                for x in 742..842 {
                    let first = pixel_rgb(&before, x, y).unwrap();
                    let second @ [red, green, blue] = pixel_rgb(&completed, x, y).unwrap();
                    let deep = (10..=14).contains(&green)
                        && i16::from(green) - i16::from(red) >= 5
                        && i16::from(green) - i16::from(blue) >= 3;


                    if is_bright_neutral_paper(first) && if remove_deep { deep } else { is_dimmed_felt(second) } {
                        let offset = y as usize * different.stride + x as usize * 4;
                        different.pixels[offset..offset + 3].copy_from_slice(&first);
                    }
                }
            }
            let report = inspect_effect(&before, &different, planned).unwrap();
            let felt = report.source_deep_felt;
            assert_eq!((felt.paper_to_deep_felt, felt.ordinary_dimmed_felt, felt.positive_felt),


                if remove_deep { (0, 407, 407) } else { (275, 0, 275) });
            assert!(!felt.verified && !report.source_replaced && !report.verified, "one insufficient component: {report}");
        }
        let felt = deep_felt_source_changes(&before, &completed, planned, 511).unwrap();
        assert_eq!(felt.positive_felt, 682);
        assert!(!felt.verified, "material source bound remains independent: {felt:?}");
    }


    /// Source-only and recipient-only changes stay independent. The source-only
    /// control retains an identical shorter native outline in another column in
    /// both images, so fresh-target context survives without non-source change.
    #[test]
    fn disjoint_felt_union_keeps_source_and_non_source_independent() {
        let before = fixture(63);
        let completed = fixture(64);
        let planned = action(&before);
        let mut recipient_only = completed.clone();
        copy_region(&before, &mut recipient_only, PixelRect::new(742, 370, 100, 158), PixelPoint::new(742, 370));
        let report = inspect_effect(&before, &recipient_only, planned).unwrap();
        assert_eq!(report.source_deep_felt.positive_felt, 0);
        assert!(!report.source_replaced && !report.verified, "recipient alone: {report}");

        let native_shorter_source = fixture(59);
        let mut controlled_before = before.clone();
        copy_region(&native_shorter_source, &mut controlled_before, PixelRect::new(1386, 385, 156, 204), PixelPoint::new(1050, 345));
        assert_eq!(action(&controlled_before), planned);
        let mut source_only = controlled_before.clone();
        copy_region(&completed, &mut source_only, PixelRect::new(714, 342, 156, 235), PixelPoint::new(714, 342));
        assert_eq!(action(&source_only).target, ActionTarget::Klondike(KlondikeTarget::Tableau {
            column: 5, top: 351, bottom: 541,
        }));
        let report = inspect_effect(&controlled_before, &source_only, planned).unwrap();
        assert!(report.source_deep_felt.verified, "independent source colour survives: {report}");
        assert_eq!(report.destination_changed, 0);
        assert!(!report.source_replaced && !report.verified, "source alone: {report}");
    }


    /// A single positive half still refuses even with more than 512 pixels.
    /// Cursor/gold changes and absent next targets cannot borrow this route.
    #[test]
    fn disjoint_felt_union_keeps_opposed_halves_exclusions_and_fresh_target() {
        let before = fixture(63);
        let completed = fixture(64);
        let planned = action(&before);
        let mut one_half = completed.clone();
        copy_region(&before, &mut one_half, PixelRect::new(742, 449, 50, 79), PixelPoint::new(742, 449));
        paint(&mut one_half, PixelRect::new(792, 449, 50, 79), [2, 12, 7]);
        let report = inspect_effect(&before, &one_half, planned).unwrap();
        assert!(report.source_deep_felt.positive_felt >= MINIMUM_CONTENT_CHANGE, "positive total survives: {report}");
        assert_eq!(report.source_deep_felt.positive_halves[0], 0);
        assert!(!report.source_deep_felt.verified && !report.source_replaced && !report.verified, "one half: {report}");

        let mut excluded = completed.clone();
        copy_region(&before, &mut excluded, PixelRect::new(742, 370, 100, 158), PixelPoint::new(742, 370));
        paint(&mut excluded, PixelRect::new(745, 395, 94, 46), [2, 12, 7]);
        let report = inspect_effect(&before, &excluded, planned).unwrap();
        assert_eq!(report.source_deep_felt.positive_felt, 0);
        assert!(!report.source_replaced && !report.verified, "cursor-only pixels: {report}");

        let mut gold = completed.clone();
        paint(&mut gold, PixelRect::new(742, 449, 100, 79), [230, 185, 70]);
        assert_eq!(inspect_effect(&before, &gold, planned).unwrap().source_deep_felt.positive_felt, 0);

        let mut no_target = completed;
        copy_region(&before, &mut no_target, PixelRect::new(882, 360, 156, 483), PixelPoint::new(882, 360));
        assert_eq!(analyse(&no_target).unwrap().prediction, PredictedAction::NoHighlight);
        let report = inspect_effect(&before, &no_target, planned).unwrap();
        assert!(!report.source_deep_felt.context_supported && !report.source_replaced && !report.verified, "no fresh target: {report}");
    }


    /// The ordinary descriptor never supplies an input target, repairs a forged
    /// before action or accepts an invalid frame layout.
    #[test]
    fn ordinary_clipped_source_rejects_noncanonical_actions_and_malformed_frames() {
        let before = fixture(57);
        let completed = fixture(58);
        let planned = action(&before);


        for target in [
            KlondikeTarget::Tableau { column: 3, top: 802, bottom: 992 },
            KlondikeTarget::Tableau { column: 3, top: 803, bottom: 993 },
            KlondikeTarget::Tableau { column: 7, top: 425, bottom: 615 },
            KlondikeTarget::Waste { offset: 2 },
            KlondikeTarget::Foundation { column: 1 },
            KlondikeTarget::Draw,
            KlondikeTarget::Solve,
        ] {
            let other = canonical_action(target).unwrap();
            let print = ordinary_source_print_changes(&before, &completed, other).unwrap();
            assert!(!print.geometry_supported && !print.verified, "{target:?}: {print:?}");
        }
        let mut forged = planned;
        forged.anchor.x += 1;
        assert!(!ordinary_source_print_changes(&before, &completed, forged).unwrap().geometry_supported);
        let mut malformed = completed;
        malformed.pixels.truncate(1);
        assert_eq!(ordinary_source_print_changes(&before, &malformed, planned), Err(HaloDetectionError::InvalidFrameLayout));
    }


    /// The recorded Draw succeeds independently of the subsequent long run.
    /// K61 is an Undo reconstruction, while K62 is the later read-only result.
    #[test]
    fn recorded_draw_result_with_nested_run_keeps_lower_scene_probe() {
        let before = fixture(61);
        let after = fixture(62);
        assert!(is_gameplay_scene(&before).unwrap());
        assert_eq!(action(&before).target, ActionTarget::Klondike(KlondikeTarget::Draw));
        assert_eq!(count_pixels(&after, PixelRect::new(868, 710, 14, 14), is_felt), 168);
        assert_eq!(count_pixels(&after, PixelRect::new(874, 710, 7, 28), is_felt), 196);
        assert!(is_gameplay_scene(&after).unwrap());
        assert_eq!(find_tableau_source(&after, PixelRect::new(726, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap(), Some(PixelRect::new(726, 372, CARD_WIDTH, 518)));
        assert_eq!(action(&after).target, ActionTarget::Klondike(KlondikeTarget::Tableau { column: 3, top: 372, bottom: 890 }));
        let report = inspect_effect(&before, &after, action(&before)).unwrap();
        assert!(report.verified && report.destination_changed >= MINIMUM_CONTENT_CHANGE, "Draw effect remains independent: {report}");
        assert!(!inspect_effect(&before, &before, action(&before)).unwrap().verified);
    }


    /// The lower probe retains every established gameplay classification,
    /// including K47's opposing column-4 outline and the inactive K01 board.
    #[test]
    fn lower_gap_probe_preserves_sixty_prior_scene_classifications() {
        let accepted = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            21, 22, 23, 24, 25, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42,
            44, 45, 46, 47, 48, 50, 51, 54, 55, 56, 57, 58, 59, 60,
        ];


        for number in 1..=60 {
            assert_eq!(is_gameplay_scene(&fixture(number)).unwrap(), accepted.contains(&number), "K{number:02}");
        }
        assert_eq!(analyse(&fixture(1)).unwrap().prediction, PredictedAction::NoHighlight);
        assert!(matches!(action(&fixture(47)).target, ActionTarget::Klondike(KlondikeTarget::Tableau { column: 4, .. })));
    }


    /// Retain the inclusive 95% requirement over exactly 196 sampled pixels.
    /// Ten unrelated dark pixels reject a scene which nine still leave valid.
    #[test]
    fn lower_gap_probe_keeps_exact_felt_fraction_boundary() {
        let mut frame = fixture(62);
        paint(&mut frame, PixelRect::new(874, 710, 7, 1), [0, 0, 0]);
        paint(&mut frame, PixelRect::new(874, 711, 2, 1), [0, 0, 0]);
        assert_eq!(count_pixels(&frame, PixelRect::new(874, 710, 7, 28), is_felt), 187);
        assert!(is_gameplay_scene(&frame).unwrap());
        paint(&mut frame, PixelRect::new(876, 711, 1, 1), [0, 0, 0]);
        assert_eq!(count_pixels(&frame, PixelRect::new(874, 710, 7, 28), is_felt), 186);
        assert!(!is_gameplay_scene(&frame).unwrap());
        assert_eq!(analyse(&frame).unwrap().prediction, PredictedAction::NoHighlight);
    }


    /// An intact source cannot repair any occluded independent gutter or the
    /// missing seven-column/four-foundation scene context of an unknown modal.
    #[test]
    fn lower_gap_probe_preserves_other_independent_scene_guards() {
        let original = fixture(62);


        for bounds in [
            PixelRect::new(160, 310, 16, 16),
            PixelRect::new(1_730, 310, 16, 16),
            PixelRect::new(874, 710, 7, 28),
            PixelRect::new(1_202, 840, 14, 14),
        ] {
            let mut occluded = original.clone();
            paint(&mut occluded, bounds, [0, 0, 0]);
            assert!(!is_gameplay_scene(&occluded).unwrap(), "{bounds:?}");
            assert_eq!(analyse(&occluded).unwrap().prediction, PredictedAction::NoHighlight);
        }


        for foundation in [false, true] {
            let mut occluded = original.clone();


            let (first, y, columns) = if foundation { (894, UPPER_Y, 4) } else { (FIRST_COLUMN_X, TABLEAU_Y, 7) };


            for column in 0..columns {
                paint(&mut occluded, PixelRect::new(first + COLUMN_PITCH * column + 20, y, 92, 6), [0, 0, 0]);
            }
            assert!(!is_gameplay_scene(&occluded).unwrap());
            assert_eq!(analyse(&occluded).unwrap().prediction, PredictedAction::NoHighlight);
        }
    }


    /// The relocated scene probe grants no source authority to a dark dashed
    /// destination or to an incomplete run with its whole right rail missing.
    #[test]
    fn lower_gap_probe_keeps_dashed_destination_and_incomplete_run_rejected() {
        let mut frame = fixture(62);
        assert_eq!(find_tableau_source(&frame, PixelRect::new(1_398, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap(), None);
        paint(&mut frame, PixelRect::new(858, 372, 10, 518), [0, 0, 0]);
        assert!(is_gameplay_scene(&frame).unwrap());
        assert_eq!(analyse(&frame).unwrap().prediction, PredictedAction::NoHighlight);
    }



    /// K65 is the Undo reconstruction of the fixed 8-diamonds source; K66 is
    /// the later failure frame after its return beneath the ordinary 9-spades.
    /// This source proof grants continuation without claiming a complete effect.
    #[test]
    fn recorded_suit_return_matches_old_print_at_independent_receiver_seam() {
        let before = fixture(65);
        let after = fixture(66);
        let planned = action(&before);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Foundation { column: 2 }));
        assert_eq!(action(&after).target, ActionTarget::Klondike(KlondikeTarget::Tableau { column: 5, top: 390, bottom: 582 }));
        let report = inspect_effect(&before, &after, planned).unwrap();
        let transfer = report.source_foundation_transfer;
        assert!(report.source_changed < MINIMUM_CONTENT_CHANGE && report.source_positive < MINIMUM_CONTENT_CHANGE, "{report}");
        assert!(report.source_identity_directions.into_iter().flatten().any(|count| count < MINIMUM_CORNER_CHANGE), "ordinary SUIT proof remains insufficient: {report}");
        assert_eq!(transfer.paired_changes, [13, 51], "{report}");
        assert_eq!(transfer.stable_paper_fields, [367, 411], "{report}");
        assert_eq!(transfer.changed_paper_fields, [0, 0], "{report}");
        assert_eq!((transfer.receiver_count, transfer.receiver_column, transfer.receiver_top), (1, 4, 615), "{report}");
        assert_eq!(transfer.receiver_material_changed, 1_146, "{report}");
        assert_eq!(transfer.matched_transfer_ink, 186, "{report}");
        assert!(transfer.verified && report.source_replaced && !report.verified, "continuation authority remains separate: {report}");
    }


    /// A persistent fresh next HALO cannot supply missing source or receiving
    /// artwork. Restore either side independently while retaining that HALO.
    #[test]
    fn suit_transfer_requires_both_replaced_source_and_new_receiving_print() {
        let before = fixture(65);
        let after = fixture(66);
        let planned = action(&before);
        let mut source_only = after.clone();
        copy_region(&before, &mut source_only, PixelRect::new(894, 332, CARD_WIDTH, TABLEAU_EFFECT_BOTTOM - 332), PixelPoint::new(894, 332));
        let mut recipient_only = after.clone();
        copy_region(&before, &mut recipient_only, planned.effect_bounds(), PixelPoint::new(1_062, UPPER_Y as i32));
        let mut halo_only = before.clone();
        // Preserve both complete exterior rails of the fresh source outline.
        copy_region(&after, &mut halo_only, PixelRect::new(1_052, 380, CARD_WIDTH + 20, 220), PixelPoint::new(1_052, 380));


        for result in [&before, &source_only, &recipient_only, &halo_only] {
            let report = inspect_effect(&before, result, planned).unwrap();
            assert!(!report.source_foundation_transfer.verified && !report.source_replaced && !report.verified, "{report}");
        }
        assert_eq!(action(&source_only), action(&after));
        assert_eq!(action(&recipient_only), action(&after));
        assert_eq!(action(&halo_only), action(&after));
    }


    /// An unchanged opposed corner or a one-way erased source fails even while
    /// the real receiving card and next recommendation remain intact.
    #[test]
    fn suit_transfer_rejects_one_corner_and_one_way_source_changes() {
        let before = fixture(65);
        let after = fixture(66);
        let planned = action(&before);
        let patches = [PixelRect::new(1_067, 117, 28, 44), PixelRect::new(1_161, 238, 28, 44)];


        for patch in patches {
            let mut result = after.clone();
            copy_region(&before, &mut result, patch, PixelPoint::new(patch.x as i32, patch.y as i32));
            let report = inspect_effect(&before, &result, planned).unwrap();
            assert!(!report.source_foundation_transfer.verified && !report.source_replaced && !report.verified, "{report}");
        }
        let mut whitened = after.clone();


        for patch in patches {
            paint(&mut whitened, patch, [255, 255, 255]);
        }
        let report = inspect_effect(&before, &whitened, planned).unwrap();
        assert!(report.source_foundation_transfer.paired_changes[0] == 0, "{report}");
        assert!(!report.source_foundation_transfer.verified && !report.source_replaced && !report.verified, "{report}");
    }


    /// Printed replacement plus a real recipient does not excuse fading of
    /// retained unprinted source paper. Neutral paper stays white in this probe.
    #[test]
    fn suit_transfer_rejects_changed_neutral_source_fields() {
        let before = fixture(65);
        let mut after = fixture(66);
        let planned = action(&before);


        for patch in [PixelRect::new(1_067, 117, 28, 44), PixelRect::new(1_161, 238, 28, 44)] {


            for y in patch.y..patch.bottom() {


                for x in patch.x..patch.right() {


                    if pixel_rgb(&before, x, y).is_some_and(is_white)
                        && pixel_rgb(&after, x, y).is_some_and(is_white)
                    {
                        paint(&mut after, PixelRect::new(x, y, 1, 1), [235, 235, 235]);
                    }
                }
            }
        }
        let report = inspect_effect(&before, &after, planned).unwrap();
        assert!(report.source_foundation_transfer.changed_paper_fields.into_iter().any(|count| count != 0), "{report}");
        assert!(!report.source_foundation_transfer.verified && !report.source_replaced && !report.verified, "{report}");
    }


    /// A common four-pixel glyph translation can pass paired changed-print
    /// totals. Retained old print at the same fitting offset rejects it.
    #[test]
    fn suit_transfer_rejects_translated_old_glyph_with_real_recipient() {
        let before = fixture(65);
        let mut after = fixture(66);
        let planned = action(&before);


        for patch in [PixelRect::new(1_067, 117, 28, 44), PixelRect::new(1_161, 238, 28, 44)] {
            paint(&mut after, patch, [255, 255, 255]);
            copy_region(&before, &mut after, patch, PixelPoint::new(patch.x as i32 + 4, patch.y as i32));
        }
        let report = inspect_effect(&before, &after, planned).unwrap();
        assert!(report.source_foundation_transfer.paired_changes.into_iter().sum::<usize>() >= MINIMUM_CORNER_CHANGE, "paired print alone is insufficient: {report}");
        assert!(report.source_foundation_transfer.translated_ink_retained, "{report}");
        assert!(!report.source_foundation_transfer.verified && !report.source_replaced && !report.verified, "{report}");
    }


    /// Receiving artwork is insufficient without its independent new gray seam.
    /// Existing matching ink cannot be counted as newly received transfer ink.
    #[test]
    fn suit_transfer_rejects_missing_geometry_and_already_present_print() {
        let before = fixture(65);
        let after = fixture(66);
        let planned = action(&before);
        let mut missing_seam = after.clone();
        paint(&mut missing_seam, PixelRect::new(912, 615, 96, 1), [255, 255, 255]);
        let report = inspect_effect(&before, &missing_seam, planned).unwrap();
        assert_eq!(report.source_foundation_transfer.receiver_count, 0, "{report}");
        assert!(!report.source_replaced && !report.verified, "{report}");
        let mut existing_print = before.clone();


        for dy in 0..44 {


            for dx in 0..28 {
                let source_rgb = pixel_rgb(&before, 1_067 + dx, 117 + dy).unwrap();


                if is_card_ink(source_rgb) && !is_rail_gold(source_rgb) {
                    let received_rgb = pixel_rgb(&after, 899 + dx, 620 + dy).unwrap();
                    paint(&mut existing_print, PixelRect::new(899 + dx, 620 + dy, 1, 1), received_rgb);
                }
            }
        }
        let report = inspect_effect(&existing_print, &after, planned).unwrap();
        assert_eq!(report.source_foundation_transfer.matched_transfer_ink, 0, "old matching recipient artwork is not a transfer: {report}");
        assert!(!report.source_replaced && !report.verified, "{report}");
    }


    /// Receiver geometry is selected before source matching. Two qualified new
    /// seams refuse even if only one might match the departing source artwork.
    #[test]
    fn suit_transfer_rejects_duplicate_independent_receiver_geometry() {
        let mut before = fixture(65);
        let mut after = fixture(66);
        let planned = action(&before);
        paint(&mut before, PixelRect::new(726, 614, CARD_WIDTH, 50), [15, 100, 60]);
        copy_region(&fixture(66), &mut after, PixelRect::new(894, 614, CARD_WIDTH, 50), PixelPoint::new(726, 614));
        let report = inspect_effect(&before, &after, planned).unwrap();
        assert_eq!(report.source_foundation_transfer.receiver_count, 2, "{report}");
        assert!(!report.source_replaced && !report.verified, "{report}");
    }


    /// The new proof cannot bypass frame layout, scene, initial-action or fresh
    /// target validation. The existing error/stop contract remains unchanged.
    #[test]
    fn suit_transfer_preserves_scene_frame_and_canonical_action_guards() {
        let before = fixture(65);
        let after = fixture(66);
        let planned = action(&before);
        let mut unsupported = after.clone();
        paint(&mut unsupported, PixelRect::new(874, 710, 7, 28), [0, 0, 0]);
        let report = inspect_effect(&before, &unsupported, planned).unwrap();
        assert!(!report.source_replaced && !report.verified, "{report}");
        let mut malformed = after.clone();
        malformed.pixels.pop();
        assert!(inspect_effect(&before, &malformed, planned).is_err());
        let mut forged = planned;
        forged.anchor.x += 1;
        let report = inspect_effect(&before, &after, forged).unwrap();
        assert!(!report.source_replaced && !report.verified, "{report}");
        let mut missing_fresh_target = after.clone();
        paint(&mut missing_fresh_target, PixelRect::new(1_062, 380, CARD_WIDTH, 210), [255, 255, 255]);
        let report = inspect_effect(&before, &missing_fresh_target, planned).unwrap();
        assert!(!report.source_foundation_transfer.verified && !report.source_replaced && !report.verified, "{report}");
    }



    /// K68 and K69 retain complete row997 lower borders and terminated exterior
    /// rails. Column 7 additionally crosses the evidenced shaded toolbar band.
    /// Each whole run remains one canonical upper-card click with no drag.
    #[test]
    fn recorded_new_long_sources_keep_complete_border_and_upper_click() {


        for (number, column, top) in [(68, 7, 425), (69, 3, 636)] {
            let frame = fixture(number);
            let x = FIRST_COLUMN_X + COLUMN_PITCH * u32::from(column - 1);
            assert!(is_gameplay_scene(&frame).unwrap());
            assert_eq!(find_tableau_source(&frame, PixelRect::new(x, 332, CARD_WIDTH, 994 - 332)).unwrap(), None,
                "the former exclusive row994 boundary still clips genuine positive rails");
            assert_eq!(find_tableau_source(&frame, PixelRect::new(x, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap(),
                Some(PixelRect::new(x, u32::from(top), CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - u32::from(top))));
            let planned = action(&frame);
            assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau { column, top, bottom: 998 }));
            assert_eq!(planned.operation(), InputOperation::Click(PixelPoint::new((x + CARD_WIDTH / 2) as i32, i32::from(top) + 40)));
            assert_eq!(planned.effect_bounds(), PixelRect::new(x, u32::from(top), CARD_WIDTH, TOOLBAR_TOP - u32::from(top)));
            let mut toolbar_changed = frame.clone();
            paint(&mut toolbar_changed, PixelRect::new(x - 10, TOOLBAR_TOP, CARD_WIDTH + 20, TABLEAU_OUTLINE_BOTTOM - TOOLBAR_TOP), [255, 255, 255]);
            let report = inspect_effect(&frame, &toolbar_changed, planned).unwrap();
            assert_eq!((report.source_changed, report.source_positive, report.destination_changed), (0, 0, 0), "{report}");
            assert!(!report.source_replaced && !report.verified, "the added recognition rows never supply effect authority: {report}");
        }
        assert!(canonical_action(KlondikeTarget::Tableau { column: 7, top: 331, bottom: 998 }).is_none(),
            "the existing native scan-top bound remains enforced");
        assert!(canonical_action(KlondikeTarget::Tableau { column: 7, top: 425, bottom: 999 }).is_none(),
            "one row beyond the complete observed border remains unsupported");
    }


    /// Dark rail support needs ordinary paired rails in two rows on both sides
    /// of the measured band. Removing any one side/row restores rejection.
    #[test]
    fn column_seven_toolbar_shadow_requires_both_ordinary_brackets() {
        let frame = fixture(68);
        let scan = PixelRect::new(1_398, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332);


        for row in [TOOLBAR_TOP - 2, TOOLBAR_TOP - 1, COLUMN_SEVEN_TOOLBAR_SHADOW_BOTTOM, COLUMN_SEVEN_TOOLBAR_SHADOW_BOTTOM + 1] {


            for rail in [PixelRect::new(1_389, row, 9, 1), PixelRect::new(1_530, row, 10, 1)] {
                assert!(count_pixels(&frame, rail, is_rail_gold) > 0);
                let mut broken = frame.clone();
                paint(&mut broken, rail, [12, 82, 45]);
                assert_eq!(find_solid_card_source(&broken, scan).unwrap(), None, "missing ordinary bracket {rail:?}");
            }
        }
        assert!(count_pixels(&frame, PixelRect::new(1_389, 954, 9, 12), is_toolbar_shadow_gold) > 0);
        assert!(count_pixels(&frame, PixelRect::new(1_530, 954, 10, 12), is_toolbar_shadow_gold) > 0);
    }


    /// Bracketing cannot excuse absent warm rail chroma. Reusing the complete
    /// block in another column cannot extend its measured shadow exception.
    #[test]
    fn column_seven_shadow_keeps_positive_colour_and_column_scope() {
        let frame = fixture(68);
        let scan = PixelRect::new(1_398, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332);


        for colour in [[95, 95, 95], [90, 100, 125], [95, 70, 65], [12, 82, 45]] {
            assert!(!is_toolbar_shadow_gold(colour));


            for x in [1_389, 1_530] {
                let mut recoloured = frame.clone();
                paint(&mut recoloured, PixelRect::new(x, 951, if x == 1_389 { 9 } else { 10 }, 15), colour);
                assert_eq!(find_solid_card_source(&recoloured, scan).unwrap(), None, "rail {x}, colour {colour:?}");
            }
        }


        for x in [FIRST_COLUMN_X, FIRST_COLUMN_X + COLUMN_PITCH * 4] {
            let mut other_column = frame.clone();
            copy_region(&frame, &mut other_column, PixelRect::new(1_388, 410, CARD_WIDTH + 20, TABLEAU_OUTLINE_BOTTOM - 410), PixelPoint::new(x as i32 - 10, 410));
            assert_eq!(find_solid_card_source(&other_column, PixelRect::new(x, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap(), None,
                "the measured column-7 shadow cannot authorise column x{x}");
        }
    }


    /// The four new recognition rows do not excuse clipping, missing closure
    /// or dark card paper. The Windows taskbar remains outside every scan.
    #[test]
    fn new_long_sources_retain_termination_closed_edges_and_paper() {


        for (number, x, top) in [(68, 1_398, 425), (69, 726, 636)] {
            let frame = fixture(number);
            let scan = PixelRect::new(x, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332);
            let corruptions = [
                (PixelRect::new(x - 8, TABLEAU_OUTLINE_BOTTOM - 1, 1, 1), [230, 185, 80]),
                (PixelRect::new(x + 18, 980, CARD_WIDTH - 36, 18), [12, 82, 45]),
                (PixelRect::new(x + 18, top - 10, CARD_WIDTH - 36, 11), [12, 82, 45]),
                (PixelRect::new(x + 18, TOOLBAR_TOP - 36, CARD_WIDTH - 36, 12), [32, 32, 32]),
            ];


            for (bounds, colour) in corruptions {
                let mut corrupted = frame.clone();
                paint(&mut corrupted, bounds, colour);
                assert_eq!(find_solid_card_source(&corrupted, scan).unwrap(), None, "K{number} corrupted {bounds:?}");
            }
            let mut below_scan = frame.clone();
            paint(&mut below_scan, PixelRect::new(x - 10, TABLEAU_OUTLINE_BOTTOM, CARD_WIDTH + 20, 1_033 - TABLEAU_OUTLINE_BOTTOM), [230, 185, 80]);
            assert_eq!(find_solid_card_source(&below_scan, scan).unwrap(), find_solid_card_source(&frame, scan).unwrap(),
                "pixels below the terminated observed outline remain irrelevant");
        }
    }


    /// K80's complete620-pixel source passes the existing rail, edge and paper
    /// guards. Its whole run needs one upper-card click, never toolbar input.
    #[test]
    fn native_taller_run_is_one_safe_upper_click() {
        let frame = fixture(80);
        assert!(is_gameplay_scene(&frame).unwrap());
        let expected = PixelRect::new(1_398, 372, CARD_WIDTH, 620);
        assert_eq!(find_tableau_source(&frame, PixelRect::new(1_398, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332)).unwrap(), Some(expected));
        let planned = action(&frame);
        assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Tableau { column: 7, top: 372, bottom: 992 }));
        assert_eq!(planned.operation(), InputOperation::Click(PixelPoint::new(1_464, 412)));
        assert_eq!(planned.effect_bounds(), PixelRect::new(1_398, 372, CARD_WIDTH, TOOLBAR_TOP - 372));
        assert!(!completion_evidence(&frame).unwrap().complete_candidate);
        assert_eq!(action(&fixture(81)).target, ActionTarget::Klondike(KlondikeTarget::Waste { offset: 1 }));
    }


    /// Taller recognition cannot excuse clipped rails, absent closure, broken
    /// rails, dark paper or a source cut off by the existing scan boundary.
    #[test]
    fn native_taller_run_retains_outline_and_scan_guards() {
        let frame = fixture(80);
        let scan = PixelRect::new(1_398, 332, CARD_WIDTH, TABLEAU_OUTLINE_BOTTOM - 332);


        for (bounds, colour) in [
            (PixelRect::new(1_389, TABLEAU_OUTLINE_BOTTOM - 1, 1, 1), [230, 185, 80]),
            (PixelRect::new(1_416, 980, CARD_WIDTH - 36, 18), [12, 82, 45]),
            (PixelRect::new(1_416, 362, CARD_WIDTH - 36, 11), [12, 82, 45]),
            (PixelRect::new(1_389, 372, 9, 620), [12, 82, 45]),
            (PixelRect::new(1_416, TOOLBAR_TOP - 36, CARD_WIDTH - 36, 12), [32, 32, 32]),
        ] {
            let mut corrupted = frame.clone();
            paint(&mut corrupted, bounds, colour);
            assert_eq!(find_tableau_source(&corrupted, scan).unwrap(), None, "corrupted {bounds:?}");
        }
        assert_eq!(find_tableau_source(&frame, PixelRect::new(1_398, 332, CARD_WIDTH, 936 - 332)).unwrap(), None,
            "an internal crossbar cannot close clipped continuing rails");
        let mut below_scan = frame.clone();
        paint(&mut below_scan, PixelRect::new(1_388, TABLEAU_OUTLINE_BOTTOM, CARD_WIDTH + 20, 1_033 - TABLEAU_OUTLINE_BOTTOM), [230, 185, 80]);
        assert_eq!(find_tableau_source(&below_scan, scan).unwrap(), find_tableau_source(&frame, scan).unwrap(),
            "pixels below the unchanged scan cannot supply outline authority");
    }


    /// Canonical geometry stays inside the unchanged native recognition scan;
    /// a larger height capacity never supplies a click below the toolbar.
    #[test]
    fn tableau_capacity_uses_existing_native_bounds() {
        assert!(canonical_action(KlondikeTarget::Tableau { column: 7, top: 332, bottom: 998 }).is_some());


        for target in [
            KlondikeTarget::Tableau { column: 7, top: 331, bottom: 998 },
            KlondikeTarget::Tableau { column: 7, top: 372, bottom: 999 },
            KlondikeTarget::Tableau { column: 7, top: 907, bottom: 998 },
            KlondikeTarget::Tableau { column: 7, top: 819, bottom: 998 },
            KlondikeTarget::Tableau { column: 0, top: 372, bottom: 992 },
            KlondikeTarget::Tableau { column: 8, top: 372, bottom: 992 },
            KlondikeTarget::Tableau { column: 7, top: 992, bottom: 372 },
        ] {
            assert!(canonical_action(target).is_none(), "unsupported {target:?}");
        }
    }

}
