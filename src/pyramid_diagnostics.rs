//! Bounded private failure packets from evidence already owned by a Pyramid worker.
//!
//! This module neither captures frames nor sends input. Saved source documents are
//! inspection material, never acquisition receipts or permission to resume a run.
//! Filesystem operations are synchronous and belong on the worker thread. Failed
//! writes retain their partial files and are reported to the caller without retry.

use std::{
    fs::{self, DirBuilder, OpenOptions},
    io::{self, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{Map, Value, json};

use crate::{
    capture::{CapturedFrame, encode_png},
    parameters::{SNAPSHOT_MAX_PNG_BYTES, session_log_directory},
    strategy::ControllerContext,
};


/// Maximum state-document size, including its final newline, in bytes.
const MAX_STATE_BYTES: usize = 256 * 1024;


/// Maximum plaintext log size in bytes.
const MAX_LOG_BYTES: usize = 128 * 1024;


/// Shared complete-deal document limit; saved bytes are not reinterpreted here.
const MAX_SOURCE_BYTES: usize = xsar::pyramid_json::MAX_DOCUMENT_BYTES;


/// Maximum optional structured details retained in one packet, in encoded bytes.
const MAX_DETAILS_BYTES: usize = 64 * 1024;


/// Maximum structured-detail depth before serialization, excluding the root.
const MAX_DETAILS_DEPTH: usize = 32;


/// Maximum structured-detail nodes visited before refusing the optional detail.
const MAX_DETAILS_NODES: usize = 8_192;


/// Exclusive directory attempts before returning a visible collision failure.
const NAME_ATTEMPTS: usize = 64;


/// Maximum retained error text in one normal packet, before JSON escaping.
const MAX_ERROR_TEXT_BYTES: usize = 16 * 1024;


/// Maximum retained context/path text before JSON escaping.
const MAX_CONTEXT_TEXT_BYTES: usize = 8 * 1024;


/// Maximum number of persistence warnings; artifact count is independently fixed.
const MAX_WARNINGS: usize = 16;


/// Maximum UTF-8 bytes retained for one persistence warning.
const MAX_WARNING_BYTES: usize = 1_024;


/// Process-local checked sequence; exhaustion refuses a new packet permanently.
static PACKET_SERIAL: AtomicU64 = AtomicU64::new(0);


/// Internal file writer, replaceable only by unit-test fault injection.
type ArtifactWriter = fn(&Path, &[u8], usize) -> io::Result<()>;


/// Pyramid operation that supplied the retained failure evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PyramidPhase {
    /// Bounded acquisition and original-board restoration.
    Reconnaissance,
    /// Offline calculation or independent replay of a route.
    Planning,
    /// Single-use execution with observed whole-action effects.
    Execution,
}


impl PyramidPhase {
    /// Return the stable lowercase phase used in packet metadata and text.
    const fn label(self) -> &'static str {
        match self {
            Self::Reconnaissance => "reconnaissance",
            Self::Planning => "planning",
            Self::Execution => "execution",
        }
    }
}


/// Failure state for the UI, independent of whether diagnostic persistence succeeds.
#[derive(Clone, Debug)]
pub struct PyramidDiagnostic {
    /// Exact request identity; diagnostic files grant no input authority.
    pub context: ControllerContext,
    /// Operation that failed or was interrupted.
    pub phase: PyramidPhase,
    /// Original request/run identity, when the caller has one.
    pub run_id: Option<u64>,
    /// Advisory local UI error correlation; never grants authority.
    pub ui_failure_id: Option<u64>,
    /// Unmodified caller error, including when its persisted copy is bounded.
    pub error: String,
    /// Reserved packet directory, retained even after a partial write failure.
    pub directory: Option<PathBuf>,
    /// Persistence failures or explicit omissions that must remain visible.
    pub persistence_error: Option<String>,
}


/// Borrowed evidence supplied by the failing worker; no fresh capture is requested.
pub(crate) struct FailureEvidence<'a> {
    /// Exact request identity at failure.
    pub context: &'a ControllerContext,
    /// Operation that failed.
    pub phase: PyramidPhase,
    /// Original request/run identity, when available.
    pub run_id: Option<u64>,
    /// Original failure, retained unchanged in the returned UI state.
    pub error: &'a str,
    /// Caller-owned counters, first mismatch and other inspection-only state.
    pub details: Value,
    /// Exact original source bytes, retained only within the shared document cap.
    pub source_document: Option<&'a [u8]>,
    /// First unmatched original frame, when already retained by the worker.
    pub first_frame: Option<&'a CapturedFrame>,
    /// Last original observed frame, when already retained by the worker.
    pub latest_frame: Option<&'a CapturedFrame>,
}


