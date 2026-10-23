//! Phase-bound native post-confirmation regression through isolated QMP.
//!
//! These are adapter/core-boundary fixtures, not complete native 24-card runs.
//! Pre-confirmation stock observations and their ledger accounting are explicitly
//! synthetic. The core then performs one actual QMP Confirm, settles and consumes
//! genuine retained post-confirmation/restored captures through QmpReconIo. The
//! original starting/restored frames come from separate retained episodes; equal
//! recognised positions establish this fixture's controller contract only.

use std::{
    collections::VecDeque,
    io::{BufRead, BufReader},
    os::unix::net::UnixListener,
    sync::mpsc,
    thread,
};

use serde_json::{Value, json};
use xsar::games::cards::{Card, Rank, Suit};

use super::starting_guard_tests::{accept, packet};
use super::*;
use crate::{
    capture::{decode_png, encode_png},
    geometry::PixelPoint,
    pyramid_recon::vision::StockFace,
    worker::{LatestFrameSlot, TestCaptureBackend},
};


/// First retained full modal with the measured post-ACK OK appearance.
const POST_THREE: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-recon/native-v15/post-confirm-3.png");


/// Second unchanged retained modal, from a separate refused episode.
const POST_FOUR: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-recon/native-v15/post-confirm-4.png");


/// Third unchanged retained modal, from the Single board refusal.
const POST_FIVE: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-recon/native-v15/post-confirm-5.png");


/// Untouched original arrangement from the Single board refusal episode.
const START: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-recon/native-v15/post-confirm-start.png");


/// Same visible arrangement from a later retained successful manual episode.
const RESTORED: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-recon/native-v15/post-confirm-restored.png");


/// Socket/capture roots remain private and unique across parallel host tests.
static NEXT_SOCKET: AtomicU64 = AtomicU64::new(0);


/// Own only a newly created isolated test directory and its transport files.
struct TestDirectory(
    /// Exclusively created socket, capture and publication directory.
    PathBuf,
);


impl Drop for TestDirectory {
    /// Remove fixture-owned data after the bounded QMP peer has closed.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}


/// Deterministic authority revocation during initial settling or the first native capture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fault {
    /// Preserve the selected fixture authority.
    None,
    /// Actual STOP latches before the first fresh capture acknowledgement.
    Stop,
    /// Actual selection changes generation before that acknowledgement.
    Context,
    /// Actual STOP latches after the adapter enters the mandatory initial wait.
    StopDuringSettle,
    /// Actual selection changes after the adapter enters the mandatory initial wait.
    ContextDuringSettle,
}


/// Probe/capture/input evidence counted independently by the local QMP peer.
#[derive(Default)]
struct WireReport {
    /// Actual native capture requests.
    captures: usize,
    /// Actual absolute-pointer probes.
    probes: usize,
    /// Actual separately acknowledged typed input arrays.
    inputs: Vec<Vec<Value>>,
    /// Unrequested original fixtures prove refusal did not consume later frames.
    remaining: usize,
}


