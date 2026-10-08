//! Shared score, Level Up and new-game transition controller for both game profiles.
//!
//! Fresh visual evidence, bounded retries and cancellation checks authorise each
//! click. A recovered gameplay frame can cancel stale completion state.

use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use super::{
    CaptureTiming, FrameObservation, WorkerEventSink, WorkerState, capture_with_client,
    click_shared_solver, milliseconds, publish_post_action_observation, send_board_progress,
    send_log, send_state, send_status, validate_probe, wait_or_stop,
};
use crate::{
    capture::CapturedFrame,
    detector::{dialog_gold_fraction_per_mille, has_dialog_gold_button},
    parameters::{
        BOARD_TRANSITION_REOBSERVE_DELAY, LEVEL_UP_APPEAR_DELAY, LEVEL_UP_SHARED_BUTTON_BRIDGE,
        LEVEL_UP_SHARED_BUTTON_INTERIOR, NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH,
        POST_GAME_MAX_CLICK_ATTEMPTS, POST_GAME_MAX_OBSERVATION_ROUNDS, POST_GAME_MOUSE_HOLD,
        POST_GAME_STAGE_DELAY, POST_GAME_TARGETS, PostGameControlVariant, PostGameStage,
        PostGameTarget, SCORE_SKIP_CONTROL, SCORE_SKIP_MAX_CLICK_ATTEMPTS, SOLVER_MOUSE_HOLD,
    },
    qmp::QmpClient,
    tracker::{PredictedAction, TableauScanState},
};


/// Minimum gold coverage required in both probes joining one tall Level Up button.
const LEVEL_UP_SHARED_BUTTON_GOLD_PER_MILLE: u32 = 650;


/// Recognised error prefix that permits another input-free Level Up observation.
const LEVEL_UP_AMBIGUITY_PREFIX: &str = "Level Up OK target is ambiguous:";


/// Result of the shared terminal-screen controller.
pub(super) enum PostGameOutcome {
    /// A new game was started and its first actionable halo captured.
    Restarted(FrameObservation),
    /// Fresh gameplay disproved stale completion state, so existing play can resume.
    RecoveredGameplay(FrameObservation),
}


