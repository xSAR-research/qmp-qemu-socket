//! Scripted physical endpoints from a real solver; no transport or guest access.
use super::*;
use std::{
    collections::VecDeque,
    sync::{OnceLock, atomic::AtomicBool},
};
use xsar::games::{
    cards::{Card, Rank},
    freecell::{
        BoardState, CascadeStack, FOUNDATION_SUITS, Metadata,
        rules::Pile,
        search::{Outcome, solve_with_control},
    },
};


/// Independently replayed synthetic route and its physical endpoint history.
struct Fixture {
    deal: InitialDeal,
    moves: Vec<RecordedMove>,
    positions: Vec<Position>,
}


/// Lost objective provenance refuses before binding, capture, pause or input.
#[test]
fn freecell_315_unverified_objectives_never_call_route_io() {
    let fixture = fixture();
    for source in ["unit", "single-frame-tcp-v3;objective=unresolved;extra=yes"] {
        let deal = InitialDeal::new(
            fixture.deal.board().clone(),
            Metadata::new("test", "synthetic", source).unwrap(),
        )
        .unwrap();
        let mut state = State::new(&deal);
        let mut io = Fake::new(fixture);
        assert!(
            drive(
                &mut state,
                &deal,
                &fixture.moves,
                Target::FullBoard,
                false,
                AutoMode::ExecuteRecorded,
                &mut io
            )
            .is_err()
        );
        assert_eq!((io.binds, io.observations), (0, 0));
        assert!(io.transfers.is_empty() && io.stages.is_empty() && io.progress.is_empty());
        assert!(!state.bound);
    }
}


/// A visible opaque objective does not veto a physically verified full-board plan.
#[test]
fn freecell_315_challenge_source_uses_frozen_route_without_goal_text_inference() {
    let fixture = fixture();
    let deal = InitialDeal::new(
        fixture.deal.board().clone(),
        Metadata::new(
            "test",
            "synthetic",
            "single-frame-tcp-v3;objective=unresolved",
        )
        .unwrap(),
    )
    .unwrap();
    let mut state = State::new(&deal);
    let mut io = Fake::new(fixture);
    io.script(0, false);
    assert_eq!(
        drive(
            &mut state,
            &deal,
            &fixture.moves,
            Target::FullBoard,
            false,
            AutoMode::ObserveGame,
            &mut io
        )
        .unwrap(),
        Completion::FullBoard
    );
    assert!(state.position.is_goal());
}


/// A checked closed prefix ends with its exact physical count and no extra transfer.
#[test]
fn freecell_315_partial_endpoint_is_proved_and_has_its_own_completion() {
    let fixture = fixture();
    let end = closure_end(&fixture.moves, closure_end(&fixture.moves, 0) + 1);
    assert!(end < fixture.moves.len());
    let prefix = &fixture.moves[..end];
    let independent = xsar::games::freecell::replay::verify_prefix_with_control(
        &fixture.deal,
        prefix,
        &AtomicBool::new(false),
    )
    .unwrap();
    let target = Target::Partial {
        foundation_cards: independent
            .endpoint
            .foundations()
            .iter()
            .copied()
            .map(u32::from)
            .sum::<u32>(),
    };
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(fixture);
    assert_eq!(io.script(0, true), end);
    let completion = drive(
        &mut state,
        &fixture.deal,
        prefix,
        target,
        false,
        AutoMode::ObserveGame,
        &mut io,
    )
    .unwrap();
    assert_eq!(completion, Completion::PartialRouteExhausted);
    assert_eq!(state.cursor, end);
    assert!(same_physical(&state.position, &fixture.positions[end]));
    assert!(!state.position.is_goal());
    assert_eq!(io.transfers.len(), 1);
    assert_eq!(io.observations, 4);
}


/// Readiness completion immediately retires sequencing without inventing a route effect.
#[test]
fn freecell_315_terminal_readiness_has_no_transfer_or_cursor_progress() {
    let fixture = fixture();
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(fixture);
    io.seen.extend([Seen::Unknown; 6]);
    io.resolve_complete = Some(Ok(true));
    assert_eq!(
        io.drive(&mut state, false).unwrap(),
        Completion::ChallengeComplete
    );
    assert_eq!((state.cursor, io.observations, io.resolve_calls), (0, 6, 1));
    assert!(io.transfers.is_empty() && io.progress.is_empty());
}


/// A terminal frame after delivery is not counted as proof of the delivered card effect.
#[test]
fn freecell_315_terminal_after_delivery_never_advances_unproved_cursor() {
    let fixture = fixture();
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(fixture);
    io.explicit = true;
    io.pair(0);
    io.seen.extend([Seen::Unknown; 6]);
    io.resolve_complete = Some(Ok(true));
    assert_eq!(
        io.drive(&mut state, false).unwrap(),
        Completion::ChallengeComplete
    );
    assert_eq!(
        (
            state.cursor,
            io.observations,
            io.transfers.len(),
            io.resolve_calls
        ),
        (0, 8, 1, 1)
    );
    assert_eq!(io.progress, [0]);
}


