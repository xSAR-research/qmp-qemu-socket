//! Native settled-frame and labelled synthetic-drift geometry recovery checks.
//!
//! Uploaded screenshots preserve settled boards, not the rejected transition
//! pixels. In-memory exact-gold derivatives reproduce the logged offsets; they
//! are regression inputs and are never described as captured failure frames.

use super::*;
use crate::geometry::PixelPoint;
use std::collections::VecDeque;

/// Original native settled screenshots supplied after the transition stops.
const LEFT_FRAMES: [&[u8]; 2] = [
    include_bytes!("../../tests/fixtures/tripeaks-redeal-20261016/left-eight.png"),
    include_bytes!("../../tests/fixtures/tripeaks-redeal-20261016/left-ace.png"),
];

/// Native stock-first screenshot from the successful transition comparison.
const DRAW_FIRST: &[u8] = include_bytes!("../../tests/fixtures/tripeaks-redeal-20261016/draw-first.png");

/// One deterministic read-only capture adapter used by the production loop.
#[derive(Default)]
struct FakeIo {
    /// Native or explicitly simulated fresh observations, or acquisition errors.
    frames: VecDeque<Result<CapturedFrame, String>>,
    /// Number of acquisitions actually requested.
    captures: usize,
    /// Published diagnostic previews, newest last.
    published: Vec<CapturedFrame>,
    /// Delays requested by recovery.
    waits: Vec<Duration>,
    /// Recovery messages emitted by the production decision loop.
    messages: Vec<String>,
    /// Number of fresh frames included in action accounting.
    accounted: usize,
    /// Current STOP flag.
    cancelled: bool,
    /// Acquisition after which the fake user presses STOP.
    cancel_after_capture: Option<usize>,
    /// Delay during which the fake user presses STOP.
    cancel_during_wait: bool,
}

impl FakeIo {
    /// Supply immutable frames without any guest-input interface.
    fn new(frames: impl IntoIterator<Item = CapturedFrame>) -> Self {
        Self { frames: frames.into_iter().map(Ok).collect(), ..Self::default() }
    }
}

impl GeometryCaptureIo for FakeIo {
    fn cancelled(&self) -> bool { self.cancelled }

    fn capture(&mut self) -> Result<(CapturedFrame, CaptureTiming), String> {
        self.captures += 1;
        self.cancelled |= self.cancel_after_capture == Some(self.captures);
        self.frames.pop_front().expect("unexpected extra geometry capture")
            .map(|frame| (frame, CaptureTiming { captures: 1, ..CaptureTiming::default() }))
    }

    fn publish(&mut self, frame: CapturedFrame) { self.published.push(frame); }

    fn wait(&mut self, delay: Duration) -> bool {
        self.waits.push(delay);
        self.cancelled |= self.cancel_during_wait;
        !self.cancelled
    }

    fn record_capture(&mut self, timing: CaptureTiming) { self.accounted += timing.captures; }

    fn log(&mut self, message: String) { self.messages.push(message); }
}

/// Decode one unchanged native PNG through the application decoder.
fn native(png: &[u8]) -> CapturedFrame { decode_png(png).expect("native TriPeaks PNG") }

/// Identify the repeatedly affected lower-left slot from the immutable profile.
fn left_slot() -> crate::cards::CardRegionPixels {
    GameMode::TriPeaks.profile().tableau_cards[19]
}

/// Replace only the lower-left exterior HALO strip in an in-memory derivative.
fn clear_left_halo(frame: &mut CapturedFrame) {
    let regions = left_slot();
    for y in regions.card_bounds.bottom()..regions.halo_bounds.bottom() {
        for x in regions.halo_bounds.x..regions.halo_bounds.right() {
            let offset = y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 4].copy_from_slice(&[20, 110, 60, 255]);
        }
    }
}

