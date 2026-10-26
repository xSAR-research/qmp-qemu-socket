//! Per-board no-win policy and finite stock progression from a verified position.
//!
//! These values describe legal remaining operations, not input authority. Native
//! delivery must retain the acquisition epoch, recognise every effect and prove
//! REDEAL appearance independently before attempting that separate click.

use super::{CompleteDeal, RecordedAction, SearchState, rules};
use crate::pyramid_progress::{ChallengeKind, ChallengeProgress};
use xsar::games::pyramid::planning::{Location, MAX_RECYCLES, STOCK_LEN};


/// Automatic action after exhaustive search has proved no full-clear route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NoWinAction {
    /// Seek useful tableau removals on the same restored current-board inventory.
    MaximumMatches,
    /// A target-aware Boards challenge cannot benefit from partial matching.
    DrainStock,
}


/// Select the explicit Boards exception without classifying no-challenge play.
///
/// A target-aware caller must supply its positively bound actual requirement.
/// This function neither proves exhaustion nor authorises a stock gesture.
pub(super) fn no_win_action(
    target_aware: bool,
    requirement: Option<ChallengeProgress>,
) -> Result<NoWinAction, String> {
    if !target_aware {
        return Ok(NoWinAction::MaximumMatches);
    }
    let requirement = requirement.ok_or("Target-aware no-win policy lacks a bound requirement")?;
    if requirement.target_reached() {
        return Err("Reached challenge target retires matching, stock and REDEAL input".to_owned());
    }
    Ok(if requirement.kind == ChallengeKind::Boards {
        NoWinAction::DrainStock
    } else {
        NoWinAction::MaximumMatches
    })
}


/// The two legal uses of D while preserving every remaining card identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StockAdvance {
    /// Move the accessible LEFT card to RIGHT without resetting the deck.
    Draw,
    /// Reset an exhausted stock from its nonempty waste, consuming one recycle.
    Recycle,
}


impl StockAdvance {
    /// Return the shared rule action; neither variant can select a card or REDEAL.
    pub(super) fn recorded(self) -> RecordedAction {
        match self {
            Self::Draw => RecordedAction::Move,
            Self::Recycle => RecordedAction::Recycle,
        }
    }


    /// Wait before a fresh stock-effect capture, preserving the RECYCLE policy.
    fn settle_delay(self) -> std::time::Duration {
        match self {
            Self::Draw => crate::parameters::COMPUTED_ROUTE_MOVE_SETTLE_DELAY,
            Self::Recycle => crate::parameters::PYRAMID_DRAW_SETTLE_DELAY,
        }
    }
}


/// One separately acknowledged D operation and its exact before/after model states.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct DrainStep {
    /// Verified current state which must still match immediately before input.
    pub(super) before: SearchState,
    /// Exact expected effect, accepted only after fresh observation.
    pub(super) after: SearchState,
    /// DRAW or RECYCLE; both use one D without catch-up or retry.
    pub(super) operation: StockAdvance,
}


/// Bounded remaining stock work, separate from the challenge's new-board allowance.
#[derive(Debug)]
pub(super) struct StockDrain {
    /// Recycles actually acknowledged and verified before this continuation.
    pub(super) executed_recycles: u8,
    /// Only remaining legal operations; at most 74 from an untouched full deck.
    pub(super) steps: Vec<DrainStep>,
    /// Final exhausted position; this alone grants no REDEAL input authority.
    pub(super) exhausted: SearchState,
}


