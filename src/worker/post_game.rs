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
    capture::CapturedFrame,
    geometry::PixelPoint,
    parameters::{
        BOARD_TRANSITION_REOBSERVE_DELAY, LEVEL_UP_APPEAR_DELAY, NOMINAL_FRAME_HEIGHT,
        NOMINAL_FRAME_WIDTH, POST_GAME_MAX_CLICK_ATTEMPTS, POST_GAME_MAX_OBSERVATION_ROUNDS,
        POST_GAME_MOUSE_HOLD, POST_GAME_STAGE_DELAY, POST_GAME_TARGETS, PostGameControlVariant,
        PostGameStage, PostGameTarget, SCORE_SKIP_CONTROL, SCORE_SKIP_MAX_CLICK_ATTEMPTS,
        SOLVER_MOUSE_HOLD, TERMINAL_CONTROL_SEARCH_ATTEMPTS,
    },
    pyramid_progress::ChallengeCompletion,
    pyramid_recon::vision::{BoardObservation, ReconScene, ReconVision, StockFace},
    qmp::QmpClient,
    strategy::ControllerContext,
    terminal_controls::{
        Control, control_point,
        profiles::{self, GameControlProfile},
    },
    tracker::{PredictedAction, TableauScanState},
};


use super::control_search::ControlLookup;


pub(super) mod normal_reward;


#[cfg(test)]
mod pyramid_independent_tests;


/// Restart input authority selected by the caller, never inferred from HALOs.
#[derive(Clone, Copy)]
enum RestartPolicy<'a> {
    /// Finish the same reward controls but release control before NewGame.
    ChallengeTarget(&'a ControllerContext, ChallengeCompletion),
    /// Preserve the existing shared Solver/HALO controller.
    ComputerVision(Option<NormalPyramidCompletion>),
    /// Stop before Solver and prove a complete untouched Pyramid board.
    IndependentPyramid(&'a ReconVision, Option<NormalPyramidCompletion>),
}


impl<'a> RestartPolicy<'a> {


    /// Return calibrated full-board recognition only for the independent policy.
    fn pyramid_vision(self) -> Option<&'a ReconVision> {
        match self {
            Self::ComputerVision(..) | Self::ChallengeTarget(..) => None,
            Self::IndependentPyramid(vision, _) => Some(vision),
        }
    }


    /// Existing Solver and HALO recovery is exclusive to Computer Vision.
    fn allows_solver(self) -> bool {
        matches!(self, Self::ComputerVision(..))
    }


    /// Positive Pyramid reward recognition is independent of new-board readiness.
    fn is_pyramid(self) -> bool {
        !matches!(self, Self::ComputerVision(..))
    }


    /// Actual final completion is distinct from strategy and current control pixels.
    fn has_normal_completion(self) -> bool {
        matches!(
            self,
            Self::ComputerVision(Some(_)) | Self::IndependentPyramid(_, Some(_))
        )
    }


    /// Normal reward pixels require the caller's actual final-completion capability.
    fn normal_reward(self, frame: &CapturedFrame) -> Result<bool, String> {
        Ok(self.has_normal_completion() && normal_reward::observe(frame)?)
    }


    /// Completion-only runs retain their original context/STOP reservation throughout.
    fn check(self, events: &WorkerEventSink, cancel: &AtomicBool) -> Result<(), String> {
        if let Self::ChallengeTarget(context, _) = self {
            events
                .latest_frame
                .reconnaissance
                .check_active(context, cancel)?;
        }
        Ok(())
    }


    /// Recheck the same actual requirement on each current terminal observation.
    ///
    /// Known contradictory counters are fatal. Unknown scenes have no control
    /// authority. Returns an error for malformed pixels or a changed requirement.
    fn challenge_completion(
        self,
        frame: &CapturedFrame,
    ) -> Result<Option<ChallengeCompletion>, String> {
        let Self::ChallengeTarget(_, bound) = self else {
            return Ok(None);
        };
        let observed =
            crate::pyramid_progress::observe_bound_challenge_complete(frame, bound.progress())?;
        if observed.is_some_and(|proof| !proof.progress().follows(bound.progress())) {
            return Err(
                "Challenge reward requirement changed; no further input authorised".to_owned(),
            );
        }
        Ok(observed)
    }


    /// Use Level Up proof or a positive challenge counting instruction, never a missing button.
    fn pyramid_reward(self, frame: &CapturedFrame) -> Result<bool, String> {
        Ok(recognised_pyramid_terminal(frame)?
            || self.normal_reward(frame)?
            || self
                .challenge_completion(frame)?
                .is_some_and(ChallengeCompletion::reward_counting))
    }
}


/// Caller-bound positive completion of the last board in a normal Pyramid series.
///
/// Construction checks the numerical last-board position and actual execution
/// outcome. The session caller must retain its qualified normal-series witness
/// in the same worker reservation, context and board generation; a displayed
/// challenge fraction or a planned final gesture cannot supply that witness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PyramidRestartEntry {
    /// Exact observed execution outcome, never delivery-only acknowledgement.
    completion: super::computed_route::BoardCompletion,
}


impl PyramidRestartEntry {


    /// Bind an actual final outcome to the caller's positively qualified last board.
    ///
    /// `progress` is the completed-board count before execution, from the caller's
    /// retained normal-series witness. `BoardClear` requires verified empty-tableau
    /// pixels; `GameComplete` requires paired specific Pyramid terminal scenes.
    /// Neither acknowledged input nor a future expected count is completion proof.
    ///
    /// # Errors
    /// Returns an error for any non-final/unsupported count, unconfirmed delivery
    /// or next-board outcome. This function performs no capture or guest input.
    pub(super) fn verified_third_completion(
        progress: crate::pyramid_progress::BoardCount,
        completion: super::computed_route::BoardCompletion,
    ) -> Result<Self, String> {
        let target = crate::game::GameMode::Pyramid.profile().boards_per_game;
        if usize::from(progress.target) != target
            || progress.completed.checked_add(1) != Some(progress.target)
            || !matches!(
                completion,
                super::computed_route::BoardCompletion::BoardClear
                    | super::computed_route::BoardCompletion::GameComplete
            )
        {
            return Err("Independent Pyramid restart requires actual verified final-board completion bound to the normal three-board series; delivery, planned goals and next-board pixels grant no restart authority".to_owned());
        }
        Ok(Self { completion })
    }
}