/// Simulate the observed exact-gold anchor drift without altering source PNGs.
fn shifted_halo(png: &[u8], delta_x: i32) -> CapturedFrame {
    let mut frame = native(png);
    clear_left_halo(&mut frame);
    let regions = left_slot();
    let offset = GameMode::TriPeaks.profile().tableau_click_offset;
    let anchor = PixelPoint::new(regions.click_point.x - offset.x + delta_x, regions.click_point.y - offset.y);
    for y in anchor.y..anchor.y + 2 {
        for x in anchor.x..anchor.x + 80 {
            let index = y as usize * frame.stride + x as usize * 4;
            frame.pixels[index..index + 4].copy_from_slice(&[237, 208, 109, 255]);
        }
    }
    frame
}

/// Read one successful observation or expose its diagnostic in an assertion.
fn capture_ok(io: &mut FakeIo, recovery: &mut GeometryRecovery) -> CaptureSeries {
    recover_capture(io, &TableauScanState::initial(), recovery, Duration::from_millis(731))
        .unwrap_or_else(|failure| panic!("{}", failure.message))
}

/// Expect recovery to stop without requiring Debug on full captured images.
fn capture_failure(io: &mut FakeIo, recovery: &mut GeometryRecovery) -> CaptureFailure {
    match recover_capture(io, &TableauScanState::initial(), recovery, Duration::from_millis(731)) {
        Ok(_) => panic!("geometry recovery unexpectedly succeeded"),
        Err(failure) => failure,
    }
}

/// Both native lower-left HALOs still produce the canonical action unchanged.
#[test]
fn original_settled_left_frames_have_canonical_centres_and_no_retry() {
    for png in LEFT_FRAMES {
        let mut io = FakeIo::new([native(png)]);
        let mut recovery = GeometryRecovery::default();
        let series = capture_ok(&mut io, &mut recovery);
        let PredictedAction::Action(action) = series.observations[0].prediction else {
            panic!("native settled lower-left card did not produce an action");
        };
        assert!(matches!(action.target, ActionTarget::Tableau(target) if target.row == 4 && target.column == 2));
        assert_eq!(action.operation(), InputOperation::Click(PixelPoint::new(366, 533)));
        assert_eq!(io.captures, 1);
        assert!(io.waits.is_empty());
        assert_eq!(recovery.delayed_captures, 0);
    }
}

/// Logged +13 px and -1 px cases remain typed rejections before any retry.
#[test]
fn simulated_exact_halo_offsets_preserve_the_canonical_guard() {
    for (png, delta_x) in LEFT_FRAMES.into_iter().zip([13, -1]) {
        let frame = shifted_halo(png, delta_x);
        assert!(matches!(
            analyse_frame(&frame, &TableauScanState::initial()),
            Err(FrameAnalysisError::Tracker(TrackerError::ClickOffsetMismatch {
                actual_x, actual_y: 533, expected_x: 366, expected_y: 533,
            })) if actual_x == 366 + delta_x
        ));
    }
}

/// A later canonical native frame resumes normal planning from fresh pixels.
#[test]
fn shifted_first_left_halo_recovers_only_after_a_fresh_canonical_capture() {
    for (png, delta_x) in LEFT_FRAMES.into_iter().zip([13, -1]) {
        let mut io = FakeIo::new([shifted_halo(png, delta_x), native(png)]);
        let mut recovery = GeometryRecovery::default();
        let series = capture_ok(&mut io, &mut recovery);
        let PredictedAction::Action(action) = series.observations[0].prediction else {
            panic!("fresh native frame did not provide an action");
        };
        assert_eq!(action.operation(), InputOperation::Click(PixelPoint::new(366, 533)));
        assert_eq!(io.captures, 2);
        assert_eq!(io.accounted, 2);
        assert_eq!(io.waits, [Duration::from_millis(731)]);
        assert_eq!(io.published.len(), 2);
        assert_eq!(recovery.delayed_captures, 1);
    }
}

