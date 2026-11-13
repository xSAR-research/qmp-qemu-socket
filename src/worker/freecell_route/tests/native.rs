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
    /// Reproduce the retained JC pickup and empty CELL4 target after the actual prefix.
    ReturnedJackToCellFour,
    Stop,
    Revoke,
    DownError,
    /// The next ordinary effect observation reports completion after release.
    CompleteAfterDrag,
    /// Ordinary effect pixels do not prove any permitted route endpoint.
    UnsupportedAfterDrag,
    /// The paced ABS experiment observes STOP after DOWN ACK, before destination motion.
    AbsoluteStopDuringHeldWait,
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
    qmp_server_for_parents(path, case, latest, context, cancel, vec![expected])
}


/// Confirm each fresh transfer probe against its own synthetic physical parent.
fn qmp_server_for_parents(
    path: &std::path::Path,
    case: Case,
    latest: LatestFrameSlot,
    context: ControllerContext,
    cancel: Arc<AtomicBool>,
    expected: Vec<CapturedFrame>,
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
                    let expected_parent = expected
                        .get(captures)
                        .or_else(|| expected.last())
                        .expect("each local probe has an expected physical parent");
                    assert_eq!(frame.frame.pixels, expected_parent.pixels);
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
                "human-monitor-command" => {
                    let command = request["arguments"]["command-line"].as_str().unwrap();
                    match command {
                        "mouse_set 0" => {
                            relative_current = false;
                        }
                        other => panic!("unexpected HMP command: {other}"),
                    }
                    mouse_selections.push(command.to_owned());
                    json!("")
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
            let stop_after_down = case == Case::AbsoluteStopDuringHeldWait
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
            }
            let delivered = packet(&mut stream, response);
            if (case == Case::DownError && rejected && requests.len() > 5)
                || (case == Case::AbsoluteStopDuringHeldWait
                    && stopped_during_hold
                    && !stop_after_down)
            {
                // Recovery can close this stream after consuming the earlier DOWN ACK.
                // Exact old-UP and fresh capabilities/UP traffic is asserted below.
                let _ = delivered;
            } else {
                delivered.unwrap();
            }
            if stop_after_down && case == Case::AbsoluteStopDuringHeldWait {
                // DOWN is acknowledged before STOP arrives inside the fixed 100ms held wait.
                thread::sleep(Duration::from_millis(20));
                latest.reconnaissance.operator_stop(&context, &cancel);
            }
        }
        if stopped_during_hold {
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
                        json!([{"name":"QEMU HID Tablet","index":0,"current":true,"absolute":true}])
                    }
                    "human-monitor-command" => {
                        assert_eq!(request["arguments"]["command-line"], "mouse_set 0");
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


/// Consume the exact acquired advisory pixels without asking either peer for another frame.
fn assert_native_preview(
    latest: &LatestFrameSlot,
    expected: &CapturedFrame,
    context: &ControllerContext,
) {
    let preview = latest
        .take()
        .expect("every native acquisition publishes a preview");
    assert_eq!(preview.context.as_ref(), Some(context));
    assert!(preview.diagnostic);
    assert!(preview.prediction.is_none() && !preview.log_prediction);
    assert_eq!(
        (
            preview.frame.width,
            preview.frame.height,
            preview.frame.stride
        ),
        (expected.width, expected.height, expected.stride)
    );
    assert_eq!(preview.frame.pixels, expected.pixels);
    assert!(
        latest.take().is_none(),
        "only the latest preview is retained"
    );
}


/// Replay the incident's 29 deliberate and ten AUTO edges from its authentic initial deal.
///
/// The unchanged returned-card frame supplies only the requested physical pile proof;
/// the complete hidden inventory comes from this checked 39-edge history.
fn returned_jack_fixture() -> (
    CapturedFrame,
    Arc<freecell_calculation::Report>,
    Position,
    RecordedMove,
) {
    let initial = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/freecell-sixes-decorative-20261111.png"
    ))
    .unwrap();
    let deal = crate::freecell_recon::recognise(
        &initial,
        Metadata::new("native-returned-JC", "1920x1080", "unit").unwrap(),
    )
    .unwrap();
    use Pile::{Cascade as P, Cell as C, Foundation as F};
    let prefix = [
        (36, P(5), C(0), false),
        (20, P(5), C(1), false),
        (27, P(5), C(2), false),
        (26, P(5), F(2), true),
        (27, C(2), F(2), true),
        (15, P(5), P(1), false),
        (40, P(6), P(1), false),
        (11, P(6), C(2), false),
        (44, P(6), C(3), false),
        (29, P(0), P(3), false),
        (37, P(2), P(0), false),
        (10, P(5), P(0), false),
        (24, P(2), P(5), false),
        (49, P(2), P(5), false),
        (13, P(2), F(1), true),
        (14, P(6), F(1), true),
        (45, P(6), P(2), false),
        (39, P(6), F(3), true),
        (40, P(1), F(3), true),
        (15, P(1), F(1), false),
        (16, P(7), F(1), false),
        (38, P(7), P(6), false),
        (11, C(2), P(6), false),
        (29, P(3), C(2), false),
        (36, C(0), P(6), false),
        (43, P(3), C(0), false),
        (42, P(3), P(7), false),
        (41, P(3), F(3), true),
        (42, P(7), F(3), false),
        (43, C(0), F(3), false),
        (44, C(3), F(3), false),
        (17, P(7), F(1), false),
        (0, P(7), F(0), true),
        (1, P(7), F(0), true),
        (18, P(3), F(1), false),
        (32, P(3), C(0), false),
        (51, P(3), C(3), false),
        (28, P(3), F(2), true),
        (51, C(3), P(3), false),
    ]
    .map(|(card_id, from, to, is_auto)| RecordedMove {
        card_id,
        from,
        to,
        is_auto,
    });
    let replay = xsar::games::freecell::replay::verify_prefix_with_control(
        &deal,
        &prefix,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!((replay.deliberate, replay.automatic), (29, 10));
    let mut parent = Position::initial(&deal);
    for action in prefix {
        if action.is_auto {
            assert_eq!(parent.automatic(), Some(action));
        }
        parent = parent.apply(action).unwrap();
    }
    assert_eq!(parent.column(0).unwrap(), &[46, 2, 22, 19, 6, 12, 37, 10]);
    assert_eq!(parent.cells(), [Some(32), Some(20), Some(29), None]);
    assert_eq!(parent.foundations(), &[2, 6, 3, 6]);
    let original = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/freecell-jc-cell4-returned-20261111.png"
    ))
    .unwrap();
    let action = RecordedMove {
        card_id: 10,
        from: P(0),
        to: C(3),
        is_auto: false,
    };
    let child = parent.apply(action).unwrap();
    let rejection =
        position::verify_piles(&original, &child, &[action.from, action.to], &mut || Ok(()))
            .unwrap_err();
    assert!(rejection.contains("PLAY 1: Exposed identity differs from physical position"));
    assert!(rejection.contains("observed 10") && rejection.contains("expected 37"));
    let mut moves = prefix.to_vec();
    moves.push(action);
    let closed = xsar::games::freecell::replay::verify_prefix_with_control(
        &deal,
        &moves,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!((closed.deliberate, closed.automatic), (30, 10));
    assert!(child.automatic().is_none());
    for (column, stack) in closed.endpoint.cascades().iter().enumerate() {
        assert_eq!(
            stack
                .iter()
                .map(xsar::games::freecell::rules::card_id)
                .collect::<Vec<_>>(),
            child.column(column).unwrap()
        );
    }
    assert_eq!(
        closed
            .endpoint
            .cells()
            .map(|card| card.map(xsar::games::freecell::rules::card_id)),
        child.cells()
    );
    assert_eq!(closed.endpoint.foundations(), child.foundations());
    let report = Arc::new(freecell_calculation::Report {
        path: PathBuf::from("native-returned-JC-closed-prefix.json"),
        deal,
        bytes: Vec::new(),
        paused: Duration::ZERO,
        outcome: Outcome::Partial {
            moves: moves.into_boxed_slice(),
            stats: Default::default(),
            deliberate_moves: closed.deliberate,
            automatic_moves: closed.automatic,
            foundation_cards: child.foundations().iter().copied().map(u32::from).sum(),
        },
    });
    (original, report, parent, action)
}


