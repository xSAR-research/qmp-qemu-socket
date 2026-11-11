//! Single-owner independent Free Cell playback, separate from Solver/HALO input.
use super::*;
use crate::geometry::PixelPoint;
use crate::{
    freecell_recon::position,
    freecell_route::{self as route, Candidate, RouteIo, State},
};
use xsar::games::freecell::{
    InitialDeal,
    rules::{Pile, Position, RecordedMove},
    search::BestEffortOutcome as Outcome,
};


/// Native Free Cell fixed-slot height from the existing RECON effect-slot bounds.
const DIAGNOSTIC_CARD_HEIGHT: u32 = 182;


/// Approved native vertical lift: floor one quarter of the 182px card height.
const DIAGNOSTIC_LIFT_PIXELS: i32 = 45;


/// Plan four native ABS held targets after proof, without recognising or sending input.
///
/// PLAY retains its authenticated point. CELL/SUIT use the centre of the same
/// canonical fixed slot whether occupied or empty. Held points are source minus
/// 45px vertically, the floor midpoint of the original source/effective target,
/// target minus 45px vertically, then the exact target. Only the final two points
/// must stay inside the destination slot. Source and observed target remain
/// bounds-checked proof coordinates; every generated point must stay in frame.
/// Invalid piles, overflow or any point outside the frame fail without fallback.
fn diagnostic_abs_path(
    source: PixelPoint,
    observed_destination: PixelPoint,
    target: Pile,
    frame: (u32, u32),
) -> Result<[PixelPoint; 4], String> {
    let slot_x = match target {
        Pile::Cascade(n) if n < 8 => None,
        Pile::Cell(n) => Some(
            *crate::freecell::CELL_X
                .get(usize::from(n))
                .ok_or("Invalid Free Cell diagnostic CELL target")?,
        ),
        Pile::Foundation(n) => {
            // Canonical C,D,H,S identities occupy physical H,C,D,S slots.
            let screen = match n {
                0 => 1,
                1 => 2,
                2 => 0,
                3 => 3,
                _ => return Err("Invalid Free Cell diagnostic SUIT target".into()),
            };
            Some(
                *crate::freecell::FOUNDATION_X
                    .get(screen)
                    .ok_or("Invalid Free Cell diagnostic physical SUIT slot")?,
            )
        }
        _ => return Err("Invalid Free Cell diagnostic PLAY target".into()),
    };
    let slot = slot_x.map(|x| {
        crate::geometry::PixelRect::new(x, 112, crate::freecell::CARD_WIDTH, DIAGNOSTIC_CARD_HEIGHT)
    });
    let destination = match slot {
        Some(rect) => rect
            .try_centre()
            .ok_or("Free Cell diagnostic slot centre overflow")?,
        None => observed_destination,
    };
    let source_upper = PixelPoint::new(
        source.x,
        source
            .y
            .checked_sub(DIAGNOSTIC_LIFT_PIXELS)
            .ok_or("Free Cell diagnostic source lift overflow")?,
    );
    let midpoint_axis = |first: i32, last: i32| {
        i32::try_from((i64::from(first) + i64::from(last)) / 2)
            .map_err(|_| "Free Cell diagnostic midpoint overflow")
    };
    let midpoint = PixelPoint::new(
        midpoint_axis(source.x, destination.x)?,
        midpoint_axis(source.y, destination.y)?,
    );
    let destination_upper = PixelPoint::new(
        destination.x,
        destination
            .y
            .checked_sub(DIAGNOSTIC_LIFT_PIXELS)
            .ok_or("Free Cell diagnostic destination lift overflow")?,
    );
    let points = [source_upper, midpoint, destination_upper, destination];
    for point in [source, observed_destination].into_iter().chain(points) {
        crate::geometry::pixel_point_to_qmp(point, frame.0, frame.1)
            .map_err(|error| format!("Invalid Free Cell diagnostic native point: {error}"))?;
    }
    if slot.is_some_and(|rect| !rect.contains(destination_upper) || !rect.contains(destination)) {
        return Err("Free Cell diagnostic held target outside fixed slot".into());
    }
    Ok(points)
}


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