/// A terminal frame at the fresh pre-input boundary pre-empts the pending drag.
#[test]
fn freecell_315_terminal_preinput_preempts_transfer() {
    let fixture = fixture();
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(fixture);
    io.explicit = true;
    io.pair(0);
    io.terminal_before_transfer = true;
    assert_eq!(
        io.drive(&mut state, false).unwrap(),
        Completion::ChallengeComplete
    );
    assert_eq!(state.cursor, 0);
    assert!(io.transfers.is_empty());
    assert_eq!(io.progress, [0]);
}


/// An already observed STOP remains authoritative even when completion is configured.
#[test]
fn freecell_315_stop_preempts_pending_terminal_observation() {
    let fixture = fixture();
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(fixture);
    io.stopped = true;
    io.resolve_complete = Some(Ok(true));
    assert!(io.drive(&mut state, false).unwrap_err().contains("STOP"));
    assert_eq!((io.binds, io.observations, io.resolve_calls), (0, 0, 0));
    assert!(io.transfers.is_empty());
}


/// Two blocked low cascades force deliberate transfers and interleaved autos.
fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let mut columns: [Vec<Card>; 8] = std::array::from_fn(|_| Vec::new());
        for (i, suit) in FOUNDATION_SUITS.into_iter().enumerate() {
            columns[i] = (1u8..=7)
                .rev()
                .map(|r| Card::new(Rank::from_value(r).unwrap(), suit))
                .collect();
            columns[i + 4] = (8u8..=13)
                .rev()
                .map(|r| Card::new(Rank::from_value(r).unwrap(), suit))
                .collect();
        }
        for low in 0..2 {
            let ace = columns[low].pop().unwrap();
            let eight = columns[low + 4].pop().unwrap();
            columns[low].push(eight);
            columns[low + 4].push(ace);
        }
        let deal = InitialDeal::new(
            BoardState::new(
                columns.map(|c| CascadeStack::from_cards(c).unwrap()),
                [None; 4],
                [0; 4],
            )
            .unwrap(),
            Metadata::new(
                "test",
                "synthetic",
                crate::freecell_recon::objective::FULL_BOARD_SOURCE,
            )
            .unwrap(),
        )
        .unwrap();
        let Outcome::Solved { moves, .. } =
            solve_with_control(&deal, &AtomicBool::new(false), None, None)
        else {
            panic!("synthetic route must solve")
        };
        let moves = moves.to_vec();
        let mut positions = vec![Position::initial(&deal)];
        for action in &moves {
            positions.push(positions.last().copied().unwrap().apply(*action).unwrap());
        }
        assert!(positions.last().unwrap().is_goal());
        assert!(moves.iter().filter(|a| !a.is_auto).count() >= 2);
        assert!(closure_end(&moves, 0) >= 2);
        Fixture {
            deal,
            moves,
            positions,
        }
    })
}


/// Find the required final endpoint of a contiguous automatic closure.
fn closure_end(moves: &[RecordedMove], mut cursor: usize) -> usize {
    while moves.get(cursor).is_some_and(|a| a.is_auto) {
        cursor += 1;
    }
    cursor
}


