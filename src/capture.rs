use std::{fs, path::Path};

use image::ImageFormat;
use thiserror::Error;

use crate::geometry::PixelPoint;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("could not read captured frame: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not decode captured PNG: {0}")]
    Decode(#[from] image::ImageError),
    #[error("decoded frame dimensions overflow host address space: {width}x{height}")]
    DimensionsOverflow { width: u32, height: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    Rgba8,
    Bgra8,
}

#[derive(Clone, Debug)]
pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub format: PixelFormat,
    pub pixels: Vec<u8>,
    /// Portal cursor metadata when available. This is the host cursor relative
    /// to the selected stream, not the Windows guest cursor moved through QMP.
    pub cursor: Option<PixelPoint>,
}

impl CapturedFrame {
    pub fn minimum_stride(&self) -> usize {
        // Return the minimum byte stride for a four-channel frame.
        self.width as usize * 4
    }

    pub fn is_layout_valid(&self) -> bool {
        // Check that the stride and pixel buffer cover the declared frame.
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
        cursor: None,
    })
}

/// Reads and decodes one PNG captured by QEMU.
pub fn read_png_frame(path: &Path) -> Result<CapturedFrame, CaptureError> {
    decode_png(&fs::read(path)?)
}

/// Boundary for the future XDG Desktop Portal/PipeWire implementation.
///
/// The initial milestone deliberately keeps capture behind an interface so UI,
/// detection and QMP work can be tested independently.
pub trait FrameSource: Send {
    type Error: std::error::Error + Send + Sync + 'static;

    fn start(&mut self) -> Result<(), Self::Error>;
    fn latest_frame(&mut self) -> Result<Option<CapturedFrame>, Self::Error>;
    fn stop(&mut self) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_PIXEL_RGB_PNG: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x7b,
        0x40, 0xe8, 0xdd, 0x00, 0x00, 0x00, 0x0f, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0xf8,
        0xcf, 0xc0, 0xc0, 0xc8, 0xc4, 0x0c, 0x00, 0x06, 0x0b, 0x01, 0x06, 0x04, 0x0b, 0x99, 0x2d,
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn png_decode_returns_exact_rgba_layout() {
        let frame = decode_png(TWO_PIXEL_RGB_PNG).expect("decode fixture PNG");

        assert_eq!((frame.width, frame.height), (2, 1));
        assert_eq!(frame.stride, 8);
        assert_eq!(frame.format, PixelFormat::Rgba8);
        assert_eq!(frame.pixels, [255, 0, 0, 255, 1, 2, 3, 255]);
        assert_eq!(frame.cursor, None);
        assert!(frame.is_layout_valid());
    }

    #[test]
    fn png_decode_rejects_non_png_input() {
        let error = decode_png(b"not a PNG").expect_err("invalid image must fail");

        assert!(matches!(error, CaptureError::Decode(_)));
    }
}
