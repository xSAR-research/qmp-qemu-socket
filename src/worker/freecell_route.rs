//! Single-owner independent Free Cell playback, separate from Solver/HALO input.
use super::*;
use crate::geometry::PixelPoint;
use crate::{
    freecell_recon::position,
    freecell_route::{self as route, Candidate, RouteIo, State},
};
use xsar::games::freecell::{
    InitialDeal,
    rules::{Position, RecordedMove},
    search::Outcome,
};


/// Immutable worker failure, shared with the disk packet and FAILURE DETAILS.
#[derive(Clone, Debug)]
pub(crate) struct Failure {
    /// Original terminal error; retirement failures remain secondary.
    pub error: String,
    /// Exact worker-owned stage, action, evidence and counters.
    pub details: serde_json::Value,
    /// Reserved packet directory, including partial persistence.
    pub directory: Option<PathBuf>,
    /// Persistence omissions/errors never replace the primary error.
    pub persistence_error: Option<String>,
}


/// Persist acquired evidence once; no input, capture or replay is performed.
fn failure(
    plan: &freecell_calculation::Identity,
    run_id: u64,
    error: String,
    details: serde_json::Value,
    first: Option<&CapturedFrame>,
    latest: Option<&CapturedFrame>,
) -> Failure {
    let saved =
        crate::pyramid_diagnostics::persist_failure(crate::pyramid_diagnostics::FailureEvidence {
            progress: None,
            context: &plan.context,
            phase: crate::pyramid_diagnostics::PyramidPhase::Execution,
            run_id: Some(run_id),
            error: &error,
            details: details.clone(),
            source_document: None,
            first_frame: first,
            latest_frame: latest,
        });
    Failure {
        error,
        details,
        directory: saved.directory,
        persistence_error: saved.persistence_error,
    }
}


/// Format checked physical action identity for both persisted and visible details.
fn stage_details(stage: route::Stage) -> serde_json::Value {
    let location = |pile: xsar::games::freecell::rules::Pile| match pile {
        xsar::games::freecell::rules::Pile::Cascade(n) => format!("PLAY {}", u16::from(n) + 1),
        xsar::games::freecell::rules::Pile::Cell(n) => format!("CELL {}", u16::from(n) + 1),
        xsar::games::freecell::rules::Pile::Foundation(n) => format!(
            "SUIT {}",
            ["Clubs", "Diamonds", "Hearts", "Spades"]
                .get(usize::from(n))
                .copied()
                .unwrap_or("invalid")
        ),
    };
    serde_json::json!({"cursor_zero_based":stage.cursor,"pending_transfer_one_based":stage.action.and_then(|_|stage.cursor.checked_add(1)),
        "total":stage.total,"autoplay_mode":format!("{:?}",stage.mode),"phase":stage.phase,
        "observation":stage.observation,"observation_limit":6,
        "action":stage.action.map(|action|serde_json::json!({"card_id":action.card_id,
            "card":if action.card_id < 52 {format!("{}{}", xsar::games::cards::Rank::from_value(action.card_id%13+1).map(|r|r.to_string()).unwrap_or_default(),
                ["C","D","H","S"].get(usize::from(action.card_id/13)).copied().unwrap_or("?"))} else {"invalid".to_owned()},
            "from":location(action.from),"to":location(action.to),"is_auto":action.is_auto}))})
}


/// Accepted semantic progress, never an input acknowledgement alone.
#[derive(Clone, Debug)]
pub(crate) struct Progress {
    /// Offline candidate identity, including source generation and context.
    pub plan: freecell_calculation::Identity,
    /// Distinct playback invocation.
    pub run_id: u64,
    /// Recorded transfers proved at the live endpoint.
    pub cursor: usize,
    /// Complete route length including automatic transfers.
    pub total: usize,
    /// Actual attempted explicit-target drag transfers.
    pub attempted: usize,
    /// Complete drag deliveries acknowledged by QMP.
    pub acknowledged: usize,
}


