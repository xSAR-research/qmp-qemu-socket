//! Klondike's evidenced one-board win dialogs and restart controls.
//!
//! K26-K30 provide the native 1920x1080 scene, title and button signatures.
//! K43 confirms the Congratulations signature on a later level and reward frame.
//! K52 retains the same OK geometry while the centre-aligned game label moves
//! two native pixels right as the level number changes. The complete mixed label
//! signature uses one coherent evidenced position; numeric text remains excluded.
//! K49 records the same Level Up controls under warm animated fireworks. Only
//! the four empty-tableau context samples permit measured red/green particle
//! lighting; their blue limit and full-RGB foundation context remain independent.
//! K53 retains New Game/Home artwork while the pointer from Level Up OK covers
//! the Home button's former upper-body sample. Its replacement uses identical
//! measured pixels in K28/K53 outside the recorded pointer, with no relaxed guard.
//! Stable RGB artwork is checked directly; changing level, score, XP and card
//! ranks are not read. A recognised terminal stage grants only its measured
//! click. The worker still owns fresh validation, cancellation, delivery and
//! bounded input-free transition observations. No absent black pixel or generic
//! gold button authorises a win or restart.

use std::{fmt, time::Duration};

use crate::{
    capture::CapturedFrame,
    detector::{HaloDetectionError, pixel_rgb},
    geometry::{PixelPoint, PixelRect},
    parameters::{
        LEVEL_UP_APPEAR_DELAY, LEVEL_UP_CONTROL_VARIANTS, NEW_GAME_CONTROL_VARIANTS,
        POST_GAME_MOUSE_HOLD, POST_GAME_STAGE_DELAY, SCORE_SKIP_CONTROL,
        SHARED_SOLVER_CONTROL, SOLVER_MOUSE_HOLD,
    },
};


/// Independently recognised stage of Charlie's observed Klondike restart sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TerminalStage {
    /// Completed-board Congratulations panel still counting its reward.
    ScoreCounting,
    /// Klondike Level Up dialog with the measured low OK button.
    LevelUp,
    /// Completed-board Congratulations panel with New Game and Home buttons.
    NewGame,
    /// Klondike Standard Draw 1 selector retaining the guest's prior difficulty.
    Play,
    /// Fresh seven-column deal with Solver off and its toolbar icon visible.
    SolverReady,
}


impl TerminalStage {


    /// Whether this positively recognised stage is itself completed-game evidence.
    /// The classifier requires occupied foundations and empty side-tableau context.
    pub(crate) const fn demonstrates_game_win(self) -> bool {
        matches!(self, Self::ScoreCounting | Self::LevelUp | Self::NewGame)
    }


    /// One measured native click; the shifted Klondike Play button is mode-owned.
    pub(crate) const fn click_point(self) -> PixelPoint {


        match self {
            Self::ScoreCounting => SCORE_SKIP_CONTROL.click_point,
            Self::LevelUp => LEVEL_UP_CONTROL_VARIANTS[0].click_point,
            Self::NewGame => NEW_GAME_CONTROL_VARIANTS[0].click_point,
            Self::Play => PixelPoint::new(705, 860),
            Self::SolverReady => SHARED_SOLVER_CONTROL.click_point,
        }
    }


    /// Existing deliberate control hold; no new input timing is inferred from PNGs.
    pub(crate) const fn mouse_hold(self) -> Duration {


        match self {
            Self::SolverReady => SOLVER_MOUSE_HOLD,
            _ => POST_GAME_MOUSE_HOLD,
        }
    }


    /// Existing transition delays; dealing uses the run's editable redeal setting.
    pub(crate) const fn settle_delay(self, board_redeal: Duration) -> Duration {


        match self {
            Self::ScoreCounting => LEVEL_UP_APPEAR_DELAY,
            Self::Play => board_redeal,
            _ => POST_GAME_STAGE_DELAY,
        }
    }
}


impl fmt::Display for TerminalStage {


    /// Describe the verified control without promoting its acknowledgement to effect.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ScoreCounting => "Klondike score counting / click to skip",
            Self::LevelUp => "Klondike Level Up OK",
            Self::NewGame => "Klondike Congratulations New Game",
            Self::Play => "Klondike Standard Draw 1 Play",
            Self::SolverReady => "Klondike fresh-deal Solver",
        })
    }
}


/// One original native artwork sample; every channel must remain near its value.
#[derive(Clone, Copy)]
struct RgbSample {
    /// Native guest column, checked by the parent frame contract before use.
    x: u32,
    /// Native guest row, checked by the parent frame contract before use.
    y: u32,
    /// Unmodified RGB8 value from the recorded fixture at this coordinate.
    rgb: [u8; 3],
}


impl RgbSample {


    /// Construct a fixed sample without decoding a template at runtime.
    const fn new(x: u32, y: u32, rgb: [u8; 3]) -> Self {
        Self { x, y, rgb }
    }
}


/// Maximum channel difference for stable title/button artwork under antialiasing.
const ARTWORK_CHANNEL_TOLERANCE: u8 = 18;


/// Tight channel tolerance keeps dimmed foundation paper distinct from empty felt.
const BOARD_CONTEXT_CHANNEL_TOLERANCE: u8 = 8;


/// Largest measured positive red shift at the four K49 side-tableau samples.
const LEVEL_UP_FIREWORK_RED_INCREASE: u8 = 53;


/// Largest measured positive green shift at the four K49 side-tableau samples.
const LEVEL_UP_FIREWORK_GREEN_INCREASE: u8 = 17;


/// Original stable artwork samples from K26; numeric level/score text is excluded.
const CONGRATULATIONS_TITLE: [RgbSample; 10] = [
    RgbSample::new(728, 239, [252, 254, 254]),
    RgbSample::new(778, 219, [255, 255, 255]),
    RgbSample::new(831, 236, [255, 255, 255]),
    RgbSample::new(883, 218, [254, 255, 255]),
    RgbSample::new(933, 236, [254, 255, 255]),
    RgbSample::new(983, 216, [250, 253, 254]),
    RgbSample::new(1033, 236, [255, 255, 255]),
    RgbSample::new(1079, 216, [250, 253, 254]),
    RgbSample::new(1134, 236, [255, 255, 255]),
    RgbSample::new(1184, 216, [251, 253, 254]),
];


/// Original stable artwork samples from K26; numeric level/score text is excluded.
const SCORE_SKIP_CAPTION: [RgbSample; 10] = [
    RgbSample::new(829, 856, [139, 167, 204]),
    RgbSample::new(854, 848, [139, 168, 204]),
    RgbSample::new(863, 861, [138, 167, 203]),
    RgbSample::new(912, 848, [139, 168, 205]),
    RgbSample::new(945, 860, [139, 167, 203]),
    RgbSample::new(978, 853, [76, 120, 178]),
    RgbSample::new(1011, 860, [128, 159, 198]),
    RgbSample::new(1044, 860, [74, 124, 182]),
    RgbSample::new(1079, 860, [75, 118, 174]),
    RgbSample::new(1109, 848, [77, 121, 178]),
];


/// Original stable artwork samples from K27; numeric level/score text is excluded.
const LEVEL_UP_TITLE: [RgbSample; 10] = [
    RgbSample::new(816, 217, [255, 255, 255]),
    RgbSample::new(849, 199, [255, 255, 255]),
    RgbSample::new(883, 217, [255, 255, 255]),
    RgbSample::new(920, 199, [255, 255, 255]),
    RgbSample::new(944, 224, [254, 255, 255]),
    RgbSample::new(962, 199, [255, 255, 255]),
    RgbSample::new(1014, 217, [255, 255, 255]),
    RgbSample::new(1041, 199, [255, 255, 255]),
    RgbSample::new(1069, 217, [255, 255, 255]),
    RgbSample::new(1103, 199, [255, 255, 255]),
];


/// Evidenced horizontal positions of the complete centre-aligned game label.
/// K27/K49 use zero; K52 changes the level text width and shifts it two pixels.
/// No free-position search or different offsets per sample are permitted.
const LEVEL_UP_LABEL_OFFSETS: [u32; 2] = [0, 2];


