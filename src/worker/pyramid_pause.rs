//! Out-of-band A* suspension at explicit safe worker checkpoints.
//!
//! This gate owns presentation and intentional waiting only. It never clears
//! STOP, executes input, restarts work or holds a controller/planning lock.

use crate::parameters::{
    DEFAULT_SEARCH_AUTO_PAUSE_CONTINUE_STEP, DEFAULT_SEARCH_AUTO_PAUSE_FIRST_STEP,
};
use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};


/// Worker phase whose retained stack can acknowledge a pause.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Inventory acquisition and restoration.
    Recon,
    /// In-place exact or best-effort frontier exploration.
    Search,
    /// Whole-action computed execution and stock continuation.
    Gameplay,
}


impl Phase {
    /// Short phase name for the acknowledged UI state.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Recon => "RECON",
            Self::Search => "SEARCH",
            Self::Gameplay => "GAMEPLAY",
        }
    }
}


/// Active A* search timing, automatic time-step pause and soft-yield status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SearchSnapshot {
    /// Active calculation elapsed time so far, excluding paused wall time.
    pub(crate) active_elapsed: Duration,
    /// Next active-search elapsed threshold triggering automatic pause (`360s`, then `+60s`).
    pub(crate) next_auto_pause_at: Duration,
    /// Active calculation time increment added on Continue after an automatic pause (`60s`).
    pub(crate) continue_step: Duration,
    /// True while suspended automatically by reaching `next_auto_pause_at`.
    pub(crate) auto_paused: bool,
    /// Highest best-effort route score discovered so far in this search.
    pub(crate) best_score: u32,
    /// True once operator `Play` has been accepted for this active search.
    pub(crate) play_requested: bool,
}


/// Advisory snapshot; the epoch fences stale UI pause and continue requests.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Snapshot {
    /// Monotonic identity of one serial A* command.
    pub(crate) epoch: u64,
    /// Last actual safe checkpoint reached by the worker.
    pub(crate) phase: Phase,
    /// Operator requested suspension; acknowledgement can still be pending.
    pub(crate) requested: bool,
    /// Worker is parked at an actual safe checkpoint.
    pub(crate) acknowledged: bool,
    /// Present only while an active A* `SearchScope` is running.
    pub(crate) search: Option<SearchSnapshot>,
}


impl Snapshot {
    /// Whether this snapshot permits pressing Continue (`Pause` already acknowledged).
    pub(crate) fn is_auto_paused(self) -> bool {
        self.requested && self.acknowledged && self.search.is_some_and(|search| search.auto_paused)
    }


    /// Whether an active or paused A* search can stop now and play/install a positive best route.
    pub(crate) fn can_play_best(self) -> bool {
        self.phase == Phase::Search
            && self
                .search
                .is_some_and(|search| search.best_score > 0 && !search.play_requested)
    }
}


/// Per-search automatic pause clock and cooperative soft-yield flag.
struct SearchState {
    /// Wall-clock origin when the active board calculation entered search.
    started: Instant,
    /// Total cumulative pause time (manual and automatic) inside this search.
    paused_total: Duration,
    /// Synthetic active clock offset used only by deterministic unit tests.
    test_clock_offset: Duration,
    /// Timestamp when the current checkpoint park began, so active elapsed freezes while paused.
    parked_since: Option<Instant>,
    /// Current active-search auto-pause threshold (`360s`, extended by `+60s` on auto-Continue).
    next_auto_pause_at: Duration,
    /// Active-search increment granted on Continue from an automatic pause (`60s`).
    continue_step: Duration,
    /// True only while parked because active elapsed time reached `next_auto_pause_at`.
    auto_paused: bool,
    /// Active elapsed time recorded when automatic pause engaged.
    auto_paused_at_active: Duration,
    /// Enabled only while an A* engine loop is executing, preventing post-search auto-pause.
    auto_pause_armed: bool,
    /// Live highest positive route score recorded across examined nodes.
    best_score: u32,
    /// Shared soft-stop flag polled cooperatively by `xsar::search`.
    yield_best: Arc<AtomicBool>,
}


impl SearchState {
    /// Measure active calculation elapsed time, freezing cleanly while parked at a checkpoint.
    fn active_elapsed_at(&self, now: Instant) -> Duration {
        let effective_now = self.parked_since.unwrap_or(now);
        effective_now
            .saturating_duration_since(self.started)
            .saturating_add(self.test_clock_offset)
            .saturating_sub(self.paused_total)
    }


