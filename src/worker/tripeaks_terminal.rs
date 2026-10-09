//! Narrow native TriPeaks terminal evidence from the supplied TP01, TP02 and TP03 PNGs.
//!
//! Fixed title and skip-prompt glyphs plus panel geometry are authority; the
//! changing level, XP, reward artwork, pointer and desktop chrome are excluded.
//! This detector cannot establish completion without fresh post-action proof.


use super::{FrameObservation, materially_changed_pixels};
use crate::{
    capture::CapturedFrame,
    detector::{has_dialog_gold_button, pixel_rgb},
    game::{ActionTarget, GameMode},
    geometry::PixelRect,
    parameters::{ACTION_CHANGE_CHANNEL_THRESHOLD, LEVEL_UP_CONTROL_VARIANTS, NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH},
    stepper::StepPlan,
    tracker::{PredictedAction, RowMask, TableauScanState, analyse_frame_with_state},
};


/// Total post-action acquisition captures for a qualified TriPeaks final-card transition.
///
/// This includes earlier ordinary result observations, not thirty additional frames.
/// No unsupported frame grants completion or input; other controller budgets remain separate.
pub(super) const FINAL_CARD_ACQUISITION_CAPTURES: usize = 30;


/// One candidate plus at most three delayed, input-free confirmation captures.
pub(super) const TERMINAL_CONFIRMATION_CAPTURES: usize = 4;


/// Distinct terminal paths cannot combine their positive observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TerminalKind {
    /// The fixed Congratulations panel authorises only its guarded reward skip.
    RewardSkip,
    /// The fixed Level Up panel over a dim reward underlay enters the existing OK stage.
    LevelUpOk,
}


impl TerminalKind {


    /// Human-readable scene evidence used by the production verification log.
    pub(super) fn label(self) -> &'static str {


        match self {
            Self::RewardSkip => "Congratulations/reward/skip",
            Self::LevelUpOk => "Level Up OK over the dim reward panel",
        }
    }
}


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


/// Fixed LEVEL-UP! glyphs and green banner, excluding antialiasing and animated art.
const LEVEL_UP_TITLE_MASK: [&str; 15] = [
    "................................................................................",
    "..???......??????.???.....???.???????..???............??....???..??????....???..",
    "..?##......######.?##.....##?.#######..##?............##....?##..?#######..?##..",
    "..?##......##????..##?....##..###????..##?............##....?##..?##??###?..##..",
    "..?##......##......###...###..###......##?............##....?##..?##...###..##..",
    "..?##......##......?##...##?..###......##?............##....?##..?##...###..##..",
    "..?##......######...##?..##...######?..##?............##....?##..?##...###..##..",
    "..?##......######...###.?##...######?..##?............##....?##..?#######...##..",
    "..?##......##.......?##.##?...###......##?.....####?..##....?##..?#####?....##..",
    "..?##......##........##?##....###......##?.....?????..##?...###..?##............",
    "..?##......##........##?##....###......##????.........###..?###..?##........??..",
    "..?######..######?...?###?....#######..#######........?#######...?##.......?##..",
    "..?######..######?....###.....#######..#######.........?#####....?##........##..",
    "................................................................................",
    "................................................................................",
];


/// Only uncovered left/right skip letters are sampled beneath the foreground panel.
/// The middle and upper rows are occluded by its sloping border and supply no evidence.
const DIM_SKIP_MASK: [&str; 9] = [
    "???????????????????????????????????????????????????????????????????????????????????",
    "???????????????????????????????????????????????????????????????????????????????????",
    "???????????????????????????????????????????????????????????????????????????????????",
    "???????????????????????????????????????????????????????????????????????????????????",
    ".#....#.#.#...##....?##.?????????????????????????????????????????..##..##..#.#..#..",
    ".#....#.#.#...#??...#.#.?????????????????????????????????????????....#.##..#.#..#..",
    "..###.#.#..##.#.#...?#?.?????????????????????????????????????????..##..#.#.#.#?#...",
    ".........................????????????????????????????????????????............#.....",
    ".........................????????????????????????????????????????............?.....",
];


/// Reward edges remain uncovered outside the foreground Level Up panel.
const DIM_REWARD_PROBES: [PixelRect; 4] = [
    PixelRect::new(507, 880, 3, 20),
    PixelRect::new(1_410, 880, 3, 20),
    PixelRect::new(550, 922, 50, 2),
    PixelRect::new(1_320, 139, 50, 2),
];


