//! Real loopback TCPv3 and local QMP bridge evidence; no guest connection.
use super::*;
use crate::geometry::PixelRect;
use crate::worker::tcp_capture_tests::{V3Reply, v3_server, v3_server_with_read_timeout};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::{
        fs::DirBuilderExt,
        net::{UnixListener, UnixStream},
    },
    sync::atomic::AtomicU64,
};
use xsar::games::freecell::rules::Pile;


/// Fault at an existing transport boundary, rather than injected controller state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Case {
    Both,
    /// Prove two occupied PLAY corners and retain the observed destination point.
    PlayDestination,
    /// Bind a complete native deal and execute the frozen queued REL mode.
    Relative,
    /// A real truncated response follows the acknowledged, retired probe.
    RelativeCaptureFailure,
    /// STOP arrives after permit retirement, before the read-only after-frame.
    RelativeStopAfterAck,
    Stop,
    Revoke,
    DownError,
    /// The next ordinary effect observation reports completion after release.
    CompleteAfterDrag,
    /// Ordinary effect pixels do not prove any permitted route endpoint.
    UnsupportedAfterDrag,
    /// The paced REL experiment observes STOP before its first held motion.
    StopDuringHeldWait,
    /// The paced ABS experiment observes STOP after DOWN ACK, before destination motion.
    AbsoluteStopDuringHeldWait,
}


impl Case {
    /// Select only the separately queued bounded REL diagnostic cases.
    fn relative_probe(self) -> bool {
        matches!(
            self,
            Self::Relative
                | Self::RelativeCaptureFailure
                | Self::RelativeStopAfterAck
                | Self::StopDuringHeldWait
        )
    }


    /// Select STOP injected at either diagnostic's first intentional held wait.
    fn stops_during_held_wait(self) -> bool {
        matches!(
            self,
            Self::StopDuringHeldWait | Self::AbsoluteStopDuringHeldWait
        )
    }
}


/// Exclusive temporary socket directory, removed after all clients close.
struct Socket(PathBuf);


impl Socket {
    /// Create a unique private directory without replacing another test's files.
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let path = std::env::temp_dir().join(format!(
            "freecell311-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .unwrap();
        Self(path)
    }


    /// Return the listener name owned by this fixture.
    fn path(&self) -> PathBuf {
        self.0.join("qmp.sock")
    }
}


impl Drop for Socket {
    /// Remove only this newly created fixture after the server has joined.
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}


/// Preserve request IDs and flush every response before the next command.
fn packet(stream: &mut UnixStream, value: Value) -> Result<(), String> {
    serde_json::to_writer(&mut *stream, &value).map_err(|e| e.to_string())?;
    stream.write_all(b"\n").map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())
}


/// Consume each published acquisition at its probe, requiring another for the next half.
fn qmp_server(
    path: &std::path::Path,
    case: Case,
    latest: LatestFrameSlot,
    context: ControllerContext,
    cancel: Arc<AtomicBool>,
    expected: CapturedFrame,
) -> thread::JoinHandle<(Vec<Value>, usize)> {
    let listener = UnixListener::bind(path).unwrap();
    listener.set_nonblocking(true).unwrap();
    thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut stream = loop {
            match listener.accept() {
                Ok((s, _)) => break s,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline);
                    thread::sleep(Duration::from_millis(5));
                }
                Err(e) => panic!("QMP accept: {e}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(15)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        packet(
            &mut stream,
            json!({"QMP":{"version":{"qemu":{"major":11,"minor":0,"micro":0}},"capabilities":[]}}),
        )
        .unwrap();
        let mut requests = Vec::new();
        let mut captures = 0;
        let mut rejected = false;
        let mut stopped_during_hold = false;
        let mut relative_current = false;
        let mut mouse_selections = Vec::new();
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap() == 0 {
                break;
            }
            let request: Value = serde_json::from_str(&line).unwrap();
            let mut error = false;
            let response = match request["execute"].as_str().unwrap() {
                "qmp_capabilities" => json!({}),
                "query-status" => {
                    let frame = latest
                        .take()
                        .expect("each probe needs fresh acquired pixels");
                    assert_eq!(frame.context.as_ref(), Some(&context));
                    assert_eq!(frame.frame.pixels, expected.pixels);
                    captures += 1;
                    json!({"running":true,"status":"running"})
                }
                "query-mice" => {
                    match case {
                        Case::Stop => {
                            latest.reconnaissance.operator_stop(&context, &cancel);
                        }
                        Case::Revoke => latest.freecell_route.lock().unwrap().revoke(),
                        _ => {}
                    };
                    if case.relative_probe() {
                        json!([
                            {"name":"QEMU HID Tablet","index":0,"current":!relative_current,"absolute":true},
                            {"name":"QEMU PS/2 Mouse","index":1,"current":relative_current,"absolute":false}
                        ])
                    } else {
                        json!([{"name":"QEMU HID Tablet","index":0,"current":true,"absolute":true}])
                    }
                }
                "human-monitor-command" => {
                    let command = request["arguments"]["command-line"].as_str().unwrap();
                    match command {
                        "mouse_set 1" => {
                            assert!(!relative_current || stopped_during_hold);
                            relative_current = true;
                        }
                        "mouse_set 0" => {
                            assert!(
                                !stopped_during_hold || !case.relative_probe(),
                                "STOP must not restore ordinary routing"
                            );
                            relative_current = false;
                        }
                        other => panic!("unexpected HMP command: {other}"),
                    }
                    mouse_selections.push(command.to_owned());
                    json!("")
                }
                "input-send-event" => {
                    let events = request["arguments"]["events"].as_array().unwrap();
                    if case.relative_probe() {
                        for event in events {
                            match event["type"].as_str().unwrap() {
                                "abs" => assert!(!relative_current),
                                "rel" | "btn" => assert!(relative_current),
                                other => panic!("unexpected input type: {other}"),
                            }
                        }
                    }
                    if case == Case::DownError
                        && events
                            .iter()
                            .any(|e| e["type"] == "btn" && e["data"]["down"] == true)
                    {
                        assert!(!rejected);
                        rejected = true;
                        error = true;
                    }
                    json!({})
                }
                other => panic!("unexpected QMP command: {other}"),
            };
            let response = if error {
                json!({"error":{"class":"GenericError","desc":"test rejected down"},"id":request["id"]})
            } else {
                json!({"return":response,"id":request["id"]})
            };
            requests.push(request);
            let stop_after_down = case.stops_during_held_wait()
                && requests.last().is_some_and(|request| {
                    request["execute"] == "input-send-event"
                        && request["arguments"]["events"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|event| event["type"] == "btn" && event["data"]["down"] == true)
                });
            if stop_after_down {
                assert!(!stopped_during_hold);
                stopped_during_hold = true;
                if case.relative_probe() {
                    // The REL held wait supplies the causal boundary before any motion.
                    latest.reconnaissance.operator_stop(&context, &cancel);
                }
            }
            let delivered = packet(&mut stream, response);
            if (case == Case::DownError && rejected && requests.len() > 5)
                || (case.stops_during_held_wait()
                    && stopped_during_hold
                    && (case.relative_probe() || !stop_after_down))
            {
                // Recovery can close this stream after consuming the earlier DOWN ACK.
                // Exact old-UP and fresh capabilities/UP traffic is asserted below.
                let _ = delivered;
            } else {
                delivered.unwrap();
            }
            if stop_after_down && case == Case::AbsoluteStopDuringHeldWait {
                // DOWN is acknowledged before STOP arrives inside either selected held wait.
                thread::sleep(Duration::from_millis(100));
                latest.reconnaissance.operator_stop(&context, &cancel);
            }
        }
        if case.relative_probe() && !stopped_during_hold {
            assert_eq!(
                mouse_selections,
                ["mouse_set 0", "mouse_set 1", "mouse_set 0"]
            );
            assert!(!relative_current, "original tablet restored after release");
        } else if stopped_during_hold && case.relative_probe() {
            assert_eq!(
                mouse_selections,
                ["mouse_set 0", "mouse_set 1", "mouse_set 1"]
            );
            assert!(
                relative_current,
                "STOP keeps the captured release target selected"
            );
        } else if stopped_during_hold {
            assert_eq!(mouse_selections, ["mouse_set 0", "mouse_set 0"]);
            assert!(!relative_current, "ABS release retains the captured tablet");
        }
        if rejected || stopped_during_hold {
            // Existing transport confirms only release-UP on one fresh connection.
            let deadline = Instant::now() + Duration::from_secs(5);
            let mut fresh = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline);
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => panic!("recovery accept: {e}"),
                }
            };
            fresh
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(fresh.try_clone().unwrap());
            packet(&mut fresh,json!({"QMP":{"version":{"qemu":{"major":11,"minor":0,"micro":0}},"capabilities":[]}})).unwrap();
            let recovery_commands: &[&str] = if stopped_during_hold {
                &[
                    "qmp_capabilities",
                    "query-mice",
                    "human-monitor-command",
                    "query-mice",
                    "input-send-event",
                ]
            } else {
                &["qmp_capabilities", "input-send-event"]
            };
            for expected_command in recovery_commands {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let request: Value = serde_json::from_str(&line).unwrap();
                assert_eq!(request["execute"], *expected_command);
                let value = match *expected_command {
                    "query-mice" => {
                        let release_relative = stopped_during_hold && case.relative_probe();
                        json!([
                            {"name":"QEMU HID Tablet","index":0,"current":!release_relative,"absolute":true},
                            {"name":"QEMU PS/2 Mouse","index":1,"current":release_relative,"absolute":false}
                        ])
                    }
                    "human-monitor-command" => {
                        assert_eq!(
                            request["arguments"]["command-line"],
                            if case.relative_probe() {
                                "mouse_set 1"
                            } else {
                                "mouse_set 0"
                            }
                        );
                        json!("")
                    }
                    "input-send-event" => {
                        button(&request, false);
                        json!({})
                    }
                    _ => json!({}),
                };
                packet(&mut fresh, json!({"return":value,"id":request["id"]})).unwrap();
                requests.push(request);
            }
        }
        assert!(
            matches!(listener.accept(), Err(error)
            if error.kind() == std::io::ErrorKind::WouldBlock),
            "no additional connection or input replay is permitted"
        );
        (requests, captures)
    })
}


