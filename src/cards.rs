//! Calibrated card rectangles used by halo detection and tableau tracking.
//!
//! These regions describe screen geometry; gameplay follows the Solver's halo
//! and does not infer card ranks or maintain a separate speculative deck model.

use crate::geometry::{PixelPoint, PixelRect};

/// Number of card positions in the four-row TriPeaks tableau.
pub const TABLEAU_CARD_COUNT: usize = 28;

/// Number of calibrated card positions in each TriPeaks row.
#[cfg(test)]
pub const TABLEAU_ROW_LENGTHS: [u8; 4] = [3, 6, 9, 10];
/// First flat-array position for each TriPeaks row.
#[cfg(test)]
pub const TABLEAU_ROW_STARTS: [usize; 4] = [0, 3, 9, 18];

/// Guest-pixel rectangles and input point for one calibrated card slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardRegionPixels {
    /// Half-open rectangle enclosing the card face.
    pub card_bounds: PixelRect,
    /// Half-open rectangle enclosing the card and its Solver highlight.
    pub halo_bounds: PixelRect,
    /// Calibrated rank corner retained as part of the audited slot geometry.
    pub rank_bounds: PixelRect,
    /// Guest pixel to click when the slot is an eligible Solver target.
    pub click_point: PixelPoint,
}

impl CardRegionPixels {
    /// Group the independently calibrated rectangles and click point.
    pub const fn new(
        card_bounds: PixelRect,
        halo_bounds: PixelRect,
        rank_bounds: PixelRect,
        click_point: PixelPoint,
    ) -> Self {
        Self {
            card_bounds,
            halo_bounds,
            rank_bounds,
            click_point,
        }
    }
}

/// Validate nesting and QMP conversion of the active calibration in tests.
#[cfg(test)]
pub fn validate_card_regions(
    pixels: CardRegionPixels,
    frame_width: u32,
    frame_height: u32,
) -> Result<(), CardRegionError> {
    use crate::geometry::{pixel_point_to_qmp, pixel_rect_to_qmp};

    pixel_rect_to_qmp(pixels.card_bounds, frame_width, frame_height)?;
    pixel_rect_to_qmp(pixels.halo_bounds, frame_width, frame_height)?;
    pixel_rect_to_qmp(pixels.rank_bounds, frame_width, frame_height)?;
    pixel_point_to_qmp(pixels.click_point, frame_width, frame_height)?;
    if !rect_contains(pixels.halo_bounds, pixels.card_bounds) {
        return Err(CardRegionError::CardOutsideHalo);
    }
    if !rect_contains(pixels.card_bounds, pixels.rank_bounds) {
        return Err(CardRegionError::RankOutsideCard);
    }
    if !pixels.card_bounds.contains(pixels.click_point) {
        return Err(CardRegionError::ClickOutsideCard);
    }
    Ok(())
}

/// Require the inner half-open rectangle to fit completely inside the outer.
#[cfg(test)]
fn rect_contains(outer: PixelRect, inner: PixelRect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.right() <= outer.right()
        && inner.bottom() <= outer.bottom()
}

/// Reasons that an audited card region cannot be converted or safely clicked.
#[cfg(test)]
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CardRegionError {
    /// A rectangle or point is invalid for the declared frame.
    #[error(transparent)]
    Geometry(#[from] crate::geometry::GeometryError),
    /// The enclosing highlight rectangle does not contain the card.
    #[error("card bounds must lie inside halo bounds")]
    CardOutsideHalo,
    /// The rank corner lies partly outside the card face.
    #[error("rank bounds must lie inside card bounds")]
    RankOutsideCard,
    /// The input point lies outside the card face.
    #[error("click point must lie inside card bounds")]
    ClickOutsideCard,
}

#[cfg(test)]
mod tests {
    //! Regression checks for the active card-calibration validation rules.

    use super::*;

    /// Construct nested rectangles for a synthetic calibrated slot.
    fn test_region(x: u32, y: u32) -> CardRegionPixels {
        let card_bounds = PixelRect::new(x, y, 100, 150);
        CardRegionPixels::new(
            card_bounds,
            PixelRect::new(x - 5, y - 5, 110, 160),
            PixelRect::new(x + 3, y + 7, 25, 22),
            card_bounds.centre(),
        )
    }

    /// Reject invalid nesting and out-of-card clicks before input conversion.
    #[test]
    fn card_geometry_rejects_non_nested_regions_and_clicks() {
        let mut pixels = test_region(100, 100);
        pixels.rank_bounds = PixelRect::new(50, 50, 25, 22);
        assert_eq!(
            validate_card_regions(pixels, 1_920, 1_080),
            Err(CardRegionError::RankOutsideCard),
        );

        let mut pixels = test_region(100, 100);
        pixels.halo_bounds = PixelRect::new(110, 110, 90, 100);
        assert_eq!(
            validate_card_regions(pixels, 1_920, 1_080),
            Err(CardRegionError::CardOutsideHalo),
        );

        let mut pixels = test_region(100, 100);
        pixels.click_point = PixelPoint::new(99, 100);
        assert_eq!(
            validate_card_regions(pixels, 1_920, 1_080),
            Err(CardRegionError::ClickOutsideCard),
        );
    }
}
