//! Deterministic orchestration checks; synthetic facts do not calibrate native pixels.

use super::*;
use std::collections::VecDeque;


/// Target-agnostic continuation acquires again without normal identity or transition input.
#[test]
fn no_challenge_handoffs_reacquire_until_stop_without_classifying_targets() {
    let mut io = Fake::new(
        unknown_board(),
        [
            BoardCompletion::AwaitingFreshBoard,
            BoardCompletion::StockRedealt,
        ],
    );
    io.stop_on_phase = Some((3, SessionPhase::Reconnaissance));
    assert!(drive_no_challenge(&mut io).is_err());
    assert_eq!(io.acquisitions(), [1, 2]);
    assert!(!io.calls.contains(&Call::Transition));
    assert!(!io.calls.contains(&Call::Restart));
    let mut uncertain = Fake::new(unknown_board(), [BoardCompletion::DeliveredUnconfirmed]);
    assert!(drive_no_challenge(&mut uncertain).is_err());
    assert_eq!(uncertain.acquisitions(), [1]);
}


/// A Boards REDEAL preserves its count; a real clear must advance it exactly once.
#[test]
fn boards_redeal_requires_unchanged_observed_counter() {
    use crate::pyramid_progress::ChallengeKind;
    for (next_count, reaches_second) in [(0, true), (1, false)] {
        let mut io = Fake::new(
            challenge_entry(ChallengeKind::Boards, 0, 2),
            [BoardCompletion::StockRedealt],
        );
        io.transitions
            .push_back(challenge_entry(ChallengeKind::Boards, next_count, 2));
        io.stop_on_phase = Some((2, SessionPhase::Reconnaissance));
        assert!(challenge::drive(&mut io).is_err());
        assert_eq!(
            io.calls
                .contains(&Call::Phase(2, SessionPhase::Reconnaissance)),
            reaches_second
        );
    }
}


mod challenges;


mod target_completion_tests;




/// Read genuine finite native completion evidence for controller tests, without guest I/O.
fn fours_completion() -> ChallengeCompletion {
    let frame = crate::capture::decode_png(include_bytes!(
        "../../../tests/fixtures/pyramid-objectives/fours-complete.png"
    ))
    .unwrap();
    crate::pyramid_progress::observe_challenge_complete(&frame)
        .unwrap()
        .unwrap()
}


/// Synthetic actual requirement keeps counters separate from intended tableau removals.
fn challenge_entry(
    kind: crate::pyramid_progress::ChallengeKind,
    current: u32,
    target: u32,
) -> Entry {
    Entry::Challenge(ChallengeProgress::new(kind, current, target).unwrap())
}


/// Actual rank progress can span several board clears; only its own terminal stops successfully.
#[test]
fn challenge_rank_progress_across_boards_stops_without_new_game() {
    let kind = crate::pyramid_progress::ChallengeKind::Cards;
    let mut io = Fake::new(
        challenge_entry(kind, 4, 12),
        [
            BoardCompletion::NextBoardReady,
            BoardCompletion::BoardClear,
            BoardCompletion::ChallengeTargetVerified(fours_completion()),
        ],
    );
    io.transitions = [challenge_entry(kind, 4, 12), challenge_entry(kind, 8, 12)]
        .into_iter()
        .collect();
    let result = challenge::drive(&mut io).unwrap();
    assert!(result.contains("Challenge target verified"));
    assert_eq!(io.acquisitions(), [1, 2, 3]);
    assert!(!io.calls.contains(&Call::Restart));
    assert_eq!(
        io.calls.iter().filter(|c| **c == Call::Transition).count(),
        2
    );
}


/// A reached actual terminal at entry requires zero acquisition or execution calls.
#[test]
fn challenge_complete_entry_finishes_rewards_without_card_input_or_restart() {
    let mut io = Fake::new(Entry::ChallengeComplete(fours_completion()), []);
    assert!(
        challenge::drive(&mut io)
            .unwrap()
            .contains("Challenge target verified")
    );
    assert_eq!(
        io.operations(),
        [Call::Entry(WorkflowMode::Continuous), Call::FinishChallenge]
    );
}


/// Changed kind, target, regressed count and normal scenes all revoke continuation.
#[test]
fn challenge_transition_never_changes_its_requirement_or_infers_missing_progress() {
    use crate::pyramid_progress::ChallengeKind;
    let kind = ChallengeKind::Cards;
    for next in [
        challenge_entry(kind, 3, 12),
        challenge_entry(kind, 5, 13),
        challenge_entry(ChallengeKind::Score, 5, 12),
        unknown_board(),
        standard_board(0),
        standard_terminal(),
    ] {
        let mut io = Fake::new(challenge_entry(kind, 4, 12), [BoardCompletion::BoardClear]);
        io.transitions.push_back(next);
        assert!(challenge::drive(&mut io).is_err());
        assert_eq!(io.acquisitions(), [1]);
        assert!(!io.calls.contains(&Call::Restart));
    }
}


