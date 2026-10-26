//! Optional series checks on the production computed-route capture/input adapter.
//!
//! Normal-one header/footer overlays and erased/empty derivatives below are
//! explicitly synthetic test scenes. They retain the known native-v7 card
//! identities but establish no acquisition, normal ordinal-two/three calibration,
//! guest effect or live acceptance. A private local QMP peer exercises the actual
//! NativeIo capture, exact-position checks, delivery and final classifier.

use super::*;
use crate::{
    capture::{decode_png, encode_png},
    geometry::PixelRect,
    pyramid_progress::{BoardCount, observe_normal},
    worker::{LatestFrameSlot, TestCaptureBackend},
};
use serde_json::{Value, json};
use std::{
    cell::Cell,
    collections::VecDeque,
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::{
        fs::DirBuilderExt,
        net::{UnixListener, UnixStream},
    },
    path::PathBuf,
    sync::{Mutex, mpsc},
    thread,
};


mod board_count_tests;


mod target_completion_tests;


mod native_123_tests;


mod cadence_tests;


mod exhausted_left_tests;




/// Accepted deal's original untouched board, before any synthetic header overlay.
const START: &[u8] = include_bytes!("../../../tests/fixtures/pyramid-recon/native-v7/baseline.png");


/// Complete measured normal-one patch; overlay use remains synthetic.
const NORMAL_HEADER: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-standard-progress/header-one.rgb");


/// Complete measured New footer; overlay use remains synthetic.
const NORMAL_FOOTER: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-standard-progress/footer-new.rgb");


/// Complete literal normal-series header bounds, outside all card recognition ROIs.
const HEADER: PixelRect = PixelRect::new(1658, 46, 160, 32);


/// Complete measured footer bounds, outside card and stock recognition ROIs.
const FOOTER: PixelRect = PixelRect::new(212, 998, 58, 31);


/// Retained specific Pyramid terminal, without synthesised control decoration.
const TERMINAL: &[u8] =
    include_bytes!("../../../tests/fixtures/ok-confirmation-20261016/pyramid/clean.png");


/// Apply complete packed RGB to a checked native rectangle, preserving alpha.
fn patch(frame: &mut CapturedFrame, bounds: PixelRect, rgb: &[u8]) {
    frame.as_view().unwrap().checked_bounds(bounds).unwrap();
    assert_eq!(
        rgb.len(),
        bounds.width as usize * bounds.height as usize * 3
    );
    let (pixels, remainder) = rgb.as_chunks::<3>();
    assert!(remainder.is_empty());
    let mut pixels = pixels.iter();
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            let offset = y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(pixels.next().unwrap());
        }
    }
    assert!(pixels.next().is_none());
}


/// Paint only an explicitly synthetic bounded derivative; preserve storage/alpha.
fn paint(frame: &mut CapturedFrame, bounds: PixelRect, rgb: [u8; 3]) {
    frame.as_view().unwrap().checked_bounds(bounds).unwrap();
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            let offset = y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&rgb);
        }
    }
}


/// Overlay real normal-one text onto the known accepted deal; this is synthetic.
fn synthetic_start() -> CapturedFrame {
    let mut frame = decode_png(START).expect("retained native-v7 board");
    patch(&mut frame, HEADER, NORMAL_HEADER);
    patch(&mut frame, FOOTER, NORMAL_FOOTER);
    assert_eq!(
        observe_normal(&frame),
        Ok(Some(BoardCount {
            completed: 0,
            target: 3
        }))
    );
    frame
}


/// Erase only measured header/footer pixels, preserving all expected card identities.
fn drift(mut frame: CapturedFrame, bounds: PixelRect) -> CapturedFrame {
    paint(&mut frame, bounds, [255; 3]);
    assert_eq!(observe_normal(&frame), Ok(None));
    frame
}


/// Real normal-one predicate, borrowed by the adapter without extra capture or input.
fn require_normal_one(
    frame: &CapturedFrame,
    _phase: FrameGuardPhase,
) -> Result<FrameGuardOutcome, String> {
    if observe_normal(frame)?
        == Some(BoardCount {
            completed: 0,
            target: 3,
        })
    {
        Ok(FrameGuardOutcome::Ready)
    } else {
        Err("normal series header/footer changed".to_owned())
    }
}