/// Native 52-card inventory with AC, 2C, then an explicit 8D-to-CELL boundary.
///
/// The other aces are buried, so the checked AUTO closure ends exactly after 2C.
/// Independent prefix replay checks legality and inventory, without claiming that
/// this synthetic prefix is a searched maximum-removal result.
fn ace_pair_fixture() -> Fixture {
    let mut columns: [Vec<Card>; 8] = std::array::from_fn(|_| Vec::new());
    for (index, suit) in FOUNDATION_SUITS.into_iter().enumerate() {
        columns[index] = (1u8..=7)
            .rev()
            .map(|rank| Card::new(Rank::from_value(rank).unwrap(), suit))
            .collect();
        columns[index + 4] = (8u8..=13)
            .rev()
            .map(|rank| Card::new(Rank::from_value(rank).unwrap(), suit))
            .collect();
    }
    for column in 1..4 {
        let ace = columns[column].pop().unwrap();
        let eight = columns[column + 4].pop().unwrap();
        columns[column].push(eight);
        columns[column + 4].insert(0, ace);
    }
    let deal = InitialDeal::new(
        BoardState::new(
            columns.map(|cards| CascadeStack::from_cards(cards).unwrap()),
            [None; 4],
            [0; 4],
        )
        .unwrap(),
        Metadata::new(
            "test",
            "synthetic-ace-pair",
            crate::freecell_recon::objective::FULL_BOARD_SOURCE,
        )
        .unwrap(),
    )
    .unwrap();
    let original = Position::initial(&deal);
    let ace = original.automatic().unwrap();
    assert_eq!(ace.card_id, 0);
    let after_ace = original.apply(ace).unwrap();
    let two = after_ace.automatic().unwrap();
    assert_eq!(two.card_id, 1);
    let after_two = after_ace.apply(two).unwrap();
    assert!(after_two.automatic().is_none());
    let explicit = RecordedMove {
        card_id: 13 + 8 - 1,
        from: Pile::Cascade(1),
        to: Pile::Cell(0),
        is_auto: false,
    };
    let endpoint = after_two.apply(explicit).unwrap();
    let moves = vec![ace, two, explicit];
    let replay = xsar::games::freecell::replay::verify_prefix_with_control(
        &deal,
        &moves,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(replay.endpoint.foundations(), &[2, 0, 0, 0]);
    Fixture {
        deal,
        moves,
        positions: vec![original, after_ace, after_two, endpoint],
    }
}


/// Exercise one approved step against the synthetic closed partial endpoint.
fn ace_pair_step(state: &mut State, io: &mut Fake<'_>) -> Result<Completion, String> {
    let fixture = io.fixture;
    drive(
        state,
        &fixture.deal,
        &fixture.moves,
        Target::Partial {
            foundation_cards: 2,
        },
        true,
        AutoMode::ObserveGame,
        io,
    )
}


/// Physical evidence independent of the local candidate vector's indexing.
#[derive(Clone, Copy)]
enum Seen {
    At(usize),
    Unknown,
    StopAt(usize),
}


/// Scripted physical evidence; one transfer invocation means one drag attempt.
struct Fake<'a> {
    fixture: &'a Fixture,
    seen: VecDeque<Seen>,
    transfers: Vec<RecordedMove>,
    progress: Vec<usize>,
    stages: Vec<Stage>,
    binds: usize,
    observations: usize,
    stopped: bool,
    uncertain: bool,
    explicit: bool,
    /// A terminal frame arrives at the transfer's fresh pre-input boundary.
    terminal_before_transfer: bool,
    /// Revoke only after this exact twice-proved endpoint is accepted and published.
    stop_after_progress: Option<usize>,
    /// Observed animation waits, paired with the number of earlier explicit transfers.
    animation_waits: Vec<(usize, std::time::Duration)>,
    /// Revoke on this animation wait to exercise the pre-next-input fence.
    stop_on_animation: bool,
    /// Graceful stop is independent of the immediate STOP latch.
    graceful_stop: bool,
    /// Request graceful stop after this many explicit transfers have started.
    graceful_after_transfer: Option<usize>,
    /// Request graceful stop while the pre-transfer pause checkpoint is being handled.
    graceful_during_transfer_check: bool,
    /// Distinguish confirmation, transient retry and first-effect cadence.
    frame_waits: Vec<&'static str>,
    /// Scripted result returned by `resolve_challenge_complete` after six failed observations.
    resolve_complete: Option<Result<bool, String>>,
    /// Number of invocations of `resolve_challenge_complete`.
    resolve_calls: usize,
    /// Scripted result returned by `resolve_normal_game_win` after six unmatched goal frames.
    resolve_win: Option<Result<bool, String>>,
    /// Number of invocations of `resolve_normal_game_win`.
    resolve_win_calls: usize,
}


impl<'a> Fake<'a> {
    /// Start a socket-free observation script.
    fn new(fixture: &'a Fixture) -> Self {
        Self {
            fixture,
            seen: VecDeque::new(),
            transfers: Vec::new(),
            progress: Vec::new(),
            stages: Vec::new(),
            binds: 0,
            observations: 0,
            stopped: false,
            uncertain: false,
            explicit: false,
            terminal_before_transfer: false,
            stop_after_progress: None,
            animation_waits: Vec::new(),
            stop_on_animation: false,
            graceful_stop: false,
            graceful_after_transfer: None,
            graceful_during_transfer_check: false,
            frame_waits: Vec::new(),
            resolve_complete: None,
            resolve_calls: 0,
            resolve_win: None,
            resolve_win_calls: 0,
        }
    }


    /// Supply two independent observations of one exact physical endpoint.
    fn pair(&mut self, cursor: usize) {
        self.seen.extend([Seen::At(cursor); 2]);
    }


    /// Script only complete expected effects, with no intermediate source click.
    fn script(&mut self, start: usize, single: bool) -> usize {
        let mut cursor = if self.explicit {
            start
        } else {
            closure_end(&self.fixture.moves, start)
        };
        self.pair(cursor);
        while self.fixture.moves.get(cursor).is_some() {
            cursor = if self.explicit {
                cursor + 1
            } else {
                closure_end(&self.fixture.moves, cursor + 1)
            };
            self.pair(cursor);
            if single {
                break;
            }
        }
        cursor
    }


    /// Exercise the production sequencing policy.
    fn drive(&mut self, state: &mut State, single: bool) -> Result<Completion, String> {
        let f = self.fixture;
        drive(
            state,
            &f.deal,
            &f.moves,
            Target::FullBoard,
            single,
            if self.explicit {
                AutoMode::ExecuteRecorded
            } else {
                AutoMode::ObserveGame
            },
            self,
        )
    }
}


