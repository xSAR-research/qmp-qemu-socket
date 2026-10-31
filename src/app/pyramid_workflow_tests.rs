//! Headless checks for explicit step consent, mode isolation and terminal ownership.

use super::{pyramid_workflow::*, *};
use crate::pyramid_diagnostics::{PyramidDiagnostic, PyramidPhase};
use crate::pyramid_workflow_model::{
    PlanningProgress, RouteGoal, RouteMethod, SessionCompletion, SessionPhase, SessionProgress,
    StopCondition, WorkflowMode,
};
use std::{
    io::Write,
    os::unix::fs::DirBuilderExt,
    sync::atomic::{AtomicU64, Ordering},
};


/// Distinguish private fixture directories created concurrently in this test process.
static EVIDENCE_DIRECTORY_SERIAL: AtomicU64 = AtomicU64::new(0);


/// Startup and forced RESET use the same three defaults without keeping stale search receipts.
#[test]
fn qol_defaults_and_reset_retire_planning_even_when_selections_already_match() {
    let selected = context();
    for already_default in [false, true] {
        let mut workflow = PyramidWorkflow::default();
        assert_eq!(WorkflowMode::default(), WorkflowMode::Continuous);
        assert_eq!(StopCondition::default(), StopCondition::ChallengeTarget);
        assert_eq!(StopCondition::ChallengeTarget.label(), "Challenge");
        assert_eq!(workflow.method(), RouteMethod::FewestActions);
        if !already_default {
            workflow
                .select_preferences(
                    WorkflowMode::Manual,
                    RouteMethod::Fast,
                    StopCondition::RepeatNormalGames,
                    selected.generation,
                    true,
                )
                .unwrap();
        }
        workflow.begin_planning(&selected, 42);
        let late = planning_update(&selected, 42, 0, workflow.method(), true);
        workflow.reset_preferences();
        assert_eq!(workflow.mode(), WorkflowMode::Continuous);
        assert_eq!(workflow.method(), RouteMethod::FewestActions);
        assert!(workflow.planning_progress().is_none());
        assert!(!workflow.receive_planning(late, &selected));
        assert_eq!(
            workflow
                .select_preferences(
                    WorkflowMode::Continuous,
                    RouteMethod::FewestActions,
                    StopCondition::ChallengeTarget,
                    selected.generation,
                    true
                )
                .unwrap(),
            None
        );
    }
}


/// RESET retires an explicit route choice so the next authenticated Kings default still applies.
#[test]
fn qol_reset_preserves_authenticated_kings_default() {
    let selected = context();
    let mut workflow = PyramidWorkflow::default();
    workflow
        .select_preferences(
            WorkflowMode::Manual,
            RouteMethod::FewestActions,
            StopCondition::ChallengeTarget,
            selected.generation,
            true,
        )
        .unwrap();
    workflow.reset_preferences();
    workflow.apply_acquisition_default(Some(RouteMethod::Fast));
    assert_eq!(workflow.method(), RouteMethod::Fast);
}


/// Bound challenges survive board changes; stale run/context/generation reports cannot replace them.
#[test]
fn execution_observability_ui_rejects_stale_reports_and_retains_bound_challenge() {
    use crate::pyramid_workflow_model::ObservedGame;
    let context = context();
    let observed = ObservedGame::Challenge {
        progress: crate::pyramid_progress::ChallengeProgress::new(
            crate::pyramid_progress::ChallengeKind::Boards,
            2,
            3,
        )
        .unwrap(),
        kings: false,
    };
    let mut workflow = PyramidWorkflow::default();
    workflow.begin_session(&context, 7, serde_json::json!({}));
    let mut current = session_update(&context, 7, 1, SessionPhase::Planning);
    current.observed_game = observed;
    assert!(workflow.receive_session_progress(
        &current,
        &context,
        Some(ActiveRun::PyramidSession),
        Some(7)
    ));
    assert_eq!(workflow.observed_game(), observed);
    for (run_id, board_generation) in [(6, 1), (7, 0)] {
        let stale = session_update(&context, run_id, board_generation, SessionPhase::Entry);
        assert!(!workflow.receive_session_progress(
            &stale,
            &context,
            Some(ActiveRun::PyramidSession),
            Some(7)
        ));
        assert_eq!(workflow.observed_game(), observed);
    }
    let next = session_update(&context, 7, 2, SessionPhase::Reconnaissance);
    assert!(workflow.receive_session_progress(
        &next,
        &context,
        Some(ActiveRun::PyramidSession),
        Some(7)
    ));
    assert_eq!(workflow.observed_game(), observed);
    let mut stale = planning_update(&context, 7, 1, RouteMethod::Fast, true);
    stale.observed_game = observed;
    assert!(!workflow.receive_planning(stale, &context));
    assert_eq!(workflow.observed_game(), observed);
    let mut changed = context.clone();
    changed.generation += 1;
    workflow.bind(&changed);
    assert!(!workflow.receive_session_progress(
        &current,
        &changed,
        Some(ActiveRun::PyramidSession),
        Some(7)
    ));
    assert_eq!(workflow.observed_game(), ObservedGame::Unknown);
}


/// Authenticated challenge advice applies at RECON, including prior explicit and no-challenge choices.
#[test]
fn challenge_default_applies_at_recon_over_prior_preferences_and_no_challenge() {
    let mut workflow = PyramidWorkflow::default();
    workflow.apply_acquisition_default(Some(RouteMethod::Fast));
    assert_eq!(workflow.method(), RouteMethod::Fast);
    workflow.apply_acquisition_default(Some(RouteMethod::FewestActions));
    assert_eq!(workflow.method(), RouteMethod::FewestActions);
    let selected = context();
    workflow
        .select_preferences(
            WorkflowMode::SingleBoard,
            RouteMethod::Fast,
            StopCondition::ChallengeTarget,
            selected.generation,
            true,
        )
        .unwrap();
    workflow.apply_acquisition_default(Some(RouteMethod::FewestActions));
    assert_eq!(workflow.method(), RouteMethod::FewestActions);
    let mut no_challenge = PyramidWorkflow::default();
    no_challenge
        .select_preferences(
            WorkflowMode::Continuous,
            RouteMethod::FewestActions,
            StopCondition::NoChallenge,
            selected.generation,
            true,
        )
        .unwrap();
    no_challenge.apply_acquisition_default(Some(RouteMethod::Fast));
    assert_eq!(no_challenge.method(), RouteMethod::Fast);
}


