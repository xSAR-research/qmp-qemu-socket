//! Same-session Pyramid calculation and finite observed execution.
//!
//! xSAR owns search, rules, independent replay and sequencing. This adapter owns
//! calibration, fresh native evidence, input guards, exclusive STOP and logs.
//! A permit is consumed on execution; failed or uncertain runs cannot resume.

#![cfg_attr(
    not(test),
    deny(
        clippy::indexing_slicing,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic
    )
)]

use crate::parameters::{
    PYRAMID_EXECUTION_DEADLINE as EXECUTION_DEADLINE,
    PYRAMID_FINAL_CONFIRMATION_WINDOW as FINAL_CONFIRMATION_WINDOW,
    PYRAMID_FINAL_OBSERVATION_DELAY as FINAL_OBSERVATION_DELAY,
};

use super::{
    QmpClient, WorkerEvent, WorkerEventSink, WorkerState, cancellable_wait, capture_screen,
    publish_pyramid_failure, send_log, send_state, send_status, validate_probe,
};
use crate::{
    capture::CapturedFrame,
    game::{GameMode, InputOperation},
    pyramid::{self, PyramidTargetKind},
    pyramid_diagnostics::{FailureEvidence, PyramidPhase},
    pyramid_progress::{ChallengeCompletion, ChallengeProgress},
    pyramid_recon::vision::{BoardObservation, ReconScene, ReconVision, StockFace},
    pyramid_search_policy,
    pyramid_workflow_model::{PlanningProgress, RouteGoal, RouteMethod},
    strategy::{ControllerContext, SolvingStrategy},
};
use std::{
    fs::File,
    io::Read,
    path::Path,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};


mod cadence;
mod diagnostics;
mod partial;


mod continuation;


mod redeal_appearance;
mod search_log;


use partial::confirm_partial_action;
use thiserror::Error;
use xsar::{
    games::pyramid::{
        CompleteDeal, TABLEAU_LEN,
        planning::{Location, RecordedAction, ReplayStep, SearchState, TableauLocation},
        replay, rules, search,
    },
    search::{BestEffortOutcome, PartialProof, SearchMethod, SearchOptions},
};


#[cfg(test)]
use xsar::search::SearchOutcome;


/// At most 28 removal actions, 72 draws and two recycles under this rule profile.
const MAX_ROUTE_ACTIONS: usize = 102;


/// At most 52 removal clicks, 72 draws and two recycles.
const MAX_ROUTE_GESTURES: usize = 126;


/// Fresh observations allowed for a source precondition; effects have their own window.
const ORDINARY_OBSERVATIONS: usize = 6;


/// Fresh observation-only final transition allowance.
const FINAL_OBSERVATIONS: usize = 20;


/// Derive a finite effect cap that cannot shorten the configured observation window.
///
/// The ceiling of window/retry plus one covers an immediate read and every retry,
/// even with zero capture latency. Reject zero intervals/windows and overflow.
fn ordinary_effect_observation_limit() -> Result<usize, String> {
    let window = crate::parameters::COMPUTED_ROUTE_EFFECT_WINDOW.as_nanos();
    let interval = crate::parameters::COMPUTED_ROUTE_REOBSERVE_DELAY.as_nanos();
    window
        .checked_sub(1)
        .and_then(|n| n.checked_div(interval))
        .and_then(|n| n.checked_add(2))
        .and_then(|n| usize::try_from(n).ok())
        .ok_or("Invalid ordinary effect observation budget".to_owned())
}


/// Return time left for a new ordinary-effect capture or read-only retry.
///
/// Both times are elapsed from the same run origin. Expiry or a backwards clock
/// withholds further observation; it never permits another input attempt.
fn ordinary_effect_remaining(acknowledged_at: Duration, now: Duration) -> Result<Duration, String> {
    let elapsed = now
        .checked_sub(acknowledged_at)
        .ok_or("Ordinary effect clock precedes its acknowledgement")?;
    crate::parameters::COMPUTED_ROUTE_EFFECT_WINDOW
        .checked_sub(elapsed)
        .filter(|remaining| !remaining.is_zero())
        .ok_or("Ordinary effect observation window expired; input is not repeated".to_owned())
}


/// Bound all phases, including native guard captures outside the shared sequencer.
///
/// Preserve the approved conservative allowance and cover the exact worst case:
/// two external captures, G guards, 6*A preconditions, the derived ordinary-effect
/// cap for each of A-1 actions, and 20 final effects. Arithmetic is checked.
fn capture_allowance(actions: usize, gestures: usize) -> Result<usize, String> {
    if actions == 0 || actions > MAX_ROUTE_ACTIONS || gestures > MAX_ROUTE_GESTURES {
        return Err("Route count outside finite Pyramid bounds".to_owned());
    }
    let approved = gestures
        .checked_add(actions)
        .and_then(|n| n.checked_mul(ORDINARY_OBSERVATIONS))
        .and_then(|n| n.checked_add(2 + FINAL_OBSERVATIONS));
    let effect_limit = ordinary_effect_observation_limit()?;
    let effects = actions
        .checked_sub(1)
        .and_then(|n| n.checked_mul(effect_limit));
    let phases = actions
        .checked_mul(ORDINARY_OBSERVATIONS)
        .zip(effects)
        .and_then(|(preconditions, effects)| preconditions.checked_add(effects))
        .and_then(|n| n.checked_add(gestures))
        .and_then(|n| n.checked_add(2 + FINAL_OBSERVATIONS));
    approved
        .zip(phases)
        .map(|(a, b)| a.max(b))
        .ok_or("Route capture allowance overflow".to_owned())
}

/// Route failures retain the exact safety boundary without retry authority.
#[derive(Debug, Error)]
enum RouteError {
    /// A failed precondition or observation requires operator inspection.
    #[error("{0}")]
    Refused(String),
}


/// Controller/calibration/original-file receipt minted only by successful reconnaissance.
struct Acquisition {
    /// Selection identity, including changed-and-back generation.
    context: ControllerContext,
    /// Immediate authority epoch, also revoked by STOP while idle.
    epoch: u64,
    /// Sole owner of original validated card identities.
    deal: CompleteDeal,
    /// Exact successfully published document, allowing moves without reauthentication.
    bytes: Vec<u8>,
    /// Immutable calibration actually used by the successful scan.
    vision: ReconVision,
    /// Actual requirement from the final restored acquisition frame, never the latest positive.
    challenge_progress: Option<ChallengeProgress>,
    /// Advisory method from the same restored native frame; never an input permit.
    default_method: RouteMethod,
    /// Advisory actual header from the same restored frame; never input authority.
    observed_game: crate::pyramid_workflow_model::ObservedGame,
}


/// Independently replayed immutable execution candidate.
struct Prepared {
    /// Calculation ID binds the UI button to this exact route.
    id: u64,
    /// Single-use origin permit; no public constructor or Clone.
    acquisition: Acquisition,
    /// Each logical action and independent expected identities/pile tops.
    trace: Box<[ReplayStep]>,
    /// Full clearance or an explicitly partial, scored independently replayed endpoint.
    objective: RouteObjective,
}


/// Plan intent retained separately from transport delivery and actual visual effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RouteObjective {
    /// The independent replay must finish with every tableau slot empty.
    Clear,
    /// Exhaustive no-win leaves the original board for a separately guarded stock drain.
    Drain,
    /// Remove this many tableau cards, then stop without series continuation authority.
    Removal {
        /// Positive number of original tableau identities removed, strictly below 28.
        removed: u8,
        /// Exhaustive maximum proof or an explicit best-found resource cutoff.
        proof: PartialProof,
    },
}


/// Private worker-owned receipt/plan storage; UI sees only ordered results.
#[derive(Default)]
pub(super) struct Store {
    /// Unconsumed successfully restored origin.
    acquisition: Option<Acquisition>,
    /// Calculated plan, consumed before connecting for execution.
    prepared: Option<Prepared>,
}


impl Store {
    /// Discard the obsolete plan while retaining its sole unconsumed acquisition.
    ///
    /// This transfers ownership without cloning or minting authority. Subsequent
    /// calculation still checks controller identity, STOP epoch and exact bytes.
    pub(super) fn discard_prepared(&mut self) {
        if let Some(plan) = self.prepared.take()
            && self.acquisition.is_none()
        {
            self.acquisition = Some(plan.acquisition);
        }
    }
}


/// Positive final effect, or complete delivery whose effect remained uncertain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BoardCompletion {
    /// Final goal-route ACK schedules acquisition; only fresh Recon can establish success.
    AwaitingFreshBoard,
    /// One positively enabled REDEAL was acknowledged; fresh starting-board proof is pending.
    StockRedealt,
    /// The retained actual requirement is bound to a positive completion event.
    ChallengeTargetVerified(ChallengeCompletion),
    /// Exact whole-action effect and actual target count; terminal completion is still pending.
    ChallengeTargetReached(ChallengeProgress),
    /// The exact replayed empty-tableau position was independently recognised.
    BoardClear,
    /// Two consecutive complete, equal new-board observations were recognised.
    NextBoardReady,
    /// Two consecutive specific Pyramid terminal scenes were recognised.
    GameComplete,
    /// Exact non-clear endpoint recognised; automatic play must stop here.
    PartialPositionVerified {
        /// Independently replayed and visually verified tableau removals.
        removed: u8,
        /// Original tableau slots still occupied at the verified endpoint.
        remaining: u8,
    },
    /// All gestures were acknowledged, but the bounded final proof was absent.
    DeliveredUnconfirmed,
}


/// Consumed-plan result for an owner that retains the current worker reservation.
#[derive(Debug)]
pub(super) struct ExecutionReport {
    /// Highest actual challenge counter observed while this consumed plan ran.
    pub(super) challenge_progress: Option<ChallengeProgress>,
    /// Actual delivery/effect result suitable for a session or manual status.
    pub(super) message: String,
    /// Typed final proof; uncertain delivery never grants continuation authority.
    pub(super) completion: BoardCompletion,
    /// Last existing observation, without any additional recording capture.
    pub(super) latest_frame: Option<CapturedFrame>,
    /// Exact calibration from the consumed acquisition, available for bounded restart.
    pub(super) vision: ReconVision,
}


/// Internal delivery result before the consumed calibration is moved to its owner.
#[derive(Debug)]
struct DriveReport {
    /// Bounded final-observation conclusion and actual transport accounting.
    message: String,
    /// Positive proof or acknowledged delivery without a confirmed final effect.
    completion: BoardCompletion,
}


/// Consecutive fresh-frame proof, restricted to the final delivered route action.
#[derive(Default)]
struct FinalConfirmation {
    /// The previous complete board; any other scene resets this candidate.
    board: Option<BoardObservation>,
    /// One preceding positively recognised Pyramid terminal frame.
    terminal: bool,
}


impl FinalConfirmation {
    /// Classify one already captured frame; unknown pixels reset consecutive proof.
    ///
    /// A matching replayed position takes precedence. A new board must contain all
    /// 28 recognised unique tableau cards, a recognised LEFT card and positive
    /// empty RIGHT evidence. Header progress and Solver artwork are not consulted.
    fn observe(
        &mut self,
        vision: &ReconVision,
        frame: &CapturedFrame,
        expected_matches: bool,
    ) -> Result<Option<BoardCompletion>, String> {
        if expected_matches {
            *self = Self::default();
            return Ok(Some(BoardCompletion::BoardClear));
        }
        if super::post_game::recognised_pyramid_terminal(frame)? {
            self.board = None;
            let confirmed = self.terminal;
            self.terminal = true;
            return Ok(confirmed.then_some(BoardCompletion::GameComplete));
        }
        self.terminal = false;
        if let Ok(ReconScene::Board(board)) = vision.observe(frame)
            && matches!(board.left, StockFace::Card(_))
            && board.right == StockFace::Empty
        {
            let confirmed = self.board.as_ref() == Some(&board);
            self.board = Some(board);
            Ok(confirmed.then_some(BoardCompletion::NextBoardReady))
        } else {
            self.board = None;
            Ok(None)
        }
    }
}


