use thiserror::Error;

pub const QMP_ABSOLUTE_MAX: u32 = 0x7fff;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PixelPoint {
    pub x: i32,
    pub y: i32,
}

impl PixelPoint {
    pub const fn new(x: i32, y: i32) -> Self {
        // Construct a pixel point from signed coordinates.
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl PixelRect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        // Construct a pixel rectangle from its origin and dimensions.
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn right(self) -> u32 {
        // Return the rectangle's exclusive right edge.
        self.x.saturating_add(self.width)
    }

    pub const fn bottom(self) -> u32 {
        // Return the rectangle's exclusive bottom edge.
        self.y.saturating_add(self.height)
    }

    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub const fn centre(self) -> PixelPoint {
        PixelPoint::new(
            self.x.saturating_add(self.width / 2) as i32,
            self.y.saturating_add(self.height / 2) as i32,
        )
    }

    pub fn contains(self, point: PixelPoint) -> bool {
        // Check whether a point lies within the rectangle's bounds.
        point.x >= self.x as i32
            && point.y >= self.y as i32
            && point.x < self.right() as i32
            && point.y < self.bottom() as i32
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QmpPoint {
    pub x: u32,
    pub y: u32,
}

impl QmpPoint {
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QmpRect {
    pub top_left: QmpPoint,
    /// QMP coordinate of the final pixel inside the half-open source rectangle.
    pub bottom_right_inclusive: QmpPoint,
}

impl QmpRect {
    pub const fn new(top_left: QmpPoint, bottom_right_inclusive: QmpPoint) -> Self {
        Self {
            top_left,
            bottom_right_inclusive,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GeometryError {
    #[error("axis extent must be non-zero")]
    InvalidExtent,
    #[error("pixel coordinate {pixel} is outside extent {extent}")]
    PixelOutsideExtent { pixel: u32, extent: u32 },
    #[error("pixel point ({x}, {y}) is outside frame {frame_width}x{frame_height}")]
    PixelPointOutsideFrame {
        x: i32,
        y: i32,
        frame_width: u32,
        frame_height: u32,
    },
    #[error("pixel rectangle must be non-empty")]
    EmptyRectangle,
    #[error("pixel rectangle overflows the u32 coordinate space")]
    RectangleOverflow,
    #[error(
        "pixel rectangle ({x}, {y}, {width}, {height}) is outside frame {frame_width}x{frame_height}"
    )]
    RectangleOutsideFrame {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        frame_width: u32,
        frame_height: u32,
    },
}

/// Convert one guest pixel coordinate to QEMU's absolute input range.
pub fn pixel_to_qmp_axis(pixel: u32, extent: u32) -> Result<u32, GeometryError> {
    // Scale a pixel coordinate into QEMU's absolute input range.
    if extent == 0 {
        return Err(GeometryError::InvalidExtent);
    }
    if pixel >= extent {
        return Err(GeometryError::PixelOutsideExtent { pixel, extent });
    }

    let numerator = u64::from(pixel) * u64::from(QMP_ABSOLUTE_MAX);
    Ok((numerator / u64::from(extent)) as u32)
}

/// Convert one guest pixel point to QEMU's absolute input coordinates.
pub fn pixel_point_to_qmp(
    point: PixelPoint,
    frame_width: u32,
    frame_height: u32,
) -> Result<QmpPoint, GeometryError> {
    if point.x < 0 || point.y < 0 || point.x as u32 >= frame_width || point.y as u32 >= frame_height
    {
        return Err(GeometryError::PixelPointOutsideFrame {
            x: point.x,
            y: point.y,
            frame_width,
            frame_height,
        });
    }

    Ok(QmpPoint::new(
        pixel_to_qmp_axis(point.x as u32, frame_width)?,
        pixel_to_qmp_axis(point.y as u32, frame_height)?,
    ))
}

/// Convert a half-open pixel rectangle to inclusive QMP corner coordinates.
pub fn pixel_rect_to_qmp(
    rect: PixelRect,
    frame_width: u32,
    frame_height: u32,
) -> Result<QmpRect, GeometryError> {
    if rect.is_empty() {
        return Err(GeometryError::EmptyRectangle);
    }

    let right = rect
        .x
        .checked_add(rect.width)
        .ok_or(GeometryError::RectangleOverflow)?;
    let bottom = rect
        .y
        .checked_add(rect.height)
        .ok_or(GeometryError::RectangleOverflow)?;

    if right > frame_width || bottom > frame_height {
        return Err(GeometryError::RectangleOutsideFrame {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            frame_width,
            frame_height,
        });
    }

    let top_left = QmpPoint::new(
        pixel_to_qmp_axis(rect.x, frame_width)?,
        pixel_to_qmp_axis(rect.y, frame_height)?,
    );
    let bottom_right_inclusive = QmpPoint::new(
        pixel_to_qmp_axis(right - 1, frame_width)?,
        pixel_to_qmp_axis(bottom - 1, frame_height)?,
    );

    Ok(QmpRect::new(top_left, bottom_right_inclusive))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_audited_calibration_vectors() {
        // QEMU's tablet range is scaled with floor(pixel * 32767 / extent).
        let cases = [
            (0, 1_920, 0),
            (1, 1_920, 17),
            (400, 1_920, 6_826),
            (960, 1_920, 16_383),
            (1_723, 1_920, 29_404),
            (1_919, 1_920, 32_749),
            (1, 1_080, 30),
            (533, 1_080, 16_171),
            (540, 1_080, 16_383),
            (800, 1_080, 24_271),
            (1_079, 1_080, 32_736),
        ];

        for (pixel, extent, expected) in cases {
            assert_eq!(pixel_to_qmp_axis(pixel, extent), Ok(expected));
        }
    }

    #[test]
    fn accepts_one_pixel_extent_and_uses_widened_arithmetic() {
        assert_eq!(pixel_to_qmp_axis(0, 1), Ok(0));
        assert_eq!(pixel_to_qmp_axis(u32::MAX - 1, u32::MAX), Ok(32_766));
    }

    #[test]
    fn rejects_zero_extent_and_coordinates_outside_extent() {
        assert_eq!(pixel_to_qmp_axis(0, 0), Err(GeometryError::InvalidExtent));
        assert_eq!(
            pixel_to_qmp_axis(1920, 1920),
            Err(GeometryError::PixelOutsideExtent {
                pixel: 1920,
                extent: 1920,
            })
        );
    }

    #[test]
    fn mapping_is_monotone_and_matches_the_integer_formula() {
        let mut previous = 0;
        for pixel in 0..1_920 {
            let actual = pixel_to_qmp_axis(pixel, 1_920).unwrap();
            let expected = (u64::from(pixel) * u64::from(QMP_ABSOLUTE_MAX) / 1_920) as u32;
            assert_eq!(actual, expected);
            assert!(actual >= previous);
            assert!(actual < QMP_ABSOLUTE_MAX);
            previous = actual;
        }
    }

    #[test]
    fn converts_half_open_rect_using_its_last_included_pixel() {
        let rect = PixelRect::new(1_655, 443, 137, 181);
        let converted = pixel_rect_to_qmp(rect, 1_920, 1_080).unwrap();

        assert_eq!(converted.top_left, QmpPoint::new(28_244, 13_440));
        assert_eq!(
            converted.bottom_right_inclusive,
            QmpPoint::new(30_565, 18_901)
        );
    }

    #[test]
    fn rejects_empty_overflowing_and_out_of_frame_rectangles() {
        assert_eq!(
            pixel_rect_to_qmp(PixelRect::new(10, 10, 0, 1), 1_920, 1_080),
            Err(GeometryError::EmptyRectangle)
        );
        assert_eq!(
            pixel_rect_to_qmp(PixelRect::new(u32::MAX, 10, 2, 1), 1_920, 1_080),
            Err(GeometryError::RectangleOverflow)
        );
        assert_eq!(
            pixel_rect_to_qmp(PixelRect::new(1_900, 1_070, 21, 10), 1_920, 1_080),
            Err(GeometryError::RectangleOutsideFrame {
                x: 1_900,
                y: 1_070,
                width: 21,
                height: 10,
                frame_width: 1_920,
                frame_height: 1_080,
            })
        );
    }

    #[test]
    fn rectangle_conversion_does_not_narrow_u32_coordinates_to_i32() {
        let rect = PixelRect::new(i32::MAX as u32 + 1, 0, 1, 1);
        let converted = pixel_rect_to_qmp(rect, u32::MAX, 1).unwrap();
        let expected_x = pixel_to_qmp_axis(rect.x, u32::MAX).unwrap();

        assert_eq!(converted.top_left, QmpPoint::new(expected_x, 0));
        assert_eq!(
            converted.bottom_right_inclusive,
            QmpPoint::new(expected_x, 0)
        );
    }
}
