//! Shared calibration, UI limits, diagnostic paths and immutable run settings.

use std::{env, path::PathBuf, sync::atomic::AtomicU64, time::Duration};

use crate::{
    cards::{CardRegionPixels, TABLEAU_CARD_COUNT},
    geometry::{PixelPoint, PixelRect},
};


/// Cargo package name displayed in window titles and diagnostic messages.
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");


/// Package version and candidate number shown by the UI and session log.
pub const RELEASE_LABEL: &str = concat!("v", env!("CARGO_PKG_VERSION"), ", candidate 2");


/// Initial application window width in egui logical points.
pub const INITIAL_WINDOW_WIDTH: f32 = 1120.0;


/// Initial application window height in egui logical points.
pub const INITIAL_WINDOW_HEIGHT: f32 = 820.0;


// A fixed-height log: 40 more logical points is approximately 2.5 monospace
// lines at the current UI font. Window growth remains available to the image.
/// Expanded diagnostic output height in egui logical points.
pub const OUTPUT_PANEL_HEIGHT: f32 = 285.0;


// Reserve space below the preview for the collapsed log and EXIT control.
// A horizontal scrollbar can consume part of the scroll area's outer height.
/// Logical points reserved below the preview for collapsed output and Exit.
pub const PREVIEW_FOOTER_RESERVE_POINTS: f32 = 96.0;


/// Logical points reserved for the preview horizontal scrollbar.
pub const PREVIEW_SCROLLBAR_ALLOWANCE_POINTS: f32 = 24.0;


/// Minimum preview viewport height in egui logical points.
pub const MIN_PREVIEW_VIEWPORT_HEIGHT_POINTS: f32 = 260.0;


/// Maximum diagnostic lines retained in the rendered output buffer.
pub const MAX_LOG_LINES: usize = 2_000;


// The rendered panel remains bounded while the complete session is retained
// in diagnostic storage for Copy Output and later diagnosis.
/// Completed games per visible-log rollover; the session log stays complete.
pub const VISIBLE_LOG_ROLLOVER_GAMES: usize = 3;


/// Prefix used when reserving a unique session log filename.
pub const SESSION_LOG_FILE_PREFIX: &str = "qmp-qemu-socket-";


/// Filename extension of private session diagnostic logs.
pub const SESSION_LOG_FILE_SUFFIX: &str = ".log";


/// Unix permission bits for a newly created private session log.
pub const SESSION_LOG_MODE: u32 = 0o600;


/// Maximum exclusive-create attempts when reserving a session log.
pub const SESSION_LOG_NAME_ATTEMPTS: usize = 32;


/// Maximum sanitised label length, in characters, in a snapshot filename.
pub const SNAPSHOT_LABEL_MAX_CHARS: usize = 48;


/// Maximum exclusive-create attempts when reserving a saved snapshot.
pub const SNAPSHOT_NAME_ATTEMPTS: usize = 32;


/// Maximum original PNG size, in bytes, accepted by manual snapshot capture.
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
/// Process-wide monotonically increasing suffix for temporary capture filenames.
pub static CAPTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);


/// Maximum exclusive-create attempts for a temporary QMP capture file.
pub const CAPTURE_NAME_ATTEMPTS: usize = 32;


/// Maximum interval between cancellation checks during intentional waits.
pub const CANCELLABLE_WAIT_SLICE: Duration = Duration::from_millis(25);


// This profile was calibrated against the QMP primary surface at 1920x1080,
// with Windows display scale and text size both set to 100%.
/// Calibrated guest framebuffer width in pixels at 100% Windows scaling.
pub const NOMINAL_FRAME_WIDTH: u32 = 1_920;


/// Calibrated guest framebuffer height in pixels at 100% Windows scaling.
pub const NOMINAL_FRAME_HEIGHT: u32 = 1_080;


// Rectangles use half-open (x, y, width, height) guest-pixel bounds. The
// original target rectangles are deliberately padded detector regions.
/// Padded TriPeaks tableau detector envelope in guest pixels.
pub const TARGET_BOARD: PixelRect = PixelRect::new(112, 96, 1_696, 544);


// The stock contracts horizontally towards the left as cards are consumed.
// Keep the visible target overlay and detector lane wide enough to contain
// every observed position instead of treating the initial deal as immutable.
/// TriPeaks stock envelope in guest pixels, including its contracting positions.
pub const TARGET_STOCK: PixelRect = PixelRect::new(700, 665, 280, 217);


/// TriPeaks face-up waste envelope in guest pixels.
pub const TARGET_WASTE: PixelRect = PixelRect::new(1_001, 665, 170, 217);


// The moving stock halo is only useful along its lower exterior edge. This
// narrow lane avoids scanning card artwork while allowing the left edge of the
// pile to contract across the bounded stock target above.
/// Guest-pixel strip containing all calibrated stock halo positions.
pub const STOCK_HALO_SCAN_BOUNDS: PixelRect = PixelRect::new(700, 862, 280, 16);


// A draw can change both the contracting stock and the waste card. Verify the
// union so late-deal draws do not fail merely because the fixed waste interior
// changed by fewer pixels than an ordinary tableau move.
/// Guest-pixel union of stock and waste used to verify a TriPeaks draw.
pub const DRAW_EFFECT_BOUNDS: PixelRect = PixelRect::new(700, 665, 471, 217);


// A gameplay scene has a large, stable green-felt patch below the tableau and
// left of the stock lane. Dialog and game-selection screens replace this
// patch with blue or dark backgrounds, providing a conservative scene gate.
/// Stable TriPeaks felt patch in guest pixels used to reject dialog scenes.
pub const GAMEPLAY_FELT_PROBE_BOUNDS: PixelRect = PixelRect::new(128, 650, 520, 180);


/// Minimum 8-bit green channel for the TriPeaks scene fingerprint.
pub const GAMEPLAY_FELT_GREEN_MINIMUM: u8 = 90;


/// Minimum green-minus-red channel difference for TriPeaks felt.
pub const GAMEPLAY_FELT_GREEN_RED_DELTA_MINIMUM: u8 = 50;


/// Minimum green-minus-blue channel difference for TriPeaks felt.
pub const GAMEPLAY_FELT_GREEN_BLUE_DELTA_MINIMUM: u8 = 25;


/// Minimum felt-pixel share, in thousandths, required for a gameplay scene.
pub const GAMEPLAY_FELT_REQUIRED_FRACTION_PER_MILLE: u32 = 800;


// Tight unions of all calibrated tableau cards and their possible halos.
/// Tight guest-pixel union of the calibrated TriPeaks card faces.
#[cfg(test)]
pub const TABLEAU_PLAY_AREA: PixelRect = PixelRect::new(128, 111, 1_664, 513);


/// Guest-pixel union of all calibrated TriPeaks tableau halo envelopes.
#[cfg(test)]
pub const TABLEAU_HALO_AREA: PixelRect = PixelRect::new(116, 97, 1_688, 541);


/// Calibrated TriPeaks card-face height in guest pixels.
pub const TABLEAU_CARD_HEIGHT: u32 = 181;


/// Horizontal guest-pixel padding around each TriPeaks card face.
pub const TABLEAU_HALO_MARGIN_X: u32 = 12;


/// Vertical guest-pixel padding around each TriPeaks card face.
pub const TABLEAU_HALO_MARGIN_Y: u32 = 14;


