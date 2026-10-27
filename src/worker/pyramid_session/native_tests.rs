//! Actual native session observations through retained PNGs and isolated QMP sockets.
//!
//! The socket fixtures invoke production entry, transition and restart methods,
//! capture ownership, PNG decoding and recognition. Restart peers acknowledge
//! local typed controls only; no guest endpoint or planning receipt is supplied.
//! Header/menu derivatives test reconciliation and stage order without claiming
//! additional native calibration, real third-clear evidence or guest effects.

use std::{
    collections::VecDeque,
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::{
        fs::DirBuilderExt,
        net::{UnixListener, UnixStream},
    },
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Instant,
};

use serde_json::{Value, json};

use super::*;
use crate::{
    capture::{PixelFormat, decode_png, encode_png},
    geometry::{PixelPoint, PixelRect},
    pyramid_recon::ReconProfileSource,
    pyramid_workflow_model::RouteMethod,
    strategy::SolvingStrategy,
    terminal_controls::{Control, control_point, profiles::PYRAMID_CONTROLS},
    worker::{ContextualWorkerEvent, LatestFrameSlot, TestCaptureBackend},
};


/// Retained untouched native board with an independently measured literal 0/3 header.
const ZERO: &[u8] = include_bytes!("../../../tests/fixtures/pyramid-progress/zero-of-three.png");


/// Existing native normal game explicitly displays Boards: 1 of 3 and the New footer.
const NORMAL_ONE: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-recon/originals/current-start.png");


/// Three unchanged operator-captured normal headers accompany their original full boards.
const NORMAL_SERIES: [&[u8]; 3] = [
    include_bytes!("../../../tests/fixtures/pyramid-standard-series/normal-1.png"),
    include_bytes!("../../../tests/fixtures/pyramid-standard-series/normal-2.png"),
    include_bytes!("../../../tests/fixtures/pyramid-standard-series/normal-3.png"),
];


/// Retained untouched native board whose visible literal count is 1/3.
const ONE: &[u8] = include_bytes!("../../../tests/fixtures/pyramid-recon/native-v13/restored.png");


/// Retained untouched native board whose visible literal count is 2/3.
const TWO: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-recon/native-v13/recon-king.png");


/// Retained partial PLAY position with a legitimate 1/3 header and seventeen absences.
const PARTIAL: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-recon/native-v13/selected.png");


/// Specific retained Pyramid level-up artwork supplies terminal decoration only.
const TERMINAL: &[u8] =
    include_bytes!("../../../tests/fixtures/ok-confirmation-20261016/pyramid/clean.png");


/// Complete real 1/3 RGB patch, used only to construct a labelled header-only derivative.
const ONE_FRACTION: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-progress/fraction-one.rgb");


/// Measured half-open native fraction bounds in full-frame pixels.
const FRACTION_BOUNDS: PixelRect = PixelRect::new(1_044, 50, 46, 28);


/// Unique fixture directory suffix; no environment or production process setting changes.
static NEXT_SOCKET: AtomicUsize = AtomicUsize::new(0);


mod readiness_tests;


/// Decode retained native bytes using the shared production PNG codec.
fn native(png: &[u8]) -> CapturedFrame {
    decode_png(png).expect("retained native session PNG")
}


/// Change only the fraction to real 1/3 RGB, preserving all original board identities.
fn zero_board_with_one_header() -> CapturedFrame {
    let mut frame = native(ZERO);
    let (pixels, remainder) = ONE_FRACTION.as_chunks::<3>();
    assert!(remainder.is_empty());
    let mut source = pixels.iter();
    for y in FRACTION_BOUNDS.y..FRACTION_BOUNDS.bottom() {
        for x in FRACTION_BOUNDS.x..FRACTION_BOUNDS.right() {
            let offset = y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 3]
                .copy_from_slice(source.next().expect("complete measured fraction RGB"));
        }
    }
    assert!(source.next().is_none());
    frame
}


/// Make a full native board's left header word unknown without altering any card pixels.
fn zero_board_without_header() -> CapturedFrame {
    let mut frame = native(ZERO);
    let label = PixelRect::new(839, 50, 96, 28);
    for y in label.y..label.bottom() {
        for x in label.x..label.right() {
            let offset = y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 3].fill(255);
        }
    }
    frame
}


/// Own only this fixture's socket and otherwise empty private directory.
struct SocketDirectory(PathBuf);


impl Drop for SocketDirectory {
    /// Preserve unexpected files for diagnosis rather than deleting recursively.
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0.join("qmp.sock"));
        let _ = fs::remove_dir(&self.0);
    }
}


/// Real worker authority and adapter inputs, independent of all process-global settings.
struct Fixture {
    /// Private capture directory and optional local socket, removed on drop when empty.
    directory: SocketDirectory,
    /// Original immutable controller identity used by every production method.
    context: ControllerContext,
    /// Automatic request whose output is never acquired or published by these tests.
    request: SessionRequest,
    /// Real bounded preview slot, authority reservation and event channel.
    events: WorkerEventSink,
    /// Monotonic cancellation latch shared with the local QMP server.
    cancel: Arc<AtomicBool>,
    /// Keep advisory event delivery available without running a UI consumer.
    _receiver: mpsc::Receiver<ContextualWorkerEvent>,
}


impl Fixture {
    /// Reserve real reconnaissance authority without connecting, acquiring or granting input.
    fn new() -> Self {
        let directory = SocketDirectory(std::env::temp_dir().join(format!(
            "pyramid-session-native-{}-{}",
            std::process::id(),
            NEXT_SOCKET.fetch_add(1, Ordering::Relaxed)
        )));
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory.0)
            .expect("unique private native session directory");
        let context = ControllerContext::new(
            directory.0.join("qmp.sock"),
            GameMode::Pyramid,
            SolvingStrategy::ShortestPath,
            3,
        );
        let request = SessionRequest {
            reuse_acquisition: None,
            run_id: 1,
            mode: WorkflowMode::Continuous,
            reconnaissance: ReconRequest {
                profile: ReconProfileSource::BundledNativeV17,
                output_path: directory.0.join("unpublished-deal.json"),
                deal_id: "native-session-test".to_owned(),
                initial_board_confirmed: true,
            },
            method: RouteMethod::Fast,
            use_challenge_default: false,
            route_goal: crate::pyramid_workflow_model::RouteGoal::MaximumCardMatches,
            stop_condition: crate::pyramid_workflow_model::StopCondition::RepeatNormalGames,
        };
        let (events, receiver) = mpsc::channel();
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
            .expect("select native session fixture");
        events
            .latest_frame
            .reconnaissance
            .queue_reconnaissance(&context, &cancel, || Ok(()))
            .expect("reserve native session fixture");
        Self {
            directory,
            context,
            request,
            events,
            cancel,
            _receiver: receiver,
        }
    }


    /// Construct the production adapter with the current bundled calibration and no receipt.
    fn adapter(&self) -> Native<'_> {
        Native {
            context: &self.context,
            request: &self.request,
            events: &self.events,
            cancel: &self.cancel,
            phase: SessionPhase::Entry,
            generation: 0,
            vision: Some(match self.request.reconnaissance.profile {
                ReconProfileSource::BundledNativeV23 => ReconVision::bundled_native_v23().unwrap(),
                ReconProfileSource::BundledNativeV22 => {
                    ReconVision::bundled_native_v22().expect("production native v22")
                }
                _ => ReconVision::bundled_native_v17().expect("production native v17"),
            }),
            output: None,
            latest: None,
            starting: None,
            series: None,
            challenge_starting: None,
            challenge_progress: None,
            original_tableau: None,
            distinct_start: false,
            restart_entry: None,
            failure_saved: false,
            method: self.request.method,
            reuse_acquisition: self.request.reuse_acquisition.clone(),
            observed_game: crate::pyramid_workflow_model::ObservedGame::Unknown,
        }
    }
}


/// A Maximum fallback is discarded before a new board and in its published UI state.
#[test]
fn new_board_resets_maximum_fallback_to_shortest_before_reconnaissance() {
    let fixture = Fixture::new();
    let mut adapter = fixture.adapter();
    adapter.phase(1, SessionPhase::Execution);
    assert_eq!(adapter.method, RouteMethod::Fast);
    adapter.phase(2, SessionPhase::Reconnaissance);
    assert_eq!(adapter.method, RouteMethod::FewestActions);
    assert!(fixture._receiver.try_iter().any(|event| matches!(
        event.event,
        WorkerEvent::PyramidSessionProgress(SessionProgress {
            board_generation: 2,
            phase: SessionPhase::Reconnaissance,
            effective_method: RouteMethod::FewestActions,
            ..
        })
    )));
}


