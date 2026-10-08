//! Regression tests for worker state, capture bounds and guarded transition contracts.

use super::post_game::{
    earlier_visible_post_game_target, known_post_game_stage, level_up_overrides_board_progress,
    new_game_visible_while_awaiting_level_up, post_game_click_status,
    require_post_game_click_retry_budget, require_post_game_observation_retry_budget,
    require_score_skip_click_budget, resolve_post_game_target, score_skip_retry_is_authorised,
};
use super::pyramid_execution::{
    PyramidRedealConfirmation, PyramidSolverRecoveryEvidence, pyramid_halo_continuation_phase,
    pyramid_solver_recovery_authorised,
};
use super::*;
use super::initial_recovery::{InitialRecoveryIo, recover_initial_target};
use crate::parameters::{
    LEVEL_UP_SHARED_BUTTON_BRIDGE, LEVEL_UP_SHARED_BUTTON_INTERIOR, POST_GAME_MAX_CLICK_ATTEMPTS,
    POST_GAME_TARGETS, PostGameControlVariant, PostGameStage, SCORE_SKIP_MAX_CLICK_ATTEMPTS,
};
use crate::pyramid::PyramidTargetKind;
use crate::strategy::{ControllerContext, SolvingStrategy};

use crate::parameters::{LEVEL_UP_CONTROL_VARIANTS, NEW_GAME_CONTROL_VARIANTS};


/// Construct a zero-filled RGBA frame for tests that supply their own scene evidence.
fn blank_frame(width: u32, height: u32) -> CapturedFrame {
    let stride = width as usize * 4;
    CapturedFrame {
        width,
        height,
        stride,
        format: crate::capture::PixelFormat::Rgba8,
        pixels: vec![0; stride * height as usize],
    }
}


/// Build a Computer Vision context for existing game-policy regression fixtures.
fn vision_context(socket_path: impl Into<PathBuf>, mode: GameMode) -> ControllerContext {
    ControllerContext::new(socket_path.into(), mode, SolvingStrategy::ComputerVision, 1)
}


/// Build a valid TriPeaks draw action using the supplied gold anchor.
fn draw_prediction(anchor: crate::geometry::PixelPoint) -> PredictedAction {
    PredictedAction::Action(
        GameMode::TriPeaks
            .profile()
            .bottom_action(0, anchor)
            .unwrap(),
    )
}


/// Check that compact UI labels distinguish draw input and each restart control.
#[test]
fn concise_status_labels_preserve_action_and_post_game_meaning() {
    assert_eq!(
        format_action_status(draw_prediction(crate::geometry::PixelPoint::new(842, 870))),
        "Draw card — TriPeaks stock DRAW"
    );
    assert_eq!(
        post_game_click_status(PostGameStage::LevelUpOk),
        "Click Level Up OK"
    );
    assert_eq!(
        post_game_click_status(PostGameStage::NewGame),
        "Start New Game"
    );
    assert_eq!(post_game_click_status(PostGameStage::Play), "Click Play");
    assert_eq!(
        post_game_click_status(PostGameStage::Solver),
        "Click Solver"
    );
}


/// Check that TriPeaks and Pyramid remain enabled for the shared input controller.
#[test]
fn both_game_modes_allow_the_shared_guarded_input_path() {
    assert_eq!(ensure_input_authorised(GameMode::TriPeaks), Ok(()));
    assert_eq!(ensure_input_authorised(GameMode::Pyramid), Ok(()));
}


/// Build the recognised Pyramid legend and optional MOVE HALO used by worker tests.
fn pyramid_move_frame(with_halo: bool) -> CapturedFrame {
    let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);


    for pixel in frame.pixels.as_chunks_mut::<4>().0 {
        pixel.copy_from_slice(&[20, 110, 60, 255]);
    }
    // Calibrated legend separators establish a positive Pyramid scene.


    for y in [169, 207, 245, 283, 321, 359] {


        for line in y..y + 2 {


            for x in 1_816..1_884 {
                let offset = line as usize * frame.stride + x as usize * 4;
                frame.pixels[offset..offset + 4].copy_from_slice(&[255; 4]);
            }
        }
    }


    if with_halo {
        let move_slot = pyramid::PYRAMID_TARGETS[0];
        let halo = pyramid::halo_probe(move_slot);


        for y in halo.y..halo.bottom() {


            for x in halo.x..halo.right() {
                let offset = y as usize * frame.stride + x as usize * 4;
                frame.pixels[offset..offset + 4].copy_from_slice(&[237, 207, 109, 255]);
            }
        }
    }
    frame
}


/// Build the recognised TriPeaks felt and optional stock HALO used by worker tests.
fn tripeaks_stock_frame(with_halo: bool) -> CapturedFrame {
    let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
    let felt = crate::parameters::GAMEPLAY_FELT_PROBE_BOUNDS;


    for y in felt.y..felt.bottom() {


        for x in felt.x..felt.right() {
            let offset = y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 4].copy_from_slice(&[20, 140, 60, 255]);
        }
    }


    if with_halo {
        let bounds = crate::parameters::STOCK_HALO_SCAN_BOUNDS;


        for x in bounds.x..bounds.x + crate::parameters::HALO_GOLD_RUN_MIN + 1 {
            let offset = bounds.y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 3]
                .copy_from_slice(&crate::parameters::GOLD_RGB_CANDIDATES[0]);
        }
    }
    frame
}


/// Add positive card-face evidence without inventing occupancy from a row hint.
fn initial_recovery_frame(mode: GameMode, with_halo: bool) -> CapturedFrame {
    let mut frame = match mode {
        GameMode::TriPeaks => tripeaks_stock_frame(with_halo),
        GameMode::Pyramid => pyramid_move_frame(with_halo),
        _ => panic!("startup fixture belongs to the shared controller"),
    };
    let face = match mode {
        GameMode::TriPeaks => {
            let probe = mode.profile().tableau_rows.last().unwrap().face_probe_bounds;
            PixelRect::new(probe.x, probe.y, crate::parameters::FACE_UP_WHITE_BLOCK_MIN_WIDTH, probe.height)
        }
        GameMode::Pyramid => {
            let slot = pyramid::PYRAMID_TARGETS.iter().find(|slot|
                slot.kind == PyramidTargetKind::Card { row: 7, column: 1 }
            ).unwrap();
            PixelRect::new(slot.bounds.x + 40, slot.bounds.y + 178, 60, 4)
        }
        _ => unreachable!(),
    };


    for y in face.y..face.bottom() {


        for x in face.x..face.right() {
            let offset = y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 4].copy_from_slice(&[255; 4]);
        }
    }
    frame
}


/// Injectable I/O records real startup order while classifying each supplied frame in production.
struct StartupTestIo {
    frames: std::collections::VecDeque<CapturedFrame>,
    scan_state: TableauScanState,
    trace: Vec<&'static str>,
    captures: usize,
    waits: usize,
    solver_attempts: usize,
    stopped: bool,
    stop_after_capture: Option<usize>,
    stop_on_wait: bool,
    stop_after_solver: bool,
    solver_error: bool,
    prediction_override: Option<PredictedAction>,
}


impl StartupTestIo {
    fn new(mode: GameMode, frames: Vec<CapturedFrame>) -> Self {
        Self {
            frames: frames.into(), scan_state: TableauScanState::for_mode(mode), trace: Vec::new(),
            captures: 0, waits: 0, solver_attempts: 0, stopped: false,
            stop_after_capture: None, stop_on_wait: false, stop_after_solver: false,
            solver_error: false, prediction_override: None,
        }
    }
}


impl InitialRecoveryIo for StartupTestIo {
    fn stopped(&self) -> bool { self.stopped }


    fn capture(&mut self) -> Result<FrameObservation, String> {
        self.trace.push("capture");
        self.captures += 1;
        let frame = self.frames.pop_front().ok_or_else(|| "injected capture failed".to_owned())?;
        let (mut observation, _) = analyse_captured_frame(frame, &self.scan_state)?;


        if let Some(prediction) = self.prediction_override { observation.prediction = prediction; }


        if self.stop_after_capture == Some(self.captures) { self.stopped = true; }
        Ok(observation)
    }


    fn wait(&mut self, delayed_round: usize, after_solver: bool) -> Result<(), String> {
        assert!((1..=3).contains(&delayed_round));
        assert_eq!(after_solver, self.solver_attempts == 1);
        self.trace.push("wait");
        self.waits += 1;


        if self.stop_on_wait {
            self.stopped = true;
            Err("injected STOP during wait".to_owned())
        } else { Ok(()) }
    }


