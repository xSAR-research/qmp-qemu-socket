use crate::{
    game::{GameMode, GameProfile, PreviewTarget, TargetSelectionPolicy},
    geometry::{PixelPoint, PixelRect},
    parameters::{NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH},
};

pub const TARGET_COUNT: usize = 31;
pub const TABLEAU_CARD_COUNT: usize = 28;

/// Semantic identity and future scan priority for a Pyramid target.
/// Rows run from the apex (1) to the bottom (7); columns run left to right.
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

/// Read-only guest-pixel geometry. It does not grant guest-input authority.
/// Bounds are half-open visible-face envelopes, excluding shadows and halos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PyramidTargetSlot {
    pub kind: PyramidTargetKind,
    pub label: &'static str,
    pub bounds: PixelRect,
    pub click_point: PixelPoint,
}

const fn target(
    kind: PyramidTargetKind,
    label: &'static str,
    bounds: PixelRect,
    click_point: PixelPoint,
) -> PyramidTargetSlot {
    PyramidTargetSlot {
        kind,
        label,
        bounds,
        click_point,
    }
}

// Measured from Issue #1's 1920x1080 captures. The envelope includes the
// one-pixel variation in antialiased card edges. See docs/pyramid-calibration.md.
pub const CARD_FACE_WIDTH: u32 = 140;
pub const CARD_FACE_HEIGHT: u32 = 187;

const fn card(row: u8, column: u8, label: &'static str, x: u32, y: u32) -> PyramidTargetSlot {
    target(
        PyramidTargetKind::Card { row, column },
        label,
        PixelRect::new(x, y, CARD_FACE_WIDTH, CARD_FACE_HEIGHT),
        // The top strip remains visible while lower rows overlap the card.
        // This point is geometry only; it says nothing about whether a card
        // is uncovered, highlighted or legal to play.
        PixelPoint::new((x + 70) as i32, (y + 24) as i32),
    )
}

/// The same visual control moves a card or recycles the pile. Geometry alone
/// cannot distinguish the operation; no automatic Move/Recycle is enabled.
pub const MOVE_TARGET: PyramidTargetSlot = target(
    PyramidTargetKind::Move,
    "PY Move/Recycle",
    PixelRect::new(920, 678, 80, 80),
    PixelPoint::new(960, 718),
);
pub const LEFT_TARGET: PyramidTargetSlot = target(
    PyramidTargetKind::Left,
    "PY Left",
    PixelRect::new(759, 678, 139, 187),
    PixelPoint::new(828, 771),
);
pub const RIGHT_TARGET: PyramidTargetSlot = target(
    PyramidTargetKind::Right,
    "PY Right",
    PixelRect::new(1_022, 678, 139, 187),
    PixelPoint::new(1_091, 771),
);

/// Retain the agreed semantic order: Move, Left, Right, then bottom to apex.
/// These coordinates are preview metadata, not detector probes or actions.
pub const PYRAMID_TARGETS: [PyramidTargetSlot; TARGET_COUNT] = [
    MOVE_TARGET,
    LEFT_TARGET,
    RIGHT_TARGET,
    card(7, 1, "PY r7c1", 288, 432),
    card(7, 2, "PY r7c2", 489, 432),
    card(7, 3, "PY r7c3", 690, 432),
    card(7, 4, "PY r7c4", 890, 432),
    card(7, 5, "PY r7c5", 1_091, 432),
    card(7, 6, "PY r7c6", 1_291, 432),
    card(7, 7, "PY r7c7", 1_492, 432),
    card(6, 1, "PY r6c1", 389, 378),
    card(6, 2, "PY r6c2", 589, 378),
    card(6, 3, "PY r6c3", 790, 378),
    card(6, 4, "PY r6c4", 990, 378),
    card(6, 5, "PY r6c5", 1_191, 378),
    card(6, 6, "PY r6c6", 1_392, 378),
    card(5, 1, "PY r5c1", 489, 325),
    card(5, 2, "PY r5c2", 690, 325),
    card(5, 3, "PY r5c3", 890, 325),
    card(5, 4, "PY r5c4", 1_091, 325),
    card(5, 5, "PY r5c5", 1_291, 325),
    card(4, 1, "PY r4c1", 589, 272),
    card(4, 2, "PY r4c2", 790, 272),
    card(4, 3, "PY r4c3", 990, 272),
    card(4, 4, "PY r4c4", 1_191, 272),
    card(3, 1, "PY r3c1", 690, 218),
    card(3, 2, "PY r3c2", 890, 218),
    card(3, 3, "PY r3c3", 1_091, 218),
    card(2, 1, "PY r2c1", 790, 165),
    card(2, 2, "PY r2c2", 990, 165),
    card(1, 1, "PY r1c1", 890, 112),
];