/// Enforce separate Pyramid strategy authority without enabling generic ExecuteSteps.
pub(super) fn validate_context(context: &ControllerContext) -> Result<(), String> {
    context.validate()?;
    if context.game_mode != GameMode::Pyramid || context.strategy != SolvingStrategy::ShortestPath {
        return Err("Computed routes require Pyramid and Shortest path".to_owned());
    }
    Ok(())
}


/// Read the bounded exact published document; metadata never authenticates the guest.
fn read_document(path: &Path) -> Result<Vec<u8>, String> {
    // Reject named pipes before an open could wait indefinitely for a writer.
    if !std::fs::metadata(path)
        .map_err(|e| format!("Cannot inspect route deal: {e}"))?
        .is_file()
    {
        return Err("Route deal must be a regular file".to_owned());
    }
    let file = File::open(path).map_err(|e| format!("Cannot open route deal: {e}"))?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Route deal must be a regular file".to_owned());
    }
    let mut bytes = Vec::new();
    file.take(65_537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65_536 {
        return Err("Route deal exceeds 65,536 bytes".to_owned());
    }
    xsar::pyramid_json::read_from(bytes.as_slice()).map_err(|e| e.to_string())?;
    Ok(bytes)
}


/// Mint the private receipt only after complete restoration/publication/diagnostics.
#[expect(
    clippy::too_many_arguments,
    reason = "One private receipt binds restored identities, header and advisory default"
)]
pub(super) fn record_acquisition(
    context: &ControllerContext,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
    deal: CompleteDeal,
    vision: ReconVision,
    output: &Path,
    challenge_progress: Option<ChallengeProgress>,
    default_method: RouteMethod,
    observed_game: crate::pyramid_workflow_model::ObservedGame,
) -> Result<(), String> {
    let control = &events.latest_frame.reconnaissance;
    control.check_active(context, cancel)?;
    let bytes = read_document(output)?;
    let published = xsar::pyramid_json::read_from(bytes.as_slice()).map_err(|e| e.to_string())?;
    if published != deal {
        return Err("Published document differs from the acquired deal".to_owned());
    }
    let epoch = control.epoch()?;
    control.check_active(context, cancel)?;
    let mut store = events
        .latest_frame
        .computed_route
        .lock()
        .map_err(|_| "Route receipt lock poisoned")?;
    *store = Store {
        acquisition: Some(Acquisition {
            context: context.clone(),
            epoch,
            deal,
            bytes,
            vision,
            challenge_progress,
            default_method,
            observed_game,
        }),
        prepared: None,
    };
    send_log(
        events,
        format!(
            "Restored acquisition detected: {}. Header facts are separate from the selected stop condition.",
            observed_game.label()
        ),
    );
    send_log(
        events,
        "Same-session restored acquisition receipt retained for input-free route calculation."
            .to_owned(),
    );
    Ok(())
}


