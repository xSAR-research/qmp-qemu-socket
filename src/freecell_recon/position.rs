//! Fresh printed-corner proofs for physically indexed Free Cell transfers.
//!
//! RECON52 and legal observed history own hidden identities. This reader locates
//! fully readable exposed rank/suit corners without counting hidden cards or
//! requiring decorative card outlines, rails, paper colours or full faces.

use super::*;
use crate::geometry::{PixelPoint, PixelRect};
use xsar::games::freecell::rules::{Pile, Position, card_id};


/// Physical foundation slots H,C,D,S mapped to canonical C,D,H,S model indices.
const SCREEN_SUITS: [usize; 4] = [2, 0, 1, 3];
/// First native tableau top, independent of subsequent compressed fan spacing.
const TABLEAU_TOP: u32 = 331;
/// Latest possible printed-corner origin whose complete crop stays above toolbar947.
const LAST_CORNER_TOP: u32 = 900;
/// Maximum separation of neighbouring native ±1 recrops of one printed corner.
const RECROP_SEPARATION: u32 = 2;


/// Authenticated click points, populated only for the piles requested in this proof.
#[derive(Debug)]
pub(crate) struct Geometry {
    /// Physical PLAY corners, or positively outlined empty-slot centres.
    play: [Option<PixelPoint>; 8],
    /// Physical CELL corners, or positively outlined empty-slot centres.
    cells: [Option<PixelPoint>; 4],
    /// Canonical C,D,H,S corners, or positively outlined empty-slot centres.
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


/// Reuse the original labelled strips and supplementary native6 rank once per frame.
fn bank(reference: &CapturedFrame) -> Result<Vec<Template>, String> {
    let mut bank = Vec::new();
    for ((&x, labels), &length) in PLAY_X.iter().zip(LABELS).zip(INITIAL_LENGTHS.iter()) {
        for (depth, code) in labels.iter().enumerate().take(length - 1) {
            let y = *ROW_Y.get(depth).ok_or("Calibration depth invalid")?;
            bank.push(Template {
                card: label(code)?,
                rank: glyph(reference, x, y, true)?,
                suit: glyph(reference, x, y, false)?,
            });
        }
    }
    append_native_six_rank(&mut bank)?;
    Ok(bank)
}


/// Require every selected pixel; invalid bounds/storage is always an error.
fn all(
    frame: &CapturedFrame,
    rect: PixelRect,
    predicate: impl Fn([u8; 3]) -> bool,
) -> Result<bool, String> {
    let view = frame.as_view().map_err(|e| e.to_string())?;
    view.checked_bounds(rect).map_err(|e| e.to_string())?;
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            if !predicate(view.pixel_rgb(x, y).ok_or("Position pixel outside frame")?) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}


/// Require measured coverage without masking or sampling a failed empty-slot stroke.
fn coverage(
    frame: &CapturedFrame,
    rect: PixelRect,
    predicate: impl Fn([u8; 3]) -> bool,
    per_mille: u64,
) -> Result<bool, String> {
    let view = frame.as_view().map_err(|e| e.to_string())?;
    view.checked_bounds(rect).map_err(|e| e.to_string())?;
    let mut matching = 0_u64;
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            if predicate(view.pixel_rgb(x, y).ok_or("Missing coverage pixel")?) {
                matching += 1;
            }
        }
    }
    Ok(matching * 1000 >= u64::from(rect.width) * u64::from(rect.height) * per_mille)
}


/// Prove the four native empty-slot strokes, independently of interior canvas RGB.
fn empty_outline(frame: &CapturedFrame, x: u32, y: u32) -> Result<(), String> {
    let green = |[r, g, b]: [u8; 3]| {
        r >= 50
            && g >= 140
            && b >= 100
            && i16::from(g) - i16::from(r) >= 50
            && i16::from(g) - i16::from(b) >= 20
    };
    for rect in [
        PixelRect::new(x + 20, y + 1, 96, 3),
        PixelRect::new(x + 20, y + 176, 96, 3),
        PixelRect::new(x + 1, y + 19, 3, 140),
        PixelRect::new(x + 130, y + 19, 3, 140),
    ] {
        if !coverage(frame, rect, green, 995)? {
            return Err("Positive native empty-slot outline unavailable".to_owned());
        }
    }
    Ok(())
}


