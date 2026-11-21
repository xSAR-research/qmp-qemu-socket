//! Offline Free Cell calculation. This adapter has no capture, QMP or input APIs.
//!
//! A dedicated reservation owns cancellation; progress is advisory and coalesced.
//! Input is parsed once and every successful route is independently replayed.

use super::*;
use std::{
    cell::{Cell, RefCell},
    io::Read,
};
use xsar::games::freecell::{
    InitialDeal,
    rules::{Position, RecordedMove, Successors},
    search::{BestEffortOutcome as Outcome, Failure, Progress as SearchProgress},
};
use xsar::search::{
    BestEffortControl, BestEffortOutcome, BestEffortProblem, BestEffortProgress, PartialProof,
    SearchProblem, SearchStop,
};


/// Exact UI correlation, unrelated to acquisition/execution authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Identity {
    /// Selected UI generation, for stale-result rejection only.
    pub context: ControllerContext,
    /// Monotonic calculation request.
    pub request_id: u64,
    /// Monotonic selected-document generation.
    pub source_generation: u64,
}


/// One replaceable actual search snapshot, including best banked count.
#[derive(Clone, Debug)]
pub(crate) struct Progress {
    /// Exact pending job.
    pub identity: Identity,
    /// Engine counters; finished means search ended, not replay acceptance.
    pub search: SearchProgress,
    /// Actual completed intentional suspension, including replay checkpoints.
    pub paused: Duration,
}


/// Frozen source identity and typed complete, exhaustive-partial or unusable result.
#[derive(Debug)]
pub(crate) struct Report {
    /// Exact pathname used, never a live acquisition receipt.
    pub path: PathBuf,
    /// Validated immutable original input, available for complete route inspection.
    pub deal: xsar::games::freecell::InitialDeal,
    /// Original loaded bytes, bounded by the codec and never reread during search.
    pub bytes: Vec<u8>,
    /// Solved is complete; Partial is independently replayed (`partial_proof` distinguishes Exhaustive vs Play interrupt).
    pub outcome: Outcome,
    /// Proof status of any [`Outcome::Partial`] (`Exhaustive` unless interrupted early by `Play`).
    pub partial_proof: PartialProof,
    /// Actual completed intentional suspension for the whole calculation.
    pub paused: Duration,
}


/// Typed authority refusal at the failing checkpoint, never inferred from strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
enum ControlError {
    /// The active request was explicitly cancelled.
    #[error("Free Cell calculation cancelled")]
    Cancelled,
    /// A stale identity cannot continue or publish a result.
    #[error("Free Cell calculation retired")]
    Retired,
    /// A poisoned authority store fails closed.
    #[error("Free Cell calculation control poisoned")]
    Poisoned,
    /// Shutdown terminates this request without a route.
    #[error("Application shutdown")]
    Shutdown,
}


/// One worker reservation plus a latest-only progress slot.
#[derive(Default)]
pub(super) struct Control {
    /// Active identity and its independent out-of-band cancellation flag.
    active: Mutex<Option<(Identity, Arc<AtomicBool>)>>,
    /// Bounded advisory mailbox, never a queued history.
    progress: Mutex<Option<Progress>>,
}


impl Control {
    /// Reserve one explicit job; a cancelled job still owns the slot until finish.
    pub(super) fn reserve(&self, identity: Identity) -> Result<Arc<AtomicBool>, String> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| "Free Cell calculation control poisoned")?;
        if active.is_some() {
            return Err("Free Cell calculation already active".to_owned());
        }
        let cancel = Arc::new(AtomicBool::new(false));
        *self
            .progress
            .lock()
            .map_err(|_| "Free Cell progress poisoned")? = None;
        *active = Some((identity, Arc::clone(&cancel)));
        Ok(cancel)
    }


    /// Refuse another identity or a cancelled reservation without changing it.
    fn check(&self, identity: &Identity) -> Result<(), ControlError> {
        let active = self.active.lock().map_err(|_| ControlError::Poisoned)?;
        match active.as_ref() {
            Some((actual, cancel)) if actual == identity => {
                if cancel.load(Ordering::Acquire) {
                    Err(ControlError::Cancelled)
                } else {
                    Ok(())
                }
            }
            _ => Err(ControlError::Retired),
        }
    }


    /// Cancel out of band; identity optionally limits cancellation to one caller.
    pub(super) fn cancel(&self, identity: Option<&Identity>) {
        if let Ok(active) = self.active.lock()
            && let Some((actual, cancel)) = active.as_ref()
            && identity.is_none_or(|expected| expected == actual)
        {
            cancel.store(true, Ordering::Release);
        }
    }


    /// True only for a currently reserved, uncancelled job.
    pub(super) fn running(&self) -> bool {
        self.active
            .lock()
            .ok()
            .is_some_and(|a| a.as_ref().is_some_and(|(_, c)| !c.load(Ordering::Acquire)))
    }


    /// True when an active reservation exists and its cancellation latch is set.
    pub(super) fn is_cancelled(&self) -> bool {
        self.active
            .lock()
            .ok()
            .is_some_and(|a| a.as_ref().is_some_and(|(_, c)| c.load(Ordering::Acquire)))
    }


    /// Retire only this exact reservation, leaving stale final events harmless.
    pub(super) fn finish(&self, identity: &Identity) -> Result<(), String> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| "Free Cell calculation retirement poisoned")?;
        if active
            .as_ref()
            .is_some_and(|(actual, _)| actual == identity)
        {
            *active = None;
        }
        Ok(())
    }


    /// Replace progress only for a still-active identity.
    fn publish(&self, progress: Progress) {
        if self.check(&progress.identity).is_ok()
            && let Ok(mut slot) = self.progress.lock()
        {
            *slot = Some(progress);
        }
    }


    /// Consume the latest snapshot; late final progress cannot revive UI ownership.
    pub(super) fn take(&self) -> Option<Progress> {
        self.progress.lock().ok()?.take()
    }
}