/// All empty CELL and canonical C,D,H,S targets use the occupied-card pickup inset.
#[test]
fn freecell_3119_native_empty_upper_slots_preserve_pickup_offset() {
    let original = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/freecell-FC13.png"
    ))
    .unwrap();
    let deal = crate::freecell_recon::recognise(
        &original,
        Metadata::new("native-empty-upper-anchors", "FC13", "unit").unwrap(),
    )
    .unwrap();
    let parent = Position::initial(&deal);
    let expected = [
        (Pile::Cell(0), PixelPoint::new(181, 132)),
        (Pile::Cell(1), PixelPoint::new(373, 132)),
        (Pile::Cell(2), PixelPoint::new(566, 132)),
        (Pile::Cell(3), PixelPoint::new(758, 132)),
        (Pile::Foundation(0), PixelPoint::new(1248, 132)),
        (Pile::Foundation(1), PixelPoint::new(1440, 132)),
        (Pile::Foundation(2), PixelPoint::new(1056, 132)),
        (Pile::Foundation(3), PixelPoint::new(1633, 132)),
    ];
    let piles = expected.map(|(pile, _)| pile);
    let geometry = position::verify_piles(&original, &parent, &piles, &mut || Ok(())).unwrap();
    for (pile, anchor) in expected {
        assert_eq!(geometry.point(pile).unwrap(), anchor);
        assert_eq!(
            diagnostic_abs_path(PixelPoint::new(233, 722), anchor, pile, (1920, 1080)).unwrap()[2],
            anchor
        );
    }
}


/// Positively outlined synthetic empty PLAY targets retain the same native pickup inset.
///
/// Only the requested empty piles are proved; a legally solved complete deal supplies
/// their empty state. Composed pixels establish host geometry, never guest effects.
#[test]
fn freecell_3119_native_empty_play_targets_preserve_pickup_offset() {
    let original = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/freecell-FC13.png"
    ))
    .unwrap();
    let mut empty = original.clone();
    let felt = original.as_view().unwrap().pixel_rgb(174, 120).unwrap();
    for x in crate::freecell::PLAY_X {
        for y in 331..947 {
            for pixel_x in x..x + 137 {
                let offset = usize::try_from(y).unwrap() * empty.stride
                    + usize::try_from(pixel_x).unwrap() * 4;
                empty.pixels[offset..offset + 4].copy_from_slice(&[felt[0], felt[1], felt[2], 255]);
            }
        }
        fixture_blit(
            &mut empty,
            &original,
            PixelRect::new(166, 112, 137, 182),
            (x, 331),
        );
    }
    let report = report();
    let mut parent = Position::initial(&report.deal);
    for _ in 0..52 {
        let Some(action) = parent.automatic() else {
            break;
        };
        parent = parent.apply(action).unwrap();
    }
    assert_eq!(parent.foundations(), &[13; 4]);
    let piles = std::array::from_fn::<_, 8, _>(|n| Pile::Cascade(n as u8));
    let geometry = position::verify_piles(&empty, &parent, &piles, &mut || Ok(())).unwrap();
    for (pile, x) in piles
        .into_iter()
        .zip([233, 425, 618, 810, 1003, 1195, 1388, 1580])
    {
        let anchor = PixelPoint::new(x, 351);
        assert_eq!(geometry.point(pile).unwrap(), anchor);
        assert_eq!(
            diagnostic_abs_path(PixelPoint::new(233, 722), anchor, pile, (1920, 1080)).unwrap()[2],
            anchor
        );
    }
}


