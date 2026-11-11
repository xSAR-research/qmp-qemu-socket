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


/// Describe native and checked QMP coordinates for logging; this never performs I/O.
fn point_details(point: PixelPoint, width: u32, height: u32) -> serde_json::Value {
    let axis = |pixel: i32, extent| {
        u32::try_from(pixel)
            .ok()
            .and_then(|pixel| xsar::geometry::pixel_to_qmp_axis(pixel, extent).ok())
    };
    serde_json::json!({"native":[point.x,point.y],"extent":[width,height],
        "absolute":[axis(point.x,width),axis(point.y,height)]})
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
        let transport = crate::single_step_telemetry::freecell_operation(
            "connect",
            || serde_json::json!({"socket":path}),
            || match crate::worker::qmp_wire::observer(path) {
                Some(observer) => {
                    xsar::qmp::QmpClient::connect_with_observer(path, config, observer)
                }
                None => xsar::qmp::QmpClient::connect_with_config(path, config),
            },
        )?;
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
        mut permitted: F,
    ) -> Result<(), QmpError> {
        let mut checkpoint = 0u8;
        let freecell = crate::single_step_telemetry::is_freecell();
        crate::single_step_telemetry::operation(
            "drag",
            || {
                if freecell {
                    serde_json::json!({"source":point_details(source,width,height),
                "destination":point_details(destination,width,height),
                "button":"left","intended_sequence":["source-move","down","held-destination-move","up"],
                "sequence_is":"planned; boundary records identify phases reached",
                "on_uncertainty":"release-only recovery, never replay; release is not rollback"})
                } else {
                    serde_json::json!({"source":format!("{source:?}"),"destination":format!("{destination:?}")})
                }
            },
            || {
                self.transport.drag(source, destination, width, height, || {
                    checkpoint = checkpoint.saturating_add(1);
                    let allowed = permitted();
                    crate::single_step_telemetry::record_freecell(|| {
                        serde_json::json!({
                            "event":"drag-boundary", "checkpoint":checkpoint,"permitted":allowed,
                            "reached":match checkpoint {
                                1=>"validated endpoints; before source movement",
                                2=>"source movement acknowledged; before button down",
                                3=>"button down sent; before held destination movement",
                                _=>"held destination movement sent; before button up"
                            },"effect_proved":false
                        })
                    });
                    allowed
                })
            },
        )
    }


    /// Run the explicitly requested Free Cell diagnostic gesture once.
    ///
    /// ABS sends source, down, the four application-planned native held targets
    /// and up, with the selected interval after source acknowledgement and each
    /// held phase. Targets are source minus 45px vertically, the native floor
    /// midpoint of the original source/effective destination, destination minus
    /// 45px vertically, then the exact destination. REL
    /// retains ten direction-only increments of two device counts per axis with
    /// two-second waits, and remains a motion probe rather than a route transfer
    /// or promise of target arrival. Neither mode adds a standalone click;
    /// delivery never proves a card effect.
    pub(crate) fn diagnostic_drag<F: FnMut() -> bool>(
        &mut self,
        source: PixelPoint,
        held_points: &[PixelPoint; 4],
        frame: (u32, u32),
        mode: crate::freecell_route::DragMode,
        timing: crate::freecell_route::DiagnosticDragTiming,
        mut permitted: F,
    ) -> Result<(), QmpError> {
        let (width, height) = frame;
        let [_, _, _, destination] = *held_points;
        let phase_wait_ms = match (mode, timing) {
            (_, crate::freecell_route::DiagnosticDragTiming::TwoSeconds)
            | (crate::freecell_route::DragMode::Relative, _) => 2_000,
            (_, crate::freecell_route::DiagnosticDragTiming::HalfSecond) => 500,
        };
        crate::single_step_telemetry::operation(
            "diagnostic-drag",
            || {
                serde_json::json!({
                    "source":point_details(source,width,height),
                    "destination":point_details(destination,width,height),"button":"left",
                    "motion_mode":format!("{mode:?}"),
                    "diagnostic_drag_timing":format!("{timing:?}"),
                    "planned_native_held_points":(mode==crate::freecell_route::DragMode::Absolute)
                        .then(||held_points.map(|point|[point.x,point.y])),
                    "intended_sequence":match mode {
                        crate::freecell_route::DragMode::Absolute => vec!["query-select-absolute-pointer",
                            "source-move-acknowledged","source-acknowledgement-wait","verify-motion-pointer","down",
                            "held-phase-wait","held-source-minus-45-y-move","held-phase-wait",
                            "held-native-midpoint-move","held-phase-wait",
                            "held-destination-minus-45-y-move","held-phase-wait",
                            "held-exact-destination-move","held-phase-wait","up"],
                        crate::freecell_route::DragMode::Relative => vec!["query-select-absolute-pointer",
                            "source-move-acknowledged","query-select-relative-pointer","wait-2000ms",
                            "down","ten-held-motion-updates-with-2000ms-gaps","wait-2000ms","up"]
                    },
                    "sequence_is":"planned; wire records identify phases reached and elapsed time",
                    "planned_held_motion_updates":match mode {
                        crate::freecell_route::DragMode::Absolute => 4,
                        crate::freecell_route::DragMode::Relative => 10
                    },
                    "planned_input_requests":match mode {
                        crate::freecell_route::DragMode::Absolute => 7,
                        crate::freecell_route::DragMode::Relative => 13
                    },
                    "source_acknowledgement_minimum_wait_ms":phase_wait_ms,
                    "held_phase_minimum_wait_ms":phase_wait_ms,
                    "motion_contract":match mode {
                        crate::freecell_route::DragMode::Absolute => "source minus 45 native pixels vertically, floor midpoint of original source and effective destination on each native axis, destination minus 45 native pixels vertically, exact destination; six selected waits from source acknowledgement through release",
                        crate::freecell_route::DragMode::Relative => "ten signum-times-two device-count increments; target arrival not claimed"
                    },
                    "route_effect_candidate":mode==crate::freecell_route::DragMode::Absolute,
                    "standalone_click":false,"effect_proved":false,
                    "on_uncertainty":"release recovery only; never repeat the transfer"
                })
            },
            || {
                let mut guard = || {
                    let allowed = permitted();
                    if !allowed {
                        crate::single_step_telemetry::record_freecell(|| {
                            serde_json::json!({
                                "event":"diagnostic-drag-authority-revoked","permitted":false,
                                "effect_proved":false
                            })
                        });
                    }
                    allowed
                };
                match mode {
                    crate::freecell_route::DragMode::Absolute => {
                        self.transport.diagnostic_drag_path_with_timing(
                            source,
                            held_points,
                            frame,
                            timing,
                            &mut guard,
                        )
                    }
                    crate::freecell_route::DragMode::Relative => self
                        .transport
                        .diagnostic_relative_drag(source, destination, width, height, &mut guard),
                }
            },
        )
    }


    /// Send one normal-duration click without replaying uncertain input.
    pub fn click(&mut self, point: PixelPoint, width: u32, height: u32) -> Result<(), QmpError> {
        let freecell = crate::single_step_telemetry::is_freecell();
        crate::single_step_telemetry::operation(
            "click",
            || {
                if freecell {
                    serde_json::json!({"point":point_details(point,width,height),"button":"left",
                "intended_sequence":["pointer","down","up"]})
                } else {
                    serde_json::json!({"x":point.x,"y":point.y,"width":width,"height":height})
                }
            },
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
        let freecell = crate::single_step_telemetry::is_freecell();
        crate::single_step_telemetry::operation(
            "click-with-hold",
            || {
                if freecell {
                    serde_json::json!({"point":point_details(point,width,height),"button":"left",
                "intended_sequence":["pointer","down","up"],"hold_ms":hold.as_millis()})
                } else {
                    serde_json::json!({"x":point.x,"y":point.y,"width":width,"height":height,"hold_ms":hold.as_millis()})
                }
            },
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
