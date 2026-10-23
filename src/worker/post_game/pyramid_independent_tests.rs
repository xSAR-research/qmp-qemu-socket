//! Native evidence and concrete QMP regressions for independent Pyramid restart.

use std::{
    collections::VecDeque,
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::{Arc, Mutex, atomic::AtomicUsize, mpsc},
    thread,
};

use serde_json::{Value, json};

use super::*;
use crate::{
    capture::{PixelFormat, decode_png, encode_png},
    game::GameMode,
    geometry::PixelRect,
    strategy::{ControllerContext, SolvingStrategy},
    worker::{LatestFrameSlot, TestCaptureBackend, analyse_captured_frame},
};


/// Retained Pyramid terminal pixels; synthetic controls below establish ordering only.
const LEVEL_UP: &[u8] =
    include_bytes!("../../../tests/fixtures/ok-confirmation-20261016/pyramid/clean.png");


/// Retained rank-complete Pyramid start whose KS needs the production v11 correction.
const FULL_START: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-recon/native-v9/baseline.png");


/// Distinct retained full start proves that consecutive board identity matters.
const OTHER_START: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-recon/native-v7/baseline.png");


/// Unique local QMP sockets remain independent when the full suite runs concurrently.
static NEXT_SOCKET: AtomicUsize = AtomicUsize::new(0);


/// Decode one retained native PNG with the actual shared production codec.
fn native(png: &[u8]) -> CapturedFrame {
    decode_png(png).expect("retained native PNG")
}


/// Create an unsupported native-sized RGBA frame for refusal and synthetic menu tests.
fn blank() -> CapturedFrame {
    CapturedFrame {
        width: 1_920,
        height: 1_080,
        stride: 1_920 * 4,
        format: PixelFormat::Rgba8,
        pixels: vec![255; 1_920 * 1_080 * 4],
    }
}


/// Replace only the supplied fixed patch, preserving every other native pixel.
fn paint(frame: &mut CapturedFrame, bounds: PixelRect, rgb: [u8; 3]) {
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            let offset = y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&rgb);
        }
    }
}


/// Bounded black caption ink on a deliberately non-gameplay frame tests menu ordering.
fn menu(control: Control) -> CapturedFrame {
    let mut frame = blank();
    let bounds = profiles::PYRAMID_CONTROLS.spec(control).bounds;
    paint(
        &mut frame,
        PixelRect::new(bounds.x + 2, bounds.y + 2, 5, 1),
        [0; 3],
    );
    frame
}


/// Native Level Up requires complete independent context; generic decoration is insufficient.
#[test]
fn pyramid_terminal_requires_title_underlay_panels_chart_and_ok() {
    let original = native(LEVEL_UP);
    assert_eq!(recognised_pyramid_terminal(&original), Ok(true));
    for bounds in [
        PixelRect::new(800, 234, 320, 60),
        PixelRect::new(798, 854, 332, 20),
        PixelRect::new(507, 880, 3, 20),
        PixelRect::new(460, 400, 3, 40),
        PixelRect::new(1_816, 169, 68, 2),
        profiles::PYRAMID_CONTROLS.ok.bounds,
    ] {
        let mut erased = original.clone();
        // White removes the exact-black OK ink; black would create a caption run.
        let colour = if bounds == profiles::PYRAMID_CONTROLS.ok.bounds {
            [255; 3]
        } else {
            [0; 3]
        };
        paint(&mut erased, bounds, colour);
        assert_eq!(
            recognised_pyramid_terminal(&erased),
            Ok(false),
            "erased {bounds:?}"
        );
    }
    assert_eq!(recognised_pyramid_terminal(&menu(Control::Ok)), Ok(false));
    assert_eq!(recognised_pyramid_terminal(&native(FULL_START)), Ok(false));
    assert_eq!(
        recognised_pyramid_terminal(&native(include_bytes!(
            "../../../tests/fixtures/tripeaks-TP02.png"
        ))),
        Ok(false)
    );
    let mut malformed = original;
    malformed.pixels.truncate(10);
    assert!(recognised_pyramid_terminal(&malformed).is_err());
}


/// Changing the Solver region, level, rank and artwork cannot grant or revoke terminal authority.
#[test]
fn pyramid_terminal_excludes_solver_progress_and_dynamic_level_artwork() {
    let mut frame = native(LEVEL_UP);
    paint(&mut frame, PixelRect::new(820, 35, 280, 60), [255; 3]);
    paint(&mut frame, PixelRect::new(680, 390, 560, 300), [0; 3]);
    assert_eq!(recognised_pyramid_terminal(&frame), Ok(true));
}


