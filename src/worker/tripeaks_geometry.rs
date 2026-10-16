//! Typed, input-free recovery from transient TriPeaks tableau geometry mismatches.
//!
//! The detector and canonical click guard remain authoritative. A rejected HALO
//! can only schedule a bounded fresh observation; it never authorises input.

use super::*;
use crate::tracker::TrackerError;

/// Maximum delayed captures after a typed mismatch within one guarded action.
pub(super) const GEOMETRY_REOBSERVATIONS: usize = 3;

/// Persistent geometry budget shared by all post-action observation rounds.
#[derive(Default)]
pub(super) struct GeometryRecovery {
    /// Delayed captures already reserved, including a subsequently cancelled wait.
    delayed_captures: usize,
}

/// Classification data that borrows no pixels and carries no input authority.
pub(super) struct FrameClassification {
    /// Current independently validated recommendation.
    prediction: PredictedAction,
    /// Positive row occupancy evidence from this immutable frame.
    observed_rows: Option<RowMask>,
    /// Independent calibrated gameplay-scene result.
    gameplay_scene: bool,
    /// Progress classification, which is never completion authority alone.
    game_progress: Option<GameProgress>,
}

impl FrameClassification {
    /// Attach classifications to the exact pixels from which they were derived.
    pub(super) fn into_observation(self, frame: CapturedFrame) -> FrameObservation {
        FrameObservation {
            frame,
            prediction: self.prediction,
            observed_rows: self.observed_rows,
            gameplay_scene: self.gameplay_scene,
            game_progress: self.game_progress,
        }
    }
}

/// Preserve the typed tableau error until the post-action recovery decision.
pub(super) enum FrameAnalysisError {
    /// Acquisition-independent calibration, scene or progress error.
    Other(String),
    /// Exact tracker error produced while selecting a gameplay recommendation.
    Tracker(TrackerError),
}

impl FrameAnalysisError {
    /// Keep existing public String diagnostics unchanged.
    pub(super) fn message(&self) -> String {
        match self {
            Self::Other(message) => message.clone(),
            Self::Tracker(error) => format!("frame analysis failed: {error}"),
        }
    }
}

/// Classify borrowed pixels while retaining typed tableau failures internally.
pub(super) fn analyse_frame(
    frame: &CapturedFrame,
    scan_state: &TableauScanState,
) -> Result<FrameClassification, FrameAnalysisError> {
    let profile = scan_state.mode().profile();
    if scan_state.mode().calibration_only() {
        if (frame.width, frame.height) != (profile.frame_width, profile.frame_height) {
            return Err(FrameAnalysisError::Other(format!(
                "calibration frame must be exactly {}x{}, got {}x{}",
                profile.frame_width, profile.frame_height, frame.width, frame.height,
            )));
        }
        if !frame.is_layout_valid() {
            return Err(FrameAnalysisError::Other(
                "calibration frame has an invalid pixel layout".to_owned(),
            ));
        }
        return Ok(FrameClassification {
            prediction: PredictedAction::CalibrationOnly { mode: scan_state.mode() },
            observed_rows: None,
            gameplay_scene: false,
            game_progress: None,
        });
    }

    let gameplay_scene = is_gameplay_scene_for_mode(frame, scan_state.mode())
        .map_err(|error| FrameAnalysisError::Other(format!("gameplay-scene analysis failed: {error}")))?;
    let game_progress = if matches!(scan_state.mode(), GameMode::Klondike | GameMode::FreeCell | GameMode::Spider) {
        None
    } else {
        Some(detect_game_progress_for_profile(frame, profile)
            .map_err(|error| FrameAnalysisError::Other(format!("Solver progress analysis failed: {error}")))?)
    };
    let (prediction, observed_rows) = if gameplay_scene {
        let analysis = analyse_frame_with_state(frame, scan_state)
            .map_err(FrameAnalysisError::Tracker)?;
        (analysis.prediction, analysis.observed_rows)
    } else {
        (PredictedAction::NoHighlight, None)
    };
    Ok(FrameClassification { prediction, observed_rows, gameplay_scene, game_progress })
}

/// Rejected pixels remain inspection-only and cannot supply a proposed action.
fn diagnostic_observation(frame: CapturedFrame) -> FrameObservation {
    FrameObservation {
        frame,
        prediction: PredictedAction::NoHighlight,
        observed_rows: None,
        gameplay_scene: false,
        game_progress: None,
    }
}

/// Capture failure plus the most recent inspection pixels, when available.
pub(super) struct CaptureFailure {
    /// Labelled reason the post-action observation stopped.
    pub(super) message: String,
    /// Latest frame, explicitly stripped of any recommendation authority.
    pub(super) observation: Option<Box<FrameObservation>>,
}

/// Read-only operations available to geometry recovery; no input method exists.
trait GeometryCaptureIo {
    /// Observe STOP before and after every wait and acquisition.
    fn cancelled(&self) -> bool;
    /// Acquire exactly one native frame using the selected existing backend.
    fn capture(&mut self) -> Result<(CapturedFrame, CaptureTiming), String>;
    /// Retain the newest pixels in the existing bounded diagnostic preview.
    fn publish(&mut self, frame: CapturedFrame);
    /// Perform and account for one configured cancellable observation delay.
    fn wait(&mut self, delay: Duration) -> bool;
    /// Account for every acquired frame, including rejected observations.
    fn record_capture(&mut self, timing: CaptureTiming);
    /// Emit a recovery decision without authorising input.
    fn log(&mut self, message: String);
}