    fn refresh_solver(&mut self) -> Result<(), String> {
        self.trace.push("solver");
        self.solver_attempts += 1;
        assert_eq!(self.solver_attempts, 1, "uncertain input must never be retried");


        if self.stop_after_solver { self.stopped = true; }


        if self.solver_error { Err("injected uncertain Solver delivery".to_owned()) } else { Ok(()) }
    }
}


/// A target in the initial or delayed fresh frame needs no Solver setup in either mode.
#[test]
fn initial_no_halo_request_adopts_fresh_or_delayed_canonical_target() {


    for mode in [GameMode::TriPeaks, GameMode::Pyramid] {


        for no_halo_frames in 0..=3 {
            let mut frames: Vec<_> = (0..no_halo_frames).map(|_| initial_recovery_frame(mode, false)).collect();
            frames.push(initial_recovery_frame(mode, true));
            let mut io = StartupTestIo::new(mode, frames);
            let (observation, plan) = recover_initial_target(mode, &mut io)
                .unwrap_or_else(|error| panic!("{}", error.message));
            assert!(observation.gameplay_scene);
            assert_eq!(plan.before(), observation.prediction);
            assert_eq!(plan.input().action().target.mode(), mode);
            assert_eq!(plan.input().operation(), InputOperation::PressDrawKey);
            assert_eq!(io.captures, no_halo_frames + 1);
            assert_eq!(io.waits, no_halo_frames);
            assert_eq!(io.solver_attempts, 0);
        }
    }
}


/// One Solver setup precedes an immediate capture; it returns one gameplay plan without spending it.
#[test]
fn initial_no_halo_solver_setup_then_fresh_target_preserves_step_once_budget() {


    for mode in [GameMode::TriPeaks, GameMode::Pyramid] {
        let settings = StepRunSettings::new(crate::parameters::AnimationSettleDelays::default(), STEP_ONCE_ACTIONS);
        let mut frames: Vec<_> = (0..4).map(|_| initial_recovery_frame(mode, false)).collect();
        frames.push(initial_recovery_frame(mode, true));
        let mut io = StartupTestIo::new(mode, frames);
        let (_, plan) = recover_initial_target(mode, &mut io)
            .unwrap_or_else(|error| panic!("{}", error.message));
        assert_eq!(io.trace, ["capture", "wait", "capture", "wait", "capture", "wait", "capture", "solver", "capture"]);
        assert_eq!(io.solver_attempts, 1);
        assert_eq!(settings.operation_limit(), 1);
        assert_eq!(plan.input().operation(), InputOperation::PressDrawKey);
        // Startup emits no Draw/card action; the unchanged executor receives exactly one plan.
        assert_eq!(plan.input().qmp_command_count(), 2);
        // Obtaining a HALO does not relax the existing effect requirement.
        assert!(verify_post_action(&plan, plan.before(), false).is_err());
    }
}


/// Persistent absence exhausts exactly eight captures, six delays and one consumed setup.
#[test]
fn initial_no_halo_absence_after_solver_stops_at_exact_budget() {


    for mode in [GameMode::TriPeaks, GameMode::Pyramid] {
        let frames = (0..8).map(|_| initial_recovery_frame(mode, false)).collect();
        let mut io = StartupTestIo::new(mode, frames);
        let failure = recover_initial_target(mode, &mut io).err().expect("missing target must stop");
        assert!(failure.solver_reserved);
        assert!(failure.observation.is_some());
        assert_eq!(failure.state, WorkerState::Uncertain);
        assert!(failure.message.contains("gameplay input sent: 0"));
        assert_eq!((io.captures, io.waits, io.solver_attempts), (8, 6, 1));
        assert!(io.frames.is_empty());
    }
}


/// Empty, unsupported and unknown Pyramid occupancy cannot enable Solver by row-mask inference.
#[test]
fn initial_no_halo_solver_requires_latest_positive_nonempty_scene() {


    for mode in [GameMode::TriPeaks, GameMode::Pyramid] {
        let empty = match mode {
            GameMode::TriPeaks => tripeaks_stock_frame(false),
            GameMode::Pyramid => pyramid_move_frame(false),
            _ => unreachable!(),
        };
        let mut unknown = empty.clone();


        if mode == GameMode::Pyramid {
            let slot = pyramid::PYRAMID_TARGETS.iter().find(|slot|
                slot.kind == PyramidTargetKind::Card { row: 7, column: 1 }
            ).unwrap();


            for y in slot.bounds.y + 178..slot.bounds.y + 182 {


                for x in slot.bounds.x + 40..slot.bounds.x + 100 {
                    let offset = y as usize * unknown.stride + x as usize * 4;
                    unknown.pixels[offset..offset + 4].copy_from_slice(&[100, 100, 100, 255]);
                }
            }
            let (observation, _) = analyse_captured_frame(unknown.clone(), &TableauScanState::for_mode(mode)).unwrap();
            assert!(observation.observed_rows.is_some(), "unknown occupancy gives a row hint, not positive card authority");
            assert!(!pyramid::has_visible_tableau_card(&unknown).unwrap());
        }


        for refused in [empty, unknown, blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT)] {
            // Prior occupied frames cannot lend authority to the latest empty/unknown/dialog frame.
            let mut frames = vec![initial_recovery_frame(mode, false); 3];
            frames.push(refused);
            let mut io = StartupTestIo::new(mode, frames);
            let failure = recover_initial_target(mode, &mut io).err().expect("no positive card evidence");
            assert!(!failure.solver_reserved);
            assert!(failure.observation.is_some());
            assert_eq!((io.captures, io.waits, io.solver_attempts), (4, 3, 0));
        }
    }
}


/// Cancellation at each startup boundary retains the latest pixels and permits no later input.
#[test]
fn initial_no_halo_stop_before_during_or_after_capture_and_solver_is_final() {


    for mode in [GameMode::TriPeaks, GameMode::Pyramid] {


        for boundary in 0..4 {
            let mut io = StartupTestIo::new(mode, vec![initial_recovery_frame(mode, false); 8]);


            match boundary {
                0 => io.stopped = true,
                1 => io.stop_on_wait = true,
                2 => io.stop_after_capture = Some(1),
                3 => io.stop_after_solver = true,
                _ => unreachable!(),
            }
            let failure = recover_initial_target(mode, &mut io).err().expect("STOP must terminate");
            assert_eq!(failure.state, WorkerState::Ready);
            let counts = match boundary { 0 => (0, 0, 0), 1 => (1, 1, 0), 2 => (1, 0, 0), 3 => (4, 3, 1), _ => unreachable!() };
            assert_eq!((io.captures, io.waits, io.solver_attempts), counts);
            assert_eq!(failure.observation.is_some(), boundary != 0);
            assert_eq!(failure.solver_reserved, boundary == 3);
        }
    }
}


/// Uncertain Solver acknowledgement consumes the reservation and cannot trigger another capture/input.
#[test]
fn initial_no_halo_uncertain_solver_delivery_is_not_retried() {


    for mode in [GameMode::TriPeaks, GameMode::Pyramid] {
        let mut io = StartupTestIo::new(mode, vec![initial_recovery_frame(mode, false); 8]);
        io.solver_error = true;
        let failure = recover_initial_target(mode, &mut io).err().expect("uncertain delivery must stop");
        assert!(failure.solver_reserved);
        assert!(failure.message.contains("uncertain Solver delivery"));
        assert!(failure.observation.is_some());
        assert_eq!((io.captures, io.waits, io.solver_attempts), (4, 3, 1));
        assert_eq!(io.trace.last(), Some(&"solver"));
    }
}


/// Forged inputs, foreign targets and ambiguous predictions fail before any setup or gameplay input.
#[test]
fn initial_no_halo_fresh_target_must_be_canonical_and_unique() {
    let mut forged = GameMode::TriPeaks.profile().bottom_action(0, crate::geometry::PixelPoint::new(842, 870)).unwrap();
    forged.specification.operation = InputOperation::Click(crate::geometry::PixelPoint::new(10, 10));


    for mode in [GameMode::TriPeaks, GameMode::Pyramid] {


        for prediction in [
            PredictedAction::Action(forged),
            PredictedAction::Action(crate::klondike::canonical_action(crate::klondike::KlondikeTarget::Draw).unwrap()),
            PredictedAction::Ambiguous { highlight_count: 2 },
            PredictedAction::CalibrationOnly { mode },
        ] {
            let mut io = StartupTestIo::new(mode, vec![initial_recovery_frame(mode, true)]);
            io.prediction_override = Some(prediction);
            let failure = recover_initial_target(mode, &mut io).err().expect("invalid target cannot authorise input");
            assert!(!failure.solver_reserved);
            assert_eq!((io.captures, io.waits, io.solver_attempts), (1, 0, 0));
        }
    }
}


