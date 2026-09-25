use std::{env, path::PathBuf, sync::atomic::AtomicU64, time::Duration};

use crate::{
    cards::{CardRegionPixels, TABLEAU_CARD_COUNT},
    geometry::{PixelPoint, PixelRect},
};

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const RELEASE_LABEL: &str = concat!("v", env!("CARGO_PKG_VERSION"));
pub const INITIAL_WINDOW_WIDTH: f32 = 1120.0;
pub const INITIAL_WINDOW_HEIGHT: f32 = 820.0;
pub const OUTPUT_PANEL_HEIGHT: f32 = 245.0;

// Reserve space below the preview for the collapsed log and EXIT control.
// A horizontal scrollbar can consume part of the scroll area's outer height.
pub const PREVIEW_FOOTER_RESERVE_POINTS: f32 = 96.0;
pub const PREVIEW_SCROLLBAR_ALLOWANCE_POINTS: f32 = 24.0;
pub const MIN_PREVIEW_VIEWPORT_HEIGHT_POINTS: f32 = 260.0;
pub const MAX_LOG_LINES: usize = 2_000;

// The rendered panel remains bounded while the complete session is retained
// in diagnostic storage for Copy Output and later diagnosis.
pub const VISIBLE_LOG_ROLLOVER_GAMES: usize = 3;
pub const SESSION_LOG_FILE_PREFIX: &str = "Solitaire-";
pub const SESSION_LOG_FILE_SUFFIX: &str = ".log";
pub const SESSION_LOG_MODE: u32 = 0o600;
pub const SESSION_LOG_NAME_ATTEMPTS: usize = 32;
pub const SNAPSHOT_LABEL_MAX_CHARS: usize = 48;
pub const SNAPSHOT_NAME_ATTEMPTS: usize = 32;
pub const SNAPSHOT_MAX_PNG_BYTES: usize = 32 * 1024 * 1024;

/// Returns the directory used for private session diagnostic logs.
///
/// Priority:
/// 1. `QMP_SESSION_LOG_DIR` environment variable
/// 2. `$HOME/tmp` if that directory exists
/// 3. Standard temporary directory (`/tmp` or `$TMPDIR`)
pub fn session_log_directory() -> PathBuf {
    if let Some(dir) = env::var_os("QMP_SESSION_LOG_DIR").filter(|d| !d.is_empty()) {
        return PathBuf::from(dir);
    }
    if let Some(home) = env::var_os("HOME").filter(|h| !h.is_empty()) {
        let home_tmp = PathBuf::from(home).join("tmp");
        if home_tmp.is_dir() {
            return home_tmp;
        }
    }
    env::temp_dir()
}

/// Returns the directory used for captured QMP evidence snapshots.
///
/// Priority:
/// 1. `QMP_SNAPSHOT_DIR` environment variable
/// 2. `$HOME/Pictures/Screenshots` if it exists
/// 3. `$HOME/Pictures` if it exists
/// 4. Standard temporary directory (`/tmp` or `$TMPDIR`)
pub fn snapshot_directory() -> PathBuf {
    if let Some(dir) = env::var_os("QMP_SNAPSHOT_DIR").filter(|d| !d.is_empty()) {
        return PathBuf::from(dir);
    }
    if let Some(home) = env::var_os("HOME").filter(|h| !h.is_empty()) {
        let home_path = PathBuf::from(home);
        let screenshots = home_path.join("Pictures").join("Screenshots");
        if screenshots.is_dir() {
            return screenshots;
        }
        let pictures = home_path.join("Pictures");
        if pictures.is_dir() {
            return pictures;
        }
    }
    env::temp_dir()
}

// Capture artefacts use one process-wide sequence because several QMP capture
// paths can reserve files during a long-running session.
pub static CAPTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
pub const CAPTURE_NAME_ATTEMPTS: usize = 32;
pub const CANCELLABLE_WAIT_SLICE: Duration = Duration::from_millis(25);

// This profile was calibrated against the QMP primary surface at 1920x1080,
// with Windows display scale and text size both set to 100%.
pub const NOMINAL_FRAME_WIDTH: u32 = 1_920;
pub const NOMINAL_FRAME_HEIGHT: u32 = 1_080;

// Rectangles use half-open (x, y, width, height) guest-pixel bounds. The
// original target rectangles are deliberately padded detector regions.
pub const TARGET_BOARD: PixelRect = PixelRect::new(112, 96, 1_696, 544);
// The stock contracts horizontally towards the left as cards are consumed.
// Keep the visible target overlay and detector lane wide enough to contain
// every observed position instead of treating the initial deal as immutable.
pub const TARGET_STOCK: PixelRect = PixelRect::new(700, 665, 280, 217);

// Compatibility name retained while callers migrate to stock/waste
// terminology. In Solitaire, the left face-down pile is the stock; the right
// face-up card is the waste.
pub const TARGET_DRAW: PixelRect = TARGET_STOCK;
pub const TARGET_WASTE: PixelRect = PixelRect::new(1_001, 665, 170, 217);

// The moving stock halo is only useful along its lower exterior edge. This
// narrow lane avoids scanning card artwork while allowing the left edge of the
// pile to contract across the bounded stock target above.
pub const STOCK_HALO_SCAN_BOUNDS: PixelRect = PixelRect::new(700, 862, 280, 16);

// A draw can change both the contracting stock and the waste card. Verify the
// union so late-deal draws do not fail merely because the fixed waste interior
// changed by fewer pixels than an ordinary tableau move.
pub const DRAW_EFFECT_BOUNDS: PixelRect = PixelRect::new(700, 665, 471, 217);

// A gameplay scene has a large, stable green-felt patch below the tableau and
// left of the stock lane. Dialogs and game-selection screens replace this
// patch with blue or dark backgrounds, providing a conservative scene gate.
pub const GAMEPLAY_FELT_PROBE_BOUNDS: PixelRect = PixelRect::new(128, 650, 520, 180);
pub const GAMEPLAY_FELT_GREEN_MINIMUM: u8 = 90;
pub const GAMEPLAY_FELT_GREEN_RED_DELTA_MINIMUM: u8 = 50;
pub const GAMEPLAY_FELT_GREEN_BLUE_DELTA_MINIMUM: u8 = 25;
pub const GAMEPLAY_FELT_REQUIRED_FRACTION_PER_MILLE: u32 = 800;

// Tight unions of all calibrated tableau cards and their possible halos.
pub const TABLEAU_PLAY_AREA: PixelRect = PixelRect::new(128, 111, 1_664, 513);
pub const TABLEAU_HALO_AREA: PixelRect = PixelRect::new(116, 97, 1_688, 541);

