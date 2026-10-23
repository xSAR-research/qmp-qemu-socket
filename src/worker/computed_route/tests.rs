//! Deterministic route-controller simulations without transport or guest input.
//!
//! The accepted 49-action recording drives an independent replay observation
//! model. Synthetic guards and ACK failures test controller boundaries; native
//! QMP probe/release behaviour remains covered by transport and live tests.

use super::*;
use crate::geometry::PixelPoint;
use xsar::games::cards::Card;


mod challenges;


/// Interrupted search must not tell the operator that a winning route cannot exist.
#[test]
fn no_route_diagnostic_requires_exhaustive_proof() {
    for reason in [
        xsar::search::SearchStop::StateLimit,
        xsar::search::SearchStop::TimeLimit,
    ] {
        let message = no_winning_route_message(PartialProof::Bounded(reason));
        assert!(message.contains("Search interrupted"));
        assert!(message.contains("whether a winning route exists is unproved"));
        assert!(!message.contains("no winning route exists"));
    }
    let exhaustive = no_winning_route_message(PartialProof::Exhaustive);
    assert!(exhaustive.contains("Search exhausted all legal positions"));
    assert!(exhaustive.contains("no winning route exists under the supported rules"));
}


/// A selected named pipe is refused without waiting for an external writer.
#[test]
fn named_pipe_document_is_refused_before_open() {
    let directory = std::env::temp_dir().join(format!("route-fifo-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("unique FIFO test directory");
    let path = directory.join("not-a-deal");
    let created = std::process::Command::new("mkfifo")
        .arg(&path)
        .status()
        .expect("native FIFO utility");
    assert!(created.success());
    let result = read_document(&path);
    std::fs::remove_file(&path).expect("remove test FIFO");
    std::fs::remove_dir(&directory).expect("remove test directory");
    assert_eq!(result.unwrap_err(), "Route deal must be a regular file");
}


/// Resolve a retained sibling fixture through the authorised local dependency.
fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../xsar/tests/fixtures/pyramid-planning")
        .join(name)
}


/// Convert one explicitly recorded original location without compacting slots.
fn location(text: &str) -> Location {
    match text {
        "DL" => Location::DL,
        "DR" => Location::DR,
        _ => {
            let (row, column) = text.split_once('|').expect("recorded row|column");
            xsar::games::pyramid::planning::location_from_row_column(
                row.parse().expect("recorded row"),
                column.parse().expect("recorded column"),
            )
            .expect("checked original location")
        }
    }
}


/// Read the accepted CSV as an oracle; independent replay establishes legality.
fn accepted_actions() -> Vec<RecordedAction> {
    let csv = std::fs::read_to_string(fixture_path("pyramid-recon-20261019-v7-01-solver.csv"))
        .expect("retained accepted 49-action recording");
    let mut lines = csv.lines();
    assert_eq!(lines.next(), Some("SEQUENCE,ACTION,CARD1,CARD2"));
    lines
        .enumerate()
        .map(|(index, line)| {
            let fields: Vec<_> = line.split(',').collect();
            assert_eq!(fields.len(), 4);
            assert_eq!(fields[0].parse::<usize>().expect("sequence"), index + 1);
            match fields[1] {
                "MATCH" => RecordedAction::Match(location(fields[2]), location(fields[3])),
                "KING" => RecordedAction::King(location(fields[2])),
                "MOVE" => RecordedAction::Move,
                "RECYCLE" => RecordedAction::Recycle,
                other => panic!("unexpected accepted action {other}"),
            }
        })
        .collect()
}


/// Construct a private plan only inside tests, without fabricating runtime authority.
fn accepted_plan() -> Prepared {
    let bytes = std::fs::read(fixture_path("pyramid-recon-20261019-v7-01.json"))
        .expect("retained complete accepted deal");
    let deal = xsar::pyramid_json::read_from(bytes.as_slice()).expect("validated accepted deal");
    let actions = accepted_actions();
    assert_eq!(actions.len(), 49);
    let trace = replay::replay(&deal, &actions, true).expect("independent accepted-route replay");
    let vision = ReconVision::load(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon/native-v7/profile.json"),
    )
    .expect("unchanged native v7 profile");
    Prepared {
        id: 17,
        acquisition: Acquisition {
            challenge_progress: None,
            default_method: RouteMethod::FewestActions,
            observed_game: crate::pyramid_workflow_model::ObservedGame::Unknown,
            context: ControllerContext::new(
                "/unused/test-qmp.sock".into(),
                GameMode::Pyramid,
                SolvingStrategy::ShortestPath,
                3,
            ),
            epoch: 9,
            deal,
            bytes,
            vision,
        },
        trace,
        objective: RouteObjective::Clear,
    }
}


/// Reading authenticated advisory facts preserves the prepared route and obeys STOP/bytes.
#[test]
fn execution_observability_metadata_lookup_cannot_consume_or_reauthorise_a_plan() {
    use std::sync::{Mutex, mpsc};
    let mut plan = accepted_plan();
    let context = plan.acquisition.context.clone();
    let cancel = AtomicBool::new(false);
    let (tx, _rx) = mpsc::channel();
    let events = WorkerEventSink {
        events: tx,
        latest_frame: super::super::LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(context.clone())),
    };
    let control = &events.latest_frame.reconnaissance;
    control.select(&context, &cancel).unwrap();
    control
        .queue_reconnaissance(&context, &cancel, || Ok(()))
        .unwrap();
    plan.acquisition.epoch = control.epoch().unwrap();
    let observed = crate::pyramid_workflow_model::ObservedGame::Challenge {
        progress: crate::pyramid_progress::ChallengeProgress::new(
            crate::pyramid_progress::ChallengeKind::Boards,
            2,
            3,
        )
        .unwrap(),
        kings: false,
    };
    plan.acquisition.observed_game = observed;
    events.latest_frame.computed_route.lock().unwrap().prepared = Some(plan);
    let path = fixture_path("pyramid-recon-20261019-v7-01.json");
    assert_eq!(
        acquisition_observed(&context, &path, &events, &cancel).unwrap(),
        observed
    );
    assert_eq!(
        events
            .latest_frame
            .computed_route
            .lock()
            .unwrap()
            .prepared
            .as_ref()
            .unwrap()
            .id,
        17
    );
    assert!(
        acquisition_observed(
            &context,
            &fixture_path("pyramid-recon-20261018-v6-01.json"),
            &events,
            &cancel
        )
        .is_err()
    );
    assert!(
        events
            .latest_frame
            .computed_route
            .lock()
            .unwrap()
            .prepared
            .is_some()
    );
    control.stop(Some(&context), &cancel);
    assert!(acquisition_observed(&context, &path, &events, &cancel).is_err());
}