/// Advance verified end-of-game screens through score skip, Level Up, New Game, Play and Solver.
///
/// `initial_observation` may establish an already-visible terminal stage. Fresh
/// observations authorise each control, with bounded observation/click budgets.
/// Returns the first actionable restarted or recovered gameplay frame; returns
/// an error on cancellation, capture failure, unsafe recognition or uncertain input.
pub(super) fn run_post_game_restart(
    qmp: &mut QmpClient,
    socket_path: &Path,
    initial_observation: Option<&FrameObservation>,
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) -> Result<PostGameOutcome, String> {
    // Advances the verified end-of-game UI through OK, New Game, Play, and Solver.
    // Every control uses a deliberate hold; an earlier dialog is retried if a
    // fresh capture proves that its prior click was not accepted. Observation
    // and confirmed-delivery click retries are both explicitly bounded.
    let transition_started = Instant::now();
    let mut capture_timing = CaptureTiming::default();
    let mut intentional_wait = Duration::ZERO;
    let mut input_wall = Duration::ZERO;
    let boards_per_game = scan_state.mode().profile().boards_per_game;
    // Only this positively confirmed TriPeaks reward entry gets the new skip
    // evidence policy. Pyramid and existing score-counting transitions retain theirs.
    let reward_entry = scan_state.mode() == crate::game::GameMode::TriPeaks
        && initial_observation.is_some_and(|observation|
            super::tripeaks_terminal::is_reward_overlay(&observation.frame) == Ok(true)
        );


    let mut known_terminal_stage = match known_post_game_stage(initial_observation) {
        Ok(stage) => stage,
        Err(error) if is_level_up_ambiguity(&error) => {
            send_log(event_tx,
                "The initial frame contains an ambiguous Level Up control; observing the dialog again before sending any post-game input.".to_owned());
            Some(0)
        }
        Err(error) => return Err(error),
    };

    *scan_state = TableauScanState::for_mode(scan_state.mode());
    send_state(event_tx, WorkerState::Verifying);
    let mut score_skip_click_attempts = 0usize;


    if known_terminal_stage.is_none() {
        send_status(
            event_tx,
            format!(
                "Detected end of game — waiting {} ms",
                POST_GAME_STAGE_DELAY.as_millis()
            ),
        );
        send_log(
            event_tx,
            format!(
                "The {boards_per_game}-board game appears complete; waiting {} ms, then capturing again before any score-skip input.",
                POST_GAME_STAGE_DELAY.as_millis(),
            ),
        );
        intentional_wait += wait_or_stop(
            POST_GAME_STAGE_DELAY,
            cancel_requested,
            "STOP was requested before the pre-score transition capture.",
        )?;

        let mut pre_score_round = 1usize;
        let mut consecutive_non_gameplay = 0usize;
        let mut consecutive_reward = 0usize;


        loop {
            send_status(event_tx, "Verifying game-end transition".to_owned());
            let (observation, timing) =
                capture_with_client(qmp, socket_path, scan_state, event_tx).map_err(|error| {
                    format!("Pre-score transition capture failed before input: {error}")
                })?;
            capture_timing.add_assign(timing);
            publish_post_action_observation(event_tx, &observation);


            if observation.gameplay_scene
                && matches!(observation.prediction, PredictedAction::Action(_))
            {
                *completed_boards = 0;
                *scan_state = TableauScanState::for_mode(scan_state.mode());
                let _ = scan_state.reconcile_observation(observation.observed_rows);
                send_log(event_tx,
                    "RECOVERED: pre-score capture found an actionable new board; discarded stale game-completion state without clicking the board or score panel.".to_owned());
                return Ok(PostGameOutcome::RecoveredGameplay(observation));
            }

            let mut level_up_ambiguous = false;


            match known_post_game_stage(Some(&observation)) {
                Ok(Some(stage)) => {
                    known_terminal_stage = Some(stage);
                    send_log(event_tx,
                        "A terminal control appeared before score skip; entering its verified stage without a centre click.".to_owned());
                    break;
                }
                Ok(None) => {}
                Err(error) if is_level_up_ambiguity(&error) => {
                    level_up_ambiguous = true;
                    send_log(
                        event_tx,
                        format!(
                            "WAITING: {error}; pre-score round {pre_score_round}/{POST_GAME_MAX_OBSERVATION_ROUNDS} sends no guest input."
                        ),
                    );
                }
                Err(error) => return Err(error),
            }


            let reward_visible = reward_entry
                && super::tripeaks_terminal::is_reward_overlay(&observation.frame)?;
            consecutive_reward = if reward_visible { consecutive_reward.saturating_add(1) } else { 0 };
            consecutive_non_gameplay = if observation.gameplay_scene || level_up_ambiguous {
                0
            } else {
                consecutive_non_gameplay.saturating_add(1)
            };


            if (reward_entry && consecutive_reward < 2)
                || (!reward_entry && consecutive_non_gameplay < 2)
            {
                require_post_game_observation_retry_budget(
                    pre_score_round,
                    "pre-score transition",
                )?;
                send_log(
                    event_tx,
                    format!(
                        "WAITING: pre-score round {pre_score_round}/{POST_GAME_MAX_OBSERVATION_ROUNDS} has non-gameplay={consecutive_non_gameplay}/2, positive TriPeaks reward={consecutive_reward}/2, reward entry={reward_entry}; rechecking in {} ms without input.",
                        BOARD_TRANSITION_REOBSERVE_DELAY.as_millis(),
                    ),
                );
                intentional_wait += wait_or_stop(
                    BOARD_TRANSITION_REOBSERVE_DELAY,
                    cancel_requested,
                    "STOP was requested during the pre-score transition wait.",
                )?;
                pre_score_round = pre_score_round.saturating_add(1);
                continue;
            }

            send_state(event_tx, WorkerState::Acting);
            send_status(event_tx, "Skip score counting".to_owned());
            input_wall += send_score_skip_click(qmp, cancel_requested, score_skip_click_attempts)?;
            score_skip_click_attempts = score_skip_click_attempts.saturating_add(1);
            send_log(
                event_tx,
                format!(
                    "Two consecutive {} captures preceded score-counting/reward panel click attempt {score_skip_click_attempts}/{SCORE_SKIP_MAX_CLICK_ATTEMPTS} at guest pixel ({}, {}) with a {} ms hold; waiting {} ms before classifying Level Up.",
                    if reward_entry { "positively recognised TriPeaks reward" } else { "non-gameplay" },
                    SCORE_SKIP_CONTROL.click_point.x,
                    SCORE_SKIP_CONTROL.click_point.y,
                    POST_GAME_MOUSE_HOLD.as_millis(),
                    LEVEL_UP_APPEAR_DELAY.as_millis(),
                ),
            );
            send_state(event_tx, WorkerState::Verifying);
            send_status(
                event_tx,
                format!(
                    "Waiting {} ms for Level Up",
                    LEVEL_UP_APPEAR_DELAY.as_millis()
                ),
            );
            intentional_wait += wait_or_stop(
                LEVEL_UP_APPEAR_DELAY,
                cancel_requested,
                "STOP was requested during the post-score-skip wait.",
            )?;
            break;
        }
    } else {
        send_log(event_tx,
            "A verified terminal dialog is already visible; entering the shared post-game controller at that stage without a score-skip click.".to_owned());
    }

    let mut click_attempts = [0usize; POST_GAME_TARGETS.len()];
    let mut recovery_solver_click_attempts = 0usize;
    let mut target_index = known_terminal_stage.unwrap_or(0);


    while let Some(target) = POST_GAME_TARGETS.get(target_index).copied() {
        let mut observation_round = 1usize;


        let matched_variant = loop {
            send_status(event_tx, format!("Waiting for {}", target.label));
            let (observation, timing) =
                capture_with_client(qmp, socket_path, scan_state, event_tx).map_err(|error| {
                    format!(
                        "Post-game {} capture failed before input: {error}.",
                        target.label
                    )
                })?;
            capture_timing.add_assign(timing);
            publish_post_action_observation(event_tx, &observation);


            if observation.gameplay_scene
                && matches!(observation.prediction, PredictedAction::Action(_))
            {
                *completed_boards = 0;
                *scan_state = TableauScanState::for_mode(scan_state.mode());
                let _ = scan_state.reconcile_observation(observation.observed_rows);
                send_log(
                    event_tx,
                    format!(
                        "RECOVERED: expected post-game stage {} but an actionable all-row HALO was found; abandoning the stale board counter and resuming normal play.",
                        target.label,
                    ),
                );
                return Ok(PostGameOutcome::RecoveredGameplay(observation));
            }

            let earlier = earlier_visible_post_game_target(&observation, target_index);
            let current = resolve_post_game_target(&observation, target);


            if let Some(error) = earlier.as_ref().err().or(current.as_ref().err()) {


                if is_level_up_ambiguity(error) {
                    require_post_game_observation_retry_budget(observation_round, target.label)?;
                    send_log(
                        event_tx,
                        format!(
                            "WAITING: {error}; waiting {} ms and capturing again (round {observation_round}/{POST_GAME_MAX_OBSERVATION_ROUNDS}).",
                            BOARD_TRANSITION_REOBSERVE_DELAY.as_millis(),
                        ),
                    );
                    intentional_wait += wait_or_stop(
                        BOARD_TRANSITION_REOBSERVE_DELAY,
                        cancel_requested,
                        "STOP was requested during ambiguous Level Up re-observation.",
                    )?;
                    observation_round = observation_round.saturating_add(1);
                    continue;
                }
                return Err(error.clone());
            }


            if let Some((stale_index, stale_target, stale_variant)) = earlier? {
                require_post_game_observation_retry_budget(observation_round, target.label)?;
                require_post_game_click_retry_budget(
                    click_attempts[stale_index],
                    stale_target.label,
                )?;
                let reprobe = qmp.probe().map_err(|error| {
                    format!(
                        "Post-game stale {} variant {} QMP probe failed while waiting for {}: {error}",
                        stale_target.label, stale_variant.label, target.label,
                    )
                })?;
                validate_probe(&reprobe).map_err(|error| {
                    format!(
                        "Post-game stale {} variant {} retry refused by the fresh QMP probe: {error}",
                        stale_target.label, stale_variant.label,
                    )
                })?;

                send_state(event_tx, WorkerState::Acting);
                send_status(
                    event_tx,
                    post_game_click_status(stale_target.stage).to_owned(),
                );
                let input_started = Instant::now();
                qmp.click_with_hold(
                    stale_variant.click_point,
                    NOMINAL_FRAME_WIDTH,
                    NOMINAL_FRAME_HEIGHT,
                    POST_GAME_MOUSE_HOLD,
                )
                .map_err(|error| {
                    format!(
                        "Post-game stale {} variant {} retry result is uncertain: {error}. No further input followed the uncertain QMP result.",
                        stale_target.label, stale_variant.label,
                    )
                })?;
                input_wall += input_started.elapsed();
                click_attempts[stale_index] = click_attempts[stale_index].saturating_add(1);
                send_log(
                    event_tx,
                    format!(
                        "RETRY: post-game stage {} variant {} remains visible while waiting for {}; click attempt {} sent at guest pixel ({}, {}) with a {} ms hold. Awaiting a verified screen transition.",
                        stale_target.label,
                        stale_variant.label,
                        target.label,
                        click_attempts[stale_index],
                        stale_variant.click_point.x,
                        stale_variant.click_point.y,
                        POST_GAME_MOUSE_HOLD.as_millis(),
                    ),
                );
                send_state(event_tx, WorkerState::Verifying);
                intentional_wait += wait_or_stop(
                    POST_GAME_STAGE_DELAY,
                    cancel_requested,
                    "STOP was requested after retrying a stale post-game stage.",
                )?;
                observation_round = observation_round.saturating_add(1);
                continue;
            }


            if let Some(variant) = current? {
                break Some(variant);
            }


            if target.stage == PostGameStage::LevelUpOk
                && new_game_visible_while_awaiting_level_up(&observation)?
            {
                send_log(
                    event_tx,
                    "RECOVERED: Level Up OK is absent but the New Game control is visible in a fresh non-gameplay frame. Waiting for a second capture to verify New Game before sending input."
                        .to_owned(),
                );
                intentional_wait += wait_or_stop(
                    POST_GAME_STAGE_DELAY,
                    cancel_requested,
                    "STOP was requested while verifying the New Game transition.",
                )?;
                break None;
            }


            if observation.gameplay_scene
                && matches!(observation.prediction, PredictedAction::NoHighlight)
                && target.stage != PostGameStage::Solver
            {
                require_post_game_observation_retry_budget(observation_round, target.label)?;
                require_post_game_click_retry_budget(
                    recovery_solver_click_attempts,
                    "recovery Solver",
                )?;
                let reprobe = qmp.probe().map_err(|error| {
                    format!("Post-game recovery Solver QMP probe failed: {error}")
                })?;
                validate_probe(&reprobe)
                    .map_err(|error| format!("Post-game recovery Solver click refused: {error}"))?;
                send_state(event_tx, WorkerState::Acting);
                send_status(event_tx, "Click Solver".to_owned());
                let input_started = Instant::now();
                click_shared_solver(qmp, cancel_requested)
                .map_err(|error| {
                    format!(
                        "Post-game recovery Solver click is uncertain: {error}. No automatic input retry followed the uncertain QMP result."
                    )
                })?;
                input_wall += input_started.elapsed();
                recovery_solver_click_attempts = recovery_solver_click_attempts.saturating_add(1);
                send_log(
                    event_tx,
                    format!(
                        "RECOVERY: expected post-game stage {} but found a gameplay board without a HALO; Solver click attempt {} sent with a {} ms hold before re-orientation.",
                        target.label,
                        recovery_solver_click_attempts,
                        SOLVER_MOUSE_HOLD.as_millis(),
                    ),
                );
                send_state(event_tx, WorkerState::Verifying);
                intentional_wait += wait_or_stop(
                    POST_GAME_STAGE_DELAY,
                    cancel_requested,
                    "STOP was requested after the post-game recovery Solver click.",
                )?;
                observation_round = observation_round.saturating_add(1);
                continue;
            }

            require_post_game_observation_retry_budget(observation_round, target.label)?;


            if score_skip_retry_is_authorised(
                target.stage,
                observation.gameplay_scene,
                observation_round,
            ) && (!reward_entry || super::tripeaks_terminal::is_reward_overlay(&observation.frame)?) {
                send_state(event_tx, WorkerState::Acting);
                send_status(event_tx, "Skip score counting".to_owned());
                input_wall +=
                    send_score_skip_click(qmp, cancel_requested, score_skip_click_attempts)?;
                score_skip_click_attempts = score_skip_click_attempts.saturating_add(1);
                send_log(
                    event_tx,
                    format!(
                        "RETRY: post-game stage {} was not recognised on observation round {observation_round}; score-skip centre click attempt {score_skip_click_attempts}/{SCORE_SKIP_MAX_CLICK_ATTEMPTS} was sent at guest pixel ({}, {}) with a {} ms hold. Waiting {} ms before the next capture; press STOP to end the loop.",
                        target.label,
                        SCORE_SKIP_CONTROL.click_point.x,
                        SCORE_SKIP_CONTROL.click_point.y,
                        POST_GAME_MOUSE_HOLD.as_millis(),
                        LEVEL_UP_APPEAR_DELAY.as_millis(),
                    ),
                );
                send_state(event_tx, WorkerState::Verifying);
                intentional_wait += wait_or_stop(
                    LEVEL_UP_APPEAR_DELAY,
                    cancel_requested,
                    "STOP was requested while waiting after a score-skip retry.",
                )?;
                observation_round = observation_round.saturating_add(1);
                continue;
            }

            send_log(
                event_tx,
                format!(
                    "WAITING: post-game stage {} was not recognised on observation round {observation_round}; allowing the guest to animate for {} ms before the next capture. Press STOP to end the loop.",
                    target.label,
                    BOARD_TRANSITION_REOBSERVE_DELAY.as_millis(),
                ),
            );
            intentional_wait += wait_or_stop(
                BOARD_TRANSITION_REOBSERVE_DELAY,
                cancel_requested,
                "STOP was requested while waiting for the next post-game screen.",
            )?;
            observation_round = observation_round.saturating_add(1);
        };


        let Some(matched_variant) = matched_variant else {
            target_index += 1;
            continue;
        };

        let reprobe = qmp
            .probe()
            .map_err(|error| format!("Post-game {} QMP probe failed: {error}", target.label))?;
        validate_probe(&reprobe).map_err(|error| {
            format!(
                "Post-game {} click refused by the fresh QMP probe: {error}",
                target.label
            )
        })?;

        send_state(event_tx, WorkerState::Acting);
        send_status(event_tx, post_game_click_status(target.stage).to_owned());
        let input_started = Instant::now();


        let mouse_hold = if target.stage == PostGameStage::Solver {
            SOLVER_MOUSE_HOLD
        } else {
            POST_GAME_MOUSE_HOLD
        };
        qmp.click_with_hold(
            matched_variant.click_point,
            NOMINAL_FRAME_WIDTH,
            NOMINAL_FRAME_HEIGHT,
            mouse_hold,
        )
        .map_err(|error| {
            format!(
                "Post-game {} click result is uncertain: {error}. The click was not retried.",
                target.label
            )
        })?;
        input_wall += input_started.elapsed();
        click_attempts[target_index] = click_attempts[target_index].saturating_add(1);


        if target.stage == PostGameStage::Solver {
            send_log(
                event_tx,
                format!(
                    "Post-game stage Solver variant {} recognised; click attempt {} sent at guest pixel ({}, {}) with a {} ms hold; QMP commands=3, input events=4. Awaiting HALO confirmation.",
                    matched_variant.label,
                    click_attempts[target_index],
                    matched_variant.click_point.x,
                    matched_variant.click_point.y,
                    SOLVER_MOUSE_HOLD.as_millis(),
                ),
            );
        } else {
            send_log(
                event_tx,
                format!(
                    "Post-game stage {} variant {} recognised; click attempt {} sent at guest pixel ({}, {}) with a {} ms hold; QMP commands=3, input events=4. Awaiting a fresh capture proving the next stage.",
                    target.label,
                    matched_variant.label,
                    click_attempts[target_index],
                    matched_variant.click_point.x,
                    matched_variant.click_point.y,
                    POST_GAME_MOUSE_HOLD.as_millis(),
                ),
            );
        }

        send_state(event_tx, WorkerState::Verifying);
        send_status(
            event_tx,
            format!(
                "Waiting {} ms for the next game state",
                POST_GAME_STAGE_DELAY.as_millis()
            ),
        );
        intentional_wait += wait_or_stop(
            POST_GAME_STAGE_DELAY,
            cancel_requested,
            "STOP was requested during the one-second post-game stage wait.",
        )?;
        target_index += 1;
    }

    let solver_target = POST_GAME_TARGETS
        .iter()
        .find(|target| target.stage == PostGameStage::Solver)
        .copied()
        .ok_or_else(|| "Post-game Solver target is not configured.".to_owned())?;


    let [solver_variant] = solver_target.control_variants else {
        return Err("Post-game Solver must have exactly one control variant.".to_owned());
    };
    let solver_variant = *solver_variant;
    let mut halo_round = 1usize;
    let mut solver_click_attempts = 1usize;


    let observation = loop {
        let (observation, timing) = capture_with_client(qmp, socket_path, scan_state, event_tx)
            .map_err(|error| format!("First post-Solver HALO capture failed: {error}"))?;
        capture_timing.add_assign(timing);
        publish_post_action_observation(event_tx, &observation);


        if observation.gameplay_scene
            && matches!(observation.prediction, PredictedAction::Action(_))
        {
            break observation;
        }


        if observation.gameplay_scene
            && matches!(observation.prediction, PredictedAction::NoHighlight)
        {
            require_post_game_observation_retry_budget(halo_round, "post-Solver HALO")?;
            require_post_game_click_retry_budget(solver_click_attempts, "Solver")?;
            let reprobe = qmp
                .probe()
                .map_err(|error| format!("Post-game Solver retry QMP probe failed: {error}"))?;
            validate_probe(&reprobe).map_err(|error| {
                format!("Post-game Solver retry refused by the fresh QMP probe: {error}")
            })?;

            send_state(event_tx, WorkerState::Acting);
            send_status(event_tx, "Click Solver".to_owned());
            let input_started = Instant::now();
            qmp.click_with_hold(
                solver_variant.click_point,
                NOMINAL_FRAME_WIDTH,
                NOMINAL_FRAME_HEIGHT,
                SOLVER_MOUSE_HOLD,
            )
            .map_err(|error| {
                format!(
                    "Post-game Solver retry result is uncertain: {error}. No automatic input retry followed this uncertain QMP result."
                )
            })?;
            input_wall += input_started.elapsed();
            solver_click_attempts = solver_click_attempts.saturating_add(1);
            send_log(
                event_tx,
                format!(
                    "No actionable HALO was visible on observation round {halo_round}; Solver click attempt {solver_click_attempts} sent at guest pixel ({}, {}) with a {} ms hold. Awaiting HALO confirmation; press STOP to end the loop.",
                    solver_variant.click_point.x,
                    solver_variant.click_point.y,
                    SOLVER_MOUSE_HOLD.as_millis(),
                ),
            );

            send_state(event_tx, WorkerState::Verifying);
            intentional_wait += wait_or_stop(
                POST_GAME_STAGE_DELAY,
                cancel_requested,
                "STOP was requested while waiting after a Solver retry.",
            )?;
        } else {
            require_post_game_observation_retry_budget(halo_round, "post-Solver HALO")?;
            send_log(
                event_tx,
                format!(
                    "WAITING: no actionable HALO is visible on observation round {halo_round}, but the Solver control is not safe to retry in this frame; allowing {} ms before the next transition capture. Press STOP to end the loop.",
                    BOARD_TRANSITION_REOBSERVE_DELAY.as_millis(),
                ),
            );
            intentional_wait += wait_or_stop(
                BOARD_TRANSITION_REOBSERVE_DELAY,
                cancel_requested,
                "STOP was requested while waiting for a safe Solver retry frame.",
            )?;
        }

        halo_round = halo_round.saturating_add(1);
    };

    *completed_boards = 0;
    *scan_state = TableauScanState::for_mode(scan_state.mode());
    let _ = scan_state.reconcile_observation(observation.observed_rows);
    send_board_progress(event_tx, scan_state, *completed_boards);
    send_status(event_tx, "Board 1 ready".to_owned());
    send_log(
        event_tx,
        format!(
            "Post-game restart verified: first HALO acquired after {halo_round} post-Solver observation round(s) and {solver_click_attempts} Solver click attempt(s); board counter reset to 0/{boards_per_game}."
        ),
    );
    send_log(
        event_tx,
        format!(
            "PROFILE post-game restart: captures={}, reserve={:.1} ms, screendump={:.1} ms, read={:.1} ms, decode={:.1} ms, detection={:.1} ms, input-wall(inclusive)={:.1} ms, intentional-waits={:.1} ms, total={:.1} ms.",
            capture_timing.captures,
            milliseconds(capture_timing.reserve),
            milliseconds(capture_timing.screendump),
            milliseconds(capture_timing.file_read),
            milliseconds(capture_timing.decode),
            milliseconds(capture_timing.detection),
            milliseconds(input_wall),
            milliseconds(intentional_wait),
            milliseconds(transition_started.elapsed()),
        ),
    );

    Ok(PostGameOutcome::Restarted(observation))
}


