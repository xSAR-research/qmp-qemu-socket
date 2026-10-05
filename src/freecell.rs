//! Free Cell's independent mode boundary and read-only calibration intake.
//!
//! Future gameplay follows the guest Solver: one click on a highlighted CELL or
//! PLAY card/run, then an editable settle and fresh capture. Card ranks, legal
//! moves and changed-pixel effect proofs are not modelled. This preparation mode
//! defines no source coordinates or input operations. Detection and the ordered
//! single-board terminal sequence will be added with their native evidence.

use crate::{
    capture::CapturedFrame,
    detector::HaloDetectionError,
    game::{GameMode, GameProfile, TargetSelectionPolicy},
    geometry::PixelPoint,
    tracker::{FrameAnalysis, PredictedAction},
};


/// Independent calibration profile; row hints and shared progress grant no input.
/// The single non-empty row hint only satisfies shared scan-state construction;
/// no fixed-row or consumed-card geometry is inherited from the earlier modes.
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


/// Reject malformed frames and dimensions outside the supplied native evidence.
fn validate_frame(frame: &CapturedFrame) -> Result<(), HaloDetectionError> {


    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }


    if frame.width != PROFILE.frame_width || frame.height != PROFILE.frame_height {
        return Err(HaloDetectionError::BoundsOutsideFrame);
    }
    Ok(())
}


/// No uncalibrated scene may authorise gameplay, Solver or terminal input.
pub fn is_gameplay_scene(frame: &CapturedFrame) -> Result<bool, HaloDetectionError> {
    validate_frame(frame)?;
    Ok(false)
}


/// Retain the native frame for read-only calibration without guessing a HALO.
pub fn analyse(frame: &CapturedFrame) -> Result<FrameAnalysis, HaloDetectionError> {
    validate_frame(frame)?;
    Ok(FrameAnalysis {
        prediction: PredictedAction::CalibrationOnly { mode: GameMode::FreeCell },
        observed_rows: None,
        top_row_face_up_count: 0,
    })
}


#[cfg(test)]
mod tests {
    //! Mode dispatch, invalid-frame rejection and calibration input isolation.

    use super::*;
    use crate::{
        capture::PixelFormat,
        stepper::{StepValidationError, plan_step},
        tracker::{TableauScanState, analyse_frame_with_state, is_gameplay_scene_for_mode},
    };


    /// Construct native storage without supplying positive scene evidence.
    fn blank_frame() -> CapturedFrame {
        CapturedFrame {
            width: PROFILE.frame_width,
            height: PROFILE.frame_height,
            stride: PROFILE.frame_width as usize * 4,
            format: PixelFormat::Rgba8,
            pixels: vec![0; PROFILE.frame_width as usize * PROFILE.frame_height as usize * 4],
        }
    }


    /// Shared analysis cannot inherit another mode's scene or action calibration.
    #[test]
    fn freecell_dispatch_is_calibration_only() {
        let frame = blank_frame();
        let state = TableauScanState::for_mode(GameMode::FreeCell);
        assert_eq!(state.mode(), GameMode::FreeCell);
        assert_eq!(analyse_frame_with_state(&frame, &state).unwrap().prediction,
            PredictedAction::CalibrationOnly { mode: GameMode::FreeCell });
        assert!(!is_gameplay_scene_for_mode(&frame, GameMode::FreeCell).unwrap());
        assert!(GameMode::FreeCell.profile().game_progress.is_none());
        assert!(GameMode::FreeCell.profile().tableau_cards.is_empty());
        assert!(GameMode::FreeCell.profile().tableau_rows.is_empty());
        assert_eq!(GameMode::FreeCell.profile().boards_per_game, 1);
    }


    /// Arbitrary bright pixels cannot turn this preparation mode into input authority.
    #[test]
    fn calibration_prediction_never_produces_an_input_plan() {
        let mut frame = blank_frame();
        frame.pixels.fill(255);
        let prediction = analyse(&frame).unwrap().prediction;
        assert!(!is_gameplay_scene(&frame).unwrap());
        assert_eq!(plan_step(prediction),
            Err(StepValidationError::CalibrationOnly { mode: GameMode::FreeCell }));
    }


    /// Invalid storage and non-native dimensions never become a calibration frame.
    #[test]
    fn analysis_rejects_invalid_native_frame_layout() {
        let mut frame = blank_frame();
        frame.pixels.truncate(8);
        assert_eq!(analyse(&frame), Err(HaloDetectionError::InvalidFrameLayout));
        let mut frame = blank_frame();
        frame.width = 1_919;
        assert_eq!(analyse(&frame), Err(HaloDetectionError::BoundsOutsideFrame));
    }
}