/// Private already-acquired motion-probe evidence; it grants no route or input authority.
#[derive(Clone, Debug)]
pub(crate) struct ProbeEvidence {
    /// Same-invocation acquired-frame ordinal, after the acknowledged probe.
    pub capture_serial: u64,
    /// Reserved private packet directory, including partial persistence.
    pub directory: Option<PathBuf>,
    /// Persistence omissions remain secondary to a completed diagnostic acquisition.
    pub persistence_error: Option<String>,
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
    /// Foundation identities at the last twice-proved physical route endpoint.
    pub foundation_cards: u32,
    /// Final semantic reason; coalesced intermediate progress carries none.
    pub completion: Option<route::Completion>,
    /// Present only for the separate bounded motion diagnostic, never route effect proof.
    pub probe_evidence: Option<ProbeEvidence>,
}


/// Expose only replay-verified route outcomes; cancellation/failure never supplies input.
fn recorded_route(outcome: &Outcome) -> Result<(&[RecordedMove], route::Target), String> {
    match outcome {
        Outcome::Solved { moves, .. } => Ok((moves, route::Target::FullBoard)),
        Outcome::Partial {
            moves,
            foundation_cards,
            ..
        } => Ok((
            moves,
            route::Target::Partial {
                foundation_cards: *foundation_cards,
            },
        )),
        _ => Err("No replay-verified Free Cell route".into()),
    }
}


/// Original immutable acquisition and independently verified route, never a live permit.
#[derive(Clone)]
struct Cached {
    /// Original calculation correlation; never a live receipt by itself.
    plan: freecell_calculation::Identity,
    /// Shared immutable frozen input and independently replayed route.
    report: Arc<freecell_calculation::Report>,
}


/// Revocable physical history for the current playback policy.
#[derive(Clone)]
struct Live {
    /// Last accepted physical endpoint.
    state: State,
    /// Frozen recorded-follow-up policy, selected at first reservation.
    mode: route::AutoMode,
}


/// Validated candidate whose private fields cannot be forged by the publication caller.
pub(super) struct Prepared {
    /// None preserves an unusable report without retaining any executable route.
    cached: Option<Cached>,
}


/// Validation cancellation stays distinct from a genuine replay/provenance failure.
#[derive(Debug, thiserror::Error)]
pub(super) enum PreparationError {
    /// The actual owning request cancelled before/during defensive replay.
    #[error("Free Cell route preparation cancelled")]
    Cancelled,
    /// Original metadata, replay or counter disagreement.
    #[error("{0}")]
    Failed(String),
}


impl Prepared {
    /// Validate outside all calculation/route locks, checking the actual request token.
    pub(super) fn new(
        plan: freecell_calculation::Identity,
        report: Arc<freecell_calculation::Report>,
        cancel: &AtomicBool,
    ) -> Result<Self, PreparationError> {
        if recorded_route(&report.outcome).is_err() {
            return Ok(Self { cached: None });
        }
        if cancel.load(Ordering::Acquire) {
            return Err(PreparationError::Cancelled);
        }
        crate::freecell_recon::objective::require_acquired(report.deal.metadata())
            .map_err(PreparationError::Failed)?;
        if let Outcome::Partial {
            moves,
            deliberate_moves,
            automatic_moves,
            foundation_cards,
            ..
        } = &report.outcome
        {
            let replay = xsar::games::freecell::replay::verify_prefix_with_control(
                &report.deal,
                moves,
                cancel,
            )
            .map_err(|error| {
                if error.kind == xsar::games::freecell::replay::ReplayFailure::Cancelled {
                    PreparationError::Cancelled
                } else {
                    PreparationError::Failed(format!("Free Cell partial replay refused: {error}"))
                }
            })?;
            if *foundation_cards >= 52
                || replay
                    .endpoint
                    .foundations()
                    .iter()
                    .copied()
                    .map(u32::from)
                    .sum::<u32>()
                    != *foundation_cards
                || replay.deliberate != *deliberate_moves
                || replay.automatic != *automatic_moves
            {
                return Err(PreparationError::Failed(
                    "Free Cell partial replay disagrees with the verified endpoint".into(),
                ));
            }
        }
        if cancel.load(Ordering::Acquire) {
            return Err(PreparationError::Cancelled);
        }
        Ok(Self {
            cached: Some(Cached { plan, report }),
        })
    }
}