/// RECON challenge advice applies on board one and after statistics arrive before board-two phase.
#[test]
fn challenge_default_applies_on_first_board_and_stats_first_later_board() {
    let selected = context();
    let mut workflow = PyramidWorkflow::default();
    workflow.bind(&selected);
    workflow
        .select_preferences(
            WorkflowMode::Continuous,
            RouteMethod::Fast,
            StopCondition::ChallengeTarget,
            selected.generation,
            true,
        )
        .unwrap();
    workflow
        .select_preferences(
            WorkflowMode::Continuous,
            RouteMethod::FewestActions,
            StopCondition::ChallengeTarget,
            selected.generation,
            true,
        )
        .unwrap();
    workflow.begin_session(&selected, 9, serde_json::json!({}));
    let owner = Some(ActiveRun::PyramidSession);
    let mut phase = session_update(&selected, 9, 1, SessionPhase::Reconnaissance);
    phase.effective_method = RouteMethod::FewestActions;
    assert!(workflow.receive_session_progress(&phase, &selected, owner, Some(9)));
    workflow.apply_acquisition_default(Some(RouteMethod::Fast));
    assert_eq!(
        workflow.method(),
        RouteMethod::Fast,
        "fresh first-board challenge advice replaces earlier Shortest"
    );
    assert!(workflow.receive_planning(
        planning_update(&selected, 9, 2, RouteMethod::FewestActions, true),
        &selected
    ));
    phase.board_generation = 2;
    phase.phase = SessionPhase::Planning;
    assert!(workflow.receive_session_progress(&phase, &selected, owner, Some(9)));
    assert!(workflow.planning_progress().unwrap().finished);
    workflow.apply_acquisition_default(Some(RouteMethod::Fast));
    assert_eq!(
        workflow.method(),
        RouteMethod::Fast,
        "new board can use its authenticated challenge advice"
    );
}


/// Outcome and stop choices reserve one generation, survive binding and cannot change while busy.
#[test]
fn stop_and_route_preferences_preserve_checked_context_invalidation() {
    let selected = context();
    let mut workflow = PyramidWorkflow::default();
    assert_eq!(
        workflow.select_preferences(
            WorkflowMode::Continuous,
            RouteMethod::FewestActions,
            StopCondition::NoChallenge,
            u64::MAX,
            true
        ),
        Ok(Some(u64::MAX))
    );
    workflow.bind(&selected);
    assert_eq!(workflow.route_goal(), RouteGoal::ShortestRoute);
    workflow.begin_session(&selected, 1, serde_json::json!({}));
    assert_eq!(
        workflow.select_preferences(
            WorkflowMode::Manual,
            RouteMethod::Fast,
            StopCondition::RepeatNormalGames,
            selected.generation,
            true
        ),
        Ok(None)
    );
    assert_eq!(workflow.mode(), WorkflowMode::Continuous);
}


/// Own only freshly created synthetic evidence; no worker or guest uses this directory.
struct EvidenceDirectory(PathBuf);


impl EvidenceDirectory {
    /// Create a private, exclusively owned directory for request-boundary regressions.
    fn new() -> Self {
        let serial = EVIDENCE_DIRECTORY_SERIAL.fetch_add(1, Ordering::Relaxed);
        let epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("test clock follows Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "pyramid-ui-evidence-{}-{epoch}-{serial}",
            std::process::id()
        ));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .expect("exclusive private evidence directory");
        Self(path)
    }


    /// Provide still-automatic fields without creating or reserving an evidence file.
    fn form(&self) -> ReconnaissanceForm {
        ReconnaissanceForm::with_defaults(Ok(EvidenceDefaults::in_directory(
            self.0.clone(),
            std::process::id(),
        )))
    }
}


impl Drop for EvidenceDirectory {
    /// Remove only this test's exclusively created directory and synthetic contents.
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}


/// Model publication with unmistakable synthetic bytes, without creating a live receipt.
fn publish_request_fixture(path: &std::path::Path) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .expect("synthetic publication cannot replace an existing file");
    file.write_all(b"synthetic published evidence; no live receipt")
        .expect("synthetic publication bytes");
}


/// Build one fixed independent-planning identity without filesystem or worker I/O.
fn context() -> ControllerContext {
    ControllerContext::new(
        PathBuf::from("/tmp/pyramid-ui.sock"),
        GameMode::Pyramid,
        SolvingStrategy::ShortestPath,
        11,
    )
}


/// Build literal acquisition fields without generating names or granting consent.
fn recon_form() -> ReconnaissanceForm {
    ReconnaissanceForm {
        use_bundled_profile: true,
        profile_path: String::new(),
        output_path: "/tmp/exact exported deal".to_owned(),
        deal_id: "test-deal".to_owned(),
        defaults: None,
        automatic_output: None,
        automatic_deal_id: None,
        defaults_error: None,
        confirmed_context: None,
    }
}


/// Represent the visible prerequisite only; tests create no worker acquisition receipt.
fn acquired_route(context: &ControllerContext) -> ComputedRouteForm {
    ComputedRouteForm {
        deal_path: "/tmp/exact exported deal".to_owned(),
        acquired_context: Some(context.clone()),
        ..ComputedRouteForm::default()
    }
}


/// Build advisory packet metadata for one exact step and request identity.
fn diagnostic(
    context: ControllerContext,
    phase: PyramidPhase,
    run_id: Option<u64>,
) -> PyramidDiagnostic {
    PyramidDiagnostic {
        progress: None,
        context,
        phase,
        run_id,
        ui_failure_id: None,
        error: "first unmatched observation".to_owned(),
        directory: Some(PathBuf::from("/tmp/private-pyramid-packet")),
        persistence_error: None,
    }
}


/// Recon's button supplies one request's consent without arming a route or future request.
#[test]
fn recon_press_arms_only_one_context_bound_request() {
    let context = context();
    let mut recon = recon_form();
    let route = acquired_route(&context);
    assert!(recon.take_request(&context).is_err());
    arm_recon(&mut recon, &context);
    let request = recon
        .take_request(&context)
        .expect("explicit button request");
    assert!(request.initial_board_confirmed);
    assert!(matches!(
        request.profile,
        ReconProfileSource::BundledNativeV23
    ));
    assert!(recon.confirmed_context.is_none());
    assert!(recon.take_request(&context).is_err());
    assert!(route.confirmed_context.is_none());
    let mut changed = context.clone();
    changed.generation += 1;
    arm_recon(&mut recon, &context);
    assert!(recon.take_request(&changed).is_err());
    assert!(recon.confirmed_context.is_none());
}


/// Solve's button authorises only the current prepared plan and dispatch consumes it.
#[test]
fn solve_press_requires_current_plan_and_consumes_consent() {
    let context = context();
    let mut route = acquired_route(&context);
    assert!(arm_solve(&mut route, &context).is_err());
    route.plan = Some(ComputedRoutePlan {
        id: 7,
        context: context.clone(),
    });
    assert!(route.execution_plan(&context).is_err());
    arm_solve(&mut route, &context).expect("explicit Solve board press");
    assert_eq!(route.execution_plan(&context), Ok(7));
    route
        .begin_request()
        .expect("dispatch consumes plan and consent");
    assert!(route.plan.is_none());
    assert!(route.confirmed_context.is_none());
    assert!(arm_solve(&mut route, &context).is_err());
    assert!(recon_form().confirmed_context.is_none());
}


/// Switching away and back cannot reuse a plan's old context to arm execution.
#[test]
fn solve_press_rejects_stale_context_without_consent() {
    let old = context();
    let mut current = old.clone();
    current.generation += 2;
    let mut route = acquired_route(&old);
    route.plan = Some(ComputedRoutePlan {
        id: 7,
        context: old.clone(),
    });
    assert!(arm_solve(&mut route, &current).is_err());
    assert!(route.confirmed_context.is_none());
    assert!(route.execution_plan(&old).is_err());
}


