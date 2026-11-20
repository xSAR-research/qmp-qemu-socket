#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Socket-free TriPeaks offline A* / best-effort route calculation and preparation.
//!
//! Reads bounded `52`-slot [`TriPeaksDealDocument`] inputs, plans routes through
//! [`crate::tripeaks_model::plan_deal_route_with_control`] (including multi-board `ClearBoard`
//! escalation for oversized Challenge quotas/scores), and installs only independently replayed
//! candidates into [`super::tripeaks_route::Store`] under exact request ownership.

use std::{
    cell::{Cell, RefCell},
    io::Read,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use xsar::search::{PartialProof, SearchLimits, SearchProgress, SearchStats, SearchStop};

use super::{WorkerEvent, WorkerEventSink, pyramid_pause, tripeaks_route};
use crate::{
    recon::Objective,
    strategy::ControllerContext,
    tripeaks_model::{
        DEFAULT_SEARCH_LIMITS, MAX_DOCUMENT_BYTES, ReplayReport, TriPeaksDealDocument,
        TriPeaksMove, TriPeaksPlanOutcome, from_slice, plan_deal_route_with_control,
    },
    tripeaks_workflow_model::format_tripeaks_objective,
};

/// Correlation token tying one UI calculation request to its exported JSON source generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Identity {
    /// Active controller context (socket, `TriPeaks`, `ShortestPath`, generation).
    pub context: ControllerContext,
    /// Monotonic calculation request identifier.
    pub request_id: u64,
    /// Source selection generation so browsing invalidates prior candidates.
    pub source_generation: u64,
}

/// Latest-only advisory search progress snapshot published to the UI during A* planning.
#[derive(Clone, Debug)]
pub(crate) struct Progress {
    /// Request identity owning the running search.
    pub identity: Identity,
    /// Underlying generic `xsar::search` expansion/depth snapshot.
    pub search: SearchProgress,
    /// Cumulative operator-paused time excluded from active search work.
    pub paused: Duration,
}

/// Determined classification produced by [`search_deal_with_control`].
#[derive(Clone, Debug)]
pub(crate) enum Outcome {
    /// Replay-verified complete solution for the target or escalated `ClearBoard` objective.
    Solved {
        /// Original target [`Objective`] requested for this board.
        objective: Objective,
        /// Replay-verified TriPeaks move sequence executable from `T = 0`.
        moves: Vec<TriPeaksMove>,
        /// Verified replay state reached after `moves`.
        replay: ReplayReport,
        /// Aggregate search counters across all solver passes.
        stats: SearchStats,
        /// `true` when direct quota/score exceeded one deck and escalated to full `28`-card clear.
        escalated_clear_board: bool,
    },
    /// Partial fallback produced when neither the target nor full `ClearBoard` solved cleanly;
    /// carries proof status (`Exhaustive` vs `Bounded`), remaining `Unknown` slot count, and any
    /// best banked partial route (`moves` may be empty when zero objective gain exists).
    Partial {
        /// Original target [`Objective`] requested for this board.
        objective: Objective,
        /// Replay-verified partial move prefix (`empty` for `ClearBoard` or `0`-gain dead ends).
        moves: Vec<TriPeaksMove>,
        /// Verified final state of `moves`.
        replay: ReplayReport,
        /// Best tableau cards cleared on the known deal (`0..=28`).
        max_tableau_cleared: u8,
        /// Target-rank cards cleared along `moves` when `objective` is `ClearRankQuota`, else `0`.
        banked_rank_count: u8,
        /// Whether partial-progress search exhausted the known subgraph or stopped at a bound.
        proof: PartialProof,
        /// Count of slots in the evaluated document still marked [`crate::recon::CardSlot::Unknown`].
        unknown_slots: usize,
        /// Aggregate search counters across all solver passes.
        stats: SearchStats,
    },
    /// Search halted cleanly under operator cancellation (`STOP` / `Cancel`).
    Cancelled(SearchStats),
    /// Search stopped before proof due to budget exhaustion or internal model mismatch.
    Failed {
        /// Human-readable cause without granting execution authority.
        failure: String,
        /// Aggregate search counters completed prior to stopping.
        stats: SearchStats,
    },
}

/// Frozen input document and determined [`Outcome`] emitted once on calculation completion.
#[derive(Clone, Debug)]
pub(crate) struct Report {
    /// Source JSON file path used for this calculation.
    pub path: PathBuf,
    /// Decoded and validated `52`-slot TriPeaks deal document.
    pub document: TriPeaksDealDocument,
    /// Bounded raw JSON bytes read from `path`.
    pub bytes: Vec<u8>,
    /// Determined search and independent-replay result.
    pub outcome: Outcome,
    /// Total cooperative pause duration accumulated while calculating.
    pub paused: Duration,
}