/// Refuse native bright card paper inside a positively outlined empty slot.
fn no_card_paper(frame: &CapturedFrame, rect: PixelRect) -> Result<(), String> {
    if !all(frame, rect, |[r, g, b]| r.min(g).min(b) < 245)? {
        return Err("Card paper interrupts empty-slot interior".to_owned());
    }
    Ok(())
}


/// Prove an empty upper slot from its four strokes and absence of native card paper.
pub(super) fn empty_top(frame: &CapturedFrame, x: u32) -> Result<(), String> {
    empty_outline(frame, x, 112)?;
    no_card_paper(frame, PixelRect::new(x + 8, 120, 120, 160))
}


/// Prove an empty PLAY slot without comparing surrounding felt or decoration.
fn empty_play(frame: &CapturedFrame, x: u32) -> Result<(), String> {
    empty_outline(frame, x, TABLEAU_TOP)?;
    no_card_paper(frame, PixelRect::new(x + 8, 340, 120, 160))
}


/// Independent label evidence, including an ambiguous but absolutely qualified shape.
struct LabelEvidence<T> {
    /// Unique label and colour, or the unchanged recognition refusal.
    value: Result<(T, bool), String>,
    /// Absolute shape quality passed; ambiguity still forbids assigning an identity.
    shape: bool,
}


impl<T> LabelEvidence<T> {
    /// Retain typed ambiguity as evidence of printed ink without accepting its label.
    fn matched(red: bool, matched: Result<T, MatchFailure>) -> Self {
        let shape = matched.is_ok()
            || matched
                .as_ref()
                .is_err_and(|cause| cause.kind == FailureKind::AmbiguousIdentity);
        Self {
            value: matched
                .map(|value| (value, red))
                .map_err(|cause| cause.to_string()),
            shape,
        }
    }


    /// Missing or incoherent ink cannot establish either a label or a printed shape.
    fn missing(cause: String) -> Self {
        Self {
            value: Err(cause),
            shape: false,
        }
    }
}


/// Independently compete a complete printed rank against all same-colour rank labels.
fn observed_rank(frame: &CapturedFrame, bank: &[Template], x: u32, y: u32) -> LabelEvidence<Rank> {
    let ranks = match glyph_variants_with_policy(frame, x, y, true, PaperPolicy::ReadOnlyRecon) {
        Ok(ranks) => ranks,
        Err(cause) => return LabelEvidence::missing(cause),
    };
    let Some(rank) = ranks.first() else {
        return LabelEvidence::missing("Missing rank crop".to_owned());
    };
    LabelEvidence::matched(
        rank.red,
        identify(&ranks, bank, |t| (t.card.rank(), &t.rank), 48, 20),
    )
}


/// Independently compete a complete printed suit against all same-colour suit labels.
fn observed_suit(frame: &CapturedFrame, bank: &[Template], x: u32, y: u32) -> LabelEvidence<Suit> {
    let suits = match glyph_variants_with_policy(frame, x, y, false, PaperPolicy::ReadOnlyRecon) {
        Ok(suits) => suits,
        Err(cause) => return LabelEvidence::missing(cause),
    };
    let Some(suit) = suits.first() else {
        return LabelEvidence::missing("Missing suit crop".to_owned());
    };
    LabelEvidence::matched(
        suit.red,
        identify(&suits, bank, |t| (t.card.suit(), &t.suit), 16, 12),
    )
}


/// Observe both complete labels against every same-colour label, never only expected.
fn observed_identity(
    frame: &CapturedFrame,
    bank: &[Template],
    x: u32,
    y: u32,
) -> Result<u8, String> {
    let (rank, rank_red) = observed_rank(frame, bank, x, y).value?;
    let (suit, suit_red) = observed_suit(frame, bank, x, y).value?;
    if rank_red != suit_red {
        return Err("Visible rank/suit colour disagreement".to_owned());
    }
    Ok(card_id(Card::new(rank, suit)))
}


