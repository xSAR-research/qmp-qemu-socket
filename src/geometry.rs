//! Guest-pixel and QMP coordinate types provided by xSAR.
//!
//! The shared mapping remains floor(pixel * 32767 / extent), with checked
//! widened arithmetic. Game geometry and calibrated regions remain local.

pub use xsar::geometry::{
    PixelPoint, PixelRect, pixel_point_to_qmp, pixel_rect_to_qmp,
};


#[cfg(test)]
pub use xsar::geometry::{GeometryError, QmpPoint};