/// Serve the ordered original captures and at most one typed Confirm gesture.
fn serve(
    listener: UnixListener,
    directory: PathBuf,
    mut frames: VecDeque<Vec<u8>>,
    fault: Fault,
    control: ReconControl,
    mut context: ControllerContext,
    cancel: Arc<AtomicBool>,
) -> WireReport {
    let mut stream = accept(&listener);
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .expect("bounded post-confirm read");
    let mut reader = BufReader::new(stream.try_clone().expect("post-confirm QMP reader"));
    packet(
        &mut stream,
        json!({"QMP": {"version": {"qemu": {"major": 11, "minor": 0, "micro": 0}}, "capabilities": []}}),
    );
    let mut report = WireReport::default();
    loop {
        let mut line = String::new();
        if reader
            .read_line(&mut line)
            .expect("post-confirm QMP request")
            == 0
        {
            break;
        }
        let request: Value = serde_json::from_str(&line).expect("post-confirm request JSON");
        let id = request["id"].as_u64().expect("post-confirm numeric ID");
        let response = match request["execute"].as_str().expect("post-confirm command") {
            "qmp_capabilities" => json!({}),
            "query-status" => json!({"running": true, "status": "running"}),
            "query-mice" => {
                report.probes += 1;
                json!([{"name": "Fixture absolute tablet", "index": 0, "current": true, "absolute": true}])
            }
            "screendump" => {
                let destination = PathBuf::from(
                    request["arguments"]["filename"]
                        .as_str()
                        .expect("private post-confirm path"),
                );
                assert_eq!(destination.parent(), Some(directory.as_path()));
                assert_eq!(request["arguments"]["format"], "png");
                assert!(
                    !directory.join("complete.json").exists(),
                    "core must not publish before its observations finish"
                );
                fs::write(
                    destination,
                    frames.pop_front().expect("bounded post-confirm frames"),
                )
                .expect("retained native PNG bytes");
                report.captures += 1;
                if report.captures == 1 {
                    match fault {
                        Fault::None | Fault::StopDuringSettle | Fault::ContextDuringSettle => {}
                        Fault::Stop => control.stop(Some(&context), &cancel),
                        Fault::Context => {
                            context.generation += 1;
                            control
                                .select(&context, &cancel)
                                .expect("actual context revocation");
                        }
                    }
                }
                json!({})
            }
            "input-send-event" => {
                let events = request["arguments"]["events"]
                    .as_array()
                    .expect("typed Confirm arrays")
                    .clone();
                assert!(
                    events.iter().all(|event| event["type"] != "key"),
                    "synthetic prior D operations are never sent on the wire"
                );
                report.inputs.push(events);
                json!({})
            }
            other => panic!("unexpected post-confirm command: {other}"),
        };
        packet(&mut stream, json!({"return": response, "id": id}));
    }
    report.remaining = frames.len();
    report
}


/// A supported original position recognised by the deployed profile and actual codec.
fn original_position() -> BoardObservation {
    let vision = ReconVision::bundled_native_v15().expect("production native-v15");
    let ReconScene::Board(original) = vision
        .observe(&decode_png(START).expect("original PNG"))
        .expect("original recognition")
    else {
        panic!("retained start must be an ordinary supported board");
    };
    original
}


/// Construct a labelled synthetic pre-confirmation stock sequence from the unused identities.
///
/// Only original tableau and initial LEFT are native observations here. The
/// remaining stock order is arbitrary, unique and explicitly synthetic; it is
/// sufficient to reach the unchanged controller's actual restoration boundary.
fn synthetic_acquisition(original: &BoardObservation, ok: PixelPoint) -> VecDeque<ReconScene> {
    let StockFace::Card(first) = original.left else {
        panic!("original native LEFT identity is required");
    };
    assert_eq!(original.right, StockFace::Empty);
    let mut stock = vec![first];
    for rank in Rank::ALL {
        for suit in Suit::ALL {
            let card = Card::new(rank, suit);
            if card != first && !original.tableau.contains(&card) {
                stock.push(card);
            }
        }
    }
    assert_eq!(stock.len(), 24, "exact complete synthetic complement");
    let mut scenes = VecDeque::from([
        ReconScene::Board(original.clone()),
        ReconScene::Board(original.clone()),
    ]);
    for left_index in std::iter::once(1).chain((3..24).step_by(2)) {
        let mut board = original.clone();
        board.left = StockFace::Card(stock[left_index]);
        board.right = StockFace::Card(stock[left_index - 1]);
        board.undo_all = Some(PixelPoint::new(1320, 978));
        scenes.push_back(ReconScene::Board(board));
    }
    let mut exhausted = original.clone();
    exhausted.left = StockFace::Empty;
    exhausted.right = StockFace::Card(stock[23]);
    exhausted.undo_all = Some(PixelPoint::new(1320, 978));
    scenes.extend([
        ReconScene::Board(exhausted.clone()),
        ReconScene::Board(exhausted),
        ReconScene::UndoConfirmation { ok },
        ReconScene::UndoConfirmation { ok },
    ]);
    scenes
}