/// Exact owner-check failures returned by [`Control`] without widening lock scope.
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
enum ControlError {
    /// A calculation reservation is already active.
    #[error("TriPeaks offline calculation is already running")]
    Busy,
    /// The worker received shutdown while planning was queued/running.
    #[error("TriPeaks calculation ended by application shutdown")]
    Shutdown,
    /// Operator cancellation was asserted for the reserved job.
    #[error("TriPeaks calculation cancelled")]
    Cancelled,
    /// Completion/progress identity no longer owns the active reservation.
    #[error("TriPeaks calculation authority retired")]
    Retired,
    /// Reservation mutex poisoned; restart calculation cleanly.
    #[error("TriPeaks calculation lock poisoned")]
    Poisoned,
}

/// Bounded worker-side reservation and coalesced progress mailbox for TriPeaks planning.
#[derive(Default)]
pub(crate) struct Control {
    /// Currently reserved calculation owner and its cancellation flag.
    active: Mutex<Option<(Identity, Arc<AtomicBool>)>>,
    /// Coalesced advisory progress snapshot readable from the UI thread.
    progress: Mutex<Option<Progress>>,
}

impl Control {
    /// Reserve exclusive ownership for `identity` without holding locks across search.
    pub(super) fn reserve(&self, identity: Identity) -> Result<Arc<AtomicBool>, String> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| ControlError::Poisoned.to_string())?;
        if active.is_some() {
            return Err(ControlError::Busy.to_string());
        }
        let cancel = Arc::new(AtomicBool::new(false));
        *active = Some((identity, Arc::clone(&cancel)));
        if let Ok(mut progress) = self.progress.lock() {
            *progress = None;
        }
        Ok(cancel)
    }

    /// Check whether `identity` still holds uncancelled ownership.
    fn check(&self, identity: &Identity) -> Result<(), ControlError> {
        let active = self.active.lock().map_err(|_| ControlError::Poisoned)?;
        let Some((current, cancel)) = active.as_ref() else {
            return Err(ControlError::Retired);
        };
        if current != identity {
            return Err(ControlError::Retired);
        }
        if cancel.load(Ordering::Acquire) {
            return Err(ControlError::Cancelled);
        }
        Ok(())
    }

    /// Assert cooperative cancellation on the active reservation (or matching `identity`).
    pub(super) fn cancel(&self, identity: Option<&Identity>) {
        if let Ok(active) = self.active.lock()
            && let Some((actual, cancel)) = active.as_ref()
            && identity.is_none_or(|expected| expected == actual)
        {
            cancel.store(true, Ordering::Release);
        }
    }

    /// Return `true` while an uncancelled TriPeaks calculation owns this control.
    pub(super) fn running(&self) -> bool {
        self.active
            .lock()
            .ok()
            .is_some_and(|a| a.as_ref().is_some_and(|(_, c)| !c.load(Ordering::Acquire)))
    }

    /// Retire only this exact `identity` reservation (e.g. if dispatch failed before worker start).
    pub(super) fn finish(&self, identity: &Identity) -> Result<(), String> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| "TriPeaks calculation retirement poisoned")?;
        if active
            .as_ref()
            .is_some_and(|(actual, _)| actual == identity)
        {
            *active = None;
        }
        Ok(())
    }

    /// Replace coalesced progress only if `progress.identity` still owns the reservation.
    fn publish(&self, progress: Progress) {
        if self.check(&progress.identity).is_ok()
            && let Ok(mut slot) = self.progress.lock()
        {
            *slot = Some(progress);
        }
    }

    /// Consume the latest advisory snapshot for UI display.
    pub(super) fn take(&self) -> Option<Progress> {
        self.progress.lock().ok()?.take()
    }
}

/// Validated TriPeaks deal input and cancellation controls passed to [`search_deal_with_control`].
pub(super) struct SearchInput<'a> {
    /// Decoded `52`-slot TriPeaks deal document.
    pub document: TriPeaksDealDocument,
    /// Optional explicit objective overriding `document.objective` (used by multi-board sessions).
    pub objective_override: Option<Objective>,
    /// Origin JSON path preserved in the final immutable [`Report`].
    pub path: PathBuf,
    /// Bounded raw JSON bytes retained in the final [`Report`].
    pub bytes: Vec<u8>,
    /// Cooperative cancellation flag checked inside A* search and pause checkpoints.
    pub cancel: &'a AtomicBool,
    /// Event sink hosting pause coordination.
    pub events: &'a WorkerEventSink,
    /// Bounded resource limits forwarded to [`plan_deal_route_with_control`].
    pub search_limits: SearchLimits,
}