/// Actual adapter method under test; neither variant reimplements its observation loop.
#[derive(Clone, Copy, Debug)]
enum Operation {
    /// Observe a fresh board with unchanged native-v22 and no challenge classification.
    StartingReadiness,
    /// Invoke actual second-generation no-challenge acquisition before any stock input.
    NoChallengeAcquire,
    /// Retain an actually bound requirement across an input-free transition.
    BoundTransition(ChallengeProgress),
    /// Inject only the already retired clear/ACK handoff; real paired captures must prove a distinct board.
    ClearTransition(ChallengeProgress),
    /// Inject an original normal board and retired ACK handoff; all next-board evidence stays native.
    NormalClearTransition,
    /// Capture the production entry's pair of fresh frames.
    Entry,
    /// Capture a genuine generic Card entry with the current native calibration.
    CardEntry,
    /// Capture an actual colour-independent native-v23 challenge entry.
    ColourEntry,
    /// Start a new target-agnostic session after the previous reservation retired.
    NoChallengeEntry,
    /// Start a new single-board session, independently of the old preview.
    SingleBoardEntry,
    /// Run the production transition's bounded consecutive-observation loop.
    Transition,
}


/// Deterministic authority loss after writing the first PNG and before its QMP ACK.
#[derive(Clone, Copy, Debug)]
enum Revocation {
    /// Retain original authority throughout the episode.
    None,
    /// Use the actual UI STOP control at the first capture response boundary.
    Stop,
    /// Select another generation, then the original, proving revocation stays latched.
    ContextRoundTrip,
}


/// Concrete wire activity; any unsupported command fails the isolated server immediately.
#[derive(Default)]
struct WireReport {
    /// Successfully supplied PNG captures, including the one delivered before revocation.
    captures: usize,
    /// Every concrete protocol command, including the capabilities handshake.
    commands: Vec<String>,
    /// Frames left unused when the production adapter returns or refuses the pair.
    remaining: usize,
}


/// Flush one local QMP response, retaining the production transport's numeric identifier.
fn packet(stream: &mut UnixStream, value: Value) {
    serde_json::to_writer(&mut *stream, &value).expect("native session QMP packet");
    stream.write_all(b"\n").expect("native session QMP newline");
    stream.flush().expect("native session QMP flush");
}


/// Accept within five seconds so an early production refusal cannot hang a socket fixture.
fn accept(listener: &UnixListener) -> UnixStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match listener.accept() {
            Ok((stream, _)) => return stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(
                    Instant::now() < deadline,
                    "native session QMP accept timed out"
                );
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("native session QMP accept: {error}"),
        }
    }
}


/// Supply exact PNGs to the real capture path; forbid probes, pointer events and key input.
fn serve(
    listener: UnixListener,
    directory: PathBuf,
    mut captures: VecDeque<Vec<u8>>,
    control: pyramid_reconnaissance::ReconControl,
    context: ControllerContext,
    cancel: Arc<AtomicBool>,
    revocation: Revocation,
) -> WireReport {
    let mut stream = accept(&listener);
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .expect("bounded native session QMP reads");
    let mut reader = BufReader::new(stream.try_clone().expect("native session QMP reader"));
    packet(
        &mut stream,
        json!({"QMP": {
            "version": {"qemu": {"major": 11, "minor": 0, "micro": 0}},
            "capabilities": []
        }}),
    );
    let mut report = WireReport::default();
    loop {
        let mut line = String::new();
        if reader
            .read_line(&mut line)
            .expect("production native session command")
            == 0
        {
            break;
        }
        let request: Value = serde_json::from_str(&line).expect("production native session JSON");
        let id = request["id"]
            .as_u64()
            .expect("production numeric native session QMP id");
        let command = request["execute"]
            .as_str()
            .expect("production native session execute");
        report.commands.push(command.to_owned());
        match command {
            "qmp_capabilities" => {}
            "screendump" => {
                assert_eq!(request["arguments"]["format"], "png");
                let destination = PathBuf::from(
                    request["arguments"]["filename"]
                        .as_str()
                        .expect("production private capture path"),
                );
                assert_eq!(destination.parent(), Some(directory.as_path()));
                let png = captures
                    .pop_front()
                    .expect("no extra native session capture authorised");
                fs::write(destination, png)
                    .expect("retained PNG at production capture destination");
                report.captures += 1;
                if report.captures == 1 {
                    match revocation {
                        Revocation::None => {}
                        Revocation::Stop => control.stop(Some(&context), &cancel),
                        Revocation::ContextRoundTrip => {
                            let mut changed = context.clone();
                            changed.generation += 1;
                            control
                                .select(&changed, &cancel)
                                .expect("change active native context");
                            control
                                .select(&context, &cancel)
                                .expect("restore original native context");
                        }
                    }
                }
            }
            other => panic!("native observation must not send QMP operation {other}"),
        }
        packet(&mut stream, json!({"return": {}, "id": id}));
    }
    report.remaining = captures.len();
    report
}


/// Native return value and independently recorded capture/authority evidence.
struct Episode {
    /// Actual production adapter result, without a test-side replacement loop.
    result: Result<Entry, String>,
    /// All concrete local QMP commands and supplied capture count.
    wire: WireReport,
    /// Whether the last real capture was preserved for inspection on failure.
    retained_frame: bool,
    /// Qualified count retained with the actual paired original identities, if any.
    starting: Option<BoardCount>,
    /// Bound normal-series count; an unrelated challenge fraction cannot supply it.
    series: Option<BoardCount>,
    /// Final cancellation latch; no fixture helper clears it after adapter execution.
    cancelled: bool,
    /// Whether the production reservation still permits the original selected context.
    owns_context: bool,
}


/// Invoke one actual adapter method over isolated AF_UNIX and the real capture/PNG path.
fn run(operation: Operation, frames: Vec<Vec<u8>>, revocation: Revocation) -> Episode {
    let _capture_backend = TestCaptureBackend::legacy_qmp();
    let mut fixture = Fixture::new();
    if matches!(
        operation,
        Operation::CardEntry
            | Operation::StartingReadiness
            | Operation::NoChallengeAcquire
            | Operation::NoChallengeEntry
            | Operation::SingleBoardEntry
    ) {
        fixture.request.reconnaissance.profile = ReconProfileSource::BundledNativeV22;
    }
    if matches!(
        operation,
        Operation::NoChallengeAcquire | Operation::NoChallengeEntry
    ) {
        fixture.request.mode = WorkflowMode::Continuous;
        fixture.request.stop_condition = StopCondition::NoChallenge;
    }
    if matches!(operation, Operation::ColourEntry) {
        fixture.request.reconnaissance.profile = ReconProfileSource::BundledNativeV23;
    }
    let explicit_start = matches!(
        operation,
        Operation::NoChallengeEntry | Operation::SingleBoardEntry
    );
    if matches!(operation, Operation::SingleBoardEntry) {
        fixture.request.mode = WorkflowMode::SingleBoard;
    }
    let expected_entry = explicit_start.then(|| native(&frames[0]));
    if explicit_start {
        // Retire a previous failed/stopped run without changing strategy or context.
        let control = &fixture.events.latest_frame.reconnaissance;
        control.stop(Some(&fixture.context), &fixture.cancel);
        control.finish();
        control
            .queue_reconnaissance(&fixture.context, &fixture.cancel, || Ok(()))
            .unwrap();
        fixture.events.publish_diagnostic_frame(native(ZERO));
    }
    let listener = UnixListener::bind(&fixture.context.socket_path)
        .expect("isolated native session QMP socket");
    listener
        .set_nonblocking(true)
        .expect("bounded native session accept");
    let directory = fixture.directory.0.clone();
    let context = fixture.context.clone();
    let control = fixture.events.latest_frame.reconnaissance.clone();
    let cancel = Arc::clone(&fixture.cancel);
    let server = thread::spawn(move || {
        serve(
            listener,
            directory,
            frames.into(),
            control,
            context,
            cancel,
            revocation,
        )
    });
    let mut adapter = fixture.adapter();
    if explicit_start {
        adapter.latest = Some(native(ZERO));
    }
    let result = match operation {
        Operation::StartingReadiness => adapter.await_no_challenge_start().map(|_| Entry::Board {
            progress: None,
            standard_three: false,
        }),
        Operation::NoChallengeAcquire => adapter.acquire(2).map(|()| Entry::Board {
            progress: None,
            standard_three: false,
        }),
        Operation::BoundTransition(bound) | Operation::ClearTransition(bound) => {
            adapter.challenge_progress = Some(bound);
            if matches!(operation, Operation::ClearTransition(_)) {
                adapter.original_tableau =
                    Some(adapter.starting_board(&native(ZERO)).unwrap().tableau);
                adapter.distinct_start = true;
            }
            let result = adapter.transition();
            if result.is_err() {
                assert_eq!(
                    adapter.challenge_progress,
                    Some(bound),
                    "A refused fresh-board pair must retain its original challenge binding"
                );
            }
            result
        }
        Operation::NormalClearTransition => {
            let original = adapter.starting_board(&native(NORMAL_SERIES[0])).unwrap();
            adapter.original_tableau = Some(original.tableau);
            adapter.distinct_start = true;
            adapter.series = Some(BoardCount {
                completed: 0,
                target: 3,
            });
            let result = adapter.transition();
            if result.is_err() {
                assert_eq!(
                    adapter.series,
                    Some(BoardCount {
                        completed: 0,
                        target: 3
                    })
                );
            }
            assert!(adapter.restart_entry.is_none());
            result
        }
        Operation::Entry | Operation::CardEntry | Operation::ColourEntry => {
            adapter.observed_entry()
        }
        Operation::NoChallengeEntry | Operation::SingleBoardEntry => {
            adapter.entry(fixture.request.mode)
        }
        Operation::Transition => adapter.transition(),
    };
    let retained_frame = adapter.latest.is_some();
    if explicit_start && result.is_ok() {
        assert_eq!(
            adapter.latest.as_ref().unwrap().pixels,
            expected_entry.unwrap().pixels
        );
    }
    let starting = adapter.starting.as_ref().map(|witness| witness.progress);
    let series = adapter.series;
    let wire = server.join().expect("native session QMP server completion");
    assert_eq!(
        wire.commands.first().map(String::as_str),
        Some("qmp_capabilities")
    );
    assert!(
        wire.commands
            .iter()
            .skip(1)
            .all(|command| command == "screendump")
    );
    assert!(!fixture.request.reconnaissance.output_path.exists());
    Episode {
        result,
        wire,
        retained_frame,
        starting,
        series,
        cancelled: fixture.cancel.load(Ordering::Acquire),
        owns_context: fixture
            .events
            .latest_frame
            .reconnaissance
            .check_active(&fixture.context, &fixture.cancel)
            .is_ok(),
    }
}