/// Authenticate retained Recon for a first automatic phase without cloning its permit.
pub(super) fn reuse_metadata(
    context: &ControllerContext,
    path: &Path,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> Result<(ReconVision, Option<ChallengeProgress>), String> {
    let mut store = events
        .latest_frame
        .computed_route
        .lock()
        .map_err(|_| "Route receipt lock poisoned")?;
    store.discard_prepared();
    let receipt = store
        .acquisition
        .as_ref()
        .ok_or("No same-session restored Recon remains available")?;
    check_acquisition(receipt, context, events, cancel)?;
    if read_document(path)? != receipt.bytes {
        return Err("Selected JSON differs from the successful same-session export".to_owned());
    }
    Ok((receipt.vision.clone(), receipt.challenge_progress))
}


/// Return a default only after exact same-session/context/export authentication.
pub(super) fn acquisition_default(
    context: &ControllerContext,
    path: &Path,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> Result<RouteMethod, String> {
    let mut store = events
        .latest_frame
        .computed_route
        .lock()
        .map_err(|_| "Route receipt lock poisoned")?;
    store.discard_prepared();
    let receipt = store
        .acquisition
        .as_ref()
        .ok_or("No same-session restored Recon remains available")?;
    check_acquisition(receipt, context, events, cancel)?;
    if read_document(path)? != receipt.bytes {
        return Err("Selected JSON differs from the successful same-session export".to_owned());
    }
    Ok(receipt.default_method)
}


/// Read advisory facts only after authenticating the current context, epoch and exact export.
/// This borrows the existing acquisition or plan and neither consumes nor replaces its permit.
pub(super) fn acquisition_observed(
    context: &ControllerContext,
    path: &Path,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> Result<crate::pyramid_workflow_model::ObservedGame, String> {
    let store = events
        .latest_frame
        .computed_route
        .lock()
        .map_err(|_| "Route receipt lock poisoned")?;
    let receipt = store
        .acquisition
        .as_ref()
        .or_else(|| store.prepared.as_ref().map(|plan| &plan.acquisition))
        .ok_or("No same-session restored Recon remains available")?;
    check_acquisition(receipt, context, events, cancel)?;
    if read_document(path)? != receipt.bytes {
        return Err("Selected JSON differs from the successful same-session export".to_owned());
    }
    Ok(receipt.observed_game)
}


/// Check current monotonic authority, including context changes while idle.
fn check_acquisition(
    acquisition: &Acquisition,
    context: &ControllerContext,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> Result<(), String> {
    let control = &events.latest_frame.reconnaissance;
    control.check_active(context, cancel)?;
    if acquisition.context != *context || acquisition.epoch != control.epoch()? {
        return Err(
            "Acquisition receipt is stale; prepare and acquire a restored deal again".to_owned(),
        );
    }
    Ok(())
}


/// Release run ownership only after every route operation has unwound.
fn finish(
    events: &WorkerEventSink,
    run_id: u64,
    plan_id: Option<u64>,
    result: Result<String, String>,
    retained_recon: bool,
) {
    events.latest_frame.reconnaissance.finish();
    send_state(
        events,
        if result.is_ok() {
            WorkerState::Ready
        } else {
            WorkerState::Uncertain
        },
    );
    let message = result
        .as_ref()
        .map_or_else(|e| format!("Computed route stopped: {e}"), Clone::clone);
    send_status(events, message.clone());
    send_log(events, message);
    let _ = events.send(WorkerEvent::ComputedRouteFinished {
        retained_recon,
        run_id,
        plan_id,
        result,
    });
}


/// Preserve the exact manual test/reference entry point.
#[cfg(test)]
pub(super) fn calculate(
    context: &ControllerContext,
    run_id: u64,
    path: &Path,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) {
    calculate_with_method(
        context,
        run_id,
        path,
        RouteMethod::FewestActions,
        events,
        cancel,
    );
}


/// Calculate one explicit manual request and release its reservation afterwards.
pub(super) fn calculate_with_method(
    context: &ControllerContext,
    run_id: u64,
    path: &Path,
    method: RouteMethod,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) {
    let result =
        calculate_owned_with_policy(context, run_id, path, method, 0, events, cancel, false);
    let retained_recon = result.as_ref().is_ok_and(|report| !report.card_route);
    if retained_recon && let Ok(mut store) = events.latest_frame.computed_route.lock() {
        store.discard_prepared();
    }
    let plan_id = result
        .as_ref()
        .ok()
        .filter(|report| report.card_route)
        .map(|_| run_id);
    finish(
        events,
        run_id,
        plan_id,
        result.map(|report| report.message),
        retained_recon,
    );
}


/// Distinguish exhaustive no-win proof from an interruption with unknown solvability.
fn no_winning_route_message(proof: PartialProof) -> String {
    match proof {
        PartialProof::Exhaustive => "Search exhausted all legal positions: no winning route exists under the supported rules. Shortest route does not authorise a partial route.".to_owned(),
        PartialProof::Bounded(reason) => format!("Search interrupted by {reason}; whether a winning route exists is unproved. No executable route was prepared."),
    }
}


/// Calculate and independently replay within an already reserved worker session.
///
/// This operation performs no guest I/O and never releases or clears STOP.
#[cfg(test)]
pub(super) fn calculate_owned(
    context: &ControllerContext,
    run_id: u64,
    path: &Path,
    method: RouteMethod,
    board_generation: u64,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> Result<String, String> {
    calculate_owned_with_policy(
        context,
        run_id,
        path,
        method,
        board_generation,
        events,
        cancel,
        false,
    )
    .map(|report| report.message)
}


/// Current-board planning result; the effective method is advisory, never a permit.
pub(super) struct PlanningReport {
    /// A nonempty replay can be dispatched by explicit Manual Solve.
    pub(super) card_route: bool,
    /// Exact outcome for the operator, including automatic fallback or Boards bypass.
    pub(super) message: String,
    /// Method actually used by the retained execution candidate.
    pub(super) method: RouteMethod,
}


/// Search and fall back while retaining one authenticated acquisition throughout.
///
/// Complete exhaustion alone selects automatic fallback or Boards stock draining.
/// An explicit operator request may replace an active Shortest engine with Maximum;
/// it proves nothing about solvability and retains this same authenticated receipt.
/// The explicit no-challenge policy suppresses header interpretation; no caller
/// can clone the acquisition or recover it after execution starts.
#[expect(
    clippy::too_many_arguments,
    reason = "Worker identity, ownership and explicit target policy remain separate"
)]
pub(super) fn calculate_owned_with_policy(
    context: &ControllerContext,
    run_id: u64,
    path: &Path,
    method: RouteMethod,
    board_generation: u64,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
    ignore_challenge: bool,
) -> Result<PlanningReport, String> {
    send_state(events, WorkerState::Validating);
    let limits = pyramid_search_policy::limits();
    // The two UI outcomes have a fixed method mapping, shared by manual/session paths.
    let mut method = method;
    let mut source_document = None;
    let mut details = serde_json::json!({
        "input_path_debug": format!("{path:?}"),
        "search_method": method.label(),
        "route_objective": method.route_goal().label(),
        "heuristic_weight": if method == RouteMethod::Fast { 2 } else { 1 },
        "board_generation": board_generation,
        "rule_profile": "two-recycle Pyramid; cost selected by route objective",
        "guest_input_sent": false,
        "limits": pyramid_search_policy::details(limits),
    });
    let result = (|| {
        validate_context(context)?;
        let mut store = events
            .latest_frame
            .computed_route
            .lock()
            .map_err(|_| "Route receipt lock poisoned")?;
        let acquisition = store
            .acquisition
            .take()
            .or_else(|| store.prepared.take().map(|plan| plan.acquisition))
            .ok_or("No same-session restored acquisition; acquire the board first")?;
        // Keep the receipt privately owned while search or its checkpoint can block.
        drop(store);
        check_acquisition(&acquisition, context, events, cancel)?;
        source_document = Some(acquisition.bytes.clone());
        record_detail(
            &mut details,
            "profile_id",
            serde_json::json!(acquisition.vision.profile_id()),
        );
        record_detail(
            &mut details,
            "observed_at_acquisition",
            serde_json::json!(acquisition.observed_game.label()),
        );
        let bytes = read_document(path)?;
        if bytes != acquisition.bytes {
            return Err("Selected JSON differs from the successful same-session export".to_owned());
        }
        let requirement = (!ignore_challenge)
            .then_some(acquisition.challenge_progress)
            .flatten();
        if requirement.is_some_and(ChallengeProgress::target_reached) {
            return Err(
                "Challenge target is already reached; no further route input authorised".to_owned(),
            );
        }
        let (actions, stats, objective, guarantee) = loop {
            check_acquisition(&acquisition, context, events, cancel)?;
            let search_work = events.latest_frame.search_control.begin(
                context,
                run_id,
                board_generation,
                method,
                cancel,
            )?;
            let local_cancel = AtomicBool::new(false);
            let route_goal = method.route_goal();
            send_log(
                events,
                format!(
                    "A* calculation owner: run={run_id}; board generation={board_generation}; source export={path:?}"
                ),
            );
            send_log(
                events,
                search_log::parameters(
                    method,
                    acquisition.observed_game,
                    acquisition.vision.profile_id(),
                    ignore_challenge,
                    limits,
                ),
            );
            let (cost, cost_metric, cost_unit) = match route_goal {
                RouteGoal::ShortestRoute => (
                    search::PyramidCost::CardMatches,
                    "MATCH and KING cost one; DRAW and RECYCLE cost zero",
                    "card-match",
                ),
                RouteGoal::MaximumCardMatches => (
                    search::PyramidCost::LogicalActions,
                    "each logical action costs one; fallback score counts tableau removals only",
                    "logical-action",
                ),
            };
            record_detail(&mut details, "cost_metric", serde_json::json!(cost_metric));
            record_detail(
                &mut details,
                "search_method",
                serde_json::json!(method.label()),
            );
            record_detail(
                &mut details,
                "route_objective",
                serde_json::json!(route_goal.label()),
            );
            record_detail(
                &mut details,
                "heuristic_weight",
                serde_json::json!(if method == RouteMethod::Fast { 2 } else { 1 }),
            );
            send_status(
                events,
                format!(
                    "Searching for {} — no automatic cutoff; use STOP to cancel…",
                    route_goal.label()
                ),
            );
            let options = SearchOptions {
                method: SearchMethod::AStar,
                heuristic_weight: std::num::NonZeroU32::new(if method == RouteMethod::Fast {
                    2
                } else {
                    1
                })
                .ok_or("Invalid internal search weight")?,
            };
            let mut last_logged = Duration::ZERO;
            let mut progress = |snapshot: xsar::search::SearchProgress| {
                // At most one intermediate row per second plus the engine's final row.
                if snapshot.finished
                    || snapshot.stats.elapsed.saturating_sub(last_logged)
                        >= crate::parameters::SEARCH_LOG_INTERVAL
                {
                    send_log(events, search_log::statistics(method, snapshot));
                    last_logged = snapshot.stats.elapsed;
                }
                // Advisory snapshots replace one bounded slot; they grant no authority.
                let mut slot = events
                    .latest_frame
                    .planning_progress
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                *slot = Some(PlanningProgress {
                    context: context.clone(),
                    run_id,
                    board_generation,
                    method,
                    observed_game: acquisition.observed_game,
                    stats: snapshot.stats,
                    finished: snapshot.finished,
                });
            };
            let checkpoint = || match events.latest_frame.pause.checkpoint_interruptible(
                super::pyramid_pause::Phase::Search,
                || check_acquisition(&acquisition, context, events, cancel),
                || search_work.maximum_requested(),
            ) {
                Ok(super::pyramid_pause::Checkpoint::Ready(paused)) => paused,
                Ok(super::pyramid_pause::Checkpoint::Interrupted(paused)) => {
                    local_cancel.store(true, std::sync::atomic::Ordering::Release);
                    paused
                }
                Err(error) => {
                    // Real cancellation/context failures retain existing STOP semantics.
                    cancel.store(true, std::sync::atomic::Ordering::Release);
                    local_cancel.store(true, std::sync::atomic::Ordering::Release);
                    error.paused
                }
            };
            let outcome = search::solve_best_effort_with_cost_and_control(
                &acquisition.deal,
                limits,
                &local_cancel,
                options,
                cost,
                Some(&mut progress),
                Some(&checkpoint),
            );
            checkpoint();
            check_acquisition(&acquisition, context, events, cancel)?;
            // This mutex handshake closes acceptance before any old actions reach replay.
            let replace = search_work.finish()?;
            check_acquisition(&acquisition, context, events, cancel)?;
            if replace {
                method = RouteMethod::Fast;
                record_detail(
                    &mut details,
                    "operator_requested_maximum",
                    serde_json::json!(true),
                );
                send_log(events, "Operator requested Maximum card matches: retiring Shortest search and reusing this authenticated RECON; no no-win proof claimed.".to_owned());
                continue;
            }
            let prepared = match outcome {
                BestEffortOutcome::Solved {
                    actions,
                    stats,
                    minimum_proven,
                } => (
                    actions,
                    stats,
                    RouteObjective::Clear,
                    if minimum_proven && route_goal == RouteGoal::ShortestRoute {
                        "fewest card matches proved (MATCH and single KING each count one; DRAW/RECYCLE count separately)".to_owned()
                    } else if minimum_proven {
                        "fewest logical actions proved; no minimum card-match claim".to_owned()
                    } else {
                        format!("winning route verified; minimum {cost_unit} count not established")
                    },
                ),
                BestEffortOutcome::Partial {
                    actions,
                    score,
                    proof,
                    stats,
                } => {
                    send_log(
                        events,
                        format!(
                            "A* no full-clear route: method={}; removal score={score}/28; proof={proof:?}. Only exhaustive absence permits automatic fallback or stock continuation.",
                            method.label()
                        ),
                    );
                    if requirement.is_some_and(|bound| {
                        bound.kind == crate::pyramid_progress::ChallengeKind::Boards
                    }) {
                        if proof != PartialProof::Exhaustive {
                            return Err(no_winning_route_message(proof));
                        }
                        send_status(events, "Boards challenge has no full-clear route; partial matching bypassed. Preparing remaining stock passes.".to_owned());
                        break (
                            Vec::new(),
                            stats,
                            RouteObjective::Drain,
                            "no winning full-clear route; Boards partial matching bypassed"
                                .to_owned(),
                        );
                    }
                    record_detail(&mut details, "fallback_score", serde_json::json!(score));
                    record_detail(
                        &mut details,
                        "fallback_proof",
                        serde_json::json!(format!("{proof:?}")),
                    );
                    if route_goal == RouteGoal::ShortestRoute {
                        if proof != PartialProof::Exhaustive {
                            record_detail(
                                &mut details,
                                "search_stats",
                                search_stats_details(stats),
                            );
                            return Err(no_winning_route_message(proof));
                        }
                        record_detail(
                            &mut details,
                            "shortest_no_win_stats",
                            search_stats_details(stats),
                        );
                        match continuation::no_win_action(requirement.is_some(), requirement)? {
                            continuation::NoWinAction::DrainStock => {
                                send_status(events, "Boards challenge has no winning route; skipping Maximum and partial matching. Stock continuation prepared.".to_owned());
                                break (
                                    Vec::new(),
                                    stats,
                                    RouteObjective::Drain,
                                    "full-clear absence proved; Boards partial matching bypassed"
                                        .to_owned(),
                                );
                            }
                            continuation::NoWinAction::MaximumMatches => {
                                check_acquisition(&acquisition, context, events, cancel)?;
                                send_status(events, "No winning Shortest route. Switching to Maximum card matches using existing Recon for this board only.".to_owned());
                                send_log(events, "Automatic current-board fallback: Shortest route → Maximum card matches; same Recon receipt retained.".to_owned());
                                method = RouteMethod::Fast;
                                continue;
                            }
                        }
                    }
                    if actions.is_empty() || score == 0 || score >= TABLEAU_LEN as u32 {
                        if proof == PartialProof::Exhaustive && score == 0 && actions.is_empty() {
                            break (Vec::new(), stats, RouteObjective::Drain,
                            "no winning route or beneficial tableau removal; stock continuation prepared".to_owned());
                        }
                        record_detail(&mut details, "search_stats", search_stats_details(stats));
                        return Err(format!(
                            "No beneficial tableau-removal route found; fallback={proof:?}; no execution plan"
                        ));
                    }
                    let removed = u8::try_from(score)
                        .map_err(|_| "Fallback score is outside tableau bounds")?;
                    let guarantee = match proof {
                        PartialProof::Exhaustive => format!(
                            "maximum removal proved: {removed}/28 tableau cards; fewest logical actions at that removal count; continuation requires the exact partial position"
                        ),
                        PartialProof::Bounded(reason) => format!(
                            "best removal found: {removed}/28 tableau cards; search stopped at {reason}; maximum and minimum logical-action count unproved; automatic play stops at the verified partial position"
                        ),
                    };
                    (
                        actions,
                        stats,
                        RouteObjective::Removal { removed, proof },
                        guarantee,
                    )
                }
                BestEffortOutcome::Stopped { reason, stats } => {
                    send_log(
                        events,
                        format!(
                            "A* interrupted: method={}; reason={reason:?}; expanded={}; generated={}; histories={}; elapsed={:.3}s; no executable route released.",
                            method.label(),
                            stats.expanded,
                            stats.generated,
                            stats.states,
                            stats.elapsed.as_secs_f64()
                        ),
                    );
                    record_detail(&mut details, "search_stats", search_stats_details(stats));
                    record_detail(
                        &mut details,
                        "search_stop",
                        serde_json::json!(format!("{reason:?}")),
                    );
                    record_detail(&mut details, "search_complete", serde_json::json!(false));
                    return Err(format!("Search interrupted: {reason:?}; no execution plan"));
                }
            };
            break prepared;
        };
        record_detail(&mut details, "search_stats", search_stats_details(stats));
        record_detail(
            &mut details,
            "candidate_actions_debug",
            serde_json::json!(format!("{actions:?}")),
        );
        let trace = if objective == RouteObjective::Drain {
            Vec::new().into_boxed_slice()
        } else {
            replay::replay(
                &acquisition.deal,
                &actions,
                objective == RouteObjective::Clear,
            )
            .map_err(|e| e.to_string())?
        };
        if let RouteObjective::Removal { removed, .. } = objective {
            let remaining = trace
                .last()
                .ok_or("Empty fallback replay")?
                .remaining_tableau;
            if remaining == 0 || usize::from(removed) + usize::from(remaining) != TABLEAU_LEN {
                return Err(
                    "Independent fallback replay differs from its tableau-removal score".to_owned(),
                );
            }
        }
        check_acquisition(&acquisition, context, events, cancel)?;
        let count = trace.len();
        let card_matches = trace
            .iter()
            .filter(|step| {
                matches!(
                    step.action,
                    RecordedAction::Match(..) | RecordedAction::King(..)
                )
            })
            .count();
        let draws = trace
            .iter()
            .filter(|step| step.action == RecordedAction::Move)
            .count();
        let recycles = trace
            .iter()
            .filter(|step| step.action == RecordedAction::Recycle)
            .count();
        let profile = acquisition.vision.profile_id().to_owned();
        let mut store = events
            .latest_frame
            .computed_route
            .lock()
            .map_err(|_| "Route receipt lock poisoned")?;
        check_acquisition(&acquisition, context, events, cancel)?;
        if store.acquisition.is_some() || store.prepared.is_some() {
            return Err("Another receipt replaced the search owner; no route published".to_owned());
        }
        store.prepared = Some(Prepared {
            id: run_id,
            acquisition,
            trace,
            objective,
        });
        let message = if objective == RouteObjective::Drain {
            format!(
                "No card route prepared; {guarantee}. Existing Recon retained. Select an automatic session for guarded stock continuation. method={}",
                method.label()
            )
        } else {
            format!(
                "Route ready — {count} logical actions, {card_matches} card matches, {draws} draws, {recycles} recycles; {guarantee}; method={}; independent replay passed; profile={profile}; expanded={}, generated={}, histories={}, elapsed={:.3}s. Execution not started.",
                method.label(),
                stats.expanded,
                stats.generated,
                stats.states,
                stats.elapsed.as_secs_f64()
            )
        };
        Ok(PlanningReport {
            message,
            method,
            card_route: objective != RouteObjective::Drain,
        })
    })();
    if let Err(error) = &result {
        publish_pyramid_failure(
            events,
            FailureEvidence {
                context,
                phase: PyramidPhase::Planning,
                run_id: Some(run_id),
                error,
                details,
                source_document: source_document.as_deref(),
                first_frame: None,
                latest_frame: None,
            },
        );
    }
    result
}


/// Preserve search work without confusing an interrupted search with unsolvability.
fn search_stats_details(stats: xsar::search::SearchStats) -> serde_json::Value {
    serde_json::json!({
        "expanded": stats.expanded,
        "generated": stats.generated,
        "retained_histories": stats.states,
        "peak_frontier": stats.peak_frontier,
        "elapsed_seconds": stats.elapsed.as_secs_f64(),
    })
}


/// Add an inspection field only to the locally created metadata object.
fn record_detail(details: &mut serde_json::Value, key: &str, value: serde_json::Value) {
    if let Some(object) = details.as_object_mut() {
        object.insert(key.to_owned(), value);
    }
}


/// Checked mapping from shared semantic location to existing native click geometry.
fn operation(location: Location) -> Result<InputOperation, String> {
    let kind = match location {
        Location::DL => PyramidTargetKind::Left,
        Location::DR => PyramidTargetKind::Right,
        Location::Tableau(slot) => {
            let (row, column) = xsar::games::pyramid::planning::tableau_coordinates(slot.index())
                .ok_or("Invalid route location")?;
            PyramidTargetKind::Card {
                row: u8::try_from(row).map_err(|_| "Invalid route row")?,
                column: u8::try_from(column).map_err(|_| "Invalid route column")?,
            }
        }
    };
    pyramid::action_for_kind(kind)
        .map(|a| a.operation())
        .ok_or("No calibrated route target".to_owned())
}


/// Expand one logical action into its ordered, finite native gestures.
fn operations(action: RecordedAction) -> Result<Vec<InputOperation>, String> {
    match action {
        RecordedAction::Match(first, second) => Ok(vec![operation(first)?, operation(second)?]),
        RecordedAction::King(location) => Ok(vec![operation(location)?]),
        RecordedAction::Move | RecordedAction::Recycle => Ok(vec![InputOperation::PressDrawKey]),
    }
}


/// Exact fixed-slot tableau identities followed by visible DL and DR identities.
type ExpectedPosition = (
    [Option<xsar::games::cards::Card>; 28],
    Option<xsar::games::cards::Card>,
    Option<xsar::games::cards::Card>,
);


/// Compute all expected visible identities from the shared rules model.
fn expected(deal: &CompleteDeal, state: SearchState) -> Result<ExpectedPosition, String> {
    let mut cards = [None; 28];
    for (index, output) in cards.iter_mut().enumerate() {
        let slot = TableauLocation::from_index(index).ok_or("Invalid expected tableau slot")?;
        if state.tableau_present(slot) {
            *output = rules::card_at(deal, u8::try_from(index).map_err(|_| "Invalid card index")?);
        }
    }
    let pile =
        |location| rules::card_index(state, location).and_then(|index| rules::card_at(deal, index));
    Ok((cards, pile(Location::DL), pile(Location::DR)))
}


/// Injectable guarded I/O boundary for deterministic route-controller testing.
trait RouteIo {
    /// Only explicit no-challenge continuation may schedule fresh Recon from a final ACK.
    fn handoff_after_final_ack(&self) -> bool {
        false
    }

    /// Only a bound automatic session may continue with input-free fresh-board observations.
    /// The caller must already have delivered every gesture of a replayed full clear.
    fn read_only_board_transition(&self) -> bool {
        false
    }


    /// Count an acknowledged RECYCLE separately from the proposed route.
    fn recycle_acknowledged(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Reject revoked execution before any new operation.
    fn check(&self) -> Result<(), String>;
    /// Suspend at a source boundary only; true requires a fresh source after resume.
    fn pause_checkpoint(&mut self) -> Result<bool, String> {
        self.check()?;
        Ok(false)
    }
    /// Obtain fresh evidence for the complete expected visible state.
    fn observe(&mut self, state: SearchState, final_effect: bool) -> Result<bool, String>;
    /// Classify only after the complete logical action has acknowledged every gesture.
    fn observe_effect(
        &mut self,
        state: SearchState,
        final_effect: bool,
    ) -> Result<EffectObservation, String> {
        self.observe(state, final_effect)
            .map(EffectObservation::Position)
    }


    /// Verify a replayed partial endpoint after every final-action gesture ACK.
    ///
    /// Native I/O also recognises the existing measured stock-boundary layouts;
    /// exact position evidence grants neither REDEAL nor next-board authority.
    fn observe_partial(&mut self, state: SearchState) -> Result<EffectObservation, String> {
        self.observe_effect(state, false)
    }


    /// Observe only after the final ACK; positive results require independent pixels.
    fn observe_final(&mut self, state: SearchState) -> Result<Option<BoardCompletion>, String> {
        self.observe(state, true)
            .map(|matched| matched.then_some(BoardCompletion::BoardClear))
    }


    /// Reuse an immediately preceding effect frame or classify a fresh source.
    fn observe_source(&mut self, state: SearchState) -> Result<EffectObservation, String> {
        self.observe(state, false).map(EffectObservation::Position)
    }


    /// Consume checked source evidence for this whole logical action.
    fn begin_action(
        &mut self,
        _state: SearchState,
        _operations: &[InputOperation],
    ) -> Result<(), String> {
        self.check()
    }


    /// Deliver once under a whole-action permit and a fresh running/absolute probe.
    fn deliver(&mut self, operation: InputOperation, state: SearchState) -> Result<(), String>;
    /// Wait cooperatively without resetting the monotonic overall deadline.
    fn wait(&mut self, delay: Duration) -> Result<(), String>;
    /// Monotonic elapsed time measured from the single run origin.
    fn elapsed(&self) -> Duration;
    /// Read actual transport attempts and acknowledgements.
    fn counts(&self) -> Counts;
    /// Record completed verified logical actions separately from acknowledgements.
    fn progress(&mut self, index: usize, total: usize);
}


/// Whole-action evidence keeps terminal success distinct from a hidden replayed position.
enum EffectObservation {
    /// Whether the ordinary exact replayed position is visible.
    Position(bool),
    /// Positive actual bound challenge completion; no remaining route input is allowed.
    Challenge(ChallengeCompletion),
    /// Exact current position reaches the bound count; retire input and await its terminal.
    ChallengePending(ChallengeProgress),
}


/// Report the delivered prefix without claiming the unobserved remainder or tableau state.
fn challenge_report(
    proof: ChallengeCompletion,
    delivered: usize,
    total: usize,
    gestures: usize,
) -> DriveReport {
    DriveReport {
        message: format!(
            "Challenge target verified: {}; {delivered}/{total} route actions delivered, {gestures} gestures acknowledged. Remaining route retired; no next-board or NewGame input.",
            proof.description()
        ),
        completion: BoardCompletion::ChallengeTargetVerified(proof),
    }
}


/// Retire a verified delivered prefix at an actual target without declaring completion.
fn challenge_pending_report(
    progress: ChallengeProgress,
    delivered: usize,
    total: usize,
    gestures: usize,
) -> Result<DriveReport, String> {
    if !progress.target_reached() {
        return Err("Pending challenge terminal requires an actually reached target".to_owned());
    }
    Ok(DriveReport {
        message: format!(
            "Challenge target counter verified: {progress:?}; exact effect of {delivered}/{total} actions verified, {gestures} gestures acknowledged. Card input retired; challenge completion still requires fresh terminal evidence."
        ),
        completion: BoardCompletion::ChallengeTargetReached(progress),
    })
}


/// Controller-visible actual attempted and acknowledged input accounting.
#[derive(Clone, Copy, Debug, Default)]
struct Counts {
    /// Attempt consumed immediately before a typed transport operation.
    attempted: usize,
    /// Transport acknowledged through release, independently of effect.
    acknowledged: usize,
    /// Monotonic run time immediately after the most recent complete transport ACK.
    last_acknowledged_at: Option<Duration>,
}


/// Observe the final delivered action without permitting any further card input.
///
/// The caller must retire its sequencer after every final gesture is acknowledged.
/// Capture starts are bounded by ACK plus two seconds; an already started capture
/// may finish later. STOP/context checks precede any positive result. A missing
/// visual proof returns a typed uncertainty, never a repeat gesture or budget error.
fn confirm_final_action(
    state: SearchState,
    io: &mut impl RouteIo,
    actions: usize,
    gestures: usize,
    acknowledged_at: Duration,
) -> Result<DriveReport, String> {
    if !state.is_goal() {
        return Err("Final route action does not clear the replayed tableau".to_owned());
    }
    let deadline = acknowledged_at
        .checked_add(FINAL_CONFIRMATION_WINDOW)
        .ok_or("Final confirmation deadline overflow")?;
    let mut previous = acknowledged_at;
    let mut observations = 0usize;
    loop {
        io.check()?;
        let now = io.elapsed();
        if now < previous {
            return Err("Final confirmation clock moved backwards".to_owned());
        }
        previous = now;
        if now >= deadline {
            let counts = io.counts();
            if io.read_only_board_transition()
                && counts.attempted == gestures
                && counts.acknowledged == gestures
            {
                return Ok(DriveReport {
                    completion: BoardCompletion::AwaitingFreshBoard,
                    message: format!(
                        "All {actions} full-clear route actions delivered; {gestures} gestures acknowledged; final effect remains unproved. Entering bounded read-only board transition; no card, REDEAL or terminal input authorised."
                    ),
                });
            }
            return Ok(DriveReport {
                message: format!(
                    "All {actions} route actions delivered; {gestures} gestures acknowledged; final board outcome unconfirmed after the two-second acknowledgement window. Inspect and reacquire before continuing; no further card input or retry."
                ),
                completion: BoardCompletion::DeliveredUnconfirmed,
            });
        }
        io.wait(FINAL_OBSERVATION_DELAY.min(deadline.saturating_sub(now)))?;
        io.check()?;
        let now = io.elapsed();
        if now <= previous {
            return Err("Final confirmation clock did not advance after waiting".to_owned());
        }
        previous = now;
        if now >= deadline {
            continue;
        }
        // The time window ordinarily permits nine captures. Retain the old
        // independent capture bound even if an injected clock fails to advance.
        if observations >= FINAL_OBSERVATIONS {
            io.wait(deadline.saturating_sub(now))?;
            continue;
        }
        observations += 1;
        let completion = io.observe_final(state)?;
        io.check()?;
        if let Some(completion) = completion {
            if let BoardCompletion::ChallengeTargetVerified(proof) = completion {
                return Ok(challenge_report(proof, actions, actions, gestures));
            }
            if let BoardCompletion::ChallengeTargetReached(progress) = completion {
                io.progress(actions, actions);
                io.check()?;
                return challenge_pending_report(progress, actions, actions, gestures);
            }
            let description = match completion {
                BoardCompletion::BoardClear => "Pyramid tableau clear verified",
                BoardCompletion::NextBoardReady => "Next Pyramid board verified",
                BoardCompletion::GameComplete => "Pyramid game completion verified",
                BoardCompletion::ChallengeTargetVerified(_) => "Pyramid challenge target verified",
                BoardCompletion::ChallengeTargetReached(_) => {
                    return Err("Pending target escaped its observation-only handover".to_owned());
                }
                BoardCompletion::DeliveredUnconfirmed
                | BoardCompletion::AwaitingFreshBoard
                | BoardCompletion::StockRedealt
                | BoardCompletion::PartialPositionVerified { .. } => {
                    return Err("Final observer returned an invalid positive completion".to_owned());
                }
            };
            io.progress(actions, actions);
            return Ok(DriveReport {
                message: format!(
                    "{description} — {actions}/{actions} actions; {gestures} gestures acknowledged; no next-board or terminal input."
                ),
                completion,
            });
        }
    }
}


/// Run one prevalidated plan using shared sequencing and injected application I/O.
fn drive(plan: &Prepared, io: &mut impl RouteIo) -> Result<DriveReport, String> {
    use xsar::execution::{
        ActionPlan, DeliveryReceipt, EffectVerdict, Limits, ObservationPhase, PreconditionVerdict,
        Request, Sequencer,
    };
    if plan.trace.is_empty() || plan.trace.len() > MAX_ROUTE_ACTIONS {
        return Err("Route action count outside finite Pyramid bounds".to_owned());
    }
    let mut states = vec![SearchState::initial()];
    let mut plans = Vec::new();
    let mut gestures = 0usize;
    for (index, step) in plan.trace.iter().enumerate() {
        let previous = *states.last().ok_or("Missing route state")?;
        let next = rules::apply(&plan.acquisition.deal, previous, step.action)
            .map_err(|e| e.to_string())?;
        let endpoint_matches = match plan.objective {
            RouteObjective::Clear => next.is_goal(),
            RouteObjective::Drain => false,
            RouteObjective::Removal { removed, .. } => {
                !next.is_goal()
                    && next.tableau_remaining() + u32::from(removed) == TABLEAU_LEN as u32
            }
        };
        if index + 1 == plan.trace.len()
            && (previous.is_goal()
                || !endpoint_matches
                || next.tableau_remaining() >= previous.tableau_remaining()
                || !matches!(
                    step.action,
                    RecordedAction::Match(..) | RecordedAction::King(..)
                ))
        {
            return Err(
                "Final route action must remove tableau cards and match the replayed objective"
                    .to_owned(),
            );
        }
        states.push(next);
        let operations = operations(step.action)?;
        gestures = gestures
            .checked_add(operations.len())
            .ok_or("Gesture count overflow")?;
        plans.push(ActionPlan {
            operations,
            settle_delay: if index + 1 == plan.trace.len() {
                FINAL_OBSERVATION_DELAY
            } else {
                match step.action {
                    RecordedAction::Move => crate::parameters::COMPUTED_ROUTE_MOVE_SETTLE_DELAY,
                    RecordedAction::Recycle => crate::parameters::PYRAMID_MOVE_SETTLE_DELAY,
                    _ => Duration::ZERO,
                }
            },
            reobserve_delay: if index + 1 == plan.trace.len() {
                FINAL_OBSERVATION_DELAY
            } else {
                crate::parameters::COMPUTED_ROUTE_REOBSERVE_DELAY
            },
        });
    }
    if gestures > MAX_ROUTE_GESTURES {
        return Err("Route gesture count exceeds Pyramid bound".to_owned());
    }
    let captures = capture_allowance(plan.trace.len(), gestures)?;
    let ordinary_effect_limit = ordinary_effect_observation_limit()?;
    let epoch = plan.id;
    let mut sequence = Sequencer::new(
        epoch,
        plans,
        Limits {
            max_actions: plan.trace.len(),
            max_gestures: gestures,
            max_captures: captures,
            max_precondition_observations: ORDINARY_OBSERVATIONS,
            max_effect_observations: ordinary_effect_limit.max(FINAL_OBSERVATIONS),
            max_duration: EXECUTION_DEADLINE,
        },
        Duration::ZERO,
    )
    .map_err(|e| e.to_string())?;
    let result = (|| {
        let mut phase_observations = 0usize;
        let mut observed_action = None;
        let mut effect_acknowledged_at = None;
        loop {
            io.check()?;
            let request = sequence
                .next(&epoch, io.elapsed())
                .map_err(|e| e.to_string())?;
            match request {
                Request::Observe {
                    ticket,
                    action_index,
                    phase,
                } => {
                    if phase == ObservationPhase::Precondition {
                        io.pause_checkpoint()?;
                    }
                    if observed_action != Some((action_index, phase)) {
                        phase_observations = 0;
                        observed_action = Some((action_index, phase));
                        effect_acknowledged_at = if phase == ObservationPhase::Effect
                            && action_index + 1 < plan.trace.len()
                        {
                            Some(
                                io.counts()
                                    .last_acknowledged_at
                                    .ok_or("Missing ordinary-action acknowledgement clock")?,
                            )
                        } else {
                            None
                        };
                    }
                    phase_observations += 1;
                    let is_effect = phase == ObservationPhase::Effect;
                    let state_index = action_index
                        .checked_add(usize::from(is_effect))
                        .ok_or("Route state overflow")?;
                    let state = *states
                        .get(state_index)
                        .ok_or("Missing expected route state")?;
                    let final_effect = is_effect && action_index + 1 == plan.trace.len();
                    if phase_observations
                        > if final_effect {
                            FINAL_OBSERVATIONS
                        } else if is_effect {
                            ordinary_effect_limit
                        } else {
                            ORDINARY_OBSERVATIONS
                        }
                    {
                        sequence.abort();
                        return Err(
                            "Bounded route observation exhausted; input is not repeated".to_owned()
                        );
                    }
                    if let Some(acknowledged_at) = effect_acknowledged_at {
                        ordinary_effect_remaining(acknowledged_at, io.elapsed()).map_err(
                            |error| {
                                format!("Action {}/{}: {error}", action_index + 1, plan.trace.len())
                            },
                        )?;
                    }
                    let observation = if is_effect {
                        io.observe_effect(state, final_effect)?
                    } else {
                        io.observe_source(state)?
                    };
                    io.check()?;
                    let completed = action_index + usize::from(is_effect);
                    let verified = match observation {
                        EffectObservation::Position(verified) => verified,
                        EffectObservation::ChallengePending(progress) => {
                            sequence.abort();
                            io.progress(completed, plan.trace.len());
                            io.check()?;
                            return challenge_pending_report(
                                progress,
                                completed,
                                plan.trace.len(),
                                io.counts().acknowledged,
                            );
                        }
                        EffectObservation::Challenge(proof) => {
                            sequence.abort();
                            return Ok(challenge_report(
                                proof,
                                completed,
                                plan.trace.len(),
                                io.counts().acknowledged,
                            ));
                        }
                    };
                    if is_effect {
                        sequence
                            .effect(
                                ticket,
                                &epoch,
                                io.elapsed(),
                                if verified {
                                    EffectVerdict::Verified
                                } else {
                                    EffectVerdict::ObserveAgain
                                },
                            )
                            .map_err(|e| e.to_string())?;
                    } else {
                        sequence
                            .precondition(
                                ticket,
                                &epoch,
                                io.elapsed(),
                                if verified {
                                    PreconditionVerdict::Ready
                                } else {
                                    PreconditionVerdict::ObserveAgain
                                },
                            )
                            .map_err(|e| e.to_string())?;
                    }
                }
                Request::Dispatch {
                    ticket,
                    action_index,
                    gesture_index,
                    operation,
                } => {
                    let state = *states.get(action_index).ok_or("Missing pre-input state")?;
                    if gesture_index == 0 {
                        let action = plan
                            .trace
                            .get(action_index)
                            .ok_or("Missing input action")?
                            .action;
                        io.begin_action(state, &operations(action)?)?;
                    }
                    let before = io.counts();
                    let delivery = io.deliver(operation, state);
                    let after = io.counts();
                    let receipt = DeliveryReceipt {
                        attempted: after.attempted > before.attempted,
                        acknowledged: after.acknowledged > before.acknowledged,
                        stop_requested: io.check().is_err(),
                    };
                    let accepted = sequence
                        .delivered(ticket, &epoch, io.elapsed(), receipt)
                        .map_err(|e| e.to_string());
                    delivery.map_err(|e| {
                        format!(
                            "Action {}/{}, gesture {}: {e}",
                            action_index + 1,
                            plan.trace.len(),
                            gesture_index + 1
                        )
                    })?;
                    accepted.map_err(|e| {
                        format!(
                            "Action {}/{}, gesture {}: {e}",
                            action_index + 1,
                            plan.trace.len(),
                            gesture_index + 1
                        )
                    })?;
                    if plan
                        .trace
                        .get(action_index)
                        .is_some_and(|step| step.action == RecordedAction::Recycle)
                    {
                        io.recycle_acknowledged()?;
                    }
                    if action_index + 1 == plan.trace.len()
                        && sequence.statistics().gestures_acknowledged == gestures
                    {
                        let acknowledged_at = after
                            .last_acknowledged_at
                            .ok_or("Missing final acknowledgement clock")?;
                        let state = *states.last().ok_or("Missing final route state")?;
                        // No subsequent sequencer request may dispatch card input.
                        sequence.abort();
                        if state.is_goal() && io.handoff_after_final_ack() {
                            io.check()?;
                            io.wait(crate::parameters::PYRAMID_CLEAR_HANDOFF_DELAY)?;
                            io.check()?;
                            return Ok(DriveReport {
                                completion: BoardCompletion::AwaitingFreshBoard,
                                message: "Final clear-route action acknowledged; three-second wait completed. Guest success remains unproved; fresh Recon must establish the next untouched board.".to_owned(),
                            });
                        }
                        if let RouteObjective::Removal { removed, .. } = plan.objective {
                            return confirm_partial_action(
                                state,
                                io,
                                plan.trace.len(),
                                gestures,
                                acknowledged_at,
                                removed,
                            );
                        }
                        return confirm_final_action(
                            state,
                            io,
                            plan.trace.len(),
                            gestures,
                            acknowledged_at,
                        );
                    }
                }
                Request::Wait { ticket, until } => {
                    let requested = until.saturating_sub(io.elapsed());
                    let delay = if let Some(acknowledged_at) = effect_acknowledged_at {
                        requested.min(ordinary_effect_remaining(acknowledged_at, io.elapsed())?)
                    } else {
                        requested
                    };
                    io.wait(delay)?;
                    if delay < requested {
                        sequence.abort();
                        return Err(
                            "Ordinary effect observation window expired; input is not repeated"
                                .to_owned(),
                        );
                    }
                    sequence
                        .waited(ticket, &epoch, io.elapsed())
                        .map_err(|e| e.to_string())?;
                }
                Request::ActionVerified { action_index } => {
                    io.progress(action_index + 1, plan.trace.len())
                }
                Request::Finished => {
                    return Err(
                        "Route finished without its final acknowledged observation window"
                            .to_owned(),
                    );
                }
                Request::Pending => {
                    sequence.abort();
                    return Err("Route awaited a missing response; no dispatch replay".to_owned());
                }
            }
        }
    })();
    result.map_err(|error| {
        format!(
            "{error}; logical actions verified={}, acknowledged gestures={}",
            sequence.statistics().actions_verified,
            sequence.statistics().gestures_acknowledged
        )
    })
}


/// Why an already captured gameplay frame is being validated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FrameGuardPhase {
    /// Fresh evidence must authorise the next whole logical action.
    Source,
    /// A whole acknowledged action awaits observed effect within its existing deadline.
    Effect,
}


/// Same-frame witness result; an unreadable effect never grants source authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FrameGuardOutcome {
    /// Positive witness matches the run's immutable identity.
    Ready,
    /// An acknowledged action still awaits its witness within the existing deadline.
    Pending,
}


/// Borrowed validation predicate; pending is admitted only at an effect boundary.
/// Neither outcome grants capture, input, replay or an extended observation budget.
type FrameGuard<'a> =
    dyn Fn(&CapturedFrame, FrameGuardPhase) -> Result<FrameGuardOutcome, String> + 'a;


/// Concrete finite worker I/O, holding exclusive run authority and loaded calibration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ExecutionPolicy {
    /// Preserve explicit one-board controls and ordinary final confirmation.
    Standard,
    /// Qualified normal-series progress permits read-only handoff on its first two boards.
    NormalSeries(crate::pyramid_progress::BoardCount),
    /// Continue partial boards under their actual requirement and existing final proof.
    Continuous,
    /// Ignore challenge targets and schedule fresh Recon after a goal-route final ACK.
    NoChallenge,
}


impl ExecutionPolicy {
    /// Only existing challenge and target-agnostic sessions may drain partial-route stock.
    fn continues_stock(self) -> bool {
        matches!(self, Self::Continuous | Self::NoChallenge)
    }
}


/// Concrete finite worker I/O, holding exclusive run authority and loaded calibration.
struct NativeIo<'a> {
    /// Explicit session policy; no-challenge suppresses target classification entirely.
    policy: ExecutionPolicy,
    /// Continuation observations use the separate positive stock-X evidence.
    stock_continuation: bool,
    /// RECYCLE operations actually acknowledged in this consumed execution.
    executed_recycles: u8,
    /// Transport retains existing release-only error recovery.
    qmp: QmpClient,
    /// Immutable prepared plan; never reconstructed after interruption.
    plan: &'a Prepared,
    /// Current operation selection.
    context: &'a ControllerContext,
    /// Existing ordered/coalesced output channels.
    events: &'a WorkerEventSink,
    /// Immediate cooperative STOP latch.
    cancel: &'a AtomicBool,
    /// Optional caller-owned series identity check on the same matched gameplay pixels.
    frame_guard: Option<&'a FrameGuard<'a>>,
    /// One monotonic execution origin, including initial captures and connection.
    started: Instant,
    /// Intentional source-boundary suspension excluded from execution/effect clocks.
    paused: Duration,
    /// Every capture, including guard captures, consumes the same total allowance.
    captures: usize,
    /// Exact route-derived maximum captures.
    capture_limit: usize,
    /// Real transport counts, retained after controller errors.
    counts: Counts,
    /// First unmatched observation; saved only when this execution fails.
    first_mismatch: Option<(String, crate::capture::CapturedFrame)>,
    /// One last native observation, retained without requesting an extra capture.
    latest_frame: Option<crate::capture::CapturedFrame>,
    /// Current refusal and expected state belonging to that exact retained frame.
    latest_observation: Option<diagnostics::Observation>,
    /// Whole-action effects independently verified before any failure.
    verified_actions: usize,
    /// Consecutive actual board or terminal evidence, used only after the final ACK.
    final_confirmation: FinalConfirmation,
    /// Actual challenge identity first bound on a matched original position.
    challenge: Option<ChallengeProgress>,
    /// Immutable original matched requirement for this one board execution.
    challenge_initial: Option<ChallengeProgress>,
    /// Consecutive complete terminals sampled only at a whole-action effect boundary.
    challenge_confirmation: Option<ChallengeCompletion>,
    /// Exact position result from the latest successful observation; grants no input authority.
    position_matches: bool,
    /// Pre-input region retained for this execution, never carried to a new board.
    redeal_before: Option<redeal_appearance::Before>,
    /// Latest matched source, consumed once by a whole logical action.
    source_proof: Option<cadence::SourceProof>,
    /// Ordered operations remaining in the current logical action.
    action_permit: Option<cadence::ActionPermit>,
}