/// Execute bounded TriPeaks A* / escalation search with pause checkpoints and cancellation fences.
///
/// Never opens QMP or capture connections.
pub(super) fn search_deal_with_control(
    input: SearchInput<'_>,
    mut check_active: impl FnMut() -> Result<bool, String>,
    mut on_progress: impl FnMut(SearchProgress, Duration),
) -> Result<Report, String> {
    let SearchInput {
        document,
        objective_override,
        path,
        bytes,
        cancel,
        events,
        search_limits,
    } = input;
    if check_active()? {
        return Ok(Report {
            path,
            document,
            bytes,
            outcome: Outcome::Cancelled(SearchStats::default()),
            paused: Duration::ZERO,
        });
    }
    let target_objective = objective_override.unwrap_or(document.objective);
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
                Err("TriPeaks search cancelled".to_owned())
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
                    Some(("TriPeaks pause duration overflow".to_owned(), false));
                cancel.store(true, Ordering::Release);
            }
        }
        paused
    };
    let mut progress = |search| {
        on_progress(search, paused_total.get());
    };

    let plan = plan_deal_route_with_control(
        &document.slots,
        target_objective,
        search_limits,
        cancel,
        Some(&mut progress),
        Some(&checkpoint),
    )
    .map_err(|err| format!("TriPeaks planning error: {err}"))?;

    if let Some((error, cancelled)) = checkpoint_error.into_inner()
        && !cancelled
    {
        return Err(error);
    }

    let mut outcome = match plan {
        TriPeaksPlanOutcome::SolvedObjective {
            actions,
            replay,
            stats,
        } => Outcome::Solved {
            objective: target_objective,
            moves: actions,
            replay,
            stats,
            escalated_clear_board: false,
        },
        TriPeaksPlanOutcome::EscalatedClearBoard {
            actions,
            replay,
            stats,
        } => Outcome::Solved {
            objective: target_objective,
            moves: actions,
            replay,
            stats,
            escalated_clear_board: true,
        },
        TriPeaksPlanOutcome::UnsolvablePartial {
            actions,
            replay,
            max_tableau_cleared,
            proof,
            unknown_slots,
            stats,
        } => {
            let banked_rank_count = match target_objective {
                Objective::ClearRankQuota { rank, .. } => replay
                    .final_position
                    .cleared_by_rank
                    .get(usize::from(rank))
                    .copied()
                    .unwrap_or(0),
                Objective::ClearBoard | Objective::ReachScoreDelta { .. } => 0,
            };
            Outcome::Partial {
                objective: target_objective,
                moves: actions,
                replay,
                max_tableau_cleared,
                banked_rank_count,
                proof,
                unknown_slots,
                stats,
            }
        }
        TriPeaksPlanOutcome::Stopped { reason, stats } => {
            if reason == SearchStop::Cancelled
                || (cancel.load(Ordering::Acquire)
                    && !matches!(
                        reason,
                        SearchStop::AllocationFailure | SearchStop::InternalInvariant
                    ))
            {
                Outcome::Cancelled(stats)
            } else {
                Outcome::Failed {
                    failure: format!("Bounded TriPeaks search stopped before proof ({reason})"),
                    stats,
                }
            }
        }
    };

    // Final cancellation retires an unconsumed route; genuine failures remain fatal.
    if !matches!(outcome, Outcome::Failed { .. }) {
        match (check_ref.borrow_mut())() {
            Ok(false) => {}
            Ok(true) => outcome = Outcome::Cancelled(stats(&outcome)),
            Err(error) => return Err(error),
        }
    }

    Ok(Report {
        path,
        document,
        bytes,
        outcome,
        paused: paused_total.get(),
    })
}