/// Validated deal input and cancellation fences passed to [`search_deal_with_control`].
pub(super) struct SearchInput<'a> {
    /// Immutable initial deal validated for acquisition objective.
    pub(super) deal: xsar::games::freecell::InitialDeal,
    /// Origin JSON path preserved in the final immutable [`Report`].
    pub(super) path: PathBuf,
    /// Exact bounded JSON bytes retained in the final [`Report`].
    pub(super) bytes: Vec<u8>,
    /// Shared monotonic cancellation latch.
    pub(super) cancel: &'a AtomicBool,
    /// Event sink hosting pause-checkpoint coordination.
    pub(super) events: &'a WorkerEventSink,
}

/// Free Cell A* problem adapter over `xsar::games::freecell::rules`.
struct FreeCellProblem<'a> {
    /// Immutable validated starting 52-card deal.
    deal: &'a InitialDeal,
}


impl SearchProblem for FreeCellProblem<'_> {
    type State = Position;
    type Action = RecordedMove;
    type Successors = Successors;

    fn initial(&self) -> Self::State {
        Position::initial(self.deal)
    }

    fn is_goal(&self, state: Self::State) -> bool {
        state.is_goal()
    }

    fn successors(&self, state: Self::State) -> Self::Successors {
        state.successors()
    }
}


impl BestEffortProblem for FreeCellProblem<'_> {
    fn score(&self, state: Self::State) -> u32 {
        state.foundation_cards()
    }
}


/// Close forced automatic foundation transfers and independently replay-verify a Free Cell route.
fn verify_freecell_best_effort(
    deal: &InitialDeal,
    raw: BestEffortOutcome<RecordedMove>,
    cancel: &AtomicBool,
) -> (Outcome, PartialProof) {
    match raw {
        BestEffortOutcome::Solved { actions, stats, .. } => {
            let moves: Box<[RecordedMove]> = actions.into_boxed_slice();
            match xsar::games::freecell::replay::verify_with_control(deal, &moves, cancel) {
                Ok(replay) => (
                    Outcome::Solved {
                        moves,
                        replay,
                        stats,
                    },
                    PartialProof::Exhaustive,
                ),
                Err(err) if err.kind == xsar::games::freecell::replay::ReplayFailure::Cancelled => {
                    (Outcome::Cancelled(stats), PartialProof::Exhaustive)
                }
                Err(err) => (
                    Outcome::Failed {
                        failure: Failure::Replay(err),
                        stats,
                    },
                    PartialProof::Exhaustive,
                ),
            }
        }
        BestEffortOutcome::Partial {
            mut actions,
            score,
            proof,
            stats,
        } => {
            let mut pos = Position::initial(deal);
            for &mv in &actions {
                let Ok(next) = pos.apply(mv) else {
                    return (
                        Outcome::Failed {
                            failure: Failure::Engine(SearchStop::InternalInvariant),
                            stats,
                        },
                        proof,
                    );
                };
                pos = next;
            }
            // Interrupted best-effort nodes may still have pending forced-automatic foundation cards.
            while let Some(auto_mv) = pos.automatic() {
                let Ok(next) = pos.apply(auto_mv) else {
                    return (
                        Outcome::Failed {
                            failure: Failure::Engine(SearchStop::InternalInvariant),
                            stats,
                        },
                        proof,
                    );
                };
                pos = next;
                actions.push(auto_mv);
            }
            let closed_foundations = pos.foundation_cards();
            let moves: Box<[RecordedMove]> = actions.into_boxed_slice();
            if pos.is_goal() {
                return match xsar::games::freecell::replay::verify_with_control(
                    deal, &moves, cancel,
                ) {
                    Ok(replay) => (
                        Outcome::Solved {
                            moves,
                            replay,
                            stats,
                        },
                        PartialProof::Exhaustive,
                    ),
                    Err(err)
                        if err.kind == xsar::games::freecell::replay::ReplayFailure::Cancelled =>
                    {
                        (Outcome::Cancelled(stats), PartialProof::Exhaustive)
                    }
                    Err(err) => (
                        Outcome::Failed {
                            failure: Failure::Replay(err),
                            stats,
                        },
                        PartialProof::Exhaustive,
                    ),
                };
            }
            match xsar::games::freecell::replay::verify_prefix_with_control(deal, &moves, cancel) {
                Ok(summary) => {
                    let verified_cards: u32 = summary
                        .endpoint
                        .foundations()
                        .iter()
                        .copied()
                        .map(u32::from)
                        .sum();
                    if verified_cards != closed_foundations
                        || verified_cards >= 52
                        || (proof == PartialProof::Exhaustive && verified_cards != score)
                    {
                        return (
                            Outcome::Failed {
                                failure: Failure::Engine(SearchStop::InternalInvariant),
                                stats,
                            },
                            proof,
                        );
                    }
                    (
                        Outcome::Partial {
                            moves,
                            deliberate_moves: summary.deliberate,
                            automatic_moves: summary.automatic,
                            foundation_cards: verified_cards,
                            stats,
                        },
                        proof,
                    )
                }
                Err(err) if err.kind == xsar::games::freecell::replay::ReplayFailure::Cancelled => {
                    (Outcome::Cancelled(stats), proof)
                }
                Err(err) => (
                    Outcome::Failed {
                        failure: Failure::Replay(err),
                        stats,
                    },
                    proof,
                ),
            }
        }
        BestEffortOutcome::Stopped {
            reason: SearchStop::Cancelled,
            stats,
        } => (Outcome::Cancelled(stats), PartialProof::Exhaustive),
        BestEffortOutcome::Stopped { reason, stats } => (
            Outcome::Failed {
                failure: Failure::Engine(reason),
                stats,
            },
            PartialProof::Exhaustive,
        ),
    }
}


