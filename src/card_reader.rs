use std::collections::VecDeque;

use thiserror::Error;

use crate::{
    capture::CapturedFrame,
    cards::{CardObservation, CardRank, CardRegions, CardSlot, CardVisibility},
    detector::{HaloDetectionError, has_gold_halo, pixel_rgb},
    geometry::PixelRect,
    parameters::{
        CARD_BACK_BRIGHT_CHANNEL_MINIMUM, CARD_BACK_BRIGHT_FRACTION_PER_MILLE,
        FACE_UP_LIGHT_FRACTION_PER_MILLE, MAXIMUM_TEMPLATE_DIMENSION_DIFFERENCE,
        MINIMUM_GLYPH_COMPONENT_PIXELS, MINIMUM_RANK_MARGIN_PER_MILLE,
        MINIMUM_RANK_SCORE_PER_MILLE, NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH,
        RANK_CHANNEL_THRESHOLD,
    },
};

/// A deterministic reader for the pinned 1920x1080 Solitaire calibration.
///
/// Rank recognition remains disabled until a labelled `Three` seed completes
/// the calibration set. Hidden/empty classification and halo detection are
/// usable now; a provisionally recognised face-up glyph returns an explicit
/// incomplete-calibration error, while missing or ambiguous glyphs have their
/// own errors.
#[derive(Clone, Copy, Debug, Default)]
pub struct CardReader;

impl CardReader {
    pub const fn new() -> Self {
        Self
    }

    pub fn read_card(
        &self,
        frame: &CapturedFrame,
        slot: &CardSlot,
    ) -> Result<CardObservation, CardReadError> {
        validate_frame(frame)?;
        validate_regions(frame, slot)?;

        let pixels = slot.regions().pixels();
        let has_halo = has_gold_halo(frame, pixels)?;
        let rank_bounds = pixels.rank_bounds;
        let light_fraction = fraction_per_mille(frame, rank_bounds, |rgb| {
            rgb.iter().all(|channel| *channel >= RANK_CHANNEL_THRESHOLD)
        })?;

        if light_fraction >= FACE_UP_LIGHT_FRACTION_PER_MILLE {
            let glyph =
                extract_glyph(frame, rank_bounds)?.ok_or(CardReadError::VisibleRankGlyphMissing)?;
            let _provisional_rank = recognise_rank(&glyph)?;
            return Err(CardReadError::RankCalibrationIncomplete {
                missing_rank: CardRank::Three,
            });
        }

        let bright_fraction = fraction_per_mille(frame, rank_bounds, |rgb| {
            rgb.iter()
                .copied()
                .max()
                .is_some_and(|channel| channel >= CARD_BACK_BRIGHT_CHANNEL_MINIMUM)
        })?;
        let visibility = if bright_fraction >= CARD_BACK_BRIGHT_FRACTION_PER_MILLE {
            CardVisibility::FaceDown
        } else {
            CardVisibility::Empty
        };

        Ok(CardObservation::new(
            *slot,
            visibility,
            CardRank::Unknown,
            has_halo,
        ))
    }
}

/// Read one calibrated card slot with the default deterministic reader.
pub fn read_card(frame: &CapturedFrame, slot: &CardSlot) -> Result<CardObservation, CardReadError> {
    CardReader::new().read_card(frame, slot)
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum CardReadError {
    #[error("captured frame has an invalid pixel layout")]
    InvalidFrameLayout,
    #[error(
        "captured frame is {actual_width}x{actual_height}; expected {expected_width}x{expected_height}"
    )]
    UnexpectedFrameDimensions {
        actual_width: u32,
        actual_height: u32,
        expected_width: u32,
        expected_height: u32,
    },
    #[error("{region} bounds must be non-empty and inside the captured frame")]
    InvalidRegion { region: &'static str },
    #[error("rank bounds must be contained by card bounds")]
    RankOutsideCard,
    #[error("card click point must be contained by card bounds")]
    ClickOutsideCard,
    #[error("card QMP geometry is inconsistent with its calibrated pixel geometry")]
    InconsistentQmpGeometry,
    #[error("rank bounds are too wide for the binary recogniser")]
    RankRegionTooWide,
    #[error(transparent)]
    Halo(#[from] HaloDetectionError),
    #[error("face-up card contains no qualifying rank glyph")]
    VisibleRankGlyphMissing,
    #[error("rank calibration is incomplete; missing a labelled {missing_rank:?} template")]
    RankCalibrationIncomplete { missing_rank: CardRank },
    #[error(
        "no confident rank match; best provisional match was {best_rank:?} at {score_per_mille}/1000"
    )]
    NoConfidentRankMatch {
        best_rank: CardRank,
        score_per_mille: u16,
    },
    #[error(
        "ambiguous rank match between {best_rank:?} and {second_rank:?}; margin was {margin_per_mille}/1000"
    )]
    AmbiguousRankMatch {
        best_rank: CardRank,
        second_rank: CardRank,
        margin_per_mille: u16,
    },
}

