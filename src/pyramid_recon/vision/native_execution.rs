//! Bundled-only execution evidence for the measured zero-deal, empty-stock position.
//!
//! Both complete stock and caption rasters supplement existing positive empty-pile
//! evidence. Ordinary Recon, custom profiles and input/completion authority are unchanged.

use super::*;


/// Whole native rasters coupled only to expected and positively recognised empty piles.
#[derive(Clone, Debug)]
pub(super) struct NativeExecution {
    /// Entire original dynamic scene region when no stock control is displayed.
    stock_absent: Pattern,
    /// Entire literal zero-deal caption from the same native original.
    zero_deals: Pattern,
}


impl NativeExecution {
    /// Decode bounded full-mask compile-time evidence with tolerance four and no omissions.
    pub(super) fn bundled() -> Result<Self, String> {
        Ok(Self {
            stock_absent: redeal::pattern(
                PixelRect::new(931, 695, 60, 50),
                include_bytes!(
                    "../../../tests/fixtures/pyramid-execution-observability/stock-absent.rgb"
                ),
            )?,
            zero_deals: redeal::pattern(
                PixelRect::new(890, 874, 140, 26),
                include_bytes!(
                    "../../../tests/fixtures/pyramid-execution-observability/deals-zero.rgb"
                ),
            )?,
        })
    }


    /// Require both complete signatures in the same frame; this is never a REDEAL permit.
    fn matches(&self, view: FrameView<'_>) -> Result<bool, String> {
        Ok(self.stock_absent.matches(view, (0, 0))? && self.zero_deals.matches(view, (0, 0))?)
    }
}


impl ReconVision {
    /// Admit the measured absent control only with both expected and positive empty piles.
    ///
    /// The caller retains the modal veto, all static anchors and exact tableau checks.
    /// Custom/legacy profiles cannot opt in by identifier. Invalid layouts propagate.
    pub(super) fn matches_native_empty_stock(
        &self,
        view: FrameView<'_>,
        left: Option<Card>,
        right: Option<Card>,
    ) -> Result<bool, String> {
        let Some(native) = &self.native_execution else {
            return Ok(false);
        };
        Ok(left.is_none()
            && right.is_none()
            && if self.colour_independent {
                self.native_shape(view, &native.stock_absent, 0)?
                    && self.native_shape(view, &native.zero_deals, 1)?
            } else {
                native.matches(view)?
            }
            && self.stock_face(view, LEFT_BOUNDS, &self.board.left_empty)? == StockFace::Empty
            && self.stock_face(view, RIGHT_BOUNDS, &self.board.right_empty)? == StockFace::Empty)
    }
}