/// Execute A* best-effort search with pause checkpoints and cancellation fences,
/// returning a frozen immutable `Report` without spawning a detached thread.
pub(super) fn search_deal_with_control(
    input: SearchInput<'_>,
    mut check_active: impl FnMut() -> Result<bool, String>,
    mut on_progress: impl FnMut(SearchProgress, Duration),
) -> Result<Report, String> {
    let SearchInput {
        deal,
        path,
        bytes,
        cancel,
        events,
    } = input;
    if check_active()? {
        return Ok(Report {
            path,
            deal,
            bytes,
            outcome: Outcome::Cancelled(xsar::search::SearchStats::default()),
            partial_proof: PartialProof::Exhaustive,
            paused: Duration::ZERO,
        });
    }
    let checkpoint_error = RefCell::new(None);
    let paused_total = Cell::new(Duration::ZERO);
    let check_ref = RefCell::new(&mut check_active);
    let checkpoint = || {
        if checkpoint_error.borrow().is_some() {
            return Duration::ZERO;
        }
        let callback_cancelled = Cell::new(false);
        let check = || match (check_ref.borrow_mut())() {
            Ok(false) => Ok(()),
            Ok(true) => {
                callback_cancelled.set(true);
                Err("Free Cell search cancelled".to_owned())
            }
            Err(error) => Err(error),
        };
        let paused = match events
            .latest_frame
            .pause
            .checkpoint(pyramid_pause::Phase::Search, check)
        {
            Ok(paused) => paused,
            Err(error) => {
                *checkpoint_error.borrow_mut() = Some((error.message, callback_cancelled.get()));
                cancel.store(true, Ordering::Release);
                error.paused
            }
        };
        match paused_total.get().checked_add(paused) {
            Some(total) => paused_total.set(total),
            None => {
                *checkpoint_error.borrow_mut() =
                    Some(("Free Cell pause duration overflow".to_owned(), false));
                cancel.store(true, Ordering::Release);
            }
        }
        paused
    };
    let (search_scope, yield_best) = events.latest_frame.pause.begin_search();
    let mut progress = |snap: BestEffortProgress| {
        events
            .latest_frame
            .pause
            .update_search_best_score(snap.best_score);
        let search = SearchProgress {
            search: snap.search,
            best_foundation_cards: snap.best_score,
        };
        crate::single_step_telemetry::record(
            || serde_json::json!({"event":"calculation-progress", "progress":format!("{search:?}"), "paused_ms":paused_total.get().as_millis()}),
        );
        on_progress(search, paused_total.get());
    };
    crate::single_step_telemetry::record(
        || serde_json::json!({"event":"calculation-search-start", "objective":"complete board first; best foundations if stopped early or exhausted", "input_frozen":true, "independent_replay_required":true}),
    );
    let problem = FreeCellProblem { deal: &deal };
    let raw_outcome = xsar::search::solve_guided_unlimited_best_effort_controlled(
        &problem,
        &|pos: Position| pos.guidance(),
        BestEffortControl {
            cancel,
            yield_best: Some(&yield_best),
            progress: Some(&mut progress),
            checkpoint: Some(&checkpoint),
        },
    );
    drop(search_scope);
    let (mut outcome, partial_proof) = verify_freecell_best_effort(&deal, raw_outcome, cancel);
    if let Some((error, cancelled)) = checkpoint_error.into_inner()
        && !cancelled
    {
        return Err(error);
    }
    // A final cancellation retires even an already replayed route. Actual failures win.
    if !matches!(outcome, Outcome::Failed { .. }) {
        match (check_ref.borrow_mut())() {
            Ok(false) => {}
            Ok(true) => outcome = Outcome::Cancelled(stats(&outcome)),
            Err(error) => return Err(error),
        }
    }
    crate::single_step_telemetry::record(
        || serde_json::json!({"event":"calculation-search-finished", "outcome":match &outcome {Outcome::Solved{..}=>"solved and independently replayed", Outcome::Partial{..}=>if partial_proof == PartialProof::Exhaustive {"exhaustive maximum independently replayed"} else {"play-best partial independently replayed"}, Outcome::Failed{..}=>"failed", Outcome::Cancelled(_)=>"cancelled"}, "stats":format!("{:?}", stats(&outcome)), "route_available":matches!(&outcome,Outcome::Solved{..}|Outcome::Partial{foundation_cards:1..=51,..})}),
    );
    Ok(Report {
        path,
        deal,
        bytes,
        outcome,
        partial_proof,
        paused: paused_total.get(),
    })
}