/// Assert checked full-display integer mapping on the actual outgoing movement request.
fn movement(request: &Value, point: PixelPoint) {
    let events = request["arguments"]["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    for (axis, value) in [("x", point.x * 32767 / 1920), ("y", point.y * 32767 / 1080)] {
        let event = events.iter().find(|e| e["data"]["axis"] == axis).unwrap();
        assert_eq!(event["type"], "abs");
        assert_eq!(event["data"]["value"], json!(value));
    }
}


/// Assert one independently acknowledged left-button transition.
fn button(request: &Value, down: bool) {
    let events = request["arguments"]["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["type"], "btn");
    assert_eq!(events[0]["data"]["button"], "left");
    assert_eq!(events[0]["data"]["down"], down);
}


/// Test Native's transport boundary, not a search result or guest movement effect.
fn exercise(case: Case, diagnostic_step: bool) {
    exercise_with_timing(
        case,
        diagnostic_step,
        route::DiagnosticDragTiming::TwoSeconds,
    );
}


/// Exercise the frozen ABS interval through real local peers without guest input.
fn exercise_with_timing(
    case: Case,
    diagnostic_step: bool,
    drag_timing: route::DiagnosticDragTiming,
) {
    let root = Socket::new();
    let mut id = plan(31);
    id.context = ControllerContext::new(
        root.path(),
        GameMode::FreeCell,
        SolvingStrategy::ShortestPath,
        7,
    );
    let original = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/freecell-FC13.png"
    ))
    .unwrap();
    let deal = crate::freecell_recon::recognise(
        &original,
        Metadata::new("native-bridge", "FC13", "unit").unwrap(),
    )
    .unwrap();
    let parent = Position::initial(&deal);
    let mut action = RecordedMove {
        card_id: *parent.column(0).unwrap().last().unwrap(),
        from: Pile::Cascade(0),
        to: Pile::Cell(0),
        is_auto: false,
    };
    if case == Case::PlayDestination {
        action = (0..8)
            .flat_map(|from| (0..8).map(move |to| (from, to)))
            .find_map(|(from, to)| {
                let candidate = RecordedMove {
                    card_id: *parent.column(usize::from(from))?.last()?,
                    from: Pile::Cascade(from),
                    to: Pile::Cascade(to),
                    is_auto: false,
                };
                parent.apply(candidate).ok().map(|_| candidate)
            })
            .expect("native fixture has a legal PLAY transfer");
    }
    parent.apply(action).unwrap();
    let geometry = position::verify(&original, &parent, &mut || Ok(())).unwrap();
    let from = geometry.point(action.from).unwrap();
    let to = geometry.point(action.to).unwrap();
    let boundary_case = matches!(case, Case::CompleteAfterDrag | Case::UnsupportedAfterDrag);
    let bind_initial = boundary_case;
    let mut frames = vec![V3Reply::Frame(original.clone())];
    if bind_initial {
        assert!(diagnostic_step);
        // Bind all 52 identities before the transfer's own fresh parent capture.
        frames.insert(0, V3Reply::Frame(original.clone()));
    }
    if boundary_case {
        let after_drag = if case == Case::CompleteAfterDrag {
            // Shared terminal appearance is a local mock, not live Free Cell evidence.
            crate::capture::decode_png(include_bytes!(
                "../../../../tests/fixtures/pyramid-objectives/fours-complete.png"
            ))
            .unwrap()
        } else {
            let mut frame = original.clone();
            for pixel in frame.pixels.as_chunks_mut::<4>().0 {
                *pixel = [0, 0, 0, 255];
            }
            frame
        };
        frames.push(V3Reply::Frame(after_drag));
    }
    if case.relative_probe() {
        // Original binding, two independent parent confirmations, fresh pre-input parent.
        frames = (0..4).map(|_| V3Reply::Frame(original.clone())).collect();
        if case == Case::RelativeCaptureFailure {
            frames.push(V3Reply::Truncated);
        } else if !matches!(case, Case::RelativeStopAfterAck | Case::StopDuringHeldWait) {
            let mut after = original.clone();
            after.pixels[..4].copy_from_slice(&[17, 29, 43, 255]);
            frames.push(V3Reply::Frame(after));
        }
    }
    let count = frames.len();
    let (address, tcp) = v3_server_with_read_timeout(vec![frames], Duration::from_secs(60));
    let _backend = TestCaptureBackend::tcp_mode(address, CaptureMode::PersistentBgrV3);
    let latest = LatestFrameSlot::default();
    let (events, receiver) = mpsc::channel();
    let sink = WorkerEventSink {
        events,
        latest_frame: latest.clone(),
        capture_context: Mutex::new(None),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let _wire_scope = crate::worker::qmp_wire::WorkerScope::install(&sink);
    let _recording = RecordingRestore(WORKER_IMAGE_DIAGNOSTICS.with(|slot| slot.replace(None)));
    let cancel_scope = tcp_capture::CancellationScope::install(Arc::clone(&cancel));
    let capture_scope = tcp_capture::WorkerCaptureScope::install(&sink);
    sink.set_capture_context(Some(id.context.clone()));
    latest.reconnaissance.select(&id.context, &cancel).unwrap();
    if !case.relative_probe() {
        latest
            .reconnaissance
            .queue_reconnaissance(&id.context, &cancel, || Ok(()))
            .unwrap();
    }
    latest
        .freecell_route
        .lock()
        .unwrap()
        .prepare(id.clone(), prefix_report(deal.clone()))
        .unwrap();
    // Keep the real handle alive: dropping it requests STOP even without a worker thread.
    let mut queued_handle = None;
    let mut command_scope = None;
    let mut drag_mode = route::DragMode::Absolute;
    let run_id = if case.relative_probe() {
        let (command_tx, command_rx) = mpsc::channel();
        let (_, event_rx) = mpsc::channel();
        let handle = WorkerHandle {
            command_tx,
            event_rx,
            latest_frame: latest.clone(),
            join: None,
            cancel_requested: Arc::clone(&cancel),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
        };
        let reserved = handle
            .execute_freecell_route(
                id.clone(),
                true,
                route::AutoMode::ExecuteRecorded,
                route::DragMode::Relative,
                Duration::ZERO,
            )
            .unwrap();
        let command = command_rx.try_recv().unwrap();
        command_scope = Some(crate::worker::qmp_wire::CommandScope::install(&command));
        let WorkerCommand::ExecuteFreeCellRoute {
            plan: queued_plan,
            run_id: queued_run,
            single_step,
            drag_mode: queued_mode,
            drag_timing: queued_timing,
            settle,
        } = command
        else {
            panic!("expected one exact Free Cell playback command");
        };
        assert_eq!(queued_plan, id);
        assert_eq!(queued_run, reserved);
        assert!(single_step);
        assert_eq!(settle, Duration::ZERO);
        assert_eq!(queued_mode, route::DragMode::Relative);
        assert_eq!(queued_timing, route::DiagnosticDragTiming::TwoSeconds);
        drag_mode = queued_mode;
        assert!(matches!(
            command_rx.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
        queued_handle = Some(handle);
        reserved
    } else {
        latest.freecell_route.lock().unwrap().reserve(&id).unwrap()
    };
    let server = qmp_server(
        &root.path(),
        case,
        latest.clone(),
        id.context.clone(),
        Arc::clone(&cancel),
        original,
    );
    let trace = pyramid_stop::StopTrace::default();
    let qmp = QmpClient::connect(&root.path()).unwrap();
    let mut native = Native {
        plan: &id,
        run_id,
        qmp,
        events: &sink,
        cancel: &cancel,
        trace: &trace,
        latest: None,
        first: None,
        frozen: None,
        stage: None,
        capture_serial: 0,
        first_rejection: None,
        last_rejection: None,
        counts: (0, 0),
        settle: Duration::ZERO,
        completion_bound: false,
        completion_observed: false,
        diagnostic_step,
        drag_mode,
        drag_timing,
        probe_delivered: false,
        probe_evidence: None,
    };
    if bind_initial {
        native.bind(&deal).unwrap();
        assert!(native.completion_bound);
    }
    let retired_seen = Arc::new(AtomicBool::new(false));
    let retired_flag = Arc::clone(&retired_seen);
    let stopped_latest = latest.clone();
    let stopped_context = id.context.clone();
    let stopped_cancel = Arc::clone(&cancel);
    let _probe_trace = crate::single_step_telemetry::Scope::freecell(
        json!({"native_case":format!("{case:?}")}),
        move |line| {
            if line.contains("\"event\":\"motion-probe-input-retired\"") {
                let store = stopped_latest.freecell_route.lock().unwrap();
                assert!(store.live.is_none() && store.running.is_some());
                drop(store);
                retired_flag.store(true, Ordering::SeqCst);
                if case == Case::RelativeStopAfterAck {
                    stopped_latest
                        .reconnaissance
                        .operator_stop(&stopped_context, &stopped_cancel);
                }
            }
        },
    );
    let mut probe_state = State::new(&deal);
    let result = if case.relative_probe() {
        route::movement_probe(
            &mut probe_state,
            &deal,
            &[action],
            route::AutoMode::ExecuteRecorded,
            &mut native,
        )
        .map(|completion| match completion {
            route::Completion::MovementProbe => route::TransferOutcome::MovementProbe,
            route::Completion::ChallengeComplete => route::TransferOutcome::ChallengeComplete,
            other => panic!("unexpected probe completion {other:?}"),
        })
    } else {
        native.transfer(&parent, action)
    };
    match case {
        Case::Both | Case::PlayDestination | Case::Relative => {
            let outcome = result.unwrap();
            assert_eq!(
                outcome,
                if case.relative_probe() {
                    route::TransferOutcome::MovementProbe
                } else {
                    route::TransferOutcome::Delivered
                }
            );
            assert_eq!(native.frozen, Some((action, from, to)));
            assert_eq!(native.counts, (1, 1));
            if case.relative_probe() {
                assert!(native.completion_bound);
                assert_eq!(
                    native.capture_serial, 5,
                    "binding, two readiness frames, pre-input parent and one retired after-frame"
                );
                assert_eq!(native.drag_mode, route::DragMode::Relative);
                assert!(!native.completion_observed);
                let store = latest.freecell_route.lock().unwrap();
                assert_eq!(store.running.as_ref(), Some(&(id.clone(), run_id)));
                assert!(store.live.is_none() && store.progress.is_none());
                assert_eq!(probe_state.cursor, 0);
                assert!(route::same_physical(&probe_state.position, &parent));
                assert!(retired_seen.load(Ordering::SeqCst));
                drop(store);
                let receipt = native.probe_evidence.as_ref().unwrap();
                assert_eq!(receipt.capture_serial, 5);
                let directory = receipt.directory.as_ref().unwrap();
                let retained = crate::capture::decode_png(
                    &std::fs::read(directory.join("last-observed.png")).unwrap(),
                )
                .unwrap();
                assert_eq!(retained.pixels, native.latest.as_ref().unwrap().pixels);
                assert_eq!(&retained.pixels[..4], &[17, 29, 43, 255]);
                let metadata: Value = serde_json::from_slice(
                    &std::fs::read(directory.join("error-state.json")).unwrap(),
                )
                .unwrap();
                assert_eq!(
                    metadata["details"]["artifact_kind"],
                    "successful motion-probe acquisition evidence"
                );
                latest
                    .freecell_route
                    .lock()
                    .unwrap()
                    .finish(
                        &id,
                        run_id,
                        Some(probe_state),
                        Some(route::Completion::MovementProbe),
                    )
                    .unwrap();
                assert!(!latest.freecell_route.lock().unwrap().available(&id));
            }
        }
        Case::RelativeCaptureFailure | Case::RelativeStopAfterAck => {
            let error = result.unwrap_err();
            assert_eq!(native.counts, (1, 1));
            assert!(native.probe_delivered && retired_seen.load(Ordering::SeqCst));
            assert_eq!(native.capture_serial, 4);
            assert_eq!(native.details()["motion_probe_delivered"], true);
            assert!(native.probe_evidence.is_none());
            assert_eq!(probe_state.cursor, 0);
            if case == Case::RelativeStopAfterAck {
                assert_eq!(
                    trace.termination(true, true),
                    crate::pyramid_workflow_model::RunTermination::OperatorStopped
                );
            } else {
                assert!(!error.is_empty());
            }
            let mut store = latest.freecell_route.lock().unwrap();
            assert!(store.live.is_none());
            assert_eq!(store.running.as_ref(), Some(&(id.clone(), run_id)));
            if case == Case::RelativeStopAfterAck {
                // The real worker's final authority check invalidates STOP before finish.
                store.revoke();
            }
            store.finish(&id, run_id, None, None).unwrap();
            assert_eq!(store.available(&id), case == Case::RelativeCaptureFailure);
        }
        Case::Stop => {
            assert!(result.is_err());
            assert_eq!(native.counts, (0, 0));
            assert_eq!(
                trace.termination(true, true),
                crate::pyramid_workflow_model::RunTermination::OperatorStopped
            );
        }
        Case::Revoke => {
            assert!(result.unwrap_err().contains("revoked"));
            assert_eq!(native.counts, (0, 0));
            assert_eq!(
                trace.termination(true, true),
                crate::pyramid_workflow_model::RunTermination::Failed
            );
        }
        Case::DownError => {
            assert!(result.unwrap_err().contains("Uncertain"));
            assert_eq!(native.counts, (1, 0));
            assert_eq!(
                trace.termination(true, false),
                crate::pyramid_workflow_model::RunTermination::Failed
            );
        }
        Case::CompleteAfterDrag => {
            assert_eq!(result.unwrap(), route::TransferOutcome::Delivered);
            assert_eq!(
                native.capture_serial, 2,
                "no post-drag capture inside transfer"
            );
            assert!(matches!(
                native
                    .observe(&[Candidate {
                        cursor: 1,
                        position: parent.apply(action).unwrap()
                    }])
                    .unwrap(),
                route::Observation::ChallengeComplete
            ));
            assert!(native.completion_observed);
            assert_eq!(native.counts, (1, 1));
            let store = latest.freecell_route.lock().unwrap();
            assert!(store.live.is_none());
            assert_eq!(store.running.as_ref(), Some(&(id.clone(), run_id)));
        }
        Case::UnsupportedAfterDrag => {
            assert_eq!(result.unwrap(), route::TransferOutcome::Delivered);
            assert_eq!(
                native.capture_serial, 2,
                "no post-drag capture inside transfer"
            );
            assert!(matches!(
                native
                    .observe(&[Candidate {
                        cursor: 1,
                        position: parent.apply(action).unwrap()
                    }])
                    .unwrap(),
                route::Observation::Unmatched
            ));
            assert!(native.first_rejection.is_some() && native.last_rejection.is_some());
            assert!(!native.completion_observed);
            assert_eq!(native.counts, (1, 1));
            assert_eq!(
                latest
                    .freecell_route
                    .lock()
                    .unwrap()
                    .live
                    .as_ref()
                    .unwrap()
                    .state
                    .cursor,
                0
            );
        }
        Case::StopDuringHeldWait | Case::AbsoluteStopDuringHeldWait => {
            assert!(result.is_err());
            assert!(cancel.load(Ordering::SeqCst));
            assert_eq!(native.counts, (1, 0));
            assert_eq!(
                native.capture_serial,
                if case.relative_probe() { 4 } else { 1 }
            );
            assert!(!native.completion_observed);
            assert_eq!(
                trace.termination(true, true),
                crate::pyramid_workflow_model::RunTermination::OperatorStopped
            );
            let mut store = latest.freecell_route.lock().unwrap();
            assert_eq!(store.running.as_ref(), Some(&(id.clone(), run_id)));
            let live = store
                .live
                .as_ref()
                .expect("exact owner survives until finish");
            assert_eq!(live.state.cursor, 0);
            assert!(route::same_physical(&live.state.position, &parent));
            store.revoke();
            store.finish(&id, run_id, None, None).unwrap();
            assert!(store.running.is_none() && store.live.is_none());
            assert!(!store.available(&id));
        }
    }
    if !case.relative_probe() && case != Case::Both && !case.stops_during_held_wait() {
        let mut store = latest.freecell_route.lock().unwrap();
        if case == Case::Stop {
            store.revoke();
        }
        store.reject(&id, run_id).unwrap();
        assert!(store.running.is_none() && store.live.is_none());
        assert_eq!(
            store.available(&id),
            !matches!(case, Case::Stop | Case::Revoke | Case::CompleteAfterDrag)
        );
    }
    drop(native);
    drop(capture_scope);
    drop(cancel_scope);
    let (requests, captures) = server.join().unwrap();
    assert_eq!(
        captures, 1,
        "one fresh VM/pointer probe before the drag only"
    );
    assert_eq!(tcp.finish(), count);
    let commands: Vec<_> = requests
        .iter()
        .map(|r| r["execute"].as_str().unwrap())
        .collect();
    let mut expected = vec!["qmp_capabilities", "query-status", "query-mice"];
    if diagnostic_step && !matches!(case, Case::Stop | Case::Revoke) {
        expected.extend([
            "query-mice",
            "query-mice",
            "human-monitor-command",
            "query-mice",
            "input-send-event",
        ]);
        if case.relative_probe() {
            expected.extend([
                "query-mice",
                "query-mice",
                "human-monitor-command",
                "query-mice",
            ]);
        }
        expected.extend(["query-mice", "input-send-event"]);
        if !case.stops_during_held_wait() {
            expected.extend(std::iter::repeat_n(
                "input-send-event",
                if case.relative_probe() { 11 } else { 5 },
            ));
        } else {
            expected.extend([
                "human-monitor-command",
                "input-send-event",
                "qmp_capabilities",
                "query-mice",
                "human-monitor-command",
                "query-mice",
                "input-send-event",
            ]);
        }
        if case.relative_probe() && !case.stops_during_held_wait() {
            expected.extend(["query-mice", "human-monitor-command", "query-mice"]);
        }
    } else if matches!(case, Case::Both | Case::DownError) {
        expected.extend(["input-send-event"; 4]);
        if case == Case::DownError {
            expected.extend(["input-send-event", "qmp_capabilities", "input-send-event"]);
        }
    }
    assert_eq!(commands, expected);
    let inputs: Vec<_> = requests
        .iter()
        .filter(|r| r["execute"] == "input-send-event")
        .collect();
    if case.relative_probe() && !case.stops_during_held_wait() {
        assert_eq!(
            inputs.len(),
            13,
            "one source, one DOWN, ten tiny REL updates, one UP; no extra click"
        );
        movement(inputs[0], from);
        button(inputs[1], true);
        for request in &inputs[2..12] {
            assert_eq!(
                request["arguments"],
                json!({"events":[
                    {"type":"rel","data":{"axis":"x","value":(to.x-from.x).signum()*2}},
                    {"type":"rel","data":{"axis":"y","value":(to.y-from.y).signum()*2}}
                ]})
            );
        }
        button(inputs[12], false);
    } else if case.stops_during_held_wait() {
        assert_eq!(inputs.len(), 4);
        movement(inputs[0], from);
        button(inputs[1], true);
        for request in inputs.iter().skip(2) {
            button(request, false);
        }
    } else if diagnostic_step
        && matches!(
            case,
            Case::Both
                | Case::PlayDestination
                | Case::CompleteAfterDrag
                | Case::UnsupportedAfterDrag
        )
    {
        assert_eq!(
            inputs.len(),
            7,
            "source then one held DOWN/source-lift/midpoint/destination-lift/destination/UP gesture; no extra click"
        );
        if case != Case::PlayDestination {
            assert_eq!(from.x, 233);
            assert!((668..=673).contains(&from.y));
            assert_eq!(to, PixelPoint::new(234, 203));
        } else {
            assert!(matches!(action.to, Pile::Cascade(_)));
        }
        movement(inputs[0], from);
        button(inputs[1], true);
        movement(inputs[2], PixelPoint::new(from.x, from.y - 45));
        movement(
            inputs[3],
            PixelPoint::new((from.x + to.x) / 2, (from.y + to.y) / 2),
        );
        movement(inputs[4], PixelPoint::new(to.x, to.y - 45));
        movement(inputs[5], to);
        button(inputs[6], false);
    } else if case == Case::Both || case == Case::DownError {
        assert_eq!(inputs.len(), if case == Case::Both { 4 } else { 6 });
        movement(inputs[0], from);
        button(inputs[1], true);
        movement(inputs[2], to);
        for request in inputs.iter().skip(3) {
            button(request, false);
        }
    } else {
        assert!(inputs.is_empty());
    }
    if diagnostic_step
        && (matches!(case, Case::Both | Case::PlayDestination)
            || case.relative_probe()
            || case.stops_during_held_wait())
    {
        let mut input_connections = Vec::new();
        let writes: Vec<_> = receiver
            .try_iter()
            .filter_map(|tagged| {
                let WorkerEvent::QmpWireLog {
                    metadata,
                    packet_line,
                } = tagged.event
                else {
                    return None;
                };
                let raw = packet_line.lines().last().unwrap();
                let packet: Value = serde_json::from_str(raw).unwrap();
                if case.relative_probe() && !case.stops_during_held_wait() {
                    let metadata: Value =
                        serde_json::from_str(metadata.strip_prefix("QMP_WIRE ").unwrap()).unwrap();
                    assert_eq!(metadata["owner"]["diagnostic_drag_mode"], "Relative");
                    assert_eq!(metadata["owner"]["run_id"], run_id);
                    assert_eq!(metadata["owner"]["request_id"], id.request_id);
                    assert_eq!(metadata["connection_kind"], "primary");
                    let observed = requests
                        .iter()
                        .find(|request| request["id"] == packet["id"])
                        .unwrap();
                    assert_eq!(
                        &packet, observed,
                        "readable packet is the one read by the peer"
                    );
                    assert_eq!(raw, serde_json::to_string(observed).unwrap());
                }
                (packet["execute"] == "input-send-event").then(|| {
                    let metadata: Value =
                        serde_json::from_str(metadata.strip_prefix("QMP_WIRE ").unwrap()).unwrap();
                    input_connections
                        .push(metadata["connection_kind"].as_str().unwrap().to_owned());
                    metadata["wire_elapsed_us"].as_u64().unwrap()
                })
            })
            .collect();
        assert_eq!(
            writes.len(),
            if case.stops_during_held_wait() {
                4
            } else if case.relative_probe() {
                13
            } else {
                7
            }
        );
        let abs_wait_us = match drag_timing {
            route::DiagnosticDragTiming::TwoSeconds => 2_000_000,
            route::DiagnosticDragTiming::HalfSecond => 500_000,
        };
        if case.stops_during_held_wait() {
            let source_wait_us = if case.relative_probe() {
                2_000_000
            } else {
                abs_wait_us
            };
            assert!(
                writes[1] - writes[0] >= source_wait_us,
                "source-to-DOWN pacing completes before STOP in the held wait"
            );
            assert_eq!(
                input_connections,
                ["primary", "primary", "primary", "release-recovery"]
            );
        } else if case.relative_probe() {
            for pair in writes.windows(2) {
                assert!(
                    pair[1] - pair[0] >= 2_000_000,
                    "each REL held-drag phase has the requested two-second wait"
                );
            }
        } else {
            for pair in writes.windows(2) {
                assert!(
                    pair[1] - pair[0] >= abs_wait_us,
                    "all six ABS intervals use the frozen selected wait"
                );
            }
            assert!(
                writes[6] - writes[0] >= 6 * abs_wait_us,
                "ABS source through release includes all six selected waits"
            );
        }
    }
    drop(command_scope);
    drop(queued_handle);
}


/// Real TCP frame and fresh VM/pointer probe precede one explicit-target transfer.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_312_native_compound_capture_probe_input_order() {
    exercise(Case::Both, false);
}


/// Bind the authentic Sixes52, then prove only JH PLAY6 and CELL1 on fresh local pixels.
///
/// The saved gold6H decoration is unchanged. A separately labelled synthetic rank
/// obstruction on unrelated PLAY3 makes a whole-board proof fail after binding.
/// One local acknowledged drag proves transport delivery, never a live guest effect.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_3114_native_sixes_binds52_and_proves_only_jh_source_and_empty_cell() {
    let original = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/freecell-sixes-decorative-20261111.png"
    ))
    .unwrap();
    let deal = crate::freecell_recon::recognise(
        &original,
        Metadata::new("native-sixes-incident", "1920x1080", "unit").unwrap(),
    )
    .unwrap();
    let parent = Position::initial(&deal);
    let action = RecordedMove {
        card_id: xsar::games::freecell::rules::card_id(Card::new(
            Rank::Jack,
            xsar::games::cards::Suit::Hearts,
        )),
        from: Pile::Cascade(5),
        to: Pile::Cell(0),
        is_auto: false,
    };
    assert_eq!(parent.column(5).unwrap().last(), Some(&action.card_id));
    assert!(parent.automatic().is_none());
    assert!(parent.apply(action).unwrap().automatic().is_none());
    let replay = xsar::games::freecell::replay::verify_prefix_with_control(
        &deal,
        &[action],
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!((replay.deliberate, replay.automatic), (1, 0));
    let report = Arc::new(freecell_calculation::Report {
        path: PathBuf::from("native-sixes-JH-to-CELL1-prefix.json"),
        deal: deal.clone(),
        bytes: Vec::new(),
        paused: Duration::ZERO,
        outcome: Outcome::Partial {
            moves: Box::new([action]),
            stats: Default::default(),
            deliberate_moves: replay.deliberate,
            automatic_moves: replay.automatic,
            foundation_cards: 0,
        },
    });
    let mut preinput = original.clone();
    // Synthetic obstruction of PLAY3's exposed QH rank; not part of the native incident.
    for y in 656..681 {
        for x in 604..632 {
            let offset =
                usize::try_from(y).unwrap() * preinput.stride + usize::try_from(x).unwrap() * 4;
            preinput.pixels[offset..offset + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    assert!(position::verify(&preinput, &parent, &mut || Ok(())).is_err());
    let root = Socket::new();
    let mut id = plan(3114);
    id.context = ControllerContext::new(
        root.path(),
        GameMode::FreeCell,
        SolvingStrategy::ShortestPath,
        3114,
    );
    let (address, tcp) = v3_server(vec![vec![
        V3Reply::Frame(original),
        V3Reply::Frame(preinput.clone()),
    ]]);
    let _backend = TestCaptureBackend::tcp_mode(address, CaptureMode::PersistentBgrV3);
    let latest = LatestFrameSlot::default();
    let (events, _receiver) = mpsc::channel();
    let sink = WorkerEventSink {
        events,
        latest_frame: latest.clone(),
        capture_context: Mutex::new(None),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let _recording = RecordingRestore(WORKER_IMAGE_DIAGNOSTICS.with(|slot| slot.replace(None)));
    let cancel_scope = tcp_capture::CancellationScope::install(Arc::clone(&cancel));
    let capture_scope = tcp_capture::WorkerCaptureScope::install(&sink);
    sink.set_capture_context(Some(id.context.clone()));
    latest.reconnaissance.select(&id.context, &cancel).unwrap();
    latest
        .reconnaissance
        .queue_reconnaissance(&id.context, &cancel, || Ok(()))
        .unwrap();
    let run_id = {
        let mut store = latest.freecell_route.lock().unwrap();
        store.prepare(id.clone(), report).unwrap();
        store.reserve(&id).unwrap()
    };
    let server = qmp_server(
        &root.path(),
        Case::Both,
        latest.clone(),
        id.context.clone(),
        Arc::clone(&cancel),
        preinput,
    );
    let trace = pyramid_stop::StopTrace::default();
    let mut native = Native {
        plan: &id,
        run_id,
        qmp: QmpClient::connect(&root.path()).unwrap(),
        events: &sink,
        cancel: &cancel,
        trace: &trace,
        latest: None,
        first: None,
        frozen: None,
        stage: None,
        capture_serial: 0,
        first_rejection: None,
        last_rejection: None,
        counts: (0, 0),
        settle: Duration::ZERO,
        completion_bound: false,
        completion_observed: false,
        diagnostic_step: false,
        drag_mode: route::DragMode::Absolute,
        drag_timing: route::DiagnosticDragTiming::TwoSeconds,
        probe_delivered: false,
        probe_evidence: None,
    };
    native.bind(&deal).unwrap();
    assert!(native.completion_bound);
    assert_eq!(native.capture_serial, 1);
    assert_eq!(native.counts, (0, 0));
    assert_eq!(
        native.transfer(&parent, action).unwrap(),
        route::TransferOutcome::Delivered
    );
    let (frozen, from, to) = native.frozen.unwrap();
    assert_eq!(frozen, action);
    assert_eq!(from.x, 1195);
    assert!((615..=620).contains(&from.y));
    assert_eq!(to, PixelPoint::new(234, 203));
    assert_eq!(native.capture_serial, 2);
    assert_eq!(native.counts, (1, 1));
    assert!(!native.completion_observed);
    assert!(native.first_rejection.is_none() && native.last_rejection.is_none());
    latest
        .freecell_route
        .lock()
        .unwrap()
        .reject(&id, run_id)
        .unwrap();
    drop(native);
    drop(capture_scope);
    drop(cancel_scope);
    let (requests, probes) = server.join().unwrap();
    assert_eq!(probes, 1);
    assert_eq!(tcp.finish(), 2);
    let commands: Vec<_> = requests
        .iter()
        .map(|r| r["execute"].as_str().unwrap())
        .collect();
    assert_eq!(
        commands,
        [
            "qmp_capabilities",
            "query-status",
            "query-mice",
            "input-send-event",
            "input-send-event",
            "input-send-event",
            "input-send-event",
        ]
    );
    movement(&requests[3], from);
    button(&requests[4], true);
    movement(&requests[5], to);
    button(&requests[6], false);
}


/// Both explicit ABS timings are copied into one queued command and remain frozen.
#[test]
fn freecell_3112_each_abs_timing_is_frozen_before_queue() {
    for timing in [
        route::DiagnosticDragTiming::TwoSeconds,
        route::DiagnosticDragTiming::HalfSecond,
    ] {
        let (command_tx, command_rx) = mpsc::channel();
        let (_, event_rx) = mpsc::channel();
        let latest_frame = LatestFrameSlot::default();
        let id = plan(3112);
        latest_frame
            .freecell_route
            .lock()
            .unwrap()
            .prepare(id.clone(), report())
            .unwrap();
        let handle = WorkerHandle {
            command_tx,
            event_rx,
            latest_frame,
            join: None,
            cancel_requested: Arc::new(AtomicBool::new(false)),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
        };
        assert!(
            handle
                .execute_freecell_route_with_timing(
                    id.clone(),
                    true,
                    route::AutoMode::ObserveGame,
                    route::DragMode::Relative,
                    route::DiagnosticDragTiming::HalfSecond,
                    Duration::ZERO,
                )
                .unwrap_err()
                .contains("requires ABS")
        );
        assert!(handle.freecell_route_available(&id));
        assert!(matches!(
            command_rx.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
        let run_id = handle
            .execute_freecell_route_with_timing(
                id.clone(),
                true,
                route::AutoMode::ObserveGame,
                route::DragMode::Absolute,
                timing,
                Duration::ZERO,
            )
            .unwrap();
        let command = command_rx.try_recv().unwrap();
        let WorkerCommand::ExecuteFreeCellRoute {
            plan,
            run_id: queued_run,
            single_step,
            drag_mode,
            drag_timing,
            settle,
        } = command
        else {
            panic!("expected one frozen ABS request")
        };
        assert_eq!(plan, id);
        assert_eq!(queued_run, run_id);
        assert!(single_step);
        assert_eq!(drag_mode, route::DragMode::Absolute);
        assert_eq!(drag_timing, timing);
        assert_eq!(settle, Duration::ZERO);
        let other_timing = match timing {
            route::DiagnosticDragTiming::TwoSeconds => route::DiagnosticDragTiming::HalfSecond,
            route::DiagnosticDragTiming::HalfSecond => route::DiagnosticDragTiming::TwoSeconds,
        };
        assert!(
            handle
                .execute_freecell_route_with_timing(
                    id,
                    true,
                    route::AutoMode::ObserveGame,
                    route::DragMode::Absolute,
                    other_timing,
                    Duration::ZERO,
                )
                .is_err()
        );
        assert_eq!(drag_timing, timing);
        assert!(matches!(
            command_rx.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
    }
}


/// ABS diagnostics send seven exact packets with six two-second gaps at an upper slot.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_3116_native_abs_single_step_has_six_two_second_gaps_and_lift_midpoint_path() {
    exercise(Case::Both, true);
}


/// The fast ABS button uses six half-second gaps and the same native lift/midpoint path.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_3116_native_abs_half_second_has_six_selected_gaps_and_lift_midpoint_path() {
    exercise_with_timing(Case::Both, true, route::DiagnosticDragTiming::HalfSecond);
}


/// A native PLAY transfer preserves its proved corner as the final exact destination.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_3116_native_abs_play_target_has_six_half_second_gaps_and_lift_midpoint_path() {
    exercise_with_timing(
        Case::PlayDestination,
        true,
        route::DiagnosticDragTiming::HalfSecond,
    );
}


/// STOP after ABS DOWN ACK permits only old/fresh safety UP before destination motion.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_3112_native_abs_stop_during_held_wait_releases_without_motion_or_replay() {
    exercise(Case::AbsoluteStopDuringHeldWait, true);
}


/// STOP during the half-second held wait permits release recovery without destination motion.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_3112_native_abs_half_second_stop_releases_without_motion_or_replay() {
    exercise_with_timing(
        Case::AbsoluteStopDuringHeldWait,
        true,
        route::DiagnosticDragTiming::HalfSecond,
    );
}


/// A queued REL mode binds all52, selects one handler and restores it without another click.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_317_native_rel_single_step_uses_queued_mode_and_restores_without_extra_click() {
    exercise(Case::Relative, true);
}


/// Ordinary effect observation retires a newly reported terminal after drag release.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_317_native_post_drag_terminal_retires_without_extra_input() {
    exercise(Case::CompleteAfterDrag, true);
}


/// Unknown ordinary effect pixels cannot advance the route after an acknowledged drag.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_317_native_post_drag_unknown_does_not_accept_effect() {
    exercise(Case::UnsupportedAfterDrag, true);
}