/// Original stable artwork samples from K27; numeric level/score text is excluded.
const LEVEL_UP_KLONDIKE: [RgbSample; 10] = [
    RgbSample::new(772, 630, [255, 255, 255]),
    RgbSample::new(792, 620, [255, 255, 255]),
    RgbSample::new(813, 630, [255, 255, 255]),
    RgbSample::new(835, 620, [255, 255, 255]),
    RgbSample::new(857, 630, [255, 255, 255]),
    RgbSample::new(878, 620, [255, 255, 255]),
    RgbSample::new(899, 630, [255, 255, 255]),
    RgbSample::new(923, 620, [255, 255, 255]),
    RgbSample::new(939, 632, [255, 255, 255]),
    RgbSample::new(963, 624, [255, 255, 255]),
];


/// Original stable artwork samples from K27; numeric level/score text is excluded.
const LEVEL_UP_OK: [RgbSample; 10] = [
    RgbSample::new(936, 816, [0, 0, 0]),
    RgbSample::new(938, 804, [0, 0, 0]),
    RgbSample::new(946, 821, [2, 1, 0]),
    RgbSample::new(954, 805, [2, 2, 1]),
    RgbSample::new(956, 816, [0, 0, 0]),
    RgbSample::new(965, 806, [0, 0, 0]),
    RgbSample::new(966, 816, [0, 0, 0]),
    RgbSample::new(971, 807, [7, 7, 4]),
    RgbSample::new(976, 817, [0, 0, 0]),
    RgbSample::new(975, 804, [0, 0, 0]),
];


/// Positive gold inside/between the OK letters, identical or one red level apart
/// in K27/K49. These samples reject an erased flat-black glyph area while the
/// surrounding button corners remain intact.
const LEVEL_UP_OK_BACKGROUND: [RgbSample; 4] = [
    RgbSample::new(943, 810, [235, 219, 124]),
    RgbSample::new(943, 816, [229, 189, 82]),
    RgbSample::new(961, 804, [246, 234, 161]),
    RgbSample::new(961, 820, [211, 164, 62]),
];


/// Original stable artwork samples from K28; numeric level/score text is excluded.
const NEW_GAME_TEXT: [RgbSample; 10] = [
    RgbSample::new(701, 860, [0, 0, 0]),
    RgbSample::new(726, 850, [0, 0, 0]),
    RgbSample::new(738, 860, [0, 0, 0]),
    RgbSample::new(765, 848, [0, 0, 0]),
    RgbSample::new(784, 857, [1, 1, 0]),
    RgbSample::new(799, 850, [0, 0, 0]),
    RgbSample::new(818, 860, [1, 1, 0]),
    RgbSample::new(837, 847, [0, 0, 0]),
    RgbSample::new(851, 860, [0, 0, 0]),
    RgbSample::new(871, 849, [0, 0, 0]),
];


/// Original stable artwork samples from K28; numeric level/score text is excluded.
const NEW_GAME_HOME: [RgbSample; 10] = [
    RgbSample::new(1082, 859, [255, 255, 255]),
    RgbSample::new(1093, 849, [255, 255, 255]),
    RgbSample::new(1107, 858, [253, 254, 254]),
    RgbSample::new(1115, 846, [255, 255, 255]),
    RgbSample::new(1129, 859, [254, 254, 255]),
    RgbSample::new(1137, 846, [255, 255, 255]),
    RgbSample::new(1151, 859, [246, 249, 252]),
    RgbSample::new(1161, 848, [252, 253, 254]),
    RgbSample::new(1171, 860, [254, 254, 255]),
    RgbSample::new(1173, 850, [255, 255, 255]),
];


/// Original stable artwork samples from K29; numeric level/score text is excluded.
const PLAY_KLONDIKE_TITLE: [RgbSample; 10] = [
    RgbSample::new(684, 192, [255, 255, 255]),
    RgbSample::new(699, 183, [255, 255, 255]),
    RgbSample::new(712, 194, [255, 255, 255]),
    RgbSample::new(732, 183, [255, 255, 255]),
    RgbSample::new(744, 194, [255, 255, 255]),
    RgbSample::new(758, 183, [255, 255, 255]),
    RgbSample::new(770, 194, [255, 255, 255]),
    RgbSample::new(789, 183, [255, 255, 255]),
    RgbSample::new(807, 194, [255, 255, 255]),
    RgbSample::new(819, 184, [255, 255, 255]),
];


/// Original stable artwork samples from K29; numeric level/score text is excluded.
const PLAY_DRAW_ONE: [RgbSample; 10] = [
    RgbSample::new(661, 318, [255, 255, 255]),
    RgbSample::new(685, 309, [255, 255, 255]),
    RgbSample::new(711, 319, [254, 254, 254]),
    RgbSample::new(735, 308, [255, 255, 255]),
    RgbSample::new(757, 320, [255, 255, 255]),
    RgbSample::new(782, 309, [253, 253, 253]),
    RgbSample::new(804, 317, [254, 254, 254]),
    RgbSample::new(830, 308, [255, 255, 255]),
    RgbSample::new(853, 320, [255, 255, 255]),
    RgbSample::new(875, 309, [255, 255, 255]),
];


/// Original stable artwork samples from K29; numeric level/score text is excluded.
const PLAY_TEXT: [RgbSample; 10] = [
    RgbSample::new(679, 867, [4, 3, 1]),
    RgbSample::new(686, 859, [0, 0, 0]),
    RgbSample::new(698, 867, [6, 4, 2]),
    RgbSample::new(699, 857, [0, 0, 0]),
    RgbSample::new(707, 867, [0, 0, 0]),
    RgbSample::new(715, 855, [0, 0, 0]),
    RgbSample::new(718, 867, [0, 0, 0]),
    RgbSample::new(726, 858, [0, 0, 0]),
    RgbSample::new(732, 866, [0, 0, 0]),
    RgbSample::new(736, 855, [0, 0, 0]),
];


/// Original stable artwork samples from K30; numeric level/score text is excluded.
const SOLVER_LABEL: [RgbSample; 10] = [
    RgbSample::new(568, 1019, [255, 255, 255]),
    RgbSample::new(575, 1014, [255, 255, 255]),
    RgbSample::new(583, 1019, [255, 255, 255]),
    RgbSample::new(590, 1011, [255, 255, 255]),
    RgbSample::new(597, 1019, [255, 255, 255]),
    RgbSample::new(603, 1012, [255, 255, 255]),
    RgbSample::new(607, 1019, [246, 247, 247]),
    RgbSample::new(615, 1012, [249, 250, 250]),
    RgbSample::new(629, 1019, [255, 255, 255]),
    RgbSample::new(630, 1011, [255, 255, 255]),
];


/// Measured static scene/control samples from K26, outside changing labels.
const COMPLETED_BOARD_CONTEXT: [RgbSample; 8] = [
    RgbSample::new(930, 117, [41, 54, 104]),
    RgbSample::new(1098, 117, [42, 54, 101]),
    RgbSample::new(1266, 117, [44, 52, 95]),
    RgbSample::new(1434, 117, [46, 50, 89]),
    RgbSample::new(410, 440, [17, 48, 105]),
    RgbSample::new(1500, 440, [17, 49, 106]),
    RgbSample::new(450, 600, [16, 56, 127]),
    RgbSample::new(1510, 700, [13, 52, 117]),
];


/// Four dimmed occupied-foundation margins from K27, unchanged by K49 fireworks.
const LEVEL_UP_FOUNDATION_CONTEXT: [RgbSample; 4] = [
    RgbSample::new(930, 117, [8, 11, 22]),
    RgbSample::new(1098, 117, [8, 11, 21]),
    RgbSample::new(1266, 117, [9, 10, 21]),
    RgbSample::new(1434, 117, [9, 10, 19]),
];


/// Empty side-tableau samples from K27; K49's warm particle lighting changes
/// red and green by up to 53 and 17 while preserving each recorded blue channel.
const LEVEL_UP_EMPTY_TABLEAU_CONTEXT: [RgbSample; 4] = [
    RgbSample::new(410, 440, [3, 10, 22]),
    RgbSample::new(1500, 440, [3, 10, 23]),
    RgbSample::new(450, 600, [3, 11, 26]),
    RgbSample::new(1510, 700, [3, 10, 25]),
];