/// Production full-start recognition does not replace unknown glyphs with inferred cards.
#[test]
fn independent_start_requires_all_28_identities_initial_dl_and_positive_empty_dr() {
    let vision = ReconVision::bundled_native_v11().expect("production v11");
    let original = native(FULL_START);
    let board = independent_pyramid_start(&vision, &original)
        .unwrap()
        .expect("native full start");
    assert!(matches!(board.left, StockFace::Card(_)));
    assert_eq!(board.right, StockFace::Empty);
    assert_eq!(independent_pyramid_start(&vision, &blank()).unwrap(), None);
    let mut unknown_rank = original.clone();
    paint(
        &mut unknown_rank,
        PixelRect::new(892, 224, 27, 24),
        [255; 3],
    );
    assert!(independent_pyramid_start(&vision, &unknown_rank).is_err());
    let mut unknown_dr = original;
    paint(
        &mut unknown_dr,
        PixelRect::new(1_022, 678, 139, 187),
        [255; 3],
    );
    assert!(
        independent_pyramid_start(&vision, &unknown_dr).is_err()
            || independent_pyramid_start(&vision, &unknown_dr)
                .unwrap()
                .is_none()
    );
}


/// A test episode records actual wire commands rather than expected application branches.
#[derive(Debug, Default)]
struct WireReport {
    /// Every requested screenshot, in wire order.
    captures: usize,
    /// Fresh absolute-pointer probes completed before controls.
    probes: usize,
    /// Native-mapped positions of actual left-button presses.
    downs: Vec<(i64, i64)>,
}


/// Remove only this episode's own socket and otherwise empty private directory.
struct SocketDirectory(PathBuf);


impl Drop for SocketDirectory {
    /// Leave any unexpected artefact intact for diagnosis rather than deleting recursively.
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0.join("qmp.sock"));
        let _ = fs::remove_dir(&self.0);
    }
}


/// Flush one acknowledged local QMP packet with the original numeric identifier.
fn packet(stream: &mut UnixStream, value: Value) {
    serde_json::to_writer(&mut *stream, &value).expect("local QMP packet");
    stream.write_all(b"\n").expect("local QMP newline");
    stream.flush().expect("local QMP flush");
}


/// Serve the exact bounded capture script and record all separately delivered input events.
fn serve(
    listener: UnixListener,
    mut captures: VecDeque<Vec<u8>>,
    cancel: Arc<AtomicBool>,
    stop_probe: Option<usize>,
) -> WireReport {
    let (mut stream, _) = listener.accept().expect("local QMP accept");
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .expect("bounded local QMP read");
    let mut reader = BufReader::new(stream.try_clone().expect("local QMP reader"));
    packet(
        &mut stream,
        json!({"QMP": {"version": {"qemu": {"major": 11, "minor": 0, "micro": 0}}, "capabilities": []}}),
    );
    let mut report = WireReport::default();
    let mut position = (0_i64, 0_i64);
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).expect("production QMP command") == 0 {
            break;
        }
        let request: Value = serde_json::from_str(&line).expect("production QMP JSON");
        let id = request["id"].as_u64().expect("production numeric QMP id");
        let response = match request["execute"].as_str().expect("production execute") {
            "qmp_capabilities" => json!({}),
            "query-status" => json!({"running": true, "status": "running"}),
            "query-mice" => {
                report.probes += 1;
                if stop_probe == Some(report.probes) {
                    cancel.store(true, Ordering::Release);
                }
                json!([{"name": "QEMU HID Tablet", "index": 0, "current": true, "absolute": true}])
            }
            "screendump" => {
                assert_eq!(request["arguments"]["format"], "png");
                let png = captures
                    .pop_front()
                    .expect("no extra capture beyond this episode's evidence");
                fs::write(
                    request["arguments"]["filename"]
                        .as_str()
                        .expect("private capture filename"),
                    png,
                )
                .expect("local capture pixels");
                report.captures += 1;
                json!({})
            }
            "input-send-event" => {
                for event in request["arguments"]["events"]
                    .as_array()
                    .expect("typed input events")
                {
                    match event["type"].as_str().expect("typed event") {
                        "abs" => match event["data"]["axis"].as_str().expect("absolute axis") {
                            "x" => position.0 = event["data"]["value"].as_i64().expect("mapped x"),
                            "y" => position.1 = event["data"]["value"].as_i64().expect("mapped y"),
                            other => panic!("unexpected absolute axis {other}"),
                        },
                        "btn" => {
                            assert_eq!(event["data"]["button"], "left");
                            if event["data"]["down"] == true {
                                report.downs.push(position);
                            }
                        }
                        other => panic!("unexpected restart event {other}"),
                    }
                }
                json!({})
            }
            other => panic!("unexpected QMP operation {other}"),
        };
        packet(&mut stream, json!({"return": response, "id": id}));
    }
    report
}


/// Actual production outcome and wire evidence, kept separate from host assertions.
struct Episode {
    /// True for restarted readiness; false for recovery or completion without restart.
    result: Result<bool, String>,
    /// Every concrete local QMP capture, probe and button press.
    wire: WireReport,
    /// Counter after the production controller returns or refuses readiness.
    completed_boards: usize,
    /// Latest original captured pixels remain available even when restart fails.
    latest_frame: Option<CapturedFrame>,
}


