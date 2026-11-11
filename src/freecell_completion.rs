//! Shared native Challenge Complete appearance without Pyramid counters or input.
//!
//! The executor must bind this observation to its fresh Free Cell challenge run.
//! A positive event retires gameplay; it supplies no card count, cursor progress,
//! Continue, restart or next-board authority.

use crate::capture::CapturedFrame;


/// Match the existing separated native title/scaffold patches on this exact frame.
///
/// # Errors
/// Invalid native storage/dimensions or bundled calibration errors propagate.
pub(crate) fn observe(frame: &CapturedFrame) -> Result<bool, String> {
    crate::game_operations::validate_native_frame(frame).map_err(|error| error.to_string())?;
    crate::pyramid_progress::is_challenge_complete(frame)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::decode_png;


    /// Common measured completion has no invented Free Cell numeric requirement.
    #[test]
    fn freecell_315_shared_completion_is_separate_from_flying_perfect_and_play() {
        for bytes in [
            include_bytes!("../tests/fixtures/pyramid-objectives/fours-complete.png").as_slice(),
            include_bytes!("../tests/fixtures/pyramid-objectives/boards-complete-continue.png")
                .as_slice(),
        ] {
            assert!(observe(&decode_png(bytes).unwrap()).unwrap());
        }
        for bytes in [
            include_bytes!("../tests/fixtures/pyramid-objectives/fours-complete-animating.png")
                .as_slice(),
            include_bytes!("../tests/fixtures/pyramid-objectives/boards-one-of-two-perfect.png")
                .as_slice(),
            include_bytes!("../tests/fixtures/freecell-challenge-highlight-20261109.png")
                .as_slice(),
        ] {
            assert!(!observe(&decode_png(bytes).unwrap()).unwrap());
        }
    }
}