/// Typed normal final-completion authority, independent of the chosen readiness strategy.
///
/// Construction is limited to the A* actual-third receipt or verified SOLVER
/// action result. Counts, reward pictures, planned actions and HALOs cannot mint it.
#[derive(Clone, Copy)]
pub(super) struct NormalPyramidCompletion {
    /// Only validated constructors in this module can mint normal completion authority.
    _private: (),
}


impl NormalPyramidCompletion {


    /// Preserve the existing independent caller's actual final-clear capability.
    fn independent(_entry: PyramidRestartEntry) -> Self {
        Self { _private: () }
    }


    /// Admit only an actual Pyramid final effect, never HALO-only continuation.
    pub(super) fn from_solver(
        mode: crate::game::GameMode,
        success: &super::ActionSuccess,
    ) -> Option<Self> {
        (mode == crate::game::GameMode::Pyramid
            && success.board_completed
            && success.series_complete
            && !success.continued_from_halo
            && success.completed_boards == mode.profile().boards_per_game)
            .then_some(Self { _private: () })
    }
}


/// Result of the shared terminal-screen controller.
pub(super) enum PostGameOutcome {
    /// Reward handling reached its policy endpoint, without clicking Continue or NewGame.
    Completed,
    /// A new game was started and its selected policy's fresh readiness proof captured.
    Restarted(FrameObservation),
    /// Fresh gameplay disproved stale completion state, so existing play can resume.
    RecoveredGameplay(FrameObservation),
}


/// Revalidate the retained actual challenge receipt and release control without transport I/O.
///
/// Counting-stage and Continue-stage completion both belong to the operator.
/// No reward, capture, Continue, NewGame or next-board acquisition is authorised.
pub(super) fn finish_pyramid_challenge(
    context: &ControllerContext,
    frame: &CapturedFrame,
    proof: ChallengeCompletion,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> Result<(), String> {
    let policy = RestartPolicy::ChallengeTarget(context, proof);
    policy.check(events, cancel)?;
    if context.game_mode != crate::game::GameMode::Pyramid
        || crate::pyramid_progress::observe_bound_challenge_complete(frame, proof.progress())?
            != Some(proof)
    {
        return Err(
            "Challenge handoff lacks its bound actual terminal receipt; no input authorised"
                .to_owned(),
        );
    }
    policy.check(events, cancel)?;
    send_log(events, "Challenge Complete verified; control released to the operator without reward, Continue, NewGame or further capture input.".to_owned());
    Ok(())
}


/// Advance verified end-of-game screens through score skip, Level Up, New Game, Play and Solver.
///
/// `initial_observation` may establish an already-visible terminal stage. Fresh
/// observations authorise each control, with bounded observation/click budgets.
/// Returns the first actionable restarted or recovered gameplay frame; returns
/// an error on cancellation, capture failure, unsafe recognition or uncertain input.
#[expect(
    clippy::too_many_arguments,
    reason = "retains existing controller state and explicit verified completion capability"
)]
pub(super) fn run_post_game_restart(
    qmp: &mut QmpClient,
    socket_path: &Path,
    initial_observation: Option<&FrameObservation>,
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    normal_completion: Option<NormalPyramidCompletion>,
) -> Result<PostGameOutcome, String> {
    run_post_game_restart_with_policy(
        qmp,
        socket_path,
        initial_observation,
        scan_state,
        completed_boards,
        event_tx,
        cancel_requested,
        RestartPolicy::ComputerVision(normal_completion),
        None,
    )
}


/// Reuse verified terminal controls, then require two full Pyramid starting-board frames.
///
/// Each already captured frame replaces `latest_frame` before recognition or STOP
/// checks, including on failure; retention itself adds no capture or input.
/// This policy never activates Solver, reads Solver state or waits for a HALO.
/// Only a positively recognised native Pyramid terminal entry or full initial
/// board is accepted. Supported dealing after Play waits read-only for a bounded
/// complete pair. STOP, uncertain delivery, unsupported evidence, exhaustion or
/// inconsistent complete arrangements end authority without recovery clicks.
#[expect(
    clippy::too_many_arguments,
    reason = "retains explicit restart state, calibration and caller-owned failure pixels"
)]
pub(super) fn run_independent_pyramid_restart_retaining_frame(
    qmp: &mut QmpClient,
    socket_path: &Path,
    initial_observation: Option<&FrameObservation>,
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    vision: &ReconVision,
    latest_frame: &mut Option<CapturedFrame>,
) -> Result<PostGameOutcome, String> {
    if let Some(observation) = initial_observation {
        *latest_frame = Some(observation.frame.clone());
    }
    if scan_state.mode() != crate::game::GameMode::Pyramid {
        return Err("Independent Pyramid restart requires the selected Pyramid game".to_owned());
    }
    require_independent_restart_active(cancel_requested)?;
    if let Some(observation) = initial_observation
        && !recognised_pyramid_terminal(&observation.frame)?
        && independent_pyramid_start(vision, &observation.frame)?.is_none()
    {
        return Err("Independent Pyramid restart has no positive terminal or full initial-board evidence; no input authorised".to_owned());
    }
    run_post_game_restart_with_policy(
        qmp,
        socket_path,
        initial_observation,
        scan_state,
        completed_boards,
        event_tx,
        cancel_requested,
        RestartPolicy::IndependentPyramid(vision, None),
        Some(latest_frame),
    )
}


