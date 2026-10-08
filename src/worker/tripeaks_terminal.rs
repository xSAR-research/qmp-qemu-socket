//! Narrow native TriPeaks reward-overlay evidence from the supplied TP01 PNG.
//!
//! Fixed title and skip-prompt glyphs plus panel geometry are authority; the
//! changing level, XP, reward artwork, pointer and desktop chrome are excluded.
//! This detector cannot establish completion without fresh post-action proof.


use super::{FrameObservation, materially_changed_pixels};
use crate::{
    capture::CapturedFrame,
    detector::pixel_rgb,
    game::{ActionTarget, GameMode},
    geometry::PixelRect,
    parameters::{ACTION_CHANGE_CHANNEL_THRESHOLD, NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH},
    stepper::StepPlan,
    tracker::{PredictedAction, RowMask, TableauScanState, analyse_frame_with_state},
};


/// One candidate plus at most three delayed, input-free confirmation captures.
pub(super) const REWARD_CONFIRMATION_CAPTURES: usize = 4;


/// Sampled title glyphs: # is solid white, . is green backdrop, ? excludes antialiasing.
const TITLE_MASK: [&str; 17] = [
    "........................................................................??..................?..................................",
    "....?#####..............................................................##.................###............................?##..",
    "...#######...............................................?#.............##............?#...##?............................?##..",
    "..###?...?..............................................?##.............##...........?##...................................##..",
    ".?##?........?###?...???.##?....?##????..??.?#..?###?..?###??.???..???..##...?###?..?###??.???...?###?...???.?##....?###...##..",
    ".###........#######..########..?#######..##?##.?#####?.######.##?..###..##..?#####?.######.###..#######..?#######..#####...##..",
    ".###.......?##...##?.###..###..###..###..###?..?...?##..?##...##?..###..##..?...?##..?##...###..##?..##?.?##?.?##..##......##..",
    ".###.......###...###.###..?##..##...?##..##?.....?####..?##...##?..###..##....?####..?##...###.###...###.?##...##..###?....##..",
    ".###.......##?...###.###...##.?##...?##..##?...####?##..?##...##?..###..##..####?##..?##...###.###...?##.?##...##..?####...##..",
    ".?###......###...##?.###...##..##...###..##?...##..?##..?##...##?..###..##..##...##..?##...###.?##...###.?##...##.....###......",
    "..########.?##?.?##..###...##..###.?###..##?...##..###..?##??.###.?###..##..##..###..?##??.###..##?.?##?.?##...##..?..###..##..",
    "...#######..?#####?..###...##..?####?##..##?...####?##...####.?#######..##..####?##...####.###..?#####?..?##...##..#####..?##..",
    ".....???......???................??.###.........??........??....??...........??........??.........???...............???....??..",
    "....................................##?........................................................................................",
    "...............................#######.........................................................................................",
    "...............................#####?..........................................................................................",
    "...............................................................................................................................",
];


/// Sampled fixed prompt glyphs; dynamic reward content is deliberately outside the mask.
const SKIP_MASK: [&str; 9] = [
    "...................................................................................",
    "..###.#.#.....#........................#..................?............#...#.......",
    ".#....#.......#........................#..................#............#...........",
    ".#....#.#.#...#.#...#.#.?#.#.#.#.#.#.?.#?.#.#.#.#?.#.#....#.#?.#...#...#??.#.#..#..",
    ".#....#.#.#...##....?##.?..#.#.#.#..##.#..#.###.#..###....#.#..#...##..##..#.#..#..",
    ".#....#.#.#...#??...#.#.?..#..??.??.##.#..#.#...#..#......#.#..#.....#.##..#.#..#..",
    "..###.#.#..##.#.#...?#?.?..#..#...#.#..#..#.?##.#...##....##.##....##..#.#.#.#?#...",
    "..............................#..............................................#.....",
    ".............................?...............................................?.....",
];


/// Distinct blue outer borders and dark interior edges around the reward panel.
const PANEL_PROBES: [(PixelRect, bool); 6] = [
    (PixelRect::new(507, 400, 3, 20), true),
    (PixelRect::new(1_410, 400, 3, 20), true),
    (PixelRect::new(550, 922, 50, 2), true),
    (PixelRect::new(1_320, 139, 50, 2), true),
    (PixelRect::new(630, 330, 16, 20), false),
    (PixelRect::new(1_275, 330, 16, 20), false),
];


/// Match one text mask separately for solid glyph and backdrop evidence.
fn text_mask_matches(
    frame: &CapturedFrame,
    origin: (u32, u32),
    mask: &[&str],
    glyph: fn([u8; 3]) -> bool,
    backdrop: fn([u8; 3]) -> bool,
) -> bool {
    let mut expected = [0u32; 2];
    let mut matched = [0u32; 2];


    for (row, samples) in mask.iter().enumerate() {


        for (column, sample) in samples.bytes().enumerate() {
            let (index, predicate) = match sample {
                b'#' => (0, glyph),
                b'.' => (1, backdrop),
                _ => continue,
            };
            expected[index] += 1;
            let rgb = pixel_rgb(frame, origin.0 + column as u32 * 4, origin.1 + row as u32 * 4);


            if rgb.is_some_and(predicate) {
                matched[index] += 1;
            }
        }
    }
    (0..2).all(|index| expected[index] > 0 && matched[index] * 100 >= expected[index] * 98)
}


