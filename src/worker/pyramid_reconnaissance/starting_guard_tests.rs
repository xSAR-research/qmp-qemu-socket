//! Same-session reconnaissance binding through production recognition and isolated QMP.
//!
//! Header predicates use retained challenge counters only as generic injected
//! constraints; they do not establish normal-series authority. Header derivatives
//! and seeded restoration ledger state are explicitly synthetic. No live guest,
//! GUI, process configuration or new capture source is used by these fixtures.

use std::{
    cell::Cell,
    collections::VecDeque,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::{atomic::AtomicUsize, mpsc},
    thread,
    time::Instant,
};

use serde_json::{Value, json};

use super::*;
use crate::{
    capture::{decode_png, encode_png},
    geometry::PixelPoint,
    pyramid_progress::{self, BoardCount},
    pyramid_recon::vision::StockFace,
    worker::{LatestFrameSlot, TestCaptureBackend},
};


/// Original complete native board with a directly observed challenge header of zero-of-three.
const ZERO: &[u8] = include_bytes!("../../../tests/fixtures/pyramid-progress/zero-of-three.png");


/// Different complete retained tableau; recognition is production evidence, not a guessed deal.
const OTHER: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-recon/native-v13/restored.png");


/// Exact measured one-of-three fraction used only for labelled header corruption derivatives.
const ONE_FRACTION: &[u8; 3_864] =
    include_bytes!("../../../tests/fixtures/pyramid-progress/fraction-one.rgb");


/// Isolated socket names do not collide with other concurrently executed native fixtures.
static NEXT_SOCKET: AtomicUsize = AtomicUsize::new(0);


/// Decode a retained original through the production capture codec.
fn native(bytes: &[u8]) -> CapturedFrame {
    decode_png(bytes).expect("retained native starting-guard PNG")
}


/// Require a real supported Board observation from the actual deployed calibration.
fn position(frame: &CapturedFrame) -> BoardObservation {
    let vision = ReconVision::bundled_native_v14().expect("production bundled native-v14");
    match vision.observe(frame).expect("production recognition") {
        ReconScene::Board(board) => board,
        other => panic!("fixture needs a supported native board, observed {other:?}"),
    }
}


/// Inject a deterministic literal challenge counter constraint, never normal-game authority.
fn require_zero_header(frame: &CapturedFrame) -> Result<(), String> {
    match pyramid_progress::observe(frame)? {
        Some(BoardCount {
            completed: 0,
            target: 3,
        }) => Ok(()),
        other => Err(format!(
            "generic fixture requires literal 0/3, observed {other:?}"
        )),
    }
}


/// Change only the measured header fraction; all original card pixels remain untouched.
fn changed_header() -> CapturedFrame {
    let mut frame = native(ZERO);
    let (rgb, remainder) = ONE_FRACTION.as_chunks::<3>();
    assert!(remainder.is_empty(), "complete RGB fixture");
    let mut rgb = rgb.iter();
    for y in 50..78 {
        for x in 1_044..1_090 {
            let offset = y * frame.stride + x * 4;
            frame.pixels[offset..offset + 3]
                .copy_from_slice(rgb.next().expect("complete measured fraction RGB"));
        }
    }
    assert!(rgb.next().is_none());
    assert_eq!(position(&frame).tableau, position(&native(ZERO)).tableau);
    frame
}


/// Guarded initial piles are exact; later stock changes and toolbar enablement stay core-owned.
#[test]
fn starting_guard_binds_tableau_and_initial_piles_but_not_undo_enablement() {
    let frame = native(ZERO);
    let original = position(&frame);
    let guard = StartingGuard {
        position: &original,
        header: &require_zero_header,
    };
    assert!(
        guard
            .validate(&frame, &ReconScene::Board(original.clone()), 0)
            .is_ok()
    );
    let mut changed = original.clone();
    changed.tableau.swap(0, 1);
    assert!(
        guard
            .validate(&frame, &ReconScene::Board(changed), 1)
            .is_err()
    );
    let mut changed = original.clone();
    changed.left = StockFace::Empty;
    assert!(
        guard
            .validate(&frame, &ReconScene::Board(changed.clone()), 0)
            .is_err()
    );
    assert!(
        guard
            .validate(&frame, &ReconScene::Board(changed), 1)
            .is_ok()
    );
    let mut changed = original.clone();
    changed.right = original.left;
    assert!(
        guard
            .validate(&frame, &ReconScene::Board(changed.clone()), 0)
            .is_err()
    );
    assert!(
        guard
            .validate(&frame, &ReconScene::Board(changed), 24)
            .is_ok()
    );
    let mut toolbar = original.clone();
    toolbar.undo_all = Some(PixelPoint::new(1, 1));
    assert!(
        guard
            .validate(&frame, &ReconScene::Board(toolbar), 0)
            .is_ok()
    );
}