/// Measured static scene/control samples from K26, outside changing labels.
const CONGRATULATIONS_FRAME: [RgbSample; 6] = [
    RgbSample::new(510, 142, [59, 115, 191]),
    RgbSample::new(1410, 142, [56, 107, 188]),
    RgbSample::new(510, 920, [59, 115, 190]),
    RgbSample::new(1410, 920, [59, 114, 190]),
    RgbSample::new(650, 190, [25, 150, 192]),
    RgbSample::new(1280, 270, [32, 175, 229]),
];


/// Measured static scene/control samples from K26, outside changing labels.
const SCORE_SKIP_BODY: [RgbSample; 4] = [
    RgbSample::new(650, 845, [24, 80, 154]),
    RgbSample::new(650, 880, [20, 73, 141]),
    RgbSample::new(1000, 845, [24, 81, 155]),
    RgbSample::new(1270, 880, [22, 77, 146]),
];


/// Measured static scene/control samples from K27, outside changing labels.
const LEVEL_UP_BUTTON_BODY: [RgbSample; 4] = [
    RgbSample::new(858, 784, [254, 247, 220]),
    RgbSample::new(1060, 784, [253, 249, 221]),
    RgbSample::new(856, 838, [190, 135, 40]),
    RgbSample::new(1060, 837, [183, 131, 42]),
];


/// Measured static scene/control samples from K28, outside changing labels.
/// The Home upper-body point at (1020, 827) is identical in K28/K53 and outside
/// K53's pointer at (959..=992, 794..=842). The old (990, 827) sample landed on
/// that cursor's shadow after Level Up OK; all six body samples remain required.
const NEW_GAME_BUTTON_BODY: [RgbSample; 6] = [
    RgbSample::new(645, 827, [252, 249, 223]),
    RgbSample::new(922, 827, [252, 249, 223]),
    RgbSample::new(645, 878, [194, 137, 42]),
    RgbSample::new(922, 878, [195, 138, 47]),
    RgbSample::new(1020, 827, [27, 86, 171]),
    RgbSample::new(1267, 878, [36, 114, 188]),
];


/// Measured static scene/control samples from K29, outside changing labels.
const PLAY_DIALOG_FRAME: [RgbSample; 6] = [
    RgbSample::new(575, 145, [58, 112, 187]),
    RgbSample::new(1078, 145, [15, 57, 113]),
    RgbSample::new(575, 921, [59, 114, 197]),
    RgbSample::new(1078, 921, [46, 100, 170]),
    RgbSample::new(640, 250, [2, 32, 56]),
    RgbSample::new(1010, 790, [3, 41, 70]),
];


/// Measured static scene/control samples from K29, outside changing labels.
const PLAY_BUTTON_BODY: [RgbSample; 6] = [
    RgbSample::new(614, 835, [252, 249, 223]),
    RgbSample::new(802, 835, [253, 248, 223]),
    RgbSample::new(615, 883, [182, 132, 37]),
    RgbSample::new(802, 883, [183, 133, 39]),
    RgbSample::new(850, 835, [26, 85, 168]),
    RgbSample::new(1040, 883, [37, 113, 188]),
];


/// Measured static scene/control samples from K30, outside changing labels.
const SOLVER_KEY: [RgbSample; 4] = [
    RgbSample::new(600, 961, [121, 117, 77]),
    RgbSample::new(612, 965, [186, 166, 105]),
    RgbSample::new(593, 983, [234, 208, 147]),
    RgbSample::new(600, 976, [227, 205, 131]),
];


/// Original mixed ink/background samples around the Draw 1 numeral in K29.
const PLAY_DRAW_ONE_NUMERAL: [RgbSample; 30] = [
    RgbSample::new(868, 303, [1, 20, 37]),
    RgbSample::new(871, 303, [182, 188, 193]),
    RgbSample::new(874, 303, [255, 255, 255]),
    RgbSample::new(877, 303, [17, 35, 51]),
    RgbSample::new(880, 303, [2, 21, 38]),
    RgbSample::new(868, 307, [1, 20, 37]),
    RgbSample::new(871, 307, [1, 20, 37]),
    RgbSample::new(874, 307, [255, 255, 255]),
    RgbSample::new(877, 307, [17, 35, 51]),
    RgbSample::new(880, 307, [1, 20, 37]),
    RgbSample::new(868, 311, [1, 20, 37]),
    RgbSample::new(871, 311, [1, 20, 37]),
    RgbSample::new(874, 311, [255, 255, 255]),
    RgbSample::new(877, 311, [17, 35, 51]),
    RgbSample::new(880, 311, [1, 20, 37]),
    RgbSample::new(868, 315, [1, 20, 37]),
    RgbSample::new(871, 315, [1, 20, 37]),
    RgbSample::new(874, 315, [255, 255, 255]),
    RgbSample::new(877, 315, [17, 35, 51]),
    RgbSample::new(880, 315, [1, 20, 37]),
    RgbSample::new(868, 319, [1, 20, 37]),
    RgbSample::new(871, 319, [1, 20, 37]),
    RgbSample::new(874, 319, [255, 255, 255]),
    RgbSample::new(877, 319, [17, 35, 51]),
    RgbSample::new(880, 319, [1, 20, 37]),
    RgbSample::new(868, 322, [1, 20, 37]),
    RgbSample::new(871, 322, [1, 20, 37]),
    RgbSample::new(874, 322, [108, 119, 128]),
    RgbSample::new(877, 322, [7, 26, 42]),
    RgbSample::new(880, 322, [1, 20, 37]),
];


/// Original non-letter samples distinguish the title from a flat white patch.
const CONGRATULATIONS_TITLE_BACKGROUND: [RgbSample; 8] = [
    RgbSample::new(736, 238, [29, 164, 212]),
    RgbSample::new(800, 213, [27, 156, 201]),
    RgbSample::new(863, 238, [29, 164, 212]),
    RgbSample::new(926, 213, [27, 156, 199]),
    RgbSample::new(989, 238, [29, 164, 212]),
    RgbSample::new(1046, 213, [27, 157, 198]),
    RgbSample::new(1120, 240, [67, 181, 219]),
    RgbSample::new(1178, 213, [27, 157, 198]),
];


/// Original non-letter samples distinguish the title from a flat white patch.
const LEVEL_UP_TITLE_BACKGROUND: [RgbSample; 7] = [
    RgbSample::new(824, 219, [24, 172, 221]),
    RgbSample::new(863, 196, [31, 179, 228]),
    RgbSample::new(875, 219, [32, 180, 229]),
    RgbSample::new(875, 203, [32, 180, 229]),
    RgbSample::new(1058, 219, [32, 180, 229]),
    RgbSample::new(1058, 196, [32, 180, 229]),
    RgbSample::new(1099, 196, [25, 173, 222]),
];


/// Original non-letter samples distinguish the title from a flat white patch.
/// The stable gap at (888, 632) is identical in K27/K49; its neighbouring shadow
/// at (884, 632) varied with background lighting and is not immutable artwork.
const LEVEL_UP_KLONDIKE_BACKGROUND: [RgbSample; 8] = [
    RgbSample::new(771, 633, [23, 58, 93]),
    RgbSample::new(800, 619, [22, 58, 93]),
    RgbSample::new(827, 631, [33, 71, 105]),
    RgbSample::new(855, 617, [26, 63, 95]),
    RgbSample::new(888, 632, [45, 81, 112]),
    RgbSample::new(906, 619, [37, 75, 105]),
    RgbSample::new(931, 632, [52, 89, 121]),
    RgbSample::new(961, 619, [21, 62, 94]),
];


/// Original non-letter samples distinguish the title from a flat white patch.
const PLAY_KLONDIKE_TITLE_BACKGROUND: [RgbSample; 8] = [
    RgbSample::new(683, 195, [32, 126, 163]),
    RgbSample::new(704, 182, [57, 142, 173]),
    RgbSample::new(719, 194, [32, 126, 163]),
    RgbSample::new(741, 180, [32, 126, 163]),
    RgbSample::new(761, 195, [32, 126, 163]),
    RgbSample::new(779, 180, [32, 126, 163]),
    RgbSample::new(801, 193, [32, 126, 163]),
    RgbSample::new(819, 181, [32, 126, 163]),
];


