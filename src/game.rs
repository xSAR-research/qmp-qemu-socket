//! Shared game contracts for calibrated actions, scene evidence and completion progress.
//! Mode-specific geometry lives in each game's module; execution lives in `worker`.

use std::{fmt, time::Duration};

use crate::{
    cards::CardRegionPixels,
    geometry::{PixelPoint, PixelRect},
    parameters::{
        ACTION_CURSOR_EXCLUSION_HALF_SIZE, AnimationSettleDelays, KEY_HOLD, MOUSE_HOLD,
        POINTER_SETTLE_DELAY, TableauRowScanProfile,
    },
};


/// User-selectable game identity.
///
/// Each mode supplies its own target detector and effect evidence while the
/// worker owns shared input delivery and the verified transition sequence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GameMode {
    /// Three-peak tableau controlled by a unique Solver recommendation.
    #[default]
    TriPeaks,
    /// Seven-row tableau with Move, Left and Right priority targets.
    Pyramid,
    /// Changing seven-column tableau, Draw 1 stock and fanned waste.
    Klondike,
    /// Eight face-up columns, four temporary cells and four suit piles.
    FreeCell,
}


impl GameMode {


    /// Modes exposed by the game selector, with the default first.
    pub const AVAILABLE: [Self; 4] = [Self::TriPeaks, Self::Pyramid, Self::Klondike, Self::FreeCell];


    /// Return the stable game name used in controls and logs.
    pub const fn label(self) -> &'static str {


        match self {
            Self::TriPeaks => "TriPeaks",
            Self::Pyramid => "Pyramid",
            Self::Klondike => "Klondike",
            Self::FreeCell => "Free Cell",
        }
    }


    /// Select the immutable geometry and detection policy for this game.
    pub const fn profile(self) -> &'static GameProfile {


        match self {
            Self::TriPeaks => &crate::tripeaks::PROFILE,
            Self::Pyramid => &crate::pyramid::PROFILE,
            Self::Klondike => &crate::klondike::PROFILE,
            Self::FreeCell => &crate::freecell::PROFILE,
        }
    }


    /// Whether this mode has enough approved evidence to send guest input.
    pub const fn input_authorised(self) -> bool {
        matches!(self, Self::TriPeaks | Self::Pyramid | Self::Klondike | Self::FreeCell)
    }


    /// Report whether the mode is limited to preview without authorised input.
    pub const fn calibration_only(self) -> bool {
        !self.input_authorised()
    }
}


impl fmt::Display for GameMode {


    /// Format the game using its stable UI label.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}


/// One labelled rectangle drawn over the read-only preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreviewTarget {
    /// Short label displayed beside the preview rectangle.
    pub label: &'static str,
    /// Native frame-pixel bounds of the target overlay.
    pub bounds: PixelRect,
    /// RGB stroke colour used only for the preview overlay.
    pub colour: [u8; 3],
}


/// Parameters for the coarse gameplay-scene discriminator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameplaySceneProfile {
    /// Frame region sampled to distinguish green gameplay from modal scenes.
    pub probe_bounds: PixelRect,
    /// Minimum green-channel brightness for a felt pixel.
    pub green_minimum: u8,
    /// Minimum green-minus-red separation for a felt pixel.
    pub green_red_delta_minimum: u8,
    /// Minimum green-minus-blue separation for a felt pixel.
    pub green_blue_delta_minimum: u8,
    /// Required felt-pixel share in thousandths of the probe area.
    pub required_fraction_per_mille: u32,
}


/// Parameters for the visual three-board progress discriminator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameProgressProfile {
    /// Interior region of the right third of the three-board progress bar.
    pub right_probe: PixelRect,
    /// Largest permitted channel value for a pixel classified as unfilled black.
    pub black_channel_maximum: u8,
    /// Minimum black-pixel share that classifies the bar as another board.
    pub black_fraction_per_mille: u32,
}