/// Boards increase exactly once per clear, while actual Score may advance by varying amounts.
#[test]
fn challenge_transition_uses_kind_specific_observed_counts() {
    use crate::pyramid_progress::ChallengeKind;
    for (kind, before, after, target, accepted) in [
        (ChallengeKind::Boards, 0, 1, 2, true),
        (ChallengeKind::Boards, 0, 0, 2, false),
        (ChallengeKind::Boards, 0, 2, 3, false),
        (ChallengeKind::Score, 1280, 2560, 4200, true),
    ] {
        let mut io = Fake::new(
            challenge_entry(kind, before, target),
            [BoardCompletion::BoardClear],
        );
        io.transitions
            .push_back(challenge_entry(kind, after, target));
        io.fail_on = Some(Call::Acquire(2));
        let error = challenge::drive(&mut io).unwrap_err();
        assert_eq!(io.acquisitions().contains(&2), accepted, "{error}");
        assert!(!io.calls.contains(&Call::Restart));
    }
}


/// Neither partial benefit nor acknowledged uncertainty grants a board transition.
#[test]
fn challenge_partial_or_uncertain_endpoint_stops_before_transition() {
    for completion in [
        BoardCompletion::PartialPositionVerified {
            removed: 12,
            remaining: 16,
        },
        BoardCompletion::DeliveredUnconfirmed,
    ] {
        let mut io = Fake::new(
            challenge_entry(crate::pyramid_progress::ChallengeKind::Score, 1280, 3000),
            [completion],
        );
        let result = challenge::drive(&mut io);
        assert_eq!(
            result.is_ok(),
            matches!(completion, BoardCompletion::PartialPositionVerified { .. })
        );
        assert!(!io.calls.contains(&Call::Transition));
        assert!(!io.calls.contains(&Call::Restart));
    }
}


/// STOP after terminal observation wins, and another objective kind's terminal cannot finish this goal.
#[test]
fn challenge_terminal_requires_bound_kind_and_unrevoked_authority() {
    use crate::pyramid_progress::ChallengeKind;
    let proof = fours_completion();
    for (kind, stop) in [(ChallengeKind::Cards, true), (ChallengeKind::Score, false)] {
        let mut io = Fake::new(
            challenge_entry(kind, 8, 12),
            [BoardCompletion::ChallengeTargetVerified(proof)],
        );
        if stop {
            io.stop_after = Some((Call::Execute(1), 1));
        }
        assert!(challenge::drive(&mut io).is_err());
        assert!(!io.calls.contains(&Call::Transition));
        assert!(!io.calls.contains(&Call::Restart));
    }
}


/// Supply explicit synthetic standard-game proof separately from the observed fraction.
fn standard_board(completed: u8) -> Entry {
    Entry::Board {
        progress: Some(BoardCount {
            completed,
            target: 3,
        }),
        standard_three: true,
    }
}


/// Represent full-board readiness whose native header and game identity remain unknown.
fn unknown_board() -> Entry {
    Entry::Board {
        progress: None,
        standard_three: false,
    }
}


/// Supply explicit synthetic supported terminal proof, never native calibration.
fn standard_terminal() -> Entry {
    Entry::Terminal {
        supported_pyramid: true,
    }
}


/// One observable fake operation; no variant performs capture, filesystem or guest I/O.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Call {
    /// Shared GAME WIN rewards finish without any NewGame/restart operation.
    FinishChallenge,
    /// Read the supplied synthetic entry fact under the selected lifetime.
    Entry(WorkflowMode),
    /// Publish an advisory phase for the given board generation.
    Phase(u64, SessionPhase),
    /// Acquire and restore one synthetic board.
    Acquire(u64),
    /// Calculate one synthetic board's route.
    Plan(u64),
    /// Consume one configured typed execution outcome.
    Execute(u64),
    /// Consume fresh observed scene facts after a positive execution outcome.
    Transition,
    /// Stand in for the guarded terminal policy, without sending input.
    Restart,
}


/// Finite fake of the actual driver's dependency boundary, including a monotonic STOP latch.
struct Fake {
    /// Actual counter supplied by execution for cross-board high-water regressions.
    observed_challenge: Option<ChallengeProgress>,
    /// Explicit synthetic starting fact; unknown progress remains unknown.
    entry: Entry,
    /// Actual invocation order, including phases but excluding pure STOP checks.
    calls: Vec<Call>,
    /// Typed execution outcomes consumed in board order.
    completions: VecDeque<BoardCompletion>,
    /// Fresh observed scenes, independent of execution completion classifications.
    transitions: VecDeque<Entry>,
    /// Explicit observed postrestart facts; there is no default successful zero.
    restarts: VecDeque<Entry>,
    /// Optional failing operation, before any later phase is reached.
    fail_on: Option<Call>,
    /// Optional phase callback that latches STOP before its corresponding operation.
    stop_on_phase: Option<(u64, SessionPhase)>,
    /// Optional operation occurrence that latches STOP immediately after completion.
    stop_after: Option<(Call, usize)>,
    /// Synthetic STOP latch, never reset by the fake or the session driver.
    stopped: bool,
}


impl Fake {
    /// Construct a finite pipeline; exhausted outcome queues return errors rather than loop.
    fn new(entry: Entry, completions: impl IntoIterator<Item = BoardCompletion>) -> Self {
        Self {
            observed_challenge: None,
            entry,
            calls: Vec::new(),
            completions: completions.into_iter().collect(),
            transitions: VecDeque::new(),
            restarts: VecDeque::new(),
            fail_on: None,
            stop_on_phase: None,
            stop_after: None,
            stopped: false,
        }
    }


