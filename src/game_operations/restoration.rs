//! Semantic restoration protocols; physical snapshots never own learned cards.

use super::OperationError;
use crate::geometry::PixelPoint;


/// Control classification must come from calibrated current-frame evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlAvailability {
    /// Exactly one enabled known control has been recognised.
    Enabled(PixelPoint),
    /// The control is positively disabled.
    Disabled,
    /// No control is recognised.
    NotFound,
    /// Competing controls or interpretations remain.
    Ambiguous,
}


/// Whether `Undo All` requires a secondary `OK` confirmation modal or may unwind directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmationPolicy {
    /// Require a twice-observed confirmation modal before verifying restoration (Pyramid).
    Required,
    /// Permit direct confirmation-less restoration (TriPeaks on Beast) or a twice-observed modal.
    DirectOrConfirmed,
}

/// Restoration input vocabulary; each variant represents exactly one attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UndoInput {
    /// Known-control Undo All, authorised by the adapter's acquisition history.
    UndoAll(PixelPoint),
    /// The independently confirmed complete modal's OK point.
    Confirm(PixelPoint),
    /// A positively enabled single Undo control.
    UndoLast(PixelPoint),
}


/// Complete semantic observations, never raw pixel-difference acceptance.
pub enum UndoScene<B> {
    /// A supported physical position; retained card knowledge stays elsewhere.
    Board(B),
    /// Complete confirmation signature with an interior OK point.
    Confirmation {
        /// Measured current OK point.
        ok: PixelPoint,
    },
    /// Supported gameplay scene while cards are actively unwinding in flight; grants no input authority.
    Animating,
    /// Unsupported or incomplete scene; no input is authorised.
    Unknown,
}


/// Existing game adapter owns fresh capture, guards, deadlines and no-replay I/O.
pub trait UndoAllIo {
    /// Physical-position snapshot, excluding accumulated knowledge and toolbar state.
    type Board;
    /// Select whether this game's `Undo All` requires a modal `OK` click or unwinds directly.
    fn confirmation_policy(&self) -> ConfirmationPolicy {
        ConfirmationPolicy::Required
    }
    /// Capture once and recognise a complete board or complete confirmation.
    fn observe(&mut self) -> Result<UndoScene<Self::Board>, OperationError>;
    /// Fresh probe and STOP/context checks around one non-replayable input.
    fn input(&mut self, input: UndoInput) -> Result<(), OperationError>;
    /// Use the adapter's existing bounded cancellable observation interval.
    fn settle(&mut self) -> Result<(), OperationError>;
}


/// Confirm the `Undo All` modal twice, deliver `Confirm(ok)`, and prove original restoration twice.
fn confirm_and_verify_restoration<I: UndoAllIo>(
    io: &mut I,
    original: &I::Board,
    current: &I::Board,
    ok: PixelPoint,
    same: &impl Fn(&I::Board, &I::Board) -> bool,
    max_observations: usize,
) -> Result<(), OperationError> {
    if !matches!(io.observe()?, UndoScene::Confirmation { ok: fresh } if fresh == ok) {
        return Err(OperationError::Verification(
            "Undo All confirmation signature changed before OK; input withheld".to_owned(),
        ));
    }
    io.input(UndoInput::Confirm(ok))?;
    io.settle()?;
    let direct_animating = io.confirmation_policy() == ConfirmationPolicy::DirectOrConfirmed;
    let mut restored = 0;
    for observation in 0..max_observations {
        match io.observe()? {
            UndoScene::Board(observed) if same(original, &observed) => {
                restored += 1;
                if restored == 2 {
                    return Ok(());
                }
            }
            UndoScene::Board(observed) if direct_animating || same(current, &observed) => {
                restored = 0;
            }
            UndoScene::Animating if direct_animating => {
                restored = 0;
            }
            UndoScene::Confirmation { ok: fresh } if fresh == ok => {
                restored = 0;
            }
            _ => {
                return Err(OperationError::Verification(
                    "Unexpected scene while verifying original-board restoration".to_owned(),
                ));
            }
        }
        if restored == 0 && observation + 1 < max_observations {
            io.settle()?;
        }
    }
    Err(OperationError::Verification(
        "OK delivery did not prove original-board restoration twice; no complete JSON published"
            .to_owned(),
    ))
}


