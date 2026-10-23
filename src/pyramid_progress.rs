//! Separately calibrated normal-series, challenge-progress and terminal observations.
//!
//! The legacy centred literal reader remains unchanged. Normal ordinals require
//! complete measured headers and a New footer. Typed challenge progress combines
//! full objective labels with bounded decimal recognition; its own positive
//! terminal reader stays distinct from Perfect and normal GAME WIN. No result
//! grants input authority; supported scene, bound goal and temporal evidence
//! remain caller-owned.

use xsar::image_matching::FrameView;

use crate::{
    capture::CapturedFrame,
    geometry::PixelRect,
    parameters::{NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH},
};


pub(crate) mod challenge;
mod complete;
mod normal;


pub(crate) use challenge::{ChallengeKind, ChallengeProgress};
pub(crate) use complete::ChallengeCompletion;


#[cfg(test)]
mod tests;


/// Literal displayed counts, separate from game classification or input authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BoardCount {
    /// Positively matched completed count; the calibrated values are zero to two.
    pub(crate) completed: u8,
    /// Positively matched target; this reader supports only the literal three.
    pub(crate) target: u8,
}


/// Largest measured per-channel label difference across the retained zero/one/two frames.
///
/// All pixels, including gold background and antialiased negative space, are
/// compared. The left label differs by at most nine; the right by at most three.
const CHANNEL_TOLERANCE: u8 = 9;


/// Number of channels in each retained row-major RGB reference.
const RGB_CHANNELS: usize = 3;


/// One complete measured patch; no pixel is masked or excluded.
#[derive(Clone, Copy)]
struct HeaderPatch {
    /// Fixed half-open native full-frame bounds in pixels.
    bounds: PixelRect,
    /// Complete row-major RGB8 bytes, with compile-time checked fixture lengths.
    rgb: &'static [u8],
}


/// Complete measured RGB bytes for the left word patch; fixture length is checked at compile time.
const LEFT_LABEL_RGB: &[u8; 8_064] =
    include_bytes!("../tests/fixtures/pyramid-progress/label-left.rgb");


/// Complete measured RGB bytes for the right word patch; no negative space is removed.
const RIGHT_LABEL_RGB: &[u8; 6_048] =
    include_bytes!("../tests/fixtures/pyramid-progress/label-right.rgb");


/// Complete measured literal zero-of-three patch from the retained native zero frame.
const ZERO_RGB: &[u8; 3_864] =
    include_bytes!("../tests/fixtures/pyramid-progress/fraction-zero.rgb");


/// Complete measured literal one-of-three patch from the restored native one frame.
const ONE_RGB: &[u8; 3_864] = include_bytes!("../tests/fixtures/pyramid-progress/fraction-one.rgb");


/// Complete measured literal two-of-three patch from the retained native king frame.
const TWO_RGB: &[u8; 3_864] = include_bytes!("../tests/fixtures/pyramid-progress/fraction-two.rgb");


/// Both words must agree; a matching fraction under another objective is insufficient.
const LABEL_PATCHES: [HeaderPatch; 2] = [
    HeaderPatch {
        bounds: PixelRect::new(839, 50, 96, 28),
        rgb: LEFT_LABEL_RGB,
    },
    HeaderPatch {
        bounds: PixelRect::new(935, 50, 72, 28),
        rgb: RIGHT_LABEL_RGB,
    },
];


/// Fixed known fractions; this is neither general digit OCR nor inferred arithmetic.
const FRACTION_PATCHES: [(u8, HeaderPatch); 3] = [
    (
        0,
        HeaderPatch {
            bounds: PixelRect::new(1_044, 50, 46, 28),
            rgb: ZERO_RGB,
        },
    ),
    (
        1,
        HeaderPatch {
            bounds: PixelRect::new(1_044, 50, 46, 28),
            rgb: ONE_RGB,
        },
    ),
    (
        2,
        HeaderPatch {
            bounds: PixelRect::new(1_044, 50, 46, 28),
            rgb: TWO_RGB,
        },
    ),
];


/// Observe only one unique calibrated literal board count in a native-sized frame.
///
/// A valid unsupported size, unknown artwork, damaged label, ambiguous fraction
/// or any uncalibrated count returns `Ok(None)`. A result carries no mode, scene,
/// completion or input authority. Success borrows the frame and allocates no heap.
///
/// # Errors
///
/// Returns an error for invalid frame stride/storage or an invalid bundled patch.
pub(crate) fn observe(frame: &CapturedFrame) -> Result<Option<BoardCount>, String> {
    let view = frame
        .as_view()
        .map_err(|error| format!("Pyramid board-count frame layout is invalid: {error}"))?;
    if (view.width(), view.height()) != (NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT) {
        return Ok(None);
    }
    for patch in LABEL_PATCHES {
        if !matches_patch(view, patch)? {
            return Ok(None);
        }
    }
    let mut matched = None;
    for (completed, patch) in FRACTION_PATCHES {
        if matches_patch(view, patch)? {
            if matched.is_some() {
                return Ok(None);
            }
            matched = Some(BoardCount {
                completed,
                target: 3,
            });
        }
    }
    Ok(matched)
}