/// Map the oracle's location directly through the authoritative semantic geometry.
fn oracle_click(location: Location) -> InputOperation {
    let kind = match location {
        Location::DL => PyramidTargetKind::Left,
        Location::DR => PyramidTargetKind::Right,
        Location::Tableau(slot) => {
            let (row, column) =
                xsar::games::pyramid::planning::row_column_from_location(Location::Tableau(slot))
                    .expect("recorded original coordinates");
            PyramidTargetKind::Card {
                row: u8::try_from(row).expect("row fits"),
                column: u8::try_from(column).expect("column fits"),
            }
        }
    };
    let target = pyramid::PYRAMID_TARGETS
        .iter()
        .find(|target| target.kind == kind)
        .expect("calibrated semantic target");
    InputOperation::Click(target.click_point)
}


/// Expand independently transcribed logical actions to their exact ordered gestures.
fn oracle_operations(action: RecordedAction) -> Vec<InputOperation> {
    match action {
        RecordedAction::Match(first, second) => vec![oracle_click(first), oracle_click(second)],
        RecordedAction::King(target) => vec![oracle_click(target)],
        RecordedAction::Move | RecordedAction::Recycle => vec![InputOperation::PressDrawKey],
    }
}


/// Independent visible model advanced from replay identities rather than rules state.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ObservedModel {
    /// Original 28 slots retain their identity after earlier removals.
    tableau: [Option<Card>; 28],
    /// Independently replayed current LEFT identity.
    left: Option<Card>,
    /// Independently replayed current RIGHT identity.
    right: Option<Card>,
}


/// Deterministic guarded I/O with independent model, virtual clock and injected faults.
struct FakeIo<'a> {
    /// Explicit test-only no-challenge policy.
    ack_handoff: bool,
    /// Deterministic cancellation at the new three-second wait.
    stop_handoff_wait: bool,
    /// Inject a genuine terminal proof after this independently completed whole action.
    challenge_after: Option<usize>,
    /// Inject a reached count only after independent exact-position verification.
    pending_after: Option<(usize, ChallengeProgress)>,
    /// Whole-action observer invocations retain action and partial-gesture counts.
    challenge_observations: Vec<(usize, usize)>,
    /// STOP may arrive during a terminal capture, before the proof is accepted.
    stop_during_challenge: bool,
    /// Read-only accepted route and original card identities.
    plan: &'a Prepared,
    /// Observations independently reconstructed from replay records.
    actual: ObservedModel,
    /// Number of logical actions whose complete gestures affected the model.
    action: usize,
    /// Acknowledged gestures within the current logical action.
    partial_gestures: usize,
    /// Actual attempted operation sequence, including an uncertain attempt.
    delivered: Vec<InputOperation>,
    /// Requested expectations and final-effect classification for audit assertions.
    observations: Vec<(SearchState, bool)>,
    /// Committed logical effects reported by the controller.
    committed: Vec<(usize, usize)>,
    /// Actual attempts and acknowledgements remain separate.
    counts: Counts,
    /// One monotonic virtual run clock, advanced only by waits.
    elapsed: Duration,
    /// Immediate synthetic STOP latch.
    stopped: bool,
    /// Set STOP immediately after this acknowledged gesture count.
    stop_after_ack: Option<usize>,
    /// Fail this transport attempt before acknowledgement.
    uncertain_attempt: Option<usize>,
    /// Return successfully from this attempt without an acknowledgement receipt.
    ackless_success_attempt: Option<usize>,
    /// Reject this delivery guard before transport is attempted.
    wrong_state_delivery: Option<usize>,
    /// Fail this fresh capability guard before transport is attempted.
    failed_probe_delivery: Option<usize>,
    /// Reject every observation before any input.
    wrong_origin: bool,
    /// Keep the observed model unchanged despite acknowledged transport.
    freeze_effects: bool,
    /// Final observation starts retain their ACK count and monotonic timestamp.
    final_observations: Vec<(Duration, usize)>,
    /// Suppress this many final visual matches without changing delivered input.
    final_misses: usize,
    /// Simulate time spent processing after the last ACK was already recorded.
    final_ack_postprocessing: Duration,
    /// Time consumed by one final capture that started before the deadline.
    final_capture_delay: Duration,
    /// STOP may arrive while a final capture is already in progress.
    stop_during_final_observation: bool,
    /// Current context may be revoked while a final capture is in progress.
    revoked_context: bool,
    /// Inject context revocation on the first final capture.
    revoke_during_final_observation: bool,
    /// Simulate a clock moving backwards during the final observation wait.
    reverse_final_clock: bool,
}


impl<'a> FakeIo<'a> {
    /// Start from the accepted immutable original and independent initial pile labels.
    fn new(plan: &'a Prepared) -> Self {
        Self {
            ack_handoff: false,
            stop_handoff_wait: false,
            challenge_after: None,
            pending_after: None,
            challenge_observations: Vec::new(),
            stop_during_challenge: false,
            plan,
            actual: ObservedModel {
                tableau: plan.acquisition.deal.tableau().map(Some),
                left: plan.acquisition.deal.stock().first().copied(),
                right: None,
            },
            action: 0,
            partial_gestures: 0,
            delivered: Vec::new(),
            observations: Vec::new(),
            committed: Vec::new(),
            counts: Counts::default(),
            elapsed: Duration::ZERO,
            stopped: false,
            stop_after_ack: None,
            uncertain_attempt: None,
            ackless_success_attempt: None,
            wrong_state_delivery: None,
            failed_probe_delivery: None,
            wrong_origin: false,
            freeze_effects: false,
            final_observations: Vec::new(),
            final_misses: 0,
            final_ack_postprocessing: Duration::ZERO,
            final_capture_delay: Duration::ZERO,
            stop_during_final_observation: false,
            revoked_context: false,
            revoke_during_final_observation: false,
            reverse_final_clock: false,
        }
    }


    /// Apply completed action identities and pile tops from the independent replay.
    fn apply_observed_effect(&mut self) {
        let step = self.plan.trace[self.action];
        if matches!(
            step.action,
            RecordedAction::Match(..) | RecordedAction::King(..)
        ) {
            for index in [step.first, step.second].into_iter().flatten() {
                if let Some(slot) = self.actual.tableau.get_mut(usize::from(index)) {
                    assert!(slot.take().is_some(), "an observed card is removed once");
                }
            }
        }
        self.actual.left = step
            .dl_after
            .and_then(|index| rules::card_at(&self.plan.acquisition.deal, index));
        self.actual.right = step
            .dr_after
            .and_then(|index| rules::card_at(&self.plan.acquisition.deal, index));
        assert_eq!(
            self.actual.tableau.iter().flatten().count(),
            usize::from(step.remaining_tableau),
        );
        self.action += 1;
        self.partial_gestures = 0;
    }
}