    /// Export an advisory snapshot for the UI thread without locks.
    fn snapshot(&self, now: Instant) -> SearchSnapshot {
        SearchSnapshot {
            active_elapsed: self.active_elapsed_at(now),
            next_auto_pause_at: self.next_auto_pause_at,
            continue_step: self.continue_step,
            auto_paused: self.auto_paused,
            best_score: self.best_score,
            play_requested: self.yield_best.load(Ordering::Acquire),
        }
    }
}


/// Scoped guard releasing active-search auto-pause and `Play` metadata on calculation exit.
pub(crate) struct SearchScope {
    /// Parent pause gate retaining the shared mutex and condvar.
    gate: Arc<Gate>,
    /// Phase active before entering search so sessions restore cleanly when search ends.
    previous_phase: Option<Phase>,
}


impl SearchScope {
    /// Arm or disarm automatic time-limit pausing without touching manual `Pause`/`Play` state.
    pub(super) fn set_auto_pause(&self, armed: bool) {
        if let Ok(mut state) = self.gate.state.lock()
            && let Some(search) = state.search.as_mut()
        {
            search.auto_pause_armed = armed;
        }
    }
}


impl Drop for SearchScope {
    /// Retire search-local auto-pause and `Play` metadata while preserving outer command phase.
    fn drop(&mut self) {
        if let Ok(mut state) = self.gate.state.lock() {
            state.search = None;
            if let Some(previous) = self.previous_phase
                && !state.aborted
                && state.phase == Some(Phase::Search)
            {
                state.phase = Some(previous);
            }
        }
        self.gate.changed.notify_all();
    }
}


/// Mutex-owned predicate; cancellation remains owned by the existing controller.
#[derive(Default)]
struct State {
    /// Never reused across serial worker commands.
    epoch: u64,
    /// Present only while a supported A* command owns this gate.
    phase: Option<Phase>,
    /// Requested suspension, independent of whether a safe point is reached.
    requested: bool,
    /// Parked worker acknowledgement, never guessed by the UI.
    acknowledged: bool,
    /// STOP or shutdown retires this epoch permanently.
    aborted: bool,
    /// Per-board active A* search timing, auto-pause step and live `Play` state.
    search: Option<SearchState>,
}


/// Shared control path independent of the serial worker command queue.
#[derive(Default)]
pub(crate) struct Gate {
    /// All pause predicates use this single mutex.
    state: Mutex<State>,
    /// Wakes suspended work on Continue, STOP or teardown.
    changed: Condvar,
}


/// Cancellation or failure together with intentional suspension already spent.
/// The caller accounts `paused` even when no continuation is permitted.
#[derive(Debug)]
pub(super) struct CheckpointError {
    /// Human-readable cancellation/context or synchronisation failure.
    pub(super) message: String,
    /// Measured intentional park time before the failure, excluding active work.
    pub(super) paused: Duration,
}


impl From<CheckpointError> for String {
    /// Preserve the existing controller error contract after its clock was accounted.
    fn from(error: CheckpointError) -> Self {
        error.message
    }
}


/// Safe checkpoint outcome; an interruption leaves the operator's PAUSE request intact.
#[derive(Debug)]
pub(super) enum Checkpoint {
    /// Work may continue after the measured intentional suspension.
    Ready(Duration),
    /// Caller should retire only its current work, accounting the same suspension.
    Interrupted(Duration),
}


impl Checkpoint {
    /// Return measured intentional suspension for the search engine's active clock.
    fn paused(self) -> Duration {
        match self {
            Self::Ready(paused) | Self::Interrupted(paused) => paused,
        }
    }
}


/// RAII owner which retires the epoch when its worker command returns.
pub(super) struct Work {
    /// Shared gate survives the UI's snapshots.
    gate: Arc<Gate>,
}


impl Gate {
    /// Reserve a fresh command epoch; poisoned/exhausted state refuses new work.
    pub(super) fn begin(self: &Arc<Self>, phase: Phase) -> Result<Work, String> {
        let mut state = self.state.lock().map_err(|_| "Pause state is poisoned")?;
        let epoch = state.epoch.checked_add(1).ok_or("Pause epoch exhausted")?;
        if state.phase.is_some() {
            return Err("Another A* command owns pause control".to_owned());
        }
        *state = State {
            epoch,
            phase: Some(phase),
            ..State::default()
        };
        Ok(Work {
            gate: Arc::clone(self),
        })
    }


