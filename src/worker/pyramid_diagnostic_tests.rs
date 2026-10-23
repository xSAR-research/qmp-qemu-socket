//! Worker dispatch regressions for advisory Pyramid failure packets.
//!
//! Each test owns its QMP/TCP listener traps, worker thread and transport directory.
//! The production persistence policy is exercised without environment changes;
//! cleanup removes only packet directories returned by that test's worker.

use std::{
    net::TcpListener,
    os::unix::{fs::DirBuilderExt, net::UnixListener},
    sync::atomic::AtomicU64,
};

use super::*;


/// Unique transport fixture sequence, independent of packet naming and authority.
static TEST_SERIAL: AtomicU64 = AtomicU64::new(0);


/// Private temporary transport directory owned by one test.
struct TestDirectory(PathBuf);


impl TestDirectory {
    /// Exclusively create a bounded local fixture without changing global settings.
    fn new() -> Self {
        for _ in 0..64 {
            let serial = TEST_SERIAL.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "pyramid-worker-diag-test-{}-{serial}",
                std::process::id()
            ));
            match fs::DirBuilder::new().mode(0o700).create(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                Err(error) => panic!("private transport fixture: {error}"),
            }
        }
        panic!("transport fixture collision allowance exhausted");
    }
}


impl Drop for TestDirectory {
    /// Remove only this test's exclusively created transport fixture.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}


/// Cleanup ownership of a packet explicitly returned by the isolated worker.
struct PacketDirectory(PathBuf);


impl PacketDirectory {
    /// Accept only a real reserved packet directory returned on this event channel.
    fn from_notice(diagnostic: &PyramidDiagnostic) -> Self {
        let path = diagnostic
            .directory
            .clone()
            .expect("reserved failure packet");
        assert!(
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("pyramid-error-"))
        );
        let metadata = fs::symlink_metadata(&path).expect("returned packet directory");
        assert!(metadata.is_dir() && !metadata.file_type().is_symlink());
        Self(path)
    }
}


impl Drop for PacketDirectory {
    /// Remove only the newly reserved packet received by this test.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}


/// A real command-loop thread whose only possible transport targets are local traps.
struct Harness {
    /// Actual queue/event interface under test.
    handle: WorkerHandle,
    /// No accepted connection is expected for a diagnostic or pre-acquisition refusal.
    qmp: UnixListener,
    /// Any unexpected capture would reach this owned listener rather than a guest.
    capture: TcpListener,
    /// Keeps the QMP socket pathname alive until the worker and listeners are dropped.
    directory: TestDirectory,
}


impl Harness {
    /// Start production dispatch with unrelated preview identity and explicit test capture routing.
    fn new() -> Self {
        let directory = TestDirectory::new();
        let socket = directory.0.join("qmp.sock");
        let qmp = UnixListener::bind(&socket).expect("owned QMP trap");
        qmp.set_nonblocking(true).expect("nonblocking QMP trap");
        let capture = TcpListener::bind("127.0.0.1:0").expect("owned capture trap");
        capture
            .set_nonblocking(true)
            .expect("nonblocking capture trap");
        let capture_address = capture
            .local_addr()
            .expect("capture trap address")
            .to_string();
        let initial = ControllerContext::new(
            socket,
            GameMode::Pyramid,
            SolvingStrategy::ComputerVision,
            700,
        );
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        let latest_frame = LatestFrameSlot::default();
        let cancel_requested = Arc::new(AtomicBool::new(false));
        latest_frame
            .reconnaissance
            .select(&initial, &cancel_requested)
            .expect("initial context");
        let sink = WorkerEventSink {
            events: event_tx,
            latest_frame: latest_frame.clone(),
            capture_context: Mutex::new(Some(initial)),
        };
        let worker_cancel = Arc::clone(&cancel_requested);
        let join = thread::Builder::new()
            .name("pyramid-diagnostic-test".to_owned())
            .spawn(move || {
                let _backend = TestCaptureBackend::tcp(capture_address);
                WORKER_IMAGE_DIAGNOSTICS.with(|slot| {
                    *slot.borrow_mut() = Some(WorkerImageDiagnostics::new(ImageDiagnostics {
                        started_at: Instant::now(),
                        directory: None,
                    }));
                });
                run_worker(command_rx, sink, worker_cancel);
            })
            .expect("isolated diagnostic worker");
        Self {
            handle: WorkerHandle {
                command_tx,
                event_rx,
                latest_frame,
                join: Some(join),
                cancel_requested,
            },
            qmp,
            capture,
            directory,
        }
    }


