#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Guarded QMP `ReconDomain` implementation for live TriPeaks stock sweeps, probing, and atomic deal publication.
//!
//! Connects [`crate::recon::ReconDomain`] and [`crate::tripeaks_model`] to checked single-frame
//! [`crate::game_operations::Session`] I/O and [`crate::game_operations::restoration::undo_all`].
//! Every click and `D` draw requires fresh scene/board authorisation, probes VM running state and
//! absolute pointer routing, rechecks `STOP`, never retries uncertain QMP deliveries, and backs out
//! Draw-23 `"No Moves Remaining"` modals via `POST_GAME_MOUSE_HOLD` before `Undo All`.

#[cfg(test)]
mod tests;

use std::{
    cell::{Cell, RefCell},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use xsar::search::{SearchLimits, SearchOptions, SearchOutcome, SearchStop};

use crate::{
    capture::CapturedFrame,
    detector::pixel_rgb,
    freecell_terminal::{ChallengeTerminalHandoff, classify_challenge_handoff},
    game::GameMode,
    game_controls::{
        LocationRequest, is_challenge_complete_frame, is_tripeaks_full_root_stock, read_card,
        wait_for_challenge_complete,
    },
    game_operations::{
        CardRead, Input, OperationError, OperationIo, Session,
        restoration::{
            ConfirmationPolicy, ControlAvailability, UndoAllIo, UndoInput, UndoScene, undo_all,
        },
        validate_native_frame,
    },
    geometry::PixelPoint,
    parameters::{POST_GAME_MOUSE_HOLD, SHARED_UNDO_ALL_CONTROL, TABLEAU_CARD_REGIONS},
    qmp::SupportedKey,
    recon::{Card, CardSlot, Objective, ReconDomain, ReconError, ReconOutcome, SlotId},
    tracker::is_gameplay_scene_for_mode,
    tripeaks_model::{
        COVERING_CHILD_MASKS, DEAL_SLOT_COUNT, DEFAULT_SEARCH_LIMITS, DealProvenance,
        INITIAL_WASTE_SLOT, PublishDealError, STOCK_CARD_COUNT, STOCK_START_SLOT,
        TABLEAU_SLOT_COUNT, TriPeaksDealDocument, TriPeaksMove, TriPeaksPosition,
        are_ranks_adjacent, best_partial_route_with_control, card_rank_index, covering_children,
        is_slot_uncovered, plan_info_gain_probe_with_control, publish_to_path,
        slot_to_tableau_coord, solve_known_subgraph_with_control, tableau_slot_id,
        validate_slot_table,
    },
    tripeaks_progress::{
        TriPeaksChallenge, UNDO_LAST_MOVE_CLICK_POINT, observe_challenge_header,
        observe_no_moves_modal,
    },
    tripeaks_workflow_model::SlotKnowledgeCounts,
};

use super::tripeaks_terminal::{FINAL_CARD_ACQUISITION_CAPTURES, terminal_kind};

/// Default maximum `Undo All` probe cycles permitted per TriPeaks board.
pub const DEFAULT_MAX_PROBES: usize = 18;

/// FNV-1a 64-bit offset basis used for semantic board identity hashes.
const FNV1A_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a 64-bit prime used for semantic board identity hashes.
const FNV1A_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Minimum bright coral arrow pixels proving TriPeaks `Undo All` is enabled.
const UNDO_ALL_MIN_ENABLED_CORAL: u32 = 300;

/// Maximum dark dimmed pixels tolerated on an enabled `Undo All` button.
const UNDO_ALL_MAX_DARK_OVERLAY: u32 = 350;

/// Maximum coral pixels tolerated when `Undo All` is disabled at `T = 0`.
const UNDO_ALL_MAX_DISABLED_CORAL: u32 = 10;

/// Minimum muted grey pixels proving `Undo All` is positively disabled at `T = 0`.
const UNDO_ALL_MIN_DISABLED_MUTED: u32 = 450;

/// Maximum dark dimmed pixels tolerated when `Undo All` is disabled at `T = 0`.
const UNDO_ALL_MAX_DISABLED_DARK: u32 = 100;

/// Consecutive matching full-root non-matching frames required to short-circuit `Undo All` early.
const MIN_CONFIRMED_DRIFT_STREAK: usize = 4;

/// Bounded delays, poll counts, and search budgets for [`LiveTriPeaksDomain`].
#[derive(Clone, Debug)]
pub struct TriPeaksReconConfig {
    /// Maximum `Undo All` probing cycles before [`ReconError::ProbeBudgetExhausted`].
    pub max_probes: usize,
    /// Optional explicit objective overriding Challenge banner auto-detection.
    pub objective_override: Option<Objective>,
    /// Forces fault-gated Challenge handoff polling even when [`Self::objective_override`] is set.
    pub force_challenge_mode: bool,
    /// Settle delay after each gameplay click or `D` key delivery.
    pub move_settle_delay: Duration,
    /// Poll interval while waiting for card flip / deal settlement.
    pub reobserve_delay: Duration,
    /// Settle delay after `Undo All` or `"Undo last move"` modal dismissal.
    pub restore_settle_delay: Duration,
    /// Maximum post-move observation rounds before failing scene verification.
    pub max_step_observations: usize,
    /// Maximum observation rounds while restoring to `T = 0` or dismissing a modal.
    pub max_restore_observations: usize,
    /// Bounded node/memory/time limits forwarded to `tripeaks_model` searches.
    pub search_limits: SearchLimits,
}

impl Default for TriPeaksReconConfig {
    fn default() -> Self {
        Self {
            max_probes: DEFAULT_MAX_PROBES,
            objective_override: None,
            force_challenge_mode: false,
            move_settle_delay: Duration::from_millis(250),
            reobserve_delay: Duration::from_millis(100),
            restore_settle_delay: Duration::from_millis(200),
            max_step_observations: 12,
            max_restore_observations: 24,
            search_limits: DEFAULT_SEARCH_LIMITS,
        }
    }
}

/// Physical observation for one of the `28` TriPeaks tableau positions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableauSlotObs {
    /// Uncovered face-up card read with verified corner rank and suit.
    Card(Card),
    /// Positive `CLASSIC BLUE` covered card back.
    FaceDown,
    /// Positive green gameplay felt (slot already cleared).
    Empty,
}

/// Physical observation for the left-hand Stock tray.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StockObs {
    /// At least one `CLASSIC BLUE` card remains in the contracting Stock pile.
    FaceDown,
    /// Dark-green empty Draw-23 Stock tray.
    Empty,
}

/// Complete settled physical snapshot of an active TriPeaks gameplay frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoardSnapshot {
    /// Row-major `28` tableau slot states (`0..3` Row 1, `3..9` Row 2, `9..18` Row 3, `18..28` Row 4).
    pub tableau: [TableauSlotObs; TABLEAU_SLOT_COUNT],
    /// Left Stock pile state (`FaceDown` or `Empty`).
    pub stock: StockObs,
    /// Active face-up top card on the Waste pile.
    pub waste: Card,
    /// Measured toolbar state of `SHARED_UNDO_ALL_CONTROL`.
    pub undo_all: ControlAvailability,
}

impl BoardSnapshot {
    /// Return `true` only when the board matches an untouched `T = 0` deal (`18` face-down Rows 1..3,
    /// `10` face-up Row 4 cards, non-empty `FaceDown` Stock, and disabled `Undo All`).
    pub fn is_untouched_root_layout(&self) -> bool {
        self.tableau[..18]
            .iter()
            .all(|obs| *obs == TableauSlotObs::FaceDown)
            && self.tableau[18..]
                .iter()
                .all(|obs| matches!(obs, TableauSlotObs::Card(_)))
            && self.stock == StockObs::FaceDown
            && self.undo_all == ControlAvailability::Disabled
    }

    /// Return the count of tableau slots not currently `Empty`.
    pub fn occupied_tableau_count(&self) -> usize {
        self.tableau
            .iter()
            .filter(|&&obs| obs != TableauSlotObs::Empty)
            .count()
    }
}

/// Read-only semantic classification of one `1920×1080` TriPeaks guest capture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrameClassification {
    /// Consistent TriPeaks gameplay frame with every tableau slot, Stock, and Waste resolved.
    Settled(BoardSnapshot),
    /// Active gameplay scene where cards or flips remain unreadable / mid-animation.
    Settling,
    /// `"No Moves Remaining"` modal is active with the calibrated `"Undo last move"` click point.
    NoMovesModal(PixelPoint),
    /// Positive `"Challenge Complete!"` overlay with black `"Continue"` lettering.
    ChallengeComplete,
    /// Positive post-win TriPeaks reward/level-up overlay (`Congratulations` or `LEVEL-UP!`).
    BoardWinOverlay,
    /// Unsupported scene, ambiguous card reading, or structurally impossible cover state.
    Unsupported(String),
}

/// Summary and validated atomic-JSON payload produced by [`run_tripeaks_recon`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriPeaksReconRunReport {
    /// Terminal [`ReconOutcome`] produced by [`crate::recon::run_epistemic_recon`].
    pub outcome: ReconOutcome<TriPeaksMove>,
    /// Resolved Challenge or Classic [`Objective`] used during solving.
    pub objective: Objective,
    /// Challenge header detected on the initial `T = 0` capture, when present.
    pub detected_challenge: Option<TriPeaksChallenge>,
    /// Deterministic 64-bit semantic hash of the verified `T = 0` board.
    pub root_hash: u64,
    /// Validated `52`-slot deal document (including any solved/partial route).
    pub document: TriPeaksDealDocument,
}

/// Errors returned by [`run_tripeaks_recon`] across QMP reconnaissance and atomic publication.
#[derive(Debug, thiserror::Error)]
pub enum TriPeaksReconFailure {
    /// Failure during scene checking, QMP delivery, CV reading, restoration, or search.
    #[error(transparent)]
    Recon(#[from] ReconError),
    /// Failure validating or publishing the `52`-slot TriPeaks JSON artifact.
    #[error(transparent)]
    Publication(#[from] PublishDealError),
}

/// Compute a deterministic 64-bit FNV-1a semantic hash over tableau, Stock, and Waste identities.
///
/// Deliberately excludes desktop clock/taskbar pixels, cursor artwork, and toolbar icon states so
/// repeated `Undo All` cycles hash identically iff the physical `T = 0` board cards match.
pub fn semantic_board_hash(snapshot: &BoardSnapshot) -> u64 {
    let mut hash = FNV1A_OFFSET_BASIS;
    let mut mix = |byte: u8| {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV1A_PRIME);
    };
    for &byte in b"tripeaks-board-v1:" {
        mix(byte);
    }
    for obs in snapshot.tableau {
        mix(match obs {
            TableauSlotObs::Card(card) => card.0,
            TableauSlotObs::FaceDown => 0xFE,
            TableauSlotObs::Empty => 0xFF,
        });
    }
    mix(match snapshot.stock {
        StockObs::Empty => 0,
        StockObs::FaceDown => 1,
    });
    mix(snapshot.waste.0);
    hash
}

/// Classify the shared `Undo All` toolbar control (`(1_301, 959, 38, 38)`) from native pixels.
///
/// # Errors
///
/// Returns [`OperationError::Invalid`] if `frame` is not packed `1920×1080` RGBA8 or pixel sampling
/// falls out of bounds.
pub fn classify_tripeaks_undo_all(
    frame: &CapturedFrame,
) -> Result<ControlAvailability, OperationError> {
    validate_native_frame(frame)?;
    let bounds = SHARED_UNDO_ALL_CONTROL.bounds;
    let mut coral = 0_u32;
    let mut muted = 0_u32;
    let mut dark = 0_u32;

    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            let [r, g, b] = pixel_rgb(frame, x, y).ok_or_else(|| {
                OperationError::Invalid("Undo All pixel coordinate out of bounds".to_owned())
            })?;
            if r >= 180 && r.saturating_sub(g) >= 80 && r.saturating_sub(b) >= 80 {
                coral += 1;
            }
            if (40..=125).contains(&r) && (35..=115).contains(&g) && (25..=105).contains(&b) {
                muted += 1;
            }
            if r <= 35 && g <= 35 && b <= 35 {
                dark += 1;
            }
        }
    }

    if coral >= UNDO_ALL_MIN_ENABLED_CORAL && dark <= UNDO_ALL_MAX_DARK_OVERLAY {
        Ok(ControlAvailability::Enabled(
            SHARED_UNDO_ALL_CONTROL.click_point,
        ))
    } else if coral <= UNDO_ALL_MAX_DISABLED_CORAL
        && muted >= UNDO_ALL_MIN_DISABLED_MUTED
        && dark <= UNDO_ALL_MAX_DISABLED_DARK
    {
        Ok(ControlAvailability::Disabled)
    } else if coral >= 50 && muted >= 300 {
        Ok(ControlAvailability::Ambiguous)
    } else {
        Ok(ControlAvailability::NotFound)
    }
}

