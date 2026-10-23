//! Fixed-capacity connected components and exact-ratio native decimal matching.
//!
//! The largest counter region is 142×32 pixels. Twenty components, each on a
//! native 20×26 canvas, bound scratch storage and work. There is no resizing,
//! nearest-only acceptance, inferred character or general-purpose OCR fallback.

use super::templates::{CHARACTERS, GLYPHS};
use crate::geometry::PixelRect;
use xsar::image_matching::FrameView;


/// Largest measured counter width, in native pixels.
const MAX_WIDTH: usize = 142;


/// Fixed measured counter-band height, in native pixels.
const HEIGHT: usize = 32;


/// Upper bound for all pixels in either supported counter region.
const MAX_PIXELS: usize = MAX_WIDTH * HEIGHT;


/// Bound exceeding the fifteen symbols of two grouped six-digit numbers.
const MAX_SYMBOLS: usize = 20;


/// Unvisited strong-white component pixel, distinct from all bounded component IDs.
const UNVISITED: u8 = u8::MAX;


/// Native unscaled template canvas width, including two pixels of padding.
const CANVAS_WIDTH: usize = 20;


/// Native unscaled template canvas height, including padding.
const CANVAS_HEIGHT: usize = 26;


/// Complete fixed canvas length; every intensity participates in matching.
const CANVAS_PIXELS: usize = CANVAS_WIDTH * CANVAS_HEIGHT;


/// Blue background floor measured on these gold header backgrounds, in channel units.
pub(super) const BLUE_FLOOR: u8 = 6;


/// Strong neutral text seeds measured across all thirteen undimmed native sources.
pub(super) const CORE_MINIMUM: u8 = 150;


/// Maximum exclusive colour spread for neutral strong-white component seeds.
pub(super) const CORE_SPREAD: u8 = 70;


/// Conservative absolute normalized distance bound, in parts per million.
///
/// The worst cross-source same-label distance is 224,568 ppm rounded upward;
/// the closest different label is 327,696 ppm rounded upward. These are finite
/// exposed calibration measurements, not a general recognition-accuracy claim.
const DISTANCE_LIMIT: u64 = 240_000;


/// Required distinct-label margin, below the measured minimum 118,736 ppm.
const LABEL_MARGIN: u64 = 100_000;


/// Integer ratio scale; acceptance compares exact fractions before any rounding.
const SCALE: u64 = 1_000_000;


/// Bounded left-to-right classified symbols; no heap allocation is required.
pub(super) struct Symbols {
    /// Only the first `length` entries carry observed characters.
    values: [u8; MAX_SYMBOLS],
    /// Number of successfully classified distinct components.
    length: usize,
}


impl Symbols {
    /// Borrow only observed symbols, excluding unused fixed-capacity storage.
    pub(super) fn as_slice(&self) -> &[u8] {
        &self.values[..self.length]
    }
}


/// One measured strong-white component with a stable ID in the label buffer.
#[derive(Clone, Copy, Default)]
struct Component {
    /// Positive component ID assigned before sorting by horizontal position.
    id: u8,
    /// Inclusive minimum horizontal coordinate relative to the counter region.
    left: usize,
    /// Inclusive minimum vertical coordinate relative to the counter region.
    top: usize,
    /// Inclusive maximum horizontal coordinate relative to the counter region.
    right: usize,
    /// Inclusive maximum vertical coordinate relative to the counter region.
    bottom: usize,
}


/// Exact bounded normalized intensity distance; zero denominator is never constructed.
#[derive(Clone, Copy)]
struct Distance {
    /// Sum of absolute intensity differences over a complete native canvas.
    difference: u64,
    /// Sum of pairwise maximum intensities over the same canvas.
    total: u64,
}


impl Distance {
    /// Compare exact fractions; each product is bounded by `(520*255)^2`.
    fn less_than(self, other: Self) -> bool {
        self.difference * other.total < other.difference * self.total
    }
}


/// Read every bounded component, rejecting edge contact, unsupported marks or ambiguity.
pub(super) fn read(view: FrameView<'_>, bounds: PixelRect) -> Option<Symbols> {
    view.checked_bounds(bounds).ok()?;
    let width = usize::try_from(bounds.width).ok()?;
    let height = usize::try_from(bounds.height).ok()?;
    if width == 0 || width > MAX_WIDTH || height != HEIGHT {
        return None;
    }
    let length = width.checked_mul(height)?;
    let mut labels = [0_u8; MAX_PIXELS];
    for (index, label) in labels[..length].iter_mut().enumerate() {
        let rgb = view.pixel_rgb(
            bounds.x + (index % width) as u32,
            bounds.y + (index / width) as u32,
        )?;
        let minimum = *rgb.iter().min()?;
        let maximum = *rgb.iter().max()?;
        if minimum >= CORE_MINIMUM && maximum - minimum < CORE_SPREAD {
            *label = UNVISITED;
        }
    }
    let mut parts = [Component::default(); MAX_SYMBOLS];
    let count = components(&mut labels[..length], width, &mut parts)?;
    if count == 0 {
        return None;
    }
    parts[..count].sort_unstable_by_key(|part| part.left);
    let mut symbols = Symbols {
        values: [0; MAX_SYMBOLS],
        length: count,
    };
    for (output, part) in symbols.values[..count].iter_mut().zip(&parts[..count]) {
        let canvas = isolate(view, bounds, width, &labels[..length], *part)?;
        *output = classify(&canvas)?;
    }
    Some(symbols)
}


