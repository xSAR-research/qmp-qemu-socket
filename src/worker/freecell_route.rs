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


/// Approved input-free spacing before the second matching endpoint proof.
const ENDPOINT_CONFIRMATION_MAX_WAIT: Duration = Duration::from_millis(100);


/// Approved first effect-observation wait; unsupported pixels retain the configured retry.
const POST_TRANSFER_MAX_WAIT: Duration = Duration::from_millis(200);


/// Plan three native ABS held targets after proof, without recognising or sending input.
///
/// Every pile retains its authenticated point, with the same pickup inset from
/// the proved corner or empty-slot origin. Held points are source minus
/// 45px vertically, the floor midpoint of the original source/effective target,
/// then the exact target. Only the final point must stay inside the destination
/// slot. Source and observed target remain
/// bounds-checked proof coordinates; every generated point must stay in frame.
/// Invalid piles, overflow or any point outside the frame fail without fallback.
fn diagnostic_abs_path(
    source: PixelPoint,
    observed_destination: PixelPoint,
    target: Pile,
    frame: (u32, u32),
) -> Result<[PixelPoint; 3], String> {
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
    let destination = observed_destination;
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
    let points = [source_upper, midpoint, destination];
    for point in [source, observed_destination].into_iter().chain(points) {
        crate::geometry::pixel_point_to_qmp(point, frame.0, frame.1)
            .map_err(|error| format!("Invalid Free Cell diagnostic native point: {error}"))?;
    }
    if slot.is_some_and(|rect| !rect.contains(destination)) {
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
    /// Exact invocation's request to finish at a verified step, never immediate cancellation.
    stop_after_step: bool,
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
        self.stop_after_step = false;
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
        self.stop_after_step = false;
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
        self.stop_after_step = false;
        self.progress = None;
        Ok(next)
    }


    /// Consume coalesced progress without granting authority.
    pub(super) fn take_progress(&mut self) -> Option<Progress> {
        self.progress.take()
    }


    /// Set an idempotent clean-boundary request only for this exact active owner.
    pub(super) fn request_stop_after_step(
        &mut self,
        plan: &freecell_calculation::Identity,
        run_id: u64,
    ) -> Result<(), String> {
        self.check(plan, run_id)?;
        self.stop_after_step = true;
        Ok(())
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
        self.stop_after_step = false;
        // A positive terminal observation has already destroyed the live permit;
        // preserve only its exact queued completion owner until this point.
        if matches!(completion, Some(route::Completion::ChallengeComplete)) {
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
                route::Completion::ChallengeComplete => {
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
    /// True only when running a challenge deal (`Objective::Unresolved`); ordinary `FullBoard` deals stay `false`.
    challenge_mode: bool,
    /// Positive common terminal report remains true even if recording later fails.
    completion_observed: bool,
    /// Every A* Free Cell playback request uses the guarded native gesture.
    diagnostic_step: bool,
}


impl Native<'_> {
    /// Wait without acquiring pixels, preserving STOP and the exact invocation fence.
    ///
    /// `duration` is selected from the configured retry interval and the explicit
    /// confirmation/post-transfer caps. Telemetry distinguishes that intentional
    /// wait from the subsequent acquisition and physical-position proof.
    fn wait_for_frame(&mut self, duration: Duration, cadence: &'static str) -> Result<(), String> {
        self.active()?;
        let started = Instant::now();
        let waited = cancellable_wait(duration, self.cancel);
        crate::single_step_telemetry::record_freecell(|| {
            serde_json::json!({
                "event":"playback-wait","cadence":cadence,
                "configured_ms":self.settle.as_millis(),"chosen_ms":duration.as_millis(),
                "elapsed_us":started.elapsed().as_micros(),"interrupted":waited.is_err(),
                "stage":self.stage.map(stage_details),"additional_captures":0
            })
        });
        if waited.is_err() {
            self.active()?;
            return Err("Free Cell settling interrupted".into());
        }
        self.active()
    }


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


    /// Prove only the requested exposed piles; hidden identities remain in authenticated history.
    fn prove(&self, expected: &Position, piles: &[Pile]) -> Result<position::Geometry, String> {
        let frame = self
            .latest
            .as_ref()
            .ok_or("Missing Free Cell route frame")?;
        position::verify_piles(frame, expected, piles, &mut || self.active())
    }


    /// Poll Challenge Complete (`wait_for_challenge_complete`) only in bound challenge mode after fault, then retire.
    ///
    /// If the 5-frame poll ends without `Continue`, inspect the latest captured frame once for an
    /// active Challenge XP-counting modal or a settled `LEVEL-UP! [OK]` modal (`Option 1A`) and
    /// retire cleanly without input; otherwise preserve `self.first` and the original fault.
    fn poll_and_retire_challenge_complete(&mut self) -> Result<bool, String> {
        if !self.completion_bound || !self.challenge_mode {
            return Ok(false);
        }
        let complete = {
            let mut io = ChallengePollIo {
                native: self,
                error_detail: None,
            };
            match crate::game_controls::wait_for_challenge_complete(&mut io) {
                Ok(complete) => complete,
                Err(error) => return Err(io.error_detail.unwrap_or_else(|| error.to_string())),
            }
        };
        if complete {
            self.retire_challenge_complete("continue_visible")?;
            return Ok(true);
        }
        if let Some(frame) = self.latest.as_ref()
            && let Some(handoff) = crate::freecell_terminal::classify_challenge_handoff(frame)
                .map_err(|error| error.to_string())?
        {
            let (reason, message) = match handoff {
                crate::freecell_terminal::ChallengeTerminalHandoff::XpCounting => (
                    "xp_counting",
                    "Free Cell challenge complete (XP counting active after 5 observations); releasing control without input",
                ),
                crate::freecell_terminal::ChallengeTerminalHandoff::LevelUp => (
                    "level_up_ok",
                    "Free Cell challenge complete (Level Up OK modal active); releasing control to operator without input",
                ),
            };
            let _ = self.events.send(WorkerEvent::Log(message.to_owned()));
            self.retire_challenge_complete(reason)?;
            return Ok(true);
        }
        Ok(false)
    }


    /// Retire route authority and save optional GAME WIN image after Challenge Complete is verified.
    ///
    /// Initial identity binding and challenge mode are mandatory. This passive stop never grants
    /// input, advances the cursor, infers an unread objective or starts another board.
    fn retire_challenge_complete(&mut self, terminal_handoff: &'static str) -> Result<(), String> {
        self.active()?;
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
        store.check(self.plan, self.run_id)?;
        store.revoke();
        self.completion_observed = true;
        drop(store);
        crate::single_step_telemetry::record_freecell(|| {
            serde_json::json!({
                "event":"game-reported-challenge-complete", "run_id":self.run_id,
                "request_id":self.plan.request_id, "capture_serial":self.capture_serial,
                "stage":self.stage.map(stage_details), "terminal_handoff":terminal_handoff,
                "remaining_route_retired":true, "route_cursor_advanced":false,
                "objective_counter_inferred":false,
            })
        });
        // The causal TCP acquisition has cached these exact terminal pixels when
        // recording is enabled. Save that entry only, after authority retirement;
        // no new acquisition, Continue or transition input belongs to this scope.
        {
            let _game_win_images = super::begin_game_win_images(self.events)?;
        }
        Ok(())
    }


    /// Persist already-acquired incident pixels without replacing the original failure.
    fn details(&self) -> serde_json::Value {
        let stage = self.stage.map(stage_details);
        serde_json::json!({"game":"Free Cell","protocol":"explicit held drag",
            "request_id":self.plan.request_id,"source_generation":self.plan.source_generation,
            "run_id":self.run_id,"stage":stage,"capture_serial":self.capture_serial,
            "complete_initial_identity_bound":self.completion_bound,
            "challenge_mode":self.challenge_mode,
            "game_reported_challenge_complete":self.completion_observed,
            "diagnostic_drag_mode":"Absolute",
            "gesture_phase_wait_ms":crate::parameters::FREECELL_ROUTE_ABS_GESTURE_INTERVAL.as_millis(),
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


    /// Read the exact invocation's resumable stop request without entering a pause.
    fn stop_after_step(&mut self) -> Result<bool, String> {
        self.active()?;
        let store = self
            .events
            .latest_frame
            .freecell_route
            .lock()
            .map_err(|_| "Free Cell playback store poisoned".to_owned())?;
        store.check(self.plan, self.run_id)?;
        Ok(store.stop_after_step)
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
        self.challenge_mode = crate::freecell_recon::objective::from_metadata(observed.metadata())
            == crate::freecell_recon::objective::Objective::Unresolved;
        Ok(())
    }


    /// Observe one exact physical route prefix, without advancing on unmatched evidence.
    fn observe(&mut self, candidates: &[Candidate]) -> Result<route::Observation, String> {
        self.capture()?;
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
        let geometry = match self.prove(parent, &[action.from, action.to]) {
            Ok(geometry) => geometry,
            Err(fault) => {
                if self.first.is_none() {
                    self.first = self.latest.clone();
                }
                if self.poll_and_retire_challenge_complete()? {
                    return Ok(route::TransferOutcome::ChallengeComplete);
                }
                return Err(fault);
            }
        };
        let from = geometry.point(action.from)?;
        let observed_to = geometry.point(action.to)?;
        let held_points = if self.diagnostic_step {
            diagnostic_abs_path(from, observed_to, action.to, (1_920, 1_080))?
        } else {
            [observed_to; 3]
        };
        let [_, _, to] = held_points;
        self.frozen = Some((action, from, to));
        if self.diagnostic_step {
            crate::single_step_telemetry::record_freecell(|| {
                serde_json::json!({"event":"diagnostic-abs-native-path-planned",
                    "source":[from.x,from.y],"observed_destination":[observed_to.x,observed_to.y],
                    "effective_destination":[to.x,to.y],
                    "destination_anchor_contract":"retain the proved corner or empty-slot point with the same native pickup inset (+15 x, +20 y)",
                    "held_points":held_points.map(|point|[point.x,point.y]),
                    "vertical_lift_native_pixels":DIAGNOSTIC_LIFT_PIXELS,
                    "midpoint_contract":"floor((original source + effective destination) / 2) independently on each native axis",
                    "held_point_roles":["source-minus-45-y","native-midpoint","exact-destination"],
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
            self.qmp
                .diagnostic_drag(from, &held_points, (1_920, 1_080), &mut permitted)
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
        let _ = self.events.send(WorkerEvent::Log(format!(
            "Free Cell drag {action:?}: {from:?} -> {to:?} acknowledged; card effect requires fresh semantic proof"
        )));
        self.active()?;
        Ok(route::TransferOutcome::Delivered)
    }


    /// Retain the configured retry horizon for unsupported or intermediate pixels.
    fn settle(&mut self) -> Result<(), String> {
        self.wait_for_frame(self.settle, "unmatched-or-intermediate-retry")
    }


    /// Confirm an eligible first proof after at most 100 milliseconds, without input.
    fn confirmation_wait(&mut self) -> Result<(), String> {
        self.wait_for_frame(
            self.settle.min(ENDPOINT_CONFIRMATION_MAX_WAIT),
            "matching-endpoint-confirmation",
        )
    }


    /// Acquire the first post-release effect after at most 200 milliseconds.
    fn post_transfer_wait(&mut self) -> Result<(), String> {
        self.wait_for_frame(
            self.settle.min(POST_TRANSFER_MAX_WAIT),
            "first-post-transfer-observation",
        )
    }


    /// Wait for an AUTO animation while checking STOP, exact ownership and pause each slice.
    fn animation_wait(&mut self, duration: Duration) -> Result<(), String> {
        let started = Instant::now();
        loop {
            self.check()?;
            let elapsed = started.elapsed();
            if elapsed >= duration {
                return Ok(());
            }
            thread::sleep((duration - elapsed).min(CANCELLABLE_WAIT_SLICE));
        }
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
            });
        }
    }


    /// Poll challenge-mode Challenge Complete only after a failed stabilization window.
    fn resolve_challenge_complete(&mut self) -> Result<bool, String> {
        self.poll_and_retire_challenge_complete()
    }


    /// Poll normal-game win entry only after the final 52-foundation endpoint is obscured by GAME WIN.
    fn resolve_normal_game_win(&mut self, goal: &Position) -> Result<bool, String> {
        if !self.completion_bound || self.challenge_mode {
            return Ok(false);
        }
        const FOUNDATIONS: [Pile; 4] = [
            Pile::Foundation(0),
            Pile::Foundation(1),
            Pile::Foundation(2),
            Pile::Foundation(3),
        ];
        let mut consecutive = 0_usize;
        for _ in 0..crate::parameters::POST_GAME_MAX_OBSERVATION_ROUNDS {
            self.check()?;
            self.wait_for_frame(
                crate::parameters::BOARD_TRANSITION_REOBSERVE_DELAY,
                "normal-game-win-poll",
            )?;
            self.capture()?;
            let frame = self
                .latest
                .as_ref()
                .ok_or("Missing Free Cell win-poll frame")?;
            let terminal = crate::freecell_terminal::is_recognised_terminal(frame)
                .map_err(|error| error.to_string())?;
            let matched = terminal || (goal.is_goal() && self.prove(goal, &FOUNDATIONS).is_ok());
            if matched {
                consecutive += 1;
                if consecutive >= 2 {
                    crate::single_step_telemetry::record_freecell(|| {
                        serde_json::json!({
                            "event":"normal-game-win-resolved",
                            "run_id":self.run_id,
                            "request_id":self.plan.request_id,
                            "capture_serial":self.capture_serial,
                            "terminal":terminal,
                        })
                    });
                    return Ok(true);
                }
            } else {
                consecutive = 0;
            }
        }
        Ok(false)
    }
}


/// Input-free `OperationIo` bridge driving `wait_for_challenge_complete` without mutating `first`.
struct ChallengePollIo<'a, 'b> {
    /// Active Free Cell route worker adapter.
    native: &'a mut Native<'b>,
    /// Exact revocation/transport/wait cause preserved across `OperationError`.
    error_detail: Option<String>,
}


impl ChallengePollIo<'_, '_> {
    /// Verify active run ownership.
    fn check_native_active(&mut self) -> Result<(), crate::game_operations::OperationError> {
        self.native.active().map_err(|cause| {
            self.error_detail = Some(cause);
            crate::game_operations::OperationError::Cancelled
        })
    }
}


impl crate::game_operations::OperationIo for ChallengePollIo<'_, '_> {
    fn check_active(&mut self) -> Result<(), crate::game_operations::OperationError> {
        self.check_native_active()
    }


    fn capture(&mut self) -> Result<CapturedFrame, crate::game_operations::OperationError> {
        self.check_native_active()?;
        let (frame, _) = capture_screen_causal(
            &mut self.native.qmp,
            &self.native.plan.context.socket_path,
            self.native.events,
        )
        .map_err(|error| {
            let detail = self.native.trace.capture_error(
                error,
                &self.native.events.latest_frame.reconnaissance,
                &self.native.plan.context,
                self.native.cancel,
            );
            self.error_detail = Some(detail.clone());
            crate::game_operations::OperationError::Transport(detail)
        })?;
        self.native.capture_serial =
            self.native.capture_serial.checked_add(1).ok_or_else(|| {
                let msg = "Free Cell capture serial exhausted".to_owned();
                self.error_detail = Some(msg.clone());
                crate::game_operations::OperationError::Invalid(msg)
            })?;
        self.native.latest = Some(frame.clone());
        self.native.events.latest_frame.publish(
            frame.clone(),
            None,
            false,
            true,
            Some(self.native.plan.context.clone()),
        );
        self.check_native_active()?;
        Ok(frame)
    }


    fn authorise(
        &mut self,
        _frame: &CapturedFrame,
        _input: crate::game_operations::Input,
    ) -> Result<(), crate::game_operations::OperationError> {
        Err(crate::game_operations::OperationError::Unsupported(
            "Challenge Complete polling never authorises guest input".to_owned(),
        ))
    }


    fn probe(&mut self) -> Result<(), crate::game_operations::OperationError> {
        Err(crate::game_operations::OperationError::Unsupported(
            "Challenge Complete polling never probes guest input".to_owned(),
        ))
    }


    fn deliver(
        &mut self,
        _input: crate::game_operations::Input,
    ) -> Result<(), crate::game_operations::OperationError> {
        Err(crate::game_operations::OperationError::Unsupported(
            "Challenge Complete polling never delivers guest input".to_owned(),
        ))
    }


    fn wait(&mut self, duration: Duration) -> Result<(), crate::game_operations::OperationError> {
        self.native
            .wait_for_frame(duration, "challenge-complete-poll")
            .map_err(|cause| {
                self.error_detail = Some(cause);
                crate::game_operations::OperationError::Cancelled
            })?;
        self.check_native_active()
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


/// Outcome and final captured evidence returned by executing one reserved Free Cell route.
pub(super) struct SessionExecution {
    /// Route progress on completion, or the structured failure packet on error.
    pub(super) result: Result<Progress, Box<Failure>>,
    /// Classified termination reason (`Completed`, `Stopped`, `Paused`, or `Failed`).
    pub(super) termination: crate::pyramid_workflow_model::RunTermination,
    /// Most recent verified guest frame observed during execution, when available.
    pub(super) latest_frame: Option<CapturedFrame>,
}

/// Execute one reserved Free Cell route and return its structured outcome without
/// retiring the outer reconnaissance session or emitting a standalone UI finish event.
pub(super) fn execute_for_session(
    plan: freecell_calculation::Identity,
    run_id: u64,
    single_step: bool,
    settle: Duration,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> SessionExecution {
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
            challenge_mode: crate::freecell_recon::objective::from_metadata(
                cached.report.deal.metadata(),
            ) == crate::freecell_recon::objective::Objective::Unresolved,
            completion_observed: false,
            diagnostic_step: plan.context.game_mode == GameMode::FreeCell
                && plan.context.strategy == SolvingStrategy::ShortestPath,
        };
        let result = route::drive(
            &mut current,
            &cached.report.deal,
            moves,
            target,
            single_step,
            live.mode,
            &mut native,
        );
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
    SessionExecution {
        result,
        termination,
        latest_frame: evidence.2,
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
    let SessionExecution {
        result,
        termination,
        ..
    } = execute_for_session(plan.clone(), run_id, single_step, settle, events, cancel);
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
