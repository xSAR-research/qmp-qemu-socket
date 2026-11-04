//! Private Pyramid request boundaries and identity-bound completion presentation.
//!
//! Planning remains input-free; execution consumes a separately validated plan.
//! Context filtering, RESET correlation and preview delivery stay with the poller.

use super::*;
use crate::pyramid_workflow_model::{SessionCompletion, SessionProgress};


impl QmpQemuSocketApp {
    /// Return the selected context only through the existing idle request gates.
    fn route_request_context(&mut self) -> Option<ControllerContext> {
        let context = self.current_preview_context();
        if !reconnaissance_controls_enabled(
            &context,
            self.strategy_choice_pending,
            self.active_run,
            self.worker_state,
            self.stop_requested,
            self.show_snapshot_dialog,
            self.snapshot_capture_pending,
        ) {
            self.computed_route_form.confirmed_context = None;
            self.push_log("Computed route refused: wait for an idle Pyramid Shortest path worker, closed snapshot review and completed STOP.");
            return None;
        }
        Some(context)
    }


    /// Reserve request identity only after the caller validates its distinct prerequisites.
    fn begin_route_request(&mut self, phase: PyramidPhase) -> Option<u64> {
        let run_id = match self.computed_route_form.begin_request() {
            Ok(id) => id,
            Err(error) => {
                self.pyramid_ui_failure(phase, error.to_owned());
                return None;
            }
        };
        self.active_run = Some(ActiveRun::ComputedRoute);
        Some(run_id)
    }


    /// Calculate only from the current restored acquisition, without guest input.
    pub(super) fn dispatch_route_planning(&mut self) {
        let Some(context) = self.route_request_context() else {
            return;
        };
        if !self.computed_route_form.calculation_ready(&context) {
            self.pyramid_ui_failure(PyramidPhase::Planning, "Complete a successful restored acquisition in this context first, then select its exact exported JSON pathname.".to_owned());
            return;
        }
        let Some(run_id) = self.begin_route_request(PyramidPhase::Planning) else {
            return;
        };
        self.pyramid_workflow.begin_planning(&context, run_id);
        self.worker_state = WorkerState::Validating;
        self.current_status = "Calculating route without guest input".to_owned();
        self.invalidate_preview();
        self.push_log("Pyramid route calculation requested without guest input. The selected document must match the successful same-session restored acquisition byte for byte.");
        let queued = self.worker.calculate_route_with_goal(
            context,
            run_id,
            PathBuf::from(&self.computed_route_form.deal_path),
            self.pyramid_workflow.route_goal(),
        );
        self.finish_route_dispatch(PyramidPhase::Planning, queued);
    }


    /// Consume one confirmed plan only after execution-specific admission succeeds.
    pub(super) fn dispatch_route_execution(&mut self) {
        let Some(context) = self.route_request_context() else {
            return;
        };
        let plan_id = match self.computed_route_form.execution_plan(&context) {
            Ok(id) => id,
            Err(error) => {
                self.pyramid_ui_failure(PyramidPhase::Execution, error.to_owned());
                return;
            }
        };
        let Some(run_id) = self.begin_route_request(PyramidPhase::Execution) else {
            return;
        };
        self.computed_route_form.revoke_acquisition();
        self.worker_state = WorkerState::Validating;
        self.current_status = "Checking computed route before execution".to_owned();
        self.invalidate_preview();
        self.push_log("Computed Pyramid route execution requested with Solver not engaged (unavailable is allowed) and foreground/no-interference confirmation. The finite route stops on uncertain delivery or unverified effect; a failed run cannot resume.");
        let queued = self.worker.execute_route(context, run_id, plan_id, true);
        self.finish_route_dispatch(PyramidPhase::Execution, queued);
    }


    /// Retire a rejected dispatch without leaving a route, acquisition or UI owner.
    fn finish_route_dispatch(&mut self, phase: PyramidPhase, queued: Result<(), String>) {
        if let Err(error) = queued {
            self.computed_route_form.active_run_id = None;
            self.computed_route_form.revoke_acquisition();
            self.active_run = None;
            self.worker_state = WorkerState::Error;
            self.current_status = "Computed route dispatch failed".to_owned();
            let report = format!("Computed route request was not accepted: {error}");
            self.computed_route_form.report = Some(report.clone());
            self.pyramid_ui_failure(phase, report);
        }
    }