/// A new reservation captures current pixels after failure without changing solving strategy.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn explicit_starts_capture_fresh_pixels_after_retired_run() {
    for operation in [Operation::NoChallengeEntry, Operation::SingleBoardEntry] {
        let episode = run(operation, vec![ONE.to_vec()], Revocation::None);
        assert_eq!(
            episode.result.unwrap(),
            Entry::Board {
                progress: None,
                standard_three: false
            }
        );
        assert_eq!(episode.wire.captures, 1);
        assert!(episode.owns_context && !episode.cancelled && episode.retained_frame);
        assert!(episode.starting.is_none() && episode.series.is_none());
    }
}


/// STOP and a context round trip during the fresh screenshot still revoke the new start.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn explicit_start_capture_preserves_stop_and_context_revocation() {
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let episode = run(Operation::NoChallengeEntry, vec![ONE.to_vec()], revocation);
        assert!(episode.result.unwrap_err().contains("revoked"));
        assert_eq!(episode.wire.captures, 1);
        assert!(episode.cancelled && !episode.owns_context);
    }
}


/// Actual supplied starting boards satisfy both whole-card and objective recognition.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_supplied_normal_and_challenge_entries_keep_their_actual_identity() {
    use crate::pyramid_progress::ChallengeKind;
    for (png, expected) in [
        (
            include_bytes!("../../../tests/fixtures/pyramid-objectives/fours-four-of-twelve.png")
                .as_slice(),
            Entry::Challenge(ChallengeProgress::new(ChallengeKind::Cards, 4, 12).unwrap()),
        ),
        (
            include_bytes!("../../../tests/fixtures/pyramid-objectives/boards-zero-of-two.png")
                .as_slice(),
            Entry::Challenge(ChallengeProgress::new(ChallengeKind::Boards, 0, 2).unwrap()),
        ),
        (
            include_bytes!(
                "../../../tests/fixtures/pyramid-objectives/normal-grandmaster-refused.png"
            )
            .as_slice(),
            Entry::Board {
                progress: Some(BoardCount {
                    completed: 0,
                    target: 3,
                }),
                standard_three: true,
            },
        ),
    ] {
        let episode = run(
            Operation::Entry,
            vec![png.to_vec(), png.to_vec()],
            Revocation::None,
        );
        assert_eq!(episode.result, Ok(expected));
        assert_eq!(episode.wire.captures, 2);
        assert_eq!(episode.wire.remaining, 0);
    }
}


/// Genuine previously unsupported Card names pass the actual paired native entry adapter.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_shared_cleared_sixes_and_nines_entries_keep_actual_card_targets() {
    for (png, target) in [
        (
            include_bytes!("../../../tests/fixtures/pyramid-card-cleared/sixes.png").as_slice(),
            12,
        ),
        (
            include_bytes!("../../../tests/fixtures/pyramid-card-cleared/nines.png").as_slice(),
            15,
        ),
    ] {
        let episode = run(
            Operation::CardEntry,
            vec![png.to_vec(), png.to_vec()],
            Revocation::None,
        );
        assert_eq!(
            episode.result,
            Ok(Entry::Challenge(
                ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Cards, 0, target)
                    .unwrap()
            ))
        );
        assert_eq!(episode.wire.captures, 2);
        assert_eq!(episode.wire.remaining, 0);
        assert!(episode.owns_context);
    }
}


/// Actual STOP/context revocation still refuses fresh generic Card observations without input.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_shared_cleared_card_entry_keeps_stop_and_context_revocation() {
    let png = include_bytes!("../../../tests/fixtures/pyramid-card-cleared/sixes.png");
    for revoke in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let episode = run(
            Operation::CardEntry,
            vec![png.to_vec(), png.to_vec()],
            revoke,
        );
        assert!(episode.result.is_err());
        assert!(!episode.owns_context);
        assert!(episode.wire.captures <= 2);
    }
}


/// A delayed common terminal retains its existing bound goal, while cold entry cannot invent it.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_bound_transition_accepts_common_terminal_without_fabricating_its_counter() {
    let bound =
        ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Score, 1280, 3000).unwrap();
    let mut frame = native(include_bytes!(
        "../../../tests/fixtures/pyramid-objectives/fours-complete.png"
    ));
    for y in 48..80 {
        for x in 814..1110 {
            let i = y * frame.stride + x * 4;
            frame.pixels[i..i + 3].fill(255);
        }
    }
    assert_eq!(
        crate::pyramid_progress::observe_challenge_complete(&frame),
        Ok(None)
    );
    let png = encode_png(&frame).unwrap();
    let episode = run(
        Operation::BoundTransition(bound),
        vec![png.clone(), png],
        Revocation::None,
    );
    let Entry::ChallengeComplete(proof) = episode.result.unwrap() else {
        panic!("bound common terminal must be distinct")
    };
    assert_eq!(proof.progress(), bound);
    assert!(!proof.progress().target_reached());
    assert!(
        proof
            .description()
            .contains("final numeric counter not read")
    );
    assert_eq!(episode.wire.captures, 2);
}


/// Literal native 0/3, 1/3 and 2/3 remain observations without standard-game identity.
#[test]
fn native_observe_reads_retained_complete_boards_and_literal_counts() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter();
    for (png, completed) in [(ZERO, 0_u8), (ONE, 1), (TWO, 2)] {
        let observation = adapter
            .observe(&native(png))
            .expect("actual native observation");
        assert!(
            matches!(observation, NativeObservation::Challenge { progress, .. } if progress.current == u32::from(completed) && progress.target.get() == 3 && progress.kind == crate::pyramid_progress::ChallengeKind::Boards)
        );
        let entry = paired_entry(&observation, &observation).expect("matching native observations");
        assert_eq!(
            entry,
            Entry::Challenge(
                ChallengeProgress::new(
                    crate::pyramid_progress::ChallengeKind::Boards,
                    u32::from(completed),
                    3
                )
                .unwrap()
            )
        );
        assert!(standard_progress(entry).is_err());
    }
}


/// Partial/unknown gameplay cannot qualify; a specific Pyramid end still needs a fresh target after restart.
#[test]
fn native_observe_preserves_partial_unknown_header_and_terminal_boundaries() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter();
    assert_eq!(
        crate::pyramid_progress::observe(&native(PARTIAL)).expect("partial real header"),
        Some(BoardCount {
            completed: 1,
            target: 3
        })
    );
    assert_eq!(
        adapter
            .observe(&native(PARTIAL))
            .expect("actual partial observation"),
        NativeObservation::Unknown
    );
    assert!(matches!(
        adapter
            .observe(&zero_board_without_header())
            .expect("unknown header"),
        NativeObservation::Board { progress: None, .. }
    ));
    let terminal = adapter
        .observe(&native(TERMINAL))
        .expect("actual terminal observation");
    assert_eq!(
        terminal,
        NativeObservation::Terminal(NativeTerminal::LevelUp)
    );
    assert_eq!(
        paired_entry(&terminal, &terminal),
        Some(Entry::Terminal {
            supported_pyramid: true
        })
    );
}


/// Two fresh identical real board/header captures supply facts and preserve the existing reservation.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_observed_entry_uses_two_real_captures_without_standard_identity() {
    let episode = run(
        Operation::Entry,
        vec![ZERO.to_vec(), ZERO.to_vec()],
        Revocation::None,
    );
    let entry = episode.result.expect("matching native entry pair");
    assert_eq!(
        entry,
        Entry::Challenge(
            ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Boards, 0, 3).unwrap()
        )
    );
    assert!(standard_progress(entry).is_err());
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 0);
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
}


/// The retained Classic entry is a bound untouched Boards challenge, with no acquisition input.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_classic_boards_zero_entry_recognises_all_cards_and_complete_requirement() {
    let bytes = include_bytes!("../../../tests/fixtures/pyramid-objectives/classic-green-zero.png");
    let episode = run(
        Operation::Entry,
        vec![bytes.to_vec(), bytes.to_vec()],
        Revocation::None,
    );
    assert_eq!(
        episode.result.unwrap(),
        Entry::Challenge(
            ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Boards, 0, 3).unwrap()
        )
    );
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 0);
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
}