/// Only positive Board observations invoke the extra predicate; core still owns scene refusal.
#[test]
fn starting_guard_leaves_dialog_and_unknown_handling_to_the_existing_core() {
    let frame = native(ZERO);
    let original = position(&frame);
    let calls = Cell::new(0);
    let header = |_frame: &CapturedFrame| {
        calls.set(calls.get() + 1);
        Err("fixture header refused".to_owned())
    };
    let guard = StartingGuard {
        position: &original,
        header: &header,
    };
    assert!(guard.validate(&frame, &ReconScene::Unknown, 0).is_ok());
    assert!(
        guard
            .validate(
                &frame,
                &ReconScene::UndoConfirmation {
                    ok: PixelPoint::new(950, 660)
                },
                24,
            )
            .is_ok()
    );
    assert_eq!(calls.get(), 0);
    assert!(
        guard
            .validate(&frame, &ReconScene::Board(original.clone()), 24)
            .is_err()
    );
    assert_eq!(calls.get(), 1);
}


/// Remove only this episode's exclusively created local files after assertions complete.
struct TestDirectory(PathBuf);


impl Drop for TestDirectory {
    /// Owns this freshly created fixture tree; no live capture or repository evidence lives here.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}


/// Real wire counts remain independent of expected application branches.
#[derive(Default)]
struct WireReport {
    /// Actual native PNGs delivered through screendump.
    captures: usize,
    /// Concrete D-key down deliveries; all releases are separately acknowledged.
    draws: usize,
    /// Concrete D-key releases acknowledged by the server.
    releases: usize,
    /// Any absolute/button traffic is a forbidden cleanup or restoration input in these cases.
    pointer_events: usize,
    /// Successful fresh VM/tablet probe pairs.
    probes: usize,
    /// Fixture captures not requested before refusal.
    remaining: usize,
}


/// Preserve the numeric production request identity in a flushed local response.
pub(super) fn packet(stream: &mut UnixStream, value: Value) {
    serde_json::to_writer(&mut *stream, &value).expect("fixture QMP JSON");
    stream.write_all(b"\n").expect("fixture QMP newline");
    stream.flush().expect("fixture QMP flush");
}


/// Bound connection establishment so a preflight regression cannot hang the host suite.
pub(super) fn accept(listener: &UnixListener) -> UnixStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match listener.accept() {
            Ok((stream, _)) => return stream,
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "fixture QMP accept timed out");
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("fixture QMP accept: {error}"),
        }
    }
}