impl RouteIo for FakeIo<'_> {
    /// Enable only the opt-in handoff scenario.
    fn handoff_after_final_ack(&self) -> bool {
        self.ack_handoff
    }

    /// Inject terminal evidence only through the controller's whole-action effect seam.
    fn observe_effect(
        &mut self,
        state: SearchState,
        final_effect: bool,
    ) -> Result<EffectObservation, String> {
        self.challenge_observations
            .push((self.action, self.partial_gestures));
        if self.challenge_after == Some(self.action) {
            self.stopped |= self.stop_during_challenge;
            return Ok(EffectObservation::Challenge(challenges::proof()));
        }
        let matched = self.observe(state, final_effect)?;
        if matched
            && let Some((action, progress)) = self.pending_after
            && action == self.action
        {
            self.stopped |= self.stop_during_challenge;
            return Ok(EffectObservation::ChallengePending(progress));
        }
        Ok(EffectObservation::Position(matched))
    }


    /// Preserve the distinct terminal result through final-clear fastpath dispatch.
    fn observe_final(&mut self, state: SearchState) -> Result<Option<BoardCompletion>, String> {
        match self.observe_effect(state, true)? {
            EffectObservation::Position(matched) => {
                Ok(matched.then_some(BoardCompletion::BoardClear))
            }
            EffectObservation::Challenge(proof) => {
                Ok(Some(BoardCompletion::ChallengeTargetVerified(proof)))
            }
            EffectObservation::ChallengePending(progress) => {
                Ok(Some(BoardCompletion::ChallengeTargetReached(progress)))
            }
        }
    }


    /// Refuse STOP at every operation boundary without any cleanup operation.
    fn check(&self) -> Result<(), String> {
        if self.stopped {
            Err("synthetic STOP latched".to_owned())
        } else if self.revoked_context {
            Err("synthetic context revoked".to_owned())
        } else {
            Ok(())
        }
    }


    /// Compare requested state with replay-derived actual identities and pile tops.
    fn observe(&mut self, state: SearchState, final_effect: bool) -> Result<bool, String> {
        self.check()?;
        self.observations.push((state, final_effect));
        let partial_endpoint = matches!(self.plan.objective, RouteObjective::Removal { .. })
            && self.action == self.plan.trace.len();
        if final_effect || partial_endpoint {
            self.final_observations
                .push((self.elapsed, self.counts.acknowledged));
            self.elapsed += self.final_capture_delay;
            self.stopped |= self.stop_during_final_observation;
            self.revoked_context |= self.revoke_during_final_observation;
            if self.final_observations.len() <= self.final_misses {
                return Ok(false);
            }
        }
        let (tableau, left, right) = expected(&self.plan.acquisition.deal, state)?;
        Ok(!self.wrong_origin
            && self.actual
                == ObservedModel {
                    tableau,
                    left,
                    right,
                })
    }


    /// Guard and attempt exactly once; only a complete acknowledged action affects pixels.
    fn deliver(&mut self, operation: InputOperation, state: SearchState) -> Result<(), String> {
        self.check()?;
        let delivery = self.counts.attempted + 1;
        if self.wrong_state_delivery == Some(delivery) || !self.observe(state, false)? {
            return Err("synthetic fresh pre-gesture position mismatch".to_owned());
        }
        if self.failed_probe_delivery == Some(delivery) {
            return Err("synthetic absolute-pointer capability unavailable".to_owned());
        }
        self.check()?;
        let oracle = oracle_operations(self.plan.trace[self.action].action);
        assert_eq!(operation, oracle[self.partial_gestures]);
        self.counts.attempted += 1;
        self.delivered.push(operation);
        if self.uncertain_attempt == Some(self.counts.attempted) {
            return Err("synthetic delivery uncertainty; no replay".to_owned());
        }
        if self.ackless_success_attempt == Some(self.counts.attempted) {
            return Ok(());
        }
        self.counts.acknowledged += 1;
        self.counts.last_acknowledged_at = Some(self.elapsed);
        self.partial_gestures += 1;
        if self.partial_gestures == oracle.len() && !self.freeze_effects {
            self.apply_observed_effect();
            if self.action == self.plan.trace.len() {
                self.elapsed += self.final_ack_postprocessing;
            }
        }
        if self.stop_after_ack == Some(self.counts.acknowledged) {
            self.stopped = true;
        }
        Ok(())
    }


    /// Advance a monotonic deterministic clock without sleeping.
    fn wait(&mut self, delay: Duration) -> Result<(), String> {
        self.check()?;
        if self.stop_handoff_wait
            && self.action == self.plan.trace.len()
            && delay == Duration::from_secs(3)
        {
            self.stopped = true;
        }
        if self.reverse_final_clock && self.action == self.plan.trace.len() {
            self.elapsed = self.elapsed.saturating_sub(delay);
        } else {
            self.elapsed += delay;
        }
        self.check()
    }


    /// Return elapsed time from the same virtual run origin.
    fn elapsed(&self) -> Duration {
        self.elapsed
    }


    /// Return actual transport accounting, independent of logical commits.
    fn counts(&self) -> Counts {
        self.counts
    }


    /// Commit only complete observed actions in one-based presentation order.
    fn progress(&mut self, index: usize, total: usize) {
        assert_eq!(index, self.action);
        assert_eq!(self.partial_gestures, 0);
        self.committed.push((index, total));
    }
}


/// A clear-route ACK schedules exactly three seconds with no final visual prerequisite.
#[test]
fn no_challenge_final_ack_waits_three_seconds_without_claiming_visual_success() {
    let plan = accepted_plan();
    let mut io = FakeIo::new(&plan);
    io.ack_handoff = true;
    io.final_misses = usize::MAX;
    let report = drive(&plan, &mut io).unwrap();
    assert_eq!(report.completion, BoardCompletion::AwaitingFreshBoard);
    assert!(io.final_observations.is_empty());
    assert_eq!(
        io.elapsed - io.counts.last_acknowledged_at.unwrap(),
        Duration::from_secs(3)
    );
    let deliveries = io.delivered.len();
    let mut stopped = FakeIo::new(&plan);
    stopped.ack_handoff = true;
    stopped.stop_handoff_wait = true;
    assert!(drive(&plan, &mut stopped).is_err());
    assert_eq!(stopped.delivered.len(), deliveries);
    assert!(stopped.final_observations.is_empty());
}