/// Four rejected frames spend three delayed captures and retain the last bytes.
#[test]
fn persistent_noncanonical_halo_stops_with_latest_diagnostic_and_fixed_budget() {
    let frames: Vec<_> = [13, -1, 13, -1].into_iter()
        .map(|offset| shifted_halo(LEFT_FRAMES[0], offset)).collect();
    let latest = frames[3].pixels.clone();
    let mut io = FakeIo::new(frames);
    let mut recovery = GeometryRecovery::default();
    let failure = capture_failure(&mut io, &mut recovery);
    assert_eq!(io.captures, 4);
    assert_eq!(io.accounted, 4);
    assert_eq!(io.waits, [Duration::from_millis(731); GEOMETRY_REOBSERVATIONS]);
    assert_eq!(recovery.delayed_captures, GEOMETRY_REOBSERVATIONS);
    let observation = failure.observation.expect("latest rejected pixels");
    assert_eq!(observation.frame.pixels, latest);
    assert_eq!(observation.prediction, PredictedAction::NoHighlight);
    assert_eq!(observation.observed_rows, None);
    assert!(failure.message.contains("after 3 delayed input-free captures"));
}

/// The last permitted delayed capture may still supply an exact canonical HALO.
#[test]
fn third_delayed_geometry_capture_can_recover_without_an_extra_observation() {
    let mut io = FakeIo::new([
        shifted_halo(LEFT_FRAMES[0], 13), shifted_halo(LEFT_FRAMES[0], -1),
        shifted_halo(LEFT_FRAMES[0], 13), native(LEFT_FRAMES[0]),
    ]);
    let mut recovery = GeometryRecovery::default();
    let series = capture_ok(&mut io, &mut recovery);
    assert!(matches!(series.observations[0].prediction,
        PredictedAction::Action(action) if action.operation() == InputOperation::Click(PixelPoint::new(366, 533))));
    assert_eq!(io.captures, 4);
    assert_eq!(io.waits.len(), 3);
    assert_eq!(recovery.delayed_captures, 3);
}

/// Acquisition failure after one mismatch retains pixels without using another delay.
#[test]
fn network_failure_during_geometry_recovery_stops_and_retains_previous_frame() {
    let rejected = shifted_halo(LEFT_FRAMES[0], 13);
    let expected_pixels = rejected.pixels.clone();
    let mut io = FakeIo::new([rejected]);
    io.frames.push_back(Err("TCP frame read failed".to_owned()));
    let mut recovery = GeometryRecovery::default();
    let failure = capture_failure(&mut io, &mut recovery);
    assert_eq!(failure.message, "TCP frame read failed");
    assert_eq!(failure.observation.expect("last acquired frame").frame.pixels, expected_pixels);
    assert_eq!(io.captures, 2);
    assert_eq!(io.accounted, 1);
    assert_eq!(io.waits.len(), 1);
    assert_eq!(recovery.delayed_captures, 1);
}

/// Missing HALOs between mismatches do not refresh the action-wide budget.
#[test]
fn no_halo_outer_round_preserves_geometry_budget_and_scan_state() {
    let mut no_halo = native(LEFT_FRAMES[0]);
    clear_left_halo(&mut no_halo);
    let state = TableauScanState::from_active_rows(RowMask::single_for(1, 4).unwrap()).unwrap();
    let original_state = state;
    let mut io = FakeIo::new([
        shifted_halo(LEFT_FRAMES[0], 13), no_halo,
        shifted_halo(LEFT_FRAMES[0], -1), shifted_halo(LEFT_FRAMES[0], 13),
        shifted_halo(LEFT_FRAMES[0], -1),
    ]);
    let mut recovery = GeometryRecovery::default();
    let first = recover_capture(&mut io, &state, &mut recovery, Duration::ZERO)
        .unwrap_or_else(|failure| panic!("{}", failure.message));
    assert_eq!(first.observations[0].prediction, PredictedAction::NoHighlight);
    assert_eq!(recovery.delayed_captures, 1);
    assert!(recover_capture(&mut io, &state, &mut recovery, Duration::ZERO).is_err());
    assert_eq!(recovery.delayed_captures, 3);
    assert_eq!(io.captures, 5);
    assert_eq!(state, original_state);
}