/// Acquire, calculate and execute remain explicit independent steps through the lifecycle.
#[test]
fn three_step_gates_follow_acquisition_then_plan_and_never_auto_solve() {
    let context = context();
    let mut route = ComputedRouteForm::default();
    let initial = step_gates(&context, true, &route);
    assert!(initial.recon && !initial.plan && !initial.solve);
    route = acquired_route(&context);
    let acquired = step_gates(&context, true, &route);
    assert!(acquired.recon && acquired.plan && !acquired.solve);
    let id = route.begin_request().expect("input-free planning request");
    route.finish(
        context.clone(),
        id,
        Some(id),
        &Ok("independent replay passed".to_owned()),
        false,
    );
    let planned = step_gates(&context, true, &route);
    assert!(planned.recon && planned.plan && planned.solve);
    assert!(route.confirmed_context.is_none());
    assert!(route.execution_plan(&context).is_err());
    route.revoke_acquisition();
    let consumed = step_gates(&context, true, &route);
    assert!(consumed.recon && !consumed.plan && !consumed.solve);
}


/// STOP, active ownership, snapshot review and unsupported modes cannot enable a step.
#[test]
fn three_step_gates_preserve_existing_idle_and_mode_boundaries() {
    let context = context();
    let mut route = acquired_route(&context);
    route.plan = Some(ComputedRoutePlan {
        id: 7,
        context: context.clone(),
    });
    let blocked = step_gates(&context, false, &route);
    assert!(!blocked.recon && !blocked.plan && !blocked.solve);
    route.active_run_id = Some(9);
    let outstanding = step_gates(&context, true, &route);
    assert!(!outstanding.recon && !outstanding.plan && !outstanding.solve);
    for game in GameMode::AVAILABLE {
        assert!(!is_workflow_mode(game, SolvingStrategy::ComputerVision));
        assert_eq!(
            is_workflow_mode(game, SolvingStrategy::ShortestPath),
            game == GameMode::Pyramid
        );
    }
}


/// Normal steps hide technical clutter; a current failure exposes it until another step.
#[test]
fn technical_details_appear_only_for_matching_failure_and_clear_on_new_step() {
    let context = context();
    let mut workflow = PyramidWorkflow::default();
    workflow.begin(
        &context,
        PyramidPhase::Planning,
        serde_json::json!({"literal_path":"/tmp/deal"}),
    );
    assert!(!workflow.details_visible(&context));
    assert!(workflow.receive(
        diagnostic(context.clone(), PyramidPhase::Planning, Some(3)),
        &context,
        Some(3)
    ));
    assert!(workflow.details_visible(&context));
    workflow.begin(&context, PyramidPhase::Execution, serde_json::json!({}));
    assert!(!workflow.details_visible(&context));
    assert!(!workflow.receive(
        diagnostic(context.clone(), PyramidPhase::Planning, Some(3)),
        &context,
        Some(4)
    ));
    assert!(!workflow.details_visible(&context));
    let mut changed = context.clone();
    changed.generation += 1;
    assert!(!workflow.receive(
        diagnostic(context.clone(), PyramidPhase::Execution, Some(4)),
        &changed,
        Some(4)
    ));
    workflow.bind(&changed);
    assert!(workflow.phase().is_none());
}


/// Late or wrong-request packets cannot reopen technical details for a newer run.
#[test]
fn diagnostics_reject_wrong_request_without_changing_current_phase() {
    let context = context();
    let mut workflow = PyramidWorkflow::default();
    workflow.begin(&context, PyramidPhase::Execution, serde_json::json!({}));
    assert!(!workflow.receive(
        diagnostic(context.clone(), PyramidPhase::Execution, Some(3)),
        &context,
        Some(4)
    ));
    assert!(!workflow.receive(
        diagnostic(context.clone(), PyramidPhase::Execution, Some(4)),
        &context,
        None
    ));
    assert!(!workflow.details_visible(&context));
    assert_eq!(workflow.phase(), Some(PyramidPhase::Execution));
}


/// Socket edits remain recoverable until Apply, then the completed context change clears error.
#[test]
fn socket_override_typing_retains_failure_editor_until_atomic_apply() {
    let context = context();
    let current_socket = context.socket_path.to_str().expect("test path");
    let mut workflow = PyramidWorkflow::default();
    workflow.begin(
        &context,
        PyramidPhase::Reconnaissance,
        serde_json::json!({}),
    );
    assert!(workflow.receive(
        diagnostic(context.clone(), PyramidPhase::Reconnaissance, None),
        &context,
        None
    ));
    *workflow.socket_editor(current_socket) = "/tmp/first".to_owned();
    workflow.bind(&context);
    assert!(workflow.details_visible(&context));
    workflow
        .socket_editor(current_socket)
        .push_str("-finished.sock");
    let path = workflow
        .socket_update(current_socket)
        .expect("bounded complete edit")
        .expect("changed socket");
    assert_eq!(path, "/tmp/first-finished.sock");
    assert!(workflow.details_visible(&context));
    let mut applied = context.clone();
    applied.socket_path = PathBuf::from(path);
    applied.generation += 1;
    workflow.bind(&applied);
    assert!(!workflow.details_visible(&applied));
    assert!(workflow.phase().is_none());
}


/// Local packet identities distinguish a delayed failure from a newly requested step.
#[test]
fn local_failure_packet_is_rejected_after_same_phase_retry_or_context_change() {
    let context = context();
    for phase in [PyramidPhase::Reconnaissance, PyramidPhase::Planning] {
        let mut workflow = PyramidWorkflow::default();
        workflow.begin(&context, phase, serde_json::json!({}));
        let old_id = workflow
            .reserve_failure_id()
            .expect("local failure identity");
        workflow.begin(&context, phase, serde_json::json!({"retried":true}));
        let mut old_packet = diagnostic(context.clone(), phase, None);
        old_packet.ui_failure_id = Some(old_id);
        assert!(!workflow.receive(
            old_packet,
            &context,
            (phase == PyramidPhase::Planning).then_some(4)
        ));
        assert!(!workflow.details_visible(&context));
        let current_id = workflow.reserve_failure_id().expect("new local failure");
        assert!(current_id > old_id);
        let mut current_packet = diagnostic(context.clone(), phase, None);
        current_packet.ui_failure_id = Some(current_id);
        assert!(workflow.receive(current_packet, &context, None));
        assert!(workflow.details_visible(&context));
        let mut changed = context.clone();
        changed.generation += 1;
        workflow.bind(&changed);
        let after_context = workflow
            .reserve_failure_id()
            .expect("counter retained across contexts");
        assert!(after_context > current_id);
    }
}


/// Untagged packets cannot replace a current keyed calculation or execution result.
#[test]
fn unkeyed_packet_cannot_interrupt_new_computed_request() {
    let context = context();
    let mut workflow = PyramidWorkflow::default();
    workflow.begin(&context, PyramidPhase::Planning, serde_json::json!({}));
    assert!(!workflow.receive(
        diagnostic(context.clone(), PyramidPhase::Planning, None),
        &context,
        Some(4)
    ));
    assert!(!workflow.details_visible(&context));
}