    /// Record a real operation before injecting its configured failure or STOP.
    ///
    /// Deliberately does not check STOP itself: these tests verify that the actual
    /// driver checks phase boundaries before invoking another dependency operation.
    fn record(&mut self, call: Call) -> Result<(), String> {
        self.calls.push(call);
        if self.fail_on == Some(call) {
            return Err(format!("injected failure at {call:?}"));
        }
        if let Some((target, occurrence)) = self.stop_after
            && call == target
            && self.calls.iter().filter(|event| **event == target).count() == occurrence
        {
            self.stopped = true;
        }
        Ok(())
    }


    /// Return only dependency operations so expected authority progression is explicit.
    fn operations(&self) -> Vec<Call> {
        self.calls
            .iter()
            .copied()
            .filter(|call| !matches!(call, Call::Phase(..)))
            .collect()
    }


    /// Return the board generations actually acquired, preserving invocation order.
    fn acquisitions(&self) -> Vec<u64> {
        self.calls
            .iter()
            .filter_map(|call| match call {
                Call::Acquire(generation) => Some(*generation),
                _ => None,
            })
            .collect()
    }
}


impl Pipeline for Fake {
    fn challenge_progress(&self) -> Option<ChallengeProgress> {
        self.observed_challenge
    }


    fn finish_challenge(
        &mut self,
        proof: ChallengeCompletion,
    ) -> Result<ChallengeCompletion, String> {
        self.record(Call::FinishChallenge)?;
        Ok(proof)
    }


    /// Refuse the synthetic STOP latch without modifying it or recording an operation.
    fn check(&self) -> Result<(), String> {
        if self.stopped {
            Err("synthetic STOP".to_owned())
        } else {
            Ok(())
        }
    }


    /// Record the exact phase and optionally latch STOP before the next operation.
    fn phase(&mut self, generation: u64, phase: SessionPhase) {
        self.calls.push(Call::Phase(generation, phase));
        if self.stop_on_phase == Some((generation, phase)) {
            self.stopped = true;
        }
    }


    /// Return a supplied fact; no native series-progress evidence is implied.
    fn entry(&mut self, mode: WorkflowMode) -> Result<Entry, String> {
        self.record(Call::Entry(mode))?;
        Ok(self.entry)
    }


    /// Record acquisition without constructing an actual deal or receipt.
    fn acquire(&mut self, generation: u64) -> Result<(), String> {
        self.record(Call::Acquire(generation))
    }


    /// Record calculation without running a solver or allocating search state.
    fn plan(&mut self, generation: u64) -> Result<(), String> {
        self.record(Call::Plan(generation))
    }


    /// Return the next synthetic effect outcome without delivering any input.
    fn execute(&mut self, generation: u64) -> Result<BoardCompletion, String> {
        self.record(Call::Execute(generation))?;
        self.completions
            .pop_front()
            .ok_or_else(|| "unconfigured fake execution".to_owned())
    }


    /// Return the next configured transition without capturing new observations.
    fn transition(&mut self) -> Result<Entry, String> {
        self.record(Call::Transition)?;
        self.transitions
            .pop_front()
            .ok_or_else(|| "unconfigured fake transition".to_owned())
    }


    /// Record terminal-policy entry, optionally latching STOP after its return.
    fn restart(&mut self) -> Result<Entry, String> {
        self.record(Call::Restart)?;
        self.restarts
            .pop_front()
            .ok_or_else(|| "unconfigured fake restart observation".to_owned())
    }
}


/// Manual dispatch is refused before even an entry probe or advisory phase.
#[test]
fn manual_mode_never_calls_the_automatic_pipeline() {
    let mut io = Fake::new(
        Entry::Terminal {
            supported_pyramid: true,
        },
        [],
    );
    let error = drive(WorkflowMode::Manual, &mut io).expect_err("manual needs separate buttons");
    assert!(error.contains("three separate buttons"));
    assert!(io.calls.is_empty());
}


/// Every positive single-board outcome stops after its three phases without transition/restart.
#[test]
fn single_board_stops_after_exactly_one_acquisition_plan_and_execution() {
    for completion in [
        BoardCompletion::BoardClear,
        BoardCompletion::NextBoardReady,
        BoardCompletion::GameComplete,
    ] {
        let mut io = Fake::new(unknown_board(), [completion]);
        let report =
            drive(WorkflowMode::SingleBoard, &mut io).expect("positive single-board effect");
        assert!(report.contains("Single board complete"));
        assert_eq!(
            io.calls,
            vec![
                Call::Phase(0, SessionPhase::Entry),
                Call::Entry(WorkflowMode::SingleBoard),
                Call::Phase(1, SessionPhase::Reconnaissance),
                Call::Acquire(1),
                Call::Phase(1, SessionPhase::Planning),
                Call::Plan(1),
                Call::Phase(1, SessionPhase::Execution),
                Call::Execute(1),
            ]
        );
    }
}


