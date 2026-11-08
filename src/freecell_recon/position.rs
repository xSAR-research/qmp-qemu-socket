//! Same-run physical position proofs for independent Free Cell transfers.
//!
//! Initial binding recognises all52 cards independently. Later proofs carry only
//! identities hidden by measured overlapping strips from that authenticated model;
//! every visible complete rank/suit and each exposed card is checked again. No
//! canonical equality, deck elimination or predicted fan spacing supplies pixels.

use super::*;
use crate::geometry::{PixelPoint, PixelRect};
use xsar::games::freecell::rules::{Pile, Position};


/// Physical suit decorations H,C,D,S mapped to canonical C,D,H,S model indices.
const SCREEN_SUITS: [usize; 4] = [2, 0, 1, 3];
/// Last supported lower tableau pixel, above the toolbar.
const TABLEAU_BOTTOM: u32 = 947;


/// Measured physical click locations supplied only by a successful position proof.
#[derive(Debug)]
pub(crate) struct Geometry {
    /// Exposed source or destination centres; empty columns have positive outlines.
    play: [PixelPoint; 8],
}


impl Geometry {
    /// Map a checked physical pile; canonical foundations use observed screen order.
    pub(crate) fn point(&self, pile: Pile) -> Result<PixelPoint, String> {
        match pile {
            Pile::Cascade(n) => self.play.get(usize::from(n)).copied(),
            Pile::Cell(n) => CELL_X
                .get(usize::from(n))
                .map(|x| PixelPoint::new((*x + 68) as i32, 203)),
            Pile::Foundation(n) => SCREEN_SUITS
                .iter()
                .position(|s| *s == usize::from(n))
                .and_then(|slot| FOUNDATION_X.get(slot))
                .map(|x| PixelPoint::new((*x + 68) as i32, 203)),
        }
        .ok_or_else(|| "Invalid Free Cell physical pile".to_owned())
    }
}


/// Reuse exactly the initial reader's independently labelled44-strip glyph bank.
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


/// Require measured coverage without masking or sampling a failed edge pixel.
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


/// Native neutral edge, independent of card artwork or expected identity.
fn neutral([r, g, b]: [u8; 3], low: u8, spread: u8) -> bool {
    r.min(g).min(b) >= low && r.max(g).max(b) <= 240 && r.max(g).max(b) - r.min(g).min(b) <= spread
}


/// Positive green playing surface; missing card edges alone never establish empty.
fn felt([r, g, b]: [u8; 3]) -> bool {
    g >= 65 && i16::from(g) - i16::from(r) >= 25 && i16::from(g) - i16::from(b) >= 20
}


/// Scan measured top borders, including compressed/nonuniform native fans.
fn seams(frame: &CapturedFrame, x: u32) -> Result<Vec<u32>, String> {
    let mut result = Vec::new();
    let mut previous = false;
    for y in 331..TABLEAU_BOTTOM - 8 {
        let hit = all(frame, PixelRect::new(x + 8, y, 121, 1), |rgb| {
            neutral(rgb, 180, 10)
        })? && all(frame, PixelRect::new(x, y + 8, 1, 1), |rgb| {
            neutral(rgb, 100, 20)
        })? && all(frame, PixelRect::new(x + 136, y + 8, 1, 1), |rgb| {
            neutral(rgb, 100, 20)
        })?;
        if hit && !previous {
            result.push(y);
        }
        previous = hit;
    }
    if result.len() > 20 {
        return Err("More than20 measured cascade cards".to_owned());
    }
    Ok(result)
}


/// Prove complete final face extent and both outer rails before using its centre.
fn face(frame: &CapturedFrame, x: u32, y: u32) -> Result<u32, String> {
    if !all(frame, PixelRect::new(x + 8, y, 121, 1), |rgb| {
        neutral(rgb, 180, 10)
    })? {
        return Err("Complete native card top edge unavailable".to_owned());
    }
    for offset in [183, 182, 181] {
        let bottom = y.checked_add(offset).ok_or("Face extent overflow")?;
        if bottom + 3 >= TABLEAU_BOTTOM {
            continue;
        }
        if all(frame, PixelRect::new(x + 8, bottom, 121, 1), |rgb| {
            neutral(rgb, 100, 10)
        })? && all(frame, PixelRect::new(x, y + 8, 1, offset - 15), |rgb| {
            neutral(rgb, 100, 20)
        })? && all(
            frame,
            PixelRect::new(x + 136, y + 8, 1, offset - 15),
            |rgb| neutral(rgb, 100, 20),
        )? {
            return Ok(bottom);
        }
    }
    Err("Complete native card lower edge/rails unavailable".to_owned())
}


