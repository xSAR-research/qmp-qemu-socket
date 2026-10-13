//! Spider's one-board win and deterministic ordered restart controls.
//!
//! SP15/SP17 evidence the shared score-skip and New Game controls. SP18 places
//! Play 100 pixels below Free Cell's location. Optional Level Up searches a
//! bounded rectangle bottom-up for five consecutive exact-black RGB pixels.
//! That run supplies the click point; it does not independently prove a win.
//! No Solve, missing DRAW or missing HALO is win authority.

use std::fmt;

use thiserror::Error;
use xsar::image_matching::{MatchError, find_black_run_bottom_up};

use crate::{
    capture::CapturedFrame,
    detector::HaloDetectionError,
    freecell_terminal::{self, TerminalStage as SharedStage},
    geometry::{PixelPoint, PixelRect},
    parameters::{NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH},
};


/// Expected control in Spider's independently entered one-board win sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalStage {
    /// Congratulations reward counting, with a click-anywhere skip caption.
    Score,
    /// Optional Level Up OK after a confirmed game win.
    LevelUp,
    /// Congratulations New Game, retaining the current game type.
    NewGame,
    /// Spider Play, retaining the guest's selected difficulty.
    Play,
}


impl fmt::Display for TerminalStage {


    /// Name the expected Spider control without inferring preceding input effect.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Score => "Spider score counting / click to skip",
            Self::LevelUp => "Spider Level Up OK",
            Self::NewGame => "Spider Congratulations New Game",
            Self::Play => "Spider Play",
        })
    }
}


/// Convert only the shared control identity; Spider retains its flow and logs.
const fn shared_stage(stage: TerminalStage) -> SharedStage {


    match stage {
        TerminalStage::Score => SharedStage::Score,
        TerminalStage::LevelUp => SharedStage::LevelUp,
        TerminalStage::NewGame => SharedStage::NewGame,
        TerminalStage::Play => SharedStage::Play,
    }
}


/// Gross search rectangle in full-frame pixels: X930..=990, Y700..=850.
pub const LEVEL_UP_SEARCH: PixelRect = PixelRect::new(930, 700, 61, 151);


/// Required contiguous RGB(0, 0, 0) pixels on one row; alpha is ignored.
pub const LEVEL_UP_BLACK_RUN: u32 = 5;