impl NativeIo<'_> {
    /// Classify fresh native pixels; ordinary frames remain memory-only diagnostics.
    fn observe_checked(
        &mut self,
        state: SearchState,
        final_effect: bool,
        allow_pending_header: bool,
    ) -> Result<bool, String> {
        self.position_matches = false;
        self.source_proof = None;
        self.challenge_confirmation = None;
        if self.action_permit.is_some() {
            self.action_permit = None;
            return Err(
                "Observation interrupted an unfinished action; no further input".to_owned(),
            );
        }
        self.check()?;
        if self.captures >= self.capture_limit {
            return Err("Route capture allowance exhausted".to_owned());
        }
        self.captures += 1;
        send_state(self.events, WorkerState::Capturing);
        let (frame, _) = capture_screen(&mut self.qmp, &self.context.socket_path, self.events)?;
        // One bounded native copy preserves the last observation even after UI consumption.
        self.latest_frame = Some(frame.clone());
        self.latest_observation = Some(diagnostics::Observation {
            capture: self.captures,
            phase: if allow_pending_header {
                FrameGuardPhase::Effect
            } else {
                FrameGuardPhase::Source
            },
            state,
            final_effect,
            acknowledged: self.counts.acknowledged,
            mismatch: None,
        });
        let result = self.classify_observation(frame, state, final_effect, allow_pending_header);
        if let Err(reason) = &result
            && let Some(observation) = &mut self.latest_observation
        {
            observation.mismatch.get_or_insert_with(|| reason.clone());
        }
        result
    }


    /// Classify one already retained frame without another capture or input attempt.
    /// The caller binds all errors to these pixels while preserving first-failure evidence.
    fn classify_observation(
        &mut self,
        frame: CapturedFrame,
        state: SearchState,
        final_effect: bool,
        allow_pending_header: bool,
    ) -> Result<bool, String> {
        // A bound completion retires input before hidden cards can be misclassified.
        if let Some(bound) = self.challenge
            && let Some(proof) =
                crate::pyramid_progress::observe_bound_challenge_complete(&frame, bound)?
        {
            if !proof.progress().follows(bound) {
                return Err("Challenge Complete requirement differs from the bound challenge; no further input".to_owned());
            }
            self.challenge = Some(proof.progress());
            self.challenge_confirmation = Some(proof);
            self.events.publish_diagnostic_frame(frame);
            self.check()?;
            return Ok(false);
        }
        let (tableau, left, right) = expected(&self.plan.acquisition.deal, state)?;
        // A winning trace can exhaust LEFT before its last match and subsequent King.
        // This proves only the exact position; the optional REDEAL point is discarded.
        let winning_stock_boundary = matches!(self.plan.objective, RouteObjective::Clear)
            && !final_effect
            && !state.is_goal()
            && left.is_none();
        let observed = if self.stock_continuation || winning_stock_boundary {
            let appeared = self
                .redeal_before
                .as_ref()
                .map(|before| before.appeared(&frame))
                .transpose()?
                .unwrap_or(false);
            self.plan
                .acquisition
                .vision
                .verify_temporal_stock_position(&frame, &tableau, left, right, appeared)
        } else {
            self.plan
                .acquisition
                .vision
                .verify_position(&frame, &tableau, left, right)
                .or_else(|ordinary_error| {
                    if final_effect && state.is_goal() {
                        self.plan
                            .acquisition
                            .vision
                            .verify_final_clear(&frame, &tableau, left, right)
                            .or_else(|_| {
                                self.plan
                                    .acquisition
                                    .vision
                                    .verify_normal_final_clear(&frame, &tableau, left, right)
                            })
                            .or_else(|error| {
                                if let Some(initial) = self.challenge_initial {
                                    self.plan
                                        .acquisition
                                        .vision
                                        .verify_board_count_final_clear(
                                            &frame, &tableau, left, right, initial,
                                        )
                                        .or_else(|_| {
                                            self.plan
                                                .acquisition
                                                .vision
                                                .verify_board_count_target_clear(
                                                    &frame, &tableau, left, right, initial,
                                                )
                                        })
                                        .or_else(|_| {
                                            self.plan
                                                .acquisition
                                                .vision
                                                .verify_challenge_final_clear(
                                                    &frame, &tableau, left, right, initial,
                                                )
                                        })
                                } else {
                                    Err(error)
                                }
                            })
                            .map_err(|alternative_error| format!("{ordinary_error}; final-clear alternatives refused: {alternative_error}"))
                    } else {
                        Err(ordinary_error)
                    }
                })
        };
        // A header refusal is fatal, rather than an unmatched position that could
        // be reclassified as a new board or terminal after the final gesture.
        let guarded = if observed.is_ok() {
            self.frame_guard
                .map_or(Ok(FrameGuardOutcome::Ready), |guard| {
                    let phase = if allow_pending_header {
                        FrameGuardPhase::Effect
                    } else {
                        FrameGuardPhase::Source
                    };
                    guard(&frame, phase).map_err(|error| {
                        format!("Computed-route frame guard rejected current gameplay: {error}")
                    })
                })
        } else {
            Ok(FrameGuardOutcome::Ready)
        };
        let reason = observed.as_ref().err().or_else(|| guarded.as_ref().err());
        if let Some(observation) = &mut self.latest_observation {
            observation.mismatch = reason.cloned();
        }
        if let Some(reason) = reason
            && self.first_mismatch.is_none()
        {
            send_log(
                self.events,
                format!("First unmatched computed-route observation: {reason}"),
            );
            self.first_mismatch = Some((reason.clone(), frame.clone()));
        }
        let mut matched = observed.is_ok();
        // A qualified acquisition owns the challenge for this gameplay lifetime.
        // Flying cards must not reclassify it or introduce extra effect captures.
        // Unbound initial entry still rejects a newly appeared challenge.
        if matched
            && self.policy != ExecutionPolicy::NoChallenge
            && self.challenge.is_none()
            && state == SearchState::initial()
            && self.counts.attempted == 0
        {
            let actual = crate::pyramid_progress::observe_challenge(&frame)?;
            if actual != self.plan.acquisition.challenge_progress {
                return Err("Fresh challenge requirement differs from the restored Recon binding; no input authorised".to_owned());
            }
        }
        // Final transition classification is restricted to observe_final below.
        self.events.publish_diagnostic_frame(frame);
        self.check()?;
        match guarded? {
            FrameGuardOutcome::Ready => {}
            FrameGuardOutcome::Pending if allow_pending_header => {
                matched = false;
                if let Some(observation) = &mut self.latest_observation {
                    observation.mismatch = Some("Same-frame series witness unreadable; effect pending".to_owned());
                }
                send_log(self.events, "Computed-route effect pending: same-frame series witness unreadable; no input authorised".to_owned());
            }
            FrameGuardOutcome::Pending => {
                return Err("Source frame guard is pending; no input authorised".to_owned());
            }
        }
        // Only the positively matched whole-board endpoint may advance the bound
        // counter. Gameplay frames never reclassify the challenge's label/target.
        if matched
            && final_effect
            && state.is_goal()
            && let Some(previous) = self.challenge
        {
            let actual = crate::pyramid_progress::observe_bound_challenge_progress(
                self.latest_frame
                    .as_ref()
                    .ok_or("Missing final native observation")?,
                previous,
            )?;
            self.check()?;
            match actual {
                Some(current) if current.follows(previous) => self.challenge = Some(current),
                None if allow_pending_header => {
                    matched = false;
                    if let Some(observation) = &mut self.latest_observation {
                        observation.mismatch = Some("Bound challenge counter unreadable; final effect pending".to_owned());
                    }
                    send_log(self.events, "Computed-route final effect pending: bound challenge counter unreadable; no input authorised".to_owned());
                }
                actual => {
                    return Err(format!(
                        "Challenge objective changed, regressed or disappeared: expected {previous:?}, observed {actual:?}; no further input"
                    ));
                }
            }
        }
        self.position_matches = matched;
        let ready = matched
            && !self
                .challenge
                .is_some_and(ChallengeProgress::target_reached);
        if ready {
            if self.redeal_before.is_none() && self.counts.attempted == 0 {
                self.redeal_before = Some(redeal_appearance::Before::capture(
                    self.latest_frame
                        .as_ref()
                        .ok_or("Missing REDEAL pre-input frame")?,
                )?);
            }
            self.source_proof = Some(cadence::SourceProof::new(self, state));
        }
        Ok(ready)
    }

}


