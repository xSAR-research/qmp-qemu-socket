//! Independent Pyramid modes, observed header facts and visible operating logs.
//!
//! This module owns presentation and single-press consent only. The serial worker
//! retains acquisition receipts, route permits, input checks and diagnostics I/O.

use super::*;
use crate::pyramid_diagnostics::{PyramidDiagnostic, PyramidPhase};
use crate::pyramid_workflow_model::{
    ObservedGame, PlanningProgress, RouteGoal, RouteMethod, SessionCompletion, SessionPhase,
    SessionProgress, SessionRequest, StopCondition, WorkflowMode,
};


/// Advisory identity of the worker-owned automatic session, never an input permit.
#[derive(Clone, Copy, Debug)]
struct SessionDisplay {
    /// Checked UI request identity retained through all worker phases.
    run_id: u64,
    /// Latest worker-established board identity; zero precedes the first acquisition.
    board_generation: u64,
    /// Most recent worker phase, independent of generic connection states.
    phase: SessionPhase,
}


/// Exact statistics owner, retained through completion for the final work display.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PlanningIdentity {
    /// Current manual calculation or automatic-session identity.
    run_id: u64,
    /// Zero for manual planning; worker-assigned generation for automatic planning.
    board_generation: u64,
    /// Immutable method selected for this calculation.
    method: RouteMethod,
}


/// Advisory workflow display; none of these fields grants guest-input authority.
#[derive(Default)]
pub(super) struct PyramidWorkflow {
    /// Idle-selected lifetime; context changes preserve this preference only.
    mode: WorkflowMode,
    /// Idle-selected search preference; startup uses Shortest route.
    method: RouteMethod,
    /// Explicit operator preference takes precedence over an advisory header default.
    method_explicit: bool,
    /// Idle-selected Continuous lifetime, distinct from the calculated route outcome.
    stop_condition: StopCondition,
    /// Context whose progress and error are displayed.
    context: Option<ControllerContext>,
    /// Most recently requested explicit step.
    phase: Option<PyramidPhase>,
    /// Complete backend or local failure, including persistence outcome.
    diagnostic: Option<PyramidDiagnostic>,
    /// Literal request fields captured before dispatch, before automatic rotation.
    request_details: Option<serde_json::Value>,
    /// Buffered socket override, applied only after the operator finishes editing.
    socket_edit: Option<String>,
    /// Last allocated local diagnostic identity, retained across context changes.
    last_ui_failure_id: u64,
    /// Only this outstanding local packet may update the currently displayed failure.
    expected_ui_failure_id: Option<u64>,
    /// Automatic owner mirrored only for exact advisory event matching.
    session: Option<SessionDisplay>,
    /// Only matching snapshots may replace the current statistics display.
    planning_identity: Option<PlanningIdentity>,
    /// Most recent matching search snapshot, without an accumulating UI queue.
    planning: Option<PlanningProgress>,
    /// Current context/board's advisory measured header, never the selected stopping policy.
    observed_game: ObservedGame,
}


impl PyramidWorkflow {
    /// Clear advisory state when the controller identity changes.
    pub(super) fn bind(&mut self, context: &ControllerContext) {
        if self.context.as_ref() != Some(context) {
            let last_ui_failure_id = self.last_ui_failure_id;
            *self = Self {
                context: Some(context.clone()),
                last_ui_failure_id,
                mode: self.mode,
                method: self.method,
                stop_condition: self.stop_condition,
                ..Self::default()
            };
        }
    }


    /// Replace advisory packet fields with the literal values prepared for dispatch.
    pub(super) fn record_request_details(&mut self, details: serde_json::Value) {
        self.request_details = Some(details);
    }


    /// Begin one explicitly requested step and discard the previous error display.
    pub(super) fn begin(
        &mut self,
        context: &ControllerContext,
        phase: PyramidPhase,
        details: serde_json::Value,
    ) {
        self.bind(context);
        self.phase = Some(phase);
        self.diagnostic = None;
        self.socket_edit = None;
        self.expected_ui_failure_id = None;
        self.request_details = Some(details);
        self.session = None;
        self.planning_identity = None;
        self.planning = None;
        if phase == PyramidPhase::Reconnaissance {
            self.observed_game = ObservedGame::Unknown;
        }
    }


    /// Accept only advisory facts after the caller has authenticated the matching acquisition.
    pub(super) fn apply_observed(&mut self, observed: ObservedGame) {
        self.observed_game = observed;
    }


    /// Return current matched advisory header facts without granting authority.
    pub(super) fn observed_game(&self) -> ObservedGame {
        self.observed_game
    }


    /// Return the selected session lifetime without affecting worker authority.
    pub(super) fn mode(&self) -> WorkflowMode {
        self.mode
    }


    /// Return the search preference fixed by the next explicit dispatch.
    #[cfg(test)]
    pub(super) fn method(&self) -> RouteMethod {
        self.method
    }


    /// Return the intended route outcome fixed by the next dispatch.
    pub(super) fn route_goal(&self) -> RouteGoal {
        self.method.route_goal()
    }


    /// Borrow the sole current or completed search snapshot for concise presentation.
    pub(super) fn planning_progress(&self) -> Option<&PlanningProgress> {
        self.planning.as_ref()
    }