/// Explicit Single Step requests queue NoHighlight as recovery authority while freezing a one-action limit.
#[test]
fn initial_no_halo_step_once_request_reaches_worker_with_one_gameplay_slot() {


    for mode in [GameMode::TriPeaks, GameMode::Pyramid] {
        let (command_tx, command_rx) = mpsc::channel();
        let (_events, event_rx) = mpsc::channel();
        let cancel_requested = Arc::new(AtomicBool::new(true));
        let handle = WorkerHandle {
            command_tx, event_rx, latest_frame: LatestFrameSlot::default(), join: None,
            cancel_requested: Arc::clone(&cancel_requested),
        };
        handle.execute_steps(
            vision_context("/nonexistent-startup-test/qmp.sock", mode), PredictedAction::NoHighlight,
            StepRunSettings::new(crate::parameters::AnimationSettleDelays::default(), STEP_ONCE_ACTIONS),
        ).unwrap();
        assert!(!cancel_requested.load(Ordering::Acquire));


        match command_rx.try_recv().expect("explicit request is queued") {
            WorkerCommand::ExecuteSteps { context, approved_prediction, settings } => {
                assert_eq!(context.game_mode, mode);
                assert_eq!(approved_prediction, PredictedAction::NoHighlight);
                assert_eq!(settings.operation_limit(), 1);
            }
            _ => panic!("only execution was requested"),
        }
        assert!(matches!(command_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }
}


/// Preparation retains captured pixels even when Computer Vision would approve a HALO.
#[test]
fn preparation_frame_has_no_prediction_from_an_actionable_pyramid_halo() {
    let frame = pyramid_move_frame(true);
    let vision = pyramid::analyse(&frame, &pyramid::PyramidBoardState::new())
        .expect("recognised Pyramid fixture");
    assert!(matches!(vision.prediction,
        PredictedAction::Action(action) if action.target == ActionTarget::Pyramid(PyramidTargetKind::Move)));
    let original_pixels = frame.pixels.clone();
    assert_eq!(preparation_prediction(&frame), Ok(None));
    assert_eq!(frame.pixels, original_pixels);
}


/// Preparation keeps the calibrated frame contract and rejects unsafe pixel storage.
#[test]
fn preparation_frame_rejects_non_native_or_malformed_capture() {
    assert!(preparation_prediction(&blank_frame(1_280, 720)).is_err());
    let mut truncated = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
    truncated.pixels.truncate(16);
    assert!(preparation_prediction(&truncated).is_err());
    let mut narrow_stride = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
    narrow_stride.stride = 1;
    assert!(preparation_prediction(&narrow_stride).is_err());
}


/// A preparation run is refused before clearing STOP or queueing any QMP work.
#[test]
fn preparation_execute_request_preserves_stop_and_never_reaches_worker_queue() {
    let (command_tx, command_rx) = mpsc::channel();
    let (_event_tx, event_rx) = mpsc::channel();
    let cancel_requested = Arc::new(AtomicBool::new(true));
    let handle = WorkerHandle {
        command_tx,
        event_rx,
        latest_frame: LatestFrameSlot::default(),
        join: None,
        cancel_requested: Arc::clone(&cancel_requested),
    };
    let context = ControllerContext::new(
        PathBuf::from("/nonexistent-preparation/qmp.sock"),
        GameMode::Pyramid,
        SolvingStrategy::ShortestPath,
        9,
    );
    let prediction = PredictedAction::Action(
        pyramid::action_for_kind(PyramidTargetKind::Move).expect("canonical Pyramid Move"),
    );
    let error = handle.execute_steps(
        context,
        prediction,
        StepRunSettings::new(crate::parameters::AnimationSettleDelays::default(), 1),
    ).expect_err("preparation cannot execute input");
    assert!(error.contains("preparation") || error.contains("route"));
    assert!(cancel_requested.load(Ordering::Acquire));
    assert!(matches!(command_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
}


/// The worker independently rejects a preparation execution command before opening QMP.
#[test]
fn preparation_execution_command_has_no_connection_capture_or_input_events() {
    let (command_tx, command_rx) = mpsc::channel();
    let (events, receiver) = mpsc::channel();
    let latest_frame = LatestFrameSlot::default();
    let sink = WorkerEventSink {
        events,
        latest_frame: latest_frame.clone(),
        capture_context: Mutex::new(None),
    };
    let context = ControllerContext::new(
        PathBuf::from("/nonexistent-preparation/qmp.sock"),
        GameMode::Pyramid,
        SolvingStrategy::ShortestPath,
        11,
    );
    let prediction = PredictedAction::Action(
        pyramid::action_for_kind(PyramidTargetKind::Move).expect("canonical Pyramid Move"),
    );
    command_tx.send(WorkerCommand::ExecuteSteps {
        context: context.clone(),
        approved_prediction: prediction,
        settings: Box::new(StepRunSettings::new(
            crate::parameters::AnimationSettleDelays::default(), 1,
        )),
    }).expect("queue rejected execution");
    command_tx.send(WorkerCommand::Shutdown).expect("queue shutdown");
    let cancel_requested = Arc::new(AtomicBool::new(true));
    run_worker(command_rx, sink, Arc::clone(&cancel_requested));
    let notifications: Vec<_> = receiver.try_iter().collect();
    assert!(!notifications.is_empty());
    assert!(notifications.iter().all(|notification| notification.context == Some(context.clone())));
    assert!(!notifications.iter().any(|notification| matches!(notification.event,
        WorkerEvent::State(WorkerState::Connecting | WorkerState::Capturing | WorkerState::Acting)
        | WorkerEvent::ActionCompleted { .. }
        | WorkerEvent::RunCompleted { .. }
        | WorkerEvent::GameCompleted
        | WorkerEvent::BoardProgress { .. })));
    assert!(latest_frame.take().is_none());
    assert!(cancel_requested.load(Ordering::Acquire));
}


/// Free Cell blank frames cannot inherit a progress bar or another mode's scene authority.
#[test]
fn freecell_native_capture_rejects_unknown_scene_and_malformed_storage() {
    let state = TableauScanState::for_mode(GameMode::FreeCell);
    let (observation, _) = analyse_captured_frame(
        blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT),
        &state,
    ).expect("native diagnostic frame");
    assert_eq!(observation.prediction, PredictedAction::NoHighlight);
    assert!(!observation.gameplay_scene);
    assert_eq!(observation.observed_rows, None);
    assert_eq!(observation.game_progress, None);
    assert!(analyse_captured_frame(blank_frame(1_280, 720), &state).is_err());
    let mut malformed = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
    malformed.pixels.truncate(16);
    assert!(analyse_captured_frame(malformed, &state).is_err());
}


/// A stale foreign preview cannot bypass Free Cell's fresh QMP acquisition boundary.
#[test]
fn freecell_execution_request_cannot_execute_a_foreign_preview_without_capture() {


    for operation_limit in [0, 1] {
        let (events, receiver) = mpsc::channel();
        let sink = WorkerEventSink {
            events,
            latest_frame: LatestFrameSlot::default(),
            capture_context: Mutex::new(None),
        };
        let mut state = TableauScanState::for_mode(GameMode::FreeCell);
        let mut completed_boards = 0;
        run_execute_steps(
            PathBuf::from("/nonexistent-freecell-calibration/qmp.sock"),
            draw_prediction(crate::geometry::PixelPoint::new(842, 870)),
            StepRunSettings::new(crate::parameters::AnimationSettleDelays::default(), operation_limit),
            &mut state,
            &mut completed_boards,
            &sink,
            &AtomicBool::new(false),
        );
        let notifications: Vec<_> = receiver.try_iter().map(|notification| notification.event).collect();
        assert!(!notifications.iter().any(|event| matches!(event,
            WorkerEvent::ActionCompleted { .. } | WorkerEvent::GameCompleted | WorkerEvent::FreeCellGameCompleted)));
        assert_eq!(completed_boards, 0);
        assert!(sink.latest_frame.take().is_none());
    }
}


/// Check that an unrecognised Pyramid image cannot create an executable plan.
#[test]
fn unknown_pyramid_capture_grants_no_action_authority() {
    let scan_state = TableauScanState::for_mode(GameMode::Pyramid);
    let (observation, _) = analyse_captured_frame(
        blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT),
        &scan_state,
    )
    .unwrap();

    assert_eq!(observation.prediction, PredictedAction::NoHighlight);
    assert_eq!(observation.observed_rows, None);
    assert!(!observation.gameplay_scene);
    assert!(plan_step(observation.prediction).is_err());
}


/// Check that Pyramid rejects captures outside the calibrated 1920×1080 contract.
#[test]
fn pyramid_gameplay_requires_the_exact_frame_contract() {
    let scan_state = TableauScanState::for_mode(GameMode::Pyramid);
    let error = analyse_captured_frame(blank_frame(1_280, 720), &scan_state)
        .err()
        .unwrap();

    assert_eq!(
        error,
        "gameplay-scene analysis failed: captured frame must be exactly 1920x1080, got 1280x720"
    );
}


/// Check every transition-phase combination blocks the ordinary repeated-pair exception.
#[test]
fn repeated_pile_halo_cannot_bypass_completion_or_redeal() {
    assert!(pyramid_halo_continuation_phase(false, false, false));


    for phase in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
        (true, true, false),
        (true, false, true),
        (false, true, true),
        (true, true, true),
    ] {
        assert!(!pyramid_halo_continuation_phase(phase.0, phase.1, phase.2));
    }
}


/// Check that animated full progress cannot confirm a redeal before two matching fresh observations.
#[test]
fn pyramid_redeal_waits_through_progress_animation_then_requires_two_fresh_matches() {
    let mut confirmation = PyramidRedealConfirmation::default();
    // Regression sequence: the apex was clicked once; a redeal is already
    // visible while its progress bar still looks full. No immediate error
    // or new-board authority comes from those transitional captures.
    assert!(!confirmation.observe(true, Some(GameProgress::GameComplete)));
    assert!(!confirmation.observe(true, Some(GameProgress::GameComplete)));
    assert!(!confirmation.observe(true, Some(GameProgress::AnotherBoard)));
    assert!(confirmation.observe(true, Some(GameProgress::AnotherBoard)));
}


/// Check that missing cards or contradictory progress reset the two-frame redeal confirmation.
#[test]
fn pyramid_redeal_requires_consecutive_positive_cards_and_progress() {
    let mut confirmation = PyramidRedealConfirmation::default();
    assert!(!confirmation.observe(true, Some(GameProgress::AnotherBoard)));


    // A modal, missing reading, or contradictory bar breaks confirmation.
    for (cards, progress) in [
        (false, Some(GameProgress::AnotherBoard)),
        (true, None),
        (true, Some(GameProgress::GameComplete)),
    ] {
        assert!(!confirmation.observe(cards, progress));
        assert!(!confirmation.observe(true, Some(GameProgress::AnotherBoard)));
    }
    assert!(confirmation.observe(true, Some(GameProgress::AnotherBoard)));


    // Missing halo alone is never board evidence, however many captures.
    for _ in 0..POST_GAME_MAX_OBSERVATION_ROUNDS {
        assert!(!confirmation.observe(false, Some(GameProgress::AnotherBoard)));
    }
}


/// Check every guard needed before a single new-board Solver recovery click.
#[test]
fn pyramid_solver_recovery_requires_a_verified_redeal_and_three_observations() {
    let accepted = PyramidSolverRecoveryEvidence {
        verified_redeal: true,
        progress: Some(GameProgress::AnotherBoard),
        gameplay_scene: true,
        effect_verified: true,
        board_empty: false,
        cards_visible: true,
        observation_round: 3,
        solver_already_sent: false,
    };
    assert!(pyramid_solver_recovery_authorised(accepted));


    for rejected in [
        PyramidSolverRecoveryEvidence { verified_redeal: false, ..accepted },
        PyramidSolverRecoveryEvidence { progress: Some(GameProgress::GameComplete), ..accepted },
        PyramidSolverRecoveryEvidence { gameplay_scene: false, ..accepted },
        PyramidSolverRecoveryEvidence { effect_verified: false, ..accepted },
        PyramidSolverRecoveryEvidence { board_empty: true, ..accepted },
        PyramidSolverRecoveryEvidence { cards_visible: false, ..accepted },
        PyramidSolverRecoveryEvidence { observation_round: 2, ..accepted },
        PyramidSolverRecoveryEvidence { solver_already_sent: true, ..accepted },
    ] {
        assert!(!pyramid_solver_recovery_authorised(rejected));
    }
}


/// Check consumed-card history persists within one context and clears when the socket changes.
#[test]
fn pyramid_consumed_slots_survive_same_context_and_reset_on_context_change() {
    let socket = PathBuf::from("/test/first.sock");
    let first_context = vision_context(socket, GameMode::Pyramid);
    let mut context = Some(first_context.clone());
    let mut state = TableauScanState::for_mode(GameMode::Pyramid);
    let removed = PyramidTargetKind::Card { row: 7, column: 1 };
    state.pyramid.mark_removed(removed);
    let mut completed_boards = 1;
    reset_scan_state_for_context(
        &first_context,
        &mut context,
        &mut state,
        &mut completed_boards,
    );
    assert!(state.pyramid.clicked_target_slots[removed.slot_index().unwrap()]);
    assert_eq!(completed_boards, 1);

    reset_scan_state_for_context(
        &vision_context("/test/second.sock", GameMode::Pyramid),
        &mut context,
        &mut state,
        &mut completed_boards,
    );
    assert!(
        state
            .pyramid
            .clicked_target_slots
            .iter()
            .all(|clicked| !clicked)
    );
    assert_eq!(completed_boards, 0);
}


/// Strategy and selection generation invalidate consumed cards and advisory progress.
#[test]
fn strategy_or_generation_change_resets_pyramid_history_and_progress() {
    let original = vision_context("/test/same.sock", GameMode::Pyramid);
    let removed = PyramidTargetKind::Card { row: 7, column: 1 };


    for changed in [
        ControllerContext { strategy: SolvingStrategy::ShortestPath, ..original.clone() },
        ControllerContext { generation: 2, ..original.clone() },
        ControllerContext { game_mode: GameMode::FreeCell, ..original.clone() },
    ] {
        let mut context = Some(original.clone());
        let mut state = TableauScanState::for_mode(GameMode::Pyramid);
        state.pyramid.mark_removed(removed);
        let mut completed_boards = 2;
        reset_scan_state_for_context(&changed, &mut context, &mut state, &mut completed_boards);
        assert_eq!(state, TableauScanState::for_mode(changed.game_mode));
        assert_eq!(completed_boards, 0);
        assert_eq!(context, Some(changed));
    }
}


/// Check manual progress reset clears consumed Pyramid cards without changing game mode.
#[test]
fn explicit_progress_reset_clears_pyramid_consumed_slots() {
    let mut state = TableauScanState::for_mode(GameMode::Pyramid);
    state
        .pyramid
        .mark_removed(PyramidTargetKind::Card { row: 1, column: 1 });
    let mut completed_boards = 2;
    reset_progress_tracking(&mut state, &mut completed_boards);
    assert_eq!(state.mode(), GameMode::Pyramid);
    assert!(
        state
            .pyramid
            .clicked_target_slots
            .iter()
            .all(|clicked| !clicked)
    );
    assert_eq!(completed_boards, 0);
}


/// Create a one-row TriPeaks evidence mask for scan-state tests.
fn row_mask(row: u8) -> RowMask {
    RowMask::single_for(row, GameMode::TriPeaks.profile().tableau_rows.len() as u8).unwrap()
}


/// Wrap caller-supplied target and row evidence in a minimal gameplay observation.
fn row_observation(
    prediction: PredictedAction,
    observed_rows: Option<RowMask>,
) -> FrameObservation {
    FrameObservation {
        frame: CapturedFrame {
            width: 1,
            height: 1,
            stride: 4,
            format: crate::capture::PixelFormat::Rgba8,
            pixels: vec![0; 4],
        },
        prediction,
        observed_rows,
        gameplay_scene: true,
        game_progress: Some(GameProgress::AnotherBoard),
    }
}


/// Paint selected dialog probes gold in a non-gameplay frame for controller tests.
fn dialog_observation(variants: &[PostGameControlVariant]) -> FrameObservation {
    // Build a non-gameplay frame whose selected dialog-control probes are gold.
    let width = NOMINAL_FRAME_WIDTH as usize;
    let height = NOMINAL_FRAME_HEIGHT as usize;
    let mut frame = CapturedFrame {
        width: NOMINAL_FRAME_WIDTH,
        height: NOMINAL_FRAME_HEIGHT,
        stride: width * 4,
        format: crate::capture::PixelFormat::Rgba8,
        pixels: vec![0; width * height * 4],
    };


    for variant in variants {
        let probe = variant.probe_bounds;


        for y in probe.y..probe.bottom() {


            for x in probe.x..probe.right() {
                let offset = y as usize * frame.stride + x as usize * 4;
                frame.pixels[offset..offset + 4].copy_from_slice(&[220, 170, 80, 255]);
            }
        }
    }

    FrameObservation {
        frame,
        prediction: PredictedAction::NoHighlight,
        observed_rows: None,
        gameplay_scene: false,
        game_progress: Some(GameProgress::GameComplete),
    }
}


/// Intermediate previews expose exact pixels while preserving actionable controller analysis.
#[test]
fn intermediate_legacy_capture_preserves_controller_prediction_without_preview_authority() {


    for (mode, frame) in [
        (GameMode::TriPeaks, tripeaks_stock_frame(true)),
        (GameMode::Pyramid, pyramid_move_frame(true)),
    ] {
        let state = TableauScanState::for_mode(mode);
        let context = ControllerContext {
            generation: 5,
            ..vision_context("/tmp/current-qmp.sock", mode)
        };
        let (events, receiver) = mpsc::channel();
        let sink = WorkerEventSink {
            events,
            latest_frame: LatestFrameSlot::default(),
            capture_context: Mutex::new(Some(context.clone())),
        };
        let original_pixels = frame.pixels.clone();
        let (expected, _) = analyse_captured_frame(frame.clone(), &state)
            .expect("recognised legacy gameplay fixture");
        assert!(matches!(expected.prediction, PredictedAction::Action(_)));
        let (actual, _) = publish_and_analyse_captured_frame(frame, &state, &sink)
            .expect("published and classified legacy capture");
        assert_eq!(actual.prediction, expected.prediction);
        assert_eq!(actual.observed_rows, expected.observed_rows);
        assert_eq!(actual.gameplay_scene, expected.gameplay_scene);
        assert_eq!(actual.game_progress, expected.game_progress);
        assert_eq!(actual.frame.pixels, original_pixels);

        let latest = sink.latest_frame.take().expect("intermediate captured preview");
        assert_eq!(latest.frame.pixels, original_pixels);
        assert_eq!(latest.context, Some(context));
        assert_eq!(latest.prediction, None);
        assert!(latest.diagnostic);
        assert!(!latest.log_prediction);
        assert_eq!(latest.coalesced_frames, 0);
        assert!(sink.latest_frame.take().is_none());
        assert!(receiver.try_recv().is_err());
    }
}


/// Missing-HALO re-observations replace stale previews without growing an image queue.
#[test]
fn intermediate_legacy_missing_halo_captures_keep_only_latest_pixels() {


    for (mode, frame) in [
        (GameMode::TriPeaks, tripeaks_stock_frame(false)),
        (GameMode::Pyramid, pyramid_move_frame(false)),
    ] {
        let state = TableauScanState::for_mode(mode);
        let context = vision_context("/tmp/current-qmp.sock", mode);
        let (events, _receiver) = mpsc::channel();
        let sink = WorkerEventSink {
            events,
            latest_frame: LatestFrameSlot::default(),
            capture_context: Mutex::new(Some(context.clone())),
        };
        let mut newest_pixels = Vec::new();


        for marker in 1..=4 {
            let mut captured = frame.clone();
            captured.pixels[..4].copy_from_slice(&[marker, 0, 0, 255]);
            newest_pixels = captured.pixels.clone();
            let (observation, _) = publish_and_analyse_captured_frame(captured, &state, &sink)
                .expect("fresh supported capture without a HALO");
            assert!(observation.gameplay_scene);
            assert_eq!(observation.prediction, PredictedAction::NoHighlight);
        }

        let latest = sink.latest_frame.take().expect("newest no-HALO capture");
        assert_eq!(latest.frame.pixels, newest_pixels);
        assert_eq!(latest.context, Some(context));
        assert_eq!(latest.prediction, None);
        assert!(latest.diagnostic);
        assert!(!latest.log_prediction);
        assert_eq!(latest.coalesced_frames, 3);
        assert!(sink.latest_frame.take().is_none());
    }
}


/// A decoded non-native frame remains inspectable even when calibrated analysis rejects it.
#[test]
fn intermediate_legacy_capture_retains_pixels_when_analysis_rejects_dimensions() {


    for mode in [GameMode::TriPeaks, GameMode::Pyramid] {
        let state = TableauScanState::for_mode(mode);
        let context = vision_context("/tmp/current-qmp.sock", mode);
        let (events, _receiver) = mpsc::channel();
        let sink = WorkerEventSink {
            events,
            latest_frame: LatestFrameSlot::default(),
            capture_context: Mutex::new(Some(context.clone())),
        };
        sink.publish_frame(
            blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT),
            PredictedAction::NoHighlight,
            false,
        );
        let mut captured = blank_frame(1_280, 720);
        captured.pixels[..4].copy_from_slice(&[71, 81, 91, 255]);
        let original_pixels = captured.pixels.clone();
        assert!(publish_and_analyse_captured_frame(captured, &state, &sink).is_err());

        let latest = sink.latest_frame.take().expect("decoded rejected capture");
        assert_eq!((latest.frame.width, latest.frame.height), (1_280, 720));
        assert_eq!(latest.frame.pixels, original_pixels);
        assert_eq!(latest.context, Some(context));
        assert_eq!(latest.prediction, None);
        assert!(latest.diagnostic);
        assert!(!latest.log_prediction);
        assert_eq!(latest.coalesced_frames, 1);
        assert!(sink.latest_frame.take().is_none());
    }
}


