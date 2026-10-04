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
//! K67 supplies a separate Level Up medal layout with larger panel artwork and
//! an OK caption four rows lower. Its complete title, mixed Klondike label,
//! frame and control signature reuses the existing completed-board context and
//! measured OK click. Changing rank, level and reward text remain excluded.
//! K86 independently measures a Master medal layout: its title is two rows
//! higher, printed game label seven rows lower and OK/frame two rows lower than
//! K67. These complete component signatures retain the same colour tolerances,
//! completed-board context and existing OK click; no runtime alignment search
//! or guessed terminal input is authorised.
//! K77's initial King has less than 30% white in its central artwork. Fresh-deal
//! recognition instead requires four positive paper margins at every measured
//! staggered face, retaining empty waste/foundations, stock and back-strip proof.
//! No rank-dependent artwork fraction or missing HALO grants Solver input.
//! Independent completion retains the recorded positive terminal/board proof.
//! Once the worker has confirmed one game win, the deterministic restart path
//! checks only the expected OK, New Game or Play button area. Positive gold and
//! dark printed caption contrast in that local area establish readiness; title,
//! rank, medal, modal frame, fireworks and surrounding artwork do not gate those
//! clicks. Local button readiness never establishes the initial game win.
//! The worker owns ordering, fresh capture, cancellation, one-shot delivery and
//! bounded input-free transition observations. Fresh-deal Solver readiness still
//! requires its positive mode-owned scene and deal geometry.

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


/// Typed stage of Charlie's observed Klondike restart sequence.
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


    /// Whether the stage belongs to the completed-game portion of the sequence.
    /// Only `classify_terminal` supplies independent completed-board proof; this
    /// enum discriminator and local readiness alone cannot establish a win.
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