/// A matching finish removes stale busy states while preserving detached/STOP semantics.
#[test]
fn matching_terminal_completion_reconciles_busy_badge_and_consumes_run() {
    let context = context();
    for (initial, stopped, succeeds, expected) in [
        (WorkerState::Capturing, false, false, WorkerState::Uncertain),
        (WorkerState::Verifying, false, true, WorkerState::Ready),
        (WorkerState::Capturing, true, true, WorkerState::Uncertain),
        (WorkerState::Detached, true, false, WorkerState::Detached),
    ] {
        let mut form = acquired_route(&context);
        let id = form.begin_request().expect("active request");
        let mut owner = Some(ActiveRun::ComputedRoute);
        let mut state = initial;
        let result = if succeeds {
            Ok("complete".to_owned())
        } else {
            Err("effect unverified".to_owned())
        };
        assert!(complete_route(
            &mut form,
            &mut owner,
            &mut state,
            RouteCompletion {
                retained_recon: false,
                context: context.clone(),
                run_id: id,
                plan_id: Some(id),
                result: &result,
                stopped,
            }
        ));
        assert_eq!(state, expected);
        assert!(owner.is_none());
        assert!(form.active_run_id.is_none());
        assert_eq!(form.plan.is_some(), succeeds && !stopped);
        assert!(form.confirmed_context.is_none());
    }
}


/// Wrong tokens and another operation's owner cannot terminate or unblock the current run.
#[test]
fn unmatched_terminal_completion_preserves_run_busy_state_and_plan() {
    let context = context();
    for owner_value in [
        Some(ActiveRun::ComputedRoute),
        Some(ActiveRun::Reconnaissance),
        Some(ActiveRun::PyramidSession),
    ] {
        let mut form = acquired_route(&context);
        let id = form.begin_request().expect("active request");
        let mut owner = owner_value;
        let mut state = WorkerState::Capturing;
        let incoming = if owner_value == Some(ActiveRun::ComputedRoute) {
            id + 1
        } else {
            id
        };
        let result = Err("old completion".to_owned());
        assert!(!complete_route(
            &mut form,
            &mut owner,
            &mut state,
            RouteCompletion {
                retained_recon: false,
                context: context.clone(),
                run_id: incoming,
                plan_id: None,
                result: &result,
                stopped: false,
            }
        ));
        assert_eq!(owner, owner_value);
        assert_eq!(state, WorkerState::Capturing);
        assert_eq!(form.active_run_id, Some(id));
    }
}


/// Build a synthetic statistics notification with no search or worker execution.
fn planning_update(
    context: &ControllerContext,
    run_id: u64,
    board_generation: u64,
    method: RouteMethod,
    finished: bool,
) -> PlanningProgress {
    PlanningProgress {
        observed_game: crate::pyramid_workflow_model::ObservedGame::Unknown,
        context: context.clone(),
        run_id,
        board_generation,
        method,
        stats: xsar::search::SearchStats {
            expanded: 8,
            generated: 21,
            states: 10,
            peak_frontier: 3,
            elapsed: Duration::from_millis(200),
        },
        finished,
    }
}


/// Build a typed automatic phase report without parsing status text or granting input.
fn session_update(
    context: &ControllerContext,
    run_id: u64,
    board_generation: u64,
    phase: SessionPhase,
) -> SessionProgress {
    SessionProgress {
        observed_game: crate::pyramid_workflow_model::ObservedGame::Unknown,
        effective_method: RouteMethod::Fast,
        context: context.clone(),
        run_id,
        board_generation,
        phase,
        message: "worker-owned phase".to_owned(),
    }
}


/// Latest fallback statistics may precede ordered phases; old Exact cannot revert them.
#[test]
fn fallback_statistics_survive_delayed_exact_events_and_reset_on_new_board() {
    let selected = context();
    let mut workflow = PyramidWorkflow::default();
    workflow.begin_session(&selected, 9, serde_json::json!({}));
    let owner = Some(ActiveRun::PyramidSession);
    let mut exact = session_update(&selected, 9, 1, SessionPhase::Planning);
    exact.effective_method = RouteMethod::FewestActions;
    assert!(workflow.receive_session_progress(&exact, &selected, owner, Some(9)));
    assert!(workflow.receive_planning(
        planning_update(&selected, 9, 1, RouteMethod::Fast, true),
        &selected
    ));
    assert_eq!(workflow.method(), RouteMethod::Fast);
    assert!(!workflow.receive_session_progress(&exact, &selected, owner, Some(9)));
    assert!(!workflow.receive_planning(
        planning_update(&selected, 9, 1, RouteMethod::FewestActions, true),
        &selected
    ));
    let maximum = session_update(&selected, 9, 1, SessionPhase::Planning);
    assert!(workflow.receive_session_progress(&maximum, &selected, owner, Some(9)));
    assert!(workflow.planning_progress().unwrap().finished);
    exact.board_generation = 2;
    exact.phase = SessionPhase::Reconnaissance;
    assert!(workflow.receive_session_progress(&exact, &selected, owner, Some(9)));
    assert_eq!(workflow.method(), RouteMethod::FewestActions);
    assert!(workflow.planning_progress().is_none());
}


/// Idle selection changes reserve a new context; disabled or exhausted changes do nothing.
#[test]
fn mode_and_method_defaults_changes_and_exhaustion_preserve_authority_boundaries() {
    let selected = context();
    let mut workflow = PyramidWorkflow::default();
    assert_eq!(
        workflow.select_options(
            WorkflowMode::SingleBoard,
            RouteMethod::FewestActions,
            selected.generation,
            false
        ),
        Ok(None)
    );
    assert_eq!(
        workflow.select_options(
            WorkflowMode::SingleBoard,
            RouteMethod::FewestActions,
            selected.generation,
            true
        ),
        Ok(Some(selected.generation))
    );
    workflow.bind(&selected);
    let mut route = acquired_route(&selected);
    route.plan = Some(ComputedRoutePlan {
        id: 3,
        context: selected.clone(),
    });
    route.revoke_plan();
    let gates = step_gates(&selected, true, &route);
    assert!(gates.recon && gates.plan && !gates.solve);
    assert!(arm_solve(&mut route, &selected).is_err());
    let mut changed = selected.clone();
    changed.generation += 1;
    assert!(!step_gates(&changed, true, &route).plan);
}


/// Automatic Start cannot bypass manual ownership, busy/STOP gates or an outstanding token.
#[test]
fn automatic_start_requires_an_explicit_automatic_mode_and_idle_controller() {
    let mut route = ComputedRouteForm::default();
    assert!(!session_start_enabled(WorkflowMode::Manual, true, &route));
    for mode in [WorkflowMode::SingleBoard, WorkflowMode::Continuous] {
        assert!(session_start_enabled(mode, true, &route));
        assert!(!session_start_enabled(mode, false, &route));
    }
    route.begin_request().expect("outstanding manual request");
    assert!(!session_start_enabled(
        WorkflowMode::SingleBoard,
        true,
        &route
    ));
    assert!(!session_start_enabled(
        WorkflowMode::Continuous,
        true,
        &route
    ));
}


