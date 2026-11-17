#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Verified TriPeaks route playback store and live QMP execution driver.
//!
//! Shares [`TriPeaksSessionDriver`], [`deliver_and_observe_tripeaks_move`], and
//! [`restore_board_to_document_root`] with [`super::tripeaks_reconnaissance`] so Manual mode
//! (`SINGLE STEP`, `MULTI STEP`, `UNDO ALL`) and automatic sessions enforce identical pre-input
//! scene verification, card-level post-move confirmation, Challenge handoff, and uncertainty locks.

use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use super::{
    CaptureBackend, CaptureMode, QmpClient, WorkerEvent, WorkerEventSink,
    game_operations::{NativeOperations, TrackedOperations},
    pyramid_pause,
    pyramid_stop::StopTrace,
    selected_capture_backend, tripeaks_calculation,
    tripeaks_reconnaissance::{
        LiveStepOutcome, TriPeaksSessionDriver, TriPeaksStepOptions, TriPeaksStepTiming,
        authorise_tripeaks_input, deliver_and_observe_tripeaks_move,
        restore_board_to_document_root, verify_snapshot_matches_document_root,
        verify_snapshot_matches_position,
    },
};
use crate::{
    capture::CapturedFrame,
    game::GameMode,
    game_operations::OperationError,
    pyramid_workflow_model::RunTermination,
    recon::Objective,
    strategy::SolvingStrategy,
    tripeaks_model::{
        FULL_TABLEAU_MASK, TriPeaksMove, TriPeaksPosition, apply_move, to_vec,
        verify_move_sequence, verify_objective_route,
    },
    tripeaks_workflow_model::format_tripeaks_objective,
};

/// Operator-requested live playback action bound by [`Store::reserve`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaybackAction {
    /// Execute and verify exactly one planned move from the active position.
    SingleStep,
    /// Execute planned moves sequentially up to completion, `Stop after step`, or `STOP`.
    MultiStep,
    /// Restore the guest board via `Undo All` back to the documented `T = 0` root layout.
    UndoAll,
}

/// Verified physical outcome ending one [`PlaybackAction`] invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Completion {
    /// One move succeeded and the board remains mid-route.
    Step,
    /// The final route move satisfied the target objective cleanly on gameplay.
    ObjectiveReached,
    /// All `28` tableau cards were cleared cleanly (`BoardCleared`).
    BoardCleared,
    /// Final tap dealt and twice-confirmed an untouched `T = 0` next board (`hash`).
    NextBoardDealt {
        /// Semantic root hash of the freshly dealt board.
        root_hash: u64,
    },
    /// Normal 3rd-board `Congratulations` or `LEVEL-UP!` overlay was twice-confirmed.
    BoardWinOverlay,
    /// `Challenge Complete!` (or XP-counting / `LEVEL-UP!` Challenge handoff) was verified.
    ChallengeComplete,
    /// Best banked partial route completed cleanly (ending on gameplay or expected draw modal).
    PartialComplete,
    /// `Undo All` returned the board cleanly to the document's `T = 0` root position.
    RestoredToRoot,
}

/// Coalesced live execution telemetry consumed by the UI between completions.
#[derive(Clone, Debug)]
pub(crate) struct Progress {
    /// Owning calculation identity.
    pub plan: tripeaks_calculation::Identity,
    /// Distinct playback reservation serial.
    pub run_id: u64,
    /// Verified moves completed so far.
    pub cursor: usize,
    /// Total planned moves in the active candidate route.
    pub total: usize,
    /// Guarded QMP deliveries attempted by this playback run.
    pub attempted: usize,
    /// Guarded QMP deliveries acknowledged by QEMU.
    pub acknowledged: usize,
    /// Cumulative score gain from `T = 0` after `cursor` moves.
    pub score_delta: u32,
    /// Cumulative tableau cards cleared (`0..=28`) after `cursor` moves.
    pub cards_cleared: u8,
    /// Final completion kind once settled.
    pub completion: Option<Completion>,
}

/// Structured failure packet retained when TriPeaks live route playback refuses or aborts.
#[derive(Clone, Debug)]
pub struct Failure {
    /// Plain-English operator diagnostic message.
    pub error: String,
    /// Retained failure artifact directory when disk persistence succeeded.
    pub directory: Option<PathBuf>,
    /// Secondary error if disk persistence could not complete.
    pub persistence_error: Option<String>,
}

/// Extract route metadata `(moves, objective_to_verify, is_partial, escalated, challenge_mode)` from
/// an independently planned calculation outcome.
fn outcome_route(
    outcome: &tripeaks_calculation::Outcome,
) -> Result<(&[TriPeaksMove], Objective, bool, bool), String> {
    match outcome {
        tripeaks_calculation::Outcome::Solved {
            objective,
            moves,
            escalated_clear_board,
            ..
        } => {
            let verify_obj = if *escalated_clear_board {
                Objective::ClearBoard
            } else {
                *objective
            };
            Ok((moves.as_slice(), verify_obj, false, *escalated_clear_board))
        }
        tripeaks_calculation::Outcome::Partial {
            objective, moves, ..
        } => Ok((moves.as_slice(), *objective, true, false)),
        tripeaks_calculation::Outcome::Cancelled(_)
        | tripeaks_calculation::Outcome::Failed { .. } => {
            Err("No replay-verified TriPeaks route is available".to_owned())
        }
    }
}

/// Immutable verified candidate bundled with its owning calculation identity.
#[derive(Clone)]
struct Cached {
    /// Original calculation correlation identity.
    plan: tripeaks_calculation::Identity,
    /// Verified document, bytes, and solved/partial outcome.
    report: Arc<tripeaks_calculation::Report>,
}

/// Mutable execution cursor anchored cleanly at a verified physical endpoint.
#[derive(Clone)]
struct LiveState {
    /// Logical board state reached after `cursor` verified moves.
    position: TriPeaksPosition,
    /// Index of the next move in `moves` (`0..=moves.len()`).
    cursor: usize,
    /// `true` when `position` has already been verified against the guest during this or prior runs.
    bound_on_guest: bool,
}

impl LiveState {
    /// Create initial root state matching `T = 0` of `slots`.
    fn at_root(
        slots: &[crate::recon::CardSlot; crate::tripeaks_model::DEAL_SLOT_COUNT],
    ) -> Result<Self, String> {
        let position = TriPeaksPosition::from_root(slots).map_err(|e| e.to_string())?;
        Ok(Self {
            position,
            cursor: 0,
            bound_on_guest: false,
        })
    }
}

/// Candidate constructed strictly outside locks after defensive model replay passes.
pub(super) struct Prepared {
    /// Verified candidate installed into [`Store`], or `None` when outcome carried no route.
    cached: Option<Cached>,
}

/// Refusal distinguishing cooperative cancellation from replay/schema corruption.
#[derive(Debug, thiserror::Error)]
pub(super) enum PreparationError {
    /// Calculation cancelled before/during defensive replay.
    #[error("TriPeaks route preparation cancelled")]
    Cancelled,
    /// Invalid document schema or replay mismatch.
    #[error("{0}")]
    Failed(String),
}

impl Prepared {
    /// Validate `report` without holding any calculation or route mutexes.
    pub(super) fn new(
        plan: tripeaks_calculation::Identity,
        report: Arc<tripeaks_calculation::Report>,
        cancel: &AtomicBool,
    ) -> Result<Self, PreparationError> {
        let Ok((moves, verify_obj, is_partial, _)) = outcome_route(&report.outcome) else {
            return Ok(Self { cached: None });
        };
        if cancel.load(Ordering::Acquire) {
            return Err(PreparationError::Cancelled);
        }
        report.document.validate().map_err(|err| {
            PreparationError::Failed(format!("TriPeaks document validation failed: {err}"))
        })?;
        let _ = TriPeaksPosition::from_root(&report.document.slots).map_err(|err| {
            PreparationError::Failed(format!("TriPeaks root position failed: {err}"))
        })?;
        if is_partial {
            verify_move_sequence(&report.document.slots, moves).map_err(|err| {
                PreparationError::Failed(format!("TriPeaks partial route replay refused: {err}"))
            })?;
        } else {
            verify_objective_route(&report.document.slots, moves, verify_obj).map_err(|err| {
                PreparationError::Failed(format!("TriPeaks solved route replay refused: {err}"))
            })?;
        }
        if cancel.load(Ordering::Acquire) {
            return Err(PreparationError::Cancelled);
        }
        Ok(Self {
            cached: Some(Cached { plan, report }),
        })
    }
}

