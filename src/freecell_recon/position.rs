//! Fresh printed-corner proofs for physically indexed Free Cell transfers.
//!
//! RECON52 and legal observed history own hidden identities. This reader locates
//! fully readable exposed rank/suit corners without counting hidden cards or
//! requiring decorative card outlines, rails, paper colours or full faces.

use super::*;
use crate::game_controls::{PlayCorner as Corner, scan_freecell_play_corners};
#[cfg(test)]
use crate::game_controls::{corner_candidate, corner_identity};
use crate::geometry::PixelPoint;
#[cfg(test)]
use crate::geometry::PixelRect;
use xsar::games::freecell::rules::{Pile, Position, card_id};


/// Physical foundation slots H,C,D,S mapped to canonical C,D,H,S model indices.
const SCREEN_SUITS: [usize; 4] = [2, 0, 1, 3];
/// First native tableau top, independent of subsequent compressed fan spacing.
const TABLEAU_TOP: u32 = 331;


/// Native pointer x inset from a proved corner or positively outlined slot origin.
const PICKUP_X_INSET: u32 = 15;


/// Native pointer y inset shared by occupied and empty source/destination anchors.
const PICKUP_Y_INSET: u32 = 20;


/// Authenticated click points, populated only for the piles requested in this proof.
#[derive(Debug)]
pub(crate) struct Geometry {
    /// Physical PLAY corners, or equally inset positively outlined empty slots.
    play: [Option<PixelPoint>; 8],
    /// Physical CELL corners, or equally inset positively outlined empty slots.
    cells: [Option<PixelPoint>; 4],
    /// Canonical C,D,H,S corners, or equally inset positively outlined empty slots.
    foundations: [Option<PixelPoint>; 4],
}


impl Geometry {
    /// Return a requested pile's authenticated point; invalid/unproved piles fail.
    pub(crate) fn point(&self, pile: Pile) -> Result<PixelPoint, String> {
        let point = match pile {
            Pile::Cascade(n) => self.play.get(usize::from(n)),
            Pile::Cell(n) => self.cells.get(usize::from(n)),
            Pile::Foundation(n) => self.foundations.get(usize::from(n)),
        };
        point
            .copied()
            .flatten()
            .ok_or_else(|| "Invalid or unverified Free Cell physical pile".to_owned())
    }
}


/// Prove an empty upper slot from its four strokes and absence of native card paper.
pub(super) fn empty_top(frame: &CapturedFrame, x: u32) -> Result<(), String> {
    crate::game_controls::freecell_empty_slot_error(frame, x, 112)
        .map_err(|error| error.to_string())?
}


/// Prove an empty PLAY slot without comparing surrounding felt or decoration.
fn empty_play(frame: &CapturedFrame, x: u32) -> Result<(), String> {
    crate::game_controls::freecell_empty_slot_error(frame, x, TABLEAU_TOP)
        .map_err(|error| error.to_string())?
}


/// Observe one printed corner identity against all complete foreground families.
fn observed_identity(frame: &CapturedFrame, x: u32, y: u32) -> Result<u8, String> {
    match crate::game_controls::classify_corner(frame, (x, y)).map_err(|error| error.to_string())? {
        CornerClassification::Card(card) => Ok(card_id(card)),
        CornerClassification::Ambiguous { detail } => Err(format!("AmbiguousIdentity: {detail}")),
        CornerClassification::ColourDisagreement { detail } => {
            Err(format!("Visible rank/suit colour disagreement: {detail}"))
        }
        CornerClassification::PartialLabel { detail, .. }
        | CornerClassification::Unreadable { detail } => Err(detail),
    }
}


