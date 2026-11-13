//! Regression tests and native fixture verification for independent Free Cell post-win restart.

use std::{
    collections::VecDeque,
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Instant,
};

use serde_json::{Value, json};

use super::*;
use crate::{
    capture::decode_png,
    freecell_recon::recognise,
    game::GameMode,
    image_diagnostics::ImageDiagnostics,
    strategy::{ControllerContext, SolvingStrategy},
    worker::{
        FrameObservation, LatestFrameSlot, TestCaptureBackend, WORKER_IMAGE_DIAGNOSTICS,
        WorkerEventSink, WorkerImageDiagnostics, analyse_captured_frame,
    },
};

/// Game win score count-up fixture (`SCORE_SKIP`).
const FC09_SCORE: &[u8] = include_bytes!("../../../tests/fixtures/freecell-FC09.png");

/// Settled Level Up modal fixture with active bounded `OK`.
const FC10_LEVEL_UP: &[u8] = include_bytes!("../../../tests/fixtures/freecell-FC10.png");

/// Congratulations + `New Game` fixture (no Level Up modal).
const FC11_NEW_GAME: &[u8] = include_bytes!("../../../tests/fixtures/freecell-FC11.png");

/// New Game difficulty / `Play` modal fixture.
const FC12_PLAY: &[u8] = include_bytes!("../../../tests/fixtures/freecell-FC12.png");

/// Fresh untouched 52-card Free Cell normal deal fixture.
const FC13_START: &[u8] = include_bytes!("../../../tests/fixtures/freecell-FC13.png");

/// Alternate Congratulations + `New Game` fixture.
const FC15_NEW_GAME: &[u8] = include_bytes!("../../../tests/fixtures/freecell-FC15.png");

/// Challenge-header Free Cell deal fixture (must be refused after `Play` or cold start).
const FC_CHALLENGE: &[u8] =
    include_bytes!("../../../tests/fixtures/freecell-challenge-20261107.png");

/// Unique socket directory counter for concurrent unit test execution.
static NEXT_SOCKET: AtomicUsize = AtomicUsize::new(0);

/// Unique temporary image-recording directory counter.
static NEXT_REC: AtomicUsize = AtomicUsize::new(0);

/// Wire summary collected from the local QMP mock server.
#[derive(Debug, Default)]
struct WireReport {
    /// Number of screendumps served.
    captures: usize,
    /// Number of mouse probes executed prior to bounded clicks.
    probes: usize,
    /// Native HID click locations (`(x, y)` in `0..=32767`).
    downs: Vec<(i64, i64)>,
}

/// Self-cleaning temporary Unix domain socket directory.
struct SocketDirectory(PathBuf);

impl Drop for SocketDirectory {
    /// Remove the mock socket and its private directory cleanly.
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0.join("qmp.sock"));
        let _ = fs::remove_dir(&self.0);
    }
}

/// Self-cleaning temporary directory for `GAME WIN` sequence PNGs.
struct RecordingDirectory(PathBuf);

impl Drop for RecordingDirectory {
    /// Remove recorded PNG files left in this test's temporary directory.
    fn drop(&mut self) {
        if let Ok(entries) = fs::read_dir(&self.0) {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }
        let _ = fs::remove_dir(&self.0);
    }
}

/// Restore thread-local `WORKER_IMAGE_DIAGNOSTICS` state after a test finishes.
struct RecordingRestore(Option<WorkerImageDiagnostics>);