    /// Change idle preferences without changing the current board's controller identity.
    ///
    /// Returns the unchanged generation for a real idle change. Plans are retired
    /// separately; physical controller changes and STOP still revoke acquisition.
    #[cfg(test)]
    pub(super) fn select_options(
        &mut self,
        mode: WorkflowMode,
        method: RouteMethod,
        generation: u64,
        idle: bool,
    ) -> Result<Option<u64>, &'static str> {
        self.select_preferences(mode, method, self.stop_condition, generation, idle)
    }


    /// Replace idle planning preferences while preserving the Recon context generation.
    pub(super) fn select_preferences(
        &mut self,
        mode: WorkflowMode,
        method: RouteMethod,
        stop_condition: StopCondition,
        generation: u64,
        idle: bool,
    ) -> Result<Option<u64>, &'static str> {
        if !idle
            || self.session.is_some()
            || (self.mode == mode && self.method == method && self.stop_condition == stop_condition)
        {
            return Ok(None);
        }
        self.mode = mode;
        if self.method != method {
            self.method_explicit = true;
        }
        self.method = method;
        self.stop_condition = stop_condition;
        Ok(Some(generation))
    }


    /// Apply an authenticated idle Recon default without replacing an explicit choice.
    pub(super) fn apply_acquisition_default(&mut self, method: Option<RouteMethod>) {
        if !self.method_explicit
            && !(self.mode == WorkflowMode::Continuous
                && self.stop_condition == StopCondition::NoChallenge)
            && let Some(method) = method
        {
            self.method = method;
        }
    }


    /// Bind manual search snapshots before queueing that exact input-free calculation.
    pub(super) fn begin_planning(&mut self, context: &ControllerContext, run_id: u64) {
        self.bind(context);
        self.planning_identity = Some(PlanningIdentity {
            run_id,
            board_generation: 0,
            method: self.method,
        });
        self.planning = None;
    }


    /// Mirror one explicit automatic dispatch while leaving all phase sequencing to worker.
    pub(super) fn begin_session(
        &mut self,
        context: &ControllerContext,
        run_id: u64,
        details: serde_json::Value,
    ) {
        self.begin(context, PyramidPhase::Reconnaissance, details);
        self.session = Some(SessionDisplay {
            run_id,
            board_generation: 0,
            phase: SessionPhase::Entry,
        });
    }


    /// Admit only current automatic progress without releasing ownership between phases.
    pub(super) fn receive_session_progress(
        &mut self,
        progress: &SessionProgress,
        context: &ControllerContext,
        owner: Option<ActiveRun>,
        active_run_id: Option<u64>,
    ) -> bool {
        if self.context.as_ref() != Some(context)
            || progress.context != *context
            || owner != Some(ActiveRun::PyramidSession)
            || active_run_id != Some(progress.run_id)
        {
            return false;
        }
        let Some(session) = self.session.as_mut().filter(|session| {
            session.run_id == progress.run_id
                && progress.board_generation >= session.board_generation
        }) else {
            return false;
        };
        // One board can move only from Exact to its automatic Maximum fallback.
        // A delayed ordered Exact phase must not overwrite newer coalesced stats.
        if progress.board_generation == session.board_generation
            && self.method == RouteMethod::Fast
            && progress.effective_method == RouteMethod::FewestActions
            && self
                .planning_identity
                .is_some_and(|identity| identity.board_generation == progress.board_generation)
        {
            return false;
        }
        if progress.board_generation != session.board_generation {
            if progress.board_generation > 1 {
                self.method_explicit = false;
            }
            self.planning_identity = None;
            self.planning = None;
        }
        session.board_generation = progress.board_generation;
        session.phase = progress.phase;
        self.method = progress.effective_method;
        self.observed_game = progress.observed_game;
        self.phase = Some(session_diagnostic_phase(progress.phase));
        if progress.phase == SessionPhase::Planning {
            let identity = PlanningIdentity {
                run_id: progress.run_id,
                board_generation: progress.board_generation,
                method: self.method,
            };
            if self.planning_identity != Some(identity) {
                self.planning_identity = Some(identity);
                self.planning = None;
            }
        }
        true
    }


    /// Keep one snapshot only when context, request, board and method all still match.
    ///
    /// Automatic statistics may arrive through their bounded slot before the
    /// corresponding ordered phase event. A matching active session may advance
    /// its advisory board identity; older generations then remain stale. No
    /// snapshot creates a session or changes worker ownership or input authority.
    /// A matching final snapshot remains displayable after its completion event;
    /// a new step or context clears its identity before any subsequent request.
    pub(super) fn receive_planning(
        &mut self,
        progress: PlanningProgress,
        context: &ControllerContext,
    ) -> bool {
        let identity = PlanningIdentity {
            run_id: progress.run_id,
            board_generation: progress.board_generation,
            method: progress.method,
        };
        if self.context.as_ref() != Some(context)
            || progress.context != *context
            || (self.planning_identity == Some(identity)
                && self.planning.as_ref().is_some_and(|previous| {
                    previous.stats.elapsed > progress.stats.elapsed
                        || (previous.finished && !progress.finished)
                }))
        {
            return false;
        }
        if let Some(session) = self.session.as_mut() {
            if session.run_id != progress.run_id
                || progress.board_generation == 0
                || progress.board_generation < session.board_generation
            {
                return false;
            }
            if progress.board_generation == session.board_generation
                && self.method == RouteMethod::Fast
                && progress.method == RouteMethod::FewestActions
                && self
                    .planning_identity
                    .is_some_and(|identity| identity.board_generation == progress.board_generation)
            {
                return false;
            }
            if progress.board_generation > session.board_generation {
                if progress.board_generation > 1 {
                    self.method_explicit = false;
                }
                session.board_generation = progress.board_generation;
                session.phase = SessionPhase::Planning;
                self.phase = Some(PyramidPhase::Planning);
            }
            self.method = progress.method;
        } else if self.planning_identity != Some(identity) {
            let fallback = self.planning_identity.is_some_and(|previous| {
                previous.run_id == progress.run_id
                    && previous.board_generation == 0
                    && progress.board_generation == 0
                    && previous.method == RouteMethod::FewestActions
                    && progress.method == RouteMethod::Fast
            });
            if !fallback {
                return false;
            }
            self.method = progress.method;
        }
        self.planning_identity = Some(identity);
        self.observed_game = progress.observed_game;
        self.planning = Some(progress);
        true
    }


    /// Accept matching advisory diagnostics without changing run ownership or permits.
    pub(super) fn receive(
        &mut self,
        diagnostic: PyramidDiagnostic,
        context: &ControllerContext,
        active_run_id: Option<u64>,
    ) -> bool {
        if self.context.as_ref() != Some(context)
            || diagnostic.context != *context
            || self.phase != Some(diagnostic.phase)
        {
            return false;
        }
        if let Some(id) = diagnostic.ui_failure_id {
            if self.expected_ui_failure_id != Some(id) {
                return false;
            }
            self.expected_ui_failure_id = None;
        } else if diagnostic
            .run_id
            .is_some_and(|id| active_run_id != Some(id))
            || (diagnostic.run_id.is_none() && active_run_id.is_some())
        {
            return false;
        }
        self.bind(context);
        self.diagnostic = Some(diagnostic);
        true
    }


    /// Whether technical forms and complete output should be visible in this context.
    pub(super) fn details_visible(&self, context: &ControllerContext) -> bool {
        self.context.as_ref() == Some(context) && self.diagnostic.is_some()
    }


    /// Return the last explicitly selected phase for accurate fallback reporting.
    pub(super) fn phase(&self) -> Option<PyramidPhase> {
        self.phase
    }


    /// Allocate a non-reused local diagnostic token without altering guest authority.
    ///
    /// Exhaustion retains the original error and requires restarting for further
    /// local persistence; the counter never wraps or resets on context changes.
    pub(super) fn reserve_failure_id(&mut self) -> Result<u64, &'static str> {
        self.expected_ui_failure_id = None;
        let id = self.last_ui_failure_id.checked_add(1)
            .ok_or("Local diagnostic identities are exhausted; the original error is retained without a new packet.")?;
        self.last_ui_failure_id = id;
        self.expected_ui_failure_id = Some(id);
        Ok(id)
    }


    /// Keep socket typing outside the active context until an explicit Apply action.
    pub(super) fn socket_editor(&mut self, current: &str) -> &mut String {
        self.socket_edit.get_or_insert_with(|| current.to_owned())
    }


    /// Validate a complete socket edit without changing the active context or its error.
    pub(super) fn socket_update(&self, current: &str) -> Result<Option<String>, &'static str> {
        let Some(pending) = self
            .socket_edit
            .as_ref()
            .filter(|pending| pending.as_str() != current)
        else {
            return Ok(None);
        };
        if !reconnaissance_path_is_valid(pending) {
            return Err(
                "QMP socket path must contain 1–4096 UTF-8 bytes without control characters.",
            );
        }
        Ok(Some(pending.clone()))
    }
}