/// Exercise either public restart entry with concrete local QMP and original PNG decoding.
fn run(
    initial: CapturedFrame,
    frames: Vec<CapturedFrame>,
    independent: bool,
    cancel_before: bool,
    stop_probe: Option<usize>,
) -> Episode {
    run_with_entry(
        initial,
        frames,
        if independent {
            TestRestartEntry::Independent
        } else {
            TestRestartEntry::ComputerVision
        },
        GameMode::Pyramid,
        cancel_before,
        stop_probe,
    )
}


/// Production entry selected for a concrete transport episode, never a test driver replica.
#[derive(Clone, Copy)]
enum TestRestartEntry {
    /// Finish the shared reward flow under a genuine challenge completion receipt.
    Challenge(ChallengeCompletion),
    /// Preserve existing Solver/HALO behaviour.
    ComputerVision,
    /// Preserve the existing specific-terminal/full-board admission rule.
    Independent,
    /// Exercise the new actual-third-completion admission into the same controller.
    Witnessed(PyramidRestartEntry),
}


/// Run one actual controller entry with isolated capture transport and selected game.
fn run_with_entry(
    initial: CapturedFrame,
    frames: Vec<CapturedFrame>,
    entry: TestRestartEntry,
    mode: GameMode,
    cancel_before: bool,
    stop_probe: Option<usize>,
) -> Episode {
    let _capture_backend = TestCaptureBackend::legacy_qmp();
    let directory = SocketDirectory(std::env::temp_dir().join(format!(
        "pyramid-independent-post-game-{}-{}",
        std::process::id(),
        NEXT_SOCKET.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&directory.0).expect("unique independent restart directory");
    let socket = directory.0.join("qmp.sock");
    let listener = UnixListener::bind(&socket).expect("local independent restart socket");
    let frames = frames
        .iter()
        .map(|frame| encode_png(frame).expect("actual capture PNG"))
        .collect();
    let cancel = Arc::new(AtomicBool::new(cancel_before));
    let server_cancel = Arc::clone(&cancel);
    let server = thread::spawn(move || serve(listener, frames, server_cancel, stop_probe));
    let (events, _receiver) = mpsc::channel();
    let sink = WorkerEventSink {
        events,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(ControllerContext::new(
            socket.clone(),
            mode,
            if matches!(entry, TestRestartEntry::ComputerVision) {
                SolvingStrategy::ComputerVision
            } else {
                SolvingStrategy::ShortestPath
            },
            1,
        ))),
    };
    let mut scan = TableauScanState::for_mode(mode);
    let (initial, _) =
        analyse_captured_frame(initial, &scan).expect("initial native classification");
    let mut qmp = QmpClient::connect(&socket).expect("actual local QMP handshake");
    let mut completed_boards = GameMode::Pyramid.profile().boards_per_game;
    let vision = ReconVision::bundled_native_v11().expect("production v11");
    let mut latest_frame = Some(initial.frame.clone());
    let completion_context =
        ControllerContext::new(socket.clone(), mode, SolvingStrategy::ShortestPath, 1);
    if matches!(entry, TestRestartEntry::Challenge(_)) {
        sink.latest_frame
            .reconnaissance
            .select(&completion_context, &cancel)
            .unwrap();
        sink.latest_frame
            .reconnaissance
            .queue_reconnaissance(&completion_context, &cancel, || Ok(()))
            .unwrap();
        cancel.store(cancel_before, Ordering::Release);
    }
    let result = match entry {
        TestRestartEntry::Challenge(proof) => finish_pyramid_challenge(
            &mut qmp,
            &completion_context,
            &initial,
            proof,
            &mut scan,
            &sink,
            &cancel,
            &mut latest_frame,
        )
        .map(|()| PostGameOutcome::Completed),
        TestRestartEntry::Independent => run_independent_pyramid_restart_retaining_frame(
            &mut qmp,
            &socket,
            Some(&initial),
            &mut scan,
            &mut completed_boards,
            &sink,
            &cancel,
            &vision,
            &mut latest_frame,
        ),
        TestRestartEntry::ComputerVision => run_post_game_restart(
            &mut qmp,
            &socket,
            Some(&initial),
            &mut scan,
            &mut completed_boards,
            &sink,
            &cancel,
        ),
        TestRestartEntry::Witnessed(entry) => run_witnessed_pyramid_restart_retaining_frame(
            &mut qmp,
            &socket,
            &initial,
            entry,
            &mut scan,
            &mut completed_boards,
            &sink,
            &cancel,
            &vision,
            &mut latest_frame,
        ),
    }
    .map(|outcome| matches!(outcome, PostGameOutcome::Restarted(_)));
    drop(qmp);
    let wire = server.join().expect("local restart server completion");
    Episode {
        result,
        wire,
        completed_boards,
        latest_frame,
    }
}


/// Existing terminal controls remain exactly OK, New Game, Play before fresh start proof.
fn restart_frames() -> Vec<CapturedFrame> {
    vec![
        native(LEVEL_UP),
        native(LEVEL_UP),
        menu(Control::NewGame),
        menu(Control::Play),
    ]
}


/// Compare exact checked integer guest mappings; no coordinate rounding or clamping.
fn wire_point(point: PixelPoint) -> (i64, i64) {
    (
        i64::from(point.x) * 32_767 / 1_920,
        i64::from(point.y) * 32_767 / 1_080,
    )
}



/// Compose an explicit transport-stage simulation from real Fours and Continue pixels.
///
/// This is not a native Fours completion appearance or a new calibration reference.
fn target_completion_fours_continue() -> CapturedFrame {
    let (mut frame, _) = challenge_reward();
    let rgb = include_bytes!("../../../tests/fixtures/pyramid-objectives/complete-continue.rgb");
    for y in 0..78 {
        for x in 0..312 {
            let source = (y * 312 + x) * 3;
            let target = (842 + y) * frame.stride + (804 + x) * 4;
            frame.pixels[target..target + 3].copy_from_slice(&rgb[source..source + 3]);
        }
    }
    frame
}


/// Already-visible genuine Continue receives no click, capture, probe or other reward input.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn target_completion_star_club_controller_releases_before_operator_continue() {
    let frame = native(include_bytes!(
        "../../../tests/fixtures/pyramid-objectives/boards-complete-continue.png"
    ));
    let proof = crate::pyramid_progress::observe_challenge_complete(&frame)
        .unwrap()
        .unwrap();
    let expected = frame.pixels.clone();
    let episode = run_with_entry(
        frame,
        vec![],
        TestRestartEntry::Challenge(proof),
        GameMode::Pyramid,
        false,
        None,
    );
    assert_eq!(episode.result, Ok(false));
    assert!(episode.wire.downs.is_empty());
    assert_eq!(episode.wire.probes, 0);
    assert_eq!(episode.wire.captures, 0);
    assert_eq!(episode.latest_frame.unwrap().pixels, expected);
}


