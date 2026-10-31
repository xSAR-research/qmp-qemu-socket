//! Worker-owned QMP and file adapters for the independent Pyramid scan.
//!
//! The pure controller owns recognition, ordering and restoration policy. This
//! adapter owns transport, immediate context revocation, original PNG evidence
//! and publication. It never retries uncertain input or restores after STOP.
//! Diagnostics retain at most two original PNG files and one latest PNG buffer;
//! normal capture decoding and the existing coalesced preview remain unchanged.

use std::{
    fs::{self, OpenOptions},
    io::{ErrorKind, Write},
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use xsar::games::pyramid::CompleteDeal;
use xsar::pyramid_json::{PublicationStatus, PublishError};

use crate::{
    capture::CapturedFrame,
    game::GameMode,
    parameters::{NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH, SNAPSHOT_MAX_PNG_BYTES},
    pyramid_diagnostics::{FailureEvidence, PyramidPhase},
    pyramid_recon::{
        self, ReconInput, ReconIo, ReconProfileSource, ReconProgress, ReconRequest,
        vision::{BoardObservation, ReconScene, ReconVision, StockFace},
    },
    qmp::QmpClient,
    strategy::{ControllerContext, SolvingStrategy},
};

use super::{
    WorkerEvent, WorkerEventSink, WorkerState, cancellable_wait, capture_screen_with_png,
    publish_pyramid_failure, send_log, send_state, send_status, validate_probe,
};


/// Per-process collision counter; directory creation remains exclusive.
static DIAGNOSTIC_SEQUENCE: AtomicU64 = AtomicU64::new(0);


/// Maximum collision attempts for one private diagnostic directory.
const DIRECTORY_ATTEMPTS: usize = 32;


/// Maximum characters retained from a failure in bounded diagnostic JSON.
const ERROR_CHARACTERS: usize = 2_048;


/// Complete bounded JSON diagnostics must fit within this independent limit.
const DIAGNOSTIC_BYTES: usize = 65_536;


/// Selection and monotonic cancellation state for one queued or running scan.
#[derive(Default)]
struct ReconControlState {
    /// Most recently requested socket, game, strategy and generation.
    requested: Option<ControllerContext>,
    /// Reservation begins before queueing, preventing another run clearing STOP.
    active: bool,
    /// Revocation survives selecting the original context again.
    revoked: bool,
    /// Monotonic identity also invalidates idle acquisition receipts.
    epoch: u64,
    /// Exhaustion permanently refuses fresh authority instead of reusing an epoch.
    epoch_exhausted: bool,
}


/// Shared request token held by the UI handle and worker, not by queued commands alone.
#[derive(Clone, Default)]
pub(super) struct ReconControl {
    /// Serialises context updates, reservations and STOP clearing.
    state: Arc<Mutex<ReconControlState>>,
}


impl ReconControl {
    /// Record selection immediately, revoking an active scan before queue dispatch.
    pub(super) fn select(
        &self,
        context: &ControllerContext,
        cancel: &AtomicBool,
    ) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|_| {
            cancel.store(true, Ordering::Release);
            "Reconnaissance request lock is poisoned; STOP remains requested".to_owned()
        })?;


        if state.epoch_exhausted {
            cancel.store(true, Ordering::Release);
            return Err("Controller epoch exhausted; restart required".to_owned());
        }
        if state.requested.as_ref() != Some(context) {
            match state.epoch.checked_add(1) {
                Some(epoch) => state.epoch = epoch,
                None => {
                    state.epoch_exhausted = true;
                    state.revoked = true;
                    cancel.store(true, Ordering::Release);
                    return Err("Controller epoch exhausted; restart required".to_owned());
                }
            }
        }


        if state.active && state.requested.as_ref() != Some(context) {
            state.revoked = true;
            cancel.store(true, Ordering::Release);
        }
        state.requested = Some(context.clone());
        Ok(())
    }


    /// Reserve one scan and clear earlier STOP only while no scan owns the latch.
    pub(super) fn queue_reconnaissance(
        &self,
        context: &ControllerContext,
        cancel: &AtomicBool,
        send: impl FnOnce() -> Result<(), String>,
    ) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Reconnaissance request lock is poisoned".to_owned())?;


        if state.epoch_exhausted {
            cancel.store(true, Ordering::Release);
            return Err("Controller epoch exhausted; restart required".to_owned());
        }
        if state.active {
            return Err(
                "Reconnaissance is already queued or running; its STOP cannot be cleared"
                    .to_owned(),
            );
        }


        if state.requested.as_ref() != Some(context) {
            return Err("Reconnaissance context changed before queueing".to_owned());
        }
        state.active = true;
        state.revoked = false;
        cancel.store(false, Ordering::Release);
        let result = send();


        if result.is_err() {
            state.active = false;
            state.revoked = true;
            cancel.store(true, Ordering::Release);
        }
        result
    }


    /// Preserve ordinary CV start behaviour, except while reconnaissance owns STOP.
    pub(super) fn queue_execution(
        &self,
        cancel: &AtomicBool,
        send: impl FnOnce() -> Result<(), String>,
    ) -> Result<(), String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "Reconnaissance request lock is poisoned".to_owned())?;


        if state.epoch_exhausted {
            cancel.store(true, Ordering::Release);
            return Err("Controller epoch exhausted; restart required".to_owned());
        }
        if state.active {
            return Err(
                "A reconnaissance run still owns STOP; wait for its finished event".to_owned(),
            );
        }
        cancel.store(false, Ordering::Release);
        let result = send();
        drop(state);
        result
    }


    /// Make STOP monotonic for the reserved scan, even if another request arrives.
    pub(super) fn stop(&self, context: Option<&ControllerContext>, cancel: &AtomicBool) {
        let state = self.state.lock();
        cancel.store(true, Ordering::Release);


        if let Ok(mut state) = state {
            state.revoked = true;
            match state.epoch.checked_add(1) {
                Some(epoch) => state.epoch = epoch,
                None => state.epoch_exhausted = true,
            }


            if let Some(context) = context {
                state.requested = Some(context.clone());
            }
        }
    }


    /// Fail closed on cancellation, context drift, missing reservation or poisoned state.
    pub(super) fn check_active(
        &self,
        context: &ControllerContext,
        cancel: &AtomicBool,
    ) -> Result<(), String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "Reconnaissance request lock is poisoned".to_owned())?;


        if cancel.load(Ordering::Acquire) || state.revoked || state.epoch_exhausted {
            return Err(
                "STOP or changed context revoked reconnaissance; no further input is permitted"
                    .to_owned(),
            );
        }


        if !state.active || state.requested.as_ref() != Some(context) {
            return Err("Reconnaissance no longer owns the selected controller context".to_owned());
        }
        Ok(())
    }


    /// Return the current authority epoch; poison never authorises a new receipt.
    pub(super) fn epoch(&self) -> Result<u64, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "Controller authority lock poisoned".to_owned())?;
        if state.epoch_exhausted {
            return Err("Controller epoch exhausted; restart required".to_owned());
        }
        Ok(state.epoch)
    }


    /// Release only after the worker has stopped all reconnaissance operations.
    pub(super) fn finish(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.active = false;
        }
    }
}


/// Require separate Pyramid consent without changing ShortestPath input capability.
pub(super) fn validate_request(
    context: &ControllerContext,
    request: &ReconRequest,
) -> Result<(), String> {
    context.validate()?;


    if context.strategy != SolvingStrategy::ShortestPath || context.game_mode != GameMode::Pyramid {
        return Err(
            "Reconnaissance requires Pyramid and the independent ShortestPath selection".to_owned(),
        );
    }


    if !request.initial_board_confirmed {
        return Err("Confirm an untouched Pyramid board with Solver not engaged (unavailable is allowed) before reconnaissance".to_owned());
    }
    request.validate()?;
    validate_output_spelling(&request.output_path)
}


/// Attempt one scan and report its final outcome without retry or cleanup input.
pub(super) fn run(
    context: &ControllerContext,
    request: &ReconRequest,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) {
    let result = run_phase(
        context,
        request,
        events,
        cancel,
        ReconRunOwner::Manual,
        None,
    );
    let default_method = result.as_ref().ok().and_then(|path| {
        let method =
            super::computed_route::acquisition_default(context, path, events, cancel).ok()?;
        let observed =
            super::computed_route::acquisition_observed(context, path, events, cancel).ok()?;
        Some((context.clone(), method, observed))
    });
    events.latest_frame.reconnaissance.finish();


    match &result {
        Ok(output) => {
            send_state(events, WorkerState::Ready);
            send_status(
                events,
                "Pyramid reconnaissance complete — restored deal exported".to_owned(),
            );
            send_log(
                events,
                format!(
                    "Complete restored Pyramid deal published at {output:?}. No search or Solver input was performed."
                ),
            );
        }
        Err(error) => {
            send_state(events, WorkerState::Uncertain);
            send_status(
                events,
                "Pyramid reconnaissance stopped — inspect diagnostics".to_owned(),
            );
            send_log(
                events,
                format!(
                    "Pyramid reconnaissance stopped: {error}. No uncertain input or restoration was retried."
                ),
            );
        }
    }
    let _ = events.send(WorkerEvent::ReconnaissanceFinished {
        result,
        default_method,
    });
}


/// Diagnostic identity of the caller that continues to own the reconnaissance reservation.
enum ReconRunOwner {
    /// Existing explicit manual acquisition retains its original completion event.
    Manual,
    /// A serial automatic session retains authority beyond this phase's return.
    Automatic {
        /// Immutable parent-session identity used by failure evidence and UI correlation.
        run_id: u64,
        /// Logical board generation whose acquisition failed or completed.
        board_generation: u64,
    },
}


/// Previously qualified challenge identity carried through same-session stock reconnaissance.
///
/// It supplies acquisition metadata only; position, restoration and active ownership
/// remain independently checked by the existing reconnaissance controller.
#[derive(Clone, Copy)]
pub(super) struct LockedChallenge {
    /// Most recent safe complete-board counter, including the immutable kind and target.
    pub(super) progress: crate::pyramid_progress::ChallengeProgress,
    /// Qualified Kings artwork selects the existing Fast default without reclassification.
    pub(super) kings: bool,
}


/// Additional same-session constraints borrowed from an independently qualified entry witness.
///
/// This guard does not grant capture, publication or input authority. Its pure
/// header predicate is evaluated only on the same already-decoded Board frame;
/// the existing controller still owns active reservation and restoration proof.
#[derive(Clone, Copy)]
pub(super) struct StartingGuard<'a> {
    /// Original complete tableau and initial piles retained by the session caller.
    pub(super) position: &'a BoardObservation,
    /// Caller-owned validation of that frame's retained session/header identity.
    pub(super) header: &'a dyn Fn(&CapturedFrame) -> Result<(), String>,
    /// Optional session-qualified challenge retained without reading action-frame headers.
    pub(super) challenge: Option<LockedChallenge>,
}