    /// Read advisory state without blocking the UI on a worker transaction.
    pub(crate) fn snapshot(&self) -> Option<Snapshot> {
        let state = self.state.lock().ok()?;
        if state.aborted {
            return None;
        }
        let now = Instant::now();
        Some(Snapshot {
            epoch: state.epoch,
            phase: state.phase?,
            requested: state.requested,
            acknowledged: state.acknowledged,
            search: state.search.as_ref().map(|search| search.snapshot(now)),
        })
    }


    /// Register an active board calculation scope when `Phase::Search` owns the gate.
    pub(crate) fn begin_search(self: &Arc<Self>) -> (SearchScope, Arc<AtomicBool>) {
        self.begin_search_with_policy(
            DEFAULT_SEARCH_AUTO_PAUSE_FIRST_STEP,
            DEFAULT_SEARCH_AUTO_PAUSE_CONTINUE_STEP,
        )
    }


    /// Register an active board calculation scope with explicit auto-pause timing steps.
    pub(crate) fn begin_search_with_policy(
        self: &Arc<Self>,
        first_auto_pause: Duration,
        continue_step: Duration,
    ) -> (SearchScope, Arc<AtomicBool>) {
        let yield_best = Arc::new(AtomicBool::new(false));
        let mut previous_phase = None;
        if let Ok(mut state) = self.state.lock()
            && !state.aborted
            && state.phase.is_some()
        {
            previous_phase = state.phase;
            state.phase = Some(Phase::Search);
            state.search = Some(SearchState {
                started: Instant::now(),
                paused_total: Duration::ZERO,
                test_clock_offset: Duration::ZERO,
                parked_since: None,
                next_auto_pause_at: first_auto_pause,
                continue_step,
                auto_paused: false,
                auto_paused_at_active: Duration::ZERO,
                auto_pause_armed: true,
                best_score: 0,
                yield_best: Arc::clone(&yield_best),
            });
            self.changed.notify_all();
        }
        (
            SearchScope {
                gate: Arc::clone(self),
                previous_phase,
            },
            yield_best,
        )
    }


    /// Record the highest recoverable route score discovered in the active solver pass.
    pub(crate) fn update_search_best_score(&self, score: u32) {
        if let Ok(mut state) = self.state.lock()
            && !state.aborted
            && let Some(search) = state.search.as_mut()
        {
            search.best_score = score;
        }
    }


    /// Advance synthetic active-search elapsed time deterministically during tests.
    #[cfg(test)]
    pub(crate) fn advance_search_clock_for_test(&self, delta: Duration) {
        if let Ok(mut state) = self.state.lock()
            && let Some(search) = state.search.as_mut()
        {
            search.test_clock_offset = search.test_clock_offset.saturating_add(delta);
        }
    }