/// Parse only the retained oracle's original locations; no controller is replicated.
fn location(text: &str) -> Location {
    match text {
        "DL" => Location::DL,
        "DR" => Location::DR,
        _ => {
            let (row, column) = text.split_once('|').unwrap();
            xsar::games::pyramid::planning::location_from_row_column(
                row.parse().unwrap(),
                column.parse().unwrap(),
            )
            .unwrap()
        }
    }
}


/// Independently replay the accepted route to obtain a legal actual-goal expectation.
fn accepted_actions() -> Vec<RecordedAction> {
    let text =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../xsar/tests/fixtures/pyramid-planning/pyramid-recon-20261019-v7-01-solver.csv",
        ))
        .unwrap();
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("SEQUENCE,ACTION,CARD1,CARD2"));
    lines
        .enumerate()
        .map(|(index, line)| {
            let fields: Vec<_> = line.split(',').collect();
            assert_eq!(fields.len(), 4);
            assert_eq!(fields[0].parse::<usize>().unwrap(), index + 1);
            match fields[1] {
                "MATCH" => RecordedAction::Match(location(fields[2]), location(fields[3])),
                "KING" => RecordedAction::King(location(fields[2])),
                "MOVE" => RecordedAction::Move,
                "RECYCLE" => RecordedAction::Recycle,
                other => panic!("unknown retained action: {other}"),
            }
        })
        .collect()
}


/// Supply a private test plan; this bypasses no production acquisition API.
fn plan(context: &ControllerContext, epoch: u64) -> Prepared {
    let bytes = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../xsar/tests/fixtures/pyramid-planning/pyramid-recon-20261019-v7-01.json"),
    )
    .unwrap();
    let deal = xsar::pyramid_json::read_from(bytes.as_slice()).unwrap();
    let trace = replay::replay(&deal, &accepted_actions(), true).unwrap();
    let vision = ReconVision::load(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon/native-v7/profile.json"),
    )
    .unwrap();
    Prepared {
        id: 17,
        acquisition: Acquisition {
            challenge_progress: None,
            default_method: RouteMethod::FewestActions,
            observed_game: crate::pyramid_workflow_model::ObservedGame::Unknown,
            context: context.clone(),
            epoch,
            deal,
            bytes,
            vision,
        },
        trace,
        objective: RouteObjective::Clear,
    }
}


/// Reuse exact positive empty-pile RGB from the unchanged profile, only in a derivative.
fn empty_pile_patch(frame: &mut CapturedFrame, name: &str) {
    let profile: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v7/profile.json",
    ))
    .unwrap();
    let pattern = &profile["board"][name];
    let bounds = pattern["bounds"].as_array().unwrap();
    let bounds = PixelRect::new(
        u32::try_from(bounds[0].as_u64().unwrap()).unwrap(),
        u32::try_from(bounds[1].as_u64().unwrap()).unwrap(),
        u32::try_from(bounds[2].as_u64().unwrap()).unwrap(),
        u32::try_from(bounds[3].as_u64().unwrap()).unwrap(),
    );
    let hex = pattern["samples"]["rgb"].as_str().unwrap();
    let (pairs, remainder) = hex.as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty());
    let rgb: Vec<_> = pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    patch(frame, bounds, &rgb);
}


/// Explicitly synthetic empty tableau/piles; expected goal comes from independent replay.
fn synthetic_goal() -> CapturedFrame {
    let mut frame = synthetic_start();
    for slot in crate::pyramid::PYRAMID_TARGETS {
        if slot.kind.is_card() {
            paint(&mut frame, slot.bounds, [30, 95, 65]);
        }
    }
    for bounds in [
        PixelRect::new(759, 678, 139, 187),
        PixelRect::new(1022, 678, 139, 187),
    ] {
        paint(&mut frame, bounds, [30, 95, 65]);
    }
    empty_pile_patch(&mut frame, "left_empty");
    empty_pile_patch(&mut frame, "right_empty");
    frame
}


/// Return the rules-derived final state without constructing private mask/cursor fields.
fn goal(plan: &Prepared) -> SearchState {
    let state = plan
        .trace
        .iter()
        .fold(SearchState::initial(), |state, step| {
            rules::apply(&plan.acquisition.deal, state, step.action).unwrap()
        });
    assert!(state.is_goal());
    state
}


