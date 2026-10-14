//! Application-owned per-game bounds for expected terminal controls.
//!
//! Keep each game's calibration together. xSAR receives only the rectangle and
//! required run length; it has no game identity, control names or input policy.

use crate::{game::GameMode, geometry::PixelRect};

use super::{BLACK_RUN_LENGTH, Control};


/// Mathematical inputs for one horizontal exact-black run search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchSpec {
    /// Half-open rectangle in native full-frame coordinates.
    pub bounds: PixelRect,
    /// Consecutive exact-black pixels required on one row.
    pub run_length: u32,
}


/// One game's complete expected-control calibration block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameControlProfile {
    /// Level Up may move vertically with the level/rank artwork.
    pub ok: SearchSpec,
    /// New Game occupies a stable caption region for this game.
    pub new_game: SearchSpec,
    /// Play occupies a stable caption region for this game.
    pub play: SearchSpec,
}


impl GameControlProfile {
    /// Select mathematical search inputs without passing game policy to xSAR.
    pub const fn spec(self, control: Control) -> SearchSpec {
        match control {
            Control::Ok => self.ok,
            Control::NewGame => self.new_game,
            Control::Play => self.play,
        }
    }
}


/// Free Cell's expected control rectangles, maintained independently of other games.
pub const FREECELL_CONTROLS: GameControlProfile = GameControlProfile {
    ok: SearchSpec { bounds: PixelRect::new(930, 700, 25, 151), run_length: BLACK_RUN_LENGTH },
    new_game: SearchSpec { bounds: PixelRect::new(810, 831, 69, 49), run_length: BLACK_RUN_LENGTH },
    play: SearchSpec { bounds: PixelRect::new(675, 740, 20, 51), run_length: BLACK_RUN_LENGTH },
};


/// Spider's expected control rectangles, maintained independently of other games.
pub const SPIDER_CONTROLS: GameControlProfile = GameControlProfile {
    ok: SearchSpec { bounds: PixelRect::new(930, 700, 25, 151), run_length: BLACK_RUN_LENGTH },
    new_game: SearchSpec { bounds: PixelRect::new(810, 831, 69, 49), run_length: BLACK_RUN_LENGTH },
    play: SearchSpec { bounds: PixelRect::new(675, 840, 20, 51), run_length: BLACK_RUN_LENGTH },
};


/// Klondike's O stripe excludes the retained pointer in native K53.
pub const KLONDIKE_CONTROLS: GameControlProfile = GameControlProfile {
    ok: SearchSpec { bounds: PixelRect::new(930, 700, 25, 151), run_length: BLACK_RUN_LENGTH },
    new_game: SearchSpec { bounds: PixelRect::new(810, 831, 69, 49), run_length: BLACK_RUN_LENGTH },
    play: SearchSpec { bounds: PixelRect::new(675, 840, 20, 51), run_length: BLACK_RUN_LENGTH },
};


/// TriPeaks has native OK evidence; New Game and Play still need live acceptance.
pub const TRIPEAKS_CONTROLS: GameControlProfile = GameControlProfile {
    ok: SearchSpec { bounds: PixelRect::new(930, 700, 25, 151), run_length: BLACK_RUN_LENGTH },
    new_game: SearchSpec { bounds: PixelRect::new(810, 831, 69, 49), run_length: BLACK_RUN_LENGTH },
    play: SearchSpec { bounds: PixelRect::new(675, 885, 20, 56), run_length: BLACK_RUN_LENGTH },
};


/// Pyramid's inherited menu calibration still requires native/live acceptance.
pub const PYRAMID_CONTROLS: GameControlProfile = GameControlProfile {
    ok: SearchSpec { bounds: PixelRect::new(930, 700, 25, 151), run_length: BLACK_RUN_LENGTH },
    new_game: SearchSpec { bounds: PixelRect::new(810, 831, 69, 49), run_length: BLACK_RUN_LENGTH },
    play: SearchSpec { bounds: PixelRect::new(675, 885, 20, 56), run_length: BLACK_RUN_LENGTH },
};


/// Choose the application calibration before invoking a mathematical search.
pub const fn for_game(game: GameMode) -> &'static GameControlProfile {
    match game {
        GameMode::FreeCell => &FREECELL_CONTROLS,
        GameMode::Spider => &SPIDER_CONTROLS,
        GameMode::Klondike => &KLONDIKE_CONTROLS,
        GameMode::TriPeaks => &TRIPEAKS_CONTROLS,
        GameMode::Pyramid => &PYRAMID_CONTROLS,
    }
}
