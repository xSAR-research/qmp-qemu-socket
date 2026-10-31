//! Concrete computed-route delivery through an isolated local QMP socket.
//!
//! Mock QEMU returns the unchanged native-v7 initial frame. These tests exercise
//! the production capture, scene validation, capability probe and input adapter;
//! they do not claim guest effects or live Windows focus verification.

use super::*;


mod redeal_tests;
use crate::geometry::{PixelPoint, pixel_point_to_qmp};
use crate::worker::{LatestFrameSlot, TestCaptureBackend};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread,
};


/// Each test owns a separate Unix socket and private capture-file directory.
static NEXT_SOCKET: AtomicU64 = AtomicU64::new(0);


/// Original native frame, without recolouring, crop substitution or rescaling.
const INITIAL_PNG: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-recon/native-v7/baseline.png");


/// Transport fault injected at one deterministic request boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Scenario {
    /// Acknowledge every request from one typed gesture.
    Acknowledged,
    /// Omit the first DOWN acknowledgement, while acknowledging its matching UP.
    MissingDownAck,
    /// Revoke the actual application authority before returning the fresh probe.
    StopAfterProbe,
}


/// Actual request trace from the QMP server, including recovery connections.
#[derive(Debug, Default)]
struct WireReport {
    /// Requested command names, in arrival order across both connections.
    commands: Vec<String>,
    /// Input event arrays paired with their one-based connection number.
    inputs: Vec<(usize, Vec<Value>)>,
    /// Handshakes include any release-only recovery connection.
    connections: usize,
    /// Native adapter wait telemetry retained by tests that consume its event channel.
    logs: Vec<String>,
}


/// Remove only the fixture-owned directory after its peer has terminated.
struct SocketDirectory(
    /// Unique socket/capture directory created by this fixture invocation.
    PathBuf,
);


impl Drop for SocketDirectory {
    /// Runtime cleanup is scoped to the unique directory created by this test.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}


/// Write one complete newline-delimited QMP response; errors fail the fixture.
fn packet(stream: &mut UnixStream, value: Value) {
    serde_json::to_writer(&mut *stream, &value).expect("mock QMP response serialises");
    stream.write_all(b"\n").expect("mock QMP response newline");
    stream.flush().expect("mock QMP response flush");
}


/// Accept within a fixed deadline so a failed recovery cannot leave a hung test.
fn accept(listener: &UnixListener) -> UnixStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match listener.accept() {
            Ok((stream, _)) => return stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(
                    Instant::now() < deadline,
                    "computed-route mock QMP accept timed out"
                );
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("computed-route mock QMP accept: {error}"),
        }
    }
}