/// Load a bounded JSON file once, run offline planning, and emit one ordered completion event.
pub(super) fn run(
    identity: Identity,
    path: &Path,
    cancel: &AtomicBool,
    events: &WorkerEventSink,
    shutdown: &AtomicBool,
) {
    let control = &events.latest_frame.tripeaks_calculation;
    let result = (|| {
        let active = || {
            if shutdown.load(Ordering::Acquire) {
                return Err(ControlError::Shutdown);
            }
            control.check(&identity)
        };
        active().map_err(|e| e.to_string())?;
        let file = std::fs::File::open(path).map_err(|e| format!("TriPeaks source open: {e}"))?;
        let mut bytes = Vec::new();
        file.take((MAX_DOCUMENT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("TriPeaks source read: {e}"))?;
        if bytes.len() > MAX_DOCUMENT_BYTES {
            return Err(format!(
                "TriPeaks source exceeds {} bytes",
                MAX_DOCUMENT_BYTES
            ));
        }
        let document =
            from_slice(&bytes).map_err(|e| format!("Invalid TriPeaks deal JSON: {e}"))?;
        search_deal_with_control(
            SearchInput {
                document,
                objective_override: None,
                path: path.to_owned(),
                bytes,
                cancel,
                events,
                search_limits: DEFAULT_SEARCH_LIMITS,
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

/// Return aggregate search statistics across every [`Outcome`] variant.
pub(crate) fn stats(outcome: &Outcome) -> SearchStats {
    match outcome {
        Outcome::Solved { stats, .. }
        | Outcome::Partial { stats, .. }
        | Outcome::Failed { stats, .. }
        | Outcome::Cancelled(stats) => *stats,
    }
}

/// Format a plain-English calculation summary shared by Session and Manual panels.
pub(crate) fn outcome_summary_message(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Solved {
            objective,
            moves,
            replay,
            escalated_clear_board: false,
            ..
        } => {
            let cleared = crate::tripeaks_model::TABLEAU_SLOT_COUNT
                .saturating_sub(replay.final_position.tableau_mask.count_ones() as usize);
            format!(
                "Route independently replayed for {}: {} steps, {cleared}/28 tableau cleared, +{} pts",
                format_tripeaks_objective(*objective, None),
                moves.len(),
                replay.final_position.score_delta,
            )
        }
        Outcome::Solved {
            objective,
            moves,
            replay,
            escalated_clear_board: true,
            ..
        } => format!(
            "Target ({}) exceeds one deck; escalated to full 28-card board clear ({} steps, +{} pts) to advance to the next deal",
            format_tripeaks_objective(*objective, None),
            moves.len(),
            replay.final_position.score_delta,
        ),
        Outcome::Partial {
            objective,
            moves,
            replay,
            max_tableau_cleared,
            banked_rank_count,
            proof,
            unknown_slots,
            ..
        } => {
            let target_label = format_tripeaks_objective(*objective, None);
            let scope_clause = match (unknown_slots, proof) {
                (0, PartialProof::Exhaustive) => format!(
                    "Proved deal unsolvable for {target_label} (max {max_tableau_cleared}/28 tableau clearable)"
                ),
                (0, PartialProof::Bounded(reason)) => format!(
                    "No complete route found for {target_label}; best-found {max_tableau_cleared}/28 tableau clearable (search stopped at {reason}; maximum unproved)"
                ),
                (unknown, PartialProof::Exhaustive) => format!(
                    "Known-cards search did not solve {target_label} ({unknown} Unknown slots remain; max {max_tableau_cleared}/28 clear over known cards)"
                ),
                (unknown, PartialProof::Bounded(reason)) => format!(
                    "Known-cards search did not solve {target_label} ({unknown} Unknown slots remain; best-found {max_tableau_cleared}/28 clear; search stopped at {reason})"
                ),
            };
            if moves.is_empty() {
                format!("{scope_clause}; zero post-recon moves queued")
            } else {
                let route_verb = if *unknown_slots == 0 && *proof == PartialProof::Exhaustive {
                    "banked partial route"
                } else {
                    "best-found partial route"
                };
                format!(
                    "{scope_clause}; {route_verb}: {} steps, +{} pts, {banked_rank_count} target-rank cards",
                    moves.len(),
                    replay.final_position.score_delta,
                )
            }
        }
        Outcome::Cancelled(_) => "Calculation cancelled; no usable route".to_owned(),
        Outcome::Failed { failure, .. } => format!("Calculation failed: {failure}"),
    }
}

/// Prepare any verified route outside locks, check final cancellation, and publish one event.
pub(super) fn complete(
    identity: Identity,
    result: Result<Report, String>,
    events: &WorkerEventSink,
) {
    complete_with_preparation(identity, result, events, tripeaks_route::Prepared::new);
}

/// Remove executable routes before retiring a cancelled calculation.
fn cancel_executable(report: &mut Arc<Report>) -> Result<(), String> {
    if matches!(
        &report.outcome,
        Outcome::Solved { .. } | Outcome::Partial { .. }
    ) {
        let counters = stats(&report.outcome);
        Arc::get_mut(report)
            .ok_or("TriPeaks report shared before cancellation retirement")?
            .outcome = Outcome::Cancelled(counters);
    }
    Ok(())
}

/// Seam separating defensive route replay (unlocked) from exact owner retirement and store install.
fn complete_with_preparation(
    identity: Identity,
    result: Result<Report, String>,
    events: &WorkerEventSink,
    prepare: impl FnOnce(
        Identity,
        Arc<Report>,
        &AtomicBool,
    ) -> Result<tripeaks_route::Prepared, tripeaks_route::PreparationError>,
) {
    let result = (|| {
        let cancel = {
            let active = events
                .latest_frame
                .tripeaks_calculation
                .active
                .lock()
                .map_err(|_| "TriPeaks calculation retirement poisoned")?;
            let Some((owner, cancel)) = active.as_ref() else {
                return Err("TriPeaks calculation owner retired".into());
            };
            if owner != &identity {
                return Err("Stale TriPeaks calculation completion".into());
            }
            Arc::clone(cancel)
        };
        let mut prepared = None;
        let result = result.map(Arc::new).and_then(|mut report| {
            match prepare(identity.clone(), Arc::clone(&report), &cancel) {
                Ok(candidate) => prepared = Some(candidate),
                Err(tripeaks_route::PreparationError::Cancelled) => {
                    cancel_executable(&mut report)?;
                }
                Err(tripeaks_route::PreparationError::Failed(error)) => return Err(error),
            }
            Ok(report)
        });
        let mut active = events
            .latest_frame
            .tripeaks_calculation
            .active
            .lock()
            .map_err(|_| "TriPeaks calculation retirement poisoned")?;
        let Some((owner, current_cancel)) = active.as_ref() else {
            return Err("TriPeaks calculation owner retired".into());
        };
        if owner != &identity || !Arc::ptr_eq(current_cancel, &cancel) {
            return Err("Stale TriPeaks calculation completion".into());
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
                .tripeaks_route
                .lock()
                .map_err(|_| "TriPeaks route store poisoned")?
                .install(prepared)?;
            Ok(report)
        })
    })();
    let _ = events.send_for_context(
        Some(identity.context.clone()),
        WorkerEvent::TriPeaksCalculationFinished { identity, result },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;
    use std::sync::mpsc;

    use super::super::LatestFrameSlot;
    use crate::{
        game::GameMode,
        recon::{Card, CardSlot},
        strategy::SolvingStrategy,
        tripeaks_model::{
            DEAL_SLOT_COUNT, DealProvenance, INITIAL_WASTE_SLOT, STOCK_START_SLOT,
            TABLEAU_SLOT_COUNT, publish_to_path, validate_slot_table,
        },
    };

    /// Monotonic counter isolating temporary JSON paths across parallel test threads.
    static NEXT_CALC_TEST_ID: AtomicU64 = AtomicU64::new(1);

    /// Construct a sequential `52`-card cyclic solvable TriPeaks deal (`Waste = AC`).
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
        let slots = cards.map(CardSlot::Known);
        assert!(validate_slot_table(&slots).is_ok());
        slots
    }

    /// Construct a deal where Row 4 has one reachable Four (`slot 18` matching Waste Five) and no
    /// further legal tableau/Stock chains exist, making `ClearBoard` prove empty partial vs `ClearRankQuota` banking 1.
    fn dead_end_with_one_four_slots() -> [CardSlot; DEAL_SLOT_COUNT] {
        let mut used = [false; 52];
        let mut suit_cursor = [0_u8; 13];
        let mut take = |rank_idx: u8| -> Card {
            let suit = suit_cursor[usize::from(rank_idx)];
            suit_cursor[usize::from(rank_idx)] = suit + 1;
            let card = Card(suit * 13 + rank_idx);
            used[usize::from(card.0)] = true;
            card
        };
        let mut cards = [Card(0); DEAL_SLOT_COUNT];
        // Initial Waste = Five (rank 4).
        cards[usize::from(INITIAL_WASTE_SLOT.0)] = take(4);
        // Row 4 col 1 (`slot 18`) = Four (rank 3) -> only legal tap at T=0; `slot 19` stays so `slot 9` stays covered.
        cards[18] = take(3);
        // Remaining 9 Row 4 slots (`19..28`) stay strictly in `{Ace(0), Two(1), King(12)}`.
        for (slot, r) in (19..TABLEAU_SLOT_COUNT).zip([0_u8, 0, 0, 0, 1, 1, 1, 12, 12]) {
            cards[slot] = take(r);
        }
        // Hidden Rows 1..3 (`0..18`, 18 cards) absorb `{Five x3, Four x3, Two x1, King x2, Three x4, Queen x4, Jack x1}`
        // so only `{Six..Ten x4, Jack x3}` (`23` cards) remain for Stock — zero rank adjacency to `{Ace, Two, Four, King}`!
        let hidden = [
            4_u8, 4, 4, 3, 3, 3, 1, 12, 12, 2, 2, 2, 2, 11, 11, 11, 11, 10,
        ];
        for (slot, r) in (0..18).zip(hidden) {
            cards[slot] = take(r);
        }
        let mut stock_idx = usize::from(STOCK_START_SLOT);
        for (id, &is_used) in used.iter().enumerate() {
            if !is_used {
                cards[stock_idx] = Card(id as u8);
                stock_idx += 1;
            }
        }
        assert_eq!(stock_idx, DEAL_SLOT_COUNT);
        let slots = cards.map(CardSlot::Known);
        assert!(validate_slot_table(&slots).is_ok());
        slots
    }

    /// Create an isolated `WorkerEventSink`, identity, and unique temporary JSON path.
    fn test_context_and_sink(tag: &str) -> (Identity, WorkerEventSink, PathBuf) {
        let id = NEXT_CALC_TEST_ID.fetch_add(1, Ordering::Relaxed);
        let context = ControllerContext::new(
            PathBuf::from(format!("/tmp/tripeaks-calc-{tag}-{id}.sock")),
            GameMode::TriPeaks,
            SolvingStrategy::ShortestPath,
            id,
        );
        let identity = Identity {
            context,
            request_id: id,
            source_generation: id,
        };
        let (tx, _rx) = mpsc::channel();
        let sink = WorkerEventSink {
            events: tx,
            latest_frame: LatestFrameSlot::default(),
            capture_context: Mutex::new(None),
        };
        let path = std::env::temp_dir().join(format!(
            "tripeaks-calc-{tag}-{}-{id}.json",
            std::process::id()
        ));
        (identity, sink, path)
    }

    /// Solving a bounded JSON file prepares both route playback and `UNDO ALL` with clean plain copy.
    #[test]
    fn loads_bounded_json_solves_direct_and_prepares_route() {
        let (identity, sink, path) = test_context_and_sink("direct");
        let doc = TriPeaksDealDocument {
            provenance: DealProvenance::RestoredAtRoot,
            probes_used: 2,
            objective: Objective::ClearBoard,
            slots: cyclic_solvable_slots(),
            planned_route: None,
        };
        publish_to_path(&doc, &path).expect("write test deal JSON");

        let cancel = sink
            .latest_frame
            .tripeaks_calculation
            .reserve(identity.clone())
            .expect("reserve calculation");
        let shutdown = AtomicBool::new(false);
        run(identity.clone(), &path, &cancel, &sink, &shutdown);
        let _ = std::fs::remove_file(&path);

        assert!(
            sink.latest_frame
                .tripeaks_route
                .lock()
                .expect("lock")
                .available(&identity)
        );
        assert!(
            sink.latest_frame
                .tripeaks_route
                .lock()
                .expect("lock")
                .undo_available(&identity)
        );
        let summary = outcome_summary_message(&Outcome::Solved {
            objective: Objective::ClearBoard,
            moves: Vec::new(),
            replay: crate::tripeaks_model::verify_move_sequence(&doc.slots, &[])
                .expect("empty replay"),
            stats: SearchStats::default(),
            escalated_clear_board: false,
        });
        let banned_bytes = [0x65_u8, 0x70, 0x69, 0x73, 0x74, 0x65, 0x6d, 0x69, 0x63];
        let banned_word = std::str::from_utf8(&banned_bytes).expect("ascii");
        assert!(!summary.to_ascii_lowercase().contains(banned_word));
    }

    /// Oversized Challenge quotas/scores escalate automatically to `ClearBoard` (`escalated_clear_board = true`).
    #[test]
    fn escalates_oversized_challenge_quota_and_score_to_clear_board() {
        for objective in [
            Objective::ClearRankQuota {
                rank: 0,
                remaining: 6,
            },
            Objective::ReachScoreDelta {
                points_needed: 1_000_000,
            },
        ] {
            let (identity, sink, path) = test_context_and_sink("escalate");
            let doc = TriPeaksDealDocument {
                provenance: DealProvenance::RestoredAtRoot,
                probes_used: 0,
                objective,
                slots: cyclic_solvable_slots(),
                planned_route: None,
            };
            publish_to_path(&doc, &path).expect("publish deal");
            let cancel = sink
                .latest_frame
                .tripeaks_calculation
                .reserve(identity.clone())
                .expect("reserve");
            run(
                identity.clone(),
                &path,
                &cancel,
                &sink,
                &AtomicBool::new(false),
            );
            let _ = std::fs::remove_file(&path);

            let (tx, rx) = mpsc::channel();
            // Run `search_deal_with_control` directly to inspect returned `Report::outcome`.
            let direct = search_deal_with_control(
                SearchInput {
                    document: doc,
                    objective_override: None,
                    path: PathBuf::from("escalate.json"),
                    bytes: Vec::new(),
                    cancel: &AtomicBool::new(false),
                    events: &WorkerEventSink {
                        events: tx,
                        latest_frame: LatestFrameSlot::default(),
                        capture_context: Mutex::new(None),
                    },
                    search_limits: DEFAULT_SEARCH_LIMITS,
                },
                || Ok(false),
                |_, _| {},
            )
            .expect("search completes");
            drop(rx);
            match direct.outcome {
                Outcome::Solved {
                    escalated_clear_board: true,
                    replay,
                    moves,
                    ..
                } => {
                    assert_eq!(replay.final_position.tableau_mask, 0);
                    assert!(!moves.is_empty());
                    let msg = outcome_summary_message(&Outcome::Solved {
                        objective,
                        moves,
                        replay,
                        stats: SearchStats::default(),
                        escalated_clear_board: true,
                    });
                    assert!(msg.contains("escalated"));
                }
                other => panic!("expected EscalatedClearBoard Solved, got {other:?}"),
            }
        }
    }

    /// Unsolvable `ClearBoard` queues zero post-recon moves, while `ClearRankQuota` banks positive rank clears.
    #[test]
    fn unsolvable_partial_banks_positive_progress_vs_empty_clear_board() {
        let slots = dead_end_with_one_four_slots();
        let (_, sink, _) = test_context_and_sink("partial");

        let clear_dead = search_deal_with_control(
            SearchInput {
                document: TriPeaksDealDocument {
                    provenance: DealProvenance::RestoredAtRoot,
                    probes_used: 0,
                    objective: Objective::ClearBoard,
                    slots,
                    planned_route: None,
                },
                objective_override: None,
                path: PathBuf::from("dead-clear.json"),
                bytes: Vec::new(),
                cancel: &AtomicBool::new(false),
                events: &sink,
                search_limits: DEFAULT_SEARCH_LIMITS,
            },
            || Ok(false),
            |_, _| {},
        )
        .expect("clear dead-end finishes");
        assert!(matches!(
            clear_dead.outcome,
            Outcome::Partial {
                ref moves,
                banked_rank_count: 0,
                proof: PartialProof::Exhaustive,
                unknown_slots: 0,
                ..
            } if moves.is_empty()
        ));

        let quota_partial = search_deal_with_control(
            SearchInput {
                document: TriPeaksDealDocument {
                    provenance: DealProvenance::RestoredAtRoot,
                    probes_used: 0,
                    objective: Objective::ClearRankQuota {
                        rank: 3,
                        remaining: 2,
                    },
                    slots,
                    planned_route: None,
                },
                objective_override: None,
                path: PathBuf::from("partial-four.json"),
                bytes: Vec::new(),
                cancel: &AtomicBool::new(false),
                events: &sink,
                search_limits: DEFAULT_SEARCH_LIMITS,
            },
            || Ok(false),
            |_, _| {},
        )
        .expect("partial four finishes");
        assert!(matches!(
            quota_partial.outcome,
            Outcome::Partial {
                ref moves,
                banked_rank_count: 1,
                proof: PartialProof::Exhaustive,
                unknown_slots: 0,
                ..
            } if !moves.is_empty()
        ));
    }

    /// Cooperative cancellation and >65,536-byte files fail closed without installing routes.
    #[test]
    fn cancellation_and_oversized_file_refusals() {
        let (identity, sink, path) = test_context_and_sink("oversized");
        let oversized = vec![b' '; MAX_DOCUMENT_BYTES + 8];
        std::fs::write(&path, &oversized).expect("write oversized file");
        let cancel = sink
            .latest_frame
            .tripeaks_calculation
            .reserve(identity.clone())
            .expect("reserve");
        run(
            identity.clone(),
            &path,
            &cancel,
            &sink,
            &AtomicBool::new(false),
        );
        let _ = std::fs::remove_file(&path);
        assert!(
            !sink
                .latest_frame
                .tripeaks_route
                .lock()
                .expect("lock")
                .available(&identity)
        );

        let (id2, sink2, path2) = test_context_and_sink("cancelled");
        let doc = TriPeaksDealDocument {
            provenance: DealProvenance::RestoredAtRoot,
            probes_used: 0,
            objective: Objective::ClearBoard,
            slots: cyclic_solvable_slots(),
            planned_route: None,
        };
        publish_to_path(&doc, &path2).expect("write valid deal");
        let cancel2 = sink2
            .latest_frame
            .tripeaks_calculation
            .reserve(id2.clone())
            .expect("reserve cancel");
        cancel2.store(true, Ordering::Release);
        run(
            id2.clone(),
            &path2,
            &cancel2,
            &sink2,
            &AtomicBool::new(false),
        );
        let _ = std::fs::remove_file(&path2);
        assert!(
            !sink2
                .latest_frame
                .tripeaks_route
                .lock()
                .expect("lock")
                .available(&id2)
        );
    }

    /// Bounded stops (`TimeLimit`/`ExpansionLimit`) map to `Outcome::Failed`, and partial outcomes
    /// preserve `PartialProof` + `Unknown` slot counts cleanly in plain operator wording.
    #[test]
    fn non_cancel_stop_maps_to_failed_and_partial_summary_distinguishes_bounded_and_unknown() {
        let (_, sink, _) = test_context_and_sink("bounded-partial");
        let full_dead = dead_end_with_one_four_slots();
        let mut with_unknowns = full_dead;
        for slot in &mut with_unknowns[0..18] {
            *slot = CardSlot::Unknown;
        }

        // 1. Tight duration bound on an unsolved deal stops before proof -> `Outcome::Failed` (not Cancelled).
        let zero_limits = SearchLimits {
            max_duration: Duration::ZERO,
            ..DEFAULT_SEARCH_LIMITS
        };
        let stopped_report = search_deal_with_control(
            SearchInput {
                document: TriPeaksDealDocument {
                    provenance: DealProvenance::RestoredAtRoot,
                    probes_used: 0,
                    objective: Objective::ClearBoard,
                    slots: with_unknowns,
                    planned_route: None,
                },
                objective_override: None,
                path: PathBuf::from("stopped-time.json"),
                bytes: Vec::new(),
                cancel: &AtomicBool::new(false),
                events: &sink,
                search_limits: zero_limits,
            },
            || Ok(false),
            |_, _| {},
        )
        .expect("zero-duration search finishes with Failed report");
        assert!(matches!(
            stopped_report.outcome,
            Outcome::Failed { ref failure, .. } if failure.contains("time limit")
        ));
        let failed_summary = outcome_summary_message(&stopped_report.outcome);
        assert!(failed_summary.contains("Calculation failed:"));
        assert!(!failed_summary.to_ascii_lowercase().contains("epistemic"));

        // 2. Partial outcome with `CardSlot::Unknown` slots remaining preserves `unknown_slots == 18`
        //    and avoids "Proved deal unsolvable" phrasing.
        let unknown_report = search_deal_with_control(
            SearchInput {
                document: TriPeaksDealDocument {
                    provenance: DealProvenance::RestoredAtRoot,
                    probes_used: 1,
                    objective: Objective::ClearRankQuota {
                        rank: 3,
                        remaining: 2,
                    },
                    slots: with_unknowns,
                    planned_route: None,
                },
                objective_override: None,
                path: PathBuf::from("unknown-partial.json"),
                bytes: Vec::new(),
                cancel: &AtomicBool::new(false),
                events: &sink,
                search_limits: DEFAULT_SEARCH_LIMITS,
            },
            || Ok(false),
            |_, _| {},
        )
        .expect("unknown partial finishes");
        assert!(matches!(
            unknown_report.outcome,
            Outcome::Partial {
                unknown_slots: 18,
                proof: PartialProof::Exhaustive,
                ..
            }
        ));
        let unknown_summary = outcome_summary_message(&unknown_report.outcome);
        assert!(unknown_summary.contains("18 Unknown slots remain"));
        assert!(!unknown_summary.contains("Proved deal unsolvable"));
        assert!(!unknown_summary.to_ascii_lowercase().contains("epistemic"));

        // 3. Synthetic `PartialProof::Bounded` summary cleanly states `best-found` and `maximum unproved`.
        let bounded_summary = outcome_summary_message(&Outcome::Partial {
            objective: Objective::ClearBoard,
            moves: Vec::new(),
            replay: crate::tripeaks_model::verify_move_sequence(&full_dead, &[])
                .expect("empty replay"),
            max_tableau_cleared: 14,
            banked_rank_count: 0,
            proof: PartialProof::Bounded(SearchStop::ExpansionLimit),
            unknown_slots: 0,
            stats: SearchStats::default(),
        });
        assert!(bounded_summary.contains("best-found 14/28"));
        assert!(bounded_summary.contains("expansion limit"));
        assert!(bounded_summary.contains("maximum unproved"));
        assert!(!bounded_summary.contains("Proved deal unsolvable"));
        assert!(!bounded_summary.to_ascii_lowercase().contains("epistemic"));
    }
}