/// A coloured gameplay surface vetoes overlapping New Game ink even with a restart permit.
/// The five black pixels are a labelled synthetic caption derivative of the actual Xbox image.
#[test]
fn native_colour_gameplay_veto_precedes_restart_caption_without_guest_io() {
    let fixture = Fixture::new();
    let mut frame = native(include_bytes!(
        "../../../tests/fixtures/pyramid-colour-independence/xbox-original.png"
    ));
    for x in 812..817 {
        let offset = 833 * frame.stride + x * 4;
        frame.pixels[offset..offset + 3].fill(0);
    }
    assert_eq!(crate::pyramid::is_gameplay_scene(&frame), Ok(false));
    assert_eq!(post_game::expected_pyramid_new_game(&frame), Ok(true));
    let reader = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    assert!(reader.native_gameplay_veto(&frame).unwrap());
    let mut adapter = fixture.adapter();
    adapter.vision = Some(reader);
    adapter.restart_entry = Some(
        post_game::PyramidRestartEntry::verified_third_completion(
            BoardCount {
                completed: 2,
                target: 3,
            },
            BoardCompletion::BoardClear,
        )
        .unwrap(),
    );
    let actual = adapter.observe(&frame).unwrap();
    assert!(
        matches!(
            actual,
            NativeObservation::Board { .. } | NativeObservation::Challenge { .. }
        ),
        "{actual:?}"
    );
    assert!(!fixture.context.socket_path.exists());
    assert!(!fixture.request.reconnaissance.output_path.exists());
}


/// Distinct original full boards cannot be paired merely because each has a recognised header.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_observed_entry_rejects_different_retained_boards() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter();
    let first = adapter
        .starting_board(&native(ZERO))
        .expect("complete retained zero board");
    let second = adapter
        .starting_board(&native(ONE))
        .expect("complete retained one board");
    assert_ne!(first, second);
    let episode = run(
        Operation::Entry,
        vec![ZERO.to_vec(), ONE.to_vec()],
        Revocation::None,
    );
    assert!(
        episode
            .result
            .expect_err("different real board pair")
            .contains("two matching")
    );
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 0);
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
}


/// Header-only change refuses the actual entry pair even when every card identity is unchanged.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_observed_entry_pairs_board_and_header_from_the_same_frames() {
    let changed = zero_board_with_one_header();
    let fixture = Fixture::new();
    let adapter = fixture.adapter();
    let original = adapter
        .starting_board(&native(ZERO))
        .expect("complete retained zero board");
    assert_eq!(Some(original), adapter.starting_board(&changed));
    assert_eq!(
        crate::pyramid_progress::observe(&changed).expect("real fraction derivative"),
        Some(BoardCount {
            completed: 1,
            target: 3
        })
    );
    let episode = run(
        Operation::Entry,
        vec![
            ZERO.to_vec(),
            encode_png(&changed).expect("header-only derivative PNG"),
        ],
        Revocation::None,
    );
    assert!(
        episode
            .result
            .expect_err("header mismatch must refuse")
            .contains("two matching")
    );
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 0);
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
}


/// An initial stale board cannot satisfy transition; two subsequent equal real boards are required.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_transition_waits_for_matching_consecutive_real_board_headers() {
    let episode = run(
        Operation::Transition,
        vec![ZERO.to_vec(), ONE.to_vec(), ONE.to_vec()],
        Revocation::None,
    );
    let entry = episode.result.expect("fresh equal transition pair");
    assert_eq!(
        entry,
        Entry::Challenge(
            ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Boards, 1, 3).unwrap()
        )
    );
    assert!(standard_progress(entry).is_err());
    assert_eq!(episode.wire.captures, 3);
    assert_eq!(episode.wire.remaining, 0);
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
}


/// An untouched native 1/3 pair contradicts the supplied 0/1 binding and cannot replace it.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_bound_transition_refuses_changed_denominator_without_replacing_binding() {
    let bound =
        ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Boards, 0, 1).unwrap();
    let actual =
        ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Boards, 1, 3).unwrap();
    assert_eq!(
        crate::pyramid_progress::observe_bound_challenge_progress(&native(ONE), bound),
        Ok(Some(actual))
    );
    let episode = run(
        Operation::BoundTransition(bound),
        vec![ONE.to_vec(), ONE.to_vec()],
        Revocation::None,
    );
    assert!(
        episode
            .result
            .expect_err("The actual 1/3 counter must contradict the bound 0/1 target")
            .contains("Challenge progress changed or regressed at the fresh board boundary")
    );
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 0);
    assert_eq!(
        episode.wire.commands,
        vec![
            "qmp_capabilities".to_owned(),
            "screendump".to_owned(),
            "screendump".to_owned(),
        ]
    );
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
}


/// An untouched native 0/3 pair regresses the supplied 1/3 binding and cannot replace it.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_bound_transition_refuses_regressed_numerator_without_replacing_binding() {
    let bound =
        ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Boards, 1, 3).unwrap();
    let actual =
        ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Boards, 0, 3).unwrap();
    assert_eq!(
        crate::pyramid_progress::observe_bound_challenge_progress(&native(ZERO), bound),
        Ok(Some(actual))
    );
    let episode = run(
        Operation::BoundTransition(bound),
        vec![ZERO.to_vec(), ZERO.to_vec()],
        Revocation::None,
    );
    assert!(
        episode
            .result
            .expect_err("The actual 0/3 counter must regress the bound 1/3 count")
            .contains("Challenge progress changed or regressed at the fresh board boundary")
    );
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 0);
    assert_eq!(
        episode.wire.commands,
        vec![
            "qmp_capabilities".to_owned(),
            "screendump".to_owned(),
            "screendump".to_owned(),
        ]
    );
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
}


/// Two identical full boards with unknown headers are skipped until a fresh positive pair arrives.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_transition_does_not_promote_unknown_header_to_zero() {
    let unknown = encode_png(&zero_board_without_header()).expect("unknown-header derivative PNG");
    let episode = run(
        Operation::Transition,
        vec![unknown.clone(), unknown, ZERO.to_vec(), ZERO.to_vec()],
        Revocation::None,
    );
    assert_eq!(
        episode.result.expect("positive pair after unknown headers"),
        Entry::Challenge(
            ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Boards, 0, 3).unwrap()
        )
    );
    assert_eq!(episode.wire.captures, 4);
    assert_eq!(episode.wire.remaining, 0);
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
}


/// Alternating real frames exhaust the production six-capture budget without accepting a pair.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_transition_refuses_at_its_actual_capture_budget() {
    let episode = run(
        Operation::Transition,
        vec![
            ZERO.to_vec(),
            ONE.to_vec(),
            ZERO.to_vec(),
            ONE.to_vec(),
            ZERO.to_vec(),
            ONE.to_vec(),
        ],
        Revocation::None,
    );
    assert!(
        episode
            .result
            .expect_err("no consecutive matching pair")
            .contains("within transition budget")
    );
    assert_eq!(episode.wire.captures, 6);
    assert_eq!(episode.wire.remaining, 0);
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
}


/// Actual entry capture retains first-frame diagnostics and stops before any second frame or input.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_entry_stop_and_context_round_trip_revoke_before_pair_completion() {
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let episode = run(
            Operation::Entry,
            vec![ZERO.to_vec(), ZERO.to_vec()],
            revocation,
        );
        assert!(
            episode
                .result
                .expect_err("revoked first entry capture")
                .contains("STOP or changed context revoked")
        );
        assert_eq!(episode.wire.captures, 1, "{revocation:?}");
        assert_eq!(episode.wire.remaining, 1, "{revocation:?}");
        assert!(episode.retained_frame && episode.cancelled && !episode.owns_context);
    }
}


/// Actual transition capture honours STOP and irreversible context loss at its post-I/O boundary.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_transition_stop_and_context_round_trip_revoke_before_pair_completion() {
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let episode = run(
            Operation::Transition,
            vec![ZERO.to_vec(), ZERO.to_vec()],
            revocation,
        );
        assert!(
            episode
                .result
                .expect_err("revoked first transition capture")
                .contains("STOP or changed context revoked")
        );
        assert_eq!(episode.wire.captures, 1, "{revocation:?}");
        assert_eq!(episode.wire.remaining, 1, "{revocation:?}");
        assert!(episode.retained_frame && episode.cancelled && !episode.owns_context);
    }
}


/// Continuous cannot enter the reconnaissance adapter using only a request or unknown facts.
#[test]
fn continuous_acquisition_without_qualified_witness_does_not_connect_or_publish() {
    let mut fixture = Fixture::new();
    fixture.request.mode = WorkflowMode::Continuous;
    let mut adapter = fixture.adapter();
    assert!(
        adapter
            .acquire(1)
            .expect_err("no starting witness")
            .contains("no qualified current starting-board witness")
    );
    assert!(adapter.output.is_none());
    assert!(adapter.starting.is_none());
    assert_eq!(
        std::fs::read_dir(&fixture.directory.0)
            .expect("fixture directory")
            .count(),
        0
    );
    assert!(
        fixture
            .events
            .latest_frame
            .reconnaissance
            .check_active(&fixture.context, &fixture.cancel)
            .is_ok(),
        "a local missing witness must not release or reacquire the parent reservation"
    );
}