/// How actionable targets are selected from one immutable frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetSelectionPolicy {
    /// Collect every calibrated hit and fail closed unless exactly one exists.
    UniqueAcrossFrame,
    /// Stop at the first hit in profile order, then scan rows lower-to-upper.
    FirstByPriority,
}


/// One lower-panel target checked before tableau rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BottomTargetProfile {
    /// Stable target label used in predictions and diagnostic messages.
    pub label: &'static str,
    /// Calibrated lower-panel region searched for a qualifying halo run.
    pub halo_scan_bounds: PixelRect,
    /// Fixed input and verification policy attached to this target.
    pub action: ActionSpecification,
}


/// Semantic identity of a calibrated tableau slot, independent of row shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableauTarget {
    /// Zero-based index into the profile's calibrated card array.
    pub slot_index: usize,
    /// One-based tableau row, counted from the top.
    pub row: u8,
    /// One-based card position within the row.
    pub column: u8,
}


/// Stable identity used to compare fresh pre/post Solver recommendations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionTarget {
    /// A TriPeaks lower-panel target from the selected profile.
    Bottom {
        /// Index into the profile's lower-panel target array.
        index: u8,
        /// Stable target name used in diagnostics.
        label: &'static str,
    },
    /// A calibrated TriPeaks tableau card.
    Tableau(TableauTarget),
    /// A target managed by Pyramid's independent detector.
    Pyramid(crate::pyramid::PyramidTargetKind),
    /// A source block or stock operation managed by Klondike's detector.
    Klondike(crate::klondike::KlondikeTarget),
    /// A CELL card or PLAY source run managed by Free Cell's detector.
    FreeCell(crate::freecell::FreeCellTarget),
}


impl ActionTarget {


    /// Identify which game calibration owns this action target.
    pub const fn mode(self) -> GameMode {


        match self {
            Self::Bottom { .. } | Self::Tableau(_) => GameMode::TriPeaks,
            Self::Pyramid(_) => GameMode::Pyramid,
            Self::Klondike(_) => GameMode::Klondike,
            Self::FreeCell(_) => GameMode::FreeCell,
        }
    }
}


impl fmt::Display for ActionTarget {


    /// Format the selected target with enough geometry to identify it in logs.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {


        match self {
            Self::Bottom { label, .. } => formatter.write_str(label),
            Self::Pyramid(kind) => write!(formatter, "Pyramid {kind}"),
            Self::Klondike(kind) => write!(formatter, "Klondike {kind}"),
            Self::FreeCell(kind) => write!(formatter, "Free Cell {kind}"),
            Self::Tableau(target) => write!(
                formatter,
                "tableau row {}, column {}",
                target.row, target.column
            ),
        }
    }
}


/// The one non-idempotent guest-input operation selected for an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputOperation {
    /// Move to a calibrated point, press and release the left mouse button.
    Click(PixelPoint),
    /// Press and release D for TriPeaks/Klondike Draw or Pyramid Move/Recycle.
    PressDrawKey,
}


/// Per-action settling category selected from the immutable run settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationClass {
    /// TriPeaks stock draw animation.
    Draw,
    /// TriPeaks tableau-card removal animation.
    Tableau,
    /// Pyramid Move or Recycle animation.
    PyramidMove,
    /// Pyramid tableau or lower-pile card match animation.
    PyramidCard,
    /// Klondike draw, recycle or source-block transfer animation.
    Klondike,
    /// Klondike auto-finish animation, with its own editable settling interval.
    KlondikeSolve,
    /// Free Cell source click followed by any automatic SUIT transfers.
    FreeCell,
}


/// Whether seeing the same target after a verified effect is valid.
///
/// Consecutive stock/draw recommendations are normal. A tableau card that is
/// still highlighted after clicking it is not accepted by the TriPeaks
/// profile. Pyramid lower-panel targets use `Allowed`; its worker distinguishes
/// verified effects from continuation authorised by fresh settled pile halos.
/// Its tableau effect verification requires positive removal evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepeatTargetPolicy {
    /// The next recommendation may have the same target identity.
    Allowed,
    /// A verified action must no longer recommend the clicked tableau slot.
    MustClear,
}