/// Reuse existing controls after the caller has actually verified the normal third clear.
///
/// `entry` belongs to the caller's current reservation/context/board generation,
/// and `initial_observation` is its fresh paired transition result. A specific
/// Pyramid terminal or the expected non-gameplay New Game stage admits the
/// unchanged independent controller. The menu itself is neither completion nor
/// reward evidence and never grants score-skip authority. Every later stage
/// still requires a fresh capture, probe and STOP check; Solver remains excluded.
/// The caller must separately prove actual normal 1-of-3 before acquiring again.
///
/// # Errors
/// Returns an error before controller input for a wrong selected game, revoked
/// STOP/context, gameplay, unknown control or unsupported terminal entry. Existing
/// capture, stage, uncertain-input and starting-board failures propagate unchanged.
#[expect(
    clippy::too_many_arguments,
    reason = "keeps the actual-clear permit and existing caller-owned restart state explicit"
)]
pub(super) fn run_witnessed_pyramid_restart_retaining_frame(
    qmp: &mut QmpClient,
    socket_path: &Path,
    initial_observation: &FrameObservation,
    entry: PyramidRestartEntry,
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    vision: &ReconVision,
    latest_frame: &mut Option<CapturedFrame>,
) -> Result<PostGameOutcome, String> {
    *latest_frame = Some(initial_observation.frame.clone());
    if scan_state.mode() != crate::game::GameMode::Pyramid {
        return Err("Independent Pyramid restart requires the selected Pyramid game".to_owned());
    }
    require_independent_restart_active(cancel_requested)?;
    if initial_observation.gameplay_scene
        || (!recognised_pyramid_terminal(&initial_observation.frame)?
            && !expected_pyramid_new_game(&initial_observation.frame)?
            && !normal_reward::observe(&initial_observation.frame)?)
    {
        return Err("Witnessed independent Pyramid restart has no supported non-gameplay terminal or expected New Game control; no input authorised".to_owned());
    }
    send_log(
        event_tx,
        format!(
            "Independent Pyramid restart retains actual normal third-board {:?} authority; the current control supplies stage evidence only.",
            entry.completion,
        ),
    );
    run_post_game_restart_with_policy(
        qmp,
        socket_path,
        Some(initial_observation),
        scan_state,
        completed_boards,
        event_tx,
        cancel_requested,
        RestartPolicy::IndependentPyramid(
            vision,
            Some(NormalPyramidCompletion::independent(entry)),
        ),
        Some(latest_frame),
    )
}


