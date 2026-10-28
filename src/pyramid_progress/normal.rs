//! Exact native normal-series header calibration, separate from challenge counts.
//!
//! Complete original `Boards: 1/2/3 of 3` artwork and its `New` footer are calibrated.
//! There is no OCR, inferred replacement glyph, resize or alignment fallback.
//! Supported Pyramid/card
//! scenes, freshness, mode policy and input authority remain caller-owned.

use super::{BoardCount, HeaderPatch, matches_patch_with_tolerance};
use crate::{
    capture::CapturedFrame,
    geometry::PixelRect,
    parameters::{NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH},
};
use xsar::image_matching::FrameView;


#[cfg(test)]
mod tests;


/// Inclusive maximum channel difference of the complete header across seven native originals.
const HEADER_TOLERANCE: u8 = 12;


/// Inclusive maximum channel difference of the complete footer across the same seven originals.
const FOOTER_TOLERANCE: u8 = 10;


/// Exact full 160×32 RGB patch; words, both ordinal digits and all background pixels remain.
const HEADER_ONE_RGB: &[u8; 15_360] =
    include_bytes!("../../tests/fixtures/pyramid-standard-progress/header-one.rgb");


/// Complete ordinal-one appearance from the retained 2.9.152 post-NewGame refusal.
///
/// All 5120 pixels retain the original per-channel tolerance of 12. This is one
/// independently measured complete appearance, without per-pixel exceptions.
const HEADER_ONE_NEW_GAME_RGB: &[u8; 15_360] =
    include_bytes!("../../tests/fixtures/pyramid-normal-entry-153/header-one-new-game.rgb");


/// Complete original ordinal-two header, including words, digits and negative space.
const HEADER_TWO_RGB: &[u8; 15_360] =
    include_bytes!("../../tests/fixtures/pyramid-standard-series/header-2.rgb");


/// Complete measured ordinal-two raster from the retained 158 normal-entry refusal.
/// It uses the original header tolerance and does not alter another ordinal.
const HEADER_TWO_159_RGB: &[u8; 15_360] =
    include_bytes!("../../tests/fixtures/pyramid-recognition-159/normal-header-two.rgb");


/// Complete original ordinal-three header; no digit substitution is used.
const HEADER_THREE_RGB: &[u8; 15_360] =
    include_bytes!("../../tests/fixtures/pyramid-standard-series/header-3.rgb");


/// Exact full 58×31 RGB footer patch, including all negative space around `New`.
const FOOTER_NEW_RGB: &[u8; 5_394] =
    include_bytes!("../../tests/fixtures/pyramid-standard-progress/footer-new.rgb");


/// Complete footer measured in the retained Grandmaster frame refused by release 2.7.118.
///
/// Three pixels exceed the original sample's tolerance by one channel unit.
/// This independent native appearance preserves every pixel and the existing bound.
const FOOTER_NEW_GRANDMASTER_RGB: &[u8; 5_394] =
    include_bytes!("../../tests/fixtures/pyramid-objectives/footer-new-grandmaster.rgb");


/// Actual current ordinals one, two and three map to zero, one and two completed boards.
///
/// Each label has an independently retained native source; the target stays three.
const NORMAL_HEADERS: [(BoardCount, HeaderPatch); 3] = [
    (
        BoardCount {
            completed: 0,
            target: 3,
        },
        HeaderPatch {
            bounds: PixelRect::new(1_658, 46, 160, 32),
            rgb: HEADER_ONE_RGB,
        },
    ),
    (
        BoardCount {
            completed: 1,
            target: 3,
        },
        HeaderPatch {
            bounds: PixelRect::new(1_658, 46, 160, 32),
            rgb: HEADER_TWO_RGB,
        },
    ),
    (
        BoardCount {
            completed: 2,
            target: 3,
        },
        HeaderPatch {
            bounds: PixelRect::new(1_658, 46, 160, 32),
            rgb: HEADER_THREE_RGB,
        },
    ),
];