/// Own only a newly created private directory; no shared environment/global counter is used.
struct Directory(PathBuf);


impl Drop for Directory {
    /// Delete only owned socket/empty directory; unexpected diagnostic files remain inspectable.
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0.join("qmp.sock"));
        let _ = fs::remove_dir(&self.0);
    }
}


/// Allocate a unique mode-0700 local transport directory without global configuration.
fn directory() -> Directory {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    for attempt in 0..100u32 {
        let path = std::env::temp_dir().join(format!(
            "route-series-{}-{stamp}-{attempt}",
            std::process::id()
        ));
        match fs::DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => return Directory(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => panic!("private series fixture directory: {error}"),
        }
    }
    panic!("private series fixture directory collision budget exhausted");
}


/// Actual peer-observed commands and fully acknowledged input transitions.
#[derive(Debug, Default)]
struct Wire {
    /// Ordered commands from the real adapter, including captures and capability probes.
    commands: Vec<String>,
    /// Actual typed events, excluding acknowledgements and synthesized logical results.
    events: Vec<Value>,
    /// Scripted frames left unused when the production adapter stopped.
    unused_frames: usize,
}


/// Write one protocol packet and flush before waiting for the next request.
fn packet(stream: &mut UnixStream, value: Value) {
    serde_json::to_writer(&mut *stream, &value).unwrap();
    stream.write_all(b"\n").unwrap();
    stream.flush().unwrap();
}


/// Serve only local screenshots, fresh probes and ACKed typed input, with finite reads.
fn serve(listener: UnixListener, directory: PathBuf, mut frames: VecDeque<Vec<u8>>) -> Wire {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "series mock accept timed out");
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("series mock accept: {error}"),
        }
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    packet(
        &mut stream,
        json!({"QMP": {"version": {"qemu": {"major": 11, "minor": 0, "micro": 0}}, "capabilities": []}}),
    );
    let mut report = Wire::default();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap() == 0 {
            break;
        }
        let request: Value = serde_json::from_str(&line).unwrap();
        let id = request["id"].as_u64().unwrap();
        let command = request["execute"].as_str().unwrap();
        report.commands.push(command.to_owned());
        let response = match command {
            "qmp_capabilities" => json!({}),
            "screendump" => {
                let filename = Path::new(request["arguments"]["filename"].as_str().unwrap());
                assert_eq!(filename.parent(), Some(directory.as_path()));
                assert_eq!(request["arguments"]["format"], "png");
                fs::write(
                    filename,
                    frames.pop_front().expect("no extra adapter capture"),
                )
                .unwrap();
                json!({})
            }
            "query-status" => json!({"running": true, "status": "running"}),
            "query-mice" => {
                json!([{"name": "Private absolute tablet", "index": 0, "current": true, "absolute": true}])
            }
            "input-send-event" => {
                report.events.extend(
                    request["arguments"]["events"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .cloned(),
                );
                json!({})
            }
            other => panic!("unexpected guarded adapter command: {other}"),
        };
        packet(&mut stream, json!({"return": response, "id": id}));
    }
    report.unused_frames = frames.len();
    report
}


/// Return production adapter accounting/diagnostics separately from wire observations.
struct Evidence<T> {
    /// Value returned by the test operation on actual NativeIo.
    result: T,
    /// Counts changed only by production delivery.
    counts: Counts,
    /// Existing total capture allowance consumption.
    captures: usize,
    /// Normal retained first-mismatch reason and exact same decoded frame.
    first: Option<(String, CapturedFrame)>,
    /// Existing retained latest observation.
    latest: Option<CapturedFrame>,
    /// Actual observed transport operations.
    wire: Wire,
}


