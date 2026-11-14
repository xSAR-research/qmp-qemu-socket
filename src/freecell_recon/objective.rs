//! Application-owned objective provenance within the existing descriptive source label.
//!
//! Visible challenge content is retained as unresolved evidence. It never selects a
//! full-board search or grants playback authority; the xSAR deal schema stays unchanged.

use super::{FailureKind, Place, RecognitionError, Region, failure};
use crate::{capture::CapturedFrame, geometry::PixelRect};
use xsar::games::freecell::{InitialDeal, Metadata};


/// Complete variable objective area, including the retained native red banner's edges.
pub(crate) const BOUNDS: PixelRect = PixelRect::new(800, 33, 320, 62);
/// Explicit complete-board provenance written only after a blank native objective proof.
pub(crate) const FULL_BOARD_SOURCE: &str = "single-frame-tcp-v3;objective=full-board";
/// Visible objective content whose semantic target and completion policy are unsupported.
pub(crate) const UNRESOLVED_SOURCE: &str = "single-frame-tcp-v3;objective=unresolved";


/// Acquisition objective classification, separate from the recognised card inventory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Objective {
    /// Calibrated ordinary board with no visible objective banner.
    FullBoard,
    /// Objective pixels are present; no target, route or completion semantics are inferred.
    Unresolved,
    /// Legacy or malformed provenance cannot establish a planning objective.
    Unverified,
}


impl Objective {
    /// Explain the objective without presenting unread text as recognised semantics.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::FullBoard => "full board",
            Self::Unresolved => "unresolved visible challenge objective",
            Self::Unverified => "unverified objective provenance",
        }
    }
}


/// Permit a bounded basename reference without interpreting it as a host pathname.
fn evidence_token(token: &str) -> bool {
    !token.is_empty()
        && token.len() <= 70
        && token != "."
        && token != ".."
        && token
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
}


/// Read only the closed provenance grammar; absent or lost markers remain unverified.
pub(crate) fn from_metadata(metadata: &Metadata) -> Objective {
    let source = metadata.source();
    let base = if let Some((base, token)) = source.split_once(";evidence=") {
        if !evidence_token(token) {
            return Objective::Unverified;
        }
        base
    } else {
        source
    };
    match base {
        FULL_BOARD_SOURCE => Objective::FullBoard,
        UNRESOLVED_SOURCE => Objective::Unresolved,
        _ => Objective::Unverified,
    }
}


/// Require retained acquisition provenance, allowing the approved opaque challenge policy.
/// Full-board-first planning does not infer challenge text; live authority needs fresh proofs.
///
/// # Errors
/// Missing or malformed acquisition markers require a fresh RECON export.
pub(crate) fn require_acquired(metadata: &Metadata) -> Result<(), String> {
    match from_metadata(metadata) {
        Objective::FullBoard | Objective::Unresolved => Ok(()),
        Objective::Unverified => Err("Free Cell objective provenance is absent or unverified; reacquire and re-export this deal with RECON before CALCULATE or PLAY".to_owned()),
    }
}


/// Classify only the bounded objective pixels; never capture, read text or require scene conformity.
pub(super) fn observe(
    frame: &CapturedFrame,
    reference: &CapturedFrame,
) -> Result<Objective, RecognitionError> {
    let bad = |cause: String| {
        failure(
            Place::Scene,
            Region::Objective,
            Some(BOUNDS),
            FailureKind::InvalidFrame,
            cause,
        )
    };
    let view = frame.as_view().map_err(|e| bad(e.to_string()))?;
    let expected = reference.as_view().map_err(|e| bad(e.to_string()))?;
    view.checked_bounds(BOUNDS)
        .map_err(|e| bad(e.to_string()))?;
    expected
        .checked_bounds(BOUNDS)
        .map_err(|e| bad(e.to_string()))?;
    let mut changed = 0_u32;
    for y in BOUNDS.y..BOUNDS.y + BOUNDS.height {
        for x in BOUNDS.x..BOUNDS.x + BOUNDS.width {
            changed += u32::from(view.pixel_rgb(x, y) != expected.pixel_rgb(x, y));
        }
    }
    let objective = if changed == 0 {
        Objective::FullBoard
    } else {
        Objective::Unresolved
    };
    crate::single_step_telemetry::record(|| {
        serde_json::json!({
            "event":"recognition-region", "region":"objective", "bounds":[BOUNDS.x,BOUNDS.y,BOUNDS.width,BOUNDS.height],
            "classification":objective.label(), "changed_rgb_pixels":changed,
            "semantic_goal_recognised":objective == Objective::FullBoard,
        })
    });
    Ok(objective)
}


/// Preserve descriptive identity while replacing only application-owned objective provenance.
pub(super) fn metadata(metadata: &Metadata, objective: Objective) -> Result<Metadata, String> {
    let source = match objective {
        Objective::FullBoard => FULL_BOARD_SOURCE,
        Objective::Unresolved => UNRESOLVED_SOURCE,
        Objective::Unverified => return Err("Cannot export unverified objective provenance".into()),
    };
    Metadata::new(metadata.deal_id(), metadata.profile(), source).map_err(|e| e.to_string())
}


/// Bind saved acquisition pixels by a bounded packet basename while preserving the deal schema.
///
/// The reference is descriptive evidence only. Failure leaves the caller's original deal intact.
pub(crate) fn attach_evidence(deal: &InitialDeal, token: &str) -> Result<InitialDeal, String> {
    if !evidence_token(token) {
        return Err("Invalid Free Cell acquisition evidence token".into());
    }
    let objective = from_metadata(deal.metadata());
    let metadata = metadata(deal.metadata(), objective)?;
    let metadata = Metadata::new(
        metadata.deal_id(),
        metadata.profile(),
        format!("{};evidence={token}", metadata.source()),
    )
    .map_err(|e| e.to_string())?;
    InitialDeal::new(deal.board().clone(), metadata).map_err(|e| e.to_string())
}


#[cfg(test)]
mod tests {
    use super::*;


    /// Only exact closed source markers survive restart through the unchanged strict codec.
    #[test]
    fn freecell_313_objective_provenance_refuses_lost_or_forged_markers() {
        for source in [
            "single-frame-tcp-v3",
            "unit",
            "",
            "single-frame-tcp-v3;objective=full-board;extra=yes",
            "single-frame-tcp-v3;objective=full-board;evidence=..",
            "single-frame-tcp-v3;objective=full-board;evidence=a;evidence=b",
        ] {
            if let Ok(metadata) = Metadata::new("test", "synthetic", source) {
                assert_eq!(from_metadata(&metadata), Objective::Unverified);
                assert!(require_acquired(&metadata).is_err());
            }
        }
        let normal = Metadata::new("test", "synthetic", FULL_BOARD_SOURCE).unwrap();
        assert_eq!(from_metadata(&normal), Objective::FullBoard);
        require_acquired(&normal).unwrap();
        let unresolved = Metadata::new("test", "synthetic", UNRESOLVED_SOURCE).unwrap();
        assert_eq!(from_metadata(&unresolved), Objective::Unresolved);
        require_acquired(&unresolved).unwrap();
    }
}