/// The actual normal ordinal one qualifies only together with the full supported Pyramid deal.
#[test]
fn native_normal_ordinal_one_is_zero_completed_with_actual_starting_identities() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter();
    let observation = adapter
        .observe(&native(NORMAL_ONE))
        .expect("actual normal source");
    let entry = paired_entry(&observation, &observation).expect("equal native facts");
    let progress = standard_progress(entry).expect("literal normal header and full Pyramid board");
    assert_eq!(
        progress,
        BoardCount {
            completed: 0,
            target: 3
        }
    );
    let witness = starting_witness(entry, &observation).expect("qualified actual identities");
    assert_eq!(witness.progress, progress);
    assert!(matches!(witness.position.left, StockFace::Card(_)));
    assert_eq!(witness.position.right, StockFace::Empty);
}


/// All three original normal ordinals retain same-frame identities and starting witnesses.
#[test]
fn native_original_series_supports_mid_game_entry_without_a_local_counter() {
    for (ordinal, png) in NORMAL_SERIES.into_iter().enumerate() {
        let fixture = Fixture::new();
        let adapter = fixture.adapter();
        let observation = adapter
            .observe(&native(png))
            .expect("original full normal board");
        let entry = paired_entry(&observation, &observation).expect("identical native facts");
        let expected = BoardCount {
            completed: u8::try_from(ordinal).unwrap(),
            target: 3,
        };
        assert_eq!(standard_progress(entry), Ok(expected));
        let witness =
            starting_witness(entry, &observation).expect("actual complete starting identities");
        assert_eq!(witness.progress, expected);
        assert!(matches!(witness.position.left, StockFace::Card(_)));
        assert_eq!(witness.position.right, StockFace::Empty);
    }
}


/// Actual QMP entry pairs original ordinal two/three without sending any operation.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_mid_game_entry_pairs_original_second_and_third_boards_without_input() {
    for (ordinal, png) in NORMAL_SERIES.into_iter().enumerate().skip(1) {
        let episode = run(
            Operation::Entry,
            vec![png.to_vec(), png.to_vec()],
            Revocation::None,
        );
        let expected = BoardCount {
            completed: u8::try_from(ordinal).unwrap(),
            target: 3,
        };
        assert_eq!(
            standard_progress(episode.result.expect("native mid-game entry")),
            Ok(expected)
        );
        assert_eq!(episode.starting, Some(expected));
        assert_eq!(episode.series, Some(expected));
        assert_eq!(episode.wire.captures, 2);
        assert_eq!(episode.wire.remaining, 0);
        assert!(episode.owns_context && !episode.cancelled);
    }
}


/// Actual entry retains its qualified normal count and original identities after both captures.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_normal_entry_pair_retains_the_starting_witness_without_input() {
    let episode = run(
        Operation::Entry,
        vec![NORMAL_ONE.to_vec(), NORMAL_ONE.to_vec()],
        Revocation::None,
    );
    let expected = BoardCount {
        completed: 0,
        target: 3,
    };
    assert_eq!(
        standard_progress(episode.result.expect("normal fresh entry")),
        Ok(expected)
    );
    assert_eq!(episode.starting, Some(expected));
    assert_eq!(episode.series, Some(expected));
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 0);
    assert!(episode.owns_context && !episode.cancelled);
}


/// Draw only bounded caption ink on a deliberately synthetic non-gameplay menu.
///
/// These pixels test accepted control routing, never native button calibration,
/// normal-series identity, actual third completion or reward/score-skip evidence.
fn synthetic_restart_menu(control: Control) -> CapturedFrame {
    let mut frame = CapturedFrame {
        width: 1_920,
        height: 1_080,
        stride: 1_920 * 4,
        format: PixelFormat::Rgba8,
        pixels: vec![255; 1_920 * 1_080 * 4],
    };
    let bounds = PYRAMID_CONTROLS.spec(control).bounds;
    for x in bounds.x + 2..bounds.x + 7 {
        let offset = (bounds.y + 2) as usize * frame.stride + x as usize * 4;
        frame.pixels[offset..offset + 3].fill(0);
    }
    frame
}


/// A bare expected control does not establish Pyramid completion or restart authority.
#[test]
fn native_bare_new_game_is_unknown_and_cannot_restart_without_a_permit() {
    let fixture = Fixture::new();
    let mut adapter = fixture.adapter();
    let menu = synthetic_restart_menu(Control::NewGame);
    assert_eq!(post_game::expected_pyramid_new_game(&menu), Ok(true));
    assert_eq!(post_game::recognised_pyramid_terminal(&menu), Ok(false));
    assert_eq!(adapter.observe(&menu), Ok(NativeObservation::Unknown));
    adapter.latest = Some(menu);
    assert!(adapter.restart_entry.is_none());
    assert!(
        adapter
            .restart()
            .expect_err("menu grants no completion authority")
            .contains("lacks supported positive Pyramid terminal evidence")
    );
    assert!(adapter.starting.is_none() && adapter.series.is_none());
    assert!(
        !fixture.context.socket_path.exists(),
        "refusal precedes transport"
    );
    assert_eq!(fs::read_dir(&fixture.directory.0).unwrap().count(), 0);
    assert!(
        adapter.check().is_ok(),
        "refusal retains the parent's reservation"
    );
}


/// Initial production observation path; private test permits make no native ordinal-three claim.
#[derive(Clone, Copy, Debug)]
enum RestartEntry {
    /// Actual entry pair recognises retained specific Level Up, without old series counts.
    ColdLevelUp,
    /// Actual transition pair consumes stage facts under a test-supplied actual-clear permit.
    WitnessedNewGame,
    /// Actual reward pair under the same actual-third-clear test capability.
    WitnessedNormalReward,
    /// Newly retained small reward under the unchanged actual-third-clear capability.
    WitnessedSmallNormalReward,
}


/// Authority boundary at which the isolated peer invokes real STOP/context revocation.
#[derive(Clone, Copy, Debug)]
enum RestartBoundary {
    /// Preserve the existing authority through all three production connections.
    None,
    /// Revoke after the first fresh controller probe, before any button press.
    FirstProbe(Revocation),
    /// Revoke at a numbered later controller probe, before its input.
    LaterProbe(usize, Revocation),
    /// Revoke before ACKing the first capture in the fresh post-restart entry pair.
    FreshEntryCapture(Revocation),
}


/// Peer-observed restart traffic, distinct from the adapter's returned logical facts.
#[derive(Default)]
struct RestartWire {
    /// Actual connection count: initial pair, controller, then independent fresh entry pair.
    connections: usize,
    /// Numeric connection identity and command name in actual arrival order.
    commands: Vec<(usize, String)>,
    /// Captures consumed per connection; no script supplies effects without production reads.
    captures: [usize; 3],
    /// Fresh absolute-pointer probes observed on the controller connection.
    probes: usize,
    /// Mapped pointer positions at actual left-button DOWN events.
    downs: Vec<(i64, i64)>,
    /// Matching UP count, proving the peer fully acknowledged each delivered click.
    ups: usize,
    /// Unconsumed scripts remain visible when authority stops the production path.
    remaining: usize,
}


/// Apply the same irreversible reservation revocation used by existing native observation tests.
fn revoke_restart(
    revocation: Revocation,
    control: &pyramid_reconnaissance::ReconControl,
    context: &ControllerContext,
    cancel: &AtomicBool,
) {
    match revocation {
        Revocation::None => {}
        Revocation::Stop => control.stop(Some(context), cancel),
        Revocation::ContextRoundTrip => {
            let mut changed = context.clone();
            changed.generation += 1;
            control.select(&changed, cancel).unwrap();
            control.select(context, cancel).unwrap();
        }
    }
}