/// STOP during REL's intentional held wait permits only same-target old/fresh safety UP.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_3110_native_rel_stop_during_held_wait_releases_without_motion_or_restore() {
    exercise(Case::StopDuringHeldWait, true);
}


/// Bind a real native Free Cell frame, then inject only the shared completion appearance.
///
/// The existing common terminal fixture proves the passive transport stop path;
/// it is not a claim of a live Free Cell terminal capture or numeric objective.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_315_native_terminal_preempts_readiness_and_preinput_without_qmp_input() {
    for preinput in [false, true] {
        terminal_boundary(preinput, false, Recording::Disabled);
    }
}


/// A terminal image without an independently bound original deal supplies no authority.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_315_native_unbound_terminal_refuses_without_qmp_input() {
    terminal_boundary(false, true, Recording::Enabled);
}


/// Enabled GAME WIN recording saves the acquired terminal only, with no additional I/O.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_315_native_terminal_records_only_acquired_terminal_frame() {
    terminal_boundary(false, false, Recording::Enabled);
}


/// A PNG write error preserves the positive terminal fact after irrevocable input retirement.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_315_native_terminal_recording_failure_keeps_retired_authority_and_original_error() {
    terminal_boundary(false, false, Recording::InvalidDestination);
}


/// Thread-owned recording destination, without changing runtime startup configuration.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Recording {
    /// The existing disabled policy must write no images.
    Disabled,
    /// A fresh private directory accepts the cached terminal PNG.
    Enabled,
    /// A pre-existing regular file forces an authentic destination validation failure.
    InvalidDestination,
}