/// A published automatic acquisition never traps a later Manual request on that path.
#[test]
fn automatic_completion_then_manual_recon_preserves_export_and_prepares_new_names() {
    for (succeeds, stopped) in [(true, false), (false, false), (false, true)] {
        let directory = EvidenceDirectory::new();
        let mut recon = directory.form();
        let mut selected = context();
        let mut workflow = PyramidWorkflow::default();
        selected.generation = workflow
            .select_options(
                WorkflowMode::SingleBoard,
                RouteMethod::Fast,
                selected.generation,
                true,
            )
            .expect("idle Single board selection")
            .expect("mode changed");
        workflow.bind(&selected);
        let first = prepare_recon_request(&mut recon, &selected)
            .expect("automatic request prepares its own names");
        publish_request_fixture(&first.output_path);
        let published = std::fs::read(&first.output_path).expect("published fixture");
        let mut route = ComputedRouteForm::default();
        let run_id = route.begin_request().expect("automatic request identity");
        workflow.begin_session(&selected, run_id, serde_json::json!({}));
        let completion = SessionCompletion {
            context: selected.clone(),
            run_id,
            result: if succeeds {
                Ok("synthetic session completion".to_owned())
            } else {
                Err("synthetic failure after publication".to_owned())
            },
        };
        let mut owner = Some(ActiveRun::PyramidSession);
        let mut state = WorkerState::Verifying;
        assert!(complete_session(
            &mut workflow,
            &mut route,
            &mut owner,
            &mut state,
            &completion,
            &selected,
            stopped,
        ));
        selected.generation = workflow
            .select_options(
                WorkflowMode::Manual,
                RouteMethod::Fast,
                selected.generation,
                true,
            )
            .expect("completed owner permits mode selection")
            .expect("Manual mode changed");
        workflow.bind(&selected);
        route.revoke_acquisition();
        let gates = step_gates(&selected, true, &route);
        assert!(gates.recon && !gates.plan && !gates.solve);
        let next = prepare_recon_request(&mut recon, &selected)
            .expect("Manual request replaces still-automatic names");
        assert_ne!(next.output_path, first.output_path);
        assert_ne!(next.deal_id, first.deal_id);
        assert!(!next.output_path.exists());
        assert!(next.initial_board_confirmed);
        assert!(recon.confirmed_context.is_none());
        assert!(recon.take_request(&selected).is_err());
        assert_eq!(std::fs::read(&first.output_path).unwrap(), published);
        let gates = step_gates(&selected, true, &route);
        assert!(gates.recon && !gates.plan && !gates.solve);
    }
}


/// Method changes revoke old preparation, and retries allocate without needing success events.
#[test]
fn manual_method_change_and_retry_prepare_distinct_requests_without_restoring_authority() {
    let directory = EvidenceDirectory::new();
    let mut recon = directory.form();
    let selected = context();
    let mut workflow = PyramidWorkflow::default();
    assert_eq!(workflow.route_goal(), RouteGoal::ShortestRoute);
    workflow.bind(&selected);
    let first = prepare_recon_request(&mut recon, &selected).unwrap();
    publish_request_fixture(&first.output_path);
    let published = std::fs::read(&first.output_path).unwrap();
    let mut route = ComputedRouteForm::default();
    route
        .acquisition_succeeded(selected.clone(), &first.output_path)
        .unwrap();
    route.plan = Some(ComputedRoutePlan {
        id: 7,
        context: selected.clone(),
    });
    assert_eq!(
        workflow
            .select_options(
                WorkflowMode::Manual,
                RouteMethod::Fast,
                selected.generation,
                true
            )
            .unwrap(),
        Some(selected.generation)
    );
    route.revoke_plan();
    let gates = step_gates(&selected, true, &route);
    assert!(gates.recon && gates.plan && !gates.solve);
    assert_eq!(std::fs::read(&first.output_path).unwrap(), published);
    let optional = prepare_recon_request(&mut recon, &selected).unwrap();
    assert_ne!(optional.output_path, first.output_path);
    assert!(!optional.output_path.exists());
}


/// Exact overrides survive preparation independently, including an already occupied literal path.
#[test]
fn prepared_recon_requests_preserve_literal_and_mixed_overrides_without_overwriting() {
    for (manual_output, manual_id) in [(true, true), (true, false), (false, true)] {
        let directory = EvidenceDirectory::new();
        let mut recon = directory.form();
        let selected = context();
        let original_output = recon.output_path.clone();
        let original_id = recon.deal_id.clone();
        let literal_output = directory.0.join(" literal output without extension ");
        publish_request_fixture(&literal_output);
        let published = std::fs::read(&literal_output).unwrap();
        if manual_output {
            recon.output_path = literal_output.to_str().unwrap().to_owned();
        }
        if manual_id {
            recon.deal_id = "Operator_ID".to_owned();
        }
        recon.edited(manual_output, manual_id);
        let request = prepare_recon_request(&mut recon, &selected)
            .expect("literal fields remain valid request values");
        if manual_output {
            assert_eq!(request.output_path, literal_output);
        } else {
            assert_ne!(request.output_path, PathBuf::from(original_output));
            assert!(!request.output_path.exists());
        }
        if manual_id {
            assert_eq!(request.deal_id, "Operator_ID");
        } else {
            assert_ne!(request.deal_id, original_id);
        }
        assert!(recon.confirmed_context.is_none());
        assert_eq!(std::fs::read(&literal_output).unwrap(), published);
        assert_eq!(std::fs::read_dir(&directory.0).unwrap().count(), 1);
    }
}


/// Failed automatic naming returns no request, creates no files and clears old consent.
#[test]
fn recon_request_naming_failures_leave_no_consent_or_new_evidence() {
    for unavailable in [true, false] {
        let directory = EvidenceDirectory::new();
        let mut recon = directory.form();
        let selected = context();
        let original_output = recon.output_path.clone();
        let original_id = recon.deal_id.clone();
        recon.confirmed_context = Some(selected.clone());
        recon.defaults = if unavailable {
            None
        } else {
            Some(EvidenceDefaults::in_directory(
                PathBuf::from(format!("/{}", "x".repeat(4_096))),
                std::process::id(),
            ))
        };
        let error = prepare_recon_request(&mut recon, &selected)
            .expect_err("unavailable or oversized naming cannot produce a request");
        assert!(!error.is_empty());
        assert!(recon.confirmed_context.is_none());
        assert_eq!(recon.output_path, original_output);
        assert_eq!(recon.deal_id, original_id);
        assert_eq!(std::fs::read_dir(&directory.0).unwrap().count(), 0);
        assert!(recon.take_request(&selected).is_err());
    }
}


