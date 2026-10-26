//! Same-execution appearance of the REDEAL region, without icon or count templates.
//!
//! The baseline is captured from a verified source before any input. Only the
//! small native region is retained. Position, stock legality, context, STOP and
//! transport acknowledgement remain the controller's independent responsibilities.

use super::CapturedFrame;
use crate::{
    capture::PixelFormat,
    geometry::{PixelPoint, PixelRect},
};


/// Complete native area in which the stock-drain REDEAL control appears.
pub(super) const REGION: PixelRect = PixelRect::new(936, 792, 48, 63);


/// Inclusive channel change in eight-bit RGB units, ignoring capture noise.
const CHANNEL_DELTA: u8 = 20;


/// Materially changed pixels required for appearance, out of 3,024 region pixels.
/// Retained no-widget→widget originals change 1,353–1,378 pixels at delta 20;
/// identical absent controls change zero, and a numeral-only change changes 114.
const APPEARANCE_PIXELS: usize = 512;


/// Immutable small-region baseline belonging exclusively to one NativeIo execution.
pub(super) struct Before {
    /// Packed native region, with no card/scene identities or input authority.
    pixels: CapturedFrame,
}


impl Before {
    /// Copy only the checked region from the fresh, verified pre-input frame.
    pub(super) fn capture(frame: &CapturedFrame) -> Result<Self, String> {
        Ok(Self {
            pixels: region(frame)?,
        })
    }


    /// Count substantive RGB changes against a fresh frame of the same native scene.
    pub(super) fn changed(&self, frame: &CapturedFrame) -> Result<usize, String> {
        let after = region(frame)?;
        super::super::materially_changed_pixels(
            &self.pixels,
            &after,
            PixelRect::new(0, 0, REGION.width, REGION.height),
            None,
            CHANNEL_DELTA,
        )
    }


    /// Report appearance only; an absent baseline or invalid frame must not be guessed.
    pub(super) fn appeared(&self, frame: &CapturedFrame) -> Result<bool, String> {
        Ok(self.changed(frame)? >= APPEARANCE_PIXELS)
    }
}


/// Integer centre of the same checked native rectangle used for comparison.
pub(super) fn centre() -> Result<PixelPoint, String> {
    Ok(PixelPoint::new(
        i32::try_from(REGION.x + REGION.width / 2).map_err(|_| "REDEAL centre x overflow")?,
        i32::try_from(REGION.y + REGION.height / 2).map_err(|_| "REDEAL centre y overflow")?,
    ))
}


/// Copy one bounded native region as packed RGBA, preserving RGB exactly.
/// Reject non-native dimensions, invalid storage/bounds and allocation failures.
fn region(frame: &CapturedFrame) -> Result<CapturedFrame, String> {
    if frame.width != 1920 || frame.height != 1080 {
        return Err("REDEAL appearance requires native dimensions".to_owned());
    }
    let view = frame.as_view().map_err(|error| error.to_string())?;
    let (right, bottom) = view
        .checked_bounds(REGION)
        .map_err(|error| error.to_string())?;
    let stride = usize::try_from(REGION.width)
        .map_err(|_| "REDEAL width overflow")?
        .checked_mul(4)
        .ok_or("REDEAL stride overflow")?;
    let length = stride
        .checked_mul(usize::try_from(REGION.height).map_err(|_| "REDEAL height overflow")?)
        .ok_or("REDEAL region overflow")?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(length)
        .map_err(|_| "Cannot reserve REDEAL region")?;
    for y in REGION.y..bottom {
        for x in REGION.x..right {
            let [red, green, blue] = view.pixel_rgb(x, y).ok_or("Unreadable REDEAL region")?;
            pixels.extend_from_slice(&[red, green, blue, 255]);
        }
    }
    Ok(CapturedFrame {
        width: REGION.width,
        height: REGION.height,
        stride,
        format: PixelFormat::Rgba8,
        pixels,
    })
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
        assert_eq!(before.changed(&absent).unwrap(), 0);
        assert!(before.appeared(&one).unwrap());
        assert!(before.appeared(&two).unwrap());
        assert!(!Before::capture(&one).unwrap().appeared(&two).unwrap());
        assert_eq!(centre().unwrap(), PixelPoint::new(960, 823));
        let mut invalid = absent.clone();
        invalid.width = 1919;
        assert!(before.changed(&invalid).is_err());
        invalid = absent;
        invalid.pixels.clear();
        assert!(Before::capture(&invalid).is_err());
    }
}