/// Original non-letter samples distinguish the title from a flat white patch.
const PLAY_DRAW_ONE_BACKGROUND: [RgbSample; 8] = [
    RgbSample::new(664, 320, [1, 20, 37]),
    RgbSample::new(696, 306, [1, 20, 37]),
    RgbSample::new(724, 320, [1, 20, 37]),
    RgbSample::new(754, 307, [1, 20, 37]),
    RgbSample::new(785, 320, [6, 25, 41]),
    RgbSample::new(816, 306, [1, 20, 37]),
    RgbSample::new(846, 320, [4, 23, 40]),
    RgbSample::new(877, 308, [17, 35, 51]),
];


/// Count immutable samples within tolerance at one coherent horizontal offset.
/// A missing pixel never contributes evidence. The caller validates native frame
/// storage; checked addition retains a fail-closed bound for translated samples.
fn count_samples(
    frame: &CapturedFrame,
    samples: &[RgbSample],
    tolerance: u8,
    horizontal_offset: u32,
) -> usize {
    samples.iter().filter(|sample| {
        sample.x.checked_add(horizontal_offset)
            .and_then(|x| pixel_rgb(frame, x, sample.y))
            .is_some_and(|actual| {
                actual.into_iter().zip(sample.rgb)
                    .all(|(channel, expected)| channel.abs_diff(expected) <= tolerance)
            })
    }).count()
}


/// Compare all channels of every immutable sample; a missing pixel fails closed.
fn matches_samples(frame: &CapturedFrame, samples: &[RgbSample], tolerance: u8) -> bool {
    count_samples(frame, samples, tolerance, 0) == samples.len()
}


/// Require both printed game text and its mixed background at one measured
/// position. This excludes moving level digits and refuses flat or independently
/// shifted partial signatures while retaining the original scene/control guards.
fn level_up_label_offset(frame: &CapturedFrame) -> Option<u32> {
    LEVEL_UP_LABEL_OFFSETS.into_iter().find(|offset| {
        count_samples(frame, &LEVEL_UP_KLONDIKE, ARTWORK_CHANNEL_TOLERANCE, *offset)
            == LEVEL_UP_KLONDIKE.len()
            && count_samples(frame, &LEVEL_UP_KLONDIKE_BACKGROUND,
                ARTWORK_CHANNEL_TOLERANCE, *offset) == LEVEL_UP_KLONDIKE_BACKGROUND.len()
    })
}


/// Require the full measured stable title/button signature, not a gold fraction.
fn matches_artwork(frame: &CapturedFrame, samples: &[RgbSample]) -> bool {
    matches_samples(frame, samples, ARTWORK_CHANNEL_TOLERANCE)
}


/// Original completed-board modal background: four occupied foundation margins
/// and empty side tableau. These paper samples exclude card ranks and suits.
fn has_completed_board_context(frame: &CapturedFrame, samples: &[RgbSample]) -> bool {
    matches_samples(frame, samples, BOARD_CONTEXT_CHANNEL_TOLERANCE)
}


/// Preserve full-RGB occupied-foundation proof and the measured dark-blue empty
/// tableau context. K49's warm fireworks alter the latter's red/green channels.
/// The red/green allowance is positive-only and capped by this supplied frame;
/// white paper, erased black context and a shifted blue backdrop still fail.
/// This exception applies only with the separate Level Up title, game label and
/// complete OK control signatures; it cannot establish a generic modal or win.
fn has_completed_level_up_context(frame: &CapturedFrame) -> bool {
    has_completed_board_context(frame, &LEVEL_UP_FOUNDATION_CONTEXT)
        && LEVEL_UP_EMPTY_TABLEAU_CONTEXT.iter()
            .all(|sample| matches_level_up_empty_sample(frame, sample))
}


/// One independently dimmed empty-tableau point with the measured finite warm
/// particle allowance; neither arbitrary black nor bright paper supplies proof.
fn matches_level_up_empty_sample(frame: &CapturedFrame, sample: &RgbSample) -> bool {
    pixel_rgb(frame, sample.x, sample.y).is_some_and(|actual| {
        actual[2].abs_diff(sample.rgb[2]) <= BOARD_CONTEXT_CHANNEL_TOLERANCE
            && actual[0] >= sample.rgb[0].saturating_sub(BOARD_CONTEXT_CHANNEL_TOLERANCE)
            && actual[0] <= sample.rgb[0].saturating_add(LEVEL_UP_FIREWORK_RED_INCREASE)
            && actual[1] >= sample.rgb[1].saturating_sub(BOARD_CONTEXT_CHANNEL_TOLERANCE)
            && actual[1] <= sample.rgb[1].saturating_add(LEVEL_UP_FIREWORK_GREEN_INCREASE)
    })
}


/// Positive Congratulations ribbon and completed-board scene shared by K26/K28.
fn has_completed_congratulations(frame: &CapturedFrame) -> bool {
    matches_artwork(frame, &CONGRATULATIONS_TITLE)
        && matches_artwork(frame, &CONGRATULATIONS_TITLE_BACKGROUND)
        && matches_artwork(frame, &CONGRATULATIONS_FRAME)
        && has_completed_board_context(frame, &COMPLETED_BOARD_CONTEXT)
}


/// Require the observed fresh-deal geometry without reading any face rank.
/// Empty foundations/waste, blue stock, seven staggered faces and six back strips
/// prevent an ordinary unresolved board from authorising Solver activation.
fn has_fresh_deal(frame: &CapturedFrame) -> Result<bool, HaloDetectionError> {


    if !super::is_gameplay_scene(frame)? || super::has_solver_banner(frame)
        || super::has_solve_control(frame)
    {
        return Ok(false);
    }


    if !super::fraction_at_least(
        frame, PixelRect::new(406, 130, 100, 139), super::is_back, 500,
    ) || !super::fraction_at_least(
        frame, PixelRect::new(576, 130, 152, 139), super::is_felt, 950,
    ) {
        return Ok(false);
    }


    for column in 0..4 {


        if !super::fraction_at_least(
            frame, PixelRect::new(910 + 168 * column, 130, 100, 139), super::is_felt, 950,
        ) {
            return Ok(false);
        }
    }


    for column in 0..7 {
        let x = 390 + 168 * column;
        let face_top = 342 + 35 * column / 2;


        if !super::fraction_at_least(
            frame, PixelRect::new(x + 18, face_top + 18, 96, 110), super::is_white, 300,
        ) || !super::fraction_at_least(
            frame, PixelRect::new(x + 18, 660, 96, 40), super::is_felt, 950,
        ) {
            return Ok(false);
        }


        if column > 0 && !super::fraction_at_least(
            frame, PixelRect::new(x + 18, 347, 96, 6), super::is_back, 900,
        ) {
            return Ok(false);
        }


        if super::find_solid_card_source(
            frame, PixelRect::new(x, 332, 132, super::TABLEAU_OUTLINE_BOTTOM - 332),
        )?.is_some() {
            return Ok(false);
        }
    }


    Ok(super::find_solid_outline(frame, PixelRect::new(390, 100, 132, 204), false)?.is_none())
}