/// Automatic phases retain one owner and discard earlier-board and wrong-session statistics.
#[test]
fn automatic_progress_matches_context_session_and_board_before_updating_statistics() {
    let context = context();
    let mut workflow = PyramidWorkflow::default();
    workflow.begin_session(&context, 7, serde_json::json!({}));
    let owner = Some(ActiveRun::PyramidSession);
    let first = session_update(&context, 7, 1, SessionPhase::Planning);
    assert!(!workflow.receive_session_progress(
        &first,
        &context,
        Some(ActiveRun::ComputedRoute),
        Some(7)
    ));
    assert!(!workflow.receive_session_progress(&first, &context, owner, Some(8)));
    assert!(workflow.receive_session_progress(&first, &context, owner, Some(7)));
    assert!(workflow.receive_planning(
        planning_update(&context, 7, 1, RouteMethod::Fast, false),
        &context
    ));
    assert_eq!(
        workflow
            .planning_progress()
            .expect("current board stats")
            .stats
            .generated,
        21
    );
    let next = session_update(&context, 7, 2, SessionPhase::Reconnaissance);
    assert!(workflow.receive_session_progress(&next, &context, owner, Some(7)));
    assert!(workflow.planning_progress().is_none());
    assert!(!workflow.receive_session_progress(&first, &context, owner, Some(7)));
    assert!(!workflow.receive_planning(
        planning_update(&context, 7, 1, RouteMethod::Fast, true),
        &context
    ));
    assert_eq!(workflow.phase(), Some(PyramidPhase::Reconnaissance));
    let planning = session_update(&context, 7, 2, SessionPhase::Planning);
    assert!(workflow.receive_session_progress(&planning, &context, owner, Some(7)));
    for (run_id, board, method) in [
        (6, 2, RouteMethod::Fast),
        (7, 1, RouteMethod::Fast),
        (7, 2, RouteMethod::FewestActions),
    ] {
        assert!(!workflow.receive_planning(
            planning_update(&context, run_id, board, method, true),
            &context
        ));
    }
    assert!(workflow.receive_planning(
        planning_update(&context, 7, 2, RouteMethod::Fast, true),
        &context
    ));
    assert!(
        workflow
            .select_options(
                WorkflowMode::Continuous,
                RouteMethod::FewestActions,
                context.generation,
                true
            )
            .expect("active session blocks preference changes")
            .is_none()
    );
}


/// A final slot notification arriving before queued phases survives those phases and finish.
#[test]
fn final_automatic_statistics_precede_phase_and_survive_session_finish() {
    let context = context();
    let mut form = acquired_route(&context);
    let id = form.begin_request().expect("automatic identity");
    let mut workflow = PyramidWorkflow::default();
    workflow.begin_session(&context, id, serde_json::json!({}));
    let mut owner = Some(ActiveRun::PyramidSession);
    let mut state = WorkerState::Capturing;
    let final_update = planning_update(&context, id, 1, RouteMethod::Fast, true);
    assert!(workflow.receive_planning(final_update.clone(), &context));
    for phase in [
        SessionPhase::Reconnaissance,
        SessionPhase::Planning,
        SessionPhase::Execution,
    ] {
        let update = session_update(&context, id, 1, phase);
        assert!(workflow.receive_session_progress(&update, &context, owner, Some(id)));
        let displayed = workflow
            .planning_progress()
            .expect("early final snapshot retained");
        assert!(displayed.finished);
        assert_eq!(displayed.stats, final_update.stats);
    }
    let completion = SessionCompletion {
        context: context.clone(),
        run_id: id,
        result: Ok("single board complete".to_owned()),
    };
    assert!(complete_session(
        &mut workflow,
        &mut form,
        &mut owner,
        &mut state,
        &completion,
        &context,
        false
    ));
    assert!(owner.is_none() && form.active_run_id.is_none());
    assert_eq!(state, WorkerState::Ready);
    assert_eq!(
        workflow
            .planning_progress()
            .expect("final snapshot survives finish")
            .stats,
        final_update.stats
    );
    assert!(workflow.receive_planning(final_update, &context));
    assert!(!workflow.receive_planning(
        planning_update(&context, id, 1, RouteMethod::Fast, false),
        &context
    ));
}


/// Early slots match the active session and advance only positive monotonic board identities.
#[test]
fn early_automatic_statistics_reject_stale_identity_and_reset_only_for_a_new_board() {
    let context = context();
    let mut changed = context.clone();
    changed.generation += 1;
    let mut workflow = PyramidWorkflow::default();
    workflow.begin_session(&context, 7, serde_json::json!({}));
    for (run_id, board, method) in [(6, 1, RouteMethod::Fast), (7, 0, RouteMethod::Fast)] {
        assert!(!workflow.receive_planning(
            planning_update(&context, run_id, board, method, true),
            &context
        ));
    }
    assert!(!workflow.receive_planning(
        planning_update(&changed, 7, 1, RouteMethod::Fast, true),
        &context
    ));
    assert!(!workflow.receive_planning(
        planning_update(&context, 7, 1, RouteMethod::Fast, true),
        &changed
    ));
    assert!(workflow.planning_progress().is_none());
    let final_update = planning_update(&context, 7, 2, RouteMethod::Fast, true);
    assert!(workflow.receive_planning(final_update.clone(), &context));
    assert!(!workflow.receive_planning(
        planning_update(&context, 7, 1, RouteMethod::Fast, true),
        &context
    ));
    assert!(!workflow.receive_session_progress(
        &session_update(&context, 7, 1, SessionPhase::Planning),
        &context,
        Some(ActiveRun::PyramidSession),
        Some(7)
    ));
    let mut older = final_update.clone();
    older.stats.elapsed = Duration::from_millis(1);
    assert!(!workflow.receive_planning(older, &context));
    assert!(!workflow.receive_planning(
        planning_update(&context, 7, 2, RouteMethod::Fast, false),
        &context
    ));
    assert_eq!(
        workflow
            .planning_progress()
            .expect("previous final unchanged")
            .stats,
        final_update.stats
    );
    let mut next = planning_update(&context, 7, 3, RouteMethod::FewestActions, false);
    next.stats.elapsed = Duration::from_millis(1);
    assert!(workflow.receive_planning(next.clone(), &context));
    assert_eq!(workflow.method(), RouteMethod::FewestActions);
    let mut phase = session_update(&context, 7, 3, SessionPhase::Planning);
    phase.effective_method = RouteMethod::FewestActions;
    assert!(workflow.receive_session_progress(
        &phase,
        &context,
        Some(ActiveRun::PyramidSession),
        Some(7)
    ));
    let displayed = workflow
        .planning_progress()
        .expect("current board snapshot retained");
    assert_eq!(displayed.board_generation, 3);
    assert_eq!(displayed.stats, next.stats);
    assert!(!displayed.finished);
}


/// A fast manual calculation's final snapshot survives completion but not a new step/context.
#[test]
fn final_manual_statistics_remain_visible_without_accepting_stale_updates() {
    let context = context();
    let mut workflow = PyramidWorkflow::default();
    workflow.begin(&context, PyramidPhase::Planning, serde_json::json!({}));
    let mut form = acquired_route(&context);
    let id = form.begin_request().expect("manual identity");
    workflow.begin_planning(&context, id);
    form.finish(
        context.clone(),
        id,
        Some(id),
        &Ok("route found".to_owned()),
        false,
    );
    let final_update = planning_update(&context, id, 0, RouteMethod::Fast, true);
    assert!(workflow.receive_planning(final_update.clone(), &context));
    assert!(!workflow.receive_planning(
        planning_update(&context, id, 0, RouteMethod::Fast, false),
        &context
    ));
    let mut changed = context.clone();
    changed.generation += 1;
    assert!(!workflow.receive_planning(final_update.clone(), &changed));
    workflow.begin(&context, PyramidPhase::Execution, serde_json::json!({}));
    assert!(workflow.planning_progress().is_none());
    assert!(!workflow.receive_planning(final_update, &context));
}