/// Serve native pixels and real protocol ACKs; recovery may send releases only.
fn serve(
    listener: UnixListener,
    directory: PathBuf,
    scenario: Scenario,
    control: super::super::pyramid_reconnaissance::ReconControl,
    context: ControllerContext,
    cancel: Arc<AtomicBool>,
    frame_bytes: &[u8],
) -> WireReport {
    let mut report = WireReport::default();
    let connection_limit = if scenario == Scenario::MissingDownAck {
        2
    } else {
        1
    };
    for connection in 1..=connection_limit {
        let mut stream = accept(&listener);
        stream
            .set_read_timeout(Some(Duration::from_secs(6)))
            .expect("bounded mock read");
        let mut reader = BufReader::new(stream.try_clone().expect("mock QMP reader clone"));
        report.connections += 1;
        packet(
            &mut stream,
            json!({"QMP": {"version": {"qemu": {"major": 11, "minor": 0, "micro": 0}}, "capabilities": []}}),
        );
        let mut withheld_down = false;
        let mut paired_up_seen = false;
        loop {
            let mut line = String::new();
            if reader
                .read_line(&mut line)
                .expect("read computed-route QMP request")
                == 0
            {
                break;
            }
            let request: Value = serde_json::from_str(&line).expect("computed-route QMP JSON");
            let id = request["id"].as_u64().expect("numeric QMP request id");
            let command = request["execute"].as_str().expect("QMP execute name");
            report.commands.push(command.to_owned());
            match command {
                "qmp_capabilities" => packet(&mut stream, json!({"return": {}, "id": id})),
                "screendump" => {
                    assert_eq!(connection, 1, "recovery cannot capture another board");
                    assert_eq!(request["arguments"]["format"], "png");
                    let filename = Path::new(
                        request["arguments"]["filename"]
                            .as_str()
                            .expect("capture filename"),
                    );
                    assert_eq!(
                        filename.parent(),
                        Some(directory.as_path()),
                        "only this fixture's reserved file is written"
                    );
                    fs::write(filename, frame_bytes)
                        .expect("mock QEMU writes unchanged native-v7 bytes");
                    packet(&mut stream, json!({"return": {}, "id": id}));
                }
                "query-status" => {
                    assert_eq!(
                        connection, 1,
                        "release recovery must not acquire new input authority"
                    );
                    packet(
                        &mut stream,
                        json!({"return": {"running": true, "status": "running"}, "id": id}),
                    );
                }
                "query-mice" => {
                    assert_eq!(
                        connection, 1,
                        "release recovery must not probe for another press"
                    );
                    if scenario == Scenario::StopAfterProbe {
                        control.stop(Some(&context), &cancel);
                    }
                    packet(
                        &mut stream,
                        json!({"return": [{"name": "Fixture absolute tablet", "index": 0, "current": true, "absolute": true}], "id": id}),
                    );
                }
                "input-send-event" => {
                    let events = request["arguments"]["events"]
                        .as_array()
                        .expect("typed input events")
                        .clone();
                    let down = events.iter().any(|event| event["data"]["down"] == true);
                    let up = events.iter().any(|event| event["data"]["down"] == false);
                    if connection > 1 {
                        assert!(
                            up && !down && events.len() == 1,
                            "fresh recovery connection may only release"
                        );
                    }
                    report.inputs.push((connection, events));
                    if scenario == Scenario::MissingDownAck && connection == 1 && down {
                        assert!(!withheld_down, "uncertain DOWN was replayed");
                        withheld_down = true;
                    } else if withheld_down && connection == 1 && up && paired_up_seen {
                        // xSAR releases once on the old stream before closing it.
                        // Do not answer after that shutdown has begun.
                    } else {
                        if withheld_down && connection == 1 && up {
                            paired_up_seen = true;
                        }
                        packet(&mut stream, json!({"return": {}, "id": id}));
                    }
                }
                other => panic!("unexpected computed-route QMP command: {other}"),
            }
        }
        if scenario == Scenario::MissingDownAck && connection == 1 {
            assert!(
                withheld_down && paired_up_seen,
                "matching UP must precede waiting for a missing DOWN ACK"
            );
        }
    }
    report
}


/// Retain concrete application counters separately from server-observed input.
struct Evidence {
    /// NativeIo's returned delivery status.
    result: Result<(), String>,
    /// Actual adapter attempt and complete-ACK accounting.
    counts: Counts,
    /// Every delivery guard acquires exactly one fresh native frame.
    captures: usize,
    /// All QMP commands, including any release-only recovery.
    wire: WireReport,
}


/// Deliver once with native recognition and current application authority.
///
/// A test-only private plan supplies the retained complete deal. This fixture
/// deliberately tests delivery, rather than fabricating acquisition acceptance
/// Establish one complete source and deliver one whole test action without recapturing.
pub(super) fn checked_delivery(
    io: &mut NativeIo<'_>,
    state: SearchState,
    operations: &[InputOperation],
) -> Result<(), String> {
    match io.observe_source(state)? {
        EffectObservation::Position(true) => {}
        _ => return Err("Action source or terminal does not authorise input".to_owned()),
    }
    io.begin_action(state, operations)?;
    for &operation in operations {
        io.deliver(operation, state)?;
    }
    Ok(())
}