/// Restore only this test thread's recording policy, including on assertion unwinding.
struct RecordingRestore(Option<WorkerImageDiagnostics>);


impl Drop for RecordingRestore {
    /// Other concurrently running tests keep their independently owned recorder.
    fn drop(&mut self) {
        WORKER_IMAGE_DIAGNOSTICS.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}


/// Exercise exact native source binding and immediate store retirement on local peers.
fn terminal_boundary(preinput: bool, unbound: bool, recording: Recording) {
    let root = Socket::new();
    let images = root.0.join("images");
    let blocked_contents = b"owned recording destination must not be overwritten";
    if recording == Recording::InvalidDestination {
        std::fs::write(&images, blocked_contents).unwrap();
    } else {
        std::fs::create_dir(&images).unwrap();
    }
    let _recording = RecordingRestore(WORKER_IMAGE_DIAGNOSTICS.with(|slot| {
        slot.replace(Some(WorkerImageDiagnostics::new(ImageDiagnostics {
            started_at: Instant::now(),
            directory: (recording != Recording::Disabled).then(|| images.clone()),
        })))
    }));
    let mut id = plan(315);
    id.context = ControllerContext::new(
        root.path(),
        GameMode::FreeCell,
        SolvingStrategy::ShortestPath,
        315,
    );
    let original = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/freecell-challenge-highlight-20261109.png"
    ))
    .unwrap();
    let terminal = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-objectives/fours-complete.png"
    ))
    .unwrap();
    let deal = crate::freecell_recon::recognise(
        &original,
        Metadata::new("native315", "highlighted", "unit").unwrap(),
    )
    .unwrap();
    let parent = Position::initial(&deal);
    let expected_terminal = terminal.clone();
    let frames = if unbound {
        vec![V3Reply::Frame(terminal)]
    } else {
        vec![V3Reply::Frame(original.clone()), V3Reply::Frame(terminal)]
    };
    let count = frames.len();
    let (address, tcp) = v3_server(vec![frames]);
    let _backend = TestCaptureBackend::tcp_mode(address, CaptureMode::PersistentBgrV3);
    let latest = LatestFrameSlot::default();
    let (events, _receiver) = mpsc::channel();
    let sink = WorkerEventSink {
        events,
        latest_frame: latest.clone(),
        capture_context: Mutex::new(None),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_scope = tcp_capture::CancellationScope::install(Arc::clone(&cancel));
    let capture_scope = tcp_capture::WorkerCaptureScope::install(&sink);
    sink.set_capture_context(Some(id.context.clone()));
    latest.reconnaissance.select(&id.context, &cancel).unwrap();
    latest
        .reconnaissance
        .queue_reconnaissance(&id.context, &cancel, || Ok(()))
        .unwrap();
    let run_id = {
        let mut store = latest.freecell_route.lock().unwrap();
        store
            .prepare(id.clone(), prefix_report(deal.clone()))
            .unwrap();
        store.reserve(&id).unwrap()
    };
    let server = qmp_server(
        &root.path(),
        Case::Both,
        latest.clone(),
        id.context.clone(),
        Arc::clone(&cancel),
        original,
    );
    let trace = pyramid_stop::StopTrace::default();
    let mut native = Native {
        plan: &id,
        run_id,
        qmp: QmpClient::connect(&root.path()).unwrap(),
        events: &sink,
        cancel: &cancel,
        trace: &trace,
        latest: None,
        first: None,
        frozen: None,
        stage: None,
        capture_serial: 0,
        first_rejection: None,
        last_rejection: None,
        counts: (0, 0),
        settle: Duration::ZERO,
        completion_bound: false,
        completion_observed: false,
        diagnostic_step: false,
        drag_mode: route::DragMode::Absolute,
        drag_timing: route::DiagnosticDragTiming::TwoSeconds,
        probe_delivered: false,
        probe_evidence: None,
    };
    if unbound {
        assert!(native.bind(&deal).is_err());
        assert!(!native.completion_bound);
        assert!(
            latest
                .freecell_route
                .lock()
                .unwrap()
                .check(&id, run_id)
                .is_ok()
        );
        latest
            .freecell_route
            .lock()
            .unwrap()
            .reject(&id, run_id)
            .unwrap();
    } else {
        native.bind(&deal).unwrap();
        assert!(native.completion_bound);
        if preinput {
            let action = parent
                .automatic()
                .expect("native fixture has a root automatic transfer");
            assert_eq!(
                native.transfer(&parent, action).unwrap(),
                route::TransferOutcome::ChallengeComplete
            );
        } else if recording == Recording::InvalidDestination {
            let error = native
                .observe(&[Candidate {
                    cursor: 0,
                    position: parent,
                }])
                .unwrap_err();
            assert_eq!(
                error,
                format!(
                    "GAME WIN entry PNG could not be saved: Image diagnostics destination is not a real directory: {}; no further transition input authorised",
                    images.display()
                )
            );
            assert!(native.completion_observed);
            assert_eq!(native.details()["game_reported_challenge_complete"], true);
        } else {
            assert_eq!(
                native
                    .observe(&[Candidate {
                        cursor: 0,
                        position: parent
                    }])
                    .unwrap(),
                route::Observation::ChallengeComplete
            );
        }
        assert!(native.active().is_err());
        let mut store = latest.freecell_route.lock().unwrap();
        assert!(store.live.is_none());
        assert_eq!(store.running, Some((id.clone(), run_id)));
        store
            .finish(
                &id,
                run_id,
                Some(State::new(&deal)),
                (recording != Recording::InvalidDestination)
                    .then_some(route::Completion::ChallengeComplete),
            )
            .unwrap();
        assert!(store.running.is_none() && !store.available(&id));
    }
    assert_eq!(native.counts, (0, 0));
    assert_eq!(native.capture_serial, u64::try_from(count).unwrap());
    let recorded: Vec<_> = if recording == Recording::InvalidDestination {
        assert!(images.is_file());
        assert_eq!(std::fs::read(&images).unwrap(), blocked_contents);
        Vec::new()
    } else {
        std::fs::read_dir(&images)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect()
    };
    if recording == Recording::Enabled && !unbound {
        assert!(native.completion_observed);
        assert_eq!(recorded.len(), 1);
        let saved = crate::capture::decode_png(&std::fs::read(&recorded[0]).unwrap()).unwrap();
        assert_eq!(
            (saved.width, saved.height),
            (expected_terminal.width, expected_terminal.height)
        );
        assert!(
            saved.pixels == expected_terminal.pixels,
            "recorded terminal pixels differ"
        );
    } else {
        assert!(recorded.is_empty());
    }
    drop(native);
    drop(capture_scope);
    drop(cancel_scope);
    let (requests, captures) = server.join().unwrap();
    assert_eq!(
        captures, 0,
        "no VM probe is required after the terminal frame"
    );
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0]["execute"], "qmp_capabilities");
    assert_eq!(tcp.finish(), count);
}


