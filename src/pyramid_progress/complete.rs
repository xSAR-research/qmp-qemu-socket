//! Positive native Challenge Complete proof, separate from Perfect and GAME WIN.
//!
//! Two separated complete-title regions and the modal scaffold establish the
//! shared event. A cold entry additionally requires a calibrated whole literal
//! requirement. A caller at a qualified whole-action boundary may bind the
//! event to its retained actual goal without fabricating a final numeric value.


use super::{ChallengeKind, ChallengeProgress, HeaderPatch, matches_patch_with_tolerance};
use crate::{
    capture::CapturedFrame,
    geometry::PixelRect,
    parameters::{NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH},
};


/// Positive completion event, with actual numeric evidence distinct from its binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ChallengeCompletion {
    /// Last actual numeric requirement, or the literal final reading when available.
    progress: ChallengeProgress,
    /// Numeric terminal header actually read; absent when only the bound event is visible.
    terminal_progress: Option<ChallengeProgress>,
    /// A complete measured Continue control is visible; the operator owns its dismissal.
    operator_continue: bool,
    /// The complete measured skip instruction positively identifies reward counting.
    reward_counting: bool,
}


impl ChallengeCompletion {
    /// Return the last actual numeric reading for comparison with the bound requirement.
    pub(crate) fn progress(self) -> ChallengeProgress {
        self.progress
    }


    /// Identify a positively visible operator endpoint without granting a Continue click.
    pub(crate) fn operator_continue(self) -> bool {
        self.operator_continue
    }


    /// Identify a positive counting stage; a missing Continue control is never skip authority.
    pub(crate) fn reward_counting(self) -> bool {
        self.reward_counting
    }


    /// Describe the event without replacing an unread final counter with its target.
    pub(crate) fn description(self) -> String {
        match self.terminal_progress {
            Some(progress) => format!("actual terminal requirement {progress:?}"),
            None => format!(
                "game-reported Challenge Complete; last observed requirement {:?}; final numeric counter not read",
                self.progress
            ),
        }
    }
}


/// Existing per-appearance title tolerance, retained for each genuine native sample.
const TITLE_TOLERANCE: u8 = 9;


/// Whole native patches exclude changing rank-specific body text and reward values.
const TERMINAL_PATCHES: [HeaderPatch; 3] = [
    HeaderPatch {
        bounds: PixelRect::new(648, 168, 64, 60),
        rgb: include_bytes!("../../tests/fixtures/pyramid-objectives/complete-title-left.rgb"),
    },
    HeaderPatch {
        bounds: PixelRect::new(1000, 168, 64, 60),
        rgb: include_bytes!("../../tests/fixtures/pyramid-objectives/complete-title-right.rgb"),
    },
    HeaderPatch {
        bounds: PixelRect::new(1387, 891, 26, 60),
        rgb: include_bytes!("../../tests/fixtures/pyramid-objectives/complete-scaffold.rgb"),
    },
];


/// Native whole headers retain words, numbers, background and negative space.
const REQUIREMENTS: [(ChallengeKind, u32, &[u8]); 3] = [
    (
        ChallengeKind::Cards,
        12,
        include_bytes!("../../tests/fixtures/pyramid-objectives/complete-fours-header.rgb"),
    ),
    (
        ChallengeKind::Cards,
        12,
        include_bytes!("../../tests/fixtures/pyramid-objectives/complete-aces-header.rgb"),
    ),
    (
        ChallengeKind::Boards,
        2,
        include_bytes!("../../tests/fixtures/pyramid-objectives/complete-boards-two-header.rgb"),
    ),
];


/// Require exact native geometry, separated terminal structure and one unique requirement.
///
/// Full header channels use a narrow inclusive one-level native allowance. The
/// scaffold is byte-exact across both originals. Animated, ordinary, Perfect,
/// malformed or unsupported frames return unknown. No allocation or input occurs.
pub(super) fn observe(frame: &CapturedFrame) -> Result<Option<ChallengeCompletion>, String> {
    if !is_complete(frame)? {
        return Ok(None);
    }
    let view = frame
        .as_view()
        .map_err(|error| format!("Pyramid challenge terminal frame is invalid: {error}"))?;
    let mut matched = None;
    for (kind, target, rgb) in REQUIREMENTS {
        if matches_patch_with_tolerance(
            view,
            HeaderPatch {
                bounds: PixelRect::new(814, 48, 296, 32),
                rgb,
            },
            1,
        )? {
            if matched.is_some() {
                return Ok(None);
            }
            let progress = ChallengeProgress::new(kind, target, target)
                .ok_or_else(|| "Bundled challenge terminal requirement is invalid".to_owned())?;
            matched = Some(ChallengeCompletion {
                progress,
                terminal_progress: Some(progress),
                operator_continue: matches_operator_continue(frame)?,
                reward_counting: matches_reward_counting(frame)?,
            });
        }
    }
    Ok(matched)
}