/// Observe the separately calibrated literal normal header and `New` footer.
///
/// All three retained normal ordinals map to zero, one or two completed boards
/// of the normal three-board game. A match does not authenticate Pyramid, an
/// initial board or input authority.
///
/// # Errors
///
/// Returns an error for invalid frame layout or an invalid bundled patch.
pub(crate) fn observe_normal(frame: &CapturedFrame) -> Result<Option<BoardCount>, String> {
    normal::observe(frame)
}


/// Read an actual challenge requirement and counter without normal-series assumptions.
///
/// # Errors
///
/// Invalid frame layout or bundled calibration metadata returns an error.
pub(crate) fn observe_challenge(
    frame: &CapturedFrame,
) -> Result<Option<ChallengeProgress>, String> {
    challenge::observe(frame)
}


/// Observe a positively measured challenge-success terminal and its exact requirement.
///
/// Callers must bind the requirement to their session, recheck STOP/context and
/// establish temporal stability before treating it as completion. No continuation
/// control, board-clear claim or normal-game restart authority is returned.
///
/// # Errors
/// Invalid native storage or bundled patch metadata returns an error.
pub(crate) fn observe_challenge_complete(
    frame: &CapturedFrame,
) -> Result<Option<ChallengeCompletion>, String> {
    complete::observe(frame)
}


/// Recognise the common settled event without granting a requirement or input authority.
pub(crate) fn is_challenge_complete(frame: &CapturedFrame) -> Result<bool, String> {
    complete::is_complete(frame)
}


/// Bind actual terminal evidence at a qualified whole-action boundary, without inventing a counter.
pub(crate) fn observe_bound_challenge_complete(
    frame: &CapturedFrame,
    bound: ChallengeProgress,
) -> Result<Option<ChallengeCompletion>, String> {
    complete::observe_bound(frame, bound)
}


/// Require every RGB channel of every patch pixel within the inclusive bound.
///
/// Checked frame coordinates and reference length precede iteration. Alpha and
/// row padding are deliberately ignored; neither supplies header evidence.
fn matches_patch(view: FrameView<'_>, patch: HeaderPatch) -> Result<bool, String> {
    matches_patch_with_tolerance(view, patch, CHANNEL_TOLERANCE)
}


/// Compare a complete RGB patch with its caller's measured inclusive channel bound.
///
/// Checked coordinates and exact reference length precede iteration. No alpha,
/// row padding, resizing, alignment search or pixel mask supplies evidence.
fn matches_patch_with_tolerance(
    view: FrameView<'_>,
    patch: HeaderPatch,
    tolerance: u8,
) -> Result<bool, String> {
    view.checked_bounds(patch.bounds)
        .map_err(|error| format!("Pyramid board-count patch bounds are invalid: {error}"))?;
    let bytes = usize::try_from(patch.bounds.width)
        .ok()
        .and_then(|width| {
            usize::try_from(patch.bounds.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(RGB_CHANNELS))
        .ok_or_else(|| "Pyramid board-count reference size overflow".to_owned())?;
    if patch.rgb.len() != bytes {
        return Err("Pyramid board-count reference length differs from its bounds".to_owned());
    }
    for (index, expected) in patch.rgb.as_chunks::<RGB_CHANNELS>().0.iter().enumerate() {
        let index = u32::try_from(index)
            .map_err(|_| "Pyramid board-count pixel index overflow".to_owned())?;
        let x = patch
            .bounds
            .x
            .checked_add(index % patch.bounds.width)
            .ok_or_else(|| "Pyramid board-count column overflow".to_owned())?;
        let y = patch
            .bounds
            .y
            .checked_add(index / patch.bounds.width)
            .ok_or_else(|| "Pyramid board-count row overflow".to_owned())?;
        let actual = view
            .pixel_rgb(x, y)
            .ok_or_else(|| "Pyramid board-count pixel is outside its checked frame".to_owned())?;
        if actual
            .into_iter()
            .zip(expected)
            .any(|(actual, expected)| actual.abs_diff(*expected) > tolerance)
        {
            return Ok(false);
        }
    }
    Ok(true)
}