/// Serve bounded retained captures and record every delivered operation on a private local socket.
fn serve(
    listener: UnixListener,
    directory: PathBuf,
    mut captures: VecDeque<Vec<u8>>,
    cancel: Arc<AtomicBool>,
    stop_after_draw: bool,
) -> WireReport {
    let mut stream = accept(&listener);
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .expect("bounded fixture reads");
    let mut reader = BufReader::new(stream.try_clone().expect("fixture QMP reader"));
    packet(
        &mut stream,
        json!({"QMP": {
            "version": {"qemu": {"major": 11, "minor": 0, "micro": 0}},
            "capabilities": [],
        }}),
    );
    let mut report = WireReport::default();
    loop {
        let mut line = String::new();
        if reader
            .read_line(&mut line)
            .expect("fixture production command")
            == 0
        {
            break;
        }
        let request: Value = serde_json::from_str(&line).expect("fixture production JSON");
        let id = request["id"]
            .as_u64()
            .expect("numeric production request ID");
        let response = match request["execute"]
            .as_str()
            .expect("production command name")
        {
            "qmp_capabilities" => json!({}),
            "query-status" => json!({"running": true, "status": "running"}),
            "query-mice" => {
                report.probes += 1;
                json!([{"name": "QEMU HID Tablet", "index": 0, "current": true, "absolute": true}])
            }
            "screendump" => {
                let destination = PathBuf::from(
                    request["arguments"]["filename"]
                        .as_str()
                        .expect("private capture destination"),
                );
                assert_eq!(destination.parent(), Some(directory.as_path()));
                assert_eq!(request["arguments"]["format"], "png");
                let png = captures
                    .pop_front()
                    .expect("no additional fixture capture authorised");
                fs::write(destination, png).expect("fixture native capture bytes");
                report.captures += 1;
                json!({})
            }
            "input-send-event" => {
                for event in request["arguments"]["events"]
                    .as_array()
                    .expect("typed input events")
                {
                    if event["type"] == "key" {
                        assert_eq!(event["data"]["key"]["type"], "qcode");
                        assert_eq!(event["data"]["key"]["data"], "d");
                        if event["data"]["down"] == true {
                            report.draws += 1;
                        } else {
                            report.releases += 1;
                            if stop_after_draw {
                                cancel.store(true, Ordering::Release);
                            }
                        }
                    } else {
                        report.pointer_events += 1;
                    }
                }
                json!({})
            }
            other => panic!("unexpected starting-guard QMP command {other}"),
        };
        packet(&mut stream, json!({"return": response, "id": id}));
    }
    report.remaining = captures.len();
    report
}


/// Public owned wrappers and a specifically seeded adapter restoration boundary.
#[derive(Clone, Copy)]
enum Operation {
    /// Production acquisition with a borrowed starting witness.
    Guarded,
    /// Existing production acquisition without a starting guard.
    Unbound,
    /// Production observation/publication boundary with synthetic completed-restoration ledger state.
    RestoredBoundary,
}


/// Native failure, independent wire report and exact normal diagnostics retained together.
struct Episode {
    /// Actual adapter/wrapper outcome.
    result: Result<PathBuf, String>,
    /// Concrete QMP traffic observed independently of the controller.
    wire: WireReport,
    /// Existing INCOMPLETE diagnostic document.
    incomplete: Value,
    /// Exact first native PNG retained by the production adapter.
    baseline: Vec<u8>,
    /// Exact last native PNG retained by the production adapter.
    latest: Vec<u8>,
    /// Existing complete-deal destination remains absent after a guard refusal.
    published: bool,
    /// Same-frame diagnostic preview remains available after refusal.
    preview_retained: bool,
}


/// Exercise real decoding, recognition, authority checks, QMP and normal diagnostic publication.
fn run_episode(frames: Vec<Vec<u8>>, operation: Operation, stop_after_draw: bool) -> Episode {
    let _capture_backend = TestCaptureBackend::legacy_qmp();
    let directory = TestDirectory(std::env::temp_dir().join(format!(
        "pyramid-starting-guard-{}-{}",
        std::process::id(),
        NEXT_SOCKET.fetch_add(1, Ordering::Relaxed),
    )));
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&directory.0)
        .expect("private guard fixture directory");
    let context = ControllerContext::new(
        directory.0.join("qmp.sock"),
        GameMode::Pyramid,
        SolvingStrategy::ShortestPath,
        9,
    );
    let request = ReconRequest {
        profile: ReconProfileSource::BundledNativeV17,
        output_path: directory.0.join("unpublished-deal.json"),
        deal_id: "starting-guard-fixture".to_owned(),
        initial_board_confirmed: true,
    };
    let (events, _receiver) = mpsc::channel();
    let events = WorkerEventSink {
        events,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(context.clone())),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    events
        .latest_frame
        .reconnaissance
        .select(&context, &cancel)
        .expect("select fixture context");
    events
        .latest_frame
        .reconnaissance
        .queue_reconnaissance(&context, &cancel, || Ok(()))
        .expect("reserve fixture context");
    let listener = UnixListener::bind(&context.socket_path).expect("isolated guard QMP socket");
    listener.set_nonblocking(true).expect("bounded QMP accept");
    let server_directory = directory.0.clone();
    let server_cancel = Arc::clone(&cancel);
    let server = thread::spawn(move || {
        serve(
            listener,
            server_directory,
            frames.into(),
            server_cancel,
            stop_after_draw,
        )
    });
    let original = position(&native(ZERO));
    let guard = StartingGuard {
        position: &original,
        header: &require_zero_header,
    };
    let result = match operation {
        Operation::Guarded => {
            run_owned_with_starting_guard(&context, &request, 71, 2, guard, &events, &cancel)
        }
        Operation::Unbound => run_owned(&context, &request, 71, 2, &events, &cancel),
        Operation::RestoredBoundary => {
            restored_boundary(&context, &request, guard, &events, &cancel)
        }
    };
    let wire = server.join().expect("local guard QMP server");
    let diagnostic = fs::read_dir(&directory.0)
        .expect("fixture directory inventory")
        .map(|entry| entry.expect("fixture entry").path())
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with(".pyramid-recon-"))
        })
        .expect("normal production diagnostics directory");
    Episode {
        result,
        wire,
        incomplete: serde_json::from_slice(
            &fs::read(diagnostic.join("INCOMPLETE.json")).expect("normal incomplete proof"),
        )
        .expect("diagnostic JSON"),
        baseline: fs::read(diagnostic.join("baseline.png")).expect("original first PNG preserved"),
        latest: fs::read(diagnostic.join("latest.png")).expect("original last PNG preserved"),
        published: request.output_path.exists(),
        preview_retained: events.latest_frame.take().is_some(),
    }
}