/// Retain a legal non-clear removal prefix, independently replayed from the full origin.
///
/// This synthetic controller fixture asserts no maximum-removal search proof.
fn partial_plan() -> Prepared {
    let mut plan = accepted_plan();
    let end = plan
        .trace
        .iter()
        .position(|step| step.remaining_tableau <= 22)
        .unwrap()
        + 1;
    let actions: Vec<_> = plan.trace[..end].iter().map(|step| step.action).collect();
    plan.trace = replay::replay(&plan.acquisition.deal, &actions, false)
        .expect("independent partial replay");
    let remaining = plan.trace.last().unwrap().remaining_tableau;
    assert!(remaining > 0);
    plan.objective = RouteObjective::Removal {
        removed: 28 - remaining,
        proof: PartialProof::Bounded(xsar::search::SearchStop::ExpansionLimit),
    };
    plan
}


/// A partial endpoint uses ordinary position evidence and commits no clear outcome.
#[test]
fn partial_route_verifies_exact_remaining_position_without_completion_authority() {
    let plan = partial_plan();
    let remaining = plan.trace.last().unwrap().remaining_tableau;
    let operations: Vec<_> = plan
        .trace
        .iter()
        .flat_map(|step| oracle_operations(step.action))
        .collect();
    let mut io = FakeIo::new(&plan);
    let report = drive(&plan, &mut io).expect("independent partial effect");
    assert_eq!(
        report.completion,
        BoardCompletion::PartialPositionVerified {
            removed: 28 - remaining,
            remaining
        }
    );
    assert_eq!(io.delivered, operations);
    assert!(
        io.observations
            .iter()
            .all(|(_, final_effect)| !final_effect)
    );
    assert_eq!(io.final_observations.len(), 1);
    assert_eq!(
        io.committed.last(),
        Some(&(plan.trace.len(), plan.trace.len()))
    );
    assert!(report.message.contains("no next-board or restart input"));
}


/// Uncertainty, STOP and context revocation never turn a partial delivery into completion.
#[test]
fn partial_final_confirmation_is_bounded_and_revocable_without_replay() {
    let plan = partial_plan();
    for cause in 0..3 {
        let mut io = FakeIo::new(&plan);
        match cause {
            0 => io.final_misses = usize::MAX,
            1 => io.stop_during_final_observation = true,
            _ => io.revoke_during_final_observation = true,
        }
        let report = drive(&plan, &mut io);
        if cause == 0 {
            assert_eq!(
                report.unwrap().completion,
                BoardCompletion::DeliveredUnconfirmed
            );
            assert_eq!(
                io.elapsed,
                io.counts.last_acknowledged_at.unwrap() + FINAL_CONFIRMATION_WINDOW
            );
            assert_eq!(io.final_observations.len(), 9);
        } else {
            assert!(report.is_err());
            assert_eq!(io.final_observations.len(), 1);
        }
        assert_eq!(io.counts.attempted, io.counts.acknowledged);
        assert_eq!(
            io.delivered.len(),
            plan.trace
                .iter()
                .flat_map(|step| oracle_operations(step.action))
                .count()
        );
        assert_eq!(io.committed.len(), plan.trace.len() - 1);
        assert!(
            io.observations
                .iter()
                .all(|(_, final_effect)| !final_effect)
        );
    }
}


/// A wrong claimed removal count is refused before any observation or input.
#[test]
fn partial_route_objective_must_match_independently_replayed_removals() {
    let mut plan = partial_plan();
    plan.objective = RouteObjective::Removal {
        removed: 1,
        proof: PartialProof::Exhaustive,
    };
    let mut io = FakeIo::new(&plan);
    let error = drive(&plan, &mut io).expect_err("misstated endpoint objective");
    assert!(error.contains("match the replayed objective"));
    assert!(io.delivered.is_empty() && io.observations.is_empty());
}


/// Full simulation retains the accepted 49 actions, exact 73 gestures and original deal.
#[test]
fn accepted_49_actions_complete_with_73_exact_gestures_and_independent_effects() {
    let plan = accepted_plan();
    let original = plan.acquisition.deal.clone();
    let original_bytes = plan.acquisition.bytes.clone();
    let expected_operations: Vec<_> = accepted_actions()
        .into_iter()
        .flat_map(oracle_operations)
        .collect();
    assert_eq!(expected_operations.len(), 73);
    assert_eq!(
        expected_operations[0],
        InputOperation::Click(PixelPoint::new(1361, 456))
    );
    assert_eq!(
        expected_operations[1],
        InputOperation::Click(PixelPoint::new(1562, 456))
    );
    let mut io = FakeIo::new(&plan);
    let result = drive(&plan, &mut io).expect("full deterministic tableau route");
    assert!(result.message.contains("49/49 actions"));
    assert!(result.message.contains("73 gestures acknowledged"));
    assert_eq!(result.completion, BoardCompletion::BoardClear);
    assert_eq!(io.delivered, expected_operations);
    assert_eq!(io.counts.attempted, 73);
    assert_eq!(io.counts.acknowledged, 73);
    assert_eq!(
        io.committed,
        (1..=49).map(|index| (index, 49)).collect::<Vec<_>>()
    );
    assert_eq!(io.actual.tableau, [None; 28]);
    assert_eq!(io.actual.left, None);
    assert_eq!(io.actual.right, None);
    assert_eq!(
        io.observations
            .iter()
            .filter(|(_, final_effect)| *final_effect)
            .count(),
        1
    );
    assert_eq!(plan.acquisition.deal, original);
    assert_eq!(plan.acquisition.bytes, original_bytes);
}


/// STOP between pair clicks leaves the first selection partial and sends no second click.
#[test]
fn stop_after_first_pair_ack_refuses_second_click_and_logical_commit() {
    let plan = accepted_plan();
    let mut io = FakeIo::new(&plan);
    let original = io.actual.clone();
    io.stop_after_ack = Some(1);
    assert!(drive(&plan, &mut io).is_err());
    assert_eq!(io.counts.attempted, 1);
    assert_eq!(io.counts.acknowledged, 1);
    assert_eq!(io.partial_gestures, 1);
    assert_eq!(io.actual, original);
    assert!(io.committed.is_empty());
    assert_eq!(io.delivered.len(), 1);
}


/// Uncertain first or second pair delivery never repeats input or claims a pair effect.
#[test]
fn uncertain_pair_delivery_retains_actual_counts_without_commit_or_replay() {
    for uncertain_attempt in [1, 2] {
        let plan = accepted_plan();
        let mut io = FakeIo::new(&plan);
        let original = io.actual.clone();
        io.uncertain_attempt = Some(uncertain_attempt);
        assert!(
            drive(&plan, &mut io)
                .expect_err("uncertain input must stop")
                .contains("uncertainty")
        );
        assert_eq!(io.counts.attempted, uncertain_attempt);
        assert_eq!(io.counts.acknowledged, uncertain_attempt - 1);
        assert_eq!(io.delivered.len(), uncertain_attempt);
        assert_eq!(io.actual, original);
        assert!(io.committed.is_empty());
    }
}