/// Availability of the three explicit steps; the worker remains authoritative.
pub(super) struct PyramidStepGates {
    /// An idle controller may request a new acquisition.
    pub(super) recon: bool,
    /// Input-free planning requires a successful acquisition in this context.
    pub(super) plan: bool,
    /// Execution requires an independently calculated plan in this context.
    pub(super) solve: bool,
}


/// Apply existing idle/STOP/snapshot gates before considering acquisition or route state.
pub(super) fn step_gates(
    context: &ControllerContext,
    idle: bool,
    route: &ComputedRouteForm,
) -> PyramidStepGates {
    let idle = idle
        && is_workflow_mode(context.game_mode, context.strategy)
        && route.active_run_id.is_none();
    PyramidStepGates {
        recon: idle,
        plan: idle && route.calculation_ready(context),
        solve: idle
            && route.active_run_id.is_none()
            && route
                .plan
                .as_ref()
                .is_some_and(|plan| plan.context == *context),
    }
}


/// Restrict all simplified rendering and controls to independent Pyramid planning.
pub(super) fn is_workflow_mode(game_mode: GameMode, strategy: SolvingStrategy) -> bool {
    game_mode == GameMode::Pyramid && strategy == SolvingStrategy::ShortestPath
}


/// Offer automatic Start while idle; the worker verifies actual standard-series entry.
pub(super) fn session_start_enabled(
    mode: WorkflowMode,
    idle: bool,
    route: &ComputedRouteForm,
) -> bool {
    matches!(mode, WorkflowMode::SingleBoard | WorkflowMode::Continuous)
        && idle
        && route.active_run_id.is_none()
}


/// Allow an idle rescan in every mode; individual Plan and Solve remain Manual-only.
pub(super) fn step_request_enabled(mode: WorkflowMode, phase: PyramidPhase, idle: bool) -> bool {
    idle && (phase == PyramidPhase::Reconnaissance || mode == WorkflowMode::Manual)
}


/// Arm only the immediately requested reconnaissance operation in this exact context.
pub(super) fn arm_recon(form: &mut ReconnaissanceForm, context: &ControllerContext) {
    form.confirmed_context = Some(context.clone());
}


/// Refresh automatic names and consume this explicit press's consent in one request.
///
/// Both Manual Recon and automatic Start use this boundary. Literal overrides stay
/// exact; naming or validation failure returns no request and leaves consent clear.
pub(super) fn prepare_recon_request(
    form: &mut ReconnaissanceForm,
    context: &ControllerContext,
) -> Result<ReconRequest, String> {
    form.prepare_next_acquisition()?;
    arm_recon(form, context);
    form.take_request(context).map_err(str::to_owned)
}