/// Existing XP skip and optional OK precede a later Continue; the operator button is never clicked.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn target_completion_continue_after_counting_preserves_shared_optional_level_up() {
    for level_up in [false, true] {
        let (counting, proof) = challenge_reward();
        let complete = target_completion_fours_continue();
        let mut frames = vec![counting.clone(), counting.clone()];
        if level_up {
            frames.extend([native(LEVEL_UP), native(LEVEL_UP)]);
        }
        frames.push(complete.clone());
        let episode = run_with_entry(
            counting,
            frames,
            TestRestartEntry::Challenge(proof),
            GameMode::Pyramid,
            false,
            None,
        );
        assert_eq!(episode.result, Ok(false), "optional Level Up = {level_up}");
        assert_eq!(episode.wire.downs.len(), if level_up { 2 } else { 1 });
        assert_eq!(
            episode.wire.downs[0],
            wire_point(SCORE_SKIP_CONTROL.click_point)
        );
        assert!(
            !episode
                .wire
                .downs
                .contains(&wire_point(PixelPoint::new(960, 881)))
        );
        assert_eq!(episode.latest_frame.unwrap().pixels, complete.pixels);
    }
}


/// Unknown control pixels are not an invitation to click the challenge or retry a reward.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn target_completion_unknown_continue_withholds_score_skip() {
    let mut frame = native(include_bytes!(
        "../../../tests/fixtures/pyramid-objectives/boards-complete-continue.png"
    ));
    let offset = 842 * frame.stride + 804 * 4;
    frame.pixels[offset] ^= 128;
    let proof = crate::pyramid_progress::observe_challenge_complete(&frame)
        .unwrap()
        .unwrap();
    let episode = run_with_entry(
        frame.clone(),
        (0..POST_GAME_MAX_OBSERVATION_ROUNDS)
            .map(|_| frame.clone())
            .collect(),
        TestRestartEntry::Challenge(proof),
        GameMode::Pyramid,
        false,
        None,
    );
    assert!(episode.result.is_err());
    assert!(episode.wire.downs.is_empty());
}

/// Retained actual terminal pixels supply proof; repeated frames test timing, not live evidence.
fn challenge_reward() -> (CapturedFrame, ChallengeCompletion) {
    let frame = native(include_bytes!(
        "../../../tests/fixtures/pyramid-objectives/fours-complete.png"
    ));
    let proof = crate::pyramid_progress::observe_challenge_complete(&frame)
        .unwrap()
        .unwrap();
    (frame, proof)
}