/// Persist one bounded private packet without acquiring evidence or retrying input.
///
/// Uses the existing session-log directory policy. New directories request mode
/// 0700 and files request mode 0600; the process umask may tighten those modes.
/// Existing names are never overwritten. Returns the original failure even when
/// the clock, reservation, encoding or any write fails. Filesystem calls remain
/// cooperative and may block; this function must run on the worker thread.
pub(crate) fn persist_failure(evidence: FailureEvidence<'_>) -> PyramidDiagnostic {
    let ui_failure_id = failure_correlation_id(&evidence);
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(timestamp) => persist_in(
            evidence,
            &session_log_directory(),
            timestamp.as_millis(),
            std::process::id(),
            &PACKET_SERIAL,
        ),
        Err(error) => diagnostic(
            &evidence,
            ui_failure_id,
            None,
            Some(format!(
                "Failure packet clock is before the Unix epoch: {error}"
            )),
        ),
    }
}


/// Reserve beneath an explicit parent so tests never mutate environment settings.
///
/// All reservation failures are returned as UI persistence errors with no packet
/// path. The serial advances on collisions and is never reset or wrapped.
fn persist_in(
    evidence: FailureEvidence<'_>,
    parent: &Path,
    epoch_millis: u128,
    process_id: u32,
    serial: &AtomicU64,
) -> PyramidDiagnostic {
    let ui_failure_id = failure_correlation_id(&evidence);
    match reserve_directory(parent, epoch_millis, process_id, serial) {
        Ok(directory) => persist_packet(evidence, directory, write_private),
        Err(error) => diagnostic(&evidence, ui_failure_id, None, Some(error.to_string())),
    }
}


/// Exclusively reserve a private packet directory without replacing any entry.
///
/// # Errors
/// Returns parent/creation errors, checked serial exhaustion, or 64 collisions.
fn reserve_directory(
    parent: &Path,
    epoch_millis: u128,
    process_id: u32,
    serial: &AtomicU64,
) -> io::Result<PathBuf> {
    fs::create_dir_all(parent)?;
    for _ in 0..NAME_ATTEMPTS {
        let number = serial
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| io::Error::other("Failure packet serial exhausted"))?;
        let directory = parent.join(format!(
            "pyramid-error-{epoch_millis}-{process_id}-{number}"
        ));
        match DirBuilder::new().mode(0o700).create(&directory) {
            Ok(()) => return Ok(directory),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "Failure packet could not reserve a unique directory after 64 collisions",
    ))
}