/// Shared bounded terminal stages with an explicit gameplay-readiness policy.
#[expect(
    clippy::too_many_arguments,
    reason = "shared controller retains explicit existing state and selected restart policy"
)]
fn run_post_game_restart_with_policy(
    qmp: &mut QmpClient,
    socket_path: &Path,
    initial_observation: Option<&FrameObservation>,
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    policy: RestartPolicy<'_>,
    mut latest_frame: Option<&mut Option<CapturedFrame>>,
) -> Result<PostGameOutcome, String> {
    let _game_win_images = super::begin_game_win_images(event_tx)?;
    policy.check(event_tx, cancel_requested)?;
    // Advances the verified end-of-game UI through OK, New Game, Play, and Solver.
    // Every control uses a deliberate hold. Acknowledged dialog controls are
    // retired from input authority; only the expected stage is considered next.
    // Missing controls receive at most two input-free fresh captures before stopping.
    // Only a clean settled-size OK first awaiting confirmation on observation
    // three may reserve one final fresh 200 ms geometry witness.
    let transition_started = Instant::now();
    let mut capture_timing = CaptureTiming::default();
    let mut intentional_wait = Duration::ZERO;
    let mut input_wall = Duration::ZERO;
    let boards_per_game = scan_state.mode().profile().boards_per_game;
    let profile = profiles::for_game(scan_state.mode());
    // Only a positively confirmed TriPeaks terminal entry gets the narrow skip
    // evidence policy. Pyramid and existing score-counting transitions retain theirs.
    let guarded_tripeaks_entry = scan_state.mode() == crate::game::GameMode::TriPeaks
        && initial_observation.is_some_and(|observation| {
            super::tripeaks_terminal::terminal_kind(&observation.frame)
                .is_ok_and(|kind| kind.is_some())
        });
    let direct_level_up_entry = guarded_tripeaks_entry
        && initial_observation.is_some_and(|observation| {
            super::tripeaks_terminal::is_direct_level_up_overlay(&observation.frame) == Ok(true)
        });


    if let Some(observation) = initial_observation {
        if let Some(slot) = latest_frame.as_deref_mut() {
            *slot = Some(observation.frame.clone());
        }
        publish_post_game_observation(event_tx, observation);
    }
    if let Some(vision) = policy.pyramid_vision()
        && let Some(observation) = initial_observation
        && independent_pyramid_start(vision, &observation.frame)?.is_some()
    {
        return verify_independent_pyramid_start(
            qmp,
            socket_path,
            Some(observation),
            scan_state,
            completed_boards,
            event_tx,
            cancel_requested,
            vision,
            false,
            &mut latest_frame,
        );
    }
    let initial_challenge_reward = matches!(policy, RestartPolicy::ChallengeTarget(..))
        && initial_observation.is_some_and(|observation| {
            crate::pyramid_progress::is_challenge_complete(&observation.frame) == Ok(true)
        });
    let normal_pyramid = policy.has_normal_completion();
    let initial_normal_reward = initial_observation
        .map(|observation| policy.normal_reward(&observation.frame))
        .transpose()?
        .unwrap_or(false);
    let mut known_terminal_stage = if initial_challenge_reward || initial_normal_reward {
        None
    } else if direct_level_up_entry {
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
            policy.check(event_tx, cancel_requested)?;
            send_status(event_tx, "Verifying game-end transition".to_owned());
            let (observation, timing) = capture_restart_observation(
                qmp,
                socket_path,
                scan_state,
                event_tx,
                &mut latest_frame,
            )
            .map_err(|error| {
                format!("Pre-score transition capture failed before input: {error}")
            })?;
            capture_timing.add_assign(timing);
            publish_post_game_observation(event_tx, &observation);
            policy.check(event_tx, cancel_requested)?;
            if policy
                .challenge_completion(&observation.frame)?
                .is_some_and(ChallengeCompletion::operator_continue)
            {
                policy.check(event_tx, cancel_requested)?;
                send_log(
                    event_tx,
                    "Challenge Complete with Continue verified; control released to the operator."
                        .to_owned(),
                );
                return Ok(PostGameOutcome::Completed);
            }


            if let Some(vision) = policy.pyramid_vision() {
                require_independent_restart_active(cancel_requested)?;
                if independent_pyramid_start(vision, &observation.frame)?.is_some() {
                    return verify_independent_pyramid_start(
                        qmp,
                        socket_path,
                        Some(&observation),
                        scan_state,
                        completed_boards,
                        event_tx,
                        cancel_requested,
                        vision,
                        false,
                        &mut latest_frame,
                    );
                }
                if observation.gameplay_scene {
                    return Err("Independent Pyramid restart found gameplay without a full untouched board; Solver and input recovery withheld".to_owned());
                }
            }


            if policy.allows_solver()
                && observation.gameplay_scene
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
            let observed_stage = if policy.is_pyramid()
                && !recognised_pyramid_terminal(&observation.frame)?
                && !(normal_pyramid && expected_pyramid_new_game(&observation.frame)?)
            {
                None
            } else if foreground_level_up {
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

            let reward_visible = if policy.is_pyramid() || normal_pyramid {
                policy.pyramid_reward(&observation.frame)?
            } else {
                guarded_tripeaks_entry
                    && super::tripeaks_terminal::is_reward_overlay(&observation.frame)?
            };
            consecutive_reward = if reward_visible {
                consecutive_reward.saturating_add(1)
            } else {
                0
            };
            consecutive_non_gameplay = if observation.gameplay_scene {
                0
            } else {
                consecutive_non_gameplay.saturating_add(1)
            };


            if ((guarded_tripeaks_entry || policy.is_pyramid() || normal_pyramid)
                && consecutive_reward < 2)
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
            policy.check(event_tx, cancel_requested)?;
            input_wall += send_score_skip_click(qmp, cancel_requested, score_skip_click_attempts)?;
            score_skip_click_attempts = score_skip_click_attempts.saturating_add(1);
            send_log(
                event_tx,
                format!(
                    "Two consecutive {} captures preceded score-counting/reward panel click attempt {score_skip_click_attempts}/{SCORE_SKIP_MAX_CLICK_ATTEMPTS} at guest pixel ({}, {}) with a {} ms hold; waiting {} ms before classifying Level Up.",
                    if guarded_tripeaks_entry {
                        "positively recognised TriPeaks reward"
                    } else {
                        "non-gameplay"
                    },
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
        if !policy.allows_solver() && target.stage == PostGameStage::Solver {
            break;
        }
        let mut observation_round = 1usize;
        let mut consecutive_post_ok_reward = 0usize;
        let mut consecutive_normal_reward = 0usize;
        let mut consecutive_control_misses = 0usize;
        let mut final_ok_confirmation = false;
        let mut lookup = bounded_control(target.stage).map(|control| {
            ControlLookup::new(scan_state.mode().label(), profile, control, event_tx)
        });


        let matched_variant = loop {
            policy.check(event_tx, cancel_requested)?;
            send_status(event_tx, format!("Waiting for {}", target.label));
            if observation_round > 1
                && let Some(lookup) = lookup.as_mut()
            {
                lookup.note_recapture();
            }
            let (observation, timing) = capture_restart_observation(
                qmp,
                socket_path,
                scan_state,
                event_tx,
                &mut latest_frame,
            )
            .map_err(|error| {
                if let Some(lookup) = lookup.as_mut() {
                    lookup.set_outcome("capture-error");
                }
                format!(
                    "Post-game {} capture failed before input: {error}.",
                    target.label
                )
            })?;
            capture_timing.add_assign(timing);
            publish_post_game_observation(event_tx, &observation);
            policy.check(event_tx, cancel_requested)?;
            if policy
                .challenge_completion(&observation.frame)?
                .is_some_and(ChallengeCompletion::operator_continue)
            {
                policy.check(event_tx, cancel_requested)?;
                if let Some(lookup) = lookup.as_mut() {
                    lookup.set_outcome("completed-before-Continue");
                }
                send_log(event_tx, "Challenge rewards complete; Continue freshly verified and left to the operator.".to_owned());
                return Ok(PostGameOutcome::Completed);
            }
            if matches!(policy, RestartPolicy::ChallengeTarget(..)) && observation.gameplay_scene {
                return Err("Challenge reward flow returned gameplay before its verified stop boundary; no recovery or restart input authorised".to_owned());
            }


            if let Some(vision) = policy.pyramid_vision() {
                require_independent_restart_active(cancel_requested)?;
                if independent_pyramid_start(vision, &observation.frame)?.is_some() {
                    if let Some(lookup) = lookup.as_mut() {
                        lookup.set_outcome("independent-full-board");
                    }
                    return verify_independent_pyramid_start(
                        qmp,
                        socket_path,
                        Some(&observation),
                        scan_state,
                        completed_boards,
                        event_tx,
                        cancel_requested,
                        vision,
                        false,
                        &mut latest_frame,
                    );
                }
                if observation.gameplay_scene {
                    if let Some(lookup) = lookup.as_mut() {
                        lookup.set_outcome("independent-partial-board");
                    }
                    return Err("Independent Pyramid restart found partial or unsupported gameplay; no Solver or recovery input authorised".to_owned());
                }
            }


            if policy.allows_solver()
                && !final_ok_confirmation
                && observation.gameplay_scene
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
                if let Some(lookup) = lookup.as_mut() {
                    lookup.set_outcome("recovered-gameplay");
                }
                return Ok(PostGameOutcome::RecoveredGameplay(observation));
            }


            let current = if let Some(lookup) = lookup.as_mut() {
                if observation.gameplay_scene {
                    lookup.forget_ok_geometry();
                    Ok(None)
                } else {
                    let raw_point = lookup.search(&observation.frame)?;
                    let point = lookup.confirm_ok(&observation.frame, raw_point)?;
                    if direct_level_up_entry
                        && target.stage == PostGameStage::LevelUpOk
                        && !super::tripeaks_terminal::is_direct_level_up_context(
                            &observation.frame,
                        )?
                    {
                        lookup.set_outcome("rejected-context");
                        send_log(event_tx, "TriPeaks OK search has no independent Level Up panel authority; no click is authorised.".to_owned());
                        Ok(None)
                    } else {
                        Ok(measured_control(
                            bounded_control(target.stage).expect("bounded lookup stage"),
                            point,
                            profile,
                        ))
                    }
                }
            } else {
                resolve_post_game_target_for_entry(
                    &observation,
                    target,
                    direct_level_up_entry,
                    profile,
                )
            };
            // A fresh match for the expected stage advances the sequence. Once
            // it is visible, pixels in a completed stage's search region must
            // not override it or authorise replay of the completed control.
            if let Some(variant) = current? {
                if matches!(policy, RestartPolicy::ChallengeTarget(..))
                    && target.stage == PostGameStage::NewGame
                {
                    policy.check(event_tx, cancel_requested)?;
                    if let Some(lookup) = lookup.as_mut() {
                        lookup.set_outcome("completed-before-NewGame");
                    }
                    send_log(event_tx, "Shared GAME WIN reward sequence complete; NewGame freshly recognised, control released before NewGame, Play or Solver input.".to_owned());
                    return Ok(PostGameOutcome::Completed);
                }
                if let Some(lookup) = lookup.as_mut() {
                    lookup.set_outcome("matched");
                }
                break Some(variant);
            }
            // The reserved fourth observation belongs only to current OK
            // readiness. It cannot grant later-control, reward or gameplay
            // recovery authority after the normal search budget is exhausted.
            if final_ok_confirmation {
                if let Some(lookup) = lookup.as_mut() {
                    lookup.set_outcome("not-found");
                }
                return Err(format!(
                    "Post-game {} did not become ready in {} consecutive fresh captures; the sole final OK confirmation was not ready. Latest failed frame retained; no fifth observation or alternative control input is authorised.",
                    target.label,
                    consecutive_control_misses + 1,
                ));
            }

            // New Game also covers games that do not award a level. A confirmed
            // TriPeaks foreground modal must still be dismissed before its underlay.
            let foreground_level_up = target.stage == PostGameStage::LevelUpOk
                && (lookup.as_ref().is_some_and(ControlLookup::has_ok_panel)
                    || (scan_state.mode() == crate::game::GameMode::TriPeaks
                        && super::tripeaks_terminal::is_direct_level_up_context(
                            &observation.frame,
                        )?));
            if target.stage == PostGameStage::LevelUpOk
                && !foreground_level_up
                && resolve_post_game_target_logged(
                    &observation,
                    POST_GAME_TARGETS[1],
                    false,
                    &format!("{} alternative", scan_state.mode().label()),
                    event_tx,
                    profile,
                )?
                .is_some()
            {
                if let Some(lookup) = lookup.as_mut() {
                    lookup.set_outcome("advanced-to-New-Game");
                }
                send_log(
                    event_tx,
                    "RECOVERED: New Game is visible in a fresh non-gameplay frame; optional Level Up is skipped. Waiting for a second capture to verify New Game before sending input."
                        .to_owned(),
                );
                intentional_wait += wait_or_stop(
                    POST_GAME_STAGE_DELAY,
                    cancel_requested,
                    "STOP was requested while verifying the New Game transition.",
                )
                .inspect_err(|_| {
                    if let Some(lookup) = lookup.as_mut() {
                        lookup.set_outcome("cancelled");
                    }
                })?;
                break None;
            }


            let normal_reward_visible = policy.normal_reward(&observation.frame)?;
            consecutive_normal_reward = if normal_reward_visible {
                consecutive_normal_reward.saturating_add(1)
            } else {
                0
            };
            let post_ok_reward = target.stage == PostGameStage::NewGame
                && click_attempts[0] > 0
                && ((direct_level_up_entry
                    && super::tripeaks_terminal::is_reward_overlay(&observation.frame)?)
                    || normal_reward_visible);
            consecutive_post_ok_reward = if post_ok_reward {
                consecutive_post_ok_reward.saturating_add(1)
            } else {
                0
            };

            if policy.allows_solver()
                && observation.gameplay_scene
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
                )
                .inspect_err(|_| {
                    if let Some(lookup) = lookup.as_mut() {
                        lookup.set_outcome("cancelled");
                    }
                })?;
                observation_round = observation_round.saturating_add(1);
                continue;
            }

            require_post_game_observation_retry_budget(observation_round, target.label)?;


            let guarded_post_ok_skip = post_ok_reward && consecutive_post_ok_reward >= 2;
            let positive_reward = if policy.is_pyramid() || normal_pyramid {
                policy.pyramid_reward(&observation.frame)?
            } else {
                super::tripeaks_terminal::is_reward_overlay(&observation.frame)?
            };
            let score_retry = score_skip_retry_is_authorised(
                target.stage,
                observation.gameplay_scene,
                observation_round,
            ) && !direct_level_up_entry
                && !foreground_level_up
                && positive_reward
                && (!normal_pyramid || consecutive_normal_reward >= 2);

            if let Some(lookup) = lookup.as_mut() {
                consecutive_control_misses += 1;
                if consecutive_control_misses >= TERMINAL_CONTROL_SEARCH_ATTEMPTS
                    && !(guarded_post_ok_skip || (score_retry && positive_reward))
                {
                    if lookup.reserve_final_ok_confirmation(consecutive_control_misses) {
                        final_ok_confirmation = true;
                    } else {
                        lookup.set_outcome("not-found");
                        return Err(format!(
                            "Post-game {} did not become ready in {consecutive_control_misses} consecutive fresh captures (missing controls or unsettled OK); the normal search limit is {TERMINAL_CONTROL_SEARCH_ATTEMPTS}, with one final input-free confirmation only for eligible glyph, interrupted or late-body OK confirmation evidence on observation three. Latest failed frame retained for inspection; no failed frame authorised a control click.",
                            target.label,
                        ));
                    }
                }
            }

            // A positively recognised reward transition retains its existing
            // input authority. An unknown non-gameplay frame cannot renew the
            // missing-control allowance by triggering a speculative score skip.
            if guarded_post_ok_skip || score_retry {
                consecutive_control_misses = 0;
                if let Some(lookup) = lookup.as_mut() {
                    lookup.set_outcome("reward-transition");
                }
                send_state(event_tx, WorkerState::Acting);
                send_status(event_tx, "Skip score counting".to_owned());
                policy.check(event_tx, cancel_requested)?;
                input_wall +=
                    send_score_skip_click(qmp, cancel_requested, score_skip_click_attempts)?;
                score_skip_click_attempts = score_skip_click_attempts.saturating_add(1);
                consecutive_post_ok_reward = 0;
                consecutive_normal_reward = 0;
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
                )
                .inspect_err(|_| {
                    if let Some(lookup) = lookup.as_mut() {
                        lookup.set_outcome("cancelled");
                    }
                })?;
                observation_round = observation_round.saturating_add(1);
                continue;
            }

            let retry_delay = lookup
                .as_ref()
                .map_or(BOARD_TRANSITION_REOBSERVE_DELAY, |lookup| {
                    lookup.retry_delay(BOARD_TRANSITION_REOBSERVE_DELAY)
                });
            send_log(
                event_tx,
                format!(
                    "WAITING: post-game stage {} was not recognised on observation round {observation_round}; allowing the guest to animate for {} ms before the next capture. Press STOP to end the loop.",
                    target.label,
                    retry_delay.as_millis(),
                ),
            );
            intentional_wait += wait_or_stop(
                retry_delay,
                cancel_requested,
                "STOP was requested while waiting for the next post-game screen.",
            )
            .inspect_err(|_| {
                if let Some(lookup) = lookup.as_mut() {
                    lookup.set_outcome("cancelled");
                }
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
            return Err(format!(
                "STOP was requested after the fresh {} probe; no further input was sent.",
                target.label
            ));
        }

        send_state(event_tx, WorkerState::Acting);
        policy.check(event_tx, cancel_requested)?;
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

    if let Some(vision) = policy.pyramid_vision() {
        return verify_independent_pyramid_start(
            qmp,
            socket_path,
            None,
            scan_state,
            completed_boards,
            event_tx,
            cancel_requested,
            vision,
            true,
            &mut latest_frame,
        );
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
        let (observation, timing) =
            capture_restart_observation(qmp, socket_path, scan_state, event_tx, &mut latest_frame)
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
            "PROFILE post-game restart: captures={}, reserve={:.1} ms, request={:.1} ms, receive/read={:.1} ms, convert/decode={:.1} ms, png-encode={:.1} ms, detection={:.1} ms, input-wall(inclusive)={:.1} ms, intentional-waits={:.1} ms, total={:.1} ms.",
            capture_timing.captures,
            milliseconds(capture_timing.reserve),
            milliseconds(capture_timing.screendump),
            milliseconds(capture_timing.file_read),
            milliseconds(capture_timing.decode),
            milliseconds(capture_timing.png_encode),
            milliseconds(capture_timing.detection),
            milliseconds(input_wall),
            milliseconds(intentional_wait),
            milliseconds(transition_started.elapsed()),
        ),
    );

    Ok(PostGameOutcome::Restarted(observation))
}


/// Capture once and retain its original pixels before any subsequent recognition or STOP error.
///
/// The optional destination belongs to the independent caller. Computer Vision
/// supplies none and retains its previous capture and publication behaviour.
fn capture_restart_observation(
    qmp: &mut QmpClient,
    socket_path: &Path,
    scan_state: &mut TableauScanState,
    event_tx: &WorkerEventSink,
    latest_frame: &mut Option<&mut Option<CapturedFrame>>,
) -> Result<(FrameObservation, CaptureTiming), String> {
    let Some(slot) = latest_frame.as_deref_mut() else {
        return capture_with_client(qmp, socket_path, scan_state, event_tx);
    };
    let (frame, mut timing) = super::capture_screen(qmp, socket_path, event_tx)?;
    *slot = Some(frame.clone());
    let (observation, detection) =
        super::publish_and_analyse_captured_frame(frame, scan_state, event_tx)?;
    timing.detection = detection;
    super::tcp_capture::log_detection(event_tx, detection);
    Ok((observation, timing))
}


/// Refuse all later independent restart work once monotonic STOP/context cancellation is set.
fn require_independent_restart_active(cancel_requested: &AtomicBool) -> Result<(), String> {
    if cancel_requested.load(Ordering::Acquire) {
        return Err(
            "STOP or context cancellation revoked independent Pyramid restart authority".to_owned(),
        );
    }
    Ok(())
}


/// Recognise a full untouched Pyramid position from actual calibrated rank/suit evidence.
///
/// All 28 fixed tableau identities and initial DL must be unique; DR must have
/// positive empty evidence. Failed/partial recognition returns no authority,
/// never an inferred identity, an inferred empty pile or HALO/Solver state.
fn independent_pyramid_start(
    vision: &ReconVision,
    frame: &CapturedFrame,
) -> Result<Option<BoardObservation>, String> {
    let ReconScene::Board(board) = vision.observe(frame)? else {
        return Ok(None);
    };
    Ok(unique_pyramid_start(&board).then_some(board))
}


/// Check the untouched-pile and unique-card requirements after complete recognition.
fn unique_pyramid_start(board: &BoardObservation) -> bool {
    matches!(board.left, StockFace::Card(left) if !board.tableau.contains(&left))
        && board.right == StockFace::Empty
        && board
            .tableau
            .iter()
            .enumerate()
            .all(|(index, card)| !board.tableau[..index].contains(card))
}


/// Observe a supported dealing frame without granting entry or replay authority.
///
/// Unreadable cards, piles or normal headers can wait. A recognised different
/// normal ordinal or challenge is a contradiction, not a replacement binding.
/// Malformed pixels, unsupported scenes and dialogs remain fatal.
fn independent_start_readiness(
    vision: &ReconVision,
    frame: &CapturedFrame,
) -> Result<crate::pyramid_recon::vision::StartingReadiness, String> {
    use crate::pyramid_recon::vision::StartingReadiness;
    let readiness = vision.starting_readiness(frame)?;
    let expected = crate::pyramid_progress::BoardCount {
        completed: 0,
        target: 3,
    };
    if crate::pyramid_progress::observe_normal_header(frame)?
        .is_some_and(|actual| actual != expected)
        || crate::pyramid_progress::observe_challenge(frame)?.is_some()
    {
        return Err("Independent Pyramid starting-board header contradicts a new normal game; no input authorised".to_owned());
    }
    if crate::pyramid_progress::observe_normal(frame)? != Some(expected) {
        return Ok(StartingReadiness::Pending(
            "New normal-game header is not yet positively confirmed".to_owned(),
        ));
    }
    Ok(match readiness {
        StartingReadiness::Ready(board) if !unique_pyramid_start(&board) => StartingReadiness::Pending(
            "Starting tableau and initial DL do not yet provide 29 unique cards with positively empty DR".to_owned(),
        ),
        other => other,
    })
}


/// Require a fresh matching complete pair within the existing six-observation bound.
///
/// A caller-supplied first observation is its actual most recent capture; otherwise
/// every observation is newly acquired after Play. Supported incomplete evidence
/// clears pair history and waits only between observations. Two complete unequal
/// positions, a positive header contradiction, STOP, context cancellation, invalid
/// pixels or transport failure refuse immediately. This helper never probes or
/// sends input; existing GAME WIN image recording scopes remain active.
#[expect(
    clippy::too_many_arguments,
    reason = "input-free readiness retains the controller's explicit capture and cancellation state"
)]
fn verify_independent_pyramid_start(
    qmp: &mut QmpClient,
    socket_path: &Path,
    first: Option<&FrameObservation>,
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    vision: &ReconVision,
    restarted: bool,
    latest_frame: &mut Option<&mut Option<CapturedFrame>>,
) -> Result<PostGameOutcome, String> {
    use crate::pyramid_recon::vision::StartingReadiness;
    let mut previous: Option<BoardObservation> = None;
    let mut reason = "No matching complete starting-board pair".to_owned();
    let mut settled = None;
    for attempt in 0..super::pyramid_session::TRANSITION_OBSERVATIONS {
        require_independent_restart_active(cancel_requested)?;
        let captured = if attempt == 0 && first.is_some() {
            None
        } else {
            let captured =
                capture_restart_observation(qmp, socket_path, scan_state, event_tx, latest_frame)?
                    .0;
            publish_post_game_observation(event_tx, &captured);
            Some(captured)
        };
        let observation = captured
            .as_ref()
            .or(first)
            .ok_or("Missing supplied first starting observation")?;
        require_independent_restart_active(cancel_requested)?;
        match independent_start_readiness(vision, &observation.frame)? {
            StartingReadiness::Ready(position) => {
                if let Some(original) = &previous {
                    if original.tableau != position.tableau
                        || original.left != position.left
                        || original.right != position.right
                    {
                        return Err("Independent Pyramid starting arrangement changed between two fresh observations; no further input authorised".to_owned());
                    }
                    settled = Some(
                        captured
                            .ok_or("Starting confirmation requires a newly captured observation")?,
                    );
                    break;
                }
                previous = Some(position);
                reason = "Waiting for an equal fresh complete starting position".to_owned();
            }
            StartingReadiness::Pending(error) => {
                previous = None;
                reason = error;
            }
        }
        send_status(
            event_tx,
            format!(
                "Waiting for new Pyramid board to settle — observation {}/{}; no input",
                attempt + 1,
                super::pyramid_session::TRANSITION_OBSERVATIONS
            ),
        );
        if attempt + 1 < super::pyramid_session::TRANSITION_OBSERVATIONS {
            wait_or_stop(
                POST_GAME_STAGE_DELAY,
                cancel_requested,
                "STOP or context cancellation revoked independent Pyramid starting-board observation",
            )?;
        }
    }
    let observation = settled.ok_or_else(|| format!("Independent Pyramid starting board did not settle within {} fresh observations; no input authorised: {reason}", super::pyramid_session::TRANSITION_OBSERVATIONS))?;
    require_independent_restart_active(cancel_requested)?;
    *completed_boards = 0;
    *scan_state = TableauScanState::for_mode(crate::game::GameMode::Pyramid);
    send_board_progress(event_tx, scan_state, 0);
    send_status(
        event_tx,
        "Pyramid board ready for reconnaissance".to_owned(),
    );
    send_log(event_tx,
        "Independent Pyramid restart verified by two fresh matching full tableau/initial-DL/empty-DR and normal-header observations; fresh guarded Recon follows; no Solver or HALO authority used.".to_owned());
    if restarted {
        Ok(PostGameOutcome::Restarted(observation))
    } else {
        Ok(PostGameOutcome::RecoveredGameplay(observation))
    }
}