/// The real common event uses shared XP skip/OK controls and stops at a fresh NewGame match.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn challenge_rewards_reuse_skip_and_ok_without_new_game_play_or_solver() {
    let (frame, proof) = challenge_reward();
    let episode = run_with_entry(
        frame.clone(),
        vec![
            frame.clone(),
            frame,
            native(LEVEL_UP),
            native(LEVEL_UP),
            menu(Control::NewGame),
        ],
        TestRestartEntry::Challenge(proof),
        GameMode::Pyramid,
        false,
        None,
    );
    assert_eq!(episode.result, Ok(false));
    assert_eq!(episode.wire.downs.len(), 2);
    assert_eq!(
        episode.wire.downs[0],
        wire_point(SCORE_SKIP_CONTROL.click_point)
    );
    assert_eq!(episode.wire.probes, 2);
    assert_eq!(episode.wire.captures, 5);
    assert_eq!(
        episode.latest_frame.unwrap().pixels,
        menu(Control::NewGame).pixels
    );
    assert_eq!(
        episode.completed_boards,
        GameMode::Pyramid.profile().boards_per_game
    );
}


/// Optional Level Up is skipped only by observed NewGame, which itself receives no input.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn challenge_rewards_without_level_up_release_after_only_score_skip() {
    let (frame, proof) = challenge_reward();
    let episode = run_with_entry(
        frame.clone(),
        vec![
            frame.clone(),
            frame,
            menu(Control::NewGame),
            menu(Control::NewGame),
            menu(Control::NewGame),
        ],
        TestRestartEntry::Challenge(proof),
        GameMode::Pyramid,
        false,
        None,
    );
    assert_eq!(episode.result, Ok(false));
    assert_eq!(
        episode.wire.downs,
        vec![wire_point(SCORE_SKIP_CONTROL.click_point)]
    );
    assert_eq!(episode.wire.probes, 1);
    assert_eq!(
        episode.latest_frame.unwrap().pixels,
        menu(Control::NewGame).pixels
    );
}


/// STOP before reward entry or after a fresh OK probe prevents every later control.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn challenge_rewards_stop_precedes_receipt_and_fresh_ok_input() {
    let (frame, proof) = challenge_reward();
    let stopped = run_with_entry(
        frame.clone(),
        vec![],
        TestRestartEntry::Challenge(proof),
        GameMode::Pyramid,
        true,
        None,
    );
    assert!(stopped.result.unwrap_err().contains("STOP"));
    assert_eq!(stopped.wire.captures, 0);
    assert!(stopped.wire.downs.is_empty());
    let stopped = run_with_entry(
        frame.clone(),
        vec![frame.clone(), frame, native(LEVEL_UP), native(LEVEL_UP)],
        TestRestartEntry::Challenge(proof),
        GameMode::Pyramid,
        false,
        Some(2),
    );
    assert!(stopped.result.unwrap_err().contains("STOP"));
    assert_eq!(stopped.wire.probes, 2);
    assert_eq!(
        stopped.wire.downs,
        vec![wire_point(SCORE_SKIP_CONTROL.click_point)]
    );
}


/// A complete native full board after Play proves readiness without any Solver/HALO stage.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn independent_restart_stops_after_play_and_two_full_fresh_starts() {
    let mut frames = restart_frames();
    frames.extend([native(FULL_START), native(FULL_START)]);
    let episode = run(native(LEVEL_UP), frames, true, false, None);
    assert_eq!(episode.result, Ok(true));
    assert_eq!(episode.completed_boards, 0);
    assert_eq!(episode.wire.captures, 6);
    assert_eq!(episode.wire.probes, 3);
    assert_eq!(episode.wire.downs.len(), 3);
    assert!(!episode.wire.downs.contains(&wire_point(
        crate::parameters::SHARED_SOLVER_CONTROL.click_point
    )));
}


/// Early full gameplay recovery also requires a fresh matching confirmation without any input.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn independent_early_full_board_recovery_never_activates_solver() {
    let episode = run(
        native(FULL_START),
        vec![native(FULL_START)],
        true,
        false,
        None,
    );
    assert_eq!(episode.result, Ok(false));
    assert_eq!(episode.completed_boards, 0);
    assert_eq!(episode.wire.captures, 1);
    assert_eq!(episode.wire.probes, 0);
    assert!(episode.wire.downs.is_empty());
}


/// Two distinct correctly recognised deals cannot combine into one restart proof.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn independent_full_start_confirmation_rejects_changed_identity() {
    let episode = run(
        native(FULL_START),
        vec![native(OTHER_START)],
        true,
        false,
        None,
    );
    assert!(episode.result.unwrap_err().contains("arrangement changed"));
    assert_eq!(episode.wire.captures, 1);
    assert_eq!(episode.wire.probes, 0);
    assert!(episode.wire.downs.is_empty());
    assert_eq!(
        episode.completed_boards,
        GameMode::Pyramid.profile().boards_per_game
    );
}


/// Unknown initial evidence and a STOP present at entry both refuse before probes or captures.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn independent_unknown_entry_and_stop_before_controls_are_input_free() {
    let unknown = run(blank(), Vec::new(), true, false, None);
    assert!(unknown.result.unwrap_err().contains("no positive terminal"));
    assert_eq!(unknown.wire.captures, 0);
    assert_eq!(unknown.wire.probes, 0);
    assert!(unknown.wire.downs.is_empty());
    let stopped = run(native(LEVEL_UP), Vec::new(), true, true, None);
    assert!(stopped.result.unwrap_err().contains("STOP"));
    assert_eq!(stopped.wire.captures, 0);
    assert_eq!(stopped.wire.probes, 0);
    assert!(stopped.wire.downs.is_empty());
}