/// Test Native's transport boundary, not a search result or guest movement effect.
fn exercise(case: Case, diagnostic_step: bool) {
    exercise_with_cadence(case, diagnostic_step, false);
}


/// Optionally measure bounded frame waits through the exact active native owner.
fn exercise_with_cadence(case: Case, diagnostic_step: bool, verify_cadence: bool) {
    let root = Socket::new();
    let mut id = plan(31);
    id.context = ControllerContext::new(
        root.path(),
        GameMode::FreeCell,
        SolvingStrategy::ShortestPath,
        7,
    );
    let (original, report, parent, mut action) = if case == Case::ReturnedJackToCellFour {
        returned_jack_fixture()
    } else if case == Case::CompleteAfterDrag {
        let original = crate::capture::decode_png(include_bytes!(
            "../../../../tests/fixtures/freecell-challenge-highlight-20261109.png"
        ))
        .unwrap();
        let deal = crate::freecell_recon::recognise(
            &original,
            Metadata::new("native-bridge", "challenge", "unit").unwrap(),
        )
        .unwrap();
        let parent = Position::initial(&deal);
        let action = RecordedMove {
            card_id: *parent.column(0).unwrap().last().unwrap(),
            from: Pile::Cascade(0),
            to: Pile::Cell(0),
            is_auto: false,
        };
        (original, prefix_report(deal), parent, action)
    } else {
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
        (original, prefix_report(deal), parent, action)
    };
    let deal = report.deal.clone();
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
    let geometry = if case == Case::ReturnedJackToCellFour {
        position::verify_piles(
            &original,
            &parent,
            &[action.from, action.to],
            &mut || Ok(()),
        )
    } else {
        position::verify(&original, &parent, &mut || Ok(()))
    }
    .unwrap();
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
        if case == Case::CompleteAfterDrag {
            let after_drag = crate::capture::decode_png(include_bytes!(
                "../../../../tests/fixtures/pyramid-objectives/boards-complete-continue.png"
            ))
            .unwrap();
            frames.push(V3Reply::Frame(after_drag.clone()));
            frames.push(V3Reply::Frame(after_drag));
        } else {
            let mut after_drag = original.clone();
            for pixel in after_drag.pixels.as_chunks_mut::<4>().0 {
                *pixel = [0, 0, 0, 255];
            }
            frames.push(V3Reply::Frame(after_drag));
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
    latest
        .reconnaissance
        .queue_reconnaissance(&id.context, &cancel, || Ok(()))
        .unwrap();
    {
        let mut store = latest.freecell_route.lock().unwrap();
        store.prepare(id.clone(), report).unwrap();
        if case == Case::ReturnedJackToCellFour {
            let cached = store.cached.as_ref().unwrap();
            let (moves, _) = recorded_route(&cached.report.outcome).unwrap();
            assert_eq!(moves.len(), 40);
            assert_eq!(moves[39], action);
            // Transport seam: replay owns this parent; no fresh original binding is claimed.
            let mut state = State::new(&deal);
            state.position = parent;
            state.cursor = 39;
            store.live = Some(Live {
                state,
                mode: route::AutoMode::ObserveGame,
            });
        }
    }
    let run_id = latest.freecell_route.lock().unwrap().reserve(&id).unwrap();
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
        challenge_mode: false,
        completion_observed: false,
        diagnostic_step,
    };
    if bind_initial {
        native.bind(&deal).unwrap();
        assert!(native.completion_bound);
        assert_native_preview(&latest, native.latest.as_ref().unwrap(), &id.context);
    }
    let planned_gesture = Arc::new(Mutex::new(None));
    let planned_trace = Arc::clone(&planned_gesture);
    let waits = Arc::new(Mutex::new(Vec::new()));
    let wait_trace = Arc::clone(&waits);
    let _probe_trace = crate::single_step_telemetry::Scope::freecell(
        json!({"native_case":format!("{case:?}")}),
        move |line| {
            let event: Value =
                serde_json::from_str(line.strip_prefix("FREE_CELL ").unwrap()).unwrap();
            if event["detail"]["event"] == "transport-attempt"
                && event["detail"]["operation"] == "diagnostic-drag"
            {
                *planned_trace.lock().unwrap() = Some(event["detail"]["details"].clone());
            }
            if event["detail"]["event"] == "playback-wait" {
                wait_trace.lock().unwrap().push(event["detail"].clone());
            }
        },
    );
    let result = native.transfer(&parent, action);
    match case {
        Case::Both | Case::PlayDestination | Case::ReturnedJackToCellFour => {
            let outcome = result.unwrap();
            assert_eq!(outcome, route::TransferOutcome::Delivered);
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
                route::Observation::Unmatched
            ));
            assert!(!native.completion_observed);
            assert!(native.first.is_some());
            assert!(native.resolve_challenge_complete().unwrap());
            assert_eq!(native.capture_serial, 4);
            assert!(native.completion_observed);
            assert_eq!(native.counts, (1, 1));
            assert_native_preview(&latest, native.latest.as_ref().unwrap(), &id.context);
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
            assert_native_preview(&latest, native.latest.as_ref().unwrap(), &id.context);
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
        Case::AbsoluteStopDuringHeldWait => {
            assert!(result.is_err());
            assert!(cancel.load(Ordering::SeqCst));
            assert_eq!(native.counts, (1, 0));
            assert_eq!(native.capture_serial, 1);
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
    if verify_cadence {
        assert_eq!(case, Case::Both);
        for configured_ms in [750, 40, 0] {
            native.settle = Duration::from_millis(configured_ms);
            native.post_transfer_wait().unwrap();
            native.confirmation_wait().unwrap();
            native.settle().unwrap();
        }
        let waits = waits.lock().unwrap();
        assert_eq!(waits.len(), 9);
        for (wait, (configured_ms, chosen_ms)) in waits.iter().zip([
            (750, 200),
            (750, 100),
            (750, 750),
            (40, 40),
            (40, 40),
            (40, 40),
            (0, 0),
            (0, 0),
            (0, 0),
        ]) {
            assert_eq!(wait["configured_ms"], configured_ms);
            assert_eq!(wait["chosen_ms"], chosen_ms);
            assert!(wait["elapsed_us"].as_u64().unwrap() >= chosen_ms * 1_000);
            assert_eq!(wait["additional_captures"], 0);
            assert_eq!(wait["interrupted"], false);
        }
        assert_eq!(native.capture_serial, 1);
        assert_eq!(native.counts, (1, 1));
        assert!(
            latest.take().is_none(),
            "waiting cannot manufacture preview frames"
        );
    }
    if case != Case::Both && case != Case::AbsoluteStopDuringHeldWait {
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
            "query-mice",
            "input-send-event",
        ]);
        if case != Case::AbsoluteStopDuringHeldWait {
            expected.extend(std::iter::repeat_n("input-send-event", 4));
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
    if case == Case::AbsoluteStopDuringHeldWait {
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
                | Case::ReturnedJackToCellFour
                | Case::CompleteAfterDrag
                | Case::UnsupportedAfterDrag
        )
    {
        assert_eq!(
            inputs.len(),
            6,
            "source then one held DOWN/source-lift/midpoint/destination/UP gesture; no extra click"
        );
        if case == Case::ReturnedJackToCellFour {
            assert_eq!(from, PixelPoint::new(233, 724));
            assert_eq!(to, PixelPoint::new(758, 132));
            movement(inputs[2], PixelPoint::new(233, 679));
            movement(inputs[3], PixelPoint::new(495, 428));
            movement(inputs[4], PixelPoint::new(758, 132));
        } else if case != Case::PlayDestination {
            assert_eq!(from.x, 233);
            assert!((668..=673).contains(&from.y));
            assert_eq!(to, PixelPoint::new(181, 132));
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
        movement(inputs[4], to);
        button(inputs[5], false);
        let planned = planned_gesture.lock().unwrap();
        let planned = planned.as_ref().unwrap();
        assert_eq!(planned["planned_input_requests"], 6);
        assert_eq!(planned["planned_held_motion_updates"], 3);
        assert_eq!(
            planned["source_acknowledgement_minimum_wait_ms"],
            json!(crate::parameters::FREECELL_ROUTE_ABS_GESTURE_INTERVAL.as_millis())
        );
        assert_eq!(
            planned["held_phase_minimum_wait_ms"],
            json!(crate::parameters::FREECELL_ROUTE_ABS_GESTURE_INTERVAL.as_millis())
        );
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
        && matches!(
            case,
            Case::Both
                | Case::PlayDestination
                | Case::ReturnedJackToCellFour
                | Case::AbsoluteStopDuringHeldWait
        )
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
            if case == Case::AbsoluteStopDuringHeldWait {
                4
            } else {
                6
            }
        );
        let abs_wait_us =
            u64::try_from(crate::parameters::FREECELL_ROUTE_ABS_GESTURE_INTERVAL.as_micros())
                .unwrap();
        if case == Case::AbsoluteStopDuringHeldWait {
            assert!(
                writes[1] - writes[0] >= abs_wait_us,
                "source-to-DOWN pacing completes before STOP in the held wait"
            );
            assert_eq!(
                input_connections,
                ["primary", "primary", "primary", "release-recovery"]
            );
        } else {
            for pair in writes.windows(2) {
                assert!(
                    pair[1] - pair[0] >= abs_wait_us,
                    "all five ABS intervals use the fixed 100ms wait"
                );
            }
            assert!(
                writes[5] - writes[0] >= 5 * abs_wait_us,
                "ABS source through release includes all five selected intervals"
            );
        }
    }
    let _ = receiver.try_iter().count();
}


/// Real TCP frame and fresh VM/pointer probe precede one explicit-target transfer.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_312_native_compound_capture_probe_input_order() {
    exercise(Case::Both, false);
}