/// Recognise only the five evidenced native stage layouts; unknown/shifted scenes return
/// `None`. Malformed frame storage or dimensions return the parent layout error.
/// Classification is read-only and each returned stage needs fresh worker validation.
pub(crate) fn classify_terminal(
    frame: &CapturedFrame,
) -> Result<Option<TerminalStage>, HaloDetectionError> {
    super::validate_frame(frame)?;


    if has_completed_congratulations(frame) {


        if matches_artwork(frame, &NEW_GAME_TEXT)
            && matches_artwork(frame, &NEW_GAME_HOME)
            && matches_artwork(frame, &NEW_GAME_BUTTON_BODY)
        {
            return Ok(Some(TerminalStage::NewGame));
        }


        if matches_artwork(frame, &SCORE_SKIP_CAPTION)
            && matches_artwork(frame, &SCORE_SKIP_BODY)
        {
            return Ok(Some(TerminalStage::ScoreCounting));
        }
    }


    if matches_artwork(frame, &LEVEL_UP_TITLE)
        && matches_artwork(frame, &LEVEL_UP_TITLE_BACKGROUND)
        && level_up_label_offset(frame).is_some()
        && matches_artwork(frame, &LEVEL_UP_OK)
        && matches_artwork(frame, &LEVEL_UP_OK_BACKGROUND)
        && matches_artwork(frame, &LEVEL_UP_BUTTON_BODY)
        && has_completed_level_up_context(frame)
    {
        return Ok(Some(TerminalStage::LevelUp));
    }


    if matches_artwork(frame, &PLAY_KLONDIKE_TITLE)
        && matches_artwork(frame, &PLAY_KLONDIKE_TITLE_BACKGROUND)
        && matches_artwork(frame, &PLAY_DRAW_ONE)
        && matches_artwork(frame, &PLAY_DRAW_ONE_BACKGROUND)
        && matches_artwork(frame, &PLAY_DRAW_ONE_NUMERAL)
        && matches_artwork(frame, &PLAY_TEXT)
        && matches_artwork(frame, &PLAY_DIALOG_FRAME)
        && matches_artwork(frame, &PLAY_BUTTON_BODY)
    {
        return Ok(Some(TerminalStage::Play));
    }


    if matches_artwork(frame, &SOLVER_LABEL) && matches_artwork(frame, &SOLVER_KEY)
        && has_fresh_deal(frame)?
    {
        return Ok(Some(TerminalStage::SolverReady));
    }

    Ok(None)
}


/// Matched and required samples for one independently required terminal guard.
#[derive(Clone, Copy, Debug)]
struct SignatureEvidence {
    /// Native sample count within the unchanged guard's colour tolerance.
    matched: usize,
    /// Total sample count, all of which the corresponding guard requires.
    required: usize,
}


impl SignatureEvidence {


    /// Record immutable samples without granting input or changing any tolerance.
    fn inspect(frame: &CapturedFrame, samples: &[RgbSample], tolerance: u8, offset: u32) -> Self {
        Self { matched: count_samples(frame, samples, tolerance, offset), required: samples.len() }
    }
}


impl fmt::Display for SignatureEvidence {


    /// Keep the actual match count next to its unchanged required count.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}", self.matched, self.required)
    }
}


/// Read-only named terminal guards for diagnosing frames absent from attachments.
/// Counts are observational evidence only; `classify_terminal` remains the sole
/// scene authority and the worker still grants exactly one fresh control input.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TerminalEvidence {
    /// Congratulations letter samples and non-letter ribbon contrast together.
    congratulations_ribbon: [SignatureEvidence; 2],
    /// Stable Congratulations frame artwork outside score and level text.
    congratulations_frame: SignatureEvidence,
    /// Occupied foundations and empty side tableau behind Congratulations.
    congratulations_context: SignatureEvidence,
    /// Counting-stage skip caption and body; both are independently required.
    score_control: [SignatureEvidence; 2],
    /// New Game letters, Home letters and paired control body evidence.
    new_game_control: [SignatureEvidence; 3],
    /// Level Up title letters and non-letter ribbon contrast together.
    level_up_title: [SignatureEvidence; 2],
    /// Label letters/background for coherent offsets zero and two respectively.
    level_up_label: [[SignatureEvidence; 2]; 2],
    /// Level Up OK glyph, internal gold contrast and external button body.
    level_up_control: [SignatureEvidence; 3],
    /// Independently dimmed occupied foundation context behind Level Up.
    level_up_foundations: SignatureEvidence,
    /// Empty side-tableau points under their finite warm-particle guard.
    level_up_tableau: SignatureEvidence,
}


impl fmt::Display for TerminalEvidence {


    /// Surface the refused scene guard without inferring missing worker pixels.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter,
            "Congratulations title={}/{}, frame={}, completed context={}, skip={}/{}, New Game={}/{}/{}; Level Up title={}/{}, label +0={}/{}, label +2={}/{}, OK={}/{}/{}, foundations={}, empty tableau={}",
            self.congratulations_ribbon[0], self.congratulations_ribbon[1],
            self.congratulations_frame, self.congratulations_context,
            self.score_control[0], self.score_control[1],
            self.new_game_control[0], self.new_game_control[1], self.new_game_control[2],
            self.level_up_title[0], self.level_up_title[1],
            self.level_up_label[0][0], self.level_up_label[0][1],
            self.level_up_label[1][0], self.level_up_label[1][1],
            self.level_up_control[0], self.level_up_control[1], self.level_up_control[2],
            self.level_up_foundations, self.level_up_tableau)
    }
}


/// Inspect every named Congratulations/Level Up guard on one valid native frame.
/// This deliberately evaluates refused alternatives too, so delayed observations
/// explain a stop after a previously recognised control without another click.
/// Malformed storage or unsupported dimensions return the existing layout error.
pub(crate) fn inspect_terminal_evidence(
    frame: &CapturedFrame,
) -> Result<TerminalEvidence, HaloDetectionError> {
    super::validate_frame(frame)?;
    let artwork = |samples: &[RgbSample]| {
        SignatureEvidence::inspect(frame, samples, ARTWORK_CHANNEL_TOLERANCE, 0)
    };
    let label = LEVEL_UP_LABEL_OFFSETS.map(|offset| {
        [SignatureEvidence::inspect(frame, &LEVEL_UP_KLONDIKE, ARTWORK_CHANNEL_TOLERANCE, offset),
            SignatureEvidence::inspect(frame, &LEVEL_UP_KLONDIKE_BACKGROUND,
                ARTWORK_CHANNEL_TOLERANCE, offset)]
    });
    Ok(TerminalEvidence {
        congratulations_ribbon: [artwork(&CONGRATULATIONS_TITLE),
            artwork(&CONGRATULATIONS_TITLE_BACKGROUND)],
        congratulations_frame: artwork(&CONGRATULATIONS_FRAME),
        congratulations_context: SignatureEvidence::inspect(frame, &COMPLETED_BOARD_CONTEXT,
            BOARD_CONTEXT_CHANNEL_TOLERANCE, 0),
        score_control: [artwork(&SCORE_SKIP_CAPTION), artwork(&SCORE_SKIP_BODY)],
        new_game_control: [artwork(&NEW_GAME_TEXT), artwork(&NEW_GAME_HOME),
            artwork(&NEW_GAME_BUTTON_BODY)],
        level_up_title: [artwork(&LEVEL_UP_TITLE), artwork(&LEVEL_UP_TITLE_BACKGROUND)],
        level_up_label: label,
        level_up_control: [artwork(&LEVEL_UP_OK), artwork(&LEVEL_UP_OK_BACKGROUND),
            artwork(&LEVEL_UP_BUTTON_BODY)],
        level_up_foundations: SignatureEvidence::inspect(frame, &LEVEL_UP_FOUNDATION_CONTEXT,
            BOARD_CONTEXT_CHANNEL_TOLERANCE, 0),
        level_up_tableau: SignatureEvidence {
            matched: LEVEL_UP_EMPTY_TABLEAU_CONTEXT.iter()
                .filter(|sample| matches_level_up_empty_sample(frame, sample)).count(),
            required: LEVEL_UP_EMPTY_TABLEAU_CONTEXT.len(),
        },
    })
}


#[cfg(test)]
mod tests {
    //! Original scene signatures, rejected controls, bounds and editable timing.
    use super::*;
    use crate::capture::decode_png;


    /// Decode one provenance-recorded terminal fixture; no image is fabricated here.
    fn fixture(number: usize) -> CapturedFrame {


        let png: &[u8] = match number {
            26 => include_bytes!("../tests/fixtures/klondike/K26.png"),
            27 => include_bytes!("../tests/fixtures/klondike/K27.png"),
            28 => include_bytes!("../tests/fixtures/klondike/K28.png"),
            29 => include_bytes!("../tests/fixtures/klondike/K29.png"),
            30 => include_bytes!("../tests/fixtures/klondike/K30.png"),
            43 => include_bytes!("../tests/fixtures/klondike/K43.png"),
            49 => include_bytes!("../tests/fixtures/klondike/K49.png"),
            52 => include_bytes!("../tests/fixtures/klondike/K52.png"),
            53 => include_bytes!("../tests/fixtures/klondike/K53.png"),
            _ => panic!("unknown terminal fixture"),
        };
        decode_png(png).expect("decode recorded terminal PNG")
    }