/// Positively recognise the shared settled completion event, without reading its counter.
pub(super) fn is_complete(frame: &CapturedFrame) -> Result<bool, String> {
    let view = frame
        .as_view()
        .map_err(|error| format!("Pyramid challenge terminal frame is invalid: {error}"))?;
    if (view.width(), view.height()) != (NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT) {
        return Ok(false);
    }
    for (index, patch) in TERMINAL_PATCHES.into_iter().enumerate() {
        let matched = matches_patch_with_tolerance(
            view,
            patch,
            if index == 2 { 0 } else { TITLE_TOLERANCE },
        )?;
        let alternate = index == 0
            && matches_patch_with_tolerance(
                view,
                HeaderPatch {
                    bounds: patch.bounds,
                    rgb: include_bytes!(
                        "../../tests/fixtures/pyramid-objectives/complete-title-left-123.rgb"
                    ),
                },
                TITLE_TOLERANCE,
            )?;
        if !matched && !alternate {
            return Ok(false);
        }
    }
    Ok(true)
}


/// Match the complete measured operator-owned Continue control in an already complete scene.
///
/// All 24,336 pixels compare byte-exactly at native geometry. This standalone
/// control match grants no completion or input authority; constructors first
/// require the independent completion event and an actual or retained requirement.
/// Returns an error for invalid frame storage or bundled metadata.
fn matches_operator_continue(frame: &CapturedFrame) -> Result<bool, String> {
    let view = frame
        .as_view()
        .map_err(|error| format!("Challenge Continue frame layout is invalid: {error}"))?;
    for rgb in [
        include_bytes!("../../tests/fixtures/pyramid-objectives/complete-continue.rgb").as_slice(),
        include_bytes!("../../tests/fixtures/pyramid-objectives/complete-continue-123.rgb")
            .as_slice(),
    ] {
        if matches_patch_with_tolerance(
            view,
            HeaderPatch {
                bounds: PixelRect::new(804, 842, 312, 78),
                rgb,
            },
            0,
        )? {
            return Ok(true);
        }
    }
    Ok(false)
}


/// Match the whole measured reward-counting instruction, without inferring it from a missing button.
///
/// All 13,200 pixels compare byte-exactly. Constructors separately require the
/// native completion event and actual or bound requirement. No input is authorised.
/// Returns an error for invalid frame storage or bundled reference metadata.
fn matches_reward_counting(frame: &CapturedFrame) -> Result<bool, String> {
    let view = frame
        .as_view()
        .map_err(|error| format!("Challenge reward frame layout is invalid: {error}"))?;
    matches_patch_with_tolerance(
        view,
        HeaderPatch {
            bounds: PixelRect::new(795, 858, 330, 40),
            rgb: include_bytes!("../../tests/fixtures/pyramid-objectives/complete-reward-skip.rgb"),
        },
        0,
    )
}


/// Bind a common terminal event to the actual requirement retained before a whole action.
///
/// The caller must obtain two fresh equal observations after every action gesture
/// acknowledged. This does not fabricate a numeric terminal reading. A calibrated
/// literal header, when present, remains authoritative even if it contradicts the binding.
pub(super) fn observe_bound(
    frame: &CapturedFrame,
    bound: ChallengeProgress,
) -> Result<Option<ChallengeCompletion>, String> {
    if let Some(literal) = observe(frame)? {
        return Ok(Some(literal));
    }
    Ok(is_complete(frame)?.then_some(ChallengeCompletion {
        progress: bound,
        terminal_progress: None,
        operator_continue: matches_operator_continue(frame)?,
        reward_counting: matches_reward_counting(frame)?,
    }))
}


#[cfg(test)]
mod tests;