    /// Apply progress only to the matching automatic request and observed board.
    pub(super) fn apply_pyramid_session_progress(&mut self, progress: SessionProgress) {
        let selected = self.current_preview_context();
        if self.pyramid_workflow.receive_session_progress(
            &progress,
            &selected,
            self.active_run,
            self.computed_route_form.active_run_id,
        ) {
            self.current_status = progress.message;
        } else {
            self.push_log("Ignored Pyramid session progress from an earlier request or board.");
        }
    }


    /// Reconcile only the matching automatic completion through the existing reducer.
    pub(super) fn apply_pyramid_session_finished(&mut self, completion: SessionCompletion) {
        let selected = self.current_preview_context();
        let phase = self
            .pyramid_workflow
            .phase()
            .unwrap_or(PyramidPhase::Execution);
        if !pyramid_workflow::complete_session(
            &mut self.pyramid_workflow,
            &mut self.computed_route_form,
            &mut self.active_run,
            &mut self.worker_state,
            &completion,
            &selected,
            self.stop_requested,
        ) {
            self.push_log(format!(
                "Ignored Pyramid session completion for inactive request {}.",
                completion.run_id
            ));
            return;
        }
        if let Err(error) = &completion.result
            && !self.pyramid_workflow.details_visible(&selected)
        {
            self.pyramid_ui_failure(phase, error.clone());
        }
        self.current_status = if completion.result.is_err() {
            "Pyramid session stopped — inspect the board"
        } else if self.stop_requested {
            "Pyramid session finished after STOP — no resume"
        } else {
            "Pyramid session finished"
        }
        .to_owned();
        if let Some(report) = self.computed_route_form.report.clone() {
            self.push_log(report);
        }
        if self.worker_state == WorkerState::Detached {
            self.stop_requested = false;
        }
    }


    /// Reconcile a matching calculation or execution completion without resume authority.
    pub(super) fn apply_computed_route_finished(
        &mut self,
        retained_recon: bool,
        run_id: u64,
        plan_id: Option<u64>,
        result: Result<String, String>,
    ) {
        let selected = self.current_preview_context();
        let phase = self
            .pyramid_workflow
            .phase()
            .unwrap_or(PyramidPhase::Execution);
        if !pyramid_workflow::complete_route(
            &mut self.computed_route_form,
            &mut self.active_run,
            &mut self.worker_state,
            pyramid_workflow::RouteCompletion {
                retained_recon,
                context: selected,
                run_id,
                plan_id,
                result: &result,
                stopped: self.stop_requested,
            },
        ) {
            self.push_log(format!(
                "Ignored computed-route completion for inactive request {run_id}."
            ));
            return;
        }
        self.pyramid_workflow.finish_search();
        if let Err(error) = &result
            && !self
                .pyramid_workflow
                .details_visible(&self.current_preview_context())
        {
            self.pyramid_ui_failure(phase, error.clone());
        }
        self.current_status = if result.is_err() {
            "Computed route stopped — inspect the board"
        } else if self.computed_route_form.plan.is_some() {
            "Route planned — press Solve board when ready"
        } else if self.stop_requested {
            "Computed route finished after STOP — route revoked"
        } else {
            "Computed route execution finished"
        }
        .to_owned();
        if let Some(report) = self.computed_route_form.report.clone() {
            self.push_log(report);
        }
        if self.worker_state == WorkerState::Detached {
            self.stop_requested = false;
        }
    }


    /// Revoke common input authority before requesting cooperative disconnection.
    ///
    /// Caller-owned failure presentation and active-run retirement remain separate.
    pub(super) fn request_stop(&mut self) -> Result<(), String> {
        self.reset_preview = None;
        self.stop_requested = true;
        self.pyramid_workflow.retire_observed();
        self.reconnaissance_form.confirmed_context = None;
        self.computed_route_form.revoke_acquisition();
        self.current_status = "Stopping…".to_owned();
        self.worker.disconnect(self.current_preview_context())
    }
}