/// Fixed LEVEL-UP! glyphs sampled every four native pixels from retained Pyramid clean.png.
///
/// # requires solid white, . the gold backdrop and ? excludes antialiasing.
/// Dynamic level/rank artwork, changing text and the Solver region are excluded.
const PYRAMID_LEVEL_UP_TITLE: [&str; 15] = [
    "................................................................................",
    "..???......??????.???.....???.???????..???............??....???..??????....???..",
    "..?##......######.?##.....##?.#######..##?............##....?##..?#######..?##..",
    "..?##......##????..##?....##..###????..##?............##....?##..?##??###?..##..",
    "..?##......##......###...###..###......##?............##....?##..?##...###..##..",
    "..?##......##......?##...##?..###......##?............##....?##..?##...###..##..",
    "..?##......######...##?..##...######?..##?............##....?##..?##...###..##..",
    "..?##......######...###.?##...######?..##?............##....?##..?#######...##..",
    "..?##......##.......?##.##?...###......##?.....####?..##....?##..?#####?....##..",
    "..?##......##........##.##....###......##?.....????...##?...###..?##............",
    "..?##......##........##?##....###......##?............###..?###..?##........??..",
    "..?######..######?...?###?....#######..#######........?#######...?##.......?##..",
    "..?######..######?....###.....#######..#######.........?#####....?##........##..",
    "................................................................................",
    "................................................................................",
];