/// or claiming that transport acknowledgement establishes a game effect.
fn deliver_once(operation: InputOperation, scenario: Scenario) -> Evidence {
    let _capture_backend = TestCaptureBackend::legacy_qmp();
    let directory = SocketDirectory(std::env::temp_dir().join(format!(
        "computed-route-wire-{}-{}",
        std::process::id(),
        NEXT_SOCKET.fetch_add(1, Ordering::Relaxed),
    )));
    fs::create_dir(&directory.0).expect("unique computed-route QMP fixture directory");
    let socket = directory.0.join("qmp.sock");
    let listener = UnixListener::bind(&socket).expect("fixture-local QMP Unix socket");
    listener
        .set_nonblocking(true)
        .expect("bounded fixture accept");
    let context =
        ControllerContext::new(socket, GameMode::Pyramid, SolvingStrategy::ShortestPath, 3);
    let cancel = Arc::new(AtomicBool::new(false));
    let (event_tx, _event_rx) = mpsc::channel();
    let events = WorkerEventSink {
        events: event_tx,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(context.clone())),
    };
    let control = &events.latest_frame.reconnaissance;
    control
        .select(&context, &cancel)
        .expect("current fixture context selected");
    control
        .queue_reconnaissance(&context, &cancel, || Ok(()))
        .expect("fixture owns STOP during delivery");
    let bytes = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../xsar/tests/fixtures/pyramid-planning/pyramid-recon-20261019-v7-01.json"),
    )
    .expect("accepted complete native-v7 deal fixture");
    let deal =
        xsar::pyramid_json::read_from(bytes.as_slice()).expect("complete fixture deal validates");
    let vision = ReconVision::load(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon/native-v7/profile.json"),
    )
    .expect("unchanged native-v7 calibration");
    let plan = Prepared {
        id: 17,
        objective: RouteObjective::Clear,
        acquisition: Acquisition {
            challenge_progress: None,
            default_method: None,
            observed_game: crate::pyramid_workflow_model::ObservedGame::Unknown,
            context: context.clone(),
            epoch: control.epoch().expect("current authority epoch"),
            deal,
            bytes,
            vision,
        },
        trace: Vec::new().into_boxed_slice(),
    };
    let native = crate::capture::decode_png(INITIAL_PNG).expect("unchanged native PNG decodes");
    let (tableau, left, right) = expected(&plan.acquisition.deal, SearchState::initial())
        .expect("initial expected position");
    plan.acquisition
        .vision
        .verify_position(&native, &tableau, left, right)
        .expect("original native frame matches retained deal");
    let peer_context = context.clone();
    let peer_cancel = cancel.clone();
    let peer_control = control.clone();
    let peer_directory = directory.0.clone();
    let server = thread::spawn(move || {
        serve(
            listener,
            peer_directory,
            scenario,
            peer_control,
            peer_context,
            peer_cancel,
            INITIAL_PNG,
        )
    });
    let qmp = QmpClient::connect(&context.socket_path)
        .expect("real production QMP handshake with local fixture");
    let mut io = NativeIo {
            execution_vision: plan.acquisition.vision.for_temporal_execution(),
            source_transition: None,
            ready_transition: None,
            failure_progress: crate::pyramid_diagnostics::progress::FailureProgress::default(),
            removals: Default::default(),
            stock_effect: None,
            effect_settings: crate::parameters::PyramidEffectSettings::default(),
        policy: ExecutionPolicy::Standard,
        stock_continuation: false,
        executed_recycles: 0,
        qmp,
        plan: &plan,
        context: &context,
        events: &events,
        cancel: &cancel,
        started: Instant::now(),
        paused: Duration::ZERO,
        captures: 0,
        capture_limit: capture_allowance(1, 1).expect("finite one-gesture capture allowance"),
        counts: Counts::default(),
        first_mismatch: None,
        latest_frame: None,
        latest_observation: None,
        verified_actions: 0,
        final_confirmation: FinalConfirmation::default(),
        challenge: None,
        challenge_initial: None,
        challenge_confirmation: None,
        position_matches: false,
        source_proof: None,
        action_permit: None,
        redeal_before: None,
        frame_guard: None,
    };
    let result = checked_delivery(&mut io, SearchState::initial(), &[operation]);
    let counts = io.counts;
    let captures = io.captures;
    drop(io); // Close the last client before joining its bounded local peer.
    let wire = server.join().expect("computed-route QMP fixture completed");
    Evidence {
        result,
        counts,
        captures,
        wire,
    }
}


/// Return actual key/button transitions in wire order, excluding pointer motion.
fn transitions(wire: &WireReport) -> Vec<(usize, bool)> {
    wire.inputs
        .iter()
        .flat_map(|(connection, events)| {
            events.iter().filter_map(move |event| {
                event["data"]["down"]
                    .as_bool()
                    .map(|down| (*connection, down))
            })
        })
        .collect()
}