impl RouteIo for Fake<'_> {
    /// STOP prohibits further transfers.
    fn check(&mut self) -> Result<(), String> {
        if self.graceful_during_transfer_check
            && self
                .stages
                .last()
                .is_some_and(|stage| stage.phase == "transfer")
        {
            self.graceful_stop = true;
        }
        if self.stopped {
            Err("test STOP".into())
        } else {
            Ok(())
        }
    }


    /// Count original-deal authentication, without granting effect proof.
    fn bind(&mut self, _: &InitialDeal) -> Result<(), String> {
        self.binds += 1;
        Ok(())
    }


    /// Match physical evidence, independent of local candidate-vector indices.
    fn observe(&mut self, candidates: &[Candidate]) -> Result<Observation, String> {
        self.observations += 1;
        let cursor = match self.seen.pop_front().expect("unexpected extra capture") {
            Seen::At(n) => n,
            Seen::Unknown => return Ok(Observation::Unmatched),
            Seen::StopAt(n) => {
                self.stopped = true;
                n
            }
        };
        Ok(candidates
            .iter()
            .position(|c| same_physical(&c.position, &self.fixture.positions[cursor]))
            .map_or(Observation::Unmatched, Observation::Position))
    }


    /// Attempt exactly one explicit-target transfer; never retry uncertainty.
    fn transfer(
        &mut self,
        parent: &Position,
        action: RecordedMove,
    ) -> Result<TransferOutcome, String> {
        self.check()?;
        if self.terminal_before_transfer {
            return Ok(TransferOutcome::ChallengeComplete);
        }
        assert!(
            self.fixture
                .moves
                .iter()
                .enumerate()
                .any(|(n, a)| *a == action && same_physical(parent, &self.fixture.positions[n]))
        );
        self.transfers.push(action);
        if self.uncertain {
            Err("test uncertain drag; never replay".into())
        } else {
            Ok(TransferOutcome::Delivered)
        }
    }


    /// Retain the exact stage independently of later cursor updates.
    fn stage(&mut self, stage: Stage) {
        self.stages.push(stage);
    }


    /// No real waiting is needed to exercise cancellation and evidence order.
    fn settle(&mut self) -> Result<(), String> {
        self.frame_waits.push("retry");
        self.check()
    }


    /// Record the wait between two eligible fresh endpoint observations.
    fn confirmation_wait(&mut self) -> Result<(), String> {
        self.frame_waits.push("confirmation");
        self.check()
    }


    /// Record the initial wait after the acknowledged transfer.
    fn post_transfer_wait(&mut self) -> Result<(), String> {
        self.frame_waits.push("post-transfer");
        self.check()
    }


    /// Observe the invocation's graceful request without changing the immediate STOP latch.
    fn stop_after_step(&mut self) -> Result<bool, String> {
        self.check()?;
        Ok(self.graceful_stop
            || self
                .graceful_after_transfer
                .is_some_and(|n| self.transfers.len() >= n))
    }


    /// Record exact AUTO delays without real sleeping in the sequencer tests.
    fn animation_wait(&mut self, duration: std::time::Duration) -> Result<(), String> {
        self.check()?;
        self.animation_waits.push((self.transfers.len(), duration));
        if self.stop_on_animation {
            self.stopped = true;
        }
        self.check()
    }


    /// Only a twice-proved physical endpoint may advance the accepted cursor.
    fn progress(&mut self, cursor: usize, total: usize, position: &Position) {
        assert!(total <= self.fixture.moves.len());
        assert!(same_physical(position, &self.fixture.positions[cursor]));
        self.progress.push(cursor);
        if self.stop_after_progress == Some(cursor) {
            self.stopped = true;
        }
    }


    /// Resolve Challenge Complete only after a full failed observation window.
    fn resolve_challenge_complete(&mut self) -> Result<bool, String> {
        self.resolve_calls += 1;
        self.resolve_complete.take().unwrap_or(Ok(false))
    }


    /// Resolve normal-game win entry only after six unmatched final goal frames.
    fn resolve_normal_game_win(&mut self, goal: &Position) -> Result<bool, String> {
        assert!(goal.is_goal());
        self.resolve_win_calls += 1;
        self.resolve_win.take().unwrap_or(Ok(false))
    }
}

/// Fault-gated completion runs only after six unmatched observations and grants no cursor credit.
#[test]
fn freecell_320_stable_polls_challenge_complete_only_after_six_unmatched_observations() {
    let fixture = fixture();
    let mut state = State::new(&fixture.deal);
    let mut positive = Fake::new(fixture);
    positive.seen.extend([Seen::Unknown; 6]);
    positive.resolve_complete = Some(Ok(true));
    assert_eq!(
        positive.drive(&mut state, false).unwrap(),
        Completion::ChallengeComplete
    );
    assert_eq!(
        (state.cursor, positive.observations, positive.resolve_calls),
        (0, 6, 1)
    );
    assert!(positive.transfers.is_empty() && positive.progress.is_empty());

    let mut negative = Fake::new(fixture);
    negative.seen.extend([Seen::Unknown; 6]);
    negative.resolve_complete = Some(Ok(false));
    assert_eq!(
        negative.drive(&mut state, false).unwrap_err(),
        "Free Cell physical position was not confirmed twice; no further input authorised"
    );
    assert_eq!(
        (state.cursor, negative.observations, negative.resolve_calls),
        (0, 6, 1)
    );
    assert!(negative.transfers.is_empty() && negative.progress.is_empty());
}