/// Arm only the currently prepared route; stale contexts never gain consent.
pub(super) fn arm_solve(
    form: &mut ComputedRouteForm,
    context: &ControllerContext,
) -> Result<(), &'static str> {
    form.confirmed_context = None;
    if !form
        .plan
        .as_ref()
        .is_some_and(|plan| plan.context == *context)
    {
        return Err(
            "Plan a route from this session's restored acquisition before solving the board.",
        );
    }
    form.confirmed_context = Some(context.clone());
    Ok(())
}


/// One dedicated completion event together with the current cooperative STOP latch.
pub(super) struct RouteCompletion<'a> {
    /// An input-free no-card result retained the current acquisition privately.
    pub(super) retained_recon: bool,
    /// Already context-validated controller identity.
    pub(super) context: ControllerContext,
    /// Outstanding calculation or execution request identity.
    pub(super) run_id: u64,
    /// Successful calculation candidate, absent after execution.
    pub(super) plan_id: Option<u64>,
    /// Worker result, borrowed until the UI completes its update.
    pub(super) result: &'a Result<String, String>,
    /// STOP was requested before this completion was handled.
    pub(super) stopped: bool,
}


/// Complete only the matching request and reconcile its terminal visible lifecycle.
///
/// Failure and STOP retain no route. A previously detached controller stays detached;
/// STOP remains latched until the existing detached-event/completion ordering clears it.
pub(super) fn complete_route(
    form: &mut ComputedRouteForm,
    owner: &mut Option<ActiveRun>,
    state: &mut WorkerState,
    completion: RouteCompletion<'_>,
) -> bool {
    let RouteCompletion {
        retained_recon,
        context,
        run_id,
        plan_id,
        result,
        stopped,
    } = completion;
    if !form.accepts_finished(*owner, run_id) {
        return false;
    }
    form.finish(context.clone(), run_id, plan_id, result, stopped);
    if retained_recon && result.is_ok() && !stopped {
        form.acquired_context = Some(context);
    }
    *owner = None;
    if *state != WorkerState::Detached {
        *state = if result.is_err() || stopped {
            WorkerState::Uncertain
        } else {
            WorkerState::Ready
        };
    }
    true
}


/// Release only the current automatic session, preserving detached and STOP semantics.
pub(super) fn complete_session(
    workflow: &mut PyramidWorkflow,
    form: &mut ComputedRouteForm,
    owner: &mut Option<ActiveRun>,
    state: &mut WorkerState,
    completion: &SessionCompletion,
    context: &ControllerContext,
    stopped: bool,
) -> bool {
    if completion.context != *context
        || workflow.context.as_ref() != Some(context)
        || *owner != Some(ActiveRun::PyramidSession)
        || form.active_run_id != Some(completion.run_id)
        || !workflow
            .session
            .is_some_and(|session| session.run_id == completion.run_id)
    {
        return false;
    }
    form.finish(
        context.clone(),
        completion.run_id,
        None,
        &completion.result,
        stopped,
    );
    form.report = Some(completion.result.as_ref().map_or_else(
        |error| format!("Pyramid session stopped: {error}"),
        Clone::clone,
    ));
    workflow.session = None;
    *owner = None;
    if *state != WorkerState::Detached {
        *state = if completion.result.is_err() || stopped {
            WorkerState::Uncertain
        } else {
            WorkerState::Ready
        };
    }
    true
}


/// Use the existing diagnostic categories without inferring authority from phase text.
fn session_diagnostic_phase(phase: SessionPhase) -> PyramidPhase {
    match phase {
        SessionPhase::Entry | SessionPhase::Reconnaissance => PyramidPhase::Reconnaissance,
        SessionPhase::Planning => PyramidPhase::Planning,
        SessionPhase::Execution
        | SessionPhase::Transition
        | SessionPhase::Completion
        | SessionPhase::Restart => PyramidPhase::Execution,
    }
}


/// Describe one operation without exposing implementation details in the normal flow.
fn phase_label(phase: PyramidPhase) -> &'static str {
    match phase {
        PyramidPhase::Reconnaissance => "Recon",
        PyramidPhase::Planning => "Plan route",
        PyramidPhase::Execution => "Solve board",
    }
}


impl QmpQemuSocketApp {
    /// Select the simplified screen only for the independent Pyramid workflow.
    pub(super) fn uses_pyramid_workflow(&self) -> bool {
        is_workflow_mode(self.game_mode, self.solving_strategy)
    }


    /// Capture literal configuration and identity for the failure display and saved packet.
    pub(super) fn pyramid_failure_details(&self) -> serde_json::Value {
        let context = self.current_preview_context();
        serde_json::json!({
            "socket": context.socket_path,
            "game": context.game_mode.label(),
            "strategy": context.strategy.label(),
            "generation": context.generation,
            "workflow_mode": self.pyramid_workflow.mode().label(),
            "route_method": self.pyramid_workflow.method.label(),
            "profile_source": if self.reconnaissance_form.use_bundled_profile { "bundled_native_v17" } else { "custom_file" },
            "custom_profile_path": self.reconnaissance_form.profile_path,
            "next_evidence_json": self.reconnaissance_form.output_path,
            "next_deal_id": self.reconnaissance_form.deal_id,
            "restored_route_json": self.computed_route_form.deal_path,
            "defaults_error": self.reconnaissance_form.defaults_error,
            "active_route_run_id": self.computed_route_form.active_run_id,
            "prepared_plan_id": self.computed_route_form.plan.as_ref().map(|plan| plan.id),
            "visible_acquisition_current": self.computed_route_form.acquired_context.as_ref() == Some(&context),
            "session_log": self.session_log.as_ref().map(|log| log.path()),
            "search_limits": crate::pyramid_search_policy::details(crate::pyramid_search_policy::limits()),
        })
    }