/// Uncovered fixed skip-prompt letters beneath the foreground native Pyramid panel.
const PYRAMID_DIM_SKIP: [&str; 9] = [
    "???????????????????????????????????????????????????????????????????????????????????",
    "???????????????????????????????????????????????????????????????????????????????????",
    "???????????????????????????????????????????????????????????????????????????????????",
    "???????????????????????????????????????????????????????????????????????????????????",
    ".#....#.#.#...##....?##.?????????????????????????????????????????..##..##..#.#..#..",
    ".#....#.#.#...#??...#.#.?????????????????????????????????????????....#.##..#.#..#..",
    "..###.#.#..##.#.#...?#?.?????????????????????????????????????????..##..#.#.#.#?#...",
    ".........................????????????????????????????????????????............#.....",
    ".........................????????????????????????????????????????............?.....",
];


/// Match solid glyph and backdrop samples separately with the retained 98% text threshold.
fn pyramid_terminal_text(
    frame: &CapturedFrame,
    origin: (u32, u32),
    mask: &[&str],
    glyph: fn([u8; 3]) -> bool,
    backdrop: fn([u8; 3]) -> bool,
) -> bool {
    let mut expected = [0u32; 2];
    let mut matched = [0u32; 2];
    for (row, samples) in mask.iter().enumerate() {
        for (column, sample) in samples.bytes().enumerate() {
            let (index, predicate) = match sample {
                b'#' => (0, glyph),
                b'.' => (1, backdrop),
                _ => continue,
            };
            expected[index] += 1;
            if crate::detector::pixel_rgb(
                frame,
                origin.0 + column as u32 * 4,
                origin.1 + row as u32 * 4,
            )
            .is_some_and(predicate)
            {
                matched[index] += 1;
            }
        }
    }
    (0..2).all(|index| expected[index] > 0 && matched[index] * 100 >= expected[index] * 98)
}