/// Worker-owned reservation and cursor state for TriPeaks Manual and Session playback.
#[derive(Default)]
pub(crate) struct Store {
    /// Currently prepared offline route candidate.
    cached: Option<Cached>,
    /// Verified physical playback state along `cached`.
    live: Option<LiveState>,
    /// Monotonically increasing run identifier.
    serial: u64,
    /// Active exclusive playback invocation owner.
    running: Option<(tripeaks_calculation::Identity, u64)>,
    /// Cooperative request to stop cleanly after completing the current verified step.
    stop_after_step: bool,
    /// Coalesced live step progress awaiting UI poll.
    progress: Option<Progress>,
}

impl Store {
    /// Install a privately prepared candidate while idle.
    pub(super) fn install(&mut self, prepared: Option<Prepared>) -> Result<(), String> {
        if self.running.is_some() {
            return Err("TriPeaks playback still owns its completion".into());
        }
        self.cached = prepared.and_then(|prepared| prepared.cached);
        self.live = None;
        self.stop_after_step = false;
        self.progress = None;
        Ok(())
    }

    /// Clear candidate and live history when context changes or session ends.
    pub(super) fn revoke(&mut self) {
        self.cached = None;
        self.retire_live();
    }

    /// Clear mutable live cursor while keeping immutable offline calculation data.
    fn retire_live(&mut self) {
        self.live = None;
        self.stop_after_step = false;
        self.progress = None;
    }

    /// True when `plan` is active, idle, and has at least one remaining step.
    pub(crate) fn available(&self, plan: &tripeaks_calculation::Identity) -> bool {
        self.running.is_none()
            && self.cached.as_ref().is_some_and(|cached| {
                &cached.plan == plan
                    && outcome_route(&cached.report.outcome).is_ok_and(|(moves, _, _, _)| {
                        self.live
                            .as_ref()
                            .map_or(!moves.is_empty(), |live| live.cursor < moves.len())
                    })
            })
    }

    /// True when `plan` is active and idle, allowing `Undo All` restoration to the document root.
    pub(crate) fn undo_available(&self, plan: &tripeaks_calculation::Identity) -> bool {
        self.running.is_none()
            && self
                .cached
                .as_ref()
                .is_some_and(|cached| &cached.plan == plan)
    }

    /// Reserve exclusive playback permission for `action`.
    pub(super) fn reserve(
        &mut self,
        plan: &tripeaks_calculation::Identity,
        action: PlaybackAction,
    ) -> Result<u64, String> {
        let allowed = match action {
            PlaybackAction::SingleStep | PlaybackAction::MultiStep => self.available(plan),
            PlaybackAction::UndoAll => self.undo_available(plan),
        };
        if !allowed {
            return Err("TriPeaks candidate is unavailable; CALCULATE again".into());
        }
        let next = self
            .serial
            .checked_add(1)
            .ok_or("TriPeaks playback identity exhausted")?;
        if self.live.is_none() {
            let cached = self.cached.as_ref().ok_or("Missing TriPeaks candidate")?;
            self.live = Some(LiveState::at_root(&cached.report.document.slots)?);
        }
        self.serial = next;
        self.running = Some((plan.clone(), next));
        self.stop_after_step = false;
        self.progress = None;
        Ok(next)
    }

    /// Take the latest coalesced playback snapshot without granting authority.
    pub(super) fn take_progress(&mut self) -> Option<Progress> {
        self.progress.take()
    }

    /// Request graceful stop after finishing the current step cleanly.
    pub(super) fn request_stop_after_step(
        &mut self,
        plan: &tripeaks_calculation::Identity,
        run_id: u64,
    ) -> Result<(), String> {
        self.check(plan, run_id)?;
        self.stop_after_step = true;
        Ok(())
    }

    /// Check active reservation ownership.
    fn check(&self, plan: &tripeaks_calculation::Identity, run_id: u64) -> Result<(), String> {
        if self.running.as_ref() != Some(&(plan.clone(), run_id))
            || self.live.is_none()
            || !self
                .cached
                .as_ref()
                .is_some_and(|cached| cached.plan == *plan)
        {
            return Err("TriPeaks playback authority revoked".into());
        }
        Ok(())
    }

    /// Retire a reservation refused prior to transport.
    pub(super) fn reject(
        &mut self,
        plan: &tripeaks_calculation::Identity,
        run_id: u64,
    ) -> Result<(), String> {
        self.finish(plan, run_id, None, None)
    }

    /// Retire exact playback ownership and commit verified cursor progression or revoke candidate.
    fn finish(
        &mut self,
        plan: &tripeaks_calculation::Identity,
        run_id: u64,
        state: Option<LiveState>,
        completion: Option<Completion>,
    ) -> Result<(), String> {
        if self.running.as_ref() != Some(&(plan.clone(), run_id)) {
            return Err("Stale TriPeaks playback completion".into());
        }
        self.running = None;
        self.stop_after_step = false;
        let Some(completion) = completion else {
            self.retire_live();
            return Ok(());
        };
        let Some(mut state) = state else {
            self.retire_live();
            return Err("Missing TriPeaks completion state".into());
        };
        let Some(cached) = self.cached.as_ref().filter(|cached| &cached.plan == plan) else {
            self.retire_live();
            return Err("TriPeaks playback authority revoked before completion".into());
        };
        let (moves, _, _, _) = outcome_route(&cached.report.outcome)?;
        match completion {
            Completion::Step => {
                if state.cursor >= moves.len() {
                    self.retire_live();
                    return Err("TriPeaks step completion cursor out of bounds".into());
                }
                state.bound_on_guest = true;
                self.live = Some(state);
            }
            Completion::RestoredToRoot => {
                let mut root_state = LiveState::at_root(&cached.report.document.slots)?;
                root_state.bound_on_guest = true;
                self.live = Some(root_state);
            }
            Completion::ObjectiveReached
            | Completion::BoardCleared
            | Completion::NextBoardDealt { .. }
            | Completion::BoardWinOverlay
            | Completion::ChallengeComplete
            | Completion::PartialComplete => {
                self.revoke();
            }
        }
        Ok(())
    }
}

/// Structured result and captured frame returned by [`execute_for_session`].
pub(super) struct SessionExecution {
    /// Route progress on completion, or the persisted failure report on error.
    pub(super) result: Result<Progress, Box<Failure>>,
    /// Causal termination kind distinguishing operator `STOP` from genuine failure.
    pub(super) termination: RunTermination,
    /// Latest verified guest frame captured during playback, if any.
    pub(super) latest_frame: Option<CapturedFrame>,
}

/// Publish coalesced progress while `plan` + `run_id` still own `tripeaks_route`.
fn publish_live_progress(events: &WorkerEventSink, progress: Progress) {
    if let Ok(mut store) = events.latest_frame.tripeaks_route.lock()
        && store.running.as_ref() == Some(&(progress.plan.clone(), progress.run_id))
    {
        store.progress = Some(progress);
    }
}

/// Query whether either operator cancellation/ownership or `stop_after_step` ended execution.
fn stop_after_step_requested(
    events: &WorkerEventSink,
    plan: &tripeaks_calculation::Identity,
    run_id: u64,
) -> Result<bool, String> {
    let store = events
        .latest_frame
        .tripeaks_route
        .lock()
        .map_err(|_| "TriPeaks route store poisoned".to_owned())?;
    store.check(plan, run_id)?;
    Ok(store.stop_after_step)
}