/// Check repeated publication retains only the newest preview and records displaced frames.
#[test]
fn latest_frame_slot_replaces_stale_full_resolution_previews() {
    let slot = LatestFrameSlot::default();
    let first = row_observation(PredictedAction::NoHighlight, None);
    let second_prediction = draw_prediction(crate::geometry::PixelPoint::new(7, 9));
    let mut second = row_observation(second_prediction, None);
    second.frame.pixels.fill(2);

    slot.publish(
        first.frame,
        Some(first.prediction),
        false,
        false,
        Some(vision_context("/tmp/old-qmp.sock", GameMode::TriPeaks)),
    );
    slot.publish(
        second.frame,
        Some(second.prediction),
        true,
        false,
        Some(vision_context("/tmp/new-qmp.sock", GameMode::Pyramid)),
    );

    let latest = slot.take().expect("latest preview");
    assert_eq!(latest.frame.pixels, vec![2; 4]);
    assert_eq!(latest.prediction, Some(second_prediction));
    assert!(latest.log_prediction);
    assert!(!latest.diagnostic);
    assert_eq!(latest.coalesced_frames, 1);
    assert_eq!(
        latest.context,
        Some(vision_context("/tmp/new-qmp.sock", GameMode::Pyramid))
    );
    assert!(slot.take().is_none());
}