/// Run actual NativeIo against isolated PNG transport; never emulate route-controller logic.
fn adapter<T>(
    frames: Vec<CapturedFrame>,
    guard: Option<&FrameGuard<'_>>,
    operation: impl FnOnce(&mut NativeIo<'_>) -> T,
) -> Evidence<T> {
    adapter_with_setup(frames, guard, |_| {}, operation)
}


/// Prepare immutable fixture state before spawning the bounded local peer.
fn adapter_with_setup<T>(
    frames: Vec<CapturedFrame>,
    guard: Option<&FrameGuard<'_>>,
    setup: impl FnOnce(&mut Prepared),
    operation: impl FnOnce(&mut NativeIo<'_>) -> T,
) -> Evidence<T> {
    adapter_with_progress(frames, guard, setup, 0, operation)
}


/// Run the same native harness with an explicit expected driver-committed prefix.
fn adapter_with_progress<T>(
    frames: Vec<CapturedFrame>,
    guard: Option<&FrameGuard<'_>>,
    setup: impl FnOnce(&mut Prepared),
    expected_verified: usize,
    operation: impl FnOnce(&mut NativeIo<'_>) -> T,
) -> Evidence<T> {
    let _capture = TestCaptureBackend::legacy_qmp();
    let directory = directory();
    let socket = directory.0.join("qmp.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let context =
        ControllerContext::new(socket, GameMode::Pyramid, SolvingStrategy::ShortestPath, 3);
    let cancel = AtomicBool::new(false);
    let (tx, _rx) = mpsc::channel();
    let events = WorkerEventSink {
        events: tx,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(context.clone())),
    };
    let control = &events.latest_frame.reconnaissance;
    control.select(&context, &cancel).unwrap();
    control
        .queue_reconnaissance(&context, &cancel, || Ok(()))
        .unwrap();
    let mut plan = plan(&context, control.epoch().unwrap());
    setup(&mut plan);
    let pngs = frames
        .iter()
        .map(|frame| encode_png(frame).unwrap())
        .collect();
    let peer_directory = directory.0.clone();
    let peer = thread::spawn(move || serve(listener, peer_directory, pngs));
    let qmp = QmpClient::connect(&context.socket_path).unwrap();
    let mut io = NativeIo {
        policy: ExecutionPolicy::Standard,
        stock_continuation: false,
        executed_recycles: 0,
        qmp,
        plan: &plan,
        context: &context,
        events: &events,
        cancel: &cancel,
        frame_guard: guard,
        started: Instant::now(),
        paused: Duration::ZERO,
        captures: 0,
        capture_limit: 20,
        counts: Counts::default(),
        first_mismatch: None,
        latest_frame: None,
        verified_actions: 0,
        final_confirmation: FinalConfirmation::default(),
        challenge: plan.acquisition.challenge_progress,
        challenge_initial: plan.acquisition.challenge_progress,
        challenge_confirmation: None,
        position_matches: false,
        source_proof: None,
        action_permit: None,
        redeal_before: None,
    };
    let result = operation(&mut io);
    let counts = io.counts;
    let captures = io.captures;
    let first = io.first_mismatch.take();
    let latest = io.latest_frame.take();
    assert_eq!(
        io.verified_actions, expected_verified,
        "only the driver commits the expected independently verified prefix"
    );
    drop(io);
    // An injected context change revokes authority but must retain its owner's reservation.
    let ownership_probe = AtomicBool::new(false);
    if control.epoch().unwrap() == plan.acquisition.epoch {
        assert!(
            control.check_active(&context, &ownership_probe).is_ok(),
            "adapter cannot release its owner's reservation"
        );
    } else {
        assert!(control.check_active(&context, &ownership_probe).is_err());
        let sent = std::cell::Cell::new(false);
        assert!(
            control
                .queue_execution(&ownership_probe, || {
                    sent.set(true);
                    Ok(())
                })
                .is_err()
        );
        assert!(
            !sent.get(),
            "a revoked reservation still blocks replacement execution"
        );
    }
    let wire = peer.join().unwrap();
    Evidence {
        result,
        counts,
        captures,
        first,
        latest,
        wire,
    }
}


/// Guard fixtures must preserve exact same-card recognition independently of the header reader.
#[test]
fn synthetic_series_header_derivatives_preserve_native_position() {
    let context = ControllerContext::new(
        "/unused/series-test.sock".into(),
        GameMode::Pyramid,
        SolvingStrategy::ShortestPath,
        3,
    );
    let plan = plan(&context, 9);
    let (tableau, left, right) = expected(&plan.acquisition.deal, SearchState::initial()).unwrap();
    for frame in [
        synthetic_start(),
        drift(synthetic_start(), HEADER),
        drift(synthetic_start(), FOOTER),
    ] {
        plan.acquisition
            .vision
            .verify_position(&frame, &tableau, left, right)
            .unwrap();
    }
    let final_state = goal(&plan);
    let (tableau, left, right) = expected(&plan.acquisition.deal, final_state).unwrap();
    assert_eq!((tableau, left, right), ([None; 28], None, None));
    plan.acquisition
        .vision
        .verify_position(&synthetic_goal(), &tableau, left, right)
        .unwrap();
}


/// One bound completion frame stops before hidden expected cards can cause a mismatch.
#[test]
fn native_challenge_terminal_releases_from_one_source_frame_without_card_mismatch() {
    let frame = decode_png(include_bytes!(
        "../../../tests/fixtures/pyramid-objectives/fours-complete.png"
    ))
    .unwrap();
    let bound =
        ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Cards, 4, 12).unwrap();
    let evidence = adapter(vec![frame], None, |io| {
        io.challenge = Some(bound);
        let EffectObservation::Challenge(proof) =
            io.observe_source(SearchState::initial()).unwrap()
        else {
            panic!("one positive bound Challenge Complete source")
        };
        assert_eq!(proof.progress().current, 12);
        assert!(
            !proof.operator_continue(),
            "the actual fixture is the counting stage"
        );
        crate::worker::post_game::finish_pyramid_challenge(
            io.context,
            io.latest_frame.as_ref().unwrap(),
            proof,
            io.events,
            io.cancel,
        )
        .unwrap();
        assert!(
            io.begin_action(SearchState::initial(), &[InputOperation::PressDrawKey])
                .is_err()
        );
    });
    assert_eq!(evidence.captures, 1);
    assert_eq!(evidence.counts.acknowledged, 0);
    assert!(
        evidence.first.is_none(),
        "terminal classification precedes card validation"
    );
    assert!(evidence.wire.events.is_empty());
    assert_eq!(evidence.wire.unused_frames, 0);
}


/// A synthetic erased terminal header tests binding without creating a final numeric reading.
#[test]
fn native_common_challenge_terminal_preserves_last_actual_score_and_rejects_wrong_literal() {
    let mut frame = decode_png(include_bytes!(
        "../../../tests/fixtures/pyramid-objectives/fours-complete.png"
    ))
    .unwrap();
    paint(&mut frame, PixelRect::new(814, 48, 296, 32), [255; 3]);
    let bound =
        ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Score, 1280, 3000).unwrap();
    let evidence = adapter(vec![frame], None, |io| {
        io.challenge = Some(bound);
        let EffectObservation::Challenge(proof) =
            io.observe_effect(SearchState::initial(), false).unwrap()
        else {
            panic!("bound common event")
        };
        assert_eq!(proof.progress(), bound);
        assert!(!proof.progress().target_reached());
        assert!(
            proof
                .description()
                .contains("final numeric counter not read")
        );
    });
    assert_eq!(evidence.captures, 1);
    assert!(evidence.wire.events.is_empty());
    let frame = decode_png(include_bytes!(
        "../../../tests/fixtures/pyramid-objectives/fours-complete.png"
    ))
    .unwrap();
    let evidence = adapter(vec![frame], None, |io| {
        io.challenge = Some(bound);
        io.observe_effect(SearchState::initial(), false)
            .err()
            .unwrap()
    });
    assert!(
        evidence
            .result
            .contains("Challenge Complete requirement differs from the bound challenge"),
        "{}",
        evidence.result
    );
    assert!(evidence.wire.events.is_empty());
}


/// Same-card header or footer drift refuses before fresh probes or any gesture.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn guarded_native_delivery_refuses_series_drift_before_first_gesture() {
    for bounds in [HEADER, FOOTER] {
        let frame = drift(synthetic_start(), bounds);
        let evidence = adapter(vec![frame.clone()], Some(&require_normal_one), |io| {
            super::wire_tests::checked_delivery(
                io,
                SearchState::initial(),
                &[operation(location("7|6")).unwrap()],
            )
        });
        assert!(
            evidence
                .result
                .unwrap_err()
                .contains("frame guard rejected")
        );
        assert_eq!(
            (
                evidence.counts.attempted,
                evidence.counts.acknowledged,
                evidence.captures
            ),
            (0, 0, 1)
        );
        assert!(evidence.wire.events.is_empty());
        assert_eq!(evidence.wire.commands, ["qmp_capabilities", "screendump"]);
        let (reason, first) = evidence.first.unwrap();
        assert!(reason.contains("normal series header/footer changed"));
        assert_eq!(first.pixels, frame.pixels);
        assert_eq!(evidence.latest.unwrap().pixels, frame.pixels);
        assert_eq!(evidence.wire.unused_frames, 0);
    }
}