impl StartingGuard<'_> {
    /// Require the original tableau and header on every positive Board observation.
    ///
    /// Before any D attempt, both initial piles must also match. Later pile changes
    /// and enabled Undo All are left to the existing stock/restoration controller.
    /// Positive confirmation dialogs and unknown scenes retain that controller's
    /// original handling; neither invokes the extra header predicate.
    fn validate(
        &self,
        frame: &CapturedFrame,
        scene: &ReconScene,
        draws_attempted: usize,
    ) -> Result<(), String> {
        let ReconScene::Board(board) = scene else {
            return Ok(());
        };
        if board.tableau != self.position.tableau {
            return Err("Reconnaissance starting guard rejected a changed original tableau; no further input authorised".to_owned());
        }
        if draws_attempted == 0
            && (board.left != self.position.left || board.right != self.position.right)
        {
            return Err("Reconnaissance starting guard rejected changed initial LEFT/RIGHT piles before D; no input authorised".to_owned());
        }
        (self.header)(frame)
            .map_err(|error| format!("Reconnaissance starting guard header mismatch: {error}"))
    }
}


/// Complete one owned acquisition phase without finishing its parent session or reservation.
///
/// Returns the same published restored-deal path as manual reconnaissance. The
/// caller owns STOP/context checks between phases and eventual reservation release.
/// Errors retain the original first-failure diagnostics with parent-session and
/// board identity; no extra capture, transport operation or retry is introduced.
pub(super) fn run_owned(
    context: &ControllerContext,
    request: &ReconRequest,
    run_id: u64,
    board_generation: u64,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> Result<PathBuf, String> {
    run_phase(
        context,
        request,
        events,
        cancel,
        ReconRunOwner::Automatic {
            run_id,
            board_generation,
        },
        None,
    )
}


/// Bind an owned acquisition to the caller's qualified same-session starting witness.
///
/// Uses the existing captures, reservation, selected profile and restoration
/// pipeline. Guard failure retains normal first/latest PNG diagnostics and aborts
/// without retry, restoration cleanup or publication. The borrowed predicate is
/// validation only; it must not perform input or grant authority.
///
/// # Errors
///
/// Returns the existing acquisition/transport/publication error or the first
/// starting-tableau, initial-pile or header mismatch, with normal diagnostics.
pub(super) fn run_owned_with_starting_guard(
    context: &ControllerContext,
    request: &ReconRequest,
    run_id: u64,
    board_generation: u64,
    guard: StartingGuard<'_>,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) -> Result<PathBuf, String> {
    run_phase(
        context,
        request,
        events,
        cancel,
        ReconRunOwner::Automatic {
            run_id,
            board_generation,
        },
        Some(guard),
    )
}


/// Share checked acquisition and first-failure publication without releasing caller authority.
fn run_phase(
    context: &ControllerContext,
    request: &ReconRequest,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
    owner: ReconRunOwner,
    starting_guard: Option<StartingGuard<'_>>,
) -> Result<PathBuf, String> {
    send_state(events, WorkerState::Validating);
    let mut details = serde_json::json!({
        "requested_output_debug": format!("{:?}", request.output_path),
        "requested_deal_id": request.deal_id,
        "profile_source_debug": format!("{:?}", request.profile),
        "explicit_recon_request": request.initial_board_confirmed,
        "no_input_retry": true,
        "stop_revokes_restoration": true,
    });
    let run_id = match owner {
        ReconRunOwner::Manual => None,
        ReconRunOwner::Automatic {
            run_id,
            board_generation,
        } => {
            details["session_run_id"] = serde_json::json!(run_id);
            details["board_generation"] = serde_json::json!(board_generation);
            details["automatic_phase"] = serde_json::json!("reconnaissance");
            Some(run_id)
        }
    };
    let result = run_checked(
        context,
        request,
        events,
        cancel,
        &mut details,
        starting_guard,
    );
    if let Err(error) = &result {
        let count = |name: &str| details.get(name).and_then(serde_json::Value::as_u64).and_then(|n| usize::try_from(n).ok()).unwrap_or(0);
        let acknowledged = |name: &str| details.get(name).and_then(serde_json::Value::as_bool).unwrap_or(false);
        let substage = if count("confirmation_attempted") != 0 {
            "Original-board restoration after confirmation"
        } else if count("undo_all_attempted") != 0 {
            "Undo All / confirmation dialog"
        } else { "Stock identity acquisition" };
        let progress = crate::pyramid_diagnostics::progress::FailureProgress {
            stage: crate::pyramid_diagnostics::progress::FailureStage::Reconnaissance,
            operation: Some(format!("{substage}; D acknowledged {}/{}; Undo All acknowledged={}; confirmation acknowledged={}; publication={}", count("draws_acknowledged"), count("draws_attempted"), acknowledged("undo_all_acknowledged"), acknowledged("confirmation_acknowledged"), details.get("publication_status").and_then(serde_json::Value::as_str).unwrap_or("unavailable"))),
            capture: count("captures_attempted"),
            attempted: count("draws_attempted") + count("undo_all_attempted") + count("confirmation_attempted"),
            acknowledged: count("draws_acknowledged") + usize::from(acknowledged("undo_all_acknowledged")) + usize::from(acknowledged("confirmation_acknowledged")),
            reason: Some(error.clone()),
            ..Default::default()
        };
        publish_pyramid_failure(
            events,
            FailureEvidence {
                progress: Some(&progress),
                context,
                phase: PyramidPhase::Reconnaissance,
                run_id,
                error,
                details,
                source_document: None,
                first_frame: None,
                latest_frame: None,
            },
        );
    }
    result
}


/// Load exactly the requested strict calibration; a failed file never falls back to bundled data.
///
/// This helper performs only the selected profile load and grants no input or
/// reservation authority. The caller retains request validation and cancellation.
pub(super) fn load_vision(request: &ReconRequest) -> Result<ReconVision, String> {
    match &request.profile {
        ReconProfileSource::BundledNativeV23 => ReconVision::bundled_native_v23(),
        #[cfg(test)]
        ReconProfileSource::BundledNativeV22 => ReconVision::bundled_native_v22(),
        #[cfg(test)]
        ReconProfileSource::BundledNativeV17 => ReconVision::bundled_native_v17(),
        ReconProfileSource::File(path) => ReconVision::load(path),
    }
}


/// Validate the profile and output before connecting; retain diagnostics through every core failure.
fn run_checked(
    context: &ControllerContext,
    request: &ReconRequest,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
    details: &mut serde_json::Value,
    starting_guard: Option<StartingGuard<'_>>,
) -> Result<PathBuf, String> {
    let control = &events.latest_frame.reconnaissance;
    validate_request(context, request)?;
    control.check_active(context, cancel)?;
    let vision = load_vision(request)?;
    control.check_active(context, cancel)?;
    let source = match &request.profile {
        ReconProfileSource::BundledNativeV23 => {
            "BundledNativeV23 (colour-independent Classic A* reader)".to_owned()
        }
        #[cfg(test)]
        ReconProfileSource::BundledNativeV22 => {
            "BundledNativeV22 (embedded in this executable)".to_owned()
        }
        #[cfg(test)]
        ReconProfileSource::BundledNativeV17 => {
            "BundledNativeV17 (embedded in this executable)".to_owned()
        }
        ReconProfileSource::File(path) => format!("File, path={path:?}"),
    };
    send_log(
        events,
        format!(
            "Pyramid reconnaissance calibration loaded: profile_id={}, source={source}. Profile identity is provenance, not input authority.",
            vision.profile_id(),
        ),
    );
    let output = resolve_new_output(&request.output_path)?;
    let mut diagnostics = Diagnostics::reserve(&output)?;
    if let Some(object) = details.as_object_mut() {
        object.insert(
            "native_diagnostics_directory_debug".to_owned(),
            serde_json::json!(format!("{:?}", diagnostics.directory)),
        );
        object.insert("native_evidence_note".to_owned(), serde_json::json!("Existing reconnaissance baseline.png, latest.png and INCOMPLETE.json remain in this private directory; no capture is requested for failure reporting"));
    }
    send_log(
        events,
        format!(
            "Private reconnaissance diagnostics: {:?}. Native PNGs are retained without resampling.",
            diagnostics.directory
        ),
    );
    diagnostics.write_start(vision.profile_id(), &request.profile)?;
    let mut ledger = InputLedger::default();
    let mut captures = 0usize;
    let mut publication = None;
    let mut failure_progress = None;
    let mut receipt_vision = None;
    let result = (|| {
        control.check_active(context, cancel)?;
        send_state(events, WorkerState::Connecting);
        let mut qmp = QmpClient::connect(&context.socket_path)
            .map_err(|error| format!("Reconnaissance QMP connection failed: {error}"))?;
        control.check_active(context, cancel)?;
        let mut io = QmpReconIo {
            qmp: &mut qmp,
            context,
            control,
            cancel,
            events,
            vision,
            diagnostics: &mut diagnostics,
            output: &output,
            ledger: &mut ledger,
            captures: &mut captures,
            publication: &mut publication,
            starting_guard,
            latest_stock: None,
            pending_stock: None,
        };
        io.probe()?;


        match pyramid_recon::run(&mut io, request) {
            Ok(deal) => {
                receipt_vision = Some(io.vision);
                Ok(deal)
            }
            Err(failure) => {
                failure_progress = Some(failure.progress);
                Err(failure.message)
            }
        }
    })();
    // Preserve final-frame decode failures until diagnostics finish. A qualified
    // same-session challenge is carried through restoration, never reclassified.
    let receipt_metadata = if result.is_ok() {
        diagnostics.restored_metadata(starting_guard.and_then(|guard| guard.challenge))
    } else {
        Ok((
            None,
            None,
            crate::pyramid_workflow_model::ObservedGame::Unknown,
        ))
    };
    let evidence = diagnostics.finish(
        &result,
        failure_progress.as_ref(),
        &ledger,
        captures,
        publication,
    );
    let publication_label = publication_label(publication);
    if let Some(object) = details.as_object_mut() {
        object.insert("captures_attempted".to_owned(), serde_json::json!(captures));
        object.insert(
            "draws_attempted".to_owned(),
            serde_json::json!(ledger.draws_attempted),
        );
        object.insert(
            "draws_acknowledged".to_owned(),
            serde_json::json!(ledger.draws_acknowledged),
        );
        object.insert(
            "undo_all_attempted".to_owned(),
            serde_json::json!(ledger.undo_attempted),
        );
        object.insert(
            "undo_all_acknowledged".to_owned(),
            serde_json::json!(ledger.undo_acknowledged),
        );
        object.insert(
            "confirmation_attempted".to_owned(),
            serde_json::json!(ledger.confirm_attempted),
        );
        object.insert(
            "confirmation_acknowledged".to_owned(),
            serde_json::json!(ledger.confirm_acknowledged),
        );
        object.insert(
            "publication_status".to_owned(),
            serde_json::json!(publication_label),
        );
    }


    match (result, evidence) {
        (Ok(deal), Ok(())) if publication == Some(PublicationStatus::Published) => {
            let vision = receipt_vision.ok_or("Missing successful acquisition calibration")?;
            let (progress, default_method, observed) = receipt_metadata?;
            super::computed_route::record_acquisition(
                context,
                events,
                cancel,
                deal,
                vision,
                &output,
                progress,
                default_method,
                observed,
            )?;
            Ok(output)
        }
        (Ok(_), Ok(())) => Err(format!(
            "Controller returned without publishing a deal; diagnostics at {:?}",
            diagnostics.directory
        )),
        (Err(error), Ok(())) => Err(format!(
            "{error}; diagnostics at {:?}; complete output publication={publication_label}. Inspect the output before any new operation",
            diagnostics.directory
        )),
        (result, Err(error)) => {
            let original = result
                .err()
                .unwrap_or_else(|| "Acquisition and restoration succeeded".to_owned());
            Err(format!(
                "{original}; diagnostic persistence failed: {error}; directory {:?}; complete output publication={publication_label}. Inspect the output before any new operation",
                diagnostics.directory
            ))
        }
    }
}


/// Independent backend caps and single-use probe authority, not game-effect evidence.
#[derive(Default)]
struct InputLedger {
    /// At most one following input may consume the last validated VM/pointer probe.
    probed: bool,
    /// Uncertain delivery permanently revokes this adapter's input capability.
    delivery_failed: bool,
    /// Draw delivery attempts, including an uncertain final attempt.
    draws_attempted: usize,
    /// Draw operations acknowledged by QMP; not proof of guest consumption.
    draws_acknowledged: usize,
    /// Undo All attempts; never retried after uncertain delivery.
    undo_attempted: usize,
    /// Acknowledged Undo All delivery, not visual proof that the dialog appeared.
    undo_acknowledged: bool,
    /// Confirmation attempts; never retried after uncertain delivery.
    confirm_attempted: usize,
    /// Acknowledged confirmation delivery, not visual proof of restoration.
    confirm_acknowledged: bool,
}


/// Consume a probe and one bounded operation reservation before attempting QMP delivery.
fn deliver_guarded_input(
    control: &ReconControl,
    context: &ControllerContext,
    cancel: &AtomicBool,
    ledger: &mut InputLedger,
    input: ReconInput,
    deliver: impl FnOnce(ReconInput) -> Result<(), String>,
) -> Result<(), String> {
    control.check_active(context, cancel)?;


    if ledger.delivery_failed {
        return Err(
            "Earlier reconnaissance input delivery was uncertain; no further input is permitted"
                .to_owned(),
        );
    }


    if !std::mem::take(&mut ledger.probed) {
        return Err(
            "Reconnaissance input requires a new validated VM and pointer probe".to_owned(),
        );
    }
    let ordered = match input {
        ReconInput::Draw => ledger.undo_attempted == 0 && ledger.confirm_attempted == 0,
        ReconInput::UndoAll(_) => {
            ledger.draws_acknowledged == 24
                && ledger.undo_attempted == 0
                && ledger.confirm_attempted == 0
        }
        ReconInput::Confirm(_) => ledger.undo_acknowledged && ledger.confirm_attempted == 0,
    };


    if !ordered {
        return Err("Reconnaissance input is out of acquisition/restoration order".to_owned());
    }
    let (count, maximum) = match input {
        ReconInput::Draw => (&mut ledger.draws_attempted, 24),
        ReconInput::UndoAll(_) => (&mut ledger.undo_attempted, 1),
        ReconInput::Confirm(_) => (&mut ledger.confirm_attempted, 1),
    };


    if *count >= maximum {
        return Err("Reconnaissance input budget exhausted; no retry is authorised".to_owned());
    }
    *count += 1;
    control.check_active(context, cancel)?;
    ledger.delivery_failed = true;
    deliver(input)?;
    ledger.delivery_failed = false;


    match input {
        ReconInput::Draw => ledger.draws_acknowledged += 1,
        ReconInput::UndoAll(_) => ledger.undo_acknowledged = true,
        ReconInput::Confirm(_) => ledger.confirm_acknowledged = true,
    }
    control.check_active(context, cancel)
}


/// Worker-thread-only adapter; immutable context and loaded vision survive the whole scan.
struct QmpReconIo<'a> {
    /// Existing application transport, with crate-owned release-only recovery.
    qmp: &'a mut QmpClient,
    /// Socket, mode, strategy and generation approved for this scan.
    context: &'a ControllerContext,
    /// Immediate request-side context and active-run latch.
    control: &'a ReconControl,
    /// Cooperative STOP flag checked at every operation boundary.
    cancel: &'a AtomicBool,
    /// Coalesced diagnostic preview and ordered status notifications.
    events: &'a WorkerEventSink,
    /// Native profile read and checked once before connection or input.
    vision: ReconVision,
    /// Private original-byte evidence for success or failure.
    diagnostics: &'a mut Diagnostics,
    /// Resolved no-clobber publication destination.
    output: &'a Path,
    /// Attempt/acknowledgement accounting and fresh probe reservation.
    ledger: &'a mut InputLedger,
    /// Capture attempts, including a failing final acquisition.
    captures: &'a mut usize,
    /// None means unattempted; errors may still acknowledge or leave publication uncertain.
    publication: &'a mut Option<PublicationStatus>,
    /// Optional borrowed same-session constraint; this never creates input authority.
    starting_guard: Option<StartingGuard<'a>>,
    /// Positively recognised original pile identities on the latest existing capture.
    latest_stock: Option<[StockFace; 2]>,
    /// Original first-D identities, retained across an intentional paired D batch.
    pending_stock: Option<[StockFace; 2]>,
}