/// Explicit synthetic acquisition prefix followed by the real production native adapter.
struct CoreBoundary<'adapter, 'io> {
    /// Existing application adapter owns all actual post-confirmation I/O.
    native: &'adapter mut QmpReconIo<'io>,
    /// Synthetic pre-confirmation observations, never presented as native frames.
    prefix: VecDeque<ReconScene>,
    /// Actual initial settling and known-stale retry delays requested by the controller.
    waits: Vec<Duration>,
    /// Native captures already attempted when each post-confirmation wait begins.
    wait_capture_counts: Vec<usize>,
    /// Optional real authority revocation injected after the wait announces Verifying.
    fault: Fault,
    /// Owned event receiver lets the fault wait for actual production settling entry.
    wait_events: Option<mpsc::Receiver<crate::worker::ContextualWorkerEvent>>,
}


impl ReconIo for CoreBoundary<'_, '_> {
    /// Actual selected context and STOP remain authoritative throughout the fixture.
    fn check_active(&self) -> Result<(), String> {
        self.native.check_active()
    }


    /// Hand over to real native capture only once the synthetic prefix is exhausted.
    fn observe(&mut self) -> Result<ReconScene, String> {
        match self.prefix.pop_front() {
            Some(scene) => Ok(scene),
            None => self.native.observe(),
        }
    }


    /// Only the single actual Confirm needs and consumes an actual fresh VM probe.
    fn probe(&mut self) -> Result<(), String> {
        if self.prefix.is_empty() {
            self.native.probe()
        } else {
            self.native.check_active()
        }
    }


    /// Seed only labelled synthetic prefix accounting; Confirm uses actual guarded QMP delivery.
    fn input(&mut self, operation: ReconInput) -> Result<(), String> {
        self.native.check_active()?;
        match operation {
            ReconInput::Draw => {
                self.native.ledger.draws_attempted += 1;
                self.native.ledger.draws_acknowledged += 1;
                Ok(())
            }
            ReconInput::UndoAll(_) => {
                self.native.ledger.undo_attempted += 1;
                self.native.ledger.undo_acknowledged = true;
                Ok(())
            }
            ReconInput::Confirm(_) => {
                assert!(
                    self.prefix.is_empty(),
                    "only Confirm crosses the native boundary"
                );
                self.native.input(operation)
            }
        }
    }


    /// Exercise real initial/stale waits and revoke only after production announces settling.
    fn wait(&mut self, duration: Duration) -> Result<(), String> {
        if self.native.ledger.confirm_acknowledged {
            let first_wait = self.waits.is_empty();
            self.waits.push(duration);
            self.wait_capture_counts.push(*self.native.captures);
            let fault = self.fault;
            if first_wait && matches!(fault, Fault::StopDuringSettle | Fault::ContextDuringSettle) {
                assert_eq!(
                    *self.native.captures, 0,
                    "initial wait precedes every native capture"
                );
                let receiver = self.wait_events.take().expect("owned wait-entry receiver");
                let control = self.native.control.clone();
                let mut context = self.native.context.clone();
                let cancel = self.native.cancel;
                thread::scope(|scope| {
                    let revoke = scope.spawn(move || {
                        loop {
                            let event = receiver
                                .recv_timeout(Duration::from_secs(2))
                                .expect("production wait must announce Verifying");
                            if matches!(event.event, WorkerEvent::State(WorkerState::Verifying)) {
                                break;
                            }
                        }
                        match fault {
                            Fault::StopDuringSettle => control.stop(Some(&context), cancel),
                            Fault::ContextDuringSettle => {
                                context.generation += 1;
                                control
                                    .select(&context, cancel)
                                    .expect("context revocation during wait");
                            }
                            _ => unreachable!("only wait-entry faults start this scoped peer"),
                        }
                    });
                    let result = self.native.wait(duration);
                    revoke.join().expect("scoped wait revocation completes");
                    result
                })
            } else {
                self.native.wait(duration)
            }
        } else {
            self.native.check_active()
        }
    }


    /// Preserve the core's actual progress/status forwarding.
    fn progress(&mut self, message: &str) {
        self.native.progress(message);
    }


    /// Leave publication entirely to the existing production adapter and core decision.
    fn publish(&mut self, deal: &CompleteDeal) -> Result<(), String> {
        self.native.publish(deal)
    }
}


