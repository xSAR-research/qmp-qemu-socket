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
use crate::parameters::{
    LEVEL_UP_SHARED_BUTTON_BRIDGE, LEVEL_UP_SHARED_BUTTON_INTERIOR, POST_GAME_MAX_CLICK_ATTEMPTS,
    POST_GAME_TARGETS, PostGameControlVariant, PostGameStage, SCORE_SKIP_MAX_CLICK_ATTEMPTS,
};
use crate::pyramid::PyramidTargetKind;

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
    let mut context = Some((socket.clone(), GameMode::Pyramid));
    let mut state = TableauScanState::for_mode(GameMode::Pyramid);
    let removed = PyramidTargetKind::Card { row: 7, column: 1 };
    state.pyramid.mark_removed(removed);
    let mut completed_boards = 1;
    reset_scan_state_for_context(
        &socket,
        GameMode::Pyramid,
        &mut context,
        &mut state,
        &mut completed_boards,
    );
    assert!(state.pyramid.clicked_target_slots[removed.slot_index().unwrap()]);
    assert_eq!(completed_boards, 1);

    reset_scan_state_for_context(
        Path::new("/test/second.sock"),
        GameMode::Pyramid,
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
        first.prediction,
        false,
        false,
        Some((PathBuf::from("/tmp/old-qmp.sock"), GameMode::TriPeaks)),
    );
    slot.publish(
        second.frame,
        second.prediction,
        true,
        false,
        Some((PathBuf::from("/tmp/new-qmp.sock"), GameMode::Pyramid)),
    );

    let latest = slot.take().expect("latest preview");
    assert_eq!(latest.frame.pixels, vec![2; 4]);
    assert_eq!(latest.prediction, second_prediction);
    assert!(latest.log_prediction);
    assert!(!latest.diagnostic);
    assert_eq!(latest.coalesced_frames, 1);
    assert_eq!(
        latest.context,
        Some((PathBuf::from("/tmp/new-qmp.sock"), GameMode::Pyramid))
    );
    assert!(slot.take().is_none());
}


/// Check a diagnostic failure frame cannot retain an actionable prediction.
#[test]
fn failed_action_publishes_latest_pixels_without_approving_a_target() {
    let (tx, _rx) = mpsc::channel();
    let sink = WorkerEventSink {
        events: tx,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(Some((PathBuf::from("/tmp/qmp.sock"), GameMode::Pyramid))),
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
    assert_eq!(diagnostic.prediction, PredictedAction::NoHighlight);
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


    match rx.try_recv().expect("board-completion event") {
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
    let first_socket = PathBuf::from("/tmp/solitaire-a.sock");
    let second_socket = PathBuf::from("/tmp/solitaire-b.sock");
    let row_three = row_mask(3);
    let mut state = TableauScanState::from_active_rows(row_three).unwrap();
    let mut remembered_context = None;
    let mut completed_boards = 2;

    reset_scan_state_for_context(
        &first_socket,
        GameMode::TriPeaks,
        &mut remembered_context,
        &mut state,
        &mut completed_boards,
    );
    assert_eq!(state, TableauScanState::initial());
    assert_eq!(completed_boards, 0);
    assert_eq!(
        remembered_context
            .as_ref()
            .map(|(path, mode)| (path.as_path(), *mode)),
        Some((first_socket.as_path(), GameMode::TriPeaks))
    );

    state = TableauScanState::from_active_rows(row_three).unwrap();
    completed_boards = 1;
    reset_scan_state_for_context(
        &first_socket,
        GameMode::TriPeaks,
        &mut remembered_context,
        &mut state,
        &mut completed_boards,
    );
    assert_eq!(state.active_rows(), row_three);
    assert_eq!(completed_boards, 1);

    reset_scan_state_for_context(
        &second_socket,
        GameMode::TriPeaks,
        &mut remembered_context,
        &mut state,
        &mut completed_boards,
    );
    assert_eq!(state, TableauScanState::initial());
    assert_eq!(completed_boards, 0);
    assert_eq!(
        remembered_context
            .as_ref()
            .map(|(path, mode)| (path.as_path(), *mode)),
        Some((second_socket.as_path(), GameMode::TriPeaks))
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
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        let felt = crate::parameters::GAMEPLAY_FELT_PROBE_BOUNDS;


        for y in felt.y..felt.bottom() {


            for x in felt.x..felt.right() {
                let offset = y as usize * frame.stride + x as usize * 4;
                frame.pixels[offset..offset + 4].copy_from_slice(&[20, 140, 60, 255]);
            }
        }


        if observation_round == 3 {
            let bounds = crate::parameters::STOCK_HALO_SCAN_BOUNDS;


            for x in bounds.x..bounds.x + crate::parameters::HALO_GOLD_RUN_MIN + 1 {
                let offset = bounds.y as usize * frame.stride + x as usize * 4;
                frame.pixels[offset..offset + 3]
                    .copy_from_slice(&crate::parameters::GOLD_RGB_CANDIDATES[0]);
            }
        }
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
