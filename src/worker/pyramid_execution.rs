//! Pyramid post-action effects, repeated pile halos and board-transition verification.
//!
//! Input delivery stays in the shared worker. This controller observes bounded
//! fresh results, preserving the distinction between effect proof and a newly
//! authorised LEFT/RIGHT pair while keeping completion/redeal guards separate.

use std::{path::Path, sync::atomic::AtomicBool, time::Instant};

use super::{
    ActionFailure, ActionProfile, ActionSuccess, CaptureSeries, FailureFramePhase,
    FrameObservation, PYRAMID_OBSERVATION_LIMIT, WorkerEventSink, WorkerState, action_failure,
    capture_series, click_shared_solver, format_prediction_target, materially_changed_pixels,
    post_game, send_board_progress, send_log, send_state, send_status, validate_probe,
    wait_or_stop,
};
use crate::{
    detector::measure_game_progress_for_profile,
    game::{ActionTarget, GameMode, GameProgress},
    parameters::{
        ACTION_CHANGE_CHANNEL_THRESHOLD, BOARD_TRANSITION_REOBSERVE_DELAY,
        POST_GAME_MAX_OBSERVATION_ROUNDS, SOLVER_MOUSE_HOLD, StepRunSettings,
    },
    pyramid::{self, PyramidTargetKind},
    qmp::QmpClient,
    stepper::{StepPlan, plan_step, verify_post_action},
    tracker::{PredictedAction, TableauScanState, analyse_frame_with_state},
};