/// Serve one bounded connection; only the middle controller may probe or send typed clicks.
#[expect(
    clippy::too_many_arguments,
    reason = "isolated peer keeps scripts and real revocation identity explicit"
)]
fn serve_restart_connection(
    mut stream: UnixStream,
    connection: usize,
    directory: PathBuf,
    mut frames: VecDeque<Vec<u8>>,
    wire: Arc<Mutex<RestartWire>>,
    control: pyramid_reconnaissance::ReconControl,
    context: ControllerContext,
    cancel: Arc<AtomicBool>,
    boundary: RestartBoundary,
) {
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    packet(
        &mut stream,
        json!({"QMP": {
            "version": {"qemu": {"major": 11, "minor": 0, "micro": 0}},
            "capabilities": []
        }}),
    );
    let mut position = (0_i64, 0_i64);
    loop {
        let mut line = String::new();
        if reader
            .read_line(&mut line)
            .expect("restart production command")
            == 0
        {
            break;
        }
        let request: Value = serde_json::from_str(&line).unwrap();
        let id = request["id"].as_u64().expect("numeric restart QMP id");
        let command = request["execute"].as_str().unwrap();
        wire.lock()
            .unwrap()
            .commands
            .push((connection, command.to_owned()));
        let response = match command {
            "qmp_capabilities" => json!({}),
            "screendump" => {
                assert_eq!(request["arguments"]["format"], "png");
                let destination = PathBuf::from(request["arguments"]["filename"].as_str().unwrap());
                assert_eq!(destination.parent(), Some(directory.as_path()));
                fs::write(
                    destination,
                    frames.pop_front().expect("no extra restart capture"),
                )
                .unwrap();
                let mut report = wire.lock().unwrap();
                report.captures[connection] += 1;
                let first_fresh = connection == 2 && report.captures[connection] == 1;
                drop(report);
                if first_fresh && let RestartBoundary::FreshEntryCapture(revocation) = boundary {
                    revoke_restart(revocation, &control, &context, &cancel);
                }
                json!({})
            }
            "query-status" => {
                assert_eq!(connection, 1, "observation pairs cannot probe for input");
                json!({"running": true, "status": "running"})
            }
            "query-mice" => {
                assert_eq!(
                    connection, 1,
                    "only restart controls need pointer authority"
                );
                let mut report = wire.lock().unwrap();
                report.probes += 1;
                let probe_number = report.probes;
                let first_probe = probe_number == 1;
                drop(report);
                if first_probe && let RestartBoundary::FirstProbe(revocation) = boundary {
                    revoke_restart(revocation, &control, &context, &cancel);
                }
                if let RestartBoundary::LaterProbe(number, revocation) = boundary
                    && number == probe_number
                {
                    revoke_restart(revocation, &control, &context, &cancel);
                }
                json!([{"name": "Private absolute restart tablet", "index": 0, "current": true, "absolute": true}])
            }
            "input-send-event" => {
                assert_eq!(connection, 1, "entry observation must remain input-free");
                for event in request["arguments"]["events"].as_array().unwrap() {
                    match event["type"].as_str().unwrap() {
                        "abs" => match event["data"]["axis"].as_str().unwrap() {
                            "x" => position.0 = event["data"]["value"].as_i64().unwrap(),
                            "y" => position.1 = event["data"]["value"].as_i64().unwrap(),
                            axis => panic!("unexpected restart pointer axis {axis}"),
                        },
                        "btn" => {
                            assert_eq!(event["data"]["button"], "left");
                            let mut report = wire.lock().unwrap();
                            if event["data"]["down"] == true {
                                report.downs.push(position);
                            } else {
                                report.ups += 1;
                            }
                        }
                        kind => panic!("restart may not send key/Solver events: {kind}"),
                    }
                }
                json!({})
            }
            command => panic!("unexpected native restart command {command}"),
        };
        packet(&mut stream, json!({"return": response, "id": id}));
    }
    wire.lock().unwrap().remaining += frames.len();
}


/// Serve one monitor client until EOF before accepting the next, matching QEMU ownership.
#[expect(
    clippy::too_many_arguments,
    reason = "bounded local peer retains explicit authority and sequential ownership"
)]
fn serve_restart(
    listener: UnixListener,
    directory: PathBuf,
    mut scripts: VecDeque<VecDeque<Vec<u8>>>,
    wire: Arc<Mutex<RestartWire>>,
    control: pyramid_reconnaissance::ReconControl,
    context: ControllerContext,
    cancel: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    boundary: RestartBoundary,
) {
    let deadline = Instant::now() + Duration::from_secs(45);
    while !scripts.is_empty() && !finished.load(Ordering::Acquire) {
        let stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(
                    Instant::now() < deadline,
                    "native restart accept budget exhausted"
                );
                thread::sleep(Duration::from_millis(5));
                continue;
            }
            Err(error) => panic!("native restart accept: {error}"),
        };
        let connection = {
            let mut report = wire.lock().unwrap();
            let connection = report.connections;
            report.connections += 1;
            connection
        };
        let frames = scripts.pop_front().unwrap();
        serve_restart_connection(
            stream,
            connection,
            directory.clone(),
            frames,
            Arc::clone(&wire),
            control.clone(),
            context.clone(),
            Arc::clone(&cancel),
            boundary,
        );
    }
    wire.lock().unwrap().remaining += scripts.iter().map(VecDeque::len).sum::<usize>();
}


/// Real Native results plus retained witness/authority, separate from mocked QMP effects.
struct RestartEpisode {
    /// Actual restart return, before the caller's fresh-zero reconciliation.
    result: Result<Entry, String>,
    /// Independently recorded control and capture sequence.
    wire: RestartWire,
    /// Original identities retained by the actual final normal entry pair.
    starting: Option<BoardObservation>,
    /// Actual count retained from those same starting-board pixels.
    series: Option<BoardCount>,
    /// A private test permit is consumed even when the restart later fails.
    permit_consumed: bool,
    /// Actual last existing pixels, including a capture whose following authority check failed.
    latest: Option<CapturedFrame>,
    /// STOP/context latch after the final real boundary check.
    cancelled: bool,
    /// Whether the original reservation is still active without release or reacquisition.
    owns_context: bool,
}


/// Traverse production initial-pair, restart controller and final fresh-entry pair.
///
/// Witnessed tests inject only a private typed BoardClear permit and logical
/// third-board context. They do not claim that native ordinal-three pixels or an
/// execution have been observed. All newly returned normal-one pixels are genuine
/// retained originals. No test controller replaces Native or the restart driver.
fn restart_episode(
    entry: RestartEntry,
    final_png: &[u8],
    boundary: RestartBoundary,
) -> RestartEpisode {
    let _capture_backend = TestCaptureBackend::legacy_qmp();
    let fixture = Fixture::new();
    let menu = synthetic_restart_menu(Control::NewGame);
    let play = synthetic_restart_menu(Control::Play);
    let reward = if matches!(entry, RestartEntry::WitnessedSmallNormalReward) {
        include_bytes!("../../../tests/fixtures/pyramid-testing-156/normal-reward-small.png")
            .as_slice()
    } else {
        NORMAL_REWARD_147
    };
    let initial_png = match entry {
        RestartEntry::ColdLevelUp => TERMINAL.to_vec(),
        RestartEntry::WitnessedNewGame => encode_png(&menu).unwrap(),
        RestartEntry::WitnessedNormalReward | RestartEntry::WitnessedSmallNormalReward => {
            reward.to_vec()
        }
    };
    let mut controller = match entry {
        RestartEntry::ColdLevelUp => vec![TERMINAL.to_vec(), TERMINAL.to_vec()],
        RestartEntry::WitnessedNewGame => Vec::new(),
        RestartEntry::WitnessedNormalReward | RestartEntry::WitnessedSmallNormalReward => vec![
            reward.to_vec(),
            reward.to_vec(),
            TERMINAL.to_vec(),
            TERMINAL.to_vec(),
            reward.to_vec(),
            reward.to_vec(),
        ],
    };
    controller.extend([
        encode_png(&menu).unwrap(),
        encode_png(&play).unwrap(),
        NORMAL_ONE.to_vec(),
        NORMAL_ONE.to_vec(),
    ]);
    let scripts: VecDeque<VecDeque<Vec<u8>>> = vec![
        vec![initial_png.clone(), initial_png].into(),
        controller.into(),
        vec![final_png.to_vec(), final_png.to_vec()].into(),
    ]
    .into();
    let listener = UnixListener::bind(&fixture.context.socket_path).unwrap();
    listener.set_nonblocking(true).unwrap();
    let directory = fixture.directory.0.clone();
    let wire = Arc::new(Mutex::new(RestartWire::default()));
    let peer_wire = Arc::clone(&wire);
    let finished = Arc::new(AtomicBool::new(false));
    let peer_finished = Arc::clone(&finished);
    let control = fixture.events.latest_frame.reconnaissance.clone();
    let context = fixture.context.clone();
    let cancel = Arc::clone(&fixture.cancel);
    let peer = thread::spawn(move || {
        serve_restart(
            listener,
            directory,
            scripts,
            peer_wire,
            control,
            context,
            cancel,
            peer_finished,
            boundary,
        )
    });
    let mut adapter = fixture.adapter();
    let initial = match entry {
        RestartEntry::ColdLevelUp => adapter.entry(WorkflowMode::Continuous),
        RestartEntry::WitnessedNewGame
        | RestartEntry::WitnessedNormalReward
        | RestartEntry::WitnessedSmallNormalReward => {
            let progress = BoardCount {
                completed: 2,
                target: 3,
            };
            adapter.generation = 3;
            adapter.series = Some(progress);
            adapter.restart_entry = Some(
                post_game::PyramidRestartEntry::verified_third_completion(
                    progress,
                    BoardCompletion::BoardClear,
                )
                .unwrap(),
            );
            adapter.transition()
        }
    };
    let result = initial.and_then(|initial| {
        assert_eq!(
            initial,
            Entry::Terminal {
                supported_pyramid: true
            }
        );
        assert!(
            adapter.starting.is_none(),
            "terminal is not a fresh starting board"
        );
        adapter.restart()
    });
    let starting = adapter
        .starting
        .as_ref()
        .map(|witness| witness.position.clone());
    let series = adapter.series;
    let permit_consumed = adapter.restart_entry.is_none();
    let latest = adapter.latest.clone();
    finished.store(true, Ordering::Release);
    peer.join()
        .expect("native restart multi-connection peer completed");
    let wire = Arc::try_unwrap(wire)
        .ok()
        .expect("peer released wire evidence")
        .into_inner()
        .unwrap();
    assert!(!fixture.request.reconnaissance.output_path.exists());
    assert_eq!(
        fs::read_dir(&fixture.directory.0).unwrap().count(),
        1,
        "only the owned socket remains"
    );
    RestartEpisode {
        result,
        wire,
        starting,
        series,
        permit_consumed,
        latest,
        cancelled: fixture.cancel.load(Ordering::Acquire),
        owns_context: adapter.check().is_ok(),
    }
}