/// Unchanged readiness permits AC once; only combined AC+2C proof credits both.
#[test]
fn freecell_319_unchanged_readiness_plays_ace_and_proves_auto_two_before_stopping() {
    let fixture = ace_pair_fixture();
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(&fixture);
    io.pair(0);
    io.pair(1);
    io.pair(2);
    assert_eq!(
        ace_pair_step(&mut state, &mut io).unwrap(),
        Completion::StepVerified
    );
    assert_eq!(state.cursor, 2);
    assert!(same_physical(&state.position, &fixture.positions[2]));
    assert_eq!(io.transfers, fixture.moves[..1]);
    assert_eq!(io.progress, [0, 2]);
    assert_eq!(io.observations, 6);
    assert!(io.seen.is_empty());

    io.pair(2);
    io.pair(3);
    assert_eq!(
        ace_pair_step(&mut state, &mut io).unwrap(),
        Completion::PartialRouteExhausted
    );
    assert_eq!(state.cursor, 3);
    assert_eq!(io.transfers, [fixture.moves[0], fixture.moves[2]]);
    assert_eq!(io.progress, [0, 2, 2, 3]);
    assert_eq!(io.binds, 1);
    assert!(io.seen.is_empty());
}


/// Twice-proved root AUTO prefixes are credited before selecting the next source.
#[test]
fn freecell_319_observed_auto_prefix_is_not_dragged_again() {
    let fixture = ace_pair_fixture();
    for observed in [1, 2] {
        let mut state = State::new(&fixture.deal);
        let mut io = Fake::new(&fixture);
        io.pair(observed);
        io.pair(observed + 1);
        let completion = ace_pair_step(&mut state, &mut io).unwrap();
        assert_eq!(state.cursor, observed + 1);
        assert_eq!(io.transfers, [fixture.moves[observed]]);
        assert_eq!(io.progress, [observed, observed + 1]);
        assert_eq!(io.observations, 4);
        assert_eq!(
            completion,
            if observed == 1 {
                Completion::StepVerified
            } else {
                Completion::PartialRouteExhausted
            }
        );
        assert!(io.seen.is_empty());
    }
}


/// Missing, wrong or singly seen AUTO effects never authorise credit or another drag.
#[test]
fn freecell_319_missing_auto_effect_exhausts_six_observations_without_credit_or_replay() {
    let fixture = ace_pair_fixture();
    for effects in [
        [Seen::At(1); 6],
        [Seen::At(3); 6],
        [
            Seen::At(1),
            Seen::At(2),
            Seen::Unknown,
            Seen::Unknown,
            Seen::Unknown,
            Seen::Unknown,
        ],
    ] {
        let mut state = State::new(&fixture.deal);
        let mut io = Fake::new(&fixture);
        io.pair(0);
        io.seen.extend(effects);
        assert_eq!(
            ace_pair_step(&mut state, &mut io).unwrap_err(),
            "Free Cell physical position was not confirmed twice; no further input authorised"
        );
        assert_eq!(state.cursor, 0);
        assert!(same_physical(&state.position, &fixture.positions[0]));
        assert_eq!(io.transfers, fixture.moves[..1]);
        assert_eq!(io.progress, [0]);
        assert_eq!(io.observations, 8);
        if effects.iter().all(|effect| matches!(effect, Seen::At(1))) {
            assert_eq!(
                io.frame_waits,
                [
                    "confirmation",
                    "post-transfer",
                    "retry",
                    "retry",
                    "retry",
                    "retry",
                    "retry"
                ]
            );
        }
        let stage = io.stages.last().unwrap();
        assert_eq!(
            (stage.cursor, stage.phase, stage.observation),
            (0, "effect", 6)
        );
        assert_eq!(stage.action, Some(fixture.moves[0]));
        assert!(io.seen.is_empty());
    }
}


/// STOP after a fresh effect capture and passive completion both prevent AUTO credit.
#[test]
fn freecell_319_stop_and_terminal_during_auto_followups_keep_original_cursor() {
    let fixture = ace_pair_fixture();
    for terminal in [false, true] {
        let mut state = State::new(&fixture.deal);
        let mut io = Fake::new(&fixture);
        io.pair(0);
        io.seen.push_back(Seen::At(2));
        if terminal {
            io.seen.extend([Seen::Unknown; 5]);
            io.resolve_complete = Some(Ok(true));
        } else {
            io.seen.push_back(Seen::StopAt(2));
        }
        let result = ace_pair_step(&mut state, &mut io);
        if terminal {
            assert_eq!(result.unwrap(), Completion::ChallengeComplete);
            assert_eq!((io.observations, io.resolve_calls), (8, 1));
        } else {
            assert!(result.unwrap_err().contains("STOP"));
            assert_eq!((io.observations, io.resolve_calls), (4, 0));
        }
        assert_eq!(state.cursor, 0);
        assert_eq!(io.progress, [0]);
        assert_eq!(io.transfers, fixture.moves[..1]);
        assert!(io.seen.is_empty());
    }
}