/// Immutable delivery and verification data attached to a profile target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionSpecification {
    /// Exactly one non-idempotent key or click operation to attempt.
    pub operation: InputOperation,
    /// Delay category applied after input delivery.
    pub animation_class: AnimationClass,
    /// Native frame region compared to measure visual change.
    pub effect_bounds: PixelRect,
    /// Required count of pixels exceeding the material-change colour threshold.
    pub minimum_changed_pixels: usize,
    /// Whether a square around a click is omitted from effect measurements.
    pub exclude_cursor_from_effect: bool,
    /// Policy for an identical next recommendation after a verified effect.
    pub repeat_target: RepeatTargetPolicy,
}


/// A complete, typed Solver recommendation derived from one frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuidedAction {
    /// Stable identity used to compare planned and freshly observed recommendations.
    pub target: ActionTarget,
    /// Detected or canonical halo anchor that justified the recommendation.
    pub anchor: PixelPoint,
    /// Immutable input, delay and visual-effect requirements.
    pub specification: ActionSpecification,
}


impl GuidedAction {


    /// Return the single guest-input operation associated with the recommendation.
    pub const fn operation(self) -> InputOperation {
        self.specification.operation
    }


    /// Return the calibrated region used to measure the action's visual effect.
    pub const fn effect_bounds(self) -> PixelRect {
        self.specification.effect_bounds
    }


    /// Return the profile's required number of materially changed effect pixels.
    pub const fn minimum_changed_pixels(self) -> usize {
        self.specification.minimum_changed_pixels
    }


    /// Return whether the same recommendation may remain after a verified effect.
    pub const fn repeat_target_policy(self) -> RepeatTargetPolicy {
        self.specification.repeat_target
    }


    /// Select the configured delay for this action class from the run settings.
    pub const fn animation_settle_delay(self, delays: AnimationSettleDelays) -> Duration {


        match self.specification.animation_class {
            AnimationClass::Draw => delays.draw,
            AnimationClass::Tableau => delays.tableau,
            AnimationClass::PyramidMove => delays.pyramid_move,
            AnimationClass::PyramidCard => delays.pyramid_card,
            AnimationClass::Klondike => delays.klondike_settle,
            AnimationClass::KlondikeSolve => delays.klondike_solve,
            AnimationClass::FreeCell => delays.freecell_settle,
        }
    }


    /// Total the deliberate key hold or pointer settling and mouse hold time.
    pub fn intentional_input_wait(self) -> Duration {


        match self.operation() {
            InputOperation::PressDrawKey => KEY_HOLD,
            InputOperation::Click(_) => POINTER_SETTLE_DELAY + MOUSE_HOLD,
        }
    }


    /// Return the cursor exclusion square for click effects when the profile requests it.
    pub const fn effect_exclusion_bounds(self) -> Option<PixelRect> {


        let InputOperation::Click(click_point) = self.operation() else {
            return None;
        };


        if !self.specification.exclude_cursor_from_effect {
            return None;
        }

        let side = ACTION_CURSOR_EXCLUSION_HALF_SIZE * 2;
        Some(PixelRect::new(
            (click_point.x as u32).saturating_sub(ACTION_CURSOR_EXCLUSION_HALF_SIZE),
            (click_point.y as u32).saturating_sub(ACTION_CURSOR_EXCLUSION_HALF_SIZE),
            side,
            side,
        ))
    }


    /// Count QMP commands in one successful, unretried input sequence.
    pub const fn qmp_command_count(self) -> usize {


        match self.operation() {
            InputOperation::PressDrawKey => 2,
            InputOperation::Click(_) => 3,
        }
    }


    /// Count guest input events carried by one successful input sequence.
    pub const fn qmp_event_count(self) -> usize {


        match self.operation() {
            InputOperation::PressDrawKey => 2,
            InputOperation::Click(_) => 4,
        }
    }
}


