//! Application input vocabulary over the reusable xSAR QMP transport.
//!
//! The crate owns protocol handling and release-only recovery. This adapter
//! retains the five controllers' Draw spelling and current timing constants.

use std::{path::Path, time::Duration};

use xsar::qmp::QmpConfig;

use crate::{
    geometry::PixelPoint,
    parameters::{
        KEY_HOLD, MOUSE_HOLD, POINTER_SETTLE_DELAY, QMP_IO_TIMEOUT, QMP_SCREENDUMP_TIMEOUT,
    },
};

pub use xsar::qmp::{QmpError, QmpProbe};


/// Application client using shared transport with the existing input settings.
pub struct QmpClient {
    /// Shared protocol client; no independent transport copy exists here.
    transport: xsar::qmp::QmpClient,
}


impl QmpClient {
    /// Connect using the application's current per-I/O and input intervals.
    pub fn connect(path: &Path) -> Result<Self, QmpError> {
        let config = QmpConfig {
            io_timeout: QMP_IO_TIMEOUT,
            screendump_timeout: QMP_SCREENDUMP_TIMEOUT,
            pointer_settle_delay: POINTER_SETTLE_DELAY,
            mouse_hold: MOUSE_HOLD,
            key_hold: KEY_HOLD,
        };
        let transport = xsar::qmp::QmpClient::connect_with_config(path, config)?;
        Ok(Self { transport })
    }


    /// Freshly read QEMU running state and its active absolute pointer.
    pub fn probe(&mut self) -> Result<QmpProbe, QmpError> {
        self.transport.probe()
    }


    /// Ask QEMU to write a PNG to an absolute, UTF-8, NUL-free path.
    pub fn screendump_png(&mut self, output_path: &Path) -> Result<(), QmpError> {
        self.transport.screendump_png(output_path)
    }


    /// Move the guest pointer without a button transition or settle delay.
    pub fn move_pointer(
        &mut self,
        point: PixelPoint,
        width: u32,
        height: u32,
    ) -> Result<(), QmpError> {
        self.transport.move_pointer(point, width, height)
    }


    /// Send one normal-duration click without replaying uncertain input.
    pub fn click(
        &mut self,
        point: PixelPoint,
        width: u32,
        height: u32,
    ) -> Result<(), QmpError> {
        self.transport.click(point, width, height)
    }


    /// Send one click using the caller's existing control-specific hold.
    pub fn click_with_hold(
        &mut self,
        point: PixelPoint,
        width: u32,
        height: u32,
        hold: Duration,
    ) -> Result<(), QmpError> {
        self.transport.click_with_hold(point, width, height, hold)
    }


    /// Map the application Draw action to the crate's neutral qcode API.
    pub fn press_draw_key(&mut self) -> Result<(), QmpError> {
        self.transport.press_key("d", KEY_HOLD)
    }
}