/// Complete delivery without positive effect proof never authorises transition or restart.
#[test]
fn delivered_unconfirmed_stops_both_automatic_modes_without_continuation() {
    for mode in [WorkflowMode::SingleBoard, WorkflowMode::Continuous] {
        let mut io = Fake::new(standard_board(2), [BoardCompletion::DeliveredUnconfirmed]);
        let error = drive(mode, &mut io).expect_err("acknowledgements are not completion");
        assert!(error.contains("completion unconfirmed"));
        assert_eq!(
            io.operations(),
            vec![
                Call::Entry(mode),
                Call::Acquire(1),
                Call::Plan(1),
                Call::Execute(1)
            ]
        );
    }
}


/// Partial success releases both automatic modes before any transition or restart call.
#[test]
fn partial_removal_stops_both_automatic_modes_even_on_the_third_board() {
    for mode in [WorkflowMode::SingleBoard, WorkflowMode::Continuous] {
        for completed in 0..3 {
            let mut io = Fake::new(
                standard_board(completed),
                [BoardCompletion::PartialPositionVerified {
                    removed: 18,
                    remaining: 10,
                }],
            );
            let report = drive(mode, &mut io).expect("verified partial endpoint");
            assert!(report.contains("18/28 tableau cards removed, 10 remain"));
            assert_eq!(
                io.operations(),
                vec![
                    Call::Entry(mode),
                    Call::Acquire(1),
                    Call::Plan(1),
                    Call::Execute(1)
                ]
            );
        }
    }
}


/// Failure in any board phase preserves the first error and never advances to later work.
#[test]
fn acquisition_planning_and_execution_failures_never_advance() {
    for (failure, tail) in [
        (Call::Acquire(1), vec![Call::Acquire(1)]),
        (Call::Plan(1), vec![Call::Acquire(1), Call::Plan(1)]),
        (
            Call::Execute(1),
            vec![Call::Acquire(1), Call::Plan(1), Call::Execute(1)],
        ),
    ] {
        for mode in [WorkflowMode::SingleBoard, WorkflowMode::Continuous] {
            let mut io = Fake::new(standard_board(0), [BoardCompletion::BoardClear]);
            io.fail_on = Some(failure);
            let error = drive(mode, &mut io).expect_err("injected phase failure");
            assert_eq!(error, format!("injected failure at {failure:?}"));
            let mut expected = vec![Call::Entry(mode)];
            expected.extend(tail.iter().copied());
            assert_eq!(io.operations(), expected);
        }
    }
}


/// A STOP already latched at entry prevents every operation, including entry observation.
#[test]
fn initial_stop_performs_no_pipeline_calls() {
    let mut io = Fake::new(standard_board(0), []);
    io.stopped = true;
    assert_eq!(
        drive(WorkflowMode::Continuous, &mut io),
        Err("synthetic STOP".to_owned())
    );
    assert!(io.calls.is_empty());
}


/// STOP delivered by a phase callback prevents that phase's dependency operation.
#[test]
fn stop_at_every_published_phase_boundary_never_calls_the_next_operation() {
    for (mode, entry, completion, stop, expected) in [
        (
            WorkflowMode::SingleBoard,
            unknown_board(),
            BoardCompletion::BoardClear,
            (0, SessionPhase::Entry),
            vec![],
        ),
        (
            WorkflowMode::SingleBoard,
            unknown_board(),
            BoardCompletion::BoardClear,
            (1, SessionPhase::Reconnaissance),
            vec![Call::Entry(WorkflowMode::SingleBoard)],
        ),
        (
            WorkflowMode::SingleBoard,
            unknown_board(),
            BoardCompletion::BoardClear,
            (1, SessionPhase::Planning),
            vec![Call::Entry(WorkflowMode::SingleBoard), Call::Acquire(1)],
        ),
        (
            WorkflowMode::SingleBoard,
            unknown_board(),
            BoardCompletion::BoardClear,
            (1, SessionPhase::Execution),
            vec![
                Call::Entry(WorkflowMode::SingleBoard),
                Call::Acquire(1),
                Call::Plan(1),
            ],
        ),
        (
            WorkflowMode::Continuous,
            standard_board(0),
            BoardCompletion::BoardClear,
            (1, SessionPhase::Transition),
            vec![
                Call::Entry(WorkflowMode::Continuous),
                Call::Acquire(1),
                Call::Plan(1),
                Call::Execute(1),
            ],
        ),
        (
            WorkflowMode::Continuous,
            standard_board(2),
            BoardCompletion::GameComplete,
            (1, SessionPhase::Restart),
            vec![
                Call::Entry(WorkflowMode::Continuous),
                Call::Acquire(1),
                Call::Plan(1),
                Call::Execute(1),
                Call::Transition,
            ],
        ),
        (
            WorkflowMode::Continuous,
            Entry::Terminal {
                supported_pyramid: true,
            },
            BoardCompletion::GameComplete,
            (0, SessionPhase::Restart),
            vec![Call::Entry(WorkflowMode::Continuous)],
        ),
    ] {
        let mut io = Fake::new(entry, [completion]);
        io.transitions.push_back(standard_terminal());
        io.stop_on_phase = Some(stop);
        let error = drive(mode, &mut io).expect_err("phase callback latched STOP");
        assert_eq!(error, "synthetic STOP");
        assert_eq!(io.operations(), expected, "STOP at {stop:?}");
    }
}