/// Convert an `xsar` card into a canonical `0..52` [`Card`].
fn recon_card_from_xsar(card: xsar::games::cards::Card) -> Result<Card, OperationError> {
    Card::try_from(card).map_err(|err| OperationError::Recognition(err.to_string()))
}

/// Classify a single native `1920×1080` frame into a TriPeaks gameplay snapshot or modal state.
///
/// # Errors
///
/// Returns [`OperationError`] when frame layout validation or underlying pixel readers fail.
pub fn observe_frame(frame: &CapturedFrame) -> Result<FrameClassification, OperationError> {
    validate_native_frame(frame)?;
    let gameplay = is_gameplay_scene_for_mode(frame, GameMode::TriPeaks)
        .map_err(|err| OperationError::Recognition(err.to_string()))?;
    if !gameplay {
        if is_challenge_complete_frame(frame)? {
            return Ok(FrameClassification::ChallengeComplete);
        }
        if let Some(point) = observe_no_moves_modal(frame)? {
            return Ok(FrameClassification::NoMovesModal(point));
        }
        if terminal_kind(frame)
            .map_err(OperationError::Recognition)?
            .is_some()
        {
            return Ok(FrameClassification::BoardWinOverlay);
        }
        return Ok(FrameClassification::Unsupported(
            "TriPeaks gameplay felt absent and no supported modal recognised".to_owned(),
        ));
    }

    let mut tableau = [TableauSlotObs::FaceDown; TABLEAU_SLOT_COUNT];
    let mut empty_mask = 0_u32;
    let mut saw_unreadable = false;

    for (idx, dst) in tableau.iter_mut().enumerate() {
        let slot_u8 = u8::try_from(idx)
            .map_err(|_| OperationError::Invalid("tableau slot index overflowed u8".to_owned()))?;
        let Some((row, column)) = slot_to_tableau_coord(SlotId(slot_u8)) else {
            return Err(OperationError::Invalid(format!(
                "invalid tableau slot index {idx}"
            )));
        };
        let req = LocationRequest::tripeaks_play(row, column);
        match read_card(frame, req)? {
            CardRead::Card(card) => *dst = TableauSlotObs::Card(recon_card_from_xsar(card)?),
            CardRead::FaceDown => *dst = TableauSlotObs::FaceDown,
            CardRead::Empty => {
                *dst = TableauSlotObs::Empty;
                empty_mask |= 1_u32 << idx;
            }
            CardRead::Unreadable => saw_unreadable = true,
            CardRead::Ambiguous => {
                return Ok(FrameClassification::Unsupported(format!(
                    "ambiguous tableau card at row {row} column {column}"
                )));
            }
        }
    }

    let stock = match read_card(frame, LocationRequest::tripeaks_draw(1))? {
        CardRead::FaceDown => StockObs::FaceDown,
        CardRead::Empty => StockObs::Empty,
        CardRead::Unreadable => {
            saw_unreadable = true;
            StockObs::Empty
        }
        CardRead::Card(_) | CardRead::Ambiguous => {
            return Ok(FrameClassification::Unsupported(
                "unexpected Stock face-up or ambiguous classification".to_owned(),
            ));
        }
    };

    let waste = match read_card(frame, LocationRequest::tripeaks_draw(2))? {
        CardRead::Card(card) => Some(recon_card_from_xsar(card)?),
        CardRead::Unreadable | CardRead::Empty => {
            saw_unreadable = true;
            None
        }
        CardRead::FaceDown | CardRead::Ambiguous => {
            return Ok(FrameClassification::Unsupported(
                "unexpected Waste face-down or ambiguous classification".to_owned(),
            ));
        }
    };

    if saw_unreadable {
        return Ok(FrameClassification::Settling);
    }
    let Some(waste) = waste else {
        return Ok(FrameClassification::Settling);
    };

    for (idx, obs) in tableau.iter().enumerate() {
        let covered = (COVERING_CHILD_MASKS[idx] & !empty_mask) != 0;
        if covered && *obs != TableauSlotObs::FaceDown {
            return Ok(FrameClassification::Unsupported(format!(
                "covered TriPeaks slot {idx} did not read FaceDown"
            )));
        }
        if !covered && *obs == TableauSlotObs::FaceDown {
            return Ok(FrameClassification::Settling);
        }
    }

    let undo_all = classify_tripeaks_undo_all(frame)?;
    let snapshot = BoardSnapshot {
        tableau,
        stock,
        waste,
        undo_all,
    };
    if snapshot.is_untouched_root_layout() && !is_tripeaks_full_root_stock(frame)? {
        return Ok(FrameClassification::Settling);
    }
    Ok(FrameClassification::Settled(snapshot))
}

/// Same-frame input authorisation callback passed to `NativeOperations` and test `OperationIo`s.
///
/// Grants no input on stale frames or unsupported scenes and validates every action coordinate
/// against live TriPeaks rules in `frame`.
///
/// # Errors
///
/// Returns [`OperationError::Verification`] if the target/hold does not match an authorised
/// gameplay or modal state, or propagates frame/card recognition errors.
pub fn authorise_tripeaks_input(frame: &CapturedFrame, input: Input) -> Result<(), OperationError> {
    validate_native_frame(frame)?;
    match input {
        Input::Key(SupportedKey::Draw) => {
            let gameplay = is_gameplay_scene_for_mode(frame, GameMode::TriPeaks)
                .map_err(|err| OperationError::Recognition(err.to_string()))?;
            if !gameplay {
                return Err(OperationError::Verification(
                    "TriPeaks gameplay scene absent before Draw".to_owned(),
                ));
            }
            if read_card(frame, LocationRequest::tripeaks_draw(1))? != CardRead::FaceDown {
                return Err(OperationError::Verification(
                    "Stock is not FaceDown before Draw".to_owned(),
                ));
            }
            if !matches!(
                read_card(frame, LocationRequest::tripeaks_draw(2))?,
                CardRead::Card(_)
            ) {
                return Err(OperationError::Verification(
                    "Waste card unreadable before Draw".to_owned(),
                ));
            }
            Ok(())
        }
        Input::MouseLeft(point) if point == SHARED_UNDO_ALL_CONTROL.click_point => {
            let gameplay = is_gameplay_scene_for_mode(frame, GameMode::TriPeaks)
                .map_err(|err| OperationError::Recognition(err.to_string()))?;
            if !gameplay {
                return Err(OperationError::Verification(
                    "TriPeaks gameplay scene absent before Undo All".to_owned(),
                ));
            }
            match classify_tripeaks_undo_all(frame)? {
                ControlAvailability::Enabled(enabled_pt) if enabled_pt == point => Ok(()),
                other => Err(OperationError::Verification(format!(
                    "Undo All control not positively enabled: {other:?}"
                ))),
            }
        }
        Input::MouseLeft(point) => authorise_tableau_tap_input(frame, point),
        Input::MouseLeftHold { point, hold }
            if point == UNDO_LAST_MOVE_CLICK_POINT && hold == POST_GAME_MOUSE_HOLD =>
        {
            match observe_no_moves_modal(frame)? {
                Some(detected) if detected == point => Ok(()),
                _ => Err(OperationError::Verification(
                    "No Moves Remaining Undo-last-move control absent before hold click".to_owned(),
                )),
            }
        }
        Input::MouseLeftHold { point, hold } => Err(OperationError::Verification(format!(
            "unauthorised TriPeaks hold click at ({}, {}) for {} ms",
            point.x,
            point.y,
            hold.as_millis()
        ))),
    }
}

/// Authorise a single left click against an uncovered, `±1` adjacent TriPeaks tableau card.
fn authorise_tableau_tap_input(
    frame: &CapturedFrame,
    point: PixelPoint,
) -> Result<(), OperationError> {
    let gameplay = is_gameplay_scene_for_mode(frame, GameMode::TriPeaks)
        .map_err(|err| OperationError::Recognition(err.to_string()))?;
    if !gameplay {
        return Err(OperationError::Verification(
            "TriPeaks gameplay scene absent before tableau click".to_owned(),
        ));
    }
    let Some((slot_idx, _)) = TABLEAU_CARD_REGIONS
        .iter()
        .enumerate()
        .find(|(_, region)| region.click_point == point)
    else {
        return Err(OperationError::Verification(format!(
            "click point ({}, {}) is not a calibrated TriPeaks target",
            point.x, point.y
        )));
    };
    let slot_u8 = u8::try_from(slot_idx)
        .map_err(|_| OperationError::Invalid("tableau index overflowed u8".to_owned()))?;
    let slot_id = SlotId(slot_u8);
    let Some((row, column)) = slot_to_tableau_coord(slot_id) else {
        return Err(OperationError::Invalid(format!(
            "invalid TriPeaks tableau slot {slot_idx}"
        )));
    };
    if let Some((left, right)) = covering_children(slot_id) {
        for child in [left, right] {
            let Some((cr, cc)) = slot_to_tableau_coord(child) else {
                return Err(OperationError::Invalid(
                    "invalid child coordinate".to_owned(),
                ));
            };
            if read_card(frame, LocationRequest::tripeaks_play(cr, cc))? != CardRead::Empty {
                return Err(OperationError::Verification(format!(
                    "TriPeaks slot ({row}, {column}) is still covered by ({cr}, {cc})"
                )));
            }
        }
    }
    let CardRead::Card(tap_card) = read_card(frame, LocationRequest::tripeaks_play(row, column))?
    else {
        return Err(OperationError::Verification(format!(
            "TriPeaks slot ({row}, {column}) is not a readable face-up card"
        )));
    };
    let CardRead::Card(waste_card) = read_card(frame, LocationRequest::tripeaks_draw(2))? else {
        return Err(OperationError::Verification(
            "Waste card is not readable before tableau click".to_owned(),
        ));
    };
    let recon_tap = recon_card_from_xsar(tap_card)?;
    let recon_waste = recon_card_from_xsar(waste_card)?;
    if !are_ranks_adjacent(card_rank_index(recon_tap), card_rank_index(recon_waste)) {
        return Err(OperationError::Verification(format!(
            "TriPeaks slot ({row}, {column}) is not +/-1 rank-adjacent to Waste"
        )));
    }
    Ok(())
}

