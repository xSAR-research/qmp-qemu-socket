//! Per-control search diagnostics spanning the first scan and any fresh retry.
//!
//! Each lookup reports pixel work separately from elapsed wall time. When the
//! first frame is already available, wall timing starts at lookup construction
//! before its first search, excluding that earlier capture. The interval includes
//! waits and recapture performed while the lookup is alive. Recaptures count
//! attempted fresh captures, including failed attempts. Its owner retains
//! capture, cancellation, retry and guest-input policy.

use std::time::{Duration, Instant};

use super::{WorkerEventSink, milliseconds, send_log};
use crate::{
    capture::CapturedFrame,
    geometry::PixelPoint,
    terminal_controls::{self, Control, profiles::{GameControlProfile, SearchSpec}},
};


/// Accumulate one expected control's scans and emit a summary on every exit path.
pub(super) struct ControlLookup<'a> {
    game: String,
    control: Control,
    spec: SearchSpec,
    events: &'a WorkerEventSink,
    started: Instant,
    searches: u64,
    recaptures: u64,
    pixels: u64,
    rows: u64,
    scan_total: Duration,
    outcome: &'static str,
}


impl<'a> ControlLookup<'a> {
    /// Start wall timing before the first search, or before its initial capture.
    pub(super) fn new(
        game: impl Into<String>,
        profile: &GameControlProfile,
        control: Control,
        events: &'a WorkerEventSink,
    ) -> Self {
        Self {
            game: game.into(), control, spec: profile.spec(control), events, started: Instant::now(),
            searches: 0, recaptures: 0, pixels: 0, rows: 0,
            scan_total: Duration::ZERO, outcome: "not-searched",
        }
    }


    /// Scan and log one already captured frame through the normal worker sink.
    pub(super) fn search(&mut self, frame: &CapturedFrame) -> Result<PixelPoint, String> {
        self.searches += 1;
        let bounds = self.spec.bounds;
        let right = bounds.width.checked_sub(1).and_then(|offset| bounds.x.checked_add(offset))
            .map_or_else(|| "invalid".to_owned(), |edge| edge.to_string());
        let bottom = bounds.height.checked_sub(1).and_then(|offset| bounds.y.checked_add(offset))
            .map_or_else(|| "invalid".to_owned(), |edge| edge.to_string());
        let prefix = format!(
            "{} {} search {}: X={}..={}, Y={}..={} inclusive; bottom-to-top rows, left-to-right pixels; {} consecutive RGB(0,0,0)",
            self.game, self.control.label(), self.searches,
            bounds.x, right, bounds.y, bottom, self.spec.run_length,
        );
        match terminal_controls::search(frame, self.spec) {
            Ok(result) => {
                self.pixels += result.pixels_examined;
                self.rows += u64::from(result.rows_examined);
                self.scan_total += result.scan_time;
                self.outcome = if result.point == PixelPoint::new(0, 0) { "missing" } else { "found" };
                send_log(self.events, format!(
                    "{prefix}; result=({}, {}); pixels={}; rows={}; scan_ms={:.3}.",
                    result.point.x, result.point.y, result.pixels_examined,
                    result.rows_examined, milliseconds(result.scan_time),
                ));
                Ok(result.point)
            }
            Err(error) => {
                self.outcome = "error";
                let message = format!("{prefix}; result=error: {error}; pixels=0; rows=0; scan_ms=0.000.");
                send_log(self.events, message);
                Err(format!("{} {} search failed: {error}", self.game, self.control.label()))
            }
        }
    }


    /// Record a fresh retry capture attempt immediately before invoking capture.
    /// Call after any cancellable wait; a failed capture still counts as attempted.
    ///
    /// Two alternative controls examined on the same fresh frame each record
    /// that frame once; their counts are per lookup, not additive capture totals.
    pub(super) fn note_recapture(&mut self) {
        self.recaptures += 1;
        self.outcome = "recapturing";
    }


    /// Describe caller-owned exits such as cancellation or a capture failure.
    pub(super) fn set_outcome(&mut self, outcome: &'static str) {
        self.outcome = outcome;
    }
}