impl QmpReconIo<'_> {
    /// Validate a native stock transition while retaining the original paired-D baseline.
    /// Stale same-arrangement frames remain available to the existing bounded observer.
    /// Specific confirmation and full-restoration proofs remain in the pure controller.
    fn record_stock_effect(
        &mut self,
        frame: &CapturedFrame,
        scene: &ReconScene,
    ) -> Result<(), String> {
        let ReconScene::Board(board) = scene else {
            return Ok(());
        };
        let current = [board.left, board.right];
        if let Some(original) = self.pending_stock
            && current != original
        {
            self.pending_stock = None;
        }
        self.latest_stock = Some(current);
        let _ = frame; // The existing rank/suit observation owns these identities.
        Ok(())
    }
}


impl ReconIo for QmpReconIo<'_> {


    /// Acknowledge suspension only when the runner has a complete stock position.
    fn pause_checkpoint(&mut self) -> Result<Duration, String> {
        let paused = self
            .events
            .latest_frame
            .pause
            .checkpoint(super::pyramid_pause::Phase::Recon, || {
                self.control.check_active(self.context, self.cancel)
            })?;
        if !paused.is_zero() {
            self.ledger.probed = false;
            self.latest_stock = None;
            self.pending_stock = None;
        }
        Ok(paused)
    }


    /// Check the queued request identity rather than relying on dispatch-time selection.
    fn check_active(&self) -> Result<(), String> {
        self.control.check_active(self.context, self.cancel)
    }


    /// Capture and publish diagnostic pixels even if STOP or recognition fails afterwards.
    fn observe(&mut self) -> Result<ReconScene, String> {
        self.check_active()?;
        self.ledger.probed = false;


        if *self.captures >= 180 {
            return Err("Reconnaissance capture budget exhausted".to_owned());
        }
        *self.captures += 1;
        send_state(self.events, WorkerState::Capturing);
        let (frame, png, _) =
            capture_screen_with_png(self.qmp, &self.context.socket_path, self.events)?;
        // Only an already acknowledged OK permits the measured pressed-button
        // appearance to be recognised as a known stale modal. The pure controller
        // then waits for restoration; this does not grant another OK attempt.
        let recognition = if self.ledger.confirm_acknowledged {
            self.vision.observe_after_confirmation(&frame)
        } else {
            self.vision.observe(&frame)
        };
        if let Some(guard) = self.starting_guard {
            // Retain original PNG evidence before any guard can refuse this frame.
            // The result is held until the same decoded frame reaches the preview.
            let guarded = (|| {
                self.diagnostics.record_capture(png)?;
                self.check_active()?;
                let scene = recognition?;
                guard.validate(&frame, &scene, self.ledger.draws_attempted)?;
                self.record_stock_effect(&frame,&scene)?;
                Ok(scene)
            })();
            self.events.publish_diagnostic_frame(frame);
            return guarded;
        }
        let recognition = recognition.and_then(|scene| {
            self.record_stock_effect(&frame, &scene)?;
            Ok(scene)
        });
        self.events.publish_diagnostic_frame(frame);
        self.diagnostics.record_capture(png)?;
        self.check_active()?;
        recognition
    }


    /// Fresh VM and pointer validation grants one input opportunity only.
    fn probe(&mut self) -> Result<(), String> {
        self.check_active()?;
        self.ledger.probed = false;
        let probe = self
            .qmp
            .probe()
            .map_err(|error| format!("Reconnaissance QMP probe failed: {error}"))?;
        validate_probe(&probe)?;
        self.check_active()?;
        self.ledger.probed = true;
        Ok(())
    }


    /// Deliver only D, Undo All or confirmation, with a final post-probe STOP check.
    fn input(&mut self, input: ReconInput) -> Result<(), String> {
        match input {
            ReconInput::Draw if self.pending_stock.is_none() => {
                let original = self.latest_stock.take().ok_or("Recon D has no checked source identities")?;
                if !matches!(original[0], StockFace::Card(_)) {
                    return Err("Recon D source LEFT is not identified; no input authorised".to_owned());
                }
                self.pending_stock = Some(original);
                send_log(self.events, "Pyramid Recon D batch: original source identities bound; two fresh disappearance checks required".to_owned());
            }
            ReconInput::Draw => {}
            ReconInput::UndoAll(_) | ReconInput::Confirm(_) => {
                self.latest_stock = None;
                self.pending_stock = None;
            }
        }
        let qmp = &mut self.qmp;
        let events = self.events;
        deliver_guarded_input(
            self.control,
            self.context,
            self.cancel,
            self.ledger,
            input,
            |input| {
                send_state(events, WorkerState::Acting);
                let result = match input {
                    ReconInput::Draw => qmp.press_draw_key(),
                    ReconInput::UndoAll(point) | ReconInput::Confirm(point) => {
                        qmp.click(point, NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT)
                    }
                };
                result.map_err(|error| format!("Uncertain reconnaissance input delivery: {error}; operation will not be replayed"))
            },
        )
    }


    /// Reuse cooperative sliced waiting; selection changes also latch STOP immediately.
    fn wait(&mut self, duration: Duration) -> Result<(), String> {
        self.check_active()?;
        self.ledger.probed = false;
        send_state(self.events, WorkerState::Verifying);
        cancellable_wait(duration, self.cancel).map_err(|_| {
            "STOP during reconnaissance settling; no cleanup input is permitted".to_owned()
        })?;
        self.check_active()
    }


    /// Forward bounded controller progress without introducing input authority.
    fn progress(&mut self, message: &str) {
        send_status(self.events, message.to_owned());
        send_log(self.events, message.to_owned());
    }


    /// Persist final original pixels, then publish through the crate's atomic no-clobber writer.
    fn publish(&mut self, deal: &CompleteDeal) -> Result<(), String> {
        self.check_active()?;


        if self.publication.is_some() {
            return Err(
                "Reconnaissance output was already published; publication is not retried"
                    .to_owned(),
            );
        }
        self.diagnostics.persist_latest()?;
        self.check_active()?;
        publish_once(self.publication, || {
            xsar::pyramid_json::write_new(self.output, deal)
        })
    }
}