    /// Surface a local failure immediately and request best-effort worker persistence.
    ///
    /// Persistence failures are displayed once and never reported recursively. This
    /// command grants no capture, QMP connection or guest-input authority.
    pub(super) fn pyramid_ui_failure(&mut self, phase: PyramidPhase, error: String) {
        let context = self.current_preview_context();
        self.pyramid_workflow.bind(&context);
        self.pyramid_workflow.phase = Some(phase);
        let mut details = self
            .pyramid_workflow
            .request_details
            .clone()
            .unwrap_or_else(|| self.pyramid_failure_details());
        let token = self.pyramid_workflow.reserve_failure_id();
        let (ui_failure_id, persistence_error) = match token {
            Ok(id) => {
                if let Some(details) = details.as_object_mut() {
                    details.insert("ui_failure_id".to_owned(), serde_json::json!(id));
                }
                (Some(id), None)
            }
            Err(error) => (None, Some(error.to_owned())),
        };
        self.pyramid_workflow.request_details = Some(details.clone());
        self.pyramid_workflow.diagnostic = Some(PyramidDiagnostic {
            context: context.clone(),
            phase,
            run_id: self.computed_route_form.active_run_id,
            ui_failure_id,
            error: error.clone(),
            directory: None,
            persistence_error,
        });
        self.push_log(error.clone());
        if ui_failure_id.is_none() {
            return;
        }
        if let Err(persistence_error) = self
            .worker
            .record_pyramid_failure(context, phase, error, details)
            && let Some(diagnostic) = self.pyramid_workflow.diagnostic.as_mut()
        {
            diagnostic.persistence_error = Some(persistence_error);
        }
    }


    /// Request exactly the pressed step; consent is armed immediately and consumed once.
    fn press_pyramid_step(&mut self, phase: PyramidPhase) {
        let context = self.current_preview_context();
        let idle = reconnaissance_controls_enabled(
            &context,
            self.strategy_choice_pending,
            self.active_run,
            self.worker_state,
            self.stop_requested,
            self.show_snapshot_dialog,
            self.snapshot_capture_pending,
        );
        if !step_request_enabled(self.pyramid_workflow.mode, phase, idle) {
            return;
        }
        let details = self.pyramid_failure_details();
        self.pyramid_workflow.begin(&context, phase, details);
        match phase {
            PyramidPhase::Reconnaissance => {
                self.dispatch_reconnaissance();
                self.reconnaissance_form.confirmed_context = None;
            }
            PyramidPhase::Planning => self.dispatch_computed_route(false),
            PyramidPhase::Execution => {
                if let Err(error) = arm_solve(&mut self.computed_route_form, &context) {
                    self.pyramid_ui_failure(phase, error.to_owned());
                    return;
                }
                self.dispatch_computed_route(true);
                self.computed_route_form.confirmed_context = None;
            }
        }
    }


    /// Change idle preferences without capture/input, retaining Recon and retiring its plan.
    fn select_pyramid_options(
        &mut self,
        mode: WorkflowMode,
        method: RouteMethod,
        stop_condition: StopCondition,
    ) {
        let context = self.current_preview_context();
        let idle = reconnaissance_controls_enabled(
            &context,
            self.strategy_choice_pending,
            self.active_run,
            self.worker_state,
            self.stop_requested,
            self.show_snapshot_dialog,
            self.snapshot_capture_pending,
        );
        let generation = match self.pyramid_workflow.select_preferences(
            mode,
            method,
            stop_condition,
            self.context_generation,
            idle,
        ) {
            Ok(Some(generation)) => generation,
            Ok(None) => return,
            Err(error) => {
                self.computed_route_form.revoke_acquisition();
                self.pyramid_ui_failure(PyramidPhase::Reconnaissance, error.to_owned());
                return;
            }
        };
        self.context_generation = generation;
        self.reconnaissance_form.confirmed_context = None;
        self.computed_route_form.revoke_plan();
        self.computed_route_form.report = None;
        self.pyramid_workflow.planning_identity = None;
        self.pyramid_workflow.planning = None;
        let context = self.current_preview_context();
        self.pyramid_workflow.bind(&context);
        self.current_status = if self.computed_route_form.calculation_ready(&context) {
            "Pyramid options changed — reusing current Recon; recalculate the route".to_owned()
        } else {
            "Pyramid options changed — Recon is available".to_owned()
        };
        if let Err(error) = self.worker.discard_pyramid_plan() {
            self.computed_route_form.revoke_acquisition();
            self.worker_state = WorkerState::Error;
            self.pyramid_ui_failure(PyramidPhase::Reconnaissance, error);
        }
    }


