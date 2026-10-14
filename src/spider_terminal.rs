//! Spider's one-board win and deterministic ordered restart controls.
//!
//! SP15/SP17 evidence score-skip and independent New Game win entry.
//! OK, New Game and Play use Spider control rectangles, scanned bottom-up
//! for five consecutive exact-black RGB pixels.
//! That run supplies the click point; it does not independently prove a win.
//! No Solve, missing DRAW or missing HALO is win authority.

use std::fmt;

use crate::{
    capture::CapturedFrame,
    detector::HaloDetectionError,
    freecell_terminal::{self, TerminalStage as SharedStage},
    geometry::PixelPoint,
    terminal_controls::{self, Control, profiles::SPIDER_CONTROLS},
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


/// Map Spider stages to control identities; bounds remain in its own profile.
pub const fn control(stage: TerminalStage) -> Option<Control> {
    freecell_terminal::control(shared_stage(stage))
}


/// Return this frame's measured click point, or (0, 0) if absent.
pub fn expected_control_point(
    frame: &CapturedFrame,
    stage: TerminalStage,
) -> Result<PixelPoint, HaloDetectionError> {
    if let Some(control) = control(stage) {
        terminal_controls::control_point(frame, SPIDER_CONTROLS.spec(control))
    } else {
        freecell_terminal::expected_control_point(frame, SharedStage::Score)
    }
}


/// Check only the expected control; this does not establish a win.
#[cfg(test)]
fn expected_control_ready(
    frame: &CapturedFrame,
    stage: TerminalStage,
) -> Result<bool, HaloDetectionError> {
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


    /// Per-game Play rectangles keep the two evidenced vertical layouts separate.
    #[test]
    fn play_layouts_use_their_games_measured_points() {
        let freecell = decode_png(include_bytes!("../tests/fixtures/freecell-FC12.png")).unwrap();
        assert_eq!(expected_control_point(&freecell, TerminalStage::Play).unwrap(), PixelPoint::new(0, 0));
        assert_eq!(freecell_terminal::expected_control_point(&freecell, SharedStage::Play).unwrap(), PixelPoint::new(680, 762));
        assert_eq!(freecell_terminal::expected_control_point(&fixture(18), SharedStage::Play).unwrap(), PixelPoint::new(0, 0));
        assert_eq!(expected_control_point(&fixture(18), TerminalStage::Play).unwrap(), PixelPoint::new(679, 862));
        assert_eq!(expected_control_point(&fixture(17), TerminalStage::NewGame).unwrap(), PixelPoint::new(863, 862));
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
        assert!(SPIDER_CONTROLS.spec(Control::Ok).bounds.right() <= frame.width);
        assert!(SPIDER_CONTROLS.spec(Control::Ok).bounds.bottom() <= frame.height);
        assert!(matches!(expected_control_point(&frame, TerminalStage::LevelUp),
            Err(HaloDetectionError::InvalidFrameLayout)));
    }


    /// The authorised rule examines only the supplied rectangle and exact RGB.
    #[test]
    fn missing_black_run_returns_zero_and_flat_black_is_not_win_authority() {
        let mut frame = fixture(20);
        let bounds = SPIDER_CONTROLS.spec(Control::Ok).bounds;
        for y in bounds.y..bounds.bottom() {
            for x in bounds.x..bounds.right() {
                let index = y as usize * frame.stride + x as usize * 4;
                frame.pixels[index..index + 4].copy_from_slice(&[1, 1, 1, 255]);
            }
        }
        assert_eq!(expected_control_point(&frame, TerminalStage::LevelUp).unwrap(), PixelPoint::new(0, 0));
        assert!(!expected_control_ready(&frame, TerminalStage::LevelUp).unwrap());
        assert_eq!(classify_win_entry(&frame).unwrap(), None);

        for pixel in frame.pixels.as_chunks_mut::<4>().0 {
            pixel.copy_from_slice(&[0, 0, 0, 255]);
        }
        assert_eq!(expected_control_point(&frame, TerminalStage::LevelUp).unwrap(),
            PixelPoint::new(bounds.x as i32, (bounds.bottom() - 1) as i32));
        assert_eq!(classify_win_entry(&frame).unwrap(), None);
    }
}