fn validate_frame(frame: &CapturedFrame) -> Result<(), CardReadError> {
    if !frame.is_layout_valid() {
        return Err(CardReadError::InvalidFrameLayout);
    }
    if frame.width != NOMINAL_FRAME_WIDTH || frame.height != NOMINAL_FRAME_HEIGHT {
        return Err(CardReadError::UnexpectedFrameDimensions {
            actual_width: frame.width,
            actual_height: frame.height,
            expected_width: NOMINAL_FRAME_WIDTH,
            expected_height: NOMINAL_FRAME_HEIGHT,
        });
    }
    Ok(())
}

fn validate_regions(frame: &CapturedFrame, slot: &CardSlot) -> Result<(), CardReadError> {
    let regions = slot.regions();
    let pixels = regions.pixels();
    checked_rect(frame, pixels.card_bounds, "card")?;
    checked_rect(frame, pixels.halo_bounds, "halo")?;
    checked_rect(frame, pixels.rank_bounds, "rank")?;

    if pixels.rank_bounds.width > 32 {
        return Err(CardReadError::RankRegionTooWide);
    }
    if !contains_rect(pixels.card_bounds, pixels.rank_bounds) {
        return Err(CardReadError::RankOutsideCard);
    }
    if !pixels.card_bounds.contains(pixels.click_point) {
        return Err(CardReadError::ClickOutsideCard);
    }
    let expected = CardRegions::from_pixels(pixels, frame.width, frame.height)
        .map_err(|_| CardReadError::InvalidRegion { region: "card" })?;
    if regions.qmp() != expected.qmp() {
        return Err(CardReadError::InconsistentQmpGeometry);
    }
    Ok(())
}

fn checked_rect(
    frame: &CapturedFrame,
    rect: PixelRect,
    region: &'static str,
) -> Result<(), CardReadError> {
    let Some(right) = rect.x.checked_add(rect.width) else {
        return Err(CardReadError::InvalidRegion { region });
    };
    let Some(bottom) = rect.y.checked_add(rect.height) else {
        return Err(CardReadError::InvalidRegion { region });
    };
    if rect.is_empty() || right > frame.width || bottom > frame.height {
        return Err(CardReadError::InvalidRegion { region });
    }
    Ok(())
}

fn contains_rect(outer: PixelRect, inner: PixelRect) -> bool {
    let Some(outer_right) = outer.x.checked_add(outer.width) else {
        return false;
    };
    let Some(outer_bottom) = outer.y.checked_add(outer.height) else {
        return false;
    };
    let Some(inner_right) = inner.x.checked_add(inner.width) else {
        return false;
    };
    let Some(inner_bottom) = inner.y.checked_add(inner.height) else {
        return false;
    };

    inner.x >= outer.x
        && inner.y >= outer.y
        && inner_right <= outer_right
        && inner_bottom <= outer_bottom
}