/// STOP latched by completed work is checked before another phase or operation begins.
#[test]
fn stop_after_each_board_operation_never_advances() {
    for (stop, expected) in [
        (Call::Acquire(1), vec![Call::Acquire(1)]),
        (Call::Plan(1), vec![Call::Acquire(1), Call::Plan(1)]),
        (
            Call::Execute(1),
            vec![Call::Acquire(1), Call::Plan(1), Call::Execute(1)],
        ),
        (
            Call::Transition,
            vec![
                Call::Acquire(1),
                Call::Plan(1),
                Call::Execute(1),
                Call::Transition,
            ],
        ),
    ] {
        let mut io = Fake::new(standard_board(2), [BoardCompletion::BoardClear]);
        io.transitions.push_back(standard_terminal());
        io.stop_after = Some((stop, 1));
        assert_eq!(
            drive(WorkflowMode::Continuous, &mut io),
            Err("synthetic STOP".to_owned())
        );
        let mut operations = vec![Call::Entry(WorkflowMode::Continuous)];
        operations.extend(expected);
        assert_eq!(io.operations(), operations);
    }
}


/// Synthetic positive entry counts finish exactly the remaining standard-game boards.
#[test]
fn continuous_known_zero_one_or_two_runs_only_remaining_boards_before_restart() {
    for cleared in 0..BOARDS_PER_GAME {
        let remaining = u64::from(BOARDS_PER_GAME - cleared);
        let outcomes = (1..remaining)
            .map(|_| BoardCompletion::NextBoardReady)
            .chain(std::iter::once(BoardCompletion::GameComplete));
        let mut io = Fake::new(standard_board(cleared), outcomes);
        io.transitions.extend(
            (cleared + 1..BOARDS_PER_GAME)
                .map(standard_board)
                .chain(std::iter::once(standard_terminal())),
        );
        io.restarts.push_back(standard_board(0));
        io.stop_after = Some((Call::Restart, 1));
        assert_eq!(
            drive(WorkflowMode::Continuous, &mut io),
            Err("synthetic STOP".to_owned())
        );
        assert_eq!(io.acquisitions(), (1..=remaining).collect::<Vec<_>>());
        let mut expected = vec![Call::Entry(WorkflowMode::Continuous)];
        for generation in 1..=remaining {
            expected.extend([
                Call::Acquire(generation),
                Call::Plan(generation),
                Call::Execute(generation),
                Call::Transition,
            ]);
        }
        expected.push(Call::Restart);
        assert_eq!(io.operations(), expected);
    }
}


/// Verified terminal entry restarts first, then completes three boards before another restart.
#[test]
fn terminal_entry_restarts_before_any_acquisition_and_stops_after_the_next_game() {
    let mut io = Fake::new(
        Entry::Terminal {
            supported_pyramid: true,
        },
        [
            BoardCompletion::NextBoardReady,
            BoardCompletion::NextBoardReady,
            BoardCompletion::GameComplete,
        ],
    );
    io.transitions
        .extend([standard_board(1), standard_board(2), standard_terminal()]);
    io.restarts.extend([standard_board(0), standard_board(0)]);
    io.stop_after = Some((Call::Restart, 2));
    assert_eq!(
        drive(WorkflowMode::Continuous, &mut io),
        Err("synthetic STOP".to_owned())
    );
    let mut expected = vec![Call::Entry(WorkflowMode::Continuous), Call::Restart];
    for generation in 1..=3 {
        expected.extend([
            Call::Acquire(generation),
            Call::Plan(generation),
            Call::Execute(generation),
            Call::Transition,
        ]);
    }
    expected.push(Call::Restart);
    assert_eq!(io.operations(), expected);
    assert_eq!(io.acquisitions(), vec![1, 2, 3]);
}


/// Board identities remain monotonic when a partially completed game rolls into a fresh game.
#[test]
fn automatic_board_generations_do_not_reset_when_the_series_restarts() {
    let mut io = Fake::new(
        standard_board(2),
        [
            BoardCompletion::GameComplete,
            BoardCompletion::NextBoardReady,
            BoardCompletion::NextBoardReady,
            BoardCompletion::GameComplete,
        ],
    );
    io.transitions.extend([
        standard_terminal(),
        standard_board(1),
        standard_board(2),
        standard_terminal(),
    ]);
    io.restarts.extend([standard_board(0), standard_board(0)]);
    io.stop_after = Some((Call::Restart, 2));
    assert_eq!(
        drive(WorkflowMode::Continuous, &mut io),
        Err("synthetic STOP".to_owned())
    );
    assert_eq!(io.acquisitions(), vec![1, 2, 3, 4]);
    assert_eq!(
        io.calls
            .iter()
            .filter(|call| **call == Call::Restart)
            .count(),
        2
    );
}


/// Positive clear results still need fresh actual progress before continuation or restart.
#[test]
fn positive_board_clear_transitions_are_verified_before_continuing_or_restarting() {
    let mut io = Fake::new(standard_board(0), [BoardCompletion::BoardClear; 3]);
    io.transitions
        .extend([standard_board(1), standard_board(2), standard_terminal()]);
    io.restarts.push_back(standard_board(0));
    io.stop_after = Some((Call::Restart, 1));
    assert_eq!(
        drive(WorkflowMode::Continuous, &mut io),
        Err("synthetic STOP".to_owned())
    );
    let mut expected = vec![Call::Entry(WorkflowMode::Continuous)];
    for generation in 1..=3 {
        expected.extend([
            Call::Acquire(generation),
            Call::Plan(generation),
            Call::Execute(generation),
            Call::Transition,
        ]);
    }
    expected.push(Call::Restart);
    assert_eq!(io.operations(), expected);
}