/// Header drift at the next whole-action boundary stops before any new input.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn guarded_native_delivery_stops_series_drift_before_next_whole_action() {
    for bounds in [HEADER, FOOTER] {
        let evidence = adapter(
            vec![synthetic_start(), drift(synthetic_start(), bounds)],
            Some(&require_normal_one),
            |io| {
                super::wire_tests::checked_delivery(
                    io,
                    SearchState::initial(),
                    &[operation(location("7|6")).unwrap()],
                )
                .unwrap();
                super::wire_tests::checked_delivery(
                    io,
                    SearchState::initial(),
                    &[operation(location("7|7")).unwrap()],
                )
            },
        );
        assert!(
            evidence
                .result
                .unwrap_err()
                .contains("frame guard rejected")
        );
        assert_eq!(
            (
                evidence.counts.attempted,
                evidence.counts.acknowledged,
                evidence.captures
            ),
            (1, 1, 2)
        );
        let transitions: Vec<_> = evidence
            .wire
            .events
            .iter()
            .filter_map(|event| event["data"]["down"].as_bool())
            .collect();
        assert_eq!(transitions, [true, false]);
        assert_eq!(
            evidence
                .wire
                .commands
                .iter()
                .filter(|command| *command == "query-mice")
                .count(),
            1
        );
        assert_eq!(
            evidence.wire.commands.last().map(String::as_str),
            Some("screendump")
        );
        assert_eq!(evidence.wire.unused_frames, 0);
    }
}