/// Record publication boundaries even when cleanup or durability fails after linking.
fn publish_once(
    publication: &mut Option<PublicationStatus>,
    publish: impl FnOnce() -> Result<(), PublishError>,
) -> Result<(), String> {


    if publication.is_some() {
        return Err("Reconnaissance publication was already attempted; inspect the destination before any new operation".to_owned());
    }
    *publication = Some(PublicationStatus::NotPublished);


    match publish() {
        Ok(()) => {
            *publication = Some(PublicationStatus::Published);
            Ok(())
        }
        Err(error) => {
            *publication = Some(error.publication_status());
            Err(format!(
                "Restored deal publication returned an error: {error}; publication={}. Inspect the destination; no retry or replacement was attempted",
                publication_label(*publication)
            ))
        }
    }
}


/// Distinguish no attempt, acknowledged nonpublication, uncertainty and publication.
fn publication_label(publication: Option<PublicationStatus>) -> &'static str {


    match publication {
        None => "not_attempted",
        Some(PublicationStatus::NotPublished) => "not_published",
        Some(PublicationStatus::Uncertain) => "uncertain",
        Some(PublicationStatus::Published) => "published",
    }
}


/// Reject raw Unix spellings that the writer refuses, before normalisation or I/O.
fn validate_output_spelling(output: &Path) -> Result<(), String> {
    let bytes = output.as_os_str().as_bytes();
    let filename = bytes
        .rsplit(|byte| *byte == b'/')
        .next()
        .unwrap_or_default();


    if filename.is_empty() || filename == b"." || filename == b".." || bytes.contains(&0) {
        return Err(
            "Output requires a plain final filename without a trailing slash, dot component or NUL"
                .to_owned(),
        );
    }
    Ok(())
}


/// Resolve an existing parent and reject every existing target, including dangling symlinks.
///
/// This is a usability preflight, not race-proof publication. The crate writer
/// enforces no-clobber atomically when the restored deal is eventually published.
pub(super) fn resolve_new_output(output: &Path) -> Result<PathBuf, String> {
    validate_output_spelling(output)?;
    let filename = output
        .file_name()
        .ok_or_else(|| "Output requires a file name".to_owned())?;
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = fs::canonicalize(parent)
        .map_err(|error| format!("Output parent cannot be resolved: {error}"))?;


    if !parent.is_dir() {
        return Err("Output parent is not a directory".to_owned());
    }
    let resolved = parent.join(filename);


    match fs::symlink_metadata(&resolved) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(resolved),
        Err(error) => Err(format!("Output existence check failed: {error}")),
        Ok(_) => Err("Output already exists; reconnaissance refuses to overwrite it".to_owned()),
    }
}


/// Private fixed-name evidence directory; never removed automatically after failure.
struct Diagnostics {
    /// Unique private directory adjacent to the requested JSON output.
    directory: PathBuf,
    /// Whether baseline.png has already been written without changing its bytes.
    baseline_written: bool,
    /// Newest original capture only, bounded by the existing PNG cap.
    latest_png: Option<Vec<u8>>,
    /// A failed final write is not silently retried at the same evidence path.
    latest_attempted: bool,
    /// Retain the first final-image persistence error for every later diagnostic report.
    latest_error: Option<String>,
}


impl Diagnostics {


    /// Reserve a mode-0700 directory within a bounded collision budget before guest input.
    fn reserve(output: &Path) -> Result<Self, String> {
        let parent = output
            .parent()
            .ok_or_else(|| "Resolved output has no parent".to_owned())?;


        for _ in 0..DIRECTORY_ATTEMPTS {
            let sequence = DIAGNOSTIC_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let directory =
                parent.join(format!(".pyramid-recon-{}-{sequence}", std::process::id()));


            match fs::DirBuilder::new().mode(0o700).create(&directory) {
                Ok(()) => {
                    return Ok(Self {
                        directory,
                        baseline_written: false,
                        latest_png: None,
                        latest_attempted: false,
                        latest_error: None,
                    });
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!(
                        "Private diagnostic directory could not be created: {error}"
                    ));
                }
            }
        }
        Err("Private diagnostic directory collision budget exhausted".to_owned())
    }


    /// Record validated bundled or file provenance before input, escaping any path.
    ///
    /// The identifier and optional path have passed profile and request validation.
    /// Bundled calibration has a null path rather than an invented filesystem source.
    /// This remains an incomplete start marker, not recognition or restoration proof.
    fn write_start(&self, profile_id: &str, profile: &ReconProfileSource) -> Result<(), String> {
        let (source, path) = match profile {
            ReconProfileSource::BundledNativeV23 => ("bundled_native_v23", None),
            #[cfg(test)]
            ReconProfileSource::BundledNativeV22 => ("bundled_native_v22", None),
            #[cfg(test)]
            ReconProfileSource::BundledNativeV17 => ("bundled_native_v17", None),
            ReconProfileSource::File(path) => ("file", Some(format!("{path:?}"))),
        };
        self.write_json(
            "INCOMPLETE-start.json",
            &serde_json::json!({
                "diagnostic_only": true,
                "status": "INCOMPLETE",
                "complete": false,
                "restored": false,
                "note": "Start marker only; not route input or proof of the final outcome",
                "profile_id": profile_id,
                "profile_source": source,
                "profile_path_debug": path,
                "captures": 0,
                "draws_acknowledged": 0,
            }),
        )
    }


    /// Persist the first original and retain only the newest original in memory.
    fn record_capture(&mut self, png: Vec<u8>) -> Result<(), String> {


        if png.is_empty() || png.len() > SNAPSHOT_MAX_PNG_BYTES {
            return Err(
                "Diagnostic original PNG is outside the existing capture size limit".to_owned(),
            );
        }
        self.latest_png = Some(png);


        if !self.baseline_written {
            write_private_new(
                &self.directory.join("baseline.png"),
                self.latest_png.as_deref().unwrap_or_default(),
            )?;
            self.baseline_written = true;
        }
        Ok(())
    }


    /// Decode the exact retained restored PNG before releasing its bounded buffer.
    ///
    /// A qualified same-session challenge keeps its kind, target, safe counter and
    /// Kings default; action-frame label pixels supply none of those facts. Unbound
    /// manual acquisition retains the existing strict restored-frame classification.
    /// Missing or invalid retained pixels refuse the receipt. No new I/O occurs.
    fn restored_metadata(
        &self,
        challenge: Option<LockedChallenge>,
    ) -> Result<
        (
            Option<crate::pyramid_progress::ChallengeProgress>,
            Option<crate::pyramid_workflow_model::RouteMethod>,
            crate::pyramid_workflow_model::ObservedGame,
        ),
        String,
    > {
        use crate::pyramid_workflow_model::ObservedGame;
        let png = self
            .latest_png
            .as_deref()
            .ok_or("Missing restored acquisition frame")?;
        let frame = crate::capture::decode_png(png).map_err(|error| error.to_string())?;
        if let Some(locked) = challenge {
            let observed = ObservedGame::Challenge {
                progress: locked.progress,
                kings: locked.kings,
            };
            return Ok((
                Some(locked.progress),
                observed.route_method_override(),
                observed,
            ));
        }
        let progress = crate::pyramid_progress::observe_challenge(&frame)?;
        let observed = ObservedGame::observe(&frame)?;
        Ok((progress, observed.route_method_override(), observed))
    }


    /// Persist the final original once without consuming pixels needed by receipt binding.
    ///
    /// `finish` releases the one bounded buffer. Failed writes retain their original
    /// error and are never retried at this evidence path.
    fn persist_latest(&mut self) -> Result<(), String> {


        if self.latest_attempted {
            return self.latest_error.clone().map_or(Ok(()), Err);
        }
        self.latest_attempted = true;


        if let Some(png) = self.latest_png.as_deref()
            && let Err(error) = write_private_new(&self.directory.join("latest.png"), png)
        {
            self.latest_error = Some(error.clone());
            return Err(error);
        }
        Ok(())
    }


    /// Retain the observed partial inventory separately from the complete-route schema.
    fn finish(
        &mut self,
        result: &Result<CompleteDeal, String>,
        progress: Option<&ReconProgress>,
        ledger: &InputLedger,
        captures: usize,
        publication: Option<PublicationStatus>,
    ) -> Result<(), String> {
        let image_result = self.persist_latest();
        self.latest_png = None;
        let tableau: Vec<String> = match result {
            Ok(deal) => deal.tableau().iter().map(ToString::to_string).collect(),
            Err(_) => progress
                .and_then(|progress| progress.tableau.as_ref())
                .into_iter()
                .flatten()
                .take(28)
                .map(ToString::to_string)
                .collect(),
        };
        let stock: Vec<String> = match result {
            Ok(deal) => deal.stock().iter().map(ToString::to_string).collect(),
            Err(_) => progress
                .into_iter()
                .flat_map(|progress| progress.stock.iter())
                .take(24)
                .map(ToString::to_string)
                .collect(),
        };
        let complete = result.is_ok() && publication == Some(PublicationStatus::Published);
        let published = match publication {
            Some(PublicationStatus::Uncertain) => None,
            Some(PublicationStatus::Published) => Some(true),
            _ => Some(false),
        };
        let restored = result.is_ok() || progress.is_some_and(|progress| progress.restored);
        let error = result
            .as_ref()
            .err()
            .map(|message| message.chars().take(ERROR_CHARACTERS).collect::<String>());
        let report = serde_json::json!({
            "diagnostic_only": true,
            "status": if complete { "COMPLETE_DIAGNOSTIC" } else { "INCOMPLETE" },
            "complete": complete,
            "restored": restored,
            "complete_output_published": published,
            "publication_status": publication_label(publication),
            "error": error,
            "tableau_count": tableau.len(),
            "tableau_top_to_bottom": tableau,
            "stock_count": stock.len(),
            "stock_first_draw_order": stock,
            "captures_attempted": captures,
            "core_captures": progress.map(|progress| progress.captures),
            "core_draws": progress.map(|progress| progress.draws),
            "draws_attempted": ledger.draws_attempted,
            "draws_acknowledged": ledger.draws_acknowledged,
            "input_delivery_uncertain": ledger.delivery_failed,
            "undo_all_attempted": ledger.undo_attempted,
            "undo_all_acknowledged": ledger.undo_acknowledged,
            "confirm_attempted": ledger.confirm_attempted,
            "confirm_acknowledged": ledger.confirm_acknowledged,
            "baseline_png": if self.baseline_written { Some("baseline.png") } else { None },
            "latest_png_note": "latest.png, when present, contains the last original capture; inspect persistence errors",
            "authority_note": "Diagnostics are not planning input; acknowledgements do not prove guest effects",
        });
        let json_result = self.write_json(
            if complete {
                "COMPLETE-diagnostics.json"
            } else {
                "INCOMPLETE.json"
            },
            &report,
        );


        match (image_result, json_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(image), Ok(())) => Err(image),
            (Ok(()), Err(json)) => Err(json),
            (Err(image), Err(json)) => Err(format!("{image}; {json}")),
        }
    }


    /// Write one bounded diagnostic document to a new private file, never to the output path.
    fn write_json(&self, filename: &str, report: &serde_json::Value) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(report)
            .map_err(|error| format!("Diagnostic encoding failed: {error}"))?;


        if bytes.len() > DIAGNOSTIC_BYTES {
            return Err("Diagnostic JSON exceeds its fixed byte limit".to_owned());
        }
        write_private_new(&self.directory.join(filename), &bytes)
    }
}