impl RouteIo for NativeIo<'_> {
    /// Revoke cached pre-pause pixels before reacquiring the next whole-action source.
    fn pause_checkpoint(&mut self) -> Result<bool, String> {
        let result = self.events.latest_frame.pause.checkpoint(
            super::pyramid_pause::Phase::Gameplay,
            || {
                check_acquisition(
                    &self.plan.acquisition,
                    self.context,
                    self.events,
                    self.cancel,
                )
            },
        );
        let paused = match result {
            Ok(paused) => paused,
            Err(error) => {
                self.paused = self.paused.saturating_add(error.paused);
                return Err(error.message);
            }
        };
        self.paused = self.paused.saturating_add(paused);
        if paused.is_zero() {
            return Ok(false);
        }
        self.source_proof = None;
        self.action_permit = None;
        self.check()?;
        Ok(true)
    }


    /// The opt-in no-challenge loop alone schedules acquisition from delivery.
    fn handoff_after_final_ack(&self) -> bool {
        self.policy == ExecutionPolicy::NoChallenge
    }


    /// Incomplete bound sessions may observe a board; the normal third clear remains strict.
    fn read_only_board_transition(&self) -> bool {
        match self.policy {
            ExecutionPolicy::Continuous => self
                .challenge
                .is_some_and(|progress| !progress.target_reached()),
            ExecutionPolicy::NormalSeries(progress) => {
                progress.target == 3
                    && progress.completed < 2
                    && self.frame_guard.is_some()
                    && self.challenge.is_none()
            }
            ExecutionPolicy::Standard | ExecutionPolicy::NoChallenge => false,
        }
    }


    /// Checked actual accounting includes route and residual stock operations.
    fn recycle_acknowledged(&mut self) -> Result<(), String> {
        self.executed_recycles = self
            .executed_recycles
            .checked_add(1)
            .filter(|count| *count <= xsar::games::pyramid::planning::MAX_RECYCLES)
            .ok_or("Acknowledged RECYCLE count exceeds supported two-recycle rules")?;
        Ok(())
    }


    /// Enforce monotonic origin/context/STOP and the one overall deadline.
    fn check(&self) -> Result<(), String> {
        check_acquisition(
            &self.plan.acquisition,
            self.context,
            self.events,
            self.cancel,
        )?;
        if self.elapsed() >= EXECUTION_DEADLINE {
            return Err("Execution deadline expired; no further input".to_owned());
        }
        Ok(())
    }


    /// Source observations still require a positively readable bound requirement.
    fn observe(&mut self, state: SearchState, final_effect: bool) -> Result<bool, String> {
        self.observe_checked(state, final_effect, false)
    }


    /// A terminal can retire a route only after a whole MATCH/KING/MOVE acknowledges.
    fn observe_effect(
        &mut self,
        state: SearchState,
        final_effect: bool,
    ) -> Result<EffectObservation, String> {
        let matched = self.observe_checked(state, final_effect, true)?;
        self.check()?;
        if let Some(proof) = self.challenge_confirmation {
            return Ok(EffectObservation::Challenge(proof));
        }
        if self.position_matches
            && let Some(progress) = self.challenge.filter(|progress| progress.target_reached())
        {
            return Ok(EffectObservation::ChallengePending(progress));
        }
        Ok(EffectObservation::Position(matched))
    }


    /// Prove the exact partial endpoint, including measured empty-stock controls.
    ///
    /// The alternative recogniser retains all card, scene and modal checks. Its
    /// scope ends with this observation, including errors, and it grants no input.
    fn observe_partial(&mut self, state: SearchState) -> Result<EffectObservation, String> {
        if !self.policy.continues_stock() {
            return self.observe_effect(state, false);
        }
        if state.is_goal()
            || !matches!(self.plan.objective, RouteObjective::Removal { removed, .. }
                if removed > 0 && state.tableau_remaining() + u32::from(removed) == TABLEAU_LEN as u32)
        {
            return Err(
                "Partial observation differs from the replayed removal endpoint".to_owned(),
            );
        }
        let previous = self.stock_continuation;
        self.stock_continuation = true;
        let result = self.observe_effect(state, false);
        self.stock_continuation = previous;
        result
    }


    /// Use the same fresh capture for exact clear, complete redeal or terminal proof.
    fn observe_final(&mut self, state: SearchState) -> Result<Option<BoardCompletion>, String> {
        let matched = match self.observe_effect(state, true)? {
            EffectObservation::Position(matched) => matched,
            EffectObservation::ChallengePending(progress) => {
                return Ok(Some(BoardCompletion::ChallengeTargetReached(progress)));
            }
            EffectObservation::Challenge(proof) => {
                return Ok(Some(BoardCompletion::ChallengeTargetVerified(proof)));
            }
        };
        let frame = self
            .latest_frame
            .as_ref()
            .ok_or("Missing final native observation")?;
        let completion =
            self.final_confirmation
                .observe(&self.plan.acquisition.vision, frame, matched)?;
        self.check()?;
        Ok(completion)
    }


    /// Share one freshly checked boundary frame with the immediately following action.
    fn observe_source(&mut self, state: SearchState) -> Result<EffectObservation, String> {
        self.check()?;
        if self
            .source_proof
            .is_some_and(|proof| proof.matches(self, state))
        {
            send_log(
                self.events,
                format!(
                    "Computed-route source reused: capture={}, acknowledged={}",
                    self.captures, self.counts.acknowledged
                ),
            );
            return Ok(EffectObservation::Position(true));
        }
        let matched = self.observe(state, false)?;
        self.check()?;
        if let Some(proof) = self.challenge_confirmation {
            return Ok(EffectObservation::Challenge(proof));
        }
        if self.position_matches
            && let Some(progress) = self.challenge.filter(|progress| progress.target_reached())
        {
            return Ok(EffectObservation::ChallengePending(progress));
        }
        Ok(EffectObservation::Position(matched))
    }


    /// Consume one exact source proof; it cannot authorise a different logical action.
    fn begin_action(
        &mut self,
        state: SearchState,
        operations: &[InputOperation],
    ) -> Result<(), String> {
        self.check()?;
        if self.action_permit.is_some() {
            self.action_permit = None;
            self.source_proof = None;
            return Err("Previous logical action is unfinished; no further input".to_owned());
        }
        let proof = self
            .source_proof
            .take()
            .ok_or("Missing checked action source; no input authorised")?;
        if !proof.matches(self, state) {
            return Err("Stale or different action source; no input authorised".to_owned());
        }
        self.action_permit = Some(cadence::ActionPermit::new(proof, operations)?);
        Ok(())
    }


    /// Consume the next permitted gesture, preserving fresh probes and STOP between clicks.
    fn deliver(&mut self, operation: InputOperation, state: SearchState) -> Result<(), String> {
        let permit = self
            .action_permit
            .take()
            .ok_or("Missing whole-action permit; no input authorised")?;
        self.check()?;
        if !permit.matches(self, state, operation) {
            return Err(
                "Input differs from the remaining action permit; no input authorised".to_owned(),
            );
        }
        let probe = self.qmp.probe().map_err(|e| e.to_string())?;
        validate_probe(&probe)?;
        self.check()?;
        self.counts.attempted = self
            .counts
            .attempted
            .checked_add(1)
            .ok_or("Gesture counter overflow")?;
        send_state(self.events, WorkerState::Acting);
        let result = match operation {
            InputOperation::Click(point) => self.qmp.click(point, 1920, 1080),
            InputOperation::PressDrawKey => self.qmp.press_draw_key(),
        };
        result.map_err(|e| {
            format!("Uncertain route delivery: {e}; inspect and reacquire, no retry")
        })?;
        self.counts.acknowledged = self
            .counts
            .acknowledged
            .checked_add(1)
            .ok_or("Gesture counter overflow")?;
        self.counts.last_acknowledged_at = Some(self.elapsed());
        self.action_permit = permit.acknowledged();
        Ok(())
    }


    /// Measure the sliced wait while preserving STOP and deadline checks on return.
    fn wait(&mut self, delay: Duration) -> Result<(), String> {
        self.source_proof = None;
        self.action_permit = None;
        self.check()?;
        let started = Instant::now();
        let result = cancellable_wait(delay, self.cancel)
            .map_err(|_| "STOP during route wait; no cleanup input".to_owned())
            .and_then(|_| self.check());
        send_log(
            self.events,
            format!(
                "Computed-route wait: configured={:.3} ms, elapsed={:.3} ms, completed={}",
                delay.as_secs_f64() * 1_000.0,
                started.elapsed().as_secs_f64() * 1_000.0,
                result.is_ok(),
            ),
        );
        result
    }


    /// Return elapsed time from the unchanged origin.
    fn elapsed(&self) -> Duration {
        self.started.elapsed().saturating_sub(self.paused)
    }


    /// Return exact I/O counts independently of logical commits.
    fn counts(&self) -> Counts {
        self.counts
    }


    /// Report only independently observed committed actions.
    fn progress(&mut self, index: usize, total: usize) {
        self.verified_actions = index;
        send_state(self.events, WorkerState::Verifying);
        send_log(
            self.events,
            format!(
                "Computed route action {index}/{total} effect verified; attempted={}, acknowledged={}",
                self.counts.attempted, self.counts.acknowledged
            ),
        );
        send_status(
            self.events,
            format!("Computed route action {index}/{total} verified"),
        );
    }
}


