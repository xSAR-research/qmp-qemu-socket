//! Complete-word recognition inside the fixed native challenge banner.
//!
//! Every letter and its native antialiasing participates in one bounded intensity
//! comparison. Rank-name pixels are not interpreted. Existing literal Boards and
//! Score labels take precedence; a separate Boards-word veto prevents a damaged
//! literal Boards label from becoming Card evidence. A separate strict Boards
//! branch requires every letter core, ordered complete words and native negative
//! space; it never promotes the weaker veto. This module grants no input.

use super::{
    LABEL_TOLERANCE,
    digits::{BLUE_FLOOR, CORE_MINIMUM, CORE_SPREAD},
};
use crate::{geometry::PixelRect, pyramid_progress::HeaderPatch};
use xsar::image_matching::FrameView;


/// Entire measured objective word band, excluding the changing numeric counter.
const SEARCH: PixelRect = PixelRect::new(814, 48, 224, 32);


/// Integer scale for exact ratio comparison; rounded diagnostic values are not used.
const SCALE: u64 = 1_000_000;


/// Absolute shared-word limit in ppm; exposed positive maximum is 187,190 ppm.
///
/// The retained Score-word negative has minimum distance 537,810 ppm. Every
/// letter additionally needs its own measured strong-white core, and both word
/// boundaries must be blank. These finite measurements do not claim OCR accuracy.
const WORD_LIMIT: u64 = 210_000;


/// One unscaled word crop; the rank name is outside this finite reference.
struct Word {
    /// Native width in pixels; every word uses the complete 32-pixel band height.
    width: u32,
    /// Complete blue-minus-background intensities in row-major native order.
    ink: &'static [u8],
    /// One strong-white point per letter, measured across the exposed positives.
    anchors: &'static [(u32, u32)],
    /// Required cores; the reserved-word veto tolerates one corrupt point but grants no input.
    required_cores: usize,
}


/// Genuine Sixes-frame word crop; the text “Sixes” is deliberately excluded.
const CLEARED: Word = Word {
    width: 84,
    ink: include_bytes!("../../../tests/fixtures/pyramid-card-cleared/cleared.ink"),
    anchors: &[
        (4, 18),
        (16, 15),
        (20, 18),
        (38, 18),
        (47, 16),
        (55, 18),
        (76, 16),
    ],
    required_cores: 7,
};


/// Genuine reserved word, used only as a veto and never as weaker Boards authority.
const BOARDS: Word = Word {
    width: 80,
    ink: include_bytes!("../../../tests/fixtures/pyramid-card-cleared/boards.ink"),
    anchors: &[(4, 16), (19, 18), (38, 18), (46, 16), (52, 17), (71, 18)],
    required_cores: 5,
};


/// Complete six-letter Boards authority; the five-core value above remains veto-only.
const COMPLETE_BOARDS: Word = Word {
    required_cores: 6,
    ..BOARDS
};


/// Original banner borders are independent of the preceding rank word and numbers.
const BANNER: [HeaderPatch; 2] = [
    HeaderPatch {
        bounds: PixelRect::new(814, 41, 292, 7),
        rgb: include_bytes!("../../../tests/fixtures/pyramid-card-cleared/banner-top.rgb"),
    },
    HeaderPatch {
        bounds: PixelRect::new(814, 80, 292, 7),
        rgb: include_bytes!("../../../tests/fixtures/pyramid-card-cleared/banner-bottom.rgb"),
    },
];