/// Retain only one candidate and one active playback identity.
#[derive(Default)]
pub(super) struct Store {
    /// Independently verified original input, retained across ordinary playback failure.
    cached: Option<Cached>,
    /// Revocable semantic cursor; absence forces fresh complete original-board binding.
    live: Option<Live>,
    /// Monotonic playback serial, never reset with candidate retirement.
    serial: u64,
    /// Exact owner until matching completion; STOP does not release it early.
    running: Option<(freecell_calculation::Identity, u64)>,
    /// Coalesced advisory progress only.
    progress: Option<Progress>,
}


impl Store {
    /// Validate socket-free fixtures before exercising the publication seam.
    #[cfg(test)]
    pub(super) fn prepare(
        &mut self,
        plan: freecell_calculation::Identity,
        report: Arc<freecell_calculation::Report>,
    ) -> Result<(), String> {
        let prepared = Prepared::new(plan, report, &AtomicBool::new(false))
            .map_err(|error| error.to_string())?;
        self.install(Some(prepared))
    }


    /// Publish only a privately validated candidate; no replay or file work occurs here.
    pub(super) fn install(&mut self, prepared: Option<Prepared>) -> Result<(), String> {
        if self.running.is_some() {
            return Err("Free Cell playback still owns its completion".into());
        }
        self.cached = prepared.and_then(|prepared| prepared.cached);
        self.live = None;
        self.progress = None;
        Ok(())
    }


    /// Revoke a candidate without letting a queued completion release another owner.
    pub(super) fn revoke(&mut self) {
        self.cached = None;
        self.retire_live();
    }


    /// Discard uncertain physical history while keeping immutable offline evidence.
    fn retire_live(&mut self) {
        self.live = None;
        self.progress = None;
    }


    /// Test readiness against the exact displayed immutable calculation identity.
    pub(super) fn available(&self, plan: &freecell_calculation::Identity) -> bool {
        self.running.is_none()
            && self.cached.as_ref().is_some_and(|cached| {
                &cached.plan == plan
                    && recorded_route(&cached.report.outcome).is_ok_and(|(moves, _)| {
                        self.live.as_ref().map_or(!moves.is_empty(), |live| {
                            !live.state.position.is_goal() && live.state.cursor < moves.len()
                        })
                    })
            })
    }


    /// Reserve before dispatch, never relying on a pathname or canonical board equality.
    #[cfg(test)]
    pub(super) fn reserve(&mut self, plan: &freecell_calculation::Identity) -> Result<u64, String> {
        self.reserve_mode(plan, route::AutoMode::ObserveGame)
    }


    /// Reserve exact cached input; a new policy or failed history restarts from the original.
    pub(super) fn reserve_mode(
        &mut self,
        plan: &freecell_calculation::Identity,
        mode: route::AutoMode,
    ) -> Result<u64, String> {
        if !self.available(plan) {
            return Err("Free Cell candidate is unavailable; CALCULATE again".into());
        }
        let next = self
            .serial
            .checked_add(1)
            .ok_or("Free Cell playback identity exhausted")?;
        if self.live.as_ref().is_none_or(|live| live.mode != mode) {
            let cached = self.cached.as_ref().ok_or("Missing Free Cell candidate")?;
            self.live = Some(Live {
                state: State::new(&cached.report.deal),
                mode,
            });
        }
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
            || self.live.is_none()
            || !self
                .cached
                .as_ref()
                .is_some_and(|cached| cached.plan == *plan)
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
        self.finish(plan, run_id, None, None)
    }


