//! Same-execution appearance of the REDEAL region, without icon or count templates.
//!
//! The baseline is captured from a verified source before any input. Only the
//! small native region is retained. Position, stock legality, context, STOP and
//! transport acknowledgement remain the controller's independent responsibilities.

use super::CapturedFrame;
use crate::geometry::{PixelPoint, PixelRect};


/// Complete native area in which the stock-drain REDEAL control appears.
pub(super) const REGION: PixelRect = PixelRect::new(936, 792, 48, 63);


/// Immutable small-region baseline belonging exclusively to one NativeIo execution.
pub(super) struct Before {
    /// Packed native region, with no card/scene identities or input authority.
    pixels: crate::pyramid_effect::RegionBefore,
}


impl Before {
    /// Copy only the checked region from the fresh, verified pre-input frame.
    pub(super) fn capture(frame: &CapturedFrame) -> Result<Self, String> {
        Ok(Self {
            pixels: crate::pyramid_effect::RegionBefore::capture(frame, REGION)?,
        })
    }


    /// Require the approved same-pixel RGB change across the complete existing region.
    pub(super) fn appeared(
        &self,
        frame: &CapturedFrame,
        settings: crate::parameters::PyramidEffectSettings,
    ) -> Result<bool, String> {
        self.pixels.changed(frame, settings)
    }


    /// Retain exact complete-region diagnostic counts under the shared joint predicate.
    pub(super) fn changed(
        &self,
        frame: &CapturedFrame,
        settings: crate::parameters::PyramidEffectSettings,
    ) -> Result<usize, String> {
        Ok(self.pixels.measure(frame, settings)?.changed_pixels)
    }

}


/// Integer centre of the same checked native rectangle used for comparison.
pub(super) fn centre() -> Result<PixelPoint, String> {
    Ok(PixelPoint::new(
        i32::try_from(REGION.x + REGION.width / 2).map_err(|_| "REDEAL centre x overflow")?,
        i32::try_from(REGION.y + REGION.height / 2).map_err(|_| "REDEAL centre y overflow")?,
    ))
}


#[cfg(test)]
mod tests {
    use super::*;


    /// Compare unchanged original crops; cross-original pairings are synthetic sequences.
    #[test]
    fn temporal_appearance_requires_substantive_change_without_count_templates() {
        let absent = crate::capture::decode_png(include_bytes!(
            "../../../tests/fixtures/pyramid-recon/native-redeal/before.png"
        ))
        .unwrap();
        let one = crate::capture::decode_png(include_bytes!(
            "../../../tests/fixtures/pyramid-recon/native-redeal/exhausted.png"
        ))
        .unwrap();
        let two = crate::capture::decode_png(include_bytes!(
            "../../../tests/fixtures/pyramid-redeal-appearance/count-two.png"
        ))
        .unwrap();
        let before = Before::capture(&absent).unwrap();
        assert!(!before.appeared(&absent, Default::default()).unwrap());
        assert!(before.appeared(&one, Default::default()).unwrap());
        assert!(before.appeared(&two, Default::default()).unwrap());
        assert!(
            !Before::capture(&one)
                .unwrap()
                .appeared(&two, Default::default())
                .unwrap()
        );
        assert_eq!(centre().unwrap(), PixelPoint::new(960, 823));
        let mut invalid = absent.clone();
        invalid.width = 1919;
        assert!(before.appeared(&invalid, Default::default()).is_err());
        invalid = absent;
        invalid.pixels.clear();
        assert!(Before::capture(&invalid).is_err());
    }
}