    /// Stop an active or paused search early to play/install the highest-score best route.
    pub(crate) fn request_play_best(&self, epoch: u64) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|_| "Pause state is poisoned")?;
        if state.epoch != epoch || state.phase != Some(Phase::Search) || state.aborted {
            return Err("Play request belongs to a retired A* search".to_owned());
        }
        let Some(search) = state.search.as_mut() else {
            return Err("No active A* calculation owns Play control".to_owned());
        };
        if search.best_score == 0 {
            return Err("No positive-score best route is available yet".to_owned());
        }
        search.yield_best.store(true, Ordering::Release);
        search.auto_paused = false;
        state.requested = false;
        state.acknowledged = false;
        self.changed.notify_all();
        Ok(())
    }


    /// Request pause or continue for the exact current epoch; never clear STOP.
    pub(crate) fn request(&self, epoch: u64, pause: bool) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|_| "Pause state is poisoned")?;
        if state.epoch != epoch || state.phase.is_none() || state.aborted {
            return Err("Pause request belongs to a retired A* command".to_owned());
        }
        if pause {
            state.requested = true;
        } else {
            state.requested = false;
            if let Some(search) = state.search.as_mut()
                && search.auto_paused
            {
                search.auto_paused = false;
                search.next_auto_pause_at = search
                    .auto_paused_at_active
                    .max(search.next_auto_pause_at)
                    .saturating_add(search.continue_step);
            }
        }
        self.changed.notify_all();
        Ok(())
    }


    /// Wake a parked checkpoint to inspect another out-of-band request without changing PAUSE.
    pub(super) fn wake(&self) {
        self.changed.notify_all();
    }


    /// Retire current pause authority and wake parked work; a later explicit START owns a new epoch.
    pub(super) fn abort(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.aborted = true;
            state.requested = false;
        }
        self.changed.notify_all();
    }


    /// Park only at a caller-defined safe boundary, returning intentional wait time.
    /// The cancellation/context check runs without this mutex held before and after
    /// each wait. A bounded wake also observes revocation that did not notify us.
    /// Errors never grant resume, replay or input authority.
    pub(super) fn checkpoint(
        &self,
        phase: Phase,
        check: impl Fn() -> Result<(), String>,
    ) -> Result<Duration, CheckpointError> {
        self.checkpoint_interruptible(phase, check, || Ok(false))
            .map(Checkpoint::paused)
    }


    /// Park with a separate cooperative interruption predicate; never clear PAUSE or STOP.
    /// Both callbacks run outside this mutex, with cancellation/context checked first.
    /// Interruption leaves the command epoch and requested suspension intact so a
    /// replacement engine parks until the operator explicitly presses CONTINUE.
    /// Return measured intentional suspension on readiness, interruption and failure.
    pub(super) fn checkpoint_interruptible(
        &self,
        phase: Phase,
        check: impl Fn() -> Result<(), String>,
        interrupt: impl Fn() -> Result<bool, String>,
    ) -> Result<Checkpoint, CheckpointError> {
        let mut started = None;
        let result = (|| -> Result<bool, String> {
            check()?;
            if interrupt()? {
                return Ok(true);
            }
            let mut state = self.state.lock().map_err(|_| "Pause state is poisoned")?;
            if state.phase.is_none() {
                return Ok(false);
            }
            if state.aborted {
                drop(state);
                check()?;
                return Err("A* pause authority was cancelled".to_owned());
            }
            state.phase = Some(phase);
            let checkpoint_now = Instant::now();
            let mut yield_pending = false;
            if phase == Phase::Search {
                let State {
                    requested, search, ..
                } = &mut *state;
                if let Some(search) = search.as_mut() {
                    if search.yield_best.load(Ordering::Acquire) {
                        *requested = false;
                        search.auto_paused = false;
                        yield_pending = true;
                    } else if !*requested && search.auto_pause_armed {
                        let active_elapsed = search.active_elapsed_at(checkpoint_now);
                        if active_elapsed >= search.next_auto_pause_at {
                            *requested = true;
                            search.auto_paused = true;
                            search.auto_paused_at_active = active_elapsed;
                        }
                    }
                }
            }
            if yield_pending || !state.requested {
                return Ok(false);
            }
            let wait_started = checkpoint_now;
            started = Some(wait_started);
            if let Some(search) = state.search.as_mut() {
                search.parked_since = Some(wait_started);
            }
            state.acknowledged = true;
            self.changed.notify_all();
            loop {
                drop(state);
                check()?;
                if interrupt()? {
                    return Ok(true);
                }
                state = self.state.lock().map_err(|_| "Pause state is poisoned")?;
                if state.aborted || state.phase.is_none() {
                    drop(state);
                    check()?;
                    return Err("A* pause authority was cancelled".to_owned());
                }
                if state
                    .search
                    .as_ref()
                    .is_some_and(|search| search.yield_best.load(Ordering::Acquire))
                {
                    state.requested = false;
                    break;
                }
                if !state.requested {
                    break;
                }
                state = self
                    .changed
                    .wait_timeout(state, Duration::from_millis(25))
                    .map_err(|_| "Pause wait state is poisoned")?
                    .0;
            }
            state.acknowledged = false;
            drop(state);
            check()?;
            Ok(false)
        })();
        let paused = started.map_or(Duration::ZERO, |started| started.elapsed());
        if let Ok(mut state) = self.state.lock() {
            if let Some(search) = state.search.as_mut() {
                search.paused_total = search.paused_total.saturating_add(paused);
                search.parked_since = None;
            }
            if matches!(result, Ok(true)) {
                // The retiring engine is no longer parked; its successor must acknowledge anew.
                state.acknowledged = false;
            }
        }
        result
            .map(|interrupted| {
                if interrupted {
                    Checkpoint::Interrupted(paused)
                } else {
                    Checkpoint::Ready(paused)
                }
            })
            .map_err(|message| CheckpointError { message, paused })
    }
}


impl Drop for Work {
    /// End the command's pause lifetime even on ordinary error unwinding.
    fn drop(&mut self) {
        if let Ok(mut state) = self.gate.state.lock() {
            state.phase = None;
            state.requested = false;
            state.acknowledged = false;
        }
        self.gate.changed.notify_all();
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    };


    /// Wait for a real parked acknowledgement without relying on arbitrary sleeps.
    fn parked(gate: &Gate) {
        let state = gate.state.lock().unwrap();
        let (state, timeout) = gate
            .changed
            .wait_timeout_while(state, Duration::from_secs(2), |state| !state.acknowledged)
            .unwrap();
        assert!(!timeout.timed_out());
        assert!(state.acknowledged);
    }