    /// Retire exact ownership; ordinary failure keeps the original route without live history.
    fn finish(
        &mut self,
        plan: &freecell_calculation::Identity,
        run_id: u64,
        state: Option<State>,
        completion: Option<route::Completion>,
    ) -> Result<(), String> {
        if self.running.as_ref() != Some(&(plan.clone(), run_id)) {
            return Err("Stale Free Cell playback completion".into());
        }
        self.running = None;
        // A positive terminal observation has already destroyed the live permit;
        // preserve only its exact queued completion owner until this point.
        if matches!(
            completion,
            Some(route::Completion::ChallengeComplete | route::Completion::MovementProbe)
        ) {
            self.revoke();
            return Ok(());
        }
        if let Some(completion) = completion {
            let Some(state) = state else {
                self.retire_live();
                return Err("Missing Free Cell completion position".into());
            };
            let Some(cached) = self.cached.as_ref().filter(|cached| &cached.plan == plan) else {
                self.retire_live();
                return Err("Free Cell playback authority revoked before completion".into());
            };
            let (moves, target) = recorded_route(&cached.report.outcome)?;
            let valid = match completion {
                route::Completion::StepVerified => {
                    state.cursor < moves.len() && !state.position.is_goal()
                }
                route::Completion::FullBoard => {
                    state.cursor == moves.len()
                        && target == route::Target::FullBoard
                        && state.position.is_goal()
                }
                route::Completion::PartialRouteExhausted => {
                    state.cursor == moves.len()
                        && matches!(target, route::Target::Partial { foundation_cards }
                        if state.position.foundation_cards() == foundation_cards)
                }
                route::Completion::ChallengeComplete | route::Completion::MovementProbe => {
                    unreachable!("handled before live access")
                }
            };
            if !valid {
                self.retire_live();
                return Err("Free Cell completion disagrees with its physical endpoint".into());
            }
            if completion == route::Completion::StepVerified {
                let live = self
                    .live
                    .as_mut()
                    .ok_or("Free Cell playback authority revoked before completion")?;
                live.state = state;
                return Ok(());
            }
            self.revoke();
            return Ok(());
        }
        self.retire_live();
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
    /// Fresh complete physical Free Cell binding, retained through same-source steps only.
    completion_bound: bool,
    /// Positive common terminal report remains true even if recording later fails.
    completion_observed: bool,
    /// Only an explicit A* Free Cell Single Step selects the diagnostic gesture.
    diagnostic_step: bool,
    /// Diagnostic motion mode frozen with this exact worker command.
    drag_mode: route::DragMode,
    /// ABS diagnostic interval frozen with this exact worker command.
    drag_timing: route::DiagnosticDragTiming,
    /// ACK receipt retained even if subsequent read-only acquisition fails.
    probe_delivered: bool,
    /// Diagnostic acquisition receipt, separate from route progress and failures.
    probe_evidence: Option<ProbeEvidence>,
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


    /// Check only the exact retired invocation for read-only post-probe acquisition.
    ///
    /// This check is never used by input, route proof or the transport guard.
    fn probe_evidence_active(&self) -> Result<(), String> {
        if !self.probe_delivered {
            return Err("No acknowledged motion probe for diagnostic acquisition".into());
        }
        self.trace.check(
            &self.events.latest_frame.reconnaissance,
            &self.plan.context,
            self.cancel,
        )?;
        let store = self
            .events
            .latest_frame
            .freecell_route
            .lock()
            .map_err(|_| "Free Cell probe owner store poisoned")?;
        if store.running.as_ref() != Some(&(self.plan.clone(), self.run_id)) || store.live.is_some()
        {
            return Err("Free Cell retired probe owner changed".into());
        }
        Ok(())
    }


    /// Retire input immediately after ACK, then acquire one explicitly diagnostic frame.
    fn finish_movement_probe(&mut self) -> Result<route::TransferOutcome, String> {
        self.probe_delivered = true;
        self.trace.check(
            &self.events.latest_frame.reconnaissance,
            &self.plan.context,
            self.cancel,
        )?;
        {
            let mut store = self
                .events
                .latest_frame
                .freecell_route
                .lock()
                .map_err(|_| "Free Cell probe retirement store poisoned")?;
            store.check(self.plan, self.run_id)?;
            store.retire_live();
        }
        self.probe_evidence_active()?;
        crate::single_step_telemetry::record_freecell(|| {
            serde_json::json!({
                "event":"motion-probe-input-retired", "request_id":self.plan.request_id,
                "run_id":self.run_id,"route_cursor_advanced":false,"effect_proved":false,
                "after_capture_intent":"read-only motion diagnostic; not route endpoint proof"
            })
        });
        if cancellable_wait(self.settle, self.cancel).is_err() {
            self.probe_evidence_active()?;
            return Err("Free Cell diagnostic acquisition wait interrupted".into());
        }
        self.probe_evidence_active()?;
        let (frame, _) =
            capture_screen_causal(&mut self.qmp, &self.plan.context.socket_path, self.events)
                .map_err(|error| {
                    self.trace.capture_error(
                        error,
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
        self.probe_evidence_active()?;
        if let Some(frame) = &self.latest {
            self.events.latest_frame.publish(
                frame.clone(),
                None,
                false,
                true,
                Some(self.plan.context.clone()),
            );
        }
        if self.retire_if_complete()? {
            return Ok(route::TransferOutcome::ChallengeComplete);
        }
        let saved = crate::pyramid_diagnostics::persist_failure(
            crate::pyramid_diagnostics::FailureEvidence {
                progress: None,
                context: &self.plan.context,
                phase: crate::pyramid_diagnostics::PyramidPhase::Execution,
                run_id: Some(self.run_id),
                error: "Free Cell REL motion probe acquisition; no route effect claimed",
                details: serde_json::json!({"artifact_kind":"successful motion-probe acquisition evidence",
                "probe":self.details(),"route_cursor_advanced":false,"effect_proved":false}),
                source_document: None,
                first_frame: None,
                latest_frame: self.latest.as_ref(),
            },
        );
        self.probe_evidence = Some(ProbeEvidence {
            capture_serial: self.capture_serial,
            directory: saved.directory,
            persistence_error: saved.persistence_error,
        });
        self.probe_evidence_active()?;
        Ok(route::TransferOutcome::MovementProbe)
    }


    /// Prove only the requested exposed piles; hidden identities remain in authenticated history.
    fn prove(&self, expected: &Position, piles: &[Pile]) -> Result<position::Geometry, String> {
        let frame = self
            .latest
            .as_ref()
            .ok_or("Missing Free Cell route frame")?;
        position::verify_piles(frame, expected, piles, &mut || self.active())
    }


    /// Retire on the game's supported completion report without inferring route effects.
    ///
    /// Initial identity binding is mandatory. This passive stop never grants input,
    /// advances the cursor, infers an unread objective or starts another board.
    fn retire_if_complete(&mut self) -> Result<bool, String> {
        if !self.completion_bound {
            return Ok(false);
        }
        let complete = crate::freecell_completion::observe(
            self.latest
                .as_ref()
                .ok_or("Missing Free Cell completion frame")?,
        )?;
        if !complete {
            return Ok(false);
        }
        if self.probe_delivered {
            self.probe_evidence_active()?;
        } else {
            self.active()?;
        }
        let mut store = self
            .events
            .latest_frame
            .freecell_route
            .lock()
            .map_err(|_| "Free Cell terminal retirement store poisoned".to_owned())?;
        self.trace.check(
            &self.events.latest_frame.reconnaissance,
            &self.plan.context,
            self.cancel,
        )?;
        if self.probe_delivered {
            if store.running.as_ref() != Some(&(self.plan.clone(), self.run_id))
                || store.live.is_some()
            {
                return Err("Free Cell retired probe terminal owner changed".into());
            }
        } else {
            store.check(self.plan, self.run_id)?;
        }
        store.revoke();
        self.completion_observed = true;
        drop(store);
        crate::single_step_telemetry::record_freecell(|| {
            serde_json::json!({
                "event":"game-reported-challenge-complete", "run_id":self.run_id,
                "request_id":self.plan.request_id, "capture_serial":self.capture_serial,
                "stage":self.stage.map(stage_details), "remaining_route_retired":true,
                "route_cursor_advanced":false, "objective_counter_inferred":false,
            })
        });
        // The causal TCP acquisition has cached these exact terminal pixels when
        // recording is enabled. Save that entry only, after authority retirement;
        // no new acquisition, Continue or transition input belongs to this scope.
        {
            let _game_win_images = super::begin_game_win_images(self.events)?;
        }
        Ok(true)
    }


    /// Persist already-acquired incident pixels without replacing the original failure.
    fn details(&self) -> serde_json::Value {
        let stage = self.stage.map(stage_details);
        serde_json::json!({"game":"Free Cell","protocol":if self.diagnostic_step && self.drag_mode==route::DragMode::Relative {"bounded relative motion probe"}else{"explicit held drag"},
            "request_id":self.plan.request_id,"source_generation":self.plan.source_generation,
            "run_id":self.run_id,"stage":stage,"capture_serial":self.capture_serial,
            "complete_initial_identity_bound":self.completion_bound,
            "game_reported_challenge_complete":self.completion_observed,
            "motion_probe_delivered":self.probe_delivered,"diagnostic_drag_mode":format!("{:?}",self.drag_mode),
            "diagnostic_drag_timing":format!("{:?}",self.drag_timing),
            "movement_probe_evidence":self.probe_evidence.as_ref().map(|e| serde_json::json!({"capture_serial":e.capture_serial,"directory":e.directory,"persistence_error":e.persistence_error})),
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
        let observed = crate::freecell_recon::recognise_playback_initial_checked(
            frame,
            deal.metadata().clone(),
            &mut || self.active(),
        )
        .map_err(|e| e.to_string())?;
        crate::freecell_recon::objective::require_acquired(observed.metadata())?;
        if !route::same_physical(&Position::initial(deal), &Position::initial(&observed)) {
            return Err(
                "Live Free Cell deal differs from frozen calculation; no input authorised".into(),
            );
        }
        self.active()?;
        self.completion_bound = true;
        Ok(())
    }


    /// Observe one exact physical route prefix, without advancing on unmatched evidence.
    fn observe(&mut self, candidates: &[Candidate]) -> Result<route::Observation, String> {
        self.capture()?;
        if self.retire_if_complete()? {
            return Ok(route::Observation::ChallengeComplete);
        }
        let action = self
            .stage
            .and_then(|stage| stage.action)
            .or_else(|| self.frozen.map(|(action, _, _)| action));
        let piles = observation_piles(candidates, action)?;
        let mut rejections = Vec::new();
        let mut matched = None;
        for (index, candidate) in candidates.iter().enumerate() {
            self.active()?;
            match self.prove(&candidate.position, &piles) {
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
        crate::single_step_telemetry::record_freecell(|| {
            serde_json::json!({
                "event":"physical-position-observation","capture_serial":self.capture_serial,
                "stage":self.stage.map(stage_details),
                "matched_cursor":matched.and_then(|index|candidates.get(index).map(|c|c.cursor)),
                "rejected_candidates":rejections,"endpoint_observation_only":true
            })
        });
        if let Some(index) = matched
            && (index + 1 == candidates.len()
                || self.stage.is_some_and(|stage| stage.phase == "readiness"))
        {
            return Ok(route::Observation::Position(index));
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
        Ok(matched.map_or(route::Observation::Unmatched, route::Observation::Position))
    }


    /// Freeze diagnostic position/action before any fallible stage operation.
    fn stage(&mut self, stage: route::Stage) {
        crate::single_step_telemetry::record_freecell(|| {
            serde_json::json!({
                "event":"playback-stage","stage":stage_details(stage)
            })
        });
        self.stage = Some(stage);
    }


    /// Deliver one explicit-target held drag from freshly proved parent geometry.
    fn transfer(
        &mut self,
        parent: &Position,
        action: RecordedMove,
    ) -> Result<route::TransferOutcome, String> {
        self.capture()?;
        if self.retire_if_complete()? {
            return Ok(route::TransferOutcome::ChallengeComplete);
        }
        let geometry = self.prove(parent, &[action.from, action.to])?;
        let from = geometry.point(action.from)?;
        let observed_to = geometry.point(action.to)?;
        let held_points = if self.diagnostic_step && self.drag_mode == route::DragMode::Absolute {
            diagnostic_abs_path(from, observed_to, action.to, (1_920, 1_080))?
        } else {
            [observed_to; 4]
        };
        let [_, _, _, to] = held_points;
        self.frozen = Some((action, from, to));
        if self.diagnostic_step && self.drag_mode == route::DragMode::Absolute {
            crate::single_step_telemetry::record_freecell(|| {
                serde_json::json!({"event":"diagnostic-abs-native-path-planned",
                    "source":[from.x,from.y],"observed_destination":[observed_to.x,observed_to.y],
                    "effective_destination":[to.x,to.y],
                    "held_points":held_points.map(|point|[point.x,point.y]),
                    "vertical_lift_native_pixels":DIAGNOSTIC_LIFT_PIXELS,
                    "midpoint_contract":"floor((original source + effective destination) / 2) independently on each native axis",
                    "held_point_roles":["source-minus-45-y","native-midpoint","destination-minus-45-y","exact-destination"],
                    "sequence_is":"planned; wire records identify phases reached","effect_proved":false})
            });
        }
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
        let mut permitted = || {
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
        };
        let delivered = if self.diagnostic_step {
            self.qmp.diagnostic_drag(
                from,
                &held_points,
                (1_920, 1_080),
                self.drag_mode,
                self.drag_timing,
                &mut permitted,
            )
        } else {
            self.qmp.drag(from, to, 1_920, 1_080, &mut permitted)
        };
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
        if self.diagnostic_step && self.drag_mode == route::DragMode::Relative {
            return self.finish_movement_probe();
        }
        let _ = self.events.send(WorkerEvent::Log(format!(
            "Free Cell drag {action:?}: {from:?} -> {to:?} acknowledged; card effect requires fresh semantic proof"
        )));
        self.active()?;
        Ok(route::TransferOutcome::Delivered)
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
    fn progress(&mut self, cursor: usize, total: usize, position: &Position) {
        crate::single_step_telemetry::record_freecell(|| {
            serde_json::json!({
                "event":"verified-playback-progress","request_id":self.plan.request_id,
                "run_id":self.run_id,"cursor":cursor,"total":total,
                "attempted":self.counts.0,"acknowledged":self.counts.1,
                "evidence":"semantic position policy, never QMP ACK alone"
            })
        });
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
                foundation_cards: position.foundation_cards(),
                completion: None,
                probe_evidence: None,
            });
        }
    }
}


/// Select the next transfer endpoints and every pile changed by its recorded AUTO follow-ups.
///
/// Candidates are already independently replayed physical positions. Untouched columns
/// supply no move-effect evidence and are not scanned for decorative or hidden-card pixels.
fn observation_piles(
    candidates: &[Candidate],
    action: Option<RecordedMove>,
) -> Result<Vec<Pile>, String> {
    let first = candidates
        .first()
        .ok_or("Missing Free Cell observation candidates")?;
    let action = action.ok_or("Missing Free Cell observation action")?;
    let mut piles = vec![action.from];
    if action.to != action.from {
        piles.push(action.to);
    }
    for candidate in candidates.iter().skip(1) {
        for column in 0_u8..8 {
            let pile = Pile::Cascade(column);
            if first.position.column(usize::from(column))
                != candidate.position.column(usize::from(column))
                && !piles.contains(&pile)
            {
                piles.push(pile);
            }
        }
        for cell in 0_u8..4 {
            let pile = Pile::Cell(cell);
            if first.position.cells()[usize::from(cell)]
                != candidate.position.cells()[usize::from(cell)]
                && !piles.contains(&pile)
            {
                piles.push(pile);
            }
        }
        for suit in 0_u8..4 {
            let pile = Pile::Foundation(suit);
            if first.position.foundations()[usize::from(suit)]
                != candidate.position.foundations()[usize::from(suit)]
                && !piles.contains(&pile)
            {
                piles.push(pile);
            }
        }
    }
    Ok(piles)
}


/// Execute one reserved invocation, permanently retiring on any refusal or interruption.
pub(super) fn run(
    plan: freecell_calculation::Identity,
    run_id: u64,
    single_step: bool,
    diagnostic: (route::DragMode, route::DiagnosticDragTiming),
    settle: Duration,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) {
    let (drag_mode, drag_timing) = diagnostic;
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
        let (cached, live) = {
            let store = events
                .latest_frame
                .freecell_route
                .lock()
                .map_err(|_| "Free Cell route store poisoned")?;
            if store.running.as_ref() != Some(&(plan.clone(), run_id)) {
                return Err("Free Cell playback owner retired".into());
            }
            let cached = store
                .cached
                .as_ref()
                .filter(|cached| cached.plan == plan)
                .cloned()
                .ok_or("Free Cell plan retired")?;
            let live = store
                .live
                .as_ref()
                .cloned()
                .ok_or("Free Cell live history retired")?;
            (cached, live)
        };
        let mut current = live.state;
        let (moves, target) = recorded_route(&cached.report.outcome)?;
        evidence.0 = serde_json::json!({"game":"Free Cell", "run_id":run_id,
            "request_id":plan.request_id,"source_generation":plan.source_generation,
            "stage":stage_details(route::Stage {cursor:current.cursor,total:moves.len(),
                action:moves.get(current.cursor).copied(),mode:live.mode,phase:"before transport",observation:0}),
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
            completion_bound: current.bound,
            completion_observed: false,
            drag_mode,
            drag_timing,
            probe_delivered: false,
            probe_evidence: None,
            diagnostic_step: single_step
                && plan.context.game_mode == GameMode::FreeCell
                && plan.context.strategy == SolvingStrategy::ShortestPath,
        };
        let result = if native.diagnostic_step && drag_mode == route::DragMode::Relative {
            route::movement_probe(
                &mut current,
                &cached.report.deal,
                moves,
                live.mode,
                &mut native,
            )
        } else {
            route::drive(
                &mut current,
                &cached.report.deal,
                moves,
                target,
                single_step,
                live.mode,
                &mut native,
            )
        };
        state = Some(current);
        balanced = native.counts.0 == native.counts.1;
        evidence = (
            native.details(),
            native.first.clone(),
            native.latest.clone(),
        );
        result.map(|completion| Progress {
            plan: plan.clone(),
            run_id,
            cursor: current.cursor,
            total: moves.len(),
            attempted: native.counts.0,
            acknowledged: native.counts.1,
            foundation_cards: current.position.foundation_cards(),
            completion: Some(completion),
            probe_evidence: native.probe_evidence.clone(),
        })
    })();
    // Order the final typed authority check with exact Store retirement. STOP latches
    // its cause before waiting to revoke this same store; unrelated revocation stays fatal.
    let mut result = result;
    if result.is_err() {
        // A later STOP revokes the cache without reclassifying an earlier genuine failure.
        trace.adopt(trace.termination(true, balanced));
    }
    let retirement = events
        .latest_frame
        .freecell_route
        .lock()
        .map_err(|_| "Free Cell route retirement poisoned".to_owned())
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
                state,
                result
                    .as_ref()
                    .ok()
                    .and_then(|progress| progress.completion),
            )
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