/// Save available artifacts once, then describe their outcomes in bounded metadata.
///
/// A failed artifact does not prevent independent remaining evidence from being
/// saved. Metadata-write failures can only be reported in the returned UI state;
/// an already created file is never rewritten to add a later warning.
fn persist_packet(
    mut evidence: FailureEvidence<'_>,
    directory: PathBuf,
    writer: ArtifactWriter,
) -> PyramidDiagnostic {
    let ui_failure_id = failure_correlation_id(&evidence);
    let mut warnings = Vec::new();
    let mut artifacts = Map::new();
    if let Some(bytes) = evidence.source_document {
        let (record, _) = record_bytes(
            &directory,
            "deal.json",
            bytes,
            MAX_SOURCE_BYTES,
            writer,
            &mut warnings,
        );
        artifacts.insert("source_document".to_owned(), record);
    }
    let mut first_png = None;
    let mut first_written = false;
    if let Some(frame) = evidence.first_frame {
        match bounded_png(frame) {
            Ok(bytes) => {
                let (record, written) = record_bytes(
                    &directory,
                    "first-unmatched.png",
                    &bytes,
                    SNAPSHOT_MAX_PNG_BYTES,
                    writer,
                    &mut warnings,
                );
                artifacts.insert("first_unmatched".to_owned(), record);
                first_written = written;
                first_png = Some(bytes);
            }
            Err(error) => {
                warn(
                    &mut warnings,
                    format!("first-unmatched.png omitted: {error}"),
                );
                artifacts.insert("first_unmatched".to_owned(), refused("first-unmatched.png"));
            }
        }
    }
    if let Some(frame) = evidence.latest_frame {
        match bounded_png(frame) {
            Ok(bytes) if first_written && first_png.as_deref() == Some(bytes.as_slice()) => {
                artifacts.insert(
                    "last_observed".to_owned(),
                    json!({
                        "file": "first-unmatched.png",
                        "status": "shared",
                        "same_as": "first_unmatched",
                        "reason": "Encoded original PNG bytes are identical",
                    }),
                );
            }
            Ok(bytes) => {
                let (record, _) = record_bytes(
                    &directory,
                    "last-observed.png",
                    &bytes,
                    SNAPSHOT_MAX_PNG_BYTES,
                    writer,
                    &mut warnings,
                );
                artifacts.insert("last_observed".to_owned(), record);
            }
            Err(error) => {
                warn(&mut warnings, format!("last-observed.png omitted: {error}"));
                artifacts.insert("last_observed".to_owned(), refused("last-observed.png"));
            }
        }
    }
    drop(first_png);
    let details = bounded_details(std::mem::take(&mut evidence.details), &mut warnings);
    let error = retained_text(
        evidence.error,
        MAX_ERROR_TEXT_BYTES,
        "error text",
        &mut warnings,
    );
    let context_debug = retained_text(
        &format!("{:?}", evidence.context),
        MAX_CONTEXT_TEXT_BYTES,
        "context debug text",
        &mut warnings,
    );
    let socket_debug = retained_text(
        &format!("{:?}", evidence.context.socket_path),
        MAX_CONTEXT_TEXT_BYTES,
        "socket debug text",
        &mut warnings,
    );
    let socket_literal = evidence.context.socket_path.to_str().map(|path| {
        retained_text(
            path,
            MAX_CONTEXT_TEXT_BYTES,
            "socket literal text",
            &mut warnings,
        )
    });
    let mut state = json!({
        "diagnostic_only": true,
        "input_authority": false,
        "resume_authority": false,
        "source_document_role": "Inspection only; not a live acquisition receipt",
        "app_version": env!("CARGO_PKG_VERSION"),
        "phase": evidence.phase.label(),
        "run_id": evidence.run_id,
        "ui_failure_id": ui_failure_id,
        "error": error,
        "error_original_bytes": evidence.error.len(),
        "context": {
            "debug": context_debug,
            "socket_path": socket_literal,
            "socket_path_debug": socket_debug,
            "game_mode": format!("{:?}", evidence.context.game_mode),
            "strategy": format!("{:?}", evidence.context.strategy),
            "generation": evidence.context.generation,
        },
        "directory_debug": format!("{directory:?}"),
        "first_frame": frame_metadata(evidence.first_frame),
        "latest_frame": frame_metadata(evidence.latest_frame),
        "details": details,
        "artifacts": artifacts,
    });
    update_warnings(&mut state, &warnings);
    let log = render_log(&evidence, &state, &mut warnings);
    let (log_record, _) = record_bytes(
        &directory,
        "error.log",
        &log,
        MAX_LOG_BYTES,
        writer,
        &mut warnings,
    );
    if let Some(records) = state.get_mut("artifacts").and_then(Value::as_object_mut) {
        records.insert("error_log".to_owned(), log_record);
        records.insert(
            "error_state".to_owned(),
            json!({
                "file": "error-state.json", "status": "self-description",
            }),
        );
    }
    update_warnings(&mut state, &warnings);
    let metadata = state_bytes(state, &evidence, &mut warnings);
    let _ = record_bytes(
        &directory,
        "error-state.json",
        &metadata,
        MAX_STATE_BYTES,
        writer,
        &mut warnings,
    );
    let persistence_error = if warnings.is_empty() {
        None
    } else {
        Some(warnings.join("; "))
    };
    diagnostic(&evidence, ui_failure_id, Some(directory), persistence_error)
}


/// Read an optional app-supplied numeric UI correlation before details are moved.
///
/// Missing, negative and nonnumeric values produce no correlation. The token is
/// advisory presentation state and never replaces context or input checks.
fn failure_correlation_id(evidence: &FailureEvidence<'_>) -> Option<u64> {
    evidence
        .details
        .get("ui_failure_id")
        .and_then(Value::as_u64)
}


/// Retain the exact caller failure and identity independently of persistence.
fn diagnostic(
    evidence: &FailureEvidence<'_>,
    ui_failure_id: Option<u64>,
    directory: Option<PathBuf>,
    persistence_error: Option<String>,
) -> PyramidDiagnostic {
    PyramidDiagnostic {
        context: evidence.context.clone(),
        phase: evidence.phase,
        run_id: evidence.run_id,
        ui_failure_id,
        error: evidence.error.to_owned(),
        directory,
        persistence_error,
    }
}


/// Encode a checked bounded original frame without recolouring or resizing it.
///
/// # Errors
/// Refuses empty/invalid layouts, excess trailing storage, raw storage above the
/// snapshot byte cap, encoder errors, and encoded PNGs above that same cap.
fn bounded_png(frame: &CapturedFrame) -> Result<Vec<u8>, String> {
    let rows = usize::try_from(frame.height).map_err(|_| "Frame height overflows host storage")?;
    let required = frame
        .stride
        .checked_mul(rows)
        .ok_or("Frame row storage overflows")?;
    if frame.width == 0
        || frame.height == 0
        || !frame.is_layout_valid()
        || required != frame.pixels.len()
        || required > SNAPSHOT_MAX_PNG_BYTES
    {
        return Err(format!(
            "Frame layout is invalid or exceeds the {SNAPSHOT_MAX_PNG_BYTES}-byte raw evidence limit"
        ));
    }
    let bytes =
        encode_png(frame).map_err(|error| format!("Original PNG encoding failed: {error}"))?;
    if bytes.len() > SNAPSHOT_MAX_PNG_BYTES {
        return Err("Encoded original PNG exceeds the snapshot byte limit".to_owned());
    }
    Ok(bytes)
}


