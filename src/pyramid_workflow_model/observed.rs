//! Advisory mode facts from existing native pixels; never an input or completion permit.

use super::RouteMethod;
use crate::{
    capture::CapturedFrame,
    pyramid_progress::{self, BoardCount, ChallengeKind, ChallengeProgress},
};


/// Actual measured header facts, kept separate from the operator's stopping policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum ObservedGame {
    /// No uniquely supported header was observed; never interpreted as normal or zero.
    #[default]
    Unknown,
    /// Literal normal board ordinal plus the supported normal-game footer.
    Normal(BoardCount),
    /// Literal challenge kind, progress and target; rank remains unknown except Kings.
    Challenge {
        /// Independently recognised actual requirement.
        progress: ChallengeProgress,
        /// Advisory positive Kings label; false does not identify another rank.
        kings: bool,
    },
}


/// Actual 132 headers retain partial progress and never convert a numeric target into completion.
#[cfg(test)]
#[test]
fn execution_observability_observed_headers_preserve_actual_progress_and_unknown_rank() {
    let load = |name: &str| {
        crate::capture::decode_png(
            &std::fs::read(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/pyramid-execution-observability")
                    .join(name),
            )
            .unwrap(),
        )
        .unwrap()
    };
    let board = ObservedGame::observe(&load("boards-two.png")).unwrap();
    assert!(
        matches!(board, ObservedGame::Challenge {progress, kings:false}
        if progress == ChallengeProgress::new(ChallengeKind::Boards, 2, 3).unwrap())
    );
    assert!(board.label().contains("2/3"));
    let score = ObservedGame::observe(&load("score-final.png")).unwrap();
    assert!(score.label().contains("2510/2900"));
    assert!(!score.label().contains("target reached"));
    let reached = ObservedGame::Challenge {
        progress: ChallengeProgress::new(ChallengeKind::Cards, 12, 12).unwrap(),
        kings: false,
    };
    assert!(reached.label().contains("rank not identified"));
    assert!(reached.label().contains("completion unconfirmed"));
    assert_eq!(ObservedGame::default(), ObservedGame::Unknown);
    let mut malformed = load("boards-two.png");
    malformed.pixels.clear();
    assert!(ObservedGame::observe(&malformed).is_err());
}


impl ObservedGame {
    /// Select this RECON's objective only for a positively identified challenge.
    /// Score and Card challenges use Maximum; Boards uses Shortest. Normal and
    /// unknown headers leave the operator's selection intact and grant no input.
    pub(crate) fn route_method_override(self) -> Option<RouteMethod> {
        match self {
            Self::Challenge { progress, .. } => Some(match progress.kind {
                ChallengeKind::Score | ChallengeKind::Cards => RouteMethod::Fast,
                ChallengeKind::Boards => RouteMethod::FewestActions,
            }),
            Self::Normal(_) | Self::Unknown => None,
        }
    }


    /// Read supported header facts from an already captured frame without capture or input.
    /// Invalid pixels/metadata return an error; unsupported or conflicting headers stay unknown.
    pub(crate) fn observe(frame: &CapturedFrame) -> Result<Self, String> {
        let challenge = pyramid_progress::observe_challenge(frame)?;
        let normal = pyramid_progress::observe_normal(frame)?;
        match (challenge, normal) {
            (Some(progress), None) => Ok(Self::Challenge {
                progress,
                kings: progress.kind == ChallengeKind::Cards
                    && pyramid_progress::challenge::is_kings_header(frame)?,
            }),
            (None, Some(progress)) => Ok(Self::Normal(progress)),
            _ => Ok(Self::Unknown),
        }
    }


    /// Display observed facts only; no selected policy or inferred completion is included.
    pub(crate) fn label(self) -> String {
        match self {
            Self::Unknown => "Unknown or unsupported header".to_owned(),
            Self::Normal(progress) => format!(
                "Normal game: board {} of {} ({} completed)",
                u16::from(progress.completed) + 1,
                progress.target,
                progress.completed
            ),
            Self::Challenge { progress, kings } => {
                let kind = match progress.kind {
                    ChallengeKind::Boards => "Boards challenge",
                    ChallengeKind::Score => "Score challenge",
                    ChallengeKind::Cards if kings => "Kings Card challenge",
                    ChallengeKind::Cards => "Card challenge (rank not identified)",
                };
                format!(
                    "{kind}: {}/{}{}",
                    progress.current,
                    progress.target,
                    if progress.target_reached() {
                        " (numeric target reached; completion unconfirmed)"
                    } else {
                        ""
                    }
                )
            }
        }
    }
}


/// All supported challenge kinds override RECON; ordinary or unknown headers do not.
#[cfg(test)]
#[test]
fn recon_route_objective_policy_uses_challenge_kind_without_rank_guessing() {
    for kind in [
        ChallengeKind::Score,
        ChallengeKind::Cards,
        ChallengeKind::Boards,
    ] {
        for kings in [false, true] {
            let observed = ObservedGame::Challenge {
                progress: ChallengeProgress::new(kind, 0, 3).unwrap(),
                kings,
            };
            assert_eq!(
                observed.route_method_override(),
                Some(match kind {
                    ChallengeKind::Score | ChallengeKind::Cards => RouteMethod::Fast,
                    ChallengeKind::Boards => RouteMethod::FewestActions,
                })
            );
        }
    }
    assert_eq!(ObservedGame::Unknown.route_method_override(), None);
    assert_eq!(
        ObservedGame::Normal(BoardCount {
            completed: 0,
            target: 3
        })
        .route_method_override(),
        None
    );
}