/// Flood eight-connected core pixels once each with a bounded explicit stack.
fn components(
    labels: &mut [u8],
    width: usize,
    parts: &mut [Component; MAX_SYMBOLS],
) -> Option<usize> {
    let mut stack = [0_usize; MAX_PIXELS];
    let mut count = 0;
    for index in 0..labels.len() {
        if labels[index] != UNVISITED {
            continue;
        }
        let part = parts.get_mut(count)?;
        count += 1;
        part.id = u8::try_from(count).ok()?;
        part.left = index % width;
        part.right = part.left;
        part.top = index / width;
        part.bottom = part.top;
        labels[index] = part.id;
        stack[0] = index;
        let mut pending = 1;
        while pending != 0 {
            pending -= 1;
            let point = stack[pending];
            let x = point % width;
            let y = point / width;
            part.left = part.left.min(x);
            part.right = part.right.max(x);
            part.top = part.top.min(y);
            part.bottom = part.bottom.max(y);
            for ny in y.saturating_sub(1)..=(y + 1).min(HEIGHT - 1) {
                for nx in x.saturating_sub(1)..=(x + 1).min(width - 1) {
                    let next = ny * width + nx;
                    if labels[next] == UNVISITED {
                        labels[next] = part.id;
                        *stack.get_mut(pending)? = next;
                        pending += 1;
                    }
                }
            }
        }
        if part.left == 0 || part.top == 0 || part.right + 1 == width || part.bottom + 1 == HEIGHT {
            return None;
        }
    }
    Some(count)
}


/// Copy the exact tight native glyph and its antialiasing, excluding other core components.
fn isolate(
    view: FrameView<'_>,
    bounds: PixelRect,
    width: usize,
    labels: &[u8],
    part: Component,
) -> Option<[u8; CANVAS_PIXELS]> {
    let glyph_width = part.right.checked_sub(part.left)?.checked_add(1)?;
    let glyph_height = part.bottom.checked_sub(part.top)?.checked_add(1)?;
    if glyph_width > 16 || glyph_height > 22 {
        return None;
    }
    let mut canvas = [0; CANVAS_PIXELS];
    for dy in 0..glyph_height {
        for dx in 0..glyph_width {
            let x = part.left + dx;
            let y = part.top + dy;
            let label = labels[y * width + x];
            if label != 0 && label != part.id {
                continue;
            }
            let rgb = view.pixel_rgb(bounds.x + x as u32, bounds.y + y as u32)?;
            canvas[(dy + 2) * CANVAS_WIDTH + dx + 2] = rgb[2].saturating_sub(BLUE_FLOOR);
        }
    }
    Some(canvas)
}


/// Return one exclusive label only after the absolute and distinct-label exact-ratio gates.
fn classify(canvas: &[u8; CANVAS_PIXELS]) -> Option<u8> {
    let mut distances = [None::<Distance>; 12];
    for (&character, sample) in CHARACTERS.iter().zip(GLYPHS.as_chunks::<CANVAS_PIXELS>().0) {
        let index = character_index(character)?;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let distance = distance(canvas, sample, dx, dy)?;
                if distances[index].is_none_or(|previous| distance.less_than(previous)) {
                    distances[index] = Some(distance);
                }
            }
        }
    }
    let mut best = None::<(usize, Distance)>;
    for (index, &distance) in distances.iter().enumerate() {
        let distance = distance?;
        if best.is_none_or(|(_, previous)| distance.less_than(previous)) {
            best = Some((index, distance));
        }
    }
    let (index, winner) = best?;
    if winner.difference * SCALE > DISTANCE_LIMIT * winner.total {
        return None;
    }
    for (other, distance) in distances.into_iter().enumerate() {
        if other == index {
            continue;
        }
        let distance = distance?;
        let numerator =
            (distance.difference * winner.total).checked_sub(winner.difference * distance.total)?;
        if numerator * SCALE < LABEL_MARGIN * distance.total * winner.total {
            return None;
        }
    }
    Some(match index {
        0..=9 => b'0' + index as u8,
        10 => b'/',
        11 => b',',
        _ => return None,
    })
}


/// Map only the finite decimal alphabet; a malformed bundled label remains a refusal.
fn character_index(character: u8) -> Option<usize> {
    match character {
        b'0'..=b'9' => Some(usize::from(character - b'0')),
        b'/' => Some(10),
        b',' => Some(11),
        _ => None,
    }
}


/// Compare every native intensity at a bounded one-pixel translation without resizing.
fn distance(
    canvas: &[u8; CANVAS_PIXELS],
    sample: &[u8; CANVAS_PIXELS],
    dx: i32,
    dy: i32,
) -> Option<Distance> {
    let mut difference = 0_u64;
    let mut total = 0_u64;
    for y in 0..CANVAS_HEIGHT {
        for x in 0..CANVAS_WIDTH {
            let sx = x as i32 + dx;
            let sy = y as i32 + dy;
            let expected = if (0..CANVAS_WIDTH as i32).contains(&sx)
                && (0..CANVAS_HEIGHT as i32).contains(&sy)
            {
                sample[sy as usize * CANVAS_WIDTH + sx as usize]
            } else {
                0
            };
            let actual = canvas[y * CANVAS_WIDTH + x];
            difference = difference.checked_add(u64::from(actual.abs_diff(expected)))?;
            total = total.checked_add(u64::from(actual.max(expected)))?;
        }
    }
    (total != 0).then_some(Distance { difference, total })
}