/// The retained returned JC sends the corrected six-event ABS packet through local peers.
///
/// The authentic 39-edge parent fixes source(233,724), lift(233,679), midpoint(495,428)
/// and CELL4 destination(758,132). Acknowledgements establish delivery only.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_3119_native_returned_jc_to_cell4_keeps_pickup_offset_and_exact_path() {
    exercise(Case::ReturnedJackToCellFour, true);
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
        challenge_mode: false,
        completion_observed: false,
        diagnostic_step: false,
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
    assert_eq!(to, PixelPoint::new(181, 132));
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


/// A single playback request is frozen with its owner before queueing; duplicate work is refused.
#[test]
fn freecell_3120_diagnostic_mode_is_frozen_before_queue_with_fixed_abs_interval() {
    assert_eq!(
        crate::parameters::FREECELL_ROUTE_ABS_GESTURE_INTERVAL,
        Duration::from_millis(100)
    );
    let (command_tx, command_rx) = mpsc::channel();
    let (_, event_rx) = mpsc::channel();
    let latest_frame = LatestFrameSlot::default();
    let id = plan(3120);
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
    let run_id = handle
        .execute_freecell_route(
            id.clone(),
            true,
            route::AutoMode::ObserveGame,
            Duration::ZERO,
        )
        .unwrap();
    let WorkerCommand::ExecuteFreeCellRoute {
        plan,
        run_id: queued_run,
        single_step,
        settle,
    } = command_rx.try_recv().unwrap()
    else {
        panic!("expected one frozen diagnostic request")
    };
    assert_eq!(plan, id);
    assert_eq!(queued_run, run_id);
    assert!(single_step);
    assert_eq!(settle, Duration::ZERO);
    assert!(
        handle
            .execute_freecell_route(id, true, route::AutoMode::ObserveGame, Duration::ZERO)
            .is_err()
    );
    assert!(matches!(
        command_rx.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
}


/// SINGLE STEP sends six exact packets with five 100-millisecond gaps at an upper slot.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_3120_native_abs_single_step_has_five_100ms_gaps_and_lift_midpoint_path() {
    exercise(Case::Both, true);
}


/// Only confirmation and first-effect waits are capped; retry and smaller settings survive.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_3118_native_frame_cadence_keeps_retry_horizon_and_uses_no_extra_captures() {
    exercise_with_cadence(Case::Both, true, true);
}


