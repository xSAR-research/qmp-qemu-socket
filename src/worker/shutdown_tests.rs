//! Real worker lifetime and queued-command retirement, without guest or image-directory access.

use super::*;


/// Supply a context whose capture would emit progress before failing its nonexistent connection.
fn context() -> ControllerContext {
    ControllerContext::new(
        std::path::PathBuf::from("/nonexistent-shutdown-test/qmp.sock"),
        crate::game::GameMode::TriPeaks,
        crate::strategy::SolvingStrategy::ComputerVision,
        1,
    )
}


/// A termination request already latched by a signal still wakes and joins an idle real worker.
#[test]
fn shutdown_signal_already_set_wakes_idle_worker_and_refuses_new_commands() {
    let shutdown = Arc::new(AtomicBool::new(true));
    let mut handle = WorkerHandle::spawn(
        ImageDiagnostics {
            started_at: Instant::now(),
            directory: None,
        },
        Arc::clone(&shutdown),
    );
    handle.request_shutdown();
    handle.request_shutdown();
    assert!(
        handle
            .capture_frame(context())
            .unwrap_err()
            .contains("shutting down")
    );
    assert!(handle.cancel_requested.load(Ordering::SeqCst));
    let started = Instant::now();
    while !handle.finish_shutdown_if_ready().unwrap() {
        if started.elapsed() > Duration::from_secs(2) {
            // Always wake a broken implementation before panicking so Drop cannot hang the suite.
            let _ = handle.command_tx.send(WorkerCommand::Shutdown);
            panic!("latched termination did not wake the idle worker");
        }
        thread::sleep(Duration::from_millis(1));
    }
    assert!(shutdown.load(Ordering::SeqCst));
    assert!(handle.finish_shutdown_if_ready().unwrap());
}


/// Queued capture commands cannot run ahead of the FIFO Shutdown after the monotonic latch.
#[test]
fn shutdown_retires_queued_capture_before_dispatch() {
    let (commands, receiver) = mpsc::channel();
    let (events, received) = mpsc::channel();
    let latest_frame = LatestFrameSlot::default();
    commands
        .send(WorkerCommand::CaptureFrame { context: context() })
        .unwrap();
    commands.send(WorkerCommand::Shutdown).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    run_worker(
        receiver,
        WorkerEventSink {
            events,
            latest_frame: latest_frame.clone(),
            capture_context: Mutex::new(None),
        },
        Arc::clone(&cancel),
        Arc::new(AtomicBool::new(true)),
    );
    assert!(cancel.load(Ordering::SeqCst));
    assert!(latest_frame.take().is_none());
    assert!(
        !received
            .try_iter()
            .any(|event| matches!(event.event, WorkerEvent::BoardProgress { .. }))
    );
}


/// Polling teardown preserves a busy thread until its completion barrier is released.
/// The bounded receiver also prevents a broken blocking join from hanging the test suite.
#[test]
fn shutdown_poll_does_not_join_or_abandon_a_busy_thread() {
    let (release, completion) = mpsc::channel::<()>();
    let join = thread::spawn(move || {
        let _ = completion.recv_timeout(Duration::from_secs(2));
    });
    let (command_tx, _commands) = mpsc::channel();
    let (_events, event_rx) = mpsc::channel();
    let mut handle = WorkerHandle {
        command_tx,
        event_rx,
        latest_frame: LatestFrameSlot::default(),
        join: Some(join),
        cancel_requested: Arc::new(AtomicBool::new(false)),
        shutdown_requested: Arc::new(AtomicBool::new(false)),
    };
    handle.request_shutdown();
    assert!(!handle.finish_shutdown_if_ready().unwrap());
    assert!(handle.join.is_some());
    release.send(()).unwrap();
    while !handle.finish_shutdown_if_ready().unwrap() {
        thread::yield_now();
    }
    assert!(handle.join.is_none());
}