/// A successful adapter return never substitutes for the actual acknowledgement receipt.
#[test]
fn missing_acknowledgement_receipt_refuses_even_when_delivery_returns_success() {
    let plan = accepted_plan();
    let mut io = FakeIo::new(&plan);
    let original = io.actual.clone();
    io.ackless_success_attempt = Some(1);
    assert!(drive(&plan, &mut io).is_err());
    assert_eq!(io.counts.attempted, 1);
    assert_eq!(io.counts.acknowledged, 0);
    assert_eq!(io.delivered.len(), 1);
    assert_eq!(io.actual, original);
    assert!(io.committed.is_empty());
}


/// Acknowledgement without observed removal exhausts observation bounds without replay.
#[test]
fn acknowledged_pair_without_effect_never_commits_or_repeats_input() {
    let plan = accepted_plan();
    let mut io = FakeIo::new(&plan);
    let original = io.actual.clone();
    io.freeze_effects = true;
    assert!(drive(&plan, &mut io).is_err());
    assert_eq!(io.counts.attempted, 2);
    assert_eq!(io.counts.acknowledged, 2);
    assert_eq!(io.delivered.len(), 2);
    assert_eq!(io.actual, original);
    assert!(io.committed.is_empty());
    assert!(
        io.observations.len() <= 12,
        "effect capture retries remain bounded"
    );
}


/// Origin mismatch refuses before any gesture, even if a replayed route is legal.
#[test]
fn wrong_origin_refuses_before_any_transport_attempt() {
    let plan = accepted_plan();
    let mut io = FakeIo::new(&plan);
    io.wrong_origin = true;
    assert!(
        drive(&plan, &mut io)
            .expect_err("wrong origin")
            .contains("Original board mismatch")
    );
    assert_eq!(io.counts.attempted, 0);
    assert_eq!(io.counts.acknowledged, 0);
    assert!(io.delivered.is_empty());
    assert!(io.committed.is_empty());
}


/// Fresh position/probe failures before either pair gesture preserve prior accounting.
#[test]
fn fresh_state_or_capability_refusal_does_not_attempt_guarded_gesture() {
    for delivery in [1, 2] {
        for capability_failure in [false, true] {
            let plan = accepted_plan();
            let mut io = FakeIo::new(&plan);
            if capability_failure {
                io.failed_probe_delivery = Some(delivery);
            } else {
                io.wrong_state_delivery = Some(delivery);
            }
            assert!(drive(&plan, &mut io).is_err());
            assert_eq!(io.counts.attempted, delivery - 1);
            assert_eq!(io.counts.acknowledged, delivery - 1);
            assert_eq!(io.delivered.len(), delivery - 1);
            assert!(io.committed.is_empty());
        }
    }
}


/// An already latched STOP refuses origin capture and every transport operation.
#[test]
fn preexisting_stop_performs_no_capture_or_input() {
    let plan = accepted_plan();
    let mut io = FakeIo::new(&plan);
    io.stopped = true;
    assert!(drive(&plan, &mut io).is_err());
    assert!(io.observations.is_empty());
    assert!(io.delivered.is_empty());
    assert!(io.committed.is_empty());
}