impl Drop for ControlLookup<'_> {
    /// Include misses, errors and cancelled retries without duplicating summaries.
    fn drop(&mut self) {
        if self.searches == 0 && self.recaptures == 0 && self.outcome == "not-searched" {
            return;
        }
        let wall_time = self.started.elapsed();
        send_log(self.events, format!(
            "{} {} search summary: searches={}; recaptures={}; second_grab={}; pixels={}; rows={}; scan_total_ms={:.3}; wall_ms={:.3}; outcome={}.",
            self.game, self.control.label(), self.searches, self.recaptures,
            self.recaptures != 0, self.pixels, self.rows,
            milliseconds(self.scan_total), milliseconds(wall_time), self.outcome,
        ));
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{LatestFrameSlot, WorkerEvent};
    use crate::{capture::{PixelFormat, decode_png}, terminal_controls::profiles::SPIDER_CONTROLS};


    /// Diagnostics preserve work counts across a miss followed by a fresh match.
    #[test]
    fn logs_each_attempt_and_one_aggregate_with_recapture() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let missing = decode_png(include_bytes!("../../tests/fixtures/spider-SP15.png")).unwrap();
        let found = decode_png(include_bytes!("../../tests/fixtures/spider-level-up-505.png")).unwrap();
        {
            let mut lookup = ControlLookup::new("Spider", &SPIDER_CONTROLS, Control::Ok, &events);
            assert_eq!(lookup.search(&missing).unwrap(), PixelPoint::new(0, 0));
            lookup.note_recapture();
            assert_eq!(lookup.search(&found).unwrap(), PixelPoint::new(944, 771));
        }
        let logs: Vec<_> = receiver.try_iter().filter_map(|event| match event.event {
            WorkerEvent::Log(message) => Some(message),
            _ => None,
        }).collect();
        assert_eq!(logs.len(), 3);
        assert!(logs[0].contains("Spider OK search 1: X=930..=954, Y=700..=850"));
        assert!(logs[0].contains("result=(0, 0); pixels=3775; rows=151; scan_ms="));
        assert!(logs[1].contains("result=(944, 771); pixels=1994; rows=80; scan_ms="));
        assert!(logs[2].contains("searches=2; recaptures=1; second_grab=true; pixels=5769; rows=231;"));
        assert!(logs[2].contains("scan_total_ms="));
        assert!(logs[2].contains("wall_ms="));
        assert!(logs[2].ends_with("outcome=found."));
    }


    /// Invalid storage remains an error and produces no fabricated pixel work.
    #[test]
    fn failure_is_logged_and_summarised_without_a_second_capture() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let malformed = CapturedFrame {
            width: 1920, height: 1080, stride: 7680,
            format: PixelFormat::Rgba8, pixels: Vec::new(),
        };
        {
            let mut lookup = ControlLookup::new("Spider", &SPIDER_CONTROLS, Control::Play, &events);
            assert!(lookup.search(&malformed).is_err());
        }
        let logs: Vec<_> = receiver.try_iter().filter_map(|event| match event.event {
            WorkerEvent::Log(message) => Some(message),
            _ => None,
        }).collect();
        assert_eq!(logs.len(), 2);
        assert!(logs[0].contains("result=error:"));
        assert!(logs[1].contains("searches=1; recaptures=0; second_grab=false; pixels=0; rows=0;"));
        assert!(logs[1].ends_with("outcome=error."));
    }


    /// An initial capture failure still reports its wall interval before any scan.
    #[test]
    fn initial_capture_error_emits_zero_search_summary() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        {
            let mut lookup = ControlLookup::new("Spider", &SPIDER_CONTROLS, Control::Ok, &events);
            lookup.set_outcome("capture-error");
        }
        let logs: Vec<_> = receiver.try_iter().filter_map(|event| match event.event {
            WorkerEvent::Log(message) => Some(message),
            _ => None,
        }).collect();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("searches=0; recaptures=0; second_grab=false; pixels=0; rows=0;"));
        assert!(logs[0].contains("wall_ms="));
        assert!(logs[0].ends_with("outcome=capture-error."));
    }
}
