//! Shared PNG decoding and RGBA frame representation provided by xSAR.
//!
//! The worker still obtains original bytes through a QMP temporary PNG file.
//! Snapshot ownership, exact-byte saving and cleanup remain application policy.

pub use xsar::capture::decode_png;
pub use xsar::image_matching::{CapturedFrame, PixelFormat};