/// Describe supplied original storage without copying pixel bytes into metadata.
fn frame_metadata(frame: Option<&CapturedFrame>) -> Value {
    frame.map_or(Value::Null, |frame| {
        json!({
            "width": frame.width, "height": frame.height, "stride": frame.stride,
            "format": format!("{:?}", frame.format), "storage_bytes": frame.pixels.len(),
        })
    })
}


/// Record a refused artifact without claiming that its destination was written.
fn refused(name: &str) -> Value {
    json!({ "file": name, "status": "omitted" })
}


/// Write one bounded artifact and describe a failure as potentially partial.
fn record_bytes(
    directory: &Path,
    name: &str,
    bytes: &[u8],
    limit: usize,
    writer: ArtifactWriter,
    warnings: &mut Vec<String>,
) -> (Value, bool) {
    if bytes.len() > limit {
        warn(
            warnings,
            format!("{name} omitted: {} bytes exceeds {limit}", bytes.len()),
        );
        return (refused(name), false);
    }
    match writer(&directory.join(name), bytes, limit) {
        Ok(()) => (
            json!({ "file": name, "status": "written", "bytes": bytes.len() }),
            true,
        ),
        Err(error) => {
            warn(
                warnings,
                format!("{name} write failed; retained path may be partial: {error}"),
            );
            (
                json!({ "file": name, "status": "write-failed", "may_be_partial": true }),
                false,
            )
        }
    }
}


/// Exclusively create, write and sync a private artifact, retaining partial data.
///
/// # Errors
/// Refuses excessive bytes and existing entries, including symlinks. Propagates
/// creation, write, flush and file-sync errors without deleting or rewriting data.
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


/// Append one bounded persistence warning; the fixed artifact count limits totals.
fn warn(warnings: &mut Vec<String>, message: String) {
    if warnings.len() < MAX_WARNINGS {
        warnings.push(prefix(&message, MAX_WARNING_BYTES).to_owned());
    }
}


/// Retain a UTF-8 prefix without slicing inside a multibyte character.
fn prefix(text: &str, limit: usize) -> &str {
    let mut end = text.len().min(limit);
    while !text.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    text.get(..end).map_or("", |value| value)
}


/// Bound persisted text while keeping the original size and omission visible.
fn retained_text(text: &str, limit: usize, label: &str, warnings: &mut Vec<String>) -> String {
    if text.len() > limit {
        warn(
            warnings,
            format!(
                "{label} truncated from {} to at most {limit} bytes",
                text.len()
            ),
        );
    }
    prefix(text, limit).to_owned()
}


/// Byte-limited JSON writer; overflowing writes fail before extending its buffer.
struct LimitedBuffer {
    /// Bytes accepted so far, never exceeding the configured cap.
    bytes: Vec<u8>,
    /// Maximum accepted encoded bytes.
    limit: usize,
}


impl Write for LimitedBuffer {
    /// Accept a complete encoder chunk or return a bounded-write error.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Diagnostic byte limit exceeded",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }


    /// No external buffered resource exists to flush.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}


/// Encode JSON within a strict byte cap without first allocating its full text.
///
/// # Errors
/// Returns serialization or bounded-writer errors, preserving no output file.
fn encode_json(value: &Value, limit: usize) -> Result<Vec<u8>, serde_json::Error> {
    let mut buffer = LimitedBuffer {
        bytes: Vec::new(),
        limit,
    };
    serde_json::to_writer(&mut buffer, value)?;
    Ok(buffer.bytes)
}


/// Reject excessive optional detail before recursive JSON encoding.
fn details_shape_bounded(value: &Value) -> bool {
    let mut pending = vec![(0usize, value)];
    let mut visited = 0usize;
    while let Some((depth, item)) = pending.pop() {
        visited = visited.saturating_add(1);
        if depth > MAX_DETAILS_DEPTH || visited > MAX_DETAILS_NODES {
            return false;
        }
        let remaining = MAX_DETAILS_NODES.saturating_sub(visited.saturating_add(pending.len()));
        match item {
            Value::Array(values) if values.len() <= remaining => {
                pending.extend(values.iter().map(|value| (depth.saturating_add(1), value)));
            }
            Value::Object(values) if values.len() <= remaining => {
                pending.extend(
                    values
                        .values()
                        .map(|value| (depth.saturating_add(1), value)),
                );
            }
            Value::Array(_) | Value::Object(_) => return false,
            _ => {}
        }
    }
    true
}


/// Drop refused nested details iteratively rather than recursively walking depth.
fn discard_details(value: Value) {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            Value::Array(values) => pending.extend(values),
            Value::Object(values) => pending.extend(values.into_iter().map(|(_, value)| value)),
            _ => {}
        }
    }
}