/// A native PLAY transfer preserves its proved corner as the final exact destination.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_3120_native_abs_play_target_has_five_100ms_gaps_and_lift_midpoint_path() {
    exercise(Case::PlayDestination, true);
}


/// STOP during the 100-millisecond held wait permits release recovery without destination motion.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_3120_native_abs_100ms_stop_releases_without_motion_or_replay() {
    exercise(Case::AbsoluteStopDuringHeldWait, true);
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
        "../../../../tests/fixtures/pyramid-objectives/boards-complete-continue.png"
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
        vec![
            V3Reply::Frame(original.clone()),
            V3Reply::Frame(terminal.clone()),
            V3Reply::Frame(terminal),
        ]
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
        challenge_mode: false,
        completion_observed: false,
        diagnostic_step: false,
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
        assert!(native.completion_bound && native.challenge_mode);
        if preinput {
            let action = parent
                .automatic()
                .expect("native fixture has a root automatic transfer");
            assert_eq!(
                native.transfer(&parent, action).unwrap(),
                route::TransferOutcome::ChallengeComplete
            );
            assert!(native.first.is_some());
        } else {
            native.stage(route::Stage {
                cursor: 0,
                total: 1,
                mode: route::AutoMode::ObserveGame,
                phase: "readiness",
                observation: 1,
                action: parent.automatic(),
            });
            if recording == Recording::InvalidDestination {
                assert_eq!(
                    native
                        .observe(&[Candidate {
                            cursor: 0,
                            position: parent,
                        }])
                        .unwrap(),
                    route::Observation::Unmatched
                );
                assert!(native.first.is_some());
                assert!(!native.completion_observed);
                let error = native.resolve_challenge_complete().unwrap_err();
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
                    route::Observation::Unmatched
                );
                assert!(native.first.is_some());
                assert!(!native.completion_observed);
                assert!(native.resolve_challenge_complete().unwrap());
            }
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


/// Ordinary `FullBoard` mode ignores Challenge Complete even when `boards-complete-continue.png` appears.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_320_ordinary_game_ignores_challenge_complete_on_recognition_fault() {
    let root = Socket::new();
    let mut id = plan(3201);
    id.context = ControllerContext::new(
        root.path(),
        GameMode::FreeCell,
        SolvingStrategy::ShortestPath,
        3201,
    );
    let original = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/freecell-FC13.png"
    ))
    .unwrap();
    let terminal = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-objectives/boards-complete-continue.png"
    ))
    .unwrap();
    let deal = crate::freecell_recon::recognise(
        &original,
        Metadata::new(
            "native320",
            "ordinary",
            crate::freecell_recon::objective::FULL_BOARD_SOURCE,
        )
        .unwrap(),
    )
    .unwrap();
    let parent = Position::initial(&deal);
    let action = RecordedMove {
        card_id: *parent.column(0).unwrap().last().unwrap(),
        from: Pile::Cascade(0),
        to: Pile::Cell(0),
        is_auto: false,
    };
    let frames = vec![
        V3Reply::Frame(original.clone()),
        V3Reply::Frame(terminal.clone()),
    ];
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
        challenge_mode: false,
        completion_observed: false,
        diagnostic_step: false,
    };
    native.bind(&deal).unwrap();
    assert!(native.completion_bound && !native.challenge_mode);
    let fault = native.transfer(&parent, action).unwrap_err();
    assert!(!fault.is_empty());
    assert!(!native.completion_observed);
    assert_eq!(native.capture_serial, 2);
    assert!(!native.resolve_challenge_complete().unwrap());
    assert_eq!(native.capture_serial, 2);
    assert_eq!(native.first.as_ref().unwrap().pixels, terminal.pixels);
    latest
        .freecell_route
        .lock()
        .unwrap()
        .reject(&id, run_id)
        .unwrap();
    drop(native);
    drop(capture_scope);
    drop(cancel_scope);
    let (_requests, probes) = server.join().unwrap();
    assert_eq!(probes, 0);
    assert_eq!(tcp.finish(), 2);
}