/// Only the matching session finish releases ownership; busy badges and STOP reconcile safely.
#[test]
fn automatic_completion_is_identity_bound_and_preserves_stop_and_detached_states() {
    let context = context();
    for (initial, stopped, succeeds, expected) in [
        (WorkerState::Capturing, false, true, WorkerState::Ready),
        (WorkerState::Acting, false, false, WorkerState::Uncertain),
        (WorkerState::Verifying, true, true, WorkerState::Uncertain),
        (WorkerState::Detached, true, false, WorkerState::Detached),
    ] {
        let mut form = acquired_route(&context);
        let id = form.begin_request().expect("session identity");
        let mut workflow = PyramidWorkflow::default();
        workflow.begin_session(&context, id, serde_json::json!({}));
        let mut owner = Some(ActiveRun::PyramidSession);
        let mut state = initial;
        let mut completion = SessionCompletion {
            context: context.clone(),
            run_id: id + 1,
            result: if succeeds {
                Ok("session complete".to_owned())
            } else {
                Err("original error".to_owned())
            },
        };
        assert!(!complete_session(
            &mut workflow,
            &mut form,
            &mut owner,
            &mut state,
            &completion,
            &context,
            stopped
        ));
        assert_eq!(form.active_run_id, Some(id));
        assert_eq!(owner, Some(ActiveRun::PyramidSession));
        completion.run_id = id;
        completion.context.generation += 1;
        assert!(!complete_session(
            &mut workflow,
            &mut form,
            &mut owner,
            &mut state,
            &completion,
            &context,
            stopped
        ));
        completion.context = context.clone();
        assert!(complete_session(
            &mut workflow,
            &mut form,
            &mut owner,
            &mut state,
            &completion,
            &context,
            stopped
        ));
        assert_eq!(state, expected);
        assert!(owner.is_none() && form.active_run_id.is_none());
        assert!(form.plan.is_none() && form.acquired_context.is_none());
        assert!(form.confirmed_context.is_none());
        assert!(!complete_session(
            &mut workflow,
            &mut form,
            &mut owner,
            &mut state,
            &completion,
            &context,
            stopped
        ));
    }
}


/// Every generic connection state preserves automatic ownership and keeps settings disabled.
#[test]
fn automatic_owner_survives_all_generic_worker_states_until_its_own_finish() {
    for state in [
        WorkerState::Detached,
        WorkerState::Ready,
        WorkerState::Uncertain,
        WorkerState::Error,
        WorkerState::Connecting,
        WorkerState::Capturing,
        WorkerState::Validating,
        WorkerState::Acting,
        WorkerState::Verifying,
    ] {
        let owner = active_run_after_worker_state(Some(ActiveRun::PyramidSession), state);
        assert_eq!(owner, Some(ActiveRun::PyramidSession));
        assert!(!controls_are_mutable(owner, state, false));
        assert!(!controls_are_mutable(owner, state, true));
    }
}


/// Failure releases only the matching run; recovery Recon and Start need no strategy toggle.
#[test]
fn failed_session_releases_optional_recon_and_new_start_in_same_context() {
    let context = context();
    let mut form = acquired_route(&context);
    let id = form.begin_request().unwrap();
    let mut workflow = PyramidWorkflow::default();
    workflow.begin_session(&context, id, serde_json::json!({}));
    let mut owner = Some(ActiveRun::PyramidSession);
    let mut state = WorkerState::Capturing;
    let completion = SessionCompletion {
        context: context.clone(),
        run_id: id,
        result: Err("retained first failure".to_owned()),
    };
    assert!(complete_session(
        &mut workflow,
        &mut form,
        &mut owner,
        &mut state,
        &completion,
        &context,
        false
    ));
    let idle = reconnaissance_controls_enabled(&context, false, owner, state, false, false, false);
    assert!(idle);
    for mode in [
        WorkflowMode::Manual,
        WorkflowMode::SingleBoard,
        WorkflowMode::Continuous,
    ] {
        assert!(step_request_enabled(
            mode,
            PyramidPhase::Reconnaissance,
            idle
        ));
        assert!(!step_request_enabled(
            mode,
            PyramidPhase::Reconnaissance,
            false
        ));
        assert_eq!(
            step_request_enabled(mode, PyramidPhase::Planning, idle),
            mode == WorkflowMode::Manual
        );
        assert_eq!(
            step_request_enabled(mode, PyramidPhase::Execution, idle),
            mode == WorkflowMode::Manual
        );
    }
    assert!(session_start_enabled(WorkflowMode::Continuous, idle, &form));
    assert!(form.acquired_context.is_none() && form.plan.is_none());
    assert!(!reconnaissance_controls_enabled(
        &context, false, owner, state, true, false, false
    ));
}


/// Automatic modes can request guarded native entry while Manual and busy states stay gated.
#[test]
fn continuous_start_retains_idle_and_active_request_gates() {
    let route = ComputedRouteForm::default();
    assert!(session_start_enabled(
        WorkflowMode::Continuous,
        true,
        &route
    ));
    assert!(
        WorkflowMode::Continuous
            .description()
            .contains("selected stop condition")
    );
    assert!(session_start_enabled(
        WorkflowMode::SingleBoard,
        true,
        &route
    ));
    assert!(!session_start_enabled(
        WorkflowMode::Continuous,
        false,
        &route
    ));
    assert!(!session_start_enabled(WorkflowMode::Manual, true, &route));
    let mut active = ComputedRouteForm::default();
    active.begin_request().expect("bounded test run");
    assert!(!session_start_enabled(
        WorkflowMode::Continuous,
        true,
        &active
    ));
}