/// One immutable plan and its mutable same-run physical history.
#[derive(Clone)]
struct Live {
    /// Original calculation correlation; never a live receipt by itself.
    plan: freecell_calculation::Identity,
    /// Shared immutable frozen input and independently replayed route.
    report: Arc<freecell_calculation::Report>,
    /// Last accepted physical endpoint.
    state: State,
    /// Frozen performer of automatic transfers, selected at first reservation.
    mode: Option<route::AutoMode>,
}


/// Retain only one candidate and one active playback identity.
#[derive(Default)]
pub(super) struct Store {
    /// Current immutable candidate with semantic cursor.
    live: Option<Live>,
    /// Monotonic playback serial, never reset with candidate retirement.
    serial: u64,
    /// Exact owner until matching completion; STOP does not release it early.
    running: Option<(freecell_calculation::Identity, u64)>,
    /// Coalesced advisory progress only.
    progress: Option<Progress>,
}


impl Store {
    /// Retain a new replay-verified offline result without granting live authority.
    pub(super) fn prepare(
        &mut self,
        plan: freecell_calculation::Identity,
        report: Arc<freecell_calculation::Report>,
    ) -> Result<(), String> {
        if self.running.is_some() {
            return Err("Free Cell playback still owns its completion".into());
        }
        self.live = None;
        self.progress = None;
        if matches!(&report.outcome, Outcome::Solved { .. }) {
            self.live = Some(Live {
                state: State::new(&report.deal),
                mode: None,
                plan,
                report,
            });
        }
        Ok(())
    }


    /// Revoke a candidate without letting a queued completion release another owner.
    pub(super) fn revoke(&mut self) {
        self.live = None;
        self.progress = None;
    }


    /// Test readiness against the exact displayed immutable calculation identity.
    pub(super) fn available(&self, plan: &freecell_calculation::Identity) -> bool {
        self.running.is_none()
            && self
                .live
                .as_ref()
                .is_some_and(|l| &l.plan == plan && !l.state.position.is_goal())
    }


    /// Reserve before dispatch, never relying on a pathname or canonical board equality.
    #[cfg(test)]
    pub(super) fn reserve(&mut self, plan: &freecell_calculation::Identity) -> Result<u64, String> {
        self.reserve_mode(plan, route::AutoMode::ObserveGame)
    }


    /// Freeze autoplay policy; successful step resumes must retain it.
    pub(super) fn reserve_mode(
        &mut self,
        plan: &freecell_calculation::Identity,
        mode: route::AutoMode,
    ) -> Result<u64, String> {
        if !self.available(plan) {
            return Err("Free Cell candidate is unavailable; CALCULATE again".into());
        }
        let live = self.live.as_mut().ok_or("Missing Free Cell candidate")?;
        if live.mode.is_some_and(|previous| previous != mode) {
            return Err("Autoplay mode changed; CALCULATE again".into());
        }
        live.mode = Some(mode);
        let next = self
            .serial
            .checked_add(1)
            .ok_or("Free Cell playback identity exhausted")?;
        self.serial = next;
        self.running = Some((plan.clone(), next));
        self.progress = None;
        Ok(next)
    }


    /// Consume coalesced progress without granting authority.
    pub(super) fn take_progress(&mut self) -> Option<Progress> {
        self.progress.take()
    }


    /// A cloned candidate grants no authority after revocation or owner replacement.
    fn check(&self, plan: &freecell_calculation::Identity, run_id: u64) -> Result<(), String> {
        if self.running.as_ref() != Some(&(plan.clone(), run_id))
            || !self.live.as_ref().is_some_and(|live| live.plan == *plan)
        {
            return Err("Free Cell playback authority revoked".into());
        }
        Ok(())
    }


    /// Retire an exact failed reservation, including queue failures before transport.
    pub(super) fn reject(
        &mut self,
        plan: &freecell_calculation::Identity,
        run_id: u64,
    ) -> Result<(), String> {
        self.finish(plan, run_id, None, false)
    }


