//! Real loopback TCPv3 and local QMP bridge evidence; no guest connection.
use super::*;
use crate::worker::tcp_capture_tests::{V3Reply, v3_server};
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
    Stop,
    Revoke,
    DownError,
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
                    json!([{"name":"QEMU HID Tablet","index":0,"current":true,"absolute":true}])
                }
                "input-send-event" => {
                    let events = request["arguments"]["events"].as_array().unwrap();
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
            let delivered = packet(&mut stream, response);
            if case == Case::DownError && rejected && requests.len() > 5 {
                let _ = delivered;
            } else {
                delivered.unwrap();
            }
        }
        if rejected {
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
            for expected_command in ["qmp_capabilities", "input-send-event"] {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let request: Value = serde_json::from_str(&line).unwrap();
                assert_eq!(request["execute"], expected_command);
                if expected_command == "input-send-event" {
                    button(&request, false);
                }
                packet(&mut fresh, json!({"return":{},"id":request["id"]})).unwrap();
                requests.push(request);
            }
        }
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
fn exercise(case: Case) {
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
    let action = RecordedMove {
        card_id: *parent.column(0).unwrap().last().unwrap(),
        from: Pile::Cascade(0),
        to: Pile::Cell(0),
        is_auto: false,
    };
    parent.apply(action).unwrap();
    let geometry = position::verify(&original, &parent, &mut || Ok(())).unwrap();
    let from = geometry.point(action.from).unwrap();
    let to = geometry.point(action.to).unwrap();
    let count = 1;
    let (address, tcp) = v3_server(vec![
        (0..count)
            .map(|_| V3Reply::Frame(original.clone()))
            .collect(),
    ]);
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
        store.prepare(id.clone(), report()).unwrap();
        store.reserve(&id).unwrap()
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
    };
    let result = native.transfer(&parent, action);
    match case {
        Case::Both => {
            result.unwrap();
            assert_eq!(native.frozen, Some((action, from, to)));
            assert_eq!(native.counts, (1, 1));
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
    }
    if case != Case::Both {
        let mut store = latest.freecell_route.lock().unwrap();
        store.reject(&id, run_id).unwrap();
        assert!(store.running.is_none() && store.live.is_none());
        assert!(store.reserve(&id).is_err());
    }
    drop(native);
    drop(capture_scope);
    drop(cancel_scope);
    let (requests, captures) = server.join().unwrap();
    assert_eq!(captures, count);
    assert_eq!(tcp.finish(), count);
    let commands: Vec<_> = requests
        .iter()
        .map(|r| r["execute"].as_str().unwrap())
        .collect();
    let mut expected = vec!["qmp_capabilities", "query-status", "query-mice"];
    if matches!(case, Case::Both | Case::DownError) {
        expected.extend(["input-send-event"; 4]);
    }
    if case == Case::DownError {
        expected.extend(["input-send-event", "qmp_capabilities", "input-send-event"]);
    }
    assert_eq!(commands, expected);
    let inputs: Vec<_> = requests
        .iter()
        .filter(|r| r["execute"] == "input-send-event")
        .collect();
    if case == Case::Both {
        assert_eq!(inputs.len(), 4);
        movement(inputs[0], from);
        button(inputs[1], true);
        movement(inputs[2], to);
        button(inputs[3], false);
    } else if case == Case::DownError {
        assert_eq!(inputs.len(), 6);
        movement(inputs[0], from);
        button(inputs[1], true);
        movement(inputs[2], to);
        for request in inputs.iter().skip(3) {
            button(request, false);
        }
    } else {
        assert!(inputs.is_empty());
    }
}


/// Real TCP frame and fresh VM/pointer probe precede one explicit-target transfer.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_312_native_compound_capture_probe_input_order() {
    exercise(Case::Both);
}


/// STOP after probing sends no input packet.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_312_native_stop_after_probe() {
    exercise(Case::Stop);
}


/// Revocation fences a cloned adapter before its next delivery.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_312_native_revoke_after_probe() {
    exercise(Case::Revoke);
}


/// Failed down is never replayed; only existing release-UP recovery remains.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_312_native_uncertain_down_release_only() {
    exercise(Case::DownError);
}