pub const TABLEAU_CARD_HEIGHT: u32 = 181;
pub const TABLEAU_HALO_MARGIN_X: u32 = 12;
pub const TABLEAU_HALO_MARGIN_Y: u32 = 14;
pub const CARD_RANK_OFFSET_X: u32 = 3;
pub const CARD_RANK_OFFSET_Y: u32 = 7;
pub const CARD_RANK_WIDTH: u32 = 25;
// Stop at local y=29: the verified rank ink fits through y=28, while suit
// artwork starts at y=29 and would contaminate a rank-template mask.
pub const CARD_RANK_HEIGHT: u32 = 22;

const fn tableau_card_region(x: u32, y: u32, width: u32) -> CardRegionPixels {
    let card_bounds = PixelRect::new(x, y, width, TABLEAU_CARD_HEIGHT);
    CardRegionPixels::new(
        card_bounds,
        PixelRect::new(
            x.saturating_sub(TABLEAU_HALO_MARGIN_X),
            y.saturating_sub(TABLEAU_HALO_MARGIN_Y),
            width.saturating_add(TABLEAU_HALO_MARGIN_X * 2),
            TABLEAU_CARD_HEIGHT.saturating_add(TABLEAU_HALO_MARGIN_Y * 2),
        ),
        PixelRect::new(
            x.saturating_add(CARD_RANK_OFFSET_X),
            y.saturating_add(CARD_RANK_OFFSET_Y),
            CARD_RANK_WIDTH,
            CARD_RANK_HEIGHT,
        ),
        card_bounds.centre(),
    )
}

// Row-major TriPeaks layout: 3 cards, 6 cards, 9 cards, then 10 cards. The
// upper rows are projected from the visible card edges in the verified
// 1920x1080 capture; the bottom row was directly observed.
pub const TABLEAU_CARD_REGIONS: [CardRegionPixels; TABLEAU_CARD_COUNT] = [
    // Row 1.
    tableau_card_region(383, 111, 136),
    tableau_card_region(892, 111, 136),
    tableau_card_region(1_401, 111, 136),
    // Row 2.
    tableau_card_region(298, 221, 136),
    tableau_card_region(468, 221, 136),
    tableau_card_region(807, 221, 136),
    tableau_card_region(977, 221, 136),
    tableau_card_region(1_316, 221, 136),
    tableau_card_region(1_486, 221, 136),
    // Row 3.
    tableau_card_region(213, 332, 136),
    tableau_card_region(383, 332, 136),
    tableau_card_region(553, 332, 136),
    tableau_card_region(722, 332, 136),
    tableau_card_region(892, 332, 136),
    tableau_card_region(1_062, 332, 136),
    tableau_card_region(1_231, 332, 136),
    tableau_card_region(1_401, 332, 136),
    tableau_card_region(1_571, 332, 136),
    // Row 4.
    tableau_card_region(128, 443, 137),
    tableau_card_region(298, 443, 136),
    tableau_card_region(468, 443, 136),
    tableau_card_region(637, 443, 137),
    tableau_card_region(807, 443, 136),
    tableau_card_region(977, 443, 136),
    tableau_card_region(1_146, 443, 137),
    tableau_card_region(1_316, 443, 136),
    tableau_card_region(1_486, 443, 136),
    tableau_card_region(1_655, 443, 137),
];

pub const STOCK_CARD_BOUNDS: PixelRect = PixelRect::new(832, 682, 136, 182);
pub const STOCK_HALO_BOUNDS: PixelRect = PixelRect::new(820, 668, 160, 210);
pub const STOCK_RANK_BOUNDS: PixelRect = PixelRect::new(835, 689, 25, 22);
pub const STOCK_CARD_REGION: CardRegionPixels = CardRegionPixels::new(
    STOCK_CARD_BOUNDS,
    STOCK_HALO_BOUNDS,
    STOCK_RANK_BOUNDS,
    STOCK_CARD_BOUNDS.centre(),
);

pub const WASTE_CARD_BOUNDS: PixelRect = PixelRect::new(1_017, 682, 136, 182);
pub const WASTE_HALO_BOUNDS: PixelRect = PixelRect::new(1_005, 668, 160, 210);
pub const WASTE_RANK_BOUNDS: PixelRect = PixelRect::new(1_020, 689, 25, 22);
pub const WASTE_CARD_REGION: CardRegionPixels = CardRegionPixels::new(
    WASTE_CARD_BOUNDS,
    WASTE_HALO_BOUNDS,
    WASTE_RANK_BOUNDS,
    WASTE_CARD_BOUNDS.centre(),
);

// The audited highlight includes a roughly 120x2 horizontal gold segment
// outside the card. A long-run detector gives this feature explicit semantics
// and avoids treating short gold details in card artwork as a halo.
pub const HALO_GOLD_RUN_MIN: u32 = 80;
pub const HALO_GOLD_LINE_OFFSET_Y: u32 = 6;
pub const HALO_GOLD_SCAN_HEIGHT: u32 = 2;

// Tableau cards can be rendered a few pixels away from the original
// calibration and the highlighted border can be darker than the two audited
// exact RGB samples. The exact detector remains the fast path. Its
// tableau-only fallback searches a narrow strip outside the card, accepts a
// gold-shaped channel relationship, and still requires the long run to begin
// close to the immutable per-slot anchor.
pub const TABLEAU_RELAXED_HALO_Y_TOLERANCE: u32 = 8;
pub const TABLEAU_RELAXED_HALO_START_X_TOLERANCE: u32 = 8;
pub const TABLEAU_RELAXED_GOLD_RED_MINIMUM: u8 = 180;
pub const TABLEAU_RELAXED_GOLD_GREEN_MINIMUM: u8 = 140;
pub const TABLEAU_RELAXED_GOLD_BLUE_MAXIMUM: u8 = 160;
pub const TABLEAU_RELAXED_GOLD_RED_GREEN_DELTA_MINIMUM: u8 = 15;
pub const TABLEAU_RELAXED_GOLD_RED_GREEN_DELTA_MAXIMUM: u8 = 72;
pub const TABLEAU_RELAXED_GOLD_GREEN_BLUE_DELTA_MINIMUM: u8 = 40;