/// Advisory live card-knowledge progress snapshot emitted during TriPeaks RECON.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TriPeaksReconProgress {
    /// Current `Known / Unknown / Unreachable` slot counts across the `52`-slot deal.
    pub(crate) counts: SlotKnowledgeCounts,
    /// Cumulative `Undo All` probing cycles completed on the current board.
    pub(crate) probes_used: usize,
    /// Resolved Challenge or Classic objective once read at `T = 0`.
    pub(crate) objective: Option<Objective>,
    /// Detected Challenge header banner at `T = 0`, if any.
    pub(crate) challenge: Option<TriPeaksChallenge>,
}

/// Physical step/restoration delays and retry bounds shared by `RECON`, `tripeaks_route`, and `tripeaks_session`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TriPeaksStepTiming {
    /// Settle delay after delivering each tableau tap or `D` Stock draw.
    pub(crate) move_settle_delay: Duration,
    /// Poll interval while waiting for animations or transitions to settle.
    pub(crate) reobserve_delay: Duration,
    /// Settle delay after `Undo All` or modal back-out delivery.
    pub(crate) restore_settle_delay: Duration,
    /// Maximum post-move observation rounds on non-terminal steps.
    pub(crate) max_step_observations: usize,
    /// Maximum observation rounds while restoring to `T = 0` or dismissing modals.
    pub(crate) max_restore_observations: usize,
}

impl From<&TriPeaksReconConfig> for TriPeaksStepTiming {
    fn from(config: &TriPeaksReconConfig) -> Self {
        Self {
            move_settle_delay: config.move_settle_delay,
            reobserve_delay: config.reobserve_delay,
            restore_settle_delay: config.restore_settle_delay,
            max_step_observations: config.max_step_observations,
            max_restore_observations: config.max_restore_observations,
        }
    }
}

/// Verified outcome returned after delivering and observing one guarded [`TriPeaksMove`].
#[derive(Clone, Debug)]
pub(crate) enum LiveStepOutcome {
    /// Ordinary settled gameplay board state (`TapTableau` or `Draw` verified against `prev`).
    Settled(BoardSnapshot),
    /// All `28` tableau cards are now `Empty` cleanly on gameplay; overlay/next board may follow.
    BoardCleared(BoardSnapshot),
    /// Twice-confirmed untouched `T = 0` next board semantic root hash.
    NextBoardDealt {
        /// Semantic root hash of the newly dealt untouched board.
        root_hash: u64,
    },
    /// Twice-confirmed TriPeaks `Congratulations` or `LEVEL-UP!` reward screen after final tap.
    BoardWinOverlay,
    /// Verified `Challenge Complete!` (or XP-counting / `LEVEL-UP!` handoff) screen.
    ChallengeComplete,
    /// `"No Moves Remaining"` modal observed when `allow_final_modal` was explicitly set.
    PartialModal,
}

/// Mode and safety options passed alongside one [`TriPeaksMove`] into [`deliver_and_observe_tripeaks_move`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TriPeaksStepOptions {
    /// Logical `TapTableau` or `Draw` action to deliver and verify.
    pub(crate) mv: TriPeaksMove,
    /// Whether Challenge completion / handoff detection is active.
    pub(crate) challenge_mode: bool,
    /// Whether landing on `"No Moves Remaining"` after the final banked move returns cleanly.
    pub(crate) allow_final_modal: bool,
}

/// Target untouched `T = 0` predicate checked during [`restore_tripeaks_with_driver`].
#[derive(Clone, Copy)]
enum RootRestoreTarget<'a> {
    /// Exact snapshot equality with a stored `T = 0` [`BoardSnapshot`] (recon probing).
    ExactSnapshot(&'a BoardSnapshot),
    /// Verified match against a saved `TriPeaksDealDocument`'s `Known` slots (`UNDO ALL` playback).
    DocumentSlots(&'a [CardSlot; DEAL_SLOT_COUNT]),
}

/// Borrowed QMP/capture/STOP context shared across `LiveTriPeaksDomain`, `tripeaks_route`, and
/// `tripeaks_session` so input delivery, modal back-out, and `Undo All` share one base implementation.
pub(crate) struct TriPeaksSessionDriver<'a, 'b, I: OperationIo> {
    /// Borrowed single-frame capture/input [`Session`].
    pub(crate) session: &'b mut Session<I>,
    /// Shared monotonic cooperative `STOP` flag.
    pub(crate) cancel: &'a AtomicBool,
    /// Immediate preview/telemetry callback invoked for every fresh capture.
    pub(crate) on_frame: &'b mut (dyn FnMut(&CapturedFrame) + 'a),
    /// Caller-owned sticky uncertain-delivery latch (`true` stops all subsequent guest input).
    pub(crate) uncertain_latched: &'b mut bool,
    /// Bounded settle delays and observation limits.
    pub(crate) timings: TriPeaksStepTiming,
}

impl<'a, 'b, I: OperationIo> TriPeaksSessionDriver<'a, 'b, I> {
    /// Convert [`OperationError`] into [`ReconError`], latching `uncertain_latched` on delivery fault.
    pub(crate) fn map_op_error(
        &mut self,
        context: &'static str,
        error: OperationError,
    ) -> ReconError {
        match error {
            OperationError::Cancelled => ReconError::Cancelled,
            OperationError::InterruptedAfterDelivery(inner)
                if matches!(*inner, OperationError::Cancelled) =>
            {
                ReconError::Cancelled
            }
            OperationError::Uncertain(inner) | OperationError::InterruptedAfterDelivery(inner) => {
                *self.uncertain_latched = true;
                ReconError::UncertainDelivery {
                    context,
                    detail: inner.to_string(),
                }
            }
            OperationError::Transport(source_msg) => ReconError::QmpTransport {
                context,
                source_msg,
            },
            other => ReconError::SceneVerification {
                detail: other.to_string(),
            },
        }
    }

    /// Fail closed immediately if `uncertain_latched`, `cancel`, or `session.check_active` triggers.
    pub(crate) fn check_ready(&mut self, context: &'static str) -> Result<(), ReconError> {
        if *self.uncertain_latched {
            return Err(ReconError::UncertainDelivery {
                context,
                detail: "earlier QMP delivery remains uncertain".to_owned(),
            });
        }
        if self.cancel.load(Ordering::Acquire) {
            return Err(ReconError::Cancelled);
        }
        if let Err(err) = self.session.check_active() {
            return Err(self.map_op_error(context, err));
        }
        Ok(())
    }

    /// Capture one fresh native frame, forward it immediately to `on_frame`, and return it.
    pub(crate) fn capture_and_publish(
        &mut self,
        context: &'static str,
    ) -> Result<&CapturedFrame, ReconError> {
        self.check_ready(context)?;
        if let Err(err) = self.session.screen_capture() {
            return Err(self.map_op_error(context, err));
        }
        let Some(frame) = self.session.current_frame() else {
            return Err(ReconError::SceneVerification {
                detail: "capture succeeded without retained current_frame".to_owned(),
            });
        };
        (self.on_frame)(frame);
        Ok(frame)
    }

    /// Cancellable wait mapped through [`Self::map_op_error`].
    pub(crate) fn wait_cancellable(
        &mut self,
        duration: Duration,
        context: &'static str,
    ) -> Result<(), ReconError> {
        self.check_ready(context)?;
        if let Err(err) = self.session.wait(duration) {
            return Err(self.map_op_error(context, err));
        }
        Ok(())
    }

    /// Deliver one guarded `SupportedKey` press, ensuring a fresh frame is bound in `session`.
    pub(crate) fn deliver_key(
        &mut self,
        key: SupportedKey,
        context: &'static str,
    ) -> Result<(), ReconError> {
        self.check_ready(context)?;
        if self.session.current_frame().is_none() {
            let _ = self.capture_and_publish(context)?;
        }
        if let Err(err) = self.session.send_key_stroke(key) {
            return Err(self.map_op_error(context, err));
        }
        Ok(())
    }

    /// Deliver one guarded left click, ensuring a fresh frame is bound in `session`.
    pub(crate) fn deliver_click(
        &mut self,
        point: PixelPoint,
        context: &'static str,
    ) -> Result<(), ReconError> {
        self.check_ready(context)?;
        if self.session.current_frame().is_none() {
            let _ = self.capture_and_publish(context)?;
        }
        if let Err(err) = self.session.mouse_left_click(point) {
            return Err(self.map_op_error(context, err));
        }
        Ok(())
    }

    /// Deliver one guarded click with explicit non-zero hold duration (e.g. `POST_GAME_MOUSE_HOLD`).
    pub(crate) fn deliver_hold_click(
        &mut self,
        point: PixelPoint,
        hold: Duration,
        context: &'static str,
    ) -> Result<(), ReconError> {
        self.check_ready(context)?;
        if self.session.current_frame().is_none() {
            let _ = self.capture_and_publish(context)?;
        }
        if let Err(err) = self.session.mouse_left_click_with_hold(point, hold) {
            return Err(self.map_op_error(context, err));
        }
        Ok(())
    }

    /// Poll until a consecutive capture is `Settled` and matches `expected`.
    pub(crate) fn capture_matching_settled(
        &mut self,
        expected: &BoardSnapshot,
        context: &'static str,
    ) -> Result<BoardSnapshot, ReconError> {
        let attempts = self.timings.max_step_observations.max(2);
        for attempt in 0..attempts {
            let frame = self.capture_and_publish(context)?;
            match observe_frame(frame).map_err(|err| self.map_op_error(context, err))? {
                FrameClassification::Settled(snap) if snap == *expected => return Ok(snap),
                FrameClassification::Settled(snap) => {
                    return Err(ReconError::SceneVerification {
                        detail: format!(
                            "unstable TriPeaks T=0 snapshot across consecutive captures (waste {:?} != {:?})",
                            snap.waste, expected.waste
                        ),
                    });
                }
                FrameClassification::Settling if attempt + 1 < attempts => {
                    self.wait_cancellable(self.timings.reobserve_delay, context)?;
                }
                other => {
                    return Err(ReconError::SceneVerification {
                        detail: format!(
                            "unexpected scene during consecutive confirmation: {other:?}"
                        ),
                    });
                }
            }
        }
        Err(ReconError::SceneVerification {
            detail: "consecutive settled confirmation timed out".to_owned(),
        })
    }

    /// Poll input-free until settled gameplay or a non-gameplay modal appears (`Settling` retries).
    pub(crate) fn acquire_non_settling_scene(
        &mut self,
        context: &'static str,
    ) -> Result<FrameClassification, ReconError> {
        let attempts = self.timings.max_restore_observations.max(1);
        for attempt in 0..attempts {
            let frame = self.capture_and_publish(context)?;
            let class = observe_frame(frame).map_err(|err| self.map_op_error(context, err))?;
            if class != FrameClassification::Settling {
                return Ok(class);
            }
            if attempt + 1 < attempts {
                self.wait_cancellable(self.timings.reobserve_delay, context)?;
            }
        }
        Ok(FrameClassification::Settling)
    }