    /// Replace a bounded synthetic test patch while leaving all other evidence intact.
    fn paint(frame: &mut CapturedFrame, bounds: PixelRect, rgb: [u8; 3]) {
        assert!(bounds.x + bounds.width <= frame.width);
        assert!(bounds.y + bounds.height <= frame.height);


        for y in bounds.y..bounds.y + bounds.height {


            for x in bounds.x..bounds.x + bounds.width {
                let index = y as usize * frame.stride + x as usize * 4;
                frame.pixels[index..index + 3].copy_from_slice(&rgb);
            }
        }
    }


    /// The five original supplied stages remain distinct, and only the first
    /// three independently establish the evidenced one-board game win.
    #[test]
    fn original_terminal_sequence_has_distinct_stages() {
        let stages = [TerminalStage::ScoreCounting, TerminalStage::LevelUp,
            TerminalStage::NewGame, TerminalStage::Play, TerminalStage::SolverReady];


        for (number, stage) in (26..=30).zip(stages) {
            assert_eq!(classify_terminal(&fixture(number)).unwrap(), Some(stage), "K{number}");
            assert_eq!(stage.demonstrates_game_win(), number <= 28);
        }
    }


    /// K43 was captured after the failed bounded Solve wait. Its settled level-46
    /// Congratulations scene is recognised by the original unmodified signature.
    #[test]
    fn later_level_congratulations_retains_the_existing_signature() {
        let frame = fixture(43);
        assert_eq!(classify_terminal(&frame).unwrap(), Some(TerminalStage::ScoreCounting));
        let mut changed_rewards = frame.clone();
        paint(&mut changed_rewards, PixelRect::new(640, 340, 640, 430), [0, 0, 0]);
        assert_eq!(classify_terminal(&changed_rewards).unwrap(), Some(TerminalStage::ScoreCounting));
    }


    /// K53 follows an acknowledged Level Up OK click and retains its pointer.
    /// Only the former Home body sample is shadowed; the replacement remains
    /// identical in both originals while all modal, glyph and body guards pass.
    #[test]
    fn new_game_with_the_prior_ok_pointer_retains_its_control_signature() {
        let discarded_body_point = [RgbSample::new(990, 827, [27, 85, 166])];


        for number in [28, 53] {
            let frame = fixture(number);
            assert!(has_completed_congratulations(&frame));
            assert!(matches_artwork(&frame, &NEW_GAME_TEXT));
            assert!(matches_artwork(&frame, &NEW_GAME_HOME));
            assert!(matches_artwork(&frame, &NEW_GAME_BUTTON_BODY));
            assert_eq!(classify_terminal(&frame).unwrap(), Some(TerminalStage::NewGame));
            let evidence = inspect_terminal_evidence(&frame).unwrap();
            assert_eq!(evidence.new_game_control[2].matched, 6);
            assert_eq!(matches_artwork(&frame, &discarded_body_point), number == 28);
        }
        assert_eq!(TerminalStage::NewGame.click_point(), PixelPoint::new(792, 840));
    }


    /// A measured cursor shadow at the discarded point never erases New Game.
    /// The required replacement point still fails closed if independently erased;
    /// this relocates one probe rather than accepting an arbitrary missing sample.
    #[test]
    fn cursor_shadow_does_not_relax_the_required_new_game_body() {
        let mut shadowed = fixture(28);
        paint(&mut shadowed, PixelRect::new(990, 827, 1, 1), [10, 32, 62]);
        assert_eq!(classify_terminal(&shadowed).unwrap(), Some(TerminalStage::NewGame));
        paint(&mut shadowed, PixelRect::new(1020, 827, 1, 1), [10, 32, 62]);
        assert_eq!(classify_terminal(&shadowed).unwrap(), None);
        assert_eq!(inspect_terminal_evidence(&shadowed).unwrap().new_game_control[2].matched, 5);
    }


    /// Occluding printed New Game with the native K53 cursor still refuses input.
    /// This controlled derivative transfers only K28/K53 differences in the
    /// recorded pointer box; it does not erase a required text guard by policy.
    #[test]
    fn new_game_cursor_occlusion_of_the_label_is_not_ignored() {
        let original = fixture(28);
        let pointer = fixture(53);
        let mut occluded = original.clone();


        for y in 0..49usize {


            for x in 0..34usize {
                let source = (794 + y) * original.stride + (959 + x) * 4;


                if original.pixels[source..source + 3] != pointer.pixels[source..source + 3] {
                    let destination = (838 + y) * occluded.stride + (760 + x) * 4;
                    occluded.pixels[destination..destination + 3]
                        .copy_from_slice(&pointer.pixels[source..source + 3]);
                }
            }
        }
        assert!(has_completed_congratulations(&occluded));
        assert!(matches_artwork(&occluded, &NEW_GAME_BUTTON_BODY));
        assert!(!matches_artwork(&occluded, &NEW_GAME_TEXT));
        assert_eq!(classify_terminal(&occluded).unwrap(), None);
    }


    /// K53 still needs both visible controls, its modal frame and the completed
    /// Klondike background. Empty or flat-colour controls and unrelated modal
    /// context cannot become New Game merely because its cursor is harmless.
    #[test]
    fn pointer_safe_new_game_requires_its_independent_scene_and_controls() {
        let bounds = [PixelRect::new(631, 815, 310, 75),
            PixelRect::new(974, 815, 310, 75), PixelRect::new(500, 136, 920, 12),
            PixelRect::new(894, 112, 132, 26), PixelRect::new(700, 200, 520, 55)];


        for bounds in bounds {


            for rgb in [[0, 0, 0], [240, 190, 90], [255, 255, 255]] {
                let mut frame = fixture(53);
                paint(&mut frame, bounds, rgb);
                assert_eq!(classify_terminal(&frame).unwrap(), None);
            }
        }
        let mut wrong_size = fixture(53);
        wrong_size.height = 1_079;
        assert!(matches!(classify_terminal(&wrong_size), Err(HaloDetectionError::BoundsOutsideFrame)));
        let mut invalid_storage = fixture(53);
        invalid_storage.pixels.clear();
        assert!(matches!(classify_terminal(&invalid_storage), Err(HaloDetectionError::InvalidFrameLayout)));
    }


    /// The later frame still requires its title, skip caption and completed-board
    /// context. A changed level does not weaken any original evidence requirement.
    #[test]
    fn later_congratulations_requires_title_control_and_completed_context() {
        let bounds = [PixelRect::new(700, 200, 520, 55),
            PixelRect::new(795, 839, 332, 31), PixelRect::new(894, 112, 132, 26)];


        for bounds in bounds {
            let mut frame = fixture(43);
            paint(&mut frame, bounds, [0, 0, 0]);
            assert_eq!(classify_terminal(&frame).unwrap(), None);
        }
    }


    /// K49 retains the original visible Level Up authority despite measured warm
    /// fireworks and a variable neighbouring text shadow. Reward numbers remain
    /// excluded, so their update cannot erase or create the recognised OK action.
    #[test]
    fn later_level_up_with_fireworks_retains_its_control_signature() {
        let frame = fixture(49);
        assert_eq!(classify_terminal(&frame).unwrap(), Some(TerminalStage::LevelUp));
        assert_eq!(TerminalStage::LevelUp.click_point(), PixelPoint::new(960, 795));
        let mut changed_level = frame.clone();
        paint(&mut changed_level, PixelRect::new(1_110, 610, 60, 38), [0, 0, 0]);
        assert_eq!(classify_terminal(&changed_level).unwrap(), Some(TerminalStage::LevelUp));
    }