/// Rank-region left inset from the TriPeaks card edge in guest pixels.
pub const CARD_RANK_OFFSET_X: u32 = 3;


/// Rank-region top inset from the TriPeaks card edge in guest pixels.
pub const CARD_RANK_OFFSET_Y: u32 = 7;


/// Calibrated TriPeaks rank-region width in guest pixels.
pub const CARD_RANK_WIDTH: u32 = 25;


// Stop at local y=29: the verified rank ink fits through y=28, while suit
// artwork starts at y=29 and would contaminate a rank-template mask.
/// Rank-region height in guest pixels, ending before the suit artwork.
pub const CARD_RANK_HEIGHT: u32 = 22;


/// Construct the card, halo, rank and click geometry for one TriPeaks slot.
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
/// Row-major TriPeaks slot geometry in guest pixels: 3, 6, 9 and 10 cards.
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


/// Initial TriPeaks stock card-face rectangle in guest pixels.
#[cfg(test)]
pub const STOCK_CARD_BOUNDS: PixelRect = PixelRect::new(832, 682, 136, 182);


/// Initial TriPeaks stock halo envelope in guest pixels.
#[cfg(test)]
pub const STOCK_HALO_BOUNDS: PixelRect = PixelRect::new(820, 668, 160, 210);


/// Reserved stock rank region in guest pixels; stock actions do not read it.
#[cfg(test)]
pub const STOCK_RANK_BOUNDS: PixelRect = PixelRect::new(835, 689, 25, 22);


/// Initial TriPeaks stock geometry used by the profile and calibration checks.
#[cfg(test)]
pub const STOCK_CARD_REGION: CardRegionPixels = CardRegionPixels::new(
    STOCK_CARD_BOUNDS,
    STOCK_HALO_BOUNDS,
    STOCK_RANK_BOUNDS,
    STOCK_CARD_BOUNDS.centre(),
);


/// TriPeaks waste card-face rectangle in guest pixels.
#[cfg(test)]
pub const WASTE_CARD_BOUNDS: PixelRect = PixelRect::new(1_017, 682, 136, 182);


/// TriPeaks waste halo envelope in guest pixels.
#[cfg(test)]
pub const WASTE_HALO_BOUNDS: PixelRect = PixelRect::new(1_005, 668, 160, 210);


/// Calibrated TriPeaks waste rank rectangle in guest pixels.
#[cfg(test)]
pub const WASTE_RANK_BOUNDS: PixelRect = PixelRect::new(1_020, 689, 25, 22);


/// TriPeaks waste face, halo, rank and centre geometry.
#[cfg(test)]
pub const WASTE_CARD_REGION: CardRegionPixels = CardRegionPixels::new(
    WASTE_CARD_BOUNDS,
    WASTE_HALO_BOUNDS,
    WASTE_RANK_BOUNDS,
    WASTE_CARD_BOUNDS.centre(),
);


// The audited highlight includes a roughly 120x2 horizontal gold segment
// outside the card. A long-run detector gives this feature explicit semantics
// and avoids treating short gold details in card artwork as a halo.
/// Minimum consecutive gold pixels in a TriPeaks horizontal halo run.
pub const HALO_GOLD_RUN_MIN: u32 = 80;


/// Guest-pixel offset below a TriPeaks card face to its gold halo line.
pub const HALO_GOLD_LINE_OFFSET_Y: u32 = 6;


/// Number of guest-pixel rows sampled for the TriPeaks gold line.
pub const HALO_GOLD_SCAN_HEIGHT: u32 = 2;


// Tableau cards can be rendered a few pixels away from the original
// calibration and the highlighted border can be darker than the two audited
// exact RGB samples. The exact detector remains the fast path. Its
// tableau-only fallback searches a narrow strip outside the card, accepts a
// gold-shaped channel relationship, and still requires the long run to begin
// close to the immutable per-slot anchor.
/// Maximum vertical guest-pixel displacement in the relaxed tableau halo scan.
pub const TABLEAU_RELAXED_HALO_Y_TOLERANCE: u32 = 8;


/// Maximum horizontal guest-pixel displacement of a relaxed halo run start.
pub const TABLEAU_RELAXED_HALO_START_X_TOLERANCE: u32 = 8;


/// Minimum 8-bit red channel accepted by the relaxed tableau gold detector.
pub const TABLEAU_RELAXED_GOLD_RED_MINIMUM: u8 = 180;


/// Minimum 8-bit green channel accepted by the relaxed tableau gold detector.
pub const TABLEAU_RELAXED_GOLD_GREEN_MINIMUM: u8 = 140;


/// Maximum 8-bit blue channel accepted by the relaxed tableau gold detector.
pub const TABLEAU_RELAXED_GOLD_BLUE_MAXIMUM: u8 = 160;


/// Lower red-minus-green channel difference for relaxed tableau gold.
pub const TABLEAU_RELAXED_GOLD_RED_GREEN_DELTA_MINIMUM: u8 = 15;


/// Upper red-minus-green channel difference for relaxed tableau gold.
pub const TABLEAU_RELAXED_GOLD_RED_GREEN_DELTA_MAXIMUM: u8 = 72;


/// Minimum green-minus-blue channel difference for relaxed tableau gold.
pub const TABLEAU_RELAXED_GOLD_GREEN_BLUE_DELTA_MINIMUM: u8 = 40;


// A face-up card presents a continuous near-white patch across this calibrated
// four-line band. Requiring a rectangular block rejects isolated snow and
// speckle in the mountain artwork on face-down card backs.
/// Minimum value in every RGB channel for a TriPeaks white face pixel.
pub const FACE_UP_WHITE_CHANNEL_MINIMUM: u8 = 250;


/// Minimum continuous white block width, in pixels, for row visibility.
pub const FACE_UP_WHITE_BLOCK_MIN_WIDTH: u32 = 16;


/// Height in guest pixels of the TriPeaks white-face probe band.
pub const FACE_UP_WHITE_SCAN_HEIGHT: u32 = 4;


/// TriPeaks row geometry linking narrow face probes to bounded halo scans.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableauRowScanProfile {
    /// One-based TriPeaks row number, with row 1 at the top.
    pub row: u8,
    /// First slot in `TABLEAU_CARD_REGIONS` for this row.
    pub first_card_index: usize,
    /// Number of calibrated slots belonging to this row.
    pub card_count: usize,
    /// Narrow band used only to decide whether this row has a face-up card.
    pub face_probe_bounds: PixelRect,
    /// Broad envelope containing the calibrated bottom halo line for the row.
    pub halo_scan_bounds: PixelRect,
}


impl TableauRowScanProfile {


    /// Return the exclusive slot index at the end of this row.
    pub const fn card_end_index(self) -> usize {
        self.first_card_index.saturating_add(self.card_count)
    }
}


// Row envelopes are deliberately half-open framebuffer coordinates. Slot
// indices let halo detection visit only calibrated cards, while the narrow
// face probe spans the usable row width. Face probes end immediately above
// each row's pointer centre and do not overlap the preceding row's halo band.
/// Audited TriPeaks row bands and their corresponding slot-index ranges.
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