/// Existing unguarded adapter callers keep the same card/scene policy and ACK accounting.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn unguarded_native_delivery_preserves_existing_header_independent_behaviour() {
    let frame = drift(drift(synthetic_start(), HEADER), FOOTER);
    let evidence = adapter(vec![frame], None, |io| {
        super::wire_tests::checked_delivery(
            io,
            SearchState::initial(),
            &[operation(location("7|6")).unwrap()],
        )
    });
    assert!(evidence.result.is_ok());
    assert_eq!(
        (
            evidence.counts.attempted,
            evidence.counts.acknowledged,
            evidence.captures
        ),
        (1, 1, 1)
    );
    assert!(evidence.first.is_none());
    assert_eq!(
        evidence
            .wire
            .events
            .iter()
            .filter_map(|event| event["data"]["down"].as_bool())
            .collect::<Vec<_>>(),
        [true, false]
    );
}


/// Exact empty-goal pixels still require the guard; refusal cannot become BoardClear.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn guarded_native_final_goal_accepts_current_series_and_refuses_changed_series() {
    for changed in [false, true] {
        let frame = if changed {
            drift(synthetic_goal(), HEADER)
        } else {
            synthetic_goal()
        };
        let evidence = adapter(vec![frame], Some(&require_normal_one), |io| {
            io.observe_final(goal(io.plan))
        });
        if changed {
            assert!(
                evidence
                    .result
                    .unwrap_err()
                    .contains("frame guard rejected")
            );
            assert!(
                evidence
                    .first
                    .unwrap()
                    .0
                    .contains("normal series header/footer changed")
            );
        } else {
            assert_eq!(evidence.result.unwrap(), Some(BoardCompletion::BoardClear));
            assert!(evidence.first.is_none());
        }
        assert_eq!(
            (
                evidence.counts.attempted,
                evidence.counts.acknowledged,
                evidence.captures
            ),
            (0, 0, 1)
        );
        assert!(evidence.wire.events.is_empty());
    }
}


