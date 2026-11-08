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


/// Explicit supported application keys, mapped to QEMU qcodes without parsing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SupportedKey {
    /// Existing Draw shortcut; other keys require an explicit application contract.
    Draw,
}


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
        crate::single_step_telemetry::operation(
            "probe",
            || serde_json::json!({}),
            || self.transport.probe(),
        )
    }


    /// Ask QEMU to write a PNG to an absolute, UTF-8, NUL-free path.
    pub fn screendump_png(&mut self, output_path: &Path) -> Result<(), QmpError> {
        crate::single_step_telemetry::operation(
            "screendump",
            || serde_json::json!({"path": output_path}),
            || self.transport.screendump_png(output_path),
        )
    }


    /// Move the guest pointer without a button transition or settle delay.
    pub fn move_pointer(
        &mut self,
        point: PixelPoint,
        width: u32,
        height: u32,
    ) -> Result<(), QmpError> {
        crate::single_step_telemetry::operation(
            "pointer",
            || serde_json::json!({"x": point.x, "y": point.y, "width": width, "height": height}),
            || self.transport.move_pointer(point, width, height),
        )
    }


    /// Deliver one explicit-target held drag, preserving neutral release-only recovery.
    pub fn drag<F: FnMut() -> bool>(
        &mut self,
        source: PixelPoint,
        destination: PixelPoint,
        width: u32,
        height: u32,
        permitted: F,
    ) -> Result<(), QmpError> {
        crate::single_step_telemetry::operation(
            "drag",
            || serde_json::json!({"source":format!("{source:?}"),"destination":format!("{destination:?}")}),
            || {
                self.transport
                    .drag(source, destination, width, height, permitted)
            },
        )
    }


    /// Send one normal-duration click without replaying uncertain input.
    pub fn click(&mut self, point: PixelPoint, width: u32, height: u32) -> Result<(), QmpError> {
        crate::single_step_telemetry::operation(
            "click",
            || serde_json::json!({"x": point.x, "y": point.y, "width": width, "height": height}),
            || self.transport.click(point, width, height),
        )
    }


    /// Send one click using the caller's existing control-specific hold.
    pub fn click_with_hold(
        &mut self,
        point: PixelPoint,
        width: u32,
        height: u32,
        hold: Duration,
    ) -> Result<(), QmpError> {
        crate::single_step_telemetry::operation(
            "click-with-hold",
            || serde_json::json!({"x": point.x, "y": point.y, "width": width, "height": height, "hold_ms": hold.as_millis()}),
            || self.transport.click_with_hold(point, width, height, hold),
        )
    }


    /// Map the application Draw action to the crate's neutral qcode API.
    pub fn press_draw_key(&mut self) -> Result<(), QmpError> {
        self.press_key(SupportedKey::Draw)
    }


    /// Send one supported key with the existing hold and release-only recovery.
    pub(crate) fn press_key(&mut self, key: SupportedKey) -> Result<(), QmpError> {
        let qcode = match key {
            SupportedKey::Draw => "d",
        };
        crate::single_step_telemetry::operation(
            "key",
            || serde_json::json!({"qcode": qcode, "hold_ms": KEY_HOLD.as_millis()}),
            || self.transport.press_key(qcode, KEY_HOLD),
        )
    }
}
