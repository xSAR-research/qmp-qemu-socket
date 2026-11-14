//! Offline Free Cell calculation. This adapter has no capture, QMP or input APIs.
//!
//! A dedicated reservation owns cancellation; progress is advisory and coalesced.
//! Input is parsed once and every successful route is independently replayed.

use super::*;
use std::{
    cell::{Cell, RefCell},
    io::Read,
};
use xsar::games::freecell::search::{BestEffortOutcome as Outcome, Progress as SearchProgress};


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
    /// Solved is complete; Partial is independently replayed and exhaustively maximal.
    pub outcome: Outcome,
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
    let mut progress = |search| {
        crate::single_step_telemetry::record(
            || serde_json::json!({"event":"calculation-progress", "progress":format!("{search:?}"), "paused_ms":paused_total.get().as_millis()}),
        );
        on_progress(search, paused_total.get());
    };
    crate::single_step_telemetry::record(
        || serde_json::json!({"event":"calculation-search-start", "objective":"complete board first; exhaustive maximum foundations if impossible", "input_frozen":true, "independent_replay_required":true}),
    );
    let mut outcome = xsar::games::freecell::search::solve_best_effort_with_control(
        &deal,
        cancel,
        Some(&mut progress),
        Some(&checkpoint),
    );
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
        || serde_json::json!({"event":"calculation-search-finished", "outcome":match &outcome {Outcome::Solved{..}=>"solved and independently replayed", Outcome::Partial{..}=>"exhaustive maximum independently replayed", Outcome::Failed{..}=>"failed", Outcome::Cancelled(_)=>"cancelled"}, "stats":format!("{:?}", stats(&outcome)), "route_available":matches!(&outcome,Outcome::Solved{..}|Outcome::Partial{foundation_cards:1..=51,..})}),
    );
    Ok(Report {
        path,
        deal,
        bytes,
        outcome,
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
pub(crate) fn outcome_summary_message(outcome: &Outcome) -> String {
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
}