/// Fresh terminal identity, target timing and actual next-board evidence govern continuation.
#[test]
fn incompatible_series_outcomes_stop_without_more_acquisition_or_restart() {
    for (cleared, completion, transition, message) in [
        (
            0,
            BoardCompletion::GameComplete,
            standard_terminal(),
            "before the supported three-board target",
        ),
        (
            2,
            BoardCompletion::NextBoardReady,
            standard_board(0),
            "after the expected third clear",
        ),
        (
            0,
            BoardCompletion::BoardClear,
            unknown_board(),
            "cannot establish standard-game identity",
        ),
        (
            0,
            BoardCompletion::BoardClear,
            Entry::Terminal {
                supported_pyramid: false,
            },
            "Terminal lacks supported",
        ),
    ] {
        let mut io = Fake::new(standard_board(cleared), [completion]);
        io.transitions.push_back(transition);
        let error =
            drive(WorkflowMode::Continuous, &mut io).expect_err("unsupported series outcome");
        assert!(error.contains(message), "{error}");
        let expected = vec![
            Call::Entry(WorkflowMode::Continuous),
            Call::Acquire(1),
            Call::Plan(1),
            Call::Execute(1),
            Call::Transition,
        ];
        assert_eq!(io.operations(), expected);
    }
}


/// Unknown/invalid counts never become zero, and single-board mode cannot restart a terminal.
#[test]
fn unsupported_entry_facts_refuse_before_acquisition() {
    for (mode, entry) in [
        (WorkflowMode::Continuous, unknown_board()),
        (WorkflowMode::Continuous, standard_board(3)),
        (
            WorkflowMode::SingleBoard,
            Entry::Terminal {
                supported_pyramid: true,
            },
        ),
    ] {
        let mut io = Fake::new(entry, []);
        let error = drive(mode, &mut io).expect_err("entry facts grant no acquisition authority");
        if mode == WorkflowMode::Continuous {
            assert!(error.contains("observed 0/3, 1/3 or 2/3"));
        } else {
            assert!(error.contains("untouched gameplay board"));
        }
        assert_eq!(io.operations(), vec![Call::Entry(mode)]);
    }
}


/// Unsupported terminal facts grant no restart or acquisition.
#[test]
fn unsupported_terminal_refuses_before_restart_or_acquisition() {
    let mut io = Fake::new(
        Entry::Terminal {
            supported_pyramid: false,
        },
        [],
    );
    let error =
        drive(WorkflowMode::Continuous, &mut io).expect_err("terminal alone is insufficient");
    assert!(
        error.contains("lacks supported Pyramid game-end evidence"),
        "{error}"
    );
    assert_eq!(io.operations(), vec![Call::Entry(WorkflowMode::Continuous)]);
}


/// A displayed three-board target is still unsupported without independent standard-mode proof.
#[test]
fn nonstandard_three_board_headers_never_authorise_acquisition() {
    for completed in 0..3 {
        let entry = Entry::Board {
            progress: Some(BoardCount {
                completed,
                target: 3,
            }),
            standard_three: false,
        };
        let mut io = Fake::new(entry, []);
        let error =
            drive(WorkflowMode::Continuous, &mut io).expect_err("fraction is not game identity");
        assert!(error.contains("cannot establish standard-game identity"));
        assert_eq!(io.operations(), [Call::Entry(WorkflowMode::Continuous)]);
    }
}


/// Stable, backward, skipped and changed-target values cannot substitute for the next observed clear.
#[test]
fn stale_regressed_skipped_changed_target_and_unknown_progress_stop_continuation() {
    for (previous, observed) in [
        (0, standard_board(0)),
        (1, standard_board(1)),
        (1, standard_board(0)),
        (0, standard_board(2)),
        (0, standard_board(3)),
        (
            0,
            Entry::Board {
                progress: Some(BoardCount {
                    completed: 1,
                    target: 4,
                }),
                standard_three: true,
            },
        ),
        (
            0,
            Entry::Board {
                progress: None,
                standard_three: true,
            },
        ),
        (
            0,
            Entry::Board {
                progress: Some(BoardCount {
                    completed: 1,
                    target: 3,
                }),
                standard_three: false,
            },
        ),
    ] {
        let mut io = Fake::new(standard_board(previous), [BoardCompletion::NextBoardReady]);
        io.transitions.push_back(observed);
        let error = drive(WorkflowMode::Continuous, &mut io).expect_err("observed mismatch stops");
        assert!(
            error.contains("No continuation input")
                || error.contains("no acquisition or continuation"),
            "{error}"
        );
        assert_eq!(
            io.operations(),
            [
                Call::Entry(WorkflowMode::Continuous),
                Call::Acquire(1),
                Call::Plan(1),
                Call::Execute(1),
                Call::Transition
            ]
        );
    }
}