/// A malformed recorded AUTO prefix refuses before any readiness capture or drag.
#[test]
fn freecell_319_malformed_auto_prefix_cannot_grant_input() {
    let mut fixture = ace_pair_fixture();
    fixture.moves[1].card_id = 52;
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(&fixture);
    assert_eq!(
        ace_pair_step(&mut state, &mut io).unwrap_err(),
        "Illegal Free Cell card transfer"
    );
    assert_eq!((state.cursor, io.observations, io.binds), (0, 0, 1));
    assert!(io.transfers.is_empty() && io.progress.is_empty());
}


/// Continuous playback repeats one explicit drag and exact AUTO closure per endpoint.
#[test]
fn freecell_319_full_route_from_unchanged_root_proves_each_combined_endpoint() {
    let fixture = fixture();
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(fixture);
    let mut expected = Vec::new();
    let mut cursor = 0;
    io.pair(cursor);
    while let Some(&action) = fixture.moves.get(cursor) {
        expected.push(action);
        cursor = closure_end(&fixture.moves, cursor + 1);
        io.pair(cursor);
    }
    assert_eq!(io.drive(&mut state, false).unwrap(), Completion::FullBoard);
    assert!(expected[0].is_auto);
    assert_eq!(io.transfers, expected);
    assert_eq!(io.observations, 2 * (io.transfers.len() + 1));
    assert_eq!(state.cursor, fixture.moves.len());
    assert!(state.position.is_goal());
    assert!(io.seen.is_empty());
}


/// SingleStep and resume drag each deliberate edge once, with one original bind.
#[test]
fn freecell_312_single_step_and_full_resume() {
    let f = fixture();
    let mut state = State::new(&f.deal);
    let mut io = Fake::new(f);
    let end = io.script(0, true);
    io.drive(&mut state, true).unwrap();
    assert_eq!(state.cursor, end);
    assert_eq!(io.transfers.len(), 1);
    assert!(io.seen.is_empty());
    io.script(end, false);
    io.drive(&mut state, false).unwrap();
    assert!(state.position.is_goal());
    assert_eq!(io.binds, 1);
    assert_eq!(
        io.transfers.len(),
        f.moves.iter().filter(|a| !a.is_auto).count()
    );
    assert!(io.seen.is_empty());
}


/// Progressing automatic prefixes reconcile before input; backward evidence still refuses.
#[test]
fn freecell_312_root_closure_and_regression() {
    let f = fixture();
    let root = closure_end(&f.moves, 0);
    let mut state = State::new(&f.deal);
    let mut io = Fake::new(f);
    io.seen.extend([
        Seen::At(0),
        Seen::At(root - 1),
        Seen::At(root),
        Seen::At(root),
    ]);
    let end = closure_end(&f.moves, root + 1);
    io.pair(end);
    io.drive(&mut state, true).unwrap();
    assert_eq!(io.transfers.len(), 1);
    assert_eq!(io.progress, [root, end]);
    let mut io = Fake::new(f);
    let mut state = State::new(&f.deal);
    io.seen.extend([Seen::At(2), Seen::Unknown, Seen::At(1)]);
    assert!(
        io.drive(&mut state, true)
            .unwrap_err()
            .contains("regressed")
    );
    assert!(io.transfers.is_empty());
}


/// Acknowledged drag without the exact planned effect must not be repeated.
#[test]
fn freecell_312_unmatched_and_wrong_effect_keep_original_cursor_and_details() {
    let f = fixture();
    let root = closure_end(&f.moves, 0);
    for seen in [Seen::Unknown, Seen::At(0)] {
        let mut state = State::new(&f.deal);
        let mut io = Fake::new(f);
        io.pair(root);
        io.seen.extend([seen; 6]);
        assert!(io.drive(&mut state, true).is_err());
        assert_eq!(state.cursor, root);
        assert_eq!(io.transfers.len(), 1);
        let stage = io.stages.last().unwrap();
        assert_eq!(
            (stage.cursor, stage.phase, stage.observation),
            (root, "effect", 6)
        );
        assert_eq!(stage.action, f.moves.get(root).copied());
        assert!(io.seen.is_empty());
    }
}


/// STOP before the drag or uncertainty during it cannot send another transfer.
#[test]
fn freecell_312_stop_and_uncertainty_never_replay() {
    let f = fixture();
    let root = closure_end(&f.moves, 0);
    let mut state = State::new(&f.deal);
    let mut io = Fake::new(f);
    io.seen.extend([Seen::At(root), Seen::StopAt(root)]);
    assert!(io.drive(&mut state, true).unwrap_err().contains("STOP"));
    assert!(io.transfers.is_empty());
    let mut state = State::new(&f.deal);
    let mut io = Fake::new(f);
    io.pair(root);
    io.uncertain = true;
    assert!(
        io.drive(&mut state, true)
            .unwrap_err()
            .contains("uncertain")
    );
    assert_eq!(io.transfers.len(), 1);
    assert_eq!(state.cursor, root);
    assert_eq!(io.observations, 2);
}