    /// Same retained thread returns after Continue; no callback executes while parked.
    #[test]
    fn pause_acknowledges_only_at_checkpoint_and_keeps_same_work() {
        let gate = Arc::new(Gate::default());
        let _work = gate.begin(Phase::Search).unwrap();
        let epoch = gate.snapshot().unwrap().epoch;
        gate.request(epoch, true).unwrap();
        assert!(!gate.snapshot().unwrap().acknowledged);
        let (tx, rx) = mpsc::channel();
        let worker_gate = Arc::clone(&gate);
        let worker = std::thread::spawn(move || {
            let local_frontier = vec![3, 5, 8];
            let paused = worker_gate.checkpoint(Phase::Search, || Ok(())).unwrap();
            tx.send((local_frontier, paused)).unwrap();
        });
        parked(&gate);
        assert!(rx.try_recv().is_err());
        gate.request(epoch, false).unwrap();
        let (frontier, elapsed) = rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(frontier, [3, 5, 8]);
        assert!(elapsed > Duration::ZERO);
        worker.join().unwrap();
    }


    /// STOP wakes a parked thread and Continue cannot resurrect its epoch.
    #[test]
    fn stop_wakes_pause_and_continue_never_clears_cancellation() {
        let gate = Arc::new(Gate::default());
        let _work = gate.begin(Phase::Gameplay).unwrap();
        let epoch = gate.snapshot().unwrap().epoch;
        gate.request(epoch, true).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let worker_gate = Arc::clone(&gate);
        let worker = std::thread::spawn(move || {
            worker_gate.checkpoint(Phase::Gameplay, || {
                if worker_cancel.load(Ordering::Acquire) {
                    Err("STOP".to_owned())
                } else {
                    Ok(())
                }
            })
        });
        parked(&gate);
        cancel.store(true, Ordering::Release);
        gate.abort();
        assert!(gate.request(epoch, false).is_err());
        let failure = worker.join().unwrap().unwrap_err();
        assert!(
            !failure.paused.is_zero(),
            "cancelled acknowledged suspension retains its duration"
        );
        assert!(cancel.load(Ordering::Acquire));
    }


    /// Completed work invalidates stale UI tokens even when a new phase uses the same gate.
    #[test]
    fn stale_pause_epoch_cannot_affect_next_work() {
        let gate = Arc::new(Gate::default());
        let work = gate.begin(Phase::Recon).unwrap();
        let old = gate.snapshot().unwrap().epoch;
        drop(work);
        let _next = gate.begin(Phase::Search).unwrap();
        assert!(gate.request(old, true).is_err());
        assert!(!gate.snapshot().unwrap().requested);
        assert_eq!(
            gate.checkpoint(Phase::Search, || Ok(())).unwrap(),
            Duration::ZERO
        );
    }

    /// A cancelled real search excludes the Gate's intentional park from active statistics.
    #[cfg(not(miri))]
    #[test]
    fn cancelled_search_accounts_acknowledged_pause_before_stop() {
        use std::cell::Cell;
        use xsar::search::{SearchLimits, SearchOptions, SearchOutcome, SearchProblem, SearchStop};
        struct Chain;
        impl SearchProblem for Chain {
            type State = u8;
            type Action = u8;
            type Successors = std::option::IntoIter<(u8, u8)>;
            /// Start at a non-goal state.
            fn initial(&self) -> u8 {
                0
            }
            /// The checkpoint must cancel before this goal can be reached.
            fn is_goal(&self, state: u8) -> bool {
                state == 1
            }
            /// Supply one deterministic transition with no external state.
            fn successors(&self, _: u8) -> Self::Successors {
                Some((1, 1)).into_iter()
            }
        }
        let gate = Arc::new(Gate::default());
        let _work = gate.begin(Phase::Search).unwrap();
        gate.request(gate.snapshot().unwrap().epoch, true).unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_gate = Arc::clone(&gate);
        let worker_cancel = Arc::clone(&cancelled);
        let worker = std::thread::spawn(move || {
            let intentional = Cell::new(Duration::ZERO);
            let checkpoint = || match worker_gate.checkpoint(Phase::Search, || {
                if worker_cancel.load(Ordering::Acquire) {
                    Err("STOP".to_owned())
                } else {
                    Ok(())
                }
            }) {
                Ok(paused) => paused,
                Err(error) => {
                    intentional.set(error.paused);
                    worker_cancel.store(true, Ordering::Release);
                    error.paused
                }
            };
            let started = Instant::now();
            let outcome = xsar::search::solve_with_control(
                &Chain,
                SearchLimits::default(),
                &worker_cancel,
                SearchOptions::default(),
                None,
                Some(&checkpoint),
            );
            (outcome, started.elapsed(), intentional.get())
        });
        parked(&gate);
        // Make a lost-pause regression observable independently of clock-call overhead.
        let parked_at = Instant::now();
        while parked_at.elapsed() < Duration::from_millis(30) {
            std::thread::yield_now();
        }
        cancelled.store(true, Ordering::Release);
        gate.abort();
        let (outcome, total, intentional) = worker.join().unwrap();
        let SearchOutcome::Stopped {
            reason: SearchStop::Cancelled,
            stats,
        } = outcome
        else {
            panic!("STOP must cancel the retained search");
        };
        assert!(intentional >= Duration::from_millis(30));
        assert!(
            stats.elapsed.saturating_add(intentional) <= total,
            "active search time excludes all measured intentional suspension"
        );
        assert_eq!(stats.expanded, 0);
    }


