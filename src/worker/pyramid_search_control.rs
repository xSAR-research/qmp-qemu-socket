//! Exact-invocation objective replacement, independent of STOP and serial commands.
//!
//! A request retires only the owned Shortest engine. The planner alone retains
//! its acquisition, decides the next objective and publishes independently replayed
//! actions. Closing an invocation atomically fences late requests and old results.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use crate::{pyramid_workflow_model::RouteMethod, strategy::ControllerContext};


/// Worker-minted search identity; advisory UI copies never authorise guest input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Snapshot {
    /// Checked monotonic identity of this exact engine invocation.
    pub(crate) token: u64,
    /// Immutable selected controller, socket and context generation.
    pub(crate) context: ControllerContext,
    /// Manual calculation or automatic-session identity.
    pub(crate) run_id: u64,
    /// Zero for manual planning; actual acquired automatic board otherwise.
    pub(crate) board_generation: u64,
    /// Objective of the engine currently owning the frontier.
    pub(crate) method: RouteMethod,
    /// An accepted replacement request is waiting for its cooperative checkpoint.
    pub(crate) maximum_requested: bool,
}


/// Single serial engine owner and a token counter which never resets on retirement.
#[derive(Default)]
struct State {
    /// Last issued token; checked exhaustion refuses further invocation ownership.
    last_token: u64,
    /// Present only from engine setup through the atomic completion handshake.
    active: Option<Snapshot>,
}


/// Bounded shared control; no command, capture, route or acquisition is queued here.
#[derive(Default)]
pub(super) struct Gate {
    /// Ownership and pending request are changed together under this mutex.
    state: Mutex<State>,
}


/// RAII invocation owner; abandonment retires only its own token.
pub(super) struct Work {
    /// Shared out-of-band request gate.
    gate: Arc<Gate>,
    /// Exact invocation token, never inferred from UI phase or statistics.
    token: u64,
}


impl Gate {
    /// Open one actual search invocation; STOP, overlap, poison and exhaustion refuse.
    pub(super) fn begin(
        self: &Arc<Self>,
        context: &ControllerContext,
        run_id: u64,
        board_generation: u64,
        method: RouteMethod,
        cancel: &AtomicBool,
    ) -> Result<Work, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Search control is poisoned")?;
        if cancel.load(Ordering::Acquire) {
            return Err("STOP is latched; no replacement search authorised".to_owned());
        }
        if state.active.is_some() {
            return Err("Another engine owns search control".to_owned());
        }
        let token = state
            .last_token
            .checked_add(1)
            .ok_or("Search token exhausted")?;
        state.last_token = token;
        state.active = Some(Snapshot {
            token,
            context: context.clone(),
            run_id,
            board_generation,
            method,
            maximum_requested: false,
        });
        Ok(Work {
            gate: Arc::clone(self),
            token,
        })
    }


    /// Read the actual engine owner without retaining the mutex across UI work.
    pub(super) fn snapshot(&self) -> Option<Snapshot> {
        self.state.lock().ok()?.active.clone()
    }


    /// Replace only the exact open Shortest invocation; duplicate or stale requests refuse.
    /// STOP is checked under the same mutex as request acceptance and never cleared.
    pub(super) fn request_maximum(
        &self,
        owner: &Snapshot,
        cancel: &AtomicBool,
    ) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Search control is poisoned")?;
        if cancel.load(Ordering::Acquire) {
            return Err("STOP is latched; objective replacement refused".to_owned());
        }
        let active = state
            .active
            .as_mut()
            .ok_or("Shortest search has already closed")?;
        if active.token != owner.token
            || active.context != owner.context
            || active.run_id != owner.run_id
            || active.board_generation != owner.board_generation
            || active.method != owner.method
        {
            return Err("Objective request belongs to a different search invocation".to_owned());
        }
        if active.method != RouteMethod::FewestActions || active.maximum_requested {
            return Err("This search cannot accept another Maximum request".to_owned());
        }
        active.maximum_requested = true;
        Ok(())
    }


    /// Retire current requests after STOP/shutdown; the next explicit run gets a new token.
    pub(super) fn abort(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.active = None;
        }
    }
}


impl Work {
    /// Observe the exact invocation's pending request without changing any cancellation flag.
    pub(super) fn maximum_requested(&self) -> Result<bool, String> {
        let state = self
            .gate
            .state
            .lock()
            .map_err(|_| "Search control is poisoned")?;
        let active = state
            .active
            .as_ref()
            .filter(|active| active.token == self.token)
            .ok_or("Search invocation was retired")?;
        Ok(active.maximum_requested)
    }