    /// Back out a twice-confirmed `"No Moves Remaining"` modal using `POST_GAME_MOUSE_HOLD`, then
    /// wait for two consecutive matching un-dimmed `Settled` frames.
    pub(crate) fn back_out_no_moves_modal(
        &mut self,
        context: &'static str,
        initial_point: PixelPoint,
        already_captured_first: bool,
    ) -> Result<BoardSnapshot, ReconError> {
        if !already_captured_first {
            let first_frame = self.capture_and_publish(context)?;
            match observe_frame(first_frame).map_err(|err| self.map_op_error(context, err))? {
                FrameClassification::NoMovesModal(pt) if pt == initial_point => {}
                other => {
                    return Err(ReconError::SceneVerification {
                        detail: format!("expected No Moves modal confirmation #1, got {other:?}"),
                    });
                }
            }
        }
        let second_frame = self.capture_and_publish(context)?;
        match observe_frame(second_frame).map_err(|err| self.map_op_error(context, err))? {
            FrameClassification::NoMovesModal(pt) if pt == initial_point => {}
            other => {
                return Err(ReconError::SceneVerification {
                    detail: format!(
                        "No Moves modal control changed before hold click (expected {initial_point:?}, got {other:?})"
                    ),
                });
            }
        }

        self.deliver_hold_click(initial_point, POST_GAME_MOUSE_HOLD, context)?;
        self.wait_cancellable(self.timings.restore_settle_delay, context)?;

        let mut last_settled: Option<BoardSnapshot> = None;
        let attempts = self.timings.max_restore_observations.max(2);
        for attempt in 0..attempts {
            let frame = self.capture_and_publish(context)?;
            match observe_frame(frame).map_err(|err| self.map_op_error(context, err))? {
                FrameClassification::Settled(snap) => {
                    if last_settled.as_ref() == Some(&snap) {
                        return Ok(snap);
                    }
                    last_settled = Some(snap);
                }
                FrameClassification::Settling => {
                    last_settled = None;
                    if attempt + 1 < attempts {
                        self.wait_cancellable(self.timings.reobserve_delay, context)?;
                    }
                }
                other => {
                    return Err(ReconError::SceneVerification {
                        detail: format!(
                            "unexpected scene after No Moves Remaining Undo-last-move: {other:?}"
                        ),
                    });
                }
            }
        }
        Err(ReconError::SceneVerification {
            detail: "board did not settle twice cleanly after No Moves Remaining back-out"
                .to_owned(),
        })
    }

    /// Poll until two consecutive captures confirm `"No Moves Remaining"` at `expected`.
    fn capture_matching_no_moves_modal(
        &mut self,
        expected: PixelPoint,
        context: &'static str,
    ) -> Result<(), ReconError> {
        let frame = self.capture_and_publish(context)?;
        match observe_frame(frame).map_err(|err| self.map_op_error(context, err))? {
            FrameClassification::NoMovesModal(point) if point == expected => Ok(()),
            other => Err(ReconError::SceneVerification {
                detail: format!(
                    "expected second No Moves Remaining modal confirmation at {expected:?}, got {other:?}"
                ),
            }),
        }
    }

    /// Input-free, fault-gated poll for Challenge Complete (`Continue`) or XP/Level-Up handoff.
    pub(crate) fn poll_challenge_complete_or_handoff(&mut self) -> Result<bool, ReconError> {
        let settled_continue = {
            let mut io = DriverChallengePollIo { driver: self };
            wait_for_challenge_complete(&mut io)
        };
        match settled_continue {
            Ok(true) => return Ok(true),
            Ok(false) => {}
            Err(err) => return Err(self.map_op_error("probe_challenge_poll", err)),
        }

        if self.session.current_frame().is_none() {
            let _ = self.capture_and_publish("probe_challenge_handoff")?;
        }
        let Some(frame) = self.session.current_frame() else {
            return Ok(false);
        };
        match classify_challenge_handoff(frame) {
            Ok(Some(ChallengeTerminalHandoff::XpCounting | ChallengeTerminalHandoff::LevelUp)) => {
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(err) => Err(ReconError::SceneVerification {
                detail: format!("Challenge handoff classification failed: {err}"),
            }),
        }
    }
}

/// Input-free `OperationIo` bridge driving [`wait_for_challenge_complete`] via [`TriPeaksSessionDriver`].
struct DriverChallengePollIo<'a, 'b, 'c, I: OperationIo> {
    /// Active single-frame driver (never given `input` authority here).
    driver: &'c mut TriPeaksSessionDriver<'a, 'b, I>,
}

impl<I: OperationIo> OperationIo for DriverChallengePollIo<'_, '_, '_, I> {
    fn check_active(&mut self) -> Result<(), OperationError> {
        if self.driver.cancel.load(Ordering::Acquire) {
            return Err(OperationError::Cancelled);
        }
        self.driver.session.check_active()
    }

    fn capture(&mut self) -> Result<CapturedFrame, OperationError> {
        self.check_active()?;
        let frame = self.driver.session.screen_capture()?.clone();
        (self.driver.on_frame)(&frame);
        Ok(frame)
    }

    fn authorise(&mut self, _frame: &CapturedFrame, _input: Input) -> Result<(), OperationError> {
        Err(OperationError::Verification(
            "Challenge Complete polling never sends guest input".to_owned(),
        ))
    }

    fn probe(&mut self) -> Result<(), OperationError> {
        Err(OperationError::Verification(
            "Challenge Complete polling never probes pointer delivery".to_owned(),
        ))
    }

    fn deliver(&mut self, _input: Input) -> Result<(), OperationError> {
        Err(OperationError::Verification(
            "Challenge Complete polling never delivers guest input".to_owned(),
        ))
    }

    fn wait(&mut self, duration: Duration) -> Result<(), OperationError> {
        self.check_active()?;
        self.driver.session.wait(duration)
    }
}

/// Bridge [`TriPeaksSessionDriver`] into [`undo_all`] under `ConfirmationPolicy::DirectOrConfirmed`.
struct DriverUndoAllIo<'a, 'b, 'c, I: OperationIo> {
    /// Active single-frame driver.
    driver: &'c mut TriPeaksSessionDriver<'a, 'b, I>,
    /// Target `T = 0` acceptance mode (exact root snapshot vs document slot table).
    target: RootRestoreTarget<'c>,
    /// Retained matched snapshot from `observe()` (`Some` once target matches).
    last_matched: Option<BoardSnapshot>,
    /// Previous untouched snapshot observed when checking drifted restores.
    last_drifted: Option<BoardSnapshot>,
    /// Number of consecutive identical untouched non-root snapshots seen so far.
    drifted_streak: usize,
    /// Set once two consecutive identical untouched frames prove a drifted restore.
    confirmed_drifted: Option<BoardSnapshot>,
    /// Captures a `"No Moves Remaining"` click target if the modal opened right as `UndoAll` ran.
    observed_modal: Option<PixelPoint>,
}

impl<I: OperationIo> UndoAllIo for DriverUndoAllIo<'_, '_, '_, I> {
    type Board = BoardSnapshot;

    fn confirmation_policy(&self) -> ConfirmationPolicy {
        ConfirmationPolicy::DirectOrConfirmed
    }

    fn observe(&mut self) -> Result<UndoScene<Self::Board>, OperationError> {
        self.driver.session.check_active()?;
        if self.driver.cancel.load(Ordering::Acquire) {
            return Err(OperationError::Cancelled);
        }
        let frame = self.driver.session.screen_capture()?;
        (self.driver.on_frame)(frame);
        match observe_frame(frame)? {
            FrameClassification::Settled(snap) if snap.is_untouched_root_layout() => {
                let matches = match self.target {
                    RootRestoreTarget::ExactSnapshot(root) => snap == *root,
                    RootRestoreTarget::DocumentSlots(slots) => {
                        verify_snapshot_matches_document_root(&snap, slots).is_ok()
                            && self.last_matched.as_ref().is_none_or(|prev| *prev == snap)
                    }
                };
                if matches {
                    self.last_drifted = None;
                    self.drifted_streak = 0;
                    self.last_matched = Some(snap.clone());
                    return Ok(UndoScene::Board(snap));
                }
                self.last_matched = None;
                if self.last_drifted.as_ref() == Some(&snap) {
                    self.drifted_streak = self.drifted_streak.saturating_add(1);
                    self.confirmed_drifted = Some(snap);
                    if self.drifted_streak >= MIN_CONFIRMED_DRIFT_STREAK {
                        return Ok(UndoScene::Unknown);
                    }
                    return Ok(UndoScene::Animating);
                }
                self.last_drifted = Some(snap);
                self.drifted_streak = 1;
                Ok(UndoScene::Animating)
            }
            FrameClassification::Settled(_) | FrameClassification::Settling => {
                self.last_drifted = None;
                self.drifted_streak = 0;
                self.last_matched = None;
                Ok(UndoScene::Animating)
            }
            FrameClassification::NoMovesModal(point) => {
                self.observed_modal = Some(point);
                Ok(UndoScene::Unknown)
            }
            FrameClassification::ChallengeComplete
            | FrameClassification::BoardWinOverlay
            | FrameClassification::Unsupported(_) => Ok(UndoScene::Unknown),
        }
    }

    fn input(&mut self, input: UndoInput) -> Result<(), OperationError> {
        if self.driver.cancel.load(Ordering::Acquire) {
            return Err(OperationError::Cancelled);
        }
        match input {
            UndoInput::UndoAll(point) | UndoInput::Confirm(point) | UndoInput::UndoLast(point) => {
                let _ = self.driver.session.mouse_left_click(point)?;
                Ok(())
            }
        }
    }

    fn settle(&mut self) -> Result<(), OperationError> {
        let delay = self.driver.timings.restore_settle_delay;
        self.driver.session.wait(delay)
    }
}

/// Check that a live settled [`BoardSnapshot`] matches an expected [`TriPeaksPosition`] across
/// all `28` tableau slots, Stock pile state, and top Waste card.
///
/// # Errors
///
/// Returns an explanatory mismatch string if any tableau occupancy, covered state, known identity,
/// Stock pile status, or Waste card diverges.
pub(crate) fn verify_snapshot_matches_position(
    snapshot: &BoardSnapshot,
    slots: &[CardSlot; DEAL_SLOT_COUNT],
    pos: &TriPeaksPosition,
) -> Result<(), String> {
    for (idx, obs) in snapshot.tableau.iter().enumerate() {
        let expected_empty = (pos.tableau_mask & (1_u32 << idx)) == 0;
        let slot_id = SlotId(u8::try_from(idx).unwrap_or(u8::MAX));
        if expected_empty {
            if *obs != TableauSlotObs::Empty {
                return Err(format!(
                    "expected cleared Empty tableau slot {idx}, observed {obs:?}"
                ));
            }
            continue;
        }
        if is_slot_uncovered(pos.tableau_mask, slot_id.0) {
            match (slots[idx], *obs) {
                (CardSlot::Known(expected), TableauSlotObs::Card(actual)) if expected == actual => {
                }
                (CardSlot::Known(expected), actual) => {
                    return Err(format!(
                        "uncovered tableau slot {idx} mismatch: expected {expected:?}, observed {actual:?}"
                    ));
                }
                (_, TableauSlotObs::Card(_)) => {}
                (_, actual) => {
                    return Err(format!(
                        "uncovered tableau slot {idx} is not a face-up card: observed {actual:?}"
                    ));
                }
            }
        } else if *obs != TableauSlotObs::FaceDown {
            return Err(format!(
                "covered tableau slot {idx} mismatch: expected FaceDown, observed {obs:?}"
            ));
        }
    }
    let expected_stock = if pos.stock_cursor >= STOCK_CARD_COUNT {
        StockObs::Empty
    } else {
        StockObs::FaceDown
    };
    if snapshot.stock != expected_stock {
        return Err(format!(
            "Stock mismatch: expected {expected_stock:?} (cursor {}), observed {:?}",
            pos.stock_cursor, snapshot.stock
        ));
    }
    if snapshot.waste != pos.waste_card {
        return Err(format!(
            "Waste mismatch: expected {:?}, observed {:?}",
            pos.waste_card, snapshot.waste
        ));
    }
    Ok(())
}

/// Check that a live settled [`BoardSnapshot`] is an untouched `T = 0` root matching `slots`.
///
/// # Errors
///
/// Returns an explanatory mismatch string if the board is not untouched or differs from `slots`.
pub(crate) fn verify_snapshot_matches_document_root(
    snapshot: &BoardSnapshot,
    slots: &[CardSlot; DEAL_SLOT_COUNT],
) -> Result<(), String> {
    if !snapshot.is_untouched_root_layout() {
        return Err("board is not an untouched TriPeaks T=0 deal".to_owned());
    }
    let root_pos = TriPeaksPosition::from_root(slots).map_err(|err| err.to_string())?;
    verify_snapshot_matches_position(snapshot, slots, &root_pos)
}