/// Occupied top slots use one coherent native face at nominal x or its measured left rounding.
/// CELL3/4 begin one pixel left of their nominal empty-slot/calibrated click anchors.
fn occupied_top(frame: &CapturedFrame, bank: &[Template], x: u32, id: u8) -> Result<(), String> {
    for origin in [Some(x), x.checked_sub(1)].into_iter().flatten() {
        if face(frame, origin, 112).is_ok() && identity(frame, bank, origin, 112, id, 183).is_ok() {
            return Ok(());
        }
    }
    Err("Complete occupied top-slot face/identity unavailable".to_owned())
}


/// A positive four-sided empty PLAY outline and its felt interior, measured natively.
fn empty_play(frame: &CapturedFrame, x: u32) -> Result<(), String> {
    let green = |[r, g, b]: [u8; 3]| {
        r >= 50
            && g >= 140
            && b >= 100
            && i16::from(g) - i16::from(r) >= 50
            && i16::from(g) - i16::from(b) >= 20
    };
    for rect in [
        PixelRect::new(x + 20, 332, 96, 3),
        PixelRect::new(x + 20, 507, 96, 3),
        PixelRect::new(x + 1, 350, 3, 140),
        PixelRect::new(x + 130, 350, 3, 140),
    ] {
        if !coverage(frame, rect, green, 995)? {
            return Err("Positive empty PLAY outline unavailable".to_owned());
        }
    }
    if !all(frame, PixelRect::new(x + 8, 340, 120, 160), felt)? {
        return Err("Empty PLAY interior unavailable".to_owned());
    }
    if !all(
        frame,
        PixelRect::new(x + 8, 513, 120, TABLEAU_BOTTOM - 513),
        felt,
    )? {
        return Err("Material below empty PLAY outline".to_owned());
    }
    Ok(())
}


/// Compare every complete visible glyph against all distinct labels, never against only expected.
fn identity(
    frame: &CapturedFrame,
    bank: &[Template],
    x: u32,
    y: u32,
    id: u8,
    visible: u32,
) -> Result<(), String> {
    if id >= 52 {
        return Err("Invalid expected card identity".to_owned());
    }
    if visible >= 30 {
        let ranks = if visible >= 31 {
            glyph_variants(frame, x, y, true)?
        } else {
            vec![glyph(frame, x, y, true)?]
        };
        if ranks
            .first()
            .is_none_or(|g| g.red != matches!(id / 13, 1 | 2))
        {
            return Err("Visible rank colour contradicts physical identity".to_owned());
        }
        let rank = identify(&ranks, bank, |t| (t.card.rank(), &t.rank), 48, 20)
            .map_err(|e| e.to_string())?;
        if rank.value() != id % 13 + 1 {
            return Err("Visible rank differs from physical position".to_owned());
        }
    }
    if visible >= 46 {
        let ranks = glyph_variants(frame, x, y, true)?;
        let suits = if visible >= 47 {
            glyph_variants(frame, x, y, false)?
        } else {
            vec![glyph(frame, x, y, false)?]
        };
        if ranks.first().map(|g| g.red) != suits.first().map(|g| g.red) {
            return Err("Visible rank/suit colour disagreement".to_owned());
        }
        let suit = identify(&suits, bank, |t| (t.card.suit(), &t.suit), 16, 12)
            .map_err(|e| e.to_string())?;
        let index = match suit {
            Suit::Clubs => 0,
            Suit::Diamonds => 1,
            Suit::Hearts => 2,
            Suit::Spades => 3,
        };
        if index != id / 13 {
            return Err("Visible suit differs from physical position".to_owned());
        }
    }
    Ok(())
}