    /// Objective interruption exits a real park but retains PAUSE for replacement work.
    #[test]
    fn interrupted_checkpoint_keeps_pause_and_reports_intentional_wait() {
        let gate = Arc::new(Gate::default());
        let _work = gate.begin(Phase::Search).unwrap();
        let epoch = gate.snapshot().unwrap().epoch;
        gate.request(epoch, true).unwrap();
        let interrupt = Arc::new(AtomicBool::new(false));
        let worker_gate = Arc::clone(&gate);
        let worker_interrupt = Arc::clone(&interrupt);
        let worker = std::thread::spawn(move || {
            worker_gate.checkpoint_interruptible(
                Phase::Search,
                || Ok(()),
                || Ok(worker_interrupt.load(Ordering::Acquire)),
            )
        });
        parked(&gate);
        interrupt.store(true, Ordering::Release);
        let outcome = worker.join().unwrap().unwrap();
        let Checkpoint::Interrupted(paused) = outcome else {
            panic!("must interrupt old engine")
        };
        assert!(!paused.is_zero());
        let retained = gate.snapshot().unwrap();
        assert_eq!(retained.epoch, epoch);
        assert!(retained.requested);
        assert!(!retained.acknowledged);
        let worker_gate = Arc::clone(&gate);
        let replacement = std::thread::spawn(move || {
            worker_gate.checkpoint_interruptible(Phase::Search, || Ok(()), || Ok(false))
        });
        parked(&gate);
        assert!(gate.snapshot().unwrap().requested);
        gate.request(epoch, false).unwrap();
        assert!(matches!(
            replacement.join().unwrap().unwrap(),
            Checkpoint::Ready(_)
        ));
    }


    /// Cancellation wins over an already pending objective switch, including after a park.
    #[test]
    fn interrupted_checkpoint_checks_stop_before_switch() {
        let gate = Arc::new(Gate::default());
        let _work = gate.begin(Phase::Search).unwrap();
        gate.request(gate.snapshot().unwrap().epoch, true).unwrap();
        let stopped = Arc::new(AtomicBool::new(false));
        let worker_gate = Arc::clone(&gate);
        let worker_stop = Arc::clone(&stopped);
        let worker = std::thread::spawn(move || {
            worker_gate.checkpoint_interruptible(
                Phase::Search,
                || {
                    if worker_stop.load(Ordering::Acquire) {
                        Err("STOP".to_owned())
                    } else {
                        Ok(())
                    }
                },
                || Ok(worker_stop.load(Ordering::Acquire)),
            )
        });
        parked(&gate);
        stopped.store(true, Ordering::Release);
        gate.abort();
        let failure = worker.join().unwrap().unwrap_err();
        assert_eq!(failure.message, "STOP");
        assert!(!failure.paused.is_zero());
        assert!(stopped.load(Ordering::Acquire));
    }


