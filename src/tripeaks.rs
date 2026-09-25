use crate::{
    game::{
        ActionSpecification, AnimationClass, BottomTargetProfile, GameMode, GameProfile,
        GameProgressProfile, GameplaySceneProfile, InputOperation, PreviewTarget,
        RepeatTargetPolicy, TargetSelectionPolicy,
    },
    geometry::PixelPoint,
    parameters::{
        BOARDS_PER_GAME, CLICK_OFFSET_X, CLICK_OFFSET_Y, DRAW_EFFECT_BOUNDS,
        GAMEPLAY_FELT_GREEN_BLUE_DELTA_MINIMUM, GAMEPLAY_FELT_GREEN_MINIMUM,
        GAMEPLAY_FELT_GREEN_RED_DELTA_MINIMUM, GAMEPLAY_FELT_PROBE_BOUNDS,
        GAMEPLAY_FELT_REQUIRED_FRACTION_PER_MILLE, MINIMUM_DRAW_CHANGED_PIXELS,
        MINIMUM_TABLEAU_CHANGED_PIXELS, NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH,
        SOLVER_PROGRESS_BLACK_CHANNEL_MAXIMUM, SOLVER_PROGRESS_BLACK_FRACTION_PER_MILLE,
        SOLVER_PROGRESS_RIGHT_PROBE, STOCK_HALO_SCAN_BOUNDS, TABLEAU_CARD_REGIONS,
        TABLEAU_ROW_SCAN_PROFILES, TARGET_BOARD, TARGET_STOCK, TARGET_WASTE,
    },
};

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

pub const PROFILE: GameProfile = GameProfile {
    mode: GameMode::TriPeaks,
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
    game_progress: Some(GameProgressProfile {
        right_probe: SOLVER_PROGRESS_RIGHT_PROBE,
        black_channel_maximum: SOLVER_PROGRESS_BLACK_CHANNEL_MAXIMUM,
        black_fraction_per_mille: SOLVER_PROGRESS_BLACK_FRACTION_PER_MILLE,
    }),
    bottom_targets: &BOTTOM_TARGETS,
    target_selection: TargetSelectionPolicy::UniqueAcrossFrame,
    tableau_cards: &TABLEAU_CARD_REGIONS,
    tableau_rows: &TABLEAU_ROW_SCAN_PROFILES,
    initial_active_rows: 0b1000,
    tableau_click_offset: PixelPoint::new(CLICK_OFFSET_X, CLICK_OFFSET_Y),
    minimum_tableau_changed_pixels: MINIMUM_TABLEAU_CHANGED_PIXELS,
    boards_per_game: BOARDS_PER_GAME,
};

const _: () = {
    assert!(PROFILE.frame_width == 1_920);
    assert!(PROFILE.frame_height == 1_080);
    assert!(PROFILE.tableau_rows.len() == 4);
    assert!(PROFILE.tableau_cards.len() == 28);
    assert!(PROFILE.bottom_targets.len() == 1);
    assert!(PROFILE.boards_per_game == 3);
};
