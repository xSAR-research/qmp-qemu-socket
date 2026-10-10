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


/// Decode the unchanged supplied native reward panel through the production PNG path.
fn tripeaks_reward_frame() -> CapturedFrame {
    decode_png(include_bytes!("../../tests/fixtures/tripeaks-TP01.png"))
        .expect("supplied native TriPeaks reward fixture")
}


/// Decode the unchanged supplied direct Level Up screenshot through the production PNG path.
fn tripeaks_level_up_frame() -> CapturedFrame {
    decode_png(include_bytes!("../../tests/fixtures/tripeaks-TP02.png"))
        .expect("supplied native TriPeaks direct Level Up fixture")
}


/// Erase a narrow probe without changing any other part of a fixture.
fn erase_reward_probe(frame: &mut CapturedFrame, bounds: PixelRect) {


    for y in bounds.y..bounds.bottom() {


        for x in bounds.x..bounds.right() {
            let offset = y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 4].copy_from_slice(&[0, 0, 0, 255]);
        }
    }
}


/// The supplied Congratulations/skip panel is positive terminal evidence, not a gameplay HALO.
#[test]
fn tripeaks_reward_original_fixture_is_narrowly_recognised() {
    let frame = tripeaks_reward_frame();
    assert_eq!(tripeaks_terminal::is_reward_overlay(&frame), Ok(true));
    let (observation, _) = analyse_captured_frame(frame, &TableauScanState::for_mode(GameMode::TriPeaks)).unwrap();
    assert!(!observation.gameplay_scene);
    assert_eq!(observation.prediction, PredictedAction::NoHighlight);
    assert_eq!(observation.observed_rows, None);
}


/// The fixed title, prompt and separate panel edges must each survive; generic dialogs are refused.
#[test]
fn tripeaks_reward_near_miss_missing_glyph_or_panel_is_refused() {
    let original = tripeaks_reward_frame();


    for probe in [
        PixelRect::new(708, 200, 508, 68),
        PixelRect::new(798, 838, 332, 36),
        PixelRect::new(507, 400, 3, 20),
        PixelRect::new(1_410, 400, 3, 20),
        PixelRect::new(630, 330, 16, 20),
    ] {
        let mut frame = original.clone();
        erase_reward_probe(&mut frame, probe);
        assert_eq!(tripeaks_terminal::is_reward_overlay(&frame), Ok(false));
    }
    assert_eq!(tripeaks_terminal::is_reward_overlay(&tripeaks_stock_frame(true)), Ok(false));
    assert_eq!(tripeaks_terminal::is_reward_overlay(&blank_frame(1_920, 1_080)), Ok(false));
    assert_eq!(tripeaks_terminal::is_reward_overlay(&blank_frame(1_280, 720)), Ok(false));
    let mut malformed = original;
    malformed.pixels.truncate(16);
    assert!(tripeaks_terminal::is_reward_overlay(&malformed).is_err());
}


/// Changing level, XP and central artwork cannot alter the fixed terminal fingerprint.
#[test]
fn tripeaks_reward_detector_excludes_variable_level_xp_and_pointer_artwork() {
    let mut frame = tripeaks_reward_frame();
    erase_reward_probe(&mut frame, PixelRect::new(700, 380, 520, 385));
    erase_reward_probe(&mut frame, PixelRect::new(1_460, 180, 100, 100));
    assert_eq!(tripeaks_terminal::is_reward_overlay(&frame), Ok(true));
}


/// One transient panel never establishes completion; missing evidence resets positives but not the budget.
#[test]
fn tripeaks_reward_confirmation_requires_two_consecutive_fresh_positives_and_is_bounded() {
    use tripeaks_terminal::{TerminalConfirmation, TerminalConfirmationStep, TerminalKind};
    let reward = Some(TerminalKind::RewardSkip);
    let mut evidence = TerminalConfirmation::default();
    assert!(!evidence.active());
    assert_eq!(evidence.observe(reward), TerminalConfirmationStep::Observe);
    assert!(evidence.active());
    assert_eq!(evidence.observe(None), TerminalConfirmationStep::Observe);
    assert_eq!(evidence.observe(reward), TerminalConfirmationStep::Observe);
    assert_eq!(evidence.observe(None), TerminalConfirmationStep::Stop);
    let mut stable = TerminalConfirmation::default();
    assert_eq!(stable.observe(reward), TerminalConfirmationStep::Observe);
    assert_eq!(stable.observe(reward), TerminalConfirmationStep::Confirmed(TerminalKind::RewardSkip));
}


/// The supplied foreground Level Up has a unique raised OK and no gameplay recommendation.
#[test]
fn tripeaks_level_up_original_fixture_selects_existing_raised_ok_only() {
    use tripeaks_terminal::TerminalKind;
    let frame = tripeaks_level_up_frame();
    assert_eq!(tripeaks_terminal::is_reward_overlay(&frame), Ok(false));
    assert_eq!(tripeaks_terminal::is_direct_level_up_overlay(&frame), Ok(true));
    assert_eq!(tripeaks_terminal::terminal_kind(&frame), Ok(Some(TerminalKind::LevelUpOk)));
    assert_eq!(tripeaks_terminal::terminal_kind(&tripeaks_reward_frame()), Ok(Some(TerminalKind::RewardSkip)));
    let (observation, _) = analyse_captured_frame(frame, &TableauScanState::for_mode(GameMode::TriPeaks)).unwrap();
    assert!(!observation.gameplay_scene);
    assert_eq!(observation.prediction, PredictedAction::NoHighlight);
    assert_eq!(observation.observed_rows, None);
    assert_eq!(resolve_post_game_target(&observation, POST_GAME_TARGETS[0]),
        Ok(Some(crate::parameters::LEVEL_UP_CONTROL_VARIANTS[1])));
    assert_eq!(known_post_game_stage(Some(&observation)), Ok(Some(0)));
    assert_eq!(level_up_overrides_board_progress(false, &observation), Ok(false));
}


/// The later supplied TP03 scene already passes the unchanged masks despite its title pointer.
#[test]
fn tripeaks_level_up_tp03_original_passes_existing_terminal_and_control_guards() {
    let frame = decode_png(include_bytes!("../../tests/fixtures/tripeaks-TP03.png")).unwrap();
    assert_eq!(tripeaks_terminal::is_reward_overlay(&frame), Ok(false));
    assert_eq!(tripeaks_terminal::terminal_kind(&frame), Ok(Some(tripeaks_terminal::TerminalKind::LevelUpOk)));
    let (observation, _) = analyse_captured_frame(frame, &TableauScanState::for_mode(GameMode::TriPeaks)).unwrap();
    assert!(!observation.gameplay_scene);
    assert_eq!(observation.prediction, PredictedAction::NoHighlight);
    assert_eq!(observation.observed_rows, None);
    assert_eq!(post_game::resolve_post_game_target_for_entry(&observation, POST_GAME_TARGETS[0], true),
        Ok(Some(LEVEL_UP_CONTROL_VARIANTS[1])));
    assert_eq!(known_post_game_stage(Some(&observation)), Ok(Some(0)));
    assert_eq!(tripeaks_terminal::FINAL_CARD_ACQUISITION_CAPTURES, 30);
    assert_eq!(crate::parameters::POST_GAME_MAX_OBSERVATION_ROUNDS, 20);
    assert_eq!(tripeaks_terminal::TERMINAL_CONFIRMATION_CAPTURES, 4);
}


/// A progression modal without the audited dim reward underlay cannot grant a game win.
#[test]
fn tripeaks_level_up_missing_title_underlay_panel_or_unique_ok_is_refused() {
    let original = tripeaks_level_up_frame();


    for probe in [
        PixelRect::new(800, 234, 320, 60),
        PixelRect::new(798, 854, 100, 20),
        PixelRect::new(1_058, 854, 72, 20),
        PixelRect::new(507, 880, 3, 20),
        PixelRect::new(1_410, 880, 3, 20),
        PixelRect::new(550, 922, 50, 2),
        PixelRect::new(1_320, 139, 50, 2),
        PixelRect::new(460, 400, 3, 40),
        PixelRect::new(1_457, 400, 3, 40),
        crate::parameters::LEVEL_UP_CONTROL_VARIANTS[1].probe_bounds,
    ] {
        let mut frame = original.clone();
        erase_reward_probe(&mut frame, probe);
        assert_eq!(tripeaks_terminal::is_direct_level_up_overlay(&frame), Ok(false), "missing {probe:?}");
    }
    let mut two_buttons = original;
    let extra = crate::parameters::LEVEL_UP_CONTROL_VARIANTS[0].probe_bounds;


    for y in extra.y..extra.bottom() {


        for x in extra.x..extra.right() {
            let offset = y as usize * two_buttons.stride + x as usize * 4;
            two_buttons.pixels[offset..offset + 3].copy_from_slice(&crate::parameters::GOLD_RGB_CANDIDATES[0]);
        }
    }
    assert_eq!(tripeaks_terminal::is_direct_level_up_overlay(&two_buttons), Ok(false));
    let mut swapped_button = two_buttons.clone();
    erase_reward_probe(&mut swapped_button, crate::parameters::LEVEL_UP_CONTROL_VARIANTS[1].probe_bounds);
    assert_eq!(tripeaks_terminal::is_direct_level_up_overlay(&swapped_button), Ok(false),
        "the evidenced raised layout cannot authorise a synthetic legacy-low click");
    let (swapped, _) = analyse_captured_frame(swapped_button, &TableauScanState::for_mode(GameMode::TriPeaks)).unwrap();
    assert_eq!(post_game::resolve_post_game_target_for_entry(&swapped, POST_GAME_TARGETS[0], true), Ok(None));
    assert_eq!(earlier_visible_post_game_target(&swapped, 1, true), Ok(None));
    assert_eq!(tripeaks_terminal::is_direct_level_up_overlay(&tripeaks_reward_frame()), Ok(false));
    assert_eq!(tripeaks_terminal::is_direct_level_up_overlay(&tripeaks_stock_frame(true)), Ok(false));
    assert_eq!(tripeaks_terminal::is_direct_level_up_overlay(&blank_frame(1_280, 720)), Ok(false));
    let mut malformed = tripeaks_level_up_frame();
    malformed.pixels.truncate(16);
    assert!(tripeaks_terminal::is_direct_level_up_overlay(&malformed).is_err());
}


/// Level, rank, central artwork, pointer and desktop chrome do not supply authority.
#[test]
fn tripeaks_level_up_detector_excludes_variable_artwork_and_desktop() {
    let mut frame = tripeaks_level_up_frame();
    erase_reward_probe(&mut frame, PixelRect::new(680, 396, 560, 292));
    erase_reward_probe(&mut frame, PixelRect::new(0, 0, 1_920, 100));
    erase_reward_probe(&mut frame, PixelRect::new(0, 1_000, 1_920, 80));
    erase_reward_probe(&mut frame, PixelRect::new(0, 550, 100, 100));
    assert_eq!(tripeaks_terminal::is_direct_level_up_overlay(&frame), Ok(true));
}


/// A change of terminal kind resets consecutive positives without renewing the four-frame budget.
#[test]
fn tripeaks_level_up_confirmation_requires_two_fresh_observations_of_same_kind() {
    use tripeaks_terminal::{TerminalConfirmation, TerminalConfirmationStep, TerminalKind};
    let reward = Some(TerminalKind::RewardSkip);
    let level_up = Some(TerminalKind::LevelUpOk);
    let mut mixed = TerminalConfirmation::default();
    assert_eq!(mixed.observe(reward), TerminalConfirmationStep::Observe);
    assert_eq!(mixed.observe(level_up), TerminalConfirmationStep::Observe);
    assert_eq!(mixed.observe(level_up), TerminalConfirmationStep::Confirmed(TerminalKind::LevelUpOk));
    let mut alternating = TerminalConfirmation::default();
    assert_eq!(alternating.observe(reward), TerminalConfirmationStep::Observe);
    assert_eq!(alternating.observe(level_up), TerminalConfirmationStep::Observe);
    assert_eq!(alternating.observe(reward), TerminalConfirmationStep::Observe);
    assert_eq!(alternating.observe(level_up), TerminalConfirmationStep::Stop);
    let mut transient = TerminalConfirmation::default();
    assert_eq!(transient.observe(level_up), TerminalConfirmationStep::Observe);
    assert_eq!(transient.observe(None), TerminalConfirmationStep::Observe);
    assert_eq!(transient.observe(None), TerminalConfirmationStep::Observe);
    assert_eq!(transient.observe(None), TerminalConfirmationStep::Stop);
}