    /// The real engine excludes interrupted intentional suspension without setting global STOP.
    #[cfg(not(miri))]
    #[test]
    fn switched_real_search_accounts_pause_without_aborting_command() {
        use std::cell::Cell;
        use xsar::search::{SearchLimits, SearchOptions, SearchOutcome, SearchProblem, SearchStop};
        /// Tiny real-search problem whose first checkpoint precedes its only edge.
        struct Chain;
        impl SearchProblem for Chain {
            type State = u8;
            type Action = u8;
            type Successors = std::option::IntoIter<(u8, u8)>;
            /// Start before the only goal, allowing the checkpoint to suspend first.
            fn initial(&self) -> u8 {
                0
            }
            /// A switch must prevent this goal becoming an accepted old result.
            fn is_goal(&self, state: u8) -> bool {
                state == 1
            }
            /// Supply one deterministic edge with no guest or application state.
            fn successors(&self, _: u8) -> Self::Successors {
                Some((1, 1)).into_iter()
            }
        }
        let gate = Arc::new(Gate::default());
        let _work = gate.begin(Phase::Search).unwrap();
        let epoch = gate.snapshot().unwrap().epoch;
        gate.request(epoch, true).unwrap();
        let switch = Arc::new(AtomicBool::new(false));
        let worker_gate = Arc::clone(&gate);
        let worker_switch = Arc::clone(&switch);
        let worker = std::thread::spawn(move || {
            let local_cancel = AtomicBool::new(false);
            let intentional = Cell::new(Duration::ZERO);
            let checkpoint = || match worker_gate.checkpoint_interruptible(
                Phase::Search,
                || Ok(()),
                || Ok(worker_switch.load(Ordering::Acquire)),
            ) {
                Ok(Checkpoint::Ready(paused)) => paused,
                Ok(Checkpoint::Interrupted(paused)) => {
                    intentional.set(intentional.get().saturating_add(paused));
                    local_cancel.store(true, Ordering::Release);
                    paused
                }
                Err(error) => panic!("unexpected pause failure: {}", error.message),
            };
            let started = Instant::now();
            let outcome = xsar::search::solve_with_control(
                &Chain,
                SearchLimits::default(),
                &local_cancel,
                SearchOptions::default(),
                None,
                Some(&checkpoint),
            );
            (outcome, started.elapsed(), intentional.get())
        });
        parked(&gate);
        let parked_at = Instant::now();
        while parked_at.elapsed() < Duration::from_millis(30) {
            std::thread::yield_now();
        }
        switch.store(true, Ordering::Release);
        let (outcome, total, intentional) = worker.join().unwrap();
        let SearchOutcome::Stopped {
            reason: SearchStop::Cancelled,
            stats,
        } = outcome
        else {
            panic!("only this engine must be cancelled")
        };
        assert!(intentional >= Duration::from_millis(30));
        assert!(stats.elapsed.saturating_add(intentional) <= total);
        assert_eq!(stats.expanded, 0);
        let retained = gate.snapshot().unwrap();
        assert_eq!(retained.epoch, epoch);
        assert!(retained.requested);
        assert!(!retained.acknowledged);
    }