// A face-up card presents a continuous near-white patch across this calibrated
// four-line band. Requiring a rectangular block rejects isolated snow and
// speckle in the mountain artwork on face-down card backs.
pub const FACE_UP_WHITE_CHANNEL_MINIMUM: u8 = 250;
pub const FACE_UP_WHITE_BLOCK_MIN_WIDTH: u32 = 16;
pub const FACE_UP_WHITE_SCAN_HEIGHT: u32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableauRowScanProfile {
    /// One-based TriPeaks row number, with row 1 at the top.
    pub row: u8,
    /// First slot in `TABLEAU_CARD_REGIONS` for this row.
    pub first_card_index: usize,
    pub card_count: usize,
    /// Narrow band used only to decide whether this row has a face-up card.
    pub face_probe_bounds: PixelRect,
    /// Broad envelope containing the calibrated bottom halo line for the row.
    pub halo_scan_bounds: PixelRect,
}

impl TableauRowScanProfile {
    pub const fn card_end_index(self) -> usize {
        self.first_card_index.saturating_add(self.card_count)
    }
}

// Row envelopes are deliberately half-open framebuffer coordinates. Slot
// indices let halo detection visit only calibrated cards, while the narrow
// face probe spans the usable row width. Face probes end immediately above
// each row's pointer centre and do not overlap the preceding row's halo band.
pub const TABLEAU_ROW_SCAN_PROFILES: [TableauRowScanProfile; 4] = [
    TableauRowScanProfile {
        row: 1,
        first_card_index: 0,
        card_count: 3,
        face_probe_bounds: PixelRect::new(387, 196, 1_146, FACE_UP_WHITE_SCAN_HEIGHT),
        halo_scan_bounds: PixelRect::new(371, 298, 1_178, HALO_GOLD_SCAN_HEIGHT),
    },
    TableauRowScanProfile {
        row: 2,
        first_card_index: 3,
        card_count: 6,
        face_probe_bounds: PixelRect::new(302, 306, 1_316, FACE_UP_WHITE_SCAN_HEIGHT),
        halo_scan_bounds: PixelRect::new(286, 408, 1_348, HALO_GOLD_SCAN_HEIGHT),
    },
    TableauRowScanProfile {
        row: 3,
        first_card_index: 9,
        card_count: 9,
        face_probe_bounds: PixelRect::new(217, 417, 1_486, FACE_UP_WHITE_SCAN_HEIGHT),
        halo_scan_bounds: PixelRect::new(201, 519, 1_518, HALO_GOLD_SCAN_HEIGHT),
    },
    TableauRowScanProfile {
        row: 4,
        first_card_index: 18,
        card_count: 10,
        face_probe_bounds: PixelRect::new(132, 528, 1_656, FACE_UP_WHITE_SCAN_HEIGHT),
        halo_scan_bounds: PixelRect::new(116, 630, 1_688, HALO_GOLD_SCAN_HEIGHT),
    },
];

// Provisional rank-reader thresholds derived from the labelled STEP 3/4
// evidence. Recognition remains fail-closed until all ranks, especially Three,
// have controlled 1920x1080 samples.
pub const RANK_CHANNEL_THRESHOLD: u8 = 220;
pub const FACE_UP_LIGHT_FRACTION_PER_MILLE: u32 = 400;
pub const CARD_BACK_BRIGHT_CHANNEL_MINIMUM: u8 = 170;
pub const CARD_BACK_BRIGHT_FRACTION_PER_MILLE: u32 = 250;
pub const MINIMUM_GLYPH_COMPONENT_PIXELS: usize = 8;
pub const MINIMUM_RANK_SCORE_PER_MILLE: u16 = 880;
pub const MINIMUM_RANK_MARGIN_PER_MILLE: u16 = 50;
pub const MAXIMUM_TEMPLATE_DIMENSION_DIFFERENCE: u8 = 2;