/// The smallest game boundary needed by the guarded controller.
#[derive(Clone, Copy, Debug)]
pub struct GameProfile {
    /// Human-readable profile name.
    pub label: &'static str,
    /// Required native capture width in pixels.
    pub frame_width: u32,
    /// Required native capture height in pixels.
    pub frame_height: u32,
    /// Read-only overlays shown on the captured image.
    pub preview_targets: &'static [PreviewTarget],
    /// Optional coarse felt discriminator used before accepting gameplay evidence.
    pub gameplay_scene: Option<GameplaySceneProfile>,
    /// Optional shared progress-bar calibration for established completion contexts.
    pub game_progress: Option<GameProgressProfile>,
    /// Lower-panel halo-run targets checked before TriPeaks tableau rows.
    pub bottom_targets: &'static [BottomTargetProfile],
    /// Policy for selecting among simultaneously detected targets.
    pub target_selection: TargetSelectionPolicy,
    /// TriPeaks slot geometry; Pyramid uses its independent target table.
    pub tableau_cards: &'static [CardRegionPixels],
    /// TriPeaks row scan windows and face probes.
    pub tableau_rows: &'static [TableauRowScanProfile],
    /// Logical row count, including modes with independent target geometry.
    pub tableau_row_count: u8,
    /// Initial row hint encoded with bit zero representing row one.
    pub initial_active_rows: u8,
    /// Audited TriPeaks offset from halo anchor to canonical click centre.
    pub tableau_click_offset: PixelPoint,
    /// TriPeaks effect threshold applied to a clicked tableau card.
    pub minimum_tableau_changed_pixels: usize,
    /// Number of completed boards in a game; Klondike's single-board policy is
    /// independently verified and never enters the shared terminal controller.
    pub boards_per_game: usize,
}


impl GameProfile {


    /// Return the number of tableau rows represented by the selected mode.
    pub const fn row_count(self) -> u8 {
        self.tableau_row_count
    }


    /// Build the valid row-bit mask used by calibration and stale-state tests.
    #[cfg(test)]
    pub const fn valid_row_bits(self) -> u8 {


        if self.row_count() >= u8::BITS as u8 {
            u8::MAX
        } else {
            (1_u8 << self.row_count()) - 1
        }
    }


    /// Resolve a TriPeaks slot index to its one-based row and column, rejecting missing slots.
    pub fn tableau_target(self, slot_index: usize) -> Option<TableauTarget> {
        self.tableau_rows.iter().find_map(|row| {
            let end = row.card_end_index();


            if slot_index < row.first_card_index || slot_index >= end {
                return None;
            }
            let column = u8::try_from(slot_index - row.first_card_index + 1).ok()?;
            Some(TableauTarget {
                slot_index,
                row: row.row,
                column,
            })
        })
    }


    /// Build the canonical TriPeaks click and effect specification for a calibrated slot.
    pub fn tableau_action(self, slot_index: usize, anchor: PixelPoint) -> Option<GuidedAction> {
        let target = self.tableau_target(slot_index)?;
        let regions = *self.tableau_cards.get(slot_index)?;
        Some(GuidedAction {
            target: ActionTarget::Tableau(target),
            anchor,
            specification: ActionSpecification {
                operation: InputOperation::Click(regions.click_point),
                animation_class: AnimationClass::Tableau,
                effect_bounds: regions.card_bounds,
                minimum_changed_pixels: self.minimum_tableau_changed_pixels,
                exclude_cursor_from_effect: true,
                repeat_target: RepeatTargetPolicy::MustClear,
            },
        })
    }


    /// Build a typed lower-panel action from a checked profile index and observed anchor.
    pub fn bottom_action(self, index: usize, anchor: PixelPoint) -> Option<GuidedAction> {
        let target = *self.bottom_targets.get(index)?;
        let index = u8::try_from(index).ok()?;
        Some(GuidedAction {
            target: ActionTarget::Bottom {
                index,
                label: target.label,
            },
            anchor,
            specification: target.action,
        })
    }
}


/// Visual interpretation of the shared progress bar, not independent proof of completion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameProgress {
    /// The right third remains sufficiently black to indicate unfinished game progress.
    AnotherBoard,
    /// The right third is filled; a separate completion context is still required.
    GameComplete,
}


