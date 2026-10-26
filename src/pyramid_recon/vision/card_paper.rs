//! Bounded card-paper evidence for native white, cream and aged cream faces.
//!
//! These predicates describe card material inside caller-owned geometry. They
//! grant neither card identity nor scene/input authority and do not sample felt.

use super::{FrameView, PixelRect};


/// Occupied stock requires this percentage of light paper across its full footprint.
/// The retained white/cream card minimum is 13,883/25,993 pixels (53.41%);
/// 45% leaves an 8.41 percentage point margin for picture cards and selection.
/// The existing RGB floor 180 includes cream paper; black-card themes are unsupported.
/// Bounded aged material extends the original light-paper evidence.
pub(super) const STOCK_PAPER_PRESENT_PERCENT: u8 = 45;


/// Recognise the measured aged-cream material independently of absolute whiteness.
/// The bounded channel relationships exclude the retained orange wood and dark
/// card themes. Values are inclusive eight-bit RGB units; ink is not paper.
pub(super) fn aged_pixel([r, g, b]: [u8; 3]) -> bool {
    r.max(g).max(b) >= 180
        && r.min(g).min(b) >= 96
        && r.max(g).max(b) - r.min(g).min(b) <= 96
        && r.abs_diff(g) <= 48
        && g.abs_diff(b) <= 64
}


/// Count aged material in a complete checked rectangle, without granting presence.
/// The shared paper scan validates nonempty bounds and frame layout before the
/// traversal. Missing pixels or invalid geometry return an error. Absence callers
/// compare this count with their existing possible-paper percentage.
pub(super) fn aged_pixels(view: FrameView<'_>, bounds: PixelRect) -> Result<u64, String> {
    crate::image_evidence::count_paper(view, bounds, 180)?;
    let right = bounds
        .x
        .checked_add(bounds.width)
        .ok_or("aged paper x overflow")?;
    let bottom = bounds
        .y
        .checked_add(bounds.height)
        .ok_or("aged paper y overflow")?;
    let mut count = 0;
    for y in bounds.y..bottom {
        for x in bounds.x..right {
            if aged_pixel(view.pixel_rgb(x, y).ok_or("missing aged paper pixel")?) {
                count += 1;
            }
        }
    }
    Ok(count)
}


/// Estimate material from at least one third of the complete bounded corner.
/// Missing material remains unknown. The returned lower median uses the same
/// minimum-channel strength units as the existing white/cream ink reader.
pub(super) fn aged_reference(pixels: &[[u8; 3]]) -> Option<u8> {
    let mut values: Vec<_> = pixels
        .iter()
        .copied()
        .filter(|rgb| aged_pixel(*rgb))
        .map(|[r, g, b]| r.min(g).min(b))
        .collect();
    if values.is_empty() || values.len() < pixels.len() / 3 {
        return None;
    }
    values.sort_unstable();
    values.get((values.len() - 1) / 2).copied()
}


/// Return sufficient material evidence for the inclusive stock-presence threshold.
/// The bright fast path returns its light count; otherwise count the complete
/// light-or-aged union. Callers compare with the shared percentage, not exact union.
/// Returns an error on invalid frame geometry, empty/outside bounds or missing
/// pixels. The existing checker validates the complete rectangle before traversal.
pub(super) fn stock_pixels(view: FrameView<'_>, bounds: PixelRect) -> Result<u64, String> {
    let light = crate::image_evidence::count_paper(view, bounds, 180)?;
    let area = u64::from(bounds.width) * u64::from(bounds.height);
    if u128::from(light) * 100 >= u128::from(area) * u128::from(STOCK_PAPER_PRESENT_PERCENT) {
        return Ok(light);
    }
    let right = bounds
        .x
        .checked_add(bounds.width)
        .ok_or("stock paper x overflow")?;
    let bottom = bounds
        .y
        .checked_add(bounds.height)
        .ok_or("stock paper y overflow")?;
    let mut count = 0;
    for y in bounds.y..bottom {
        for x in bounds.x..right {
            let rgb = view.pixel_rgb(x, y).ok_or("missing stock paper pixel")?;
            if rgb.iter().all(|channel| *channel >= 180) || aged_pixel(rgb) {
                count += 1;
            }
        }
    }
    Ok(count)
}


/// Lower per-channel paper medians distinguish red ink from warm paper shading.
pub(super) fn aged_channels(pixels: &[[u8; 3]]) -> Option<[u8; 3]> {
    let paper: Vec<_> = pixels
        .iter()
        .copied()
        .filter(|pixel| aged_pixel(*pixel))
        .collect();
    if paper.is_empty() || paper.len() < pixels.len() / 3 {
        return None;
    }
    let mut result = [0; 3];
    for (channel, value) in result.iter_mut().enumerate() {
        let mut values: Vec<_> = paper.iter().map(|pixel| pixel[channel]).collect();
        values.sort_unstable();
        *value = values[(values.len() - 1) / 2];
    }
    Some(result)
}