impl StockDrain {
    /// Derive remaining passes without repeating already consumed RECYCLEs.
    ///
    /// Rejects a cleared board, inconsistent acknowledged recycle accounting,
    /// invalid shared transitions or allocation failure. The source deal/state
    /// remain unchanged; only existing shared rules produce successor states.
    pub(super) fn new(
        deal: &CompleteDeal,
        state: SearchState,
        executed_recycles: u8,
    ) -> Result<Self, String> {
        if state.is_goal() {
            return Err(
                "A replayed full clear requires the new-board path, not stock draining".to_owned(),
            );
        }
        if state.recycles() != executed_recycles || executed_recycles > MAX_RECYCLES {
            return Err("Verified state and executed RECYCLE accounting differ".to_owned());
        }
        // Three stock passes plus two deck resets; this is an input bound, not
        // a search cutoff. Removed stock identities only reduce the allowance.
        let maximum = STOCK_LEN * (usize::from(MAX_RECYCLES) + 1) + usize::from(MAX_RECYCLES);
        let mut steps = Vec::new();
        steps
            .try_reserve_exact(maximum)
            .map_err(|error| format!("Cannot allocate finite stock continuation: {error}"))?;
        let mut current = state;
        loop {
            let operation = if rules::card_index(current, Location::DL).is_some() {
                StockAdvance::Draw
            } else if current.recycles() < MAX_RECYCLES
                && rules::card_index(current, Location::DR).is_some()
            {
                StockAdvance::Recycle
            } else {
                break;
            };
            if steps.len() >= maximum {
                return Err("Stock continuation exceeds three legal passes".to_owned());
            }
            let after = rules::apply(deal, current, operation.recorded())
                .map_err(|error| format!("Invalid remaining stock transition: {error}"))?;
            steps.push(DrainStep {
                before: current,
                after,
                operation,
            });
            current = after;
        }
        Ok(Self {
            executed_recycles,
            steps,
            exhausted: current,
        })
    }
}


/// Recognise a complete stock effect within the existing six-observation budget.
///
/// A reached target retires input even before a visible terminal is available.
fn observe_stock(
    io: &mut super::NativeIo<'_>,
    state: SearchState,
) -> Result<Option<super::DriveReport>, String> {
    use super::{BoardCompletion, DriveReport, EffectObservation, RouteIo};
    for attempt in 0..super::ORDINARY_OBSERVATIONS {
        io.check()?;
        match io.observe_source(state)? {
            EffectObservation::Position(true) => return Ok(None),
            EffectObservation::Challenge(proof) => return Ok(Some(DriveReport {
                completion: BoardCompletion::ChallengeTargetVerified(proof),
                message: "Challenge completion verified during stock continuation; all further input retired".to_owned(),
            })),
            EffectObservation::ChallengePending(progress) => return Ok(Some(DriveReport {
                completion: BoardCompletion::ChallengeTargetReached(progress),
                message: "Actual challenge target reached during stock continuation; terminal proof pending, all further input retired".to_owned(),
            })),
            EffectObservation::Position(false) => {}
        }
        if attempt + 1 < super::ORDINARY_OBSERVATIONS {
            io.wait(super::FINAL_OBSERVATION_DELAY)?;
        }
    }
    Err("Stock effect could not be verified; input is not repeated".to_owned())
}