/// A raw preparation frame replaces prior HALO metadata while retaining its own identity.
#[test]
fn preparation_preview_carries_no_prior_prediction_and_preserves_generation() {
    let slot = LatestFrameSlot::default();
    let vision = vision_context("/tmp/same-qmp.sock", GameMode::Pyramid);
    let preparation = ControllerContext {
        strategy: SolvingStrategy::ShortestPath,
        generation: 2,
        ..vision.clone()
    };
    let prediction = PredictedAction::Action(
        pyramid::action_for_kind(PyramidTargetKind::Move).expect("canonical Pyramid Move"),
    );
    slot.publish(blank_frame(1, 1), Some(prediction), true, false, Some(vision));
    let mut raw = blank_frame(1, 1);
    raw.pixels.fill(81);
    slot.publish(raw, None, false, false, Some(preparation.clone()));
    let latest = slot.take().expect("preparation preview");
    assert_eq!(latest.context, Some(preparation));
    assert_eq!(latest.prediction, None);
    assert!(!latest.log_prediction);
    assert!(!latest.diagnostic);
    assert_eq!(latest.frame.pixels, vec![81; 4]);
    assert_eq!(latest.coalesced_frames, 1);
}


/// Check a diagnostic failure frame cannot retain an actionable prediction.
#[test]
fn failed_action_publishes_latest_pixels_without_approving_a_target() {
    let (tx, _rx) = mpsc::channel();
    let sink = WorkerEventSink {
        events: tx,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(vision_context("/tmp/qmp.sock", GameMode::Pyramid))),
    };
    let mut observation = row_observation(
        PredictedAction::Action(
            pyramid::action_for_kind(PyramidTargetKind::Move).expect("Move action"),
        ),
        None,
    );
    observation.frame.pixels.fill(71);
    send_post_action_frame(&sink, observation);
    let diagnostic = sink.latest_frame.take().expect("last failure frame");
    assert!(diagnostic.diagnostic);
    assert_eq!(diagnostic.prediction, None);
    assert!(diagnostic.frame.pixels.iter().all(|pixel| *pixel == 71));
}


