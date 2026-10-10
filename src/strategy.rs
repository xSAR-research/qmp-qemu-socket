//! Solving strategy and immutable request identity, separate from game calibration.

use std::path::PathBuf;

use crate::game::GameMode;


/// User-selected controller; independent preparation does not grant general input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SolvingStrategy {
    /// Read-only Pyramid preparation; reconnaissance uses separate explicit authority.
    ShortestPath,
    /// Existing guest Solver/HALO controllers for all five game types.
    #[default]
    ComputerVision,
}


impl SolvingStrategy {


    /// Strategies displayed at startup and in the idle selector.
    pub const AVAILABLE: [Self; 2] = [Self::ShortestPath, Self::ComputerVision];


    /// Return the requested user-facing strategy name.
    pub const fn label(self) -> &'static str {


        match self {
            Self::ShortestPath => "Shortest path route calculation",
            Self::ComputerVision => "Computer Vision solving",
        }
    }


    /// Explain the selected method before strategy selection.
    pub const fn description(self) -> &'static str {


        match self {
            Self::ShortestPath => "Observe the environment, build a route model and calculate a path using Dijkstra or A*. Replan when new information reveals an impasse.",
            Self::ComputerVision => "Follow the game's built-in Solver highlights using screen captures and QMP input.",
        }
    }


    /// Whether ordinary execution requests may grant gameplay input, excluding reconnaissance.
    pub const fn permits_input(self) -> bool {
        matches!(self, Self::ComputerVision)
    }
}


/// Socket, game, strategy and selection generation attached to every worker request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControllerContext {
    /// Selected existing QMP Unix socket.
    pub socket_path: PathBuf,
    /// Game calibration, independent of solving strategy.
    pub game_mode: GameMode,
    /// Controller requested for this observation or run.
    pub strategy: SolvingStrategy,
    /// Incremented when selection or socket changes, including a change back.
    pub generation: u64,
}


impl ControllerContext {


    /// Construct one immutable request identity.
    pub fn new(
        socket_path: PathBuf,
        game_mode: GameMode,
        strategy: SolvingStrategy,
        generation: u64,
    ) -> Self {
        Self { socket_path, game_mode, strategy, generation }
    }


    /// Reject unsupported strategy/game combinations before connection or input.
    pub fn validate(&self) -> Result<(), String> {


        if self.strategy == SolvingStrategy::ShortestPath && self.game_mode != GameMode::Pyramid {
            return Err("Independent preparation currently supports Pyramid only".to_owned());
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    //! Capability and game-combination checks at the controller boundary.

    use super::*;


    /// Preparation stays Pyramid-only and cannot grant guest-input authority.
    #[test]
    fn preparation_is_pyramid_only_and_never_input_authority() {
        assert!(!SolvingStrategy::ShortestPath.permits_input());
        assert!(SolvingStrategy::ComputerVision.permits_input());


        for game in GameMode::AVAILABLE {
            let independent = ControllerContext::new(PathBuf::from("/tmp/qmp.sock"), game, SolvingStrategy::ShortestPath, 1);
            assert_eq!(independent.validate().is_ok(), game == GameMode::Pyramid);
            let vision = ControllerContext::new(independent.socket_path, game, SolvingStrategy::ComputerVision, 1);
            assert!(vision.validate().is_ok());
        }
    }
}
