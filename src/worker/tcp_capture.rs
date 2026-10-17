//! Retain one serial TCP capture client on its worker thread, with bounded statistics.
//!
//! Endpoint, wire mode and controller identity define a connection's lifetime.
//! Failed operations discard the connection without retrying the capture or
//! replaying input. Only complete native full-display RGBA frames leave this module.

use xsar::tcp::TcpCaptureTiming;

use super::*;


/// Fixed sample bound for rolling acquisition percentiles; lifetime counters are separate.
const SAMPLE_LIMIT: usize = 256;


/// Summaries use monotonic elapsed time and also flush on disconnect/shutdown.
const SUMMARY_INTERVAL: Duration = Duration::from_secs(60);


/// One retained client, exclusively borrowed by the serial worker during acquisition.
struct CachedClient {
    /// Configured endpoint; no protocol fallback changes this address.
    address: String,
    /// Chosen framing and connection lifetime.
    mode: CaptureMode,
    /// Game, socket and generation attached to the request's evidence.
    context: Option<ControllerContext>,
    /// Reusable xSAR transport; response buffers move into each returned frame.
    client: ScreenCaptureClient,
}


/// Fixed-memory rolling acquisition times and saturating lifetime outcome counters.
struct Statistics {
    /// Most recent successful complete acquisition times, in milliseconds.
    samples: [f64; SAMPLE_LIMIT],
    /// Next ring entry to replace.
    next: usize,
    /// Initialised sample count, bounded by `SAMPLE_LIMIT`.
    count: usize,
    /// Successful acquisitions since this worker statistics scope began.
    successes: u64,
    /// Failed or cancelled acquisitions since this worker statistics scope began.
    failures: u64,
    /// Lifetime maximum for successful acquisitions, in milliseconds.
    lifetime_max_ms: f64,
    /// Monotonic timestamp of the previous summary.
    last_summary: Instant,
}


impl Default for Statistics {
    /// Start an empty fixed window; no sample allocation grows with runtime.
    fn default() -> Self {
        Self {
            samples: [0.0; SAMPLE_LIMIT],
            next: 0,
            count: 0,
            successes: 0,
            failures: 0,
            lifetime_max_ms: 0.0,
            last_summary: Instant::now(),
        }
    }
}


impl Statistics {
    /// Record one fully validated success without retaining pixels or request metadata.
    fn success(&mut self, milliseconds: f64) {
        self.samples[self.next] = milliseconds;
        self.next = (self.next + 1) % SAMPLE_LIMIT;
        self.count = (self.count + 1).min(SAMPLE_LIMIT);
        self.successes = self.successes.saturating_add(1);
        self.lifetime_max_ms = self.lifetime_max_ms.max(milliseconds);
    }


    /// Describe successful rolling samples and separately labelled lifetime counters.
    fn summary(&mut self, force: bool) -> Option<String> {
        if self.successes == 0 && self.failures == 0 {
            return None;
        }
        if !force && self.last_summary.elapsed() < SUMMARY_INTERVAL {
            return None;
        }
        self.last_summary = Instant::now();
        let mut sorted = self.samples;
        sorted[..self.count].sort_unstable_by(f64::total_cmp);
        let (median, p95, maximum) = if self.count == 0 {
            (0.0, 0.0, 0.0)
        } else {
            let middle = self.count / 2;
            let median = if self.count.is_multiple_of(2) {
                (sorted[middle - 1] + sorted[middle]) / 2.0
            } else {
                sorted[middle]
            };
            (
                median,
                sorted[(95 * self.count).div_ceil(100) - 1],
                sorted[self.count - 1],
            )
        };
        Some(format!(
            "TCP capture statistics: rolling successful samples={}/{}; acquisition median={median:.1} ms; p95={p95:.1} ms; max={maximum:.1} ms; lifetime successes={}; failures={}; successful max={:.1} ms. Window excludes failed acquisitions; counters cover this worker scope.",
            self.count, SAMPLE_LIMIT, self.successes, self.failures, self.lifetime_max_ms,
        ))
    }
}


thread_local! {
    /// TCP ownership is confined to the same thread as serial capture execution.
    static CLIENT: RefCell<Option<CachedClient>> = const { RefCell::new(None) };
    /// Cancellation is attached only to guarded commands, preserving later manual capture.
    static CANCELLATION: RefCell<Option<Arc<AtomicBool>>> = const { RefCell::new(None) };
    /// Fixed storage is independent of the number of overnight captures.
    static STATISTICS: RefCell<Statistics> = RefCell::new(Statistics::default());
}


/// Restore a command's previous cancellation context on return or unwind.
pub(super) struct CancellationScope(Option<Arc<AtomicBool>>);


