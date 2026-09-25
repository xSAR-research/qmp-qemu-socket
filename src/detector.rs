use thiserror::Error;

use crate::{
    capture::{CapturedFrame, PixelFormat},
    cards::CardRegionPixels,
    game::{GameProfile, GameProgress, GameplaySceneProfile},
    geometry::{PixelPoint, PixelRect},
    parameters::{
        DIALOG_GOLD_BLUE_MAXIMUM, DIALOG_GOLD_GREEN_BLUE_DELTA_MINIMUM, DIALOG_GOLD_GREEN_MINIMUM,
        DIALOG_GOLD_RED_GREEN_DELTA_MINIMUM, DIALOG_GOLD_RED_MINIMUM,
        DIALOG_GOLD_REQUIRED_FRACTION_PER_MILLE, FACE_UP_WHITE_BLOCK_MIN_WIDTH,
        FACE_UP_WHITE_CHANNEL_MINIMUM, GOLD_CHANNEL_TOLERANCE, GOLD_RGB_CANDIDATES,
        HALO_GOLD_LINE_OFFSET_Y, HALO_GOLD_RUN_MIN, HALO_GOLD_SCAN_HEIGHT,
        TABLEAU_RELAXED_GOLD_BLUE_MAXIMUM, TABLEAU_RELAXED_GOLD_GREEN_BLUE_DELTA_MINIMUM,
        TABLEAU_RELAXED_GOLD_GREEN_MINIMUM, TABLEAU_RELAXED_GOLD_RED_GREEN_DELTA_MAXIMUM,
        TABLEAU_RELAXED_GOLD_RED_GREEN_DELTA_MINIMUM, TABLEAU_RELAXED_GOLD_RED_MINIMUM,
        TABLEAU_RELAXED_HALO_START_X_TOLERANCE, TABLEAU_RELAXED_HALO_Y_TOLERANCE,
    },
};