fn fraction_per_mille(
    frame: &CapturedFrame,
    rect: PixelRect,
    predicate: impl Fn([u8; 3]) -> bool,
) -> Result<u32, CardReadError> {
    let mut matching = 0_u32;
    let total = rect.width.saturating_mul(rect.height);
    for y in rect.y..rect.bottom() {
        for x in rect.x..rect.right() {
            let rgb = pixel_rgb(frame, x, y).ok_or(CardReadError::InvalidFrameLayout)?;
            if predicate(rgb) {
                matching += 1;
            }
        }
    }
    Ok(matching.saturating_mul(1_000) / total)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BinaryGlyph {
    width: u8,
    height: u8,
    rows: Vec<u32>,
}

fn extract_glyph(
    frame: &CapturedFrame,
    rank_bounds: PixelRect,
) -> Result<Option<BinaryGlyph>, CardReadError> {
    let width = rank_bounds.width as usize;
    let height = rank_bounds.height as usize;
    let mut foreground = vec![false; width * height];
    for local_y in 0..height {
        for local_x in 0..width {
            let rgb = pixel_rgb(
                frame,
                rank_bounds.x + local_x as u32,
                rank_bounds.y + local_y as u32,
            )
            .ok_or(CardReadError::InvalidFrameLayout)?;
            foreground[local_y * width + local_x] = rgb
                .iter()
                .copied()
                .min()
                .is_some_and(|value| value < RANK_CHANNEL_THRESHOLD);
        }
    }

    let mut visited = vec![false; foreground.len()];
    let mut retained = vec![false; foreground.len()];
    for start in 0..foreground.len() {
        if !foreground[start] || visited[start] {
            continue;
        }

        let mut component = Vec::new();
        let mut queue = VecDeque::from([start]);
        visited[start] = true;
        while let Some(index) = queue.pop_front() {
            component.push(index);
            let x = index % width;
            let y = index / width;
            for delta_y in -1_i32..=1 {
                for delta_x in -1_i32..=1 {
                    if delta_x == 0 && delta_y == 0 {
                        continue;
                    }
                    let neighbour_x = x as i32 + delta_x;
                    let neighbour_y = y as i32 + delta_y;
                    if neighbour_x < 0
                        || neighbour_y < 0
                        || neighbour_x >= width as i32
                        || neighbour_y >= height as i32
                    {
                        continue;
                    }
                    let neighbour = neighbour_y as usize * width + neighbour_x as usize;
                    if foreground[neighbour] && !visited[neighbour] {
                        visited[neighbour] = true;
                        queue.push_back(neighbour);
                    }
                }
            }
        }

        if component.len() >= MINIMUM_GLYPH_COMPONENT_PIXELS {
            for index in component {
                retained[index] = true;
            }
        }
    }

    let Some(first) = retained.iter().position(|pixel| *pixel) else {
        return Ok(None);
    };
    let mut left = first % width;
    let mut right = left;
    let mut top = first / width;
    let mut bottom = top;
    for (index, present) in retained.iter().copied().enumerate() {
        if !present {
            continue;
        }
        let x = index % width;
        let y = index / width;
        left = left.min(x);
        right = right.max(x);
        top = top.min(y);
        bottom = bottom.max(y);
    }

    let glyph_width = right - left + 1;
    let glyph_height = bottom - top + 1;
    let mut rows = Vec::with_capacity(glyph_height);
    for y in top..=bottom {
        let mut row = 0_u32;
        for x in left..=right {
            if retained[y * width + x] {
                row |= 1_u32 << (x - left);
            }
        }
        rows.push(row);
    }

    Ok(Some(BinaryGlyph {
        width: glyph_width as u8,
        height: glyph_height as u8,
        rows,
    }))
}

#[derive(Clone, Copy, Debug)]
struct RankTemplate {
    rank: CardRank,
    width: u8,
    height: u8,
    rows: &'static [u32],
}

fn recognise_rank(glyph: &BinaryGlyph) -> Result<CardRank, CardReadError> {
    let mut best = (CardRank::Unknown, 0_u16);
    let mut second = (CardRank::Unknown, 0_u16);
    for template in RANK_TEMPLATES {
        let score = glyph_similarity_per_mille(glyph, template);
        if score > best.1 {
            second = best;
            best = (template.rank, score);
        } else if score > second.1 {
            second = (template.rank, score);
        }
    }

    if best.1 < MINIMUM_RANK_SCORE_PER_MILLE {
        return Err(CardReadError::NoConfidentRankMatch {
            best_rank: best.0,
            score_per_mille: best.1,
        });
    }
    let margin = best.1.saturating_sub(second.1);
    if margin < MINIMUM_RANK_MARGIN_PER_MILLE {
        return Err(CardReadError::AmbiguousRankMatch {
            best_rank: best.0,
            second_rank: second.0,
            margin_per_mille: margin,
        });
    }
    Ok(best.0)
}

fn glyph_similarity_per_mille(glyph: &BinaryGlyph, template: &RankTemplate) -> u16 {
    if glyph.width.abs_diff(template.width) > MAXIMUM_TEMPLATE_DIMENSION_DIFFERENCE
        || glyph.height.abs_diff(template.height) > MAXIMUM_TEMPLATE_DIMENSION_DIFFERENCE
    {
        return 0;
    }

    let glyph_pixels: u32 = glyph.rows.iter().map(|row| row.count_ones()).sum();
    let template_pixels: u32 = template.rows.iter().map(|row| row.count_ones()).sum();
    let total = glyph_pixels + template_pixels;
    if total == 0 {
        return 0;
    }

    let mut best_intersection = 0_u32;
    for delta_y in -1_i32..=1 {
        for delta_x in -1_i32..=1 {
            let mut intersection = 0_u32;
            for glyph_y in 0..i32::from(glyph.height) {
                let template_y = glyph_y - delta_y;
                if template_y < 0 || template_y >= i32::from(template.height) {
                    continue;
                }

                let template_row = template.rows[template_y as usize];
                let shifted_template = if delta_x < 0 {
                    template_row >> delta_x.unsigned_abs()
                } else {
                    template_row << delta_x as u32
                };
                intersection += (glyph.rows[glyph_y as usize] & shifted_template).count_ones();
            }
            best_intersection = best_intersection.max(intersection);
        }
    }

    ((2_000 * best_intersection) / total) as u16
}

// Provisional seed templates. Three is deliberately absent pending a labelled
// capture. Evidence SHA-256 values:
// - 1920 STEP 4 before: 4948762d698f7aead910cf4adfc062812f0d2a70ace4fce91a495b9d22cf00ee
// - 1920 STEP 4 after:  28f5a30cce25b9f8e554b74b8c6aa271da94cc75b5caea7135ee7576d4a168a8
// - 2560 STEP 3:        d3b244f5e6793489719eed916890dd1ae7777b631ee4d4415b50d732b7557227
// Seeds for Two, Four, Six, and King come from the 2560 evidence and therefore
// remain provisional at the pinned 1920 resolution. All other seeds come from
// the 1920 STEP 4 evidence. A no-match or close match remains an explicit error.
const ACE_SEED: RankTemplate = RankTemplate {
    rank: CardRank::Ace,
    width: 18,
    height: 19,
    rows: &[
        0x00000f80, 0x00001fc0, 0x00001fc0, 0x00001fc0, 0x00003fe0, 0x00003fe0, 0x00003ff0,
        0x00007ff0, 0x00007df8, 0x00007cf8, 0x0000f8f8, 0x0000fff8, 0x0001fffc, 0x0001fffc,
        0x0001fffe, 0x0003f9fe, 0x0003e03e, 0x0003e03e, 0x0003e01f,
    ],
};

const TWO_SEED: RankTemplate = RankTemplate {
    rank: CardRank::Two,
    width: 13,
    height: 19,
    rows: &[
        0x000007fc, 0x00000ffe, 0x00001ffe, 0x00001ffe, 0x00001f06, 0x00001f00, 0x00001f00,
        0x00001f00, 0x00001f80, 0x00000fe0, 0x000007f0, 0x000003f8, 0x000000fc, 0x0000007e,
        0x0000003e, 0x00001fff, 0x00001fff, 0x00001fff, 0x00001fff,
    ],
};

const FOUR_SEED: RankTemplate = RankTemplate {
    rank: CardRank::Four,
    width: 15,
    height: 20,
    rows: &[
        0x00000e00, 0x00001f00, 0x00001f80, 0x00001f80, 0x00001fc0, 0x00001fe0, 0x00001ff0,
        0x00001ff0, 0x00001ff8, 0x00001f7c, 0x00001f3e, 0x00001f1e, 0x00007fff, 0x00007fff,
        0x00007fff, 0x00007fff, 0x00001f00, 0x00001f00, 0x00001f00, 0x00000f00,
    ],
};

const FIVE_SEED: RankTemplate = RankTemplate {
    rank: CardRank::Five,
    width: 12,
    height: 20,
    rows: &[
        0x000007fe, 0x000007fe, 0x000007ff, 0x00000fff, 0x000007ff, 0x0000001f, 0x0000001f,
        0x000001ff, 0x000003ff, 0x000007ff, 0x00000fff, 0x00000fc0, 0x00000f80, 0x00000f80,
        0x00000f80, 0x00000fef, 0x000007ff, 0x000003ff, 0x000001ff, 0x0000007c,
    ],
};

const SIX_SEED: RankTemplate = RankTemplate {
    rank: CardRank::Six,
    width: 14,
    height: 20,
    rows: &[
        0x00001fc0, 0x00001ff0, 0x00001ff8, 0x00001ff8, 0x000000fc, 0x0000007e, 0x0000003e,
        0x00000ffe, 0x00001ffe, 0x00003ffe, 0x00003fff, 0x00003e3f, 0x00003e3e, 0x00003e3e,
        0x00003e3e, 0x00003e7e, 0x00001ffc, 0x00001ff8, 0x00000ff8, 0x000001c0,
    ],
};

const SEVEN_SEED: RankTemplate = RankTemplate {
    rank: CardRank::Seven,
    width: 13,
    height: 19,
    rows: &[
        0x00001fff, 0x00001fff, 0x00001fff, 0x00001fff, 0x00000f80, 0x00000f80, 0x00000fc0,
        0x000007c0, 0x000003e0, 0x000003e0, 0x000001f0, 0x000001f0, 0x000001f0, 0x000001f8,
        0x000000f8, 0x000000f8, 0x000000f8, 0x0000007c, 0x00000078,
    ],
};

const EIGHT_SEED: RankTemplate = RankTemplate {
    rank: CardRank::Eight,
    width: 14,
    height: 20,
    rows: &[
        0x000003e0, 0x000007f8, 0x00001ffc, 0x00001ffe, 0x00001f3e, 0x00001e3e, 0x00001e3e,
        0x00001ffe, 0x00000ffc, 0x00000ff8, 0x00001ffc, 0x00001ffe, 0x00003e3e, 0x00003e1f,
        0x00003e1f, 0x00003f3f, 0x00001ffe, 0x00001ffc, 0x000007f8, 0x000001e0,
    ],
};

const NINE_SEED: RankTemplate = RankTemplate {
    rank: CardRank::Nine,
    width: 14,
    height: 20,
    rows: &[
        0x000001f0, 0x000007f8, 0x00000ffc, 0x00000ffe, 0x00001f3e, 0x00001e1f, 0x00003e1f,
        0x00003e1f, 0x00003f3f, 0x00003ffe, 0x00003ffe, 0x00003ffc, 0x00001ef0, 0x00001f00,
        0x00001f00, 0x00000fce, 0x00000ffe, 0x000007fe, 0x000001fe, 0x00000070,
    ],
};

const TEN_SEED: RankTemplate = RankTemplate {
    rank: CardRank::Ten,
    width: 25,
    height: 20,
    rows: &[
        0x000000c0, 0x001f81f0, 0x007fe1fc, 0x00ffe1ff, 0x00fff1ff, 0x00f9f1ff, 0x00f0f9f7,
        0x01f0f9f0, 0x01f0f9f0, 0x01f0f9f0, 0x01f0f9f0, 0x01f0f9f0, 0x01f0f9f0, 0x00f0f9f0,
        0x00f0f9f0, 0x00f9f1f0, 0x00fff1f0, 0x007fe1f0, 0x003fc1f0, 0x001f80f0,
    ],
};

const JACK_SEED: RankTemplate = RankTemplate {
    rank: CardRank::Jack,
    width: 9,
    height: 20,
    rows: &[
        0x000001e0, 0x000001e0, 0x000001e0, 0x000001e0, 0x000001e0, 0x000001e0, 0x000001e0,
        0x000001e0, 0x000001e0, 0x000001e0, 0x000001e0, 0x000001e0, 0x000001e0, 0x000001f0,
        0x000001f0, 0x000001ff, 0x000000ff, 0x000000ff, 0x0000003f, 0x0000000e,
    ],
};

const QUEEN_SEED: RankTemplate = RankTemplate {
    rank: CardRank::Queen,
    width: 20,
    height: 22,
    rows: &[
        0x00000fc0, 0x00007ff0, 0x0000fff8, 0x0000fffc, 0x0001fffc, 0x0003f07e, 0x0003f03f,
        0x0007e03f, 0x0007e01f, 0x0007e01f, 0x0007e01f, 0x0007e01f, 0x0003e03f, 0x0003f03f,
        0x0003f87e, 0x0001fffe, 0x0000fffc, 0x0001fff8, 0x0003ffe0, 0x0007ff80, 0x000fe000,
        0x000fc000,
    ],
};

const KING_SEED: RankTemplate = RankTemplate {
    rank: CardRank::King,
    width: 16,
    height: 20,
    rows: &[
        0x0000f81e, 0x0000fc1f, 0x00007e1f, 0x00003f1f, 0x00001f1f, 0x00001f9f, 0x00000fdf,
        0x000007df, 0x000007ff, 0x000003ff, 0x000003ff, 0x000007ff, 0x00000fdf, 0x00001f9f,
        0x00001f9f, 0x00003f1f, 0x00007e1f, 0x00007e1f, 0x0000fc1f, 0x0000f81e,
    ],
};

const RANK_TEMPLATES: &[RankTemplate] = &[
    ACE_SEED, TWO_SEED, FOUR_SEED, FIVE_SEED, SIX_SEED, SEVEN_SEED, EIGHT_SEED, NINE_SEED,
    TEN_SEED, JACK_SEED, QUEEN_SEED, KING_SEED,
];

#[cfg(test)]
mod tests {
    use crate::{
        capture::PixelFormat,
        cards::{CardLocation, CardRegionPixels},
        geometry::PixelPoint,
        parameters::GOLD_RGB_CANDIDATES,
    };

    use super::*;

    fn test_frame(background: [u8; 3]) -> CapturedFrame {
        let mut pixels =
            Vec::with_capacity(NOMINAL_FRAME_WIDTH as usize * NOMINAL_FRAME_HEIGHT as usize * 4);
        for _ in 0..NOMINAL_FRAME_WIDTH as usize * NOMINAL_FRAME_HEIGHT as usize {
            pixels.extend_from_slice(&[background[0], background[1], background[2], 255]);
        }
        CapturedFrame {
            width: NOMINAL_FRAME_WIDTH,
            height: NOMINAL_FRAME_HEIGHT,
            stride: NOMINAL_FRAME_WIDTH as usize * 4,
            format: PixelFormat::Rgba8,
            pixels,
            cursor: None,
        }
    }

    fn test_slot() -> CardSlot {
        let card = PixelRect::new(100, 100, 136, 181);
        let pixels = CardRegionPixels::new(
            card,
            PixelRect::new(88, 86, 160, 209),
            PixelRect::new(103, 107, 25, 22),
            PixelPoint::new(168, 190),
        );
        CardSlot::from_pixels(
            CardLocation::Waste,
            pixels,
            NOMINAL_FRAME_WIDTH,
            NOMINAL_FRAME_HEIGHT,
        )
        .unwrap()
    }

    fn set_pixel(frame: &mut CapturedFrame, x: u32, y: u32, rgb: [u8; 3]) {
        let offset = y as usize * frame.stride + x as usize * 4;
        frame.pixels[offset..offset + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }

    fn fill_rect(frame: &mut CapturedFrame, rect: PixelRect, rgb: [u8; 3]) {
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                set_pixel(frame, x, y, rgb);
            }
        }
    }

    fn render_template(frame: &mut CapturedFrame, rect: PixelRect, template: &RankTemplate) {
        for (local_y, row) in template.rows.iter().copied().enumerate() {
            for local_x in 0..template.width {
                if row & (1_u32 << local_x) != 0 {
                    set_pixel(
                        frame,
                        rect.x + u32::from(local_x),
                        rect.y + local_y as u32,
                        [0, 0, 0],
                    );
                }
            }
        }
    }

    #[test]
    fn provisional_recogniser_matches_seeds_but_public_reader_stays_disabled() {
        assert!(
            RANK_TEMPLATES
                .iter()
                .all(|template| template.rank != CardRank::Three)
        );

        for template in RANK_TEMPLATES {
            let mut frame = test_frame([25, 120, 75]);
            let slot = test_slot();
            let pixels = slot.regions().pixels();
            fill_rect(&mut frame, pixels.card_bounds, [248, 248, 248]);
            render_template(&mut frame, pixels.rank_bounds, template);

            let glyph = extract_glyph(&frame, pixels.rank_bounds).unwrap().unwrap();
            assert_eq!(recognise_rank(&glyph), Ok(template.rank));
            assert_eq!(
                read_card(&frame, &slot),
                Err(CardReadError::RankCalibrationIncomplete {
                    missing_rank: CardRank::Three,
                })
            );
        }
    }

    #[test]
    fn detects_face_down_empty_and_halo_without_inventing_a_rank() {
        let slot = test_slot();
        let mut face_down = test_frame([25, 120, 75]);
        fill_rect(
            &mut face_down,
            slot.regions().pixels().rank_bounds,
            [40, 190, 120],
        );
        for x in 101..220 {
            set_pixel(&mut face_down, x, 287, GOLD_RGB_CANDIDATES[0]);
        }

        let observation = read_card(&face_down, &slot).unwrap();
        assert_eq!(observation.visibility, CardVisibility::FaceDown);
        assert_eq!(observation.rank, CardRank::Unknown);
        assert!(observation.has_halo);

        let empty = read_card(&test_frame([25, 120, 75]), &slot).unwrap();
        assert_eq!(empty.visibility, CardVisibility::Empty);
        assert_eq!(empty.rank, CardRank::Unknown);
        assert!(!empty.has_halo);
    }

    #[test]
    fn rejects_visible_glyphs_without_a_confident_calibrated_match() {
        let slot = test_slot();
        let pixels = slot.regions().pixels();
        let mut frame = test_frame([25, 120, 75]);
        fill_rect(&mut frame, pixels.card_bounds, [248, 248, 248]);
        fill_rect(
            &mut frame,
            PixelRect::new(pixels.rank_bounds.x, pixels.rank_bounds.y, 12, 20),
            [0, 0, 0],
        );

        let glyph = extract_glyph(&frame, pixels.rank_bounds).unwrap().unwrap();
        assert!(matches!(
            recognise_rank(&glyph),
            Err(CardReadError::NoConfidentRankMatch { .. })
        ));
        assert!(matches!(
            read_card(&frame, &slot),
            Err(CardReadError::NoConfidentRankMatch { .. })
        ));
    }

    #[test]
    fn rejects_wrong_frame_size_and_inconsistent_qmp_geometry() {
        let slot = test_slot();
        let small = CapturedFrame {
            width: 100,
            height: 100,
            stride: 400,
            format: PixelFormat::Rgba8,
            pixels: vec![0; 40_000],
            cursor: None,
        };
        assert!(matches!(
            read_card(&small, &slot),
            Err(CardReadError::UnexpectedFrameDimensions { .. })
        ));

        let inconsistent = CardSlot::from_pixels(
            slot.location(),
            slot.regions().pixels(),
            2_560,
            NOMINAL_FRAME_HEIGHT,
        )
        .unwrap();
        assert_eq!(
            read_card(&test_frame([25, 120, 75]), &inconsistent),
            Err(CardReadError::InconsistentQmpGeometry)
        );
    }
}