impl CancellationScope {
    /// Bind TCP socket waits to the guarded run's existing cooperative STOP flag.
    pub(super) fn install(cancel: Arc<AtomicBool>) -> Self {
        Self(CANCELLATION.with(|slot| slot.replace(Some(cancel))))
    }
}


impl Drop for CancellationScope {
    /// Do not carry a completed run's STOP flag into an explicit later manual capture.
    fn drop(&mut self) {
        CANCELLATION.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}


/// Ensure worker shutdown closes TCP and emits the last bounded statistics summary.
pub(super) struct WorkerCaptureScope<'a>(&'a WorkerEventSink);


impl<'a> WorkerCaptureScope<'a> {
    /// Reset prior thread-local state before this worker begins dispatching commands.
    pub(super) fn install(events: &'a WorkerEventSink) -> Self {
        reset(None);
        Self(events)
    }
}


impl Drop for WorkerCaptureScope<'_> {
    /// Disconnect even when the command channel closes or a worker call unwinds.
    fn drop(&mut self) {
        reset(Some(self.0));
    }
}


/// Decode explicit wire selection; an unknown value never triggers protocol guessing.
pub(super) fn parse_mode(value: &str) -> Result<CaptureMode, String> {
    match value {
        "v2" => Ok(CaptureMode::LegacyV2),
        "v3" => Ok(CaptureMode::BgrV3),
        "v3-persistent" => Ok(CaptureMode::PersistentBgrV3),
        _ => Err("QMP_CAPTURE_WIRE must be v2, v3 or v3-persistent".to_owned()),
    }
}


/// Close the current transport and start a fresh bounded statistics scope.
pub(super) fn reset(events: Option<&WorkerEventSink>) {
    CLIENT.with(|slot| *slot.borrow_mut() = None);
    STATISTICS.with(|slot| {
        let mut statistics = slot.borrow_mut();
        if let Some(events) = events
            && let Some(summary) = statistics.summary(true)
        {
            send_log(events, summary);
        }
        *statistics = Statistics::default();
    });
}


/// Measure only publication into the coalesced slot, excluding caller clones and UI rendering.
pub(super) fn log_preview_scheduling(events: &WorkerEventSink, elapsed: Duration) {
    if CLIENT.with(|slot| slot.borrow().is_some()) {
        send_log(
            events,
            format!(
                "TCP preview scheduling: {:.3} ms into bounded latest-frame slot; excludes caller frame cloning, acquisition, detection and UI rendering.",
                milliseconds(elapsed),
            ),
        );
    }
}


/// Report shared scene/target analysis separately from acquisition and preview scheduling.
pub(super) fn log_detection(events: &WorkerEventSink, elapsed: Duration) {
    if CLIENT.with(|slot| slot.borrow().is_some()) {
        send_log(
            events,
            format!(
                "TCP shared scene/target detection: {:.3} ms; excludes acquisition, PNG encoding and preview scheduling.",
                milliseconds(elapsed),
            ),
        );
    }
}


/// Require the application's calibrated, fully initialised RGBA frame before publication.
fn validate_native_frame(frame: &CapturedFrame) -> Result<(), String> {
    if frame.width != NOMINAL_FRAME_WIDTH
        || frame.height != NOMINAL_FRAME_HEIGHT
        || frame.stride != NOMINAL_FRAME_WIDTH as usize * 4
        || frame.pixels.len() != NOMINAL_FRAME_WIDTH as usize * NOMINAL_FRAME_HEIGHT as usize * 4
        || !frame.is_layout_valid()
    {
        return Err(format!(
            "TCP capture requires native {}x{} packed RGBA8; received {}x{}, stride={}, bytes={}",
            NOMINAL_FRAME_WIDTH,
            NOMINAL_FRAME_HEIGHT,
            frame.width,
            frame.height,
            frame.stride,
            frame.pixels.len(),
        ));
    }
    Ok(())
}


/// Log disjoint client stages; local total includes validation and scheduling overhead.
fn log_measurement(events: &WorkerEventSink, mode: CaptureMode, timing: &TcpCaptureTiming) {
    let metadata = &timing.metadata;
    send_log(
        events,
        format!(
            "TCP capture: mode={mode:?}; protocol={}; session={}; request={}; frame={}; display-generation={}; reused={}; request-bytes={}; header-bytes={}; decoded-bytes={}; payload-bytes={}; connect={:.3} ms; request-write={:.3} ms; header-wait={:.3} ms; payload-allocation={:.3} ms; payload-receive={:.3} ms; destination-prepare={:.3} ms; conversion={:.3} ms; acquisition-total={:.3} ms. Header wait starts after request write; total includes these stages. PNG, detection and preview scheduling are separate; no compression or regional placement.",
            metadata.protocol_version,
            metadata.session_id,
            metadata.request_id,
            metadata.frame_id,
            metadata.display_generation,
            metadata.reused_connection,
            metadata.request_bytes,
            metadata.response_header_bytes,
            metadata.decoded_bytes,
            metadata.payload_bytes,
            timing.connect_ms,
            timing.request_write_ms,
            timing.header_wait_ms,
            timing.allocation_ms,
            timing.pixel_read_ms,
            timing.canvas_prepare_ms,
            timing.conversion_ms,
            timing.total_ms,
        ),
    );
}