/// Independent fixed sides of the foreground panel, outside its changing level artwork.
const LEVEL_UP_PANEL_PROBES: [PixelRect; 2] = [
    PixelRect::new(460, 400, 3, 40),
    PixelRect::new(1_457, 400, 3, 40),
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


/// Prove one fresh terminal observation from its actual immutable planning and result frames.
///
/// None means the narrow overlay is absent. A visible overlay with unsupported
/// action context, fresh gameplay evidence or insufficient effect fails closed.
pub(super) fn terminal_action_evidence(
    plan: StepPlan,
    before: &FrameObservation,
    after: &FrameObservation,
) -> Result<Option<(TerminalKind, usize)>, String> {


    let Some(kind) = terminal_kind(&after.frame)? else { return Ok(None); };


    if after.gameplay_scene || after.prediction != PredictedAction::NoHighlight
        || after.observed_rows.is_some() || !last_tableau_context(plan, before)?
    {
        return Err(format!("TriPeaks {} lacks a fresh canonical final-card context; no terminal input authorised", kind.label()));
    }
    let changed = materially_changed_pixels(
        &before.frame, &after.frame, plan.input().effect_bounds(), plan.input().effect_exclusion_bounds(),
        ACTION_CHANGE_CHANNEL_THRESHOLD,
    )?;
    let required = plan.input().minimum_changed_pixels();


    if changed < required {
        return Err(format!(
            "TriPeaks {} cannot prove the last action's effect: changed pixels={changed}, required={required}; no terminal input authorised", kind.label(),
        ));
    }
    Ok(Some((kind, changed)))
}


/// Recognise the two separately audited terminal layouts without granting completion.
pub(super) fn terminal_kind(frame: &CapturedFrame) -> Result<Option<TerminalKind>, String> {


    if is_reward_overlay(frame)? {
        Ok(Some(TerminalKind::RewardSkip))
    } else if is_direct_level_up_overlay(frame)? {
        Ok(Some(TerminalKind::LevelUpOk))
    } else {
        Ok(None)
    }
}


/// Require the fixed Level Up panel, its evidenced raised OK and the dim reward underlay.
///
/// A progression modal without the uncovered reward prompt and borders is refused.
/// Changing level, rank and central artwork never supply terminal authority.
pub(super) fn is_direct_level_up_overlay(frame: &CapturedFrame) -> Result<bool, String> {


    if !frame.is_layout_valid() {
        return Err("TriPeaks terminal frame has invalid pixel storage".to_owned());
    }


    if (frame.width, frame.height) != (NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT) {
        return Ok(false);
    }


    if !text_mask_matches(frame, (800, 234), &LEVEL_UP_TITLE_MASK,
        |rgb| rgb.into_iter().all(|channel| channel >= 245),
        |[red, green, blue]| blue < 20 && green >= 100 && green >= red.saturating_add(25),
    ) || !text_mask_matches(frame, (798, 838), &DIM_SKIP_MASK,
        |[red, green, blue]| (20..=35).contains(&red) && (27..=40).contains(&green) && (37..=50).contains(&blue),
        |[red, green, blue]| red <= 10 && (12..=23).contains(&green) && (27..=38).contains(&blue),
    ) {
        return Ok(false);
    }


    for bounds in DIM_REWARD_PROBES {


        if !rectangle_matches(frame, bounds, |[red, green, blue]|
            (10..=15).contains(&red) && (20..=26).contains(&green) && (35..=43).contains(&blue)
                && green >= red.saturating_add(8) && blue >= green.saturating_add(12))
        {
            return Ok(false);
        }
    }


    for bounds in LEVEL_UP_PANEL_PROBES {


        if !rectangle_matches(frame, bounds, |rgb| rgb.into_iter().all(|channel| channel >= 235)) {
            return Ok(false);
        }
    }
    let [legacy, raised] = LEVEL_UP_CONTROL_VARIANTS;
    Ok(has_dialog_gold_button(frame, raised.probe_bounds).map_err(|error| error.to_string())?
        && !has_dialog_gold_button(frame, legacy.probe_bounds).map_err(|error| error.to_string())?)
}


/// Separate fixed patches require 95% coverage, excluding isolated decorative pixels.
fn rectangle_matches(frame: &CapturedFrame, bounds: PixelRect, predicate: fn([u8; 3]) -> bool) -> bool {
    let mut matched = 0u32;


    for y in bounds.y..bounds.bottom() {


        for x in bounds.x..bounds.right() {
            matched += u32::from(pixel_rgb(frame, x, y).is_some_and(predicate));
        }
    }
    matched * 100 >= bounds.width * bounds.height * 95
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
pub(super) enum TerminalConfirmationStep {
    /// Obtain another delayed capture within the fixed confirmation allowance.
    Observe,
    /// Two consecutive positives establish the narrowly supported terminal scene.
    Confirmed(TerminalKind),
    /// Evidence did not stabilise; stop with the latest diagnostic frame.
    Stop,
}


/// Consecutive positive evidence is reset whenever the recognised overlay disappears.
#[derive(Default)]
pub(super) struct TerminalConfirmation {
    /// Captures spent after the first positive candidate, including that candidate.
    captures: usize,
    /// Consecutive positive post-action observations, never advisory preview frames.
    positives: usize,
    /// Only consecutive observations of the same audited terminal kind can confirm.
    kind: Option<TerminalKind>,
}


impl TerminalConfirmation {


    /// Whether a candidate has started the finite input-free confirmation phase.
    pub(super) fn active(&self) -> bool { self.captures > 0 }


    /// Observe once, resetting transient evidence while preserving the spent budget.
    pub(super) fn observe(&mut self, kind: Option<TerminalKind>) -> TerminalConfirmationStep {
        self.captures += 1;
        self.positives = match kind {
            Some(kind) if self.kind == Some(kind) => self.positives + 1,
            Some(_) => 1,
            None => 0,
        };
        self.kind = kind;


        if self.positives >= 2 {
            TerminalConfirmationStep::Confirmed(kind.expect("two positive observations retain their terminal kind"))
        } else if self.captures >= TERMINAL_CONFIRMATION_CAPTURES {
            TerminalConfirmationStep::Stop
        } else {
            TerminalConfirmationStep::Observe
        }
    }
}