/// Actual adapter outcome plus independent transport, ledger and publication evidence.
struct Episode {
    /// Controller or direct pre-ACK observation result, without a live acceptance claim.
    result: Result<(), String>,
    /// Independently counted actual wire operations.
    wire: WireReport,
    /// Actual initial settling and subsequent known-stale observation waits.
    waits: Vec<Duration>,
    /// Actual native capture attempts preceding each wait; the first must be zero.
    wait_capture_counts: Vec<usize>,
    /// Complete output publication was attempted/succeeded only by the real adapter.
    publication: Option<PublicationStatus>,
    /// Original first/latest pixels, or two empty vectors when settling prevented all captures.
    retained: (Vec<u8>, Vec<u8>),
    /// Confirmation phase accounting; the pre-ACK case seeds only its attempted flag.
    confirms: (usize, bool),
    /// Counts actual adapter captures independently of the synthetic prefix.
    captures: usize,
    /// Real complete output exists only after successful core restoration proof.
    published: bool,
}


/// Execute actual post-confirmation I/O, optionally with an attempted but unacknowledged phase.
fn episode(frames: Vec<Vec<u8>>, run_core: bool, fault: Fault) -> Episode {
    let _capture_backend = TestCaptureBackend::legacy_qmp();
    let directory = TestDirectory(std::env::temp_dir().join(format!(
        "pyramid-post-confirm-{}-{}",
        std::process::id(),
        NEXT_SOCKET.fetch_add(1, Ordering::Relaxed)
    )));
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&directory.0)
        .expect("private post-confirm directory");
    let context = ControllerContext::new(
        directory.0.join("qmp.sock"),
        GameMode::Pyramid,
        SolvingStrategy::ShortestPath,
        19,
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let (events, receiver) = mpsc::channel();
    let events = WorkerEventSink {
        events,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(context.clone())),
    };
    let control = &events.latest_frame.reconnaissance;
    control
        .select(&context, &cancel)
        .expect("post-confirm selected context");
    control
        .queue_reconnaissance(&context, &cancel, || Ok(()))
        .expect("post-confirm reservation");
    let request = ReconRequest {
        profile: ReconProfileSource::File(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-recon/native-v15/profile.json"),
        ),
        output_path: directory.0.join("complete.json"),
        deal_id: "synthetic-prefix-native-restoration".to_owned(),
        initial_board_confirmed: true,
    };
    let output = resolve_new_output(&request.output_path).expect("private new output");
    let mut diagnostics = Diagnostics::reserve(&output).expect("production evidence directory");
    let vision = load_vision(&request).expect("production profile load");
    diagnostics
        .write_start(vision.profile_id(), &request.profile)
        .expect("production diagnostic start");
    let listener =
        UnixListener::bind(&context.socket_path).expect("isolated post-confirm QMP socket");
    listener
        .set_nonblocking(true)
        .expect("bounded post-confirm accept");
    let server_directory = directory.0.clone();
    let server_cancel = Arc::clone(&cancel);
    let server_control = control.clone();
    let server_context = context.clone();
    let server = thread::spawn(move || {
        serve(
            listener,
            server_directory,
            frames.into(),
            fault,
            server_control,
            server_context,
            server_cancel,
        )
    });
    let mut qmp =
        QmpClient::connect(&context.socket_path).expect("real post-confirm QMP handshake");
    let mut ledger = if run_core {
        InputLedger::default()
    } else {
        InputLedger {
            confirm_attempted: 1,
            ..InputLedger::default()
        }
    };
    let mut captures = 0;
    let mut publication = None;
    let result;
    let waits;
    let wait_capture_counts;
    let receipt_vision;
    {
        let mut native = QmpReconIo {
            qmp: &mut qmp,
            context: &context,
            control,
            cancel: &cancel,
            events: &events,
            vision,
            diagnostics: &mut diagnostics,
            output: &output,
            ledger: &mut ledger,
            captures: &mut captures,
            publication: &mut publication,
            starting_guard: None,
        };
        if run_core {
            let original = original_position();
            let phase_scene = native
                .vision
                .observe_after_confirmation(&decode_png(POST_THREE).expect("retained post-ACK PNG"))
                .expect("phase-specific native recognition");
            let ReconScene::UndoConfirmation { ok } = phase_scene else {
                panic!("retained post-ACK frame supplies only known-stale modal evidence");
            };
            let mut boundary = CoreBoundary {
                native: &mut native,
                prefix: synthetic_acquisition(&original, ok),
                waits: Vec::new(),
                wait_capture_counts: Vec::new(),
                fault,
                wait_events: Some(receiver),
            };
            result = pyramid_recon::run(&mut boundary, &request).map_err(|failure| failure.message);
            assert!(
                boundary.prefix.is_empty(),
                "the synthetic acquisition boundary is complete"
            );
            waits = boundary.waits;
            wait_capture_counts = boundary.wait_capture_counts;
        } else {
            result = native
                .observe()
                .and_then(|scene| {
                    if scene == ReconScene::Unknown {
                        Ok(())
                    } else {
                        Err(format!("unacknowledged phase recognised {scene:?}"))
                    }
                })
                .and_then(|()| Err("expected strict Unknown before confirmation ACK".to_owned()));
            waits = Vec::new();
            wait_capture_counts = Vec::new();
        }
        receipt_vision = native.vision;
    }
    let receipt_progress = result
        .as_ref()
        .ok()
        .map(|_| diagnostics.restored_progress());
    let receipt_default = result.as_ref().ok().map(|_| diagnostics.restored_default());
    let receipt_observed = result
        .as_ref()
        .ok()
        .map(|_| diagnostics.restored_observed());
    diagnostics
        .finish(&result, None, &ledger, captures, publication)
        .expect("production final evidence");
    assert!(
        diagnostics.latest_png.is_none(),
        "finalisation releases the PNG buffer"
    );
    let result = result.and_then(|deal| {
        let progress = receipt_progress.ok_or("Missing successful restoration binding")??;
        super::super::computed_route::record_acquisition(
            &context,
            &events,
            &cancel,
            deal,
            receipt_vision,
            &output,
            progress,
            receipt_default.ok_or("Missing successful default binding")??,
            receipt_observed.ok_or("Missing successful observed header")??,
        )?;
        let (_, retained) =
            super::super::computed_route::reuse_metadata(&context, &output, &events, &cancel)?;
        assert_eq!(
            super::super::computed_route::acquisition_observed(
                &context, &output, &events, &cancel
            )?,
            crate::pyramid_workflow_model::ObservedGame::observe(
                &decode_png(RESTORED).expect("actual restored frame")
            )?
        );
        assert_eq!(
            super::super::computed_route::acquisition_default(&context, &output, &events, &cancel)?,
            crate::pyramid_workflow_model::RouteMethod::FewestActions,
            "this actual non-Kings restored frame retains the Shortest default",
        );
        assert_eq!(
            retained, progress,
            "exact restored-frame requirement survives publication"
        );
        assert_eq!(
            retained,
            crate::pyramid_progress::observe_challenge(
                &decode_png(RESTORED).expect("native restored PNG")
            )?,
            "receipt binds the actual final restored pixels",
        );
        Ok(())
    });
    let retained = if captures == 0 {
        assert!(!diagnostics.directory.join("baseline.png").exists());
        assert!(!diagnostics.directory.join("latest.png").exists());
        (Vec::new(), Vec::new())
    } else {
        (
            fs::read(diagnostics.directory.join("baseline.png"))
                .expect("retained first actual capture"),
            fs::read(diagnostics.directory.join("latest.png"))
                .expect("retained last actual capture"),
        )
    };
    drop(qmp);
    let wire = server.join().expect("post-confirm local peer completion");
    Episode {
        result: result.map(|_| ()),
        wire,
        waits,
        wait_capture_counts,
        publication,
        retained,
        confirms: (ledger.confirm_attempted, ledger.confirm_acknowledged),
        captures,
        published: output.exists(),
    }
}


