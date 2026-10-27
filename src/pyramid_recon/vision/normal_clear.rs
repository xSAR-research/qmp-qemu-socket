//! Final-only native normal-board absence proof, independent of the moving Perfect artwork.
//!
//! An explicitly opted-in profile, full normal header/footer, independent scene
//! anchors, one coherent set of 28 unobscured bands and exact exclusive piles are required.
//! Callers retain the final whole-action ACK, STOP, context and transition gates.

use super::*;


/// Existing DRAW role, which only a coupled positive empty-pile witness may replace.
const DRAW_SCENE: PixelRect = PixelRect::new(931, 695, 60, 50);


/// Full native empty-circle region, retained unchanged from the v17 calibration.
const LEFT_CIRCLE: PixelRect = PixelRect::new(767, 712, 60, 60);


/// Upper 21 slots have unobscured top bands; the bottom seven use their lower faces.
const BOTTOM_ROW_FIRST_SLOT: usize = 21;


/// Per-card native x offset, width and height of the positive absence band.
const BAND: (u32, u32, u32) = (40, 60, 4);


/// Native offsets inside the card: upper lip, or the unobscured bottom-row lower face.
const BAND_Y: (u32, u32) = (4, 164);


/// Explicit schema-2 capability; legacy profiles never acquire this proof implicitly.
#[derive(Clone, Debug)]
pub(super) struct NormalClearProfile {
    /// Positive full-region empty circle; card recognition never implies absence.
    left_circle: Pattern,
    /// Full exhausted-stock cross; an independently checked RIGHT card may remain.
    left_exhausted: Pattern,
}


impl NormalClearProfile {
    /// Validate the original native roles and the full measured empty-circle region.
    ///
    /// Returns an error for altered geometry, a partial mask or tolerance above four.
    pub(super) fn parse(value: &Value, scene: &[Pattern], left: &Pattern) -> Result<Self, String> {
        let fields = object(value, &["left_circle", "left_exhausted"])?;
        require(
            scene
                .get(EXECUTION_EMPTY_STOCK_SCENE_INDEX)
                .is_some_and(|pattern| pattern.bounds == DRAW_SCENE)
                && left.bounds == LEFT_CIRCLE,
            "normal final-clear requires original DRAW and LEFT roles",
        )?;
        let left_circle = Pattern::patch(&fields["left_circle"], LEFT_CIRCLE)?;
        require(
            left_circle.bounds == LEFT_CIRCLE
                && left_circle.tolerance <= 4
                && left_circle.pixels.iter().all(Option::is_some),
            "normal final-clear circle requires full native region and tolerance<=4",
        )?;
        let left_exhausted = Pattern::patch(&fields["left_exhausted"], LEFT_CIRCLE)?;
        require(
            left_exhausted.bounds == LEFT_CIRCLE
                && left_exhausted.tolerance <= 4
                && left_exhausted.pixels.iter().all(Option::is_some),
            "normal final-clear exhausted LEFT requires full native region and tolerance<=4",
        )?;
        Ok(Self {
            left_circle,
            left_exhausted,
        })
    }
}