const fn preview_targets() -> [PreviewTarget; TARGET_COUNT] {
    let mut targets = [PreviewTarget {
        label: "",
        bounds: PixelRect::new(0, 0, 0, 0),
        colour: [80, 190, 255],
    }; TARGET_COUNT];
    let mut index = 0;
    while index < TARGET_COUNT {
        let slot = PYRAMID_TARGETS[index];
        targets[index] = PreviewTarget {
            label: slot.label,
            bounds: slot.bounds,
            colour: if slot.kind.is_card() {
                [80, 190, 255]
            } else {
                [255, 192, 48]
            },
        };
        index += 1;
    }
    targets
}

pub const PREVIEW_TARGETS: [PreviewTarget; TARGET_COUNT] = preview_targets();

/// Read-only profile used to collect Pyramid calibration evidence. Its action
/// target lists are empty, and the worker independently rejects input for the
/// mode, so the measured preview geometry cannot become click authority.
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
    use crate::geometry::{pixel_point_to_qmp, pixel_rect_to_qmp};

    #[test]
    fn semantic_order_covers_every_card_once_from_bottom_to_apex() {
        assert_eq!(PYRAMID_TARGETS[..3], [MOVE_TARGET, LEFT_TARGET, RIGHT_TARGET]);
        let mut index = 3;
        for row in (1_u8..=7).rev() {
            for column in 1..=row {
                assert_eq!(
                    PYRAMID_TARGETS[index].kind,
                    PyramidTargetKind::Card { row, column }
                );
                index += 1;
            }
        }
        assert_eq!(index, TARGET_COUNT);
        assert_eq!(index - 3, TABLEAU_CARD_COUNT);
    }

    #[test]
    fn every_bound_and_hit_point_is_inside_the_frame_and_qmp_mappable() {
        for target in PYRAMID_TARGETS {
            assert!(target.bounds.contains(target.click_point), "{}", target.label);
            assert!(
                pixel_rect_to_qmp(target.bounds, NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT).is_ok()
            );
            assert!(
                pixel_point_to_qmp(target.click_point, NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT)
                    .is_ok()
            );
        }
    }

    #[test]
    fn tableau_hit_points_avoid_overlapping_lower_rows() {
        for target in PYRAMID_TARGETS {
            let PyramidTargetKind::Card { row, .. } = target.kind else {
                continue;
            };
            for other in PYRAMID_TARGETS {
                if let PyramidTargetKind::Card { row: other_row, .. } = other.kind {
                    if other_row > row {
                        assert!(
                            !other.bounds.contains(target.click_point),
                            "{} overlaps {}",
                            target.label,
                            other.label
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn precise_preview_metadata_does_not_authorise_pyramid_input() {
        assert!(!GameMode::Pyramid.input_authorised());
        assert!(CALIBRATION_PROFILE.bottom_targets.is_empty());
        assert!(CALIBRATION_PROFILE.tableau_cards.is_empty());
        assert!(CALIBRATION_PROFILE.tableau_rows.is_empty());
        assert_eq!(PREVIEW_TARGETS.len(), TARGET_COUNT);
        for (preview, target) in PREVIEW_TARGETS.iter().zip(PYRAMID_TARGETS) {
            assert_eq!(preview.bounds, target.bounds);
            assert_eq!(preview.label, target.label);
        }
    }
}