/// Scan bounded native origins via shared `game_controls`; hidden cards and fan pitch are deliberately unmeasured.
fn corners(
    frame: &CapturedFrame,
    x: u32,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<Vec<Corner>, String> {
    scan_freecell_play_corners(frame, x, check)
}


/// Prove the bottommost supported printed corner and its expected exposed identity.
fn occupied_play(
    frame: &CapturedFrame,
    x: u32,
    expected: u8,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<PixelPoint, String> {
    let observed = corners(frame, x, check)?;
    let last = observed
        .last()
        .ok_or("Exposed printed corner unavailable")?;
    let id = last.id.ok_or_else(|| {
        format!(
            "Unreadable exposed printed corner: {}",
            last.failure.as_deref().unwrap_or("no identity")
        )
    })?;
    if id != expected {
        return Err(format!(
            "Exposed identity differs from physical position: observed {id} at native y{}, expected {expected}",
            last.top()
        ));
    }
    if observed
        .iter()
        .filter(|corner| corner.id == Some(id))
        .count()
        != 1
    {
        return Err("Duplicate exposed identity in physical column".to_owned());
    }
    Ok(PixelPoint::new(
        (x + PICKUP_X_INSET) as i32,
        (last.top() + PICKUP_Y_INSET) as i32,
    ))
}


/// Independently read one upper-slot corner at the two calibrated native x origins.
fn occupied_top(frame: &CapturedFrame, x: u32, expected: u8) -> Result<PixelPoint, String> {
    let mut found = None;
    for origin in [Some(x), x.checked_sub(1)].into_iter().flatten() {
        if let Ok(id) = observed_identity(frame, origin, 112) {
            if found.is_some_and(|(_, previous)| previous != id) {
                return Err("Ambiguous occupied top-slot identity".to_owned());
            }
            found.get_or_insert((origin, id));
        }
    }
    let (origin, id) = found.ok_or("Occupied top-slot printed identity unavailable")?;
    if id != expected {
        return Err("Occupied top-slot identity differs from physical position".to_owned());
    }
    Ok(PixelPoint::new(
        (origin + PICKUP_X_INSET) as i32,
        (112 + PICKUP_Y_INSET) as i32,
    ))
}


/// Prove only requested physical piles from fresh supported native scene pixels.
///
/// Hidden identities/counts come from caller-owned RECON52 and legal history. Each
/// requested occupied pile independently exposes the expected complete printed card;
/// each requested empty pile requires positive native empty-slot evidence.
///
/// # Errors
/// Refuses STOP/context changes, unsupported frames, invalid positions/piles, missing,
/// ambiguous, duplicate or contradictory exposed identities and unproved empty slots.
pub(crate) fn verify_piles(
    frame: &CapturedFrame,
    expected: &Position,
    piles: &[Pile],
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<Geometry, String> {
    crate::game_operations::validate_native_frame(frame).map_err(|e| e.to_string())?;
    expected.validate().map_err(|e| e.to_string())?;
    check()?;
    let reference = decode_png(REFERENCE).map_err(|e| e.to_string())?;
    supported_scene(frame, &reference, check).map_err(|e| e.to_string())?;
    let mut geometry = Geometry {
        play: [None; 8],
        cells: [None; 4],
        foundations: [None; 4],
    };
    for &pile in piles {
        check()?;
        let (slot, point) = match pile {
            Pile::Cascade(n) => {
                let column = usize::from(n);
                let &x = PLAY_X.get(column).ok_or("Invalid requested cascade")?;
                let ids = expected.column(column).ok_or("Invalid expected cascade")?;
                let point = if let Some(&id) = ids.last() {
                    occupied_play(frame, x, id, check)
                } else {
                    empty_play(frame, x).and_then(|()| {
                        if corners(frame, x, check)?.is_empty() {
                            Ok(PixelPoint::new(
                                (x + PICKUP_X_INSET) as i32,
                                (TABLEAU_TOP + PICKUP_Y_INSET) as i32,
                            ))
                        } else {
                            Err("Printed card interrupts empty physical column".to_owned())
                        }
                    })
                }
                .map_err(|cause| format!("PLAY {}: {cause}", column + 1))?;
                (geometry.play.get_mut(column), point)
            }
            Pile::Cell(n) => {
                let cell = usize::from(n);
                let &x = CELL_X.get(cell).ok_or("Invalid requested cell")?;
                let id = expected
                    .cells()
                    .get(cell)
                    .copied()
                    .ok_or("Invalid expected cell")?;
                let point = if let Some(id) = id {
                    occupied_top(frame, x, id)
                } else {
                    empty_top(frame, x).map(|()| {
                        PixelPoint::new((x + PICKUP_X_INSET) as i32, (112 + PICKUP_Y_INSET) as i32)
                    })
                }
                .map_err(|cause| format!("CELL {}: {cause}", cell + 1))?;
                (geometry.cells.get_mut(cell), point)
            }
            Pile::Foundation(n) => {
                let suit = usize::from(n);
                let screen = SCREEN_SUITS
                    .iter()
                    .position(|s| *s == suit)
                    .ok_or("Invalid requested foundation")?;
                let &x = FOUNDATION_X
                    .get(screen)
                    .ok_or("Invalid physical foundation")?;
                let &rank = expected
                    .foundations()
                    .get(suit)
                    .ok_or("Invalid expected foundation")?;
                let point = if rank > 0 {
                    occupied_top(frame, x, n * 13 + rank - 1)
                } else {
                    empty_top(frame, x).map(|()| {
                        PixelPoint::new((x + PICKUP_X_INSET) as i32, (112 + PICKUP_Y_INSET) as i32)
                    })
                }
                .map_err(|cause| format!("SUIT {}: {cause}", screen + 1))?;
                (geometry.foundations.get_mut(suit), point)
            }
        };
        *slot.ok_or("Invalid physical geometry slot")? = Some(point);
    }
    check()?;
    Ok(geometry)
}


/// Prove every exposed physical pile; hidden inventory still belongs to RECON52/history.
///
/// # Errors
/// Returns the same scene, cancellation, identity and empty-slot errors as `verify_piles`.
#[cfg(test)]
pub(crate) fn verify(
    frame: &CapturedFrame,
    expected: &Position,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<Geometry, String> {
    let piles: Vec<Pile> = (0..8)
        .map(Pile::Cascade)
        .chain((0..4).map(Pile::Cell))
        .chain((0..4).map(Pile::Foundation))
        .collect();
    verify_piles(frame, expected, &piles, check)
}


#[cfg(test)]
mod tests;
