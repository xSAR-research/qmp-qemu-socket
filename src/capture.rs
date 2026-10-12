//! Shared PNG decoding, PNG encoding, RGBA frame and TCP capture client provided by xSAR.
//!
//! Captures are performed over guest-side TCP using ScreenCaptureClient.
//! Snapshot ownership, exact-byte saving and cleanup remain application policy.

pub use xsar::capture::{decode_png, encode_png};
pub use xsar::image_matching::{CapturedFrame, PixelFormat};
pub use xsar::tcp::ScreenCaptureClient;