/// STOP after probing sends no input packet.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_312_native_stop_after_probe() {
    exercise(Case::Stop, false);
}


/// Revocation fences a cloned adapter before its next delivery.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_312_native_revoke_after_probe() {
    exercise(Case::Revoke, false);
}


/// Failed down is never replayed; only existing release-UP recovery remains.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_312_native_uncertain_down_release_only() {
    exercise(Case::DownError, false);
}


/// A post-ACK capture error cannot re-enable or replay the retired motion probe.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_318_native_probe_capture_failure_keeps_ack_and_retired_owner() {
    exercise(Case::RelativeCaptureFailure, true);
}


/// STOP after successful delivery and retirement prevents the read-only after-frame.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_318_native_probe_stop_after_ack_skips_capture_and_all_further_input() {
    exercise(Case::RelativeStopAfterAck, true);
}


/// Synthetic retained-pixel boards for a recorded AC transfer followed by AUTO 2C.
struct ClubFollowup {
    /// All 52 original identities remain independently readable before input.
    initial: CapturedFrame,
    /// Only the explicit AC transfer has occurred; the required follow-up is absent.
    explicit: CapturedFrame,
    /// Both recorded edges have occurred and every physical pile matches.
    combined: CapturedFrame,
    /// Replay-checked transport fixture, with one deliberate edge still pending.
    report: Arc<freecell_calculation::Report>,
}