impl Drop for RecordingRestore {
    /// Swap back any previous thread-local recording state.
    fn drop(&mut self) {
        WORKER_IMAGE_DIAGNOSTICS.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}

/// Write one newline-delimited QMP JSON reply packet.
fn packet(stream: &mut UnixStream, value: Value) {
    serde_json::to_writer(&mut *stream, &value).expect("write QMP packet");
    stream.write_all(b"\n").expect("write QMP newline");
    stream.flush().expect("flush QMP stream");
}

/// Run a deterministic local QMP script serving raw PNG fixtures and recording ABS clicks.
fn serve(listener: UnixListener, mut captures: VecDeque<Vec<u8>>) -> WireReport {
    let (mut stream, _) = listener.accept().expect("accept QMP client");
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .expect("set QMP read timeout");
    let mut reader = BufReader::new(stream.try_clone().expect("clone QMP stream"));
    packet(
        &mut stream,
        json!({"QMP": {"version": {"qemu": {"major": 11, "minor": 0, "micro": 0}}, "capabilities": []}}),
    );
    let mut report = WireReport::default();
    let mut position = (0_i64, 0_i64);
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).expect("read QMP command") == 0 {
            break;
        }
        let request: Value = serde_json::from_str(&line).expect("parse QMP JSON");
        let id = request["id"].as_u64().expect("QMP request id");
        let response = match request["execute"].as_str().expect("QMP execute") {
            "qmp_capabilities" => json!({}),
            "query-status" => json!({"running": true, "status": "running"}),
            "query-mice" => {
                report.probes += 1;
                json!([{"name": "QEMU HID Tablet", "index": 0, "current": true, "absolute": true}])
            }
            "screendump" => {
                let png = captures.pop_front().expect("sufficient mock captures");
                fs::write(
                    request["arguments"]["filename"]
                        .as_str()
                        .expect("screendump path"),
                    png,
                )
                .expect("write mock PNG");
                report.captures += 1;
                json!({})
            }
            "input-send-event" => {
                for event in request["arguments"]["events"]
                    .as_array()
                    .expect("input events")
                {
                    if event["type"] == "abs" {
                        let axis = event["data"]["axis"].as_str().expect("abs axis");
                        let value = event["data"]["value"].as_i64().expect("abs value");
                        match axis {
                            "x" => position.0 = value,
                            "y" => position.1 = value,
                            other => panic!("unexpected axis {other}"),
                        }
                    } else if event["type"] == "btn"
                        && event["data"]["down"].as_bool() == Some(true)
                    {
                        report.downs.push(position);
                    }
                }
                json!({})
            }
            other => panic!("unexpected QMP command {other}"),
        };
        packet(&mut stream, json!({"id": id, "return": response}));
    }
    report
}

/// Analyse raw PNG bytes under Free Cell `ScanState`.
fn analyse_png(png: &[u8]) -> FrameObservation {
    let frame = decode_png(png).expect("decode Free Cell PNG");
    let scan_state = TableauScanState::for_mode(GameMode::FreeCell);
    analyse_captured_frame(frame, &scan_state)
        .expect("analyse Free Cell frame")
        .0
}

/// Convert a host native pixel into expected QEMU HID ABS `(x, y)` coordinates.
fn native_abs(point: PixelPoint) -> (i64, i64) {
    (
        (i64::from(point.x) * 32_767) / i64::from(NOMINAL_FRAME_WIDTH),
        (i64::from(point.y) * 32_767) / i64::from(NOMINAL_FRAME_HEIGHT),
    )
}

/// Check that no delivered click targets CV's legacy Solver region (`x <= 1050` with `y <= 150`).
fn assert_no_solver_click(downs: &[(i64, i64)]) {
    let solver_max_y = (150_i64 * 32_767) / i64::from(NOMINAL_FRAME_HEIGHT);
    assert!(
        downs.iter().all(|&(_, y)| y > solver_max_y),
        "Solver region click must never be emitted in independent Free Cell restart: {downs:?}"
    );
}

