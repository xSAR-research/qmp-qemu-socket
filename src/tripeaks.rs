//! TriPeaks profile assembled from the audited geometry and shared game contracts.
//! The worker and tracker consume this immutable profile without owning its calibration.

use crate::{
    game::{
        ActionSpecification, AnimationClass, BottomTargetProfile, GameProfile,
        GameplaySceneProfile, InputOperation, PreviewTarget, RepeatTargetPolicy,
        TargetSelectionPolicy,
    },
    geometry::PixelPoint,
    parameters::{
        BOARDS_PER_GAME, CLICK_OFFSET_X, CLICK_OFFSET_Y, DRAW_EFFECT_BOUNDS,
        GAMEPLAY_FELT_GREEN_BLUE_DELTA_MINIMUM, GAMEPLAY_FELT_GREEN_MINIMUM,
        GAMEPLAY_FELT_GREEN_RED_DELTA_MINIMUM, GAMEPLAY_FELT_PROBE_BOUNDS,
        GAMEPLAY_FELT_REQUIRED_FRACTION_PER_MILLE, MINIMUM_DRAW_CHANGED_PIXELS,
        MINIMUM_TABLEAU_CHANGED_PIXELS, NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH,
        SHARED_SOLVER_PROGRESS_PROFILE, STOCK_HALO_SCAN_BOUNDS, TABLEAU_CARD_REGIONS,
        TABLEAU_ROW_SCAN_PROFILES, TARGET_BOARD, TARGET_STOCK, TARGET_WASTE,
    },
};


/// Board, stock and waste overlays in native TriPeaks frame coordinates.
pub const PREVIEW_TARGETS: [PreviewTarget; 3] = [
    PreviewTarget {
        label: "target-board",
        bounds: TARGET_BOARD,
        colour: [80, 190, 255],
    },
    PreviewTarget {
        label: "stock",
        bounds: TARGET_STOCK,
        colour: [255, 190, 50],
    },
    PreviewTarget {
        label: "waste",
        bounds: TARGET_WASTE,
        colour: [120, 235, 150],
    },
];


/// TriPeaks stock halo policy: send Draw once, then verify its calibrated effect.
pub const BOTTOM_TARGETS: [BottomTargetProfile; 1] = [BottomTargetProfile {
    label: "TriPeaks stock DRAW",
    halo_scan_bounds: STOCK_HALO_SCAN_BOUNDS,
    action: ActionSpecification {
        operation: InputOperation::PressDrawKey,
        animation_class: AnimationClass::Draw,
        effect_bounds: DRAW_EFFECT_BOUNDS,
        minimum_changed_pixels: MINIMUM_DRAW_CHANGED_PIXELS,
        exclude_cursor_from_effect: false,
        repeat_target: RepeatTargetPolicy::Allowed,
    },
}];


/// Complete TriPeaks scene, target, row-scan and action calibration.
pub const PROFILE: GameProfile = GameProfile {
    label: "TriPeaks",
    frame_width: NOMINAL_FRAME_WIDTH,
    frame_height: NOMINAL_FRAME_HEIGHT,
    preview_targets: &PREVIEW_TARGETS,
    gameplay_scene: Some(GameplaySceneProfile {
        probe_bounds: GAMEPLAY_FELT_PROBE_BOUNDS,
        green_minimum: GAMEPLAY_FELT_GREEN_MINIMUM,
        green_red_delta_minimum: GAMEPLAY_FELT_GREEN_RED_DELTA_MINIMUM,
        green_blue_delta_minimum: GAMEPLAY_FELT_GREEN_BLUE_DELTA_MINIMUM,
        required_fraction_per_mille: GAMEPLAY_FELT_REQUIRED_FRACTION_PER_MILLE,
    }),
    game_progress: Some(SHARED_SOLVER_PROGRESS_PROFILE),
    bottom_targets: &BOTTOM_TARGETS,
    target_selection: TargetSelectionPolicy::UniqueAcrossFrame,
    tableau_cards: &TABLEAU_CARD_REGIONS,
    tableau_rows: &TABLEAU_ROW_SCAN_PROFILES,
    tableau_row_count: 4,
    initial_active_rows: 0b1000,
    tableau_click_offset: PixelPoint::new(CLICK_OFFSET_X, CLICK_OFFSET_Y),
    minimum_tableau_changed_pixels: MINIMUM_TABLEAU_CHANGED_PIXELS,
    boards_per_game: BOARDS_PER_GAME,
};


/// Compile-time checks for the fixed frame size, target counts and three-board game.
const _: () = {
    assert!(PROFILE.frame_width == 1_920);
    assert!(PROFILE.frame_height == 1_080);
    assert!(PROFILE.tableau_rows.len() == 4);
    assert!(PROFILE.tableau_cards.len() == 28);
    assert!(PROFILE.bottom_targets.len() == 1);
    assert!(PROFILE.boards_per_game == 3);
};