/// A STOP delivered during the final control's fresh probe prevents its button input.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn independent_stop_after_play_probe_prevents_play_and_solver() {
    let episode = run(native(LEVEL_UP), restart_frames(), true, false, Some(3));
    assert!(episode.result.unwrap_err().contains("STOP"));
    assert_eq!(episode.wire.probes, 3);
    assert_eq!(episode.wire.captures, 4);
    assert_eq!(episode.wire.downs.len(), 2);
}


/// Unknown post-Play pixels cannot fall through to recovery Solver or commit a reset counter.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn independent_unknown_after_play_refuses_without_solver_or_extra_capture() {
    let mut frames = restart_frames();
    frames.push(blank());
    let episode = run(native(LEVEL_UP), frames, true, false, None);
    assert!(
        episode
            .result
            .unwrap_err()
            .contains("fully recognised 28-card")
    );
    assert_eq!(episode.wire.captures, 5);
    let latest = episode
        .latest_frame
        .expect("failed post-Play observation retained");
    assert_eq!(latest.pixels, blank().pixels);
    assert_ne!(latest.pixels, native(LEVEL_UP).pixels);
    assert_eq!(episode.wire.downs.len(), 3);
    assert_eq!(
        episode.completed_boards,
        GameMode::Pyramid.profile().boards_per_game
    );
}


/// Existing Computer Vision still clicks Solver and requires its actionable post-Solver HALO.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn default_restart_preserves_solver_and_halo_behaviour() {
    let mut halo = native(FULL_START);
    paint(
        &mut halo,
        crate::pyramid::halo_probe(crate::pyramid::PYRAMID_TARGETS[0]),
        [237, 207, 109],
    );
    let (observation, _) =
        analyse_captured_frame(halo.clone(), &TableauScanState::for_mode(GameMode::Pyramid))
            .expect("existing Pyramid HALO classifier");
    assert!(matches!(observation.prediction, PredictedAction::Action(_)));
    let mut frames = restart_frames();
    frames.extend([native(FULL_START), halo]);
    let episode = run(native(LEVEL_UP), frames, false, false, None);
    assert_eq!(episode.result, Ok(true));
    assert_eq!(episode.wire.downs.len(), 4);
    assert_eq!(
        episode.wire.downs.last(),
        Some(&wire_point(
            crate::parameters::SHARED_SOLVER_CONTROL.click_point
        ))
    );
    assert_eq!(episode.completed_boards, 0);
}


/// A decoded unsupported-size frame survives the original shared-analysis error after Play.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn independent_analysis_failure_retains_decoded_pixels_and_original_error() {
    let unsupported = CapturedFrame {
        width: 12,
        height: 13,
        stride: 12 * 4,
        format: PixelFormat::Rgba8,
        pixels: vec![77; 12 * 13 * 4],
    };
    let expected_error = analyse_captured_frame(
        unsupported.clone(),
        &TableauScanState::for_mode(GameMode::Pyramid),
    )
    .err()
    .expect("unsupported dimensions must fail shared analysis");
    let mut frames = restart_frames();
    frames.push(unsupported.clone());
    let episode = run(native(LEVEL_UP), frames, true, false, None);
    assert_eq!(episode.result, Err(expected_error));
    let latest = episode
        .latest_frame
        .expect("decoded failure evidence retained");
    assert_eq!((latest.width, latest.height, latest.stride), (12, 13, 48));
    assert_eq!(latest.pixels, unsupported.pixels);
    assert_eq!(episode.wire.captures, 5);
    assert_eq!(episode.wire.downs.len(), 3);
    assert_eq!(
        episode.completed_boards,
        GameMode::Pyramid.profile().boards_per_game
    );
}


/// A controlled final-board outcome exercises admission, not native completion calibration.
fn third_clear_entry() -> PyramidRestartEntry {
    PyramidRestartEntry::verified_third_completion(
        crate::pyramid_progress::BoardCount {
            completed: 2,
            target: 3,
        },
        crate::worker::computed_route::BoardCompletion::BoardClear,
    )
    .expect("controlled actual-third-clear caller witness")
}


/// Keep native scene and stock pixels while positively clearing every tableau probe.
///
/// This deliberate derivative tests the gameplay veto; it is not a retained win.
fn empty_gameplay_with_new_game_ink() -> CapturedFrame {
    let mut frame = native(FULL_START);
    for slot in crate::pyramid::PYRAMID_TARGETS {
        if slot.kind.is_card() {
            paint(&mut frame, slot.bounds, [30, 95, 65]);
        }
    }
    let bounds = profiles::PYRAMID_CONTROLS.new_game.bounds;
    paint(
        &mut frame,
        PixelRect::new(bounds.x + 2, bounds.y + 2, 5, 1),
        [0; 3],
    );
    frame
}


