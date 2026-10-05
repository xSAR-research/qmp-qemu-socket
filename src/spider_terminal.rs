//! Spider's one-board win and deterministic ordered restart controls.
//!
//! SP15/SP17 evidence the shared score-skip and New Game controls. SP18 places
//! Play 100 pixels below Free Cell's location. Level Up is optional and retains
//! the existing local OK control. No Solve, missing DRAW or missing HALO is win
//! authority. Shared local caption contrast excludes level and panel artwork.

use std::fmt;

use crate::{
    capture::CapturedFrame,
    detector::HaloDetectionError,
    freecell_terminal::{self, TerminalStage as SharedStage},
    geometry::PixelPoint,
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


/// Check only the expected local caption/control at its evidenced location.
pub fn expected_control_ready(
    frame: &CapturedFrame,
    stage: TerminalStage,
) -> Result<bool, HaloDetectionError> {


    if stage == TerminalStage::Play {
        return freecell_terminal::play_control_ready_at(frame, 100);
    }
    freecell_terminal::expected_control_ready(frame, shared_stage(stage))
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


/// Fixed native click points inside the evidenced controls; no timing inferred.
pub const fn click_point(stage: TerminalStage) -> PixelPoint {


    match stage {
        TerminalStage::Play => PixelPoint::new(709, 861),
        _ => freecell_terminal::click_point(shared_stage(stage)),
    }
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
            (17, TerminalStage::NewGame), (18, TerminalStage::Play)] {
            let frame = fixture(number);


            for stage in [TerminalStage::Score, TerminalStage::LevelUp,
                TerminalStage::NewGame, TerminalStage::Play] {
                assert_eq!(expected_control_ready(&frame, stage).unwrap(), stage == expected,
                    "SP{number:02} / {stage}");
            }
            assert_eq!(classify_win_entry(&frame).unwrap(),
                (expected != TerminalStage::Play).then_some(expected));
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
}