/// Resolve the control variant supported by this immutable observation.
///
/// Gameplay authorises only Solver without a halo. Dialog probes require a
/// non-gameplay frame; overlapping Level Up probes need a connected gold button.
/// Returns `None` when absent and an error for ambiguous or invalid evidence.
pub(super) fn resolve_post_game_target(
    observation: &FrameObservation,
    target: PostGameTarget,
) -> Result<Option<PostGameControlVariant>, String> {


    // Resolve the exact control variant that supplied fresh visual authority.
    // Multiple probes can overlap one tall button, but only a strongly gold
    // bridge across the gap and its left interior authorise the proven low
    // click point. Disconnected probes stay ambiguous and receive no input.
    if target.requires_gameplay_scene {


        let [variant] = target.control_variants else {
            return Err(format!(
                "{} must have exactly one gameplay control variant",
                target.label
            ));
        };
        return Ok((observation.gameplay_scene
            && target.stage == PostGameStage::Solver
            && matches!(observation.prediction, PredictedAction::NoHighlight))
        .then_some(*variant));
    }


    if observation.gameplay_scene {
        return Ok(None);
    }


    if target.control_variants.is_empty() {
        return Err(format!(
            "{} has no configured control variants",
            target.label
        ));
    }

    let mut matched_variant: Option<PostGameControlVariant> = None;


    for variant in target.control_variants.iter().copied() {
        let is_visible =
            has_dialog_gold_button(&observation.frame, variant.probe_bounds).map_err(|error| {
                format!(
                    "{} variant {} target analysis failed: {error}",
                    target.label, variant.label
                )
            })?;


        if !is_visible {
            continue;
        }


        if let Some(previous) = matched_variant {


            if target.stage == PostGameStage::LevelUpOk
                && level_up_has_one_shared_button(&observation.frame)?
            {
                return Ok(Some(previous));
            }
            return Err(format!(
                "{} target is ambiguous: variants {} and {} both matched; no guest input was sent",
                target.label, previous.label, variant.label
            ));
        }
        matched_variant = Some(variant);
    }

    Ok(matched_variant)
}