/// Negative 5-frame Challenge Complete polling preserves original recognition fault and `first`.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_320_negative_challenge_complete_poll_preserves_original_fault_and_first_frame() {
    let root = Socket::new();
    let mut id = plan(3202);
    id.context = ControllerContext::new(
        root.path(),
        GameMode::FreeCell,
        SolvingStrategy::ShortestPath,
        3202,
    );
    let original = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/freecell-challenge-highlight-20261109.png"
    ))
    .unwrap();
    let mut damaged = original.clone();
    for y in 656..696_u32 {
        for x in 219..248_u32 {
            let offset = y as usize * damaged.stride + x as usize * 4;
            damaged.pixels[offset..offset + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let miss = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-objectives/fours-complete-animating.png"
    ))
    .expect("fours-complete-animating png");
    let deal = crate::freecell_recon::recognise(
        &original,
        Metadata::new("native320", "challenge-miss", "unit").unwrap(),
    )
    .unwrap();
    let parent = Position::initial(&deal);
    let action = RecordedMove {
        card_id: *parent.column(0).unwrap().last().unwrap(),
        from: Pile::Cascade(0),
        to: Pile::Cell(0),
        is_auto: false,
    };
    let mut frames = vec![
        V3Reply::Frame(original.clone()),
        V3Reply::Frame(damaged.clone()),
    ];
    for _ in 0..5 {
        frames.push(V3Reply::Frame(miss.clone()));
    }
    let (address, tcp) = v3_server_with_read_timeout(vec![frames], Duration::from_secs(60));
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
        challenge_mode: false,
        completion_observed: false,
        diagnostic_step: false,
    };
    native.bind(&deal).unwrap();
    assert!(native.completion_bound && native.challenge_mode);
    let fault = native.transfer(&parent, action).unwrap_err();
    assert!(fault.contains("PLAY 1"), "unexpected fault: {fault}");
    assert!(!native.completion_observed);
    assert_eq!(native.capture_serial, 7);
    assert_eq!(native.first.as_ref().unwrap().pixels, damaged.pixels);
    let mut store = latest.freecell_route.lock().unwrap();
    assert!(store.live.is_some());
    store.reject(&id, run_id).unwrap();
    drop(store);
    drop(native);
    drop(capture_scope);
    drop(cancel_scope);
    let (_requests, probes) = server.join().unwrap();
    assert_eq!(probes, 0);
    assert_eq!(tcp.finish(), 7);
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
            Duration::ZERO,
        )
        .unwrap();
    let command = command_rx.try_recv().unwrap();
    let _command_scope = crate::worker::qmp_wire::CommandScope::install(&command);
    let WorkerCommand::ExecuteFreeCellRoute {
        plan,
        run_id: queued_run,
        single_step,
        settle,
    } = command
    else {
        panic!("expected one exact Free Cell playback command");
    };
    assert_eq!(plan, id);
    assert_eq!(queued_run, run_id);
    assert!(single_step);
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
    run(plan, queued_run, single_step, settle, &sink, &cancel);
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
        assert_eq!(inputs.len(), 6);
        let to = PixelPoint::new(1248, 132);
        movement(inputs[0], from);
        button(inputs[1], true);
        movement(inputs[2], PixelPoint::new(from.x, from.y - 45));
        movement(
            inputs[3],
            PixelPoint::new((from.x + to.x) / 2, (from.y + to.y) / 2),
        );
        movement(inputs[4], to);
        button(inputs[5], false);
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