    /// Dispatch one explicit automatic start without UI-driven phase chaining.
    ///
    /// Existing defaults and literal overrides supply the first acquisition. Still
    /// automatic names are refreshed before dispatch, so a new attempt cannot reuse
    /// an earlier session's published file. The worker owns every later acquisition.
    fn press_pyramid_session(&mut self) {
        let context = self.current_preview_context();
        let idle = reconnaissance_controls_enabled(
            &context,
            self.strategy_choice_pending,
            self.active_run,
            self.worker_state,
            self.stop_requested,
            self.show_snapshot_dialog,
            self.snapshot_capture_pending,
        );
        if !session_start_enabled(self.pyramid_workflow.mode, idle, &self.computed_route_form) {
            return;
        }
        let details = self.pyramid_failure_details();
        self.pyramid_workflow
            .begin(&context, PyramidPhase::Reconnaissance, details);
        let reuse_acquisition = self
            .computed_route_form
            .calculation_ready(&context)
            .then(|| std::path::PathBuf::from(&self.computed_route_form.deal_path));
        if reuse_acquisition.is_none() {
            self.computed_route_form.revoke_acquisition();
        } else {
            self.computed_route_form.revoke_plan();
        }
        let reconnaissance = prepare_recon_request(&mut self.reconnaissance_form, &context);
        self.pyramid_workflow
            .record_request_details(self.pyramid_failure_details());
        let reconnaissance = match reconnaissance {
            Ok(request) => request,
            Err(error) => {
                self.pyramid_ui_failure(PyramidPhase::Reconnaissance, error.to_owned());
                return;
            }
        };
        let run_id = match self.computed_route_form.begin_request() {
            Ok(run_id) => run_id,
            Err(error) => {
                self.pyramid_ui_failure(PyramidPhase::Reconnaissance, error.to_owned());
                return;
            }
        };
        let details = self.pyramid_failure_details();
        self.pyramid_workflow
            .begin_session(&context, run_id, details);
        self.active_run = Some(ActiveRun::PyramidSession);
        self.worker_state = WorkerState::Validating;
        self.reconnaissance_report = None;
        self.current_status = "Checking the supported board and actual game progress".to_owned();
        self.invalidate_preview();
        let request = SessionRequest {
            run_id,
            mode: self.pyramid_workflow.mode,
            reconnaissance,
            reuse_acquisition,
            method: self.pyramid_workflow.method,
            use_challenge_default: !self.pyramid_workflow.method_explicit,
            route_goal: self.pyramid_workflow.route_goal(),
            stop_condition: self.pyramid_workflow.stop_condition,
        };
        self.push_log(format!(
            "Explicit Pyramid {} start requested with {} search. The worker retains one session until completion, STOP or error; guest Solver activation and resume are not authorised.",
            request.mode.label(), request.method.label(),
        ));
        if let Err(error) = self.worker.start_pyramid_session(context, request) {
            self.computed_route_form.active_run_id = None;
            self.computed_route_form.revoke_acquisition();
            self.active_run = None;
            self.pyramid_workflow.session = None;
            self.worker_state = WorkerState::Error;
            self.current_status = "Pyramid session dispatch failed".to_owned();
            self.pyramid_ui_failure(
                PyramidPhase::Reconnaissance,
                format!("Pyramid session request was not accepted: {error}"),
            );
        }
    }