/// Require gold coverage across both the Level Up bridge and button interior.
///
/// Returns an error for unreadable probe pixels; disconnected gold regions do
/// not authorise the shared lower click point.
fn level_up_has_one_shared_button(frame: &CapturedFrame) -> Result<bool, String> {


    for (label, probe) in [
        ("bridge", LEVEL_UP_SHARED_BUTTON_BRIDGE),
        ("interior", LEVEL_UP_SHARED_BUTTON_INTERIOR),
    ] {
        let fraction = dialog_gold_fraction_per_mille(frame, probe)
            .map_err(|error| format!("Level Up shared button {label} analysis failed: {error}"))?;


        if fraction < LEVEL_UP_SHARED_BUTTON_GOLD_PER_MILLE {
            return Ok(false);
        }
    }
    Ok(true)
}


/// Recognise the specific overlapping-Level-Up error eligible for input-free re-observation.
pub(super) fn is_level_up_ambiguity(error: &str) -> bool {
    error.starts_with(LEVEL_UP_AMBIGUITY_PREFIX)
}


/// Find the latest still-visible control preceding the expected post-game stage.
///
/// Returns its index, target and verified variant, or an error for an invalid
/// index or ambiguous visual evidence. QMP delivery alone does not advance a stage.
pub(super) fn earlier_visible_post_game_target(
    observation: &FrameObservation,
    expected_target_index: usize,
) -> Result<Option<(usize, PostGameTarget, PostGameControlVariant)>, String> {
    // A QMP acknowledgement proves delivery, not that the guest accepted the
    // click. Re-detect earlier controls before authorising a later-stage click.
    let earlier_targets = POST_GAME_TARGETS
        .get(..expected_target_index)
        .ok_or_else(|| format!("invalid post-game target index: {expected_target_index}"))?;


    for (index, target) in earlier_targets.iter().copied().enumerate().rev() {


        if let Some(variant) = resolve_post_game_target(observation, target)? {
            return Ok(Some((index, target, variant)));
        }
    }

    Ok(None)
}