/// Checked native pixel mapping used only to inspect the actual wire button order.
fn restart_wire_point(point: PixelPoint) -> (i64, i64) {
    (
        i64::from(point.x) * 32_767 / 1_920,
        i64::from(point.y) * 32_767 / 1_080,
    )
}


/// Both restart entries end at actual normal-one identities, without Solver or score-skip clicks.
fn assert_normal_restart(episode: RestartEpisode, entry: RestartEntry) {
    let progress = BoardCount {
        completed: 0,
        target: 3,
    };
    assert_eq!(
        restarted_progress(episode.result.expect("fresh actual restart entry")),
        Ok(progress)
    );
    assert_eq!(episode.series, Some(progress));
    let fixture = Fixture::new();
    let expected = fixture
        .adapter()
        .starting_board(&native(NORMAL_ONE))
        .unwrap();
    assert_eq!(episode.starting, Some(expected));
    assert_eq!(episode.latest.unwrap().pixels, native(NORMAL_ONE).pixels);
    assert!(episode.permit_consumed && episode.owns_context && !episode.cancelled);
    assert_eq!(episode.wire.connections, 3);
    let (controls, captures): (&[Control], [usize; 3]) = match entry {
        RestartEntry::ColdLevelUp => (&[Control::Ok, Control::NewGame, Control::Play], [2, 6, 2]),
        RestartEntry::WitnessedNewGame => (&[Control::NewGame, Control::Play], [2, 4, 2]),
        RestartEntry::WitnessedNormalReward | RestartEntry::WitnessedSmallNormalReward => {
            unreachable!("reward flow has separate point assertions")
        }
    };
    let expected_points: Vec<_> = controls
        .iter()
        .map(|&control| {
            let frame = if control == Control::Ok {
                native(TERMINAL)
            } else {
                synthetic_restart_menu(control)
            };
            restart_wire_point(control_point(&frame, PYRAMID_CONTROLS.spec(control)).unwrap())
        })
        .collect();
    assert_eq!(episode.wire.downs, expected_points);
    assert_eq!(episode.wire.ups, controls.len());
    assert_eq!(episode.wire.probes, controls.len());
    assert_eq!(episode.wire.captures, captures);
    assert_eq!(episode.wire.remaining, 0);
    assert!(!episode.wire.downs.contains(&restart_wire_point(
        crate::parameters::SHARED_SOLVER_CONTROL.click_point
    )));
    for (connection, command) in &episode.wire.commands {
        if *connection != 1 {
            assert!(matches!(
                command.as_str(),
                "qmp_capabilities" | "screendump"
            ));
        }
    }
}


/// Cold specific Level Up needs no old-series assertion; the restarted normal-one pair supplies identity.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_cold_level_up_restart_proves_a_fresh_real_normal_one_entry() {
    assert_normal_restart(
        restart_episode(RestartEntry::ColdLevelUp, NORMAL_ONE, RestartBoundary::None),
        RestartEntry::ColdLevelUp,
    );
}


/// An explicit test-only actual-clear permit admits forward NewGame/Play stages into the real controller.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_witnessed_new_game_restart_proves_a_fresh_real_normal_one_entry() {
    assert_normal_restart(
        restart_episode(
            RestartEntry::WitnessedNewGame,
            NORMAL_ONE,
            RestartBoundary::None,
        ),
        RestartEntry::WitnessedNewGame,
    );
}


/// The compatibility controller's local zero cannot promote a fresh Board Challenge fraction to normal mode.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_restart_keeps_actual_challenge_entry_distinct_from_normal_one() {
    let episode = restart_episode(RestartEntry::WitnessedNewGame, ZERO, RestartBoundary::None);
    assert!(restarted_progress(episode.result.expect("actual challenge pair observed")).is_err());
    assert!(episode.starting.is_none() && episode.series.is_none());
    assert_eq!(episode.wire.captures, [2, 4, 2]);
    assert_eq!(episode.wire.downs.len(), 2);
    assert!(episode.permit_consumed && episode.owns_context && !episode.cancelled);
}


/// STOP/context revocation after the fresh control probe prevents even the first restart gesture.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_restart_checks_real_stop_and_context_before_control_input() {
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let episode = restart_episode(
            RestartEntry::WitnessedNewGame,
            NORMAL_ONE,
            RestartBoundary::FirstProbe(revocation),
        );
        assert!(
            episode
                .result
                .expect_err("fresh restart probe revoked authority")
                .contains("STOP")
        );
        assert_eq!(episode.wire.connections, 2);
        assert_eq!(episode.wire.captures, [2, 1, 0]);
        assert_eq!(episode.wire.probes, 1);
        assert!(episode.wire.downs.is_empty() && episode.wire.ups == 0);
        assert!(episode.permit_consumed && episode.cancelled && !episode.owns_context);
        assert!(episode.starting.is_none() && episode.series.is_none());
    }
}


/// Actual fresh-entry captures recheck authority and cannot retain a witness from an interrupted pair.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_restart_checks_real_stop_and_context_before_fresh_pair_completion() {
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let episode = restart_episode(
            RestartEntry::WitnessedNewGame,
            NORMAL_ONE,
            RestartBoundary::FreshEntryCapture(revocation),
        );
        assert!(
            episode
                .result
                .expect_err("new entry pair revoked")
                .contains("STOP or changed context revoked")
        );
        assert_eq!(episode.wire.connections, 3);
        assert_eq!(episode.wire.captures, [2, 4, 1]);
        assert_eq!(episode.wire.downs.len(), 2);
        assert_eq!(episode.wire.ups, 2);
        assert_eq!(episode.wire.remaining, 1);
        assert!(episode.permit_consumed && episode.cancelled && !episode.owns_context);
        assert!(episode.starting.is_none() && episode.series.is_none());
        assert_eq!(episode.latest.unwrap().pixels, native(NORMAL_ONE).pixels);
    }
}


/// The native session finaliser releases a genuine counting terminal without a socket or acquisition.
#[test]
fn native_challenge_counting_handoff_needs_no_qmp_or_next_acquisition() {
    let fixture = Fixture::new();
    let frame = native(include_bytes!(
        "../../../tests/fixtures/pyramid-objectives/fours-complete.png"
    ));
    let bound =
        ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Cards, 4, 12).unwrap();
    let proof = crate::pyramid_progress::observe_bound_challenge_complete(&frame, bound)
        .unwrap()
        .unwrap();
    assert!(
        !proof.operator_continue(),
        "the actual source is the counting stage"
    );
    let mut io = fixture.adapter();
    io.latest = Some(frame);
    io.challenge_progress = Some(bound);
    assert_eq!(io.finish_challenge(proof).unwrap(), proof);
    assert!(
        !fixture.context.socket_path.exists(),
        "the input-free finaliser never connects"
    );
    assert!(!fixture.request.reconnaissance.output_path.exists());
    assert!(io.output.is_none() && io.starting.is_none() && io.restart_entry.is_none());
}


/// Two actual mixed-legend Score captures use production entry without guest input.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_143_mixed_legend_entry_pairs_actual_score_and_preserves_revocation() {
    let bytes = include_bytes!("../../../tests/fixtures/pyramid-mixed-legend-143/score-entry.png");
    let episode = run(
        Operation::ColourEntry,
        vec![bytes.to_vec(), bytes.to_vec()],
        Revocation::None,
    );
    assert_eq!(
        episode.result.unwrap(),
        Entry::Challenge(
            ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Score, 775, 2200)
                .unwrap()
        )
    );
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 0);
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let episode = run(
            Operation::ColourEntry,
            vec![bytes.to_vec(), bytes.to_vec()],
            revocation,
        );
        assert!(
            episode
                .result
                .expect_err("revoked first actual entry capture")
                .contains("STOP or changed context revoked")
        );
        assert_eq!(episode.wire.captures, 1);
        assert_eq!(episode.wire.remaining, 1);
        assert!(episode.retained_frame && episode.cancelled && !episode.owns_context);
    }
}