/// Execute one reserved TriPeaks playback action against the live guest using [`TriPeaksSessionDriver`].
///
/// When `record_terminal_win_scope` is `true` (standalone Manual route mode), saves the cached entry
/// `GAME WIN` PNG scope upon observing `BoardWinOverlay` or `ChallengeComplete` (honouring `--no-image`).
/// Automatic sessions in `Continuous` mode pass `false` on intermediate/restartable transitions so
/// [`super::post_game::run_independent_tripeaks_restart_retaining_frame`] owns the single win sequence scope.
pub(super) fn execute_for_session(
    plan: tripeaks_calculation::Identity,
    run_id: u64,
    action: PlaybackAction,
    timings: TriPeaksStepTiming,
    record_terminal_win_scope: bool,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> SessionExecution {
    let mut final_state: Option<LiveState> = None;
    let mut latest_frame: Option<CapturedFrame> = None;
    let mut first_frame: Option<CapturedFrame> = None;
    let mut document_for_diag = None;
    let trace = StopTrace::default();
    let mut balanced = true;
    let mut attempted = 0_usize;
    let mut acknowledged = 0_usize;

    let result: Result<Progress, String> = (|| {
        let (cached, mut current) = {
            let store = events
                .latest_frame
                .tripeaks_route
                .lock()
                .map_err(|_| "TriPeaks route store poisoned".to_owned())?;
            if store.running.as_ref() != Some(&(plan.clone(), run_id)) {
                return Err("TriPeaks playback owner retired".into());
            }
            let cached = store
                .cached
                .as_ref()
                .filter(|c| c.plan == plan)
                .cloned()
                .ok_or("TriPeaks plan retired")?;
            let live = store
                .live
                .as_ref()
                .cloned()
                .ok_or("TriPeaks live history retired")?;
            (cached, live)
        };
        document_for_diag = Some(cached.report.document.clone());
        let (moves, verify_obj, is_partial, escalated_clear_board) =
            outcome_route(&cached.report.outcome)?;
        let slots = cached.report.document.slots;
        let challenge_mode = escalated_clear_board
            || !matches!(cached.report.document.objective, Objective::ClearBoard);

        match selected_capture_backend()? {
            CaptureBackend::Tcp {
                mode: CaptureMode::BgrV3 | CaptureMode::PersistentBgrV3,
                ..
            } => {}
            _ => return Err("Independent TriPeaks playback requires native TCPv3".into()),
        }

        trace.check(&events.latest_frame.reconnaissance, &plan.context, cancel)?;
        let mut qmp = QmpClient::connect(&plan.context.socket_path).map_err(|e| e.to_string())?;
        let context_ref = &plan.context;
        let check_ownership = || {
            trace
                .check(&events.latest_frame.reconnaissance, context_ref, cancel)
                .and_then(|()| {
                    let store = events
                        .latest_frame
                        .tripeaks_route
                        .lock()
                        .map_err(|_| "TriPeaks route store poisoned".to_owned())?;
                    store.check(&plan, run_id)
                })
                .map_err(|err| {
                    if cancel.load(Ordering::Acquire) {
                        OperationError::Cancelled
                    } else {
                        OperationError::Adapter(err)
                    }
                })
        };

        let inner_ops = NativeOperations {
            qmp: &mut qmp,
            socket: &plan.context.socket_path,
            events,
            cancel,
            active: check_ownership,
            guard: authorise_tripeaks_input,
            last_timing: None,
        };
        let route_io = TrackedOperations::new(inner_ops);
        let mut session = crate::game_operations::Session::new(route_io);
        let mut on_frame = |frame: &CapturedFrame| {
            events.publish_diagnostic_frame(frame.clone());
        };
        let mut uncertain_latched = false;
        let mut driver = TriPeaksSessionDriver {
            session: &mut session,
            cancel,
            on_frame: &mut on_frame,
            uncertain_latched: &mut uncertain_latched,
            timings,
        };

        let drive_result: Result<Completion, String> = (|| {
            if action == PlaybackAction::UndoAll {
                let restored =
                    restore_board_to_document_root(&mut driver, "tripeaks_manual_undo_all", &slots)
                        .map_err(|err| err.to_string())?;
                current = LiveState::at_root(&slots)?;
                current.bound_on_guest = true;
                let _ = restored;
                return Ok(Completion::RestoredToRoot);
            }

            if current.cursor >= moves.len() {
                return Err(
                    "TriPeaks route cursor has already reached the end of the route".into(),
                );
            }

            let initial_class = driver
                .acquire_non_settling_scene("tripeaks_route_pre_check")
                .map_err(|err| format!("Pre-execution scene check failed: {err}"))?;
            let first_snap = match initial_class {
                crate::worker::tripeaks_reconnaissance::FrameClassification::Settled(snap) => snap,
                other => {
                    return Err(format!(
                        "Pre-execution expected settled TriPeaks gameplay board, got {other:?}"
                    ));
                }
            };
            let mut current_snap = driver
                .capture_matching_settled(&first_snap, "tripeaks_route_pre_check_confirm")
                .map_err(|err| format!("Pre-execution scene confirmation failed: {err}"))?;

            if current.cursor == 0 && !current.bound_on_guest {
                verify_snapshot_matches_document_root(&current_snap, &slots).map_err(|reason| {
                    format!("Guest board does not match exported T=0 TriPeaks document: {reason}")
                })?;
            } else {
                verify_snapshot_matches_position(&current_snap, &slots, &current.position)
                    .map_err(|reason| {
                        format!(
                            "Guest board diverges from expected position at step {}: {reason}",
                            current.cursor
                        )
                    })?;
            }

            let single_step = action == PlaybackAction::SingleStep;
            let mut newly_observed = Vec::new();
            while current.cursor < moves.len() {
                let check_pause = || {
                    trace.check(&events.latest_frame.reconnaissance, context_ref, cancel)?;
                    Ok(())
                };
                if let Err(pause_err) = events
                    .latest_frame
                    .pause
                    .checkpoint(pyramid_pause::Phase::Gameplay, check_pause)
                {
                    let _ = trace.termination(true, true);
                    return Err(pause_err.message);
                }

                let mv = moves[current.cursor];
                let allow_final_modal = is_partial && current.cursor + 1 == moves.len();
                let step_outcome = deliver_and_observe_tripeaks_move(
                    &mut driver,
                    &current_snap,
                    TriPeaksStepOptions {
                        mv,
                        challenge_mode,
                        allow_final_modal,
                    },
                    &mut newly_observed,
                )
                .map_err(|err| err.to_string())?;

                let (next_pos, _) = apply_move(&current.position, &slots, mv)
                    .map_err(|err| format!("TriPeaks model step replay error: {err}"))?;
                current.position = next_pos;
                current.cursor += 1;
                current.bound_on_guest = true;

                let io_stats = driver.session.io();
                let cards_cleared =
                    u8::try_from((FULL_TABLEAU_MASK & !current.position.tableau_mask).count_ones())
                        .unwrap_or(28);
                publish_live_progress(
                    events,
                    Progress {
                        plan: plan.clone(),
                        run_id,
                        cursor: current.cursor,
                        total: moves.len(),
                        attempted: io_stats.attempted,
                        acknowledged: io_stats.acknowledged,
                        score_delta: current.position.score_delta,
                        cards_cleared,
                        completion: None,
                    },
                );

                match step_outcome {
                    LiveStepOutcome::Settled(next_snap) => {
                        verify_snapshot_matches_position(&next_snap, &slots, &current.position)
                            .map_err(|reason| {
                                format!(
                                    "Post-move board mismatch after step {}: {reason}",
                                    current.cursor
                                )
                            })?;
                        current_snap = next_snap;
                    }
                    LiveStepOutcome::BoardCleared(next_snap) => {
                        current_snap = next_snap;
                        if current.cursor >= moves.len() {
                            return Ok(Completion::BoardCleared);
                        }
                    }
                    LiveStepOutcome::NextBoardDealt { root_hash } => {
                        return Ok(Completion::NextBoardDealt { root_hash });
                    }
                    LiveStepOutcome::BoardWinOverlay => {
                        if record_terminal_win_scope {
                            let _game_win_images = super::begin_game_win_images(events)?;
                        }
                        return Ok(Completion::BoardWinOverlay);
                    }
                    LiveStepOutcome::ChallengeComplete => {
                        if record_terminal_win_scope {
                            let _game_win_images = super::begin_game_win_images(events)?;
                        }
                        return Ok(Completion::ChallengeComplete);
                    }
                    LiveStepOutcome::PartialModal => {
                        return Ok(Completion::PartialComplete);
                    }
                }

                if current.cursor >= moves.len() {
                    if is_partial {
                        return Ok(Completion::PartialComplete);
                    }
                    if verify_obj == Objective::ClearBoard && current.position.tableau_mask == 0 {
                        return Ok(Completion::BoardCleared);
                    }
                    if challenge_mode
                        && !escalated_clear_board
                        && driver
                            .poll_challenge_complete_or_handoff()
                            .map_err(|err| err.to_string())?
                    {
                        if record_terminal_win_scope {
                            let _game_win_images = super::begin_game_win_images(events)?;
                        }
                        return Ok(Completion::ChallengeComplete);
                    }
                    return Ok(Completion::ObjectiveReached);
                }

                if single_step || stop_after_step_requested(events, &plan, run_id)? {
                    return Ok(Completion::Step);
                }
            }
            Ok(if is_partial {
                Completion::PartialComplete
            } else {
                Completion::ObjectiveReached
            })
        })();

        let finished_io = session.into_io();
        attempted = finished_io.attempted;
        acknowledged = finished_io.acknowledged;
        balanced = attempted == acknowledged;
        first_frame = finished_io.first;
        latest_frame = finished_io.latest;

        let completion = drive_result?;
        final_state = Some(current.clone());
        let cleared =
            u8::try_from((FULL_TABLEAU_MASK & !current.position.tableau_mask).count_ones())
                .unwrap_or(28);
        Ok(Progress {
            plan: plan.clone(),
            run_id,
            cursor: current.cursor,
            total: moves.len(),
            attempted,
            acknowledged,
            score_delta: current.position.score_delta,
            cards_cleared: cleared,
            completion: Some(completion),
        })
    })();

    let mut result = result;
    if result.is_err() {
        trace.adopt(trace.termination(true, balanced));
    }
    let retirement = events
        .latest_frame
        .tripeaks_route
        .lock()
        .map_err(|_| "TriPeaks route retirement poisoned".to_owned())
        .and_then(|mut store| {
            if let Err(error) =
                trace.check(&events.latest_frame.reconnaissance, &plan.context, cancel)
            {
                if store.running.as_ref() == Some(&(plan.clone(), run_id)) {
                    store.revoke();
                }
                if result.is_ok() {
                    result = Err(error);
                }
            }
            store.finish(
                &plan,
                run_id,
                final_state,
                result
                    .as_ref()
                    .ok()
                    .and_then(|progress| progress.completion),
            )
        });
    if retirement.is_err() {
        trace.adopt(RunTermination::Failed);
    }
    if let Err(error) = retirement
        && result.is_ok()
    {
        result = Err(error);
    }

    let termination = trace.termination(result.is_err(), balanced);
    let final_result = result.map_err(|error| {
        let mut directory = None;
        let mut persistence_error = None;
        if termination != RunTermination::OperatorStopped {
            let source_bytes = document_for_diag.as_ref().and_then(|doc| to_vec(doc).ok());
            let packet = crate::pyramid_diagnostics::persist_failure(
                crate::pyramid_diagnostics::FailureEvidence {
                    progress: None,
                    context: &plan.context,
                    phase: crate::pyramid_diagnostics::PyramidPhase::Execution,
                    run_id: Some(run_id),
                    error: &error,
                    details: serde_json::json!({
                        "game": "TriPeaks",
                        "action": format!("{action:?}"),
                        "run_id": run_id,
                        "request_id": plan.request_id,
                        "source_generation": plan.source_generation,
                        "attempted": attempted,
                        "acknowledged": acknowledged,
                    }),
                    source_document: source_bytes.as_deref(),
                    first_frame: first_frame.as_ref(),
                    latest_frame: latest_frame.as_ref(),
                },
            );
            directory = packet.directory;
            persistence_error = packet.persistence_error;
        }
        Box::new(Failure {
            error,
            directory,
            persistence_error,
        })
    });

    SessionExecution {
        result: final_result,
        termination,
        latest_frame,
    }
}

/// Format a plain-English Manual playback status/log message for one completed [`Progress`].
pub(crate) fn progress_summary_message(progress: &Progress, objective: Objective) -> String {
    let target = format_tripeaks_objective(objective, None);
    match progress.completion {
        Some(Completion::Step) => format!(
            "Step {}/{} verified ({}/28 tableau cleared, +{} pts).",
            progress.cursor, progress.total, progress.cards_cleared, progress.score_delta
        ),
        Some(Completion::ObjectiveReached) => format!(
            "Route completed for {target} ({}/{} steps, {}/28 tableau cleared, +{} pts).",
            progress.cursor, progress.total, progress.cards_cleared, progress.score_delta
        ),
        Some(Completion::BoardCleared) => format!(
            "All 28 tableau cards cleared ({}/{} steps, +{} pts).",
            progress.cursor, progress.total, progress.score_delta
        ),
        Some(Completion::NextBoardDealt { .. }) => format!(
            "Board cleared and next untouched TriPeaks deal appeared ({}/{} steps, +{} pts).",
            progress.cursor, progress.total, progress.score_delta
        ),
        Some(Completion::BoardWinOverlay) => format!(
            "TriPeaks 3/3 game win confirmed ({}/{} steps, +{} pts).",
            progress.cursor, progress.total, progress.score_delta
        ),
        Some(Completion::ChallengeComplete) => format!(
            "TriPeaks Challenge Complete verified after step {}/{} (+{} pts); zero Continue clicks sent.",
            progress.cursor, progress.total, progress.score_delta
        ),
        Some(Completion::PartialComplete) => format!(
            "Partial banked route finished ({}/{} steps, {}/28 tableau cleared, +{} pts); zero Redeal clicks sent.",
            progress.cursor, progress.total, progress.cards_cleared, progress.score_delta
        ),
        Some(Completion::RestoredToRoot) => {
            "Board restored cleanly via Undo All to the exported T=0 layout.".to_owned()
        }
        None => format!(
            "Executing TriPeaks step {}/{} ({}/28 tableau, +{} pts)…",
            progress.cursor, progress.total, progress.cards_cleared, progress.score_delta
        ),
    }
}

/// Standalone worker runner for [`super::WorkerCommand::ExecuteTriPeaksRoute`].
pub(super) fn run(
    plan: tripeaks_calculation::Identity,
    run_id: u64,
    action: PlaybackAction,
    timings: TriPeaksStepTiming,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) {
    let context_valid = plan.context.validate().map(|()| {
        if plan.context.game_mode != GameMode::TriPeaks
            || plan.context.strategy != SolvingStrategy::ShortestPath
        {
            Err("TriPeaks route playback requires ShortestPath + TriPeaks".to_owned())
        } else {
            Ok(())
        }
    });
    if let Err(err) = context_valid.and_then(|inner| inner) {
        reject(plan, run_id, err, events);
        return;
    }

    let SessionExecution {
        result,
        termination,
        ..
    } = execute_for_session(plan.clone(), run_id, action, timings, true, events, cancel);
    events.latest_frame.reconnaissance.finish();
    let _ = events.send_for_context(
        Some(plan.context.clone()),
        WorkerEvent::TriPeaksRouteFinished {
            plan,
            run_id,
            result,
            termination,
        },
    );
}

/// Refuse a reserved request before live transport while preserving clean store retirement.
pub(super) fn reject(
    plan: tripeaks_calculation::Identity,
    run_id: u64,
    error: String,
    events: &WorkerEventSink,
) {
    let retirement = events
        .latest_frame
        .tripeaks_route
        .lock()
        .map_err(|_| "TriPeaks route retirement poisoned".to_owned())
        .and_then(|mut store| store.reject(&plan, run_id));
    let full_error = match retirement {
        Ok(()) => error,
        Err(sec) => format!("{error}; retirement also failed: {sec}"),
    };
    events.latest_frame.reconnaissance.finish();
    let _ = events.send_for_context(
        Some(plan.context.clone()),
        WorkerEvent::TriPeaksRouteFinished {
            plan,
            run_id,
            result: Err(Box::new(Failure {
                error: full_error,
                directory: None,
                persistence_error: None,
            })),
            termination: RunTermination::Failed,
        },
    );
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        io::{BufRead, BufReader, ErrorKind, Read, Write},
        net::{TcpListener, TcpStream},
        os::unix::net::{UnixListener, UnixStream},
        path::{Path, PathBuf},
        sync::{Mutex, OnceLock, atomic::AtomicU64, mpsc},
        thread,
        time::{Duration, Instant},
    };

    use serde_json::{Value, json};
    use xsar::tcp::protocol::v3;

    use super::super::{LatestFrameSlot, TestCaptureBackend};
    use super::*;
    use crate::{
        capture::decode_png,
        freecell::PLAY_X,
        game_controls::{FREECELL_ROW_Y, LocationRequest, read_card},
        game_operations::CardRead,
        geometry::{PixelPoint, PixelRect},
        parameters::{
            NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH, SHARED_UNDO_ALL_CONTROL, STOCK_CARD_BOUNDS,
            TABLEAU_CARD_REGIONS, TARGET_STOCK, WASTE_CARD_BOUNDS,
        },
        recon::{Card, CardSlot, SlotId},
        strategy::ControllerContext,
        tripeaks_model::{
            COVERING_CHILD_MASKS, DEAL_SLOT_COUNT, DealProvenance, INITIAL_WASTE_SLOT,
            STOCK_CARD_COUNT, STOCK_START_SLOT, TriPeaksDealDocument, slot_to_tableau_coord,
        },
        worker::tripeaks_reconnaissance::{StockObs, TableauSlotObs},
    };

    /// Monotonic counter isolating socket and JSON test paths across parallel threads.
    static NEXT_ROUTE_TEST_ID: AtomicU64 = AtomicU64::new(1);

    /// Cached native `1920×1080` TriPeaks/FreeCell fixture frames and discovered `52`-card corners.
    struct BaseAssets {
        /// Free Cell fixture providing native `34 × 52` corner crops for all `52` cards.
        fc01: CapturedFrame,
        /// Untouched `T = 0` TriPeaks scene (`Undo All` disabled).
        left_ace: CapturedFrame,
        /// Mid-play TriPeaks scene (`Undo All` enabled).
        draw_first: CapturedFrame,
        /// Calibrated Draw-23 dark-green empty Stock tray fixture.
        stock_empty: CapturedFrame,
        /// Corner `(x, y)` origins in `fc01` indexed by canonical card ID `0..52`.
        corner_origins: [(u32, u32); DEAL_SLOT_COUNT],
    }

    /// Global lazy decoded asset bank shared across route unit tests.
    static ROUTE_ASSETS: OnceLock<BaseAssets> = OnceLock::new();

    /// Borrow decoded shared fixtures and discovered `52`-card corner coordinates.
    fn assets() -> &'static BaseAssets {
        ROUTE_ASSETS.get_or_init(|| {
            let decode = |bytes: &[u8], label: &str| {
                decode_png(bytes).unwrap_or_else(|err| panic!("failed decoding {label}: {err}"))
            };
            let fc01 = decode(
                include_bytes!("../../tests/fixtures/freecell-FC01.png"),
                "freecell-FC01",
            );
            let left_ace = decode(
                include_bytes!("../../tests/fixtures/tripeaks-redeal-20261016/left-ace.png"),
                "left-ace",
            );
            let draw_first = decode(
                include_bytes!("../../tests/fixtures/tripeaks-redeal-20261016/draw-first.png"),
                "draw-first",
            );
            let stock_empty = decode(
                include_bytes!(
                    "../../tests/fixtures/tripeaks-calibration-20261116/stock-empty.png"
                ),
                "stock-empty",
            );

            let mut corner_origins = [None; DEAL_SLOT_COUNT];
            for (col_idx, &x) in PLAY_X.iter().enumerate() {
                let max_rows = if col_idx < 4 { 7 } else { 6 };
                for (row_idx, &y) in FREECELL_ROW_Y.iter().take(max_rows).enumerate() {
                    let col_u8 = u8::try_from(col_idx + 1).expect("col in 1..=8");
                    let row_u8 = u8::try_from(row_idx + 1).expect("row in 1..=7");
                    let CardRead::Card(xc) =
                        read_card(&fc01, LocationRequest::freecell_play(col_u8, row_u8))
                            .expect("valid fc01 read")
                    else {
                        panic!("FC01 ({col_u8}, {row_u8}) did not decode as Card");
                    };
                    let rc = Card::try_from(xc).expect("valid canonical card");
                    corner_origins[usize::from(rc.0)] = Some((x, y));
                }
            }
            BaseAssets {
                fc01,
                left_ace,
                draw_first,
                stock_empty,
                corner_origins: corner_origins
                    .map(|opt| opt.expect("FC01 contains all 52 canonical cards")),
            }
        })
    }

    /// Fast route test timings avoiding second-long sleeps on local loopback mocks.
    const fn fast_timings() -> TriPeaksStepTiming {
        TriPeaksStepTiming {
            move_settle_delay: Duration::from_millis(1),
            reobserve_delay: Duration::from_millis(1),
            restore_settle_delay: Duration::from_millis(1),
            max_step_observations: 6,
            max_restore_observations: 8,
        }
    }

    /// Build a canonical `52`-card cyclic solvable `ClearBoard` deal (`28` direct taps from `T = 0`).
    fn cyclic_solvable_slots() -> [CardSlot; DEAL_SLOT_COUNT] {
        let order: [usize; 28] = [
            18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 9, 10, 11, 12, 13, 14, 15, 16, 17, 3, 4, 5, 6,
            7, 8, 0, 1, 2,
        ];
        let mut used = [false; 52];
        let mut suit_cursor = [0_u8; 13];
        let mut take_rank = |rank_idx: u8| -> Card {
            let suit = suit_cursor[usize::from(rank_idx)];
            suit_cursor[usize::from(rank_idx)] = suit + 1;
            let card = Card(suit * 13 + rank_idx);
            used[usize::from(card.0)] = true;
            card
        };
        let mut cards = [Card(0); DEAL_SLOT_COUNT];
        cards[usize::from(INITIAL_WASTE_SLOT.0)] = take_rank(0);
        for (step, &slot) in order.iter().enumerate() {
            cards[slot] = take_rank(((step + 1) % 13) as u8);
        }
        let mut stock_idx = usize::from(STOCK_START_SLOT);
        for (id, &taken) in used.iter().enumerate() {
            if !taken {
                cards[stock_idx] = Card(id as u8);
                stock_idx += 1;
            }
        }
        assert_eq!(stock_idx, DEAL_SLOT_COUNT);
        cards.map(CardSlot::Known)
    }

    /// Copy `src_rect` from `src` onto `dst` starting at `dst_origin`.
    fn copy_rect(
        dst: &mut CapturedFrame,
        dst_origin: PixelPoint,
        src: &CapturedFrame,
        src_rect: PixelRect,
    ) {
        let row_bytes = src_rect.width as usize * 4;
        for row in 0..src_rect.height {
            let src_y = (src_rect.y + row) as usize;
            let dst_y = (dst_origin.y as u32 + row) as usize;
            let src_off = src_y * src.stride + src_rect.x as usize * 4;
            let dst_off = dst_y * dst.stride + (dst_origin.x as u32) as usize * 4;
            dst.pixels[dst_off..dst_off + row_bytes]
                .copy_from_slice(&src.pixels[src_off..src_off + row_bytes]);
        }
    }

    /// Paint one `CLASSIC BLUE` face-down card back at `(dst_x, dst_y, width, 181)`.
    fn paint_face_down_card(dst: &mut CapturedFrame, dst_x: u32, dst_y: u32, width: u32) {
        let base = assets();
        let copy_w = width.min(STOCK_CARD_BOUNDS.width);
        copy_rect(
            dst,
            PixelPoint::new(dst_x as i32, dst_y as i32),
            &base.left_ace,
            PixelRect::new(STOCK_CARD_BOUNDS.x, STOCK_CARD_BOUNDS.y, copy_w, 181),
        );
        if width > copy_w {
            copy_rect(
                dst,
                PixelPoint::new((dst_x + copy_w) as i32, dst_y as i32),
                &base.left_ace,
                PixelRect::new(
                    STOCK_CARD_BOUNDS.x + copy_w - 1,
                    STOCK_CARD_BOUNDS.y,
                    1,
                    181,
                ),
            );
        }
    }

    /// Paint one face-up synthetic TriPeaks card with calibrated `FC01` rank/suit corner.
    fn paint_face_up_card(
        dst: &mut CapturedFrame,
        dst_x: u32,
        dst_y: u32,
        width: u32,
        target: Card,
    ) {
        let base = assets();
        let copy_w = width.min(WASTE_CARD_BOUNDS.width);
        copy_rect(
            dst,
            PixelPoint::new(dst_x as i32, dst_y as i32),
            &base.left_ace,
            PixelRect::new(WASTE_CARD_BOUNDS.x, WASTE_CARD_BOUNDS.y, copy_w, 181),
        );
        if width > copy_w {
            copy_rect(
                dst,
                PixelPoint::new((dst_x + copy_w) as i32, dst_y as i32),
                &base.left_ace,
                PixelRect::new(
                    WASTE_CARD_BOUNDS.x + copy_w - 1,
                    WASTE_CARD_BOUNDS.y,
                    1,
                    181,
                ),
            );
        }
        for dy in 4..177_u32 {
            for dx in 4..width.saturating_sub(4) {
                let offset = (dst_y + dy) as usize * dst.stride + (dst_x + dx) as usize * 4;
                dst.pixels[offset..offset + 4].copy_from_slice(&[254, 254, 254, 255]);
            }
        }
        let (src_x, src_y) = base.corner_origins[usize::from(target.0)];
        for dy in 0..52_u32 {
            for dx in 0..34_u32 {
                if dx + dy < 3 {
                    continue;
                }
                let so = (src_y + dy) as usize * base.fc01.stride + (src_x + dx) as usize * 4;
                let doff = (dst_y + dy) as usize * dst.stride + (dst_x + dx) as usize * 4;
                dst.pixels[doff..doff + 4].copy_from_slice(&base.fc01.pixels[so..so + 4]);
            }
        }
    }

    /// Convert a logical position + truth deck into visual slot observations.
    fn render_bgr_frame(
        truth: &[CardSlot; DEAL_SLOT_COUNT],
        position: TriPeaksPosition,
        undo_enabled: bool,
        corrupt_slot_18: Option<Card>,
    ) -> Vec<u8> {
        let base = assets();
        let mut frame = base.left_ace.clone();
        for y in 96..(96 + 545_u32) {
            let src_y = 650 + (y - 96) % 180;
            let mut x = 115_u32;
            while x < 115 + 1_690 {
                let chunk = ((115 + 1_690) - x).min(520);
                copy_rect(
                    &mut frame,
                    PixelPoint::new(x as i32, y as i32),
                    &base.left_ace,
                    PixelRect::new(128, src_y, chunk, 1),
                );
                x += chunk;
            }
        }
        for (idx, region) in TABLEAU_CARD_REGIONS.iter().enumerate() {
            let bounds = region.card_bounds;
            let bit = 1_u32 << idx;
            let slot_obs = if (position.tableau_mask & bit) == 0 {
                TableauSlotObs::Empty
            } else if (COVERING_CHILD_MASKS[idx] & position.tableau_mask) != 0 {
                TableauSlotObs::FaceDown
            } else if idx == 18
                && let Some(fake) = corrupt_slot_18
            {
                TableauSlotObs::Card(fake)
            } else if let CardSlot::Known(card_val) = truth[idx] {
                TableauSlotObs::Card(card_val)
            } else {
                TableauSlotObs::FaceDown
            };
            match slot_obs {
                TableauSlotObs::Empty => {}
                TableauSlotObs::FaceDown => {
                    paint_face_down_card(&mut frame, bounds.x, bounds.y, bounds.width);
                }
                TableauSlotObs::Card(card_val) => {
                    paint_face_up_card(&mut frame, bounds.x, bounds.y, bounds.width, card_val);
                }
            }
        }
        let stock = if position.stock_cursor >= STOCK_CARD_COUNT {
            StockObs::Empty
        } else {
            StockObs::FaceDown
        };
        if stock == StockObs::Empty {
            copy_rect(
                &mut frame,
                PixelPoint::new(TARGET_STOCK.x as i32, TARGET_STOCK.y as i32),
                &base.stock_empty,
                TARGET_STOCK,
            );
        }
        paint_face_up_card(
            &mut frame,
            WASTE_CARD_BOUNDS.x,
            WASTE_CARD_BOUNDS.y,
            WASTE_CARD_BOUNDS.width,
            position.waste_card,
        );
        if undo_enabled {
            let undo_bounds = SHARED_UNDO_ALL_CONTROL.bounds;
            copy_rect(
                &mut frame,
                PixelPoint::new(undo_bounds.x as i32, undo_bounds.y as i32),
                &base.draw_first,
                undo_bounds,
            );
        }
        let mut bgr = Vec::with_capacity(frame.pixels.len() / 4 * 3);
        for px in frame.pixels.as_chunks::<4>().0 {
            bgr.extend_from_slice(&[px[2], px[1], px[0]]);
        }
        bgr
    }

    /// Cache key for synthesized `24`-bit BGR `BgrV3` capture buffers.
    #[derive(Clone, Copy, Eq, Hash, PartialEq)]
    struct BgrCacheKey {
        /// Remaining tableau bitmask.
        tableau_mask: u32,
        /// Draw index (`0..=23`).
        stock_cursor: u8,
        /// Current Waste card ID.
        waste_id: u8,
        /// `Undo All` toolbar state.
        undo_enabled: bool,
        /// Optional Row-4 visual corruption card ID.
        corrupt_slot_18: Option<u8>,
    }

    /// Shared simulated TriPeaks Windows 11 guest state accessed by QMP and `BgrV3` threads.
    struct MockGuestState {
        /// Ground-truth `52`-card deal matching the candidate document.
        slots: [CardSlot; DEAL_SLOT_COUNT],
        /// Root position at `T = 0`.
        root: TriPeaksPosition,
        /// Current live position mutated on QMP inputs.
        position: TriPeaksPosition,
        /// Moves applied since `T = 0`.
        history: Vec<TriPeaksMove>,
        /// Optional Row-4 card corruption forcing a pre-execution mismatch error.
        corrupt_slot_18: Option<Card>,
        /// Total `input-send-event` clicks/keys committed by the simulated guest.
        qmp_inputs_delivered: usize,
        /// Optional deterministic callback requesting `stop_after_step` on the Nth completed input.
        stop_after_step_hook: Option<(
            usize,
            Arc<WorkerEventSink>,
            tripeaks_calculation::Identity,
            u64,
        )>,
    }

    /// Running loopback QMP `AF_UNIX` + TCP `BgrV3` test harness with RAII shutdown.
    struct LoopbackGuest {
        /// Shared mutable guest board and telemetry state.
        state: Arc<Mutex<MockGuestState>>,
        /// Shutdown latch torn down cleanly when dropped.
        shutdown: Arc<AtomicBool>,
        /// Guard holding the thread-local `CaptureMode::BgrV3` selection.
        _backend: TestCaptureBackend,
        /// Background QMP listener join handle.
        qmp_thread: Option<thread::JoinHandle<()>>,
        /// Background TCP `BgrV3` server join handle.
        tcp_thread: Option<thread::JoinHandle<()>>,
        /// Unix domain socket path removed on drop.
        socket_path: PathBuf,
    }

    impl Drop for LoopbackGuest {
        /// Signal server threads to stop, join them, and clean up the temporary socket file.
        fn drop(&mut self) {
            self.shutdown.store(true, Ordering::Release);
            if let Some(handle) = self.qmp_thread.take() {
                let _ = handle.join();
            }
            if let Some(handle) = self.tcp_thread.take() {
                let _ = handle.join();
            }
            let _ = std::fs::remove_file(&self.socket_path);
        }
    }

    /// Write one newline-delimited QMP packet and flush.
    fn write_qmp_packet(stream: &mut UnixStream, value: Value) -> bool {
        serde_json::to_writer(&mut *stream, &value).is_ok()
            && stream.write_all(b"\n").is_ok()
            && stream.flush().is_ok()
    }

    /// Inverse of QMP `px * 32767 / extent` used to locate clicked controls cleanly.
    fn qmp_to_pixel(qmp: u32, extent: u32) -> i32 {
        ((u64::from(qmp) * u64::from(extent) + 16_383) / 32_767) as i32
    }

    /// Spawn a persistent `AF_UNIX` QMP + TCP `BgrV3` loopback guest serving `slots`.
    fn spawn_loopback_guest(
        socket_path: PathBuf,
        slots: [CardSlot; DEAL_SLOT_COUNT],
        corrupt_slot_18: Option<Card>,
    ) -> LoopbackGuest {
        let _ = assets();
        let root = TriPeaksPosition::from_root(&slots).expect("valid root");
        let state = Arc::new(Mutex::new(MockGuestState {
            slots,
            root,
            position: root,
            history: Vec::new(),
            corrupt_slot_18,
            qmp_inputs_delivered: 0,
            stop_after_step_hook: None,
        }));
        let shutdown = Arc::new(AtomicBool::new(false));

        let qmp_listener = UnixListener::bind(&socket_path).expect("bind QMP unix socket");
        qmp_listener
            .set_nonblocking(true)
            .expect("set QMP listener nonblocking");
        let qmp_state = Arc::clone(&state);
        let qmp_shutdown = Arc::clone(&shutdown);
        let qmp_thread = thread::spawn(move || {
            let mut last_abs: Option<(u32, u32)> = None;
            while !qmp_shutdown.load(Ordering::Acquire) {
                match qmp_listener.accept() {
                    Ok((mut stream, _)) => {
                        let _ = stream.set_nonblocking(false);
                        let _ = stream.set_read_timeout(Some(Duration::from_millis(50)));
                        let mut reader = BufReader::new(stream.try_clone().expect("clone QMP"));
                        if !write_qmp_packet(
                            &mut stream,
                            json!({"QMP": {"version": {"qemu": {"major": 11, "minor": 0, "micro": 0}}, "capabilities": []}}),
                        ) {
                            continue;
                        }
                        while !qmp_shutdown.load(Ordering::Acquire) {
                            let mut line = String::new();
                            match reader.read_line(&mut line) {
                                Ok(0) => break,
                                Ok(_) => {}
                                Err(e)
                                    if matches!(
                                        e.kind(),
                                        ErrorKind::WouldBlock | ErrorKind::TimedOut
                                    ) =>
                                {
                                    continue;
                                }
                                Err(_) => break,
                            }
                            let Ok(req) = serde_json::from_str::<Value>(&line) else {
                                break;
                            };
                            let id = req["id"].as_u64().unwrap_or(0);
                            match req["execute"].as_str().unwrap_or("") {
                                "qmp_capabilities" => {
                                    if !write_qmp_packet(
                                        &mut stream,
                                        json!({"return": {}, "id": id}),
                                    ) {
                                        break;
                                    }
                                }
                                "query-status" => {
                                    if !write_qmp_packet(
                                        &mut stream,
                                        json!({"return": {"running": true, "status": "running"}, "id": id}),
                                    ) {
                                        break;
                                    }
                                }
                                "query-mice" => {
                                    if !write_qmp_packet(
                                        &mut stream,
                                        json!({"return": [{"name": "QEMU HID Tablet", "index": 0, "current": true, "absolute": true}], "id": id}),
                                    ) {
                                        break;
                                    }
                                }
                                "input-send-event" => {
                                    let events = req["arguments"]["events"]
                                        .as_array()
                                        .expect("events array");
                                    let mut gx = None;
                                    let mut gy = None;
                                    let mut hook_to_fire = None;
                                    for ev in events {
                                        match ev["type"].as_str() {
                                            Some("abs") => {
                                                let val = ev["data"]["value"].as_u64().unwrap_or(0)
                                                    as u32;
                                                match ev["data"]["axis"].as_str() {
                                                    Some("x") => gx = Some(val),
                                                    Some("y") => gy = Some(val),
                                                    _ => {}
                                                }
                                            }
                                            Some("btn") if ev["data"]["button"] == "left" => {
                                                let down =
                                                    ev["data"]["down"].as_bool().unwrap_or(false);
                                                if !down && let Some((qx, qy)) = last_abs {
                                                    let px = PixelPoint::new(
                                                        qmp_to_pixel(qx, NOMINAL_FRAME_WIDTH),
                                                        qmp_to_pixel(qy, NOMINAL_FRAME_HEIGHT),
                                                    );
                                                    let mut guard = qmp_state.lock().expect("lock");
                                                    guard.qmp_inputs_delivered += 1;
                                                    if px == SHARED_UNDO_ALL_CONTROL.click_point {
                                                        guard.history.clear();
                                                        guard.position = guard.root;
                                                    } else if let Some((slot_idx, _)) =
                                                        TABLEAU_CARD_REGIONS
                                                            .iter()
                                                            .enumerate()
                                                            .find(|(_, r)| {
                                                                r.click_point.x.abs_diff(px.x) <= 1
                                                                    && r.click_point
                                                                        .y
                                                                        .abs_diff(px.y)
                                                                        <= 1
                                                            })
                                                    {
                                                        let (row, col) = slot_to_tableau_coord(
                                                            SlotId(slot_idx as u8),
                                                        )
                                                        .expect("valid coord");
                                                        let mv = TriPeaksMove::TapTableau {
                                                            row,
                                                            column: col,
                                                        };
                                                        let (next, _) = apply_move(
                                                            &guard.position,
                                                            &guard.slots,
                                                            mv,
                                                        )
                                                        .expect("legal guest tap");
                                                        guard.history.push(mv);
                                                        guard.position = next;
                                                    }
                                                    if let Some((
                                                        target,
                                                        ref sink,
                                                        ref plan,
                                                        run_id,
                                                    )) = guard.stop_after_step_hook
                                                        && guard.qmp_inputs_delivered == target
                                                    {
                                                        hook_to_fire = Some((
                                                            Arc::clone(sink),
                                                            plan.clone(),
                                                            run_id,
                                                        ));
                                                    }
                                                }
                                            }
                                            Some("key") => {
                                                let down =
                                                    ev["data"]["down"].as_bool().unwrap_or(false);
                                                if !down && ev["data"]["key"]["data"] == "d" {
                                                    let mut guard = qmp_state.lock().expect("lock");
                                                    guard.qmp_inputs_delivered += 1;
                                                    let (next, _) = apply_move(
                                                        &guard.position,
                                                        &guard.slots,
                                                        TriPeaksMove::Draw,
                                                    )
                                                    .expect("legal guest draw");
                                                    guard.history.push(TriPeaksMove::Draw);
                                                    guard.position = next;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    if let (Some(x), Some(y)) = (gx, gy) {
                                        last_abs = Some((x, y));
                                    }
                                    if let Some((sink, plan, run_id)) = hook_to_fire {
                                        let _ = sink
                                            .latest_frame
                                            .tripeaks_route
                                            .lock()
                                            .expect("store lock")
                                            .request_stop_after_step(&plan, run_id);
                                    }
                                    if !write_qmp_packet(
                                        &mut stream,
                                        json!({"return": {}, "id": id}),
                                    ) {
                                        break;
                                    }
                                }
                                _ => {
                                    if !write_qmp_packet(
                                        &mut stream,
                                        json!({"return": {}, "id": id}),
                                    ) {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                    }
                    Err(_) => break,
                }
            }
        });

        let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("bind BgrV3 listener");
        let address = tcp_listener.local_addr().expect("tcp addr").to_string();
        tcp_listener
            .set_nonblocking(true)
            .expect("set tcp nonblocking");
        let tcp_state = Arc::clone(&state);
        let tcp_shutdown = Arc::clone(&shutdown);
        let tcp_thread = thread::spawn(move || {
            let mut cache: HashMap<BgrCacheKey, Arc<Vec<u8>>> = HashMap::new();
            let mut seq = 0_u64;
            while !tcp_shutdown.load(Ordering::Acquire) {
                match tcp_listener.accept() {
                    Ok((mut stream, _)) => {
                        let _ = stream.set_nonblocking(false);
                        let _ = stream.set_read_timeout(Some(Duration::from_millis(50)));
                        let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
                        let _ = serve_v3_stream(
                            &mut stream,
                            &tcp_state,
                            &tcp_shutdown,
                            &mut cache,
                            &mut seq,
                        );
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                    }
                    Err(_) => break,
                }
            }
        });

        LoopbackGuest {
            state,
            shutdown,
            _backend: TestCaptureBackend::tcp_mode(address, CaptureMode::BgrV3),
            qmp_thread: Some(qmp_thread),
            tcp_thread: Some(tcp_thread),
            socket_path,
        }
    }

    /// Serve one `BgrV3` client stream using cached synthetic `1920×1080` 24-bit BGR frames.
    fn serve_v3_stream(
        stream: &mut TcpStream,
        state: &Arc<Mutex<MockGuestState>>,
        shutdown: &AtomicBool,
        cache: &mut HashMap<BgrCacheKey, Arc<Vec<u8>>>,
        seq: &mut u64,
    ) -> std::io::Result<()> {
        let mut req_buf = [0_u8; v3::REQUEST_LEN];
        loop {
            if shutdown.load(Ordering::Acquire) {
                return Ok(());
            }
            match stream.read_exact(&mut req_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(()),
                Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                    continue;
                }
                Err(e) => return Err(e),
            }
            let req = v3::Request::from_bytes(req_buf)
                .map_err(|err| std::io::Error::new(ErrorKind::InvalidData, err))?;
            *seq += 1;
            let (slots, key) = {
                let g = state.lock().expect("guest state lock");
                (
                    g.slots,
                    BgrCacheKey {
                        tableau_mask: g.position.tableau_mask,
                        stock_cursor: g.position.stock_cursor,
                        waste_id: g.position.waste_card.0,
                        undo_enabled: !g.history.is_empty(),
                        corrupt_slot_18: g.corrupt_slot_18.map(|c| c.0),
                    },
                )
            };
            let bgr = Arc::clone(cache.entry(key).or_insert_with(|| {
                let pos = TriPeaksPosition {
                    tableau_mask: key.tableau_mask,
                    stock_cursor: key.stock_cursor,
                    waste_card: Card(key.waste_id),
                    streak: 0,
                    score_delta: 0,
                    cleared_by_rank: [0; 13],
                };
                Arc::new(render_bgr_frame(
                    &slots,
                    pos,
                    key.undo_enabled,
                    key.corrupt_slot_18.map(Card),
                ))
            }));
            let header = v3::ResponseHeader::success(
                1,
                req.request_id(),
                *seq,
                7,
                NOMINAL_FRAME_WIDTH,
                NOMINAL_FRAME_HEIGHT,
            )
            .expect("valid v3 response header");
            stream.write_all(&header.to_bytes())?;
            stream.write_all(&bgr)?;
            stream.flush()?;
        }
    }

    /// Build and install a verified `ClearBoard` route candidate into `sink`.
    fn prepare_and_install_candidate(
        socket_path: &Path,
        slots: [CardSlot; DEAL_SLOT_COUNT],
    ) -> (
        Arc<WorkerEventSink>,
        mpsc::Receiver<super::super::ContextualWorkerEvent>,
        tripeaks_calculation::Identity,
        Arc<AtomicBool>,
    ) {
        let id = NEXT_ROUTE_TEST_ID.fetch_add(1, Ordering::Relaxed);
        let context = ControllerContext::new(
            socket_path.to_path_buf(),
            GameMode::TriPeaks,
            SolvingStrategy::ShortestPath,
            id,
        );
        let plan = tripeaks_calculation::Identity {
            context: context.clone(),
            request_id: id,
            source_generation: id,
        };
        let (tx, rx) = mpsc::channel();
        let sink = Arc::new(WorkerEventSink {
            events: tx,
            latest_frame: LatestFrameSlot::default(),
            capture_context: Mutex::new(None),
        });
        let doc = TriPeaksDealDocument {
            provenance: DealProvenance::RestoredAtRoot,
            probes_used: 0,
            objective: Objective::ClearBoard,
            slots,
            planned_route: None,
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let report = tripeaks_calculation::search_deal_with_control(
            tripeaks_calculation::SearchInput {
                document: doc,
                objective_override: None,
                path: std::env::temp_dir().join(format!(
                    "tripeaks-route-src-{}-{id}.json",
                    std::process::id()
                )),
                bytes: Vec::new(),
                cancel: &cancel,
                events: &sink,
            },
            || Ok(false),
            |_, _| {},
        )
        .expect("calculation report");
        let prepared = Prepared::new(plan.clone(), Arc::new(report), &cancel).expect("prepared");
        sink.latest_frame
            .tripeaks_route
            .lock()
            .expect("route store")
            .install(Some(prepared))
            .expect("install route");
        (sink, rx, plan, cancel)
    }

    /// Execute one reserved playback action and return the emitted completion event payload.
    fn execute_one(
        sink: &WorkerEventSink,
        rx: &mpsc::Receiver<super::super::ContextualWorkerEvent>,
        plan: &tripeaks_calculation::Identity,
        action: PlaybackAction,
        cancel: &Arc<AtomicBool>,
    ) -> (RunTermination, Result<Progress, String>, u64) {
        let run_id = sink
            .latest_frame
            .tripeaks_route
            .lock()
            .expect("lock store")
            .reserve(plan, action)
            .expect("reserve action");
        reserve_run_scope(sink, plan, cancel);
        run(plan.clone(), run_id, action, fast_timings(), sink, cancel);
        drain_route_finished(rx, run_id)
    }

    /// Arm context ownership and capture context exactly as `WorkerHandle` does before dispatch.
    fn reserve_run_scope(
        sink: &WorkerEventSink,
        plan: &tripeaks_calculation::Identity,
        cancel: &Arc<AtomicBool>,
    ) {
        sink.set_capture_context(Some(plan.context.clone()));
        sink.latest_frame
            .reconnaissance
            .select(&plan.context, cancel)
            .expect("select context");
        sink.latest_frame
            .reconnaissance
            .queue_reconnaissance(&plan.context, cancel, || Ok(()))
            .expect("queue recon scope");
    }

    /// Drain events until the matching `TriPeaksRouteFinished` for `run_id` arrives.
    fn drain_route_finished(
        rx: &mpsc::Receiver<super::super::ContextualWorkerEvent>,
        run_id: u64,
    ) -> (RunTermination, Result<Progress, String>, u64) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let rem = deadline.saturating_duration_since(Instant::now());
            let item = rx.recv_timeout(rem).expect("receive TriPeaksRouteFinished");
            if let WorkerEvent::TriPeaksRouteFinished {
                run_id: got_id,
                result,
                termination,
                ..
            } = item.event
                && got_id == run_id
            {
                return (termination, result.map_err(|f| f.error), got_id);
            }
        }
    }

    /// Return a unique temporary `AF_UNIX` QMP socket path.
    fn temp_socket_path(tag: &str) -> PathBuf {
        let id = NEXT_ROUTE_TEST_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "tripeaks-route-{tag}-{}-{id}.sock",
            std::process::id()
        ))
    }

    /// Exercise `SINGLE STEP`, `MULTI STEP` + `Stop after step`, `UNDO ALL` to `T=0`, and final clear.
    #[test]
    fn single_step_multi_step_stop_after_step_and_undo_all() {
        let slots = cyclic_solvable_slots();
        let sock = temp_socket_path("step-undo");
        let guest = spawn_loopback_guest(sock.clone(), slots, None);
        let (sink, rx, plan, cancel) = prepare_and_install_candidate(&sock, slots);
        let _cap_cancel =
            super::super::tcp_capture::CancellationScope::install(Arc::clone(&cancel));

        // 1. SINGLE STEP: advances cursor 0 -> 1.
        let (term1, res1, _) = execute_one(&sink, &rx, &plan, PlaybackAction::SingleStep, &cancel);
        assert_eq!(term1, RunTermination::Completed);
        let prog1 = res1.expect("SingleStep ok");
        assert_eq!(prog1.completion, Some(Completion::Step));
        assert_eq!(prog1.cursor, 1);

        // 2. MULTI STEP + Stop after step on the 2nd click: advances 1 -> 2 and pauses cleanly.
        let run2 = sink
            .latest_frame
            .tripeaks_route
            .lock()
            .expect("store")
            .reserve(&plan, PlaybackAction::MultiStep)
            .expect("reserve MultiStep");
        {
            let mut g = guest.state.lock().expect("guest");
            g.stop_after_step_hook = Some((2, Arc::clone(&sink), plan.clone(), run2));
        }
        reserve_run_scope(&sink, &plan, &cancel);
        run(
            plan.clone(),
            run2,
            PlaybackAction::MultiStep,
            fast_timings(),
            &sink,
            &cancel,
        );
        let (term2, res2, _) = drain_route_finished(&rx, run2);
        assert_eq!(term2, RunTermination::Completed);
        let prog2 = res2.expect("MultiStep + StopAfterStep ok");
        assert_eq!(prog2.completion, Some(Completion::Step));
        assert_eq!(prog2.cursor, 2);
        assert!(
            sink.latest_frame
                .tripeaks_route
                .lock()
                .expect("store")
                .available(&plan)
        );

        // 3. UNDO ALL: returns guest and cursor cleanly back to T = 0.
        let (term3, res3, _) = execute_one(&sink, &rx, &plan, PlaybackAction::UndoAll, &cancel);
        assert_eq!(term3, RunTermination::Completed);
        let prog3 = res3.expect("UndoAll ok");
        assert_eq!(prog3.completion, Some(Completion::RestoredToRoot));
        assert_eq!(prog3.cursor, 0);
        assert!(guest.state.lock().expect("guest").history.is_empty());

        // 4. Full MULTI STEP from T = 0 clears all 28 cards cleanly.
        let (term4, res4, _) = execute_one(&sink, &rx, &plan, PlaybackAction::MultiStep, &cancel);
        assert_eq!(term4, RunTermination::Completed);
        let prog4 = res4.expect("Full MultiStep ok");
        assert_eq!(prog4.completion, Some(Completion::BoardCleared));
        assert_eq!(prog4.cursor, 28);
        assert_eq!(prog4.cards_cleared, 28);
    }

    /// Pre-execution mismatch on the guest refuses before any QMP input is sent.
    #[test]
    fn refuses_mismatched_guest_board_before_any_qmp_input() {
        let slots = cyclic_solvable_slots();
        let sock = temp_socket_path("mismatch");
        let wrong_card = match (slots[18], slots[19]) {
            (CardSlot::Known(a), CardSlot::Known(b)) if a != b => b,
            _ => Card(51),
        };
        let guest = spawn_loopback_guest(sock.clone(), slots, Some(wrong_card));
        let (sink, rx, plan, cancel) = prepare_and_install_candidate(&sock, slots);
        let _cap_cancel =
            super::super::tcp_capture::CancellationScope::install(Arc::clone(&cancel));

        let (term, res, _) = execute_one(&sink, &rx, &plan, PlaybackAction::SingleStep, &cancel);
        assert_eq!(term, RunTermination::Failed);
        let err = res.expect_err("mismatched root must fail");
        assert!(
            err.contains("mismatch") || err.contains("match"),
            "err={err}"
        );
        assert_eq!(guest.state.lock().expect("guest").qmp_inputs_delivered, 0);
    }

    /// Pre-asserted `STOP` aborts before any QMP input with `RunTermination::OperatorStopped`.
    #[test]
    fn stop_fence_prevents_input_and_preserves_operator_stopped_termination() {
        let slots = cyclic_solvable_slots();
        let sock = temp_socket_path("stop");
        let guest = spawn_loopback_guest(sock.clone(), slots, None);
        let (sink, rx, plan, cancel) = prepare_and_install_candidate(&sock, slots);
        let _cap_cancel =
            super::super::tcp_capture::CancellationScope::install(Arc::clone(&cancel));

        let run_id = sink
            .latest_frame
            .tripeaks_route
            .lock()
            .expect("store")
            .reserve(&plan, PlaybackAction::SingleStep)
            .expect("reserve SingleStep");
        reserve_run_scope(&sink, &plan, &cancel);
        sink.latest_frame
            .reconnaissance
            .operator_stop(&plan.context, &cancel);
        run(
            plan.clone(),
            run_id,
            PlaybackAction::SingleStep,
            fast_timings(),
            &sink,
            &cancel,
        );
        let (term, res, _) = drain_route_finished(&rx, run_id);
        assert_eq!(term, RunTermination::OperatorStopped);
        assert!(res.is_err());
        assert_eq!(guest.state.lock().expect("guest").qmp_inputs_delivered, 0);
        assert!(
            !sink
                .latest_frame
                .tripeaks_route
                .lock()
                .expect("store")
                .available(&plan)
        );
    }
}