/// All positive effect classifications require an independent current scene and counter observation.
#[test]
fn every_positive_execution_outcome_requires_observed_transition() {
    for completion in [
        BoardCompletion::BoardClear,
        BoardCompletion::NextBoardReady,
        BoardCompletion::GameComplete,
    ] {
        let mut io = Fake::new(standard_board(0), [completion]);
        io.transitions.push_back(standard_board(1));
        io.stop_on_phase = Some((2, SessionPhase::Reconnaissance));
        assert_eq!(
            drive(WorkflowMode::Continuous, &mut io),
            Err("synthetic STOP".to_owned())
        );
        assert_eq!(
            io.operations(),
            [
                Call::Entry(WorkflowMode::Continuous),
                Call::Acquire(1),
                Call::Plan(1),
                Call::Execute(1),
                Call::Transition
            ]
        );
        assert!(
            io.calls
                .contains(&Call::Phase(2, SessionPhase::Reconnaissance))
        );
    }
}


/// A failure while observing the transition never falls back to the prior execution classification.
#[test]
fn transition_failure_preserves_error_without_acquisition_or_restart() {
    let mut io = Fake::new(standard_board(2), [BoardCompletion::GameComplete]);
    io.fail_on = Some(Call::Transition);
    let error = drive(WorkflowMode::Continuous, &mut io).expect_err("transition failure");
    assert_eq!(error, "injected failure at Transition");
    assert_eq!(
        io.operations(),
        [
            Call::Entry(WorkflowMode::Continuous),
            Call::Acquire(1),
            Call::Plan(1),
            Call::Execute(1),
            Call::Transition
        ]
    );
}


/// Recovered 1/3 or 2/3 gameplay and unsupported restart facts are never silently reset to zero.
#[test]
fn postrestart_requires_actual_supported_zero_before_any_next_acquisition() {
    for observed in [
        standard_board(1),
        standard_board(2),
        unknown_board(),
        standard_terminal(),
        Entry::Board {
            progress: Some(BoardCount {
                completed: 0,
                target: 4,
            }),
            standard_three: true,
        },
        Entry::Board {
            progress: Some(BoardCount {
                completed: 0,
                target: 3,
            }),
            standard_three: false,
        },
        Entry::Board {
            progress: None,
            standard_three: true,
        },
    ] {
        for terminal_entry in [false, true] {
            let entry = if terminal_entry {
                standard_terminal()
            } else {
                standard_board(2)
            };
            let mut io = Fake::new(entry, [BoardCompletion::GameComplete]);
            io.transitions.push_back(standard_terminal());
            io.restarts.push_back(observed);
            let error =
                drive(WorkflowMode::Continuous, &mut io).expect_err("no observed valid zero");
            assert!(
                error.contains("no acquisition") || error.contains("No acquisition"),
                "{error}"
            );
            let expected = if terminal_entry {
                vec![Call::Entry(WorkflowMode::Continuous), Call::Restart]
            } else {
                vec![
                    Call::Entry(WorkflowMode::Continuous),
                    Call::Acquire(1),
                    Call::Plan(1),
                    Call::Execute(1),
                    Call::Transition,
                    Call::Restart,
                ]
            };
            assert_eq!(io.operations(), expected);
        }
    }
}


/// STOP from entry or restart completion wins before any later phase, including output acquisition.
#[test]
fn stop_after_entry_and_restart_never_advances() {
    for entry in [standard_board(0), standard_terminal()] {
        let mut io = Fake::new(entry, []);
        io.stop_after = Some((Call::Entry(WorkflowMode::Continuous), 1));
        assert_eq!(
            drive(WorkflowMode::Continuous, &mut io),
            Err("synthetic STOP".to_owned())
        );
        assert_eq!(io.operations(), [Call::Entry(WorkflowMode::Continuous)]);
    }
    let mut io = Fake::new(standard_terminal(), []);
    io.restarts.push_back(standard_board(0));
    io.stop_after = Some((Call::Restart, 1));
    assert_eq!(
        drive(WorkflowMode::Continuous, &mut io),
        Err("synthetic STOP".to_owned())
    );
    assert_eq!(
        io.operations(),
        [Call::Entry(WorkflowMode::Continuous), Call::Restart]
    );
}


/// A failed terminal controller never authorises a later acquisition or a synthetic reset.
#[test]
fn restart_failure_preserves_error_and_consumes_no_new_board() {
    let mut io = Fake::new(standard_terminal(), []);
    io.fail_on = Some(Call::Restart);
    assert_eq!(
        drive(WorkflowMode::Continuous, &mut io),
        Err("injected failure at Restart".to_owned())
    );
    assert_eq!(
        io.operations(),
        [Call::Entry(WorkflowMode::Continuous), Call::Restart]
    );
}