/// Create an exclusive mode-0600 evidence file and surface write or flush failures.
///
/// Failed partial diagnostics are intentionally retained; unlike the planning
/// document, these are explicitly diagnostic files, never accepted route input.
fn write_private_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| format!("Diagnostic file creation failed at {path:?}: {error}"))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("Diagnostic file persistence failed at {path:?}: {error}"))
}


#[cfg(test)]
mod starting_guard_tests;


#[cfg(test)]
mod tests {
    //! No live guest input: request queues, cancellation, bounded evidence and mock delivery.

    /// Actual Card counters choose Maximum regardless of rank; a missing counter supplies no override.
    #[test]
    fn challenge_default_requires_the_same_restored_card_counter() {
        use crate::pyramid_workflow_model::RouteMethod;
        let mut diagnostics = super::Diagnostics {
            directory: std::path::PathBuf::new(),
            baseline_written: false,
            latest_png: None,
            latest_attempted: false,
            latest_error: None,
        };
        assert!(
            diagnostics
                .restored_metadata(None)
                .map(|(_, method, _)| method)
                .is_err()
        );
        for png in [
            include_bytes!("../../tests/fixtures/pyramid-kings-default/kings-one.png").as_slice(),
            include_bytes!("../../tests/fixtures/pyramid-kings-default/kings-two.png").as_slice(),
        ] {
            diagnostics.latest_png = Some(png.to_vec());
            assert_eq!(
                diagnostics
                    .restored_metadata(None)
                    .map(|(_, method, _)| method),
                Ok(Some(RouteMethod::Fast))
            );
            let mut frame = crate::capture::decode_png(png).unwrap();
            for y in 48..80 {
                for x in 1038..1110 {
                    let offset = y * frame.stride + x * 4;
                    frame.pixels[offset..offset + 3].fill(255);
                }
            }
            diagnostics.latest_png = Some(crate::capture::encode_png(&frame).unwrap());
            assert_eq!(
                diagnostics
                    .restored_metadata(None)
                    .map(|(_, method, _)| method),
                Ok(None)
            );
        }
        diagnostics.latest_png = Some(
            include_bytes!("../../tests/fixtures/pyramid-objectives/fours-four-of-twelve.png")
                .to_vec(),
        );
        assert_eq!(
            diagnostics
                .restored_metadata(None)
                .map(|(_, method, _)| method),
            Ok(Some(RouteMethod::Fast))
        );
    }


    /// Real Score and Board headers reach restored advice without changing recognition.
    #[test]
    fn recon_route_objective_uses_restored_score_and_board_headers() {
        use crate::pyramid_workflow_model::RouteMethod;
        for (png, expected) in [
            (
                include_bytes!(
                    "../../tests/fixtures/pyramid-execution-observability/score-final.png"
                )
                .as_slice(),
                RouteMethod::Fast,
            ),
            (
                include_bytes!(
                    "../../tests/fixtures/pyramid-execution-observability/boards-two.png"
                )
                .as_slice(),
                RouteMethod::FewestActions,
            ),
        ] {
            let diagnostics = super::Diagnostics {
                directory: std::path::PathBuf::new(),
                baseline_written: false,
                latest_png: Some(png.to_vec()),
                latest_attempted: false,
                latest_error: None,
            };
            assert_eq!(
                diagnostics.restored_metadata(None).unwrap().1,
                Some(expected)
            );
        }
    }


    use std::{
        cell::Cell,
        ffi::OsString,
        os::unix::{
            ffi::OsStringExt,
            fs::{PermissionsExt, symlink},
        },
        sync::mpsc,
    };

    use xsar::games::{
        cards::{Card, Rank, Suit},
        pyramid::{DealMetadata, ProvenanceKind, Restoration},
    };

    use super::*;
    use crate::worker::{LatestFrameSlot, WorkerCommand, WorkerHandle};
    use crate::{
        geometry::PixelPoint,
        parameters::{AnimationSettleDelays, StepRunSettings},
        tracker::PredictedAction,
    };


    /// Build the only strategy/game combination allowed to request reconnaissance.
    fn context() -> ControllerContext {
        ControllerContext::new(
            PathBuf::from("/not-a-live-qmp/recon.sock"),
            GameMode::Pyramid,
            SolvingStrategy::ShortestPath,
            17,
        )
    }


    /// Valid request fields only; tests never load this fictitious profile as evidence.
    fn request() -> ReconRequest {
        ReconRequest {
            profile: ReconProfileSource::File(PathBuf::from("/not-a-live-profile/profile.json")),
            output_path: PathBuf::from("/not-an-output-parent/deal.json"),
            deal_id: "worker-test".to_owned(),
            initial_board_confirmed: true,
        }
    }


    /// Construct a handle without a worker thread, retaining every queued command for inspection.
    fn handle() -> (WorkerHandle, mpsc::Receiver<WorkerCommand>) {
        let (command_tx, command_rx) = mpsc::channel();
        let (_, event_rx) = mpsc::channel();
        let handle = WorkerHandle {
            command_tx,
            event_rx,
            latest_frame: LatestFrameSlot::default(),
            join: None,
            cancel_requested: Arc::new(AtomicBool::new(true)),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
        };
        (handle, command_rx)
    }


    /// Reserve an isolated active control token without creating a queue or connection.
    fn active_control() -> (ReconControl, ControllerContext, AtomicBool) {
        let control = ReconControl::default();
        let context = context();
        let cancel = AtomicBool::new(true);
        control.select(&context, &cancel).unwrap();
        control
            .queue_reconnaissance(&context, &cancel, || Ok(()))
            .unwrap();
        (control, context, cancel)
    }


    /// Create a private disposable test directory with collision-safe names.
    fn test_directory() -> PathBuf {


        for _ in 0..DIRECTORY_ATTEMPTS {
            let sequence = DIAGNOSTIC_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let directory = std::env::temp_dir().join(format!(
                "pyramid-recon-worker-test-{}-{sequence}",
                std::process::id()
            ));


            match fs::DirBuilder::new().mode(0o700).create(&directory) {
                Ok(()) => return directory,
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("test directory: {error}"),
            }
        }
        panic!("test directory collision budget exhausted");
    }


    /// Explicitly synthetic complete inventory used only for worker test assertions.
    fn synthetic_deal() -> CompleteDeal {
        let deck: [Card; 52] =
            std::array::from_fn(|index| Card::new(Rank::ALL[index % 13], Suit::ALL[index / 13]));
        let metadata = DealMetadata::try_new(
            "worker-diagnostics",
            ProvenanceKind::SyntheticFixture,
            Restoration::NotApplicable,
            "worker-test",
            "Synthetic test; not native recognition",
            None,
        )
        .unwrap();
        CompleteDeal::try_new(
            deck[..28].try_into().unwrap(),
            deck[28..].try_into().unwrap(),
            metadata,
        )
        .unwrap()
    }