    #[test]
    fn search_auto_pause_360s_and_continue_plus_60s_excludes_paused_time() {
        let gate = Arc::new(Gate::default());
        let _work = gate.begin(Phase::Search).unwrap();
        let (_scope, yield_best) = gate.begin_search();
        let snap0 = gate.snapshot().unwrap();
        let search0 = snap0.search.unwrap();
        assert_eq!(search0.next_auto_pause_at, Duration::from_secs(360));
        assert_eq!(search0.continue_step, Duration::from_secs(60));
        assert!(!search0.auto_paused);
        assert_eq!(search0.best_score, 0);
        assert!(!snap0.can_play_best());

        // Before 360s: checkpoint returns Ready immediately.
        gate.advance_search_clock_for_test(Duration::from_secs(359));
        assert_eq!(
            gate.checkpoint(Phase::Search, || Ok(())).unwrap(),
            Duration::ZERO
        );
        assert!(!gate.snapshot().unwrap().is_auto_paused());

        // Manual pause + wait + Continue does NOT bump the 360s limit and excludes wait time.
        let epoch = gate.snapshot().unwrap().epoch;
        gate.request(epoch, true).unwrap();
        let g = Arc::clone(&gate);
        let t = std::thread::spawn(move || g.checkpoint(Phase::Search, || Ok(())).unwrap());
        parked(&gate);
        std::thread::sleep(Duration::from_millis(30));
        gate.request(epoch, false).unwrap();
        let manual_paused = t.join().unwrap();
        assert!(manual_paused >= Duration::from_millis(30));
        let after_manual = gate.snapshot().unwrap();
        assert!(!after_manual.is_auto_paused());
        assert_eq!(
            after_manual.search.unwrap().next_auto_pause_at,
            Duration::from_secs(360)
        );
        assert!(after_manual.search.unwrap().active_elapsed < Duration::from_secs(360));

        // Reaching 360s active time triggers automatic pause.
        gate.advance_search_clock_for_test(Duration::from_secs(1));
        let g = Arc::clone(&gate);
        let t = std::thread::spawn(move || g.checkpoint(Phase::Search, || Ok(())).unwrap());
        parked(&gate);
        let paused_snap = gate.snapshot().unwrap();
        let auto_paused_at_1 = paused_snap.search.unwrap().active_elapsed;
        assert!(paused_snap.is_auto_paused());
        assert_eq!(
            paused_snap.search.unwrap().next_auto_pause_at,
            Duration::from_secs(360)
        );
        std::thread::sleep(Duration::from_millis(30));

        // Pressing Continue adds +60s -> 420s limit.
        gate.request(epoch, false).unwrap();
        let auto_paused_1 = t.join().unwrap();
        assert!(auto_paused_1 >= Duration::from_millis(30));
        let after_continue_1 = gate.snapshot().unwrap();
        assert!(!after_continue_1.is_auto_paused());
        let expected_limit_2 = auto_paused_at_1 + Duration::from_secs(60);
        assert_eq!(
            after_continue_1.search.unwrap().next_auto_pause_at,
            expected_limit_2
        );
        // Active clock did NOT jump past 420s during the auto-pause sleep.
        assert!(gate.checkpoint(Phase::Search, || Ok(())).unwrap() == Duration::ZERO);

        // Reaching 420s auto-pauses again; Continue bumps +60s -> 480s.
        gate.advance_search_clock_for_test(Duration::from_secs(60));
        let g = Arc::clone(&gate);
        let t = std::thread::spawn(move || g.checkpoint(Phase::Search, || Ok(())).unwrap());
        parked(&gate);
        let paused_snap_2 = gate.snapshot().unwrap();
        assert!(paused_snap_2.is_auto_paused());
        let expected_limit_3 =
            paused_snap_2.search.unwrap().active_elapsed + Duration::from_secs(60);
        gate.request(epoch, false).unwrap();
        let _ = t.join().unwrap();
        assert_eq!(
            gate.snapshot().unwrap().search.unwrap().next_auto_pause_at,
            expected_limit_3
        );

        // Reaching 480s auto-pauses again; Play is rejected at score=0, allowed when score > 0 and wakes immediately.
        assert!(gate.request_play_best(epoch).is_err());
        gate.update_search_best_score(12);
        assert!(gate.snapshot().unwrap().can_play_best());
        gate.advance_search_clock_for_test(Duration::from_secs(60));
        let g = Arc::clone(&gate);
        let t = std::thread::spawn(move || g.checkpoint(Phase::Search, || Ok(())).unwrap());
        parked(&gate);
        assert!(gate.snapshot().unwrap().is_auto_paused());
        assert!(gate.snapshot().unwrap().can_play_best());
        gate.request_play_best(epoch).unwrap();
        let _ = t.join().unwrap();
        assert!(yield_best.load(Ordering::Acquire));
        assert!(!gate.snapshot().unwrap().can_play_best());
    }

    #[test]
    fn session_recon_begin_search_promotes_phase_restores_on_drop_and_respects_disarm() {
        let gate = Arc::new(Gate::default());
        let _work = gate.begin(Phase::Recon).unwrap();
        assert_eq!(gate.snapshot().unwrap().phase, Phase::Recon);

        let (scope, _yield_best) = gate.begin_search();
        let active = gate.snapshot().unwrap();
        assert_eq!(active.phase, Phase::Search);
        assert!(active.search.is_some());

        // Disarming auto-pause (post-search verification boundary) suppresses auto-pause at >= 360s,
        // while still allowing manual Pause.
        gate.advance_search_clock_for_test(Duration::from_secs(400));
        scope.set_auto_pause(false);
        assert_eq!(
            gate.checkpoint(Phase::Search, || Ok(())).unwrap(),
            Duration::ZERO
        );
        assert!(!gate.snapshot().unwrap().is_auto_paused());

        // Manual Pause still parks while auto-pause is disarmed.
        gate.request(active.epoch, true).unwrap();
        let g = Arc::clone(&gate);
        let t = std::thread::spawn(move || g.checkpoint(Phase::Search, || Ok(())).unwrap());
        parked(&gate);
        let manual = gate.snapshot().unwrap();
        assert!(manual.acknowledged && !manual.is_auto_paused());
        gate.request(active.epoch, false).unwrap();
        let _ = t.join().unwrap();

        // Dropping SearchScope clears search metadata and restores outer Phase::Recon.
        drop(scope);
        let restored = gate.snapshot().unwrap();
        assert_eq!(restored.phase, Phase::Recon);
        assert!(restored.search.is_none());
    }
}