pub fn detect_game_progress_for_profile(
    frame: &CapturedFrame,
    profile: &GameProfile,
) -> Result<GameProgress, HaloDetectionError> {
    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }

    let progress = profile
        .game_progress
        .ok_or(HaloDetectionError::MissingProfileCalibration {
            target: "game progress",
        })?;
    let (right, bottom) = checked_bounds(progress.right_probe, frame)?;
    let mut black_pixels = 0_u32;
    let total_pixels = progress
        .right_probe
        .width
        .saturating_mul(progress.right_probe.height);

    for y in progress.right_probe.y..bottom {
        for x in progress.right_probe.x..right {
            let rgb = pixel_rgb(frame, x, y).ok_or(HaloDetectionError::InvalidFrameLayout)?;
            if rgb
                .into_iter()
                .all(|channel| channel <= progress.black_channel_maximum)
            {
                black_pixels = black_pixels.saturating_add(1);
            }
        }
    }

    if black_pixels.saturating_mul(1_000)
        >= total_pixels.saturating_mul(progress.black_fraction_per_mille)
    {
        Ok(GameProgress::AnotherBoard)
    } else {
        Ok(GameProgress::GameComplete)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoldHit {
    /// Top-left pixel of the qualifying 2x2 block.
    pub anchor: PixelPoint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoldRunHit {
    /// First pixel in the qualifying horizontal run.
    pub anchor: PixelPoint,
    pub length: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WhiteBlockHit {
    /// Top-left pixel of the qualifying near-white rectangle.
    pub anchor: PixelPoint,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum HaloDetectionError {
    #[error("{target} detector has no calibrated game profile")]
    MissingProfileCalibration { target: &'static str },
    #[error("captured frame has an invalid pixel layout")]
    InvalidFrameLayout,
    #[error("detector bounds must be non-empty")]
    EmptyBounds,
    #[error("detector bounds lie outside the captured frame")]
    BoundsOutsideFrame,
    #[error("card bounds must be contained by halo bounds")]
    CardOutsideHalo,
    #[error("the calibrated halo scan lines must be contained by halo bounds")]
    HaloLinesOutsideHalo,
}

pub fn is_goldish(rgb: [u8; 3]) -> bool {
    // Check whether an RGB colour is within tolerance of a gold candidate.
    GOLD_RGB_CANDIDATES.iter().any(|candidate| {
        rgb[0].abs_diff(candidate[0]) <= GOLD_CHANNEL_TOLERANCE
            && rgb[1].abs_diff(candidate[1]) <= GOLD_CHANNEL_TOLERANCE
            && rgb[2].abs_diff(candidate[2]) <= GOLD_CHANNEL_TOLERANCE
    })
}

/// Recognise a darker gold-like colour by channel relationship.
///
/// This deliberately does not replace [`is_goldish`]. It is used only after
/// an exact tableau-halo miss, together with strict run length, exterior-strip
/// and calibrated-start constraints.
pub fn is_relational_tableau_gold(rgb: [u8; 3]) -> bool {
    let [red, green, blue] = rgb;
    let red_green_delta = i16::from(red) - i16::from(green);
    let green_blue_delta = i16::from(green) - i16::from(blue);

    red >= TABLEAU_RELAXED_GOLD_RED_MINIMUM
        && green >= TABLEAU_RELAXED_GOLD_GREEN_MINIMUM
        && blue <= TABLEAU_RELAXED_GOLD_BLUE_MAXIMUM
        && red_green_delta >= i16::from(TABLEAU_RELAXED_GOLD_RED_GREEN_DELTA_MINIMUM)
        && red_green_delta <= i16::from(TABLEAU_RELAXED_GOLD_RED_GREEN_DELTA_MAXIMUM)
        && green_blue_delta >= i16::from(TABLEAU_RELAXED_GOLD_GREEN_BLUE_DELTA_MINIMUM)
}

pub fn is_near_white(rgb: [u8; 3]) -> bool {
    rgb.into_iter()
        .all(|channel| channel >= FACE_UP_WHITE_CHANNEL_MINIMUM)
}

fn is_gameplay_felt_green(rgb: [u8; 3], profile: GameplaySceneProfile) -> bool {
    let [red, green, blue] = rgb;

    green >= profile.green_minimum
        && i16::from(green) - i16::from(red) >= i16::from(profile.green_red_delta_minimum)
        && i16::from(green) - i16::from(blue) >= i16::from(profile.green_blue_delta_minimum)
}

fn is_dialog_gold(rgb: [u8; 3]) -> bool {
    let [red, green, blue] = rgb;

    red >= DIALOG_GOLD_RED_MINIMUM
        && green >= DIALOG_GOLD_GREEN_MINIMUM
        && blue <= DIALOG_GOLD_BLUE_MAXIMUM
        && i16::from(red) - i16::from(green) >= i16::from(DIALOG_GOLD_RED_GREEN_DELTA_MINIMUM)
        && i16::from(green) - i16::from(blue) >= i16::from(DIALOG_GOLD_GREEN_BLUE_DELTA_MINIMUM)
}

/// Find a qualifying 2x2 gold block using a deterministic row-major scan.
///
/// Returning a defined anchor matters while click placement uses a signed
/// offset. A later milestone can replace this with connected-component bounds
/// without changing the capture or action-controller interfaces.
pub fn find_first_gold_block(frame: &CapturedFrame, region: PixelRect) -> Option<GoldHit> {
    // Find the first qualifying 2x2 gold block within the requested region.
    if !frame.is_layout_valid() || frame.width < 2 || frame.height < 2 {
        return None;
    }

    let left = region.x.min(frame.width);
    let top = region.y.min(frame.height);
    let right = region.right().min(frame.width);
    let bottom = region.bottom().min(frame.height);

    if right.saturating_sub(left) < 2 || bottom.saturating_sub(top) < 2 {
        return None;
    }

    for y in top..(bottom - 1) {
        for x in left..(right - 1) {
            if is_goldish(pixel_rgb(frame, x, y)?)
                && is_goldish(pixel_rgb(frame, x + 1, y)?)
                && is_goldish(pixel_rgb(frame, x, y + 1)?)
                && is_goldish(pixel_rgb(frame, x + 1, y + 1)?)
            {
                return Some(GoldHit {
                    anchor: PixelPoint::new(x as i32, y as i32),
                });
            }
        }
    }

    None
}

/// Find the first long horizontal gold run on a card's calibrated halo lines.
///
/// Only the two audited lines below the card are examined, avoiding both card
/// artwork and the bulk of the padded halo rectangle. Either line may carry the
/// qualifying run. Its returned Y coordinate is normalised to the calibrated
/// baseline so the existing anchor-to-click invariant remains deterministic.
pub fn find_gold_halo_run(
    frame: &CapturedFrame,
    regions: CardRegionPixels,
) -> Result<Option<GoldRunHit>, HaloDetectionError> {
    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }

    let (card_right, card_bottom) = checked_bounds(regions.card_bounds, frame)?;
    let (halo_right, halo_bottom) = checked_bounds(regions.halo_bounds, frame)?;

    if regions.card_bounds.x < regions.halo_bounds.x
        || regions.card_bounds.y < regions.halo_bounds.y
        || card_right > halo_right
        || card_bottom > halo_bottom
    {
        return Err(HaloDetectionError::CardOutsideHalo);
    }

    let scan_y = card_bottom
        .checked_add(HALO_GOLD_LINE_OFFSET_Y)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
    let scan_bottom = scan_y
        .checked_add(HALO_GOLD_SCAN_HEIGHT)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
    if scan_y < regions.halo_bounds.y || scan_bottom > halo_bottom {
        return Err(HaloDetectionError::HaloLinesOutsideHalo);
    }

    for y in scan_y..scan_bottom {
        if let Some((run_start, run_length)) = find_horizontal_run(
            frame,
            regions.halo_bounds.x,
            halo_right,
            y,
            HALO_GOLD_RUN_MIN,
            is_goldish,
        ) {
            return Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(run_start as i32, scan_y as i32),
                length: run_length,
            }));
        }
    }

    Ok(None)
}

/// Find the first exact-colour horizontal halo run inside one tight profile
/// region. The profile controls ordering; this primitive performs no action
/// selection and sends no guest input.
pub fn find_gold_run_in_bounds(
    frame: &CapturedFrame,
    bounds: PixelRect,
) -> Result<Option<GoldRunHit>, HaloDetectionError> {
    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }

    let (right, bottom) = checked_bounds(bounds, frame)?;
    for y in bounds.y..bottom {
        if let Some((run_start, run_length)) =
            find_horizontal_run(frame, bounds.x, right, y, HALO_GOLD_RUN_MIN, is_goldish)
        {
            return Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(run_start as i32, y as i32),
                length: run_length,
            }));
        }
    }

    Ok(None)
}