/// A complete acknowledgement creates no second confirmation or synthetic native D input.
fn assert_one_confirm(episode: &Episode) {
    assert_eq!(episode.confirms, (1, true));
    assert_eq!(episode.wire.probes, 1);
    assert_eq!(
        episode.wire.inputs.len(),
        3,
        "one movement, one DOWN and one UP"
    );
    let transitions: Vec<bool> = episode
        .wire
        .inputs
        .iter()
        .flat_map(|events| {
            events
                .iter()
                .filter_map(|event| event["data"]["down"].as_bool())
        })
        .collect();
    assert_eq!(transitions, [true, false]);
    let vision = ReconVision::bundled_native_v15().expect("frozen production native-v15");
    let ReconScene::UndoConfirmation { ok } = vision
        .observe_after_confirmation(&decode_png(POST_THREE).expect("unchanged modal PNG"))
        .expect("full post-confirmation modal")
    else {
        panic!("a complete post-confirmation modal is required");
    };
    let mapped = crate::geometry::pixel_point_to_qmp(ok, 1920, 1080)
        .expect("checked measured Confirm mapping");
    assert_eq!(
        episode.wire.inputs[0],
        [
            json!({"type": "abs", "data": {"axis": "x", "value": mapped.x}}),
            json!({"type": "abs", "data": {"axis": "y", "value": mapped.y}}),
        ]
    );
    for events in episode.wire.inputs.iter().skip(1) {
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["type"], "btn");
        assert_eq!(events[0]["data"]["button"], "left");
    }
}


