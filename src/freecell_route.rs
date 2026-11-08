//! Complete-route physical transfer sequencing, independent of Solver/HALO play.
//!
//! A delivery ACK never advances the route. Each accepted endpoint is observed
//! twice; an attempted or interrupted transfer retires its caller's live permit.
use xsar::games::freecell::{
    InitialDeal,
    rules::{Position, RecordedMove},
};


/// Frozen contract for who performs recorded automatic foundation transfers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum AutoMode {
    /// Game autoplay is enabled; wait for its complete observed closure, without input.
    #[default]
    ObserveGame,
    /// Game autoplay is disabled; execute every recorded transfer with semantic proof.
    ExecuteRecorded,
}


/// One contiguous physically replayed route endpoint supplied to observation.
#[derive(Clone, Copy)]
pub(crate) struct Candidate {
    /// Next unapplied recorded move at this endpoint.
    pub cursor: usize,
    /// Exact physical board, never compared through canonical Position equality.
    pub position: Position,
}


/// Immutable diagnostic context; it never grants input or changes the cursor.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Stage {
    /// Zero-based next recorded transfer and complete route length.
    pub cursor: usize,
    /// Number of recorded transfers, including automatic records.
    pub total: usize,
    /// Frozen automatic-transfer policy.
    pub mode: AutoMode,
    /// Requested checked physical edge, when one is pending.
    pub action: Option<RecordedMove>,
    /// Binding, readiness, transfer, effect or completion.
    pub phase: &'static str,
    /// One-based observation number; zero outside a proof window.
    pub observation: usize,
}


/// Existing worker I/O with one fresh observation per call and no automatic input retry.
pub(crate) trait RouteIo {
    /// Check current owner, STOP, shutdown and cooperative pause.
    fn check(&mut self) -> Result<(), String>;
    /// Independently recognise all 52 original identities before retaining hidden history.
    fn bind(&mut self, deal: &InitialDeal) -> Result<(), String>;
    /// Capture once; return one exact candidate index, or unsupported/transient evidence.
    fn observe(&mut self, candidates: &[Candidate]) -> Result<Option<usize>, String>;
    /// Deliver one held drag with both endpoints from one fresh physical parent proof.
    fn transfer(&mut self, parent: &Position, action: RecordedMove) -> Result<(), String>;
    /// Retain diagnostic context without extra I/O or sequencing effects.
    fn stage(&mut self, _stage: Stage) {}
    /// Existing interruptible settling policy; no input or extra acquisition here.
    fn settle(&mut self) -> Result<(), String>;
    /// Publish authoritative accepted route progress.
    fn progress(&mut self, cursor: usize, total: usize, position: &Position);
}


/// Same-run accepted physical state; permit ownership lives in the worker store.
#[derive(Clone, Copy)]
pub(crate) struct State {
    /// Immutable physical endpoint last proved twice.
    pub position: Position,
    /// Next recorded transfer; never incremented on delivery alone.
    pub cursor: usize,
    /// True only after independently matching the original deal in this run.
    pub bound: bool,
}


impl State {
    /// Start with no live authority, even for an independently replayed offline plan.
    pub(crate) fn new(deal: &InitialDeal) -> Self {
        Self {
            position: Position::initial(deal),
            cursor: 0,
            bound: false,
        }
    }
}


/// Compare every physical sequence/slot; Position's canonical Eq is deliberately unsuitable.
pub(crate) fn same_physical(a: &Position, b: &Position) -> bool {
    (0..8).all(|n| a.column(n) == b.column(n))
        && a.cells() == b.cells()
        && a.foundations() == b.foundations()
}


/// Build only the current endpoint and its contiguous recorded automatic successors.
fn candidates(
    position: Position,
    cursor: usize,
    moves: &[RecordedMove],
) -> Result<Vec<Candidate>, String> {
    let mut result = vec![Candidate { cursor, position }];
    let mut current = position;
    for (index, action) in moves.iter().enumerate().skip(cursor) {
        if !action.is_auto {
            break;
        }
        current = current.apply(*action).map_err(|e| e.to_string())?;
        result.push(Candidate {
            cursor: index + 1,
            position: current,
        });
    }
    Ok(result)
}