/// Execute one independent Free Cell restart episode under `TestCaptureBackend::legacy_qmp()`.
fn run_episode(
    initial: Option<FrameObservation>,
    previous_deal: Option<&InitialDeal>,
    captures: Vec<&[u8]>,
    record_images: bool,
) -> (
    Result<PostGameOutcome, String>,
    WireReport,
    Vec<PathBuf>,
    Vec<String>,
) {
    let suffix = NEXT_SOCKET.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("qmp-fc-postgame-{}-{suffix}", std::process::id()));
    fs::create_dir_all(&dir).expect("create socket temp dir");
    let sock_dir = SocketDirectory(dir.clone());
    let sock_path = sock_dir.0.join("qmp.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind mock QMP socket");

    let rec_dir = if record_images {
        let rec_idx = NEXT_REC.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("qmp-fc-win-rec-{}-{rec_idx}", std::process::id()));
        fs::create_dir_all(&path).expect("create recording temp dir");
        Some(RecordingDirectory(path))
    } else {
        None
    };

    let _backend = TestCaptureBackend::legacy_qmp();
    let _rec_restore = RecordingRestore(WORKER_IMAGE_DIAGNOSTICS.with(|slot| {
        slot.replace(Some(WorkerImageDiagnostics::new(ImageDiagnostics {
            directory: rec_dir.as_ref().map(|d| d.0.clone()),
            started_at: Instant::now(),
        })))
    }));

    let queue: VecDeque<Vec<u8>> = captures.into_iter().map(|b| b.to_vec()).collect();
    let server = thread::spawn(move || serve(listener, queue));

    let cancel = Arc::new(AtomicBool::new(false));
    let context = ControllerContext::new(
        sock_path.clone(),
        GameMode::FreeCell,
        SolvingStrategy::ShortestPath,
        3300 + suffix as u64,
    );
    let latest = LatestFrameSlot::default();
    latest
        .reconnaissance
        .select(&context, &cancel)
        .expect("select context");
    latest
        .reconnaissance
        .queue_reconnaissance(&context, &cancel, || Ok(()))
        .expect("reserve reconnaissance");

    let (tx, rx) = mpsc::channel();
    let sink = WorkerEventSink {
        events: tx,
        latest_frame: latest.clone(),
        capture_context: Mutex::new(Some(context.clone())),
    };
    let trace = super::super::pyramid_stop::StopTrace::default();
    if record_images {
        super::super::record_game_win_png(FC09_SCORE, Some(Duration::from_millis(100)), &sink)
            .expect("seed entry PNG");
    }
    let mut qmp = QmpClient::connect(&sock_path).expect("connect mock QMP");
    let mut scan_state = TableauScanState::for_mode(GameMode::FreeCell);
    let mut completed_boards = 0_usize;
    let mut latest_captured = None;
    let outcome = run_independent_freecell_restart_retaining_frame(FreeCellRestartContext {
        qmp: &mut qmp,
        socket_path: &sock_path,
        initial_observation: initial.as_ref(),
        previous_deal,
        scan_state: &mut scan_state,
        completed_boards: &mut completed_boards,
        event_tx: &sink,
        cancel_requested: &cancel,
        latest_frame: &mut latest_captured,
        authority: RestartAuthority {
            context: &context,
            trace: &trace,
        },
    });
    drop(qmp);
    drop(_rec_restore);
    let wire = server.join().expect("join QMP server");

    let mut logs = Vec::new();
    while let Ok(contextual) = rx.try_recv() {
        if let super::super::WorkerEvent::Log(line) = contextual.event {
            logs.push(line);
        }
    }

    let mut saved_pngs = Vec::new();
    if let Some(dir) = &rec_dir
        && let Ok(entries) = fs::read_dir(&dir.0)
    {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("png") {
                saved_pngs.push(entry.path());
            }
        }
    }

    (outcome, wire, saved_pngs, logs)
}

#[test]
#[ignore = "requires local AF_UNIX socket"]
fn freecell_330_restart_full_sequence_with_level_up_and_default_image_recording() {
    let captures = vec![
        FC09_SCORE,
        FC09_SCORE,
        FC10_LEVEL_UP,
        FC10_LEVEL_UP,
        FC11_NEW_GAME,
        FC12_PLAY,
        FC13_START,
        FC13_START,
    ];
    let (outcome, wire, saved_pngs, _logs) = run_episode(None, None, captures, true);
    assert!(
        matches!(outcome, Ok(PostGameOutcome::Restarted(_))),
        "unexpected error: {:?}",
        outcome.err()
    );
    assert_eq!(wire.captures, 8);
    assert_eq!(wire.downs.len(), 4);
    assert_eq!(wire.downs[0], native_abs(SCORE_SKIP_CONTROL.click_point));
    assert_no_solver_click(&wire.downs);
    assert!(
        !saved_pngs.is_empty(),
        "default GAME WIN image recording must capture PNG sequence frames"
    );
}