    /// Build a requested Pyramid identity distinct from the existing preview identity.
    fn context(&self, generation: u64) -> ControllerContext {
        ControllerContext::new(
            self.directory.0.join("qmp.sock"),
            GameMode::Pyramid,
            SolvingStrategy::ShortestPath,
            generation,
        )
    }


    /// Observe a bounded ordered event prefix and acquire packet cleanup immediately.
    fn receive_until(
        &self,
        complete: impl Fn(&WorkerEvent) -> bool,
    ) -> (Vec<ContextualWorkerEvent>, Vec<PacketDirectory>) {
        let started = Instant::now();
        let mut notifications = Vec::new();
        let mut packets = Vec::new();
        loop {
            let remaining = Duration::from_secs(10)
                .checked_sub(started.elapsed())
                .expect("bounded worker notification deadline");
            let notification = self
                .handle
                .event_rx
                .recv_timeout(remaining)
                .expect("worker notification before deadline");
            if let WorkerEvent::PyramidDiagnostic(diagnostic) = &notification.event {
                packets.push(PacketDirectory::from_notice(diagnostic));
            }
            let done = complete(&notification.event);
            notifications.push(notification);
            if done {
                return (notifications, packets);
            }
            assert!(
                notifications.len() < 32,
                "unexpected diagnostic event flood"
            );
        }
    }