/// Prove one fresh reward observation from its actual immutable planning and result frames.
///
/// None means the narrow overlay is absent. A visible overlay with unsupported
/// action context, fresh gameplay evidence or insufficient effect fails closed.
pub(super) fn reward_action_evidence(
    plan: StepPlan,
    before: &FrameObservation,
    after: &FrameObservation,
) -> Result<Option<usize>, String> {


    if !is_reward_overlay(&after.frame)? {
        return Ok(None);
    }


    if after.gameplay_scene || after.prediction != PredictedAction::NoHighlight
        || after.observed_rows.is_some() || !last_tableau_context(plan, before)?
    {
        return Err("TriPeaks reward overlay lacks a fresh canonical final-card context; no terminal input authorised".to_owned());
    }
    let changed = materially_changed_pixels(
        &before.frame, &after.frame, plan.input().effect_bounds(), plan.input().effect_exclusion_bounds(),
        ACTION_CHANGE_CHANNEL_THRESHOLD,
    )?;
    let required = plan.input().minimum_changed_pixels();


    if changed < required {
        return Err(format!(
            "TriPeaks reward overlay cannot prove the last action's effect: changed pixels={changed}, required={required}; no terminal input authorised"
        ));
    }
    Ok(Some(changed))
}


/// Require the audited fixed native panel, positive title and positive skip prompt.
///
/// The 98% text thresholds apply independently to foreground and backdrop,
/// preventing a solid white or green rectangle from impersonating the title.
pub(super) fn is_reward_overlay(frame: &CapturedFrame) -> Result<bool, String> {


    if !frame.is_layout_valid() {
        return Err("TriPeaks terminal frame has invalid pixel storage".to_owned());
    }


    if (frame.width, frame.height) != (NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT) {
        return Ok(false);
    }


    if !text_mask_matches(frame, (708, 200), &TITLE_MASK,
        |rgb| rgb.into_iter().all(|channel| channel >= 245),
        |[red, green, blue]| blue < 60 && green >= 100 && green >= red.saturating_add(25),
    ) || !text_mask_matches(frame, (798, 838), &SKIP_MASK,
        |[red, green, blue]| red >= 110 && green >= 140 && blue >= 180,
        |[red, green, blue]| red < 90 && green < 130 && blue < 190,
    ) {
        return Ok(false);
    }


    for (bounds, border) in PANEL_PROBES {
        let mut matched = 0u32;


        for y in bounds.y..bounds.bottom() {


            for x in bounds.x..bounds.right() {
                let Some([red, green, blue]) = pixel_rgb(frame, x, y) else { return Ok(false); };
                let fits = if border {
                    red >= 40 && green >= 95 && blue >= 150
                        && green >= red.saturating_add(30) && blue >= green.saturating_add(50)
                } else {
                    red <= 10 && (20..=55).contains(&green) && (45..=90).contains(&blue)
                };


                if fits { matched += 1; }
            }
        }


        if matched * 100 < bounds.width * bounds.height * 95 {
            return Ok(false);
        }
    }
    Ok(true)
}


/// Require a fresh canonical top-row click on the sole positively exposed card.
pub(super) fn last_tableau_context(plan: StepPlan, before: &FrameObservation) -> Result<bool, String> {
    let action = plan.input().action();


    let ActionTarget::Tableau(target) = action.target else { return Ok(false); };


    if target.row != 1 || !before.gameplay_scene || before.prediction != plan.before()
        || before.observed_rows != Some(RowMask::all_for(1))
        || GameMode::TriPeaks.profile().tableau_action(target.slot_index, action.anchor) != Some(action)
    {
        return Ok(false);
    }
    let analysis = analyse_frame_with_state(&before.frame, &TableauScanState::for_mode(GameMode::TriPeaks))
        .map_err(|error| format!("TriPeaks final-card context failed: {error}"))?;
    Ok(analysis.top_row_face_up_count == 1)
}


/// Result of one fresh terminal observation; this state never emits input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RewardConfirmationStep {
    /// Obtain another delayed capture within the fixed confirmation allowance.
    Observe,
    /// Two consecutive positives establish the narrowly supported terminal scene.
    Confirmed,
    /// Evidence did not stabilise; stop with the latest diagnostic frame.
    Stop,
}


/// Consecutive positive evidence is reset whenever the recognised overlay disappears.
#[derive(Default)]
pub(super) struct RewardConfirmation {
    /// Captures spent after the first positive candidate, including that candidate.
    captures: usize,
    /// Consecutive positive post-action observations, never advisory preview frames.
    positives: usize,
}


impl RewardConfirmation {


    /// Whether a candidate has started the finite input-free confirmation phase.
    pub(super) fn active(&self) -> bool { self.captures > 0 }


    /// Observe once, resetting transient evidence while preserving the spent budget.
    pub(super) fn observe(&mut self, positive: bool) -> RewardConfirmationStep {
        self.captures += 1;
        self.positives = if positive { self.positives + 1 } else { 0 };


        if self.positives >= 2 {
            RewardConfirmationStep::Confirmed
        } else if self.captures >= REWARD_CONFIRMATION_CAPTURES {
            RewardConfirmationStep::Stop
        } else {
            RewardConfirmationStep::Observe
        }
    }
}