/// MULTI fixes ABS/100ms and proves both native transfers before accepting the endpoint.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_3120_native_multi_step_uses_two_verified_100ms_six_event_abs_gestures() {
    let fixture = club_followup_fixture();
    let report = prefix_report(fixture.report.deal.clone());
    let (moves, _) = recorded_route(&report.outcome).unwrap();
    assert_eq!(moves.len(), 2);
    let parent = Position::initial(&report.deal);
    let after_first = parent.apply(moves[0]).unwrap();
    let sources = [
        position::verify(&fixture.initial, &parent, &mut || Ok(()))
            .unwrap()
            .point(moves[0].from)
            .unwrap(),
        position::verify(&fixture.explicit, &after_first, &mut || Ok(()))
            .unwrap()
            .point(moves[1].from)
            .unwrap(),
    ];
    let root = Socket::new();
    let mut id = plan(3117);
    id.context = ControllerContext::new(
        root.path(),
        GameMode::FreeCell,
        SolvingStrategy::ShortestPath,
        3117,
    );
    let mut frames: Vec<_> = (0..4)
        .map(|_| V3Reply::Frame(fixture.initial.clone()))
        .collect();
    frames.extend((0..3).map(|_| V3Reply::Frame(fixture.explicit.clone())));
    frames.extend((0..2).map(|_| V3Reply::Frame(fixture.combined.clone())));
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
        .prepare(id.clone(), report)
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
            false,
            route::AutoMode::ExecuteRecorded,
            Duration::ZERO,
        )
        .unwrap();
    let command = command_rx.try_recv().unwrap();
    let _command_scope = crate::worker::qmp_wire::CommandScope::install(&command);
    let WorkerCommand::ExecuteFreeCellRoute {
        plan,
        run_id: queued_run,
        single_step,
        settle,
    } = command
    else {
        panic!("expected one MULTI STEP playback command")
    };
    assert_eq!(plan, id);
    assert_eq!(queued_run, run_id);
    assert!(!single_step);
    let server = qmp_server_for_parents(
        &root.path(),
        Case::Both,
        latest.clone(),
        id.context.clone(),
        Arc::clone(&cancel),
        vec![fixture.initial, fixture.explicit],
    );
    run(plan, queued_run, single_step, settle, &sink, &cancel);
    let events: Vec<_> = receiver.try_iter().collect();
    let progress = events
        .iter()
        .find_map(|event| {
            if let WorkerEvent::FreeCellRouteFinished {
                plan,
                run_id: finished,
                result,
                termination,
            } = &event.event
            {
                assert_eq!(plan, &id);
                assert_eq!(*finished, run_id);
                assert_eq!(
                    *termination,
                    crate::pyramid_workflow_model::RunTermination::Completed
                );
                Some(
                    result
                        .as_ref()
                        .unwrap_or_else(|failure| panic!("{}: {}", failure.error, failure.details)),
                )
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(
        progress.completion,
        Some(route::Completion::PartialRouteExhausted)
    );
    assert_eq!((progress.cursor, progress.total), (2, 2));
    assert_eq!((progress.attempted, progress.acknowledged), (2, 2));
    assert_eq!(progress.foundation_cards, 2);
    assert!(!latest.freecell_route.lock().unwrap().available(&id));
    drop(capture_scope);
    drop(cancel_scope);
    let (requests, probes) = server.join().unwrap();
    assert_eq!(probes, 2);
    assert_eq!(tcp.finish(), 9);
    let inputs: Vec<_> = requests
        .iter()
        .filter(|request| request["execute"] == "input-send-event")
        .collect();
    assert_eq!(inputs.len(), 12);
    for (gesture, source) in inputs.as_chunks::<6>().0.iter().zip(sources) {
        let to = PixelPoint::new(1248, 132);
        movement(gesture[0], source);
        button(gesture[1], true);
        movement(gesture[2], PixelPoint::new(source.x, source.y - 45));
        movement(
            gesture[3],
            PixelPoint::new((source.x + to.x) / 2, (source.y + to.y) / 2),
        );
        movement(gesture[4], to);
        button(gesture[5], false);
    }
    let writes: Vec<_> = events
        .iter()
        .filter_map(|event| {
            let WorkerEvent::QmpWireLog {
                metadata,
                packet_line,
            } = &event.event
            else {
                return None;
            };
            let packet: Value = serde_json::from_str(packet_line.lines().last().unwrap()).unwrap();
            if packet["execute"] != "input-send-event" {
                return None;
            }
            let metadata: Value =
                serde_json::from_str(metadata.strip_prefix("QMP_WIRE ").unwrap()).unwrap();
            assert_eq!(metadata["owner"]["single_step"], false);
            assert_eq!(metadata["owner"]["diagnostic_drag_mode"], "Absolute");
            assert_eq!(metadata["owner"]["gesture_phase_wait_ms"], 100);
            Some(metadata["wire_elapsed_us"].as_u64().unwrap())
        })
        .collect();
    assert_eq!(writes.len(), 12);
    for gesture in writes.as_chunks::<6>().0 {
        assert!(gesture.windows(2).all(|pair| pair[1] - pair[0] >= 100_000));
    }
    assert!(
        writes[6] - writes[5] >= 500_000,
        "the proved AUTO record waits at least 500ms before the next source event"
    );
    assert!(matches!(
        command_rx.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    drop(handle);
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
            vec![V3Reply::Frame(different.clone())],
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
            .execute_freecell_route(id.clone(), single_step, mode, Duration::ZERO)
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
        run(plan, queued_run, single_step, settle, &sink, &cancel);
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
    assert_native_preview(&latest, &fixture.initial, &id.context);
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
    assert_native_preview(&latest, &different, &id.context);
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
    assert_native_preview(&latest, &fixture.combined, &id.context);
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

/// Option 1A Challenge Complete handoffs (`XpCounting` and `LevelUp`) retire cleanly after 5-frame poll with zero input.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_320_xp_counting_and_level_up_retire_after_five_frame_poll_without_input() {
    let original = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/freecell-challenge-highlight-20261109.png"
    ))
    .expect("original challenge png");
    let mut damaged = original.clone();
    for y in 656..696_u32 {
        for x in 219..248_u32 {
            let offset = y as usize * damaged.stride + x as usize * 4;
            damaged.pixels[offset..offset + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let deal = crate::freecell_recon::recognise(
        &original,
        Metadata::new("native320", "challenge-1a", "unit").expect("valid metadata"),
    )
    .expect("recognise challenge deal");
    let parent = Position::initial(&deal);
    let action = RecordedMove {
        card_id: *parent.column(0).expect("col 0").last().expect("last card"),
        from: Pile::Cascade(0),
        to: Pile::Cell(0),
        is_auto: false,
    };

    for (idx, (label, bytes, expected_log)) in [
        (
            "xp",
            include_bytes!(
                "../../../../tests/fixtures/freecell-challenge-complete-xp-20261113.png"
            )
            .as_slice(),
            "XP counting active after 5 observations",
        ),
        (
            "level_up",
            include_bytes!("../../../../tests/fixtures/freecell-FC10.png").as_slice(),
            "Level Up OK modal active",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let root = Socket::new();
        let mut id = plan(3210 + u64::try_from(idx).expect("idx fits u64"));
        id.context = ControllerContext::new(
            root.path(),
            GameMode::FreeCell,
            SolvingStrategy::ShortestPath,
            id.request_id,
        );
        let handoff_frame = crate::capture::decode_png(bytes).expect("handoff png");
        let mut frames = vec![
            V3Reply::Frame(original.clone()),
            V3Reply::Frame(damaged.clone()),
        ];
        for _ in 0..5 {
            frames.push(V3Reply::Frame(handoff_frame.clone()));
        }
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
        let _recording = RecordingRestore(WORKER_IMAGE_DIAGNOSTICS.with(|slot| slot.replace(None)));
        let cancel_scope = tcp_capture::CancellationScope::install(Arc::clone(&cancel));
        let capture_scope = tcp_capture::WorkerCaptureScope::install(&sink);
        sink.set_capture_context(Some(id.context.clone()));
        latest
            .reconnaissance
            .select(&id.context, &cancel)
            .expect("select recon");
        latest
            .reconnaissance
            .queue_reconnaissance(&id.context, &cancel, || Ok(()))
            .expect("queue recon");
        let run_id = {
            let mut store = latest.freecell_route.lock().expect("lock store");
            store
                .prepare(id.clone(), prefix_report(deal.clone()))
                .expect("prepare route");
            store.reserve(&id).expect("reserve route")
        };
        let server = qmp_server(
            &root.path(),
            Case::Both,
            latest.clone(),
            id.context.clone(),
            Arc::clone(&cancel),
            original.clone(),
        );
        let trace = pyramid_stop::StopTrace::default();
        let mut native = Native {
            plan: &id,
            run_id,
            qmp: QmpClient::connect(&root.path()).expect("connect qmp"),
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
            completion_bound: true,
            challenge_mode: false,
            completion_observed: false,
            diagnostic_step: false,
        };
        native.bind(&deal).expect("bind deal");
        assert!(native.completion_bound && native.challenge_mode);
        let outcome = native
            .transfer(&parent, action)
            .unwrap_or_else(|err| panic!("{label} transfer failed: {err}"));
        assert_eq!(outcome, route::TransferOutcome::ChallengeComplete);
        assert!(native.completion_observed);
        assert_eq!(native.capture_serial, 7);
        assert_eq!(native.counts, (0, 0));
        let mut saw_handoff_log = false;
        while let Ok(event) = receiver.try_recv() {
            if let WorkerEvent::Log(line) = event.event
                && line.contains(expected_log)
            {
                saw_handoff_log = true;
            }
        }
        assert!(
            saw_handoff_log,
            "{label} missing expected handoff log substring {expected_log}"
        );
        latest
            .freecell_route
            .lock()
            .expect("lock store")
            .reject(&id, run_id)
            .expect("reject route");
        drop(native);
        drop(capture_scope);
        drop(cancel_scope);
        let (requests, probes) = server.join().expect("join qmp");
        assert_eq!(probes, 0, "{label} must send zero QMP probes");
        assert!(
            !requests
                .iter()
                .any(|request| request["execute"] == "input-send-event"),
            "{label} must send zero QMP inputs"
        );
        assert_eq!(tcp.finish(), 7, "{label} capture count");
    }
}


/// Native `resolve_normal_game_win` requires two consecutive terminal frames on normal deals
/// and refuses both challenge deals and non-terminal frames without mutating `first`.
#[test]
#[ignore = "requires local AF_UNIX and TCP sockets"]
fn freecell_330_native_normal_game_win_poll_requires_two_consecutive_matches_and_respects_challenge_mode()
 {
    for challenge_mode in [false, true] {
        let root = Socket::new();
        let mut id = plan(3301);
        id.context = ControllerContext::new(
            root.path(),
            GameMode::FreeCell,
            SolvingStrategy::ShortestPath,
            3301,
        );
        let original = crate::capture::decode_png(include_bytes!(
            "../../../../tests/fixtures/freecell-FC13.png"
        ))
        .expect("decode FC13");
        let win_frame = crate::capture::decode_png(include_bytes!(
            "../../../../tests/fixtures/freecell-FC09.png"
        ))
        .expect("decode FC09");
        let meta = if challenge_mode {
            Metadata::new("native330", "challenge", "unit").expect("meta")
        } else {
            Metadata::new("native330", "normal", "unit").expect("meta")
        };
        let deal = crate::freecell_recon::recognise(&original, meta).expect("recognise FC13");
        let goal = Position::initial(&deal);
        let frames = if challenge_mode {
            vec![V3Reply::Frame(original.clone())]
        } else {
            // Miss once, then return two consecutive FC09 GAME WIN frames.
            vec![
                V3Reply::Frame(original.clone()),
                V3Reply::Frame(original.clone()),
                V3Reply::Frame(win_frame.clone()),
                V3Reply::Frame(win_frame.clone()),
            ]
        };
        let expected_captures = frames.len();
        let (address, tcp) = v3_server_with_read_timeout(vec![frames], Duration::from_secs(60));
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
            original.clone(),
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
            first: Some(original.clone()),
            frozen: None,
            stage: None,
            capture_serial: 0,
            first_rejection: None,
            last_rejection: None,
            counts: (0, 0),
            settle: Duration::ZERO,
            completion_bound: true,
            challenge_mode: false,
            completion_observed: false,
            diagnostic_step: true,
        };
        native.bind(&deal).unwrap();
        native.challenge_mode = challenge_mode;
        let resolved = native.resolve_normal_game_win(&goal).unwrap();
        assert_eq!(resolved, !challenge_mode);
        assert_eq!(native.first.as_ref().unwrap().pixels, original.pixels);
        latest
            .freecell_route
            .lock()
            .unwrap()
            .reject(&id, run_id)
            .unwrap();
        drop(native);
        drop(capture_scope);
        drop(cancel_scope);
        let (requests, _probes) = server.join().unwrap();
        assert!(
            !requests
                .iter()
                .any(|request| request["execute"] == "input-send-event")
        );
        assert_eq!(tcp.finish(), expected_captures);
    }
}