/// Exercise the real private receipt minting and calculation without a guest socket.
#[test]
fn same_session_receipt_accepts_identical_moved_document_and_consumes_execution_once() {
    use std::sync::{Mutex, mpsc};
    let plan = accepted_plan();
    let context = plan.acquisition.context.clone();
    let cancel = AtomicBool::new(false);
    let (tx, _rx) = mpsc::channel();
    let events = WorkerEventSink {
        events: tx,
        latest_frame: super::super::LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(context.clone())),
    };
    let control = &events.latest_frame.reconnaissance;
    control.select(&context, &cancel).unwrap();
    control
        .queue_reconnaissance(&context, &cancel, || Ok(()))
        .unwrap();
    let source = fixture_path("pyramid-recon-20261019-v7-01.json");
    record_acquisition(
        &context,
        &events,
        &cancel,
        plan.acquisition.deal,
        plan.acquisition.vision,
        &source,
        None,
        RouteMethod::FewestActions,
        crate::pyramid_workflow_model::ObservedGame::Unknown,
    )
    .unwrap();
    control.finish();
    let directory = std::env::temp_dir().join(format!(
        "route-receipt-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::create_dir(&directory).unwrap();
    let moved = directory.join("original-moved-without-extension");
    std::fs::write(&moved, &plan.acquisition.bytes).unwrap();
    control
        .queue_reconnaissance(&context, &cancel, || Ok(()))
        .unwrap();
    calculate(&context, 900, &moved, &events, &cancel);
    {
        let store = events.latest_frame.computed_route.lock().unwrap();
        let prepared = store
            .prepared
            .as_ref()
            .expect("matching exact bytes authenticate moved filename");
        assert_eq!(prepared.id, 900);
        assert_eq!(prepared.trace.last().unwrap().remaining_tableau, 0);
        assert_eq!(prepared.objective, RouteObjective::Clear);
    }
    // Consent mismatch consumes the permit before QMP and cannot be retried.
    control
        .queue_reconnaissance(&context, &cancel, || Ok(()))
        .unwrap();
    execute(&context, 901, 900, false, &events, &cancel);
    let store = events.latest_frame.computed_route.lock().unwrap();
    assert!(store.prepared.is_none());
    assert!(store.acquisition.is_none());
    std::fs::remove_file(moved).unwrap();
    std::fs::remove_dir(directory).unwrap();
}


/// The automatic Fast consumer runs real weighted search and independent replay.
///
/// A test-only acquisition authenticates exact bytes without a guest socket. The
/// owned planner must retain its reservation, publish final real work counters
/// with the board identity, and avoid claiming a minimum card-match cost.
#[test]
fn owned_fast_calculation_retains_authority_and_final_weighted_search_statistics() {
    use std::sync::{Mutex, mpsc};
    let plan = accepted_plan();
    let context = plan.acquisition.context.clone();
    let original = plan.acquisition.deal.clone();
    let original_bytes = plan.acquisition.bytes.clone();
    let cancel = AtomicBool::new(false);
    let (tx, rx) = mpsc::channel();
    let events = WorkerEventSink {
        events: tx,
        latest_frame: super::super::LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(context.clone())),
    };
    let control = &events.latest_frame.reconnaissance;
    control.select(&context, &cancel).unwrap();
    control
        .queue_reconnaissance(&context, &cancel, || Ok(()))
        .unwrap();
    record_acquisition(
        &context,
        &events,
        &cancel,
        plan.acquisition.deal,
        plan.acquisition.vision,
        &fixture_path("pyramid-recon-20261019-v7-01.json"),
        None,
        RouteMethod::FewestActions,
        crate::pyramid_workflow_model::ObservedGame::Unknown,
    )
    .expect("private test-only same-session receipt");
    let epoch = control.epoch().unwrap();
    let directory = std::env::temp_dir().join(format!(
        "route-fast-owned-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::create_dir(&directory).unwrap();
    let moved = directory.join("same-deal-without-extension");
    std::fs::write(&moved, &original_bytes).unwrap();

    let result = calculate_owned(
        &context,
        902,
        &moved,
        RouteMethod::Fast,
        7,
        &events,
        &cancel,
    );
    std::fs::remove_file(moved).unwrap();
    std::fs::remove_dir(directory).unwrap();
    let report = result.expect("real Fast weighted search and independent replay");
    assert!(
        report.contains("winning route verified; minimum logical-action count not established"),
        "{report}"
    );
    assert!(report.contains("method=Weighted A* (weight 2); independent replay passed"));
    assert!(!report.contains("optimal under"));
    assert!(control.check_active(&context, &cancel).is_ok());
    assert_eq!(control.epoch().unwrap(), epoch);
    assert!(
        control
            .queue_reconnaissance(&context, &cancel, || {
                panic!("owned planning must retain the existing reservation")
            })
            .unwrap_err()
            .contains("already queued or running")
    );
    assert!(!cancel.load(std::sync::atomic::Ordering::Acquire));
    let store = events.latest_frame.computed_route.lock().unwrap();
    assert!(store.acquisition.is_none());
    let prepared = store
        .prepared
        .as_ref()
        .expect("independently verified route");
    assert_eq!(prepared.id, 902);
    assert_eq!(prepared.acquisition.context, context);
    assert_eq!(prepared.acquisition.epoch, epoch);
    assert_eq!(prepared.acquisition.bytes, original_bytes);
    assert_eq!(prepared.acquisition.deal, original);
    let actions: Vec<_> = prepared.trace.iter().map(|step| step.action).collect();
    let independently_replayed = replay::replay(&original, &actions, true)
        .expect("calculated actions independently clear the original tableau");
    assert_eq!(prepared.trace, independently_replayed);
    assert_eq!(prepared.trace.last().unwrap().remaining_tableau, 0);

    let progress = events
        .latest_frame
        .planning_progress
        .lock()
        .unwrap()
        .take()
        .expect("final search snapshot replaces the bounded latest slot");
    assert_eq!(progress.context, context);
    assert_eq!(progress.run_id, 902);
    assert_eq!(progress.board_generation, 7);
    assert_eq!(progress.method, RouteMethod::Fast);
    assert!(progress.finished);
    assert!(progress.stats.expanded > 0);
    assert!(progress.stats.generated >= actions.len());
    assert!(progress.stats.states > actions.len());
    assert!(progress.stats.peak_frontier > 0);
    assert!(progress.stats.elapsed > Duration::ZERO);
    assert!(report.contains(&format!("expanded={}", progress.stats.expanded)));
    assert!(report.contains(&format!("generated={}", progress.stats.generated)));
    assert!(report.contains(&format!("histories={}", progress.stats.states)));
    assert!(report.contains(&format!(
        "elapsed={:.3}s",
        progress.stats.elapsed.as_secs_f64()
    )));
    assert!(
        !rx.try_iter()
            .any(|envelope| matches!(envelope.event, WorkerEvent::ComputedRouteFinished { .. }))
    );
}


/// A standalone valid export cannot create live authority and never reaches QMP.
#[test]
fn calculation_without_successful_session_receipt_has_no_plan() {
    use std::sync::{Mutex, mpsc};
    let context = accepted_plan().acquisition.context;
    let cancel = AtomicBool::new(false);
    let (tx, rx) = mpsc::channel();
    let events = WorkerEventSink {
        events: tx,
        latest_frame: super::super::LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(context.clone())),
    };
    let control = &events.latest_frame.reconnaissance;
    control.select(&context, &cancel).unwrap();
    control
        .queue_reconnaissance(&context, &cancel, || Ok(()))
        .unwrap();
    calculate(
        &context,
        5,
        &fixture_path("pyramid-recon-20261019-v7-01.json"),
        &events,
        &cancel,
    );
    assert!(
        events
            .latest_frame
            .computed_route
            .lock()
            .unwrap()
            .prepared
            .is_none()
    );
    assert!(rx.try_iter().any(|event| matches!(
        event.event,
        WorkerEvent::ComputedRouteFinished {
            run_id: 5,
            result: Err(_),
            ..
        }
    )));
}


/// Epoch drift cannot be undone by selecting the original context and clearing STOP.
#[test]
fn context_changed_and_back_or_idle_stop_cannot_reauthenticate_acquisition() {
    use std::sync::{Mutex, mpsc};
    for idle_stop in [false, true] {
        let plan = accepted_plan();
        let context = plan.acquisition.context.clone();
        let cancel = AtomicBool::new(false);
        let (tx, _rx) = mpsc::channel();
        let events = WorkerEventSink {
            events: tx,
            latest_frame: super::super::LatestFrameSlot::default(),
            capture_context: Mutex::new(Some(context.clone())),
        };
        let control = &events.latest_frame.reconnaissance;
        control.select(&context, &cancel).unwrap();
        control
            .queue_reconnaissance(&context, &cancel, || Ok(()))
            .unwrap();
        record_acquisition(
            &context,
            &events,
            &cancel,
            plan.acquisition.deal,
            plan.acquisition.vision,
            &fixture_path("pyramid-recon-20261019-v7-01.json"),
            None,
            RouteMethod::FewestActions,
            crate::pyramid_workflow_model::ObservedGame::Unknown,
        )
        .unwrap();
        control.finish();
        if idle_stop {
            control.stop(Some(&context), &cancel);
        } else {
            let mut changed = context.clone();
            changed.generation += 1;
            control.select(&changed, &cancel).unwrap();
            control.select(&context, &cancel).unwrap();
        }
        control
            .queue_reconnaissance(&context, &cancel, || Ok(()))
            .unwrap();
        calculate(
            &context,
            8,
            &fixture_path("pyramid-recon-20261019-v7-01.json"),
            &events,
            &cancel,
        );
        let store = events.latest_frame.computed_route.lock().unwrap();
        assert!(store.prepared.is_none());
        assert!(store.acquisition.is_none());
    }
}


/// Phase-derived capture bounds cover every allowed schedule without splitting pairs.
#[test]
fn capture_budget_covers_phase_worst_cases_and_rejects_invalid_counts() {
    for actions in 1..=MAX_ROUTE_ACTIONS {
        for gestures in actions..=MAX_ROUTE_GESTURES {
            let allowance = capture_allowance(actions, gestures).unwrap();
            assert!(allowance >= 2 + gestures + 6 * actions + 6 * (actions - 1) + 20);
            assert!(allowance >= 2 + 6 * (gestures + actions) + 20);
        }
    }
    assert!(capture_allowance(0, 0).is_err());
    assert!(capture_allowance(usize::MAX, usize::MAX).is_err());
}


/// The accepted last MATCH waits only after both clicks are fully acknowledged.
#[test]
fn final_match_confirmation_starts_200ms_after_all_73_acknowledgements() {
    let plan = accepted_plan();
    assert!(matches!(
        plan.trace.last().unwrap().action,
        RecordedAction::Match(..)
    ));
    let mut io = FakeIo::new(&plan);
    let result = drive(&plan, &mut io).expect("accepted final MATCH");
    let ack = io
        .counts
        .last_acknowledged_at
        .expect("actual last ACK clock");
    assert_eq!(result.completion, BoardCompletion::BoardClear);
    assert_eq!(
        io.final_observations,
        vec![(ack + FINAL_OBSERVATION_DELAY, 73)]
    );
    assert_eq!(io.elapsed, ack + FINAL_OBSERVATION_DELAY);
    assert_eq!(io.delivered.len(), 73);
    assert_eq!(io.committed.last(), Some(&(49, 49)));
}


/// Swap two original identities only in this synthetic model to end with one KING.
///
/// The full modified route is independently replayed. This private controller
/// fixture creates no acquisition receipt, export or native recognition claim.
fn synthetic_last_king_plan() -> Prepared {
    let mut plan = accepted_plan();
    let mut tableau = *plan.acquisition.deal.tableau();
    tableau.swap(0, 1);
    let deal = CompleteDeal::try_new(
        tableau,
        *plan.acquisition.deal.stock(),
        plan.acquisition.deal.metadata().clone(),
    )
    .expect("identity swap preserves complete unique deck");
    let mut actions = accepted_actions();
    actions[47] = RecordedAction::Match(location("2|1"), Location::DR);
    actions[48] = RecordedAction::King(location("1|1"));
    plan.trace = replay::replay(&deal, &actions, true).expect("synthetic legal last KING");
    plan.acquisition.deal = deal;
    plan
}


/// The single last KING receives the same short window without another card gesture.
#[test]
fn final_king_confirmation_starts_after_its_only_acknowledged_gesture() {
    let plan = synthetic_last_king_plan();
    assert!(matches!(
        plan.trace.last().unwrap().action,
        RecordedAction::King(..)
    ));
    let mut io = FakeIo::new(&plan);
    let result = drive(&plan, &mut io).expect("synthetic final KING");
    let ack = io.counts.last_acknowledged_at.unwrap();
    assert_eq!(result.completion, BoardCompletion::BoardClear);
    assert_eq!(
        io.final_observations,
        vec![(ack + FINAL_OBSERVATION_DELAY, 73)]
    );
    assert_eq!(io.delivered.last(), Some(&oracle_click(location("1|1"))));
    assert_eq!(io.delivered.len(), 73);
}


/// Missing final pixels stop at ACK plus two seconds without input or capture replay.
#[test]
fn unconfirmed_final_delivery_returns_typed_uncertainty_after_two_seconds() {
    let plan = accepted_plan();
    let mut io = FakeIo::new(&plan);
    io.final_misses = usize::MAX;
    let result = drive(&plan, &mut io).expect("delivery is known, final effect is uncertain");
    let ack = io.counts.last_acknowledged_at.unwrap();
    assert_eq!(result.completion, BoardCompletion::DeliveredUnconfirmed);
    assert!(result.message.contains("unconfirmed"));
    assert!(!result.message.contains("BudgetExceeded"));
    assert_eq!(io.elapsed, ack + FINAL_CONFIRMATION_WINDOW);
    assert_eq!(io.final_observations.len(), 9);
    assert!(
        io.final_observations
            .iter()
            .all(|(started, count)| { *started < ack + FINAL_CONFIRMATION_WINDOW && *count == 73 })
    );
    assert_eq!(io.delivered.len(), 73);
    assert_eq!(io.committed.len(), 48);
}


/// Time after ACK is already part of the budget; entering the window cannot reset it.
#[test]
fn final_confirmation_deadline_is_anchored_to_ack_not_later_controller_work() {
    let plan = accepted_plan();
    let mut io = FakeIo::new(&plan);
    io.final_ack_postprocessing = Duration::from_millis(2_500);
    let result = drive(&plan, &mut io).expect("final ACK already exhausted its window");
    assert_eq!(result.completion, BoardCompletion::DeliveredUnconfirmed);
    assert!(io.final_observations.is_empty());
    assert_eq!(io.delivered.len(), 73);
    assert_eq!(
        io.elapsed,
        io.counts.last_acknowledged_at.unwrap() + Duration::from_millis(2_500)
    );
}


/// A capture begun within the window may finish later; it never permits another input.
#[test]
fn final_in_flight_capture_may_finish_late_without_starting_another_capture() {
    let plan = accepted_plan();
    for missing in [false, true] {
        let mut io = FakeIo::new(&plan);
        io.final_capture_delay = Duration::from_secs(3);
        io.final_misses = if missing { usize::MAX } else { 0 };
        let result = drive(&plan, &mut io).expect("bounded capture start, cooperative finish");
        assert_eq!(
            result.completion,
            if missing {
                BoardCompletion::DeliveredUnconfirmed
            } else {
                BoardCompletion::BoardClear
            }
        );
        let ack = io.counts.last_acknowledged_at.unwrap();
        assert_eq!(
            io.final_observations,
            vec![(ack + FINAL_OBSERVATION_DELAY, 73)]
        );
        assert_eq!(
            io.elapsed,
            ack + FINAL_OBSERVATION_DELAY + Duration::from_secs(3)
        );
        assert_eq!(io.delivered.len(), 73);
    }
}


/// STOP and context revocation outrank otherwise positive final pixels, with no cleanup.
#[test]
fn final_stop_or_context_change_cannot_commit_or_continue_after_delivery() {
    let plan = accepted_plan();
    for mode in 0..3 {
        let mut io = FakeIo::new(&plan);
        match mode {
            0 => io.stop_after_ack = Some(73),
            1 => io.stop_during_final_observation = true,
            _ => io.revoke_during_final_observation = true,
        }
        let error = drive(&plan, &mut io).expect_err("revoked final observation cannot succeed");
        assert!(error.contains(match mode {
            0 => "Aborted",
            1 => "STOP",
            _ => "context revoked",
        }));
        assert_eq!(io.delivered.len(), 73);
        assert_eq!(io.committed.len(), 48);
        assert_eq!(io.final_observations.len(), usize::from(mode != 0));
    }
}


/// A regressing final clock is an explicit refusal and cannot authorise a capture.
#[test]
fn final_clock_regression_refuses_before_a_capture_without_replaying_input() {
    let plan = accepted_plan();
    let mut io = FakeIo::new(&plan);
    io.reverse_final_clock = true;
    let error = drive(&plan, &mut io).expect_err("clock regression");
    assert!(error.contains("clock did not advance"));
    assert!(io.final_observations.is_empty());
    assert_eq!(io.delivered.len(), 73);
}


/// A partial last MATCH cannot enter the final window or repeat an uncertain gesture.
#[test]
fn final_pair_requires_both_complete_acknowledgements_before_observation() {
    let plan = accepted_plan();
    for attempt in [72, 73] {
        let mut io = FakeIo::new(&plan);
        io.uncertain_attempt = Some(attempt);
        assert!(drive(&plan, &mut io).is_err());
        assert_eq!(io.counts.attempted, attempt);
        assert_eq!(io.counts.acknowledged, attempt - 1);
        assert_eq!(io.delivered.len(), attempt);
        assert!(io.final_observations.is_empty());
        assert_eq!(io.committed.len(), 48);
    }
    let mut io = FakeIo::new(&plan);
    io.stop_after_ack = Some(72);
    assert!(drive(&plan, &mut io).is_err());
    assert_eq!(io.counts.acknowledged, 72);
    assert_eq!(io.delivered.len(), 72);
    assert!(io.final_observations.is_empty());
}


/// Only an independently replayed final tableau removal may enable terminal proof.
#[test]
fn incomplete_route_is_refused_before_any_capture_or_input() {
    let mut plan = accepted_plan();
    plan.trace = plan.trace[..48].to_vec().into_boxed_slice();
    let mut io = FakeIo::new(&plan);
    let error = drive(&plan, &mut io).expect_err("apex still present");
    assert!(error.contains("match the replayed objective"));
    assert!(io.observations.is_empty());
    assert!(io.delivered.is_empty());
}


/// Decode retained original pixels without acquiring any new guest observation.
fn native_completion_fixture(relative: &str) -> CapturedFrame {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .expect("retained original completion fixture");
    crate::capture::decode_png(&bytes).expect("retained native PNG")
}


/// New-board proof requires two consecutive equal actual full-board observations.
#[test]
fn final_new_board_proof_resets_for_another_deal_or_unknown_native_pixels() {
    let vision = ReconVision::bundled_native_v11().unwrap();
    let first = native_completion_fixture("tests/fixtures/pyramid-recon/native-v9/baseline.png");
    let other = native_completion_fixture("tests/fixtures/pyramid-recon/native-v7/baseline.png");
    let mut unknown = first.clone();
    unknown.pixels.fill(0);
    let mut proof = FinalConfirmation::default();
    assert_eq!(proof.observe(&vision, &first, false), Ok(None));
    assert_eq!(proof.observe(&vision, &other, false), Ok(None));
    assert_eq!(proof.observe(&vision, &first, false), Ok(None));
    assert_eq!(proof.observe(&vision, &unknown, false), Ok(None));
    assert_eq!(proof.observe(&vision, &first, false), Ok(None));
    assert_eq!(
        proof.observe(&vision, &first, false),
        Ok(Some(BoardCompletion::NextBoardReady))
    );
    // Independent clear proof wins, and resets any earlier redeal candidate.
    assert_eq!(
        proof.observe(&vision, &first, true),
        Ok(Some(BoardCompletion::BoardClear))
    );
    assert_eq!(proof.observe(&vision, &first, false), Ok(None));
}


/// Two specific terminal frames are required; board or unknown pixels reset proof.
#[test]
fn final_terminal_proof_requires_two_consecutive_specific_native_scenes() {
    let vision = ReconVision::bundled_native_v11().unwrap();
    let terminal =
        native_completion_fixture("tests/fixtures/ok-confirmation-20261016/pyramid/clean.png");
    let board = native_completion_fixture("tests/fixtures/pyramid-recon/native-v9/baseline.png");
    let mut unknown = terminal.clone();
    unknown.pixels.fill(255);
    let mut proof = FinalConfirmation::default();
    assert_eq!(proof.observe(&vision, &terminal, false), Ok(None));
    assert_eq!(proof.observe(&vision, &unknown, false), Ok(None));
    assert_eq!(proof.observe(&vision, &terminal, false), Ok(None));
    assert_eq!(proof.observe(&vision, &board, false), Ok(None));
    assert_eq!(proof.observe(&vision, &terminal, false), Ok(None));
    assert_eq!(
        proof.observe(&vision, &terminal, false),
        Ok(Some(BoardCompletion::GameComplete))
    );
    assert_eq!(proof.observe(&vision, &board, false), Ok(None));
}


/// An owned execution failure consumes authority but never releases its session reservation.
#[test]
fn owned_execution_failure_does_not_finish_or_release_the_existing_reservation() {
    use std::sync::{Mutex, mpsc};
    let plan = accepted_plan();
    let context = plan.acquisition.context.clone();
    let cancel = AtomicBool::new(false);
    let (tx, rx) = mpsc::channel();
    let events = WorkerEventSink {
        events: tx,
        latest_frame: super::super::LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(context.clone())),
    };
    let control = &events.latest_frame.reconnaissance;
    control.select(&context, &cancel).unwrap();
    control
        .queue_reconnaissance(&context, &cancel, || Ok(()))
        .unwrap();
    events.latest_frame.computed_route.lock().unwrap().prepared = Some(plan);
    let result = execute_owned(&context, 991, 17, false, 7, &events, &cancel);
    assert!(
        result
            .expect_err("consent mismatch before connection")
            .contains("consent")
    );
    assert!(control.check_active(&context, &cancel).is_ok());
    let store = events.latest_frame.computed_route.lock().unwrap();
    assert!(store.prepared.is_none());
    assert!(store.acquisition.is_none());
    assert!(
        !rx.try_iter()
            .any(|envelope| matches!(envelope.event, WorkerEvent::ComputedRouteFinished { .. }))
    );
}