/// Require 95% matching RGB coverage in one checked, fixed native terminal patch.
fn pyramid_terminal_patch(
    frame: &CapturedFrame,
    bounds: crate::geometry::PixelRect,
    predicate: fn([u8; 3]) -> bool,
) -> bool {
    let mut matched = 0u32;
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            matched += u32::from(crate::detector::pixel_rgb(frame, x, y).is_some_and(predicate));
        }
    }
    matched * 100 >= bounds.width * bounds.height * 95
}


/// Positively recognise the retained Pyramid Level Up over reward layout.
///
/// The fixed gold title, uncovered dim skip text, reward borders, foreground
/// panel sides, dim Pyramid legend and bounded OK must all agree. Generic gold,
/// a bare OK or another game's panel grants no terminal authority. Unsupported
/// reward-only/menu layouts return false. This is scene evidence; the caller
/// still owns final-action and consecutive-fresh-frame completion proof.
pub(super) fn recognised_pyramid_terminal(frame: &CapturedFrame) -> Result<bool, String> {
    use crate::geometry::PixelRect;
    if !frame.is_layout_valid() {
        return Err("Pyramid terminal frame has invalid pixel storage".to_owned());
    }
    if (frame.width, frame.height) != (NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT) {
        return Ok(false);
    }
    if !pyramid_terminal_text(
        frame,
        (800, 234),
        &PYRAMID_LEVEL_UP_TITLE,
        |rgb| rgb.into_iter().all(|channel| channel >= 245),
        |[red, green, blue]| blue <= 15 && red >= 100 && green >= 50 && green <= red,
    ) || !pyramid_terminal_text(
        frame,
        (798, 838),
        &PYRAMID_DIM_SKIP,
        |[red, green, blue]| {
            (20..=35).contains(&red) && (27..=40).contains(&green) && (37..=50).contains(&blue)
        },
        |[red, green, blue]| red <= 10 && (12..=23).contains(&green) && (27..=38).contains(&blue),
    ) {
        return Ok(false);
    }
    for bounds in [
        PixelRect::new(507, 880, 3, 20),
        PixelRect::new(1_410, 880, 3, 20),
        PixelRect::new(550, 922, 50, 2),
        PixelRect::new(1_320, 139, 50, 2),
    ] {
        if !pyramid_terminal_patch(frame, bounds, |[red, green, blue]| {
            (10..=15).contains(&red) && (20..=26).contains(&green) && (35..=43).contains(&blue)
        }) {
            return Ok(false);
        }
    }
    for x in [460, 1_457] {
        if !pyramid_terminal_patch(frame, PixelRect::new(x, 400, 3, 40), |rgb| {
            rgb.into_iter().all(|channel| channel >= 235)
        }) {
            return Ok(false);
        }
    }
    for y in [169, 207, 245, 283, 321, 359] {
        if !pyramid_terminal_patch(
            frame,
            PixelRect::new(1_816, y, 68, 2),
            |[red, green, blue]| {
                (8..=12).contains(&red) && (8..=14).contains(&green) && (15..=21).contains(&blue)
            },
        ) {
            return Ok(false);
        }
    }
    Ok(
        control_point(frame, profiles::PYRAMID_CONTROLS.spec(Control::Ok))
            .map_err(|error| error.to_string())?
            != PixelPoint::new(0, 0),
    )
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
fn measured_control(
    control: Control,
    point: PixelPoint,
    profile: &GameControlProfile,
) -> Option<PostGameControlVariant> {
    (point != PixelPoint::new(0, 0)).then_some(PostGameControlVariant::new(
        "bounded-black-run",
        profile.spec(control).bounds,
        point,
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
    resolve_bounded_dialog_target(
        &observation.frame,
        observation.gameplay_scene,
        target,
        profile,
    )
}


/// Reuse one borrowed-frame gameplay veto and bounded dialog-control policy.
///
/// The caller establishes the expected stage; a returned caption point is not
/// completion, reward or arbitrary-menu identity. Invalid native storage and
/// unsupported dialog targets return the existing descriptive lookup error.
fn resolve_bounded_dialog_target(
    frame: &CapturedFrame,
    gameplay_scene: bool,
    target: PostGameTarget,
    profile: &GameControlProfile,
) -> Result<Option<PostGameControlVariant>, String> {
    if gameplay_scene {
        return Ok(None);
    }
    let Some(control) = bounded_control(target.stage) else {
        return Err(format!("{} is not a bounded dialog control", target.label));
    };
    let point = control_point(frame, profile.spec(control))
        .map_err(|error| format!("{} target analysis failed: {error}", target.label))?;
    Ok(measured_control(control, point, profile))
}


/// Observe only the expected Pyramid New Game stage without cloning or capturing pixels.
///
/// Uses the same native Pyramid gameplay predicate and bounded caption resolver
/// as the shared controller. Full or empty gameplay vetoes overlapping stock ink.
/// A true result grants stage evidence only: the caller must already possess
/// actual third-clear authority or a separately recognised completed terminal.
///
/// # Errors
/// Returns an error for invalid/unsupported native pixels or missing New Game
/// configuration. No guest I/O, reward classification or input authority is added.
pub(super) fn expected_pyramid_new_game(frame: &CapturedFrame) -> Result<bool, String> {
    let gameplay_scene = crate::pyramid::is_gameplay_scene(frame)
        .map_err(|error| format!("Pyramid New Game scene analysis failed: {error}"))?;
    let target = POST_GAME_TARGETS
        .iter()
        .find(|target| target.stage == PostGameStage::NewGame)
        .copied()
        .ok_or_else(|| "Pyramid New Game target is not configured".to_owned())?;
    Ok(
        resolve_bounded_dialog_target(frame, gameplay_scene, target, &profiles::PYRAMID_CONTROLS)?
            .is_some(),
    )
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
    if direct_level_up_entry
        && target.stage == PostGameStage::LevelUpOk
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
        return resolve_post_game_target_for_entry(
            observation,
            target,
            direct_level_up_entry,
            profile,
        );
    };
    if observation.gameplay_scene {
        return Ok(None);
    }
    let mut lookup = ControlLookup::new(game, profile, control, event_tx);
    let point = lookup.search(&observation.frame)?;
    if direct_level_up_entry
        && target.stage == PostGameStage::LevelUpOk
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
    // After the first Level Up wait, give a score or Golden Ticket animation
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
