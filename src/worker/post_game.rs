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
    click_shared_solver, milliseconds, publish_post_game_observation, send_board_progress,
    send_log, send_state, send_status, validate_probe, wait_or_stop,
};
use crate::{
    parameters::{
        BOARD_TRANSITION_REOBSERVE_DELAY, LEVEL_UP_APPEAR_DELAY,
        NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH,
        POST_GAME_MAX_CLICK_ATTEMPTS, POST_GAME_MAX_OBSERVATION_ROUNDS, POST_GAME_MOUSE_HOLD,
        POST_GAME_STAGE_DELAY, POST_GAME_TARGETS, PostGameControlVariant, PostGameStage,
        PostGameTarget, SCORE_SKIP_CONTROL, SCORE_SKIP_MAX_CLICK_ATTEMPTS, SOLVER_MOUSE_HOLD,
    },
    geometry::PixelPoint,
    qmp::QmpClient,
    terminal_controls::{Control, control_point, profiles::{self, GameControlProfile}},
    tracker::{PredictedAction, TableauScanState},
};


use super::control_search::ControlLookup;


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
    // Every control uses a deliberate hold. Acknowledged dialog controls are
    // retired from input authority; only the expected stage is considered next.
    // Missing controls receive one input-free fresh capture before stopping.
    let transition_started = Instant::now();
    let mut capture_timing = CaptureTiming::default();
    let mut intentional_wait = Duration::ZERO;
    let mut input_wall = Duration::ZERO;
    let boards_per_game = scan_state.mode().profile().boards_per_game;
    let profile = profiles::for_game(scan_state.mode());
    // Only a positively confirmed TriPeaks terminal entry gets the narrow skip
    // evidence policy. Pyramid and existing score-counting transitions retain theirs.
    let guarded_tripeaks_entry = scan_state.mode() == crate::game::GameMode::TriPeaks
        && initial_observation.is_some_and(|observation|
            super::tripeaks_terminal::terminal_kind(&observation.frame).is_ok_and(|kind| kind.is_some())
        );
    let direct_level_up_entry = guarded_tripeaks_entry
        && initial_observation.is_some_and(|observation|
            super::tripeaks_terminal::is_direct_level_up_overlay(&observation.frame) == Ok(true)
        );


    if let Some(observation) = initial_observation {
        publish_post_game_observation(event_tx, observation);
    }
    let mut known_terminal_stage = if direct_level_up_entry {
        Some(0)
    } else {
        known_post_game_stage(initial_observation, profile)?
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
            publish_post_game_observation(event_tx, &observation);


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

            let foreground_level_up = scan_state.mode() == crate::game::GameMode::TriPeaks
                && super::tripeaks_terminal::is_direct_level_up_overlay(&observation.frame)?;
            let observed_stage = if foreground_level_up {
                Some(0)
            } else {
                known_post_game_stage(Some(&observation), profile)?
            };
            if let Some(stage) = observed_stage {
                known_terminal_stage = Some(stage);
                send_log(event_tx,
                    "A terminal control appeared before score skip; entering its verified stage without a centre click.".to_owned());
                break;
            }

            let reward_visible = guarded_tripeaks_entry
                && super::tripeaks_terminal::is_reward_overlay(&observation.frame)?;
            consecutive_reward = if reward_visible { consecutive_reward.saturating_add(1) } else { 0 };
            consecutive_non_gameplay = if observation.gameplay_scene {
                0
            } else {
                consecutive_non_gameplay.saturating_add(1)
            };


            if (guarded_tripeaks_entry && consecutive_reward < 2)
                || (!guarded_tripeaks_entry && consecutive_non_gameplay < 2)
            {
                require_post_game_observation_retry_budget(
                    pre_score_round,
                    "pre-score transition",
                )?;
                send_log(
                    event_tx,
                    format!(
                        "WAITING: pre-score round {pre_score_round}/{POST_GAME_MAX_OBSERVATION_ROUNDS} has non-gameplay={consecutive_non_gameplay}/2, positive TriPeaks reward={consecutive_reward}/2, guarded TriPeaks entry={guarded_tripeaks_entry}; rechecking in {} ms without input.",
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
                    if guarded_tripeaks_entry { "positively recognised TriPeaks reward" } else { "non-gameplay" },
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
        let mut consecutive_post_ok_reward = 0usize;
        let mut consecutive_control_misses = 0usize;
        let mut lookup = bounded_control(target.stage)
            .map(|control| ControlLookup::new(scan_state.mode().label(), profile, control, event_tx));


        let matched_variant = loop {
            send_status(event_tx, format!("Waiting for {}", target.label));
            if observation_round > 1 {
                if let Some(lookup) = lookup.as_mut() { lookup.note_recapture(); }
            }
            let (observation, timing) =
                capture_with_client(qmp, socket_path, scan_state, event_tx).map_err(|error| {
                    if let Some(lookup) = lookup.as_mut() { lookup.set_outcome("capture-error"); }
                    format!(
                        "Post-game {} capture failed before input: {error}.",
                        target.label
                    )
                })?;
            capture_timing.add_assign(timing);
            publish_post_game_observation(event_tx, &observation);


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
                if let Some(lookup) = lookup.as_mut() { lookup.set_outcome("recovered-gameplay"); }
                return Ok(PostGameOutcome::RecoveredGameplay(observation));
            }

            // New Game also covers games that do not award a level. A confirmed
            // TriPeaks foreground modal must still be dismissed before its underlay.
            let foreground_level_up = target.stage == PostGameStage::LevelUpOk
                && scan_state.mode() == crate::game::GameMode::TriPeaks
                && super::tripeaks_terminal::is_direct_level_up_context(&observation.frame)?;
            if target.stage == PostGameStage::LevelUpOk && !foreground_level_up
                && resolve_post_game_target_logged(&observation, POST_GAME_TARGETS[1], false,
                    &format!("{} alternative", scan_state.mode().label()), event_tx, profile)?.is_some()
            {
                if let Some(lookup) = lookup.as_mut() { lookup.set_outcome("advanced-to-New-Game"); }
                send_log(
                    event_tx,
                    "RECOVERED: New Game is visible in a fresh non-gameplay frame; optional Level Up is skipped. Waiting for a second capture to verify New Game before sending input."
                        .to_owned(),
                );
                intentional_wait += wait_or_stop(
                    POST_GAME_STAGE_DELAY,
                    cancel_requested,
                    "STOP was requested while verifying the New Game transition.",
                ).map_err(|error| {
                    if let Some(lookup) = lookup.as_mut() { lookup.set_outcome("cancelled"); }
                    error
                })?;
                break None;
            }


            let current = if let Some(lookup) = lookup.as_mut() {
                if observation.gameplay_scene {
                    Ok(None)
                } else {
                    let point = lookup.search(&observation.frame)?;
                    if direct_level_up_entry && target.stage == PostGameStage::LevelUpOk
                        && !super::tripeaks_terminal::is_direct_level_up_context(&observation.frame)?
                    {
                        lookup.set_outcome("rejected-context");
                        send_log(event_tx, "TriPeaks OK search has no independent Level Up panel authority; no click is authorised.".to_owned());
                        Ok(None)
                    } else {
                        Ok(measured_control(bounded_control(target.stage).expect("bounded lookup stage"), point, profile))
                    }
                }
            } else {
                resolve_post_game_target_for_entry(&observation, target, direct_level_up_entry, profile)
            };
            // A fresh match for the expected stage advances the sequence. Once
            // it is visible, pixels in a completed stage's search region must
            // not override it or authorise replay of the completed control.
            if let Some(variant) = current? {
                if let Some(lookup) = lookup.as_mut() { lookup.set_outcome("matched"); }
                break Some(variant);
            }

            let post_ok_reward = direct_level_up_entry && target.stage == PostGameStage::NewGame
                && click_attempts[0] > 0 && super::tripeaks_terminal::is_reward_overlay(&observation.frame)?;
            consecutive_post_ok_reward = if post_ok_reward { consecutive_post_ok_reward.saturating_add(1) } else { 0 };

            if observation.gameplay_scene
                && matches!(observation.prediction, PredictedAction::NoHighlight)
                && target.stage != PostGameStage::Solver
            {
                consecutive_control_misses = 0;
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
                ).map_err(|error| {
                    if let Some(lookup) = lookup.as_mut() { lookup.set_outcome("cancelled"); }
                    error
                })?;
                observation_round = observation_round.saturating_add(1);
                continue;
            }

            require_post_game_observation_retry_budget(observation_round, target.label)?;


            let guarded_post_ok_skip = post_ok_reward && consecutive_post_ok_reward >= 2;
            let positive_reward = super::tripeaks_terminal::is_reward_overlay(&observation.frame)?;
            let score_retry = score_skip_retry_is_authorised(
                target.stage, observation.gameplay_scene, observation_round,
            ) && !direct_level_up_entry && positive_reward;

            if let Some(lookup) = lookup.as_mut() {
                consecutive_control_misses += 1;
                if consecutive_control_misses >= 2
                    && !(guarded_post_ok_skip || (score_retry && positive_reward))
                {
                    lookup.set_outcome("not-found");
                    return Err(format!(
                        "Post-game {} was absent in two consecutive fresh captures; the second was an input-free retry. Latest failed frame retained for inspection; neither failed frame authorised a control click.",
                        target.label,
                    ));
                }
            }

            // A positively recognised reward transition retains its existing
            // input authority. An unknown non-gameplay frame cannot renew the
            // two-search allowance by triggering a speculative score skip.
            if guarded_post_ok_skip || score_retry {
                consecutive_control_misses = 0;
                if let Some(lookup) = lookup.as_mut() { lookup.set_outcome("reward-transition"); }
                send_state(event_tx, WorkerState::Acting);
                send_status(event_tx, "Skip score counting".to_owned());
                input_wall +=
                    send_score_skip_click(qmp, cancel_requested, score_skip_click_attempts)?;
                score_skip_click_attempts = score_skip_click_attempts.saturating_add(1);
                consecutive_post_ok_reward = 0;
                send_log(
                    event_tx,
                    format!(
                        "RETRY: post-game stage {} was not recognised on observation round {observation_round}; score-skip centre click attempt {score_skip_click_attempts}/{SCORE_SKIP_MAX_CLICK_ATTEMPTS} was sent at guest pixel ({}, {}) with a {} ms hold, confirmed post-OK reward={guarded_post_ok_skip}. Waiting {} ms before the next capture; press STOP to end the loop.",
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
                ).map_err(|error| {
                    if let Some(lookup) = lookup.as_mut() { lookup.set_outcome("cancelled"); }
                    error
                })?;
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
            ).map_err(|error| {
                if let Some(lookup) = lookup.as_mut() { lookup.set_outcome("cancelled"); }
                error
            })?;
            observation_round = observation_round.saturating_add(1);
        };


        // Lookup wall time ends before QMP probing, clicking and the next-stage wait.
        drop(lookup);
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


        if cancel_requested.load(Ordering::Acquire) {
            return Err(format!("STOP was requested after the fresh {} probe; no further input was sent.", target.label));
        }

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
        publish_post_game_observation(event_tx, &observation);


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


/// Map shared post-game stages to the bounded caption search.
fn bounded_control(stage: PostGameStage) -> Option<Control> {
    match stage {
        PostGameStage::LevelUpOk => Some(Control::Ok),
        PostGameStage::NewGame => Some(Control::NewGame),
        PostGameStage::Play => Some(Control::Play),
        PostGameStage::Solver => None,
    }
}


/// Carry the measured coordinate through the existing post-game input contract.
fn measured_control(control: Control, point: PixelPoint, profile: &GameControlProfile) -> Option<PostGameControlVariant> {
    (point != PixelPoint::new(0, 0)).then_some(PostGameControlVariant::new(
        "bounded-black-run", profile.spec(control).bounds, point,
    ))
}


/// Resolve an expected control from this immutable observation.
///
/// Gameplay authorises only Solver without a halo. The three dialog controls
/// use the first exact-black run inside their own bounded caption rectangle.
pub(super) fn resolve_post_game_target(
    observation: &FrameObservation,
    target: PostGameTarget,
    profile: &GameControlProfile,
) -> Result<Option<PostGameControlVariant>, String> {
    if target.requires_gameplay_scene {
        let [variant] = target.control_variants else {
            return Err(format!("{} must have exactly one gameplay control variant", target.label));
        };
        return Ok((observation.gameplay_scene
            && target.stage == PostGameStage::Solver
            && matches!(observation.prediction, PredictedAction::NoHighlight))
            .then_some(*variant));
    }
    if observation.gameplay_scene {
        return Ok(None);
    }
    let Some(control) = bounded_control(target.stage) else {
        return Err(format!("{} is not a bounded dialog control", target.label));
    };
    let point = control_point(&observation.frame, profile.spec(control))
        .map_err(|error| format!("{} target analysis failed: {error}", target.label))?;
    Ok(measured_control(control, point, profile))
}


/// A direct TriPeaks entry requires the full fresh Level Up fingerprint for each OK.
///
/// A caption match cannot retry a foreground modal after its audited underlay disappears.
pub(super) fn resolve_post_game_target_for_entry(
    observation: &FrameObservation,
    target: PostGameTarget,
    direct_level_up_entry: bool,
    profile: &GameControlProfile,
) -> Result<Option<PostGameControlVariant>, String> {


    if direct_level_up_entry && target.stage == PostGameStage::LevelUpOk
        && !super::tripeaks_terminal::is_direct_level_up_context(&observation.frame)?
    {
        return Ok(None);
    }
    resolve_post_game_target(observation, target, profile)
}


/// Log a recovery or alternative control search without repeating its pixel scan.
fn resolve_post_game_target_logged(
    observation: &FrameObservation,
    target: PostGameTarget,
    direct_level_up_entry: bool,
    game: &str,
    event_tx: &WorkerEventSink,
    profile: &GameControlProfile,
) -> Result<Option<PostGameControlVariant>, String> {
    let Some(control) = bounded_control(target.stage) else {
        return resolve_post_game_target_for_entry(observation, target, direct_level_up_entry, profile);
    };
    if observation.gameplay_scene { return Ok(None); }
    let mut lookup = ControlLookup::new(game, profile, control, event_tx);
    let point = lookup.search(&observation.frame)?;
    if direct_level_up_entry && target.stage == PostGameStage::LevelUpOk
        && !super::tripeaks_terminal::is_direct_level_up_context(&observation.frame)?
    {
        lookup.set_outcome("rejected-context");
        return Ok(None);
    }
    Ok(measured_control(control, point, profile))
}


/// Allow a recognised Level Up dialog to supersede an already-verified board transition.
///
/// Returns false without board-completion authority and propagates invalid pixel storage.
pub(super) fn level_up_overrides_board_progress(
    board_completion_verified: bool,
    observation: &FrameObservation,
    profile: &GameControlProfile,
) -> Result<bool, String> {


    // A recognised Level Up dialog is stronger evidence than the earlier
    // progress-bar branch and prevents a stale redeal state from looping.
    if !board_completion_verified {
        return Ok(false);
    }

    Ok(resolve_post_game_target(observation, POST_GAME_TARGETS[0], profile)?.is_some())
}


/// Locate an already-visible Level Up or New Game stage in an optional observation.
///
/// Returns `None` when neither is recognised and an error for invalid pixel storage.
pub(super) fn known_post_game_stage(
    observation: Option<&FrameObservation>,
    profile: &GameControlProfile,
) -> Result<Option<usize>, String> {


    let Some(observation) = observation else {
        return Ok(None);
    };


    if new_game_visible_while_awaiting_level_up(observation, profile)? {
        return Ok(Some(1));
    }

    if resolve_post_game_target(observation, POST_GAME_TARGETS[0], profile)?.is_some() {
        return Ok(Some(0));
    }
    Ok(None)
}


/// Check for a bounded New Game match; propagate invalid pixel storage.
pub(super) fn new_game_visible_while_awaiting_level_up(
    observation: &FrameObservation,
    profile: &GameControlProfile,
) -> Result<bool, String> {
    Ok(resolve_post_game_target(observation, POST_GAME_TARGETS[1], profile)?.is_some())
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