    /// Requests require explicit board consent, the independent strategy and Pyramid.
    #[test]
    fn request_gate_preserves_stop_and_normal_shortest_path_rejection() {
        let (handle, commands) = handle();
        let mut unconfirmed = request();
        unconfirmed.initial_board_confirmed = false;
        assert!(handle.reconnoitre(context(), unconfirmed).is_err());


        for mode in GameMode::AVAILABLE {
            let mut selection = context();
            selection.game_mode = mode;
            selection.strategy = SolvingStrategy::ComputerVision;
            assert!(handle.reconnoitre(selection, request()).is_err());
        }
        assert!(
            handle
                .execute_steps(
                    context(),
                    PredictedAction::NoHighlight,
                    StepRunSettings::new(AnimationSettleDelays::default(), 1)
                )
                .is_err()
        );
        assert!(!SolvingStrategy::ShortestPath.permits_input());
        assert!(handle.cancel_requested.load(Ordering::Acquire));
        assert!(matches!(
            commands.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
    }


    /// Selecting bundled calibration never supplies consent or clears STOP by itself.
    #[test]
    fn bundled_profile_request_keeps_consent_and_queue_authority_separate() {
        let (handle, commands) = handle();
        let mut bundled = request();
        bundled.profile = ReconProfileSource::BundledNativeV17;
        bundled.initial_board_confirmed = false;
        assert!(handle.reconnoitre(context(), bundled.clone()).is_err());
        assert!(handle.cancel_requested.load(Ordering::Acquire));
        assert!(matches!(
            commands.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
        bundled.initial_board_confirmed = true;
        assert!(validate_request(&context(), &bundled).is_ok());
        handle.reconnoitre(context(), bundled).unwrap();
        assert!(
            matches!(commands.try_recv().unwrap(), WorkerCommand::Reconnoitre { request, .. }
            if request.profile == ReconProfileSource::BundledNativeV17)
        );
        assert!(handle.reconnoitre(context(), request()).is_err());
        assert!(matches!(
            commands.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
    }


    /// Shared loading honours an explicit older profile and never substitutes bundled calibration on error.
    #[test]
    fn shared_profile_load_preserves_requested_source_without_fallback() {
        let mut requested = request();
        requested.profile = ReconProfileSource::BundledNativeV17;
        assert_eq!(
            load_vision(&requested).unwrap().profile_id(),
            "pyramid-native-1920-classic-blue-v17"
        );
        requested.profile = ReconProfileSource::File(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-recon/native-v8/profile.json"),
        );
        assert_eq!(
            load_vision(&requested).unwrap().profile_id(),
            "pyramid-native-1920-classic-blue-v8"
        );
        requested.profile = ReconProfileSource::File(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-recon/native-v9/INCOMPLETE.json"),
        );
        assert!(load_vision(&requested).is_err());
    }


    /// The distinct command is queued once and immediately owns the active-run latch.
    #[test]
    fn one_reconnaissance_reservation_prevents_duplicate_queueing() {
        let (handle, commands) = handle();
        handle.reconnoitre(context(), request()).unwrap();
        assert!(!handle.cancel_requested.load(Ordering::Acquire));
        assert!(
            matches!(commands.try_recv().unwrap(), WorkerCommand::Reconnoitre { context: selected, .. } if selected == context())
        );
        assert!(handle.reconnoitre(context(), request()).is_err());
        assert!(matches!(
            commands.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
    }


    /// STOP cannot be cleared by another scan or a normal CV execution request.
    #[test]
    fn stop_survives_new_reconnaissance_and_cv_requests() {
        let (handle, commands) = handle();
        handle.reconnoitre(context(), request()).unwrap();
        commands.try_recv().unwrap();
        handle.disconnect(context()).unwrap();
        assert!(matches!(
            commands.try_recv().unwrap(),
            WorkerCommand::Disconnect(_)
        ));
        assert!(handle.reconnoitre(context(), request()).is_err());
        let mut vision = context();
        vision.strategy = SolvingStrategy::ComputerVision;
        assert!(
            handle
                .execute_steps(
                    vision,
                    PredictedAction::NoHighlight,
                    StepRunSettings::new(AnimationSettleDelays::default(), 1)
                )
                .is_err()
        );
        assert!(handle.cancel_requested.load(Ordering::Acquire));
        assert!(
            handle
                .latest_frame
                .reconnaissance
                .check_active(&context(), &handle.cancel_requested)
                .is_err()
        );
        assert!(matches!(
            commands.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
    }


    /// Every identity component revokes before command dispatch, even after switching back.
    #[test]
    fn context_revocation_is_immediate_and_monotonic() {


        for component in 0..4 {
            let (handle, commands) = handle();
            handle.reconnoitre(context(), request()).unwrap();
            commands.try_recv().unwrap();
            let mut changed = context();


            match component {
                0 => changed.socket_path = PathBuf::from("/another-socket"),
                1 => changed.game_mode = GameMode::Spider,
                2 => changed.strategy = SolvingStrategy::ComputerVision,
                3 => changed.generation += 1,
                _ => unreachable!(),
            }
            let _ = handle.capture_frame(changed);
            assert!(handle.cancel_requested.load(Ordering::Acquire));
            handle.capture_frame(context()).unwrap();
            handle.cancel_requested.store(false, Ordering::Release);
            assert!(
                handle
                    .latest_frame
                    .reconnaissance
                    .check_active(&context(), &handle.cancel_requested)
                    .is_err()
            );
        }
    }


    /// Snapshot and reset selections also update the token without waiting for dispatch.
    #[test]
    fn snapshot_and_reset_revoke_a_running_scan_immediately() {


        for snapshot in [false, true] {
            let (handle, _) = handle();
            let control = &handle.latest_frame.reconnaissance;
            control
                .select(&context(), &handle.cancel_requested)
                .unwrap();
            control
                .queue_reconnaissance(&context(), &handle.cancel_requested, || Ok(()))
                .unwrap();
            let mut changed = context();
            changed.generation += 1;


            if snapshot {
                let _ = handle.prepare_snapshot(changed, 10);
            } else {
                let _ = handle.reset_progress(changed);
            }
            assert!(handle.cancel_requested.load(Ordering::Acquire));
            assert!(
                control
                    .check_active(&context(), &handle.cancel_requested)
                    .is_err()
            );
        }
    }


    /// Closing the command channel cannot leave a phantom reservation or clear STOP.
    #[test]
    fn closed_queue_releases_reservation_and_requests_stop() {
        let (handle, commands) = handle();
        drop(commands);
        assert!(handle.reconnoitre(context(), request()).is_err());
        assert!(handle.cancel_requested.load(Ordering::Acquire));
        assert!(
            !handle
                .latest_frame
                .reconnaissance
                .state
                .lock()
                .unwrap()
                .active
        );
    }


    /// A finished scan releases its reservation, but no request implicitly resumes it.
    #[test]
    fn finished_scan_requires_a_new_explicit_reservation() {
        let (control, context, cancel) = active_control();
        control.stop(Some(&context), &cancel);
        control.finish();
        assert!(control.check_active(&context, &cancel).is_err());
        control
            .queue_reconnaissance(&context, &cancel, || Ok(()))
            .unwrap();
        assert!(control.check_active(&context, &cancel).is_ok());
    }


    /// Unchanged-context advisory captures do not revoke or broaden scan authority.
    #[test]
    fn same_context_read_only_request_keeps_the_token() {
        let (handle, commands) = handle();
        handle.reconnoitre(context(), request()).unwrap();
        commands.try_recv().unwrap();
        handle.capture_frame(context()).unwrap();
        assert!(
            handle
                .latest_frame
                .reconnaissance
                .check_active(&context(), &handle.cancel_requested)
                .is_ok()
        );
        assert!(matches!(
            commands.try_recv().unwrap(),
            WorkerCommand::CaptureFrame { .. }
        ));
    }


    /// Mock delivery cannot occur without a fresh probe or after post-probe STOP.
    #[test]
    fn fresh_probe_and_final_stop_guard_every_delivery() {
        let (control, context, cancel) = active_control();
        let delivered = Cell::new(0);
        let mut ledger = InputLedger::default();
        let delivery = |_: ReconInput| {
            delivered.set(delivered.get() + 1);
            Ok(())
        };
        assert!(
            deliver_guarded_input(
                &control,
                &context,
                &cancel,
                &mut ledger,
                ReconInput::Draw,
                delivery
            )
            .is_err()
        );
        ledger.probed = true;
        control.stop(Some(&context), &cancel);
        assert!(
            deliver_guarded_input(
                &control,
                &context,
                &cancel,
                &mut ledger,
                ReconInput::Draw,
                delivery
            )
            .is_err()
        );
        assert_eq!(delivered.get(), 0);
        assert_eq!(ledger.draws_attempted, 0);
    }


    /// Input accounting distinguishes attempts, acknowledgements and later STOP.
    #[test]
    fn acknowledged_input_is_recorded_before_a_late_stop() {
        let (control, context, cancel) = active_control();
        let mut ledger = InputLedger {
            probed: true,
            ..InputLedger::default()
        };
        assert!(
            deliver_guarded_input(
                &control,
                &context,
                &cancel,
                &mut ledger,
                ReconInput::Draw,
                |_| {
                    control.stop(Some(&context), &cancel);
                    Ok(())
                }
            )
            .is_err()
        );
        assert_eq!(ledger.draws_attempted, 1);
        assert_eq!(ledger.draws_acknowledged, 1);
        assert!(!ledger.probed);
        assert!(!ledger.delivery_failed);
    }


    /// Uncertain delivery consumes the operation and permanently refuses later inputs.
    #[test]
    fn uncertain_draw_is_never_replayed_or_followed_by_restoration() {
        let (control, context, cancel) = active_control();
        let mut ledger = InputLedger {
            probed: true,
            ..InputLedger::default()
        };
        assert!(
            deliver_guarded_input(
                &control,
                &context,
                &cancel,
                &mut ledger,
                ReconInput::Draw,
                |_| Err("uncertain".to_owned())
            )
            .is_err()
        );


        for operation in [
            ReconInput::Draw,
            ReconInput::UndoAll(PixelPoint::new(1320, 978)),
            ReconInput::Confirm(PixelPoint::new(900, 600)),
        ] {
            ledger.probed = true;
            assert!(
                deliver_guarded_input(
                    &control,
                    &context,
                    &cancel,
                    &mut ledger,
                    operation,
                    |_| panic!("no retry or cleanup delivery")
                )
                .is_err()
            );
        }
        assert_eq!(ledger.draws_attempted, 1);
        assert_eq!(ledger.draws_acknowledged, 0);
        assert_eq!(ledger.undo_attempted, 0);
        assert_eq!(ledger.confirm_attempted, 0);
    }


    /// Independent backend maxima prevent an accidental core loop granting extra input.
    #[test]
    fn backend_caps_each_whitelisted_input_and_consumes_probes_once() {
        let (control, context, cancel) = active_control();
        let mut ledger = InputLedger::default();
        let delivered = Cell::new(0);
        let delivery = |_: ReconInput| {
            delivered.set(delivered.get() + 1);
            Ok(())
        };


        for (operation, maximum) in [
            (ReconInput::Draw, 24),
            (ReconInput::UndoAll(PixelPoint::new(1320, 978)), 1),
            (ReconInput::Confirm(PixelPoint::new(900, 600)), 1),
        ] {
            for _ in 0..maximum {
                ledger.probed = true;
                deliver_guarded_input(
                    &control,
                    &context,
                    &cancel,
                    &mut ledger,
                    operation,
                    delivery,
                )
                .unwrap();
                assert!(
                    deliver_guarded_input(
                        &control,
                        &context,
                        &cancel,
                        &mut ledger,
                        operation,
                        delivery
                    )
                    .is_err()
                );
            }
            ledger.probed = true;
            assert!(
                deliver_guarded_input(
                    &control,
                    &context,
                    &cancel,
                    &mut ledger,
                    operation,
                    delivery
                )
                .is_err()
            );
        }
        assert_eq!(delivered.get(), 26);
        assert_eq!(ledger.draws_acknowledged, 24);
    }


    /// Restoration operations require delivery order as well as the core's visual evidence.
    #[test]
    fn backend_rejects_out_of_order_undo_and_confirmation() {
        let (control, context, cancel) = active_control();
        let mut ledger = InputLedger::default();
        let undo = ReconInput::UndoAll(PixelPoint::new(1320, 978));
        let confirm = ReconInput::Confirm(PixelPoint::new(900, 600));


        for draws in [0, 23] {
            ledger.draws_acknowledged = draws;


            for operation in [undo, confirm] {
                ledger.probed = true;
                assert!(
                    deliver_guarded_input(
                        &control,
                        &context,
                        &cancel,
                        &mut ledger,
                        operation,
                        |_| panic!("out-of-order input")
                    )
                    .is_err()
                );
            }
        }
        ledger.draws_acknowledged = 24;
        ledger.probed = true;
        assert!(
            deliver_guarded_input(
                &control,
                &context,
                &cancel,
                &mut ledger,
                confirm,
                |_| panic!("Undo All was not acknowledged")
            )
            .is_err()
        );
        ledger.probed = true;
        deliver_guarded_input(&control, &context, &cancel, &mut ledger, undo, |_| Ok(())).unwrap();
        assert!(ledger.undo_acknowledged);


        for operation in [ReconInput::Draw, undo] {
            ledger.probed = true;
            assert!(
                deliver_guarded_input(
                    &control,
                    &context,
                    &cancel,
                    &mut ledger,
                    operation,
                    |_| panic!("restoration cannot return to draw or repeat Undo All")
                )
                .is_err()
            );
        }
        ledger.probed = true;
        deliver_guarded_input(
            &control,
            &context,
            &cancel,
            &mut ledger,
            confirm,
            |_| Ok(()),
        )
        .unwrap();
        assert!(ledger.confirm_acknowledged);


        for operation in [ReconInput::Draw, undo, confirm] {
            ledger.probed = true;
            assert!(
                deliver_guarded_input(
                    &control,
                    &context,
                    &cancel,
                    &mut ledger,
                    operation,
                    |_| panic!("no input after confirmation")
                )
                .is_err()
            );
        }
    }


    /// Undo All uncertainty consumes its reservation and never grants confirmation authority.
    #[test]
    fn uncertain_undo_cannot_be_confirmed_or_replayed() {
        let (control, context, cancel) = active_control();
        let mut ledger = InputLedger {
            probed: true,
            draws_attempted: 24,
            draws_acknowledged: 24,
            ..InputLedger::default()
        };
        let undo = ReconInput::UndoAll(PixelPoint::new(1320, 978));
        assert!(
            deliver_guarded_input(&control, &context, &cancel, &mut ledger, undo, |_| Err(
                "uncertain Undo All".to_owned()
            ))
            .is_err()
        );
        assert_eq!(ledger.undo_attempted, 1);
        assert!(!ledger.undo_acknowledged);


        for operation in [undo, ReconInput::Confirm(PixelPoint::new(900, 600))] {
            ledger.probed = true;
            assert!(
                deliver_guarded_input(
                    &control,
                    &context,
                    &cancel,
                    &mut ledger,
                    operation,
                    |_| panic!("no retry after uncertain Undo All")
                )
                .is_err()
            );
        }
        assert_eq!(ledger.confirm_attempted, 0);
    }


    /// Publication errors retain the writer's boundary and never permit implicit retries.
    #[test]
    fn publication_errors_preserve_acknowledged_and_uncertain_destinations() {
        let cases = [
            (PublishError::InvalidPath, PublicationStatus::NotPublished),
            (
                PublishError::Io {
                    stage: xsar::pyramid_json::PublishStage::LinkDestination,
                    status: PublicationStatus::Uncertain,
                    source: std::io::Error::other("simulated link failure"),
                    cleanup: None,
                },
                PublicationStatus::Uncertain,
            ),
            (
                PublishError::Published {
                    cleanup: None,
                    directory_sync: Some(std::io::Error::other("simulated directory sync failure")),
                },
                PublicationStatus::Published,
            ),
        ];


        for (error, expected) in cases {
            let mut publication = None;
            let result = publish_once(&mut publication, || Err(error));
            assert!(
                result
                    .unwrap_err()
                    .contains(publication_label(Some(expected)))
            );
            assert_eq!(publication, Some(expected));
            assert!(
                publish_once(&mut publication, || panic!(
                    "publication must not be replayed"
                ))
                .is_err()
            );
        }
        let mut publication = None;
        publish_once(&mut publication, || Ok(())).unwrap();
        assert_eq!(publication, Some(PublicationStatus::Published));
        assert!(
            publish_once(&mut publication, || panic!(
                "successful publication must not be repeated"
            ))
            .is_err()
        );
    }


    /// The real schema2 publisher preserves worker one-attempt and no-overwrite boundaries.
    #[test]
    fn publication_once_writes_schema_two_without_overwriting_or_retrying() {
        let directory = test_directory();
        let output = directory.join("synthetic-schema2-deal.json");
        let original = synthetic_deal();
        let mut tableau = *original.tableau();
        let mut stock = *original.stock();
        tableau.rotate_left(9);
        stock.reverse();
        let deal = CompleteDeal::try_new(tableau, stock, original.metadata().clone()).unwrap();
        let mut publication = None;
        publish_once(&mut publication, || {
            xsar::pyramid_json::write_new(&output, &deal)
        })
        .unwrap();
        assert_eq!(publication, Some(PublicationStatus::Published));
        let original_bytes = fs::read(&output).unwrap();
        let document: serde_json::Value = serde_json::from_slice(&original_bytes).unwrap();
        assert_eq!(document["schema_version"], 2);
        assert_eq!(document["tableau"].as_array().unwrap().len(), 28);
        assert_eq!(document["stock"].as_array().unwrap().len(), 24);
        assert_eq!(
            xsar::pyramid_json::read_from(fs::File::open(&output).unwrap()).unwrap(),
            deal
        );
        assert!(
            publish_once(&mut publication, || panic!(
                "successful schema2 publication must not be repeated"
            ))
            .is_err()
        );
        assert_eq!(publication, Some(PublicationStatus::Published));
        let mut collision = None;
        assert!(
            publish_once(&mut collision, || xsar::pyramid_json::write_new(
                &output, &deal
            ))
            .is_err()
        );
        assert_eq!(collision, Some(PublicationStatus::Uncertain));
        assert!(
            publish_once(&mut collision, || panic!(
                "refused schema2 publication must not be retried"
            ))
            .is_err()
        );
        assert_eq!(collision, Some(PublicationStatus::Uncertain));
        assert_eq!(fs::read(&output).unwrap(), original_bytes);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_dir_all(directory).unwrap();
    }


    /// Uncertainty is represented as unknown, never as a false claim that no output exists.
    #[test]
    fn diagnostic_publication_status_is_not_collapsed_to_a_boolean() {
        for publication in [
            PublicationStatus::NotPublished,
            PublicationStatus::Uncertain,
            PublicationStatus::Published,
        ] {
            let directory = test_directory();
            let mut diagnostics = Diagnostics::reserve(&directory.join("deal.json")).unwrap();
            let progress = ReconProgress {
                restored: true,
                ..ReconProgress::default()
            };
            diagnostics
                .finish(
                    &Err("publication failed".to_owned()),
                    Some(&progress),
                    &InputLedger::default(),
                    30,
                    Some(publication),
                )
                .unwrap();
            let bytes = fs::read(diagnostics.directory.join("INCOMPLETE.json")).unwrap();
            let report: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(
                report["publication_status"],
                publication_label(Some(publication))
            );
            assert_eq!(report["restored"], true);
            assert_eq!(report["complete"], false);
            let published = match publication {
                PublicationStatus::NotPublished => serde_json::Value::Bool(false),
                PublicationStatus::Uncertain => serde_json::Value::Null,
                PublicationStatus::Published => serde_json::Value::Bool(true),
            };
            assert_eq!(report["complete_output_published"], published);
            fs::remove_dir_all(directory).unwrap();
        }
    }


    /// Resolve output parents without permitting existing files, directories or dangling links.
    #[test]
    fn output_preflight_rejects_every_existing_entry() {
        let directory = test_directory();
        let output = directory.join("new.json");
        assert_eq!(
            resolve_new_output(&output).unwrap(),
            fs::canonicalize(&directory).unwrap().join("new.json")
        );
        write_private_new(&output, b"keep").unwrap();
        assert!(resolve_new_output(&output).is_err());
        assert_eq!(fs::read(&output).unwrap(), b"keep");
        assert!(resolve_new_output(&directory).is_err());
        let dangling = directory.join("dangling.json");
        symlink(directory.join("absent"), &dangling).unwrap();
        assert!(resolve_new_output(&dangling).is_err());
        assert!(resolve_new_output(&directory.join("absent/child.json")).is_err());
        fs::remove_dir_all(directory).unwrap();
    }


    /// Raw trailing components and NUL fail before any path lookup or normalisation.
    #[test]
    fn output_preflight_rejects_raw_invalid_spellings_without_changing_paths() {
        let directory = test_directory();
        let existing = directory.join("keep.json");
        write_private_new(&existing, b"existing bytes remain unchanged").unwrap();
        let child = directory.join("keep-directory");
        fs::create_dir(&child).unwrap();
        let link = directory.join("keep-link");
        symlink(&existing, &link).unwrap();


        for base in [
            &existing,
            &child,
            &link,
            &directory.join("new.json"),
            &directory.join("missing-parent/new.json"),
        ] {
            for suffix in ["/", "//", "/.", "/..", "\0", "\0.json"] {
                let mut spelling = base.as_os_str().to_os_string();
                spelling.push(suffix);
                let error = resolve_new_output(Path::new(&spelling)).unwrap_err();
                assert!(error.starts_with("Output requires a plain final filename"));
            }
        }


        for spelling in [
            "",
            ".",
            "..",
            "/",
            "/.",
            "/..",
            "missing/.",
            "missing/..",
            "missing\0/new.json",
        ] {
            assert!(
                resolve_new_output(Path::new(spelling))
                    .unwrap_err()
                    .starts_with("Output requires a plain final filename")
            );
        }
        let non_unicode = directory.join(OsString::from_vec(b"valid-\xff.json".to_vec()));
        assert!(resolve_new_output(&non_unicode).is_ok());
        assert!(!non_unicode.exists());
        assert_eq!(
            fs::read(&existing).unwrap(),
            b"existing bytes remain unchanged"
        );
        assert!(child.is_dir());
        assert_eq!(fs::read_link(&link).unwrap(), existing);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 3);
        fs::remove_dir_all(directory).unwrap();
    }


    /// Malformed output is refused at queue and worker boundaries before profile I/O.
    #[test]
    fn invalid_raw_output_cannot_clear_stop_queue_work_or_load_a_profile() {
        let (handle, commands) = handle();
        let mut invalid = request();
        invalid.output_path = PathBuf::from("new.json/.");
        let error = handle.reconnoitre(context(), invalid.clone()).unwrap_err();
        assert!(error.starts_with("Output requires a plain final filename"));
        assert!(handle.cancel_requested.load(Ordering::Acquire));
        assert!(matches!(
            commands.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));

        let (control, context, cancel) = active_control();
        let (events, receiver) = mpsc::channel();
        let latest = LatestFrameSlot {
            reconnaissance: control,
            ..LatestFrameSlot::default()
        };
        let sink = WorkerEventSink {
            events,
            latest_frame: latest,
            capture_context: Mutex::new(Some(context.clone())),
        };
        run(&context, &invalid, &sink, &cancel);
        let notifications: Vec<_> = receiver.try_iter().collect();
        assert!(
            notifications
                .iter()
                .any(|notification| matches!(&notification.event,
            WorkerEvent::ReconnaissanceFinished { result: Err(error), .. }
                if error.starts_with("Output requires a plain final filename")))
        );
        assert!(!notifications.iter().any(|notification| matches!(
            notification.event,
            WorkerEvent::State(
                WorkerState::Connecting | WorkerState::Capturing | WorkerState::Acting
            )
        )));
        assert!(sink.latest_frame.take().is_none());
    }


    /// Original PNG buffers are copied byte-for-byte, with two files despite many captures.
    #[test]
    fn diagnostics_are_private_bounded_and_preserve_original_bytes() {
        let directory = test_directory();
        let mut diagnostics = Diagnostics::reserve(&directory.join("deal.json")).unwrap();
        assert_eq!(
            fs::metadata(&diagnostics.directory)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        diagnostics
            .write_start(
                "synthetic-worker-test",
                &ReconProfileSource::File(PathBuf::from("synthetic-profile.json")),
            )
            .unwrap();
        let first = b"synthetic original byte marker 0".to_vec();
        diagnostics.record_capture(first.clone()).unwrap();


        for index in 1..180 {
            diagnostics
                .record_capture(format!("synthetic original byte marker {index}").into_bytes())
                .unwrap();
        }
        diagnostics
            .finish(
                &Err("test stop".to_owned()),
                None,
                &InputLedger::default(),
                180,
                None,
            )
            .unwrap();
        assert_eq!(
            fs::read(diagnostics.directory.join("baseline.png")).unwrap(),
            first
        );
        assert_eq!(
            fs::read(diagnostics.directory.join("latest.png")).unwrap(),
            b"synthetic original byte marker 179"
        );
        assert!(diagnostics.latest_png.is_none());
        let files: Vec<_> = fs::read_dir(&diagnostics.directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(
            files
                .iter()
                .filter(|path| path.extension().is_some_and(|extension| extension == "png"))
                .count(),
            2
        );


        for file in files {
            assert_eq!(
                fs::metadata(file).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        fs::remove_dir_all(directory).unwrap();
    }


    /// Start provenance retains the loaded identifier and an escaped non-Unicode path.
    #[test]
    fn diagnostic_start_records_profile_identity_without_claiming_recognition() {
        let directory = test_directory();
        let diagnostics = Diagnostics::reserve(&directory.join("deal.json")).unwrap();
        let profile = PathBuf::from(OsString::from_vec(b"profile-\n-\xff.json".to_vec()));
        diagnostics
            .write_start(
                "synthetic-worker-test",
                &ReconProfileSource::File(profile.clone()),
            )
            .unwrap();
        let bytes = fs::read(diagnostics.directory.join("INCOMPLETE-start.json")).unwrap();
        let report: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(report["profile_id"], "synthetic-worker-test");
        assert_eq!(report["profile_source"], "file");
        assert_eq!(report["profile_path_debug"], format!("{profile:?}"));
        assert!(
            !report["profile_path_debug"]
                .as_str()
                .unwrap()
                .contains('\n')
        );
        assert_eq!(report["status"], "INCOMPLETE");
        assert_eq!(report["complete"], false);
        assert_eq!(report["restored"], false);
        assert_eq!(report["captures"], 0);
        assert_eq!(report["draws_acknowledged"], 0);
        assert!(xsar::pyramid_json::from_slice(&bytes).is_err());
        assert!(bytes.len() <= DIAGNOSTIC_BYTES);
        fs::remove_dir_all(directory).unwrap();
    }


    /// Embedded calibration records its source without inventing a runtime file path.
    #[test]
    fn diagnostic_start_records_bundled_profile_without_a_path() {
        let directory = test_directory();
        let diagnostics = Diagnostics::reserve(&directory.join("deal.json")).unwrap();
        diagnostics
            .write_start(
                "pyramid-native-1920-classic-blue-v17",
                &ReconProfileSource::BundledNativeV17,
            )
            .unwrap();
        let bytes = fs::read(diagnostics.directory.join("INCOMPLETE-start.json")).unwrap();
        let report: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(report["profile_id"], "pyramid-native-1920-classic-blue-v17");
        assert_eq!(report["profile_source"], "bundled_native_v17");
        assert!(report["profile_path_debug"].is_null());
        assert_eq!(report["status"], "INCOMPLETE");
        assert_eq!(report["complete"], false);
        assert_eq!(report["restored"], false);
        assert_eq!(report["captures"], 0);
        assert_eq!(report["draws_acknowledged"], 0);
        assert!(xsar::pyramid_json::from_slice(&bytes).is_err());
        assert!(bytes.len() <= DIAGNOSTIC_BYTES);
        fs::remove_dir_all(directory).unwrap();
    }


    /// Partial inventory and delivery counters cannot masquerade as complete route input.
    #[test]
    fn failed_inventory_is_explicitly_incomplete_and_not_a_deal_document() {
        let directory = test_directory();
        let mut diagnostics = Diagnostics::reserve(&directory.join("deal.json")).unwrap();
        let deal = synthetic_deal();
        let progress = ReconProgress {
            tableau: Some(*deal.tableau()),
            stock: deal.stock()[..3].to_vec(),
            captures: 8,
            draws: 3,
            restored: false,
        };
        let ledger = InputLedger {
            draws_attempted: 3,
            draws_acknowledged: 2,
            delivery_failed: true,
            ..InputLedger::default()
        };
        diagnostics
            .finish(
                &Err("uncertain D".to_owned()),
                Some(&progress),
                &ledger,
                8,
                None,
            )
            .unwrap();
        let bytes = fs::read(diagnostics.directory.join("INCOMPLETE.json")).unwrap();
        let report: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(report["status"], "INCOMPLETE");
        assert_eq!(report["complete"], false);
        assert_eq!(report["restored"], false);
        assert_eq!(report["tableau_count"], 28);
        assert_eq!(report["stock_count"], 3);
        assert_eq!(report["draws_attempted"], 3);
        assert_eq!(report["draws_acknowledged"], 2);
        assert_eq!(report["stock_first_draw_order"][0], "3 of hearts");
        assert!(xsar::pyramid_json::from_slice(&bytes).is_err());
        assert!(!directory.join("deal.json").exists());
        fs::remove_dir_all(directory).unwrap();
    }


    /// Diagnostic write failures are visible and never overwrite existing evidence.
    #[test]
    fn diagnostic_collision_preserves_first_failure_without_retry() {
        let directory = test_directory();
        let mut diagnostics = Diagnostics::reserve(&directory.join("deal.json")).unwrap();
        diagnostics.record_capture(b"baseline".to_vec()).unwrap();
        let latest = diagnostics.directory.join("latest.png");
        write_private_new(&latest, b"keep-existing").unwrap();
        assert!(diagnostics.persist_latest().is_err());
        assert!(diagnostics.persist_latest().is_err());
        assert_eq!(fs::read(latest).unwrap(), b"keep-existing");
        assert!(
            write_private_new(&diagnostics.directory.join("baseline.png"), b"replace").is_err()
        );
        assert_eq!(
            fs::read(diagnostics.directory.join("baseline.png")).unwrap(),
            b"baseline"
        );
        fs::remove_dir_all(directory).unwrap();
    }


    /// An invalid profile fails before QMP, directory reservation or any guest input.
    #[test]
    fn invalid_profile_finishes_with_error_before_io_setup() {
        let directory = test_directory();
        let profile = directory.join("invalid-profile.json");
        write_private_new(&profile, b"{}").unwrap();
        let (control, context, cancel) = active_control();
        let (events, receiver) = mpsc::channel();
        let latest = LatestFrameSlot {
            reconnaissance: control,
            ..LatestFrameSlot::default()
        };
        let sink = WorkerEventSink {
            events,
            latest_frame: latest,
            capture_context: Mutex::new(Some(context.clone())),
        };
        let mut request = request();
        request.profile = ReconProfileSource::File(profile);
        request.output_path = directory.join("deal.json");
        run(&context, &request, &sink, &cancel);
        let notifications: Vec<_> = receiver.try_iter().collect();
        assert!(notifications.iter().any(|notification| matches!(
            &notification.event,
            WorkerEvent::ReconnaissanceFinished { result: Err(_), .. }
        )));
        assert!(!notifications.iter().any(|notification| matches!(
            notification.event,
            WorkerEvent::State(
                WorkerState::Connecting | WorkerState::Capturing | WorkerState::Acting
            )
        )));
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        assert!(sink.latest_frame.take().is_none());
        assert!(
            !sink
                .latest_frame
                .reconnaissance
                .state
                .lock()
                .unwrap()
                .active
        );
        fs::remove_dir_all(directory).unwrap();
    }


    /// Errors remain bounded even when a faulty adapter supplies excessive text.
    #[test]
    fn diagnostic_error_text_has_a_fixed_character_limit() {
        let directory = test_directory();
        let mut diagnostics = Diagnostics::reserve(&directory.join("deal.json")).unwrap();
        diagnostics
            .finish(
                &Err("\u{1f642}".repeat(50_000)),
                None,
                &InputLedger::default(),
                0,
                None,
            )
            .unwrap();
        let bytes = fs::read(diagnostics.directory.join("INCOMPLETE.json")).unwrap();
        assert!(bytes.len() <= DIAGNOSTIC_BYTES);
        let report: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            report["error"].as_str().unwrap().chars().count(),
            ERROR_CHARACTERS
        );
        fs::remove_dir_all(directory).unwrap();
    }
}


#[cfg(test)]
mod authority_epoch_tests {
    use super::*;


    /// Exhausted STOP cannot be cleared by reserving a new same-context run.
    #[test]
    fn saturated_stop_permanently_refuses_fresh_authority() {
        let control = ReconControl::default();
        let cancel = AtomicBool::new(false);
        let context = ControllerContext::new(
            PathBuf::from("/tmp/epoch-test"),
            crate::game::GameMode::Pyramid,
            SolvingStrategy::ShortestPath,
            1,
        );
        control.select(&context, &cancel).unwrap();
        control.state.lock().unwrap().epoch = u64::MAX;
        control.stop(Some(&context), &cancel);
        assert!(control.epoch().is_err());
        assert!(
            control
                .queue_reconnaissance(&context, &cancel, || panic!("must not dispatch"))
                .is_err()
        );
        assert!(
            control
                .queue_execution(&cancel, || panic!("must not dispatch"))
                .is_err()
        );
        assert!(cancel.load(Ordering::Acquire));
    }


    /// Context drift at the counter bound latches STOP before returning refusal.
    #[test]
    fn saturated_selection_latches_revocation() {
        let control = ReconControl::default();
        let cancel = AtomicBool::new(false);
        control.state.lock().unwrap().epoch = u64::MAX;
        let context = ControllerContext::new(
            PathBuf::from("/tmp/epoch-test"),
            crate::game::GameMode::Pyramid,
            SolvingStrategy::ShortestPath,
            1,
        );
        assert!(control.select(&context, &cancel).is_err());
        assert!(cancel.load(Ordering::Acquire));
        assert!(control.epoch().is_err());
    }
}


#[cfg(test)]
mod post_confirmation_tests;