// Post-action evidence must change materially inside the action's
// expected effect region. The calibration produced 10,065 >=20-channel changes in the
// waste interior. This lower bound remains deliberately below that observation
// while rejecting small pointer, antialiasing, and compression artefacts.
/// Minimum absolute change in any 8-bit RGB channel to count a changed pixel.
pub const ACTION_CHANGE_CHANNEL_THRESHOLD: u8 = 20;


/// Minimum cursor-excluded changed pixels proving a TriPeaks draw effect.
pub const MINIMUM_DRAW_CHANGED_PIXELS: usize = 1_024;


/// Minimum cursor-excluded changed pixels proving a TriPeaks tableau effect.
pub const MINIMUM_TABLEAU_CHANGED_PIXELS: usize = 2_048;


// The guest cursor is visible in QMP screen dumps. Ignore a centred square
// around a tableau click so cursor relocation cannot prove that the card moved.
/// Half-width and half-height in pixels of the excluded guest cursor square.
pub const ACTION_CURSOR_EXCLUSION_HALF_SIZE: u32 = 48;


/// Calibrated guest-pixel bounds and click point of a visible control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlTarget {
    /// Visible control envelope in half-open guest-pixel coordinates.
    pub bounds: PixelRect,
    /// Calibrated guest-pixel point inside the control hit area.
    pub click_point: PixelPoint,
}


/// One recognised visual layout of a post-game control and its paired click.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PostGameControlVariant {
    /// Diagnostic name of the observed layout variant.
    pub label: &'static str,
    /// Guest-pixel gold probe paired with this specific layout.
    pub probe_bounds: PixelRect,
    /// Guest-pixel click point permitted only after recognising this variant.
    pub click_point: PixelPoint,
}


/// Ordered control identities used by the shared post-game state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostGameStage {
    /// Acknowledge the score-completion Level Up dialog.
    LevelUpOk,
    /// Request a new game from the results screen.
    NewGame,
    /// Start the selected next game.
    Play,
    /// Enable Solver highlighting on a recognised gameplay scene.
    Solver,
}


/// Recognition variants and scene requirements for one post-game stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PostGameTarget {
    /// Control identity expected by the ordered post-game state machine.
    pub stage: PostGameStage,
    /// Human-readable stage label for diagnostics and preview.
    pub label: &'static str,
    /// Observed layouts that may satisfy this stage.
    pub control_variants: &'static [PostGameControlVariant],
    /// Whether recognising this control also requires a gameplay scene.
    pub requires_gameplay_scene: bool,
}


impl PostGameTarget {


    /// Associate one post-game stage with its recognised layouts and scene gate.
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


    /// Pair a layout-specific probe with its calibrated guest-pixel click point.
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


    /// Construct a calibrated control definition without granting permission to click.
    pub const fn new(bounds: PixelRect, click_point: PixelPoint) -> Self {
        Self {
            bounds,
            click_point,
        }
    }
}


// Shared bottom-toolbar controls at the 1920x1080 guest layout, observed in
// both TriPeaks and Pyramid. Bounds cover the visible icons, not the larger
// invisible hit areas. Undo All confirmation is a separate, uncalibrated UI.
/// Shared Solver toolbar icon bounds and click point in guest pixels.
pub const SHARED_SOLVER_CONTROL: ControlTarget =
    ControlTarget::new(PixelRect::new(585, 959, 35, 36), PixelPoint::new(602, 977));


/// Central score-animation region and skip click point in guest pixels.
pub const SCORE_SKIP_CONTROL: ControlTarget = ControlTarget::new(
    PixelRect::new(760, 360, 400, 360),
    PixelPoint::new(960, 540),
);


// Calibrated from the 1920x1080 Challenge Complete capture. This is reserved
// for a future challenge flow; the current guarded run does not click it.
/// Recorded challenge Continue geometry; shown for calibration, never automated.
pub const CHALLENGE_COMPLETE_CONTINUE_CONTROL: ControlTarget =
    ControlTarget::new(PixelRect::new(805, 844, 308, 72), PixelPoint::new(959, 880));


/// Shared Undo All toolbar icon bounds and click point in guest pixels.
pub const SHARED_UNDO_ALL_CONTROL: ControlTarget = ControlTarget::new(
    PixelRect::new(1_301, 959, 38, 38),
    PixelPoint::new(1_320, 978),
);


/// Shared single Undo toolbar icon bounds and click point in guest pixels.
pub const SHARED_UNDO_CONTROL: ControlTarget = ControlTarget::new(
    PixelRect::new(1_661, 960, 37, 36),
    PixelPoint::new(1_680, 978),
);


/// One shared definition for both game previews and future control consumers.
pub const SHARED_TOOLBAR_CONTROLS: [(&str, ControlTarget); 3] = [
    ("Solver", SHARED_SOLVER_CONTROL),
    ("Undo All", SHARED_UNDO_ALL_CONTROL),
    ("Undo", SHARED_UNDO_CONTROL),
];


// Level Up has two observed layouts. Each probe remains paired with the click
// point calibrated for that layout so recognising one variant can never
// authorise the other's click. The raised probe uses a clean patch to the right
// of the OK text and remains disjoint from the proven legacy probe. A tall
// button can cover both probes; that case needs additional body evidence.
/// Observed Level Up button layouts, each with its own probe and click point.
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


// On a tall Level Up OK button, gold joins both layout bands across this
// interior strip. A high gold fraction here and in the left button interior
// can distinguish one tall button from two unrelated matching probes.
/// Guest-pixel gold strip joining both probes on a tall Level Up button.
pub const LEVEL_UP_SHARED_BUTTON_BRIDGE: PixelRect = PixelRect::new(1_010, 775, 50, 25);


/// Independent guest-pixel interior patch confirming a tall Level Up button.
pub const LEVEL_UP_SHARED_BUTTON_INTERIOR: PixelRect = PixelRect::new(945, 792, 115, 22);


/// Calibrated New Game button probe and click point in guest pixels.
pub const NEW_GAME_CONTROL_VARIANTS: [PostGameControlVariant; 1] = [PostGameControlVariant::new(
    "default",
    PixelRect::new(850, 850, 80, 30),
    PixelPoint::new(792, 840),
)];


/// Calibrated Play button probe and click point in guest pixels.
pub const PLAY_CONTROL_VARIANTS: [PostGameControlVariant; 1] = [PostGameControlVariant::new(
    "default",
    PixelRect::new(730, 905, 80, 30),
    PixelPoint::new(705, 905),
)];


/// Shared Solver control represented for post-game stage recognition.
pub const SOLVER_CONTROL_VARIANTS: [PostGameControlVariant; 1] = [PostGameControlVariant::new(
    "default",
    SHARED_SOLVER_CONTROL.bounds,
    SHARED_SOLVER_CONTROL.click_point,
)];


// The post-game sequence is ordered and fail-closed. Dialog stages use
// separate patches; the two Level Up layout bands can be part of one tall
// button only with positive bridge and interior evidence. Solver requires a
// gameplay frame with no halo before using its already-audited click point.
/// Ordered Level Up, New Game, Play and Solver control definitions.
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


// The Solver progress bar's four-pixel interior is at y=86..=89 in the
// calibrated 1920x1080 guest frame. Sample its middle two rows: y=84..=85
// is the gold top border, which stays gold even when the remainder is black.
// The right third is black before the final board and filled on game completion.
/// Guest-pixel patch inside the right third of the shared progress bar.
pub const SOLVER_PROGRESS_RIGHT_PROBE: PixelRect = PixelRect::new(1_050, 87, 38, 2);