pub fn has_gameplay_felt_for_profile(
    frame: &CapturedFrame,
    profile: &GameProfile,
) -> Result<bool, HaloDetectionError> {
    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }

    let scene = profile
        .gameplay_scene
        .ok_or(HaloDetectionError::MissingProfileCalibration {
            target: "gameplay scene",
        })?;
    let (right, bottom) = checked_bounds(scene.probe_bounds, frame)?;
    let total_pixels = u64::from(scene.probe_bounds.width) * u64::from(scene.probe_bounds.height);
    let mut matching_pixels = 0_u64;

    for y in scene.probe_bounds.y..bottom {
        for x in scene.probe_bounds.x..right {
            if pixel_rgb(frame, x, y).is_some_and(|rgb| is_gameplay_felt_green(rgb, scene)) {
                matching_pixels += 1;
            }
        }
    }

    Ok(matching_pixels * 1_000 >= total_pixels * u64::from(scene.required_fraction_per_mille))
}

/// Report whether a calibrated dialog band contains enough gold-gradient pixels.
///
/// This detector is intentionally broader than halo matching. Callers must
/// also enforce the ordered post-game stage and reject gameplay scenes.
pub fn has_dialog_gold_button(
    frame: &CapturedFrame,
    probe_bounds: PixelRect,
) -> Result<bool, HaloDetectionError> {
    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }

    let (right, bottom) = checked_bounds(probe_bounds, frame)?;
    let total_pixels = u64::from(probe_bounds.width) * u64::from(probe_bounds.height);
    let mut matching_pixels = 0_u64;

    for y in probe_bounds.y..bottom {
        for x in probe_bounds.x..right {
            if pixel_rgb(frame, x, y).is_some_and(is_dialog_gold) {
                matching_pixels = matching_pixels.saturating_add(1);
            }
        }
    }

    Ok(matching_pixels.saturating_mul(1_000)
        >= total_pixels.saturating_mul(u64::from(DIALOG_GOLD_REQUIRED_FRACTION_PER_MILLE)))
}

/// Find a tableau halo without allowing rendering drift to move the click.
///
/// The audited exact-colour/two-line detector remains the fast path. If it
/// misses, a narrow exterior strip is searched for a darker gold-shaped run.
/// A fallback run must begin close to the slot's calibrated X anchor, and its
/// hit always returns the immutable calibrated anchor rather than an observed
/// fringe pixel. Exact hits retain their observed anchor so the tracker's
/// existing offset invariant continues to reject shifted exact evidence.
pub fn find_tableau_gold_halo_run(
    frame: &CapturedFrame,
    regions: CardRegionPixels,
    click_offset_x: i32,
) -> Result<Option<GoldRunHit>, HaloDetectionError> {
    if let Some(exact_hit) = find_gold_halo_run(frame, regions)? {
        return Ok(Some(exact_hit));
    }

    let canonical_anchor = canonical_tableau_halo_anchor(regions, click_offset_x)?;
    let canonical_x =
        u32::try_from(canonical_anchor.x).map_err(|_| HaloDetectionError::BoundsOutsideFrame)?;

    let (_, card_bottom) = checked_bounds(regions.card_bounds, frame)?;
    let (halo_right, halo_bottom) = checked_bounds(regions.halo_bounds, frame)?;
    let canonical_y =
        u32::try_from(canonical_anchor.y).map_err(|_| HaloDetectionError::BoundsOutsideFrame)?;

    // Do not admit card artwork: even the upward side of the tolerance window
    // is clamped to the calibrated exterior at the card's bottom edge.
    let strip_top = canonical_y
        .saturating_sub(TABLEAU_RELAXED_HALO_Y_TOLERANCE)
        .max(card_bottom)
        .max(regions.halo_bounds.y);
    let strip_bottom = canonical_y
        .checked_add(HALO_GOLD_SCAN_HEIGHT)
        .and_then(|bottom| bottom.checked_add(TABLEAU_RELAXED_HALO_Y_TOLERANCE))
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?
        .min(halo_bottom);

    for y in strip_top..strip_bottom {
        if let Some((_, run_length)) = find_horizontal_run_near_anchor(
            frame,
            regions.halo_bounds.x,
            halo_right,
            y,
            HALO_GOLD_RUN_MIN,
            canonical_x,
            TABLEAU_RELAXED_HALO_START_X_TOLERANCE,
            is_relational_tableau_gold,
        ) {
            return Ok(Some(GoldRunHit {
                anchor: canonical_anchor,
                length: run_length,
            }));
        }
    }

    Ok(None)
}