/// Acquire once through the retained xSAR client; errors drop its session without retry.
pub(super) fn capture(
    address: &str,
    mode: CaptureMode,
    events: &WorkerEventSink,
) -> Result<(CapturedFrame, CaptureTiming), String> {
    let context = events
        .capture_context
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let cancel = CANCELLATION.with(|slot| slot.borrow().clone());
    let result = CLIENT.with(|slot| {
        let mut cached = slot.borrow_mut();
        if cached.as_ref().is_none_or(|entry| {
            entry.address != address || entry.mode != mode || entry.context != context
        }) {
            *cached = Some(CachedClient {
                address: address.to_owned(),
                mode,
                context,
                client: ScreenCaptureClient::with_mode(Some(address.to_owned()), mode),
            });
        }
        let entry = cached
            .as_mut()
            .ok_or_else(|| "TCP client ownership was lost".to_owned())?;
        let inactive_cancel = AtomicBool::new(false);
        let (tcp_frame, tcp_timing) = entry
            .client
            .capture_full_cancellable(cancel.as_deref().unwrap_or(&inactive_cancel))
            .map_err(|error| format!("TCP capture failed: {error}"))?;
        let frame = CapturedFrame {
            width: tcp_frame.width,
            height: tcp_frame.height,
            stride: tcp_frame.stride,
            format: PixelFormat::Rgba8,
            pixels: tcp_frame.pixels,
        };
        validate_native_frame(&frame)?;
        Ok((frame, tcp_timing))
    });
    match result {
        Ok((frame, measured)) => {
            log_measurement(events, mode, &measured);
            STATISTICS.with(|slot| {
                let mut statistics = slot.borrow_mut();
                statistics.success(measured.total_ms);
                if let Some(summary) = statistics.summary(false) {
                    send_log(events, summary);
                }
            });
            let timing = CaptureTiming {
                captures: 1,
                screendump: Duration::from_secs_f64(
                    (measured.connect_ms + measured.request_ms) / 1000.0,
                ),
                file_read: Duration::from_secs_f64(
                    (measured.allocation_ms + measured.pixel_read_ms) / 1000.0,
                ),
                decode: Duration::from_secs_f64(
                    (measured.canvas_prepare_ms + measured.conversion_ms) / 1000.0,
                ),
                ..CaptureTiming::default()
            };
            Ok((frame, timing))
        }
        Err(error) => {
            CLIENT.with(|slot| *slot.borrow_mut() = None);
            STATISTICS.with(|slot| {
                let mut statistics = slot.borrow_mut();
                statistics.failures = statistics.failures.saturating_add(1);
                if let Some(summary) = statistics.summary(false) {
                    send_log(events, summary);
                }
            });
            Err(error)
        }
    }
}


#[cfg(test)]
mod tests {
    //! Validate fixed statistics storage and explicit mode selection without sockets.

    use super::*;


    /// A long run replaces rolling entries while preserving labelled lifetime counters.
    #[test]
    fn statistics_storage_is_bounded_and_percentiles_are_labelled() {
        let mut statistics = Statistics::default();
        for value in 1..=1_024 {
            statistics.success(f64::from(value));
        }
        statistics.failures = 3;
        assert_eq!(statistics.count, SAMPLE_LIMIT);
        assert_eq!(statistics.next, 0);
        assert!(statistics.summary(false).is_none());
        let summary = statistics.summary(true).unwrap();
        assert!(summary.contains("rolling successful samples=256/256"));
        assert!(summary.contains("median=896.5 ms; p95=1012.0 ms; max=1024.0 ms"));
        assert!(summary.contains("lifetime successes=1024; failures=3"));
    }


    /// Version selection stays explicit and rejects unknown spellings.
    #[test]
    fn wire_modes_have_no_implicit_fallback() {
        assert_eq!(parse_mode("v2").unwrap(), CaptureMode::LegacyV2);
        assert_eq!(parse_mode("v3").unwrap(), CaptureMode::BgrV3);
        assert_eq!(
            parse_mode("v3-persistent").unwrap(),
            CaptureMode::PersistentBgrV3
        );
        assert!(parse_mode("auto").is_err());
    }
}