/// Restore the original position while preserving the established Recon trace.
///
/// The caller proves acquisition completeness and owns the known Undo All control.
/// Depending on [`UndoAllIo::confirmation_policy`], either a twice-observed modal
/// confirmation authorises `OK` or the game may unwind directly; in both modes two
/// consecutive original snapshots prove restoration.
pub fn undo_all<I: UndoAllIo>(
    io: &mut I,
    original: &I::Board,
    current: &I::Board,
    control: PixelPoint,
    same: impl Fn(&I::Board, &I::Board) -> bool,
    max_observations: usize,
) -> Result<(), OperationError> {
    if max_observations < 2 {
        return Err(OperationError::Invalid(
            "restoration needs at least two observations".to_owned(),
        ));
    }
    let policy = io.confirmation_policy();
    io.input(UndoInput::UndoAll(control))?;
    if policy == ConfirmationPolicy::Required {
        let mut confirmation_target = None;
        for observation in 0..max_observations {
            match io.observe()? {
                UndoScene::Confirmation { ok } => {
                    confirmation_target = Some(ok);
                    break;
                }
                UndoScene::Board(observed) if same(current, &observed) => {}
                _ => {
                    return Err(OperationError::Verification(
                        "Unexpected scene after Undo All; confirmation input withheld".to_owned(),
                    ));
                }
            }
            if observation + 1 < max_observations {
                io.settle()?;
            }
        }
        let ok = confirmation_target.ok_or_else(|| {
            OperationError::Verification(
                "Undo All confirmation did not appear within the observation budget".to_owned(),
            )
        })?;
        return confirm_and_verify_restoration(io, original, current, ok, &same, max_observations);
    }

    io.settle()?;
    let mut restored = 0;
    for observation in 0..max_observations {
        match io.observe()? {
            UndoScene::Board(observed) if same(original, &observed) => {
                restored += 1;
                if restored == 2 {
                    return Ok(());
                }
            }
            UndoScene::Board(_) | UndoScene::Animating => {
                restored = 0;
            }
            UndoScene::Confirmation { ok } => {
                return confirm_and_verify_restoration(
                    io,
                    original,
                    current,
                    ok,
                    &same,
                    max_observations,
                );
            }
            UndoScene::Unknown => {
                return Err(OperationError::Verification(
                    "Unexpected scene after Undo All; restoration aborted".to_owned(),
                ));
            }
        }
        if restored == 0 && observation + 1 < max_observations {
            io.settle()?;
        }
    }
    Err(OperationError::Verification(
        "Undo All did not prove original-board restoration twice; no complete JSON published"
            .to_owned(),
    ))
}


/// A complete single-Undo observation, before or after its one action.
pub struct UndoLastObservation<B> {
    /// Actual recognised physical position, including any automatic transfers.
    pub board: B,
    /// Calibrated toolbar result; shared geometry alone cannot produce Enabled.
    pub control: ControlAvailability,
}


/// Single Undo needs calibrated enablement and an independently known predecessor.
pub trait UndoLastIo {
    /// Physical snapshot that includes compound moves and automatic transfers.
    type Board;
    /// Fresh supported observation; unimplemented game adapters return Unsupported.
    fn observe(&mut self) -> Result<UndoLastObservation<Self::Board>, OperationError>;
    /// One guarded Undo click; implementation permanently refuses uncertain replay.
    fn input(&mut self, point: PixelPoint) -> Result<(), OperationError>;
    /// Existing game-selected cancellable settle interval.
    fn settle(&mut self) -> Result<(), OperationError>;
}


/// Undo once and require two semantic predecessor observations; retain knowledge.
///
/// No production single-Undo calibration currently exists. This protocol is usable
/// by a measured adapter, and refuses missing/disabled/ambiguous control evidence.
pub fn undo_last<I: UndoLastIo>(
    io: &mut I,
    previous: &I::Board,
    current: &I::Board,
    control: PixelPoint,
    same: impl Fn(&I::Board, &I::Board) -> bool,
    max_observations: usize,
) -> Result<(), OperationError> {
    if max_observations < 2 {
        return Err(OperationError::Invalid(
            "restoration needs at least two observations".to_owned(),
        ));
    }
    if same(previous, current) {
        return Err(OperationError::Verification(
            "Undo predecessor must differ from current physical position".to_owned(),
        ));
    }
    let before = io.observe()?;
    if !same(current, &before.board) {
        return Err(OperationError::Verification(
            "Undo predecessor history does not match current physical position".to_owned(),
        ));
    }
    match before.control {
        ControlAvailability::Enabled(point) if point == control => {}
        ControlAvailability::Enabled(_) | ControlAvailability::Ambiguous => {
            return Err(OperationError::Ambiguous);
        }
        ControlAvailability::Disabled => return Err(OperationError::Disabled),
        ControlAvailability::NotFound => return Err(OperationError::NotFound),
    }
    io.input(control)?;
    io.settle()?;
    let mut restored = 0;
    for observation in 0..max_observations {
        let after = io.observe()?;
        if same(previous, &after.board) {
            restored += 1;
            if restored == 2 {
                return Ok(());
            }
        } else if same(current, &after.board) {
            restored = 0;
        } else {
            return Err(OperationError::Verification(
                "Undo produced an unexpected physical position".to_owned(),
            ));
        }
        if restored == 0 && observation + 1 < max_observations {
            io.settle()?;
        }
    }
    Err(OperationError::Verification(
        "Undo delivery did not prove the previous physical position twice".to_owned(),
    ))
}