/// Drain only the legal remaining passes, then consume one positively enabled REDEAL.
///
/// The acquisition stays privately borrowed throughout; every acknowledged D has
/// an independent effect proof. Missing appearance, STOP, changed context or any
/// uncertain delivery ends this consumed execution without retry or cleanup.
pub(super) fn drain_and_redeal(
    io: &mut super::NativeIo<'_>,
    state: SearchState,
) -> Result<super::DriveReport, String> {
    use super::{BoardCompletion, DriveReport, InputOperation, RouteIo};
    let drain = StockDrain::new(&io.plan.acquisition.deal, state, io.executed_recycles)?;
    super::send_log(
        io.events,
        format!(
            "Stock continuation: {} RECYCLE acknowledged; {} legal remaining D operations",
            drain.executed_recycles,
            drain.steps.len()
        ),
    );
    io.stock_continuation = true;
    for step in &drain.steps {
        io.pause_checkpoint()?;
        if let Some(report) = observe_stock(io, step.before)? {
            return Ok(report);
        }
        io.begin_action(step.before, &[InputOperation::PressDrawKey])?;
        io.deliver(InputOperation::PressDrawKey, step.before)?;
        if step.operation == StockAdvance::Recycle {
            io.recycle_acknowledged()?;
        }
        io.wait(step.operation.settle_delay())?;
        if let Some(report) = observe_stock(io, step.after)? {
            return Ok(report);
        }
    }
    io.pause_checkpoint()?;
    if let Some(report) = observe_stock(io, drain.exhausted)? {
        return Ok(report);
    }
    let (tableau, left, right) = super::expected(&io.plan.acquisition.deal, drain.exhausted)?;
    if left.is_some() {
        return Err("Residual stock is not exhausted; no REDEAL authorised".to_owned());
    }
    let frame = io
        .latest_frame
        .as_ref()
        .ok_or("Missing fresh exhausted-stock frame")?;
    let before = io
        .redeal_before
        .as_ref()
        .ok_or("Missing pre-input REDEAL region; no click authorised")?;
    let changed = before.changed(frame)?;
    let appeared = before.appeared(frame)?;
    super::send_log(
        io.events,
        format!(
            "REDEAL region appearance: {changed}/3024 materially changed pixels; appeared={appeared}; acknowledged RECYCLE={}",
            io.executed_recycles
        ),
    );
    if !appeared {
        // Legal stock is exhausted and observe_stock independently proved this
        // exact same-frame endpoint. Missing appearance grants no click and
        // does not establish that the challenge target was achieved.
        io.check()?;
        io.source_proof = None;
        io.action_permit = None;
        let remaining = u8::try_from(drain.exhausted.tableau_remaining())
            .map_err(|_| "Invalid exhausted tableau count")?;
        let removed = 28_u8
            .checked_sub(remaining)
            .ok_or("Invalid exhausted tableau count")?;
        return Ok(DriveReport {
            completion: BoardCompletion::PartialPositionVerified { removed, remaining },
            message: format!(
                "Legal stock exhausted; exact partial endpoint verified ({removed}/28 removed, {remaining} remaining). REDEAL region did not substantively change; no click authorised. Challenge completion remains unproved; control released without further input or retry."
            ),
        });
    }
    io.plan
        .acquisition
        .vision
        .verify_temporal_stock_position(frame, &tableau, None, right, appeared)?;
    let point = super::redeal_appearance::centre()?;
    let probe = io.qmp.probe().map_err(|error| error.to_string())?;
    super::validate_probe(&probe)?;
    io.check()?;
    io.source_proof = None;
    io.action_permit = None;
    // The point was derived from the latest complete same-frame position/control proof.
    io.counts.attempted = io
        .counts
        .attempted
        .checked_add(1)
        .ok_or("Gesture counter overflow")?;
    io.qmp
        .click(point, 1920, 1080)
        .map_err(|error| format!("Uncertain REDEAL delivery: {error}; no retry"))?;
    io.counts.acknowledged = io
        .counts
        .acknowledged
        .checked_add(1)
        .ok_or("Gesture counter overflow")?;
    io.counts.last_acknowledged_at = Some(io.elapsed());
    io.check()?;
    io.wait(crate::parameters::PYRAMID_REDEAL_SETTLE_DELAY)?;
    Ok(DriveReport {
        completion: BoardCompletion::StockRedealt,
        message: format!(
            "One positively enabled REDEAL acknowledged after {} RECYCLE operations; fresh Recon must prove the new board",
            io.executed_recycles
        ),
    })
}


#[cfg(test)]
mod tests {
    use super::*;


    /// Decode the unchanged accepted real deal through the authoritative codec.
    fn deal() -> CompleteDeal {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../xsar/tests/fixtures/pyramid-planning/pyramid-recon-20261019-v7-01.json");
        let bytes = std::fs::read(path).unwrap();
        xsar::pyramid_json::read_from(bytes.as_slice()).unwrap()
    }


    /// Advance a real state without matches, retaining exact pass/cursor semantics.
    fn advance(
        deal: &CompleteDeal,
        mut state: SearchState,
        actions: &[RecordedAction],
    ) -> SearchState {
        for action in actions {
            state = rules::apply(deal, state, *action).unwrap();
        }
        state
    }