/// STOP before acquisition, during a wait or after capture cannot reach input.
#[test]
fn stop_checks_bound_capture_and_wait_progress() {
    let mut before = FakeIo { cancelled: true, ..FakeIo::default() };
    let failure = capture_failure(&mut before, &mut GeometryRecovery::default());
    assert_eq!(before.captures, 0);
    assert!(failure.observation.is_none());

    let mut during = FakeIo::new([shifted_halo(LEFT_FRAMES[0], 13), native(LEFT_FRAMES[0])]);
    during.cancel_during_wait = true;
    assert!(capture_failure(&mut during, &mut GeometryRecovery::default()).message.contains("STOP"));
    assert_eq!(during.captures, 1);
    assert_eq!(during.waits.len(), 1);

    let mut after = FakeIo::new([native(LEFT_FRAMES[0])]);
    after.cancel_after_capture = Some(1);
    let failure = capture_failure(&mut after, &mut GeometryRecovery::default());
    assert_eq!(after.captures, 1);
    assert!(after.waits.is_empty());
    assert_eq!(failure.observation.expect("STOP capture pixels").prediction, PredictedAction::NoHighlight);
}

/// Acquisition and invalid-frame errors stop immediately without geometry waits.
#[test]
fn unrelated_acquisition_and_layout_errors_are_not_geometry_retries() {
    let mut network = FakeIo { frames: [Err("TCP frame read failed".to_owned())].into(), ..FakeIo::default() };
    let failure = capture_failure(&mut network, &mut GeometryRecovery::default());
    assert_eq!(failure.message, "TCP frame read failed");
    assert_eq!(network.captures, 1);
    assert!(network.waits.is_empty());

    let mut frame = native(LEFT_FRAMES[0]);
    frame.pixels.truncate(1);
    let mut malformed = FakeIo::new([frame]);
    let failure = capture_failure(&mut malformed, &mut GeometryRecovery::default());
    assert!(failure.message.contains("invalid pixel layout"));
    assert_eq!(malformed.captures, 1);
    assert!(malformed.waits.is_empty());
}

/// A fresh canonical frame is found even with stale upper-row scan hints.
#[test]
fn stale_row_hint_cannot_hide_settled_lower_left_halo() {
    let state = TableauScanState::from_active_rows(RowMask::single_for(1, 4).unwrap()).unwrap();
    let mut io = FakeIo::new([shifted_halo(LEFT_FRAMES[1], -1), native(LEFT_FRAMES[1])]);
    let series = recover_capture(&mut io, &state, &mut GeometryRecovery::default(), Duration::ZERO)
        .unwrap_or_else(|failure| panic!("{}", failure.message));
    assert!(matches!(series.observations[0].prediction,
        PredictedAction::Action(action) if action.operation() == InputOperation::Click(PixelPoint::new(366, 533))));
    assert_eq!(state.active_rows().bits(), 1);
    assert_eq!(io.captures, 2);
}

/// Stock-first recommendations retain their original action and zero retries.
#[test]
fn native_draw_first_transition_does_not_enter_geometry_recovery() {
    let mut io = FakeIo::new([native(DRAW_FIRST)]);
    let mut recovery = GeometryRecovery::default();
    let series = capture_ok(&mut io, &mut recovery);
    assert!(matches!(series.observations[0].prediction,
        PredictedAction::Action(action) if action.operation() == InputOperation::PressDrawKey));
    assert_eq!(io.captures, 1);
    assert!(io.waits.is_empty());
    assert_eq!(recovery.delayed_captures, 0);
}