/// A normal reward is a terminal stage only under an actual third-clear receipt.
#[test]
fn native_147_normal_reward_requires_actual_third_clear_receipt() {
    let fixture = Fixture::new();
    let mut adapter = fixture.adapter();
    let frame = native(include_bytes!(
        "../../../tests/fixtures/pyramid-normal-reward-147/normal-reward.png"
    ));
    assert_eq!(adapter.observe(&frame).unwrap(), NativeObservation::Unknown);
    adapter.restart_entry = Some(
        post_game::PyramidRestartEntry::verified_third_completion(
            BoardCount {
                completed: 2,
                target: 3,
            },
            BoardCompletion::BoardClear,
        )
        .unwrap(),
    );
    assert!(matches!(
        adapter.observe(&frame).unwrap(),
        NativeObservation::Terminal(_)
    ));
}


/// Unchanged reward-only incident PNG, not a synthetic terminal control image.
const NORMAL_REWARD_147: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-normal-reward-147/normal-reward.png");


/// Production Native transition, witnessed shared stages and independent fresh entry stay connected.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_147_normal_reward_full_restart_retains_fresh_board_without_solver() {
    let episode = restart_episode(
        RestartEntry::WitnessedNormalReward,
        NORMAL_ONE,
        RestartBoundary::None,
    );
    let progress = BoardCount {
        completed: 0,
        target: 3,
    };
    assert_eq!(restarted_progress(episode.result.unwrap()), Ok(progress));
    assert_eq!(episode.series, Some(progress));
    let expected = Fixture::new()
        .adapter()
        .starting_board(&native(NORMAL_ONE))
        .unwrap();
    assert_eq!(episode.starting, Some(expected));
    assert_eq!(episode.latest.unwrap().pixels, native(NORMAL_ONE).pixels);
    assert!(episode.permit_consumed && episode.owns_context && !episode.cancelled);
    assert_eq!(episode.wire.connections, 3);
    assert_eq!(episode.wire.captures, [2, 10, 2]);
    assert_eq!(episode.wire.probes, 5);
    assert_eq!(episode.wire.ups, 5);
    assert_eq!(episode.wire.remaining, 0);
    let skip = restart_wire_point(crate::parameters::SCORE_SKIP_CONTROL.click_point);
    let ok = restart_wire_point(
        control_point(&native(TERMINAL), PYRAMID_CONTROLS.spec(Control::Ok)).unwrap(),
    );
    let new_game = restart_wire_point(
        control_point(
            &synthetic_restart_menu(Control::NewGame),
            PYRAMID_CONTROLS.spec(Control::NewGame),
        )
        .unwrap(),
    );
    let play = restart_wire_point(
        control_point(
            &synthetic_restart_menu(Control::Play),
            PYRAMID_CONTROLS.spec(Control::Play),
        )
        .unwrap(),
    );
    assert_eq!(episode.wire.downs, vec![skip, ok, skip, new_game, play]);
    assert!(!episode.wire.downs.contains(&restart_wire_point(
        crate::parameters::SHARED_SOLVER_CONTROL.click_point
    )));
    for (connection, command) in &episode.wire.commands {
        if *connection != 1 {
            assert!(matches!(
                command.as_str(),
                "qmp_capabilities" | "screendump"
            ));
        }
    }
}


/// STOP or changed context after acknowledged OK prevents the next reward skip.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_147_post_ok_probe_revocation_sends_no_reward_press() {
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let episode = restart_episode(
            RestartEntry::WitnessedNormalReward,
            NORMAL_ONE,
            RestartBoundary::LaterProbe(3, revocation),
        );
        assert!(episode.result.is_err());
        assert_eq!(episode.wire.connections, 2);
        assert_eq!(episode.wire.captures, [2, 6, 0]);
        assert_eq!(episode.wire.probes, 3);
        assert_eq!(episode.wire.downs.len(), 2);
        assert_eq!(episode.wire.ups, 2);
        assert!(episode.starting.is_none());
    }
}


/// Actual native paired captures must distinguish the next tableau from the consumed original.
/// The private clear/ACK handoff is injected; this is not the missing original route trace.
#[test]
fn native_150_clear_transition_requires_distinct_original_tableau() {
    let bound =
        ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Boards, 0, 3).unwrap();
    let mut changed = native(ONE);
    let original = native(ZERO);
    // Two genuine board images with one explicitly composed counter/header, not a recorded transition.
    let header = zero_board_with_one_header();
    for y in 20..100 {
        let start = y * changed.stride + 784 * 4;
        let end = y * changed.stride + 1122 * 4;
        changed.pixels[start..end].copy_from_slice(&header.pixels[start..end]);
    }
    let vision = ReconVision::bundled_native_v17().unwrap();
    let ReconScene::Board(first) = vision.observe(&original).unwrap() else {
        panic!("original board")
    };
    let ReconScene::Board(next) = vision.observe(&changed).unwrap() else {
        panic!("composed next board")
    };
    assert_ne!(first.tableau, next.tableau);
    let next_png = encode_png(&changed).unwrap();
    let accepted = run(
        Operation::ClearTransition(bound),
        vec![next_png.clone(), next_png.clone()],
        Revocation::None,
    );
    assert_eq!(
        accepted.result,
        Ok(Entry::Challenge(
            ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Boards, 1, 3).unwrap()
        ))
    );
    assert_eq!(accepted.wire.captures, 2);
    let same = encode_png(&header).unwrap();
    let rejected = run(
        Operation::ClearTransition(bound),
        vec![same.clone(); 6],
        Revocation::None,
    );
    assert!(rejected.result.unwrap_err().contains("transition budget"));
    assert_eq!(rejected.wire.captures, 6);
    let delayed = run(
        Operation::ClearTransition(bound),
        vec![same.clone(), same, next_png.clone(), next_png.clone()],
        Revocation::None,
    );
    assert!(matches!(delayed.result, Ok(Entry::Challenge(_))));
    assert_eq!(delayed.wire.captures, 4);
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let refused = run(
            Operation::ClearTransition(bound),
            vec![next_png.clone(), next_png.clone()],
            revocation,
        );
        assert!(refused.result.is_err());
        assert_eq!(refused.wire.captures, 1);
    }
    let redeal = run(
        Operation::BoundTransition(bound),
        vec![ZERO.to_vec(), ZERO.to_vec()],
        Revocation::None,
    );
    assert_eq!(redeal.result, Ok(Entry::Challenge(bound)));
    assert_eq!(redeal.wire.captures, 2);
}



/// Retired normal ACKs observe distinct equal frames without input, restart receipts or a third clear.
#[test]
fn native_150_normal_clear_handoff_waits_for_distinct_pair() {
    let old = NORMAL_SERIES[0].to_vec();
    let next = NORMAL_SERIES[1].to_vec();
    let accepted = run(
        Operation::NormalClearTransition,
        vec![old.clone(), old.clone(), next.clone(), next.clone()],
        Revocation::None,
    );
    assert_eq!(
        accepted.result,
        Ok(Entry::Board {
            progress: Some(BoardCount {
                completed: 1,
                target: 3
            }),
            standard_three: true
        })
    );
    assert_eq!(accepted.wire.captures, 4);
    let stale = run(
        Operation::NormalClearTransition,
        vec![old; 6],
        Revocation::None,
    );
    assert!(stale.result.unwrap_err().contains("transition budget"));
    assert_eq!(stale.wire.captures, 6);
    let skipped = run(
        Operation::NormalClearTransition,
        vec![NORMAL_SERIES[2].to_vec(); 2],
        Revocation::None,
    );
    assert!(skipped.result.is_err());
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let stopped = run(
            Operation::NormalClearTransition,
            vec![next.clone(), next.clone()],
            revocation,
        );
        assert!(stopped.result.is_err());
        assert_eq!(stopped.wire.captures, 1);
    }
}


/// Actual saved frames establish a fresh v23 normal entry; STOP and context revocation stay input-free.
#[test]
fn native_153_manual_entry_binds_actual_pair_and_preserves_revocation() {
    let frames = vec![
        include_bytes!("../../../tests/fixtures/pyramid-normal-entry-153/automatic-entry.png")
            .to_vec(),
        include_bytes!("../../../tests/fixtures/pyramid-normal-entry-153/manual-entry.png")
            .to_vec(),
    ];
    let episode = run(Operation::ColourEntry, frames.clone(), Revocation::None);
    let expected = BoardCount {
        completed: 0,
        target: 3,
    };
    assert_eq!(
        standard_progress(episode.result.expect("actual fresh normal pair")),
        Ok(expected)
    );
    assert_eq!(episode.starting, Some(expected));
    assert_eq!(episode.series, Some(expected));
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 0);
    assert!(
        episode
            .wire
            .commands
            .iter()
            .all(|command| matches!(command.as_str(), "qmp_capabilities" | "screendump"))
    );
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let stopped = run(Operation::ColourEntry, frames.clone(), revocation);
        assert!(
            stopped
                .result
                .expect_err("fresh capture revocation")
                .contains("STOP or changed context revoked")
        );
        assert_eq!(stopped.wire.captures, 1);
        assert_eq!(stopped.wire.remaining, 1);
        assert!(
            stopped
                .wire
                .commands
                .iter()
                .all(|command| matches!(command.as_str(), "qmp_capabilities" | "screendump"))
        );
    }
}


mod testing_156;