/// Prove this exact physical expected position from supplied fresh native pixels.
/// Hidden identities require a caller-owned initial binding and legal observed history.
pub(crate) fn verify(
    frame: &CapturedFrame,
    expected: &Position,
    check: &mut impl FnMut() -> Result<(), String>,
) -> Result<Geometry, String> {
    crate::game_operations::validate_native_frame(frame).map_err(|e| e.to_string())?;
    expected.validate().map_err(|e| e.to_string())?;
    check()?;
    let reference = decode_png(REFERENCE).map_err(|e| e.to_string())?;
    same_region(
        frame,
        &reference,
        PixelRect::new(825, 36, 270, 50),
        Place::Scene,
        Region::Header,
        FailureKind::UnsupportedScene,
    )
    .map_err(|e| e.to_string())?;
    let bank = bank(&reference)?;
    let mut geometry = Geometry {
        play: [PixelPoint::new(0, 0); 8],
    };
    for (column, &x) in PLAY_X.iter().enumerate() {
        check()?;
        let ids = expected.column(column).ok_or("Invalid expected cascade")?;
        let tops = seams(frame, x)?;
        if (!tops.is_empty() && tops.first() != Some(&331)) || tops.len() != ids.len() {
            return Err(format!(
                "PLAY {} measured {} cards, expected {}",
                column + 1,
                tops.len(),
                ids.len()
            ));
        }
        let point = if let Some(&last) = tops.last() {
            let bottom = face(frame, x, last)?;
            // Native drop shadow extends four rows below the border.
            if bottom + 5 < TABLEAU_BOTTOM - 5
                && !all(
                    frame,
                    PixelRect::new(x + 8, bottom + 5, 121, TABLEAU_BOTTOM - 5 - bottom - 5),
                    felt,
                )?
            {
                return Err(format!("PLAY {} material below final face", column + 1));
            }
            for (depth, (&top, &id)) in tops.iter().zip(ids).enumerate() {
                check()?;
                let visible = tops.get(depth + 1).copied().unwrap_or(bottom + 1) - top;
                if depth + 1 < tops.len() && visible > 16 {
                    for xx in [x, x + 136] {
                        if !all(frame, PixelRect::new(xx, top + 8, 1, visible - 16), |rgb| {
                            neutral(rgb, 100, 20)
                        })? {
                            return Err(format!(
                                "PLAY {} depth {} interrupted rail",
                                column + 1,
                                depth + 1
                            ));
                        }
                    }
                }
                identity(frame, &bank, x, top, id, visible)
                    .map_err(|e| format!("PLAY {} depth {}: {e}", column + 1, depth + 1))?;
            }
            PixelPoint::new((x + 68) as i32, (last + 80) as i32)
        } else {
            empty_play(frame, x)?;
            PixelPoint::new((x + 67) as i32, 421)
        };
        *geometry
            .play
            .get_mut(column)
            .ok_or("Invalid geometry column")? = point;
    }
    for (slot, (&x, id)) in CELL_X.iter().zip(expected.cells()).enumerate() {
        check()?;
        if let Some(id) = id {
            occupied_top(frame, &bank, x, id).map_err(|e| format!("CELL {}: {e}", slot + 1))?;
        } else {
            same_region(
                frame,
                &reference,
                PixelRect::new(x, 112, CARD_WIDTH, 182),
                Place::Cell(slot),
                Region::TopSlot,
                FailureKind::TopSlotNotEmpty,
            )
            .map_err(|e| e.to_string())?;
        }
    }
    for (slot, (&x, &suit)) in FOUNDATION_X.iter().zip(&SCREEN_SUITS).enumerate() {
        check()?;
        let rank = *expected
            .foundations()
            .get(suit)
            .ok_or("Invalid foundation")?;
        if rank > 0 {
            occupied_top(frame, &bank, x, (suit as u8) * 13 + rank - 1)
                .map_err(|e| format!("SUIT {}: {e}", slot + 1))?;
        } else {
            same_region(
                frame,
                &reference,
                PixelRect::new(x, 112, CARD_WIDTH, 182),
                Place::Foundation(slot),
                Region::TopSlot,
                FailureKind::TopSlotNotEmpty,
            )
            .map_err(|e| e.to_string())?;
        }
    }
    check()?;
    Ok(geometry)
}


#[cfg(test)]
mod tests;