/// Retain bounded structured detail or an explicit inspection-only omission.
fn bounded_details(value: Value, warnings: &mut Vec<String>) -> Value {
    if details_shape_bounded(&value) && encode_json(&value, MAX_DETAILS_BYTES).is_ok() {
        return value;
    }
    warn(
        warnings,
        "Structured details omitted: depth, node or 65536-byte limit exceeded".to_owned(),
    );
    discard_details(value);
    json!({ "omitted": true, "reason": "Structured detail exceeds diagnostic bounds" })
}


/// Refresh warnings known before the immutable state document is published.
fn update_warnings(state: &mut Value, warnings: &[String]) {
    if let Some(fields) = state.as_object_mut() {
        fields.insert("write_warnings".to_owned(), json!(warnings));
    }
}


/// Render bounded plaintext with important failure identity before optional JSON.
fn render_log(
    evidence: &FailureEvidence<'_>,
    state: &Value,
    warnings: &mut Vec<String>,
) -> Vec<u8> {
    let context = prefix(&format!("{:?}", evidence.context), MAX_CONTEXT_TEXT_BYTES).to_owned();
    let mut text = format!(
        "Pyramid failure packet\nApp version: {}\nPhase: {}\nRun ID: {:?}\nUI failure ID: {:?}\nContext: {}\nInspection only; no input or resume authority.\nError:\n{}\nState:\n",
        env!("CARGO_PKG_VERSION"),
        evidence.phase.label(),
        evidence.run_id,
        state.get("ui_failure_id").and_then(Value::as_u64),
        context,
        prefix(evidence.error, MAX_ERROR_TEXT_BYTES),
    );
    match encode_json(state, MAX_STATE_BYTES) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(json) => text.push_str(&json),
            Err(_) => warn(
                warnings,
                "Plaintext state omitted: invalid encoded UTF-8".to_owned(),
            ),
        },
        Err(_) => warn(
            warnings,
            "Plaintext optional state omitted: metadata byte limit exceeded".to_owned(),
        ),
    }
    text.push('\n');
    if text.len() > MAX_LOG_BYTES {
        const NOTE: &str =
            "\n[Plaintext detail truncated; inspect error-state.json and UI persistence error.]\n";
        let mut bounded = prefix(&text, MAX_LOG_BYTES.saturating_sub(NOTE.len())).to_owned();
        bounded.push_str(NOTE);
        warn(
            warnings,
            "error.log detail truncated at 131072 bytes".to_owned(),
        );
        return bounded.into_bytes();
    }
    text.into_bytes()
}