/// Two successive fresh equal endpoints within the existing six-observation policy.
fn stable(io: &mut impl RouteIo, choices: &[Candidate], stage: Stage) -> Result<Candidate, String> {
    let mut previous = None;
    let mut furthest = 0;
    for observation in 1..=6 {
        io.stage(Stage {
            observation,
            ..stage
        });
        io.check()?;
        let observed = io.observe(choices)?;
        if let Some(index) = observed {
            let candidate = choices
                .get(index)
                .ok_or("Invalid observed route endpoint")?;
            if candidate.cursor < furthest {
                return Err("Irreversible Free Cell position regressed".into());
            }
            furthest = candidate.cursor;
            let endpoint = index + 1 == choices.len();
            if endpoint && previous == Some(index) {
                return Ok(*candidate);
            }
        }
        previous = observed;
        io.settle()?;
    }
    Err("Free Cell physical position was not confirmed twice; no further input authorised".into())
}


/// Accept an observed endpoint and publish its physical cursor together.
fn accept(state: &mut State, candidate: Candidate, total: usize, io: &mut impl RouteIo) {
    state.position = candidate.position;
    state.cursor = candidate.cursor;
    io.progress(state.cursor, total, &state.position);
}


/// Execute one deliberate transfer plus contiguous autos, or the complete verified route.
///
/// Each recorded transfer uses one guarded explicit-target held drag. Delivery ACK
/// remains separate from two fresh physical child/auto-closure proofs. Any error
/// permanently ends the live invocation without retrying the transfer.
pub(crate) fn drive(
    state: &mut State,
    deal: &InitialDeal,
    moves: &[RecordedMove],
    single_step: bool,
    mode: AutoMode,
    io: &mut impl RouteIo,
) -> Result<(), String> {
    let mut stage = Stage {
        cursor: state.cursor,
        total: moves.len(),
        mode,
        action: moves.get(state.cursor).copied(),
        phase: "binding",
        observation: 0,
    };
    io.stage(stage);
    io.check()?;
    if !state.bound {
        if state.cursor != 0 {
            return Err("Unbound route has a noninitial physical position".into());
        }
        io.bind(deal)?;
        state.bound = true;
    }
    let initial = match mode {
        AutoMode::ObserveGame => candidates(state.position, state.cursor, moves)?,
        AutoMode::ExecuteRecorded => vec![Candidate {
            cursor: state.cursor,
            position: state.position,
        }],
    };
    stage.phase = "readiness";
    let initial = stable(io, &initial, stage)?;
    accept(state, initial, moves.len(), io);
    let mut deliberate = false;
    while let Some(&action) = moves.get(state.cursor) {
        if single_step && deliberate && !action.is_auto {
            break;
        }
        if mode == AutoMode::ObserveGame && action.is_auto {
            return Err(
                "Automatic closure was not observed; automatic records are never dragged".into(),
            );
        }
        stage = Stage {
            cursor: state.cursor,
            action: Some(action),
            phase: "transfer",
            observation: 0,
            ..stage
        };
        io.stage(stage);
        let next_cursor = state.cursor.checked_add(1).ok_or("Route cursor overflow")?;
        let next = state.position.apply(action).map_err(|e| e.to_string())?;
        let children = match mode {
            AutoMode::ObserveGame => candidates(next, next_cursor, moves)?,
            AutoMode::ExecuteRecorded => vec![Candidate {
                cursor: next_cursor,
                position: next,
            }],
        };
        io.check()?;
        io.transfer(&state.position, action)?;
        io.settle()?;
        stage.phase = "effect";
        let endpoint = stable(io, &children, stage)?;
        accept(state, endpoint, moves.len(), io);
        deliberate |= !action.is_auto;
    }
    io.stage(Stage {
        cursor: state.cursor,
        action: None,
        phase: "final-check",
        observation: 0,
        ..stage
    });
    io.check()?;
    if state.cursor == moves.len() && !state.position.is_goal() {
        return Err("Free Cell route ended before all 52 foundation cards".into());
    }
    io.stage(Stage {
        cursor: state.cursor,
        action: None,
        phase: "complete",
        observation: 0,
        ..stage
    });
    Ok(())
}


#[cfg(test)]
mod tests;