/// Allow a recognised Level Up dialog to supersede an already-verified board transition.
///
/// Returns false without board-completion authority and propagates probe ambiguity.
pub(super) fn level_up_overrides_board_progress(
    board_completion_verified: bool,
    observation: &FrameObservation,
) -> Result<bool, String> {


    // A recognised Level Up dialog is stronger evidence than the earlier
    // progress-bar branch and prevents a stale redeal state from looping.
    if !board_completion_verified {
        return Ok(false);
    }

    Ok(resolve_post_game_target(observation, POST_GAME_TARGETS[0])?.is_some())
}


/// Locate an already-visible Level Up or New Game stage in an optional observation.
///
/// Returns `None` when neither is recognised and an error for ambiguous controls.
pub(super) fn known_post_game_stage(
    observation: Option<&FrameObservation>,
) -> Result<Option<usize>, String> {


    let Some(observation) = observation else {
        return Ok(None);
    };


    if resolve_post_game_target(observation, POST_GAME_TARGETS[0])?.is_some() {
        return Ok(Some(0));
    }


    if new_game_visible_while_awaiting_level_up(observation)? {
        return Ok(Some(1));
    }
    Ok(None)
}


/// Check for a positively recognised New Game control; propagate invalid or ambiguous probes.
pub(super) fn new_game_visible_while_awaiting_level_up(
    observation: &FrameObservation,
) -> Result<bool, String> {
    Ok(resolve_post_game_target(observation, POST_GAME_TARGETS[1])?.is_some())
}