/// Delivery-only and next-board evidence cannot construct a restart permit at any count.
#[test]
fn witnessed_restart_requires_actual_last_board_completion() {
    use crate::worker::computed_route::BoardCompletion;

    for completed in [0, 1, 2, 3, u8::MAX] {
        for completion in [
            BoardCompletion::DeliveredUnconfirmed,
            BoardCompletion::NextBoardReady,
        ] {
            assert!(
                PyramidRestartEntry::verified_third_completion(
                    crate::pyramid_progress::BoardCount {
                        completed,
                        target: 3,
                    },
                    completion,
                )
                .is_err()
            );
        }
    }
    for completion in [BoardCompletion::BoardClear, BoardCompletion::GameComplete] {
        for progress in [
            crate::pyramid_progress::BoardCount {
                completed: 0,
                target: 3,
            },
            crate::pyramid_progress::BoardCount {
                completed: 1,
                target: 3,
            },
            crate::pyramid_progress::BoardCount {
                completed: 3,
                target: 3,
            },
            crate::pyramid_progress::BoardCount {
                completed: 1,
                target: 2,
            },
            crate::pyramid_progress::BoardCount {
                completed: 3,
                target: 4,
            },
            crate::pyramid_progress::BoardCount {
                completed: u8::MAX,
                target: 0,
            },
        ] {
            assert!(PyramidRestartEntry::verified_third_completion(progress, completion).is_err());
        }
        assert!(
            PyramidRestartEntry::verified_third_completion(
                crate::pyramid_progress::BoardCount {
                    completed: 2,
                    target: 3
                },
                completion,
            )
            .is_ok()
        );
    }
}


/// A black New Game-region run on full or empty gameplay is only overlapping stock ink.
#[test]
fn expected_new_game_vetoes_full_and_empty_gameplay_without_cloning() {
    let empty = empty_gameplay_with_new_game_ink();
    assert_eq!(crate::pyramid::board_is_empty(&empty), Ok(true));
    let mut full = native(FULL_START);
    let bounds = profiles::PYRAMID_CONTROLS.new_game.bounds;
    paint(
        &mut full,
        PixelRect::new(bounds.x + 2, bounds.y + 2, 5, 1),
        [0; 3],
    );
    for frame in [&full, &empty] {
        assert_eq!(crate::pyramid::is_gameplay_scene(frame), Ok(true));
        assert_ne!(
            control_point(frame, profiles::PYRAMID_CONTROLS.new_game).unwrap(),
            PixelPoint::new(0, 0),
        );
        assert_eq!(expected_pyramid_new_game(frame), Ok(false));
        let (observation, _) = analyse_captured_frame(
            frame.clone(),
            &TableauScanState::for_mode(GameMode::Pyramid),
        )
        .unwrap();
        assert_eq!(
            known_post_game_stage(Some(&observation), &profiles::PYRAMID_CONTROLS),
            Ok(None)
        );
    }
}


/// The borrowed predicate shares the existing stage policy without promoting menus to rewards.
#[test]
fn expected_new_game_is_only_current_bounded_stage_evidence() {
    let new_game = menu(Control::NewGame);
    assert_eq!(expected_pyramid_new_game(&new_game), Ok(true));
    let (observation, _) = analyse_captured_frame(
        new_game.clone(),
        &TableauScanState::for_mode(GameMode::Pyramid),
    )
    .unwrap();
    assert_eq!(
        known_post_game_stage(Some(&observation), &profiles::PYRAMID_CONTROLS),
        Ok(Some(1))
    );
    assert_eq!(recognised_pyramid_terminal(&new_game), Ok(false));
    for frame in [blank(), menu(Control::Play), menu(Control::Ok)] {
        assert_eq!(expected_pyramid_new_game(&frame), Ok(false));
        assert_eq!(recognised_pyramid_terminal(&frame), Ok(false));
    }
    let mut invalid = new_game;
    invalid.pixels.truncate(4);
    assert!(expected_pyramid_new_game(&invalid).is_err());
}


/// Actual controller input is exactly New Game then Play, with no score skip, OK or Solver.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn witnessed_new_game_reuses_only_forward_stages_and_two_fresh_full_starts() {
    let mut play = menu(Control::Play);
    let bounds = profiles::PYRAMID_CONTROLS.new_game.bounds;
    paint(
        &mut play,
        PixelRect::new(bounds.x + 2, bounds.y + 2, 5, 1),
        [0; 3],
    );
    let episode = run_with_entry(
        menu(Control::NewGame),
        vec![
            menu(Control::NewGame),
            play,
            native(FULL_START),
            native(FULL_START),
        ],
        TestRestartEntry::Witnessed(third_clear_entry()),
        GameMode::Pyramid,
        false,
        None,
    );
    assert_eq!(episode.result, Ok(true));
    assert_eq!(episode.completed_boards, 0);
    assert_eq!(episode.wire.captures, 4);
    assert_eq!(episode.wire.probes, 2);
    let expected: Vec<_> = [Control::NewGame, Control::Play]
        .into_iter()
        .map(|control| {
            let bounds = profiles::PYRAMID_CONTROLS.spec(control).bounds;
            wire_point(PixelPoint::new(
                i32::try_from(bounds.x + 2).expect("native control X"),
                i32::try_from(bounds.y + 2).expect("native control Y"),
            ))
        })
        .collect();
    assert_eq!(episode.wire.downs, expected);
}