/// Find a solid near-white rectangle in a calibrated face-up-card probe band.
///
/// Every pixel in a qualifying column must be near-white across the full band
/// height. Consecutive columns form the block, so isolated bright pixels and
/// the speckled snow on face-down card backs cannot qualify.
pub fn find_face_up_white_block(
    frame: &CapturedFrame,
    probe_bounds: PixelRect,
) -> Result<Option<WhiteBlockHit>, HaloDetectionError> {
    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }

    let (right, bottom) = checked_bounds(probe_bounds, frame)?;
    let mut run_start = 0;
    let mut run_width = 0;

    for x in probe_bounds.x..right {
        let column_is_white =
            (probe_bounds.y..bottom).all(|y| pixel_rgb(frame, x, y).is_some_and(is_near_white));

        if column_is_white {
            if run_width == 0 {
                run_start = x;
            }
            run_width += 1;
        } else if run_width >= FACE_UP_WHITE_BLOCK_MIN_WIDTH {
            return Ok(Some(WhiteBlockHit {
                anchor: PixelPoint::new(run_start as i32, probe_bounds.y as i32),
                width: run_width,
                height: probe_bounds.height,
            }));
        } else {
            run_width = 0;
        }
    }

    Ok(
        (run_width >= FACE_UP_WHITE_BLOCK_MIN_WIDTH).then_some(WhiteBlockHit {
            anchor: PixelPoint::new(run_start as i32, probe_bounds.y as i32),
            width: run_width,
            height: probe_bounds.height,
        }),
    )
}

/// Report whether a calibrated row probe contains a face-up white block.
pub fn has_face_up_white_block(
    frame: &CapturedFrame,
    probe_bounds: PixelRect,
) -> Result<bool, HaloDetectionError> {
    Ok(find_face_up_white_block(frame, probe_bounds)?.is_some())
}

/// Report whether the calibrated exterior halo contains a qualifying gold run.
pub fn has_gold_halo(
    frame: &CapturedFrame,
    regions: CardRegionPixels,
) -> Result<bool, HaloDetectionError> {
    Ok(find_gold_halo_run(frame, regions)?.is_some())
}

fn find_horizontal_run(
    frame: &CapturedFrame,
    left: u32,
    right: u32,
    y: u32,
    minimum_length: u32,
    predicate: fn([u8; 3]) -> bool,
) -> Option<(u32, u32)> {
    let mut run_start = 0;
    let mut run_length = 0;

    for x in left..right {
        if pixel_rgb(frame, x, y).is_some_and(predicate) {
            if run_length == 0 {
                run_start = x;
            }
            run_length += 1;
        } else if run_length >= minimum_length {
            return Some((run_start, run_length));
        } else {
            run_length = 0;
        }
    }

    (run_length >= minimum_length).then_some((run_start, run_length))
}

#[allow(clippy::too_many_arguments)]
fn find_horizontal_run_near_anchor(
    frame: &CapturedFrame,
    left: u32,
    right: u32,
    y: u32,
    minimum_length: u32,
    canonical_start: u32,
    start_tolerance: u32,
    predicate: fn([u8; 3]) -> bool,
) -> Option<(u32, u32)> {
    let mut run_start = 0;
    let mut run_length = 0;

    for x in left..right {
        if pixel_rgb(frame, x, y).is_some_and(predicate) {
            if run_length == 0 {
                run_start = x;
            }
            run_length += 1;
        } else {
            if run_length >= minimum_length
                && run_start.abs_diff(canonical_start) <= start_tolerance
            {
                return Some((run_start, run_length));
            }
            run_length = 0;
        }
    }

    (run_length >= minimum_length && run_start.abs_diff(canonical_start) <= start_tolerance)
        .then_some((run_start, run_length))
}