/// Consume the executable permit before connection, and never retain it on failure.
pub(super) fn execute(
    context: &ControllerContext,
    run_id: u64,
    plan_id: u64,
    confirmed: bool,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) {
    let result =
        execute_owned(context, run_id, plan_id, confirmed, 0, events, cancel).and_then(|report| {
            if report.completion == BoardCompletion::DeliveredUnconfirmed {
                Err(report.message)
            } else {
                Ok(report.message)
            }
        });
    finish(events, run_id, None, result, false);
}


/// Consume one plan while retaining the caller's exclusive worker reservation.
///
/// No outcome retains executable authority, clears STOP or releases the reservation.
/// Every error and delivered-but-unconfirmed outcome persists existing diagnostics;
/// the caller decides whether a positively confirmed result permits another phase.
pub(super) fn execute_owned(
    context: &ControllerContext,
    run_id: u64,
    plan_id: u64,
    confirmed: bool,
    board_generation: u64,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> Result<ExecutionReport, String> {
    execute_owned_guarded(
        context,
        run_id,
        plan_id,
        confirmed,
        board_generation,
        None,
        ExecutionPolicy::Standard,
        events,
        cancel,
    )
}


/// Execute with a borrowed validation-only predicate on existing matched gameplay captures.
///
/// The caller retains its qualified same-session series witness. The predicate
/// runs after exact expected-position recognition, including an empty-tableau
/// goal, and must perform no capture, input or authority mutation. A refusal is
/// fatal and retains normal first/latest diagnostics. Unmatched specific terminal
/// and fresh-board frames remain available to the existing final classifier;
/// their hidden or changed gameplay headers are not checked by this predicate.
/// Source/effect phase is explicit: a challenge guard may defer an unreadable
/// effect header to the existing confirmation window, but cannot permit input.
/// No additional capture, wait, input or retry budget is introduced.
///
/// # Errors
/// Returns the existing execution error or a predicate refusal. STOP/context
/// revocation retains priority after observation, and no refusal can become
/// completion or leave executable authority available for another attempt.
#[expect(
    clippy::too_many_arguments,
    reason = "retains the existing execution identity and caller-owned frame predicate explicitly"
)]
pub(super) fn execute_owned_with_frame_guard(
    context: &ControllerContext,
    run_id: u64,
    plan_id: u64,
    confirmed: bool,
    board_generation: u64,
    frame_guard: &FrameGuard<'_>,
    policy: ExecutionPolicy,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> Result<ExecutionReport, String> {
    execute_owned_guarded(
        context,
        run_id,
        plan_id,
        confirmed,
        board_generation,
        Some(frame_guard),
        policy,
        events,
        cancel,
    )
}


/// Execute a session's explicit continuation policy under its existing reservation.
///
/// The plan is consumed once. Policy changes final handoff and residual-stock work;
/// it cannot bypass context, STOP, exact position or fresh probe checks.
pub(super) fn execute_session(
    context: &ControllerContext,
    run_id: u64,
    board_generation: u64,
    policy: ExecutionPolicy,
    frame_guard: Option<&FrameGuard<'_>>,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> Result<ExecutionReport, String> {
    execute_owned_guarded(
        context,
        run_id,
        run_id,
        true,
        board_generation,
        frame_guard,
        policy,
        events,
        cancel,
    )
}


/// Consume one plan and retain existing execution/diagnostic bounds with an optional guard.
///
/// The borrowed predicate cannot outlive this execution. Existing callers use
/// `None`, preserving their card/scene recognition and final transition policy.
///
/// # Errors
/// Returns a consumed-plan, authority, transport, recognition, guard or finite
/// execution error and persists the same bounded first/latest evidence.
#[expect(
    clippy::too_many_arguments,
    reason = "one implementation preserves both unguarded and caller-guarded owned execution"
)]
fn execute_owned_guarded(
    context: &ControllerContext,
    run_id: u64,
    plan_id: u64,
    confirmed: bool,
    board_generation: u64,
    frame_guard: Option<&FrameGuard<'_>>,
    policy: ExecutionPolicy,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> Result<ExecutionReport, String> {
    let mut counts = Counts::default();
    let mut source_document = None;
    let mut first_frame = None;
    let mut latest_frame = None;
    let mut details = serde_json::json!({
        "plan_id": plan_id,
        "board_generation": board_generation,
        "explicit_solve_request": confirmed,
        "no_resume": true,
        "no_cleanup_input": true,
        "final_confirmation_window_seconds": FINAL_CONFIRMATION_WINDOW.as_secs_f64(),
        "ordinary_effect_window_seconds": crate::parameters::COMPUTED_ROUTE_EFFECT_WINDOW.as_secs_f64(),
    });
    let result = (|| {
        let mut store = events
            .latest_frame
            .computed_route
            .lock()
            .map_err(|_| "Route receipt lock poisoned")?;
        let prepared = store.prepared.take();
        store.acquisition = None;
        drop(store);
        let plan =
            prepared.ok_or("No executable route; calculate from a fresh restored acquisition")?;
        source_document = Some(plan.acquisition.bytes.clone());
        validate_context(context)?;
        record_detail(
            &mut details,
            "profile_id",
            serde_json::json!(plan.acquisition.vision.profile_id()),
        );
        record_detail(
            &mut details,
            "planned_actions",
            serde_json::json!(plan.trace.len()),
        );
        if !confirmed || plan.id != plan_id {
            return Err("Route consent or plan identity mismatch; acquire again".to_owned());
        }
        check_acquisition(&plan.acquisition, context, events, cancel)?;
        let gestures = plan.trace.iter().try_fold(0usize, |sum, step| {
            operations(step.action).and_then(|ops| {
                sum.checked_add(ops.len())
                    .ok_or("Gesture overflow".to_owned())
            })
        })?;
        let started = Instant::now();
        let qmp = QmpClient::connect(&context.socket_path).map_err(|e| e.to_string())?;
        let mut io = NativeIo {
            policy,
            stock_continuation: false,
            executed_recycles: 0,
            qmp,
            plan: &plan,
            context,
            events,
            cancel,
            frame_guard,
            started,
            paused: Duration::ZERO,
            captures: 0,
            capture_limit: (if plan.objective == RouteObjective::Drain {
                0
            } else {
                capture_allowance(plan.trace.len(), gestures)?
            })
            .checked_add(if policy.continues_stock() { 1200 } else { 0 })
            .ok_or("Continuation capture allowance overflow")?,
            counts,
            first_mismatch: None,
            latest_frame: None,
            latest_observation: None,
            verified_actions: 0,
            final_confirmation: FinalConfirmation::default(),
            challenge: if policy == ExecutionPolicy::NoChallenge {
                None
            } else {
                plan.acquisition.challenge_progress
            },
            challenge_initial: if policy == ExecutionPolicy::NoChallenge {
                None
            } else {
                plan.acquisition.challenge_progress
            },
            challenge_confirmation: None,
            position_matches: false,
            source_proof: None,
            action_permit: None,
            redeal_before: None,
        };
        let planned_recycles = plan
            .trace
            .iter()
            .filter(|step| step.action == RecordedAction::Recycle)
            .count();
        send_log(
            events,
            format!("Route planned RECYCLE operations: {planned_recycles}"),
        );
        let result = if plan.objective == RouteObjective::Drain {
            if !policy.continues_stock() {
                Err("Exhaustive no-win stock continuation requires an automatic session; no card input sent".to_owned())
            } else {
                continuation::drain_and_redeal(&mut io, SearchState::initial())
            }
        } else {
            drive(&plan, &mut io).and_then(|report| {
                if policy.continues_stock()
                    && matches!(
                        report.completion,
                        BoardCompletion::PartialPositionVerified { .. }
                    )
                {
                    let state =
                        plan.trace
                            .iter()
                            .try_fold(SearchState::initial(), |state, step| {
                                rules::apply(&plan.acquisition.deal, state, step.action)
                                    .map_err(|error| error.to_string())
                            })?;
                    continuation::drain_and_redeal(&mut io, state)
                } else {
                    Ok(report)
                }
            })
        };
        counts = io.counts;
        record_detail(
            &mut details,
            "planned_recycles",
            serde_json::json!(planned_recycles),
        );
        record_detail(
            &mut details,
            "executed_recycles",
            serde_json::json!(io.executed_recycles),
        );
        record_detail(&mut details, "captures", serde_json::json!(io.captures));
        record_detail(
            &mut details,
            "capture_limit",
            serde_json::json!(io.capture_limit),
        );
        record_detail(
            &mut details,
            "verified_actions",
            serde_json::json!(io.verified_actions),
        );
        record_detail(
            &mut details,
            "last_acknowledged_seconds",
            serde_json::json!(counts.last_acknowledged_at.map(|time| time.as_secs_f64())),
        );
        latest_frame = io.latest_frame.take();
        let diagnostic_failure = result.is_err()
            || result
                .as_ref()
                .is_ok_and(|report| report.completion == BoardCompletion::DeliveredUnconfirmed);
        let mut result = result;
        if diagnostic_failure {
            if let Some(observation) = &io.latest_observation {
                match observation.value(&plan.acquisition.deal) {
                    Ok(value) => record_detail(&mut details, "last_observation", value),
                    Err(error) => record_detail(
                        &mut details,
                        "last_observation_diagnostic_error",
                        serde_json::json!(error),
                    ),
                }
                record_detail(
                    &mut details,
                    "last_unmatched_observation",
                    serde_json::json!(observation.mismatch),
                );
                if let Some(reason) = &observation.mismatch {
                    result = result.map_err(|error| {
                        format!(
                            "{error}; last unmatched observation (capture {}): {reason}",
                            observation.capture
                        )
                    });
                }
            }
            match diagnostics::pending_action(&plan, io.verified_actions) {
                Ok(action) => {
                    record_detail(&mut details, "pending_action", serde_json::json!(action))
                }
                Err(error) => record_detail(
                    &mut details,
                    "pending_action_diagnostic_error",
                    serde_json::json!(error),
                ),
            }
        }
        if diagnostic_failure && let Some((reason, frame)) = io.first_mismatch.take() {
            record_detail(
                &mut details,
                "first_unmatched_observation",
                serde_json::json!(&reason),
            );
            events.publish_diagnostic_frame(frame.clone());
            first_frame = Some(frame);
            result = result.map_err(|e| format!("{e}; first unmatched observation: {reason}"));
        }
        let challenge_progress = io.challenge;
        drop(io);
        let report = result?;
        record_detail(
            &mut details,
            "completion",
            serde_json::json!(format!("{:?}", report.completion)),
        );
        Ok(ExecutionReport {
            challenge_progress,
            message: report.message,
            completion: report.completion,
            latest_frame: latest_frame.clone(),
            vision: plan.acquisition.vision,
        })
    })();
    let result = result.map_err(|e| format!("{}: {}; gestures attempted={}, acknowledged={}; logical effect may be incomplete. Inspect and reacquire before another execution; no cleanup input sent.", if counts.attempted == 0 { "Route refused before input" } else { "Route execution stopped" }, RouteError::Refused(e), counts.attempted, counts.acknowledged));
    let diagnostic_error = match &result {
        Err(error) => Some(error.as_str()),
        Ok(report) if report.completion == BoardCompletion::DeliveredUnconfirmed => {
            Some(report.message.as_str())
        }
        Ok(_) => None,
    };
    if let Some(error) = diagnostic_error {
        record_detail(
            &mut details,
            "gestures_attempted",
            serde_json::json!(counts.attempted),
        );
        record_detail(
            &mut details,
            "gestures_acknowledged",
            serde_json::json!(counts.acknowledged),
        );
        publish_pyramid_failure(
            events,
            FailureEvidence {
                context,
                phase: PyramidPhase::Execution,
                run_id: Some(run_id),
                error,
                details,
                source_document: source_document.as_deref(),
                first_frame: first_frame.as_ref(),
                latest_frame: latest_frame.as_ref(),
            },
        );
    }
    result
}


#[cfg(test)]
mod tests;


#[cfg(test)]
mod wire_tests;


#[cfg(test)]
mod series_guard_tests;