    /// Retire the matching invocation; every error permanently destroys its candidate.
    fn finish(
        &mut self,
        plan: &freecell_calculation::Identity,
        run_id: u64,
        state: Option<State>,
        success: bool,
    ) -> Result<(), String> {
        if self.running.as_ref() != Some(&(plan.clone(), run_id)) {
            return Err("Stale Free Cell playback completion".into());
        }
        self.running = None;
        if success {
            let Some(state) = state else {
                self.revoke();
                return Err("Missing Free Cell completion position".into());
            };
            let Some(live) = self.live.as_mut().filter(|live| &live.plan == plan) else {
                self.revoke();
                return Err("Free Cell playback authority revoked before completion".into());
            };
            live.state = state;
            if !state.position.is_goal() {
                return Ok(());
            }
        }
        self.revoke();
        Ok(())
    }
}


/// One native adapter, retaining only first rejected and latest acquired pixels.
struct Native<'a> {
    /// Immutable owning identity.
    plan: &'a freecell_calculation::Identity,
    /// Playback invocation used in progress and logs.
    run_id: u64,
    /// Existing QMP transport with release-only recovery.
    qmp: QmpClient,
    /// Existing event and bounded preview channels.
    events: &'a WorkerEventSink,
    /// Immediate operator cancellation.
    cancel: &'a AtomicBool,
    /// Causal STOP versus real-failure classification.
    trace: &'a pyramid_stop::StopTrace,
    /// Acquired failure pixels remain paired with the current frame.
    latest: Option<CapturedFrame>,
    /// Immutable first unsupported pixels, distinct from the terminal frame.
    first: Option<CapturedFrame>,
    /// Frozen source and destination coordinates of this compound transfer.
    frozen: Option<(RecordedMove, PixelPoint, PixelPoint)>,
    /// Exact phase and bounded first/latest rejected observations.
    stage: Option<route::Stage>,
    /// Monotonic acquired-frame ordinal within this invocation.
    capture_serial: u64,
    /// First rejected proof remains historical when later errors differ.
    first_rejection: Option<serde_json::Value>,
    /// Most recent candidate rejection, not a substitute for terminal error.
    last_rejection: Option<serde_json::Value>,
    /// Actual input attempts and acknowledged deliveries.
    counts: (usize, usize),
    /// Central application timing copied at the invocation boundary.
    settle: Duration,
}


impl Native<'_> {
    /// Check the exact active authority without consulting a later cancellation latch.
    fn active(&self) -> Result<(), String> {
        self.trace.check(
            &self.events.latest_frame.reconnaissance,
            &self.plan.context,
            self.cancel,
        )?;
        let result = self
            .events
            .latest_frame
            .freecell_route
            .lock()
            .map_err(|_| "Free Cell playback store poisoned".to_owned())
            .and_then(|store| store.check(self.plan, self.run_id));
        if result.is_err() {
            self.trace
                .adopt(crate::pyramid_workflow_model::RunTermination::Failed);
        }
        result
    }


    /// Acquire one native TCPv3 frame; no fallback, retry or additional recording capture.
    fn capture(&mut self) -> Result<(), String> {
        self.active()?;
        let (frame, _) =
            capture_screen_causal(&mut self.qmp, &self.plan.context.socket_path, self.events)
                .map_err(|e| {
                    self.trace.capture_error(
                        e,
                        &self.events.latest_frame.reconnaissance,
                        &self.plan.context,
                        self.cancel,
                    )
                })?;
        self.capture_serial = self
            .capture_serial
            .checked_add(1)
            .ok_or("Free Cell capture serial exhausted")?;
        self.latest = Some(frame);
        if let Some(frame) = &self.latest {
            self.events.latest_frame.publish(
                frame.clone(),
                None,
                false,
                true,
                Some(self.plan.context.clone()),
            );
        }
        self.active()
    }


    /// Prove one supplied model; hidden cards depend only on already authenticated history.
    fn prove(&self, expected: &Position) -> Result<position::Geometry, String> {
        let frame = self
            .latest
            .as_ref()
            .ok_or("Missing Free Cell route frame")?;
        position::verify(frame, expected, &mut || self.active())
    }


    /// Persist already-acquired incident pixels without replacing the original failure.
    fn details(&self) -> serde_json::Value {
        let stage = self.stage.map(stage_details);
        serde_json::json!({"game":"Free Cell","protocol":"explicit held drag",
            "request_id":self.plan.request_id,"source_generation":self.plan.source_generation,
            "run_id":self.run_id,"stage":stage,"capture_serial":self.capture_serial,
            "attempted":self.counts.0,"acknowledged":self.counts.1,"guest_effect_proven_by_ack":false,
            "first_rejection_historical":self.first_rejection,"last_rejection":self.last_rejection,
            "frozen_endpoints":self.frozen.map(|(action,from,to)|format!("{action:?}: {from:?} -> {to:?}")),
        })
    }
}