/// Specific paired terminal completion keeps optional OK and the existing independent stages.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn witnessed_specific_terminal_keeps_existing_ok_new_game_play_controller() {
    let entry = PyramidRestartEntry::verified_third_completion(
        crate::pyramid_progress::BoardCount {
            completed: 2,
            target: 3,
        },
        crate::worker::computed_route::BoardCompletion::GameComplete,
    )
    .expect("controlled paired specific terminal outcome");
    let mut frames = restart_frames();
    frames.extend([native(FULL_START), native(FULL_START)]);
    let episode = run_with_entry(
        native(LEVEL_UP),
        frames,
        TestRestartEntry::Witnessed(entry),
        GameMode::Pyramid,
        false,
        None,
    );
    assert_eq!(episode.result, Ok(true));
    assert_eq!(episode.wire.captures, 6);
    assert_eq!(episode.wire.probes, 3);
    assert_eq!(episode.wire.downs.len(), 3);
    assert_eq!(episode.completed_boards, 0);
    assert!(
        !episode
            .wire
            .downs
            .contains(&wire_point(SCORE_SKIP_CONTROL.click_point))
    );
    assert!(!episode.wire.downs.contains(&wire_point(
        crate::parameters::SHARED_SOLVER_CONTROL.click_point,
    )));
}


/// The old wrapper still refuses menu-only entry; the new permit never bypasses scene/STOP guards.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn witnessed_admission_preserves_old_wrapper_and_refuses_unsafe_current_scenes() {
    let old = run(menu(Control::NewGame), Vec::new(), true, false, None);
    assert!(old.result.unwrap_err().contains("no positive terminal"));
    assert_eq!(old.wire.captures, 0);
    assert_eq!(old.wire.probes, 0);
    assert!(old.wire.downs.is_empty());
    for (frame, mode, cancelled) in [
        (empty_gameplay_with_new_game_ink(), GameMode::Pyramid, false),
        (native(FULL_START), GameMode::Pyramid, false),
        (blank(), GameMode::Pyramid, false),
        (menu(Control::Play), GameMode::Pyramid, false),
        (menu(Control::NewGame), GameMode::TriPeaks, false),
        (menu(Control::NewGame), GameMode::Pyramid, true),
    ] {
        let episode = run_with_entry(
            frame,
            Vec::new(),
            TestRestartEntry::Witnessed(third_clear_entry()),
            mode,
            cancelled,
            None,
        );
        assert!(episode.result.is_err());
        assert_eq!(episode.wire.captures, 0);
        assert_eq!(episode.wire.probes, 0);
        assert!(episode.wire.downs.is_empty());
        assert_eq!(
            episode.completed_boards,
            GameMode::Pyramid.profile().boards_per_game
        );
        assert!(episode.latest_frame.is_some());
    }
}


/// STOP during the fresh Play probe revokes its click and all later restart work.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn witnessed_restart_checks_stop_after_fresh_play_probe() {
    let episode = run_with_entry(
        menu(Control::NewGame),
        vec![menu(Control::NewGame), menu(Control::Play)],
        TestRestartEntry::Witnessed(third_clear_entry()),
        GameMode::Pyramid,
        false,
        Some(2),
    );
    assert!(episode.result.unwrap_err().contains("STOP"));
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.probes, 2);
    assert_eq!(episode.wire.downs.len(), 1);
    assert_eq!(
        episode.completed_boards,
        GameMode::Pyramid.profile().boards_per_game
    );
}


/// Missing Play cannot replay New Game or obtain a score-skip click from menu-only pixels.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn witnessed_restart_missing_play_never_replays_or_skips_score() {
    let episode = run_with_entry(
        menu(Control::NewGame),
        vec![menu(Control::NewGame), blank(), blank(), blank()],
        TestRestartEntry::Witnessed(third_clear_entry()),
        GameMode::Pyramid,
        false,
        None,
    );
    assert!(
        episode
            .result
            .unwrap_err()
            .contains("Play did not become ready")
    );
    assert_eq!(episode.wire.captures, 4);
    assert_eq!(episode.wire.probes, 1);
    assert_eq!(episode.wire.downs.len(), 1);
    assert_eq!(
        episode.completed_boards,
        GameMode::Pyramid.profile().boards_per_game
    );
    assert_eq!(episode.latest_frame.unwrap().pixels, blank().pixels);
}
