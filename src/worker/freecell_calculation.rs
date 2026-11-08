//! Offline Free Cell calculation. This adapter has no capture, QMP or input APIs.
//!
//! A dedicated reservation owns cancellation; progress is advisory and coalesced.
//! Input is parsed once and every successful route is independently replayed.

use super::*;
use std::{
    cell::{Cell, RefCell},
    io::Read,
};
use xsar::games::freecell::search::{Outcome, Progress as SearchProgress};


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


/// Frozen source identity and typed complete/no-route result.
#[derive(Debug)]
pub(crate) struct Report {
    /// Exact pathname used, never a live acquisition receipt.
    pub path: PathBuf,
    /// Validated immutable original input, available for complete route inspection.
    pub deal: xsar::games::freecell::InitialDeal,
    /// Original loaded bytes, bounded by the codec and never reread during search.
    pub bytes: Vec<u8>,
    /// Only Solved contains a replay-verified complete route.
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
        let file = std::fs::File::open(path).map_err(|e| format!("Free Cell source open: {e}"))?;
        let mut bytes = Vec::new();
        std::io::Read::take(file, (xsar::freecell_json::MAX_DOCUMENT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("Free Cell source read: {e}"))?;
        let deal = xsar::freecell_json::from_slice(&bytes)
            .map_err(|e| format!("Invalid Free Cell input: {e}"))?;
        active().map_err(|e| e.to_string())?;
        let checkpoint_error = RefCell::new(None);
        let paused_total = Cell::new(Duration::ZERO);
        let callback_cause = Cell::new(None);
        let checkpoint = || {
            if checkpoint_error.borrow().is_some() {
                return Duration::ZERO;
            }
            callback_cause.set(None);
            let check = || {
                active().map_err(|cause| {
                    callback_cause.set(Some(cause));
                    cause.to_string()
                })
            };
            let paused = match events
                .latest_frame
                .pause
                .checkpoint(pyramid_pause::Phase::Search, check)
            {
                Ok(paused) => paused,
                Err(error) => {
                    let cancelled = callback_cause.get() == Some(ControlError::Cancelled);
                    *checkpoint_error.borrow_mut() = Some((error.message, cancelled));
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
            control.publish(Progress {
                identity: identity.clone(),
                search,
                paused: paused_total.get(),
            })
        };
        let mut outcome = xsar::games::freecell::search::solve_with_control(
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
            match active() {
                Ok(()) => {}
                Err(ControlError::Cancelled) => outcome = Outcome::Cancelled(stats(&outcome)),
                Err(error) => return Err(error.to_string()),
            }
        }
        Ok(Report {
            path: path.to_owned(),
            deal,
            bytes,
            outcome,
            paused: paused_total.get(),
        })
    })();
    complete(identity, result, events);
}


/// Return authoritative final statistics independently of the coalesced progress slot.
pub(crate) fn stats(outcome: &Outcome) -> xsar::search::SearchStats {
    match outcome {
        Outcome::Solved { stats, .. }
        | Outcome::Failed { stats, .. }
        | Outcome::Exhausted(stats)
        | Outcome::Cancelled(stats) => *stats,
    }
}


/// Retire the exact reservation before publishing one immutable final outcome.
pub(super) fn complete(
    identity: Identity,
    result: Result<Report, String>,
    events: &WorkerEventSink,
) {
    // Retirement and candidate retention share the calculation reservation lock.
    // Cancellation takes this lock before revoking the Store, so it cannot be lost
    // between final offline validation and candidate publication.
    let result = (|| {
        let mut active = events
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
        let result = result.map(|mut report| {
            if cancel.load(Ordering::Acquire) && matches!(&report.outcome, Outcome::Solved { .. }) {
                report.outcome = Outcome::Cancelled(stats(&report.outcome));
            }
            Arc::new(report)
        });
        *active = None;
        result.and_then(|report| {
            events
                .latest_frame
                .freecell_route
                .lock()
                .map_err(|_| "Free Cell route store poisoned")?
                .prepare(identity.clone(), Arc::clone(&report))?;
            Ok(report)
        })
    })();
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
            Metadata::new("worker-offline", "synthetic", "unit").unwrap(),
        )
        .unwrap()
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