impl RouteIo for Native<'_> {
    /// Pause only at safe sequencing boundaries; active checks still surround each input.
    fn check(&mut self) -> Result<(), String> {
        self.events
            .latest_frame
            .pause
            .checkpoint(pyramid_pause::Phase::Gameplay, || self.active())
            .map_err(|e| e.message)?;
        self.active()
    }


    /// Independently read every original identity before carrying compressed hidden strips.
    fn bind(&mut self, deal: &InitialDeal) -> Result<(), String> {
        self.capture()?;
        let frame = self.latest.as_ref().ok_or("Missing initial frame")?;
        let observed =
            crate::freecell_recon::recognise_checked(frame, deal.metadata().clone(), &mut || {
                self.active()
            })
            .map_err(|e| e.to_string())?;
        if !route::same_physical(&Position::initial(deal), &Position::initial(&observed)) {
            return Err(
                "Live Free Cell deal differs from frozen calculation; no input authorised".into(),
            );
        }
        Ok(())
    }


    /// Observe one exact physical route prefix, without advancing on unmatched evidence.
    fn observe(&mut self, candidates: &[Candidate]) -> Result<Option<usize>, String> {
        self.capture()?;
        let mut rejections = Vec::new();
        let mut matched = None;
        for (index, candidate) in candidates.iter().enumerate() {
            self.active()?;
            match self.prove(&candidate.position) {
                Ok(_) => {
                    if matched.is_none() {
                        matched = Some(index);
                    }
                }
                Err(reason) => {
                    self.active()?;
                    rejections.push(serde_json::json!({"cursor":candidate.cursor,"reason":reason}));
                }
            }
        }
        if matched.is_some_and(|index| index + 1 == candidates.len()) {
            return Ok(matched);
        }
        let rejection = serde_json::json!({"capture_serial":self.capture_serial,
            "observation":self.stage.map(|s|s.observation),"stage":self.stage.map(stage_details),
            "matched_intermediate_cursor":matched.and_then(|index|candidates.get(index).map(|c|c.cursor)),
            "candidates":rejections});
        if self.first.is_none() {
            self.first = self.latest.clone();
            self.first_rejection = Some(rejection.clone());
        }
        self.last_rejection = Some(rejection);
        Ok(matched)
    }


    /// Freeze diagnostic position/action before any fallible stage operation.
    fn stage(&mut self, stage: route::Stage) {
        self.stage = Some(stage);
    }


    /// Deliver one explicit-target held drag from freshly proved parent geometry.
    fn transfer(&mut self, parent: &Position, action: RecordedMove) -> Result<(), String> {
        self.capture()?;
        let geometry = self.prove(parent)?;
        let from = geometry.point(action.from)?;
        let to = geometry.point(action.to)?;
        self.frozen = Some((action, from, to));
        self.active()?;
        let probe = self.qmp.probe().map_err(|e| e.to_string())?;
        validate_probe(&probe)?;
        self.active()?;
        self.counts.0 = self
            .counts
            .0
            .checked_add(1)
            .ok_or("Free Cell attempted counter exhausted")?;
        // Borrow disjoint resources: each neutral transport phase rechecks the same
        // context and live plan without acquiring pixels or retaining a lock over I/O.
        let plan = self.plan;
        let run_id = self.run_id;
        let events = self.events;
        let cancel = self.cancel;
        let trace = self.trace;
        let mut interrupted = None;
        let delivered = self.qmp.drag(from, to, 1_920, 1_080, || {
            let active = trace
                .check(&events.latest_frame.reconnaissance, &plan.context, cancel)
                .and_then(|()| {
                    events
                        .latest_frame
                        .freecell_route
                        .lock()
                        .map_err(|_| "Free Cell route store poisoned".to_owned())?
                        .check(plan, run_id)
                });
            if let Err(error) = active {
                interrupted = Some(error);
                false
            } else {
                true
            }
        });
        if let Err(error) = delivered {
            return Err(format!(
                "Uncertain Free Cell drag; never replay: {error}{}",
                interrupted
                    .map(|cause| format!("; {cause}"))
                    .unwrap_or_default()
            ));
        }
        self.counts.1 = self
            .counts
            .1
            .checked_add(1)
            .ok_or("Free Cell ACK counter exhausted")?;
        let _ = self.events.send(WorkerEvent::Log(format!(
            "Free Cell drag {action:?}: {from:?} -> {to:?} acknowledged; card effect requires fresh semantic proof"
        )));
        self.active()
    }


    /// Wait cooperatively without authorising another input or capture.
    fn settle(&mut self) -> Result<(), String> {
        if cancellable_wait(self.settle, self.cancel).is_err() {
            self.active()?;
            return Err("Free Cell settling interrupted".into());
        }
        self.active()
    }


    /// Coalesce only semantic progress; ACK accounting remains separate.
    fn progress(&mut self, cursor: usize, total: usize, _position: &Position) {
        if let Ok(mut store) = self.events.latest_frame.freecell_route.lock()
            && store.running.as_ref() == Some(&(self.plan.clone(), self.run_id))
        {
            store.progress = Some(Progress {
                plan: self.plan.clone(),
                run_id: self.run_id,
                cursor,
                total,
                attempted: self.counts.0,
                acknowledged: self.counts.1,
            });
        }
    }
}