/// Maximum value in every RGB channel classifying a progress pixel as black.
pub const SOLVER_PROGRESS_BLACK_CHANNEL_MAXIMUM: u8 = 32;


/// Black-pixel share, in thousandths, required to classify another board.
pub const SOLVER_PROGRESS_BLACK_FRACTION_PER_MILLE: u32 = 850;


/// Progress-bar calibration shared by TriPeaks and Pyramid completion checks.
pub const SHARED_SOLVER_PROGRESS_PROFILE: crate::game::GameProgressProfile =
    crate::game::GameProgressProfile {
        right_probe: SOLVER_PROGRESS_RIGHT_PROBE,
        black_channel_maximum: SOLVER_PROGRESS_BLACK_CHANNEL_MAXIMUM,
        black_fraction_per_mille: SOLVER_PROGRESS_BLACK_FRACTION_PER_MILLE,
    };


// Dialog buttons use a gold gradient rather than the exact two-colour halo.
// The ordered stage machine also requires the correct scene class and band.
/// Minimum 8-bit red channel accepted as a gold dialog-button pixel.
pub const DIALOG_GOLD_RED_MINIMUM: u8 = 170;


/// Minimum 8-bit green channel accepted as a gold dialog-button pixel.
pub const DIALOG_GOLD_GREEN_MINIMUM: u8 = 105;


/// Maximum 8-bit blue channel accepted as a gold dialog-button pixel.
pub const DIALOG_GOLD_BLUE_MAXIMUM: u8 = 190;


/// Minimum red-minus-green channel difference for a gold dialog button.
pub const DIALOG_GOLD_RED_GREEN_DELTA_MINIMUM: u8 = 8;


/// Minimum green-minus-blue channel difference for a gold dialog button.
pub const DIALOG_GOLD_GREEN_BLUE_DELTA_MINIMUM: u8 = 18;


/// Minimum gold share, in thousandths, inside a recognised dialog probe.
pub const DIALOG_GOLD_REQUIRED_FRACTION_PER_MILLE: u32 = 80;


// Convert a tableau gold-anchor coordinate to the intended card centre. The
// stock action uses qcode D and must not use this offset.
/// Horizontal guest-pixel offset from a TriPeaks gold anchor to the card centre.
pub const CLICK_OFFSET_X: i32 = 61;


/// Signed vertical guest-pixel offset from a TriPeaks halo to the card centre.
pub const CLICK_OFFSET_Y: i32 = -97;


/// Audited RGB halo colours used by the exact TriPeaks detector.
pub const GOLD_RGB_CANDIDATES: [[u8; 3]; 2] = [[237, 208, 109], [236, 207, 106]];


/// Maximum absolute 8-bit channel distance from an exact gold sample.
pub const GOLD_CHANNEL_TOLERANCE: u8 = 2;


// Compile-time gates remain independent so Multi-Step authority can be
// withdrawn without disabling the single-operation path. Runtime execution
// still revalidates fresh evidence before every planned input.
/// Compile-time authority gate for a single guarded gameplay operation.
pub const STEP_ONCE_INPUT_ENABLED: bool = true;


/// Independent compile-time authority gate for guarded multi-step execution.
pub const MULTI_STEP_INPUT_ENABLED: bool = true;


/// Gameplay-operation limit for Step Once.
pub const STEP_ONCE_ACTIONS: usize = 1;


/// Zero sentinel meaning no numeric operation limit; STOP remains available.
pub const UNBOUNDED_MULTI_STEP_ACTIONS: usize = 0;


/// Default multi-step operation limit, using the unbounded sentinel.
pub const DEFAULT_MULTI_STEP_ACTIONS: usize = UNBOUNDED_MULTI_STEP_ACTIONS;


/// Initial Klondike operation budget; zero selects continuous play until STOP.
pub const KLONDIKE_DEFAULT_MULTI_STEP_ACTIONS: usize = UNBOUNDED_MULTI_STEP_ACTIONS;


/// Largest finite Klondike operation budget; zero means continuous until stopped.
pub const KLONDIKE_MAX_MULTI_STEP_ACTIONS: usize = 10_000;


/// Initial Free Cell operation budget; zero selects continuous play until STOP.
pub const FREECELL_DEFAULT_MULTI_STEP_ACTIONS: usize = UNBOUNDED_MULTI_STEP_ACTIONS;


/// Largest finite Free Cell operation budget; zero remains continuous.
pub const FREECELL_MAX_MULTI_STEP_ACTIONS: usize = 10_000;


/// Default delayed Free Cell observations per unresolved animation or terminal stage.
pub const FREECELL_DEFAULT_OBSERVATION_LIMIT: usize = 20;


/// Largest editable Free Cell observation allowance; each context remains bounded.
pub const FREECELL_MAX_OBSERVATION_LIMIT: usize = 100;


// Allow Microsoft Solitaire's action-specific animation to finish before the
// first post-action screen dump. These constants remain the session defaults;
// the Params window can tune a bounded copy for the next guarded run.
/// Smallest editable animation-settle interval in milliseconds.
pub const MINIMUM_ANIMATION_SETTLE_DELAY_MS: u64 = 0;


/// Largest editable animation-settle interval in milliseconds.
pub const MAXIMUM_ANIMATION_SETTLE_DELAY_MS: u64 = 5_000;


/// Default TriPeaks draw-to-capture settling interval in milliseconds.
pub const DRAW_ANIMATION_SETTLE_DELAY_MS: u64 = 500;


/// Default TriPeaks card-to-capture settling interval in milliseconds.
pub const TABLEAU_ANIMATION_SETTLE_DELAY_MS: u64 = 750;


/// Default shared board-redeal settling interval in milliseconds.
pub const BOARD_REDEAL_SETTLE_DELAY_MS: u64 = 4_000;


/// Default Pyramid Move/Recycle-to-capture interval in milliseconds.
pub const PYRAMID_MOVE_SETTLE_DELAY_MS: u64 = 1_000;


/// Default Pyramid card or pile click-to-capture interval in milliseconds.
pub const PYRAMID_CARD_SETTLE_DELAY_MS: u64 = 2_000;


/// Default Pyramid no-halo recapture interval in milliseconds.
pub const PYRAMID_REOBSERVE_DELAY_MS: u64 = 1_000;


/// Default TriPeaks input-free interval while waiting for a late Solver halo.
pub const TRIPEAKS_REOBSERVE_DELAY_MS: u64 = 1_000;


/// Klondike action settling interval selected from Charlie's Beast gameplay.
pub const KLONDIKE_SETTLE_DELAY_MS: u64 = 750;


/// Separate editable Solve animation interval, retaining the existing delay
/// until Charlie measures a more suitable value on the Beast.
pub const KLONDIKE_SOLVE_SETTLE_DELAY_MS: u64 = KLONDIKE_SETTLE_DELAY_MS;


/// Initial Klondike input-free result recapture interval in milliseconds.
pub const KLONDIKE_REOBSERVE_DELAY_MS: u64 = 1_000;


/// Initial editable Free Cell action settle; Beast gameplay may refine it.
pub const FREECELL_SETTLE_DELAY_MS: u64 = 750;


