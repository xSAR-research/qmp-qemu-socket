//! Bounded native challenge objectives, separate from normal game ordinals.
//!
//! Complete measured words identify Score, Boards or rank-independent Card progress.
//! Native connected components and a finite graded glyph bank read the counter.
//! This observation carries no scene, terminal, acquisition or input authority.

use std::num::NonZeroU32;

use super::{HeaderPatch, matches_patch_with_tolerance};
use crate::{
    capture::CapturedFrame,
    geometry::PixelRect,
    parameters::{NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH},
};
use xsar::image_matching::FrameView;


mod cleared;
mod decimal;
mod digits;
mod templates;


#[cfg(test)]
mod tests;


/// Positively observed challenge requirement; a denominator cannot identify it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ChallengeKind {
    /// Number of actually cleared boards, independent of normal current ordinals.
    Boards,
    /// Displayed cumulative score, including a possible final overshoot.
    Score,
    /// Displayed Card progress; the preceding rank name is deliberately not identified.
    Cards,
}


/// Literal objective, current progress and positive target belonging to one frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ChallengeProgress {
    /// Requirement identity fixed for a running challenge.
    pub(crate) kind: ChallengeKind,
    /// Actual displayed value, never inferred from a local action count.
    pub(crate) current: u32,
    /// Actual nonzero displayed target, fixed through the challenge.
    pub(crate) target: NonZeroU32,
}


impl ChallengeProgress {
    /// Validate finite decimal values; only Score may exceed its displayed target.
    pub(crate) fn new(kind: ChallengeKind, current: u32, target: u32) -> Option<Self> {
        let target = NonZeroU32::new(target)?;
        if current > 999_999
            || target.get() > 999_999
            || (kind != ChallengeKind::Score && current > target.get())
        {
            return None;
        }
        Some(Self {
            kind,
            current,
            target,
        })
    }


    /// Require unchanged objective kind/target and genuinely observed monotonic progress.
    pub(crate) fn follows(self, previous: Self) -> bool {
        self.kind == previous.kind
            && self.target == previous.target
            && self.current >= previous.current
    }


    /// Report the observed numeric requirement only; this does not classify a terminal scene.
    pub(crate) fn target_reached(self) -> bool {
        self.current >= self.target.get()
    }
}


/// One complete native objective label; multiple appearances may name the same requirement.
struct Label {
    /// Positively transcribed Boards or Score words belonging to this appearance.
    kind: ChallengeKind,
    /// Whole RGB label and its negative space, excluding the changing counter.
    patch: HeaderPatch,
}


/// Existing finite label bound; added native appearances do not raise it.
const LABEL_TOLERANCE: u8 = 9;


/// Observe one unique measured objective and a well-formed native decimal counter.
///
/// Unknown artwork, incomplete shared words, glyphs, punctuation or ambiguous labels return
/// `None`. All successful scratch storage is fixed-capacity and stack-owned.
///
/// # Errors
///
/// Invalid frame storage or bundled reference metadata returns an error.
pub(super) fn observe(frame: &CapturedFrame) -> Result<Option<ChallengeProgress>, String> {
    let view = frame
        .as_view()
        .map_err(|error| format!("Pyramid challenge frame layout is invalid: {error}"))?;
    if (view.width(), view.height()) != (NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT) {
        return Ok(None);
    }
    let Some(kind) = identify_label(view)? else {
        return Ok(None);
    };
    let bounds = match kind {
        ChallengeKind::Score => PixelRect::new(958, 48, 142, 32),
        ChallengeKind::Boards | ChallengeKind::Cards => PixelRect::new(1_038, 48, 72, 32),
    };
    let Some(tokens) = digits::read(view, bounds) else {
        return Ok(None);
    };
    Ok(decimal::parse(kind, tokens.as_slice()))
}


/// Identify the measured Kings words for a default only, after generic Card recognition.
pub(crate) fn is_kings_header(frame: &CapturedFrame) -> Result<bool, String> {
    let view = frame.as_view().map_err(|error| error.to_string())?;
    if (view.width(), view.height()) != (NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT) {
        return Ok(false);
    }
    matches_patch_with_tolerance(
        view,
        HeaderPatch {
            bounds: PixelRect::new(814, 48, 224, 32),
            rgb: include_bytes!("../../tests/fixtures/pyramid-kings-default/kings-label.rgb"),
        },
        LABEL_TOLERANCE,
    )
}


/// Require all words and background pixels, rejecting cross-objective ambiguity.
fn identify_label(view: FrameView<'_>) -> Result<Option<ChallengeKind>, String> {
    let mut selected = None;
    for label in &templates::LABELS {
        if matches_patch_with_tolerance(view, label.patch, LABEL_TOLERANCE)? {
            if selected.is_some_and(|kind| kind != label.kind) {
                return Ok(None);
            }
            selected = Some(label.kind);
        }
    }
    match selected {
        Some(kind) => Ok(Some(kind)),
        None => cleared::observe(view).map(|matched| matched.then_some(ChallengeKind::Cards)),
    }
}