/// Permit another observation below the configured limit; otherwise return a labelled stop reason.
pub(super) fn require_post_game_observation_retry_budget(
    observation_round: usize,
    target_label: &str,
) -> Result<(), String> {


    if observation_round < POST_GAME_MAX_OBSERVATION_ROUNDS {
        return Ok(());
    }

    Err(format!(
        "Post-game {target_label} was not safely resolved after {observation_round} observation rounds; the bounded retry budget is exhausted and no further guest input was sent."
    ))
}


/// Permit another confirmed-delivery click below the stage limit; otherwise return a stop reason.
pub(super) fn require_post_game_click_retry_budget(
    completed_attempts: usize,
    target_label: &str,
) -> Result<(), String> {


    if completed_attempts < POST_GAME_MAX_CLICK_ATTEMPTS {
        return Ok(());
    }

    Err(format!(
        "Post-game {target_label} remained visible after {completed_attempts} confirmed-delivery click attempts; the bounded click budget is exhausted and no further guest input was sent."
    ))
}


/// Reject a score-skip retry once its total confirmed-delivery click budget is exhausted.
pub(super) fn require_score_skip_click_budget(completed_attempts: usize) -> Result<(), String> {


    if completed_attempts < SCORE_SKIP_MAX_CLICK_ATTEMPTS {
        return Ok(());
    }

    Err(format!(
        "Level Up remained unrecognised after {completed_attempts} confirmed-delivery score-skip centre clicks; the bounded score-skip budget is exhausted and no further guest input was sent."
    ))
}