    /// K52 changes the centred label position by two pixels while every measured
    /// title, OK glyph, button-body and occupied-board context guard still passes.
    /// The prior whole-scene refusal must not change the already valid click.
    #[test]
    fn later_level_number_uses_the_measured_coherent_label_position() {
        let frame = fixture(52);
        assert!(!matches_artwork(&frame, &LEVEL_UP_KLONDIKE));
        assert!(!matches_artwork(&frame, &LEVEL_UP_KLONDIKE_BACKGROUND));
        assert_eq!(level_up_label_offset(&frame), Some(2));
        assert!(matches_artwork(&frame, &LEVEL_UP_TITLE));
        assert!(matches_artwork(&frame, &LEVEL_UP_OK));
        assert!(matches_artwork(&frame, &LEVEL_UP_OK_BACKGROUND));
        assert!(matches_artwork(&frame, &LEVEL_UP_BUTTON_BODY));
        assert!(has_completed_level_up_context(&frame));
        assert_eq!(classify_terminal(&frame).unwrap(), Some(TerminalStage::LevelUp));
        assert_eq!(TerminalStage::LevelUp.click_point(), PixelPoint::new(960, 795));
        let mut changed_level = frame.clone();
        paint(&mut changed_level, PixelRect::new(1_110, 610, 60, 38), [0, 0, 0]);
        assert_eq!(classify_terminal(&changed_level).unwrap(), Some(TerminalStage::LevelUp));
    }


    /// Recognition cannot combine nominal-position letters with background at
    /// the translated position; both independent patterns must share one offset.
    #[test]
    fn level_up_label_patterns_must_have_one_coherent_position() {
        let mut frame = fixture(52);
        paint(&mut frame, PixelRect::new(760, 610, 220, 35), [0, 0, 0]);


        for sample in LEVEL_UP_KLONDIKE {
            paint(&mut frame, PixelRect::new(sample.x, sample.y, 1, 1), sample.rgb);
        }


        for sample in LEVEL_UP_KLONDIKE_BACKGROUND {
            paint(&mut frame, PixelRect::new(sample.x + 2, sample.y, 1, 1), sample.rgb);
        }
        assert!(matches_artwork(&frame, &LEVEL_UP_KLONDIKE));
        assert_eq!(count_samples(&frame, &LEVEL_UP_KLONDIKE_BACKGROUND,
            ARTWORK_CHANNEL_TOLERANCE, 2), LEVEL_UP_KLONDIKE_BACKGROUND.len());
        assert_eq!(level_up_label_offset(&frame), None);
        assert_eq!(classify_terminal(&frame).unwrap(), None);
    }


    /// Only the two supplied centre positions are permitted. A further synthetic
    /// label shift leaves all other real scene pixels intact but grants no OK.
    #[test]
    fn unmeasured_label_position_does_not_authorise_the_control() {
        let original = fixture(52);
        let mut shifted = original.clone();
        paint(&mut shifted, PixelRect::new(760, 610, 222, 35), [0, 0, 0]);


        for y in 610..645usize {
            let source = y * original.stride + 760 * 4;
            let destination = y * shifted.stride + 762 * 4;
            shifted.pixels[destination..destination + 220 * 4]
                .copy_from_slice(&original.pixels[source..source + 220 * 4]);
        }
        assert!(matches_artwork(&shifted, &LEVEL_UP_TITLE));
        assert!(matches_artwork(&shifted, &LEVEL_UP_OK));
        assert_eq!(level_up_label_offset(&shifted), None);
        assert_eq!(classify_terminal(&shifted).unwrap(), None);
    }


    /// The shifted label never bypasses a visible title/control, mixed game name,
    /// foundation or empty-tableau guard. These are controlled adverse derivatives.
    #[test]
    fn shifted_label_still_requires_every_independent_scene_guard() {
        let bounds = [PixelRect::new(804, 185, 315, 46),
            PixelRect::new(760, 610, 222, 35), PixelRect::new(930, 800, 50, 24),
            PixelRect::new(840, 775, 240, 75)];


        for bounds in bounds {


            for rgb in [[0, 0, 0], [255, 255, 255]] {
                let mut frame = fixture(52);
                paint(&mut frame, bounds, rgb);
                assert_eq!(classify_terminal(&frame).unwrap(), None);
            }
        }


        for sample in LEVEL_UP_FOUNDATION_CONTEXT {
            let mut frame = fixture(52);
            paint(&mut frame, PixelRect::new(sample.x, sample.y, 1, 1), [0, 0, 0]);
            assert_eq!(classify_terminal(&frame).unwrap(), None);
        }


        for sample in LEVEL_UP_EMPTY_TABLEAU_CONTEXT {
            let mut frame = fixture(52);
            paint(&mut frame, PixelRect::new(sample.x, sample.y, 1, 1), [255, 255, 255]);
            assert_eq!(classify_terminal(&frame).unwrap(), None);
        }
    }


    /// Named diagnostic counts distinguish the coherent label from independently
    /// required controls. Inspection cannot establish authority on malformed frames.
    #[test]
    fn terminal_evidence_reports_each_guard_and_rejects_bad_storage() {
        let frame = fixture(52);
        let evidence = inspect_terminal_evidence(&frame).unwrap();
        assert_eq!(evidence.level_up_title[0].matched, LEVEL_UP_TITLE.len());
        assert_eq!(evidence.level_up_label[0][0].matched, 6);
        assert_eq!(evidence.level_up_label[1][0].matched, LEVEL_UP_KLONDIKE.len());
        assert_eq!(evidence.level_up_label[1][1].matched, LEVEL_UP_KLONDIKE_BACKGROUND.len());
        assert_eq!(evidence.level_up_control[0].matched, LEVEL_UP_OK.len());
        assert_eq!(evidence.level_up_foundations.matched, LEVEL_UP_FOUNDATION_CONTEXT.len());
        assert_eq!(evidence.level_up_tableau.matched, LEVEL_UP_EMPTY_TABLEAU_CONTEXT.len());
        assert!(evidence.to_string().contains("label +2=10/10/8/8"));
        let mut invalid = frame;
        invalid.pixels.clear();
        assert!(matches!(inspect_terminal_evidence(&invalid),
            Err(HaloDetectionError::InvalidFrameLayout)));
    }


    /// Restoring only K49's animated side context to K27 values isolates the
    /// shadow rejection independently of fireworks. This controlled derivative
    /// does not claim to reproduce any unrecorded early worker observation.
    #[test]
    fn later_level_up_shadow_is_independent_of_fireworks_context() {
        let mut frame = fixture(49);


        for sample in LEVEL_UP_EMPTY_TABLEAU_CONTEXT {
            paint(&mut frame, PixelRect::new(sample.x, sample.y, 1, 1), sample.rgb);
        }
        let discarded_shadow = [RgbSample::new(884, 632, [1, 2, 3])];
        assert!(!matches_artwork(&frame, &discarded_shadow));
        assert!(has_completed_level_up_context(&frame));
        assert_eq!(classify_terminal(&frame).unwrap(), Some(TerminalStage::LevelUp));
    }


    /// The measured warm context allowance never bypasses the mixed title,
    /// Klondike label, OK glyph, gold control or any occupied-foundation sample.
    #[test]
    fn later_level_up_requires_its_independent_title_control_and_foundations() {
        let bounds = [PixelRect::new(804, 185, 315, 46),
            PixelRect::new(760, 610, 220, 35), PixelRect::new(930, 800, 50, 24),
            PixelRect::new(840, 775, 240, 75)];


        for bounds in bounds {
            let mut frame = fixture(49);
            paint(&mut frame, bounds, [0, 0, 0]);
            assert_eq!(classify_terminal(&frame).unwrap(), None);
        }


        for sample in LEVEL_UP_FOUNDATION_CONTEXT {
            let mut frame = fixture(49);
            paint(&mut frame, PixelRect::new(sample.x, sample.y, 1, 1), [0, 0, 0]);
            assert_eq!(classify_terminal(&frame).unwrap(), None);
        }


        for rgb in [[0, 0, 0], [255, 255, 255]] {
            let mut frame = fixture(49);
            paint(&mut frame, PixelRect::new(760, 610, 220, 35), rgb);
            assert_eq!(classify_terminal(&frame).unwrap(), None);
        }
    }