/// Production adapter exposing only existing capture, wait and diagnostics.
struct QmpGeometryCapture<'a> {
    /// Existing QMP connection; acquisition may use the configured TCP backend.
    qmp: &'a mut QmpClient,
    /// Active QMP socket used by the legacy acquisition path.
    socket_path: &'a Path,
    /// Existing event and preview sink.
    event_tx: &'a WorkerEventSink,
    /// Shared STOP state.
    cancel_requested: &'a AtomicBool,
    /// Accounting for the current guarded action.
    profile: &'a mut ActionProfile,
}

impl GeometryCaptureIo for QmpGeometryCapture<'_> {
    fn cancelled(&self) -> bool { self.cancel_requested.load(Ordering::Acquire) }

    fn capture(&mut self) -> Result<(CapturedFrame, CaptureTiming), String> {
        capture_screen(self.qmp, self.socket_path, self.event_tx)
    }

    fn publish(&mut self, frame: CapturedFrame) {
        self.event_tx.publish_diagnostic_frame(frame);
    }

    fn wait(&mut self, delay: Duration) -> bool {
        match cancellable_wait(delay, self.cancel_requested) {
            Ok(waited) => { self.profile.intentional_wait += waited; true }
            Err(waited) => { self.profile.intentional_wait += waited; false }
        }
    }

    fn record_capture(&mut self, timing: CaptureTiming) { self.profile.capture.add_assign(timing); }

    fn log(&mut self, message: String) { send_log(self.event_tx, message); }
}

/// Capture TriPeaks post-action evidence with one action-wide geometry budget.
pub(super) fn capture_post_action_series(
    qmp: &mut QmpClient,
    scan_state: &TableauScanState,
    context: GuardedActionContext<'_>,
    profile: &mut ActionProfile,
    recovery: &mut GeometryRecovery,
    delay: Duration,
) -> Result<CaptureSeries, CaptureFailure> {
    let mut io = QmpGeometryCapture {
        qmp,
        socket_path: context.socket_path,
        event_tx: context.event_tx,
        cancel_requested: context.cancel_requested,
        profile,
    };
    recover_capture(&mut io, scan_state, recovery, delay)
}

/// Retry only typed TriPeaks click-offset mismatches, using fresh pixels each time.
fn recover_capture(
    io: &mut impl GeometryCaptureIo,
    scan_state: &TableauScanState,
    recovery: &mut GeometryRecovery,
    delay: Duration,
) -> Result<CaptureSeries, CaptureFailure> {
    let mut latest_rejected = None;
    loop {
        if io.cancelled() {
            return Err(CaptureFailure {
                message: "STOP was requested before the next TriPeaks geometry capture; no input was retried".to_owned(),
                observation: latest_rejected,
            });
        }
        let (frame, mut timing) = match io.capture() {
            Ok(captured) => captured,
            Err(error) => return Err(CaptureFailure { message: error, observation: latest_rejected }),
        };
        io.publish(frame.clone());
        if io.cancelled() {
            io.record_capture(timing);
            return Err(CaptureFailure {
                message: "STOP was requested after TriPeaks geometry capture; no input was retried".to_owned(),
                observation: Some(Box::new(diagnostic_observation(frame))),
            });
        }
        let detection_started = Instant::now();
        let analysis = analyse_frame(&frame, scan_state);
        timing.detection = detection_started.elapsed();
        io.record_capture(timing);
        if io.cancelled() {
            return Err(CaptureFailure {
                message: "STOP was requested after TriPeaks geometry analysis; no input was retried".to_owned(),
                observation: Some(Box::new(diagnostic_observation(frame))),
            });
        }
        match analysis {
            Ok(classification) => return Ok(CaptureSeries {
                observations: vec![classification.into_observation(frame)],
                // The adapter accounted for all captures, including rejected frames.
                timing: CaptureTiming::default(),
            }),
            Err(FrameAnalysisError::Tracker(error @ TrackerError::ClickOffsetMismatch { .. }))
                if scan_state.mode() == GameMode::TriPeaks =>
            {
                latest_rejected = Some(Box::new(diagnostic_observation(frame)));
                if recovery.delayed_captures >= GEOMETRY_REOBSERVATIONS {
                    return Err(CaptureFailure {
                        message: format!(
                            "TriPeaks tableau geometry remained non-canonical after {GEOMETRY_REOBSERVATIONS} delayed input-free captures: {error}. The latest frame is retained; no card, Draw or Solver input was replayed",
                        ),
                        observation: latest_rejected,
                    });
                }
                recovery.delayed_captures += 1;
                io.log(format!(
                    "TriPeaks transient tableau geometry rejected: {error}; geometry re-observation {}/{GEOMETRY_REOBSERVATIONS} for this action after {} ms. A fresh canonical HALO is required; no card, Draw or Solver input is authorised by this recovery.",
                    recovery.delayed_captures, delay.as_millis(),
                ));
                if io.cancelled() || !io.wait(delay) || io.cancelled() {
                    return Err(CaptureFailure {
                        message: "STOP was requested during TriPeaks input-free geometry recovery; no input was retried".to_owned(),
                        observation: latest_rejected,
                    });
                }
            }
            Err(error) => return Err(CaptureFailure {
                message: error.message(),
                observation: Some(Box::new(diagnostic_observation(frame))),
            }),
        }
    }
}

#[cfg(test)]
#[path = "tripeaks_geometry_tests.rs"]
mod tests;
