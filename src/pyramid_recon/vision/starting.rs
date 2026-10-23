//! Input-free readiness for a newly dealt board using unchanged native recognition.
//!
//! Unsupported scenes, dialogs and invalid frames remain errors. Incomplete card
//! evidence on a supported scene may be observed again within the caller's bound;
//! it never supplies an inferred identity or permission to draw.

use super::*;


/// Complete untouched position or a supported scene whose cards are not ready.
pub(crate) enum StartingReadiness {
    /// All identities are unique, LEFT is a card and RIGHT is positively empty.
    Ready(BoardObservation),
    /// Original recognition refusal, retained for bounded diagnostic reporting.
    Pending(String),
}


impl ReconVision {


    /// Check a post-handoff starting frame without interpreting challenge headers.
    ///
    /// A ready result still needs an equal fresh observation and normal Recon.
    /// Invalid dimensions/layout, unsupported scene and any recognised dialog are
    /// fatal. Only card/pile evidence on the original supported board may wait.
    pub(crate) fn starting_readiness(
        &self,
        frame: &CapturedFrame,
    ) -> Result<StartingReadiness, String> {
        require(
            frame.width == 1920 && frame.height == 1080,
            "starting readiness requires a native 1920x1080 frame",
        )?;
        let view = frame
            .as_view()
            .map_err(|_| "invalid starting frame layout")?;
        require(
            self.board.matches(view)? && !self.dialog.matches(view, false)?,
            "starting readiness requires the supported board without a dialog",
        )?;
        match self.observe(frame) {
            Ok(ReconScene::Board(board))
                if matches!(board.left, StockFace::Card(_)) && board.right == StockFace::Empty =>
            {
                Ok(StartingReadiness::Ready(board))
            }
            Ok(ReconScene::Board(_)) => Ok(StartingReadiness::Pending(
                "Starting board requires a recognised LEFT card and positively empty RIGHT"
                    .to_owned(),
            )),
            Ok(_) => Err("Starting scene changed during readiness recognition".to_owned()),
            Err(error) => Ok(StartingReadiness::Pending(error)),
        }
    }
}