/// Equal native positions and counters still carry no fabricated normal-game identity.
#[test]
fn native_pairing_requires_same_board_and_counter_without_promoting_three_to_standard() {
    use xsar::games::cards::{Card, Rank, Suit};

    let position = BoardObservation {
        tableau: std::array::from_fn(|index| {
            Card::new(Rank::ALL[index % 13], Suit::ALL[index / 13])
        }),
        left: StockFace::Card(Card::new(Rank::ALL[2], Suit::ALL[2])),
        right: StockFace::Empty,
        undo_all: None,
    };
    for completed in 0..3 {
        let count = BoardCount {
            completed,
            target: 3,
        };
        let first = NativeObservation::Board {
            standard_three: false,
            position: position.clone(),
            progress: Some(count),
        };
        let entry = paired_entry(&first, &first).expect("matching full-board and header facts");
        assert_eq!(
            entry,
            Entry::Board {
                progress: Some(count),
                standard_three: false
            }
        );
        assert!(
            standard_progress(entry).is_err(),
            "native denominator is never standard-mode proof"
        );
        let changed_count = NativeObservation::Board {
            standard_three: false,
            position: position.clone(),
            progress: Some(BoardCount {
                completed: completed + 1,
                target: 3,
            }),
        };
        assert_eq!(paired_entry(&first, &changed_count), None);
        let mut changed_position = position.clone();
        changed_position.tableau.swap(0, 1);
        assert_eq!(
            paired_entry(
                &first,
                &NativeObservation::Board {
                    standard_three: false,
                    position: changed_position,
                    progress: Some(count),
                }
            ),
            None
        );
        assert_eq!(
            paired_entry(
                &first,
                &NativeObservation::Board {
                    standard_three: false,
                    position: position.clone(),
                    progress: None,
                }
            ),
            None
        );
        assert_eq!(
            paired_entry(
                &first,
                &NativeObservation::Terminal(NativeTerminal::LevelUp)
            ),
            None
        );
        assert_eq!(paired_entry(&first, &NativeObservation::Unknown), None);
    }
    assert_eq!(
        paired_entry(
            &NativeObservation::Terminal(NativeTerminal::LevelUp),
            &NativeObservation::Terminal(NativeTerminal::LevelUp)
        ),
        Some(Entry::Terminal {
            supported_pyramid: true
        })
    );
    assert_eq!(
        paired_entry(&NativeObservation::Unknown, &NativeObservation::Unknown),
        None
    );
    let unknown_header = NativeObservation::Board {
        standard_three: false,
        position,
        progress: None,
    };
    assert_eq!(
        paired_entry(&unknown_header, &unknown_header),
        Some(unknown_board())
    );
}


/// A qualified entry keeps its actual board identities; counts alone cannot mint that witness.
#[test]
fn starting_witness_requires_qualified_matching_board_and_header_facts() {
    use xsar::games::cards::{Card, Rank, Suit};

    let position = BoardObservation {
        tableau: std::array::from_fn(|index| {
            Card::new(Rank::ALL[index % 13], Suit::ALL[index / 13])
        }),
        left: StockFace::Card(Card::new(Rank::ALL[2], Suit::ALL[2])),
        right: StockFace::Empty,
        undo_all: None,
    };
    let progress = BoardCount {
        completed: 1,
        target: 3,
    };
    let observation = NativeObservation::Board {
        standard_three: true,
        position: position.clone(),
        progress: Some(progress),
    };
    let witness = starting_witness(standard_board(1), &observation)
        .expect("qualified matching facts retain their actual identities");
    assert_eq!(witness.position, position);
    assert_eq!(witness.progress, progress);
    for entry in [
        unknown_board(),
        standard_board(0),
        standard_board(2),
        standard_terminal(),
    ] {
        assert!(starting_witness(entry, &observation).is_none());
    }
    let unqualified_observation = NativeObservation::Board {
        position: position.clone(),
        progress: Some(progress),
        standard_three: false,
    };
    assert!(starting_witness(standard_board(1), &unqualified_observation).is_none());
    let unqualified = paired_entry(&unqualified_observation, &unqualified_observation)
        .expect("matching observed pair without mode proof");
    assert!(
        starting_witness(unqualified, &observation).is_none(),
        "a displayed denominator of three supplies no standard-mode authority"
    );
    for observed in [
        NativeObservation::Unknown,
        NativeObservation::Terminal(NativeTerminal::LevelUp),
        NativeObservation::Board {
            standard_three: false,
            position: position.clone(),
            progress: None,
        },
    ] {
        assert!(starting_witness(standard_board(1), &observed).is_none());
    }
    for invalid_piles in [
        (StockFace::Empty, StockFace::Empty),
        (position.left, position.left),
    ] {
        let mut changed = position.clone();
        (changed.left, changed.right) = invalid_piles;
        let observed = NativeObservation::Board {
            standard_three: true,
            position: changed,
            progress: Some(progress),
        };
        assert!(starting_witness(standard_board(1), &observed).is_none());
    }
}


/// Advancing menus and Level Up are separate scene kinds even with an actual third-clear witness.
#[test]
fn different_native_terminal_kinds_never_form_a_confirmed_pair() {
    let level_up = NativeObservation::Terminal(NativeTerminal::LevelUp);
    let new_game = NativeObservation::Terminal(NativeTerminal::NewGame);
    assert_eq!(paired_entry(&level_up, &new_game), None);
    assert_eq!(paired_entry(&new_game, &level_up), None);
    for terminal in [level_up, new_game] {
        assert_eq!(
            paired_entry(&terminal, &terminal),
            Some(standard_terminal())
        );
        assert!(starting_witness(standard_terminal(), &terminal).is_none());
    }
}