    /// Atomically close request acceptance and consume pending replacement before using results.
    /// An accepted request wins over a concurrently solved old outcome; later requests refuse.
    pub(super) fn finish(self) -> Result<bool, String> {
        let mut state = self
            .gate
            .state
            .lock()
            .map_err(|_| "Search control is poisoned")?;
        if !state
            .active
            .as_ref()
            .is_some_and(|active| active.token == self.token)
        {
            return Err("Search invocation was retired before completion".to_owned());
        }
        let active = state
            .active
            .take()
            .ok_or("Search invocation has no owner")?;
        Ok(active.maximum_requested)
    }
}


impl Drop for Work {
    /// Retire an abandoned invocation without clearing a later owner or STOP.
    fn drop(&mut self) {
        if let Ok(mut state) = self.gate.state.lock()
            && state
                .active
                .as_ref()
                .is_some_and(|active| active.token == self.token)
        {
            state.active = None;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{game::GameMode, strategy::SolvingStrategy};


    /// Create a controller identity without opening a socket or minting an acquisition.
    fn context() -> ControllerContext {
        ControllerContext::new(
            "/unused/search-control.sock".into(),
            GameMode::Pyramid,
            SolvingStrategy::ShortestPath,
            5,
        )
    }


    /// Exact owner matching prevents another board, run, controller or method changing work.
    #[test]
    fn requests_require_complete_live_invocation_identity() {
        let gate = Arc::new(Gate::default());
        let cancel = AtomicBool::new(false);
        let work = gate
            .begin(&context(), 31, 7, RouteMethod::FewestActions, &cancel)
            .unwrap();
        let owner = gate.snapshot().unwrap();
        let mut changed = owner.clone();
        changed.context.generation += 1;
        assert!(gate.request_maximum(&changed, &cancel).is_err());
        changed = owner.clone();
        changed.run_id += 1;
        assert!(gate.request_maximum(&changed, &cancel).is_err());
        changed = owner.clone();
        changed.board_generation += 1;
        assert!(gate.request_maximum(&changed, &cancel).is_err());
        changed = owner.clone();
        changed.token += 1;
        assert!(gate.request_maximum(&changed, &cancel).is_err());
        changed = owner.clone();
        changed.method = RouteMethod::Fast;
        assert!(gate.request_maximum(&changed, &cancel).is_err());
        assert!(!work.maximum_requested().unwrap());
        gate.request_maximum(&owner, &cancel).unwrap();
        assert!(work.maximum_requested().unwrap());
        assert!(gate.request_maximum(&owner, &cancel).is_err());
        assert!(work.finish().unwrap());
        assert!(gate.snapshot().is_none());
        assert!(gate.request_maximum(&owner, &cancel).is_err());
    }


    /// Each mutex ordering of completion versus request has one consistent observable winner.
    #[test]
    fn completion_handshake_never_accepts_an_ignored_request() {
        use std::sync::Barrier;
        for _ in 0..32 {
            let gate = Arc::new(Gate::default());
            let cancel = Arc::new(AtomicBool::new(false));
            let work = gate
                .begin(&context(), 31, 7, RouteMethod::FewestActions, &cancel)
                .unwrap();
            let owner = gate.snapshot().unwrap();
            let barrier = Arc::new(Barrier::new(2));
            let request_gate = Arc::clone(&gate);
            let request_cancel = Arc::clone(&cancel);
            let request_barrier = Arc::clone(&barrier);
            let request = std::thread::spawn(move || {
                request_barrier.wait();
                request_gate
                    .request_maximum(&owner, &request_cancel)
                    .is_ok()
            });
            barrier.wait();
            let switched = work.finish().unwrap();
            assert_eq!(switched, request.join().unwrap());
            assert!(gate.snapshot().is_none());
        }
    }


    /// STOP cannot be cleared; stale RAII teardown cannot retire a newly opened engine.
    #[test]
    fn stop_and_abandoned_tokens_never_transfer_authority() {
        let gate = Arc::new(Gate::default());
        let cancel = AtomicBool::new(false);
        let old = gate
            .begin(&context(), 31, 7, RouteMethod::FewestActions, &cancel)
            .unwrap();
        let stale = gate.snapshot().unwrap();
        cancel.store(true, Ordering::Release);
        assert!(gate.request_maximum(&stale, &cancel).is_err());
        gate.abort();
        assert!(old.maximum_requested().is_err());
        assert!(
            gate.begin(&context(), 31, 7, RouteMethod::Fast, &cancel)
                .is_err()
        );
        assert!(cancel.load(Ordering::Acquire));
        // Only the test's explicit new-run boundary releases its simulated STOP.
        cancel.store(false, Ordering::Release);
        let next = gate
            .begin(&context(), 32, 8, RouteMethod::Fast, &cancel)
            .unwrap();
        let current = gate.snapshot().unwrap();
        assert!(current.token > stale.token);
        drop(old);
        assert_eq!(gate.snapshot().unwrap(), current);
        assert!(gate.request_maximum(&stale, &cancel).is_err());
        assert!(gate.request_maximum(&current, &cancel).is_err());
        assert!(!next.finish().unwrap());
    }
}