/// Execute one guarded [`TriPeaksMove`] using `driver`, confirm physical change (or terminal /
/// automatic next-board / banked-modal transition), and append newly uncovered `0..18` parent
/// cards to `newly_observed`.
pub(crate) fn deliver_and_observe_tripeaks_move<I: OperationIo>(
    driver: &mut TriPeaksSessionDriver<'_, '_, I>,
    current: &BoardSnapshot,
    options: TriPeaksStepOptions,
    newly_observed: &mut Vec<(SlotId, Card)>,
) -> Result<LiveStepOutcome, ReconError> {
    let prev_occupied = current.occupied_tableau_count();
    let tap_target = match options.mv {
        TriPeaksMove::TapTableau { row, column } => {
            let slot_id =
                tableau_slot_id(row, column).ok_or_else(|| ReconError::SceneVerification {
                    detail: format!("invalid TriPeaks tableau coordinate ({row}, {column})"),
                })?;
            let slot_idx = usize::from(slot_id.0);
            let TableauSlotObs::Card(tapped_card) = current.tableau[slot_idx] else {
                return Err(ReconError::SceneVerification {
                    detail: format!("tableau target ({row}, {column}) is not readable Card"),
                });
            };
            if !are_ranks_adjacent(card_rank_index(tapped_card), card_rank_index(current.waste)) {
                return Err(ReconError::SceneVerification {
                    detail: format!(
                        "tableau target ({row}, {column}) is not +/-1 adjacent to Waste"
                    ),
                });
            }
            driver.deliver_click(
                TABLEAU_CARD_REGIONS[slot_idx].click_point,
                "tripeaks_tap_tableau",
            )?;
            Some((slot_idx, tapped_card))
        }
        TriPeaksMove::Draw => {
            if current.stock != StockObs::FaceDown {
                return Err(ReconError::SceneVerification {
                    detail: "Stock Draw attempted when Stock is not FaceDown".to_owned(),
                });
            }
            driver.deliver_key(SupportedKey::Draw, "tripeaks_stock_draw")?;
            None
        }
    };

    driver.wait_cancellable(driver.timings.move_settle_delay, "tripeaks_step_settle")?;

    let final_card_step = tap_target.is_some() && prev_occupied == 1;
    let attempts = if final_card_step {
        driver
            .timings
            .max_step_observations
            .max(FINAL_CARD_ACQUISITION_CAPTURES)
    } else {
        driver.timings.max_step_observations.max(1)
    };

    for attempt in 0..attempts {
        let frame = driver.capture_and_publish("tripeaks_step_observe")?;
        let class = observe_frame(frame)
            .map_err(|err| driver.map_op_error("tripeaks_step_observe", err))?;
        match class {
            FrameClassification::ChallengeComplete => {
                let second = driver
                    .capture_and_publish("tripeaks_challenge_complete_confirm")?
                    .clone();
                if is_challenge_complete_frame(&second).map_err(|err| {
                    driver.map_op_error("tripeaks_challenge_complete_confirm", err)
                })? {
                    return Ok(LiveStepOutcome::ChallengeComplete);
                }
                return Err(ReconError::SceneVerification {
                    detail: "Challenge Complete frame flickered on second confirmation".to_owned(),
                });
            }
            FrameClassification::BoardWinOverlay if final_card_step => {
                let second = driver
                    .capture_and_publish("tripeaks_board_win_confirm")?
                    .clone();
                match observe_frame(&second)
                    .map_err(|err| driver.map_op_error("tripeaks_board_win_confirm", err))?
                {
                    FrameClassification::ChallengeComplete => {
                        return Ok(LiveStepOutcome::ChallengeComplete);
                    }
                    FrameClassification::BoardWinOverlay => {
                        return Ok(LiveStepOutcome::BoardWinOverlay);
                    }
                    _ => {
                        return Err(ReconError::SceneVerification {
                            detail: "board win overlay did not hold across two confirmations"
                                .to_owned(),
                        });
                    }
                }
            }
            FrameClassification::Unsupported(detail) if options.challenge_mode => {
                if driver.poll_challenge_complete_or_handoff()? {
                    return Ok(LiveStepOutcome::ChallengeComplete);
                }
                if final_card_step && attempt + 1 < attempts {
                    driver.wait_cancellable(
                        driver.timings.reobserve_delay,
                        "tripeaks_step_reobserve",
                    )?;
                    continue;
                }
                return Err(ReconError::SceneVerification {
                    detail: format!(
                        "unsupported Challenge scene after step without completion: {detail}"
                    ),
                });
            }
            FrameClassification::Unsupported(_) if final_card_step && attempt + 1 < attempts => {
                driver
                    .wait_cancellable(driver.timings.reobserve_delay, "tripeaks_step_reobserve")?;
            }
            FrameClassification::NoMovesModal(point) if options.allow_final_modal => {
                driver.capture_matching_no_moves_modal(point, "tripeaks_partial_modal")?;
                return Ok(LiveStepOutcome::PartialModal);
            }
            FrameClassification::Settled(next)
                if final_card_step && next.is_untouched_root_layout() =>
            {
                let settled =
                    driver.capture_matching_settled(&next, "tripeaks_auto_next_board_confirm")?;
                let root_hash = semantic_board_hash(&settled);
                return Ok(LiveStepOutcome::NextBoardDealt { root_hash });
            }
            FrameClassification::Settled(next) => {
                if next == *current {
                    if attempt + 1 < attempts {
                        driver.wait_cancellable(
                            driver.timings.reobserve_delay,
                            "tripeaks_step_reobserve",
                        )?;
                        continue;
                    }
                    return Err(ReconError::SceneVerification {
                        detail: format!("move {:?} produced no physical board change", options.mv),
                    });
                }
                match tap_target {
                    Some((slot_idx, tapped_card)) => {
                        verify_tableau_step_transition(
                            current,
                            &next,
                            slot_idx,
                            tapped_card,
                            newly_observed,
                        )?;
                    }
                    None => {
                        if next.tableau != current.tableau || next.waste == current.waste {
                            return Err(ReconError::SceneVerification {
                                detail:
                                    "Draw caused unexpected tableau or unchanged Waste transition"
                                        .to_owned(),
                            });
                        }
                    }
                }
                if final_card_step && next.occupied_tableau_count() == 0 {
                    return Ok(LiveStepOutcome::BoardCleared(next));
                }
                return Ok(LiveStepOutcome::Settled(next));
            }
            FrameClassification::Settling if attempt + 1 < attempts => {
                driver
                    .wait_cancellable(driver.timings.reobserve_delay, "tripeaks_step_reobserve")?;
            }
            other => {
                return Err(ReconError::SceneVerification {
                    detail: format!(
                        "unexpected overlay/state {other:?} after move {:?}",
                        options.mv
                    ),
                });
            }
        }
    }
    Err(ReconError::SceneVerification {
        detail: format!(
            "move {:?} did not settle within observation budget",
            options.mv
        ),
    })
}

/// Dismiss any `"No Moves Remaining"` modal, execute guarded `Undo All` under
/// `ConfirmationPolicy::DirectOrConfirmed`, and verify twice-settled restoration to `slots`.
///
/// # Errors
///
/// Returns [`ReconError`] on cancellation, uncertain QMP delivery, transport error, missing
/// `Undo All` control, modal dismissal failure, or if restored cards mismatch `slots`.
pub(crate) fn restore_board_to_document_root<I: OperationIo>(
    driver: &mut TriPeaksSessionDriver<'_, '_, I>,
    context: &'static str,
    slots: &[CardSlot; DEAL_SLOT_COUNT],
) -> Result<BoardSnapshot, ReconError> {
    let restored =
        restore_tripeaks_with_driver(driver, context, RootRestoreTarget::DocumentSlots(slots))?;
    if let Err(reason) = verify_snapshot_matches_document_root(&restored, slots) {
        return Err(ReconError::SceneVerification {
            detail: format!("restored T=0 board does not match exported deal document: {reason}"),
        });
    }
    Ok(restored)
}

/// Core shared `Undo All` restoration loop parameterized by `target`.
fn restore_tripeaks_with_driver<I: OperationIo>(
    driver: &mut TriPeaksSessionDriver<'_, '_, I>,
    context: &'static str,
    target: RootRestoreTarget<'_>,
) -> Result<BoardSnapshot, ReconError> {
    let mut current = match driver.acquire_non_settling_scene(context)? {
        FrameClassification::Settled(snap) if snap.is_untouched_root_layout() => {
            return driver.capture_matching_settled(&snap, context);
        }
        FrameClassification::Settled(snap) => snap,
        FrameClassification::NoMovesModal(pt) => {
            let un_dimmed = driver.back_out_no_moves_modal(context, pt, true)?;
            if un_dimmed.is_untouched_root_layout() {
                return Ok(un_dimmed);
            }
            un_dimmed
        }
        other => {
            return Err(ReconError::SceneVerification {
                detail: format!("unexpected scene before Undo All: {other:?}"),
            });
        }
    };

    let mut modal_backed_out = false;
    loop {
        if current.undo_all != ControlAvailability::Enabled(SHARED_UNDO_ALL_CONTROL.click_point) {
            return Err(ReconError::SceneVerification {
                detail: format!(
                    "Undo All button is not Enabled before restoration: {:?}",
                    current.undo_all
                ),
            });
        }

        let max_obs = driver.timings.max_restore_observations.max(2);
        let placeholder_expected = match target {
            RootRestoreTarget::ExactSnapshot(root) => root.clone(),
            RootRestoreTarget::DocumentSlots(_) => current.clone(),
        };
        let mut undo_io = DriverUndoAllIo {
            driver,
            target,
            last_matched: None,
            last_drifted: None,
            drifted_streak: 0,
            confirmed_drifted: None,
            observed_modal: None,
        };
        let result = undo_all(
            &mut undo_io,
            &placeholder_expected,
            &current,
            SHARED_UNDO_ALL_CONTROL.click_point,
            |expected, observed| match target {
                RootRestoreTarget::ExactSnapshot(_) => observed == expected,
                RootRestoreTarget::DocumentSlots(slots) => {
                    verify_snapshot_matches_document_root(observed, slots).is_ok()
                }
            },
            max_obs,
        );
        let matched = undo_io.last_matched.take();
        let confirmed_drifted = undo_io.confirmed_drifted.take();
        let observed_modal = undo_io.observed_modal.take();

        match result {
            Ok(()) => {
                return match target {
                    RootRestoreTarget::ExactSnapshot(root) => Ok(root.clone()),
                    RootRestoreTarget::DocumentSlots(_) => {
                        matched.ok_or_else(|| ReconError::SceneVerification {
                            detail: "Undo All completed without retained matched snapshot"
                                .to_owned(),
                        })
                    }
                };
            }
            Err(op_err) => {
                if let Some(drifted) = confirmed_drifted
                    && !*driver.uncertain_latched
                    && !driver.cancel.load(Ordering::Acquire)
                {
                    return Ok(drifted);
                }
                if !modal_backed_out
                    && let Some(modal_pt) = observed_modal
                    && !*driver.uncertain_latched
                    && !driver.cancel.load(Ordering::Acquire)
                    && !matches!(
                        op_err,
                        OperationError::Cancelled
                            | OperationError::Uncertain(_)
                            | OperationError::InterruptedAfterDelivery(_)
                            | OperationError::Transport(_)
                    )
                {
                    modal_backed_out = true;
                    let un_dimmed = driver.back_out_no_moves_modal(context, modal_pt, true)?;
                    if un_dimmed.is_untouched_root_layout() {
                        return Ok(un_dimmed);
                    }
                    current = un_dimmed;
                    continue;
                }
                return Err(driver.map_op_error(context, op_err));
            }
        }
    }
}