/// Copy bounded native fixture rectangles; composition never claims captured gameplay.
fn fixture_blit(
    destination: &mut CapturedFrame,
    source: &CapturedFrame,
    from: PixelRect,
    to: (u32, u32),
) {
    source.as_view().unwrap().checked_bounds(from).unwrap();
    destination
        .as_view()
        .unwrap()
        .checked_bounds(PixelRect::new(to.0, to.1, from.width, from.height))
        .unwrap();
    let bytes = usize::try_from(from.width).unwrap() * 4;
    for row in 0..from.height {
        let a = usize::try_from(from.y + row).unwrap() * source.stride
            + usize::try_from(from.x).unwrap() * 4;
        let b = usize::try_from(to.1 + row).unwrap() * destination.stride
            + usize::try_from(to.0).unwrap() * 4;
        destination.pixels[b..b + bytes].copy_from_slice(&source.pixels[a..a + bytes]);
    }
}


/// Expose FC13's buried 2C by exchanging it with QS, retaining the complete inventory.
///
/// Every composed frame is synthetic. Native FC18 supplies complete 2C and 4S faces;
/// the revealed 5D combines FC13's complete printed corner with a native face body.
/// These fixtures prove sequencing and recognition through local peers, never live timing.
fn club_followup_fixture() -> ClubFollowup {
    let original = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/freecell-FC13.png"
    ))
    .unwrap();
    let donor = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/freecell-FC18.png"
    ))
    .unwrap();
    let mut initial = original.clone();
    fixture_blit(
        &mut initial,
        &original,
        PixelRect::new(795, 650, 137, 53),
        (795, 384),
    );
    fixture_blit(
        &mut initial,
        &donor,
        PixelRect::new(1233, 112, 137, 183),
        (795, 650),
    );
    let deal = crate::freecell_recon::recognise(
        &initial,
        Metadata::new("synthetic-retained-pixels", "FC13-clubs", "unit").unwrap(),
    )
    .unwrap();
    let parent = Position::initial(&deal);
    position::verify(&initial, &parent, &mut || Ok(())).unwrap();
    let ace = RecordedMove {
        card_id: 0,
        from: Pile::Cascade(0),
        to: Pile::Foundation(0),
        is_auto: true,
    };
    let two = RecordedMove {
        card_id: 1,
        from: Pile::Cascade(3),
        to: Pile::Foundation(0),
        is_auto: true,
    };
    assert_eq!(parent.automatic(), Some(ace));
    let after_ace = parent.apply(ace).unwrap();
    assert_eq!(after_ace.automatic(), Some(two));
    let after_two = after_ace.apply(two).unwrap();
    assert!(after_two.automatic().is_none());
    let next = RecordedMove {
        card_id: 17,
        from: Pile::Cascade(3),
        to: Pile::Cell(0),
        is_auto: false,
    };
    // 5D reveals TS; moving 4S instead would expose AH and leave an unrecorded AUTO.
    assert!(after_two.apply(next).unwrap().automatic().is_none());
    let moves = [ace, two, next];
    let replay = xsar::games::freecell::replay::verify_prefix_with_control(
        &deal,
        &moves,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!((replay.deliberate, replay.automatic), (1, 2));
    let report = Arc::new(freecell_calculation::Report {
        path: PathBuf::from("synthetic-FC13-club-followups.json"),
        deal,
        bytes: Vec::new(),
        paused: Duration::ZERO,
        // This replay-checked transport seam makes no exhaustive maximum claim.
        outcome: Outcome::Partial {
            moves: Box::new(moves),
            stats: Default::default(),
            deliberate_moves: replay.deliberate,
            automatic_moves: replay.automatic,
            foundation_cards: 2,
        },
    });
    let mut explicit = initial.clone();
    fixture_blit(
        &mut explicit,
        &original,
        PixelRect::new(50, 597, 143, 350),
        (215, 597),
    );
    fixture_blit(
        &mut explicit,
        &donor,
        PixelRect::new(410, 543, 137, 183),
        (218, 597),
    );
    fixture_blit(
        &mut explicit,
        &original,
        PixelRect::new(218, 650, 137, 183),
        (1233, 112),
    );
    position::verify(&explicit, &after_ace, &mut || Ok(())).unwrap();
    let mut combined = explicit.clone();
    fixture_blit(
        &mut combined,
        &original,
        PixelRect::new(50, 597, 143, 350),
        (792, 597),
    );
    fixture_blit(
        &mut combined,
        &original,
        PixelRect::new(795, 650, 137, 183),
        (795, 597),
    );
    fixture_blit(
        &mut combined,
        &original,
        PixelRect::new(795, 597, 137, 53),
        (795, 597),
    );
    fixture_blit(
        &mut combined,
        &donor,
        PixelRect::new(1233, 112, 137, 183),
        (1233, 112),
    );
    position::verify(&combined, &after_two, &mut || Ok(())).unwrap();
    assert!(position::verify(&explicit, &after_two, &mut || Ok(())).is_err());
    assert!(position::verify(&combined, &after_ace, &mut || Ok(())).is_err());
    ClubFollowup {
        initial,
        explicit,
        combined,
        report,
    }
}