    /// Complete queued work and stop before asserting that no transport was contacted.
    fn shutdown(&mut self) {
        self.handle
            .command_tx
            .send(WorkerCommand::Shutdown)
            .expect("queue shutdown");
        self.handle
            .join
            .take()
            .expect("worker thread")
            .join()
            .expect("worker completed");
        assert_eq!(
            self.qmp.accept().expect_err("no QMP connection").kind(),
            ErrorKind::WouldBlock
        );
        assert_eq!(
            self.capture
                .accept()
                .expect_err("no capture connection")
                .kind(),
            ErrorKind::WouldBlock
        );
        assert!(self.handle.latest_frame.take().is_none());
    }
}


/// Parse a worker-created packet without converting its source into run authority.
fn packet_state(directory: &PacketDirectory) -> serde_json::Value {
    serde_json::from_slice(&fs::read(directory.0.join("error-state.json")).expect("state document"))
        .expect("bounded state JSON")
}


/// A local refusal stays on the worker, preserves STOP/epoch and explicit advisory identity.
#[test]
fn local_refusal_queue_preserves_context_ui_id_and_stop_without_transport() {
    let mut harness = Harness::new();
    let context = harness.context(17);
    harness
        .handle
        .cancel_requested
        .store(true, Ordering::Release);
    let epoch = harness
        .handle
        .latest_frame
        .reconnaissance
        .epoch()
        .expect("initial epoch");
    let error = "Acquire refused: output filename is empty";
    let worker_id = harness
        .handle
        .join
        .as_ref()
        .expect("worker handle")
        .thread()
        .id();
    assert_ne!(worker_id, thread::current().id());
    harness
        .handle
        .record_pyramid_failure(
            context.clone(),
            PyramidPhase::Reconnaissance,
            error.to_owned(),
            serde_json::json!({ "ui_failure_id": 7001, "requested_step": "acquire" }),
        )
        .expect("queue local refusal");
    let (mut notifications, packets) =
        harness.receive_until(|event| matches!(event, WorkerEvent::PyramidDiagnostic(_)));
    harness.shutdown();
    notifications.extend(harness.handle.event_rx.try_iter());
    assert!(harness.handle.cancel_requested.load(Ordering::Acquire));
    assert_eq!(
        harness
            .handle
            .latest_frame
            .reconnaissance
            .epoch()
            .expect("unchanged epoch"),
        epoch
    );
    assert_eq!(packets.len(), 1);
    let notice = notifications
        .iter()
        .find(|notice| matches!(&notice.event, WorkerEvent::PyramidDiagnostic(_)))
        .expect("explicit diagnostic notice");
    assert_eq!(notice.context, Some(context.clone()));
    assert!(
        notice.emitted_at.is_some(),
        "worker-local clock stamps the notice"
    );
    let WorkerEvent::PyramidDiagnostic(diagnostic) = &notice.event else {
        unreachable!()
    };
    assert_eq!(diagnostic.context, context);
    assert_eq!(diagnostic.phase, PyramidPhase::Reconnaissance);
    assert_eq!(diagnostic.run_id, None);
    assert_eq!(diagnostic.ui_failure_id, Some(7001));
    assert_eq!(diagnostic.error, error);
    assert!(diagnostic.persistence_error.is_none());
    assert!(notifications.iter().all(|notice| matches!(
        &notice.event,
        WorkerEvent::Log(_) | WorkerEvent::PyramidDiagnostic(_)
    )));
    let metadata = packet_state(&packets[0]);
    assert_eq!(metadata["context"]["generation"], 17);
    assert_eq!(metadata["ui_failure_id"], 7001);
    assert_eq!(metadata["details"]["requested_step"], "acquire");
    assert_eq!(metadata["first_frame"], serde_json::Value::Null);
    assert_eq!(metadata["latest_frame"], serde_json::Value::Null);
}


/// Missing acquisition emits an honest frameless packet before the original matched finish error.
#[test]
fn calculation_refusal_packet_precedes_matching_finish_and_preserves_original_error() {
    const RUN_ID: u64 = 8802;
    const ERROR: &str = "No same-session restored acquisition; acquire the board first";
    let mut harness = Harness::new();
    let context = harness.context(23);
    let unopened = harness
        .directory
        .0
        .join("unopened extensionless deal with spaces");
    harness
        .handle
        .calculate_route(context.clone(), RUN_ID, unopened.clone())
        .expect("queue calculation");
    let (mut notifications, packets) = harness.receive_until(|event| {
        matches!(event,
        WorkerEvent::ComputedRouteFinished { run_id, .. } if *run_id == RUN_ID)
    });
    harness.shutdown();
    notifications.extend(harness.handle.event_rx.try_iter());
    let diagnostic_index = notifications
        .iter()
        .position(|notice| matches!(&notice.event, WorkerEvent::PyramidDiagnostic(_)))
        .expect("planning diagnostic before finish");
    let finished_index = notifications
        .iter()
        .position(|notice| {
            matches!(&notice.event,
        WorkerEvent::ComputedRouteFinished { run_id, .. } if *run_id == RUN_ID)
        })
        .expect("matching calculation finish");
    assert!(diagnostic_index < finished_index);
    assert_eq!(packets.len(), 1);
    assert_eq!(
        notifications[diagnostic_index].context,
        Some(context.clone())
    );
    assert_eq!(notifications[finished_index].context, Some(context.clone()));
    let WorkerEvent::PyramidDiagnostic(diagnostic) = &notifications[diagnostic_index].event else {
        unreachable!()
    };
    assert_eq!(diagnostic.context, context);
    assert_eq!(diagnostic.phase, PyramidPhase::Planning);
    assert_eq!(diagnostic.run_id, Some(RUN_ID));
    assert_eq!(diagnostic.ui_failure_id, None);
    assert_eq!(diagnostic.error, ERROR);
    let WorkerEvent::ComputedRouteFinished {
        retained_recon: _,
        run_id,
        plan_id,
        result,
    } = &notifications[finished_index].event
    else {
        unreachable!()
    };
    assert_eq!(*run_id, RUN_ID);
    assert_eq!(*plan_id, None);
    assert_eq!(result, &Err(ERROR.to_owned()));
    assert!(
        !unopened.exists(),
        "no acquisition must be refused before opening input"
    );
    let metadata = packet_state(&packets[0]);
    assert_eq!(metadata["phase"], "planning");
    assert_eq!(metadata["run_id"], RUN_ID);
    assert_eq!(metadata["error"], ERROR);
    assert_eq!(metadata["details"]["guest_input_sent"], false);
    assert!(metadata["details"].get("search_stats").is_none());
    assert_eq!(metadata["first_frame"], serde_json::Value::Null);
    assert_eq!(metadata["latest_frame"], serde_json::Value::Null);
    assert_eq!(metadata["input_authority"], false);
    assert_eq!(metadata["resume_authority"], false);
    let files: std::collections::BTreeSet<_> = fs::read_dir(&packets[0].0)
        .expect("frameless packet")
        .map(|entry| entry.expect("packet entry").file_name())
        .collect();
    let expected: std::collections::BTreeSet<_> = ["error-state.json", "error.log"]
        .map(std::ffi::OsString::from)
        .into_iter()
        .collect();
    assert_eq!(files, expected);
    assert!(
        notifications
            .iter()
            .all(|notice| notice.emitted_at.is_some())
    );
    assert!(!notifications.iter().any(|notice| matches!(
        &notice.event,
        WorkerEvent::State(WorkerState::Connecting | WorkerState::Capturing | WorkerState::Acting)
            | WorkerEvent::SnapshotPrepared { .. }
            | WorkerEvent::ActionCompleted { .. }
    )));
}
