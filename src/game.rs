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
/// TriPeaks is executable. Pyramid is exposed only for read-only calibration
/// until its geometry and transition evidence have been approved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    TriPeaks,
    Pyramid,
}

impl GameMode {
    pub const AVAILABLE: [Self; 2] = [Self::TriPeaks, Self::Pyramid];

    pub const fn label(self) -> &'static str {
        match self {
            Self::TriPeaks => "TriPeaks",
            Self::Pyramid => "Pyramid",
        }
    }

    pub const fn profile(self) -> &'static GameProfile {
        match self {
            Self::TriPeaks => &crate::tripeaks::PROFILE,
            Self::Pyramid => &crate::pyramid::CALIBRATION_PROFILE,
        }
    }

    /// Whether this mode has enough approved evidence to send guest input.
    pub const fn input_authorised(self) -> bool {
        matches!(self, Self::TriPeaks)
    }

    pub const fn calibration_only(self) -> bool {
        !self.input_authorised()
    }
}

impl fmt::Display for GameMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

/// One labelled rectangle drawn over the read-only preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreviewTarget {
    pub label: &'static str,
    pub bounds: PixelRect,
    pub colour: [u8; 3],
}

/// Parameters for the coarse gameplay-scene discriminator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameplaySceneProfile {
    pub probe_bounds: PixelRect,
    pub green_minimum: u8,
    pub green_red_delta_minimum: u8,
    pub green_blue_delta_minimum: u8,
    pub required_fraction_per_mille: u32,
}

/// Parameters for the visual three-board progress discriminator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameProgressProfile {
    pub right_probe: PixelRect,
    pub black_channel_maximum: u8,
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
    pub label: &'static str,
    pub halo_scan_bounds: PixelRect,
    pub action: ActionSpecification,
}

/// Semantic identity of a calibrated tableau slot, independent of row shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableauTarget {
    pub slot_index: usize,
    pub row: u8,
    pub column: u8,
}

/// Stable identity used to compare fresh pre/post Solver recommendations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionTarget {
    Bottom { index: u8, label: &'static str },
    Tableau(TableauTarget),
}

impl fmt::Display for ActionTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bottom { label, .. } => formatter.write_str(label),
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
    Click(PixelPoint),
    PressDrawKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationClass {
    Draw,
    Tableau,
}

/// Whether seeing the same target after a verified effect is valid.
///
/// Consecutive stock/draw recommendations are normal. A tableau card that is
/// still highlighted after clicking it is not accepted by the TriPeaks
/// profile. Pyramid bottom targets can later use `Allowed` without weakening
/// tableau verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepeatTargetPolicy {
    Allowed,
    MustClear,
}

/// Immutable delivery and verification data attached to a profile target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionSpecification {
    pub operation: InputOperation,
    pub animation_class: AnimationClass,
    pub effect_bounds: PixelRect,
    pub minimum_changed_pixels: usize,
    pub exclude_cursor_from_effect: bool,
    pub repeat_target: RepeatTargetPolicy,
}

/// A complete, typed Solver recommendation derived from one frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuidedAction {
    pub target: ActionTarget,
    pub anchor: PixelPoint,
    pub specification: ActionSpecification,
}

impl GuidedAction {
    pub const fn operation(self) -> InputOperation {
        self.specification.operation
    }

    pub const fn effect_bounds(self) -> PixelRect {
        self.specification.effect_bounds
    }

    pub const fn minimum_changed_pixels(self) -> usize {
        self.specification.minimum_changed_pixels
    }

    pub const fn repeat_target_policy(self) -> RepeatTargetPolicy {
        self.specification.repeat_target
    }

    pub const fn animation_settle_delay(self, delays: AnimationSettleDelays) -> Duration {
        match self.specification.animation_class {
            AnimationClass::Draw => delays.draw,
            AnimationClass::Tableau => delays.tableau,
        }
    }

    pub fn intentional_input_wait(self) -> Duration {
        match self.operation() {
            InputOperation::PressDrawKey => KEY_HOLD,
            InputOperation::Click(_) => POINTER_SETTLE_DELAY + MOUSE_HOLD,
        }
    }

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

    pub const fn qmp_command_count(self) -> usize {
        match self.operation() {
            InputOperation::PressDrawKey => 2,
            InputOperation::Click(_) => 3,
        }
    }

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
    pub mode: GameMode,
    pub label: &'static str,
    pub frame_width: u32,
    pub frame_height: u32,
    pub preview_targets: &'static [PreviewTarget],
    pub gameplay_scene: Option<GameplaySceneProfile>,
    pub game_progress: Option<GameProgressProfile>,
    pub bottom_targets: &'static [BottomTargetProfile],
    pub target_selection: TargetSelectionPolicy,
    pub tableau_cards: &'static [CardRegionPixels],
    pub tableau_rows: &'static [TableauRowScanProfile],
    pub initial_active_rows: u8,
    pub tableau_click_offset: PixelPoint,
    pub minimum_tableau_changed_pixels: usize,
    pub boards_per_game: usize,
}

impl GameProfile {
    pub const fn valid_row_bits(self) -> u8 {
        if self.tableau_rows.len() >= u8::BITS as usize {
            u8::MAX
        } else {
            (1_u8 << self.tableau_rows.len()) - 1
        }
    }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameProgress {
    AnotherBoard,
    GameComplete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tripeaks_remains_the_default_and_only_executable_mode() {
        let mode = GameMode::default();
        let profile = mode.profile();

        assert_eq!(mode, GameMode::TriPeaks);
        assert_eq!(GameMode::AVAILABLE, [GameMode::TriPeaks, GameMode::Pyramid]);
        assert!(GameMode::TriPeaks.input_authorised());
        assert!(!GameMode::TriPeaks.calibration_only());
        assert!(!GameMode::Pyramid.input_authorised());
        assert!(GameMode::Pyramid.calibration_only());
        assert_eq!(profile.mode, mode);
        assert_eq!(profile.label, "TriPeaks");
        assert_eq!(profile.valid_row_bits(), 0b1111);
        assert_eq!(profile.initial_active_rows, 0b1000);
    }

    #[test]
    fn pyramid_profile_cannot_supply_an_action_before_calibration() {
        let profile = GameMode::Pyramid.profile();

        assert_eq!(profile.mode, GameMode::Pyramid);
        assert!(profile.bottom_targets.is_empty());
        assert!(profile.tableau_cards.is_empty());
        assert!(profile.tableau_rows.is_empty());
        assert_eq!(profile.gameplay_scene, None);
        assert_eq!(profile.game_progress, None);
        assert_eq!(profile.initial_active_rows, 0);
        assert_eq!(profile.valid_row_bits(), 0);
    }

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
    }
}