/// Explicit-record mode stops after one drag even when the next record is AUTO.
#[test]
fn freecell_319_explicit_records_stop_after_one_transfer_even_auto() {
    let f = fixture();
    let mut state = State::new(&f.deal);
    let mut io = Fake::new(f);
    io.explicit = true;
    let end = io.script(0, true);
    io.drive(&mut state, true).unwrap();
    assert_eq!(state.cursor, end);
    assert_eq!(end, 1);
    assert!(f.moves[0].is_auto);
    assert_eq!(io.transfers, f.moves[..1]);
    io.script(end, false);
    io.drive(&mut state, false).unwrap();
    assert_eq!(io.transfers, f.moves);
    assert!(state.position.is_goal());
    assert_eq!(io.binds, 1);
}


/// Revocation after a proved effect retains its cursor and pre-empts AUTO waiting.
#[test]
fn freecell_312_final_check_retains_accepted_cursor() {
    let f = fixture();
    let root = closure_end(&f.moves, 0);
    let end = closure_end(&f.moves, root + 1);
    let mut state = State::new(&f.deal);
    let mut io = Fake::new(f);
    io.pair(root);
    io.pair(end);
    io.stop_after_progress = Some(end);
    assert!(io.drive(&mut state, true).unwrap_err().contains("STOP"));
    assert_eq!(state.cursor, end);
    assert_eq!(io.progress, [root, end]);
    assert_eq!((io.observations, io.transfers.len()), (4, 1));
    assert!(io.seen.is_empty());
    assert_eq!(
        io.animation_waits.len(),
        f.moves[..root]
            .iter()
            .filter(|action| action.is_auto)
            .count()
    );
}


/// Every reached AUTO record receives its own half-second animation wait.
#[test]
fn freecell_3117_auto_waits_cover_explicit_followup_and_readiness_once() {
    let fixture = ace_pair_fixture();
    let half = std::time::Duration::from_millis(500);
    for initial in [0, 1, 2] {
        let mut state = State::new(&fixture.deal);
        let mut io = Fake::new(&fixture);
        io.pair(initial);
        io.pair(if initial == 2 { 3 } else { 2 });
        ace_pair_step(&mut state, &mut io).unwrap();
        assert_eq!(io.animation_waits.len(), 2);
        assert!(
            io.animation_waits
                .iter()
                .all(|(_, duration)| *duration == half)
        );
        assert_eq!(
            io.animation_waits
                .iter()
                .filter(|(transfers, _)| *transfers == 0)
                .count(),
            initial
        );
        assert_eq!(io.transfers.len(), 1);
    }
}


/// Explicit-record mode and continuous playback apply the same per-AUTO wait.
#[test]
fn freecell_3117_continuous_explicit_auto_waits_precede_next_transfer() {
    let fixture = ace_pair_fixture();
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(&fixture);
    io.pair(0);
    io.pair(1);
    io.pair(2);
    io.pair(3);
    let completion = drive(
        &mut state,
        &fixture.deal,
        &fixture.moves,
        Target::Partial {
            foundation_cards: 2,
        },
        false,
        AutoMode::ExecuteRecorded,
        &mut io,
    )
    .unwrap();
    assert_eq!(completion, Completion::PartialRouteExhausted);
    assert_eq!(io.transfers, fixture.moves);
    assert_eq!(
        io.animation_waits,
        [
            (1, std::time::Duration::from_millis(500)),
            (2, std::time::Duration::from_millis(500))
        ]
    );
}


/// STOP during an AUTO animation wait prevents every following explicit transfer.
#[test]
fn freecell_3117_stop_during_auto_wait_preserves_proved_cursor_and_no_next_input() {
    let fixture = ace_pair_fixture();
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(&fixture);
    io.pair(0);
    io.pair(2);
    io.stop_on_animation = true;
    assert!(
        drive(
            &mut state,
            &fixture.deal,
            &fixture.moves,
            Target::Partial {
                foundation_cards: 2
            },
            false,
            AutoMode::ObserveGame,
            &mut io
        )
        .unwrap_err()
        .contains("STOP")
    );
    assert_eq!(state.cursor, 2);
    assert_eq!(io.transfers, fixture.moves[..1]);
    assert_eq!(io.animation_waits.len(), 1);
}