/// Initial editable Free Cell input-free observation interval.
pub const FREECELL_REOBSERVE_DELAY_MS: u64 = 1_000;


/// Default TriPeaks draw settling interval as a typed duration.
pub const DRAW_ANIMATION_SETTLE_DELAY: Duration =
    Duration::from_millis(DRAW_ANIMATION_SETTLE_DELAY_MS);


/// Default TriPeaks tableau settling interval as a typed duration.
pub const TABLEAU_ANIMATION_SETTLE_DELAY: Duration =
    Duration::from_millis(TABLEAU_ANIMATION_SETTLE_DELAY_MS);


/// Default shared board-redeal settling interval as a typed duration.
pub const BOARD_REDEAL_SETTLE_DELAY: Duration = Duration::from_millis(BOARD_REDEAL_SETTLE_DELAY_MS);


// Retain the short wait for initial planning and existing effect-proof retries.
// Ordinary missing-HALO recovery uses its separate bounded, editable interval.
/// Short TriPeaks planning/effect recapture delay; not Solver-recovery timing.
pub const NO_HIGHLIGHT_REOBSERVE_DELAY: Duration = Duration::from_millis(150);


/// Number of boards in the calibrated shared Solver game flow.
pub const BOARDS_PER_GAME: usize = 3;


/// Settling interval after ordinary post-game control clicks.
pub const POST_GAME_STAGE_DELAY: Duration = Duration::from_secs(1);


// Give the score-counting transition time to finish after each centre click
// before looking for Level Up OK. Other post-game controls keep their settle.
/// Delay after score-skip clicks before looking for Level Up.
pub const LEVEL_UP_APPEAR_DELAY: Duration = Duration::from_secs(3);


/// Delay between observations of an unresolved board transition.
pub const BOARD_TRANSITION_REOBSERVE_DELAY: Duration = Duration::from_secs(2);


/// Maximum fresh captures while resolving one post-game stage.
pub const POST_GAME_MAX_OBSERVATION_ROUNDS: usize = 20;


/// Maximum independently recognised clicks for a post-game control.
pub const POST_GAME_MAX_CLICK_ATTEMPTS: usize = 3;


// The initial centre click counts towards this total. A confirmed QMP delivery
// may be repeated only while Level Up is the expected stage and fresh captures
// still do not recognise its OK control.
/// Maximum score-skip clicks, including the initial central click.
pub const SCORE_SKIP_MAX_CLICK_ATTEMPTS: usize = 3;


/// Pause after absolute pointer movement before a gameplay mouse press.
pub const POINTER_SETTLE_DELAY: Duration = Duration::from_millis(100);


/// Ordinary gameplay mouse-button hold duration.
pub const MOUSE_HOLD: Duration = Duration::from_millis(50);


// Post-game controls proved susceptible to a normal 50 ms click. A deliberate
// hold is paired with fresh visual transition verification before advancing.
/// Longer button hold used for calibrated post-game controls.
pub const POST_GAME_MOUSE_HOLD: Duration = Duration::from_millis(500);


/// Button hold used when enabling Solver highlighting.
pub const SOLVER_MOUSE_HOLD: Duration = Duration::from_millis(500);


/// Draw-key hold duration between QMP key-down and key-up.
pub const KEY_HOLD: Duration = Duration::from_millis(20);


/// Read/write timeout for ordinary QMP protocol traffic.
pub const QMP_IO_TIMEOUT: Duration = Duration::from_millis(750);


/// Extended QMP timeout for PNG screen dump generation.
pub const QMP_SCREENDUMP_TIMEOUT: Duration = Duration::from_secs(5);


/// Per-action durations copied into an immutable execution settings snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationSettleDelays {
    /// TriPeaks draw-to-capture settling duration.
    pub draw: Duration,
    /// TriPeaks tableau-click-to-capture settling duration.
    pub tableau: Duration,
    /// Delay between bounded TriPeaks observations before Solver recovery.
    pub tripeaks_reobserve: Duration,
    /// Shared duration allowed for the next board to finish dealing.
    pub board_redeal: Duration,
    /// Pyramid Move/Recycle-to-capture settling duration.
    pub pyramid_move: Duration,
    /// Pyramid tableau or lower-pile click-to-capture settling duration.
    pub pyramid_card: Duration,
    /// Delay before a fresh Pyramid capture when no eligible halo is visible.
    pub pyramid_reobserve: Duration,
    /// Klondike action-to-capture settling duration.
    pub klondike_settle: Duration,
    /// Klondike Solve animation delay before bounded completion observations.
    pub klondike_solve: Duration,
    /// Delay between bounded input-free Klondike result observations.
    pub klondike_reobserve: Duration,
    /// Free Cell click-to-capture settling duration, including automatic suit moves.
    pub freecell_settle: Duration,
    /// Delay between bounded input-free Free Cell result observations.
    pub freecell_reobserve: Duration,
}


impl AnimationSettleDelays {


    /// Set TriPeaks and redeal durations, retaining the other modes' defaults.
    pub const fn from_millis(draw_ms: u64, tableau_ms: u64, board_redeal_ms: u64) -> Self {
        Self {
            draw: Duration::from_millis(draw_ms),
            tableau: Duration::from_millis(tableau_ms),
            tripeaks_reobserve: Duration::from_millis(TRIPEAKS_REOBSERVE_DELAY_MS),
            board_redeal: Duration::from_millis(board_redeal_ms),
            pyramid_move: Duration::from_millis(PYRAMID_MOVE_SETTLE_DELAY_MS),
            pyramid_card: Duration::from_millis(PYRAMID_CARD_SETTLE_DELAY_MS),
            pyramid_reobserve: Duration::from_millis(PYRAMID_REOBSERVE_DELAY_MS),
            klondike_settle: Duration::from_millis(KLONDIKE_SETTLE_DELAY_MS),
            klondike_solve: Duration::from_millis(KLONDIKE_SOLVE_SETTLE_DELAY_MS),
            klondike_reobserve: Duration::from_millis(KLONDIKE_REOBSERVE_DELAY_MS),
            freecell_settle: Duration::from_millis(FREECELL_SETTLE_DELAY_MS),
            freecell_reobserve: Duration::from_millis(FREECELL_REOBSERVE_DELAY_MS),
        }
    }


    /// Apply the bounded TriPeaks recapture interval to the next run snapshot.
    pub fn with_tripeaks_reobserve_millis(mut self, reobserve_ms: u64) -> Self {
        self.tripeaks_reobserve = Duration::from_millis(reobserve_ms.clamp(
            MINIMUM_ANIMATION_SETTLE_DELAY_MS,
            MAXIMUM_ANIMATION_SETTLE_DELAY_MS,
        ));
        self
    }


    /// Apply the session's Pyramid timing without changing TriPeaks timing.
    /// Clamp here as well as in Params so the worker snapshot stays bounded.
    pub fn with_pyramid_millis(mut self, move_ms: u64, card_ms: u64, reobserve_ms: u64) -> Self {
        let bounded = |milliseconds: u64| {
            Duration::from_millis(milliseconds.clamp(
                MINIMUM_ANIMATION_SETTLE_DELAY_MS,
                MAXIMUM_ANIMATION_SETTLE_DELAY_MS,
            ))
        };
        self.pyramid_move = bounded(move_ms);
        self.pyramid_card = bounded(card_ms);
        self.pyramid_reobserve = bounded(reobserve_ms);
        self
    }