/// Check completion events report committed board counts immediately and clamp the current board.
#[test]
fn board_progress_reports_committed_completion_without_waiting_for_redeal() {
    let (tx, rx) = mpsc::channel();
    let sink = WorkerEventSink {
        events: tx,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(None),
    };
    let scan_state = TableauScanState::initial();
    send_board_progress(&sink, &scan_state, 1);


    match rx.try_recv().expect("board-completion event").event {
        WorkerEvent::BoardProgress {
            current_board,
            completed_boards,
            boards_per_game,
        } => {
            assert_eq!(
                (current_board, completed_boards, boards_per_game),
                (2, 1, 3)
            );
        }
        _ => panic!("expected a board-completion event"),
    }
}


/// Check TriPeaks draw diagnostics describe key input without inventing a pointer click.
#[test]
fn draw_plan_reports_key_only_input_without_a_pointer_target() {
    let prediction = draw_prediction(crate::geometry::PixelPoint::new(842, 870));
    let plan = plan_step(prediction).expect("key-only draw plan");

    assert_eq!(
        format_step_plan(plan),
        Ok("Stable plan: TriPeaks stock DRAW via qcode D; pointer unchanged; gold anchor=(842, 870); commands=2, events=2.".to_owned())
    );
}


/// Check temporary capture creation location, private permissions and normal-cleanup deletion.
#[test]
fn capture_artifact_is_private_beside_socket_and_removed_on_drop() {
    let test_directory = std::env::temp_dir().join(format!(
        "solitaire-worker-test-{}-{}",
        std::process::id(),
        CAPTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&test_directory).expect("create worker test directory");
    let socket_path = test_directory.join("qmp.sock");

    let artifact = CaptureArtifact::reserve_beside(&socket_path).expect("reserve capture artifact");
    let capture_file = artifact.file_name().to_owned();
    assert_eq!(capture_file.parent(), Some(test_directory.as_path()));
    assert!(capture_file.exists());


    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&capture_file)
            .expect("capture metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }

    drop(artifact);
    assert!(!capture_file.exists());
    fs::remove_dir(&test_directory).expect("remove worker test directory");
}


/// Check effect counting honours the channel threshold and ignores excluded cursor pixels.
#[test]
fn effect_comparison_applies_threshold_and_cursor_exclusion() {
    let before = CapturedFrame {
        width: 3,
        height: 2,
        stride: 12,
        format: crate::capture::PixelFormat::Rgba8,
        pixels: vec![0; 24],
    };
    let mut after = before.clone();
    after.pixels[0] = 19;
    after.pixels[4] = 20;
    after.pixels[8] = 100;
    after.pixels[12 + 2] = 21;

    assert_eq!(
        materially_changed_pixels(
            &before,
            &after,
            PixelRect::new(0, 0, 3, 2),
            Some(PixelRect::new(2, 0, 1, 1)),
            20,
        ),
        Ok(2)
    );
}


/// Check lower-row evidence or any actionable halo prevents premature TriPeaks completion.
#[test]
fn row_three_halo_forbids_completion_when_row_one_is_empty() {
    let row_three_halo = PredictedAction::Action(
        GameMode::TriPeaks
            .profile()
            .tableau_action(9, crate::geometry::PixelPoint::new(296, 630))
            .unwrap(),
    );

    assert!(!settled_completion_evidence(
        row_three_halo,
        Some(row_mask(3)),
        true,
        2,
        1,
    ));
    assert!(!settled_completion_evidence(
        PredictedAction::NoHighlight,
        Some(row_mask(3)),
        true,
        2,
        1,
    ));
    assert!(settled_completion_evidence(
        PredictedAction::NoHighlight,
        None,
        true,
        2,
        1,
    ));
    assert!(!settled_completion_evidence(
        PredictedAction::NoHighlight,
        None,
        true,
        1,
        0,
    ));
    assert!(settled_completion_evidence(
        PredictedAction::NoHighlight,
        None,
        false,
        2,
        0,
    ));
}


/// Check Level Up supersedes progress evidence only after board completion is verified.
#[test]
fn level_up_dialog_overrides_only_a_verified_redeal_state() {
    let observation = dialog_observation(&[LEVEL_UP_CONTROL_VARIANTS[1]]);

    assert_eq!(
        level_up_overrides_board_progress(false, &observation),
        Ok(false)
    );
    assert_eq!(
        level_up_overrides_board_progress(true, &observation),
        Ok(true)
    );
}


/// Check each calibrated gold probe authorises only its associated dialog stage.
#[test]
fn dialog_probes_cannot_authorise_a_different_post_game_stage() {
    let dialog_targets = &POST_GAME_TARGETS[..3];


    for (expected_index, expected_target) in dialog_targets.iter().copied().enumerate() {


        for expected_variant in expected_target.control_variants.iter().copied() {
            let observation = dialog_observation(&[expected_variant]);


            for (actual_index, actual_target) in dialog_targets.iter().copied().enumerate() {
                assert_eq!(
                    resolve_post_game_target(&observation, actual_target),
                    Ok((expected_index == actual_index).then_some(expected_variant))
                );
            }
        }
    }
}


/// Check disconnected legacy and raised Level Up matches produce an explicit ambiguity error.
#[test]
fn level_up_layout_ambiguity_fails_closed() {
    let observation = dialog_observation(&LEVEL_UP_CONTROL_VARIANTS);
    let error = resolve_post_game_target(&observation, POST_GAME_TARGETS[0])
        .expect_err("both Level Up layouts must be ambiguous");

    assert!(error.contains("legacy-low"));
    assert!(error.contains("raised"));
    assert!(error.contains("no guest input was sent"));
}


/// Check connected gold probes resolve one tall Level Up button to the proven lower click.
#[test]
fn one_tall_level_up_button_connects_both_probes_and_uses_proven_low_click() {
    let mut observation = dialog_observation(&LEVEL_UP_CONTROL_VARIANTS);


    for bounds in [
        LEVEL_UP_SHARED_BUTTON_BRIDGE,
        LEVEL_UP_SHARED_BUTTON_INTERIOR,
    ] {


        for y in bounds.y..bounds.bottom() {


            for x in bounds.x..bounds.right() {
                let offset = y as usize * observation.frame.stride + x as usize * 4;
                observation.frame.pixels[offset..offset + 4].copy_from_slice(&[220, 170, 80, 255]);
            }
        }
    }

    assert_eq!(
        resolve_post_game_target(&observation, POST_GAME_TARGETS[0]),
        Ok(Some(LEVEL_UP_CONTROL_VARIANTS[0]))
    );
    assert_eq!(known_post_game_stage(Some(&observation)), Ok(Some(0)));
}


/// Check gold in the probe gap alone cannot resolve an ambiguous Level Up button.
#[test]
fn a_gold_bridge_without_the_button_interior_cannot_authorise_a_click() {
    let mut observation = dialog_observation(&LEVEL_UP_CONTROL_VARIANTS);
    let bounds = LEVEL_UP_SHARED_BUTTON_BRIDGE;


    for y in bounds.y..bounds.bottom() {


        for x in bounds.x..bounds.right() {
            let offset = y as usize * observation.frame.stride + x as usize * 4;
            observation.frame.pixels[offset..offset + 4].copy_from_slice(&[220, 170, 80, 255]);
        }
    }
    assert!(resolve_post_game_target(&observation, POST_GAME_TARGETS[0]).is_err());
}


/// Check the raised Level Up layout uses its calibrated click coordinate.
#[test]
fn raised_level_up_resolves_its_own_click_point() {
    let raised = LEVEL_UP_CONTROL_VARIANTS[1];
    let observation = dialog_observation(&[raised]);

    assert_eq!(
        resolve_post_game_target(&observation, POST_GAME_TARGETS[0]),
        Ok(Some(raised))
    );
    assert_eq!(
        raised.click_point,
        crate::geometry::PixelPoint::new(960, 770)
    );
}


/// Check a visible New Game can advance recovery when Level Up was absent.
#[test]
fn new_game_can_be_verified_when_level_up_is_missing() {
    let new_game = dialog_observation(&[NEW_GAME_CONTROL_VARIANTS[0]]);
    assert_eq!(
        resolve_post_game_target(&new_game, POST_GAME_TARGETS[0]),
        Ok(None)
    );
    assert_eq!(
        new_game_visible_while_awaiting_level_up(&new_game),
        Ok(true)
    );

    let level_up = dialog_observation(&[LEVEL_UP_CONTROL_VARIANTS[1]]);
    assert_eq!(
        new_game_visible_while_awaiting_level_up(&level_up),
        Ok(false)
    );
}


/// Check restart entry selects only proven Level Up/New Game stages and rejects ambiguity.
#[test]
fn recognised_terminal_stage_skips_only_the_completed_post_game_steps() {
    assert_eq!(known_post_game_stage(None), Ok(None));
    let unknown = dialog_observation(&[]);
    assert_eq!(known_post_game_stage(Some(&unknown)), Ok(None));
    let level_up = dialog_observation(&[LEVEL_UP_CONTROL_VARIANTS[0]]);
    assert_eq!(known_post_game_stage(Some(&level_up)), Ok(Some(0)));
    let new_game = dialog_observation(&[NEW_GAME_CONTROL_VARIANTS[0]]);
    assert_eq!(known_post_game_stage(Some(&new_game)), Ok(Some(1)));
    let ambiguous = dialog_observation(&LEVEL_UP_CONTROL_VARIANTS);
    assert!(known_post_game_stage(Some(&ambiguous)).is_err());
}


/// Check a still-visible earlier control is identified before input to a later stage.
#[test]
fn stale_dialog_is_reported_before_the_expected_later_stage() {
    let raised = LEVEL_UP_CONTROL_VARIANTS[1];
    let level_up = dialog_observation(&[raised]);
    assert_eq!(
        earlier_visible_post_game_target(&level_up, 1),
        Ok(Some((0, POST_GAME_TARGETS[0], raised)))
    );

    let new_game_variant = NEW_GAME_CONTROL_VARIANTS[0];
    let new_game = dialog_observation(&[new_game_variant]);
    assert_eq!(earlier_visible_post_game_target(&new_game, 1), Ok(None));
    assert_eq!(
        earlier_visible_post_game_target(&new_game, 2),
        Ok(Some((1, POST_GAME_TARGETS[1], new_game_variant)))
    );
}


/// Check observation and confirmed-click retries stop exactly at their configured limits.
#[test]
fn post_game_retry_budgets_stop_at_the_configured_limits() {
    assert_eq!(
        require_post_game_observation_retry_budget(POST_GAME_MAX_OBSERVATION_ROUNDS - 1, "test"),
        Ok(())
    );
    assert!(
        require_post_game_observation_retry_budget(POST_GAME_MAX_OBSERVATION_ROUNDS, "test")
            .is_err()
    );

    assert_eq!(
        require_post_game_click_retry_budget(POST_GAME_MAX_CLICK_ATTEMPTS - 1, "test"),
        Ok(())
    );
    assert!(require_post_game_click_retry_budget(POST_GAME_MAX_CLICK_ATTEMPTS, "test").is_err());
}


/// Check the initial score-skip click consumes budget and a fourth attempt is refused.
#[test]
fn score_skip_budget_counts_the_initial_click_and_stops_before_a_fourth() {


    for completed_attempts in 0..SCORE_SKIP_MAX_CLICK_ATTEMPTS {
        assert_eq!(require_score_skip_click_budget(completed_attempts), Ok(()));
    }

    let error = require_score_skip_click_budget(SCORE_SKIP_MAX_CLICK_ATTEMPTS)
        .expect_err("a fourth score-skip click must be refused");
    assert!(error.contains("confirmed-delivery score-skip centre clicks"));
    assert!(error.contains("no further guest input was sent"));
}


/// Check score-skip repeats require a second non-gameplay observation while awaiting Level Up.
#[test]
fn score_skip_retry_requires_an_input_free_follow_up_capture() {
    assert!(!score_skip_retry_is_authorised(
        PostGameStage::LevelUpOk,
        false,
        1,
    ));
    assert!(score_skip_retry_is_authorised(
        PostGameStage::LevelUpOk,
        false,
        2,
    ));
    assert!(!score_skip_retry_is_authorised(
        PostGameStage::LevelUpOk,
        true,
        2,
    ));
    assert!(!score_skip_retry_is_authorised(
        PostGameStage::NewGame,
        false,
        2,
    ));
}


/// Check invalid frame dimensions and effect rectangles return errors instead of indexing pixels.
#[test]
fn effect_comparison_rejects_mismatched_dimensions_and_bounds() {
    let before = CapturedFrame {
        width: 2,
        height: 1,
        stride: 8,
        format: crate::capture::PixelFormat::Rgba8,
        pixels: vec![0; 8],
    };
    let mut after = before.clone();
    after.width = 1;
    after.stride = 4;

    assert!(
        materially_changed_pixels(&before, &after, PixelRect::new(0, 0, 1, 1), None, 20,).is_err()
    );
    assert!(
        materially_changed_pixels(&before, &before, PixelRect::new(1, 0, 2, 1), None, 20,).is_err()
    );
}


/// Check cursor-sized changes alone cannot satisfy the configured action-effect threshold.
#[test]
fn cursor_sized_change_does_not_meet_the_material_effect_floor() {
    let width = 64;
    let height = 64;
    let before = CapturedFrame {
        width,
        height,
        stride: width as usize * 4,
        format: crate::capture::PixelFormat::Rgba8,
        pixels: vec![0; width as usize * height as usize * 4],
    };
    let mut after = before.clone();


    for y in 0..48usize {


        for x in 0..32usize {
            let offset = y * after.stride + x * 4;
            after.pixels[offset..offset + 3].fill(255);
        }
    }

    let changed = materially_changed_pixels(
        &before,
        &after,
        PixelRect::new(0, 0, width, height),
        None,
        ACTION_CHANGE_CHANNEL_THRESHOLD,
    )
    .unwrap();
    assert_eq!(changed, 32 * 48);
    assert!(changed < crate::parameters::MINIMUM_TABLEAU_CHANGED_PIXELS);
}


/// Check capture-only validation cycles do not inflate attempted-input timing statistics.
#[test]
fn run_profile_distinguishes_validation_cycles_from_input_attempts() {
    let mut profile = RunProfile::new(Duration::from_millis(7));
    profile.record(ActionProfile {
        total: Duration::from_millis(10),
        ..ActionProfile::default()
    });
    profile.record(ActionProfile {
        total: Duration::from_millis(20),
        input_attempted: true,
        ..ActionProfile::default()
    });

    assert_eq!(profile.cycles, 2);
    assert_eq!(profile.attempted, 1);
    assert_eq!(profile.action_total_sum, Duration::from_millis(20));
    assert_eq!(profile.action_total_min, Some(Duration::from_millis(20)));
    assert_eq!(profile.action_total_max, Some(Duration::from_millis(20)));
}


/// Check a pre-existing STOP request interrupts a wait immediately.
#[test]
fn cancellable_wait_observes_an_existing_stop_request() {
    let cancelled = AtomicBool::new(true);
    let result = cancellable_wait(Duration::from_secs(1), &cancelled);

    assert!(result.is_err());
    assert!(result.unwrap_err() < Duration::from_secs(1));
}


/// Check one unambiguous frame can update row hints without approving guest input.
#[test]
fn single_capture_can_commit_non_authoritative_row_evidence() {
    let mut state = TableauScanState::initial();
    let row_three = row_mask(3);
    let observations = [row_observation(
        PredictedAction::NoHighlight,
        Some(row_three),
    )];

    assert!(reconcile_observation_series(&mut state, &observations));
    assert_eq!(state.active_rows(), row_three);
}


/// Check ambiguous halo analysis leaves the existing row scan unchanged.
#[test]
fn ambiguous_single_capture_does_not_change_row_state() {
    let initial = TableauScanState::initial();
    let row_three = row_mask(3);

    let mut ambiguous_state = initial;
    let ambiguous = [row_observation(
        PredictedAction::Ambiguous { highlight_count: 2 },
        Some(row_three),
    )];
    assert!(!reconcile_observation_series(
        &mut ambiguous_state,
        &ambiguous
    ));
    assert_eq!(ambiguous_state, initial);
}


/// Check scan hints reset on socket/game changes and persist within the same context.
#[test]
fn socket_or_mode_change_resets_the_row_hint_but_same_context_preserves_it() {
    let first_context = vision_context("/tmp/solitaire-a.sock", GameMode::TriPeaks);
    let second_context = vision_context("/tmp/solitaire-b.sock", GameMode::TriPeaks);
    let row_three = row_mask(3);
    let mut state = TableauScanState::from_active_rows(row_three).unwrap();
    let mut remembered_context = None;
    let mut completed_boards = 2;

    reset_scan_state_for_context(
        &first_context,
        &mut remembered_context,
        &mut state,
        &mut completed_boards,
    );
    assert_eq!(state, TableauScanState::initial());
    assert_eq!(completed_boards, 0);
    assert_eq!(
        remembered_context,
        Some(first_context.clone())
    );

    state = TableauScanState::from_active_rows(row_three).unwrap();
    completed_boards = 1;
    reset_scan_state_for_context(
        &first_context,
        &mut remembered_context,
        &mut state,
        &mut completed_boards,
    );
    assert_eq!(state.active_rows(), row_three);
    assert_eq!(completed_boards, 1);

    reset_scan_state_for_context(
        &second_context,
        &mut remembered_context,
        &mut state,
        &mut completed_boards,
    );
    assert_eq!(state, TableauScanState::initial());
    assert_eq!(completed_boards, 0);
    assert_eq!(
        remembered_context,
        Some(second_context)
    );
}


/// Check the manual reset restores initial row hints and zero completed boards.
#[test]
fn manual_progress_reset_clears_board_and_row_tracking() {
    let row_two = row_mask(2);
    let mut state = TableauScanState::from_active_rows(row_two).unwrap();
    let mut completed_boards = 2usize;

    reset_progress_tracking(&mut state, &mut completed_boards);

    assert_eq!(state, TableauScanState::initial());
    assert_eq!(completed_boards, 0);
}


/// Check a new run resets a completed series while retaining partial-game progress.
#[test]
fn new_actionable_run_resets_only_a_completed_three_board_series() {
    let row_three = row_mask(3);
    let mut state = TableauScanState::from_active_rows(row_three).unwrap();
    let boards_per_game = state.mode().profile().boards_per_game;
    let mut completed_boards = boards_per_game - 1;

    assert!(!reset_completed_series_for_new_run(
        &mut state,
        &mut completed_boards,
    ));
    assert_eq!(state.active_rows(), row_three);
    assert_eq!(completed_boards, boards_per_game - 1);

    completed_boards = boards_per_game;
    assert!(reset_completed_series_for_new_run(
        &mut state,
        &mut completed_boards,
    ));
    assert_eq!(state, TableauScanState::initial());
    assert_eq!(completed_boards, 0);
}


/// Klondike unknown scenes cannot inherit the three-board progress detector or action authority.
#[test]
fn unknown_klondike_capture_has_no_progress_or_action_authority() {
    let scan_state = TableauScanState::for_mode(GameMode::Klondike);
    let (observation, _) = analyse_captured_frame(
        blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT),
        &scan_state,
    ).expect("valid frame layout");
    assert!(!observation.gameplay_scene);
    assert_eq!(observation.game_progress, None);
    assert_eq!(observation.prediction, PredictedAction::NoHighlight);
}