/// Ordered, coalesced and acquisition advisory paths share one challenge identity until interruption.
#[test]
fn observed_challenge_latch_rejects_reclassification_and_late_interrupted_events() {
    use crate::pyramid_progress::{BoardCount, ChallengeKind, ChallengeProgress};
    use crate::pyramid_workflow_model::ObservedGame;
    let selected = context();
    let owner = Some(ActiveRun::PyramidSession);
    let bound = ChallengeProgress::new(ChallengeKind::Cards, 2, 10).unwrap();
    let locked = ObservedGame::Challenge {
        progress: bound,
        kings: true,
    };
    let mut workflow = PyramidWorkflow::default();
    workflow.begin_session(&selected, 7, serde_json::json!({}));
    workflow.apply_observed(locked);
    for observed in [
        ObservedGame::Unknown,
        ObservedGame::Normal(BoardCount {
            completed: 0,
            target: 3,
        }),
        ObservedGame::Challenge {
            progress: ChallengeProgress::new(ChallengeKind::Boards, 2, 3).unwrap(),
            kings: false,
        },
        ObservedGame::Challenge {
            progress: ChallengeProgress::new(ChallengeKind::Cards, 2, 9).unwrap(),
            kings: false,
        },
        ObservedGame::Challenge {
            progress: ChallengeProgress::new(ChallengeKind::Cards, 1, 10).unwrap(),
            kings: false,
        },
    ] {
        let mut ordered = session_update(&selected, 7, 1, SessionPhase::Planning);
        ordered.observed_game = observed;
        assert!(workflow.receive_session_progress(&ordered, &selected, owner, Some(7)));
        let mut coalesced = planning_update(&selected, 7, 1, ordered.effective_method, false);
        coalesced.observed_game = observed;
        assert!(workflow.receive_planning(coalesced, &selected));
        workflow.apply_observed(observed);
        assert_eq!(workflow.observed_game(), locked);
    }
    let advanced = ChallengeProgress::new(ChallengeKind::Cards, 3, 10).unwrap();
    workflow.apply_observed(ObservedGame::Challenge {
        progress: advanced,
        kings: false,
    });
    assert_eq!(
        workflow.observed_game(),
        ObservedGame::Challenge {
            progress: advanced,
            kings: true
        }
    );

    workflow.retire_observed();
    let mut late = session_update(&selected, 7, 2, SessionPhase::Planning);
    late.observed_game = locked;
    assert!(
        workflow.receive_session_progress(&late, &selected, owner, Some(7)),
        "STOP keeps its owner until settlement"
    );
    let mut coalesced = planning_update(&selected, 7, 2, late.effective_method, false);
    coalesced.observed_game = locked;
    assert!(workflow.receive_planning(coalesced, &selected));
    workflow.apply_observed(locked);
    assert_eq!(workflow.observed_game(), ObservedGame::Unknown);

    workflow.begin_session(&selected, 8, serde_json::json!({}));
    assert!(!workflow.receive_session_progress(&late, &selected, owner, Some(8)));
    workflow.apply_observed(locked);
    assert_eq!(
        workflow.observed_game(),
        locked,
        "explicit START opens a new lifetime"
    );
    workflow.reset_preferences();
    let mut reset_late = session_update(&selected, 8, 1, SessionPhase::Planning);
    reset_late.observed_game = locked;
    assert!(workflow.receive_session_progress(&reset_late, &selected, owner, Some(8)));
    workflow.apply_observed(locked);
    assert_eq!(workflow.observed_game(), ObservedGame::Unknown);
    workflow.begin(
        &selected,
        PyramidPhase::Reconnaissance,
        serde_json::json!({}),
    );
    workflow.apply_observed(locked);
    assert_eq!(
        workflow.observed_game(),
        locked,
        "explicit RECON opens a new lifetime"
    );
}


/// Construct an actual-owner-shaped advisory snapshot without granting worker authority.
fn search_owner(
    context: &ControllerContext,
    run_id: u64,
    board_generation: u64,
) -> crate::worker::pyramid_search_control::Snapshot {
    crate::worker::pyramid_search_control::Snapshot {
        token: 71,
        context: context.clone(),
        run_id,
        board_generation,
        method: RouteMethod::FewestActions,
        maximum_requested: false,
    }
}


/// Only an active, matching manual Shortest invocation enables the control.
#[test]
fn maximum_switch_availability_requires_actual_matching_manual_owner() {
    let selected = context();
    let mut workflow = PyramidWorkflow::default();
    workflow.begin(&selected, PyramidPhase::Planning, serde_json::json!({}));
    workflow.begin_planning(&selected, 42);
    let search = search_owner(&selected, 42, 0);
    let owner = Some(ActiveRun::ComputedRoute);
    assert!(workflow.maximum_switch_available(&search, &selected, owner, Some(42), false));
    assert!(!workflow.maximum_switch_available(&search, &selected, owner, Some(42), true));
    assert!(!workflow.maximum_switch_available(&search, &selected, None, Some(42), false));
    assert!(!workflow.maximum_switch_available(&search, &selected, owner, Some(43), false));
    let mut changed = search.clone();
    changed.context.generation += 1;
    assert!(!workflow.maximum_switch_available(&changed, &selected, owner, Some(42), false));
    changed = search.clone();
    changed.board_generation = 1;
    assert!(!workflow.maximum_switch_available(&changed, &selected, owner, Some(42), false));
    changed = search.clone();
    changed.method = RouteMethod::Fast;
    assert!(!workflow.maximum_switch_available(&changed, &selected, owner, Some(42), false));
    changed = search.clone();
    changed.maximum_requested = true;
    assert!(!workflow.maximum_switch_available(&changed, &selected, owner, Some(42), false));
    workflow.mark_maximum_requested(&search);
    assert!(workflow.maximum_pending());
    assert!(!workflow.maximum_switch_available(&search, &selected, owner, Some(42), false));
    assert_eq!(workflow.method(), RouteMethod::Fast);
    assert!(!workflow.receive_planning(
        planning_update(&selected, 42, 0, RouteMethod::FewestActions, true),
        &selected
    ));
    assert!(workflow.maximum_pending());
    assert!(workflow.receive_planning(
        planning_update(&selected, 42, 0, RouteMethod::Fast, false),
        &selected
    ));
    assert!(!workflow.maximum_pending());
    assert_eq!(workflow.method(), RouteMethod::Fast);
}


/// An accepted automatic switch rejects old phase/statistics rows without changing lifetime.
#[test]
fn maximum_pending_fences_delayed_exact_events_and_retires_on_new_board() {
    let selected = context();
    let mut workflow = PyramidWorkflow::default();
    workflow.begin_session(&selected, 42, serde_json::json!({}));
    let owner = Some(ActiveRun::PyramidSession);
    let mut exact = session_update(&selected, 42, 3, SessionPhase::Planning);
    exact.effective_method = RouteMethod::FewestActions;
    assert!(workflow.receive_session_progress(&exact, &selected, owner, Some(42)));
    let search = search_owner(&selected, 42, 3);
    assert!(workflow.maximum_switch_available(&search, &selected, owner, Some(42), false));
    let mut next = search.clone();
    next.board_generation += 1;
    assert!(!workflow.maximum_switch_available(&next, &selected, owner, Some(42), false));
    workflow.mark_maximum_requested(&search);
    assert!(workflow.maximum_pending());
    assert_eq!(workflow.mode(), WorkflowMode::Continuous);
    assert!(!workflow.receive_session_progress(&exact, &selected, owner, Some(42)));
    assert!(!workflow.receive_planning(
        planning_update(&selected, 42, 3, RouteMethod::FewestActions, true),
        &selected
    ));
    assert!(workflow.maximum_pending());
    exact.board_generation = 4;
    exact.phase = SessionPhase::Reconnaissance;
    assert!(workflow.receive_session_progress(&exact, &selected, owner, Some(42)));
    assert!(!workflow.maximum_pending());
    assert_eq!(workflow.method(), RouteMethod::FewestActions);
}


/// Completion, STOP and context/new-step changes clear the transient pending indicator.
#[test]
fn maximum_pending_is_cleared_on_each_display_retirement() {
    let selected = context();
    for retirement in 0..4 {
        let mut workflow = PyramidWorkflow::default();
        workflow.begin(&selected, PyramidPhase::Planning, serde_json::json!({}));
        workflow.begin_planning(&selected, 42);
        workflow.mark_maximum_requested(&search_owner(&selected, 42, 0));
        match retirement {
            0 => workflow.finish_search(),
            1 => workflow.retire_observed(),
            2 => {
                let mut changed = selected.clone();
                changed.generation += 1;
                workflow.bind(&changed);
            }
            _ => workflow.begin(&selected, PyramidPhase::Execution, serde_json::json!({})),
        }
        assert!(!workflow.maximum_pending());
    }
}