// STEP 8 post-action evidence must change materially inside the action's
// expected effect region. STEP 4 produced 10,065 >=20-channel changes in the
// waste interior. This lower bound remains deliberately below that observation
// while rejecting small pointer, antialiasing, and compression artefacts.
pub const ACTION_CHANGE_CHANNEL_THRESHOLD: u8 = 20;
pub const MINIMUM_DRAW_CHANGED_PIXELS: usize = 1_024;
pub const MINIMUM_TABLEAU_CHANGED_PIXELS: usize = 2_048;
// Compatibility name retained for callers that have not yet selected the
// threshold through `PlannedInput::minimum_changed_pixels`.
pub const MINIMUM_ACTION_CHANGED_PIXELS: usize = MINIMUM_TABLEAU_CHANGED_PIXELS;
// The guest cursor is visible in QMP screendumps. Ignore a centred square
// around a tableau click so cursor relocation cannot prove that the card moved.
pub const ACTION_CURSOR_EXCLUSION_HALF_SIZE: u32 = 48;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlTarget {
    pub bounds: PixelRect,
    pub click_point: PixelPoint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PostGameControlVariant {
    pub label: &'static str,
    pub probe_bounds: PixelRect,
    pub click_point: PixelPoint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostGameStage {
    LevelUpOk,
    NewGame,
    Play,
    Solver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PostGameTarget {
    pub stage: PostGameStage,
    pub label: &'static str,
    pub control_variants: &'static [PostGameControlVariant],
    pub requires_gameplay_scene: bool,
}

impl PostGameTarget {
    pub const fn new(
        stage: PostGameStage,
        label: &'static str,
        control_variants: &'static [PostGameControlVariant],
        requires_gameplay_scene: bool,
    ) -> Self {
        Self {
            stage,
            label,
            control_variants,
            requires_gameplay_scene,
        }
    }
}

impl PostGameControlVariant {
    pub const fn new(
        label: &'static str,
        probe_bounds: PixelRect,
        click_point: PixelPoint,
    ) -> Self {
        Self {
            label,
            probe_bounds,
            click_point,
        }
    }
}

impl ControlTarget {
    pub const fn new(bounds: PixelRect, click_point: PixelPoint) -> Self {
        Self {
            bounds,
            click_point,
        }
    }
}

// Conservative visible-icon bounds and their centres. These are not claims
// about the application's larger invisible hit areas.
pub const SOLVER_CONTROL: ControlTarget =
    ControlTarget::new(PixelRect::new(585, 959, 35, 36), PixelPoint::new(602, 977));
pub const SCORE_SKIP_CONTROL: ControlTarget = ControlTarget::new(
    PixelRect::new(760, 360, 400, 360),
    PixelPoint::new(960, 540),
);
pub const UNDO_ALL_CONTROL: ControlTarget = ControlTarget::new(
    PixelRect::new(1_301, 959, 38, 38),
    PixelPoint::new(1_320, 978),
);
pub const UNDO_CONTROL: ControlTarget = ControlTarget::new(
    PixelRect::new(1_661, 960, 37, 36),
    PixelPoint::new(1_680, 978),
);

// Level Up has two observed layouts. Each probe remains paired with the click
// point calibrated for that layout so recognising one variant can never
// authorise the other's click. The raised probe uses a clean patch to the right
// of the OK text and remains disjoint from the proven legacy probe.
pub const LEVEL_UP_CONTROL_VARIANTS: [PostGameControlVariant; 2] = [
    PostGameControlVariant::new(
        "legacy-low",
        PixelRect::new(990, 800, 80, 30),
        PixelPoint::new(960, 795),
    ),
    PostGameControlVariant::new(
        "raised",
        PixelRect::new(1_010, 755, 50, 20),
        PixelPoint::new(960, 770),
    ),
];

pub const NEW_GAME_CONTROL_VARIANTS: [PostGameControlVariant; 1] = [PostGameControlVariant::new(
    "default",
    PixelRect::new(850, 850, 80, 30),
    PixelPoint::new(792, 840),
)];

pub const PLAY_CONTROL_VARIANTS: [PostGameControlVariant; 1] = [PostGameControlVariant::new(
    "default",
    PixelRect::new(730, 905, 80, 30),
    PixelPoint::new(705, 905),
)];

pub const SOLVER_CONTROL_VARIANTS: [PostGameControlVariant; 1] = [PostGameControlVariant::new(
    "default",
    SOLVER_CONTROL.bounds,
    SOLVER_CONTROL.click_point,
)];

// The post-game sequence is ordered and fail-closed. Dialog variants use
// mutually exclusive tight patches; no earlier button may satisfy a later
// stage. Solver instead requires a gameplay frame with no halo before using
// its already-audited click point.
pub const POST_GAME_TARGETS: [PostGameTarget; 4] = [
    PostGameTarget::new(
        PostGameStage::LevelUpOk,
        "Level Up OK",
        &LEVEL_UP_CONTROL_VARIANTS,
        false,
    ),
    PostGameTarget::new(
        PostGameStage::NewGame,
        "New Game",
        &NEW_GAME_CONTROL_VARIANTS,
        false,
    ),
    PostGameTarget::new(PostGameStage::Play, "Play", &PLAY_CONTROL_VARIANTS, false),
    PostGameTarget::new(
        PostGameStage::Solver,
        "Solver",
        &SOLVER_CONTROL_VARIANTS,
        true,
    ),
];

// The Solver header's progress remainder is a two-pixel line. Sampling a
// conservative patch in its right third distinguishes another board (black)
// from a completed three-board game (the patch becomes non-black).
pub const SOLVER_PROGRESS_RIGHT_PROBE: PixelRect = PixelRect::new(1_050, 84, 38, 2);
pub const SOLVER_PROGRESS_BLACK_CHANNEL_MAXIMUM: u8 = 32;
pub const SOLVER_PROGRESS_BLACK_FRACTION_PER_MILLE: u32 = 850;

// Dialog buttons use a gold gradient rather than the exact two-colour halo.
// The ordered stage machine also requires the correct scene class and band.
pub const DIALOG_GOLD_RED_MINIMUM: u8 = 170;
pub const DIALOG_GOLD_GREEN_MINIMUM: u8 = 105;
pub const DIALOG_GOLD_BLUE_MAXIMUM: u8 = 190;
pub const DIALOG_GOLD_RED_GREEN_DELTA_MINIMUM: u8 = 8;
pub const DIALOG_GOLD_GREEN_BLUE_DELTA_MINIMUM: u8 = 18;
pub const DIALOG_GOLD_REQUIRED_FRACTION_PER_MILLE: u32 = 80;

// Convert a tableau gold-anchor coordinate to the intended card centre. The
// stock action uses qcode D and must not use this offset.
pub const CLICK_OFFSET_X: i32 = 61;
pub const CLICK_OFFSET_Y: i32 = -97;

pub const GOLD_RGB_CANDIDATES: [[u8; 3]; 2] = [[237, 208, 109], [236, 207, 106]];
pub const GOLD_CHANNEL_TOLERANCE: u8 = 2;

// Compile-time gates remain independent so Multi-Step authority can be
// withdrawn without disabling the single-operation path. Runtime execution
// still revalidates fresh evidence before every planned input.
pub const STEP_ONCE_INPUT_ENABLED: bool = true;
pub const MULTI_STEP_INPUT_ENABLED: bool = true;

pub const STEP_ONCE_ACTIONS: usize = 1;
pub const UNBOUNDED_MULTI_STEP_ACTIONS: usize = 0;
pub const DEFAULT_MULTI_STEP_ACTIONS: usize = UNBOUNDED_MULTI_STEP_ACTIONS;

// Allow Microsoft Solitaire's action-specific animation to finish before the
// first post-action screendump. These constants remain the session defaults;
// the Params window can tune a bounded copy for the next guarded run.
pub const MINIMUM_ANIMATION_SETTLE_DELAY_MS: u64 = 0;
pub const MAXIMUM_ANIMATION_SETTLE_DELAY_MS: u64 = 5_000;
pub const DRAW_ANIMATION_SETTLE_DELAY_MS: u64 = 500;
pub const TABLEAU_ANIMATION_SETTLE_DELAY_MS: u64 = 750;
pub const BOARD_REDEAL_SETTLE_DELAY_MS: u64 = 4_000;
pub const DRAW_ANIMATION_SETTLE_DELAY: Duration =
    Duration::from_millis(DRAW_ANIMATION_SETTLE_DELAY_MS);
pub const TABLEAU_ANIMATION_SETTLE_DELAY: Duration =
    Duration::from_millis(TABLEAU_ANIMATION_SETTLE_DELAY_MS);
pub const BOARD_REDEAL_SETTLE_DELAY: Duration = Duration::from_millis(BOARD_REDEAL_SETTLE_DELAY_MS);
// A missing next highlight can be a late animation or an in-board transition.
// Re-observe without a fixed round limit until a target appears or STOP is
// pressed; never repeat the guest input that preceded it.
pub const NO_HIGHLIGHT_REOBSERVE_DELAY: Duration = Duration::from_millis(150);
pub const BOARDS_PER_GAME: usize = 3;
pub const POST_GAME_STAGE_DELAY: Duration = Duration::from_secs(1);
pub const BOARD_TRANSITION_REOBSERVE_DELAY: Duration = Duration::from_secs(2);
pub const POST_GAME_MAX_OBSERVATION_ROUNDS: usize = 20;
pub const POST_GAME_MAX_CLICK_ATTEMPTS: usize = 3;
// The initial centre click counts towards this total. A confirmed QMP delivery
// may be repeated only while Level Up is the expected stage and fresh captures
// still do not recognise its OK control.
pub const SCORE_SKIP_MAX_CLICK_ATTEMPTS: usize = 3;
pub const POINTER_SETTLE_DELAY: Duration = Duration::from_millis(100);
pub const MOUSE_HOLD: Duration = Duration::from_millis(50);
// Post-game controls proved susceptible to a normal 50 ms click. A deliberate
// hold is paired with fresh visual transition verification before advancing.
pub const POST_GAME_MOUSE_HOLD: Duration = Duration::from_millis(500);
pub const SOLVER_MOUSE_HOLD: Duration = Duration::from_millis(500);
pub const KEY_HOLD: Duration = Duration::from_millis(20);
pub const QMP_IO_TIMEOUT: Duration = Duration::from_millis(750);
pub const QMP_SCREENDUMP_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationSettleDelays {
    pub draw: Duration,
    pub tableau: Duration,
    pub board_redeal: Duration,
}

impl AnimationSettleDelays {
    pub const fn from_millis(draw_ms: u64, tableau_ms: u64, board_redeal_ms: u64) -> Self {
        Self {
            draw: Duration::from_millis(draw_ms),
            tableau: Duration::from_millis(tableau_ms),
            board_redeal: Duration::from_millis(board_redeal_ms),
        }
    }
}

impl Default for AnimationSettleDelays {
    fn default() -> Self {
        Self {
            draw: DRAW_ANIMATION_SETTLE_DELAY,
            tableau: TABLEAU_ANIMATION_SETTLE_DELAY,
            board_redeal: BOARD_REDEAL_SETTLE_DELAY,
        }
    }
}

/// Immutable parameters captured when a Step Once or Multi-Step run begins.
///
/// Keeping one snapshot prevents live UI edits from changing animation timing
/// or the authority limit part-way through a run.
/// An operation limit of zero means that the run continues across completed
/// games until a fail-closed anomaly or the user pressing STOP.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepRunSettings {
    animation_delays: AnimationSettleDelays,
    operation_limit: usize,
}

impl StepRunSettings {
    pub const fn new(animation_delays: AnimationSettleDelays, operation_limit: usize) -> Self {
        Self {
            animation_delays,
            operation_limit,
        }
    }

    pub const fn animation_delays(self) -> AnimationSettleDelays {
        self.animation_delays
    }

    pub const fn operation_limit(self) -> usize {
        self.operation_limit
    }

    pub const fn is_unbounded(self) -> bool {
        self.operation_limit == UNBOUNDED_MULTI_STEP_ACTIONS
    }

    pub const fn bounded_operation_limit(self) -> Option<usize> {
        if self.is_unbounded() {
            None
        } else {
            Some(self.operation_limit)
        }
    }
}

impl Default for StepRunSettings {
    fn default() -> Self {
        Self::new(AnimationSettleDelays::default(), DEFAULT_MULTI_STEP_ACTIONS)
    }
}

pub const QMP_SOCKET_FILENAME: &str = "solitaire-solver-qmp.sock";

pub fn default_qmp_socket_path() -> PathBuf {
    // Build the default QMP socket path from the runtime directory or fallback.
    env::var_os("XDG_RUNTIME_DIR")
        .filter(|runtime_dir| !runtime_dir.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if let Some(dir) = env::var_os("QMP_RUNTIME_DIR").filter(|d| !d.is_empty()) {
                PathBuf::from(dir)
            } else if let Some(home) = env::var_os("HOME").filter(|h| !h.is_empty()) {
                let home_tmp = PathBuf::from(home).join("tmp").join("qemu-runtime");
                if home_tmp.is_dir() {
                    home_tmp
                } else {
                    env::temp_dir().join("qemu-runtime")
                }
            } else {
                env::temp_dir().join("qemu-runtime")
            }
        })
        .join(QMP_SOCKET_FILENAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cards::{CardRegions, TABLEAU_ROW_LENGTHS, TABLEAU_ROW_STARTS},
        geometry::{QmpPoint, pixel_point_to_qmp},
    };

    fn assert_rect_fits_frame(rect: PixelRect) {
        assert!(!rect.is_empty());
        assert!(rect.right() <= NOMINAL_FRAME_WIDTH);
        assert!(rect.bottom() <= NOMINAL_FRAME_HEIGHT);
    }

    #[test]
    fn animation_settle_defaults_and_custom_values_remain_distinct() {
        let defaults = AnimationSettleDelays::default();
        assert_eq!(defaults.draw, DRAW_ANIMATION_SETTLE_DELAY);
        assert_eq!(defaults.tableau, TABLEAU_ANIMATION_SETTLE_DELAY);
        assert_eq!(defaults.board_redeal, BOARD_REDEAL_SETTLE_DELAY);
        assert_eq!(DRAW_ANIMATION_SETTLE_DELAY_MS, 500);
        assert_eq!(TABLEAU_ANIMATION_SETTLE_DELAY_MS, 750);
        assert_eq!(BOARD_REDEAL_SETTLE_DELAY_MS, 4_000);
        assert!(DRAW_ANIMATION_SETTLE_DELAY_MS <= MAXIMUM_ANIMATION_SETTLE_DELAY_MS);
        assert!(TABLEAU_ANIMATION_SETTLE_DELAY_MS <= MAXIMUM_ANIMATION_SETTLE_DELAY_MS);
        assert!(BOARD_REDEAL_SETTLE_DELAY_MS <= MAXIMUM_ANIMATION_SETTLE_DELAY_MS);

        let custom = AnimationSettleDelays::from_millis(123, 987, 2_345);
        assert_eq!(custom.draw, Duration::from_millis(123));
        assert_eq!(custom.tableau, Duration::from_millis(987));
        assert_eq!(custom.board_redeal, Duration::from_millis(2_345));
        assert_eq!(BOARDS_PER_GAME, 3);
        assert_eq!(BOARD_REDEAL_SETTLE_DELAY, Duration::from_secs(4));
        assert_eq!(POST_GAME_STAGE_DELAY, Duration::from_secs(1));
        assert_eq!(POST_GAME_MAX_OBSERVATION_ROUNDS, 20);
        assert_eq!(POST_GAME_MAX_CLICK_ATTEMPTS, 3);
        assert_eq!(SCORE_SKIP_MAX_CLICK_ATTEMPTS, 3);
        assert_eq!(NO_HIGHLIGHT_REOBSERVE_DELAY, Duration::from_millis(150));
        assert_eq!(MOUSE_HOLD, Duration::from_millis(50));
        assert_eq!(POST_GAME_MOUSE_HOLD, Duration::from_millis(500));
        assert_eq!(SOLVER_MOUSE_HOLD, Duration::from_millis(500));
    }

    #[test]
    fn step_run_settings_snapshot_timing_and_run_authority() {
        let defaults = StepRunSettings::default();
        assert_eq!(
            defaults.animation_delays(),
            AnimationSettleDelays::default()
        );
        assert_eq!(defaults.operation_limit(), DEFAULT_MULTI_STEP_ACTIONS);

        let bounded =
            StepRunSettings::new(AnimationSettleDelays::from_millis(12, 34, 56), 1_000_000);
        assert_eq!(
            bounded.animation_delays(),
            AnimationSettleDelays::from_millis(12, 34, 56)
        );
        assert_eq!(bounded.operation_limit(), 1_000_000);
        assert_eq!(bounded.bounded_operation_limit(), Some(1_000_000));

        let unbounded = StepRunSettings::new(
            AnimationSettleDelays::default(),
            UNBOUNDED_MULTI_STEP_ACTIONS,
        );
        assert!(unbounded.is_unbounded());
        assert_eq!(unbounded.bounded_operation_limit(), None);
        assert_eq!(
            (
                UNBOUNDED_MULTI_STEP_ACTIONS,
                STEP_ONCE_ACTIONS,
                DEFAULT_MULTI_STEP_ACTIONS
            ),
            (0, 1, 0)
        );
    }

    fn final_pixel(rect: PixelRect) -> PixelPoint {
        PixelPoint::new((rect.right() - 1) as i32, (rect.bottom() - 1) as i32)
    }

    #[test]
    fn calibration_matches_verified_capture() {
        // Lock the constants to the audited 1920x1080 STEP 3B/4 evidence.
        assert_eq!((NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT), (1_920, 1_080));
        assert_eq!(TARGET_BOARD, PixelRect::new(112, 96, 1_696, 544));
        assert_eq!(TARGET_STOCK, PixelRect::new(700, 665, 280, 217));
        assert_eq!(TARGET_DRAW, TARGET_STOCK);
        assert_eq!(TARGET_WASTE, PixelRect::new(1_001, 665, 170, 217));
        assert_eq!(STOCK_HALO_SCAN_BOUNDS, PixelRect::new(700, 862, 280, 16));
        assert_eq!(DRAW_EFFECT_BOUNDS, PixelRect::new(700, 665, 471, 217));
        assert_eq!(TABLEAU_PLAY_AREA, PixelRect::new(128, 111, 1_664, 513));
        assert_eq!(TABLEAU_HALO_AREA, PixelRect::new(116, 97, 1_688, 541));
        assert_eq!((CLICK_OFFSET_X, CLICK_OFFSET_Y), (61, -97));
        assert_eq!(GOLD_RGB_CANDIDATES, [[237, 208, 109], [236, 207, 106]]);
        assert_eq!(GOLD_CHANNEL_TOLERANCE, 2);
        assert_eq!(ACTION_CHANGE_CHANNEL_THRESHOLD, 20);
        assert_eq!(MINIMUM_DRAW_CHANGED_PIXELS, 1_024);
        assert_eq!(MINIMUM_TABLEAU_CHANGED_PIXELS, 2_048);
        assert_eq!(
            MINIMUM_ACTION_CHANGED_PIXELS,
            MINIMUM_TABLEAU_CHANGED_PIXELS
        );
        assert_eq!(ACTION_CURSOR_EXCLUSION_HALF_SIZE, 48);
    }

    #[test]
    fn post_game_targets_preserve_order_variants_and_solver_coordinate() {
        assert_eq!(POST_GAME_TARGETS.len(), 4);
        assert_eq!(POST_GAME_TARGETS[0].stage, PostGameStage::LevelUpOk);
        assert_eq!(POST_GAME_TARGETS[1].stage, PostGameStage::NewGame);
        assert_eq!(POST_GAME_TARGETS[2].stage, PostGameStage::Play);
        assert_eq!(POST_GAME_TARGETS[3].stage, PostGameStage::Solver);

        let legacy_level_up = LEVEL_UP_CONTROL_VARIANTS[0];
        let raised_level_up = LEVEL_UP_CONTROL_VARIANTS[1];
        assert_eq!(
            legacy_level_up.probe_bounds,
            PixelRect::new(990, 800, 80, 30)
        );
        assert_eq!(legacy_level_up.click_point, PixelPoint::new(960, 795));
        assert_eq!(
            raised_level_up.probe_bounds,
            PixelRect::new(1_010, 755, 50, 20)
        );
        assert_eq!(raised_level_up.click_point, PixelPoint::new(960, 770));
        assert!(raised_level_up.probe_bounds.bottom() <= legacy_level_up.probe_bounds.y);
        assert_eq!(POST_GAME_TARGETS[0].control_variants.len(), 2);
        assert_eq!(
            POST_GAME_TARGETS[1].control_variants[0].probe_bounds,
            PixelRect::new(850, 850, 80, 30)
        );
        assert_eq!(
            POST_GAME_TARGETS[2].control_variants[0].probe_bounds,
            PixelRect::new(730, 905, 80, 30)
        );
        assert_eq!(
            POST_GAME_TARGETS[3].control_variants[0].click_point,
            PixelPoint::new(602, 977)
        );
        assert_eq!(
            POST_GAME_TARGETS[3].control_variants[0].click_point,
            SOLVER_CONTROL.click_point
        );
        assert!(POST_GAME_TARGETS[3].requires_gameplay_scene);
        assert_eq!(SCORE_SKIP_CONTROL.click_point, PixelPoint::new(960, 540));
        assert_eq!(
            SOLVER_PROGRESS_RIGHT_PROBE,
            PixelRect::new(1_050, 84, 38, 2)
        );
        assert_eq!(BOARD_TRANSITION_REOBSERVE_DELAY, Duration::from_secs(2));

        for target in POST_GAME_TARGETS {
            assert!(!target.control_variants.is_empty());
            for variant in target.control_variants {
                assert_rect_fits_frame(variant.probe_bounds);
                assert!(variant.click_point.x >= 0);
                assert!(variant.click_point.y >= 0);
                assert!((variant.click_point.x as u32) < NOMINAL_FRAME_WIDTH);
                assert!((variant.click_point.y as u32) < NOMINAL_FRAME_HEIGHT);
            }
        }
    }

    #[test]
    fn calibrated_regions_fit_the_frame() {
        // Check half-open bounds for every active detector region.
        assert_eq!((TARGET_BOARD.right(), TARGET_BOARD.bottom()), (1_808, 640));
        assert_eq!((TARGET_DRAW.right(), TARGET_DRAW.bottom()), (980, 882));
        assert_eq!((TARGET_WASTE.right(), TARGET_WASTE.bottom()), (1_171, 882));
        assert!(TARGET_BOARD.right() <= NOMINAL_FRAME_WIDTH);
        assert!(TARGET_BOARD.bottom() <= NOMINAL_FRAME_HEIGHT);
        assert!(TARGET_DRAW.right() <= NOMINAL_FRAME_WIDTH);
        assert!(TARGET_DRAW.bottom() <= NOMINAL_FRAME_HEIGHT);
        assert!(TARGET_WASTE.right() <= NOMINAL_FRAME_WIDTH);
        assert!(TARGET_WASTE.bottom() <= NOMINAL_FRAME_HEIGHT);
        for rect in [
            STOCK_HALO_SCAN_BOUNDS,
            DRAW_EFFECT_BOUNDS,
            GAMEPLAY_FELT_PROBE_BOUNDS,
        ] {
            assert_rect_fits_frame(rect);
        }
        assert!(TARGET_STOCK.contains(PixelPoint::new(
            STOCK_HALO_SCAN_BOUNDS.x as i32,
            STOCK_HALO_SCAN_BOUNDS.y as i32,
        )));
        assert_eq!(DRAW_EFFECT_BOUNDS.right(), TARGET_WASTE.right());
        assert_eq!(DRAW_EFFECT_BOUNDS.bottom(), TARGET_WASTE.bottom());
        assert!(GAMEPLAY_FELT_PROBE_BOUNDS.right() <= TARGET_STOCK.x);
    }

    #[test]
    fn gameplay_felt_probe_uses_conservative_green_relationships() {
        assert_eq!(
            GAMEPLAY_FELT_PROBE_BOUNDS,
            PixelRect::new(128, 650, 520, 180)
        );
        assert_eq!(GAMEPLAY_FELT_GREEN_MINIMUM, 90);
        assert_eq!(GAMEPLAY_FELT_GREEN_RED_DELTA_MINIMUM, 50);
        assert_eq!(GAMEPLAY_FELT_GREEN_BLUE_DELTA_MINIMUM, 25);
        assert_eq!(GAMEPLAY_FELT_REQUIRED_FRACTION_PER_MILLE, 800);
        assert!(GAMEPLAY_FELT_REQUIRED_FRACTION_PER_MILLE <= 1_000);
    }

    #[test]
    fn tableau_matrix_matches_the_audited_row_major_layout() {
        let expected_cards = [
            PixelRect::new(383, 111, 136, 181),
            PixelRect::new(892, 111, 136, 181),
            PixelRect::new(1_401, 111, 136, 181),
            PixelRect::new(298, 221, 136, 181),
            PixelRect::new(468, 221, 136, 181),
            PixelRect::new(807, 221, 136, 181),
            PixelRect::new(977, 221, 136, 181),
            PixelRect::new(1_316, 221, 136, 181),
            PixelRect::new(1_486, 221, 136, 181),
            PixelRect::new(213, 332, 136, 181),
            PixelRect::new(383, 332, 136, 181),
            PixelRect::new(553, 332, 136, 181),
            PixelRect::new(722, 332, 136, 181),
            PixelRect::new(892, 332, 136, 181),
            PixelRect::new(1_062, 332, 136, 181),
            PixelRect::new(1_231, 332, 136, 181),
            PixelRect::new(1_401, 332, 136, 181),
            PixelRect::new(1_571, 332, 136, 181),
            PixelRect::new(128, 443, 137, 181),
            PixelRect::new(298, 443, 136, 181),
            PixelRect::new(468, 443, 136, 181),
            PixelRect::new(637, 443, 137, 181),
            PixelRect::new(807, 443, 136, 181),
            PixelRect::new(977, 443, 136, 181),
            PixelRect::new(1_146, 443, 137, 181),
            PixelRect::new(1_316, 443, 136, 181),
            PixelRect::new(1_486, 443, 136, 181),
            PixelRect::new(1_655, 443, 137, 181),
        ];

        assert_eq!(TABLEAU_ROW_LENGTHS, [3, 6, 9, 10]);
        assert_eq!(TABLEAU_ROW_STARTS, [0, 3, 9, 18]);
        assert_eq!(
            TABLEAU_CARD_REGIONS.map(|region| region.card_bounds),
            expected_cards
        );
    }

    #[test]
    fn row_scan_profiles_match_the_calibrated_bands() {
        let expected = [
            TableauRowScanProfile {
                row: 1,
                first_card_index: 0,
                card_count: 3,
                face_probe_bounds: PixelRect::new(387, 196, 1_146, 4),
                halo_scan_bounds: PixelRect::new(371, 298, 1_178, 2),
            },
            TableauRowScanProfile {
                row: 2,
                first_card_index: 3,
                card_count: 6,
                face_probe_bounds: PixelRect::new(302, 306, 1_316, 4),
                halo_scan_bounds: PixelRect::new(286, 408, 1_348, 2),
            },
            TableauRowScanProfile {
                row: 3,
                first_card_index: 9,
                card_count: 9,
                face_probe_bounds: PixelRect::new(217, 417, 1_486, 4),
                halo_scan_bounds: PixelRect::new(201, 519, 1_518, 2),
            },
            TableauRowScanProfile {
                row: 4,
                first_card_index: 18,
                card_count: 10,
                face_probe_bounds: PixelRect::new(132, 528, 1_656, 4),
                halo_scan_bounds: PixelRect::new(116, 630, 1_688, 2),
            },
        ];

        assert_eq!(TABLEAU_ROW_SCAN_PROFILES, expected);
        assert_eq!(FACE_UP_WHITE_CHANNEL_MINIMUM, 250);
        assert_eq!(FACE_UP_WHITE_BLOCK_MIN_WIDTH, 16);
        assert_eq!(FACE_UP_WHITE_SCAN_HEIGHT, 4);
        assert_eq!(HALO_GOLD_SCAN_HEIGHT, 2);

        for (row_index, profile) in TABLEAU_ROW_SCAN_PROFILES.iter().enumerate() {
            assert_eq!(profile.row as usize, row_index + 1);
            assert_eq!(profile.first_card_index, TABLEAU_ROW_STARTS[row_index]);
            assert_eq!(profile.card_count, TABLEAU_ROW_LENGTHS[row_index] as usize);
            assert!(profile.card_end_index() <= TABLEAU_CARD_REGIONS.len());
            assert_rect_fits_frame(profile.face_probe_bounds);
            assert_rect_fits_frame(profile.halo_scan_bounds);

            let cards = &TABLEAU_CARD_REGIONS[profile.first_card_index..profile.card_end_index()];
            let first = cards.first().expect("calibrated row cannot be empty");
            let last = cards.last().expect("calibrated row cannot be empty");
            assert_eq!(profile.face_probe_bounds.x, first.card_bounds.x + 4);
            assert_eq!(
                profile.face_probe_bounds.right(),
                last.card_bounds.right() - 4
            );
            assert_eq!(profile.halo_scan_bounds.x, first.halo_bounds.x);
            assert_eq!(profile.halo_scan_bounds.right(), last.halo_bounds.right());
            assert_eq!(
                profile.halo_scan_bounds.y,
                first.card_bounds.bottom() + HALO_GOLD_LINE_OFFSET_Y
            );
            assert_eq!(profile.halo_scan_bounds.height, HALO_GOLD_SCAN_HEIGHT);

            for card in cards {
                assert!(profile.face_probe_bounds.y >= card.card_bounds.y);
                assert!(profile.face_probe_bounds.bottom() <= card.card_bounds.bottom());
                assert!(profile.face_probe_bounds.bottom() <= card.click_point.y as u32);
            }

            if row_index > 0 {
                let preceding = TABLEAU_ROW_SCAN_PROFILES[row_index - 1];
                assert!(profile.face_probe_bounds.y >= preceding.halo_scan_bounds.bottom());
            }
            if let Some(next_row) = TABLEAU_ROW_SCAN_PROFILES.get(row_index + 1) {
                let next_card_top = TABLEAU_CARD_REGIONS[next_row.first_card_index]
                    .card_bounds
                    .y;
                assert!(profile.face_probe_bounds.bottom() < next_card_top);
            }
        }
    }

    #[test]
    fn every_tableau_slot_preserves_the_anchor_to_click_invariant() {
        for regions in TABLEAU_CARD_REGIONS {
            let anchor = PixelPoint::new(
                regions.card_bounds.x as i32 + 7,
                (regions.card_bounds.bottom() + HALO_GOLD_LINE_OFFSET_Y) as i32,
            );
            assert_eq!(
                PixelPoint::new(anchor.x + CLICK_OFFSET_X, anchor.y + CLICK_OFFSET_Y),
                regions.click_point,
            );
        }
    }

    #[test]
    fn every_card_rank_halo_and_click_fits_the_frame() {
        for pixels in TABLEAU_CARD_REGIONS
            .iter()
            .chain([&STOCK_CARD_REGION, &WASTE_CARD_REGION])
        {
            assert_rect_fits_frame(pixels.card_bounds);
            assert_rect_fits_frame(pixels.halo_bounds);
            assert_rect_fits_frame(pixels.rank_bounds);

            assert!(pixels.halo_bounds.contains(PixelPoint::new(
                pixels.card_bounds.x as i32,
                pixels.card_bounds.y as i32,
            )));
            assert!(pixels.halo_bounds.contains(final_pixel(pixels.card_bounds)));
            assert!(pixels.card_bounds.contains(PixelPoint::new(
                pixels.rank_bounds.x as i32,
                pixels.rank_bounds.y as i32,
            )));
            assert!(pixels.card_bounds.contains(final_pixel(pixels.rank_bounds)));
            assert!(pixels.card_bounds.contains(pixels.click_point));

            CardRegions::from_pixels(*pixels, NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT)
                .expect("calibrated card geometry must convert to QMP coordinates");
        }
    }

    #[test]
    fn stock_and_waste_regions_are_distinct_and_audited() {
        assert_eq!(STOCK_CARD_BOUNDS, PixelRect::new(832, 682, 136, 182));
        assert_eq!(STOCK_HALO_BOUNDS, PixelRect::new(820, 668, 160, 210));
        assert_eq!(STOCK_RANK_BOUNDS, PixelRect::new(835, 689, 25, 22));
        assert_eq!(STOCK_CARD_REGION.click_point, PixelPoint::new(900, 773));

        assert_eq!(WASTE_CARD_BOUNDS, PixelRect::new(1_017, 682, 136, 182));
        assert_eq!(WASTE_HALO_BOUNDS, PixelRect::new(1_005, 668, 160, 210));
        assert_eq!(WASTE_RANK_BOUNDS, PixelRect::new(1_020, 689, 25, 22));
        assert_eq!(WASTE_CARD_REGION.click_point, PixelPoint::new(1_085, 773));

        assert!(STOCK_CARD_BOUNDS.right() < WASTE_CARD_BOUNDS.x);
        assert_eq!(
            (
                HALO_GOLD_RUN_MIN,
                HALO_GOLD_LINE_OFFSET_Y,
                HALO_GOLD_SCAN_HEIGHT
            ),
            (80, 6, 2)
        );
    }

    #[test]
    fn control_targets_map_to_the_audited_qmp_coordinates() {
        let cases = [
            (SOLVER_CONTROL, QmpPoint::new(10_273, 29_641)),
            (UNDO_ALL_CONTROL, QmpPoint::new(22_527, 29_672)),
            (UNDO_CONTROL, QmpPoint::new(28_671, 29_672)),
        ];

        for (target, expected_qmp) in cases {
            assert_rect_fits_frame(target.bounds);
            assert!(target.bounds.contains(target.click_point));
            assert_eq!(
                pixel_point_to_qmp(
                    target.click_point,
                    NOMINAL_FRAME_WIDTH,
                    NOMINAL_FRAME_HEIGHT,
                ),
                Ok(expected_qmp),
            );
        }
    }

    #[test]
    fn click_offset_maps_audited_anchor_to_card_centre() {
        // The STEP 4 board highlight began at (1662, 630).
        let anchor = PixelPoint::new(1_662, 630);
        let centre = PixelPoint::new(anchor.x + CLICK_OFFSET_X, anchor.y + CLICK_OFFSET_Y);
        assert_eq!(centre, PixelPoint::new(1_723, 533));
        assert_eq!(TABLEAU_CARD_REGIONS[27].click_point, centre);
        assert!((1_655..1_792).contains(&centre.x));
        assert!((443..624).contains(&centre.y));
    }
}