/// Serialize final metadata or retain a small explicit fallback with the failure.
fn state_bytes(
    mut state: Value,
    evidence: &FailureEvidence<'_>,
    warnings: &mut Vec<String>,
) -> Vec<u8> {
    update_warnings(&mut state, warnings);
    match encode_json(&state, MAX_STATE_BYTES.saturating_sub(1)) {
        Ok(mut bytes) => {
            bytes.push(b'\n');
            bytes
        }
        Err(_) => {
            warn(
                warnings,
                "error-state.json optional metadata omitted: 262144-byte limit exceeded".to_owned(),
            );
            let fallback = json!({
                "diagnostic_only": true, "input_authority": false, "resume_authority": false,
                "app_version": env!("CARGO_PKG_VERSION"), "phase": evidence.phase.label(),
                "run_id": evidence.run_id, "error": prefix(evidence.error, 4096),
                "ui_failure_id": state.get("ui_failure_id"),
                "error_original_bytes": evidence.error.len(),
                "context_debug": prefix(&format!("{:?}", evidence.context), 2048),
                "details_omitted": true, "write_warnings": warnings,
                "artifacts": state.get("artifacts"),
            });
            match encode_json(&fallback, MAX_STATE_BYTES.saturating_sub(1)) {
                Ok(mut bytes) => {
                    bytes.push(b'\n');
                    bytes
                }
                Err(_) => {
                    warn(
                        warnings,
                        "error-state.json fallback could not be serialized".to_owned(),
                    );
                    b"{\"diagnostic_only\":true,\"persistence_failed\":true}\n".to_vec()
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    //! Exact original evidence, private creation, refusal and partial-write tests.

    use std::os::unix::{ffi::OsStringExt, fs::PermissionsExt};

    use super::*;
    use crate::{
        capture::{PixelFormat, decode_png},
        game::GameMode,
        strategy::SolvingStrategy,
    };


    /// Own and remove only a uniquely reserved test directory.
    struct TestDirectory(PathBuf);


    impl TestDirectory {
        /// Reserve a test directory without changing HOME or any log configuration.
        fn new() -> Self {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("test clock");
            Self(
                reserve_directory(
                    &std::env::temp_dir(),
                    now.as_millis(),
                    std::process::id(),
                    &PACKET_SERIAL,
                )
                .expect("private test directory"),
            )
        }
    }


    impl Drop for TestDirectory {
        /// Remove only the private directory owned by this test instance.
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }


    /// Construct a fixed immutable identity with a literal non-Unicode socket name.
    fn context() -> ControllerContext {
        ControllerContext::new(
            PathBuf::from(std::ffi::OsString::from_vec(b"/tmp/qmp-\xff.sock".to_vec())),
            GameMode::Pyramid,
            SolvingStrategy::ShortestPath,
            17,
        )
    }


    /// Supply a padded RGBA original with distinct alpha and non-image padding.
    fn frame() -> CapturedFrame {
        CapturedFrame {
            width: 2,
            height: 2,
            stride: 12,
            format: PixelFormat::Rgba8,
            pixels: vec![
                1, 2, 3, 4, 5, 6, 7, 8, 90, 91, 92, 93, 11, 12, 13, 14, 15, 16, 17, 18, 94, 95, 96,
                97,
            ],
        }
    }


    /// Construct test evidence from borrowed caller-owned originals.
    fn evidence<'a>(
        context: &'a ControllerContext,
        source: Option<&'a [u8]>,
    ) -> FailureEvidence<'a> {
        FailureEvidence {
            context,
            phase: PyramidPhase::Execution,
            run_id: Some(41),
            error: "Fresh pre-gesture position mismatch; no further input",
            details: json!({ "verified_actions": 10, "attempted": 12, "acknowledged": 12 }),
            source_document: source,
            first_frame: None,
            latest_frame: None,
        }
    }


    /// Parse one bounded packet's state document.
    fn state(directory: &Path) -> Value {
        serde_json::from_slice(&fs::read(directory.join("error-state.json")).expect("state file"))
            .expect("state JSON")
    }


    /// Original document bytes and all image channels survive a private deduplicated packet.
    #[test]
    fn exact_source_pixels_private_modes_and_duplicate_reference() {
        let parent = TestDirectory::new();
        let context = context();
        let source = b"{\n\"literal\":\"original bytes\"}\n\xff";
        let original = frame();
        let mut input = evidence(&context, Some(source));
        input.first_frame = Some(&original);
        input.latest_frame = Some(&original);
        let result = persist_in(input, &parent.0, 123, 7, &AtomicU64::new(0));
        assert!(
            result.persistence_error.is_none(),
            "{:?}",
            result.persistence_error
        );
        assert_eq!(result.context, context);
        assert_eq!(result.run_id, Some(41));
        assert_eq!(result.ui_failure_id, None);
        assert_eq!(result.phase, PyramidPhase::Execution);
        let directory = result.directory.expect("packet directory");
        assert_eq!(
            fs::read(directory.join("deal.json")).expect("source"),
            source
        );
        let png = fs::read(directory.join("first-unmatched.png")).expect("original PNG");
        let decoded = decode_png(&png).expect("saved native PNG");
        assert_eq!((decoded.width, decoded.height, decoded.stride), (2, 2, 8));
        assert_eq!(
            decoded.pixels,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 13, 14, 15, 16, 17, 18]
        );
        assert!(!directory.join("last-observed.png").exists());
        let metadata = state(&directory);
        assert_eq!(
            metadata["artifacts"]["last_observed"]["file"],
            "first-unmatched.png"
        );
        assert_eq!(metadata["artifacts"]["last_observed"]["status"], "shared");
        assert_eq!(metadata["context"]["socket_path"], Value::Null);
        assert!(
            metadata["context"]["socket_path_debug"]
                .as_str()
                .expect("debug path")
                .contains("\\xFF")
        );
        assert_eq!(metadata["input_authority"], false);
        assert_eq!(metadata["resume_authority"], false);
        assert_eq!(
            fs::metadata(&directory)
                .expect("directory metadata")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        for entry in fs::read_dir(&directory).expect("packet entries") {
            let path = entry.expect("entry").path();
            assert_eq!(
                fs::metadata(path)
                    .expect("private file")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
    }


    /// Different observed originals remain separate and preserve their distinct pixels.
    #[test]
    fn distinct_last_frame_is_saved_without_replacing_first() {
        let parent = TestDirectory::new();
        let context = context();
        let first = frame();
        let mut last = frame();
        last.pixels[0] = 201;
        let mut input = evidence(&context, None);
        input.first_frame = Some(&first);
        input.latest_frame = Some(&last);
        let result = persist_in(input, &parent.0, 124, 7, &AtomicU64::new(0));
        assert!(result.persistence_error.is_none());
        let directory = result.directory.expect("packet directory");
        let original = decode_png(&fs::read(directory.join("first-unmatched.png")).expect("first"))
            .expect("first PNG");
        let observed = decode_png(&fs::read(directory.join("last-observed.png")).expect("last"))
            .expect("last PNG");
        assert_eq!(original.pixels[0], 1);
        assert_eq!(observed.pixels[0], 201);
    }


    /// Directory collisions and existing artifact paths are never overwritten.
    #[test]
    fn exclusive_names_collision_bound_and_checked_serial() {
        let parent = TestDirectory::new();
        let context = context();
        let collision = parent.0.join("pyramid-error-125-7-0");
        fs::write(&collision, b"retained entry").expect("collision entry");
        let result = persist_in(
            evidence(&context, None),
            &parent.0,
            125,
            7,
            &AtomicU64::new(0),
        );
        assert!(
            result
                .directory
                .expect("next unused name")
                .ends_with("pyramid-error-125-7-1")
        );
        assert_eq!(
            fs::read(&collision).expect("retained collision"),
            b"retained entry"
        );
        assert!(write_private(&collision, b"replacement", 100).is_err());
        assert_eq!(
            fs::read(&collision).expect("not overwritten"),
            b"retained entry"
        );
        for number in 0..NAME_ATTEMPTS {
            fs::write(
                parent.0.join(format!("pyramid-error-126-7-{number}")),
                b"collision",
            )
            .expect("bounded collision entry");
        }
        let exhausted = persist_in(
            evidence(&context, None),
            &parent.0,
            126,
            7,
            &AtomicU64::new(0),
        );
        assert!(exhausted.directory.is_none());
        assert!(
            exhausted
                .persistence_error
                .expect("collision error")
                .contains("64 collisions")
        );
        let serial = persist_in(
            evidence(&context, None),
            &parent.0,
            127,
            7,
            &AtomicU64::new(u64::MAX),
        );
        assert!(serial.directory.is_none());
        assert!(
            serial
                .persistence_error
                .expect("serial error")
                .contains("serial exhausted")
        );
    }


    /// Simulate a real exclusive source-file write failing after three retained bytes.
    fn partial_source_writer(path: &Path, bytes: &[u8], limit: usize) -> io::Result<()> {
        if path.file_name().is_some_and(|name| name == "deal.json") {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(path)?;
            file.write_all(
                bytes
                    .get(..3)
                    .ok_or_else(|| io::Error::other("test source too short"))?,
            )?;
            return Err(io::Error::other("injected failure after 3 source bytes"));
        }
        write_private(path, bytes, limit)
    }


    /// Partial data and other artifacts survive while both UI and JSON expose failure.
    #[test]
    fn partial_write_retains_first_failure_and_independent_packet_data() {
        let directory = TestDirectory::new();
        let context = context();
        let result = persist_packet(
            evidence(&context, Some(b"original-source")),
            directory.0.clone(),
            partial_source_writer,
        );
        assert_eq!(result.directory.as_ref(), Some(&directory.0));
        assert!(
            result
                .persistence_error
                .expect("partial error")
                .contains("after 3 source bytes")
        );
        assert_eq!(
            fs::read(directory.0.join("deal.json")).expect("partial source retained"),
            b"ori"
        );
        let metadata = state(&directory.0);
        assert_eq!(
            metadata["artifacts"]["source_document"]["status"],
            "write-failed"
        );
        assert_eq!(
            metadata["artifacts"]["source_document"]["may_be_partial"],
            true
        );
        assert!(
            metadata["write_warnings"]
                .to_string()
                .contains("after 3 source bytes")
        );
        assert!(
            fs::read_to_string(directory.0.join("error.log"))
                .expect("text error")
                .contains("Fresh pre-gesture position mismatch")
        );
    }


    /// Boundary source bytes are exact; oversized source/detail and malformed images refuse visibly.
    #[test]
    fn bounded_evidence_refusals_preserve_the_exact_ui_failure() {
        let parent = TestDirectory::new();
        let context = context();
        let exact = vec![0xff; MAX_SOURCE_BYTES];
        let result = persist_in(
            evidence(&context, Some(&exact)),
            &parent.0,
            128,
            7,
            &AtomicU64::new(0),
        );
        assert!(result.persistence_error.is_none());
        assert_eq!(
            fs::read(result.directory.expect("exact boundary").join("deal.json"))
                .expect("source bytes"),
            exact
        );
        let excessive = vec![0; MAX_SOURCE_BYTES + 1];
        let mut invalid = frame();
        invalid.pixels.clear();
        let mut input = evidence(&context, Some(&excessive));
        input.details = json!({ "excessive": "x".repeat(MAX_DETAILS_BYTES + 1) });
        input.first_frame = Some(&invalid);
        let expected_error = input.error.to_owned();
        let result = persist_in(input, &parent.0, 129, 7, &AtomicU64::new(0));
        assert_eq!(result.error, expected_error);
        let warning = result.persistence_error.expect("visible omissions");
        assert!(warning.contains("deal.json omitted"));
        assert!(warning.contains("first-unmatched.png omitted"));
        assert!(warning.contains("Structured details omitted"));
        let directory = result.directory.expect("partial evidence packet");
        assert!(!directory.join("deal.json").exists());
        assert!(!directory.join("first-unmatched.png").exists());
        assert_eq!(state(&directory)["details"]["omitted"], true);
        assert!(
            fs::metadata(directory.join("error-state.json"))
                .expect("bounded state")
                .len()
                <= MAX_STATE_BYTES as u64
        );
        assert!(
            fs::metadata(directory.join("error.log"))
                .expect("bounded log")
                .len()
                <= MAX_LOG_BYTES as u64
        );
    }


    /// Deep detail and multibyte errors cannot exceed text/JSON limits or replace existing files.
    #[test]
    fn deep_details_unicode_limits_and_metadata_failure_remain_visible() {
        let directory = TestDirectory::new();
        let context = context();
        let mut input = evidence(&context, None);
        let error = "\u{0001}é".repeat(MAX_ERROR_TEXT_BYTES);
        input.error = &error;
        for _ in 0..MAX_DETAILS_DEPTH + 2 {
            input.details = Value::Array(vec![input.details]);
        }
        fs::write(
            directory.0.join("error-state.json"),
            b"first retained state",
        )
        .expect("existing state");
        let result = persist_packet(input, directory.0.clone(), write_private);
        assert_eq!(result.error, error);
        assert!(
            result
                .persistence_error
                .expect("metadata failure")
                .contains("error-state.json write failed")
        );
        assert_eq!(
            fs::read(directory.0.join("error-state.json")).expect("unchanged state"),
            b"first retained state"
        );
        let log = fs::read(directory.0.join("error.log")).expect("bounded plaintext");
        assert!(log.len() <= MAX_LOG_BYTES);
        assert!(String::from_utf8(log).is_ok());
    }


    /// Large escaped identity/error text keeps complete source data and bounded reports.
    #[test]
    fn large_plaintext_report_is_explicitly_bounded_without_losing_source() {
        let parent = TestDirectory::new();
        let mut context = context();
        context.socket_path = PathBuf::from("\u{0001}".repeat(MAX_CONTEXT_TEXT_BYTES * 2));
        let error = "\u{0002}".repeat(MAX_ERROR_TEXT_BYTES * 2);
        let mut input = evidence(&context, Some(b"retained original source"));
        input.error = &error;
        input.details = json!({ "retained": "x".repeat(MAX_DETAILS_BYTES - 128) });
        let result = persist_in(input, &parent.0, 130, 7, &AtomicU64::new(0));
        assert_eq!(result.error, error);
        let warnings = result
            .persistence_error
            .expect("explicit report truncation");
        assert!(warnings.contains("error.log detail truncated"));
        let directory = result.directory.expect("bounded reports directory");
        let log = fs::read(directory.join("error.log")).expect("bounded log");
        assert!(log.len() <= MAX_LOG_BYTES);
        assert!(
            String::from_utf8(log)
                .expect("UTF-8 log")
                .ends_with("persistence error.]\n")
        );
        let metadata = fs::read(directory.join("error-state.json")).expect("bounded state");
        assert!(metadata.len() <= MAX_STATE_BYTES);
        let metadata: Value = serde_json::from_slice(&metadata).expect("bounded valid JSON");
        assert_eq!(metadata["error_original_bytes"], error.len());
        assert!(
            metadata["write_warnings"]
                .to_string()
                .contains("error.log detail truncated")
        );
        assert_eq!(
            fs::read(directory.join("deal.json")).expect("exact source"),
            b"retained original source"
        );
    }


    /// UI correlation survives omitted details and both partial and reservation failure.
    #[test]
    fn advisory_ui_id_survives_failed_persistence_and_oversized_details() {
        let directory = TestDirectory::new();
        let context = context();
        let mut input = evidence(&context, Some(b"original source"));
        input.details = json!({
            "ui_failure_id": 42,
            "oversized": "x".repeat(MAX_DETAILS_BYTES + 1),
        });
        let result = persist_packet(input, directory.0.clone(), partial_source_writer);
        assert_eq!(result.ui_failure_id, Some(42));
        assert!(
            result
                .persistence_error
                .expect("partial persistence")
                .contains("after 3 source bytes")
        );
        let metadata = state(&directory.0);
        assert_eq!(metadata["ui_failure_id"], 42);
        assert_eq!(metadata["details"]["omitted"], true);
        let blocked = directory.0.join("not-a-directory");
        fs::write(&blocked, b"retained entry").expect("blocked parent");
        let mut input = evidence(&context, None);
        input.details = json!({ "ui_failure_id": 43 });
        let result = persist_in(input, &blocked, 131, 7, &AtomicU64::new(0));
        assert_eq!(result.ui_failure_id, Some(43));
        assert!(result.directory.is_none());
        assert!(result.persistence_error.is_some());
        assert_eq!(
            fs::read(blocked).expect("unmodified parent entry"),
            b"retained entry"
        );
        for value in [Value::Null, json!(-1), json!("44")] {
            let mut input = evidence(&context, None);
            input.details = json!({ "ui_failure_id": value });
            assert_eq!(failure_correlation_id(&input), None);
        }
    }
}