#[cfg(test)]
mod tests {
    //! Profile boundaries and action policies shared by the game modes.
    use super::*;


    /// Klondike cannot inherit the established modes' three-board progress authority.
    #[test]
    fn klondike_has_independent_geometry_and_no_shared_progress() {
        let profile = GameMode::Klondike.profile();
        assert_eq!(profile.label, "Klondike");
        assert!(profile.game_progress.is_none());
        assert!(profile.tableau_cards.is_empty());
        assert!(profile.tableau_rows.is_empty());
        let draw = crate::klondike::canonical_action(crate::klondike::KlondikeTarget::Draw)
            .expect("Draw has a canonical action");
        assert_eq!(draw.target.mode(), GameMode::Klondike);
        assert_eq!(draw.operation(), InputOperation::PressDrawKey);
    }


    /// Keep the initial mode, selectable modes and authorised calibration contracts stable.


    #[test]
    fn tripeaks_remains_the_default_with_four_independent_modes() {
        let mode = GameMode::default();
        let profile = mode.profile();

        assert_eq!(mode, GameMode::TriPeaks);
        assert_eq!(GameMode::AVAILABLE, [GameMode::TriPeaks, GameMode::Pyramid, GameMode::Klondike, GameMode::FreeCell]);
        assert!(GameMode::TriPeaks.input_authorised());
        assert!(!GameMode::TriPeaks.calibration_only());
        assert!(GameMode::Pyramid.input_authorised());
        assert!(!GameMode::Pyramid.calibration_only());
        assert!(GameMode::Klondike.input_authorised());
        assert!(!GameMode::Klondike.calibration_only());
        assert!(GameMode::FreeCell.input_authorised());
        assert!(!GameMode::FreeCell.calibration_only());
        assert_eq!(profile.label, "TriPeaks");
        assert_eq!(profile.valid_row_bits(), 0b1111);
        assert_eq!(profile.initial_active_rows, 0b1000);
    }


    /// Keep Pyramid target analysis separate while sharing the accepted progress probe.


    #[test]
    fn pyramid_uses_its_own_detector_and_the_shared_progress_calibration() {
        let profile = GameMode::Pyramid.profile();

        assert_eq!(profile.label, "Pyramid");
        assert!(profile.bottom_targets.is_empty());
        assert!(profile.tableau_cards.is_empty());
        assert!(profile.tableau_rows.is_empty());
        assert_eq!(
            profile.game_progress,
            GameMode::TriPeaks.profile().game_progress
        );
        assert_eq!(profile.valid_row_bits(), 0b111_1111);
        assert_eq!(profile.row_count(), 7);
        assert_eq!(
            ActionTarget::Pyramid(crate::pyramid::PyramidTargetKind::Move).mode(),
            GameMode::Pyramid
        );
    }


    /// Check TriPeaks repeat policies and per-action configured settling delays.


    #[test]
    fn repeated_bottom_actions_and_tableau_actions_have_distinct_policies() {
        let profile = *GameMode::TriPeaks.profile();
        let bottom = profile.bottom_action(0, PixelPoint::new(842, 868)).unwrap();
        let tableau = profile
            .tableau_action(27, PixelPoint::new(1_662, 630))
            .unwrap();

        assert_eq!(bottom.repeat_target_policy(), RepeatTargetPolicy::Allowed);
        assert_eq!(
            tableau.repeat_target_policy(),
            RepeatTargetPolicy::MustClear
        );
        assert_eq!(bottom.operation(), InputOperation::PressDrawKey);
        assert!(matches!(tableau.operation(), InputOperation::Click(_)));

        let delays =
            AnimationSettleDelays::from_millis(120, 340, 560).with_pyramid_millis(725, 1_350, 950);
        assert_eq!(
            bottom.animation_settle_delay(delays),
            Duration::from_millis(120)
        );
        assert_eq!(
            tableau.animation_settle_delay(delays),
            Duration::from_millis(340)
        );
    }
}