    /// Untouched stock has exactly 72 moves and two recycles, never a third reset.
    #[test]
    fn untouched_stock_uses_three_passes_and_two_recycles() {
        let deal = deal();
        let drain = StockDrain::new(&deal, SearchState::initial(), 0).unwrap();
        assert_eq!(drain.executed_recycles, 0);
        assert_eq!(drain.steps.len(), 74);
        assert_eq!(
            drain
                .steps
                .iter()
                .filter(|step| step.operation == StockAdvance::Recycle)
                .count(),
            2
        );
        assert_eq!(drain.exhausted.recycles(), 2);
        assert_eq!(drain.exhausted.stock_cursor(), STOCK_LEN);
        assert_eq!(drain.exhausted.tableau_remaining(), 28);
        for adjacent in drain.steps.windows(2) {
            assert_eq!(adjacent[0].after, adjacent[1].before);
        }
    }


    /// Already executed passes reduce continuation instead of adding two new recycles.
    #[test]
    fn consumed_recycles_and_cursor_are_accounted_from_actual_position() {
        let deal = deal();
        let mut actions = vec![RecordedAction::Move; STOCK_LEN];
        actions.push(RecordedAction::Recycle);
        actions.extend([RecordedAction::Move; 7]);
        let state = advance(&deal, SearchState::initial(), &actions);
        let drain = StockDrain::new(&deal, state, 1).unwrap();
        assert_eq!(drain.steps.len(), 42);
        assert_eq!(
            drain
                .steps
                .iter()
                .filter(|step| step.operation == StockAdvance::Recycle)
                .count(),
            1
        );
        assert_eq!(drain.steps.first().unwrap().before, state);
        assert!(StockDrain::new(&deal, state, 0).is_err());
        let state = advance(
            &deal,
            state,
            &drain
                .steps
                .iter()
                .map(|step| step.operation.recorded())
                .collect::<Vec<_>>(),
        );
        assert!(StockDrain::new(&deal, state, 2).unwrap().steps.is_empty());
    }


    /// On the last pass only its remaining cards are moved; no recycle is added.
    #[test]
    fn last_pass_finishes_without_recycling_or_repeating_exhaustion() {
        let deal = deal();
        let full = StockDrain::new(&deal, SearchState::initial(), 0).unwrap();
        let state = full
            .steps
            .iter()
            .filter(|step| step.after.recycles() == 2)
            .nth(10)
            .unwrap()
            .after;
        let drain = StockDrain::new(&deal, state, 2).unwrap();
        assert_eq!(drain.steps.len(), STOCK_LEN - state.stock_cursor());
        assert!(
            drain
                .steps
                .iter()
                .all(|step| step.operation == StockAdvance::Draw)
        );
    }


    /// Board bypass requires target-aware binding; explicit no-challenge ignores it.
    #[test]
    fn boards_bypass_and_score_card_precedence_require_actual_target_binding() {
        let boards = ChallengeProgress::new(ChallengeKind::Boards, 0, 2).unwrap();
        assert_eq!(
            no_win_action(true, Some(boards)).unwrap(),
            NoWinAction::DrainStock
        );
        assert_eq!(
            no_win_action(false, Some(boards)).unwrap(),
            NoWinAction::MaximumMatches
        );
        assert_eq!(
            no_win_action(false, None).unwrap(),
            NoWinAction::MaximumMatches
        );
        for kind in [ChallengeKind::Score, ChallengeKind::Cards] {
            let pending = ChallengeProgress::new(kind, 1, 2).unwrap();
            assert_eq!(
                no_win_action(true, Some(pending)).unwrap(),
                NoWinAction::MaximumMatches
            );
            let reached = ChallengeProgress::new(kind, 2, 2).unwrap();
            assert!(no_win_action(true, Some(reached)).is_err());
        }
        assert!(no_win_action(true, None).is_err());
    }
}