/// Observe the result of an already-delivered Pyramid action without replaying it.
///
/// `before_series` supplies the planning frame; `settings` fixes settling delays
/// for the run. Result observations verify effects, bounded redeals and terminal
/// dialogs, or allow a fresh repeated LEFT/RIGHT halo pair during ordinary play.
/// Returns accepted evidence and counters, or a failure with the latest frame.
/// Fresh-halo continuation never proves an effect or increments completion.
pub(super) fn verify_pyramid_action(
    qmp: &mut QmpClient,
    socket_path: &Path,
    mut before_series: CaptureSeries,
    plan: StepPlan,
    kind: PyramidTargetKind,
    settings: StepRunSettings,
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    action_started: Instant,
    mut profile: ActionProfile,
) -> Result<ActionSuccess, ActionFailure> {
    // Input planning and delivery are shared. Pyramid supplies only its
    // semantic effect and board-transition evidence to this bounded verifier.
    let mut last_observation = None;
    let mut observation_rounds = 0usize;
    let mut changed_pixels = 0usize;
    let mut board_completed = false;
    let mut series_complete = false;
    let mut continued_from_halo = false;
    let result = (|| -> Result<PredictedAction, String> {
        let before = before_series
            .observations
            .pop()
            .ok_or_else(|| "Pyramid verification has no planning frame".to_owned())?;
        let expected_final_transition = pyramid::final_tableau_action(&before.frame, kind)
            .map_err(|error| format!("Pyramid final-action analysis failed: {error}"))?;


        let observation_limit = if expected_final_transition {
            POST_GAME_MAX_OBSERVATION_ROUNDS
        } else {
            PYRAMID_OBSERVATION_LIMIT
        };
        let mut effect_verified = false;
        let mut awaiting_redeal = false;
        let mut solver_recovery_sent = false;
        let mut redeal_no_halo_captures = 0usize;
        let mut redeal_confirmation = PyramidRedealConfirmation::default();
        let mut phase_round = 0usize;
        let boards_per_game = scan_state.mode().profile().boards_per_game;


        loop {
            phase_round += 1;


            if phase_round > observation_limit {
                let last_state = last_observation
                    .as_ref()
                    .map(|observation: &FrameObservation| {
                        format!(
                            "last scene={}, target={}, progress={:?}",
                            observation.gameplay_scene,
                            format_prediction_target(observation.prediction),
                            observation.game_progress,
                        )
                    })
                    .unwrap_or_else(|| "no result frame was available".to_owned());
                return Err(format!(
                    "Pyramid verification stopped after {observation_limit} observations in this phase; effect verified={effect_verified}, awaiting redeal={awaiting_redeal}; {last_state}. No card, Move, or Solver input was retried. The latest unverified result frame is displayed for inspection; use Capture Frame before another run."
                ));
            }
            observation_rounds += 1;
            send_log(
                event_tx,
                format!(
                    "Pyramid result observation {observation_rounds} (phase {phase_round}/{observation_limit}); one fresh capture, no replay of the previous card or Move input."
                ),
            );
            let mut series = capture_series(qmp, socket_path, scan_state, cancel_requested)?;
            profile.capture.add_assign(series.timing);
            last_observation = series.observations.pop();
            let observation = last_observation
                .as_mut()
                .ok_or_else(|| "Pyramid result capture contained no frame".to_owned())?;
            let analysis_started = Instant::now();


            if !observation.gameplay_scene {
                redeal_no_halo_captures = 0;
                redeal_confirmation.observe(false, observation.game_progress);


                let terminal_visible = if board_completed || expected_final_transition {


                    match post_game::level_up_overrides_board_progress(true, observation) {
                        Ok(true) => true,
                        Ok(false) => {
                            post_game::new_game_visible_while_awaiting_level_up(observation)?
                        }
                        Err(error) if post_game::is_level_up_ambiguity(&error) => {
                            profile.validation_effect += analysis_started.elapsed();
                            send_log(
                                event_tx,
                                format!(
                                    "WAITING: {error}; the final Pyramid card was clicked once, so observing again in {} ms without input.",
                                    BOARD_TRANSITION_REOBSERVE_DELAY.as_millis(),
                                ),
                            );
                            profile.intentional_wait += wait_or_stop(
                                BOARD_TRANSITION_REOBSERVE_DELAY,
                                cancel_requested,
                                "STOP was requested during ambiguous Pyramid Level Up re-observation.",
                            )?;
                            continue;
                        }
                        Err(error) => return Err(error),
                    }
                } else {
                    false
                };


                if terminal_visible {
                    *completed_boards = boards_per_game;
                    board_completed = true;
                    series_complete = true;
                    profile.validation_effect += analysis_started.elapsed();
                    send_log(event_tx,
                        "Pyramid final-tableau action followed by a recognised terminal dialog confirms game completion; the shared post-game controller will resume at the visible stage.".to_owned());
                    return Ok(PredictedAction::NoHighlight);
                }
                profile.validation_effect += analysis_started.elapsed();
                send_log(event_tx,
                    "Pyramid result is not a recognised gameplay scene; observing the transition without guest input.".to_owned());
            } else {


                if !board_completed
                    && expected_final_transition
                    && pyramid::redeal_after_final_tableau_action(
                        &before.frame,
                        &observation.frame,
                        kind,
                    )
                    .map_err(|error| format!("Pyramid redeal analysis failed: {error}"))?
                {


                    // A progress reading during the deal animation is not final.
                    // Require two consecutive fresh frames with restored cards AND
                    // AnotherBoard, without replaying input while they disagree.
                    if !redeal_confirmation.observe(true, observation.game_progress) {
                        let evidence = measure_game_progress_for_profile(
                            &observation.frame,
                            scan_state.mode().profile(),
                        )
                        .map_err(|error| {
                            format!("Pyramid redeal progress analysis failed: {error}")
                        })?;
                        send_log(
                            event_tx,
                            format!(
                                "WAITING: Pyramid final-apex result has restored bottom cards; progress={:?}, right-probe black={}/{}, matching redeal captures={}/2. Waiting {} ms for a fresh capture without input.",
                                evidence.classification,
                                evidence.black_pixels,
                                evidence.total_pixels,
                                redeal_confirmation.matching_captures,
                                settings.animation_delays().pyramid_reobserve.as_millis(),
                            ),
                        );
                        profile.validation_effect += analysis_started.elapsed();
                        profile.intentional_wait += wait_or_stop(
                            settings.animation_delays().pyramid_reobserve,
                            cancel_requested,
                            "STOP was requested while Pyramid redeal progress settled; no input was retried.",
                        )?;
                        continue;
                    }
                    *completed_boards = completed_boards
                        .saturating_add(1)
                        .min(boards_per_game.saturating_sub(1));
                    board_completed = true;
                    effect_verified = true;
                    *scan_state = TableauScanState::for_mode(GameMode::Pyramid);
                    refresh_observation_prediction(observation, scan_state)?;
                    send_board_progress(event_tx, scan_state, *completed_boards);
                    send_log(event_tx,
                        "Pyramid automatic redeal verified: the planning frame had only the highlighted apex remaining, and the fresh result has new positive bottom-row cards with AnotherBoard progress in two consecutive captures. Per-board state reset; no new-board slot was marked consumed.".to_owned());
                } else if !awaiting_redeal {
                    redeal_confirmation.observe(false, observation.game_progress);
                }


                if !effect_verified {
                    let evidence = pyramid::measure_effect(&before.frame, &observation.frame, kind)
                        .map_err(|error| format!("Pyramid effect analysis failed: {error}"))?;
                    effect_verified = evidence.verified;
                    send_log(
                        event_tx,
                        format!(
                            "Pyramid effect evidence for {kind:?}: verified={effect_verified}, before face={:?}, after face={:?}, pile interior changed pixels={:?}, opposite pile={:?}, highlighted partner={:?}, pair interior changed pixels={:?}, removed highlighted partner={:?}; next target={}.",
                            evidence.before_face,
                            evidence.after_face,
                            evidence.pile_changed_pixels,
                            evidence.opposite_pile,
                            evidence.highlighted_partner,
                            evidence.pair_changed_pixels,
                            evidence.removed_highlighted_partner,
                            format_prediction_target(observation.prediction),
                        ),
                    );


                    if effect_verified {
                        changed_pixels = materially_changed_pixels(
                            &before.frame,
                            &observation.frame,
                            plan.input().effect_bounds(),
                            plan.input().effect_exclusion_bounds(),
                            ACTION_CHANGE_CHANNEL_THRESHOLD,
                        )?;
                        // Only the clicked card is retired. An unclicked
                        // partner and all three lower controls remain eligible.
                        scan_state.pyramid.mark_removed(kind);
                        refresh_observation_prediction(observation, scan_state)?;
                        send_log(
                            event_tx,
                            format!(
                                "Pyramid effect verified for {kind:?}; clicked-card state updated and the same result frame rescanned. Changed effect pixels={changed_pixels}."
                            ),
                        );
                    }
                }


                if effect_verified {
                    let board_empty =
                        pyramid::board_is_empty(&observation.frame).map_err(|error| {
                            format!("Pyramid board-presence analysis failed: {error}")
                        })?;
                    let cards_visible = pyramid::has_visible_tableau_card(&observation.frame)
                        .map_err(|error| {
                            format!("Pyramid card-presence analysis failed: {error}")
                        })?;


                    if !board_completed && board_empty {
                        let progress = observation.game_progress.ok_or_else(|| {
                            "Verified empty Pyramid board has no progress-bar classification"
                                .to_owned()
                        })?;
                        series_complete = progress == GameProgress::GameComplete;


                        *completed_boards = if series_complete {
                            boards_per_game
                        } else {
                            completed_boards
                                .saturating_add(1)
                                .min(boards_per_game.saturating_sub(1))
                        };
                        board_completed = true;
                        send_board_progress(event_tx, scan_state, *completed_boards);
                        send_log(
                            event_tx,
                            format!(
                                "Pyramid board completion verified by the action effect and positive empty-tableau evidence; shared progress bar={progress:?}."
                            ),
                        );
                        profile.validation_effect += analysis_started.elapsed();


                        if series_complete {
                            return Ok(PredictedAction::NoHighlight);
                        }
                        awaiting_redeal = true;
                        phase_round = 0;
                        solver_recovery_sent = false;
                        send_status(event_tx, "Waiting for Pyramid redeal".to_owned());
                        profile.intentional_wait += wait_or_stop(
                            settings.animation_delays().board_redeal,
                            cancel_requested,
                            "STOP was requested during the Pyramid redeal wait; the completed-board input was not retried.",
                        )?;
                        continue;
                    }


                    if awaiting_redeal
                        && redeal_confirmation.observe(cards_visible, observation.game_progress)
                    {
                        // The preceding board was positively empty. Fresh
                        // gameplay with cards now proves a new board exists.
                        *scan_state = TableauScanState::for_mode(GameMode::Pyramid);
                        refresh_observation_prediction(observation, scan_state)?;
                        awaiting_redeal = false;
                        send_log(event_tx,
                            "Pyramid redeal confirmed by two consecutive non-empty gameplay frames with AnotherBoard progress; per-board clicked-card state reset.".to_owned());
                    }


                    if expected_final_transition && !board_completed {
                        send_log(event_tx,
                            "WAITING: the final-apex effect is verified, but board/game completion is not yet verified. Ignoring transitional halos and recapturing without input.".to_owned());
                    } else if !awaiting_redeal {


                        if matches!(observation.prediction, PredictedAction::Action(_)) {


                            let after = if board_completed {
                                plan_step(observation.prediction).map(|next| next.before())
                            } else {
                                verify_post_action(&plan, observation.prediction, true)
                            }
                            .map_err(|error| {
                                format!("Pyramid next-target validation failed: {error}")
                            })?;
                            profile.validation_effect += analysis_started.elapsed();
                            return Ok(after);
                        }


                        if !matches!(observation.prediction, PredictedAction::NoHighlight) {
                            return Err("Pyramid produced an invalid next-target classification; no further input was sent".to_owned());
                        }


                        redeal_no_halo_captures = if board_completed
                            && cards_visible
                            && observation.game_progress == Some(GameProgress::AnotherBoard)
                        {
                            redeal_no_halo_captures.saturating_add(1)
                        } else {
                            0
                        };


                        if pyramid_solver_recovery_authorised(
                            board_completed,
                            observation.game_progress,
                            observation.gameplay_scene,
                            effect_verified,
                            board_empty,
                            cards_visible,
                            redeal_no_halo_captures,
                            solver_recovery_sent,
                        ) {
                            profile.validation_effect += analysis_started.elapsed();
                            let probe_started = Instant::now();
                            let probe = qmp.probe().map_err(|error| {
                                format!("Pyramid Solver recovery QMP probe failed: {error}")
                            })?;
                            validate_probe(&probe)?;
                            profile.validation_effect += probe_started.elapsed();
                            send_state(event_tx, WorkerState::Acting);
                            send_status(event_tx, "Click Solver".to_owned());
                            profile.input_wall += click_shared_solver(qmp, cancel_requested)?;
                            profile.intentional_wait += SOLVER_MOUSE_HOLD;
                            solver_recovery_sent = true;
                            send_log(event_tx,
                                "Pyramid Solver re-activated only after positive board completion and verified redeal; the new board has cards but no halo after three settled captures. No ordinary no-halo action triggered Solver.".to_owned());
                            send_state(event_tx, WorkerState::Verifying);
                            profile.intentional_wait += wait_or_stop(
                                settings.animation_delays().pyramid_reobserve,
                                cancel_requested,
                                "STOP was requested after Pyramid Solver activation; input was not retried.",
                            )?;
                            continue;
                        }
                        send_log(
                            event_tx,
                            format!(
                                "WAITING: Pyramid result has no eligible halo after observation {phase_round}; effect verified, board empty={board_empty}, board completed={board_completed}, progress={:?}. Waiting {} ms for another fresh QMP capture; no Solver or gameplay input sent.",
                                observation.game_progress,
                                settings.animation_delays().pyramid_reobserve.as_millis(),
                            ),
                        );
                    }
                } else if pyramid_halo_continuation_phase(
                    expected_final_transition,
                    board_completed,
                    awaiting_redeal,
                ) && pyramid::has_repeated_pile_pair(
                    &before.frame,
                    &observation.frame,
                    kind,
                )
                .map_err(|error| format!("Pyramid fresh-pair analysis failed: {error}"))?
                {
                    let next = plan_step(observation.prediction)
                        .map_err(|error| format!("Pyramid fresh-pair plan failed: {error}"))?;


                    if !matches!(
                        next.input().action().target,
                        ActionTarget::Pyramid(PyramidTargetKind::Left | PyramidTargetKind::Right)
                    ) {
                        return Err("Fresh Pyramid pair did not produce a pile target; no further input sent".to_owned());
                    }
                    continued_from_halo = true;
                    changed_pixels = materially_changed_pixels(
                        &before.frame,
                        &observation.frame,
                        plan.input().effect_bounds(),
                        plan.input().effect_exclusion_bounds(),
                        ACTION_CHANGE_CHANNEL_THRESHOLD,
                    )?;
                    send_log(event_tx,
                        "Pyramid fresh LEFT/RIGHT HALO pair accepted after the configured settle and new capture. Previous effect remains unproven; identical replacement cards are allowed. The fresh pair authorises the next operation; no card history or completion counter changed.".to_owned());
                    profile.validation_effect += analysis_started.elapsed();
                    return Ok(next.before());
                } else {
                    send_log(
                        event_tx,
                        format!(
                            "WAITING: Pyramid {kind:?} effect is not verified after observation {phase_round}; scene={}, progress={:?}. Waiting {} ms and recapturing without input.",
                            observation.gameplay_scene,
                            observation.game_progress,
                            settings.animation_delays().pyramid_reobserve.as_millis(),
                        ),
                    );
                }
                profile.validation_effect += analysis_started.elapsed();
            }

            profile.intentional_wait += wait_or_stop(
                settings.animation_delays().pyramid_reobserve,
                cancel_requested,
                "STOP was requested during Pyramid result verification; the action was not retried.",
            )?;
        }
    })();


    let after = match result {
        Ok(after) => after,
        Err(error) => {
            return Err(action_failure(
                action_started,
                profile,
                WorkerState::Uncertain,
                error,
                last_observation,
                FailureFramePhase::PostAction,
            ));
        }
    };


    let Some(observation) = last_observation else {
        return Err(action_failure(
            action_started,
            profile,
            WorkerState::Uncertain,
            "Pyramid verification completed without a result frame".to_owned(),
            None,
            FailureFramePhase::PostAction,
        ));
    };
    profile.total = action_started.elapsed();
    Ok(ActionSuccess {
        plan,
        after,
        observation,
        changed_pixels,
        observation_rounds,
        board_completed,
        series_complete,
        completed_boards: *completed_boards,
        continued_from_halo,
        profile,
    })
}