fn canonical_tableau_halo_anchor(
    regions: CardRegionPixels,
    click_offset_x: i32,
) -> Result<PixelPoint, HaloDetectionError> {
    let x = regions
        .click_point
        .x
        .checked_sub(click_offset_x)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
    let y = regions
        .card_bounds
        .y
        .checked_add(regions.card_bounds.height)
        .and_then(|bottom| bottom.checked_add(HALO_GOLD_LINE_OFFSET_Y))
        .and_then(|value| i32::try_from(value).ok())
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;

    Ok(PixelPoint::new(x, y))
}

fn checked_bounds(
    bounds: PixelRect,
    frame: &CapturedFrame,
) -> Result<(u32, u32), HaloDetectionError> {
    if bounds.is_empty() {
        return Err(HaloDetectionError::EmptyBounds);
    }

    let right = bounds
        .x
        .checked_add(bounds.width)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
    let bottom = bounds
        .y
        .checked_add(bounds.height)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
    if right > frame.width || bottom > frame.height {
        return Err(HaloDetectionError::BoundsOutsideFrame);
    }

    Ok((right, bottom))
}

pub(crate) fn pixel_rgb(frame: &CapturedFrame, x: u32, y: u32) -> Option<[u8; 3]> {
    // Read one pixel and normalise its channel order to RGB.
    let offset = y as usize * frame.stride + x as usize * 4;
    let pixel = frame.pixels.get(offset..offset + 4)?;

    match frame.format {
        PixelFormat::Rgba8 => Some([pixel[0], pixel[1], pixel[2]]),
        PixelFormat::Bgra8 => Some([pixel[2], pixel[1], pixel[0]]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::{
        CLICK_OFFSET_X, FACE_UP_WHITE_SCAN_HEIGHT, SOLVER_PROGRESS_RIGHT_PROBE,
    };

    fn find_tableau_halo(
        frame: &CapturedFrame,
        regions: CardRegionPixels,
    ) -> Result<Option<GoldRunHit>, HaloDetectionError> {
        find_tableau_gold_halo_run(frame, regions, CLICK_OFFSET_X)
    }

    fn frame_with_format(width: u32, height: u32, format: PixelFormat) -> CapturedFrame {
        // Create a blank four-channel frame for detector tests.
        CapturedFrame {
            width,
            height,
            stride: width as usize * 4,
            format,
            pixels: vec![0; width as usize * height as usize * 4],
            cursor: None,
        }
    }

    fn rgba_frame(width: u32, height: u32) -> CapturedFrame {
        frame_with_format(width, height, PixelFormat::Rgba8)
    }

    fn set_pixel(frame: &mut CapturedFrame, x: u32, y: u32, rgb: [u8; 3]) {
        // Set one test-frame pixel to the supplied RGB colour.
        let offset = y as usize * frame.stride + x as usize * 4;
        let pixel = match frame.format {
            PixelFormat::Rgba8 => [rgb[0], rgb[1], rgb[2], 255],
            PixelFormat::Bgra8 => [rgb[2], rgb[1], rgb[0], 255],
        };
        frame.pixels[offset..offset + 4].copy_from_slice(&pixel);
    }

    fn card_regions() -> CardRegionPixels {
        CardRegionPixels::new(
            PixelRect::new(40, 40, 120, 140),
            PixelRect::new(28, 26, 144, 168),
            PixelRect::new(43, 47, 25, 22),
            PixelPoint::new(100, 110),
        )
    }

    fn set_horizontal_run(frame: &mut CapturedFrame, start_x: u32, y: u32, length: u32) {
        set_coloured_horizontal_run(frame, start_x, y, length, GOLD_RGB_CANDIDATES[0]);
    }

    fn set_coloured_horizontal_run(
        frame: &mut CapturedFrame,
        start_x: u32,
        y: u32,
        length: u32,
        colour: [u8; 3],
    ) {
        for x in start_x..start_x + length {
            set_pixel(frame, x, y, colour);
        }
    }

    fn set_white_block(
        frame: &mut CapturedFrame,
        start_x: u32,
        start_y: u32,
        width: u32,
        height: u32,
        value: u8,
    ) {
        for y in start_y..start_y + height {
            for x in start_x..start_x + width {
                set_pixel(frame, x, y, [value; 3]);
            }
        }
    }

    #[test]
    fn finds_a_two_by_two_gold_block() {
        // Verify that a qualifying 2x2 gold block is found at its anchor.
        let mut frame = rgba_frame(5, 5);
        for y in 2..=3 {
            for x in 1..=2 {
                set_pixel(&mut frame, x, y, GOLD_RGB_CANDIDATES[0]);
            }
        }

        let hit = find_first_gold_block(&frame, PixelRect::new(0, 0, 5, 5));
        assert_eq!(hit.map(|found| found.anchor), Some(PixelPoint::new(1, 2)));
    }

    #[test]
    fn rejects_a_single_gold_pixel() {
        // Verify that one gold pixel does not qualify as a block.
        let mut frame = rgba_frame(4, 4);
        set_pixel(&mut frame, 2, 2, GOLD_RGB_CANDIDATES[0]);

        assert_eq!(
            find_first_gold_block(&frame, PixelRect::new(0, 0, 4, 4)),
            None
        );
    }

    #[test]
    fn finds_a_long_halo_run_below_the_card() {
        let mut frame = rgba_frame(220, 220);
        set_horizontal_run(&mut frame, 41, 186, 119);

        assert_eq!(
            find_gold_halo_run(&frame, card_regions()),
            Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(41, 186),
                length: 119,
            }))
        );
        assert_eq!(has_gold_halo(&frame, card_regions()), Ok(true));
    }

    #[test]
    fn accepts_a_halo_run_exactly_at_the_minimum_length() {
        let mut frame = rgba_frame(220, 220);
        set_horizontal_run(&mut frame, 41, 186, HALO_GOLD_RUN_MIN);

        assert_eq!(
            find_gold_halo_run(&frame, card_regions()),
            Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(41, 186),
                length: HALO_GOLD_RUN_MIN,
            }))
        );
    }

    #[test]
    fn canonicalises_a_run_on_the_second_halo_line_to_the_baseline() {
        let mut frame = rgba_frame(220, 220);
        set_horizontal_run(&mut frame, 41, 187, 119);

        assert_eq!(
            find_gold_halo_run(&frame, card_regions()),
            Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(41, 186),
                length: 119,
            }))
        );
    }

    #[test]
    fn tableau_fast_path_preserves_the_audited_anchor() {
        let mut frame = rgba_frame(220, 220);
        set_horizontal_run(&mut frame, 41, 186, 119);

        assert_eq!(
            find_tableau_halo(&frame, card_regions()),
            Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(41, 186),
                length: 119,
            }))
        );
    }

    #[test]
    fn tableau_fallback_finds_a_dark_halo_on_a_shifted_exterior_line() {
        let mut frame = rgba_frame(220, 220);
        set_coloured_horizontal_run(&mut frame, 35, 182, 119, [225, 194, 100]);

        assert_eq!(
            find_tableau_halo(&frame, card_regions()),
            Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(39, 186),
                length: 119,
            }))
        );
    }

    #[test]
    fn tableau_fallback_canonicalises_a_tolerated_fringe_start() {
        let mut frame = rgba_frame(220, 220);
        let observed_start = 39 - TABLEAU_RELAXED_HALO_START_X_TOLERANCE;
        set_coloured_horizontal_run(&mut frame, observed_start, 190, 100, [225, 194, 100]);

        assert_eq!(
            find_tableau_halo(&frame, card_regions()),
            Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(39, 186),
                length: 100,
            }))
        );
    }

    #[test]
    fn tableau_fallback_rejects_non_gold_channel_relationships() {
        for colour in [
            [250, 250, 250],
            [90, 160, 100],
            [220, 30, 30],
            [220, 210, 100],
            [170, 140, 80],
        ] {
            let mut frame = rgba_frame(220, 220);
            set_coloured_horizontal_run(&mut frame, 39, 182, 119, colour);

            assert_eq!(
                find_tableau_halo(&frame, card_regions()),
                Ok(None),
                "unexpected fallback match for {colour:?}"
            );
        }
    }

    #[test]
    fn tableau_fallback_rejects_a_gold_run_starting_outside_x_tolerance() {
        let mut frame = rgba_frame(220, 220);
        let wrong_start = 39 + TABLEAU_RELAXED_HALO_START_X_TOLERANCE + 1;
        set_coloured_horizontal_run(&mut frame, wrong_start, 182, 100, [225, 194, 100]);

        assert_eq!(find_tableau_halo(&frame, card_regions()), Ok(None));
    }

    #[test]
    fn tableau_fallback_is_bounded_to_the_exterior_strip() {
        for accepted_y in [180, 193] {
            let mut frame = rgba_frame(220, 220);
            set_coloured_horizontal_run(&mut frame, 39, accepted_y, 100, [225, 194, 100]);
            assert!(
                find_tableau_halo(&frame, card_regions()).unwrap().is_some(),
                "expected exterior line y={accepted_y} to be accepted"
            );
        }

        for rejected_y in [179, 194] {
            let mut frame = rgba_frame(220, 220);
            set_coloured_horizontal_run(&mut frame, 39, rejected_y, 100, [225, 194, 100]);
            assert_eq!(
                find_tableau_halo(&frame, card_regions()),
                Ok(None),
                "unexpected fallback match outside exterior strip at y={rejected_y}"
            );
        }
    }

    #[test]
    fn ignores_gold_runs_outside_the_two_calibrated_halo_lines() {
        let mut frame = rgba_frame(220, 220);
        set_horizontal_run(&mut frame, 41, 185, 119);
        set_horizontal_run(&mut frame, 41, 188, 119);

        assert_eq!(find_gold_halo_run(&frame, card_regions()), Ok(None));
    }

    #[test]
    fn ignores_a_long_gold_run_inside_card_artwork() {
        let mut frame = rgba_frame(220, 220);
        set_horizontal_run(&mut frame, 40, 150, 120);

        assert_eq!(find_gold_halo_run(&frame, card_regions()), Ok(None));
        assert_eq!(has_gold_halo(&frame, card_regions()), Ok(false));
    }

    #[test]
    fn rejects_an_exterior_run_shorter_than_the_threshold() {
        let mut frame = rgba_frame(220, 220);
        set_horizontal_run(&mut frame, 41, 186, HALO_GOLD_RUN_MIN - 1);

        assert_eq!(find_gold_halo_run(&frame, card_regions()), Ok(None));
    }

    #[test]
    fn detects_a_halo_in_bgra_frames() {
        let mut frame = frame_with_format(220, 220, PixelFormat::Bgra8);
        set_horizontal_run(&mut frame, 41, 186, 119);

        assert_eq!(has_gold_halo(&frame, card_regions()), Ok(true));
    }

    #[test]
    fn detects_a_solid_near_white_face_block() {
        let mut frame = rgba_frame(80, 40);
        let probe = PixelRect::new(10, 12, 60, 4);
        set_white_block(
            &mut frame,
            23,
            12,
            FACE_UP_WHITE_BLOCK_MIN_WIDTH + 5,
            4,
            FACE_UP_WHITE_CHANNEL_MINIMUM,
        );

        assert_eq!(
            find_face_up_white_block(&frame, probe),
            Ok(Some(WhiteBlockHit {
                anchor: PixelPoint::new(23, 12),
                width: FACE_UP_WHITE_BLOCK_MIN_WIDTH + 5,
                height: 4,
            }))
        );
        assert_eq!(has_face_up_white_block(&frame, probe), Ok(true));
    }

    #[test]
    fn white_block_threshold_and_size_boundaries_are_exact() {
        let probe = PixelRect::new(10, 12, 60, FACE_UP_WHITE_SCAN_HEIGHT);

        let mut exact = rgba_frame(80, 40);
        set_white_block(
            &mut exact,
            probe.x,
            probe.y,
            FACE_UP_WHITE_BLOCK_MIN_WIDTH,
            FACE_UP_WHITE_SCAN_HEIGHT,
            FACE_UP_WHITE_CHANNEL_MINIMUM,
        );
        assert_eq!(
            find_face_up_white_block(&exact, probe),
            Ok(Some(WhiteBlockHit {
                anchor: PixelPoint::new(probe.x as i32, probe.y as i32),
                width: FACE_UP_WHITE_BLOCK_MIN_WIDTH,
                height: FACE_UP_WHITE_SCAN_HEIGHT,
            }))
        );

        let mut below_threshold = rgba_frame(80, 40);
        set_white_block(
            &mut below_threshold,
            probe.right() - FACE_UP_WHITE_BLOCK_MIN_WIDTH,
            probe.y,
            FACE_UP_WHITE_BLOCK_MIN_WIDTH,
            FACE_UP_WHITE_SCAN_HEIGHT,
            FACE_UP_WHITE_CHANNEL_MINIMUM - 1,
        );
        assert_eq!(find_face_up_white_block(&below_threshold, probe), Ok(None));

        let mut at_right_edge = rgba_frame(80, 40);
        let right_edge_start = probe.right() - FACE_UP_WHITE_BLOCK_MIN_WIDTH;
        set_white_block(
            &mut at_right_edge,
            right_edge_start,
            probe.y,
            FACE_UP_WHITE_BLOCK_MIN_WIDTH,
            FACE_UP_WHITE_SCAN_HEIGHT,
            FACE_UP_WHITE_CHANNEL_MINIMUM,
        );
        assert_eq!(
            find_face_up_white_block(&at_right_edge, probe),
            Ok(Some(WhiteBlockHit {
                anchor: PixelPoint::new(right_edge_start as i32, probe.y as i32),
                width: FACE_UP_WHITE_BLOCK_MIN_WIDTH,
                height: FACE_UP_WHITE_SCAN_HEIGHT,
            }))
        );
    }

    #[test]
    fn rejects_short_partial_and_speckled_white_patterns() {
        let probe = PixelRect::new(10, 12, 60, 4);

        let mut short = rgba_frame(80, 40);
        set_white_block(
            &mut short,
            23,
            12,
            FACE_UP_WHITE_BLOCK_MIN_WIDTH - 1,
            4,
            255,
        );
        assert_eq!(find_face_up_white_block(&short, probe), Ok(None));

        let mut partial_height = rgba_frame(80, 40);
        set_white_block(
            &mut partial_height,
            23,
            12,
            FACE_UP_WHITE_BLOCK_MIN_WIDTH + 5,
            3,
            255,
        );
        assert_eq!(find_face_up_white_block(&partial_height, probe), Ok(None));

        let mut speckled = rgba_frame(80, 40);
        for x in probe.x..probe.right() {
            set_pixel(&mut speckled, x, probe.y + x % probe.height, [255; 3]);
        }
        assert_eq!(find_face_up_white_block(&speckled, probe), Ok(None));
    }

    #[test]
    fn recognises_near_white_in_bgra_frames() {
        let mut frame = frame_with_format(80, 40, PixelFormat::Bgra8);
        let probe = PixelRect::new(10, 12, 60, 4);
        set_white_block(
            &mut frame,
            23,
            12,
            FACE_UP_WHITE_BLOCK_MIN_WIDTH,
            4,
            FACE_UP_WHITE_CHANNEL_MINIMUM,
        );

        assert_eq!(has_face_up_white_block(&frame, probe), Ok(true));
    }

    #[test]
    fn dialog_gold_requires_a_meaningful_gradient_area() {
        let mut frame = rgba_frame(20, 20);
        let probe = PixelRect::new(5, 5, 10, 10);

        for x in 5..13 {
            set_pixel(&mut frame, x, 5, [220, 170, 80]);
        }
        assert_eq!(has_dialog_gold_button(&frame, probe), Ok(true));

        let mut false_colour = rgba_frame(20, 20);
        for x in 5..13 {
            set_pixel(&mut false_colour, x, 5, [220, 215, 210]);
        }
        assert_eq!(has_dialog_gold_button(&false_colour, probe), Ok(false));
    }

    #[test]
    fn solver_progress_uses_the_black_right_third_as_another_board() {
        let mut frame = rgba_frame(1_920, 1_080);
        assert_eq!(
            detect_game_progress_for_profile(&frame, &crate::tripeaks::PROFILE),
            Ok(GameProgress::AnotherBoard)
        );

        for y in SOLVER_PROGRESS_RIGHT_PROBE.y..SOLVER_PROGRESS_RIGHT_PROBE.bottom() {
            for x in SOLVER_PROGRESS_RIGHT_PROBE.x..SOLVER_PROGRESS_RIGHT_PROBE.right() {
                set_pixel(&mut frame, x, y, [172, 142, 84]);
            }
        }

        assert_eq!(
            detect_game_progress_for_profile(&frame, &crate::tripeaks::PROFILE),
            Ok(GameProgress::GameComplete)
        );
    }

    #[test]
    fn rejects_invalid_layout_and_geometry() {
        let mut invalid_layout = rgba_frame(220, 220);
        invalid_layout.pixels.truncate(4);
        assert_eq!(
            has_gold_halo(&invalid_layout, card_regions()),
            Err(HaloDetectionError::InvalidFrameLayout)
        );

        let mut outside = card_regions();
        outside.halo_bounds = PixelRect::new(28, 26, 300, 168);
        assert_eq!(
            has_gold_halo(&rgba_frame(220, 220), outside),
            Err(HaloDetectionError::BoundsOutsideFrame)
        );

        let mut not_nested = card_regions();
        not_nested.halo_bounds = PixelRect::new(50, 50, 100, 100);
        assert_eq!(
            has_gold_halo(&rgba_frame(220, 220), not_nested),
            Err(HaloDetectionError::CardOutsideHalo)
        );

        assert_eq!(
            has_face_up_white_block(&invalid_layout, PixelRect::new(10, 10, 20, 4)),
            Err(HaloDetectionError::InvalidFrameLayout)
        );
        assert_eq!(
            has_face_up_white_block(&rgba_frame(20, 20), PixelRect::new(10, 10, 20, 4)),
            Err(HaloDetectionError::BoundsOutsideFrame)
        );
        assert_eq!(
            has_dialog_gold_button(&invalid_layout, PixelRect::new(10, 10, 20, 4)),
            Err(HaloDetectionError::InvalidFrameLayout)
        );
    }
}