/// Threshold one complete glyph region against its own90th-percentile background.
/// The native45% contrast predicate matches `ReadOnlyRecon`; colour/label acceptance
/// remains the independent reader's responsibility. Fixed arrays bound scan storage.
fn local_ink<const N: usize>(frame: &CapturedFrame, rect: PixelRect) -> Result<[bool; N], String> {
    let count = usize::try_from(rect.width)
        .ok()
        .and_then(|width| {
            usize::try_from(rect.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or("Glyph support area overflow")?;
    if count != N {
        return Err("Glyph support dimensions differ from native crop".to_owned());
    }
    let view = frame.as_view().map_err(|cause| cause.to_string())?;
    view.checked_bounds(rect)
        .map_err(|cause| cause.to_string())?;
    let mut samples = [0_u8; N];
    for (sample, (x, y)) in samples.iter_mut().zip(
        (rect.y..rect.y + rect.height)
            .flat_map(|y| (rect.x..rect.x + rect.width).map(move |x| (x, y))),
    ) {
        let [r, g, b] = view
            .pixel_rgb(x, y)
            .ok_or("Missing local glyph support pixel")?;
        *sample = r.min(g).min(b);
    }
    let mut background = samples;
    background.sort_unstable();
    let percentile = N.checked_mul(9).ok_or("Glyph percentile overflow")? / 10;
    let paper = u32::from(
        *background
            .get(percentile)
            .ok_or("Missing local glyph background")?,
    );
    Ok(samples.map(|value| 100 * paper.saturating_sub(u32::from(value)) > 45 * paper))
}


/// Locate printed-corner support cheaply before allocating the nine RGB recrops.
/// A quiet local top margin bounds the corner; no card border/paper colour is tested.
/// Partial/mixed glyphs remain candidates so failed lower identities cannot be skipped.
fn corner_candidate(frame: &CapturedFrame, x: u32, y: u32) -> Result<bool, String> {
    let view = frame.as_view().map_err(|e| e.to_string())?;
    view.checked_bounds(PixelRect::new(x + 2, y + 3, 26, 43))
        .map_err(|e| e.to_string())?;
    let minimum = |xx, yy| -> Result<u8, String> {
        let [r, g, b] = view.pixel_rgb(xx, yy).ok_or("Missing corner pixel")?;
        Ok(r.min(g).min(b))
    };
    let mut background = [0_u8; 80];
    for (slot, (xx, yy)) in background
        .iter_mut()
        .zip((y + 3..y + 7).flat_map(|yy| (x + 5..x + 25).map(move |xx| (xx, yy))))
    {
        *slot = minimum(xx, yy)?;
    }
    background.sort_unstable();
    let paper = u32::from(*background.get(72).ok_or("Missing corner background")?);
    let ink = |xx, yy| -> Result<bool, String> {
        Ok(100 * paper.saturating_sub(u32::from(minimum(xx, yy)?)) > 45 * paper)
    };
    let mut gutter_ink = 0;
    for yy in y + 3..y + 7 {
        for xx in x + 5..x + 25 {
            gutter_ink += usize::from(ink(xx, yy)?);
        }
    }
    if gutter_ink > 8 {
        return Ok(false);
    }
    let ranks = local_ink::<{ 26 * 23 }>(frame, PixelRect::new(x + 2, y + 7, 26, 23))?;
    let rank_ink = ranks.iter().filter(|ink| **ink).count();
    let rank_top = ranks.iter().take(26 * 5).filter(|ink| **ink).count();
    let rank_bottom = ranks
        .iter()
        .skip(26 * 17)
        .take(26 * 3)
        .filter(|ink| **ink)
        .count();
    let suits = local_ink::<{ 18 * 16 }>(frame, PixelRect::new(x + 5, y + 30, 18, 16))?;
    let suit_ink = suits.iter().filter(|ink| **ink).count();
    let rank = rank_ink >= 30 && rank_top >= 2 && rank_bottom >= 2;
    Ok(rank || suit_ink >= 20)
}


/// Retain a partial corner when an independently qualified label shape establishes
/// its layout. Ambiguous labels are blocking evidence only, never accepted identity.
/// Incidental ink without a qualified all-label shape supplies no printed corner.
fn corner_identity(
    frame: &CapturedFrame,
    bank: &[Template],
    x: u32,
    y: u32,
) -> Option<Result<u8, String>> {
    let rank = observed_rank(frame, bank, x, y);
    let suit = observed_suit(frame, bank, x, y);
    let printed_shape = rank.shape || suit.shape;
    match (rank.value, suit.value) {
        (Ok((rank, red)), Ok((suit, suit_red))) => Some(if red == suit_red {
            Ok(card_id(Card::new(rank, suit)))
        } else {
            Err("Visible rank/suit colour disagreement".to_owned())
        }),
        (Err(cause), Ok(_)) => Some(Err(format!("Unreadable printed rank: {cause}"))),
        (Ok(_), Err(cause)) => Some(Err(format!("Unreadable printed suit: {cause}"))),
        (Err(rank), Err(suit)) if printed_shape => Some(Err(format!(
            "Unreadable printed labels: rank {rank}; suit {suit}"
        ))),
        (Err(_), Err(_)) => None,
    }
}


/// Nearby recrops of one physical corner, including unreadable candidate evidence.
struct Corner {
    /// First candidate origin in this connected ±1 recrop cluster.
    first: u32,
    /// Last candidate origin in this cluster.
    last: u32,
    /// Independently observed identity, if any candidate passed all-label matching.
    id: Option<u8>,
    /// Last failed candidate diagnostic, retained if no crop supplies an identity.
    failure: Option<String>,
}


impl Corner {
    /// Merge one neighbouring crop, refusing contradictory identities in one corner.
    fn add(&mut self, y: u32, observed: Result<u8, String>) -> Result<(), String> {
        self.last = y;
        match observed {
            Ok(id) => {
                if self.id.is_some_and(|previous| previous != id) {
                    return Err("Ambiguous identities within one printed corner".to_owned());
                }
                self.id = Some(id);
            }
            Err(cause) => self.failure = Some(cause),
        }
        Ok(())
    }


    /// Centre the connected native recrop origins without claiming a measured card edge.
    fn top(&self) -> u32 {
        self.first + (self.last - self.first) / 2
    }
}


/// Scan bounded native origins; hidden cards and fan pitch are deliberately unmeasured.
fn corners(
    frame: &CapturedFrame,
    bank: &[Template],
    x: u32,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<Vec<Corner>, String> {
    let mut result: Vec<Corner> = Vec::new();
    for y in TABLEAU_TOP..=LAST_CORNER_TOP {
        if (y - TABLEAU_TOP).is_multiple_of(8) {
            check()?;
        }
        if !corner_candidate(frame, x, y)? {
            continue;
        }
        let Some(observed) = corner_identity(frame, bank, x, y) else {
            continue;
        };
        if let Some(previous) = result.last_mut()
            && y - previous.last <= RECROP_SEPARATION
        {
            previous.add(y, observed)?;
        } else {
            let mut corner = Corner {
                first: y,
                last: y,
                id: None,
                failure: None,
            };
            corner.add(y, observed)?;
            result.push(corner);
        }
    }
    check()?;
    Ok(result)
}


/// Prove the bottommost supported printed corner and its expected exposed identity.
fn occupied_play(
    frame: &CapturedFrame,
    bank: &[Template],
    x: u32,
    expected: u8,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<PixelPoint, String> {
    let observed = corners(frame, bank, x, check)?;
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
    Ok(PixelPoint::new((x + 15) as i32, (last.top() + 20) as i32))
}


/// Independently read one upper-slot corner at the two calibrated native x origins.
fn occupied_top(
    frame: &CapturedFrame,
    bank: &[Template],
    x: u32,
    expected: u8,
) -> Result<PixelPoint, String> {
    let mut found = None;
    for origin in [Some(x), x.checked_sub(1)].into_iter().flatten() {
        if let Ok(id) = observed_identity(frame, bank, origin, 112) {
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
    Ok(PixelPoint::new((origin + 15) as i32, 132))
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
    let bank = bank(&reference)?;
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
                    occupied_play(frame, &bank, x, id, check)
                } else {
                    empty_play(frame, x).and_then(|()| {
                        if corners(frame, &bank, x, check)?.is_empty() {
                            Ok(PixelPoint::new((x + 67) as i32, 421))
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
                    occupied_top(frame, &bank, x, id)
                } else {
                    empty_top(frame, x).map(|()| PixelPoint::new((x + 68) as i32, 203))
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
                    occupied_top(frame, &bank, x, n * 13 + rank - 1)
                } else {
                    empty_top(frame, x).map(|()| PixelPoint::new((x + 68) as i32, 203))
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