    /// Apply bounded Klondike timing without changing another mode's settings.
    pub fn with_klondike_millis(mut self, settle_ms: u64, reobserve_ms: u64) -> Self {
        let bounded = |milliseconds: u64| {
            Duration::from_millis(milliseconds.clamp(
                MINIMUM_ANIMATION_SETTLE_DELAY_MS,
                MAXIMUM_ANIMATION_SETTLE_DELAY_MS,
            ))
        };
        self.klondike_settle = bounded(settle_ms);
        self.klondike_reobserve = bounded(reobserve_ms);
        self
    }


    /// Apply bounded Free Cell timing without changing another mode's settings.
    pub fn with_freecell_millis(mut self, settle_ms: u64, reobserve_ms: u64) -> Self {
        let bounded = |milliseconds: u64| {
            Duration::from_millis(milliseconds.clamp(
                MINIMUM_ANIMATION_SETTLE_DELAY_MS,
                MAXIMUM_ANIMATION_SETTLE_DELAY_MS,
            ))
        };
        self.freecell_settle = bounded(settle_ms);
        self.freecell_reobserve = bounded(reobserve_ms);
        self
    }


    /// Set the separately editable Solve delay without changing card/draw timing.
    pub fn with_klondike_solve_millis(mut self, solve_ms: u64) -> Self {
        self.klondike_solve = Duration::from_millis(solve_ms.clamp(
            MINIMUM_ANIMATION_SETTLE_DELAY_MS,
            MAXIMUM_ANIMATION_SETTLE_DELAY_MS,
        ));
        self
    }
}


impl Default for AnimationSettleDelays {


    /// Return per-mode defaults, including the observed Klondike action interval.
    fn default() -> Self {
        Self {
            draw: DRAW_ANIMATION_SETTLE_DELAY,
            tableau: TABLEAU_ANIMATION_SETTLE_DELAY,
            tripeaks_reobserve: Duration::from_millis(TRIPEAKS_REOBSERVE_DELAY_MS),
            board_redeal: BOARD_REDEAL_SETTLE_DELAY,
            pyramid_move: Duration::from_millis(PYRAMID_MOVE_SETTLE_DELAY_MS),
            pyramid_card: Duration::from_millis(PYRAMID_CARD_SETTLE_DELAY_MS),
            pyramid_reobserve: Duration::from_millis(PYRAMID_REOBSERVE_DELAY_MS),
            klondike_settle: Duration::from_millis(KLONDIKE_SETTLE_DELAY_MS),
            klondike_solve: Duration::from_millis(KLONDIKE_SOLVE_SETTLE_DELAY_MS),
            klondike_reobserve: Duration::from_millis(KLONDIKE_REOBSERVE_DELAY_MS),
            freecell_settle: Duration::from_millis(FREECELL_SETTLE_DELAY_MS),
            freecell_reobserve: Duration::from_millis(FREECELL_REOBSERVE_DELAY_MS),
        }
    }
}


/// Immutable parameters captured when a Step Once or Multi-Step run begins.
///
/// Keeping one snapshot prevents live UI edits from changing animation timing
/// or the authority limit part-way through a run.
/// An operation limit of zero means continuous operation until STOP or a guarded
/// stop condition. Only continuous runs may advance through each mode's verified
/// game-completion and restart policy. Klondike owns its single-board win and
/// terminal controls independently of the shared three-board games.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepRunSettings {
    /// Action timings frozen when the run starts.
    animation_delays: AnimationSettleDelays,
    /// Gameplay operation limit; zero means unbounded until STOP or uncertainty.
    operation_limit: usize,
    /// Delayed input-free Free Cell captures permitted for one unresolved context.
    freecell_observation_limit: usize,
}


impl StepRunSettings {


    /// Freeze timing and operation authority for one guarded run.
    pub const fn new(animation_delays: AnimationSettleDelays, operation_limit: usize) -> Self {
        Self {
            animation_delays,
            operation_limit,
            freecell_observation_limit: FREECELL_DEFAULT_OBSERVATION_LIMIT,
        }
    }


    /// Return the immutable timing values captured when the run began.
    pub const fn animation_delays(self) -> AnimationSettleDelays {
        self.animation_delays
    }


    /// Return the raw operation limit; zero represents an unbounded run.
    pub const fn operation_limit(self) -> usize {
        self.operation_limit
    }


    /// Freeze a bounded Free Cell observation allowance without changing other modes.
    pub fn with_freecell_observation_limit(mut self, limit: usize) -> Self {
        self.freecell_observation_limit = limit.clamp(1, FREECELL_MAX_OBSERVATION_LIMIT);
        self
    }


    /// Return the snapshotted delayed observation allowance for each Free Cell context.
    pub const fn freecell_observation_limit(self) -> usize {
        self.freecell_observation_limit
    }


    /// Report whether this run uses the zero sentinel for unlimited operations.
    pub const fn is_unbounded(self) -> bool {
        self.operation_limit == UNBOUNDED_MULTI_STEP_ACTIONS
    }


    /// Return a finite operation count, or None for an unbounded run.
    pub const fn bounded_operation_limit(self) -> Option<usize> {


        if self.is_unbounded() {
            None
        } else {
            Some(self.operation_limit)
        }
    }
}


impl Default for StepRunSettings {


    /// Return the calibrated default values for a newly created settings snapshot.
    fn default() -> Self {
        Self::new(AnimationSettleDelays::default(), DEFAULT_MULTI_STEP_ACTIONS)
    }
}


/// Application-specific Unix socket basename used by default path discovery.
pub const QMP_SOCKET_FILENAME: &str = "qmp-qemu-socket.sock";


