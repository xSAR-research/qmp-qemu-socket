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
            !self.dialog_matches(view, false)?,
            "starting readiness requires the supported board without a dialog",
        )?;
        if !self.board_scene_matches(view)? {
            require(
                self.starting_surface_pending(view)?,
                "starting readiness requires the supported board without a dialog",
            )?;
            return Ok(StartingReadiness::Pending(
                "Starting board dynamic stock control is not yet positively recognised".to_owned(),
            ));
        }
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

    /// Retain only pending observation for a bundled surface whose stock control is still dealing.
    ///
    /// Every original static scene role and positive legend remains mandatory.
    /// Omitting the dynamic stock role can never produce Ready or input authority;
    /// complete readiness still uses the unchanged whole-board classifier above.
    /// Custom and legacy profiles cannot opt in. Malformed metadata/pixels propagate.
    fn starting_surface_pending(&self, view: FrameView<'_>) -> Result<bool, String> {
        if !self.colour_independent || self.board.scene.len() < 3 {
            return Ok(false);
        }
        for (index, pattern) in self.board.scene.iter().enumerate() {
            if index != EXECUTION_EMPTY_STOCK_SCENE_INDEX
                && !self.native_scene_anchor(view, index, pattern)?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::decode_png;


    /// Copy complete retained corner pixels without manufacturing a rank or suit glyph.
    fn copy_corner(frame: &mut CapturedFrame, from: (u32, u32), to: (u32, u32)) {
        let original = frame.clone();
        for y in 4..53_usize {
            for x in 3..34_usize {
                let source = (from.1 as usize + y) * original.stride + (from.0 as usize + x) * 4;
                let dest = (to.1 as usize + y) * frame.stride + (to.0 as usize + x) * 4;
                frame.pixels[dest..dest + 4].copy_from_slice(&original.pixels[source..source + 4]);
            }
        }
    }


    /// Complete but duplicate identities or a visibly occupied RIGHT can never become Ready.
    #[test]
    fn native_151_starting_duplicates_and_occupied_right_remain_pending() {
        let vision = ReconVision::bundled_native_v23().unwrap();
        let original = decode_png(include_bytes!(
            "../../../tests/fixtures/pyramid-standard-series/normal-1.png"
        ))
        .unwrap();
        assert!(matches!(
            vision.starting_readiness(&original).unwrap(),
            StartingReadiness::Ready(_)
        ));
        let mut duplicate_tableau = original.clone();
        copy_corner(
            &mut duplicate_tableau,
            TABLEAU_ORIGINS[0],
            TABLEAU_ORIGINS[1],
        );
        assert!(matches!(
            vision.starting_readiness(&duplicate_tableau).unwrap(),
            StartingReadiness::Pending(_)
        ));
        let mut duplicate_left = original.clone();
        copy_corner(
            &mut duplicate_left,
            TABLEAU_ORIGINS[0],
            (LEFT_BOUNDS.x, LEFT_BOUNDS.y),
        );
        assert!(matches!(
            vision.starting_readiness(&duplicate_left).unwrap(),
            StartingReadiness::Pending(_)
        ));
        let mut occupied_right = original.clone();
        for y in 0..LEFT_BOUNDS.height as usize {
            let source =
                (LEFT_BOUNDS.y as usize + y) * original.stride + LEFT_BOUNDS.x as usize * 4;
            let dest =
                (RIGHT_BOUNDS.y as usize + y) * occupied_right.stride + RIGHT_BOUNDS.x as usize * 4;
            occupied_right.pixels[dest..dest + LEFT_BOUNDS.width as usize * 4]
                .copy_from_slice(&original.pixels[source..source + LEFT_BOUNDS.width as usize * 4]);
        }
        assert!(matches!(
            vision.starting_readiness(&occupied_right).unwrap(),
            StartingReadiness::Pending(_)
        ));
    }
}
