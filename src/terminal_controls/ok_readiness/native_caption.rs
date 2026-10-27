//! Complete current-frame Klondike OK appearance in native CLASSIC BLUE geometry.
//!
//! Calibration is the exact 60 by 35 RGB patch from retained observation 17,
//! where the original exact-black scanner recognised (944,771). All 2,100 pixels,
//! including both letters, their holes and surrounding button material, must pass
//! the per-channel limit of 12. Independent observations 18 and 19 differ by at
//! most 10 and 3 respectively. This is one bounded appearance, not a near-black
//! threshold, body-only approval, font classifier or completion detector.

use xsar::image_matching::FrameView;

use crate::geometry::{PixelPoint, PixelRect};


/// Fixed native horizontal origin, covering both O and K.
const LEFT: u32 = 930;


/// Complete caption width in native pixels.
const WIDTH: u32 = 60;


/// Complete caption height, centred on the measured current button.
const HEIGHT: u32 = 35;


/// Packed RGB bytes per complete native caption row: 60 pixels times three channels.
const ROW_BYTES: usize = 180;


/// Largest permitted absolute difference in every RGB channel of every pixel.
const CHANNEL_LIMIT: u8 = 12;


/// Packed row-major RGB copied without modification from the byte-original capture.
const RGB: &[u8; 6_300] = include_bytes!("klondike-ok-caption.rgb");


/// Match all current caption pixels and derive a point inside the measured body.
///
/// The caller validates native layout and settled body dimensions first. Missing
/// pixels, coordinate overflow, a point outside the body or any channel mismatch
/// returns `None`. At most 2,100 borrowed RGB observations are performed; no
/// capture, allocation, input or historical coordinate is used.
pub(super) fn recognise(
    view: FrameView<'_>,
    body: PixelRect,
    work: &mut u64,
) -> Option<PixelPoint> {
    let centre_y = body.y.checked_add(body.height.checked_sub(1)? / 2)?;
    let top = centre_y.checked_sub(HEIGHT / 2)?;
    for (dy, row) in RGB.as_chunks::<ROW_BYTES>().0.iter().enumerate() {
        let y = top.checked_add(u32::try_from(dy).ok()?)?;
        for (dx, expected) in row.as_chunks::<3>().0.iter().enumerate() {
            let x = LEFT.checked_add(u32::try_from(dx).ok()?)?;
            *work += 1;
            let actual = view.pixel_rgb(x, y)?;
            if actual
                .into_iter()
                .zip(expected)
                .any(|(a, b)| a.abs_diff(*b) > CHANNEL_LIMIT)
            {
                return None;
            }
        }
    }
    let point = PixelPoint::new(
        i32::try_from(LEFT.checked_add(WIDTH / 2)?).ok()?,
        i32::try_from(centre_y).ok()?,
    );
    body.contains(point).then_some(point)
}


#[cfg(test)]
mod tests;