/// Fresh bare gold cannot retry OK after a direct terminal handover loses its fixed panel evidence.
#[test]
fn tripeaks_level_up_direct_entry_rechecks_full_panel_for_current_and_stale_ok() {
    let scan = TableauScanState::for_mode(GameMode::TriPeaks);
    let (original, _) = analyse_captured_frame(tripeaks_level_up_frame(), &scan).unwrap();
    let raised = crate::parameters::LEVEL_UP_CONTROL_VARIANTS[1];
    assert_eq!(post_game::resolve_post_game_target_for_entry(&original, POST_GAME_TARGETS[0], true), Ok(Some(raised)));
    assert_eq!(earlier_visible_post_game_target(&original, 1, true), Ok(Some((0, POST_GAME_TARGETS[0], raised))));
    let gold_only = dialog_observation(&[raised]);
    assert_eq!(resolve_post_game_target(&gold_only, POST_GAME_TARGETS[0]), Ok(Some(raised)));
    assert_eq!(post_game::resolve_post_game_target_for_entry(&gold_only, POST_GAME_TARGETS[0], true), Ok(None));
    assert_eq!(earlier_visible_post_game_target(&gold_only, 1, true), Ok(None));
    assert_eq!(post_game::resolve_post_game_target_for_entry(&gold_only, POST_GAME_TARGETS[0], false), Ok(Some(raised)),
        "existing non-direct and Pyramid control policy remains unchanged");
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


/// Keep exact RGB counts with different strides, alpha changes and a partial ROI.
#[test]
fn effect_comparison_preserves_padded_rgb_counts_and_mask_intersections() {
    let before = blank_frame(3, 2);
    let mut after = before.clone();
    after.stride = 16;
    after.pixels = vec![255; 32];


    for row in 0..2 {
        after.pixels[row * 16..row * 16 + 12].fill(0);


        for column in 0..3 {
            after.pixels[row * 16 + column * 4 + 3] = 255;
        }
    }
    after.pixels[0] = 20;
    after.pixels[5] = 19;
    after.pixels[26] = 255;
    let bounds = PixelRect::new(0, 0, 3, 2);
    let exclusion = Some(PixelRect::new(0, 0, 1, 1));
    assert_eq!(
        materially_changed_pixels(&before, &after, bounds, None, 20),
        Ok(2)
    );
    assert_eq!(
        materially_changed_pixels(&after, &before, bounds, None, 20),
        Ok(2)
    );
    assert_eq!(
        materially_changed_pixels(&before, &after, bounds, exclusion, 20),
        Ok(1)
    );
    assert_eq!(
        materially_changed_pixels(&before, &after, bounds, None, 255),
        Ok(1)
    );
    assert_eq!(
        materially_changed_pixels(&before, &before, bounds, exclusion, 0),
        Ok(5)
    );
    assert_eq!(
        materially_changed_pixels(&before, &after, PixelRect::new(1, 0, 2, 2), exclusion, 20),
        Ok(1)
    );
}


/// Invalid masks and either malformed frame stop comparison rather than return zero.
#[test]
fn effect_comparison_rejects_invalid_masks_and_independent_layouts() {
    let frame = blank_frame(3, 2);
    let bounds = PixelRect::new(0, 0, 3, 2);


    for exclusion in [
        PixelRect::new(0, 0, 0, 1),
        PixelRect::new(2, 0, 2, 1),
        PixelRect::new(u32::MAX, 0, 2, 1),
    ] {
        let error =
            materially_changed_pixels(&frame, &frame, bounds, Some(exclusion), 20).unwrap_err();
        assert!(error.contains("exclusion 0"));
    }
    let mut malformed = frame.clone();
    malformed.stride = usize::MAX;
    assert!(
        materially_changed_pixels(&malformed, &frame, bounds, None, 20)
            .unwrap_err()
            .contains("before frame")
    );
    assert!(
        materially_changed_pixels(&frame, &malformed, bounds, None, 20)
            .unwrap_err()
            .contains("after frame")
    );
    malformed.stride = frame.stride;
    malformed.pixels.pop();
    assert!(materially_changed_pixels(&frame, &malformed, bounds, None, 20).is_err());
    assert!(
        materially_changed_pixels(&frame, &frame, PixelRect::new(u32::MAX, 0, 2, 1), None, 20)
            .is_err()
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
        earlier_visible_post_game_target(&level_up, 1, false),
        Ok(Some((0, POST_GAME_TARGETS[0], raised)))
    );

    let new_game_variant = NEW_GAME_CONTROL_VARIANTS[0];
    let new_game = dialog_observation(&[new_game_variant]);
    assert_eq!(earlier_visible_post_game_target(&new_game, 1, false), Ok(None));
    assert_eq!(
        earlier_visible_post_game_target(&new_game, 2, false),
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


/// Exercise the production worker, captures, classifier and xSAR transport through local QMP.
mod tripeaks_reward_qmp_regressions {
    use super::*;
    use std::{
        fs,
        io::{BufRead, BufReader, Write},
        os::unix::net::{UnixListener, UnixStream},
        sync::atomic::AtomicU64,
        thread,
    };
    use serde_json::{Value, json};
    use crate::geometry::PixelPoint;
    use crate::parameters::{
        AnimationSettleDelays, CLICK_OFFSET_X, GAMEPLAY_FELT_PROBE_BOUNDS,
        GOLD_RGB_CANDIDATES, HALO_GOLD_LINE_OFFSET_Y, HALO_GOLD_RUN_MIN,
        SCORE_SKIP_CONTROL, SHARED_SOLVER_CONTROL, TABLEAU_CARD_REGIONS,
        PLAY_CONTROL_VARIANTS,
    };


    /// Original native capture; no reconstructed terminal artwork is used.
    const REWARD_PNG: &[u8] = include_bytes!("../../tests/fixtures/tripeaks-TP01.png");
    /// Original native Level Up capture used for direct terminal authority.
    const LEVEL_UP_PNG: &[u8] = include_bytes!("../../tests/fixtures/tripeaks-TP02.png");
    /// Original supported Level Up capture with pointer overlap at the later acquisition boundary.
    const LATE_LEVEL_UP_PNG: &[u8] = include_bytes!("../../tests/fixtures/tripeaks-TP03.png");
    /// Unique temporary socket directories when Cargo runs tests concurrently.
    static NEXT_REWARD_SOCKET: AtomicU64 = AtomicU64::new(1);


    /// Inject one precise recovery or cancellation boundary through QMP.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Scenario {
        /// Acknowledged skip followed by fresh actionable gameplay.
        Recover,
        /// Cooperative STOP while the first reward capture is acknowledged.
        StopAfterFirstReward,
        /// Cooperative STOP at the terminal pre-input pointer probe.
        StopAfterSkipProbe,
        /// Causal button-down error followed by release-only transport recovery.
        UncertainSkip,
        /// Six unsupported animation captures precede the original reward overlay.
        LateReward,
        /// The terminal animation remains unsupported throughout the full budget.
        UnknownOnly,
        /// One positive reward candidate disappears throughout confirmation.
        TransientReward,
        /// Acknowledged skip loses positive reward evidence before a possible retry.
        SkipDisappears,
        /// Fresh canonical source pixels differ from reward by fewer than the effect floor.
        InsufficientEffect,
        /// A visible nonempty redeal resumes the existing missing-HALO Solver path.
        RedealNoHalo,
        /// Direct native Level Up followed by the complete supported restart sequence.
        DirectLevelUp,
        /// STOP is raised after the fresh OK pointer probe and before button input.
        StopAfterLevelUpProbe,
        /// Uncertain native OK delivery permits only release recovery.
        UncertainLevelUp,
        /// One reward candidate precedes a distinct Level Up confirmation episode.
        RewardThenLevelUp,
        /// STOP on the second Level Up acquisition exposes premature mixed-stage commits.
        MixedStageReset,
        /// Six unsupported animation frames precede a direct native Level Up.
        LateLevelUp,
        /// Acknowledged OK uncovers reward, then skip continues the full restart.
        LevelUpRewardAfterOk,
        /// Level Up disappears into reward before any OK delivery.
        LevelUpDisappearsBeforeOk,
        /// A bare gold former OK probe cannot authorise another OK click.
        StaleBareGoldAfterOk,
        /// Reward and Level Up alternate throughout the four-capture allowance.
        AlternatingTerminalKinds,
        /// The first supported native panel appears on qualified acquisition twenty-one.
        CandidateTwentyOne,
        /// The last allowed acquisition supplies a candidate that confirms on capture thirty-one.
        CandidateThirty,
        /// A candidate at thirty disappears throughout its separate confirmation allowance.
        TransientCandidateThirty,
        /// Different supported kinds alternate from candidate thirty through capture thirty-three.
        MixedCandidateThirty,
        /// Cooperative STOP is raised before the twenty-first unknown screenshot ACK.
        StopExtendedAcquisition,
        /// Positive nonempty gameplay after twenty-one unknown frames resumes existing Solver recovery.
        ExtendedRedealNoHalo,
    }


    /// Whether this scenario uses native Level Up plus synthetic later-stage controls.
    fn level_up_scenario(scenario: Scenario) -> bool {
        matches!(scenario, Scenario::DirectLevelUp | Scenario::StopAfterLevelUpProbe
            | Scenario::UncertainLevelUp | Scenario::RewardThenLevelUp | Scenario::MixedStageReset
            | Scenario::LateLevelUp | Scenario::LevelUpRewardAfterOk | Scenario::LevelUpDisappearsBeforeOk
            | Scenario::StaleBareGoldAfterOk | Scenario::AlternatingTerminalKinds
            | Scenario::CandidateTwentyOne | Scenario::CandidateThirty
            | Scenario::TransientCandidateThirty | Scenario::MixedCandidateThirty)
    }


    /// New acquisition-boundary episodes use only the original TP03 supported sample.
    fn late_level_up_scenario(scenario: Scenario) -> bool {
        matches!(scenario, Scenario::CandidateTwentyOne | Scenario::CandidateThirty
            | Scenario::TransientCandidateThirty | Scenario::MixedCandidateThirty)
    }


    /// Original pixels supplied for each independently requested screendump.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum CaptureKind {
        /// Canonical sole exposed top-row card.
        Initial,
        /// Unmodified user-supplied reward overlay.
        Reward,
        /// Fresh actionable stock recommendation following the skip.
        Recovered,
        /// Cleared card effect with no recognised terminal or gameplay scene.
        Unsupported,
        /// Positive row-four face evidence with a recognised felt scene and no HALO.
        Nonempty,
        /// Independently acquired original native Level Up capture.
        LevelUp,
        /// Existing calibrated synthetic New Game control frame.
        NewGame,
        /// Existing calibrated synthetic Play control frame.
        Play,
        /// A gold-only former OK probe without native dialog title or artwork.
        BareGold,
    }


    /// Semantic record of the actual commands received by the local QMP server.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum WireEvent {
        /// One fresh filesystem screenshot request.
        Capture(CaptureKind),
        /// Fresh running-VM and active-pointer input probe completed.
        Probe,
        /// Absolute pointer movement to a mapped native pixel coordinate.
        Move((i64, i64)),
        /// One guest left-button press, counted separately from releases.
        Down((i64, i64)),
        /// Idempotent button release, including xSAR's release-only recovery.
        Up,
    }


    /// Completed server evidence retained after production worker termination.
    #[derive(Debug, Default)]
    struct WireReport {
        /// Ordered requests and input transitions observed on all connections.
        events: Vec<WireEvent>,
        /// Independently acquired original reward frames.
        reward_captures: usize,
        /// Independently acquired unsupported terminal-animation frames.
        unsupported_captures: usize,
        /// Positive nonempty gameplay observations before the existing Solver recovery.
        nonempty_captures: usize,
        /// Independently acquired original native Level Up frames.
        level_up_captures: usize,
        /// Gold-only earlier-control captures that receive no input authority.
        bare_gold_captures: usize,
        /// Normal connection plus any release-only recovery connection.
        connections: usize,
        /// Release events acknowledged on the recovery connection.
        recovery_releases: usize,
        /// Whether this run injected STOP at the fresh skip probe.
        stopped_after_probe: bool,
        /// Whether STOP was injected at the fresh native OK input probe.
        stopped_after_level_up_probe: bool,
    }


    impl WireReport {
        /// Count actual presses at one audited coordinate.
        fn downs_at(&self, point: (i64, i64)) -> usize {
            self.events.iter().filter(|event| **event == WireEvent::Down(point)).count()
        }
        /// Count all presses, catching unexpected controls and replayed actions.
        fn downs(&self) -> usize {
            self.events.iter().filter(|event| matches!(event, WireEvent::Down(_))).count()
        }
    }


    /// Remove only this test's unique socket directory after both peers exit.
    struct SocketDirectory(
        /// Unique directory containing only this test's socket and capture files.
        PathBuf,
    );
    impl Drop for SocketDirectory {
        /// Clean up the test-owned directory after the server thread is joined.
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
    }


    /// Convert an audited pixel coordinate through the production mapping.
    fn wire_point(point: PixelPoint) -> (i64, i64) {
        let mapped = pixel_point_to_qmp(point, NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT)
            .expect("canonical test point maps to QMP");
        (i64::from(mapped.x), i64::from(mapped.y))
    }


    /// Paint only explicitly calibrated fixture rectangles.
    fn paint(frame: &mut CapturedFrame, bounds: PixelRect, rgba: [u8; 4]) {
        for y in bounds.y..bounds.bottom() {
            for x in bounds.x..bounds.right() {
                let offset = y as usize * frame.stride + x as usize * 4;
                frame.pixels[offset..offset + 4].copy_from_slice(&rgba);
            }
        }
    }


    /// Preserve r1c3 source geometry used by the existing reward regressions.
    fn last_top_card(material_effect: bool) -> (CapturedFrame, PredictedAction) {
        last_top_card_at(2, material_effect)
    }


    /// One positively exposed top-row card with canonical HALO and controlled effect evidence.
    fn last_top_card_at(slot_index: usize, material_effect: bool) -> (CapturedFrame, PredictedAction) {
        let mut frame = if material_effect {
            blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT)
        } else {
            decode_png(REWARD_PNG).expect("original pixels under weak-effect source")
        };
        paint(&mut frame, GAMEPLAY_FELT_PROBE_BOUNDS, [20, 140, 60, 255]);
        let card = TABLEAU_CARD_REGIONS[slot_index];
        if material_effect {
            paint(&mut frame, card.card_bounds, [255; 4]);
        } else {
            // All lower-row probes are outside r1c3's audited effect rectangle.
            // Remove incidental white artwork without changing that rectangle.
            for row in GameMode::TriPeaks.profile().tableau_rows.iter().skip(1) {
                paint(&mut frame, row.face_probe_bounds, [0, 0, 0, 255]);
            }
            let face = GameMode::TriPeaks.profile().tableau_rows[0].face_probe_bounds;
            paint(&mut frame, PixelRect::new(card.card_bounds.x + 5, face.y,
                crate::parameters::FACE_UP_WHITE_BLOCK_MIN_WIDTH, face.height), [255; 4]);
        }
        let anchor = PixelPoint::new(
            card.click_point.x - CLICK_OFFSET_X,
            (card.card_bounds.bottom() + HALO_GOLD_LINE_OFFSET_Y) as i32,
        );
        let gold = GOLD_RGB_CANDIDATES[0];
        paint(&mut frame, PixelRect::new(
            anchor.x as u32, anchor.y as u32, HALO_GOLD_RUN_MIN + 1, 1,
        ), [gold[0], gold[1], gold[2], 255]);
        let expected = PredictedAction::Action(GameMode::TriPeaks.profile()
            .tableau_action(slot_index, anchor).expect("canonical top-row action"));
        let state = TableauScanState::for_mode(GameMode::TriPeaks);
        let analysis = analyse_frame_with_state(&frame, &state).expect("native last-card analysis");
        assert_eq!(analysis.prediction, expected);
        assert_eq!(analysis.top_row_face_up_count, 1);
        assert_eq!(analysis.observed_rows, RowMask::single_for(1, 4));
        assert!(is_gameplay_scene_for_mode(&frame, GameMode::TriPeaks)
            .expect("last-card gameplay fingerprint"));
        (frame, expected)
    }


    /// No PNG encoder is exported by xSAR. This test-only lossless encoder uses
    /// standard PNG RGBA rows and stored zlib blocks, without adding a dependency.
    fn encode_native_png(frame: &CapturedFrame) -> Vec<u8> {
        /// PNG CRC32 over chunk type and bytes, using a local lookup table.
        fn crc(bytes: impl Iterator<Item = u8>) -> u32 {
            let mut table = [0_u32; 256];
            for (index, value) in table.iter_mut().enumerate() {
                let mut remainder = index as u32;
                for _ in 0..8 {
                    remainder = if remainder & 1 != 0 {
                        (remainder >> 1) ^ 0xedb8_8320
                    } else { remainder >> 1 };
                }
                *value = remainder;
            }
            let mut checksum = u32::MAX;
            for byte in bytes {
                checksum = (checksum >> 8) ^ table[((checksum ^ u32::from(byte)) & 255) as usize];
            }
            !checksum
        }
        /// Append one length-prefixed PNG chunk and its checked CRC.
        fn chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
            png.extend_from_slice(&(data.len() as u32).to_be_bytes());
            png.extend_from_slice(kind);
            png.extend_from_slice(data);
            png.extend_from_slice(&crc(kind.iter().copied().chain(data.iter().copied())).to_be_bytes());
        }
        let row_bytes = frame.width as usize * 4;
        let mut raw = Vec::with_capacity((row_bytes + 1) * frame.height as usize);
        for y in 0..frame.height as usize {
            raw.push(0); // PNG filter None.
            raw.extend_from_slice(&frame.pixels[y * frame.stride..y * frame.stride + row_bytes]);
        }
        let mut compressed = vec![0x78, 0x01]; // Valid zlib header, stored DEFLATE.
        let count = raw.len().div_ceil(65_535);
        for (index, block) in raw.chunks(65_535).enumerate() {
            compressed.push(u8::from(index + 1 == count));
            let length = block.len() as u16;
            compressed.extend_from_slice(&length.to_le_bytes());
            compressed.extend_from_slice(&(!length).to_le_bytes());
            compressed.extend_from_slice(block);
        }
        let (mut a, mut b) = (1_u32, 0_u32);
        for block in raw.chunks(5_552) {
            for &byte in block { a += u32::from(byte); b += a; }
            a %= 65_521;
            b %= 65_521;
        }
        compressed.extend_from_slice(&((b << 16) | a).to_be_bytes());
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut header = Vec::new();
        header.extend_from_slice(&frame.width.to_be_bytes());
        header.extend_from_slice(&frame.height.to_be_bytes());
        header.extend_from_slice(&[8, 6, 0, 0, 0]);
        chunk(&mut png, b"IHDR", &header);
        chunk(&mut png, b"IDAT", &compressed);
        chunk(&mut png, b"IEND", &[]);
        let round_trip = decode_png(&png).expect("test PNG decodes through production xSAR");
        assert_eq!(round_trip.pixels, frame.pixels);
        png
    }


    /// Flush one newline-delimited QMP response before reading another request.
    fn packet(stream: &mut UnixStream, value: Value) {
        serde_json::to_writer(&mut *stream, &value).expect("mock QMP packet");
        stream.write_all(b"\n").expect("mock QMP newline");
        stream.flush().expect("mock QMP flush");
    }
    /// Acknowledge one exact numeric QMP request identifier.
    fn ack(stream: &mut UnixStream, id: u64) { packet(stream, json!({"return": {}, "id": id})); }


    /// Bounded local accept, including xSAR's release-only recovery connection.
    fn accept(listener: &UnixListener) -> UnixStream {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match listener.accept() {
                Ok((stream, _)) => return stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "mock QMP accept timed out");
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("mock QMP accept: {error}"),
            }
        }
    }


    /// Pre-encoded fixtures keep PNG encoding outside QMP's handshake timeout.
    struct MockPngs {
        /// Fresh canonical planning frame.
        initial_png: Vec<u8>,
        /// Fresh actionable post-Solver frame.
        recovered_png: Vec<u8>,
        /// Cleared unsupported animation frame.
        unsupported_png: Vec<u8>,
        /// Positively nonempty gameplay without a HALO.
        nonempty_png: Vec<u8>,
        /// Existing synthetic calibrated New Game control.
        new_game_png: Vec<u8>,
        /// Existing synthetic calibrated Play control.
        play_png: Vec<u8>,
        /// Bare gold OK probe, retaining none of the native Level Up evidence.
        bare_gold_png: Vec<u8>,
    }


    /// Resolve the calibrated OK variant from the actual native Level Up pixels.
    fn native_level_up_control() -> PostGameControlVariant {
        let frame = decode_png(LEVEL_UP_PNG).expect("original Level Up fixture");
        let (observation, _) = analyse_captured_frame(frame,
            &TableauScanState::for_mode(GameMode::TriPeaks)).expect("native Level Up classification");
        assert!(!observation.gameplay_scene);
        resolve_post_game_target(&observation, POST_GAME_TARGETS[0])
            .expect("native Level Up control is unambiguous")
            .expect("native Level Up has one calibrated OK target")
    }


    /// Serve original pixels and record semantic input ordering through real QMP.
    fn serve(
        listener: UnixListener, pngs: MockPngs,
        scenario: Scenario, cancel: Arc<AtomicBool>,
    ) -> WireReport {
        let MockPngs { initial_png, recovered_png, unsupported_png, nonempty_png,
            new_game_png, play_png, bare_gold_png } = pngs;
        let mut report = WireReport::default();
        let skip = wire_point(SCORE_SKIP_CONTROL.click_point);
        let level_up_png = if late_level_up_scenario(scenario) { LATE_LEVEL_UP_PNG } else { LEVEL_UP_PNG };
        let level_up = if level_up_scenario(scenario) {
            wire_point(native_level_up_control().click_point)
        } else { (0, 0) };
        let mut position = (0_i64, 0_i64);
        let mut pending_skip_error = None;
        let mut error_delivered = false;
        loop {
            let mut stream = accept(&listener);
            report.connections += 1;
            stream.set_read_timeout(Some(Duration::from_secs(15))).expect("mock QMP read timeout");
            let mut reader = BufReader::new(stream.try_clone().expect("mock QMP stream clone"));
            packet(&mut stream, json!({"QMP": {"version": {"qemu": {"major": 11, "minor": 0, "micro": 0}}, "capabilities": []}}));
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).expect("read production QMP request") == 0 { break; }
                let request: Value = serde_json::from_str(&line).expect("production QMP request JSON");
                let id = request["id"].as_u64().expect("production QMP numeric id");
                match request["execute"].as_str().expect("production QMP execute") {
                    "qmp_capabilities" => ack(&mut stream, id),
                    "query-status" => packet(&mut stream, json!({"return": {"running": true, "status": "running"}, "id": id})),
                    "query-mice" => {
                        report.events.push(WireEvent::Probe);
                        if scenario == Scenario::StopAfterSkipProbe && report.reward_captures >= 2 {
                            report.stopped_after_probe = true;
                            cancel.store(true, Ordering::Release);
                        }
                        if scenario == Scenario::StopAfterLevelUpProbe
                            && report.level_up_captures >= 2 && report.downs_at(level_up) == 0
                        {
                            report.stopped_after_level_up_probe = true;
                            cancel.store(true, Ordering::Release);
                        }
                        packet(&mut stream, json!({"return": [{"name": "QEMU HID Tablet", "index": 0, "current": true, "absolute": true}], "id": id}));
                    }
                    "screendump" => {
                        assert_eq!(request["arguments"]["format"], "png");
                        let (kind, bytes) = if !report.events.iter().any(|event| matches!(event, WireEvent::Capture(_))) {
                            (CaptureKind::Initial, initial_png.as_slice())
                        } else if level_up_scenario(scenario) {
                            assert!(!(scenario == Scenario::UncertainLevelUp && error_delivered),
                                "uncertain OK must stop before any further capture");
                            if report.downs_at(wire_point(SHARED_SOLVER_CONTROL.click_point)) > 0 {
                                (CaptureKind::Recovered, recovered_png.as_slice())
                            } else if report.downs_at(wire_point(PLAY_CONTROL_VARIANTS[0].click_point)) > 0 {
                                report.nonempty_captures += 1;
                                (CaptureKind::Nonempty, nonempty_png.as_slice())
                            } else if report.downs_at(wire_point(NEW_GAME_CONTROL_VARIANTS[0].click_point)) > 0 {
                                (CaptureKind::Play, play_png.as_slice())
                            } else if report.downs_at(level_up) > 0 {
                                if scenario == Scenario::LevelUpRewardAfterOk && report.downs_at(skip) == 0 {
                                    report.reward_captures += 1;
                                    (CaptureKind::Reward, REWARD_PNG)
                                } else if scenario == Scenario::StaleBareGoldAfterOk {
                                    report.bare_gold_captures += 1;
                                    (CaptureKind::BareGold, bare_gold_png.as_slice())
                                } else {
                                    (CaptureKind::NewGame, new_game_png.as_slice())
                                }
                            } else if late_level_up_scenario(scenario)
                                && (report.unsupported_captures < if scenario == Scenario::CandidateTwentyOne { 20 } else { 29 }
                                    || (scenario == Scenario::TransientCandidateThirty && report.level_up_captures > 0))
                            {
                                report.unsupported_captures += 1;
                                (CaptureKind::Unsupported, unsupported_png.as_slice())
                            } else if scenario == Scenario::MixedCandidateThirty {
                                let spent = report.level_up_captures + report.reward_captures;
                                assert!(spent < 4, "mixed kinds at acquisition thirty must exhaust their separate four-capture allowance");
                                if spent.is_multiple_of(2) {
                                    report.level_up_captures += 1;
                                    (CaptureKind::LevelUp, level_up_png)
                                } else {
                                    report.reward_captures += 1;
                                    (CaptureKind::Reward, REWARD_PNG)
                                }
                            } else if scenario == Scenario::LateLevelUp && report.unsupported_captures < 6 {
                                report.unsupported_captures += 1;
                                (CaptureKind::Unsupported, unsupported_png.as_slice())
                            } else if scenario == Scenario::LevelUpDisappearsBeforeOk && report.level_up_captures >= 2 {
                                report.reward_captures += 1;
                                (CaptureKind::Reward, REWARD_PNG)
                            } else if scenario == Scenario::AlternatingTerminalKinds {
                                let spent = report.reward_captures + report.level_up_captures;
                                assert!(spent < 4, "alternating terminal kinds must exhaust confirmation in four captures");
                                if spent.is_multiple_of(2) {
                                    report.reward_captures += 1;
                                    (CaptureKind::Reward, REWARD_PNG)
                                } else {
                                    report.level_up_captures += 1;
                                    (CaptureKind::LevelUp, level_up_png)
                                }
                            } else if matches!(scenario, Scenario::RewardThenLevelUp | Scenario::MixedStageReset)
                                && report.reward_captures == 0
                            {
                                report.reward_captures += 1;
                                (CaptureKind::Reward, REWARD_PNG)
                            } else {
                                report.level_up_captures += 1;
                                (CaptureKind::LevelUp, level_up_png)
                            }
                        } else if report.downs_at(skip) > 0 {
                            assert_ne!(scenario, Scenario::UncertainSkip, "uncertain skip must stop before any further capture");
                            if scenario == Scenario::SkipDisappears {
                                report.unsupported_captures += 1;
                                (CaptureKind::Unsupported, unsupported_png.as_slice())
                            } else {
                                (CaptureKind::Recovered, recovered_png.as_slice())
                            }
                        } else if matches!(scenario, Scenario::UnknownOnly | Scenario::StopExtendedAcquisition)
                            || (scenario == Scenario::LateReward && report.unsupported_captures < 6)
                            || (scenario == Scenario::TransientReward && report.reward_captures >= 1)
                        {
                            report.unsupported_captures += 1;
                            (CaptureKind::Unsupported, unsupported_png.as_slice())
                        } else if matches!(scenario, Scenario::RedealNoHalo | Scenario::ExtendedRedealNoHalo) {
                            let unknown_limit = if scenario == Scenario::ExtendedRedealNoHalo { 21 } else { 1 };
                            if report.unsupported_captures < unknown_limit {
                                report.unsupported_captures += 1;
                                (CaptureKind::Unsupported, unsupported_png.as_slice())
                            } else if report.downs_at(wire_point(SHARED_SOLVER_CONTROL.click_point)) > 0 {
                                (CaptureKind::Recovered, recovered_png.as_slice())
                            } else {
                                report.nonempty_captures += 1;
                                (CaptureKind::Nonempty, nonempty_png.as_slice())
                            }
                        } else {
                            report.reward_captures += 1;
                            (CaptureKind::Reward, REWARD_PNG)
                        };
                        report.events.push(WireEvent::Capture(kind));
                        // The production capture guard already reserved this file.
                        fs::write(request["arguments"]["filename"].as_str().expect("absolute capture filename"), bytes)
                            .expect("mock QEMU writes original PNG");
                        if scenario == Scenario::StopExtendedAcquisition && report.unsupported_captures == 21 {
                            cancel.store(true, Ordering::Release);
                        }
                        if scenario == Scenario::StopAfterFirstReward && kind == CaptureKind::Reward {
                            cancel.store(true, Ordering::Release);
                        }
                        if scenario == Scenario::SkipDisappears && report.unsupported_captures == 3 {
                            cancel.store(true, Ordering::Release);
                        }
                        if scenario == Scenario::MixedStageReset && report.level_up_captures == 2 {
                            cancel.store(true, Ordering::Release);
                        }
                        if (scenario == Scenario::LevelUpDisappearsBeforeOk && report.reward_captures == 3)
                            || (scenario == Scenario::StaleBareGoldAfterOk && report.bare_gold_captures == 3)
                        {
                            cancel.store(true, Ordering::Release);
                        }
                        ack(&mut stream, id);
                    }
                    "input-send-event" => {
                        let events = request["arguments"]["events"].as_array().expect("QMP input events");
                        let mut has_move = false;
                        let mut down = false;
                        let mut up = false;
                        for event in events {
                            match event["type"].as_str().expect("QMP event type") {
                                "abs" => {
                                    let value = event["data"]["value"].as_i64().expect("absolute axis value");
                                    match event["data"]["axis"].as_str().expect("absolute axis") {
                                        "x" => position.0 = value,
                                        "y" => position.1 = value,
                                        other => panic!("unexpected absolute axis: {other}"),
                                    }
                                    has_move = true;
                                }
                                "btn" => {
                                    assert_eq!(event["data"]["button"], "left");
                                    if event["data"]["down"].as_bool().expect("button direction") { down = true; } else { up = true; }
                                }
                                other => panic!("unexpected gameplay/recovery event: {other}"),
                            }
                        }
                        if has_move { report.events.push(WireEvent::Move(position)); }
                        if down {
                            report.events.push(WireEvent::Down(position));
                            assert!(report.connections == 1, "release-only recovery must never press down");
                        }
                        if up {
                            report.events.push(WireEvent::Up);
                            if report.connections > 1 { report.recovery_releases += 1; }
                        }
                        if down && ((scenario == Scenario::UncertainSkip && position == skip)
                            || (scenario == Scenario::UncertainLevelUp && position == level_up))
                        {
                            assert!(pending_skip_error.is_none() && !error_delivered, "skip down was retried");
                            pending_skip_error = Some(id);
                            // xSAR sends UP before collecting DOWN/UP acknowledgements.
                        } else if up && pending_skip_error.is_some() {
                            let down_id = pending_skip_error.take().expect("pending injected down id");
                            let description = if scenario == Scenario::UncertainLevelUp {
                                "injected uncertain Level Up OK"
                            } else { "injected uncertain reward skip" };
                            packet(&mut stream, json!({"error": {"class": "GenericError", "desc": description}, "id": down_id}));
                            error_delivered = true;
                            // Leave the paired UP unacknowledged. Permit only release recovery.
                        } else if error_delivered && report.connections == 1 {
                            assert!(up && !down && !has_move, "only an old-stream release may follow uncertain skip");
                        } else {
                            ack(&mut stream, id);
                        }
                    }
                    command => panic!("unexpected production QMP command: {command}"),
                }
            }
            if matches!(scenario, Scenario::UncertainSkip | Scenario::UncertainLevelUp)
                && error_delivered && report.connections == 1
            {
                continue;
            }
            break;
        }
        report
    }


    /// Combined wire, worker notification and last original-frame evidence.
    struct ResultEvidence {
        /// Complete semantic command trace from the real transport boundary.
        wire: WireReport,
        /// Ordered production worker notifications.
        events: Vec<WorkerEvent>,
        /// Final bounded preview, preserving the captured pixels.
        latest: LatestWorkerFrame,
        /// Committed board count after the production run stopped.
        completed_boards: usize,
        /// Exact expected actionable post-skip frame.
        recovered: CapturedFrame,
        /// Exact cleared, unsupported animation frame for bounded negative runs.
        unsupported: CapturedFrame,
    }


    /// Run one final-card action through concrete QMP and the shared terminal controller.
    fn run(scenario: Scenario) -> ResultEvidence {
        let directory = SocketDirectory(std::env::temp_dir().join(format!(
            "tripeaks-reward-qmp-{}-{}", std::process::id(), NEXT_REWARD_SOCKET.fetch_add(1, Ordering::Relaxed),
        )));
        fs::create_dir(&directory.0).expect("unique mock QMP directory");
        let socket = directory.0.join("qmp.sock");
        let listener = UnixListener::bind(&socket).expect("mock QMP Unix socket");
        listener.set_nonblocking(true).expect("bounded mock accept");
        let material_effect = scenario != Scenario::InsufficientEffect;
        let (initial, approved) = if level_up_scenario(scenario) {
            last_top_card_at(1, material_effect)
        } else { last_top_card(material_effect) };
        let recovered = tripeaks_stock_frame(true);
        let unsupported = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        let mut nonempty = tripeaks_stock_frame(false);
        let row_four = GameMode::TriPeaks.profile().tableau_rows[3].face_probe_bounds;
        paint(&mut nonempty, PixelRect::new(row_four.x, row_four.y,
            crate::parameters::FACE_UP_WHITE_BLOCK_MIN_WIDTH, row_four.height), [255; 4]);
        let state = TableauScanState::for_mode(GameMode::TriPeaks);
        assert!(matches!(analyse_frame_with_state(&recovered, &state).expect("recovered frame analysis").prediction, PredictedAction::Action(_)));
        let original_reward = decode_png(REWARD_PNG).expect("original reward fixture");
        assert_eq!((original_reward.width, original_reward.height), (NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT));
        assert!(!is_gameplay_scene_for_mode(&original_reward, GameMode::TriPeaks).expect("reward is outside gameplay"));
        let plan = plan_step(approved).expect("canonical final plan");
        let changed = materially_changed_pixels(&initial, &original_reward,
            plan.input().effect_bounds(), plan.input().effect_exclusion_bounds(),
            ACTION_CHANGE_CHANNEL_THRESHOLD).expect("original reward effect comparison");
        if material_effect {
            assert!(changed >= plan.input().minimum_changed_pixels(), "reward proves the final action effect");
        } else {
            assert!(changed < plan.input().minimum_changed_pixels(), "positive terminal artwork cannot satisfy effect proof");
            assert!(changed <= 64, "only the 16x4 card-face evidence changed within the source rectangle");
        }
        let clear_effect = materially_changed_pixels(&initial, &unsupported,
            plan.input().effect_bounds(), plan.input().effect_exclusion_bounds(),
            ACTION_CHANGE_CHANNEL_THRESHOLD).expect("unsupported animation still proves card removal");
        assert!(clear_effect >= plan.input().minimum_changed_pixels());
        let (unsupported_observation, _) = analyse_captured_frame(unsupported.clone(), &state)
            .expect("native unsupported animation classification");
        assert!(!unsupported_observation.gameplay_scene);
        assert_eq!(unsupported_observation.prediction, PredictedAction::NoHighlight);
        assert_eq!(unsupported_observation.observed_rows, None);
        let (nonempty_observation, _) = analyse_captured_frame(nonempty.clone(), &state)
            .expect("positive native redeal classification");
        assert!(nonempty_observation.gameplay_scene);
        assert_eq!(nonempty_observation.prediction, PredictedAction::NoHighlight);
        assert_eq!(nonempty_observation.observed_rows, RowMask::single_for(4, 4));
        let initial_png = encode_native_png(&initial);
        let recovered_png = encode_native_png(&recovered);
        let unsupported_png = encode_native_png(&unsupported);
        let nonempty_png = if matches!(scenario, Scenario::RedealNoHalo | Scenario::ExtendedRedealNoHalo)
            || level_up_scenario(scenario)
        {
            encode_native_png(&nonempty)
        } else { Vec::new() };
        let (new_game_png, play_png) = if level_up_scenario(scenario) {
            let native_bytes = if late_level_up_scenario(scenario) { LATE_LEVEL_UP_PNG } else { LEVEL_UP_PNG };
            let level = decode_png(native_bytes).expect("original native Level Up");
            let changed = materially_changed_pixels(&initial, &level,
                plan.input().effect_bounds(), plan.input().effect_exclusion_bounds(),
                ACTION_CHANGE_CHANNEL_THRESHOLD).expect("native Level Up final-card effect proof");
            assert!(changed >= plan.input().minimum_changed_pixels());
            (encode_native_png(&dialog_observation(&NEW_GAME_CONTROL_VARIANTS).frame),
                encode_native_png(&dialog_observation(&PLAY_CONTROL_VARIANTS).frame))
        } else { (Vec::new(), Vec::new()) };
        let bare_gold_png = if scenario == Scenario::StaleBareGoldAfterOk {
            encode_native_png(&dialog_observation(&[native_level_up_control()]).frame)
        } else { Vec::new() };
        let pngs = MockPngs { initial_png, recovered_png, unsupported_png, nonempty_png,
            new_game_png, play_png, bare_gold_png };
        let cancel = Arc::new(AtomicBool::new(false));
        let server_cancel = Arc::clone(&cancel);
        let server = thread::spawn(move || serve(listener, pngs, scenario, server_cancel));
        let (events, receiver) = mpsc::channel();
        let sink = WorkerEventSink {
            events, latest_frame: LatestFrameSlot::default(),
            capture_context: Mutex::new(Some(vision_context(&socket, GameMode::TriPeaks))),
        };
        let mut scan_state = TableauScanState::for_mode(GameMode::TriPeaks);
        let mut completed_boards = GameMode::TriPeaks.profile().boards_per_game - 1;
        let delays = AnimationSettleDelays::from_millis(0, 0, 0).with_tripeaks_reobserve_millis(0);
        run_execute_steps(socket, approved, StepRunSettings::new(delays, 1),
            &mut scan_state, &mut completed_boards, &sink, &cancel);
        let wire = server.join().expect("production worker mock QMP server");
        let events = receiver.try_iter().map(|event| event.event).collect();
        let latest = sink.latest_frame.take().expect("worker retains latest original capture");
        ResultEvidence { wire, events, latest, completed_boards, recovered, unsupported }
    }


    /// Count positively accepted gameplay actions without counting terminal setup.
    fn action_count(evidence: &ResultEvidence) -> usize {
        evidence.events.iter().filter(|event| matches!(event, WorkerEvent::ActionCompleted { .. })).count()
    }
    /// Reject replay, Solver input or any control outside the final source and centre skip.
    fn assert_only_original_and_skip(evidence: &ResultEvidence, skips: usize) {
        let source = wire_point(TABLEAU_CARD_REGIONS[2].click_point);
        let skip = wire_point(SCORE_SKIP_CONTROL.click_point);
        assert_eq!(evidence.wire.downs_at(source), 1, "original gameplay source was delivered once");
        assert_eq!(evidence.wire.downs_at(skip), skips, "centre skip count");
        assert_eq!(evidence.wire.downs_at(wire_point(SHARED_SOLVER_CONTROL.click_point)), 0, "reward recovery sends no Solver refresh");
        assert_eq!(evidence.wire.downs(), 1 + skips, "no other gameplay or terminal input");
    }


    /// Two fresh original terminal observations establish effect proof before one skip.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_original_reward_two_fresh_captures_skip_once_and_recover_through_qmp() {
        let evidence = run(Scenario::Recover);
        assert_only_original_and_skip(&evidence, 1);
        let skip = wire_point(SCORE_SKIP_CONTROL.click_point);
        let source = wire_point(TABLEAU_CARD_REGIONS[2].click_point);
        let source_down = evidence.wire.events.iter().position(|event| *event == WireEvent::Down(source)).expect("source input");
        let skip_down = evidence.wire.events.iter().position(|event| *event == WireEvent::Down(skip)).expect("skip input");
        let confirmations = evidence.wire.events[source_down + 1..skip_down].iter()
            .filter(|event| **event == WireEvent::Capture(CaptureKind::Reward)).count();
        assert!(confirmations >= 2, "two separate reward screendumps preceded skip authority");
        assert!(confirmations <= 6, "confirmation and shared terminal entry remain bounded");
        assert_eq!(action_count(&evidence), 1);
        assert!(evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::ActionCompleted { changed_pixels, continued_from_halo: false, .. }
            if *changed_pixels >= crate::parameters::MINIMUM_TABLEAU_CHANGED_PIXELS)));
        assert!(evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::BoardProgress { completed_boards: 3, .. })));
        assert!(evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::RunCompleted { requested_operations: 1, verified_operations: 1, .. })));
        assert_eq!(evidence.completed_boards, 0, "actionable recovered board clears stale completion state");
        assert_eq!(evidence.latest.frame.pixels, evidence.recovered.pixels);
        assert!(matches!(evidence.latest.prediction, Some(PredictedAction::Action(_))));
    }


    /// Classify immutable native frames and build the actual production final-card proof input.
    fn native_reward_proof_pair(material_effect: bool) -> (StepPlan, FrameObservation, FrameObservation) {
        let (frame, expected) = last_top_card(material_effect);
        let scan = TableauScanState::for_mode(GameMode::TriPeaks);
        let (before, _) = analyse_captured_frame(frame, &scan).expect("fresh planning classification");
        let (after, _) = analyse_captured_frame(decode_png(REWARD_PNG).unwrap(), &scan)
            .expect("fresh result classification");
        (plan_step(expected).unwrap(), before, after)
    }


    /// Native proof accepts the supplied panel only with actual sole-card context and material effect.
    #[test]
    fn tripeaks_reward_native_proof_accepts_final_card_and_actual_effect() {
        let (plan, before, after) = native_reward_proof_pair(true);
        let (kind, changed) = tripeaks_terminal::terminal_action_evidence(plan, &before, &after)
            .expect("supported final-card proof").expect("positive native panel");
        assert_eq!(kind, tripeaks_terminal::TerminalKind::RewardSkip);
        assert!(changed >= plan.input().minimum_changed_pixels());
        let mut confirmation = tripeaks_terminal::TerminalConfirmation::default();
        assert_eq!(confirmation.observe(Some(tripeaks_terminal::TerminalKind::RewardSkip)), tripeaks_terminal::TerminalConfirmationStep::Observe);
        let (_, _, independently_classified) = native_reward_proof_pair(true);
        assert!(tripeaks_terminal::terminal_action_evidence(plan, &before, &independently_classified).unwrap().is_some());
        assert_eq!(confirmation.observe(Some(tripeaks_terminal::TerminalKind::RewardSkip)), tripeaks_terminal::TerminalConfirmationStep::Confirmed(tripeaks_terminal::TerminalKind::RewardSkip));
    }


    /// Classify the actual direct Level Up result against an independently acquired r1c2 source.
    fn native_level_up_proof_pair(material_effect: bool) -> (StepPlan, FrameObservation, FrameObservation) {
        let scan = TableauScanState::for_mode(GameMode::TriPeaks);
        let (frame, expected) = if material_effect {
            last_top_card_at(1, true)
        } else {
            let mut frame = decode_png(LEVEL_UP_PNG).unwrap();
            paint(&mut frame, GAMEPLAY_FELT_PROBE_BOUNDS, [20, 140, 60, 255]);


            for row in GameMode::TriPeaks.profile().tableau_rows.iter().skip(1) {
                paint(&mut frame, row.face_probe_bounds, [0, 0, 0, 255]);
            }
            let card = TABLEAU_CARD_REGIONS[1];
            let face = GameMode::TriPeaks.profile().tableau_rows[0].face_probe_bounds;
            paint(&mut frame, PixelRect::new(card.card_bounds.x + 5, face.y,
                crate::parameters::FACE_UP_WHITE_BLOCK_MIN_WIDTH, face.height), [255; 4]);
            let anchor = PixelPoint::new(card.click_point.x - CLICK_OFFSET_X,
                (card.card_bounds.bottom() + HALO_GOLD_LINE_OFFSET_Y) as i32);
            let gold = GOLD_RGB_CANDIDATES[0];
            paint(&mut frame, PixelRect::new(anchor.x as u32, anchor.y as u32,
                HALO_GOLD_RUN_MIN + 1, 1), [gold[0], gold[1], gold[2], 255]);
            let action = GameMode::TriPeaks.profile().tableau_action(1, anchor).unwrap();
            (frame, PredictedAction::Action(action))
        };
        let (before, _) = analyse_captured_frame(frame, &scan).unwrap();
        assert_eq!(before.prediction, expected);
        assert!(tripeaks_terminal::last_tableau_context(plan_step(expected).unwrap(), &before).unwrap());
        let (after, _) = analyse_captured_frame(decode_png(LEVEL_UP_PNG).unwrap(), &scan).unwrap();
        (plan_step(expected).unwrap(), before, after)
    }


    /// The supplied Level Up uses the real final-card proof and excludes advisory preview authority.
    #[test]
    fn tripeaks_level_up_native_proof_accepts_r1c2_final_card_and_actual_effect() {
        let (plan, before, after) = native_level_up_proof_pair(true);
        let (kind, changed) = tripeaks_terminal::terminal_action_evidence(plan, &before, &after).unwrap().unwrap();
        assert_eq!(kind, tripeaks_terminal::TerminalKind::LevelUpOk);
        assert!(changed >= plan.input().minimum_changed_pixels());
        let mut confirmation = tripeaks_terminal::TerminalConfirmation::default();
        assert_eq!(confirmation.observe(Some(kind)), tripeaks_terminal::TerminalConfirmationStep::Observe);
        let (_, _, independent) = native_level_up_proof_pair(true);
        assert!(tripeaks_terminal::terminal_action_evidence(plan, &before, &independent).unwrap().is_some());
        assert_eq!(confirmation.observe(Some(kind)), tripeaks_terminal::TerminalConfirmationStep::Confirmed(kind));
    }


    /// The original late screenshot needs the same actual final-card effect and same-kind confirmations.
    #[test]
    fn tripeaks_level_up_tp03_native_final_card_proof_preserves_confirmation_policy() {
        let (plan, before, _) = native_level_up_proof_pair(true);
        let scan = TableauScanState::for_mode(GameMode::TriPeaks);
        let png = include_bytes!("../../tests/fixtures/tripeaks-TP03.png");
        let (after, _) = analyse_captured_frame(decode_png(png).unwrap(), &scan).unwrap();
        let (kind, changed) = tripeaks_terminal::terminal_action_evidence(plan, &before, &after).unwrap().unwrap();
        assert_eq!(kind, tripeaks_terminal::TerminalKind::LevelUpOk);
        assert!(changed >= plan.input().minimum_changed_pixels());
        let mut confirmation = tripeaks_terminal::TerminalConfirmation::default();
        assert_eq!(confirmation.observe(Some(kind)), tripeaks_terminal::TerminalConfirmationStep::Observe);
        let (fresh, _) = analyse_captured_frame(decode_png(png).unwrap(), &scan).unwrap();
        let (fresh_kind, _) = tripeaks_terminal::terminal_action_evidence(plan, &before, &fresh).unwrap().unwrap();
        assert_eq!(confirmation.observe(Some(fresh_kind)), tripeaks_terminal::TerminalConfirmationStep::Confirmed(kind));
    }


    /// Positive modal artwork cannot replace the unchanged last-action effect floor.
    #[test]
    fn tripeaks_level_up_native_proof_rejects_insufficient_last_action_effect() {
        let (plan, before, after) = native_level_up_proof_pair(false);
        let changed = materially_changed_pixels(&before.frame, &after.frame,
            plan.input().effect_bounds(), plan.input().effect_exclusion_bounds(), ACTION_CHANGE_CHANNEL_THRESHOLD).unwrap();
        assert!(changed <= 64);
        assert!(changed < plan.input().minimum_changed_pixels());
        let error = tripeaks_terminal::terminal_action_evidence(plan, &before, &after).unwrap_err();
        assert!(error.contains("effect") && error.contains("required="));
    }


    /// Draw, foreign or forged actions and multiple exposed cards cannot adopt direct Level Up authority.
    #[test]
    fn tripeaks_level_up_native_proof_refuses_unrelated_noncanonical_or_nonfinal_context() {
        let (plan, before, after) = native_level_up_proof_pair(true);
        let (stock, _) = analyse_captured_frame(tripeaks_stock_frame(true),
            &TableauScanState::for_mode(GameMode::TriPeaks)).unwrap();
        assert!(tripeaks_terminal::terminal_action_evidence(plan_step(stock.prediction).unwrap(), &stock, &after).is_err());
        let foreign = plan_step(PredictedAction::Action(pyramid::action_for_kind(PyramidTargetKind::Move).unwrap())).unwrap();
        assert!(tripeaks_terminal::terminal_action_evidence(foreign, &before, &after).is_err());
        let mut forged = plan.input().action();
        forged.specification.operation = InputOperation::PressDrawKey;
        assert!(tripeaks_terminal::terminal_action_evidence(plan_step(PredictedAction::Action(forged)).unwrap(), &before, &after).is_err());
        let mut frame = before.frame;
        paint(&mut frame, TABLEAU_CARD_REGIONS[2].card_bounds, [255; 4]);
        let scan = TableauScanState::for_mode(GameMode::TriPeaks);
        let (before, _) = analyse_captured_frame(frame, &scan).unwrap();
        assert_eq!(before.prediction, plan.before());
        assert_eq!(analyse_frame_with_state(&before.frame, &scan).unwrap().top_row_face_up_count, 2);
        assert!(tripeaks_terminal::terminal_action_evidence(plan, &before, &after).is_err());
    }


    /// Real fresh gameplay targets or positive row evidence remain separate from terminal fingerprints.
    #[test]
    fn tripeaks_level_up_native_proof_refuses_fresh_gameplay_target_or_rows() {
        let (plan, before, _) = native_level_up_proof_pair(true);
        let scan = TableauScanState::for_mode(GameMode::TriPeaks);


        for with_halo in [false, true] {
            let mut frame = decode_png(LEVEL_UP_PNG).unwrap();
            paint(&mut frame, GAMEPLAY_FELT_PROBE_BOUNDS, [20, 140, 60, 255]);
            let card = TABLEAU_CARD_REGIONS[1];
            let face = GameMode::TriPeaks.profile().tableau_rows[0].face_probe_bounds;
            paint(&mut frame, PixelRect::new(card.card_bounds.x + 5, face.y,
                crate::parameters::FACE_UP_WHITE_BLOCK_MIN_WIDTH, face.height), [255; 4]);


            if with_halo {
                let anchor = plan.input().action().anchor;
                let gold = GOLD_RGB_CANDIDATES[0];
                paint(&mut frame, PixelRect::new(anchor.x as u32, anchor.y as u32,
                    HALO_GOLD_RUN_MIN + 1, 1), [gold[0], gold[1], gold[2], 255]);
            }
            assert_eq!(tripeaks_terminal::is_direct_level_up_overlay(&frame), Ok(true));
            let (after, _) = analyse_captured_frame(frame, &scan).unwrap();
            assert!(after.gameplay_scene && after.observed_rows.is_some());
            assert_eq!(matches!(after.prediction, PredictedAction::Action(_)), with_halo);
            assert!(tripeaks_terminal::terminal_action_evidence(plan, &before, &after).is_err());
        }
    }


    /// Native original pixels with a tiny source-probe change cannot meet the existing effect floor.
    #[test]
    fn tripeaks_reward_native_proof_rejects_insufficient_effect() {
        let (plan, before, after) = native_reward_proof_pair(false);
        let changed = materially_changed_pixels(&before.frame, &after.frame,
            plan.input().effect_bounds(), plan.input().effect_exclusion_bounds(), ACTION_CHANGE_CHANNEL_THRESHOLD).unwrap();
        assert!(changed <= 64);
        assert!(changed < plan.input().minimum_changed_pixels());
        let error = tripeaks_terminal::terminal_action_evidence(plan, &before, &after).unwrap_err();
        assert!(error.contains("effect") && error.contains("required="));
    }


    /// Draw, foreign actions and forged top-row input definitions cannot adopt reward authority.
    #[test]
    fn tripeaks_reward_native_proof_refuses_unrelated_or_noncanonical_context() {
        let (plan, before, after) = native_reward_proof_pair(true);
        let (stock, _) = analyse_captured_frame(tripeaks_stock_frame(true),
            &TableauScanState::for_mode(GameMode::TriPeaks)).unwrap();
        let draw = plan_step(stock.prediction).unwrap();
        assert!(tripeaks_terminal::terminal_action_evidence(draw, &stock, &after).is_err());
        let foreign = plan_step(PredictedAction::Action(
            pyramid::action_for_kind(PyramidTargetKind::Move).unwrap())).unwrap();
        assert!(tripeaks_terminal::terminal_action_evidence(foreign, &before, &after).is_err());
        let mut forged = plan.input().action();
        forged.specification.operation = InputOperation::PressDrawKey;
        let forged_plan = plan_step(PredictedAction::Action(forged)).unwrap();
        assert!(tripeaks_terminal::terminal_action_evidence(forged_plan, &before, &after).is_err());
    }


    /// Actual fresh gameplay classifications and positive row evidence cannot become terminal input.
    #[test]
    fn tripeaks_reward_native_proof_refuses_fresh_target_and_nonempty_rows() {
        let (plan, before, _) = native_reward_proof_pair(true);
        let scan = TableauScanState::for_mode(GameMode::TriPeaks);


        for with_halo in [false, true] {
            let mut frame = decode_png(REWARD_PNG).unwrap();
            paint(&mut frame, GAMEPLAY_FELT_PROBE_BOUNDS, [20, 140, 60, 255]);
            let card = TABLEAU_CARD_REGIONS[2];
            let face = GameMode::TriPeaks.profile().tableau_rows[0].face_probe_bounds;
            paint(&mut frame, PixelRect::new(card.card_bounds.x + 5, face.y,
                crate::parameters::FACE_UP_WHITE_BLOCK_MIN_WIDTH, face.height), [255; 4]);


            if with_halo {
                let anchor = plan.input().action().anchor;
                let gold = GOLD_RGB_CANDIDATES[0];
                paint(&mut frame, PixelRect::new(anchor.x as u32, anchor.y as u32,
                    HALO_GOLD_RUN_MIN + 1, 1), [gold[0], gold[1], gold[2], 255]);
            }
            assert_eq!(tripeaks_terminal::is_reward_overlay(&frame), Ok(true));
            let (after, _) = analyse_captured_frame(frame, &scan).unwrap();
            assert!(after.gameplay_scene && after.observed_rows.is_some());
            assert_eq!(matches!(after.prediction, PredictedAction::Action(_)), with_halo);
            assert!(tripeaks_terminal::terminal_action_evidence(plan, &before, &after).is_err());
        }
    }


    /// A second genuinely exposed top-row card prevents the sole-final-card completion shortcut.
    #[test]
    fn tripeaks_reward_native_proof_refuses_another_exposed_card() {
        let (plan, before, after) = native_reward_proof_pair(true);
        let mut frame = before.frame;
        paint(&mut frame, TABLEAU_CARD_REGIONS[1].card_bounds, [255; 4]);
        let scan = TableauScanState::for_mode(GameMode::TriPeaks);
        let (before, _) = analyse_captured_frame(frame, &scan).unwrap();
        assert_eq!(before.prediction, plan.before());
        assert_eq!(analyse_frame_with_state(&before.frame, &scan).unwrap().top_row_face_up_count, 2);
        assert!(tripeaks_terminal::terminal_action_evidence(plan, &before, &after).is_err());
    }


    /// The synthetic host-QMP fixture encoder is verified independently of socket availability.
    #[test]
    fn tripeaks_reward_native_qmp_fixture_png_roundtrip_preserves_context() {
        let (frame, expected) = last_top_card(true);
        let decoded = decode_png(&encode_native_png(&frame)).unwrap();
        assert_eq!(decoded.pixels, frame.pixels);
        let (observation, _) = analyse_captured_frame(decoded,
            &TableauScanState::for_mode(GameMode::TriPeaks)).unwrap();
        assert_eq!(observation.prediction, expected);
        assert!(tripeaks_terminal::last_tableau_context(plan_step(expected).unwrap(), &observation).unwrap());
    }


    /// STOP after one candidate retains original pixels without committing a win.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_stop_after_first_original_reward_capture_sends_no_terminal_input() {
        let evidence = run(Scenario::StopAfterFirstReward);
        assert_only_original_and_skip(&evidence, 0);
        assert_eq!(evidence.wire.reward_captures, 1);
        assert_eq!(action_count(&evidence), 0);
        assert_eq!(evidence.completed_boards, 2, "one terminal candidate cannot commit completion");
        assert_eq!(evidence.latest.frame.pixels, decode_png(REWARD_PNG).unwrap().pixels);
        assert!(evidence.events.iter().any(|event| matches!(event, WorkerEvent::Log(message) if message.contains("STOP"))));
    }


    /// STOP after the fresh terminal probe closes the final pre-input race.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_stop_after_fresh_skip_probe_sends_no_centre_press() {
        let evidence = run(Scenario::StopAfterSkipProbe);
        assert_only_original_and_skip(&evidence, 0);
        assert!(evidence.wire.stopped_after_probe, "STOP was raised at the terminal pre-input probe");
        assert!(evidence.wire.reward_captures >= 2);
        assert_eq!(action_count(&evidence), 1, "final gameplay action was already effect verified");
        assert!(!evidence.events.iter().any(|event| matches!(event, WorkerEvent::RunCompleted { .. })));
        assert_eq!(evidence.latest.frame.pixels, decode_png(REWARD_PNG).unwrap().pixels);
    }


    /// Uncertain skip acknowledgement permits only releases and stops the controller.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_uncertain_reward_skip_allows_release_only_and_never_repeats_input() {
        let evidence = run(Scenario::UncertainSkip);
        assert_only_original_and_skip(&evidence, 1);
        assert_eq!(action_count(&evidence), 1);
        assert_eq!(evidence.wire.connections, 2, "xSAR opened only its release-recovery connection");
        assert_eq!(evidence.wire.recovery_releases, 1);
        assert!(!evidence.wire.events.contains(&WireEvent::Capture(CaptureKind::Recovered)));
        assert!(!evidence.events.iter().any(|event| matches!(event, WorkerEvent::RunCompleted { .. })));
        assert!(evidence.events.iter().any(|event| matches!(event, WorkerEvent::State(WorkerState::Uncertain))));
        assert!(evidence.events.iter().any(|event| matches!(event, WorkerEvent::Log(message)
            if message.contains("injected uncertain reward skip") && message.contains("uncertain"))));
        assert_eq!(evidence.latest.frame.pixels, decode_png(REWARD_PNG).unwrap().pixels);
    }


    /// Reward animation may arrive after six unsupported result frames without input.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_late_reward_outlasts_four_captures_then_skips_once_through_qmp() {
        let evidence = run(Scenario::LateReward);
        assert_only_original_and_skip(&evidence, 1);
        assert_eq!(evidence.wire.unsupported_captures, 6);
        assert!(evidence.wire.unsupported_captures > 4, "terminal animation has a distinct bounded allowance");
        let first_reward = evidence.wire.events.iter().position(|event|
            *event == WireEvent::Capture(CaptureKind::Reward)).expect("late original reward capture");
        assert_eq!(evidence.wire.events[..first_reward].iter().filter(|event|
            **event == WireEvent::Capture(CaptureKind::Unsupported)).count(), 6);
        assert_eq!(evidence.wire.events[..first_reward].iter().filter(|event|
            matches!(event, WireEvent::Down(_))).count(), 1, "late animation remains input-free after the final card");
        assert_eq!(action_count(&evidence), 1);
        assert!(evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::RunCompleted { requested_operations: 1, verified_operations: 1, .. })));
        assert_eq!(evidence.completed_boards, 0);
        assert_eq!(evidence.latest.frame.pixels, evidence.recovered.pixels);
    }


    /// Thirty qualified unsupported acquisitions stop before completion or terminal input.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_unknown_terminal_animation_stops_at_thirty_fresh_qmp_captures() {
        let evidence = run(Scenario::UnknownOnly);
        assert_only_original_and_skip(&evidence, 0);
        assert_eq!(evidence.wire.unsupported_captures, 30);
        assert_eq!(evidence.wire.reward_captures, 0);
        assert_eq!(action_count(&evidence), 0);
        assert_eq!(evidence.completed_boards, 2, "unknown frames cannot commit completion");
        assert_eq!(evidence.latest.frame.pixels, evidence.unsupported.pixels);
        assert!(!evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::RunCompleted { .. } | WorkerEvent::GameCompleted)));
        assert!(evidence.events.iter().any(|event| matches!(event, WorkerEvent::State(WorkerState::Uncertain))));
    }


    /// One disappearing reward candidate cannot survive its four-capture confirmation budget.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_transient_reward_stops_after_four_confirmation_captures_without_skip() {
        let evidence = run(Scenario::TransientReward);
        assert_only_original_and_skip(&evidence, 0);
        assert_eq!(evidence.wire.reward_captures, 1);
        assert_eq!(evidence.wire.unsupported_captures, 3);
        assert_eq!(action_count(&evidence), 0);
        assert_eq!(evidence.completed_boards, 2);
        assert_eq!(evidence.latest.frame.pixels, evidence.unsupported.pixels);
        assert!(!evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::RunCompleted { .. } | WorkerEvent::GameCompleted)));
        assert!(evidence.events.iter().any(|event| matches!(event, WorkerEvent::State(WorkerState::Uncertain))));
    }


    /// A second non-gameplay frame cannot retry skip without fresh positive reward evidence.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_reward_disappearing_after_skip_refuses_centre_retry_through_qmp() {
        let evidence = run(Scenario::SkipDisappears);
        assert_only_original_and_skip(&evidence, 1);
        assert_eq!(evidence.wire.unsupported_captures, 3);
        let skip = wire_point(SCORE_SKIP_CONTROL.click_point);
        let skip_down = evidence.wire.events.iter().position(|event|
            *event == WireEvent::Down(skip)).expect("first acknowledged skip");
        let unsupported = evidence.wire.events[skip_down + 1..].iter().filter(|event|
            **event == WireEvent::Capture(CaptureKind::Unsupported)).count();
        assert_eq!(unsupported, 3, "two input-free unknown rounds preceded STOP on the third");
        assert!(!evidence.wire.events[skip_down + 1..].iter().any(|event|
            matches!(event, WireEvent::Down(_))), "fresh unknown frames never authorise a retry");
        assert_eq!(action_count(&evidence), 1);
        assert_eq!(evidence.completed_boards, 3, "only the already-confirmed win remains committed");
        assert_eq!(evidence.latest.frame.pixels, evidence.unsupported.pixels);
        assert!(!evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::RunCompleted { .. } | WorkerEvent::GameCompleted)));
        assert!(evidence.events.iter().any(|event| matches!(event, WorkerEvent::Log(message) if message.contains("STOP"))));
    }


    /// Original reward recognition cannot replace the previous action's material effect proof.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_original_reward_with_insufficient_source_effect_refuses_completion_through_qmp() {
        let evidence = run(Scenario::InsufficientEffect);
        assert_only_original_and_skip(&evidence, 0);
        assert_eq!(evidence.wire.reward_captures, 1);
        assert_eq!(action_count(&evidence), 0);
        assert_eq!(evidence.completed_boards, 2);
        assert_eq!(evidence.latest.frame.pixels, decode_png(REWARD_PNG).unwrap().pixels);
        assert!(!evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::RunCompleted { .. } | WorkerEvent::GameCompleted)));
        assert!(evidence.events.iter().any(|event| matches!(event, WorkerEvent::State(WorkerState::Uncertain))));
        assert!(evidence.events.iter().any(|event| matches!(event, WorkerEvent::Log(message)
            if message.contains("effect") && message.contains("required="))));
    }


    /// Positive no-HALO gameplay clears terminal waiting and preserves the existing redeal policy.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_nonempty_no_halo_redeal_resumes_existing_solver_recovery_through_qmp() {
        let evidence = run(Scenario::RedealNoHalo);
        let source = wire_point(TABLEAU_CARD_REGIONS[2].click_point);
        let solver = wire_point(SHARED_SOLVER_CONTROL.click_point);
        let skip = wire_point(SCORE_SKIP_CONTROL.click_point);
        assert_eq!(evidence.wire.downs_at(source), 1);
        assert_eq!(evidence.wire.downs_at(solver), 1);
        assert_eq!(evidence.wire.downs_at(skip), 0);
        assert_eq!(evidence.wire.downs(), 2, "only the original final-row source and existing Solver recovery were delivered");
        assert_eq!(evidence.wire.unsupported_captures, 1);
        assert_eq!(evidence.wire.reward_captures, 0);
        assert!(evidence.wire.nonempty_captures > 0);
        let solver_down = evidence.wire.events.iter().position(|event|
            *event == WireEvent::Down(solver)).expect("existing Solver recovery");
        assert!(evidence.wire.events[..solver_down].contains(&WireEvent::Capture(CaptureKind::Nonempty)),
            "positive nonempty gameplay precedes Solver authority");
        assert_eq!(action_count(&evidence), 1);
        assert!(evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::ActionCompleted { changed_pixels, continued_from_halo: false, .. }
            if *changed_pixels >= crate::parameters::MINIMUM_TABLEAU_CHANGED_PIXELS)));
        assert!(evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::RunCompleted { requested_operations: 1, verified_operations: 1, .. })));
        assert_eq!(evidence.completed_boards, 2, "redeal evidence cannot force a game win");
        assert!(!evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::BoardProgress { completed_boards: 3, .. } | WorkerEvent::GameCompleted)));
        assert_eq!(evidence.latest.frame.pixels, evidence.recovered.pixels);
    }


    /// Successful native OK handover follows only the supported later-stage controls.
    fn assert_level_up_restart(evidence: &ResultEvidence) {
        let expected = [
            wire_point(TABLEAU_CARD_REGIONS[1].click_point),
            wire_point(native_level_up_control().click_point),
            wire_point(NEW_GAME_CONTROL_VARIANTS[0].click_point),
            wire_point(PLAY_CONTROL_VARIANTS[0].click_point),
            wire_point(SHARED_SOLVER_CONTROL.click_point),
        ];
        let actual: Vec<_> = evidence.wire.events.iter().filter_map(|event| {
            if let WireEvent::Down(point) = event { Some(*point) } else { None }
        }).collect();
        assert_eq!(actual, expected, "one final r1c2 click and one ordered OK/NewGame/Play/Solver sequence");
        assert_eq!(evidence.wire.downs_at(wire_point(SCORE_SKIP_CONTROL.click_point)), 0,
            "native Level Up bypasses centre skipping");
        let ok_index = evidence.wire.events.iter().position(|event|
            *event == WireEvent::Down(expected[1])).expect("native OK input");
        assert!(evidence.wire.events[..ok_index].iter().filter(|event|
            **event == WireEvent::Capture(CaptureKind::LevelUp)).count() >= 2,
            "fresh same-kind native observations precede OK");
        assert_eq!(action_count(evidence), 1);
        assert_eq!(evidence.events.iter().filter(|event| matches!(event,
            WorkerEvent::GameCompleted)).count(), 1);
        assert!(evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::RunCompleted { requested_operations: 1, verified_operations: 1, .. })));
        assert_eq!(evidence.completed_boards, 0);
        assert_eq!(evidence.latest.frame.pixels, evidence.recovered.pixels);
        assert!(matches!(evidence.latest.prediction, Some(PredictedAction::Action(_))));
    }


    /// Native Level Up is directly confirmed and handed to the complete supported restart flow.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_native_level_up_enters_ok_new_game_play_solver_sequence_through_qmp() {
        let evidence = run(Scenario::DirectLevelUp);
        assert_eq!(evidence.wire.reward_captures, 0);
        assert_level_up_restart(&evidence);
    }


    /// STOP after the live OK pointer probe sends neither OK nor later-stage controls.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_native_level_up_stop_after_fresh_ok_probe_refuses_button_input() {
        let evidence = run(Scenario::StopAfterLevelUpProbe);
        assert!(evidence.wire.stopped_after_level_up_probe);
        assert_eq!(evidence.wire.downs(), 1);
        assert_eq!(evidence.wire.downs_at(wire_point(TABLEAU_CARD_REGIONS[1].click_point)), 1);
        assert_eq!(evidence.wire.downs_at(wire_point(native_level_up_control().click_point)), 0);
        assert_eq!(evidence.wire.downs_at(wire_point(SCORE_SKIP_CONTROL.click_point)), 0);
        assert_eq!(action_count(&evidence), 1);
        assert_eq!(evidence.completed_boards, 3);
        assert_eq!(evidence.latest.frame.pixels, decode_png(LEVEL_UP_PNG).unwrap().pixels);
        assert!(!evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::RunCompleted { .. } | WorkerEvent::GameCompleted)));
    }


    /// A native OK delivery error permits release-only recovery and no replay or later input.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_native_level_up_uncertain_ok_stops_without_replay_through_qmp() {
        let evidence = run(Scenario::UncertainLevelUp);
        assert_eq!(evidence.wire.downs(), 2);
        assert_eq!(evidence.wire.downs_at(wire_point(TABLEAU_CARD_REGIONS[1].click_point)), 1);
        assert_eq!(evidence.wire.downs_at(wire_point(native_level_up_control().click_point)), 1);
        assert_eq!(evidence.wire.downs_at(wire_point(SCORE_SKIP_CONTROL.click_point)), 0);
        assert_eq!(evidence.wire.connections, 2);
        assert_eq!(evidence.wire.recovery_releases, 1);
        assert_eq!(action_count(&evidence), 1);
        assert_eq!(evidence.completed_boards, 3);
        assert!(!evidence.wire.events.contains(&WireEvent::Capture(CaptureKind::NewGame)));
        assert!(!evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::RunCompleted { .. } | WorkerEvent::GameCompleted)));
        assert!(evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::State(WorkerState::Uncertain))));
        assert!(evidence.events.iter().any(|event| matches!(event, WorkerEvent::Log(message)
            if message.contains("injected uncertain Level Up OK") && message.contains("uncertain"))));
        assert_eq!(evidence.latest.frame.pixels, decode_png(LEVEL_UP_PNG).unwrap().pixels);
    }


    /// A reward candidate followed by Level Up changes terminal kind and still completes once.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_reward_then_native_level_up_uses_fresh_same_kind_confirmation_through_qmp() {
        let evidence = run(Scenario::RewardThenLevelUp);
        assert_eq!(evidence.wire.reward_captures, 1);
        assert_level_up_restart(&evidence);
    }


    /// One reward and one Level Up cannot commit a mixed-stage win before the next fresh capture.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_reward_to_level_up_reset_prevents_premature_commit_through_qmp() {
        let evidence = run(Scenario::MixedStageReset);
        assert_eq!(evidence.wire.reward_captures, 1);
        assert_eq!(evidence.wire.level_up_captures, 2);
        assert_eq!(evidence.wire.downs(), 1);
        assert_eq!(evidence.wire.downs_at(wire_point(TABLEAU_CARD_REGIONS[1].click_point)), 1);
        assert_eq!(action_count(&evidence), 0,
            "reward plus first Level Up must not emit ActionCompleted before the stopped second Level Up");
        assert_eq!(evidence.completed_boards, 2);
        assert!(!evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::RunCompleted { .. } | WorkerEvent::GameCompleted
            | WorkerEvent::BoardProgress { completed_boards: 3, .. })));
        assert_eq!(evidence.latest.frame.pixels, decode_png(LEVEL_UP_PNG).unwrap().pixels);
    }


    /// Direct native Level Up can arrive after six input-free unsupported animation frames.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_late_native_level_up_outlasts_six_unknown_captures_then_restarts_through_qmp() {
        let evidence = run(Scenario::LateLevelUp);
        assert_eq!(evidence.wire.unsupported_captures, 6);
        assert_eq!(evidence.wire.reward_captures, 0);
        let first_level = evidence.wire.events.iter().position(|event|
            *event == WireEvent::Capture(CaptureKind::LevelUp)).expect("late native Level Up");
        assert_eq!(evidence.wire.events[..first_level].iter().filter(|event|
            matches!(event, WireEvent::Down(_))).count(), 1);
        assert_level_up_restart(&evidence);
    }


    /// Acknowledged OK may uncover reward; two fresh positives precede one guarded skip.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_native_level_up_ok_uncovers_reward_then_skips_and_restarts_through_qmp() {
        let evidence = run(Scenario::LevelUpRewardAfterOk);
        let source = wire_point(TABLEAU_CARD_REGIONS[1].click_point);
        let ok = wire_point(native_level_up_control().click_point);
        let skip = wire_point(SCORE_SKIP_CONTROL.click_point);
        let expected = [source, ok, skip, wire_point(NEW_GAME_CONTROL_VARIANTS[0].click_point),
            wire_point(PLAY_CONTROL_VARIANTS[0].click_point), wire_point(SHARED_SOLVER_CONTROL.click_point)];
        let actual: Vec<_> = evidence.wire.events.iter().filter_map(|event| {
            if let WireEvent::Down(point) = event { Some(*point) } else { None }
        }).collect();
        assert_eq!(actual, expected, "OK precedes reward skip, then only supported restart controls");
        let ok_down = evidence.wire.events.iter().position(|event| *event == WireEvent::Down(ok)).unwrap();
        let skip_down = evidence.wire.events.iter().position(|event| *event == WireEvent::Down(skip)).unwrap();
        let ok_up = ok_down + 1 + evidence.wire.events[ok_down + 1..].iter()
            .position(|event| *event == WireEvent::Up).expect("acknowledged OK paired release");
        assert!(skip_down > ok_up);
        assert!(evidence.wire.events[ok_up + 1..skip_down].iter().filter(|event|
            **event == WireEvent::Capture(CaptureKind::Reward)).count() >= 2,
            "both reward captures were acquired after OK acknowledgement");
        assert!(!evidence.wire.events[..ok_up].contains(&WireEvent::Capture(CaptureKind::Reward)),
            "no centre reward authority existed before native OK delivery");
        assert_eq!(evidence.wire.downs_at(skip), 1);
        assert_eq!(action_count(&evidence), 1);
        assert_eq!(evidence.events.iter().filter(|event| matches!(event, WorkerEvent::GameCompleted)).count(), 1);
        assert!(evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::RunCompleted { requested_operations: 1, verified_operations: 1, .. })));
        assert_eq!(evidence.completed_boards, 0);
        assert_eq!(evidence.latest.frame.pixels, evidence.recovered.pixels);
    }


    /// Reward appearing before OK acknowledgement cannot authorise either OK or centre input.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_level_up_disappearing_into_reward_before_ok_refuses_skip_through_qmp() {
        let evidence = run(Scenario::LevelUpDisappearsBeforeOk);
        assert_eq!(evidence.wire.level_up_captures, 2);
        assert_eq!(evidence.wire.reward_captures, 3);
        assert_eq!(evidence.wire.downs(), 1);
        assert_eq!(evidence.wire.downs_at(wire_point(TABLEAU_CARD_REGIONS[1].click_point)), 1);
        assert_eq!(action_count(&evidence), 1, "only the final card's confirmed win was committed");
        assert_eq!(evidence.completed_boards, 3);
        assert!(!evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::GameCompleted | WorkerEvent::RunCompleted { .. })));
        assert_eq!(evidence.latest.frame.pixels, decode_png(REWARD_PNG).unwrap().pixels);
    }


    /// A gold-only stale OK probe cannot repeat the already acknowledged native OK input.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_bare_gold_stale_ok_after_native_ack_is_not_retried_through_qmp() {
        let evidence = run(Scenario::StaleBareGoldAfterOk);
        assert_eq!(evidence.wire.bare_gold_captures, 3);
        assert_eq!(evidence.wire.downs(), 2);
        assert_eq!(evidence.wire.downs_at(wire_point(TABLEAU_CARD_REGIONS[1].click_point)), 1);
        assert_eq!(evidence.wire.downs_at(wire_point(native_level_up_control().click_point)), 1);
        assert_eq!(evidence.wire.downs_at(wire_point(SCORE_SKIP_CONTROL.click_point)), 0);
        assert_eq!(action_count(&evidence), 1);
        assert_eq!(evidence.completed_boards, 3);
        assert!(!evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::GameCompleted | WorkerEvent::RunCompleted { .. })));
        assert_eq!(evidence.latest.frame.pixels,
            dialog_observation(&[native_level_up_control()]).frame.pixels);
    }


    /// Alternating supported terminal kinds exhausts the fixed confirmation budget without a win.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_alternating_reward_and_level_up_never_commits_mixed_completion_through_qmp() {
        let evidence = run(Scenario::AlternatingTerminalKinds);
        assert_eq!(evidence.wire.reward_captures, 2);
        assert_eq!(evidence.wire.level_up_captures, 2);
        assert_eq!(evidence.wire.downs(), 1);
        assert_eq!(evidence.wire.downs_at(wire_point(TABLEAU_CARD_REGIONS[1].click_point)), 1);
        assert_eq!(action_count(&evidence), 0);
        assert_eq!(evidence.completed_boards, 2);
        assert!(!evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::GameCompleted | WorkerEvent::RunCompleted { .. }
            | WorkerEvent::BoardProgress { completed_boards: 3, .. })));
        assert!(evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::State(WorkerState::Uncertain))));
        assert_eq!(evidence.latest.frame.pixels, decode_png(LEVEL_UP_PNG).unwrap().pixels);
    }


    /// Inspect acquisition position independently of the later confirmation and restart captures.
    fn assert_late_candidate_restart(evidence: &ResultEvidence, unsupported: usize) {
        let first_candidate = evidence.wire.events.iter().position(|event|
            *event == WireEvent::Capture(CaptureKind::LevelUp)).expect("original TP03 terminal candidate");
        assert_eq!(evidence.wire.events[..first_candidate].iter().filter(|event|
            **event == WireEvent::Capture(CaptureKind::Unsupported)).count(), unsupported);
        assert_eq!(evidence.wire.unsupported_captures, unsupported);
        assert_eq!(evidence.wire.events[..first_candidate].iter().filter(|event|
            matches!(event, WireEvent::Down(_))).count(), 1,
            "only the canonical source click preceded fresh terminal evidence");
        assert_level_up_restart(evidence);
    }


    /// Candidate twenty-one is admitted beyond the old twenty-capture limit without extra input.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_tp03_candidate_twenty_one_confirms_and_runs_existing_restart_through_qmp() {
        let evidence = run(Scenario::CandidateTwentyOne);
        assert_late_candidate_restart(&evidence, 20);
    }


    /// Candidate thirty consumes the last acquisition slot, then confirms independently at thirty-one.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_tp03_candidate_thirty_confirms_on_thirty_one_and_restarts_through_qmp() {
        let evidence = run(Scenario::CandidateThirty);
        assert_late_candidate_restart(&evidence, 29);
        let first_candidate = evidence.wire.events.iter().position(|event|
            *event == WireEvent::Capture(CaptureKind::LevelUp)).unwrap();
        let next_capture = evidence.wire.events[first_candidate + 1..].iter().find(|event|
            matches!(event, WireEvent::Capture(_))).expect("independent confirmation at thirty-one");
        assert_eq!(*next_capture, WireEvent::Capture(CaptureKind::LevelUp));
    }


    /// A disappearing candidate at thirty receives four confirmation captures, then stops at thirty-three.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_transient_tp03_candidate_thirty_exhausts_confirmation_on_thirty_three_through_qmp() {
        let evidence = run(Scenario::TransientCandidateThirty);
        assert_eq!(evidence.wire.unsupported_captures, 32);
        assert_eq!(evidence.wire.level_up_captures, 1);
        assert_eq!(evidence.wire.reward_captures, 0);
        assert_eq!(evidence.wire.unsupported_captures + evidence.wire.level_up_captures, 33);
        assert_eq!(evidence.wire.downs(), 1);
        assert_eq!(evidence.wire.downs_at(wire_point(TABLEAU_CARD_REGIONS[1].click_point)), 1);
        assert_eq!(action_count(&evidence), 0);
        assert_eq!(evidence.completed_boards, 2);
        assert_eq!(evidence.latest.frame.pixels, evidence.unsupported.pixels);
        assert!(!evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::GameCompleted | WorkerEvent::RunCompleted { .. }
            | WorkerEvent::BoardProgress { completed_boards: 3, .. })));
    }


    /// Mixed supported kinds beginning at thirty cannot extend the separate four-capture allowance.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_mixed_tp03_candidate_thirty_exhausts_confirmation_on_thirty_three_through_qmp() {
        let evidence = run(Scenario::MixedCandidateThirty);
        assert_eq!(evidence.wire.unsupported_captures, 29);
        assert_eq!(evidence.wire.level_up_captures, 2);
        assert_eq!(evidence.wire.reward_captures, 2);
        assert_eq!(evidence.wire.unsupported_captures + evidence.wire.level_up_captures
            + evidence.wire.reward_captures, 33);
        assert_eq!(evidence.wire.downs(), 1);
        assert_eq!(evidence.wire.downs_at(wire_point(TABLEAU_CARD_REGIONS[1].click_point)), 1);
        assert_eq!(action_count(&evidence), 0);
        assert_eq!(evidence.completed_boards, 2);
        assert_eq!(evidence.latest.frame.pixels, decode_png(REWARD_PNG).unwrap().pixels);
        assert!(!evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::GameCompleted | WorkerEvent::RunCompleted { .. }
            | WorkerEvent::BoardProgress { completed_boards: 3, .. })));
    }


    /// STOP on extended acquisition twenty-one prevents completion and every follow-up input.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_stop_on_unknown_acquisition_twenty_one_retains_pixels_without_input_through_qmp() {
        let evidence = run(Scenario::StopExtendedAcquisition);
        assert_only_original_and_skip(&evidence, 0);
        assert_eq!(evidence.wire.unsupported_captures, 21);
        assert_eq!(evidence.wire.level_up_captures, 0);
        assert_eq!(evidence.wire.reward_captures, 0);
        assert_eq!(action_count(&evidence), 0);
        assert_eq!(evidence.completed_boards, 2);
        assert_eq!(evidence.latest.frame.pixels, evidence.unsupported.pixels);
        assert!(evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::Log(message) if message.contains("STOP"))));
    }


    /// Nonempty gameplay at acquisition twenty-two escapes pending terminal acquisition into old Solver recovery.
    #[test]
    #[ignore = "requires local AF_UNIX sockets"]
    fn tripeaks_nonempty_gameplay_after_twenty_one_unknown_captures_preserves_solver_path_through_qmp() {
        let evidence = run(Scenario::ExtendedRedealNoHalo);
        let source = wire_point(TABLEAU_CARD_REGIONS[2].click_point);
        let solver = wire_point(SHARED_SOLVER_CONTROL.click_point);
        assert_eq!(evidence.wire.unsupported_captures, 21);
        assert_eq!(evidence.wire.level_up_captures, 0);
        assert_eq!(evidence.wire.reward_captures, 0);
        assert_eq!(evidence.wire.downs_at(source), 1);
        assert_eq!(evidence.wire.downs_at(solver), 1);
        assert_eq!(evidence.wire.downs(), 2);
        let solver_down = evidence.wire.events.iter().position(|event|
            *event == WireEvent::Down(solver)).expect("existing Solver recovery");
        assert!(evidence.wire.events[..solver_down].contains(&WireEvent::Capture(CaptureKind::Nonempty)));
        assert_eq!(action_count(&evidence), 1);
        assert!(evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::ActionCompleted { changed_pixels, continued_from_halo: false, .. }
            if *changed_pixels >= crate::parameters::MINIMUM_TABLEAU_CHANGED_PIXELS)));
        assert!(evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::RunCompleted { requested_operations: 1, verified_operations: 1, .. })));
        assert_eq!(evidence.completed_boards, 2);
        assert_eq!(evidence.latest.frame.pixels, evidence.recovered.pixels);
        assert!(!evidence.events.iter().any(|event| matches!(event,
            WorkerEvent::GameCompleted | WorkerEvent::BoardProgress { completed_boards: 3, .. })));
    }
}