/// Klondike never emits the existing games' shared completed-board counters.
#[test]
fn klondike_does_not_publish_three_board_progress() {
    let (events, receiver) = mpsc::channel();
    let sink = WorkerEventSink {
        events,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(None),
    };
    send_board_progress(&sink, &TableauScanState::for_mode(GameMode::Klondike), 0);
    assert!(receiver.try_recv().is_err());
}


/// Delayed stock HALO appearance authorises D before Solver recovery is reserved.
#[test]
fn tripeaks_delayed_stock_halo_uses_fresh_draw_without_solver() {
    let scan_state = TableauScanState::for_mode(GameMode::TriPeaks);
    let mut recovery = TriPeaksHaloRecovery::default();
    let mut draw_plan = None;


    for observation_round in 1..=3 {
        let frame = tripeaks_stock_frame(observation_round == 3);
        let (observation, _) = analyse_captured_frame(frame, &scan_state)
            .expect("classify one fresh TriPeaks frame");
        assert!(observation.gameplay_scene);


        if matches!(observation.prediction, PredictedAction::NoHighlight) {
            assert_eq!(
                recovery.next_missing_halo(observation.gameplay_scene),
                TriPeaksHaloRecoveryStep::Observe {
                    delayed_round: observation_round,
                    after_solver: false,
                },
            );
        } else {
            draw_plan = Some(plan_step(observation.prediction).expect("fresh stock plan"));
            break;
        }
    }
    let plan = draw_plan.expect("late stock HALO must create a plan");
    assert_eq!(plan.input().operation(), InputOperation::PressDrawKey);
    assert!(!recovery.solver_reserved);
    assert_eq!(recovery.delayed_observations, 2);
}