/// The real adapter keeps every retained altered OK modal unknown until an actual ACK.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_post_confirm_modal_is_unknown_before_confirmation_ack() {
    for png in [POST_THREE, POST_FOUR, POST_FIVE] {
        let episode = episode(vec![png.to_vec()], false, Fault::None);
        assert!(
            episode
                .result
                .as_ref()
                .is_err_and(|error| error == "expected strict Unknown before confirmation ACK")
        );
        assert_eq!(
            episode.confirms,
            (1, false),
            "an attempt alone cannot activate the phase"
        );
        assert_eq!(
            (episode.captures, episode.wire.captures, episode.wire.probes),
            (1, 1, 0)
        );
        assert!(episode.wire.inputs.is_empty());
        assert_eq!(episode.retained, (png.to_vec(), png.to_vec()));
        assert_eq!(episode.publication, None);
        assert!(!episode.published);
    }
}


/// The first capture follows settling; stale modals wait without replay before two restored proofs.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_post_confirm_modal_waits_then_core_requires_two_actual_restored_positions() {
    let episode = episode(
        vec![
            POST_THREE.to_vec(),
            POST_FOUR.to_vec(),
            POST_FIVE.to_vec(),
            RESTORED.to_vec(),
            RESTORED.to_vec(),
        ],
        true,
        Fault::None,
    );
    assert!(episode.result.is_ok(), "{:?}", episode.result);
    assert_one_confirm(&episode);
    assert_eq!(
        (
            episode.captures,
            episode.wire.captures,
            episode.wire.remaining
        ),
        (5, 5, 0)
    );
    assert_eq!(episode.waits, vec![Duration::from_secs(1); 4]);
    assert_eq!(episode.wait_capture_counts, [0, 1, 2, 3]);
    assert_eq!(episode.publication, Some(PublicationStatus::Published));
    assert!(episode.published);
    assert_eq!(episode.retained, (POST_THREE.to_vec(), RESTORED.to_vec()));
}


/// A single matching original interrupted by a known stale modal cannot count as two consecutive proofs.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_post_confirm_restoration_requires_consecutive_actual_originals() {
    let episode = episode(
        vec![
            RESTORED.to_vec(),
            POST_FIVE.to_vec(),
            RESTORED.to_vec(),
            POST_FOUR.to_vec(),
            RESTORED.to_vec(),
            POST_THREE.to_vec(),
        ],
        true,
        Fault::None,
    );
    assert!(
        episode
            .result
            .as_ref()
            .is_err_and(|error| error.contains("restoration twice")),
        "{:?}",
        episode.result
    );
    assert_one_confirm(&episode);
    assert_eq!(
        (
            episode.captures,
            episode.wire.captures,
            episode.wire.remaining
        ),
        (6, 6, 0)
    );
    assert_eq!(episode.waits, vec![Duration::from_secs(1); 3]);
    assert_eq!(episode.wait_capture_counts, [0, 2, 4]);
    assert_eq!(episode.publication, None);
    assert!(!episode.published);
}