/// Hidden/changed old headers on unmatched positive terminal/new-board frames are not guarded.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn guarded_native_final_preserves_specific_terminal_and_fresh_board_classifier() {
    for (png, expected_completion) in [
        (TERMINAL, BoardCompletion::GameComplete),
        (START, BoardCompletion::NextBoardReady),
    ] {
        let calls = Cell::new(0usize);
        let guard = |_: &CapturedFrame, _phase| {
            calls.set(calls.get() + 1);
            Err("old gameplay header must not be required here".to_owned())
        };
        let frame = decode_png(png).unwrap();
        let evidence = adapter(vec![frame.clone(), frame], Some(&guard), |io| {
            let state = goal(io.plan);
            assert_eq!(io.observe_final(state).unwrap(), None);
            io.observe_final(state)
        });
        assert_eq!(evidence.result.unwrap(), Some(expected_completion));
        assert_eq!(calls.get(), 0);
        assert_eq!(
            (
                evidence.counts.attempted,
                evidence.counts.acknowledged,
                evidence.captures
            ),
            (0, 0, 2)
        );
        assert!(evidence.wire.events.is_empty());
        assert_eq!(evidence.wire.unused_frames, 0);
    }
}


/// Independently solve and replay a retained failure deal using its recorded exact method.
fn final_fixture_plan(plan: &mut Prepared, name: &str, action_count: usize) {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pyramid-recon/native-v17");
    let bytes = fs::read(root.join(name)).unwrap();
    let deal = xsar::pyramid_json::read_from(bytes.as_slice()).unwrap();
    let options = SearchOptions {
        method: SearchMethod::AStar,
        heuristic_weight: std::num::NonZeroU32::new(1).unwrap(),
    };
    let outcome = search::solve_with_options(
        &deal,
        pyramid_search_policy::limits(),
        &AtomicBool::new(false),
        options,
        None,
    );
    let xsar::search::SearchOutcome::Solved { actions, .. } = outcome else {
        panic!("retained native final deal must solve");
    };
    assert_eq!(
        actions.len(),
        action_count,
        "same deterministic Fewest actions route as the saved116 run"
    );
    plan.trace = replay::replay(&deal, &actions, true).unwrap();
    plan.acquisition.bytes = bytes;
    plan.acquisition.deal = deal;
    plan.acquisition.vision = ReconVision::bundled_native_v17().unwrap();
}


/// Actual capture adapter reserves Perfect proof for final phase and retains fatal series guard.
///
/// Header/footer overlays and repeated fixture frames are explicitly synthetic. The
/// exact deals are independently solved/replayed; no guest receipt, ACK or normal
/// ordinal-two/three proof is created. The driver separately tests whole-action ACKs.
#[test]
fn native_perfect_final_clear_retains_phase_exact_piles_and_series_guard() {
    for (image, deal, actions, right) in [
        (
            "final-clear-right-card.png",
            "right-card-deal.json",
            47,
            Some(xsar::games::cards::Card::new(
                xsar::games::cards::Rank::Five,
                xsar::games::cards::Suit::Hearts,
            )),
        ),
        (
            "final-clear-empty-stock.png",
            "empty-stock-deal.json",
            46,
            None,
        ),
    ] {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pyramid-recon/native-v17");
        let mut frame = decode_png(&fs::read(root.join(image)).unwrap()).unwrap();
        patch(&mut frame, HEADER, NORMAL_HEADER);
        patch(&mut frame, FOOTER, NORMAL_FOOTER);
        let evidence = adapter_with_setup(
            vec![frame.clone(), frame.clone(), drift(frame, HEADER)],
            Some(&require_normal_one),
            |plan| final_fixture_plan(plan, deal, actions),
            |io| {
                let state = goal(io.plan);
                let (tableau, actual_left, actual_right) =
                    expected(&io.plan.acquisition.deal, state).unwrap();
                assert_eq!(tableau, [None; 28]);
                assert_eq!((actual_left, actual_right), (None, right));
                assert!(
                    !io.observe(state, false).unwrap(),
                    "ordinary phase retains original strict evidence"
                );
                assert_eq!(
                    io.observe_final(state).unwrap(),
                    Some(BoardCompletion::BoardClear)
                );
                assert!(
                    io.observe_final(state)
                        .unwrap_err()
                        .contains("frame guard rejected")
                );
            },
        );
        assert_eq!(evidence.captures, 3);
        assert_eq!(
            (evidence.counts.attempted, evidence.counts.acknowledged),
            (0, 0)
        );
        assert!(
            evidence.wire.events.is_empty(),
            "observation sends no input"
        );
    }
}


