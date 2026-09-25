use crate::{
    game::{GameMode, GameProfile, PreviewTarget, TargetSelectionPolicy},
    geometry::{PixelPoint, PixelRect},
    parameters::{NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH},
};

pub const TARGET_COUNT: usize = 31;
pub const TABLEAU_CARD_COUNT: usize = 28;

/// Semantic identity and priority for one future Pyramid Solver target.
///
/// Card rows use the conventional Pyramid numbering: row 1 is the apex and
/// row 7 is the bottom. The constant array deliberately scans row 7 first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PyramidTargetKind {
    Move,
    Left,
    Right,
    Card { row: u8, column: u8 },
}

impl PyramidTargetKind {
    pub const fn is_card(self) -> bool {
        matches!(self, Self::Card { .. })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PyramidTargetSlot {
    pub kind: PyramidTargetKind,
}

const fn slot(kind: PyramidTargetKind) -> PyramidTargetSlot {
    PyramidTargetSlot { kind }
}

/// Approved semantic order only. Halo probes and click points are withheld
/// until screenshot calibration establishes exact guest-pixel coordinates.
pub const PYRAMID_TARGETS: [PyramidTargetSlot; TARGET_COUNT] = [
    slot(PyramidTargetKind::Move),
    slot(PyramidTargetKind::Left),
    slot(PyramidTargetKind::Right),
    slot(PyramidTargetKind::Card { row: 7, column: 1 }),
    slot(PyramidTargetKind::Card { row: 7, column: 2 }),
    slot(PyramidTargetKind::Card { row: 7, column: 3 }),
    slot(PyramidTargetKind::Card { row: 7, column: 4 }),
    slot(PyramidTargetKind::Card { row: 7, column: 5 }),
    slot(PyramidTargetKind::Card { row: 7, column: 6 }),
    slot(PyramidTargetKind::Card { row: 7, column: 7 }),
    slot(PyramidTargetKind::Card { row: 6, column: 1 }),
    slot(PyramidTargetKind::Card { row: 6, column: 2 }),
    slot(PyramidTargetKind::Card { row: 6, column: 3 }),
    slot(PyramidTargetKind::Card { row: 6, column: 4 }),
    slot(PyramidTargetKind::Card { row: 6, column: 5 }),
    slot(PyramidTargetKind::Card { row: 6, column: 6 }),
    slot(PyramidTargetKind::Card { row: 5, column: 1 }),
    slot(PyramidTargetKind::Card { row: 5, column: 2 }),
    slot(PyramidTargetKind::Card { row: 5, column: 3 }),
    slot(PyramidTargetKind::Card { row: 5, column: 4 }),
    slot(PyramidTargetKind::Card { row: 5, column: 5 }),
    slot(PyramidTargetKind::Card { row: 4, column: 1 }),
    slot(PyramidTargetKind::Card { row: 4, column: 2 }),
    slot(PyramidTargetKind::Card { row: 4, column: 3 }),
    slot(PyramidTargetKind::Card { row: 4, column: 4 }),
    slot(PyramidTargetKind::Card { row: 3, column: 1 }),
    slot(PyramidTargetKind::Card { row: 3, column: 2 }),
    slot(PyramidTargetKind::Card { row: 3, column: 3 }),
    slot(PyramidTargetKind::Card { row: 2, column: 1 }),
    slot(PyramidTargetKind::Card { row: 2, column: 2 }),
    slot(PyramidTargetKind::Card { row: 1, column: 1 }),
];

// These two envelopes are read-only positioning aids, not detector or input
// coordinates. Exact 2x2 halo probes and click points remain uncalibrated.
const TABLEAU_CALIBRATION_BOUNDS: PixelRect = PixelRect::new(112, 96, 1_696, 544);
const LOWER_PANEL_CALIBRATION_BOUNDS: PixelRect = PixelRect::new(680, 620, 560, 340);

const PREVIEW_TARGETS: [PreviewTarget; 2] = [
    PreviewTarget {
        label: "pyramid-tableau-calibration",
        bounds: TABLEAU_CALIBRATION_BOUNDS,
        colour: [48, 180, 255],
    },
    PreviewTarget {
        label: "pyramid-lower-calibration",
        bounds: LOWER_PANEL_CALIBRATION_BOUNDS,
        colour: [255, 192, 48],
    },
];

/// Read-only profile used to collect Pyramid calibration evidence. Its action
/// target lists are empty, and the worker independently rejects input for the
/// mode, so these broad preview envelopes cannot become click authority.
pub const CALIBRATION_PROFILE: GameProfile = GameProfile {
    mode: GameMode::Pyramid,
    label: "Pyramid (calibration only)",
    frame_width: NOMINAL_FRAME_WIDTH,
    frame_height: NOMINAL_FRAME_HEIGHT,
    preview_targets: &PREVIEW_TARGETS,
    gameplay_scene: None,
    game_progress: None,
    bottom_targets: &[],
    target_selection: TargetSelectionPolicy::FirstByPriority,
    tableau_cards: &[],
    tableau_rows: &[],
    initial_active_rows: 0,
    tableau_click_offset: PixelPoint::new(0, 0),
    minimum_tableau_changed_pixels: 0,
    boards_per_game: 3,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_order_is_move_left_right_then_bottom_to_apex() {
        assert_eq!(PYRAMID_TARGETS.len(), TARGET_COUNT);
        assert_eq!(
            PYRAMID_TARGETS[..3],
            [
                slot(PyramidTargetKind::Move),
                slot(PyramidTargetKind::Left),
                slot(PyramidTargetKind::Right),
            ]
        );
        assert_eq!(
            PYRAMID_TARGETS[3].kind,
            PyramidTargetKind::Card { row: 7, column: 1 }
        );
        assert_eq!(
            PYRAMID_TARGETS[TARGET_COUNT - 1].kind,
            PyramidTargetKind::Card { row: 1, column: 1 }
        );
        assert_eq!(
            PYRAMID_TARGETS
                .iter()
                .filter(|target| target.kind.is_card())
                .count(),
            TABLEAU_CARD_COUNT
        );
    }

    #[test]
    fn every_card_slot_has_a_valid_pyramid_coordinate() {
        for target in &PYRAMID_TARGETS[3..] {
            let PyramidTargetKind::Card { row, column } = target.kind else {
                panic!("non-card target appeared after the lower-panel targets");
            };
            assert!((1..=7).contains(&row));
            assert!((1..=row).contains(&column));
        }
    }
}