/// Resolve QMP_SOCKET_PATH first, then runtime-directory and temporary fallbacks.
pub fn default_qmp_socket_path() -> PathBuf {


    // An explicit path keeps an existing QEMU launch usable during the socket
    // rename; otherwise use this application's own socket filename.
    if let Some(path) = env::var_os("QMP_SOCKET_PATH").filter(|path| !path.is_empty()) {
        return PathBuf::from(path);
    }
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
    //! Calibration bounds, control ordering and immutable run-setting regressions.

    use super::*;
    use crate::{
        cards::{TABLEAU_ROW_LENGTHS, TABLEAU_ROW_STARTS, validate_card_regions},
        geometry::{QmpPoint, pixel_point_to_qmp},
    };


    /// Assert that a non-empty calibrated rectangle fits the nominal guest frame.
    fn assert_rect_fits_frame(rect: PixelRect) {
        assert!(!rect.is_empty());
        assert!(rect.right() <= NOMINAL_FRAME_WIDTH);
        assert!(rect.bottom() <= NOMINAL_FRAME_HEIGHT);
    }


    /// Verify that action-specific defaults and custom timings are not conflated.
    #[test]
    fn animation_settle_defaults_and_custom_values_remain_distinct() {
        let defaults = AnimationSettleDelays::default();
        assert_eq!(defaults.draw, DRAW_ANIMATION_SETTLE_DELAY);
        assert_eq!(defaults.tableau, TABLEAU_ANIMATION_SETTLE_DELAY);
        assert_eq!(defaults.board_redeal, BOARD_REDEAL_SETTLE_DELAY);
        assert_eq!(DRAW_ANIMATION_SETTLE_DELAY_MS, 500);
        assert_eq!(TABLEAU_ANIMATION_SETTLE_DELAY_MS, 750);
        assert_eq!(BOARD_REDEAL_SETTLE_DELAY_MS, 4_000);


        const {
            assert!(DRAW_ANIMATION_SETTLE_DELAY_MS <= MAXIMUM_ANIMATION_SETTLE_DELAY_MS);
            assert!(TABLEAU_ANIMATION_SETTLE_DELAY_MS <= MAXIMUM_ANIMATION_SETTLE_DELAY_MS);
            assert!(BOARD_REDEAL_SETTLE_DELAY_MS <= MAXIMUM_ANIMATION_SETTLE_DELAY_MS);
        }

        let custom = AnimationSettleDelays::from_millis(123, 987, 2_345);
        assert_eq!(custom.draw, Duration::from_millis(123));
        assert_eq!(custom.tableau, Duration::from_millis(987));
        assert_eq!(custom.board_redeal, Duration::from_millis(2_345));
        assert_eq!(BOARDS_PER_GAME, 3);
        assert_eq!(BOARD_REDEAL_SETTLE_DELAY, Duration::from_secs(4));
        assert_eq!(POST_GAME_STAGE_DELAY, Duration::from_secs(1));
        assert_eq!(LEVEL_UP_APPEAR_DELAY, Duration::from_secs(3));
        assert_eq!(POST_GAME_MAX_OBSERVATION_ROUNDS, 20);
        assert_eq!(POST_GAME_MAX_CLICK_ATTEMPTS, 3);
        assert_eq!(SCORE_SKIP_MAX_CLICK_ATTEMPTS, 3);
        assert_eq!(NO_HIGHLIGHT_REOBSERVE_DELAY, Duration::from_millis(150));
        assert_eq!(MOUSE_HOLD, Duration::from_millis(50));
        assert_eq!(POST_GAME_MOUSE_HOLD, Duration::from_millis(500));
        assert_eq!(SOLVER_MOUSE_HOLD, Duration::from_millis(500));
    }


    /// Verify timing snapshots and finite versus unbounded operation limits.
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


    /// Recapture edits are bounded, mode-local and frozen for the active run.
    #[test]
    fn tripeaks_reobserve_is_bounded_and_snapshotted() {
        let defaults = AnimationSettleDelays::default();
        assert_eq!(defaults.tripeaks_reobserve, Duration::from_millis(1_000));
        let edited = defaults.with_tripeaks_reobserve_millis(1_250);
        let active = StepRunSettings::new(edited, 0);
        let next = edited.with_tripeaks_reobserve_millis(u64::MAX);
        assert_eq!(active.animation_delays().tripeaks_reobserve, Duration::from_millis(1_250));
        assert_eq!(next.tripeaks_reobserve, Duration::from_millis(5_000));
        assert_eq!(edited.with_tripeaks_reobserve_millis(0).tripeaks_reobserve, Duration::ZERO);
        assert_eq!(next.draw, defaults.draw);
        assert_eq!(next.tableau, defaults.tableau);
        assert_eq!(next.pyramid_reobserve, defaults.pyramid_reobserve);
        assert_eq!(next.klondike_reobserve, defaults.klondike_reobserve);
    }


    /// Verify Pyramid timing clamping and independent TriPeaks settings.
    #[test]
    fn pyramid_timing_is_bounded_and_snapshotted_without_changing_tripeaks() {
        let defaults = AnimationSettleDelays::default();
        assert_eq!(defaults.pyramid_move, Duration::from_millis(1_000));
        assert_eq!(defaults.pyramid_card, Duration::from_millis(2_000));
        assert_eq!(defaults.pyramid_reobserve, Duration::from_millis(1_000));

        let mut next_run = AnimationSettleDelays::from_millis(123, 987, 2_345)
            .with_pyramid_millis(650, 1_250, 900);
        let active_run = StepRunSettings::new(next_run, 4);
        next_run = next_run.with_pyramid_millis(0, u64::MAX, u64::MAX);
        assert_eq!(
            active_run.animation_delays().pyramid_move,
            Duration::from_millis(650)
        );
        assert_eq!(
            active_run.animation_delays().pyramid_card,
            Duration::from_millis(1_250)
        );
        assert_eq!(
            active_run.animation_delays().pyramid_reobserve,
            Duration::from_millis(900)
        );
        assert_eq!(next_run.pyramid_move, Duration::ZERO);
        assert_eq!(next_run.pyramid_card, Duration::from_millis(5_000));
        assert_eq!(next_run.pyramid_reobserve, Duration::from_millis(5_000));
        assert_eq!(next_run.draw, Duration::from_millis(123));
        assert_eq!(next_run.tableau, Duration::from_millis(987));
        assert_eq!(next_run.board_redeal, Duration::from_millis(2_345));
    }


    /// Verify that Klondike settings are bounded and independent in active runs.
    #[test]
    fn klondike_timing_and_budget_defaults_are_independent() {
        let defaults = AnimationSettleDelays::default();
        assert_eq!(defaults.klondike_settle, Duration::from_millis(750));
        assert_eq!(defaults.klondike_solve, Duration::from_millis(750));
        assert_eq!(defaults.klondike_reobserve, Duration::from_millis(1_000));
        let next_run = AnimationSettleDelays::from_millis(123, 987, 2_345)
            .with_pyramid_millis(500, 750, 900)
            .with_klondike_millis(1_500, 250)
            .with_klondike_solve_millis(2_000);
        let active_run = StepRunSettings::new(next_run, KLONDIKE_DEFAULT_MULTI_STEP_ACTIONS);
        let changed = next_run.with_klondike_millis(0, u64::MAX);
        assert_eq!(
            active_run.animation_delays().klondike_settle,
            Duration::from_millis(1_500)
        );
        assert_eq!(
            active_run.animation_delays().klondike_reobserve,
            Duration::from_millis(250)
        );
        assert_eq!(changed.klondike_settle, Duration::ZERO);
        assert_eq!(changed.klondike_reobserve, Duration::from_millis(5_000));
        assert_eq!(active_run.animation_delays().klondike_solve, Duration::from_millis(2_000));
        assert_eq!(changed.klondike_solve, next_run.klondike_solve);
        assert_eq!(next_run.with_klondike_solve_millis(u64::MAX).klondike_solve, Duration::from_millis(5_000));
        assert_eq!(next_run.with_klondike_solve_millis(0).klondike_solve, Duration::ZERO);
        assert_eq!(changed.draw, next_run.draw);
        assert_eq!(changed.tableau, next_run.tableau);
        assert_eq!(changed.board_redeal, next_run.board_redeal);
        assert_eq!(changed.pyramid_move, next_run.pyramid_move);
        assert_eq!(changed.pyramid_card, next_run.pyramid_card);
        assert_eq!(changed.pyramid_reobserve, next_run.pyramid_reobserve);
        assert_eq!(active_run.bounded_operation_limit(), None);
        assert!(active_run.is_unbounded());
        assert_eq!(KLONDIKE_MAX_MULTI_STEP_ACTIONS, 10_000);
        let continuous_run = StepRunSettings::new(defaults, UNBOUNDED_MULTI_STEP_ACTIONS);
        assert!(continuous_run.is_unbounded());
        assert_eq!(continuous_run.bounded_operation_limit(), None);
        assert_eq!(
            continuous_run.animation_delays().klondike_settle,
            Duration::from_millis(750)
        );
    }


    /// Free Cell timings are independent, bounded and immutable for an active run.
    #[test]
    fn freecell_timing_and_budget_are_bounded_and_snapshotted() {
        let defaults = AnimationSettleDelays::default();
        assert_eq!(defaults.freecell_settle, Duration::from_millis(750));
        assert_eq!(defaults.freecell_reobserve, Duration::from_millis(1_000));
        let edited = defaults.with_freecell_millis(1_250, 850);
        let active = StepRunSettings::new(edited, FREECELL_DEFAULT_MULTI_STEP_ACTIONS);
        let next = edited.with_freecell_millis(0, u64::MAX);
        assert_eq!(active.animation_delays().freecell_settle, Duration::from_millis(1_250));
        assert_eq!(active.animation_delays().freecell_reobserve, Duration::from_millis(850));
        assert_eq!(next.freecell_settle, Duration::ZERO);
        assert_eq!(next.freecell_reobserve, Duration::from_millis(5_000));
        assert_eq!(next.with_freecell_millis(u64::MAX, 0).freecell_settle, Duration::from_millis(5_000));
        assert_eq!(next.with_freecell_millis(u64::MAX, 0).freecell_reobserve, Duration::ZERO);
        assert_eq!(next.draw, defaults.draw);
        assert_eq!(next.tableau, defaults.tableau);
        assert_eq!(next.tripeaks_reobserve, defaults.tripeaks_reobserve);
        assert_eq!(next.pyramid_move, defaults.pyramid_move);
        assert_eq!(next.pyramid_card, defaults.pyramid_card);
        assert_eq!(next.pyramid_reobserve, defaults.pyramid_reobserve);
        assert_eq!(next.klondike_settle, defaults.klondike_settle);
        assert_eq!(next.klondike_solve, defaults.klondike_solve);
        assert_eq!(next.klondike_reobserve, defaults.klondike_reobserve);
        assert_eq!(next.board_redeal, defaults.board_redeal);
        assert!(active.is_unbounded());
        assert_eq!(active.bounded_operation_limit(), None);
        assert_eq!(active.freecell_observation_limit(), 20);
        assert_eq!(active.with_freecell_observation_limit(0).freecell_observation_limit(), 1);
        assert_eq!(active.with_freecell_observation_limit(usize::MAX).freecell_observation_limit(), 100);
        assert_eq!(active.with_freecell_observation_limit(7).freecell_observation_limit(), 7);
    }


    /// Return the last included pixel of a non-empty half-open rectangle.
    fn final_pixel(rect: PixelRect) -> PixelPoint {
        PixelPoint::new((rect.right() - 1) as i32, (rect.bottom() - 1) as i32)
    }


    /// Lock active geometry and detector thresholds to the audited capture.
    #[test]
    fn calibration_matches_verified_capture() {
        // Lock the constants to the audited 1920x1080 capture evidence.
        assert_eq!((NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT), (1_920, 1_080));
        assert_eq!(TARGET_BOARD, PixelRect::new(112, 96, 1_696, 544));
        assert_eq!(TARGET_STOCK, PixelRect::new(700, 665, 280, 217));
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
        assert_eq!(ACTION_CURSOR_EXCLUSION_HALF_SIZE, 48);
    }


    /// Verify stage ordering and the calibrated control variants and hit points.
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
            SHARED_SOLVER_CONTROL.click_point
        );
        assert!(POST_GAME_TARGETS[3].requires_gameplay_scene);
        assert_eq!(SCORE_SKIP_CONTROL.click_point, PixelPoint::new(960, 540));
        assert_eq!(
            CHALLENGE_COMPLETE_CONTINUE_CONTROL.bounds,
            PixelRect::new(805, 844, 308, 72)
        );
        assert_eq!(
            CHALLENGE_COMPLETE_CONTINUE_CONTROL.click_point,
            PixelPoint::new(959, 880)
        );
        assert!(
            CHALLENGE_COMPLETE_CONTINUE_CONTROL
                .bounds
                .contains(CHALLENGE_COMPLETE_CONTINUE_CONTROL.click_point)
        );
        assert_eq!(
            SOLVER_PROGRESS_RIGHT_PROBE,
            PixelRect::new(1_050, 87, 38, 2)
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


    /// Verify half-open bounds of active TriPeaks detector regions.
    #[test]
    fn calibrated_regions_fit_the_frame() {
        // Check half-open bounds for every active detector region.
        assert_eq!((TARGET_BOARD.right(), TARGET_BOARD.bottom()), (1_808, 640));
        assert_eq!((TARGET_STOCK.right(), TARGET_STOCK.bottom()), (980, 882));
        assert_eq!((TARGET_WASTE.right(), TARGET_WASTE.bottom()), (1_171, 882));
        assert!(TARGET_BOARD.right() <= NOMINAL_FRAME_WIDTH);
        assert!(TARGET_BOARD.bottom() <= NOMINAL_FRAME_HEIGHT);
        assert!(TARGET_STOCK.right() <= NOMINAL_FRAME_WIDTH);
        assert!(TARGET_STOCK.bottom() <= NOMINAL_FRAME_HEIGHT);
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


    /// Verify that the TriPeaks scene gate uses the audited felt thresholds.
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


        const { assert!(GAMEPLAY_FELT_REQUIRED_FRACTION_PER_MILLE <= 1_000); }
    }


    /// Verify all TriPeaks slot coordinates against the audited row layout.
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


    /// Verify row indexing, probe envelopes and their relationships to card geometry.
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


    /// Verify that each TriPeaks halo anchor maps to its calibrated card centre.
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


    /// Verify card-region containment and QMP mapping for all calibrated slots.
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

            validate_card_regions(*pixels, NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT)
                .expect("calibrated card geometry must convert to QMP coordinates");
        }
    }


    /// Verify distinct stock/waste geometry and their audited rank and halo bounds.
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


    /// Verify shared toolbar click points map to the recorded QMP coordinates.
    #[test]
    fn control_targets_map_to_the_audited_qmp_coordinates() {
        let cases = [
            (SHARED_SOLVER_CONTROL, QmpPoint::new(10_273, 29_641)),
            (SHARED_UNDO_ALL_CONTROL, QmpPoint::new(22_527, 29_672)),
            (SHARED_UNDO_CONTROL, QmpPoint::new(28_671, 29_672)),
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


    /// Verify the sample halo anchor plus calibrated offsets gives the card centre.
    #[test]
    fn click_offset_maps_audited_anchor_to_card_centre() {
        // The calibrated board highlight began at (1662, 630).
        let anchor = PixelPoint::new(1_662, 630);
        let centre = PixelPoint::new(anchor.x + CLICK_OFFSET_X, anchor.y + CLICK_OFFSET_Y);
        assert_eq!(centre, PixelPoint::new(1_723, 533));
        assert_eq!(TABLEAU_CARD_REGIONS[27].click_point, centre);
        assert!((1_655..1_792).contains(&centre.x));
        assert!((443..624).contains(&centre.y));
    }
}