/// A persistent altered OK modal consumes the six-observation bound without replay or publication.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_post_confirm_persistent_modal_exhausts_bound_without_second_confirm() {
    let episode = episode(vec![POST_FIVE.to_vec(); 6], true, Fault::None);
    assert!(
        episode
            .result
            .as_ref()
            .is_err_and(|error| error.contains("restoration twice")),
        "{:?}",
        episode.result
    );
    assert_one_confirm(&episode);
    assert_eq!(
        (
            episode.captures,
            episode.wire.captures,
            episode.wire.remaining
        ),
        (6, 6, 0)
    );
    assert_eq!(episode.waits, vec![Duration::from_secs(1); 6]);
    assert_eq!(episode.wait_capture_counts, [0, 1, 2, 3, 4, 5]);
    assert_eq!(episode.publication, None);
    assert!(!episode.published);
    assert_eq!(episode.retained, (POST_FIVE.to_vec(), POST_FIVE.to_vec()));
}


/// Actual STOP/context revocation inside initial settling prevents the first native capture.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_post_confirm_initial_settle_revocation_prevents_first_capture() {
    for fault in [Fault::StopDuringSettle, Fault::ContextDuringSettle] {
        let episode = episode(vec![RESTORED.to_vec(), RESTORED.to_vec()], true, fault);
        assert!(
            episode
                .result
                .as_ref()
                .is_err_and(|error| error.contains("STOP")),
            "{:?}",
            episode.result
        );
        assert_one_confirm(&episode);
        assert_eq!(
            (
                episode.captures,
                episode.wire.captures,
                episode.wire.remaining
            ),
            (0, 0, 2),
            "neither restored frame may be requested after wait-entry revocation"
        );
        assert_eq!(episode.waits, [Duration::from_secs(1)]);
        assert_eq!(episode.wait_capture_counts, [0]);
        assert_eq!(episode.retained, (Vec::new(), Vec::new()));
        assert_eq!(episode.publication, None);
        assert!(!episode.published);
    }
}


/// STOP and generation changes after the actual ACK forbid observation retries, cleanup and publication.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_post_confirm_stop_or_context_revocation_forbids_further_io() {
    for fault in [Fault::Stop, Fault::Context] {
        let episode = episode(
            vec![POST_FIVE.to_vec(), RESTORED.to_vec(), RESTORED.to_vec()],
            true,
            fault,
        );
        assert!(
            episode
                .result
                .as_ref()
                .is_err_and(|error| error.contains("STOP or changed context")),
            "{:?}",
            episode.result
        );
        assert_one_confirm(&episode);
        assert_eq!(
            (
                episode.captures,
                episode.wire.captures,
                episode.wire.remaining
            ),
            (1, 1, 2)
        );
        assert_eq!(episode.waits, [Duration::from_secs(1)]);
        assert_eq!(episode.wait_capture_counts, [0]);
        assert_eq!(episode.publication, None);
        assert!(!episode.published);
        assert_eq!(episode.retained, (POST_FIVE.to_vec(), POST_FIVE.to_vec()));
    }
}


/// Unknown corruption still aborts immediately even after one acknowledged confirmation.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_post_confirm_corrupted_modal_never_becomes_known_stale() {
    let mut frame = decode_png(POST_FIVE).expect("unchanged original post-confirm PNG");
    // Explicitly synthetic title corruption; all production scene gates still apply.
    for y in 417..439 {
        for x in 729..907 {
            let offset = y * frame.stride + x * 4;
            frame.pixels[offset..offset + 3].fill(0);
        }
    }
    let corrupted = encode_png(&frame).expect("labelled synthetic modal derivative");
    let episode = episode(
        vec![corrupted.clone(), RESTORED.to_vec(), RESTORED.to_vec()],
        true,
        Fault::None,
    );
    assert!(
        episode
            .result
            .as_ref()
            .is_err_and(|error| error.contains("Unknown native scene")),
        "{:?}",
        episode.result
    );
    assert_one_confirm(&episode);
    assert_eq!(
        (
            episode.captures,
            episode.wire.captures,
            episode.wire.remaining
        ),
        (1, 1, 2)
    );
    assert_eq!(episode.waits, [Duration::from_secs(1)]);
    assert_eq!(episode.wait_capture_counts, [0]);
    assert_eq!(episode.publication, None);
    assert!(!episode.published);
    assert_eq!(episode.retained, (corrupted.clone(), corrupted));
}