// Context may authorise another centre click only while waiting for Level Up
// on a non-gameplay frame. An ambiguous gameplay frame must remain input-free.
/// Allow score-skip retries only after an input-free follow-up capture outside gameplay while awaiting Level Up.
pub(super) fn score_skip_retry_is_authorised(
    expected_stage: PostGameStage,
    gameplay_scene: bool,
    observation_round: usize,
) -> bool {
    // After the first 3-second wait, give a score or Golden Ticket animation
    // one more input-free observation before repeating a confirmed click.
    expected_stage == PostGameStage::LevelUpOk && !gameplay_scene && observation_round >= 2
}


/// Send one bounded score-skip click after cancellation checks and a fresh running-VM probe.
///
/// Returns input wall time. Budget, probe, cancellation and uncertain-delivery
/// errors stop the controller without replaying the click.
fn send_score_skip_click(
    qmp: &mut QmpClient,
    cancel_requested: &AtomicBool,
    completed_attempts: usize,
) -> Result<Duration, String> {
    require_score_skip_click_budget(completed_attempts)?;
    let attempt = completed_attempts.saturating_add(1);


    if cancel_requested.load(Ordering::Acquire) {
        return Err(format!(
            "STOP was requested before score-skip click attempt {attempt}; no guest input was sent."
        ));
    }

    let reprobe = qmp
        .probe()
        .map_err(|error| format!("Score-skip click attempt {attempt} QMP probe failed: {error}"))?;
    validate_probe(&reprobe).map_err(|error| {
        format!("Score-skip click attempt {attempt} was refused by the fresh QMP probe: {error}")
    })?;


    if cancel_requested.load(Ordering::Acquire) {
        return Err(format!(
            "STOP was requested after the score-skip probe and before click attempt {attempt}; no guest input was sent."
        ));
    }

    let input_started = Instant::now();
    qmp.click_with_hold(
        SCORE_SKIP_CONTROL.click_point,
        NOMINAL_FRAME_WIDTH,
        NOMINAL_FRAME_HEIGHT,
        POST_GAME_MOUSE_HOLD,
    )
    .map_err(|error| {
        format!(
            "Score-skip click attempt {attempt} result is uncertain: {error}. No automatic retry followed the uncertain QMP result."
        )
    })?;
    Ok(input_started.elapsed())
}


/// Return the concise status label for the pending post-game control.
pub(super) const fn post_game_click_status(stage: PostGameStage) -> &'static str {


    match stage {
        PostGameStage::LevelUpOk => "Click Level Up OK",
        PostGameStage::NewGame => "Start New Game",
        PostGameStage::Play => "Click Play",
        PostGameStage::Solver => "Click Solver",
    }
}