impl ReconVision {
    /// Verify an empty native normal board after the independently replayed final action.
    ///
    /// All 21 upper-slot top bands and seven bottom-row lower bands must contain
    /// 240/240 unchanged felt pixels. The bundled colour-independent policy may
    /// instead prove all 28 complete top bands when Perfect crosses a lower band;
    /// the two layouts are never mixed slot by slot. This avoids moving artwork without
    /// interpreting it as board, challenge or game completion. Original scene and
    /// exact pile checks remain mandatory, including their existing coupled empty
    /// alternatives. The result proves only the expected empty board in this frame.
    ///
    /// # Errors
    /// Returns an error for missing opt-in calibration, any remaining expected card,
    /// malformed/unsupported frames, absent normal header/footer, modal, scene,
    /// positive absence or exclusive pile mismatch. Retains no input authority.
    pub(crate) fn verify_normal_final_clear(
        &self,
        frame: &CapturedFrame,
        expected: &[Option<Card>; 28],
        left: Option<Card>,
        right: Option<Card>,
    ) -> Result<(), String> {
        require(
            expected.iter().all(Option::is_none),
            "normal final-clear requires replayed empty tableau",
        )?;
        let calibration = self
            .board
            .normal_final_clear
            .as_ref()
            .ok_or("no normal final-clear calibration")?;
        require(
            crate::pyramid_progress::observe_normal(frame)?.is_some(),
            "normal final-clear requires calibrated normal header and footer",
        )?;
        let view = frame
            .as_view()
            .map_err(|_| "invalid normal final-clear frame layout".to_owned())?;
        require(
            !self.dialog_matches(view, true)?,
            "normal final-clear refused complete Undo All dialog",
        )?;
        // The final board has no REDEAL input authority. The complete exhausted
        // cross proves empty LEFT; the exact replayed RIGHT is checked separately.
        let exhausted = left.is_none()
            && self.native_stock_layout(view, LEFT_BOUNDS, &calibration.left_exhausted)?;
        let circle = left.is_none()
            && self.native_stock_layout(view, LEFT_BOUNDS, &calibration.left_circle)?;
        for (index, pattern) in self.board.scene.iter().enumerate() {
            if (exhausted || circle) && index == EXECUTION_EMPTY_STOCK_SCENE_INDEX {
                continue;
            }
            require(
                self.native_scene_anchor(view, index, pattern)?,
                "normal final-clear scene anchor mismatch",
            )?;
        }
        self.verify_normal_clear_bands(view, BAND_Y.1)
            .or_else(|lower_error| {
                if !self.colour_independent {
                    return Err(lower_error);
                }
                for (index, (x, y)) in TABLEAU_ORIGINS.iter().copied().enumerate() {
                    let offset = if index < BOTTOM_ROW_FIRST_SLOT {
                        BAND_Y.0
                    } else {
                        BAND_Y.1
                    };
                    require(
                        !self.native_clear_band_material_veto(
                            view,
                            PixelRect::new(x + BAND.0, y + offset, BAND.1, BAND.2),
                        )?,
                        &format!("normal final-clear slot {index} still contains card material"),
                    )?;
                }
                self.verify_normal_clear_bands(view, BAND_Y.0)
                    .map_err(|top_error| {
                        format!("{lower_error}; normal top-band proof: {top_error}")
                    })
            })?;
        let left_empty = if exhausted {
            &calibration.left_exhausted
        } else if circle {
            &calibration.left_circle
        } else {
            &self.board.left_empty
        };
        let face = |card: Option<Card>| card.map_or(StockFace::Empty, StockFace::Card);
        require(
            self.stock_face(view, LEFT_BOUNDS, left_empty)? == face(left),
            "normal final-clear LEFT mismatch",
        )?;
        require(
            self.stock_face(view, RIGHT_BOUNDS, &self.board.right_empty)? == face(right),
            "normal final-clear RIGHT mismatch",
        )
    }


    /// Prove one entire native absence layout using the unchanged per-band policy.
    ///
    /// `bottom_offset` is the calibrated native y offset for all seven bottom slots:
    /// either the original lower face or the existing final-clear upper lip. Every
    /// other slot uses its upper lip. An individual failed band rejects this whole
    /// layout; no lower/top per-slot combination is admitted. Returns an error for
    /// invalid bounds, unexplained paper, edges, aged material or unknown surfaces.
    fn verify_normal_clear_bands(
        &self,
        view: FrameView<'_>,
        bottom_offset: u32,
    ) -> Result<(), String> {
        for (index, (x, y)) in TABLEAU_ORIGINS.iter().copied().enumerate() {
            let offset = if index < BOTTOM_ROW_FIRST_SLOT {
                BAND_Y.0
            } else {
                bottom_offset
            };
            let band = PixelRect::new(x + BAND.0, y + offset, BAND.1, BAND.2);
            view.checked_bounds(band)
                .map_err(|_| "normal final-clear band outside frame".to_owned())?;
            require(
                self.native_absence_band(view, band)?,
                &format!("normal final-clear slot {index} lacks complete absence band"),
            )?;
        }
        Ok(())
    }
}