/// A persistently missing HALO cannot cycle through repeated Solver clicks.
#[test]
fn tripeaks_missing_halo_has_one_solver_refresh_and_two_bounded_budgets() {
    let mut recovery = TriPeaksHaloRecovery::default();
    let mut delayed_before_solver = 0;
    let mut delayed_after_solver = 0;
    let mut solver_operations = 0;
    let mut stop_operations = 0;


    for _ in 0..20 {


        match recovery.next_missing_halo(true) {
            TriPeaksHaloRecoveryStep::Observe { after_solver, .. } => {


                if after_solver {
                    delayed_after_solver += 1;
                } else {
                    delayed_before_solver += 1;
                }
            }
            TriPeaksHaloRecoveryStep::RefreshSolver => solver_operations += 1,
            TriPeaksHaloRecoveryStep::Stop => stop_operations += 1,
        }
    }
    assert_eq!(delayed_before_solver, TRIPEAKS_HALO_REOBSERVATION_LIMIT);
    assert_eq!(delayed_after_solver, TRIPEAKS_HALO_REOBSERVATION_LIMIT);
    assert_eq!(solver_operations, 1);
    assert_eq!(stop_operations, 13);
}


/// Unsupported scenes never reserve Solver, even after the input-free budget.
#[test]
fn tripeaks_unsupported_scene_exhausts_observations_without_solver() {
    let mut recovery = TriPeaksHaloRecovery::default();


    for delayed_round in 1..=TRIPEAKS_HALO_REOBSERVATION_LIMIT {
        assert_eq!(
            recovery.next_missing_halo(false),
            TriPeaksHaloRecoveryStep::Observe {
                delayed_round,
                after_solver: false,
            },
        );
    }
    assert_eq!(recovery.next_missing_halo(false), TriPeaksHaloRecoveryStep::Stop);
    assert!(!recovery.solver_reserved);
}


/// A verified redeal's own Solver activation cannot be followed by another refresh.
#[test]
fn tripeaks_new_board_solver_starts_with_refresh_authority_consumed() {
    let mut recovery = TriPeaksHaloRecovery::after_solver();


    for delayed_round in 1..=TRIPEAKS_HALO_REOBSERVATION_LIMIT {
        assert_eq!(
            recovery.next_missing_halo(true),
            TriPeaksHaloRecoveryStep::Observe {
                delayed_round,
                after_solver: true,
            },
        );
    }
    assert_eq!(recovery.next_missing_halo(true), TriPeaksHaloRecoveryStep::Stop);
}


/// Pyramid MOVE/Recycle diagnostics describe the canonical key-only operation.
#[test]
fn pyramid_move_key_plan_keeps_pointer_unchanged_and_uses_two_events() {
    let action = pyramid::action_for_kind(PyramidTargetKind::Move).expect("MOVE target");
    let prediction = PredictedAction::Action(action);
    let plan = plan_step(prediction).expect("canonical MOVE plan");
    let detail = format_step_plan(plan).expect("supported Pyramid D operation");
    assert!(detail.contains("Pyramid MOVE/Recycle via qcode D"));
    assert!(detail.contains("pointer unchanged; commands=2, events=2"));
    assert_eq!(format_action_status(prediction), "MOVE/Recycle — Pyramid via D");
    assert_eq!(
        format_prediction_target(prediction),
        "Pyramid MOVE/Recycle via qcode D; pointer unchanged",
    );
}



/// Spider capture has no inherited three-board progress or blank-frame input authority.
#[test]
fn spider_native_capture_rejects_unknown_scene_and_malformed_storage() {
    let state = TableauScanState::for_mode(GameMode::Spider);
    let (observation, _) = analyse_captured_frame(
        blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT),
        &state,
    ).expect("native Spider diagnostic frame");
    assert_eq!(observation.prediction, PredictedAction::NoHighlight);
    assert!(!observation.gameplay_scene);
    assert_eq!(observation.observed_rows, None);
    assert_eq!(observation.game_progress, None);
    assert!(analyse_captured_frame(blank_frame(1_280, 720), &state).is_err());
    let mut malformed = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
    malformed.pixels.truncate(16);
    assert!(analyse_captured_frame(malformed, &state).is_err());
}


/// Spider stock deals must pass the shared delivery guard as a key action.
#[test]
fn spider_draw_shared_delivery_plan_accepts_only_the_canonical_key() {
    let action = crate::spider::canonical_action(crate::spider::SpiderTarget::Draw)
        .expect("canonical Spider DRAW");
    let plan = plan_step(PredictedAction::Action(action)).expect("Spider DRAW plan");
    assert_eq!(format_step_plan(plan).expect("shared Spider key-delivery guard"),
        "Stable plan: Spider DRAW via qcode D; pointer unchanged; commands=2, events=2.");
    assert_eq!(format_action_status(PredictedAction::Action(action)), "Draw card — Spider DRAW");
}