/// Optional boxed live card-knowledge observer callback stored on [`LiveTriPeaksDomain`].
type ReconProgressHook<'a> = Option<Box<dyn FnMut(TriPeaksReconProgress) + 'a>>;

/// Live QMP + capture [`ReconDomain`] adapter for TriPeaks reconnaissance.
pub struct LiveTriPeaksDomain<'a, I: OperationIo> {
    /// Guarded single-frame input/capture session owning the no-replay uncertainty latch.
    session: Session<I>,
    /// Shared cooperative `STOP` latch.
    cancel: &'a AtomicBool,
    /// Configured timeouts, observation budgets, and search bounds.
    config: TriPeaksReconConfig,
    /// Immediate UI/telemetry frame observer invoked on every fresh capture.
    on_frame: Box<dyn FnMut(&CapturedFrame) + 'a>,
    /// Optional `Settled` first capture from [`Self::prepare_initial_observation`], consumed once.
    pre_observed_root: Option<BoardSnapshot>,
    /// Verified untouched `T = 0` physical snapshot established in `initialize_and_sweep_stock`.
    root_snapshot: Option<BoardSnapshot>,
    /// Deterministic 64-bit semantic hash of `root_snapshot`.
    root_hash: Option<u64>,
    /// Resolved Challenge or Classic objective for this deal.
    resolved_objective: Option<Objective>,
    /// Challenge header recognised at `T = 0`, when present.
    detected_challenge: Option<TriPeaksChallenge>,
    /// Whether fault-gated `Challenge Complete!` polling is active.
    challenge_mode: bool,
    /// Latest known/unknown/unreachable `52`-slot table observed by RECON and search.
    latest_slots: Cell<[CardSlot; DEAL_SLOT_COUNT]>,
    /// Count of successful `Undo All` restoration cycles completed after initial Stock sweep.
    undo_cycles_completed: usize,
    /// Moves executed cleanly when [`ReconOutcome::CompletedDuringRecon`] intercepted probing.
    completed_probe_moves: Option<Vec<TriPeaksMove>>,
    /// Latched `true` on any uncertain delivery or post-delivery interruption (other than pure `STOP`).
    uncertain_latched: bool,
    /// Optional advisory card-knowledge observer invoked after slot/probe updates.
    on_progress: RefCell<ReconProgressHook<'a>>,
}

impl<'a, I: OperationIo> LiveTriPeaksDomain<'a, I> {
    /// Wrap a fresh [`Session`] and configuration without performing guest I/O yet.
    pub fn new(
        session: Session<I>,
        cancel: &'a AtomicBool,
        config: TriPeaksReconConfig,
        on_frame: Box<dyn FnMut(&CapturedFrame) + 'a>,
    ) -> Self {
        let challenge_mode = config.force_challenge_mode;
        Self {
            session,
            cancel,
            config,
            on_frame,
            pre_observed_root: None,
            root_snapshot: None,
            root_hash: None,
            resolved_objective: None,
            detected_challenge: None,
            challenge_mode,
            latest_slots: Cell::new([CardSlot::Unknown; DEAL_SLOT_COUNT]),
            undo_cycles_completed: 0,
            completed_probe_moves: None,
            uncertain_latched: false,
            on_progress: RefCell::new(None),
        }
    }

    /// Install an advisory progress callback receiving updated `Known / Unknown / Unreachable` counts
    /// and `Probes used` after Stock sweep and each probe cycle.
    pub(crate) fn set_progress_observer(
        &mut self,
        observer: Box<dyn FnMut(TriPeaksReconProgress) + 'a>,
    ) {
        *self.on_progress.borrow_mut() = Some(observer);
    }

    /// Borrow a [`TriPeaksSessionDriver`] over this domain's `session`, `cancel`, `on_frame`, and
    /// `uncertain_latched` fields.
    fn driver(&mut self) -> TriPeaksSessionDriver<'a, '_, I> {
        TriPeaksSessionDriver {
            session: &mut self.session,
            cancel: self.cancel,
            on_frame: &mut *self.on_frame,
            uncertain_latched: &mut self.uncertain_latched,
            timings: TriPeaksStepTiming::from(&self.config),
        }
    }

    /// Return the most recently captured raw frame held by [`Session`], if any.
    pub(crate) fn current_frame(&self) -> Option<&CapturedFrame> {
        self.session.current_frame()
    }

    /// Forward an updated progress snapshot to the installed observer, if any.
    fn emit_progress(&self, slots: &[CardSlot; DEAL_SLOT_COUNT]) {
        if let Ok(mut guard) = self.on_progress.try_borrow_mut()
            && let Some(observer) = guard.as_mut()
        {
            observer(TriPeaksReconProgress {
                counts: SlotKnowledgeCounts::from_slots(slots),
                probes_used: self.undo_cycles_completed,
                objective: self.resolved_objective,
                challenge: self.detected_challenge,
            });
        }
    }

    /// Return the resolved Challenge or Classic [`Objective`], if initialised.
    pub const fn resolved_objective(&self) -> Option<Objective> {
        self.resolved_objective
    }

    /// Return the detected Challenge ribbon state from `T = 0`, if any.
    pub const fn detected_challenge(&self) -> Option<TriPeaksChallenge> {
        self.detected_challenge
    }

    /// Override the resolved objective only when `T = 0` had no live Challenge header and no
    /// explicit override was already configured.
    pub(crate) fn apply_fallback_objective_if_normal(&mut self, fallback: Objective) {
        if self.detected_challenge.is_none() && self.config.objective_override.is_none() {
            self.resolved_objective = Some(fallback);
        }
    }

    /// Return `true` when Challenge-mode completion handoff inspection is active.
    pub const fn is_challenge_mode(&self) -> bool {
        self.challenge_mode
    }

    /// Return the latest `52`-slot card table tracked across `RECON` and search.
    pub fn latest_slots(&self) -> [CardSlot; DEAL_SLOT_COUNT] {
        self.latest_slots.get()
    }

    /// Return the deterministic semantic hash of the verified `T = 0` root board, if initialised.
    pub const fn root_hash(&self) -> Option<u64> {
        self.root_hash
    }

    /// Return the number of probe `Undo All` cycles completed after the initial Stock sweep.
    pub const fn undo_cycles_completed(&self) -> usize {
        self.undo_cycles_completed
    }

    /// Return the executed move prefix if probing completed live (`CompletedDuringRecon`).
    pub fn completed_probe_moves(&self) -> Option<&[TriPeaksMove]> {
        self.completed_probe_moves.as_deref()
    }

