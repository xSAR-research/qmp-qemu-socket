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
        let accepted = pyramid_workflow::complete_session(
            &mut self.pyramid_workflow,
            &mut self.computed_route_form,
            &mut self.active_run,
            &mut self.worker_state,
            &completion,
            &selected,
            self.stop_requested,
        );
        if !accepted {
            self.push_log(format!(
                "Ignored Pyramid session completion for inactive request {}.",
                completion.run_id
            ));
            return;
        }
        if super::reset_preview::dispatch_completion(accepted, completion.termination, || {
            self.current_status = "Waiting".to_owned();
            if let Some(report) = self.computed_route_form.report.clone() {
                self.push_log(report);
            }
            self.begin_stop_preview(selected.clone());
        }) == super::reset_preview::CompletionDispatch::StopPreview
        {
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


    /// Apply advisory progress emitted by the worker-owned automatic Free Cell session.
    pub(super) fn apply_freecell_session_progress(
        &mut self,
        progress: crate::pyramid_workflow_model::FreeCellSessionProgress,
    ) {
        let selected = self.current_preview_context();
        if self
            .freecell_workflow
            .receive_session_progress(&progress, &selected, self.active_run)
        {
            self.current_status = progress.message;
        } else {
            self.push_log("Ignored Free Cell session progress from an earlier request.");
        }
    }


    /// Reconcile the terminal Free Cell automatic session event and release UI ownership.
    pub(super) fn apply_freecell_session_finished(
        &mut self,
        completion: crate::pyramid_workflow_model::FreeCellSessionCompletion,
    ) {
        let selected = self.current_preview_context();
        let accepted = pyramid_workflow::complete_freecell_session(
            &mut self.freecell_workflow,
            &mut self.active_run,
            &mut self.worker_state,
            &completion,
            &selected,
            self.stop_requested,
        );
        if !accepted {
            self.push_log(format!(
                "Ignored Free Cell session completion for inactive request {}.",
                completion.run_id
            ));
            return;
        }
        if super::reset_preview::dispatch_completion(accepted, completion.termination, || {
            self.current_status = "Waiting".to_owned();
            if let Some(report) = self.freecell_workflow.report.clone() {
                self.push_log(report);
            }
            self.begin_stop_preview(selected.clone());
        }) == super::reset_preview::CompletionDispatch::StopPreview
        {
            return;
        }
        if let Err(error) = &completion.result {
            self.failure_window.general(
                "Free Cell session",
                error,
                serde_json::json!({
                    "run_id": completion.run_id,
                    "completed_games": completion.completed_games,
                    "export_path": completion.export_path,
                }),
            );
        }
        self.current_status = if completion.result.is_err() {
            "Free Cell session stopped — inspect the board"
        } else if let Some(cards) = completion.unsolvable_foundation_cards {
            let _ = cards;
            "Unsolvable deal proved — deal a new game and press START"
        } else if self.stop_requested {
            "Free Cell session finished after STOP — no resume"
        } else {
            "Free Cell session finished"
        }
        .to_owned();
        if let Some(report) = self.freecell_workflow.report.clone() {
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
        termination: crate::pyramid_workflow_model::RunTermination,
    ) {
        let selected = self.current_preview_context();
        let phase = self
            .pyramid_workflow
            .phase()
            .unwrap_or(PyramidPhase::Execution);
        let accepted = pyramid_workflow::complete_route(
            &mut self.computed_route_form,
            &mut self.active_run,
            &mut self.worker_state,
            pyramid_workflow::RouteCompletion {
                retained_recon,
                termination,
                context: selected.clone(),
                run_id,
                plan_id,
                result: &result,
                stopped: self.stop_requested,
            },
        );
        if !accepted {
            self.push_log(format!(
                "Ignored computed-route completion for inactive request {run_id}."
            ));
            return;
        }
        self.pyramid_workflow.finish_search();
        if super::reset_preview::dispatch_completion(accepted, termination, || {
            self.current_status = "Waiting".to_owned();
            if let Some(report) = self.computed_route_form.report.clone() {
                self.push_log(report);
            }
            self.begin_stop_preview(selected.clone());
        }) == super::reset_preview::CompletionDispatch::StopPreview
        {
            return;
        }
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


    /// Queue one raw preview behind the already queued Disconnect, retaining STOP and history.
    pub(super) fn begin_stop_preview(&mut self, context: ControllerContext) {
        if self.exiting || self.active_run.is_some() || self.current_preview_context() != context {
            return;
        }
        let Some(request_id) = self.reset_preview_serial.checked_add(1) else {
            self.worker_state = WorkerState::Error;
            self.pyramid_ui_failure(
                PyramidPhase::Reconnaissance,
                "STOP preview request identity exhausted".to_owned(),
            );
            return;
        };
        self.reset_preview_serial = request_id;
        let _ = self.worker.take_latest_frame();
        self.invalidate_preview();
        self.stop_requested = true;
        self.reset_preview = Some(super::reset_preview::ResetPreview::after_stop(
            request_id,
            context.clone(),
            self.worker_state,
        ));
        self.worker_state = WorkerState::Connecting;
        if let Err(error) = self.worker.capture_reset_preview(context, request_id) {
            self.reset_preview = None;
            self.worker_state = WorkerState::Error;
            self.pyramid_ui_failure(PyramidPhase::Reconnaissance, error);
        }
    }


    /// Revoke common input authority before requesting cooperative disconnection.
    ///
    /// Caller-owned failure presentation and active-run retirement remain separate.
    pub(super) fn request_stop(&mut self) -> Result<(), String> {
        self.trace_freecell_ui("STOP-request", serde_json::json!({
            "owner":format!("{:?}",self.active_run), "stop_already_requested":self.stop_requested,
        }));
        if self.active_run == Some(ActiveRun::FreeCellCalculation) {
            self.worker.cancel_freecell_calculation();
            self.stop_requested = true;
            self.current_status = "Cancelling offline calculation…".to_owned();
            self.trace_freecell_ui(
                "STOP-result",
                serde_json::json!({
                    "requested":true,"kind":"offline cancellation","guest_input_attempted":false,
                }),
            );
            return Ok(());
        }
        self.reset_preview = None;
        self.stop_requested = true;
        self.pyramid_workflow.retire_observed();
        self.reconnaissance_form.confirmed_context = None;
        self.computed_route_form.revoke_acquisition();
        self.current_status = "Stopping…".to_owned();
        let result = self.worker.disconnect(self.current_preview_context());
        self.revoke_freecell_playback();
        self.trace_freecell_ui(
            "STOP-result",
            serde_json::json!({
                "queued":result.is_ok(),"error":result.as_ref().err(),
                "authority_revoked":true,"guest_effect_proved":false,
            }),
        );
        result
    }
}