/// Execute one reserved invocation, permanently retiring on any refusal or interruption.
pub(super) fn run(
    plan: freecell_calculation::Identity,
    run_id: u64,
    single_step: bool,
    settle: Duration,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) {
    let mut state = None;
    let trace = pyramid_stop::StopTrace::default();
    let mut balanced = true;
    let mut evidence = (
        serde_json::json!({"game":"Free Cell","run_id":run_id,
        "request_id":plan.request_id,"source_generation":plan.source_generation,
        "stage":"before native adapter","attempted":0,"acknowledged":0}),
        None,
        None,
    );
    let result = (|| {
        let live = {
            let store = events
                .latest_frame
                .freecell_route
                .lock()
                .map_err(|_| "Free Cell route store poisoned")?;
            if store.running.as_ref() != Some(&(plan.clone(), run_id)) {
                return Err("Free Cell playback owner retired".into());
            }
            store
                .live
                .as_ref()
                .filter(|l| l.plan == plan)
                .cloned()
                .ok_or("Free Cell plan retired")?
        };
        let mut current = live.state;
        let Outcome::Solved { moves, .. } = &live.report.outcome else {
            return Err("No complete Free Cell route".into());
        };
        evidence.0 = serde_json::json!({"game":"Free Cell", "run_id":run_id,
            "request_id":plan.request_id,"source_generation":plan.source_generation,
            "stage":live.mode.map(|mode|stage_details(route::Stage {cursor:current.cursor,total:moves.len(),
                action:moves.get(current.cursor).copied(),mode,phase:"before transport",observation:0})),
            "attempted":0,"acknowledged":0,"capture_serial":0});
        match selected_capture_backend()? {
            CaptureBackend::Tcp {
                mode: CaptureMode::BgrV3 | CaptureMode::PersistentBgrV3,
                ..
            } => {}
            _ => return Err("Independent Free Cell playback requires native TCPv3".into()),
        }
        trace.check(&events.latest_frame.reconnaissance, &plan.context, cancel)?;
        let qmp = QmpClient::connect(&plan.context.socket_path).map_err(|e| e.to_string())?;
        let mut native = Native {
            plan: &plan,
            run_id,
            qmp,
            events,
            cancel,
            trace: &trace,
            latest: None,
            first: None,
            frozen: None,
            stage: None,
            capture_serial: 0,
            first_rejection: None,
            last_rejection: None,
            counts: (0, 0),
            settle,
        };
        let result = route::drive(
            &mut current,
            &live.report.deal,
            moves,
            single_step,
            live.mode.ok_or("Missing frozen autoplay policy")?,
            &mut native,
        );
        state = Some(current);
        balanced = native.counts.0 == native.counts.1;
        evidence = (
            native.details(),
            native.first.clone(),
            native.latest.clone(),
        );
        result.map(|()| Progress {
            plan: plan.clone(),
            run_id,
            cursor: current.cursor,
            total: moves.len(),
            attempted: native.counts.0,
            acknowledged: native.counts.1,
        })
    })();
    // Order the final typed authority check with exact Store retirement. STOP latches
    // its cause before waiting to revoke this same store; unrelated revocation stays fatal.
    let mut result = result;
    let retirement = events
        .latest_frame
        .freecell_route
        .lock()
        .map_err(|_| "Free Cell route retirement poisoned".to_owned())
        .and_then(|mut store| {
            if result.is_ok()
                && let Err(error) =
                    trace.check(&events.latest_frame.reconnaissance, &plan.context, cancel)
            {
                result = Err(error);
            }
            store.finish(&plan, run_id, state, result.is_ok())
        });
    if retirement.is_err() {
        trace.adopt(crate::pyramid_workflow_model::RunTermination::Failed);
    }
    if let Err(error) = retirement {
        if let Some(object) = evidence.0.as_object_mut() {
            object.insert("retirement_error".into(), error.clone().into());
        }
        if result.is_ok() {
            result = Err(error);
        }
    }
    let result = result.map_err(|error| {
        Box::new(failure(
            &plan,
            run_id,
            error,
            evidence.0,
            evidence.1.as_ref(),
            evidence.2.as_ref(),
        ))
    });
    let termination = trace.termination(result.is_err(), balanced);
    events.latest_frame.reconnaissance.finish();
    let _ = events.send_for_context(
        Some(plan.context.clone()),
        WorkerEvent::FreeCellRouteFinished {
            plan,
            run_id,
            result,
            termination,
        },
    );
}


/// Refuse a reserved request before transport, retaining exact completion ownership.
pub(super) fn reject(
    plan: freecell_calculation::Identity,
    run_id: u64,
    error: String,
    events: &WorkerEventSink,
) {
    let retirement = events
        .latest_frame
        .freecell_route
        .lock()
        .map_err(|_| "Free Cell retirement poisoned".to_owned())
        .and_then(|mut store| store.reject(&plan, run_id));
    let result = Err(Box::new(failure(
        &plan,
        run_id,
        error,
        serde_json::json!({"game":"Free Cell","phase":"dispatch","run_id":run_id,
            "request_id":plan.request_id,"retirement_error":retirement.err(),"attempted":0,"acknowledged":0}),
        None,
        None,
    )));
    events.latest_frame.reconnaissance.finish();
    let _ = events.send_for_context(
        Some(plan.context.clone()),
        WorkerEvent::FreeCellRouteFinished {
            plan,
            run_id,
            result,
            termination: crate::pyramid_workflow_model::RunTermination::Failed,
        },
    );
}


#[cfg(test)]
mod tests;