    /// Return `true` when the cooperative `STOP` flag is asserted.
    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::Acquire)
    }

    /// Return `true` when an uncertain input attempt permanently latched this domain.
    pub const fn is_uncertain_latched(&self) -> bool {
        self.uncertain_latched
    }

    /// Consume the domain and return its underlying [`Session`] (e.g. for inspection).
    pub fn into_session(self) -> Session<I> {
        self.session
    }

    /// Capture the initial `T = 0` frame (if needed), resolve Challenge/Normal [`Objective`], and
    /// cache the first settled root snapshot so [`ReconDomain::initialize_and_sweep_stock`] avoids a
    /// redundant capture.
    ///
    /// # Errors
    ///
    /// Returns [`ReconError`] on cancellation, transport/capture fault, unexpected scene/modal at
    /// `T = 0`, or unsupported single-card Challenge objective.
    pub fn prepare_initial_observation(&mut self) -> Result<Objective, ReconError> {
        if let Some(obj) = self.resolved_objective
            && self.pre_observed_root.is_some()
        {
            return Ok(obj);
        }
        let mut challenge = None;
        let first_snap =
            self.capture_first_untouched_root("initial_objective_probe", &mut challenge)?;
        let obj = self.resolve_objective_from_challenge(challenge)?;
        self.pre_observed_root = Some(first_snap);
        Ok(obj)
    }

    /// Abort immediately if `uncertain_latched`, `cancel`, or `OperationIo::check_active` refuses.
    fn check_ready(&mut self, context: &'static str) -> Result<(), ReconError> {
        if self.uncertain_latched {
            return Err(ReconError::UncertainDelivery {
                context,
                detail: "earlier QMP delivery remains uncertain".to_owned(),
            });
        }
        if self.cancel.load(Ordering::Acquire) {
            return Err(ReconError::Cancelled);
        }
        if let Err(err) = self.session.check_active() {
            return Err(self.map_op_error(context, err));
        }
        Ok(())
    }

    /// Map [`OperationError`] to [`ReconError`], permanently latching transport uncertainty.
    fn map_op_error(&mut self, context: &'static str, error: OperationError) -> ReconError {
        match error {
            OperationError::Cancelled => ReconError::Cancelled,
            OperationError::InterruptedAfterDelivery(inner)
                if matches!(*inner, OperationError::Cancelled) =>
            {
                ReconError::Cancelled
            }
            OperationError::Uncertain(inner) | OperationError::InterruptedAfterDelivery(inner) => {
                self.uncertain_latched = true;
                ReconError::UncertainDelivery {
                    context,
                    detail: inner.to_string(),
                }
            }
            OperationError::Transport(source_msg) => ReconError::QmpTransport {
                context,
                source_msg,
            },
            other => ReconError::SceneVerification {
                detail: other.to_string(),
            },
        }
    }

    /// Capture one fresh `1920×1080` frame, forward it immediately to `on_frame`, and return it.
    fn capture_and_publish(&mut self, context: &'static str) -> Result<&CapturedFrame, ReconError> {
        self.check_ready(context)?;
        if let Err(err) = self.session.screen_capture() {
            return Err(self.map_op_error(context, err));
        }
        let Some(frame) = self.session.current_frame() else {
            return Err(ReconError::SceneVerification {
                detail: "capture succeeded without retained current_frame".to_owned(),
            });
        };
        (self.on_frame)(frame);
        Ok(frame)
    }

    /// Run [`Session::wait`] with unified error mapping.
    fn wait_cancellable(
        &mut self,
        duration: Duration,
        context: &'static str,
    ) -> Result<(), ReconError> {
        self.check_ready(context)?;
        if let Err(err) = self.session.wait(duration) {
            return Err(self.map_op_error(context, err));
        }
        Ok(())
    }

    /// Deliver one guarded `SupportedKey` press, latching uncertainty on any delivery failure.
    fn deliver_key(&mut self, key: SupportedKey, context: &'static str) -> Result<(), ReconError> {
        self.check_ready(context)?;
        if self.session.current_frame().is_none() {
            let _ = self.capture_and_publish(context)?;
        }
        if let Err(err) = self.session.send_key_stroke(key) {
            return Err(self.map_op_error(context, err));
        }
        Ok(())
    }

    /// Resolve and store objective and Challenge mode from an optional detected banner.
    fn resolve_objective_from_challenge(
        &mut self,
        challenge: Option<TriPeaksChallenge>,
    ) -> Result<Objective, ReconError> {
        self.detected_challenge = challenge;
        self.challenge_mode = self.config.force_challenge_mode || challenge.is_some();
        if let Some(override_obj) = self.config.objective_override {
            self.resolved_objective = Some(override_obj);
            return Ok(override_obj);
        }
        let objective = match challenge {
            Some(c) => c
                .to_recon_objective()
                .ok_or_else(|| ReconError::SceneVerification {
                    detail: format!("unsupported TriPeaks Challenge objective at T=0: {c:?}"),
                })?,
            None => Objective::ClearBoard,
        };
        self.resolved_objective = Some(objective);
        Ok(objective)
    }

    /// Acquire the first `Settled` untouched `T = 0` capture and optional Challenge banner.
    fn capture_first_untouched_root(
        &mut self,
        context: &'static str,
        challenge_out: &mut Option<TriPeaksChallenge>,
    ) -> Result<BoardSnapshot, ReconError> {
        let attempts = self.config.max_restore_observations.max(1);
        for attempt in 0..attempts {
            let frame = self.capture_and_publish(context)?;
            let observed_challenge =
                observe_challenge_header(frame).map_err(|err| ReconError::SceneVerification {
                    detail: format!("Challenge header read failed: {err}"),
                })?;
            match observe_frame(frame).map_err(|err| self.map_op_error(context, err))? {
                FrameClassification::Settled(snap) => {
                    if !snap.is_untouched_root_layout() {
                        return Err(ReconError::SceneVerification {
                            detail: "board is not an untouched TriPeaks T=0 deal (Rows 1..3 face-down, 10 Row-4 face-up, FaceDown Stock, disabled Undo All)".to_owned(),
                        });
                    }
                    *challenge_out = observed_challenge;
                    return Ok(snap);
                }
                FrameClassification::Settling if attempt + 1 < attempts => {
                    self.wait_cancellable(self.config.reobserve_delay, context)?;
                }
                FrameClassification::Settling => {
                    return Err(ReconError::SceneVerification {
                        detail: "initial TriPeaks board stayed Settling past observation budget"
                            .to_owned(),
                    });
                }
                other => {
                    return Err(ReconError::SceneVerification {
                        detail: format!(
                            "expected untouched TriPeaks T=0 board, observed {other:?}"
                        ),
                    });
                }
            }
        }
        Err(ReconError::SceneVerification {
            detail: "exhausted initial T=0 observation budget".to_owned(),
        })
    }

    /// Acquire two consecutive identical untouched `T = 0` captures, leaving capture #2 in `session`.
    fn acquire_twice_confirmed_root(&mut self) -> Result<BoardSnapshot, ReconError> {
        let mut first = self.pre_observed_root.take();
        if first.is_none() {
            let mut challenge = None;
            let snap =
                self.capture_first_untouched_root("initialize_root_first", &mut challenge)?;
            self.resolve_objective_from_challenge(challenge)?;
            first = Some(snap);
        }
        let first_snap = first.ok_or_else(|| ReconError::SceneVerification {
            detail: "missing initial root observation".to_owned(),
        })?;
        let second_snap = self.capture_matching_settled(&first_snap, "initialize_root_second")?;
        if !second_snap.is_untouched_root_layout() {
            return Err(ReconError::SceneVerification {
                detail: "second T=0 observation left untouched root state".to_owned(),
            });
        }
        Ok(second_snap)
    }

    /// Poll briefly until a consecutive capture is `Settled` and matches `expected`.
    fn capture_matching_settled(
        &mut self,
        expected: &BoardSnapshot,
        context: &'static str,
    ) -> Result<BoardSnapshot, ReconError> {
        let attempts = self.config.max_step_observations.max(2);
        for attempt in 0..attempts {
            let frame = self.capture_and_publish(context)?;
            match observe_frame(frame).map_err(|err| self.map_op_error(context, err))? {
                FrameClassification::Settled(snap) if snap == *expected => return Ok(snap),
                FrameClassification::Settled(snap) => {
                    return Err(ReconError::SceneVerification {
                        detail: format!(
                            "unstable TriPeaks T=0 snapshot across consecutive captures (waste {:?} != {:?})",
                            snap.waste, expected.waste
                        ),
                    });
                }
                FrameClassification::Settling if attempt + 1 < attempts => {
                    self.wait_cancellable(self.config.reobserve_delay, context)?;
                }
                other => {
                    return Err(ReconError::SceneVerification {
                        detail: format!(
                            "unexpected scene during consecutive confirmation: {other:?}"
                        ),
                    });
                }
            }
        }
        Err(ReconError::SceneVerification {
            detail: "consecutive settled confirmation timed out".to_owned(),
        })
    }

    /// Execute `23` guarded `D` draws, verify empty Stock at Draw 23, and populate Stock slots `29..52`.
    fn sweep_all_stock_draws(
        &mut self,
        root: &BoardSnapshot,
        slots: &mut [CardSlot; DEAL_SLOT_COUNT],
    ) -> Result<BoardSnapshot, ReconError> {
        let mut current = root.clone();
        for draw_idx in 0..STOCK_CARD_COUNT {
            self.deliver_key(SupportedKey::Draw, "stock_sweep_draw")?;
            self.wait_cancellable(self.config.move_settle_delay, "stock_sweep_settle")?;
            let next = self.poll_stock_draw_step(draw_idx, root, &current)?;
            let target_slot = SlotId(STOCK_START_SLOT + draw_idx);
            check_single_deck_slot_binding(slots, target_slot, next.waste)?;
            slots[usize::from(target_slot.0)] = CardSlot::Known(next.waste);
            current = next;
        }
        Ok(current)
    }

    /// Wait for the post-`D` Waste card transition (preserving Draw-23 empty Stock proof).
    fn poll_stock_draw_step(
        &mut self,
        draw_idx: u8,
        root: &BoardSnapshot,
        prev: &BoardSnapshot,
    ) -> Result<BoardSnapshot, ReconError> {
        let is_final_draw = draw_idx + 1 == STOCK_CARD_COUNT;
        let expected_stock = if is_final_draw {
            StockObs::Empty
        } else {
            StockObs::FaceDown
        };
        let target_slot = SlotId(STOCK_START_SLOT + draw_idx);
        let attempts = self.config.max_step_observations.max(1);

        for attempt in 0..attempts {
            let frame = self.capture_and_publish("stock_sweep_observe")?;
            match observe_frame(frame)
                .map_err(|err| self.map_op_error("stock_sweep_observe", err))?
            {
                FrameClassification::Settled(next) => {
                    if next.tableau != root.tableau {
                        return Err(ReconError::SceneVerification {
                            detail: format!(
                                "tableau changed unexpectedly during Stock draw {}",
                                draw_idx + 1
                            ),
                        });
                    }
                    if next.waste == prev.waste || next.stock != expected_stock {
                        if attempt + 1 < attempts {
                            self.wait_cancellable(self.config.reobserve_delay, "stock_sweep_poll")?;
                            continue;
                        }
                        if next.stock != expected_stock {
                            return Err(ReconError::SceneVerification {
                                detail: format!(
                                    "Stock state mismatch after Draw {} (expected {expected_stock:?}, got {:?})",
                                    draw_idx + 1,
                                    next.stock
                                ),
                            });
                        }
                        return Err(ReconError::VisionClassificationFailed { slot: target_slot });
                    }
                    return Ok(next);
                }
                FrameClassification::Settling if attempt + 1 < attempts => {
                    self.wait_cancellable(self.config.reobserve_delay, "stock_sweep_poll")?;
                }
                FrameClassification::Settling => {
                    return Err(ReconError::VisionClassificationFailed { slot: target_slot });
                }
                other => {
                    return Err(ReconError::SceneVerification {
                        detail: format!(
                            "unexpected overlay {other:?} during Stock draw {}",
                            draw_idx + 1
                        ),
                    });
                }
            }
        }
        Err(ReconError::VisionClassificationFailed { slot: target_slot })
    }

    /// Dismiss any `"No Moves Remaining"` modal, execute guarded `Undo All`, and prove twice-verified
    /// `T = 0` restoration (returning any twice-verified drifted untouched snapshot so callers emit
    /// [`ReconError::RootRestoreMismatch`]).
    fn restore_to_untouched_root(
        &mut self,
        context: &'static str,
        root: &BoardSnapshot,
    ) -> Result<BoardSnapshot, ReconError> {
        let mut driver = self.driver();
        restore_tripeaks_with_driver(&mut driver, context, RootRestoreTarget::ExactSnapshot(root))
    }

    /// Sync the shared `[CardSlot; 52]` cache with `slots` from `recon` (preserving deduction / `Unreachable`).
    fn sync_latest_slots(&self, slots: &[CardSlot]) {
        if let Ok(fixed) = validate_slot_table(slots) {
            self.latest_slots.set(*fixed);
            self.emit_progress(fixed);
        }
    }

    /// Merge verified `(SlotId, Card)` probe observations into `latest_slots` (for `CompletedDuringRecon`).
    fn merge_observations_into_latest(&self, observations: &[(SlotId, Card)]) {
        let mut updated = self.latest_slots.get();
        for &(slot, card) in observations {
            let idx = usize::from(slot.0);
            if idx >= DEAL_SLOT_COUNT || card.0 >= 52 {
                continue;
            }
            let conflict = updated.iter().enumerate().any(|(other_idx, entry)| {
                (other_idx == idx && matches!(*entry, CardSlot::Known(prev) if prev != card))
                    || (other_idx != idx && *entry == CardSlot::Known(card))
            });
            if !conflict {
                updated[idx] = CardSlot::Known(card);
            }
        }
        self.latest_slots.set(updated);
        self.emit_progress(&updated);
    }

    /// Execute one probe move via `Session`, confirm its effect or live completion, and append newly
    /// uncovered `(SlotId, Card)` parent readings.
    fn execute_one_probe_step(
        &mut self,
        mv: TriPeaksMove,
        current: &mut BoardSnapshot,
        executed_prefix: &mut Vec<TriPeaksMove>,
        newly_observed: &mut Vec<(SlotId, Card)>,
    ) -> Result<bool, ReconError> {
        let challenge_mode = self.challenge_mode;
        let outcome = {
            let mut driver = self.driver();
            deliver_and_observe_tripeaks_move(
                &mut driver,
                current,
                TriPeaksStepOptions {
                    mv,
                    challenge_mode,
                    allow_final_modal: false,
                },
                newly_observed,
            )?
        };
        executed_prefix.push(mv);
        match outcome {
            LiveStepOutcome::Settled(next) => {
                *current = next;
                Ok(false)
            }
            LiveStepOutcome::BoardCleared(_)
            | LiveStepOutcome::NextBoardDealt { .. }
            | LiveStepOutcome::BoardWinOverlay
            | LiveStepOutcome::ChallengeComplete
            | LiveStepOutcome::PartialModal => {
                self.merge_observations_into_latest(newly_observed);
                self.completed_probe_moves = Some(executed_prefix.clone());
                Ok(true)
            }
        }
    }
}

/// Ensure single-deck uniqueness before binding `card` into `slots[slot]`.
fn check_single_deck_slot_binding(
    slots: &[CardSlot; DEAL_SLOT_COUNT],
    slot: SlotId,
    card: Card,
) -> Result<(), ReconError> {
    let target_idx = usize::from(slot.0);
    if target_idx >= DEAL_SLOT_COUNT || card.0 >= 52 {
        return Err(ReconError::VisionClassificationFailed { slot });
    }
    if let CardSlot::Known(prev) = slots[target_idx]
        && prev != card
    {
        return Err(ReconError::NonDeterministicSlot {
            slot,
            previous: prev,
            observed: card,
        });
    }
    for (other_idx, entry) in slots.iter().enumerate() {
        if other_idx != target_idx
            && let CardSlot::Known(existing) = *entry
            && existing == card
        {
            let first_u8 = u8::try_from(other_idx).unwrap_or(u8::MAX);
            return Err(ReconError::DuplicateCard {
                first_slot: SlotId(first_u8),
                second_slot: slot,
                card,
            });
        }
    }
    Ok(())
}