/// Both accepted gesture kinds require fresh native evidence and fresh VM probes.
#[test]
fn native_delivery_probes_after_fresh_capture_before_one_click_or_draw_key() {
    let point = PixelPoint::new(828, 771);
    for operation in [InputOperation::Click(point), InputOperation::PressDrawKey] {
        let evidence = deliver_once(operation, Scenario::Acknowledged);
        assert!(evidence.result.is_ok(), "{:?}", evidence.result);
        assert_eq!(
            (
                evidence.counts.attempted,
                evidence.counts.acknowledged,
                evidence.captures
            ),
            (1, 1, 1)
        );
        assert_eq!(evidence.wire.connections, 1);
        assert_eq!(transitions(&evidence.wire), [(1, true), (1, false)]);
        let prefix = [
            "qmp_capabilities",
            "screendump",
            "query-status",
            "query-mice",
        ];
        assert_eq!(
            evidence
                .wire
                .commands
                .iter()
                .take(4)
                .map(String::as_str)
                .collect::<Vec<_>>(),
            prefix
        );
        assert!(
            evidence
                .wire
                .commands
                .iter()
                .skip(4)
                .all(|command| command == "input-send-event")
        );
        match operation {
            InputOperation::Click(_) => {
                let mapped = pixel_point_to_qmp(point, 1920, 1080).expect("native checked mapping");
                let first = &evidence.wire.inputs.first().expect("pointer movement").1;
                assert_eq!(
                    first,
                    &vec![
                        json!({"type": "abs", "data": {"axis": "x", "value": mapped.x}}),
                        json!({"type": "abs", "data": {"axis": "y", "value": mapped.y}}),
                    ]
                );
                assert_eq!(evidence.wire.inputs.len(), 3, "movement, DOWN, UP only");
                for (_, events) in evidence.wire.inputs.iter().skip(1) {
                    assert_eq!(events[0]["type"], "btn");
                    assert_eq!(events[0]["data"]["button"], "left");
                }
            }
            InputOperation::PressDrawKey => {
                assert_eq!(evidence.wire.inputs.len(), 2, "D DOWN and UP only");
                for (_, events) in &evidence.wire.inputs {
                    assert_eq!(events[0]["type"], "key");
                    assert_eq!(
                        events[0]["data"]["key"],
                        json!({"type": "qcode", "data": "d"})
                    );
                }
            }
        }
    }
}


/// Missing ACK consumes the attempt and permits release recovery without a press replay.
#[test]
fn native_delivery_missing_down_ack_has_only_one_press_and_release_only_recovery() {
    let evidence = deliver_once(InputOperation::PressDrawKey, Scenario::MissingDownAck);
    assert!(
        evidence
            .result
            .as_ref()
            .is_err_and(|error| error.contains("Uncertain route delivery")),
        "{:?}",
        evidence.result
    );
    assert_eq!(
        (
            evidence.counts.attempted,
            evidence.counts.acknowledged,
            evidence.captures
        ),
        (1, 0, 1)
    );
    assert_eq!(evidence.wire.connections, 2);
    assert_eq!(
        transitions(&evidence.wire),
        [(1, true), (1, false), (1, false), (2, false)]
    );
    assert_eq!(
        evidence
            .wire
            .commands
            .iter()
            .filter(|name| name.as_str() == "screendump")
            .count(),
        1
    );
    assert_eq!(
        evidence
            .wire
            .commands
            .iter()
            .filter(|name| name.as_str() == "query-mice")
            .count(),
        1
    );
}


/// A fresh successful probe cannot restore authority revoked while it was in flight.
#[test]
fn native_delivery_stop_before_probe_ack_sends_no_pointer_key_or_button_input() {
    let evidence = deliver_once(
        InputOperation::Click(PixelPoint::new(828, 771)),
        Scenario::StopAfterProbe,
    );
    assert!(
        evidence.result.is_err(),
        "STOP must refuse the guarded delivery"
    );
    assert_eq!(
        (
            evidence.counts.attempted,
            evidence.counts.acknowledged,
            evidence.captures
        ),
        (0, 0, 1)
    );
    assert!(
        evidence.wire.inputs.is_empty(),
        "STOP permits no new input, including pointer motion"
    );
    assert_eq!(
        evidence.wire.commands,
        [
            "qmp_capabilities",
            "screendump",
            "query-status",
            "query-mice"
        ]
    );
}


#[cfg(test)]
mod stock_empty_tests;