/// Independently solve the exact retained normal deal with its recorded unit-cost method.
fn normal_final_fixture_plan(plan: &mut Prepared, label: &str, action_count: usize) {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pyramid-recon/native-v18");
    let bytes = fs::read(root.join(format!("{label}-deal.json"))).unwrap();
    let deal = xsar::pyramid_json::read_from(bytes.as_slice()).unwrap();
    let outcome = search::solve_with_options(
        &deal,
        pyramid_search_policy::limits(),
        &AtomicBool::new(false),
        SearchOptions {
            method: SearchMethod::AStar,
            heuristic_weight: std::num::NonZeroU32::new(1).unwrap(),
        },
        None,
    );
    let xsar::search::SearchOutcome::Solved { actions, .. } = outcome else {
        panic!("retained normal final deal must solve");
    };
    assert_eq!(
        actions.len(),
        action_count,
        "recorded exact logical-action route"
    );
    plan.trace = replay::replay(&deal, &actions, true).unwrap();
    plan.acquisition.bytes = bytes;
    plan.acquisition.deal = deal;
    plan.acquisition.vision = ReconVision::bundled_native_v18().unwrap();
}


/// Actual NativeIo classifies all three original normal finals only at the final seam.
///
/// Repeated frames and a changed normal header are synthetic transport inputs. The
/// three deals are independently solved/replayed; no live receipt, ACK or restart
/// evidence is claimed. Production driver tests separately require whole-action ACKs.
#[test]
fn native_normal_final_clear_retains_exact_replay_final_phase_and_fatal_series_guard() {
    for (ordinal, label, actions, left, right) in [
        (
            0,
            "normal-one",
            53,
            Some(xsar::games::cards::Card::new(
                xsar::games::cards::Rank::Three,
                xsar::games::cards::Suit::Hearts,
            )),
            Some(xsar::games::cards::Card::new(
                xsar::games::cards::Rank::Nine,
                xsar::games::cards::Suit::Clubs,
            )),
        ),
        (1, "normal-two", 47, None, None),
        (2, "normal-three", 44, None, None),
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon/native-v18")
            .join(format!("{label}-perfect.png"));
        let frame = decode_png(&fs::read(path).unwrap()).unwrap();
        let mut changed = frame.clone();
        let header = if ordinal == 0 {
            include_bytes!("../../../tests/fixtures/pyramid-standard-series/header-2.rgb")
                .as_slice()
        } else {
            NORMAL_HEADER
        };
        patch(&mut changed, HEADER, header);
        let guard = |observed: &CapturedFrame, _phase| {
            if observe_normal(observed)?
                == Some(BoardCount {
                    completed: ordinal,
                    target: 3,
                })
            {
                Ok(FrameGuardOutcome::Ready)
            } else {
                Err("normal series changed after acquisition".to_owned())
            }
        };
        let evidence = adapter_with_setup(
            vec![frame.clone(), frame, changed],
            Some(&guard),
            |plan| normal_final_fixture_plan(plan, label, actions),
            |io| {
                let state = goal(io.plan);
                let (tableau, actual_left, actual_right) =
                    expected(&io.plan.acquisition.deal, state).unwrap();
                assert_eq!(tableau, [None; 28]);
                assert_eq!((actual_left, actual_right), (left, right));
                assert!(
                    !io.observe(state, false).unwrap(),
                    "ordinary phase remains strict"
                );
                assert_eq!(
                    io.observe_final(state).unwrap(),
                    Some(BoardCompletion::BoardClear)
                );
                assert!(
                    io.observe_final(state)
                        .unwrap_err()
                        .contains("frame guard rejected")
                );
            },
        );
        assert_eq!(evidence.captures, 3);
        assert_eq!(
            (evidence.counts.attempted, evidence.counts.acknowledged),
            (0, 0)
        );
        assert!(evidence.wire.events.is_empty());
        assert_eq!(evidence.wire.unused_frames, 0);
    }
}