/// Record a newly revealed `(slot, card)` parent observation while rejecting mid-probe drift.
fn record_observed_parent(
    newly_observed: &mut Vec<(SlotId, Card)>,
    slot: SlotId,
    card: Card,
) -> Result<(), ReconError> {
    for &(prev_slot, prev_card) in newly_observed.iter() {
        if prev_slot == slot {
            if prev_card != card {
                return Err(ReconError::NonDeterministicSlot {
                    slot,
                    previous: prev_card,
                    observed: card,
                });
            }
            return Ok(());
        }
    }
    newly_observed.push((slot, card));
    Ok(())
}

/// Compute the 28-bit `Empty` mask from a [`BoardSnapshot`].
fn empty_tableau_mask(snapshot: &BoardSnapshot) -> u32 {
    let mut mask = 0_u32;
    for (idx, obs) in snapshot.tableau.iter().enumerate() {
        if *obs == TableauSlotObs::Empty {
            mask |= 1_u32 << idx;
        }
    }
    mask
}

/// Verify an expected single-card tableau removal and harvest any newly uncovered `0..18` parent cards.
fn verify_tableau_step_transition(
    prev: &BoardSnapshot,
    next: &BoardSnapshot,
    tapped_slot_idx: usize,
    tapped_card: Card,
    newly_observed: &mut Vec<(SlotId, Card)>,
) -> Result<(), ReconError> {
    if next.stock != prev.stock {
        return Err(ReconError::SceneVerification {
            detail: "Stock changed unexpectedly during tableau tap".to_owned(),
        });
    }
    if next.tableau[tapped_slot_idx] != TableauSlotObs::Empty {
        return Err(ReconError::SceneVerification {
            detail: format!("tapped tableau slot {tapped_slot_idx} did not become Empty"),
        });
    }
    if next.waste != tapped_card {
        return Err(ReconError::SceneVerification {
            detail: format!(
                "Waste became {:?} instead of tapped card {:?}",
                next.waste, tapped_card
            ),
        });
    }

    let prev_empty = empty_tableau_mask(prev);
    let next_empty = empty_tableau_mask(next);
    for (idx, (&prev_obs, &next_obs)) in prev.tableau.iter().zip(&next.tableau).enumerate() {
        if idx == tapped_slot_idx {
            continue;
        }
        let was_covered = (COVERING_CHILD_MASKS[idx] & !prev_empty) != 0;
        let is_covered = (COVERING_CHILD_MASKS[idx] & !next_empty) != 0;
        let newly_uncovered = was_covered && !is_covered;

        if newly_uncovered {
            match next_obs {
                TableauSlotObs::Card(card) => {
                    let slot_u8 = u8::try_from(idx).unwrap_or(u8::MAX);
                    record_observed_parent(newly_observed, SlotId(slot_u8), card)?;
                }
                TableauSlotObs::FaceDown | TableauSlotObs::Empty => {
                    return Err(ReconError::SceneVerification {
                        detail: format!(
                            "newly uncovered parent slot {idx} resolved to {next_obs:?}"
                        ),
                    });
                }
            }
        } else if next_obs != prev_obs {
            return Err(ReconError::SceneVerification {
                detail: format!(
                    "unrelated tableau slot {idx} changed from {prev_obs:?} to {next_obs:?}"
                ),
            });
        }
    }
    Ok(())
}

impl<I: OperationIo> ReconDomain for LiveTriPeaksDomain<'_, I> {
    type Move = TriPeaksMove;

    fn initialize_and_sweep_stock(&mut self) -> Result<(Vec<CardSlot>, u64), ReconError> {
        let root = self.acquire_twice_confirmed_root()?;
        let hash = semantic_board_hash(&root);
        self.root_snapshot = Some(root.clone());
        self.root_hash = Some(hash);

        let mut slots = [CardSlot::Unknown; DEAL_SLOT_COUNT];
        for (idx, obs) in root.tableau.iter().enumerate() {
            if let TableauSlotObs::Card(card) = *obs {
                let slot_u8 = u8::try_from(idx).unwrap_or(u8::MAX);
                check_single_deck_slot_binding(&slots, SlotId(slot_u8), card)?;
                slots[idx] = CardSlot::Known(card);
            }
        }
        check_single_deck_slot_binding(&slots, INITIAL_WASTE_SLOT, root.waste)?;
        slots[usize::from(INITIAL_WASTE_SLOT.0)] = CardSlot::Known(root.waste);

        let _after_sweep = self.sweep_all_stock_draws(&root, &mut slots)?;
        let restored = self.restore_to_untouched_root("initial_stock_sweep_undo_all", &root)?;
        let restored_hash = semantic_board_hash(&restored);
        if restored_hash != hash {
            return Err(ReconError::RootRestoreMismatch {
                expected: hash,
                actual: restored_hash,
            });
        }

        self.latest_slots.set(slots);
        self.emit_progress(&slots);
        Ok((slots.to_vec(), hash))
    }

    fn solve_known_subgraph(
        &self,
        slots: &[CardSlot],
        objective: Objective,
    ) -> Option<Vec<Self::Move>> {
        self.sync_latest_slots(slots);
        if self.uncertain_latched || self.cancel.load(Ordering::Acquire) {
            return None;
        }
        match solve_known_subgraph_with_control(
            slots,
            objective,
            self.config.search_limits,
            SearchOptions::default(),
            self.cancel,
            None,
            None,
        ) {
            SearchOutcome::Solved { actions, .. } => Some(actions.into_vec()),
            SearchOutcome::Stopped {
                reason: SearchStop::Cancelled,
                ..
            }
            | SearchOutcome::Unsolvable { .. }
            | SearchOutcome::Stopped { .. } => None,
        }
    }

    fn plan_info_gain_probe(&self, slots: &[CardSlot]) -> Option<Vec<Self::Move>> {
        self.sync_latest_slots(slots);
        if self.uncertain_latched || self.cancel.load(Ordering::Acquire) {
            // Return one sentinel move so `run_epistemic_recon` enters `execute_probe_and_observe`
            // and exits immediately with `ReconError::Cancelled` / `UncertainDelivery` rather
            // than mistaking cancellation as `UnsolvableBestPartial`.
            return Some(vec![TriPeaksMove::Draw]);
        }
        let probe = plan_info_gain_probe_with_control(
            slots,
            self.config.search_limits,
            self.cancel,
            None,
            None,
        );
        if self.cancel.load(Ordering::Acquire) {
            return Some(vec![TriPeaksMove::Draw]);
        }
        probe
    }

    fn execute_probe_and_observe(
        &mut self,
        probe: &[Self::Move],
    ) -> Result<Option<Vec<(SlotId, Card)>>, ReconError> {
        self.check_ready("probe_entry")?;
        if probe.is_empty() {
            return Err(ReconError::SceneVerification {
                detail: "planned empty TriPeaks probe".to_owned(),
            });
        }
        let root = self
            .root_snapshot
            .clone()
            .ok_or_else(|| ReconError::SceneVerification {
                detail: "execute_probe_and_observe called before initialize_and_sweep_stock"
                    .to_owned(),
            })?;
        let pre_frame = if self.session.current_frame().is_none() {
            self.capture_and_publish("probe_entry_verify")?
        } else {
            self.session
                .current_frame()
                .ok_or_else(|| ReconError::SceneVerification {
                    detail: "missing current frame at probe entry".to_owned(),
                })?
        };
        match observe_frame(pre_frame)
            .map_err(|err| self.map_op_error("probe_entry_verify", err))?
        {
            FrameClassification::Settled(snap) if snap == root => {}
            other => {
                return Err(ReconError::SceneVerification {
                    detail: format!("probe started away from verified T=0 root state: {other:?}"),
                });
            }
        }

        let mut current = root;
        let mut executed_prefix = Vec::with_capacity(probe.len());
        let mut newly_observed = Vec::new();
        for &mv in probe {
            let completed_live = self.execute_one_probe_step(
                mv,
                &mut current,
                &mut executed_prefix,
                &mut newly_observed,
            )?;
            if completed_live {
                return Ok(None);
            }
        }
        Ok(Some(newly_observed))
    }

    fn undo_all_and_hash(&mut self) -> Result<u64, ReconError> {
        self.check_ready("probe_undo_all")?;
        let root = self
            .root_snapshot
            .clone()
            .ok_or_else(|| ReconError::SceneVerification {
                detail: "undo_all_and_hash called before initialize_and_sweep_stock".to_owned(),
            })?;
        let restored = self.restore_to_untouched_root("probe_undo_all", &root)?;
        self.undo_cycles_completed = self.undo_cycles_completed.saturating_add(1);
        let slots = self.latest_slots.get();
        self.emit_progress(&slots);
        Ok(semantic_board_hash(&restored))
    }

    fn best_partial_route(&self, slots: &[CardSlot]) -> Vec<Self::Move> {
        self.sync_latest_slots(slots);
        if self.uncertain_latched || self.cancel.load(Ordering::Acquire) {
            return Vec::new();
        }
        best_partial_route_with_control(slots, self.config.search_limits, self.cancel, None, None)
    }
}

/// Run the full TriPeaks live reconnaissance pipeline (`initialize_and_sweep_stock` -> probing /
/// `Undo All` -> solver) and optionally publish the validated `52`-slot JSON deal artifact.
///
/// # Errors
///
/// Returns [`TriPeaksReconFailure`] on QMP/recognition/cancellation faults, or on JSON validation
/// or atomic file publication refusal (preserving any pre-existing destination file).
pub fn run_tripeaks_recon<I: OperationIo>(
    domain: &mut LiveTriPeaksDomain<'_, I>,
    destination: Option<&Path>,
) -> Result<TriPeaksReconRunReport, TriPeaksReconFailure> {
    let objective = domain.prepare_initial_observation()?;
    let max_probes = domain.config.max_probes;
    let outcome = crate::recon::run_epistemic_recon(domain, objective, true, max_probes)?;

    if domain.is_uncertain_latched() {
        return Err(TriPeaksReconFailure::Recon(ReconError::UncertainDelivery {
            context: "tripeaks_recon_complete",
            detail: "uncertain QMP delivery prevents JSON publication".to_owned(),
        }));
    }
    if domain.is_cancelled() {
        return Err(TriPeaksReconFailure::Recon(ReconError::Cancelled));
    }

    let root_hash = domain.root_hash().ok_or_else(|| {
        TriPeaksReconFailure::Recon(ReconError::SceneVerification {
            detail: "root hash missing after TriPeaks RECON".to_owned(),
        })
    })?;

    let (provenance, probes_used, planned_route) = match &outcome {
        ReconOutcome::SolvedFromRoot { route, probes_used } => (
            DealProvenance::RestoredAtRoot,
            *probes_used,
            Some(route.clone()),
        ),
        ReconOutcome::CompletedDuringRecon { probes_used } => (
            DealProvenance::CompletedDuringRecon,
            *probes_used,
            domain.completed_probe_moves().map(ToOwned::to_owned),
        ),
        ReconOutcome::UnsolvableBestPartial {
            best_partial_route, ..
        } => (
            DealProvenance::RestoredAtRoot,
            domain.undo_cycles_completed(),
            Some(best_partial_route.clone()),
        ),
    };

    let document = TriPeaksDealDocument {
        provenance,
        probes_used,
        objective,
        slots: domain.latest_slots(),
        planned_route,
    };
    domain.emit_progress(&document.slots);
    document.validate().map_err(PublishDealError::Encode)?;

    if let Some(path) = destination {
        publish_to_path(&document, path)?;
    }

    Ok(TriPeaksReconRunReport {
        outcome,
        objective,
        detected_challenge: domain.detected_challenge(),
        root_hash,
        document,
    })
}