/// Translate a complete constant sample group by one measured vertical offset.
/// Constant evaluation rejects row underflow or overflow. Independent samples
/// are never searched or aligned at runtime; colours and horizontal positions
/// remain those of the original complete group.
const fn shift_sample_rows<const LENGTH: usize>(
    samples: [RgbSample; LENGTH],
    upward_rows: u32,
    downward_rows: u32,
) -> [RgbSample; LENGTH] {
    let mut shifted = samples;
    let mut index = 0;


    while index < LENGTH {
        shifted[index].y = shifted[index].y - upward_rows + downward_rows;
        index += 1;
    }


    shifted
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


/// K86's complete title letters match K67 two rows higher, within the original
/// colour tolerance. Level and rank artwork do not contribute to this group.
const LEVEL_UP_MASTER_TITLE: [RgbSample; 10] =
    shift_sample_rows(LEVEL_UP_MEDAL_TITLE, 2, 0);


/// The same coherent title translation retains every measured blue/white
/// contrast point; a flat white title patch cannot supply this evidence.
const LEVEL_UP_MASTER_TITLE_BACKGROUND: [RgbSample; 7] =
    shift_sample_rows(LEVEL_UP_MEDAL_TITLE_BACKGROUND, 2, 0);


/// K86's complete printed Klondike name lies seven rows below K67. Numeric
/// level, changing rank and reward text remain outside these letter samples.
const LEVEL_UP_MASTER_KLONDIKE: [RgbSample; 10] =
    shift_sample_rows(LEVEL_UP_KLONDIKE, 0, 7);


/// Original native K86 background values between its translated game letters.
/// The medal's blue rays differ from K67 at one point beyond the unchanged
/// tolerance, so this complete mixed group is measured independently.
const LEVEL_UP_MASTER_KLONDIKE_BACKGROUND: [RgbSample; 8] = [
    RgbSample::new(771, 640, [22, 54, 90]),
    RgbSample::new(800, 626, [20, 56, 92]),
    RgbSample::new(827, 638, [33, 69, 104]),
    RgbSample::new(855, 624, [29, 65, 98]),
    RgbSample::new(888, 639, [42, 78, 110]),
    RgbSample::new(906, 626, [46, 84, 114]),
    RgbSample::new(931, 639, [26, 62, 94]),
    RgbSample::new(961, 626, [22, 62, 95]),
];


/// K86's complete black OK caption lies two rows below the K67 caption.
const LEVEL_UP_MASTER_OK: [RgbSample; 10] =
    shift_sample_rows(LEVEL_UP_MEDAL_OK, 0, 2);


/// All four inner gold contrast points share K86's measured OK translation.
const LEVEL_UP_MASTER_OK_BACKGROUND: [RgbSample; 4] =
    shift_sample_rows(LEVEL_UP_MEDAL_OK_BACKGROUND, 0, 2);


/// Both upper and lower button edges retain their full translated body proof.
const LEVEL_UP_MASTER_BUTTON_BODY: [RgbSample; 4] =
    shift_sample_rows(LEVEL_UP_MEDAL_BUTTON_BODY, 0, 2);


/// K86's enlarged modal frame matches the complete K67 frame two rows lower,
/// independently of the title, game label, OK control and completed board.
const LEVEL_UP_MASTER_FRAME: [RgbSample; 8] =
    shift_sample_rows(LEVEL_UP_MEDAL_FRAME, 0, 2);


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


/// Complete title letters from K67's larger medal dialog, measured independently
/// of K27. Its two changed glyph edges do not relax the original title signature.
const LEVEL_UP_MEDAL_TITLE: [RgbSample; 10] = [
    RgbSample::new(816, 217, [255, 255, 255]),
    RgbSample::new(849, 199, [255, 255, 255]),
    RgbSample::new(884, 217, [255, 255, 255]),
    RgbSample::new(920, 199, [255, 255, 255]),
    RgbSample::new(944, 224, [255, 255, 255]),
    RgbSample::new(962, 199, [255, 255, 255]),
    RgbSample::new(1015, 217, [252, 254, 255]),
    RgbSample::new(1041, 199, [253, 254, 255]),
    RgbSample::new(1069, 217, [255, 255, 255]),
    RgbSample::new(1103, 199, [255, 255, 255]),
];


/// K67's positive blue gaps distinguish title text from an erased white banner.
const LEVEL_UP_MEDAL_TITLE_BACKGROUND: [RgbSample; 7] = [
    RgbSample::new(824, 217, [24, 172, 221]),
    RgbSample::new(863, 196, [31, 179, 228]),
    RgbSample::new(875, 219, [32, 180, 229]),
    RgbSample::new(875, 203, [32, 180, 229]),
    RgbSample::new(1058, 219, [32, 180, 229]),
    RgbSample::new(1058, 196, [32, 180, 229]),
    RgbSample::new(1099, 196, [25, 173, 222]),
];


/// K67 retains the original Klondike letter coordinates at offset zero, with
/// these independently measured background gaps. No level or rank text is read.
const LEVEL_UP_MEDAL_KLONDIKE_BACKGROUND: [RgbSample; 8] = [
    RgbSample::new(771, 633, [38, 71, 103]),
    RgbSample::new(800, 619, [34, 70, 104]),
    RgbSample::new(827, 631, [21, 59, 92]),
    RgbSample::new(855, 617, [26, 62, 95]),
    RgbSample::new(888, 632, [31, 68, 100]),
    RgbSample::new(906, 619, [33, 71, 102]),
    RgbSample::new(931, 632, [48, 86, 117]),
    RgbSample::new(961, 619, [37, 77, 110]),
];


/// K67's lower OK glyph is complete black print, not a generic dark button area.
const LEVEL_UP_MEDAL_OK: [RgbSample; 10] = [
    RgbSample::new(936, 820, [0, 0, 0]),
    RgbSample::new(938, 808, [0, 0, 0]),
    RgbSample::new(946, 826, [5, 4, 1]),
    RgbSample::new(954, 809, [0, 0, 0]),
    RgbSample::new(956, 820, [0, 0, 0]),
    RgbSample::new(965, 810, [1, 1, 1]),
    RgbSample::new(966, 820, [0, 0, 0]),
    RgbSample::new(972, 811, [7, 6, 4]),
    RgbSample::new(976, 821, [1, 1, 0]),
    RgbSample::new(975, 808, [1, 1, 1]),
];


/// K67's gold inside the OK glyph rejects a flattened black caption region.
const LEVEL_UP_MEDAL_OK_BACKGROUND: [RgbSample; 4] = [
    RgbSample::new(943, 814, [237, 222, 132]),
    RgbSample::new(943, 820, [230, 195, 88]),
    RgbSample::new(961, 808, [246, 235, 166]),
    RgbSample::new(961, 824, [216, 170, 67]),
];


/// K67's four measured light/dark button corners preserve the complete control.
/// The prior native click (960, 795) remains inside its visible upper gold body.
const LEVEL_UP_MEDAL_BUTTON_BODY: [RgbSample; 4] = [
    RgbSample::new(858, 788, [253, 247, 222]),
    RgbSample::new(1060, 788, [252, 249, 223]),
    RgbSample::new(856, 842, [183, 132, 37]),
    RgbSample::new(1060, 841, [176, 129, 40]),
];


/// K67's complete enlarged panel: white opposed edges and lower point, with
/// independent blue/dark inner corners. Medal, rank and reward text are excluded.
const LEVEL_UP_MEDAL_FRAME: [RgbSample; 8] = [
    RgbSample::new(462, 320, [248, 255, 255]),
    RgbSample::new(1457, 320, [248, 255, 255]),
    RgbSample::new(462, 750, [249, 255, 255]),
    RgbSample::new(1457, 750, [249, 255, 255]),
    RgbSample::new(960, 908, [235, 242, 245]),
    RgbSample::new(560, 720, [32, 67, 123]),
    RgbSample::new(1360, 720, [33, 68, 125]),
    RgbSample::new(750, 760, [8, 18, 31]),
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


/// Independently recognise K67's larger Level Up layout. Every artwork group
/// must pass at its one supplied position, along with the unchanged occupied
/// foundations and empty-tableau context. This grants the existing typed OK
/// stage only; it neither infers a new delay nor retries a previous control.
fn has_completed_medal_level_up(frame: &CapturedFrame) -> bool {
    matches_artwork(frame, &LEVEL_UP_MEDAL_TITLE)
        && matches_artwork(frame, &LEVEL_UP_MEDAL_TITLE_BACKGROUND)
        && matches_artwork(frame, &LEVEL_UP_KLONDIKE)
        && matches_artwork(frame, &LEVEL_UP_MEDAL_KLONDIKE_BACKGROUND)
        && matches_artwork(frame, &LEVEL_UP_MEDAL_OK)
        && matches_artwork(frame, &LEVEL_UP_MEDAL_OK_BACKGROUND)
        && matches_artwork(frame, &LEVEL_UP_MEDAL_BUTTON_BODY)
        && matches_artwork(frame, &LEVEL_UP_MEDAL_FRAME)
        && has_completed_level_up_context(frame)
}


/// Recognise K86's independently measured Master medal components as a complete
/// layout. Original small/Pro layouts keep their own branches. Every title,
/// printed game label, OK and frame group must pass its fixed recorded position
/// alongside the unchanged completed-board context before the existing OK stage
/// is returned; a known sequence or generic gold button never supplies authority.
fn has_completed_master_medal_level_up(frame: &CapturedFrame) -> bool {
    matches_artwork(frame, &LEVEL_UP_MASTER_TITLE)
        && matches_artwork(frame, &LEVEL_UP_MASTER_TITLE_BACKGROUND)
        && matches_artwork(frame, &LEVEL_UP_MASTER_KLONDIKE)
        && matches_artwork(frame, &LEVEL_UP_MASTER_KLONDIKE_BACKGROUND)
        && matches_artwork(frame, &LEVEL_UP_MASTER_OK)
        && matches_artwork(frame, &LEVEL_UP_MASTER_OK_BACKGROUND)
        && matches_artwork(frame, &LEVEL_UP_MASTER_BUTTON_BODY)
        && matches_artwork(frame, &LEVEL_UP_MASTER_FRAME)
        && has_completed_level_up_context(frame)
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


/// Four narrow paper margins independently establish one complete initial face.
/// K30/K77 retain at least 98% white in their side margins and complete horizontal
/// margins. The 95% floor permits their measured edge antialiasing while excluding
/// central court artwork from authority. Neither a back nor felt supplies paper.
fn has_initial_face_paper(frame: &CapturedFrame, column: u32) -> bool {


    if column >= 7 {
        return false;
    }
    let x = 390 + 168 * column;
    let face_top = 342 + 35 * column / 2;
    [
        PixelRect::new(x + 3, face_top + 18, 4, 138),
        PixelRect::new(x + 125, face_top + 18, 4, 138),
        PixelRect::new(x + 18, face_top + 3, 96, 4),
        PixelRect::new(x + 18, face_top + 169, 96, 3),
    ].into_iter().all(|bounds| {
        super::fraction_at_least(frame, bounds, super::is_white, 950)
    })
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


        if !has_initial_face_paper(frame, column) || !super::fraction_at_least(
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


/// Recognise the supplied native layouts for the five terminal stages; unknown or
/// shifted scenes return `None`. Malformed storage or dimensions return the parent layout error.
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


    if has_completed_medal_level_up(frame) {
        return Ok(Some(TerminalStage::LevelUp));
    }


    if has_completed_master_medal_level_up(frame) {
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


/// Positive local readiness for one expected control in an already confirmed win.
/// These counts are not scene classification or independent completion evidence.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ExpectedControlEvidence {
    /// The single stage requested by the deterministic restart controller.
    stage: TerminalStage,
    /// Positive warm gold or cream pixels inside the fixed button body region.
    gold_pixels: u32,
    /// Number of pixels in that local body region.
    body_pixels: u32,
    /// Required gold fraction, expressed per thousand for checked integer comparison.
    gold_per_mille: u32,
    /// Dark printed-caption pixels with nearby positive gold contrast.
    caption_ink_pixels: u32,
}


impl ExpectedControlEvidence {


    /// Require the local gold body and material printed lettering together.
    /// No precise glyph pixels, title, level or decorative artwork are compared.
    pub(crate) const fn ready(self) -> bool {
        self.gold_pixels * 1_000 >= self.body_pixels * self.gold_per_mille
            && self.caption_ink_pixels >= 48
    }
}


impl fmt::Display for ExpectedControlEvidence {


    /// Report only the expected control's local readiness, separate from win proof.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "expected={}, local gold={}/{} (required {} per mille), caption contrast={} (required 48), ready={}; decorative artwork is not checked",
            self.stage, self.gold_pixels, self.body_pixels, self.gold_per_mille,
            self.caption_ink_pixels, self.ready())
    }
}


/// Warm button gold and its pale upper gradient, without exact RGB signatures.
/// Blue dialog artwork, black lettering and white/grey flat patches are excluded.
fn is_control_gold([red, green, blue]: [u8; 3]) -> bool {
    red >= 140 && green >= 95
        && u16::from(red) >= u16::from(blue) + 20
        && u16::from(green) >= u16::from(blue) + 12
}


/// Measure only the expected printed button at its existing native click region.
/// The local OK window contains the prior small/medal captions and K91 caption;
/// no search extends beyond that control. New Game and Play body regions retain
/// sufficient independent horizontal coverage to reject each other's overlap.
/// The largest body has 24,192 pixels, keeping all count products within u32.
/// Missing storage or unsupported frame dimensions fail before any measurement.
pub(crate) fn inspect_expected_control(
    frame: &CapturedFrame,
    stage: TerminalStage,
) -> Result<Option<ExpectedControlEvidence>, HaloDetectionError> {
    super::validate_frame(frame)?;


    let (body, caption, gold_per_mille) = match stage {
        TerminalStage::LevelUp => (PixelRect::new(848, 750, 224, 108),
            PixelRect::new(928, 768, 62, 69), 550),
        TerminalStage::NewGame => (PixelRect::new(642, 824, 288, 55),
            PixelRect::new(693, 831, 191, 44), 800),
        TerminalStage::Play => (PixelRect::new(612, 834, 191, 50),
            PixelRect::new(673, 841, 72, 39), 850),
        TerminalStage::ScoreCounting | TerminalStage::SolverReady => return Ok(None),
    };
    let gold_pixels = super::count_pixels(frame, body, is_control_gold);
    let mut caption_ink_pixels = 0;


    for y in caption.y..caption.y + caption.height {


        for x in caption.x..caption.x + caption.width {
            let printed_ink = pixel_rgb(frame, x, y)
                .is_some_and(|rgb| rgb.into_iter().all(|channel| channel <= 70));


            if printed_ink && [(4, 0), (-4, 0), (0, 4), (0, -4)].into_iter()
                .any(|(dx, dy)| {
                    x.checked_add_signed(dx).zip(y.checked_add_signed(dy))
                        .and_then(|(column, row)| pixel_rgb(frame, column, row))
                        .is_some_and(is_control_gold)
                })
            {
                caption_ink_pixels += 1;
            }
        }
    }
    Ok(Some(ExpectedControlEvidence {
        stage, gold_pixels, body_pixels: body.width * body.height,
        gold_per_mille, caption_ink_pixels,
    }))
}


/// Gate the one expected restart control after independent game-win confirmation.
/// Local readiness is deliberately separate from `classify_terminal`, so a gold
/// button cannot declare a win. Fresh-deal Solver retains its existing scene proof.
pub(crate) fn expected_control_ready(
    frame: &CapturedFrame,
    stage: TerminalStage,
) -> Result<bool, HaloDetectionError> {


    if let Some(evidence) = inspect_expected_control(frame, stage)? {
        return Ok(evidence.ready());
    }
    Ok(classify_terminal(frame)? == Some(stage))
}


/// Locate the entry of a restart sequence whose game win is already confirmed.
/// A later Level Up/New Game frame may outlive the score panel; local buttons
/// allow entry without re-proving completed-game artwork. This function is never
/// called by completion detection, and Play cannot skip earlier restart stages.
pub(crate) fn classify_confirmed_win_entry(
    frame: &CapturedFrame,
) -> Result<Option<TerminalStage>, HaloDetectionError> {


    if let Some(stage) = classify_terminal(frame)? {
        return Ok(Some(stage));
    }


    for stage in [TerminalStage::LevelUp, TerminalStage::NewGame] {


        if expected_control_ready(frame, stage)? {
            return Ok(Some(stage));
        }
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
    /// K67 medal-layout title, label, OK and frame groups, all independently required.
    level_up_medal: [SignatureEvidence; 8],
    /// K86 Master-layout title, label, OK and frame groups, all independently required.
    level_up_master: [SignatureEvidence; 8],
    /// Static Solver word and key artwork, separate from fresh-board proof.
    solver_control: [SignatureEvidence; 2],
    /// Independent initial-deal topology; no-HALO alone cannot make this true.
    fresh_deal: bool,
}


impl fmt::Display for TerminalEvidence {


    /// Surface the refused scene guard without inferring missing worker pixels.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter,
            "Congratulations title={}/{}, frame={}, completed context={}, skip={}/{}, New Game={}/{}/{}; Level Up title={}/{}, label +0={}/{}, label +2={}/{}, OK={}/{}/{}, foundations={}, empty tableau={}; medal layout title={}/{}, label={}/{}, OK={}/{}/{}, frame={}; Master layout title={}/{}, label={}/{}, OK={}/{}/{}, frame={}; Solver word={}, key={}, fresh-deal={}",
            self.congratulations_ribbon[0], self.congratulations_ribbon[1],
            self.congratulations_frame, self.congratulations_context,
            self.score_control[0], self.score_control[1],
            self.new_game_control[0], self.new_game_control[1], self.new_game_control[2],
            self.level_up_title[0], self.level_up_title[1],
            self.level_up_label[0][0], self.level_up_label[0][1],
            self.level_up_label[1][0], self.level_up_label[1][1],
            self.level_up_control[0], self.level_up_control[1], self.level_up_control[2],
            self.level_up_foundations, self.level_up_tableau,
            self.level_up_medal[0], self.level_up_medal[1],
            self.level_up_medal[2], self.level_up_medal[3],
            self.level_up_medal[4], self.level_up_medal[5], self.level_up_medal[6],
            self.level_up_medal[7], self.level_up_master[0], self.level_up_master[1],
            self.level_up_master[2], self.level_up_master[3], self.level_up_master[4],
            self.level_up_master[5], self.level_up_master[6], self.level_up_master[7],
            self.solver_control[0], self.solver_control[1],
            self.fresh_deal)
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
        level_up_medal: [artwork(&LEVEL_UP_MEDAL_TITLE),
            artwork(&LEVEL_UP_MEDAL_TITLE_BACKGROUND), artwork(&LEVEL_UP_KLONDIKE),
            artwork(&LEVEL_UP_MEDAL_KLONDIKE_BACKGROUND), artwork(&LEVEL_UP_MEDAL_OK),
            artwork(&LEVEL_UP_MEDAL_OK_BACKGROUND), artwork(&LEVEL_UP_MEDAL_BUTTON_BODY),
            artwork(&LEVEL_UP_MEDAL_FRAME)],
        level_up_master: [artwork(&LEVEL_UP_MASTER_TITLE),
            artwork(&LEVEL_UP_MASTER_TITLE_BACKGROUND), artwork(&LEVEL_UP_MASTER_KLONDIKE),
            artwork(&LEVEL_UP_MASTER_KLONDIKE_BACKGROUND), artwork(&LEVEL_UP_MASTER_OK),
            artwork(&LEVEL_UP_MASTER_OK_BACKGROUND), artwork(&LEVEL_UP_MASTER_BUTTON_BODY),
            artwork(&LEVEL_UP_MASTER_FRAME)],
        solver_control: [artwork(&SOLVER_LABEL), artwork(&SOLVER_KEY)],
        fresh_deal: has_fresh_deal(frame)?,
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
            67 => include_bytes!("../tests/fixtures/klondike/K67.png"),
            77 => include_bytes!("../tests/fixtures/klondike/K77.png"),
            86 => include_bytes!("../tests/fixtures/klondike/K86.png"),
            91 => include_bytes!("../tests/fixtures/klondike/K91.png"),
            92 => include_bytes!("../tests/fixtures/klondike/K92.png"),
            93 => include_bytes!("../tests/fixtures/klondike/K93.png"),
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


    /// Keep only the local expected button; all other artwork is deliberately erased.
    /// This controlled derivative proves readiness has no decorative dependencies.
    fn local_button_only(number: usize, stage: TerminalStage) -> CapturedFrame {
        let original = fixture(number);


        let bounds = match stage {
            TerminalStage::LevelUp => PixelRect::new(848, 750, 224, 108),
            TerminalStage::NewGame => PixelRect::new(642, 824, 288, 55),
            TerminalStage::Play => PixelRect::new(612, 834, 191, 50),
            _ => panic!("test helper requires a local terminal button"),
        };
        let mut local = original.clone();
        local.pixels.fill(0);


        for y in bounds.y..bounds.y + bounds.height {
            let start = y as usize * local.stride + bounds.x as usize * 4;
            let end = start + bounds.width as usize * 4;
            local.pixels[start..end].copy_from_slice(&original.pixels[start..end]);
        }
        local
    }


    /// Grandmaster artwork changes cannot erase the local expected controls.
    /// The three body windows distinguish overlapping New Game/Play surfaces.
    #[test]
    fn confirmed_win_buttons_follow_the_expected_stage_across_native_themes() {
        let stages = [TerminalStage::LevelUp, TerminalStage::NewGame, TerminalStage::Play];


        for (number, expected) in [(27, stages[0]), (49, stages[0]), (52, stages[0]),
            (67, stages[0]), (86, stages[0]), (91, stages[0]), (28, stages[1]),
            (53, stages[1]), (92, stages[1]), (29, stages[2]), (93, stages[2])]
        {
            let frame = fixture(number);


            for stage in stages {
                assert_eq!(expected_control_ready(&frame, stage).unwrap(), stage == expected,
                    "K{number}, expected {expected}, checked {stage}");
            }
            assert!(expected_control_ready(&local_button_only(number, expected), expected).unwrap());
        }
    }


    /// Gold/printed button readiness grants progression only after an existing win.
    /// Its button alone cannot enter independent completion evidence on a blank scene.
    #[test]
    fn local_expected_controls_never_prove_initial_completion() {


        for (number, stage) in [(91, TerminalStage::LevelUp),
            (92, TerminalStage::NewGame), (93, TerminalStage::Play)]
        {
            let frame = local_button_only(number, stage);
            assert!(expected_control_ready(&frame, stage).unwrap());
            assert_eq!(classify_terminal(&frame).unwrap(), None);
            assert!(!super::super::completion_evidence(&frame).unwrap().complete_candidate);
        }
        assert_eq!(classify_confirmed_win_entry(&fixture(91)).unwrap(), Some(TerminalStage::LevelUp));
        assert_eq!(classify_confirmed_win_entry(&fixture(92)).unwrap(), Some(TerminalStage::NewGame));
    }


    /// An erased caption, dark/white/flat control and absent body refuse readiness.
    /// Unknown or unchanged windows never authorise the following expected stage.
    #[test]
    fn expected_button_readiness_requires_gold_and_material_print_contrast() {


        for (number, stage, body, caption) in [
            (91, TerminalStage::LevelUp, PixelRect::new(848, 750, 224, 108), PixelRect::new(928, 768, 62, 69)),
            (92, TerminalStage::NewGame, PixelRect::new(642, 824, 288, 55), PixelRect::new(693, 831, 191, 44)),
            (93, TerminalStage::Play, PixelRect::new(612, 834, 191, 50), PixelRect::new(673, 841, 72, 39)),
        ] {


            for rgb in [[0, 0, 0], [255, 255, 255], [240, 190, 90], [12, 82, 45]] {
                let mut erased = fixture(number);
                paint(&mut erased, body, rgb);
                assert!(!expected_control_ready(&erased, stage).unwrap());
            }
            let mut no_print = fixture(number);
            paint(&mut no_print, caption, [240, 190, 90]);
            assert!(!expected_control_ready(&no_print, stage).unwrap());
        }
        assert!(!expected_control_ready(&fixture(91), TerminalStage::NewGame).unwrap());
        assert!(!expected_control_ready(&fixture(92), TerminalStage::Play).unwrap());
    }


    /// Unsupported geometry and malformed byte storage fail before local sampling.
    #[test]
    fn expected_button_readiness_rejects_invalid_frame_layouts() {
        let mut wrong_size = fixture(91);
        wrong_size.height = 1_079;
        assert_eq!(expected_control_ready(&wrong_size, TerminalStage::LevelUp), Err(HaloDetectionError::BoundsOutsideFrame));
        let mut malformed = fixture(91);
        malformed.pixels.clear();
        assert_eq!(expected_control_ready(&malformed, TerminalStage::LevelUp), Err(HaloDetectionError::InvalidFrameLayout));
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


    /// K67's measured larger panel has a complete alternate signature. The
    /// original signature refuses it; the existing OK point remains positive
    /// gold inside its button, retaining the original hold and transition wait.
    #[test]
    fn medal_level_up_recognises_the_existing_ok_control() {
        let frame = fixture(67);
        assert!(!matches_artwork(&frame, &LEVEL_UP_TITLE));
        assert!(!matches_artwork(&frame, &LEVEL_UP_OK));
        assert!(has_completed_level_up_context(&frame));
        assert!(has_completed_medal_level_up(&frame));
        assert_eq!(classify_terminal(&frame).unwrap(), Some(TerminalStage::LevelUp));
        assert_eq!(pixel_rgb(&frame, 960, 795), Some([253, 248, 223]));
        assert_eq!(TerminalStage::LevelUp.click_point(), PixelPoint::new(960, 795));
        assert_eq!(TerminalStage::LevelUp.mouse_hold(), POST_GAME_MOUSE_HOLD);
        assert_eq!(TerminalStage::LevelUp.settle_delay(Duration::ZERO), POST_GAME_STAGE_DELAY);
        let evidence = inspect_terminal_evidence(&frame).unwrap();
        assert!(evidence.level_up_medal.iter().all(|guard| guard.matched == guard.required));
        assert!(evidence.to_string().contains("medal layout title=10/10/7/7, label=10/10/8/8, OK=10/10/4/4/4/4, frame=8/8"));
    }


    /// K86's Master medal changes independently aligned title, label and button
    /// artwork. It still requires the completed board and measured existing OK
    /// point, rather than a generic gold control or an assumed terminal sequence.
    #[test]
    fn master_medal_level_up_recognises_the_existing_ok_control() {
        let frame = fixture(86);
        assert!(!has_completed_medal_level_up(&frame));
        assert!(has_completed_master_medal_level_up(&frame));
        assert!(has_completed_level_up_context(&frame));
        assert_eq!(pixel_rgb(&frame, 960, 795), Some([253, 248, 223]));
        assert_eq!(classify_terminal(&frame).unwrap(), Some(TerminalStage::LevelUp));
        assert_eq!(TerminalStage::LevelUp.click_point(), PixelPoint::new(960, 795));
        assert_eq!(TerminalStage::LevelUp.mouse_hold(), POST_GAME_MOUSE_HOLD);
        assert_eq!(TerminalStage::LevelUp.settle_delay(Duration::ZERO), POST_GAME_STAGE_DELAY);
        let evidence = inspect_terminal_evidence(&frame).unwrap();
        assert!(evidence.level_up_master.iter().all(|guard| guard.matched == guard.required));
        assert_eq!(evidence.level_up_medal[0].matched, 7);
        assert_eq!(evidence.level_up_medal[4].matched, 6);
        assert!(evidence.to_string().contains("Master layout title=10/10/7/7, label=10/10/8/8, OK=10/10/4/4/4/4, frame=8/8"));
    }


    /// Each complete Master group is independently required. Erasing a title,
    /// printed game label, OK control or frame cannot turn a known sequence into
    /// click authority, even with all remaining native scene pixels intact.
    #[test]
    fn master_medal_level_up_requires_complete_scene_and_control_artwork() {
        let bounds = [PixelRect::new(804, 177, 315, 55),
            PixelRect::new(758, 616, 220, 35), PixelRect::new(839, 780, 241, 80),
            PixelRect::new(456, 710, 110, 52)];


        for bounds in bounds {


            for rgb in [[0, 0, 0], [255, 255, 255], [240, 190, 90]] {
                let mut frame = fixture(86);
                paint(&mut frame, bounds, rgb);
                assert_eq!(classify_terminal(&frame).unwrap(), None,
                    "missing K86 artwork {bounds:?} {rgb:?}");
            }
        }


        for samples in [&LEVEL_UP_MASTER_TITLE[..], &LEVEL_UP_MASTER_TITLE_BACKGROUND[..],
            &LEVEL_UP_MASTER_KLONDIKE[..], &LEVEL_UP_MASTER_KLONDIKE_BACKGROUND[..],
            &LEVEL_UP_MASTER_OK[..], &LEVEL_UP_MASTER_OK_BACKGROUND[..],
            &LEVEL_UP_MASTER_BUTTON_BODY[..], &LEVEL_UP_MASTER_FRAME[..]]
        {
            let sample = samples[0];
            let contrast = if sample.rgb.iter().all(|channel| *channel > 200) {
                [0, 0, 0]
            } else {
                [255, 255, 255]
            };
            let mut frame = fixture(86);
            paint(&mut frame, PixelRect::new(sample.x, sample.y, 1, 1), contrast);
            assert_eq!(classify_terminal(&frame).unwrap(), None,
                "missing K86 sample ({}, {})", sample.x, sample.y);
        }
    }


    /// A visible Master dialog does not establish game win when any occupied
    /// foundation or independent empty-tableau context sample is missing.
    #[test]
    fn master_medal_level_up_requires_each_completed_board_context_point() {


        for sample in LEVEL_UP_FOUNDATION_CONTEXT.into_iter()
            .chain(LEVEL_UP_EMPTY_TABLEAU_CONTEXT)
        {


            for rgb in [[0, 0, 0], [255, 255, 255]] {
                let mut frame = fixture(86);
                paint(&mut frame, PixelRect::new(sample.x, sample.y, 1, 1), rgb);
                assert_eq!(classify_terminal(&frame).unwrap(), None);
            }
        }
    }


    /// The Master branch requires its one recorded control position. A complete
    /// real OK glyph shifted horizontally cannot authorise the old click point.
    #[test]
    fn master_medal_level_up_rejects_a_displaced_ok_control() {
        let original = fixture(86);
        let mut moved = original.clone();
        paint(&mut moved, PixelRect::new(839, 780, 257, 80), [0, 0, 0]);


        for y in 780..860usize {
            let source = y * original.stride + 839 * 4;
            let destination = y * moved.stride + 855 * 4;
            moved.pixels[destination..destination + 241 * 4]
                .copy_from_slice(&original.pixels[source..source + 241 * 4]);
        }
        assert!(matches_artwork(&moved, &LEVEL_UP_MASTER_TITLE));
        assert!(has_completed_level_up_context(&moved));
        assert_eq!(classify_terminal(&moved).unwrap(), None);
    }


    /// Changing medal art, level digits, rank or next-reward text cannot erase
    /// or create the separate title, Klondike label, complete OK and frame proof.
    /// Synthetic erasure does not calibrate another unseen medal layout.
    #[test]
    fn master_medal_level_up_excludes_changing_rank_and_reward_artwork() {
        let mut frame = fixture(86);


        for bounds in [PixelRect::new(740, 330, 440, 273),
            PixelRect::new(1_110, 610, 60, 45), PixelRect::new(900, 652, 140, 41),
            PixelRect::new(790, 702, 370, 39)]
        {
            paint(&mut frame, bounds, [0, 0, 0]);
        }
        assert_eq!(classify_terminal(&frame).unwrap(), Some(TerminalStage::LevelUp));
    }


    /// Unknown flat colours and malformed frames cannot acquire terminal input
    /// from the new Master signature. Native frame validation runs first.
    #[test]
    fn master_medal_level_up_rejects_unknown_and_malformed_native_frames() {


        for rgb in [[0, 0, 0], [255, 255, 255], [240, 190, 90], [20, 120, 70]] {
            let mut frame = fixture(86);
            paint(&mut frame, PixelRect::new(0, 0, 1_920, 1_080), rgb);
            assert_eq!(classify_terminal(&frame).unwrap(), None);
        }
        let mut wrong_size = fixture(86);
        wrong_size.height = 1_079;
        assert!(matches!(classify_terminal(&wrong_size), Err(HaloDetectionError::BoundsOutsideFrame)));
        let mut malformed = fixture(86);
        malformed.pixels.clear();
        assert!(matches!(classify_terminal(&malformed), Err(HaloDetectionError::InvalidFrameLayout)));
    }


    /// K67's numeric level, rank and next reward caption cannot alter the
    /// independent title, game label, complete OK artwork or board context.
    /// Controlled erasure does not claim an unseen rank layout was calibrated.
    #[test]
    fn medal_level_up_excludes_numeric_rank_and_reward_text() {
        let mut frame = fixture(67);


        for bounds in [PixelRect::new(1_110, 610, 60, 38),
            PixelRect::new(920, 652, 85, 41), PixelRect::new(790, 702, 370, 39)]
        {
            paint(&mut frame, bounds, [0, 0, 0]);
        }
        assert_eq!(classify_terminal(&frame).unwrap(), Some(TerminalStage::LevelUp));
    }


    /// K67 requires its title, mixed printed game name, positive gold/black OK
    /// and enlarged panel frame independently. Flat white, gold and black patches
    /// cannot substitute for any one of these positively observed scene groups.
    #[test]
    fn medal_level_up_requires_complete_scene_and_control_artwork() {
        let bounds = [PixelRect::new(804, 185, 315, 46),
            PixelRect::new(760, 610, 220, 35), PixelRect::new(839, 780, 241, 76),
            PixelRect::new(456, 710, 110, 50)];


        for bounds in bounds {


            for rgb in [[0, 0, 0], [255, 255, 255], [240, 190, 90]] {
                let mut frame = fixture(67);
                paint(&mut frame, bounds, rgb);
                assert_eq!(classify_terminal(&frame).unwrap(), None, "missing K67 artwork {bounds:?} {rgb:?}");
            }
        }


        for sample in LEVEL_UP_MEDAL_FRAME {
            let mut frame = fixture(67);
            let contrast = if sample.rgb.iter().all(|channel| *channel > 200) {
                [0, 0, 0]
            } else {
                [255, 255, 255]
            };
            paint(&mut frame, PixelRect::new(sample.x, sample.y, 1, 1), contrast);
            assert!(!matches_artwork(&frame, &LEVEL_UP_TITLE));
            assert_eq!(inspect_terminal_evidence(&frame).unwrap().level_up_medal[7].matched, 7);
            assert_eq!(classify_terminal(&frame).unwrap(), None);
        }
    }


    /// A full medal/OK dialog cannot claim a win when any foundation margin or
    /// independently empty side-tableau point lacks the existing completed board.
    #[test]
    fn medal_level_up_requires_each_completed_board_context_point() {


        for sample in LEVEL_UP_FOUNDATION_CONTEXT.into_iter()
            .chain(LEVEL_UP_EMPTY_TABLEAU_CONTEXT)
        {


            for rgb in [[0, 0, 0], [255, 255, 255]] {
                let mut frame = fixture(67);
                paint(&mut frame, PixelRect::new(sample.x, sample.y, 1, 1), rgb);
                assert_eq!(classify_terminal(&frame).unwrap(), None);
            }
        }
    }


    /// The new artwork never grants the old click when its visible OK has moved
    /// elsewhere. Its displaced real glyph remains intact but fixed guards refuse.
    #[test]
    fn medal_level_up_rejects_a_displaced_ok_control() {
        let original = fixture(67);
        let mut moved = original.clone();
        paint(&mut moved, PixelRect::new(839, 780, 257, 76), [0, 0, 0]);


        for y in 780..856usize {
            let source = y * original.stride + 839 * 4;
            let destination = y * moved.stride + 855 * 4;
            moved.pixels[destination..destination + 241 * 4]
                .copy_from_slice(&original.pixels[source..source + 241 * 4]);
        }
        assert!(matches_artwork(&moved, &LEVEL_UP_MEDAL_TITLE));
        assert!(has_completed_level_up_context(&moved));
        assert_eq!(classify_terminal(&moved).unwrap(), None);
    }


    /// Unknown flat scenes and malformed native layouts cannot use the new medal
    /// signature. Native bounds/storage validation runs before reading any sample.
    #[test]
    fn medal_level_up_rejects_unknown_and_malformed_native_frames() {


        for rgb in [[0, 0, 0], [255, 255, 255], [240, 190, 90], [20, 120, 70]] {
            let mut frame = fixture(67);
            paint(&mut frame, PixelRect::new(0, 0, 1_920, 1_080), rgb);
            assert_eq!(classify_terminal(&frame).unwrap(), None);
        }
        let mut wrong_size = fixture(67);
        wrong_size.height = 1_079;
        assert!(matches!(classify_terminal(&wrong_size), Err(HaloDetectionError::BoundsOutsideFrame)));
        let mut malformed = fixture(67);
        malformed.pixels.clear();
        assert!(matches!(classify_terminal(&malformed), Err(HaloDetectionError::InvalidFrameLayout)));
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


    /// Native K77's dense King artwork cannot reject the complete initial deal.
    #[test]
    fn solver_ready_accepts_native_initial_court_card_geometry() {
        let frame = fixture(77);
        assert!(!super::super::fraction_at_least(
            &frame, PixelRect::new(408, 360, 96, 110), super::super::is_white, 300,
        ));
        assert_eq!(classify_terminal(&frame).unwrap(), Some(TerminalStage::SolverReady));
        assert_eq!(classify_terminal(&fixture(30)).unwrap(), Some(TerminalStage::SolverReady));
        let evidence = inspect_terminal_evidence(&frame).unwrap();
        assert!(evidence.fresh_deal);
        assert!(evidence.solver_control.iter().all(|guard| guard.matched == guard.required));
        assert!(!has_initial_face_paper(&frame, 7));
        assert!(!has_initial_face_paper(&frame, u32::MAX));
    }


    /// Each side, top and bottom margin independently protects the initial face.
    #[test]
    fn solver_ready_requires_all_seven_complete_paper_perimeters() {


        for number in [30, 77] {


            for column in 0..7 {
                let x = 390 + 168 * column;
                let top = 342 + 35 * column / 2;


                for bounds in [
                    PixelRect::new(x + 3, top + 18, 4, 138),
                    PixelRect::new(x + 125, top + 18, 4, 138),
                    PixelRect::new(x + 18, top + 3, 96, 4),
                    PixelRect::new(x + 18, top + 169, 96, 3),
                ] {
                    let mut erased = fixture(number);
                    paint(&mut erased, bounds, [20, 120, 70]);
                    assert_eq!(classify_terminal(&erased).unwrap(), None,
                        "K{number} column {} erased {bounds:?}", column + 1);
                    assert!(!inspect_terminal_evidence(&erased).unwrap().fresh_deal);
                }
            }
        }
    }


    /// A genuinely dealt board still needs empty upper piles and its Solver control.
    #[test]
    fn native_court_card_deal_retains_solver_and_topology_guards() {


        for bounds in [
            PixelRect::new(406, 130, 100, 139),
            PixelRect::new(576, 130, 152, 139),
            PixelRect::new(910, 130, 100, 139),
            PixelRect::new(1_078, 130, 100, 139),
            PixelRect::new(1_246, 130, 100, 139),
            PixelRect::new(1_414, 130, 100, 139),
            PixelRect::new(576, 347, 96, 6),
            PixelRect::new(744, 347, 96, 6),
            PixelRect::new(912, 347, 96, 6),
            PixelRect::new(1_080, 347, 96, 6),
            PixelRect::new(1_248, 347, 96, 6),
            PixelRect::new(1_416, 347, 96, 6),
            PixelRect::new(560, 1_000, 80, 30),
            PixelRect::new(584, 954, 38, 42),
        ] {
            let mut erased = fixture(77);
            paint(&mut erased, bounds, [0, 0, 0]);
            assert_eq!(classify_terminal(&erased).unwrap(), None,
                "K77 erased required fresh-deal/control guard {bounds:?}");
        }
        let mut active = fixture(77);
        paint(&mut active, PixelRect::new(828, 36, 263, 1), [240, 190, 90]);
        paint(&mut active, PixelRect::new(828, 85, 263, 1), [240, 190, 90]);
        assert_eq!(classify_terminal(&active).unwrap(), None);
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


        for number in [26, 27, 28, 29, 30, 67] {
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