/// Compare every native border pixel except its six background-blended corner pixels.
/// The two rounded corners each contain three antialiased pixels mixed with
/// the board artwork. All remaining2038pixels per strip keep tolerance9 and
/// fixed geometry. Complete words and numeric counter checks stay separate.
/// Returns checked-bounds or malformed-reference errors; no input is granted.
fn banner_matches(view: FrameView<'_>) -> Result<bool, String> {
    for (strip, patch) in BANNER.into_iter().enumerate() {
        view.checked_bounds(patch.bounds)
            .map_err(|_| "banner outside frame")?;
        let (pixels, remainder) = patch.rgb.as_chunks::<3>();
        let width = usize::try_from(patch.bounds.width).map_err(|_| "banner width overflow")?;
        if !remainder.is_empty() || pixels.len() != width * 7 || width != 292 {
            return Err("invalid native banner reference".to_owned());
        }
        for (index, expected) in pixels.iter().enumerate() {
            let dx = index % width;
            let dy = index / width;
            let edge_row = if strip == 0 { dy } else { 6 - dy };
            if (edge_row == 0 && (dx < 2 || dx >= width - 2))
                || (edge_row == 1 && (dx == 0 || dx == width - 1))
            {
                continue;
            }
            let x = patch.bounds.x + u32::try_from(dx).map_err(|_| "banner x overflow")?;
            let y = patch.bounds.y + u32::try_from(dy).map_err(|_| "banner y overflow")?;
            let actual = view.pixel_rgb(x, y).ok_or("banner pixel outside frame")?;
            if actual
                .into_iter()
                .zip(*expected)
                .any(|(a, b)| a.abs_diff(b) > LABEL_TOLERANCE)
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}


/// Require the native banner and complete shared word, excluding the reserved Boards word.
///
/// # Errors
/// Invalid frame bounds or bundled RGB metadata returns an error. Unknown words,
/// incomplete letters and malformed private intensity metadata return false.
pub(super) fn observe(view: FrameView<'_>) -> Result<bool, String> {
    if !banner_matches(view)? {
        return Ok(false);
    }
    Ok(matches(view, &CLEARED).unwrap_or(false)
        && !matches(view, &BOARDS).unwrap_or(true)
        && !boards_fragment(view).unwrap_or(true))
}


/// Veto a damaged reserved Boards prefix when five complete letter shapes still survive.
///
/// Native letter columns are measured from the unchanged Boards ink canvas.
/// This uses the existing word-distance bound independently per surviving letter
/// and requires its neutral-white core. It grants no objective or input authority.
/// Invalid bounds/metadata return None; at most 145 native origins are inspected.
fn boards_fragment(view: FrameView<'_>) -> Option<bool> {
    let Some(cleared) = origin(view, &CLEARED)? else {
        return Some(false);
    };
    let Some(last) = cleared.checked_sub(BOARDS.width) else {
        return Some(false);
    };
    let columns = [0_usize, 16, 30, 43, 51, 65, 80];
    for x in SEARCH.x..=last {
        let mut matched = 0;
        for (letter, bounds) in columns.windows(2).enumerate() {
            let (dx, dy) = *BOARDS.anchors.get(letter)?;
            if !strong(view.pixel_rgb(x.checked_add(dx)?, SEARCH.y.checked_add(dy)?)?) {
                continue;
            }
            let mut difference = 0_u64;
            let mut total = 0_u64;
            for row in 0..32_usize {
                for column in bounds[0]..bounds[1] {
                    let expected = *BOARDS.ink.get(row.checked_mul(80)?.checked_add(column)?)?;
                    let actual = view.pixel_rgb(
                        x.checked_add(u32::try_from(column).ok()?)?,
                        SEARCH.y.checked_add(u32::try_from(row).ok()?)?,
                    )?[2]
                        .saturating_sub(BLUE_FLOOR);
                    difference = difference.checked_add(u64::from(actual.abs_diff(expected)))?;
                    total = total.checked_add(u64::from(actual.max(expected)))?;
                }
            }
            if total != 0 && difference.checked_mul(SCALE)? <= WORD_LIMIT.checked_mul(total)? {
                matched += 1;
            }
        }
        if matched >= BOARDS.required_cores {
            return Some(true);
        }
    }
    Some(false)
}


/// Recognise complete ordered Boards/cleared words without an extra raster appearance bank.
///
/// All thirteen measured letter cores, full-word distances and blank boundaries
/// must match. Existing native banner and whole-label background pixels retain
/// their original RGB tolerance, including red-only negative-space corruption.
/// Invalid frame or bundled template metadata returns an error; incomplete words
/// or inconsistent background return false. Counter parsing remains independent.
pub(super) fn observe_boards(view: FrameView<'_>) -> Result<bool, String> {
    if !banner_matches(view)? {
        return Ok(false);
    }
    let Some(boards) = origin(view, &COMPLETE_BOARDS).flatten() else {
        return Ok(false);
    };
    let Some(cleared) = origin(view, &CLEARED).flatten() else {
        return Ok(false);
    };
    if boards
        .checked_add(COMPLETE_BOARDS.width)
        .is_none_or(|end| end > cleared)
    {
        return Ok(false);
    }
    let reference = super::templates::LABELS
        .iter()
        .find(|label| label.kind == super::ChallengeKind::Boards)
        .ok_or("Missing native Boards background reference")?;
    view.checked_bounds(reference.patch.bounds)
        .map_err(|_| "Boards background outside frame")?;
    let (pixels, remainder) = reference.patch.rgb.as_chunks::<3>();
    if !remainder.is_empty() {
        return Err("Invalid native Boards background samples".to_owned());
    }
    let width = usize::try_from(reference.patch.bounds.width)
        .map_err(|_| "Invalid native Boards background width")?;
    if width == 0 {
        return Err("Empty native Boards background width".to_owned());
    }
    let height = usize::try_from(reference.patch.bounds.height)
        .map_err(|_| "Invalid Boards background height")?;
    if Some(pixels.len()) != width.checked_mul(height) {
        return Err("Incomplete native Boards background samples".to_owned());
    }
    let mut selected = 0_usize;
    for (index, expected) in pixels.iter().enumerate() {
        let x = reference.patch.bounds.x
            + u32::try_from(index % width).map_err(|_| "Boards background x overflow")?;
        let y = reference.patch.bounds.y
            + u32::try_from(index / width).map_err(|_| "Boards background y overflow")?;
        // The complete literal reference also contains genuine weak letter
        // fringes absent from the shared Sixes-word mask. They belong to ink,
        // not background. All complete-word and core limits stay unchanged.
        let literal_ink = expected[2] > BLUE_FLOOR
            && [(&COMPLETE_BOARDS, boards), (&CLEARED, cleared)]
                .into_iter()
                .any(|(word, origin)| x.checked_sub(origin).is_some_and(|dx| dx < word.width));
        if literal_ink
            || ink_at(&COMPLETE_BOARDS, boards, x, y).ok_or("Invalid Boards word mask")?
            || ink_at(&CLEARED, cleared, x, y).ok_or("Invalid cleared word mask")?
        {
            continue;
        }
        let actual = view
            .pixel_rgb(x, y)
            .ok_or("Boards background outside frame")?;
        if actual
            .into_iter()
            .zip(*expected)
            .any(|(actual, expected)| actual.abs_diff(expected) > LABEL_TOLERANCE)
        {
            return Ok(false);
        }
        selected += 1;
    }
    Ok(selected > 0)
}


/// Locate antialiased ink in one complete measured word; all other pixels are negative space.
fn ink_at(word: &Word, origin: u32, x: u32, y: u32) -> Option<bool> {
    let Some(column) = x.checked_sub(origin).filter(|column| *column < word.width) else {
        return Some(false);
    };
    let Some(row) = y.checked_sub(SEARCH.y).filter(|row| *row < SEARCH.height) else {
        return Some(false);
    };
    let width = usize::try_from(word.width).ok()?;
    let index = usize::try_from(row)
        .ok()?
        .checked_mul(width)?
        .checked_add(usize::try_from(column).ok()?)?;
    word.ink.get(index).map(|ink| *ink != 0)
}


/// Use the unchanged counter reader's measured neutral-white seed definition.
fn strong(rgb: [u8; 3]) -> bool {
    let minimum = rgb.iter().copied().min().unwrap_or(0);
    let maximum = rgb.iter().copied().max().unwrap_or(0);
    minimum >= CORE_MINIMUM && maximum - minimum < CORE_SPREAD
}


/// Search at most 145 native origins with constant scratch space and early core rejection.
///
/// Metadata/bounds failure returns None. Every full word intensity participates;
/// products are bounded by `224 * 32 * 255 * SCALE`, well inside u64.
fn matches(view: FrameView<'_>, word: &Word) -> Option<bool> {
    origin(view, word).map(|origin| origin.is_some())
}


/// Return the first bounded complete-word origin, distinguishing invalid data from no match.
fn origin(view: FrameView<'_>, word: &Word) -> Option<Option<u32>> {
    view.checked_bounds(SEARCH).ok()?;
    let width = usize::try_from(word.width).ok()?;
    if !(2..=224).contains(&width)
        || word.ink.len() != width.checked_mul(32)?
        || word.required_cores == 0
        || word.required_cores > word.anchors.len()
    {
        return None;
    }
    let last = SEARCH.right().checked_sub(word.width)?;
    for x in SEARCH.x..=last {
        let mut cores = 0;
        for &(dx, dy) in word.anchors {
            if dx >= word.width || dy >= SEARCH.height {
                return None;
            }
            if strong(view.pixel_rgb(x + dx, SEARCH.y + dy)?) {
                cores += 1;
            }
        }
        if cores < word.required_cores || !boundaries(view, x, word.width)? {
            continue;
        }
        let mut difference = 0_u64;
        let mut total = 0_u64;
        for (index, &expected) in word.ink.iter().enumerate() {
            let dx = u32::try_from(index % width).ok()?;
            let dy = u32::try_from(index / width).ok()?;
            let rgb = view.pixel_rgb(x + dx, SEARCH.y + dy)?;
            let actual = rgb.get(2)?.saturating_sub(BLUE_FLOOR);
            difference = difference.checked_add(u64::from(actual.abs_diff(expected)))?;
            total = total.checked_add(u64::from(actual.max(expected)))?;
        }
        if total != 0 && difference * SCALE <= WORD_LIMIT * total {
            return Some(Some(x));
        }
    }
    Some(None)
}


/// Require two blank native columns at each side, rejecting adjacent joined text.
fn boundaries(view: FrameView<'_>, x: u32, width: u32) -> Option<bool> {
    for dx in [0, 1, width.checked_sub(2)?, width.checked_sub(1)?] {
        for y in SEARCH.y..SEARCH.bottom() {
            if strong(view.pixel_rgb(x + dx, y)?) {
                return Some(false);
            }
        }
    }
    Some(true)
}


#[cfg(test)]
mod tests;