/// Preserve the precise validation error from the shared or bounded detector.
#[derive(Debug, Error)]
pub enum TerminalDetectionError {
    /// Shared score, New Game or Play recognition rejected the frame.
    #[error(transparent)]
    Shared(#[from] HaloDetectionError),
    /// Checked black-run scanning rejected the frame or bounds.
    #[error(transparent)]
    Scan(#[from] MatchError),
    /// Full-frame click coordinates require the calibrated native geometry.
    #[error("Spider terminal detection requires a 1920x1080 frame; received {width}x{height}")]
    UnsupportedGeometry {
        /// Observed frame width.
        width: u32,
        /// Observed frame height.
        height: u32,
    },
}


/// Return the first qualifying run's leftmost full-frame pixel, or (0, 0).
/// The expected win-stage context supplies meaning; no glyph or colour-body
/// checks are added to the five-black-pixel rule.
pub fn level_up_click_point(frame: &CapturedFrame) -> Result<PixelPoint, TerminalDetectionError> {
    let view = frame.as_view()?;
    if frame.width != NOMINAL_FRAME_WIDTH || frame.height != NOMINAL_FRAME_HEIGHT {
        return Err(TerminalDetectionError::UnsupportedGeometry {
            width: frame.width, height: frame.height,
        });
    }
    Ok(find_black_run_bottom_up(view, LEVEL_UP_SEARCH, LEVEL_UP_BLACK_RUN)?)
}


/// Return a freshly recognised control's click point, or (0, 0) if absent.
/// Level Up uses its measured run; the other controls retain their fixed points.
pub fn expected_control_point(
    frame: &CapturedFrame,
    stage: TerminalStage,
) -> Result<PixelPoint, TerminalDetectionError> {
    if stage == TerminalStage::LevelUp {
        return level_up_click_point(frame);
    }
    let ready = if stage == TerminalStage::Play {
        freecell_terminal::play_control_ready_at(frame, 100)?
    } else {
        freecell_terminal::expected_control_ready(frame, shared_stage(stage))?
    };
    Ok(if !ready {
        PixelPoint::new(0, 0)
    } else if stage == TerminalStage::Play {
        PixelPoint::new(709, 861)
    } else {
        freecell_terminal::click_point(shared_stage(stage))
    })
}


/// Check only the currently expected control; this does not establish a win.
#[cfg(test)]
fn expected_control_ready(
    frame: &CapturedFrame,
    stage: TerminalStage,
) -> Result<bool, TerminalDetectionError> {
    Ok(expected_control_point(frame, stage)? != PixelPoint::new(0, 0))
}


/// Independent entry requires Congratulations plus its own score/New Game
/// caption. Generic gold, absent DRAW, absent HALO and isolated OK/Play cannot win.
pub fn classify_win_entry(
    frame: &CapturedFrame,
) -> Result<Option<TerminalStage>, HaloDetectionError> {
    Ok(match freecell_terminal::classify_win_entry(frame)? {
        Some(SharedStage::Score) => Some(TerminalStage::Score),
        Some(SharedStage::NewGame) => Some(TerminalStage::NewGame),
        Some(SharedStage::LevelUp | SharedStage::Play) | None => None,
    })
}


#[cfg(test)]
mod tests {
    //! Native Spider controls, optional Level Up and shared layout isolation.

    use super::*;
    use crate::capture::decode_png;


    /// Decode byte-identical native Spider terminal/deal evidence.
    fn fixture(number: u8) -> CapturedFrame {
        let path = format!("{}/tests/fixtures/spider-SP{number:02}.png", env!("CARGO_MANIFEST_DIR"));
        decode_png(&std::fs::read(path).unwrap()).unwrap()
    }


    /// Each evidenced screen admits only its own local expected control.
    #[test]
    fn native_spider_terminal_controls_and_independent_win_entry() {


        for (number, expected) in [(15, TerminalStage::Score),
            (17, TerminalStage::NewGame), (18, TerminalStage::Play),
            (20, TerminalStage::LevelUp)] {
            let frame = fixture(number);


            for stage in [TerminalStage::Score, TerminalStage::LevelUp,
                TerminalStage::NewGame, TerminalStage::Play] {
                assert_eq!(expected_control_ready(&frame, stage).unwrap(), stage == expected,
                    "SP{number:02} / {stage}");
            }
            assert_eq!(classify_win_entry(&frame).unwrap(),
                matches!(expected, TerminalStage::Score | TerminalStage::NewGame)
                    .then_some(expected));
        }
        assert_eq!(classify_win_entry(&fixture(19)).unwrap(), None);
    }


    /// The accepted Free Cell Play location is preserved and cannot authorise
    /// Spider Play; Spider's measured vertical shift does not alter Free Cell.
    #[test]
    fn play_layouts_are_mode_specific_and_outside_offsets_stop() {
        let frame = decode_png(include_bytes!("../tests/fixtures/freecell-FC12.png")).unwrap();
        assert!(freecell_terminal::expected_control_ready(&frame, SharedStage::Play).unwrap());
        assert!(!expected_control_ready(&frame, TerminalStage::Play).unwrap());
        assert!(expected_control_ready(&fixture(18), TerminalStage::Play).unwrap());
        assert!(!freecell_terminal::expected_control_ready(&fixture(18), SharedStage::Play).unwrap());
        assert_eq!(freecell_terminal::play_control_ready_at(&frame, u32::MAX),
            Err(HaloDetectionError::BoundsOutsideFrame));
    }


    /// Charlie confirms the optional OK uses the accepted local control; its
    /// readiness never independently establishes a Spider game win.
    #[test]
    fn optional_level_up_uses_existing_local_ok_only() {
        let frame = decode_png(include_bytes!("../tests/fixtures/freecell-FC10.png")).unwrap();
        assert!(expected_control_ready(&frame, TerminalStage::LevelUp).unwrap());
        assert_eq!(classify_win_entry(&frame).unwrap(), None);
    }


    /// Native Level Up positions produce their own measured black-run clicks.
    #[test]
    fn level_up_locations_are_measured_without_win_authority() {
        let current = decode_png(include_bytes!("../tests/fixtures/spider-level-up-505.png")).unwrap();
        let legacy = decode_png(include_bytes!("../tests/fixtures/freecell-FC10.png")).unwrap();
        for (frame, expected) in [
            (legacy, PixelPoint::new(941, 822)),
            (fixture(20), PixelPoint::new(942, 803)),
            (current, PixelPoint::new(944, 771)),
        ] {
            assert_eq!(level_up_click_point(&frame).unwrap(), expected);
            assert_eq!(expected_control_point(&frame, TerminalStage::LevelUp).unwrap(), expected);
            assert_eq!(classify_win_entry(&frame).unwrap(), None);
        }
    }


    /// A valid image that contains the ROI still needs native geometry because
    /// QMP click conversion uses the calibrated full-display dimensions.
    #[test]
    fn level_up_rejects_non_native_geometry_even_when_the_search_fits() {
        let mut frame = fixture(20);
        frame.width = 1_024;
        frame.height = 900;
        assert!(frame.as_view().is_ok());
        assert!(LEVEL_UP_SEARCH.right() <= frame.width);
        assert!(LEVEL_UP_SEARCH.bottom() <= frame.height);
        assert!(matches!(level_up_click_point(&frame),
            Err(TerminalDetectionError::UnsupportedGeometry { width: 1_024, height: 900 })));
    }


    /// The authorised rule examines only the supplied rectangle and exact RGB.
    #[test]
    fn missing_black_run_returns_zero_and_flat_black_is_not_win_authority() {
        let mut frame = fixture(20);
        for y in LEVEL_UP_SEARCH.y..LEVEL_UP_SEARCH.bottom() {
            for x in LEVEL_UP_SEARCH.x..LEVEL_UP_SEARCH.right() {
                let index = y as usize * frame.stride + x as usize * 4;
                frame.pixels[index..index + 4].copy_from_slice(&[1, 1, 1, 255]);
            }
        }
        assert_eq!(level_up_click_point(&frame).unwrap(), PixelPoint::new(0, 0));
        assert!(!expected_control_ready(&frame, TerminalStage::LevelUp).unwrap());
        assert_eq!(classify_win_entry(&frame).unwrap(), None);

        for pixel in frame.pixels.as_chunks_mut::<4>().0 {
            pixel.copy_from_slice(&[0, 0, 0, 255]);
        }
        assert_eq!(level_up_click_point(&frame).unwrap(), PixelPoint::new(930, 850));
        assert_eq!(classify_win_entry(&frame).unwrap(), None);
    }
}