    /// Show mode-specific explicit actions, concise progress, STOP and the preview.
    pub(super) fn pyramid_workflow_screen(&mut self, ui: &mut egui::Ui) {
        let context = self.current_preview_context();
        self.pyramid_workflow.bind(&context);
        let idle = reconnaissance_controls_enabled(
            &context,
            self.strategy_choice_pending,
            self.active_run,
            self.worker_state,
            self.stop_requested,
            self.show_snapshot_dialog,
            self.snapshot_capture_pending,
        );
        ui.horizontal(|ui| {
            ui.heading("Pyramid");
            let mut selected = self.solving_strategy;
            ui.add_enabled_ui(idle, |ui| {
                egui::ComboBox::from_id_salt("pyramid-strategy")
                    .selected_text(self.solving_strategy.label())
                    .show_ui(ui, |ui| {
                        for strategy in SolvingStrategy::AVAILABLE {
                            ui.selectable_value(&mut selected, strategy, strategy.label());
                        }
                    });
            });
            if selected != self.solving_strategy {
                self.select_strategy(selected);
            }
            {
                let stop_enabled = (self.active_run.is_some() || self.worker_state.is_busy())
                    && !self.stop_requested;
                if bevel_button(ui, "STOP", STATE_RED, false, stop_enabled).clicked() {
                    self.stop_requested = true;
                    self.reconnaissance_form.confirmed_context = None;
                    self.computed_route_form.revoke_acquisition();
                    self.current_status = "Stopping…".to_owned();
                    if let Err(error) = self.worker.disconnect(self.current_preview_context()) {
                        self.worker_state = WorkerState::Error;
                        self.current_status = "Stop request failed".to_owned();
                        let phase = self
                            .pyramid_workflow
                            .phase
                            .unwrap_or(PyramidPhase::Reconnaissance);
                        self.pyramid_ui_failure(phase, format!("STOP dispatch failed: {error}"));
                    }
                }
            }
        });
        if !self.uses_pyramid_workflow() {
            return;
        }
        let mut mode = self.pyramid_workflow.mode;
        let mut route_goal = self.pyramid_workflow.route_goal();
        let mut stop_condition = self.pyramid_workflow.stop_condition;
        ui.add_enabled_ui(idle, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Mode");
                egui::ComboBox::from_id_salt("pyramid-workflow-mode")
                    .selected_text(mode.label())
                    .show_ui(ui, |ui| {
                        for option in [
                            WorkflowMode::Manual,
                            WorkflowMode::SingleBoard,
                            WorkflowMode::Continuous,
                        ] {
                            ui.selectable_value(&mut mode, option, option.label());
                        }
                    });
                ui.label("Route objective");
                egui::ComboBox::from_id_salt("pyramid-route-method")
                    .selected_text(route_goal.label())
                    .show_ui(ui, |ui| {
                        for option in [RouteGoal::ShortestRoute, RouteGoal::MaximumCardMatches] {
                            if ui
                                .selectable_value(&mut route_goal, option, option.label())
                                .clicked()
                            {
                                self.pyramid_workflow.method_explicit = true;
                            }
                        }
                    });
                ui.add_enabled_ui(mode == WorkflowMode::Continuous, |ui| {
                    ui.label("Stop condition");
                    egui::ComboBox::from_id_salt("pyramid-stop-condition")
                        .selected_text(stop_condition.label())
                        .show_ui(ui, |ui| {
                            for option in [
                                StopCondition::RepeatNormalGames,
                                StopCondition::ChallengeTarget,
                                StopCondition::NoChallenge,
                            ] {
                                ui.selectable_value(&mut stop_condition, option, option.label());
                            }
                        });
                });
            });
        });
        self.select_pyramid_options(mode, route_goal.method(), stop_condition);
        let context = self.current_preview_context();
        let idle = reconnaissance_controls_enabled(
            &context,
            self.strategy_choice_pending,
            self.active_run,
            self.worker_state,
            self.stop_requested,
            self.show_snapshot_dialog,
            self.snapshot_capture_pending,
        );
        ui.small(self.pyramid_workflow.mode.description());
        ui.label(format!(
            "Observed: {}",
            self.pyramid_workflow.observed_game().label()
        ));
        ui.small(format!(
            "Selected mode: {}; stop condition: {}",
            self.pyramid_workflow.mode.label(),
            self.pyramid_workflow.stop_condition.label()
        ));
        ui.small(self.pyramid_workflow.route_goal().description());
        if self.pyramid_workflow.mode == WorkflowMode::Continuous {
            ui.small(self.pyramid_workflow.stop_condition.description());
        }
        ui.label("Start acquisition on an untouched supported Pyramid board. Keep Solitaire in the foreground and leave the guest untouched while work runs. Keep the guest Solver disengaged; an unavailable Solver is allowed.");
        if self.pyramid_workflow.mode == WorkflowMode::Manual {
            let gates = step_gates(&context, idle, &self.computed_route_form);
            ui.horizontal_wrapped(|ui| {
                if bevel_button(ui, "1. Recon", SETUP_BLUE, false, gates.recon)
                    .on_hover_text("Read all 52 cards and restore the untouched board; this uses guarded guest input").clicked() {
                    self.press_pyramid_step(PyramidPhase::Reconnaissance);
                }
                if bevel_button(ui, "2. Plan route", SETUP_BLUE, false, gates.plan)
                    .on_hover_text("Calculate and independently replay a route without guest input").clicked() {
                    self.press_pyramid_step(PyramidPhase::Planning);
                }
                if bevel_button(ui, "3. Solve board", ACTION_GREEN, false, gates.solve)
                    .on_hover_text("Execute the current prepared route once; this press authorises that route only").clicked() {
                    self.press_pyramid_step(PyramidPhase::Execution);
                }
            });
            ui.small("Recon reads 52 cards and restores the board. Plan route sends no guest input. Solve board executes once; it never starts automatically in Manual mode.");
        } else {
            if bevel_button(ui, "Recon (optional rescan)", SETUP_BLUE, false, idle)
                .on_hover_text("Replace the current board inventory with a fresh guarded Recon; changing planning options does not require this")
                .clicked()
            {
                self.press_pyramid_step(PyramidPhase::Reconnaissance);
            }
            let (label, description) = match self.pyramid_workflow.mode {
                WorkflowMode::SingleBoard => (
                    "Start single board",
                    "Authorise acquisition, planning and execution of this board once, then stop",
                ),
                WorkflowMode::Continuous => (
                    "Start continuous",
                    "Follow the selected stop condition, with guarded stock continuation when supported",
                ),
                WorkflowMode::Manual => ("Start", "Select an automatic mode first"),
            };
            if bevel_button(
                ui,
                label,
                ACTION_GREEN,
                false,
                session_start_enabled(self.pyramid_workflow.mode, idle, &self.computed_route_form),
            )
            .on_hover_text(description)
            .clicked()
            {
                self.press_pyramid_session();
            }
            if self.pyramid_workflow.mode == WorkflowMode::Continuous {
                ui.small("No challenge repeats boards without interpreting targets. The other stop conditions require their recognised objective. STOP or uncertainty releases control.");
            }
        }
        let phase = self
            .pyramid_workflow
            .session
            .map(|session| session.phase.label())
            .or_else(|| self.pyramid_workflow.phase.map(phase_label))
            .unwrap_or("Ready");
        ui.label(format!("{phase} · {}", self.current_status));
        if let Some(progress) = self.pyramid_workflow.planning_progress() {
            ui.label(format!(
                "{} · {:.2}s · {} positions examined · {} candidate moves · {} histories retained{}",
                progress.method.label(),
                progress.stats.elapsed.as_secs_f64(),
                progress.stats.expanded,
                progress.stats.generated,
                progress.stats.states,
                if progress.finished {
                    " · search finished"
                } else {
                    " · searching; STOP to cancel"
                },
            ))
            .on_hover_text(progress.method.description());
        }
        if let Some(report) = &self.computed_route_form.report {
            ui.label(report.as_str());
        }
        if self.pyramid_workflow.details_visible(&context) {
            self.pyramid_error_details(ui);
        }
        ui.separator();
        self.preview(ui);
        self.output(ui);
    }


    /// Display the actual failure, saved-packet outcome and bounded custom overrides.
    fn pyramid_error_details(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.heading("Pyramid failure details");
            if let Some(diagnostic) = &self.pyramid_workflow.diagnostic {
                ui.colored_label(Color32::LIGHT_RED, &diagnostic.error);
            }
            egui::ScrollArea::vertical().id_salt("pyramid-failure-details")
                .max_height(260.0).auto_shrink([false, false]).show(ui, |ui| {
                    if let Some(diagnostic) = &self.pyramid_workflow.diagnostic {
                        ui.label(format!("Step: {}; request: {:?}", phase_label(diagnostic.phase), diagnostic.run_id));
                        match &diagnostic.directory {
                            Some(directory) => { ui.label(format!("Saved failure packet: {}", directory.display())); }
                            None => { ui.label("Saved failure packet: unavailable or persistence pending."); }
                        }
                        if let Some(error) = &diagnostic.persistence_error {
                            ui.colored_label(Color32::YELLOW, format!("Diagnostic persistence failed: {error}"));
                        }
                    }
                    match &self.session_log {
                        Some(log) => { ui.label(format!("Session log: {}", log.path().display())); }
                        None => { ui.label("Session log: unavailable; the visible output retains the current tail."); }
                    }
                    if let Some(details) = &self.pyramid_workflow.request_details {
                        ui.label("Fields at the requested step:");
                        match serde_json::to_string_pretty(details) {
                            Ok(details) => { ui.monospace(details); }
                            Err(error) => { ui.label(format!("Could not format request fields: {error}")); }
                        }
                    }
                    ui.label("Inspect the guest before another Recon. Failed or stopped routes cannot resume.");
                    self.pyramid_error_overrides(ui);
                });
        });
    }


    /// Retain literal custom paths and inspection capture behind the failure display.
    fn pyramid_error_overrides(&mut self, ui: &mut egui::Ui) {
        let idle = controls_are_mutable(self.active_run, self.worker_state, self.stop_requested)
            && !self.show_snapshot_dialog
            && !self.snapshot_capture_pending;
        ui.add_enabled_ui(idle, |ui| {
            ui.label("QMP Unix socket");
            ui.add(egui::TextEdit::singleline(self.pyramid_workflow.socket_editor(&self.qmp_socket_path))
                .char_limit(RECON_PATH_MAX_BYTES).desired_width(f32::INFINITY));
            let mut socket_changed = false;
            if ui.button("Apply socket change").clicked() {
                match self.pyramid_workflow.socket_update(&self.qmp_socket_path) {
                    Ok(Some(path)) => {
                        self.qmp_socket_path = path;
                        socket_changed = true;
                    }
                    Ok(None) => {}
                    Err(error) => {
                        let phase = self.pyramid_workflow.phase.unwrap_or(PyramidPhase::Reconnaissance);
                        self.pyramid_ui_failure(phase, error.to_owned());
                    }
                }
            }
            let source_changed = ui.checkbox(&mut self.reconnaissance_form.use_bundled_profile,
                "Use bundled native v17 calibration").changed();
            let mut profile_changed = false;
            if !self.reconnaissance_form.use_bundled_profile {
                ui.label("Custom native profile path");
                profile_changed = ui.add(egui::TextEdit::singleline(&mut self.reconnaissance_form.profile_path)
                    .char_limit(RECON_PATH_MAX_BYTES).desired_width(f32::INFINITY)).changed();
            }
            ui.label("Next evidence JSON path");
            let output_changed = ui.add(egui::TextEdit::singleline(&mut self.reconnaissance_form.output_path)
                .char_limit(RECON_PATH_MAX_BYTES).desired_width(f32::INFINITY)).changed();
            ui.label("Next Deal ID");
            let deal_changed = ui.add(egui::TextEdit::singleline(&mut self.reconnaissance_form.deal_id)
                .char_limit(RECON_DEAL_ID_MAX_BYTES)).changed();
            ui.label("Restored route JSON path");
            let route_changed = ui.add(egui::TextEdit::singleline(&mut self.computed_route_form.deal_path)
                .char_limit(RECON_PATH_MAX_BYTES).desired_width(f32::INFINITY)).changed();
            ui.small("Paths are literal. A moved unchanged export is accepted; the worker verifies the same-session receipt and exact bytes.");
            if socket_changed {
                self.context_generation = self.context_generation.wrapping_add(1);
                self.computed_route_form.revoke_acquisition();
                self.invalidate_preview();
                self.current_status = "Socket changed — Recon required".to_owned();
                let context = self.current_preview_context();
                self.pyramid_workflow.bind(&context);
            }
            if socket_changed || source_changed || profile_changed || output_changed || deal_changed || route_changed {
                self.reconnaissance_form.edited(output_changed, deal_changed);
                self.computed_route_form.revoke_plan();
                self.computed_route_form.report = None;
            }
            ui.horizontal(|ui| {
                if bevel_button(ui, "Capture Frame", SETUP_BLUE, false, true).clicked() {
                    self.request_capture("Failure inspection capture");
                }
                if bevel_button(ui, "Capture PNG", SETUP_BLUE, false, true).clicked() {
                    self.snapshot_label.clear();
                    self.show_snapshot_dialog = true;
                    self.snapshot_focus_pending = true;
                    self.prepare_snapshot();
                }
            });
            ui.small("Capture Frame/PNG request new pixels. They do not replace the saved failure packet.");
        });
    }
}


#[cfg(test)]
mod identity_tests {
    //! Boundary checks for local diagnostic identities without worker or filesystem I/O.

    use super::*;


    /// Diagnostic identity exhaustion never reuses an earlier expected packet token.
    #[test]
    fn failure_identity_exhaustion_is_sticky_across_context_changes() {
        let mut workflow = PyramidWorkflow {
            last_ui_failure_id: u64::MAX,
            expected_ui_failure_id: Some(u64::MAX),
            ..PyramidWorkflow::default()
        };
        assert!(workflow.reserve_failure_id().is_err());
        assert!(workflow.expected_ui_failure_id.is_none());
        let context = ControllerContext::new(
            PathBuf::from("/tmp/changed.sock"),
            GameMode::Pyramid,
            SolvingStrategy::ShortestPath,
            8,
        );
        workflow.bind(&context);
        assert!(workflow.reserve_failure_id().is_err());
        assert_eq!(workflow.last_ui_failure_id, u64::MAX);
    }
}
