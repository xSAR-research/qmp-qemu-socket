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
        search::{Outcome, solve_with_control},
    },
};


/// Independently replayed synthetic route and its physical endpoint history.
struct Fixture {
    deal: InitialDeal,
    moves: Vec<RecordedMove>,
    positions: Vec<Position>,
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
            Metadata::new("test", "synthetic", "unit").unwrap(),
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
        let mut deliberate = false;
        while let Some(action) = self.fixture.moves.get(cursor) {
            if single && deliberate && !action.is_auto {
                break;
            }
            deliberate |= !action.is_auto;
            cursor = if self.explicit {
                cursor + 1
            } else {
                closure_end(&self.fixture.moves, cursor + 1)
            };
            self.pair(cursor);
        }
        cursor
    }


    /// Exercise the production sequencing policy.
    fn drive(&mut self, state: &mut State, single: bool) -> Result<(), String> {
        let f = self.fixture;
        drive(
            state,
            &f.deal,
            &f.moves,
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
    fn observe(&mut self, candidates: &[Candidate]) -> Result<Option<usize>, String> {
        self.observations += 1;
        let cursor = match self.seen.pop_front().expect("unexpected extra capture") {
            Seen::At(n) => n,
            Seen::Unknown => return Ok(None),
            Seen::StopAt(n) => {
                self.stopped = true;
                n
            }
        };
        Ok(candidates
            .iter()
            .position(|c| same_physical(&c.position, &self.fixture.positions[cursor])))
    }


    /// Attempt exactly one explicit-target transfer; never retry uncertainty.
    fn transfer(&mut self, parent: &Position, action: RecordedMove) -> Result<(), String> {
        self.check()?;
        assert!(self.explicit || !action.is_auto);
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
            Ok(())
        }
    }


    /// Retain the exact stage independently of later cursor updates.
    fn stage(&mut self, stage: Stage) {
        self.stages.push(stage);
    }


    /// No real waiting is needed to exercise cancellation and evidence order.
    fn settle(&mut self) -> Result<(), String> {
        self.check()
    }


    /// Only a twice-proved physical endpoint may advance the accepted cursor.
    fn progress(&mut self, cursor: usize, total: usize, position: &Position) {
        assert_eq!(total, self.fixture.moves.len());
        assert!(same_physical(position, &self.fixture.positions[cursor]));
        self.progress.push(cursor);
    }
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


/// Delayed automatic prefixes are input-free until the whole closure is proved.
#[test]
fn freecell_312_root_closure_and_regression() {
    let f = fixture();
    let root = closure_end(&f.moves, 0);
    let mut state = State::new(&f.deal);
    let mut io = Fake::new(f);
    io.seen.extend([
        Seen::At(0),
        Seen::At(0),
        Seen::At(root - 1),
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


/// Explicit autoplay-off mode drags every original record exactly once.
#[test]
fn freecell_312_explicit_autos_preserve_single_deliberate_contract() {
    let f = fixture();
    let mut state = State::new(&f.deal);
    let mut io = Fake::new(f);
    io.explicit = true;
    let end = io.script(0, true);
    io.drive(&mut state, true).unwrap();
    assert_eq!(state.cursor, end);
    assert_eq!(io.transfers.iter().filter(|a| !a.is_auto).count(), 1);
    io.script(end, false);
    io.drive(&mut state, false).unwrap();
    assert_eq!(io.transfers, f.moves);
    assert!(state.position.is_goal());
    assert_eq!(io.binds, 1);
    let mut state = State::new(&f.deal);
    let mut io = Fake::new(f);
    io.seen.extend([Seen::At(0); 6]);
    assert!(io.drive(&mut state, true).is_err());
    assert!(io.transfers.is_empty());
}


/// Revocation after a proved effect reports the actual accepted cursor.
#[test]
fn freecell_312_final_check_retains_accepted_cursor() {
    let f = fixture();
    let root = closure_end(&f.moves, 0);
    let end = closure_end(&f.moves, root + 1);
    let mut state = State::new(&f.deal);
    let mut io = Fake::new(f);
    io.pair(root);
    io.seen.extend([Seen::At(end), Seen::StopAt(end)]);
    assert!(io.drive(&mut state, true).is_err());
    assert_eq!(state.cursor, end);
    let stage = io.stages.last().unwrap();
    assert_eq!((stage.cursor, stage.phase), (end, "final-check"));
}
