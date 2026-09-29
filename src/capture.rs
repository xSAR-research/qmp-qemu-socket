//! Decode QMP PNG bytes into checked in-memory pixel frames.
//!
//! The worker still obtains those bytes through QMP's temporary PNG file;
//! this module only performs the in-memory decoding stage.

use image::ImageFormat;
use thiserror::Error;


/// PNG decoding and decoded-layout failures.
#[derive(Debug, Error)]
pub enum CaptureError {
    /// The source bytes could not be decoded as a PNG.
    #[error("could not decode captured PNG: {0}")]
    Decode(#[from] image::ImageError),
    /// The decoded dimensions cannot be represented as a host byte buffer.
    #[error("decoded frame dimensions overflow host address space: {width}x{height}")]
    DimensionsOverflow {
        /// Decoded image width in pixels.
        width: u32,
        /// Decoded image height in pixels.
        height: u32,
    },
}


/// Supported byte ordering, with eight bits per channel and four bytes per pixel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    /// Red, green, blue and alpha, in that byte order.
    Rgba8,
}


/// Owned pixel buffer and layout metadata for one captured guest image.
#[derive(Clone, Debug)]
pub struct CapturedFrame {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Number of bytes between successive image rows.
    pub stride: usize,
    /// Channel order used by the pixel buffer.
    pub format: PixelFormat,
    /// Owned row-major image bytes, including any row padding.
    pub pixels: Vec<u8>,
}


impl CapturedFrame {


    /// Return the packed byte stride for this four-channel frame.
    pub fn minimum_stride(&self) -> usize {
        self.width as usize * 4
    }


    /// Check that the stride and buffer cover every declared image row.
    pub fn is_layout_valid(&self) -> bool {
        let required = self.stride.saturating_mul(self.height as usize);
        self.stride >= self.minimum_stride() && self.pixels.len() >= required
    }
}


/// Decodes one PNG into a tightly packed, top-to-bottom RGBA frame.
///
/// QEMU's `screendump` image contains guest framebuffer pixels only; it does
/// not carry host cursor metadata.
pub fn decode_png(bytes: &[u8]) -> Result<CapturedFrame, CaptureError> {
    let rgba = image::load_from_memory_with_format(bytes, ImageFormat::Png)?.into_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    let width_usize = width as usize;
    let height_usize = height as usize;
    let stride = width_usize
        .checked_mul(4)
        .ok_or(CaptureError::DimensionsOverflow { width, height })?;
    let expected_len = stride
        .checked_mul(height_usize)
        .ok_or(CaptureError::DimensionsOverflow { width, height })?;
    let pixels = rgba.into_raw();


    if pixels.len() != expected_len {
        return Err(CaptureError::DimensionsOverflow { width, height });
    }

    Ok(CapturedFrame {
        width,
        height,
        stride,
        format: PixelFormat::Rgba8,
        pixels,
    })
}


#[cfg(test)]
mod tests {
    //! Exact-byte PNG decoding and invalid-input checks.
    use super::*;


    /// Minimal lossless fixture containing one red pixel and one dark RGB pixel.
    const TWO_PIXEL_RGB_PNG: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x7b,
        0x40, 0xe8, 0xdd, 0x00, 0x00, 0x00, 0x0f, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0xf8,
        0xcf, 0xc0, 0xc0, 0xc8, 0xc4, 0x0c, 0x00, 0x06, 0x0b, 0x01, 0x06, 0x04, 0x0b, 0x99, 0x2d,
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];


    /// Verify exact RGBA bytes and dimensions after decoding.
    #[test]
    fn png_decode_returns_exact_rgba_layout() {
        let frame = decode_png(TWO_PIXEL_RGB_PNG).expect("decode fixture PNG");

        assert_eq!((frame.width, frame.height), (2, 1));
        assert_eq!(frame.stride, 8);
        assert_eq!(frame.format, PixelFormat::Rgba8);
        assert_eq!(frame.pixels, [255, 0, 0, 255, 1, 2, 3, 255]);
        assert!(frame.is_layout_valid());
    }


    /// Reject malformed source bytes without constructing a usable frame.
    #[test]
    fn png_decode_rejects_non_png_input() {
        let error = decode_png(b"not a PNG").expect_err("invalid image must fail");

        assert!(matches!(error, CaptureError::Decode(_)));
    }
}