/// Load bounded input once, compute/replay offline, and publish one ordered result.
pub(super) fn run(
    identity: Identity,
    path: &Path,
    cancel: &AtomicBool,
    events: &WorkerEventSink,
    shutdown: &AtomicBool,
) {
    let control = &events.latest_frame.freecell_calculation;
    let result = (|| {
        let active = || {
            if shutdown.load(Ordering::Acquire) {
                return Err(ControlError::Shutdown);
            }
            control.check(&identity)
        };
        active().map_err(|e| e.to_string())?;
        crate::single_step_telemetry::record(
            || serde_json::json!({"event":"calculation-source-open", "path":path.display().to_string()}),
        );
        let file = std::fs::File::open(path).map_err(|e| format!("Free Cell source open: {e}"))?;
        let mut bytes = Vec::new();
        std::io::Read::take(file, (xsar::freecell_json::MAX_DOCUMENT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("Free Cell source read: {e}"))?;
        crate::single_step_telemetry::record(
            || serde_json::json!({"event":"calculation-source-read", "bytes":bytes.len(), "maximum_bytes":xsar::freecell_json::MAX_DOCUMENT_BYTES}),
        );
        let deal = xsar::freecell_json::from_slice(&bytes)
            .map_err(|e| format!("Invalid Free Cell input: {e}"))?;
        crate::single_step_telemetry::record(
            || serde_json::json!({"event":"calculation-source-parsed", "deal_id":deal.metadata().deal_id(), "objective":crate::freecell_recon::objective::from_metadata(deal.metadata()).label()}),
        );
        crate::freecell_recon::objective::require_acquired(deal.metadata())?;
        search_deal_with_control(
            SearchInput {
                deal,
                path: path.to_owned(),
                bytes,
                cancel,
                events,
            },
            || match active() {
                Ok(()) => Ok(false),
                Err(ControlError::Cancelled) => Ok(true),
                Err(other) => Err(other.to_string()),
            },
            |search, paused| {
                control.publish(Progress {
                    identity: identity.clone(),
                    search,
                    paused,
                });
            },
        )
    })();
    complete(identity, result, events);
}


/// Return authoritative final statistics independently of the coalesced progress slot.
pub(crate) fn stats(outcome: &Outcome) -> xsar::search::SearchStats {
    match outcome {
        Outcome::Solved { stats, .. }
        | Outcome::Partial { stats, .. }
        | Outcome::Failed { stats, .. }
        | Outcome::Cancelled(stats) => *stats,
    }
}


/// Format the shared Free Cell calculation summary line for both Manual and Session modes.
#[cfg(test)]
pub(crate) fn outcome_summary_message(outcome: &Outcome) -> String {
    format_summary_message(outcome, PartialProof::Exhaustive)
}


/// Format the calculation summary distinguishing exhaustive proof from Play-interrupted routes.
pub(crate) fn report_summary_message(report: &Report) -> String {
    format_summary_message(&report.outcome, report.partial_proof)
}


/// Shared formatter for complete, exhaustive-partial and Play-interrupted Free Cell summaries.
fn format_summary_message(outcome: &Outcome, partial_proof: PartialProof) -> String {
    match outcome {
        Outcome::Solved { moves, replay, .. } => format!(
            "Complete route independently replayed: {} transfers; {} deliberate, {} automatic; 52 foundation cards",
            moves.len(),
            replay.deliberate,
            replay.automatic
        ),
        Outcome::Partial {
            moves,
            deliberate_moves,
            automatic_moves,
            foundation_cards,
            ..
        } if partial_proof != PartialProof::Exhaustive => format!(
            "Play requested — stopped search early and independently replayed best partial route: {foundation_cards}/52 foundation cards; {} transfers, {deliberate_moves} deliberate, {automatic_moves} automatic",
            moves.len()
        ),
        Outcome::Partial {
            moves,
            deliberate_moves,
            automatic_moves,
            foundation_cards,
            ..
        } => format!(
            "Proved maximum partial route: {foundation_cards}/52 foundation cards; {} transfers, {deliberate_moves} deliberate, {automatic_moves} automatic; search graph exhausted; independently replayed{}",
            moves.len(),
            if moves.is_empty() {
                "; no foundation progress is reachable"
            } else {
                ""
            }
        ),
        Outcome::Cancelled(_) => "Calculation cancelled; no usable route".to_owned(),
        Outcome::Failed { failure, .. } => format!("Calculation failed: {failure}"),
    }
}


/// Retire the exact reservation before publishing one immutable final outcome.
pub(super) fn complete(
    identity: Identity,
    result: Result<Report, String>,
    events: &WorkerEventSink,
) {
    complete_with_preparation(identity, result, events, freecell_route::Prepared::new);
}


/// Remove executable outcomes before publication without masking an original failure.
fn cancel_executable(report: &mut Arc<Report>) -> Result<(), String> {
    if matches!(
        &report.outcome,
        Outcome::Solved { .. } | Outcome::Partial { .. }
    ) {
        let counters = stats(&report.outcome);
        // Only this completion owns the unpublished report after its prepared
        // candidate is dropped; no UI/event or route permit has seen this Arc.
        Arc::get_mut(report)
            .ok_or("Free Cell report shared before cancellation retirement")?
            .outcome = Outcome::Cancelled(counters);
    }
    Ok(())
}


/// Prepare without locks, then atomically fence/cancel/retire the exact publication.
///
/// The private validator seam lets tests exercise real cancellation while both
/// mutexes are available, without timing races or changing the production path.
fn complete_with_preparation(
    identity: Identity,
    result: Result<Report, String>,
    events: &WorkerEventSink,
    prepare: impl FnOnce(
        Identity,
        Arc<Report>,
        &AtomicBool,
    ) -> Result<freecell_route::Prepared, freecell_route::PreparationError>,
) {
    let result = (|| {
        let cancel = {
            let active = events
                .latest_frame
                .freecell_calculation
                .active
                .lock()
                .map_err(|_| "Free Cell calculation retirement poisoned")?;
            let Some((owner, cancel)) = active.as_ref() else {
                return Err("Free Cell calculation owner retired".into());
            };
            if owner != &identity {
                return Err("Stale Free Cell calculation completion".into());
            }
            Arc::clone(cancel)
        };
        // Defensive replay and all route-length work use the actual request's
        // cancellation flag outside both authority locks.
        let mut prepared = None;
        let result = result.map(Arc::new).and_then(|mut report| {
            match prepare(identity.clone(), Arc::clone(&report), &cancel) {
                Ok(candidate) => prepared = Some(candidate),
                Err(freecell_route::PreparationError::Cancelled) => cancel_executable(&mut report)?,
                Err(freecell_route::PreparationError::Failed(error)) => return Err(error),
            }
            Ok(report)
        });
        // Final cancellation and O(1) candidate installation still share the
        // original reservation lock, so cancellation cannot be lost at publication.
        let mut active = events
            .latest_frame
            .freecell_calculation
            .active
            .lock()
            .map_err(|_| "Free Cell calculation retirement poisoned")?;
        let Some((owner, current_cancel)) = active.as_ref() else {
            return Err("Free Cell calculation owner retired".into());
        };
        if owner != &identity || !Arc::ptr_eq(current_cancel, &cancel) {
            return Err("Stale Free Cell calculation completion".into());
        }
        let result = result.and_then(|mut report| {
            if cancel.load(Ordering::Acquire) {
                prepared = None;
                cancel_executable(&mut report)?;
            }
            Ok(report)
        });
        *active = None;
        result.and_then(|report| {
            events
                .latest_frame
                .freecell_route
                .lock()
                .map_err(|_| "Free Cell route store poisoned")?
                .install(prepared)?;
            Ok(report)
        })
    })();
    crate::single_step_telemetry::record(
        || serde_json::json!({"event":"calculation-completion", "request_id":identity.request_id, "source_generation":identity.source_generation, "error":result.as_ref().err()}),
    );
    let _ = events.send_for_context(
        Some(identity.context.clone()),
        WorkerEvent::FreeCellCalculationFinished { identity, result },
    );
}


#[cfg(test)]
mod tests {
    use super::*;
    use xsar::games::{
        cards::{Card, Rank},
        freecell::{BoardState, CascadeStack, FOUNDATION_SUITS, InitialDeal, Metadata},
    };


    /// Legal closed prefix at the trusted domain/consumer seam, not a maximum-search proof.
    fn partial_completion_report() -> Report {
        let original = automatic_deal();
        let columns = std::array::from_fn(|column| {
            let mut cards: Vec<_> = original.board().cascades()[column].iter().collect();
            if column < 3 {
                cards.reverse();
            }
            CascadeStack::from_cards(cards).unwrap()
        });
        let deal = InitialDeal::new(
            BoardState::new(columns, [None; 4], [0; 4]).unwrap(),
            original.metadata().clone(),
        )
        .unwrap();
        let mut position = xsar::games::freecell::rules::Position::initial(&deal);
        let mut moves = Vec::new();
        while let Some(action) = position.automatic() {
            position = position.apply(action).unwrap();
            moves.push(action);
        }
        let replay = xsar::games::freecell::replay::verify_prefix_with_control(
            &deal,
            &moves,
            &AtomicBool::new(false),
        )
        .unwrap();
        assert!(
            !moves.is_empty()
                && replay
                    .endpoint
                    .foundations()
                    .iter()
                    .copied()
                    .map(u32::from)
                    .sum::<u32>()
                    < 52
        );
        Report {
            path: PathBuf::from("partial-retirement.json"),
            deal,
            bytes: Vec::new(),
            partial_proof: PartialProof::Exhaustive,
            paused: Duration::ZERO,
            outcome: Outcome::Partial {
                moves: moves.into_boxed_slice(),
                stats: Default::default(),
                deliberate_moves: replay.deliberate,
                automatic_moves: replay.automatic,
                foundation_cards: replay
                    .endpoint
                    .foundations()
                    .iter()
                    .copied()
                    .map(u32::from)
                    .sum::<u32>(),
            },
        }
    }


    /// Freeze one explicit source/context owner without any actual file or transport.
    fn completion_identity() -> Identity {
        Identity {
            context: ControllerContext::new(
                PathBuf::from("/nonexistent/315-cancel.sock"),
                GameMode::FreeCell,
                SolvingStrategy::ShortestPath,
                315,
            ),
            request_id: 315,
            source_generation: 1,
        }
    }


    /// Actual cancellation can acquire both locks before/after defensive replay and wins publication.
    #[test]
    fn freecell_315_completion_replay_is_outside_locks_and_final_cancel_wins() {
        for cancel_before_replay in [true, false] {
            let identity = completion_identity();
            let latest = LatestFrameSlot::default();
            let cancellation = latest
                .freecell_calculation
                .reserve(identity.clone())
                .unwrap();
            let (events, receiver) = mpsc::channel();
            let sink = WorkerEventSink {
                events,
                latest_frame: latest.clone(),
                capture_context: Mutex::new(None),
            };
            complete_with_preparation(
                identity.clone(),
                Ok(partial_completion_report()),
                &sink,
                |plan, report, cancel| {
                    assert!(
                        latest.freecell_calculation.active.try_lock().is_ok(),
                        "replay holds cancellation lock"
                    );
                    assert!(
                        latest.freecell_route.try_lock().is_ok(),
                        "replay holds route lock"
                    );
                    assert!(
                        std::ptr::eq(cancel, cancellation.as_ref()),
                        "wrong request cancellation token"
                    );
                    if cancel_before_replay {
                        latest.freecell_calculation.cancel(Some(&plan));
                    }
                    let prepared = freecell_route::Prepared::new(plan.clone(), report, cancel);
                    if !cancel_before_replay {
                        latest.freecell_calculation.cancel(Some(&plan));
                    }
                    prepared
                },
            );
            let WorkerEvent::FreeCellCalculationFinished { result, .. } =
                receiver.try_recv().unwrap().event
            else {
                panic!("calculation completion only")
            };
            assert!(matches!(result.unwrap().outcome, Outcome::Cancelled(_)));
            assert!(!latest.freecell_route.lock().unwrap().available(&identity));
            assert!(latest.freecell_calculation.active.lock().unwrap().is_none());
            assert!(receiver.try_recv().is_err() && latest.take().is_none());
        }
    }


    /// Even a repeated identity cannot publish through a replacement request's token.
    #[test]
    fn freecell_315_completion_rechecks_actual_reservation_after_validation() {
        let identity = completion_identity();
        let latest = LatestFrameSlot::default();
        let original = latest
            .freecell_calculation
            .reserve(identity.clone())
            .unwrap();
        let (events, receiver) = mpsc::channel();
        let sink = WorkerEventSink {
            events,
            latest_frame: latest.clone(),
            capture_context: Mutex::new(None),
        };
        complete_with_preparation(
            identity.clone(),
            Ok(partial_completion_report()),
            &sink,
            |plan, report, cancel| {
                let prepared = freecell_route::Prepared::new(plan.clone(), report, cancel)?;
                latest.freecell_calculation.finish(&plan).unwrap();
                let replacement = latest.freecell_calculation.reserve(plan).unwrap();
                assert!(!Arc::ptr_eq(&original, &replacement));
                Ok(prepared)
            },
        );
        let WorkerEvent::FreeCellCalculationFinished { result, .. } =
            receiver.try_recv().unwrap().event
        else {
            panic!("calculation completion only")
        };
        assert!(
            result
                .unwrap_err()
                .contains("Stale Free Cell calculation completion")
        );
        assert!(latest.freecell_calculation.running());
        assert!(!latest.freecell_route.lock().unwrap().available(&identity));
        latest.freecell_calculation.finish(&identity).unwrap();
    }


    /// Original loading/domain failures stay original when cancellation is also requested.
    #[test]
    fn freecell_315_completion_cancel_does_not_mask_original_failure() {
        for failed_report in [false, true] {
            let identity = completion_identity();
            let latest = LatestFrameSlot::default();
            latest
                .freecell_calculation
                .reserve(identity.clone())
                .unwrap();
            latest.freecell_calculation.cancel(Some(&identity));
            let (events, receiver) = mpsc::channel();
            let sink = WorkerEventSink {
                events,
                latest_frame: latest.clone(),
                capture_context: Mutex::new(None),
            };
            let error = "original source read failed";
            let incoming = if failed_report {
                let mut report = partial_completion_report();
                report.outcome = Outcome::Failed {
                    stats: Default::default(),
                    failure: xsar::games::freecell::search::Failure::Replay(
                        xsar::games::freecell::replay::ReplayError {
                            step: 7,
                            kind: xsar::games::freecell::replay::ReplayFailure::Illegal,
                        },
                    ),
                };
                Ok(report)
            } else {
                Err(error.to_owned())
            };
            complete(identity.clone(), incoming, &sink);
            let WorkerEvent::FreeCellCalculationFinished { result, .. } =
                receiver.try_recv().unwrap().event
            else {
                panic!("calculation completion only")
            };
            if failed_report {
                assert!(matches!(result.unwrap().outcome, Outcome::Failed { .. }));
            } else {
                assert_eq!(result.unwrap_err(), error);
            }
            assert!(!latest.freecell_route.lock().unwrap().available(&identity));
            assert!(latest.freecell_calculation.active.lock().unwrap().is_none());
        }
    }


    /// Complete synthetic initial deal whose entire route is automatic.
    fn automatic_deal() -> InitialDeal {
        let mut columns: [Vec<Card>; 8] = std::array::from_fn(|_| Vec::new());
        for (i, card) in (1..=13)
            .rev()
            .flat_map(|rank| {
                FOUNDATION_SUITS.map(|suit| Card::new(Rank::from_value(rank).unwrap(), suit))
            })
            .enumerate()
        {
            columns[i % 8].push(card);
        }
        InitialDeal::new(
            BoardState::new(
                columns.map(|c| CascadeStack::from_cards(c).unwrap()),
                [None; 4],
                [0; 4],
            )
            .unwrap(),
            Metadata::new(
                "worker-offline",
                "synthetic",
                crate::freecell_recon::objective::FULL_BOARD_SOURCE,
            )
            .unwrap(),
        )
        .unwrap()
    }


    /// Unresolved acquired sources solve offline; legacy sources retain no route or frame.
    #[test]
    fn freecell_315_offline_objective_provenance_has_no_guest_effect() {
        let original = automatic_deal();
        for (index, source) in ["unit", "single-frame-tcp-v3;objective=unresolved"]
            .into_iter()
            .enumerate()
        {
            let path = std::env::temp_dir().join(format!(
                "freecell-objective-{}-{index}.json",
                std::process::id()
            ));
            let deal = InitialDeal::new(
                original.board().clone(),
                Metadata::new("test", "synthetic", source).unwrap(),
            )
            .unwrap();
            let bytes = xsar::freecell_json::to_vec(&deal).unwrap();
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap();
            std::io::Write::write_all(&mut file, &bytes).unwrap();
            drop(file);
            let identity = Identity {
                context: ControllerContext::new(
                    PathBuf::from("/nonexistent-offline/no-qmp.sock"),
                    GameMode::FreeCell,
                    SolvingStrategy::ShortestPath,
                    1,
                ),
                request_id: index as u64 + 1,
                source_generation: 1,
            };
            let latest = LatestFrameSlot::default();
            let cancel = latest
                .freecell_calculation
                .reserve(identity.clone())
                .unwrap();
            let (sender, receiver) = mpsc::channel();
            let events = WorkerEventSink {
                events: sender,
                latest_frame: latest.clone(),
                capture_context: Mutex::new(None),
            };
            run(
                identity.clone(),
                &path,
                &cancel,
                &events,
                &AtomicBool::new(false),
            );
            let event = receiver.recv_timeout(Duration::from_secs(1)).unwrap();
            let WorkerEvent::FreeCellCalculationFinished {
                identity: actual,
                result,
            } = event.event
            else {
                panic!("only calculation completion permitted")
            };
            assert_eq!(actual, identity);
            if index == 0 {
                assert!(result.unwrap_err().contains("objective"));
                assert!(latest.freecell_calculation.take().is_none());
                assert!(!latest.freecell_route.lock().unwrap().available(&identity));
            } else {
                let report = result.unwrap();
                assert_eq!(report.bytes, bytes);
                assert_eq!(report.deal, deal);
                assert!(matches!(&report.outcome, Outcome::Solved { .. }));
                assert!(latest.freecell_route.lock().unwrap().available(&identity));
            }
            assert!(latest.take().is_none());
            assert!(receiver.try_recv().is_err());
            std::fs::remove_file(path).unwrap();
        }
    }


    /// Wait only for an actual acknowledged pause, with a bounded test timeout.
    fn paused(gate: &pyramid_pause::Gate) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if gate.snapshot().is_some_and(|s| s.acknowledged) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "worker did not acknowledge pause"
            );
            std::thread::yield_now();
        }
    }


    /// Offline pause retains frozen bytes; continue solves, cancel wakes and yields no route.
    #[test]
    fn freecell_310_worker_pause_continue_cancel_freezes_input_and_uses_no_guest() {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        for cancel_parked in [false, true] {
            let root = std::env::temp_dir().join(format!(
                "freecell-offline-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&root).unwrap();
            let path = root.join("original.json");
            let bytes = xsar::freecell_json::to_vec(&automatic_deal()).unwrap();
            std::fs::write(&path, &bytes).unwrap();
            let identity = Identity {
                context: ControllerContext::new(
                    PathBuf::from("/nonexistent-offline/no-qmp.sock"),
                    GameMode::FreeCell,
                    SolvingStrategy::ShortestPath,
                    4,
                ),
                request_id: 1,
                source_generation: 2,
            };
            let latest = LatestFrameSlot::default();
            let cancel = latest
                .freecell_calculation
                .reserve(identity.clone())
                .unwrap();
            assert!(
                latest
                    .freecell_calculation
                    .reserve(identity.clone())
                    .is_err()
            );
            let mut stale = identity.clone();
            stale.request_id = 0;
            latest.freecell_calculation.cancel(Some(&stale));
            assert!(!cancel.load(Ordering::Acquire));
            let work = latest.pause.begin(pyramid_pause::Phase::Search).unwrap();
            let epoch = latest.pause.snapshot().unwrap().epoch;
            latest.pause.request(epoch, true).unwrap();
            let (sender, receiver) = mpsc::channel();
            let worker_latest = latest.clone();
            let worker_id = identity.clone();
            let worker_path = path.clone();
            let worker = std::thread::spawn(move || {
                let events = WorkerEventSink {
                    events: sender,
                    latest_frame: worker_latest,
                    capture_context: Mutex::new(None),
                };
                run(
                    worker_id,
                    &worker_path,
                    &cancel,
                    &events,
                    &AtomicBool::new(false),
                );
                drop(work);
            });
            paused(&latest.pause);
            // Input has already been read/validated: mutation cannot alter the frozen deal.
            std::fs::write(&path, b"changed while paused").unwrap();
            assert!(receiver.try_recv().is_err());
            if cancel_parked {
                latest.freecell_calculation.cancel(Some(&identity));
                latest.pause.wake();
            } else {
                latest.pause.request(epoch, false).unwrap();
            }
            let event = receiver.recv_timeout(Duration::from_secs(10)).unwrap();
            worker.join().unwrap();
            assert_eq!(event.context, Some(identity.context.clone()));
            let WorkerEvent::FreeCellCalculationFinished {
                identity: actual,
                result,
            } = event.event
            else {
                panic!("only offline completion permitted")
            };
            assert_eq!(actual, identity);
            let report = result.unwrap();
            assert_eq!(report.bytes, bytes);
            assert!(!report.paused.is_zero());
            if cancel_parked {
                assert!(matches!(report.outcome, Outcome::Cancelled(_)));
            } else {
                let Outcome::Solved { moves, replay, .. } = &report.outcome else {
                    panic!("verified full route")
                };
                assert_eq!((moves.len(), replay.foundations), (52, [13; 4]));
            }
            assert!(receiver.try_recv().is_err());
            assert!(latest.take().is_none());
            assert!(!latest.freecell_calculation.running());
            assert!(latest.pause.request(epoch, false).is_err());
            assert!(latest.freecell_calculation.reserve(identity).is_ok());
            std::fs::remove_dir_all(root).unwrap();
        }
    }


    /// Play interrupt closes trailing forced automatic foundation transfers (including 52/52 promotion).
    #[test]
    fn freecell_350_play_interrupt_closes_forced_automatic_transfers_and_promotes_to_solved() {
        let auto_deal = automatic_deal();
        let first_auto = Position::initial(&auto_deal).automatic().unwrap();
        let (promoted, proof) = verify_freecell_best_effort(
            &auto_deal,
            xsar::search::BestEffortOutcome::Partial {
                actions: vec![first_auto],
                score: 1,
                proof: PartialProof::Bounded(SearchStop::Interrupted),
                stats: xsar::search::SearchStats::default(),
            },
            &AtomicBool::new(false),
        );
        assert_eq!(proof, PartialProof::Exhaustive);
        assert!(matches!(
            promoted,
            Outcome::Solved {
                ref moves,
                replay,
                ..
            } if moves.len() == 52 && replay.foundations == [13; 4]
        ));

        // Build a deal where ascending filler puts remaining Aces at bottoms (initial automatic=None)
        // and moving top KS -> Cell(0) uncovers AC (forced auto) while burying 2C keeps < 52.
        let two_c = Card::new(
            Rank::from_value(2).unwrap(),
            xsar::games::cards::Suit::Clubs,
        );
        let ace_c = Card::new(Rank::Ace, xsar::games::cards::Suit::Clubs);
        let king_s = Card::new(Rank::King, xsar::games::cards::Suit::Spades);
        let mut columns: [Vec<Card>; 8] = std::array::from_fn(|_| Vec::new());
        let all_cards: Vec<Card> = (1..=13)
            .flat_map(|rank| {
                FOUNDATION_SUITS.map(|suit| Card::new(Rank::from_value(rank).unwrap(), suit))
            })
            .filter(|&c| c != two_c && c != ace_c && c != king_s)
            .collect();
        // Column 0 (len 7): [2C, <4 filler cards>, AC, KS]; cols 1..3 get 7 each; cols 4..7 get 6 each.
        let mut rest = all_cards.into_iter();
        columns[0].push(two_c);
        columns[0].extend(rest.by_ref().take(4));
        columns[0].push(ace_c);
        columns[0].push(king_s);
        for (col_idx, col) in columns.iter_mut().enumerate().skip(1) {
            let needed = if col_idx < 4 { 7 } else { 6 };
            col.extend(rest.by_ref().take(needed));
        }
        let partial_deal = InitialDeal::new(
            BoardState::new(
                columns.map(|c| CascadeStack::from_cards(c).unwrap()),
                [None; 4],
                [0; 4],
            )
            .unwrap(),
            auto_deal.metadata().clone(),
        )
        .unwrap();
        let initial_pos = Position::initial(&partial_deal);
        assert!(initial_pos.automatic().is_none());
        let deliberate = xsar::games::freecell::rules::RecordedMove {
            card_id: xsar::games::freecell::rules::card_id(Card::new(
                Rank::King,
                xsar::games::cards::Suit::Spades,
            )),
            from: xsar::games::freecell::rules::Pile::Cascade(0),
            to: xsar::games::freecell::rules::Pile::Cell(0),
            is_auto: false,
        };
        assert!(
            xsar::games::freecell::replay::verify_prefix_with_control(
                &partial_deal,
                &[deliberate],
                &AtomicBool::new(false),
            )
            .is_err(),
            "unclosed deliberate prefix must be rejected before automatic closure"
        );
        let (closed_partial, partial_proof) = verify_freecell_best_effort(
            &partial_deal,
            xsar::search::BestEffortOutcome::Partial {
                actions: vec![deliberate],
                score: 0,
                proof: PartialProof::Bounded(SearchStop::Interrupted),
                stats: xsar::search::SearchStats::default(),
            },
            &AtomicBool::new(false),
        );
        assert_eq!(
            partial_proof,
            PartialProof::Bounded(SearchStop::Interrupted)
        );
        assert!(matches!(
            closed_partial,
            Outcome::Partial {
                deliberate_moves: 1,
                automatic_moves: 1..,
                foundation_cards: 1..=51,
                ..
            }
        ));
    }


    /// Real FreeCell calculation hits 360s auto-pause, extends +60s on Continue, installs route on Play, and discards on Cancel.
    #[test]
    fn freecell_350_run_auto_pause_360s_continue_60s_and_play_vs_cancel() {
        let mut columns: [Vec<Card>; 8] = std::array::from_fn(|_| Vec::new());
        // Bury 2s at bottom beneath 3..K while exposing Aces on top -> immediate best_score >= 1, deep search.
        let mut rest = Vec::new();
        for suit in FOUNDATION_SUITS {
            for rank in 3..=13 {
                rest.push(Card::new(Rank::from_value(rank).unwrap(), suit));
            }
        }
        let mut rest_iter = rest.into_iter();
        for (col_idx, suit) in FOUNDATION_SUITS.into_iter().enumerate() {
            columns[col_idx].push(Card::new(Rank::from_value(2).unwrap(), suit));
            columns[col_idx].extend(rest_iter.by_ref().take(5));
            columns[col_idx].push(Card::new(Rank::Ace, suit));
        }
        for col in columns.iter_mut().skip(4) {
            col.extend(rest_iter.by_ref().take(6));
        }
        let deal = InitialDeal::new(
            BoardState::new(
                columns.map(|c| CascadeStack::from_cards(c).unwrap()),
                [None; 4],
                [0; 4],
            )
            .unwrap(),
            Metadata::new(
                "freecell-350-auto-pause",
                "synthetic",
                crate::freecell_recon::objective::FULL_BOARD_SOURCE,
            )
            .unwrap(),
        )
        .unwrap();
        let bytes = xsar::freecell_json::to_vec(&deal).unwrap();

        for (case_idx, trigger_cancel) in [false, true].into_iter().enumerate() {
            let root = std::env::temp_dir().join(format!(
                "freecell-350-step-play-{}-{case_idx}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(&root).unwrap();
            let path = root.join("deal.json");
            std::fs::write(&path, &bytes).unwrap();

            let latest = LatestFrameSlot::default();
            let identity = Identity {
                context: ControllerContext::new(
                    PathBuf::from("/nonexistent/freecell-350.sock"),
                    GameMode::FreeCell,
                    SolvingStrategy::ShortestPath,
                    350,
                ),
                request_id: 10 + u64::try_from(case_idx).unwrap(),
                source_generation: 1,
            };
            let cancel = latest
                .freecell_calculation
                .reserve(identity.clone())
                .unwrap();
            let work = latest.pause.begin(pyramid_pause::Phase::Search).unwrap();
            let (sender, receiver) = mpsc::channel();
            let worker_latest = latest.clone();
            let worker_id = identity.clone();
            let worker_path = path.clone();
            let worker = std::thread::spawn(move || {
                let events = WorkerEventSink {
                    events: sender,
                    latest_frame: worker_latest,
                    capture_context: Mutex::new(None),
                };
                run(
                    worker_id,
                    &worker_path,
                    &cancel,
                    &events,
                    &AtomicBool::new(false),
                );
                drop(work);
            });

            // Wait until best_score > 0 is reported live while searching.
            let deadline = Instant::now() + Duration::from_secs(5);
            let epoch = loop {
                if let Some(snap) = latest.pause.snapshot()
                    && snap.can_play_best()
                {
                    break snap.epoch;
                }
                assert!(
                    Instant::now() < deadline,
                    "FreeCell search did not reach positive score"
                );
                std::thread::sleep(Duration::from_millis(5));
            };

            // Advance active clock to 360s -> must auto-pause cleanly.
            latest
                .pause
                .advance_search_clock_for_test(Duration::from_secs(360));
            let deadline = Instant::now() + Duration::from_secs(5);
            let paused_active = loop {
                if let Some(snap) = latest.pause.snapshot()
                    && snap.is_auto_paused()
                {
                    break snap.search.unwrap().active_elapsed;
                }
                assert!(
                    Instant::now() < deadline,
                    "FreeCell search did not auto-pause at 360s"
                );
                std::thread::sleep(Duration::from_millis(5));
            };

            // Continue adds +60s.
            latest.pause.request(epoch, false).unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                if let Some(snap) = latest.pause.snapshot()
                    && !snap.requested
                    && let Some(search) = snap.search
                    && search.next_auto_pause_at >= paused_active + Duration::from_secs(60)
                {
                    break;
                }
                assert!(Instant::now() < deadline, "Continue did not extend by +60s");
                std::thread::sleep(Duration::from_millis(5));
            }

            // Advance another +60s -> second auto-pause.
            latest
                .pause
                .advance_search_clock_for_test(Duration::from_secs(60));
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                if let Some(snap) = latest.pause.snapshot()
                    && snap.is_auto_paused()
                {
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "FreeCell search did not auto-pause on +60s step"
                );
                std::thread::sleep(Duration::from_millis(5));
            }

            if trigger_cancel {
                latest.freecell_calculation.cancel(Some(&identity));
                latest.pause.wake();
            } else {
                latest.pause.request_play_best(epoch).unwrap();
            }

            let event = receiver.recv_timeout(Duration::from_secs(10)).unwrap();
            worker.join().unwrap();
            let WorkerEvent::FreeCellCalculationFinished { result, .. } = event.event else {
                panic!("expected FreeCellCalculationFinished");
            };
            let report = result.unwrap();
            if trigger_cancel {
                assert!(matches!(report.outcome, Outcome::Cancelled(_)));
                assert!(!latest.freecell_route.lock().unwrap().available(&identity));
            } else {
                assert_eq!(
                    report.partial_proof,
                    PartialProof::Bounded(SearchStop::Interrupted)
                );
                assert!(matches!(
                    report.outcome,
                    Outcome::Partial {
                        foundation_cards: 1..=51,
                        ..
                    }
                ));
                assert!(latest.freecell_route.lock().unwrap().available(&identity));
            }
            std::fs::remove_dir_all(root).unwrap();
        }
    }
}