/// Drive the real queued worker over local peers, retaining the final parent fence.
fn recorded_followup_boundary(parent_changed: bool) {
    let fixture = club_followup_fixture();
    let geometry = position::verify_piles(
        &fixture.initial,
        &Position::initial(&fixture.report.deal),
        &[Pile::Cascade(0), Pile::Foundation(0)],
        &mut || Ok(()),
    )
    .unwrap();
    let from = geometry.point(Pile::Cascade(0)).unwrap();
    assert_eq!(from.x, 233);
    assert!((668..=673).contains(&from.y));
    let root = Socket::new();
    let mut id = plan(319);
    id.context = ControllerContext::new(
        root.path(),
        GameMode::FreeCell,
        SolvingStrategy::ShortestPath,
        319,
    );
    let mut frames: Vec<_> = (0..3)
        .map(|_| V3Reply::Frame(fixture.initial.clone()))
        .collect();
    if parent_changed {
        frames.push(V3Reply::Frame(fixture.explicit.clone()));
    } else {
        frames.extend([
            V3Reply::Frame(fixture.initial.clone()),
            V3Reply::Frame(fixture.explicit.clone()),
            V3Reply::Frame(fixture.combined.clone()),
            V3Reply::Frame(fixture.combined.clone()),
        ]);
    }
    let count = frames.len();
    let (address, tcp) = v3_server_with_read_timeout(vec![frames], Duration::from_secs(60));
    let _backend = TestCaptureBackend::tcp_mode(address, CaptureMode::PersistentBgrV3);
    let latest = LatestFrameSlot::default();
    let (events, receiver) = mpsc::channel();
    let sink = WorkerEventSink {
        events,
        latest_frame: latest.clone(),
        capture_context: Mutex::new(None),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let _wire_scope = crate::worker::qmp_wire::WorkerScope::install(&sink);
    let _recording = RecordingRestore(WORKER_IMAGE_DIAGNOSTICS.with(|slot| slot.replace(None)));
    let cancel_scope = tcp_capture::CancellationScope::install(Arc::clone(&cancel));
    let capture_scope = tcp_capture::WorkerCaptureScope::install(&sink);
    sink.set_capture_context(Some(id.context.clone()));
    latest
        .freecell_route
        .lock()
        .unwrap()
        .prepare(id.clone(), Arc::clone(&fixture.report))
        .unwrap();
    let (command_tx, command_rx) = mpsc::channel();
    let (_, event_rx) = mpsc::channel();
    let handle = WorkerHandle {
        command_tx,
        event_rx,
        latest_frame: latest.clone(),
        join: None,
        cancel_requested: Arc::clone(&cancel),
        shutdown_requested: Arc::new(AtomicBool::new(false)),
    };
    let run_id = handle
        .execute_freecell_route(
            id.clone(),
            true,
            route::AutoMode::ObserveGame,
            route::DragMode::Absolute,
            Duration::ZERO,
        )
        .unwrap();
    let command = command_rx.try_recv().unwrap();
    let _command_scope = crate::worker::qmp_wire::CommandScope::install(&command);
    let WorkerCommand::ExecuteFreeCellRoute {
        plan,
        run_id: queued_run,
        single_step,
        drag_mode,
        drag_timing,
        settle,
    } = command
    else {
        panic!("expected one exact Free Cell playback command");
    };
    assert_eq!(plan, id);
    assert_eq!(queued_run, run_id);
    assert!(single_step);
    assert_eq!(drag_mode, route::DragMode::Absolute);
    let observations = Arc::new(Mutex::new(Vec::new()));
    let observed = Arc::clone(&observations);
    let _telemetry = crate::single_step_telemetry::Scope::freecell(
        json!({"fixture":"synthetic FC13 AC plus AUTO 2C"}),
        move |line| {
            let value: Value =
                serde_json::from_str(line.strip_prefix("FREE_CELL ").unwrap()).unwrap();
            if value["detail"]["event"] == "physical-position-observation" {
                observed.lock().unwrap().push(value["detail"].clone());
            }
        },
    );
    let server = qmp_server(
        &root.path(),
        Case::Both,
        latest.clone(),
        id.context.clone(),
        Arc::clone(&cancel),
        fixture.initial,
    );
    run(
        plan,
        queued_run,
        single_step,
        (drag_mode, drag_timing),
        settle,
        &sink,
        &cancel,
    );
    let (context, progress, termination) = receiver
        .try_iter()
        .find_map(|event| match event.event {
            WorkerEvent::FreeCellRouteFinished {
                plan,
                run_id: finished_run,
                result,
                termination,
            } => {
                assert_eq!(plan, id);
                assert_eq!(finished_run, run_id);
                Some((event.context, result, termination))
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(context, Some(id.context.clone()));
    let store = latest.freecell_route.lock().unwrap();
    assert!(store.running.is_none());
    if parent_changed {
        let failure = progress.unwrap_err();
        assert!(
            failure
                .error
                .contains("PLAY 1: Exposed identity differs from physical position"),
            "{}",
            failure.error
        );
        assert_eq!(failure.details["attempted"], 0);
        assert_eq!(failure.details["acknowledged"], 0);
        assert_eq!(
            termination,
            crate::pyramid_workflow_model::RunTermination::Failed
        );
        assert!(store.live.is_none() && store.available(&id));
    } else {
        let progress =
            progress.unwrap_or_else(|failure| panic!("{}: {}", failure.error, failure.details));
        assert_eq!(
            termination,
            crate::pyramid_workflow_model::RunTermination::Completed
        );
        assert_eq!(progress.completion, Some(route::Completion::StepVerified));
        assert_eq!((progress.cursor, progress.total), (2, 3));
        assert_eq!((progress.attempted, progress.acknowledged), (1, 1));
        assert_eq!(progress.foundation_cards, 2);
        let live = store.live.as_ref().unwrap();
        let cached = store.cached.as_ref().unwrap();
        assert_eq!(live.state.cursor, 2);
        let (moves, _) = recorded_route(&cached.report.outcome).unwrap();
        assert_eq!(moves[live.state.cursor].card_id, 17);
        let expected = Position::initial(&cached.report.deal)
            .apply(moves[0])
            .unwrap()
            .apply(moves[1])
            .unwrap();
        assert!(route::same_physical(&live.state.position, &expected));
        assert!(store.available(&id));
        let observations = observations.lock().unwrap();
        let effects: Vec<_> = observations
            .iter()
            .filter(|value| value["stage"]["phase"] == "effect")
            .collect();
        assert_eq!(effects.len(), 3);
        assert_eq!(effects[0]["matched_cursor"], 1);
        assert_eq!(effects[1]["matched_cursor"], 2);
        assert_eq!(effects[2]["matched_cursor"], 2);
    }
    drop(store);
    drop(capture_scope);
    drop(cancel_scope);
    let (requests, probes) = server.join().unwrap();
    assert_eq!(tcp.finish(), count);
    let inputs: Vec<_> = requests
        .iter()
        .filter(|request| request["execute"] == "input-send-event")
        .collect();
    if parent_changed {
        assert_eq!(probes, 0);
        assert!(inputs.is_empty());
        assert_eq!(requests.len(), 1);
    } else {
        assert_eq!(probes, 1);
        assert_eq!(inputs.len(), 7);
        let to = PixelPoint::new(1301, 203);
        movement(inputs[0], from);
        button(inputs[1], true);
        movement(inputs[2], PixelPoint::new(from.x, from.y - 45));
        movement(
            inputs[3],
            PixelPoint::new((from.x + to.x) / 2, (from.y + to.y) / 2),
        );
        movement(inputs[4], PixelPoint::new(to.x, to.y - 45));
        movement(inputs[5], to);
        button(inputs[6], false);
        assert_eq!(
            inputs
                .iter()
                .filter(|request| request["arguments"]["events"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|event| event["type"] == "btn" && event["data"]["down"] == true))
                .count(),
            1,
            "AUTO 2C must not cause another explicit drag or click"
        );
    }
    assert!(matches!(
        command_rx.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    drop(handle);
}


/// AUTO-marked AC is explicitly dragged once; two combined proofs credit AUTO 2C and stop.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_319_native_single_step_auto_record_and_followup_stop_at_cursor_two() {
    recorded_followup_boundary(false);
}


/// A late recorded AUTO invalidates the fresh parent and cannot trigger stale input or replay.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_319_native_late_auto_before_transfer_refuses_stale_parent_without_input() {
    recorded_followup_boundary(true);
}


/// Post-ACK failure keeps immutable input, refuses a different full board, and rebinds a restored one.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_3110_native_failed_route_retry_rebinds_all52_and_preserves_failure_packet() {
    let fixture = club_followup_fixture();
    let mut different = fixture.initial.clone();
    fixture_blit(
        &mut different,
        &fixture.initial,
        PixelRect::new(218, 331, 137, 502),
        (410, 331),
    );
    fixture_blit(
        &mut different,
        &fixture.initial,
        PixelRect::new(410, 331, 137, 502),
        (218, 331),
    );
    let different_deal =
        crate::freecell_recon::recognise(&different, fixture.report.deal.metadata().clone())
            .unwrap();
    assert!(!route::same_physical(
        &Position::initial(&different_deal),
        &Position::initial(&fixture.report.deal)
    ));
    let failed_frames = (0..10)
        .map(|_| V3Reply::Frame(fixture.initial.clone()))
        .collect();
    let restored_frames = (0..4)
        .map(|_| V3Reply::Frame(fixture.initial.clone()))
        .chain((0..2).map(|_| V3Reply::Frame(fixture.combined.clone())))
        .collect();
    let (address, tcp) = v3_server_with_read_timeout(
        vec![
            failed_frames,
            vec![V3Reply::Frame(different)],
            restored_frames,
        ],
        Duration::from_secs(60),
    );
    let _backend = TestCaptureBackend::tcp_mode(address, CaptureMode::PersistentBgrV3);
    let root = Socket::new();
    let mut id = plan(3110);
    id.context = ControllerContext::new(
        root.path(),
        GameMode::FreeCell,
        SolvingStrategy::ShortestPath,
        3110,
    );
    let latest = LatestFrameSlot::default();
    let (events, receiver) = mpsc::channel();
    let sink = WorkerEventSink {
        events,
        latest_frame: latest.clone(),
        capture_context: Mutex::new(None),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let _wire_scope = crate::worker::qmp_wire::WorkerScope::install(&sink);
    let _recording = RecordingRestore(WORKER_IMAGE_DIAGNOSTICS.with(|slot| slot.replace(None)));
    sink.set_capture_context(Some(id.context.clone()));
    latest
        .freecell_route
        .lock()
        .unwrap()
        .prepare(id.clone(), Arc::clone(&fixture.report))
        .unwrap();
    let (command_tx, command_rx) = mpsc::channel();
    let (_, event_rx) = mpsc::channel();
    let handle = WorkerHandle {
        command_tx,
        event_rx,
        latest_frame: latest.clone(),
        join: None,
        cancel_requested: Arc::clone(&cancel),
        shutdown_requested: Arc::new(AtomicBool::new(false)),
    };
    let invoke = |single_step, mode| {
        let cancel_scope = tcp_capture::CancellationScope::install(Arc::clone(&cancel));
        let capture_scope = tcp_capture::WorkerCaptureScope::install(&sink);
        let run_id = handle
            .execute_freecell_route(
                id.clone(),
                single_step,
                mode,
                route::DragMode::Absolute,
                Duration::ZERO,
            )
            .unwrap();
        {
            let store = latest.freecell_route.lock().unwrap();
            let state = store.live.as_ref().unwrap().state;
            assert_eq!(state.cursor, 0);
            assert!(!state.bound);
            assert!(Arc::ptr_eq(
                &store.cached.as_ref().unwrap().report,
                &fixture.report
            ));
        }
        let command = command_rx.try_recv().unwrap();
        let _command_scope = crate::worker::qmp_wire::CommandScope::install(&command);
        let WorkerCommand::ExecuteFreeCellRoute {
            plan,
            run_id: queued_run,
            single_step,
            drag_mode,
            drag_timing,
            settle,
        } = command
        else {
            panic!("expected the explicit cached recovery request");
        };
        assert_eq!(queued_run, run_id);
        let server = qmp_server(
            &root.path(),
            Case::Both,
            latest.clone(),
            id.context.clone(),
            Arc::clone(&cancel),
            fixture.initial.clone(),
        );
        run(
            plan,
            queued_run,
            single_step,
            (drag_mode, drag_timing),
            settle,
            &sink,
            &cancel,
        );
        let (result, termination) = receiver
            .try_iter()
            .find_map(|event| match event.event {
                WorkerEvent::FreeCellRouteFinished {
                    plan,
                    run_id: finished,
                    result,
                    termination,
                } => {
                    assert_eq!(plan, id);
                    assert_eq!(finished, run_id);
                    Some((result, termination))
                }
                _ => None,
            })
            .unwrap();
        drop(capture_scope);
        drop(cancel_scope);
        let (requests, probes) = server.join().unwrap();
        std::fs::remove_file(root.path()).unwrap();
        (run_id, result, termination, requests, probes)
    };
    let (failed_run, failed, termination, requests, probes) =
        invoke(false, route::AutoMode::ExecuteRecorded);
    let failed = failed.unwrap_err();
    assert_eq!(
        termination,
        crate::pyramid_workflow_model::RunTermination::Failed
    );
    assert_eq!(failed.details["attempted"], 1);
    assert_eq!(failed.details["acknowledged"], 1);
    assert_eq!(failed.details["capture_serial"], 10);
    assert_eq!(probes, 1);
    assert!(
        requests
            .iter()
            .any(|request| request["execute"] == "input-send-event")
    );
    let packet = failed.directory.as_ref().unwrap().join("error-state.json");
    let original_packet = std::fs::read(&packet).unwrap();
    let original_details = failed.details.clone();
    {
        let store = latest.freecell_route.lock().unwrap();
        assert!(store.available(&id) && store.live.is_none());
        assert!(Arc::ptr_eq(
            &store.cached.as_ref().unwrap().report,
            &fixture.report
        ));
    }
    let (mismatch_run, mismatch, termination, requests, probes) =
        invoke(true, route::AutoMode::ObserveGame);
    let mismatch = mismatch.unwrap_err();
    assert!(mismatch.error.contains("differs from frozen calculation"));
    assert_eq!(mismatch.details["attempted"], 0);
    assert_eq!(mismatch.details["acknowledged"], 0);
    assert_eq!(mismatch.details["capture_serial"], 1);
    assert_eq!(
        termination,
        crate::pyramid_workflow_model::RunTermination::Failed
    );
    assert_eq!(probes, 0);
    assert!(
        !requests
            .iter()
            .any(|request| request["execute"] == "input-send-event")
    );
    assert!(handle.freecell_route_available(&id));
    let (restored_run, restored, termination, _, probes) =
        invoke(true, route::AutoMode::ObserveGame);
    let restored =
        restored.unwrap_or_else(|failure| panic!("{}: {}", failure.error, failure.details));
    assert!(restored_run > mismatch_run && mismatch_run > failed_run);
    assert_eq!(
        termination,
        crate::pyramid_workflow_model::RunTermination::Completed
    );
    assert_eq!(restored.completion, Some(route::Completion::StepVerified));
    assert_eq!((restored.cursor, restored.total), (2, 3));
    assert_eq!((restored.attempted, restored.acknowledged), (1, 1));
    assert_eq!(probes, 1);
    assert_eq!(std::fs::read(packet).unwrap(), original_packet);
    assert_eq!(failed.details, original_details);
    assert_eq!(tcp.finish(), 17);
    assert!(handle.freecell_route_available(&id));
    assert!(Arc::ptr_eq(
        &latest
            .freecell_route
            .lock()
            .unwrap()
            .cached
            .as_ref()
            .unwrap()
            .report,
        &fixture.report
    ));
}