/// Permit repeated pile halos only outside final-card, completed-board and redeal phases.
pub(super) fn pyramid_halo_continuation_phase(
    expected_final_transition: bool,
    board_completed: bool,
    awaiting_redeal: bool,
) -> bool {
    !expected_final_transition && !board_completed && !awaiting_redeal
}


/// Rescan the same captured pixels after scan-history changes; return detector errors without capturing again.
fn refresh_observation_prediction(
    observation: &mut FrameObservation,
    scan_state: &TableauScanState,
) -> Result<(), String> {
    let analysis = analyse_frame_with_state(&observation.frame, scan_state)
        .map_err(|error| format!("Result-frame rescan failed: {error}"))?;
    observation.prediction = analysis.prediction;
    observation.observed_rows = analysis.observed_rows;
    Ok(())
}


/// Consecutive independent observations guard redeal against an animated bar.
/// Effect verification intentionally is not an input: an early verified pile
/// change must not prevent a later redeal from clearing the old board state.
#[derive(Default)]
pub(super) struct PyramidRedealConfirmation {
    /// Consecutive matching fresh observations, saturated at two.
    matching_captures: u8,
}


impl PyramidRedealConfirmation {


    /// Count consecutive fresh frames showing restored cards and AnotherBoard progress.
    ///
    /// Reset on contradictory or missing evidence; report confirmation after two matches.
    pub(super) fn observe(&mut self, restored_cards: bool, progress: Option<GameProgress>) -> bool {


        self.matching_captures = if restored_cards && progress == Some(GameProgress::AnotherBoard) {
            self.matching_captures.saturating_add(1).min(2)
        } else {
            0
        };
        self.matching_captures == 2
    }
}


/// Authorise one Solver recovery only after verified redeal and three settled no-halo observations.
///
/// The caller supplies scene, card, effect and progress evidence; ordinary
/// missing halos and previously attempted Solver recovery cannot qualify.
pub(super) fn pyramid_solver_recovery_authorised(
    verified_redeal: bool,
    progress: Option<GameProgress>,
    gameplay_scene: bool,
    effect_verified: bool,
    board_empty: bool,
    cards_visible: bool,
    observation_round: usize,
    solver_already_sent: bool,
) -> bool {
    verified_redeal
        && progress == Some(GameProgress::AnotherBoard)
        && gameplay_scene
        && effect_verified
        && !board_empty
        && cards_visible
        && observation_round >= 3
        && !solver_already_sent
}