/// A requested graceful stop completes one effect and its AUTO closure, then can resume.
#[test]
fn freecell_3118_graceful_stop_finishes_current_step_and_retains_resume() {
    let fixture = fixture();
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(fixture);
    let end = io.script(0, true);
    io.graceful_after_transfer = Some(1);
    assert_eq!(
        io.drive(&mut state, false).unwrap(),
        Completion::StepVerified
    );
    assert_eq!(state.cursor, end);
    assert_eq!(io.transfers.len(), 1);
    assert!(!io.stopped && state.bound);
    assert_eq!(
        io.animation_waits.len(),
        fixture.moves[..end].iter().filter(|m| m.is_auto).count()
    );
    io.graceful_after_transfer = None;
    io.script(end, true);
    assert_eq!(
        io.drive(&mut state, true).unwrap(),
        Completion::StepVerified
    );
    assert!(state.cursor > end);
    assert_eq!(io.transfers.len(), 2);
}


/// A graceful request before input leaves the twice-proved readiness endpoint resumable.
#[test]
fn freecell_3118_graceful_stop_before_transfer_sends_no_input() {
    let fixture = fixture();
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(fixture);
    let root = closure_end(&fixture.moves, 0);
    io.pair(root);
    io.graceful_stop = true;
    assert_eq!(
        io.drive(&mut state, false).unwrap(),
        Completion::StepVerified
    );
    assert_eq!(state.cursor, root);
    assert!(io.transfers.is_empty() && !io.stopped && state.bound);
}


/// A request received during the pause checkpoint prevents the next transfer on resume.
#[test]
fn freecell_3118_graceful_stop_during_pre_transfer_check_sends_no_input() {
    let fixture = fixture();
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(fixture);
    let root = closure_end(&fixture.moves, 0);
    io.pair(root);
    io.graceful_during_transfer_check = true;
    assert_eq!(
        io.drive(&mut state, false).unwrap(),
        Completion::StepVerified
    );
    assert!(io.graceful_stop && !io.stopped && state.bound);
    assert_eq!(state.cursor, root);
    assert_eq!(io.progress, [root]);
    assert_eq!(io.observations, 2);
    assert!(io.transfers.is_empty() && io.seen.is_empty());
}


/// Only an eligible physical endpoint receives the short confirmation cadence.
#[test]
fn freecell_3118_confirmation_cadence_keeps_six_fresh_effect_attempts() {
    let fixture = fixture();
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(fixture);
    let root = closure_end(&fixture.moves, 0);
    io.pair(root);
    io.seen.extend([Seen::Unknown; 6]);
    assert!(io.drive(&mut state, true).is_err());
    assert_eq!(state.cursor, root);
    assert_eq!(io.observations, 8);
    assert_eq!(io.transfers.len(), 1);
    assert_eq!(
        io.frame_waits,
        [
            "confirmation",
            "post-transfer",
            "retry",
            "retry",
            "retry",
            "retry",
            "retry"
        ]
    );
}


/// Final 52-card win animation covering `[K, K, K, K]` resolves via `resolve_normal_game_win`.
#[test]
fn freecell_330_normal_goal_win_entry_resolves_after_six_obscured_effect_frames() {
    let fixture = fixture();
    let final_explicit = fixture.moves.len() - 1;
    let mut state = State::new(&fixture.deal);
    state.bound = true;
    state.cursor = final_explicit;
    state.position = fixture.positions[final_explicit];
    let mut io = Fake::new(fixture);
    io.explicit = true;
    io.pair(final_explicit);
    io.seen.extend([Seen::Unknown; 6]);
    io.resolve_win = Some(Ok(true));
    assert_eq!(io.drive(&mut state, false).unwrap(), Completion::FullBoard);
    assert_eq!(io.resolve_win_calls, 1);
    assert_eq!(state.cursor, fixture.moves.len());
    assert!(state.position.is_goal());
    assert_eq!(io.progress.last().copied(), Some(fixture.moves.len()));
}


/// Mid-route and partial targets never invoke normal-game win resolution.
#[test]
fn freecell_330_non_goal_and_partial_routes_never_poll_normal_win_entry() {
    let fixture = fixture();
    let mut state = State::new(&fixture.deal);
    let mut io = Fake::new(fixture);
    io.pair(closure_end(&fixture.moves, 0));
    io.seen.extend([Seen::Unknown; 6]);
    io.resolve_win = Some(Ok(true));
    assert!(io.drive(&mut state, true).is_err());
    assert_eq!(io.resolve_win_calls, 0);

    let partial_end = fixture.moves.len() - 1;
    let partial_cards = fixture.positions[partial_end].foundation_cards();
    assert!(partial_cards < 52);
    let mut partial_state = State::new(&fixture.deal);
    partial_state.bound = true;
    partial_state.cursor = partial_end - 1;
    partial_state.position = fixture.positions[partial_end - 1];
    let mut partial_io = Fake::new(fixture);
    partial_io.pair(partial_end - 1);
    partial_io.seen.extend([Seen::Unknown; 6]);
    partial_io.resolve_win = Some(Ok(true));
    assert!(
        drive(
            &mut partial_state,
            &fixture.deal,
            &fixture.moves[..partial_end],
            Target::Partial {
                foundation_cards: partial_cards,
            },
            false,
            AutoMode::ExecuteRecorded,
            &mut partial_io,
        )
        .is_err()
    );
    assert_eq!(partial_io.resolve_win_calls, 0);
}