    /// Each empty-tableau point retains its dark blue signature and finite warm
    /// lighting limits. Black, white and isolated blue patches cannot grant OK.
    #[test]
    fn fireworks_context_rejects_outside_measured_colour_support() {


        for sample in LEVEL_UP_EMPTY_TABLEAU_CONTEXT {
            let unsupported = [[0, 0, 0], [255, 255, 255], [0, 0, sample.rgb[2]],
                [sample.rgb[0] + LEVEL_UP_FIREWORK_RED_INCREASE + 1, sample.rgb[1], sample.rgb[2]],
                [sample.rgb[0], sample.rgb[1] + LEVEL_UP_FIREWORK_GREEN_INCREASE + 1, sample.rgb[2]],
                [sample.rgb[0], sample.rgb[1], sample.rgb[2] + BOARD_CONTEXT_CHANNEL_TOLERANCE + 1]];


            for rgb in unsupported {
                let mut frame = fixture(49);
                paint(&mut frame, PixelRect::new(sample.x, sample.y, 1, 1), rgb);
                assert_eq!(classify_terminal(&frame).unwrap(), None, "unsupported context {rgb:?}");
            }
        }
    }


    /// Removing the visible control/caption never authorises an inferred click.
    #[test]
    fn every_stage_requires_its_own_visible_control() {
        let bounds = [PixelRect::new(795, 839, 332, 31), PixelRect::new(840, 775, 240, 75),
            PixelRect::new(631, 815, 310, 75), PixelRect::new(602, 825, 214, 70),
            PixelRect::new(560, 1_000, 95, 30)];


        for (number, bounds) in (26..=30).zip(bounds) {
            let mut frame = fixture(number);
            paint(&mut frame, bounds, [0, 0, 0]);
            assert_eq!(classify_terminal(&frame).unwrap(), None, "control missing K{number}");
        }
    }


    /// Missing title or wrong Draw selector invalidates its otherwise intact button.
    #[test]
    fn dialog_titles_and_draw_one_selector_are_required() {
        let bounds = [PixelRect::new(700, 200, 520, 55), PixelRect::new(760, 610, 220, 35),
            PixelRect::new(700, 200, 520, 55), PixelRect::new(648, 300, 245, 30)];


        for (number, bounds) in (26..=29).zip(bounds) {
            let mut frame = fixture(number);
            paint(&mut frame, bounds, [0, 0, 0]);
            assert_eq!(classify_terminal(&frame).unwrap(), None, "title missing K{number}");
        }
    }


    /// Flat white title patches cannot satisfy the original mixed artwork pattern.
    #[test]
    fn white_title_blocks_are_not_recognised_as_text() {
        let bounds = [PixelRect::new(700, 200, 520, 55), PixelRect::new(804, 185, 315, 46),
            PixelRect::new(700, 200, 520, 55), PixelRect::new(674, 175, 153, 27)];


        for (number, bounds) in (26..=29).zip(bounds) {
            let mut frame = fixture(number);
            paint(&mut frame, bounds, [255, 255, 255]);
            assert_eq!(classify_terminal(&frame).unwrap(), None, "white title K{number}");
        }
    }


    /// Changing the displayed level number does not change terminal authority.
    #[test]
    fn changing_numeric_level_text_is_outside_the_signature() {


        for number in 26..=28 {
            let mut frame = fixture(number);


            let numeric_bounds = if number == 27 {
                PixelRect::new(1_110, 610, 60, 38)
            } else {
                PixelRect::new(1_004, 584, 62, 37)
            };
            paint(&mut frame, numeric_bounds, [0, 0, 0]);
            assert_eq!(classify_terminal(&frame).unwrap(), classify_terminal(&fixture(number)).unwrap());
        }
    }


    /// Empty or erased foundation context cannot establish a completed game.
    #[test]
    fn win_dialogs_require_the_completed_klondike_board_context() {


        for number in 26..=28 {
            let mut frame = fixture(number);
            paint(&mut frame, PixelRect::new(894, 112, 132, 26), [0, 0, 0]);
            assert_eq!(classify_terminal(&frame).unwrap(), None, "foundation missing K{number}");
        }
    }


    /// Solver activation is never offered on a board with an active recommendation.
    #[test]
    fn solver_ready_rejects_an_active_banner() {
        let mut frame = fixture(30);
        paint(&mut frame, PixelRect::new(828, 36, 263, 1), [240, 190, 90]);
        paint(&mut frame, PixelRect::new(828, 85, 263, 1), [240, 190, 90]);
        assert_eq!(classify_terminal(&frame).unwrap(), None);
    }


    /// A missing initial back strip or non-empty foundation is not a fresh deal.
    #[test]
    fn solver_ready_requires_the_initial_deal_geometry() {
        let mut missing_back = fixture(30);
        paint(&mut missing_back, PixelRect::new(576, 347, 96, 6), [0, 0, 0]);
        assert_eq!(classify_terminal(&missing_back).unwrap(), None);
        let mut occupied_foundation = fixture(30);
        paint(&mut occupied_foundation, PixelRect::new(910, 130, 100, 139), [255, 255, 255]);
        assert_eq!(classify_terminal(&occupied_foundation).unwrap(), None);
    }


    /// Unknown uniform colours, storage and dimensions cannot grant terminal input.
    #[test]
    fn unknown_or_malformed_frames_are_rejected() {


        for rgb in [[0, 0, 0], [255, 255, 255], [240, 190, 90], [20, 120, 70]] {
            let mut frame = fixture(26);
            paint(&mut frame, PixelRect::new(0, 0, 1_920, 1_080), rgb);
            assert_eq!(classify_terminal(&frame).unwrap(), None);
        }
        let mut wrong_size = fixture(26);
        wrong_size.width = 1_919;
        assert!(matches!(classify_terminal(&wrong_size), Err(HaloDetectionError::BoundsOutsideFrame)));
        let mut invalid_storage = fixture(26);
        invalid_storage.pixels.clear();
        assert!(matches!(classify_terminal(&invalid_storage), Err(HaloDetectionError::InvalidFrameLayout)));
    }


    /// Shifted screenshots must not preserve a control's old native click authority.
    #[test]
    fn shifted_native_scenes_do_not_authorise_old_coordinates() {


        for number in 26..=30 {
            let original = fixture(number);
            let mut shifted = original.clone();
            paint(&mut shifted, PixelRect::new(0, 0, 1_920, 1_080), [0, 0, 0]);


            for y in 0..1_080usize {
                let source = y * original.stride;
                let destination = y * shifted.stride + 16 * 4;
                shifted.pixels[destination..destination + (1_920 - 16) * 4]
                    .copy_from_slice(&original.pixels[source..source + (1_920 - 16) * 4]);
            }
            assert_eq!(classify_terminal(&shifted).unwrap(), None, "shifted K{number}");
        }
    }


    /// Controls retain measured points and existing delays, including editable deal wait.
    #[test]
    fn measured_actions_reuse_existing_hold_and_settle_contracts() {
        assert_eq!(TerminalStage::ScoreCounting.click_point(), PixelPoint::new(960, 540));
        assert_eq!(TerminalStage::LevelUp.click_point(), PixelPoint::new(960, 795));
        assert_eq!(TerminalStage::NewGame.click_point(), PixelPoint::new(792, 840));
        assert_eq!(TerminalStage::Play.click_point(), PixelPoint::new(705, 860));
        assert_eq!(TerminalStage::SolverReady.click_point(), PixelPoint::new(602, 977));
        assert_eq!(TerminalStage::ScoreCounting.settle_delay(Duration::ZERO), LEVEL_UP_APPEAR_DELAY);
        assert_eq!(TerminalStage::NewGame.settle_delay(Duration::ZERO), POST_GAME_STAGE_DELAY);
        assert_eq!(TerminalStage::Play.settle_delay(Duration::from_millis(1_234)), Duration::from_millis(1_234));
        assert_eq!(TerminalStage::Play.mouse_hold(), POST_GAME_MOUSE_HOLD);
        assert_eq!(TerminalStage::SolverReady.mouse_hold(), SOLVER_MOUSE_HOLD);
    }
}
