//! Bounded failure telemetry from existing captures; never captures or sends input.

use std::{
    fs::{self, DirBuilder, OpenOptions},
    io::{self, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};

use crate::{
    capture::{CapturedFrame, encode_png},
    parameters::{SNAPSHOT_MAX_PNG_BYTES, session_log_directory},
    strategy::ControllerContext,
};

use super::super::{CaptureTiming, StepRunSettings};

/// Process-local diagnostic identity, deliberately distinct from a UI request ID.
static RUN_SERIAL: AtomicU64 = AtomicU64::new(0);

/// Maximum encoded state and individual text field sizes.
const STATE_LIMIT: usize = 256 * 1024;

/// Maximum retained UTF-8 bytes in an externally supplied description.
const TEXT_LIMIT: usize = 16 * 1024;

/// Same-operation metadata; absence of a classification does not mean rejection.
#[derive(Clone)]
struct CaptureRecord {
    /// Monotonic ordinal within this worker invocation.
    ordinal: u64,
    /// Worker elapsed time at completion of this capture/classification.
    worker_elapsed_ms: Option<u128>,
    /// Validated TCP identity from this exact capture; null for legacy QMP.
    transport: Option<super::super::CaptureIdentity>,
    /// Existing classifier decision; null for raw diagnostic capture.
    classified_gameplay_scene: Option<bool>,
    /// Native width, height and byte stride.
    layout: Value,
    /// Measured capture stages, in milliseconds; no new timing gate.
    timings_ms: Value,
    /// Last input state at this capture, without claiming its physical effect.
    last_input: Option<InputRecord>,
}

/// Most recent attempted non-idempotent delivery and its acknowledgement state.
#[derive(Clone)]
struct InputRecord {
    /// Gameplay, Solver or terminal delivery description.
    operation: String,
    /// Capture whose evidence preceded this attempt, if available.
    evidence_capture_ordinal: Option<u64>,
    /// Delivery finished successfully; this never proves card movement.
    acknowledged: bool,
}

/// First classified rejected pixels, retained once and never replaced.
struct FirstRejected {
    /// Exact frame clone from that classifier invocation.
    frame: CapturedFrame,
    /// Metadata copied at the same boundary as those pixels.
    record: CaptureRecord,
}

/// Fixed-size run history: one first-rejected frame and latest metadata only.
pub(super) struct RunDiagnostics {
    /// Separate process-local diagnostic identity; null on counter exhaustion.
    run_id: Option<u64>,
    /// Monotonic successful capture count; overflow refuses only telemetry.
    ordinal: u64,
    /// First scene-false capture; never a raw diagnostic or failed acquisition.
    first: Option<FirstRejected>,
    /// Metadata for the existing controller-owned latest pixels.
    latest: Option<CaptureRecord>,
    /// Most recently attempted input, including uncertain delivery.
    last_input: Option<InputRecord>,
}

impl RunDiagnostics {
    /// Begin telemetry without filesystem access or changing run authority.
    pub(super) fn new() -> Self {
        Self {
            run_id: RUN_SERIAL
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
                .ok(),
            ordinal: 0,
            first: None,
            latest: None,
            last_input: None,
        }
    }

    /// Retain exact first-rejected pixels and metadata from one existing capture.
    pub(super) fn capture(
        &mut self,
        frame: &CapturedFrame,
        timing: CaptureTiming,
        scene: Option<bool>,
    ) {
        let Some(ordinal) = self.ordinal.checked_add(1) else {
            self.latest = None;
            return;
        };
        self.ordinal = ordinal;
        let record = CaptureRecord {
            ordinal,
            worker_elapsed_ms: super::super::worker_elapsed().map(|d| d.as_millis()),
            transport: timing.identity,
            classified_gameplay_scene: scene,
            layout: json!([frame.width, frame.height, frame.stride]),
            timings_ms: json!({ "reserve": timing.reserve.as_secs_f64()*1000.0,
                "acquisition": timing.screendump.as_secs_f64()*1000.0,
                "read": timing.file_read.as_secs_f64()*1000.0, "decode": timing.decode.as_secs_f64()*1000.0,
                "png_encode": timing.png_encode.as_secs_f64()*1000.0, "classification": timing.detection.as_secs_f64()*1000.0 }),
            last_input: self.last_input.clone(),
        };
        if scene == Some(false) && self.first.is_none() {
            self.first = Some(FirstRejected {
                frame: frame.clone(),
                record: record.clone(),
            });
        }
        self.latest = Some(record);
    }

    /// Mark an actual attempt immediately before the existing transport call.
    pub(super) fn attempt(&mut self, operation: String) {
        self.last_input = Some(InputRecord {
            operation: prefix(&operation).to_owned(),
            evidence_capture_ordinal: self.latest.as_ref().map(|r| r.ordinal),
            acknowledged: false,
        });
    }

    /// Mark only successful complete transport return, never semantic effect.
    pub(super) fn acknowledged(&mut self, success: bool) {
        if let Some(input) = self.last_input.as_mut() {
            input.acknowledged = success;
        }
    }

    /// Save supplied pixels once after failure, preserving partial files and errors.
    pub(super) fn persist(
        &self,
        latest: Option<&CapturedFrame>,
        context: Option<&ControllerContext>,
        settings: StepRunSettings,
        started: Option<Duration>,
        original_error: &str,
    ) -> Result<PathBuf, String> {
        self.persist_in(
            &session_log_directory(),
            latest,
            context,
            settings,
            started,
            original_error,
        )
    }

    /// Private destination seam for deterministic no-overwrite and failure tests.
    fn persist_in(
        &self,
        root: &Path,
        latest: Option<&CapturedFrame>,
        context: Option<&ControllerContext>,
        settings: StepRunSettings,
        started: Option<Duration>,
        original_error: &str,
    ) -> Result<PathBuf, String> {
        let run_id = self
            .run_id
            .ok_or("Klondike diagnostic identity exhausted")?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis();
        let mut destination = None;
        for attempt in 0..32 {
            let path = root.join(format!(
                "klondike-error-{stamp}-{}-{run_id}-{attempt}",
                std::process::id()
            ));
            match DirBuilder::new().mode(0o700).create(&path) {
                Ok(()) => {
                    destination = Some(path);
                    break;
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(format!("Klondike packet directory: {e}")),
            }
        }
        let path = destination.ok_or("Klondike packet directory collision budget exhausted")?;
        let mut warnings = Vec::new();
        let mut save = |name: &str, frame: &CapturedFrame| {
            if let Err(e) = original_png(frame).and_then(|bytes| {
                write_private(&path.join(name), &bytes, SNAPSHOT_MAX_PNG_BYTES)
                    .map_err(|e| e.to_string())
            }) {
                warnings.push(format!("{name}: {}", prefix(&e)));
            }
        };
        if let Some(first) = &self.first {
            save("first-rejected.png", &first.frame);
        }
        if let Some(frame) = latest {
            save("last-observed.png", frame);
        }
        let first = self.first.as_ref().map(|f| describe(&f.record, &f.frame));
        let last = latest
            .zip(self.latest.as_ref())
            .map(|(frame, record)| describe(record, frame));
        let state = json!({ "schema": 1, "application_version": env!("CARGO_PKG_VERSION"),
            "diagnostic_run_id": run_id, "identity_scope": "process-local diagnostic invocation, not UI request",
            "context": context.map(|c| prefix(&format!("{c:?}")).to_owned()),
            "settings": prefix(&format!("{settings:?}")), "run_started_worker_ms": started.map(|d| d.as_millis()),
            "original_error": prefix(original_error), "first_rejected": first, "last_observed": last,
            "last_input": self.last_input.as_ref().map(InputRecord::value), "persistence_warnings": warnings,
            "latest_metadata_unavailable": latest.is_some() && self.latest.is_none(),
            "evidence_contract": "existing exact decoded pixels; no new capture/input; ACK is delivery only; diagnostic counts grant no authority" });
        let mut bytes = serde_json::to_vec_pretty(&state)
            .map_err(|e| format!("Klondike state encoding: {e}"))?;
        bytes.push(b'\n');
        write_private(&path.join("state.json"), &bytes, STATE_LIMIT)
            .map_err(|e| format!("Klondike state write at {}: {e}", path.display()))?;
        fs::File::open(&path)
            .and_then(|f| f.sync_all())
            .map_err(|e| format!("Klondike packet sync at {}: {e}", path.display()))?;
        if warnings.is_empty() {
            Ok(path)
        } else {
            Err(format!(
                "Klondike packet retained at {}; {}",
                path.display(),
                warnings.join("; ")
            ))
        }
    }
}

/// Report all predicates on existing pixels, keeping the classifier decision distinct.
fn describe(record: &CaptureRecord, frame: &CapturedFrame) -> Value {
    let inspection = crate::klondike::scene_diagnostics::inspect(frame);
    match inspection {
        Ok(report) => json!({ "capture": record.value(),
            "diagnostic_agrees_with_classifier": record.classified_gameplay_scene.map(|v| v == report.diagnostic_scene),
            "scene_probes": report.value() }),
        Err(error) => {
            json!({ "capture": record.value(), "scene_inspection_error": prefix(&error.to_string()) })
        }
    }
}

/// Bound original storage and encoded bytes before exclusive publication.
fn original_png(frame: &CapturedFrame) -> Result<Vec<u8>, String> {
    let required = usize::try_from(frame.height)
        .ok()
        .and_then(|h| frame.stride.checked_mul(h));
    if frame.width == 0
        || frame.height == 0
        || !frame.is_layout_valid()
        || required != Some(frame.pixels.len())
        || frame.pixels.len() > SNAPSHOT_MAX_PNG_BYTES
    {
        return Err("Invalid or oversized original diagnostic frame".to_owned());
    }
    let bytes = encode_png(frame).map_err(|e| e.to_string())?;
    if bytes.len() > SNAPSHOT_MAX_PNG_BYTES {
        return Err("Oversized diagnostic PNG".to_owned());
    }
    Ok(bytes)
}

/// Exclusive private write; never follows an existing file or retries partial output.
fn write_private(path: &Path, bytes: &[u8], limit: usize) -> io::Result<()> {
    if bytes.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Diagnostic byte limit exceeded",
        ));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.flush()?;
    file.sync_all()
}

/// Truncate bounded descriptions only at UTF-8 character boundaries.
fn prefix(text: &str) -> &str {
    let mut end = text.len().min(TEXT_LIMIT);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

impl InputRecord {
    /// Bounded descriptive delivery evidence, never a verified-effect claim.
    fn value(&self) -> Value {
        json!({"operation": self.operation, "evidence_capture_ordinal": self.evidence_capture_ordinal, "acknowledged": self.acknowledged})
    }
}

impl CaptureRecord {
    /// Encode same-capture identities independently of aggregate profiling.
    fn value(&self) -> Value {
        json!({"ordinal": self.ordinal, "worker_elapsed_ms": self.worker_elapsed_ms,
            "transport": self.transport.map(|i| json!({"protocol_version": i.protocol_version,
                "session_id": i.session_id,"request_id": i.request_id,"frame_id": i.frame_id,
                "display_generation": i.display_generation,"reused_connection": i.reused_connection})),
            "classified_gameplay_scene": self.classified_gameplay_scene, "layout": self.layout,
            "timings_ms": self.timings_ms, "last_input": self.last_input.as_ref().map(InputRecord::value)})
    }
}

#[cfg(test)]
mod tests;