/// Complete measured appearances, grouped by their literal ordinal identity.
///
/// The three original source patches stay independently retained. Two coherent
/// appearances for ordinal one still denote one count, never two board changes.
const NORMAL_APPEARANCES: [(BoardCount, HeaderPatch); 5] = [
    NORMAL_HEADERS[0],
    NORMAL_HEADERS[1],
    NORMAL_HEADERS[2],
    (
        BoardCount {
            completed: 0,
            target: 3,
        },
        HeaderPatch {
            bounds: PixelRect::new(1_658, 46, 160, 32),
            rgb: HEADER_ONE_NEW_GAME_RGB,
        },
    ),
    (
        BoardCount {
            completed: 1,
            target: 3,
        },
        HeaderPatch {
            bounds: PixelRect::new(1_658, 46, 160, 32),
            rgb: HEADER_TWO_159_RGB,
        },
    ),
];


/// Positive measured `New` text is required along with the complete normal ordinal header.
const NEW_FOOTER: HeaderPatch = HeaderPatch {
    bounds: PixelRect::new(212, 998, 58, 31),
    rgb: FOOTER_NEW_RGB,
};


/// The same whole native footer in its separately measured Grandmaster appearance.
const GRANDMASTER_NEW_FOOTER: HeaderPatch = HeaderPatch {
    bounds: PixelRect::new(212, 998, 58, 31),
    rgb: FOOTER_NEW_GRANDMASTER_RGB,
};


/// Observe only calibrated complete native header/footer artwork without allocating on success.
///
/// Valid unsupported geometry or artwork returns `None`. Invalid frame layout
/// or bundled patch metadata returns an error before interpretation. This
/// observation neither identifies Pyramid nor grants input or terminal authority.
///
/// # Errors
///
/// Returns an error for invalid frame layout or invalid bundled patch metadata.
pub(super) fn observe(frame: &CapturedFrame) -> Result<Option<BoardCount>, String> {
    let view = frame
        .as_view()
        .map_err(|error| format!("Pyramid normal board-count frame layout is invalid: {error}"))?;
    if (view.width(), view.height()) != (NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT) {
        return Ok(None);
    }
    if !matches_patch_with_tolerance(view, NEW_FOOTER, FOOTER_TOLERANCE)?
        && !matches_patch_with_tolerance(view, GRANDMASTER_NEW_FOOTER, FOOTER_TOLERANCE)?
    {
        return Ok(None);
    }
    match_header(view, &NORMAL_APPEARANCES)
}


/// Read only a complete normal ordinal for contradiction checks in an already bound run.
///
/// This omits the New footer and cannot establish normal-game identity, source
/// authority or completion. Unsupported native artwork returns `None`.
///
/// # Errors
///
/// Returns an error for malformed pixels or bundled header metadata.
pub(super) fn observe_header(frame: &CapturedFrame) -> Result<Option<BoardCount>, String> {
    let view = frame
        .as_view()
        .map_err(|error| format!("Pyramid normal header frame layout is invalid: {error}"))?;
    if (view.width(), view.height()) != (NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT) {
        return Ok(None);
    }
    match_header(view, &NORMAL_APPEARANCES)
}


/// Return a unique ordinal from complete appearances; different ordinals refuse.
///
/// Same-ordinal complete appearances share one semantic identity. A second
/// different ordinal remains ambiguous and never chooses the nearest label.
fn match_header(
    view: FrameView<'_>,
    headers: &[(BoardCount, HeaderPatch)],
) -> Result<Option<BoardCount>, String> {
    let mut matched = None;
    for &(count, patch) in headers {
        if matches_patch_with_tolerance(view, patch, HEADER_TOLERANCE)? {
            if matched.is_some_and(|previous| previous != count) {
                return Ok(None);
            }
            matched = Some(count);
        }
    }
    Ok(matched)
}
