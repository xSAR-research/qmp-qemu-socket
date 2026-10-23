//! Rank-independent shared-word recognition inside the fixed native challenge banner.
//!
//! Every letter and its native antialiasing participates in one bounded intensity
//! comparison. Rank-name pixels are not interpreted. Existing literal Boards and
//! Score labels take precedence; a separate Boards-word veto prevents a damaged
//! literal Boards label from becoming Card evidence. This module grants no input.

use super::{
    LABEL_TOLERANCE,
    digits::{BLUE_FLOOR, CORE_MINIMUM, CORE_SPREAD},
};
use crate::{
    geometry::PixelRect,
    pyramid_progress::{HeaderPatch, matches_patch_with_tolerance},
};
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


/// Require the native banner and complete shared word, excluding the reserved Boards word.
///
/// # Errors
/// Invalid frame bounds or bundled RGB metadata returns an error. Unknown words,
/// incomplete letters and malformed private intensity metadata return false.
pub(super) fn observe(view: FrameView<'_>) -> Result<bool, String> {
    for patch in BANNER {
        if !matches_patch_with_tolerance(view, patch, LABEL_TOLERANCE)? {
            return Ok(false);
        }
    }
    Ok(matches(view, &CLEARED).unwrap_or(false) && !matches(view, &BOARDS).unwrap_or(true))
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
            return Some(true);
        }
    }
    Some(false)
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