/// Seed only ledger phase accounting to test the real adapter after acknowledged restoration.
///
/// This does not claim a native full-stock acquisition or replace the core's two
/// restoration checks. If both observations succeed, the actual publisher would
/// receive an existing complete fixture; a changed header must prevent that call.
fn restored_boundary(
    context: &ControllerContext,
    request: &ReconRequest,
    guard: StartingGuard<'_>,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> Result<PathBuf, String> {
    let output = resolve_new_output(&request.output_path)?;
    let mut diagnostics = Diagnostics::reserve(&output)?;
    let vision = load_vision(request)?;
    diagnostics.write_start(vision.profile_id(), &request.profile)?;
    let mut ledger = InputLedger {
        draws_attempted: 24,
        draws_acknowledged: 24,
        undo_attempted: 1,
        undo_acknowledged: true,
        confirm_attempted: 1,
        confirm_acknowledged: true,
        ..InputLedger::default()
    };
    let mut captures = 0;
    let mut publication = None;
    let deal = xsar::pyramid_json::read_from(
        include_bytes!("../../../tests/fixtures/pyramid-recon/native-v13/deal.json").as_slice(),
    )
    .expect("existing complete fixture");
    let mut qmp = QmpClient::connect(&context.socket_path).map_err(|error| error.to_string())?;
    let result = {
        let mut io = QmpReconIo {
            qmp: &mut qmp,
            context,
            control: &events.latest_frame.reconnaissance,
            cancel,
            events,
            vision,
            diagnostics: &mut diagnostics,
            output: &output,
            ledger: &mut ledger,
            captures: &mut captures,
            publication: &mut publication,
            starting_guard: Some(guard),
        };
        (|| {
            io.observe()?;
            io.observe()?;
            io.publish(&deal)?;
            Ok(output.clone())
        })()
    };
    let diagnostic_result = result.as_ref().map(|_| deal).map_err(Clone::clone);
    diagnostics.finish(&diagnostic_result, None, &ledger, captures, publication)?;
    result
}


/// Common assertions bind guard failures to existing first/latest evidence and no publication.
fn assert_refusal(episode: &Episode, captures: usize, draws: usize) {
    assert!(episode.result.is_err());
    assert_eq!(episode.wire.captures, captures);
    assert_eq!(episode.wire.draws, draws);
    assert_eq!(episode.wire.releases, draws);
    assert_eq!(episode.wire.pointer_events, 0);
    assert_eq!(episode.wire.remaining, 0);
    assert_eq!(episode.incomplete["status"], "INCOMPLETE");
    assert_eq!(episode.incomplete["publication_status"], "not_attempted");
    assert!(!episode.published);
    assert!(episode.preview_retained);
}