#[test]
#[ignore = "requires local AF_UNIX socket"]
fn freecell_330_restart_skips_absent_level_up_and_honours_no_image() {
    let captures = vec![
        FC09_SCORE,
        FC09_SCORE,
        FC11_NEW_GAME,
        FC11_NEW_GAME,
        FC12_PLAY,
        FC13_START,
        FC13_START,
    ];
    let (outcome, wire, saved_pngs, _logs) = run_episode(None, None, captures, false);
    assert!(
        matches!(outcome, Ok(PostGameOutcome::Restarted(_))),
        "unexpected error: {:?}",
        outcome.err()
    );
    assert_eq!(wire.captures, 7);
    assert_eq!(wire.downs.len(), 3);
    assert_eq!(wire.downs[0], native_abs(SCORE_SKIP_CONTROL.click_point));
    assert_no_solver_click(&wire.downs);
    assert!(
        saved_pngs.is_empty(),
        "--no-image disables GAME WIN sequence PNGs"
    );
}

#[test]
#[ignore = "requires local AF_UNIX socket"]
fn freecell_330_restart_cold_entries_from_level_up_new_game_and_play_skip_score_click() {
    // Direct Level Up entry (`FC10`)
    let (outcome, wire, _, _) = run_episode(
        Some(analyse_png(FC10_LEVEL_UP)),
        None,
        vec![
            FC10_LEVEL_UP,
            FC10_LEVEL_UP,
            FC11_NEW_GAME,
            FC12_PLAY,
            FC13_START,
            FC13_START,
        ],
        false,
    );
    assert!(
        matches!(outcome, Ok(PostGameOutcome::Restarted(_))),
        "unexpected error: {:?}",
        outcome.err()
    );
    assert_eq!(wire.downs.len(), 3);
    assert_no_solver_click(&wire.downs);

    // Direct New Game entry (`FC15`)
    let (outcome, wire, _, _) = run_episode(
        Some(analyse_png(FC15_NEW_GAME)),
        None,
        vec![FC15_NEW_GAME, FC12_PLAY, FC13_START, FC13_START],
        false,
    );
    assert!(
        matches!(outcome, Ok(PostGameOutcome::Restarted(_))),
        "unexpected error: {:?}",
        outcome.err()
    );
    assert_eq!(wire.downs.len(), 2);

    // Direct Play modal entry (`FC12`)
    let (outcome, wire, _, _) = run_episode(
        Some(analyse_png(FC12_PLAY)),
        None,
        vec![FC12_PLAY, FC13_START, FC13_START],
        false,
    );
    assert!(
        matches!(outcome, Ok(PostGameOutcome::Restarted(_))),
        "unexpected error: {:?}",
        outcome.err()
    );
    assert_eq!(wire.downs.len(), 1);
}

#[test]
#[ignore = "requires local AF_UNIX socket"]
fn freecell_330_restart_refuses_stale_previous_deal_and_challenge_deal() {
    let fc13_frame = decode_png(FC13_START).expect("decode FC13");
    let meta = xsar::games::freecell::Metadata::new("fc13", crate::freecell_recon::PROFILE, "test")
        .expect("meta");
    let fc13_deal = recognise(&fc13_frame, meta).expect("recognise FC13");

    // Stale unchanged board after Play across all 6 observations must refuse without further clicks.
    let (outcome, wire, _, _) = run_episode(
        Some(analyse_png(FC12_PLAY)),
        Some(&fc13_deal),
        vec![
            FC12_PLAY, FC13_START, FC13_START, FC13_START, FC13_START, FC13_START, FC13_START,
        ],
        false,
    );
    assert!(outcome.is_err());
    assert!(
        outcome
            .as_ref()
            .err()
            .is_some_and(|e| e.contains("previous completed deal")),
        "unexpected error: {:?}",
        outcome.err()
    );
    assert_eq!(wire.downs.len(), 1);

    // Challenge board appearing after Play must fail immediately.
    let (outcome, _, _, _) = run_episode(
        Some(analyse_png(FC12_PLAY)),
        Some(&fc13_deal),
        vec![FC12_PLAY, FC_CHALLENGE],
        false,
    );
    assert!(outcome.is_err());
    assert!(
        outcome
            .as_ref()
            .err()
            .is_some_and(|e| e.contains("challenge board")),
        "unexpected error: {:?}",
        outcome.err()
    );
}