/// The first different supported tableau is rejected before even one D delivery.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn guarded_owned_rejects_changed_first_native_board_before_first_draw() {
    let episode = run_episode(vec![OTHER.to_vec()], Operation::Guarded, false);
    assert_refusal(&episode, 1, 0);
    assert!(
        episode
            .result
            .as_ref()
            .unwrap_err()
            .contains("changed original tableau")
    );
    assert_eq!(episode.baseline, OTHER);
    assert_eq!(episode.latest, OTHER);
    assert_eq!(episode.incomplete["draws_attempted"], 0);
    assert_eq!(episode.wire.probes, 1);
}


/// The second starting capture cannot substitute a different native deal before D.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn guarded_owned_rejects_changed_second_native_board_before_first_draw() {
    let episode = run_episode(
        vec![ZERO.to_vec(), OTHER.to_vec()],
        Operation::Guarded,
        false,
    );
    assert_refusal(&episode, 2, 0);
    assert!(
        episode
            .result
            .as_ref()
            .unwrap_err()
            .contains("changed original tableau")
    );
    assert_eq!(episode.baseline, ZERO);
    assert_eq!(episode.latest, OTHER);
    assert_eq!(episode.wire.probes, 1);
}


/// A header-only derivative is refused on either fresh starting observation before D.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn guarded_owned_rejects_first_and_second_header_drift_before_first_draw() {
    let changed = encode_png(&changed_header()).expect("labelled header derivative PNG");
    for second in [false, true] {
        let frames = if second {
            vec![ZERO.to_vec(), changed.clone()]
        } else {
            vec![changed.clone()]
        };
        let episode = run_episode(frames, Operation::Guarded, false);
        assert_refusal(&episode, if second { 2 } else { 1 }, 0);
        assert!(
            episode
                .result
                .as_ref()
                .unwrap_err()
                .contains("starting guard header mismatch")
        );
        assert_eq!(episode.latest, changed);
        assert_eq!(episode.wire.probes, 1);
    }
}


/// A same-frame header refusal after one acknowledged D prevents another D or cleanup input.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn guarded_owned_header_drift_after_one_draw_sends_no_next_draw_or_cleanup() {
    let changed = encode_png(&changed_header()).expect("labelled post-D header derivative");
    let episode = run_episode(
        vec![ZERO.to_vec(), ZERO.to_vec(), changed.clone()],
        Operation::Guarded,
        false,
    );
    assert_refusal(&episode, 3, 1);
    assert!(
        episode
            .result
            .as_ref()
            .unwrap_err()
            .contains("starting guard header mismatch")
    );
    assert_eq!(episode.incomplete["draws_attempted"], 1);
    assert_eq!(episode.incomplete["draws_acknowledged"], 1);
    assert_eq!(episode.incomplete["undo_all_attempted"], 0);
    assert_eq!(episode.incomplete["confirm_attempted"], 0);
    assert_eq!(episode.wire.probes, 2);
    assert_eq!(episode.latest, changed);
}


/// A changed second restored header reaches no publisher, even after seeded ACK accounting.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn guarded_adapter_checks_restored_header_before_publication() {
    let changed = encode_png(&changed_header()).expect("labelled restored-header derivative");
    let episode = run_episode(
        vec![ZERO.to_vec(), changed.clone()],
        Operation::RestoredBoundary,
        false,
    );
    assert_refusal(&episode, 2, 0);
    assert!(
        episode
            .result
            .as_ref()
            .unwrap_err()
            .contains("starting guard header mismatch")
    );
    assert_eq!(episode.incomplete["draws_acknowledged"], 24);
    assert_eq!(episode.incomplete["confirm_acknowledged"], true);
    assert_eq!(episode.wire.probes, 0);
    assert_eq!(episode.latest, changed);
}


/// The unchanged unbound wrapper remains header-indifferent and honours STOP after one D.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn unbound_owned_path_preserves_existing_header_indifference_and_stop() {
    let changed = encode_png(&changed_header()).expect("labelled header-only derivative");
    let episode = run_episode(vec![ZERO.to_vec(), changed], Operation::Unbound, true);
    assert_refusal(&episode, 2, 1);
    assert!(
        !episode
            .result
            .as_ref()
            .unwrap_err()
            .contains("starting guard")
    );
    assert_eq!(episode.incomplete["draws_attempted"], 1);
    assert_eq!(episode.wire.probes, 2);
}
