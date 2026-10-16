//! Input-free Level Up readiness observations over borrowed native pixels.
//!
//! A black run can appear while the modal is still entering, and the retained
//! pointer can itself supply a run. Before input, require consecutive matching
//! button geometry at its settled size and a current exact-black match inside
//! the button's central caption band. The invariant `You've Reached` caption
//! is measured for diagnostics only; its independent fade does not gate input.
//! Callers supply fresh captures within their finite normal and conditional
//! final-confirmation budgets. This module neither captures frames nor sends
//! or repeats input.

use xsar::image_matching::FrameView;

use crate::{
    capture::CapturedFrame,
    detector::HaloDetectionError,
    geometry::{PixelPoint, PixelRect},
    parameters::{NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH},
};

use super::{BLACK_RUN_LENGTH, profiles::SearchSpec, search};

/// Maximum RGB comparisons in one readiness observation, including a glyph rescan.
pub const MAX_READINESS_PIXEL_COMPARISONS: u64 = 150_000;

/// Stable horizontal position of the invariant caption in native captures.
const CAPTION_X: u32 = 835;

/// First allowed caption baseline origin, including the lower-rank layouts.
const CAPTION_FIRST_Y: u32 = 280;

/// Last allowed caption baseline origin, excluding variable level/rank artwork.
const CAPTION_LAST_Y: u32 = 380;

/// Opaque white core pixels sampled from the native Spider level-505 fixed caption.
///
/// Bits describe only the invariant words, never the level number or rank art.
/// Translation in Y covers every retained native Level Up layout. A 90% white
/// match permits antialiasing differences: the 13 positive native fixtures score
/// at least 93%, whereas the recorded fading Free Cell caption scores below 8%.
/// This mask contributes diagnostics only and never authorises or blocks input.
const CAPTION_ROWS: [[u64; 4]; 24] = [
    [0x0000000000380070, 0x7fc0000000000003, 0x0000000000000000, 0x0038000000000007],
    [0x80000000003c00f0, 0xffc0000000000003, 0x0000000000000003, 0x0038000000000007],
    [0x80000000001c00e0, 0xffc0000000000003, 0x000000000000000f, 0x0038000000000007],
    [0x80000000000c01e0, 0x01c0000000000003, 0x000000000000000f, 0x0038000000000007],
    [0x80000000000e01c0, 0x01c0000000000003, 0x000000000000001e, 0x0038000000000007],
    [0x80000000000603c0, 0x01c0000000000003, 0x000000000000001c, 0x0038000000000007],
    [0x8000000000070380, 0x01c0000000000003, 0x000000000000001c, 0x0038000000000007],
    [0x03807003f8030780, 0x01c00007c0380180, 0x03f0003f000f801c, 0x003878003e000f87],
    [0x0380700ffe038700, 0x01c0001ff03801c0, 0x07fc00ffc03fe01c, 0x0039fe007f801fc7],
    [0x0380701e0f018700, 0x01c00038381c0380, 0x041e01e0c070701e, 0x003b0f00e1c03c27],
    [0x0380703c0781ce00, 0x01c000301c1c0380, 0x000f01c00060380e, 0x003c0781c0e0381f],
    [0x038070380380ce00, 0x83c000700e0c0300, 0x0007838000e01807, 0x003c03838070380f],
    [0x0380703801c0fc00, 0xffc000700e0e0700, 0x0003838000e01c01, 0x003801c380707807],
    [0x0380707001c07c00, 0xffc000600e0e0700, 0x0003838000c01c00, 0x003801c380307007],
    [0x0380707001c07800, 0xc1c0007ffe060600, 0x000383fe00fffc01, 0x003801c3fff87007],
    [0x0380707001c03800, 0x81c0007ffe070e00, 0x0001c39fc1fffc03, 0x003801c3fff87007],
    [0x0380707001c03800, 0x01c000000e030e00, 0x0001c380e0001c07, 0x003801c000387007],
    [0x0380707001c03800, 0x01c000000e030c00, 0x0003838070001c0e, 0x003801c000387007],
    [0x0380703801c03800, 0x01c000000e039c00, 0x0003838070001c0e, 0x003801c000707007],
    [0x0380703803803800, 0x01c000000e019c00, 0x000383c070001c1c, 0x003c03c000707007],
    [0x03c0703c03803800, 0x01c000001c019800, 0x000703c07000383c, 0x003c038000707007],
    [0x03a0f01e0f003800, 0x01c000203c019800, 0x000f03a0f0407838, 0x003a078101e07007],
    [0x039fe00ffe003800, 0x01c0003ff800f000, 0x07fe039fe07ff078, 0x0039ff01ffc07007],
    [0x038fc003f8003800, 0x01c0001fe000f000, 0x03f8038fc03fc070, 0x0038fe00ff807007],
];

/// Caption holes and gaps that remain dark in all retained positive native fixtures.
///
/// These negative samples prevent a uniformly bright patch or firework washout
/// from satisfying the white-stroke mask by itself.
const CAPTION_DARK_PROBES: [(u32, u32); 20] = [
    (850, 20), (862, 14), (874, 8), (886, 14), (898, 8),
    (910, 8), (922, 14), (934, 20), (946, 14), (958, 14),
    (970, 14), (982, 20), (994, 2), (1006, 14), (1018, 14),
    (1030, 14), (1042, 14), (1054, 20), (1066, 8), (1078, 25),
];

/// Consecutive button observations belonging to one expected Level Up lookup.
#[derive(Default)]
pub struct OkReadiness {
    /// Geometry from the immediately preceding observation, including fading text.
    previous_button: Option<PixelRect>,
}

/// Evidence from one frame; input is permitted only when `ready` is true.
#[derive(Clone, Copy, Debug)]
pub struct OkObservation {
    /// Settled consecutive geometry and the current exact-black glyph passed.
    pub ready: bool,
    /// Only fresh geometry agreement remains after a settled-size current glyph.
    /// This permits a shorter input-free confirmation wait, never a click.
    pub confirmation_pending: bool,
    /// Latest measured central glyph point, or `(0, 0)` while input is withheld.
    pub point: PixelPoint,
    /// Stable reason for accepting or withholding input in persistent logs.
    pub reason: &'static str,
    /// Measured body, retained for foreground-modal policy even while entering.
    pub button: Option<PixelRect>,
    /// Diagnostic only: the fixed caption meets the calibrated white threshold.
    pub caption_present: bool,
    /// Diagnostic caption origin when its white-stroke and dark-gap mask matches.
    pub caption_y: Option<u32>,
    /// Current body agrees with the immediately preceding observed geometry.
    pub geometry_stable: bool,
    /// Central caption band used to reject pointer runs below the glyphs.
    pub glyph_bounds: Option<PixelRect>,
    /// Actual RGB comparisons in geometry, caption checks and any glyph rescan.
    pub pixels_examined: u64,
}

impl OkReadiness {
    /// Begin independent readiness evidence for one Level Up acknowledgement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inspect a caller-supplied fresh native capture without copying its pixels.
    ///
    /// Missing black glyphs may still provide geometry evidence. Missing body
    /// geometry resets consecutive evidence. A current valid glyph is required
    /// for readiness; a previous coordinate is never retained or replayed.
    /// The caption measurements remain logged without affecting this decision.
    pub fn observe(
        &mut self,
        frame: &CapturedFrame,
        raw_point: PixelPoint,
    ) -> Result<OkObservation, HaloDetectionError> {
        let view = frame.as_view().map_err(super::scan_error)?;
        if frame.width != NOMINAL_FRAME_WIDTH || frame.height != NOMINAL_FRAME_HEIGHT {
            return Err(HaloDetectionError::InvalidFrameLayout);
        }
        let mut pixels_examined = 0;
        let button = measure_button(view, &mut pixels_examined);
        let geometry_stable = match (self.previous_button, button) {
            (Some(previous), Some(current)) => same_geometry(previous, current),
            _ => false,
        };
        self.previous_button = button;
        let caption_y = if button.is_some() {
            find_caption(view, &mut pixels_examined)
        } else {
            None
        };
        let caption_present = caption_y.is_some();
        let glyph_bounds = button.map(|bounds| {
            let centre_y = bounds.y + (bounds.height - 1) / 2;
            PixelRect::new(930, centre_y - 17, 25, 35)
        });
        let mut glyph_point = PixelPoint::new(0, 0);
        if raw_point != PixelPoint::new(0, 0)
            && let Some(bounds) = glyph_bounds
        {
            if bounds.contains(raw_point) {
                glyph_point = raw_point;
            } else {
                let result = search(frame, SearchSpec { bounds, run_length: BLACK_RUN_LENGTH })?;
                pixels_examined += result.pixels_examined;
                glyph_point = result.point;
            }
        }
        let reason = match button {
            None => "missing-button-body",
            Some(bounds) if !settled_size(bounds) => "entrance-button-size",
            Some(_) if !geometry_stable => "button-geometry-changing",
            Some(_) if glyph_point == PixelPoint::new(0, 0) => "missing-current-caption-glyph",
            Some(_) => "settled-button-and-glyph",
        };
        let ready = reason == "settled-button-and-glyph";
        let confirmation_pending = !ready
            && !geometry_stable
            && glyph_point != PixelPoint::new(0, 0)
            && button.is_some_and(settled_size);
        Ok(OkObservation {
            ready,
            confirmation_pending,
            point: if ready { glyph_point } else { PixelPoint::new(0, 0) },
            reason,
            button,
            caption_present,
            caption_y,
            geometry_stable,
            glyph_bounds,
            pixels_examined,
        })
    }
}

/// Count one checked RGB observation inside the already validated native frame.
fn rgb(view: FrameView<'_>, x: u32, y: u32, work: &mut u64) -> [u8; 3] {
    *work += 1;
    // Every caller uses fixed native bounds or edges measured within them.
    view.pixel_rgb(x, y).unwrap_or([255, 0, 255])
}

/// Broad cream/gold body predicate that permits native edge antialiasing.
fn is_gold([red, green, blue]: [u8; 3]) -> bool {
    red >= 145 && green >= 85 && red >= green && u16::from(green) >= u16::from(blue) + 15
}

/// Measure a button's unobscured right-hand strip and upper horizontal body edge.
fn measure_button(view: FrameView<'_>, work: &mut u64) -> Option<PixelRect> {
    let mut y = 700;
    while y < 880 {
        if !is_gold(rgb(view, 1050, y, work)) {
            y += 1;
            continue;
        }
        let top = y;
        y += 1;
        while y < 880 && is_gold(rgb(view, 1050, y, work)) {
            y += 1;
        }
        let height = y - top;
        if !(55..=85).contains(&height) {
            continue;
        }
        let edge_y = top + 15;
        let mut left = 1050;
        while left > 810 && is_gold(rgb(view, left - 1, edge_y, work)) {
            left -= 1;
        }
        let mut right = 1051;
        while right < 1110 && is_gold(rgb(view, right, edge_y, work)) {
            right += 1;
        }
        let width = right - left;
        if (180..=270).contains(&width) {
            return Some(PixelRect::new(left, top, width, height));
        }
    }
    None
}

/// Settled native body dimensions, independently of rank-dependent vertical offset.
fn settled_size(bounds: PixelRect) -> bool {
    (235..=239).contains(&bounds.width) && (68..=73).contains(&bounds.height)
}

/// One-pixel native edge antialiasing is allowed without hiding entrance zooming.
fn same_geometry(previous: PixelRect, current: PixelRect) -> bool {
    previous.x.abs_diff(current.x) <= 1
        && previous.y.abs_diff(current.y) <= 1
        && previous.width.abs_diff(current.width) <= 1
        && previous.height.abs_diff(current.height) <= 1
}

/// Measure the diagnostic bright caption while permitting its native Y translations.
fn find_caption(view: FrameView<'_>, work: &mut u64) -> Option<u32> {
    let total: u32 = CAPTION_ROWS.iter().flatten().map(|bits| bits.count_ones()).sum();
    let allowed_misses = total / 10;
    'candidate: for top in CAPTION_FIRST_Y..=CAPTION_LAST_Y {
        let mut misses = 0;
        for (row, chunks) in CAPTION_ROWS.iter().enumerate() {
            for (chunk, &mask) in chunks.iter().enumerate() {
                let mut remaining = mask;
                while remaining != 0 {
                    let column = remaining.trailing_zeros();
                    remaining &= remaining - 1;
                    let [red, green, blue] = rgb(
                        view, CAPTION_X + chunk as u32 * 64 + column, top + row as u32, work,
                    );
                    if red < 235 || green < 235 || blue < 235 {
                        misses += 1;
                        if misses > allowed_misses {
                            continue 'candidate;
                        }
                    }
                }
            }
        }
        let mut dark = 0;
        for &(x, row) in &CAPTION_DARK_PROBES {
            let [red, green, blue] = rgb(view, x, top + row, work);
            if red <= 195 && green <= 195 && blue <= 195 {
                dark += 1;
            }
        }
        if dark >= 18 {
            return Some(top);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    //! Native entrance/fading/settled captures and independent layout regressions.

    use super::*;
    use crate::{capture::{PixelFormat, decode_png}, game::GameMode};
    use super::super::{Control, profiles::for_game};

    /// Decode an unchanged retained native fixture.
    fn fixture(name: &str) -> CapturedFrame {
        let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        decode_png(&std::fs::read(path).unwrap()).unwrap()
    }

    /// Supply the same shared exact-black scanner used by the controller.
    fn raw(frame: &CapturedFrame, game: GameMode) -> PixelPoint {
        search(frame, for_game(game).spec(Control::Ok)).unwrap().point
    }

    /// Entrance geometry and the missing central glyph withhold input until frame three.
    #[test]
    fn recorded_freecell_entrance_fading_and_settled_frames_need_three_observations() {
        let entrance = fixture("game-win-readiness/early.png");
        let fading = fixture("game-win-readiness/fading.png");
        let settled = fixture("game-win-readiness/settled.png");
        let mut readiness = OkReadiness::new();
        let first = readiness.observe(&entrance, raw(&entrance, GameMode::FreeCell)).unwrap();
        assert_eq!(first.button, Some(PixelRect::new(848, 709, 219, 65)));
        assert_eq!(first.reason, "entrance-button-size");
        assert!(!first.ready && !first.caption_present && !first.geometry_stable);
        assert!(!first.confirmation_pending);
        let second = readiness.observe(&fading, raw(&fading, GameMode::FreeCell)).unwrap();
        assert_eq!(second.button, Some(PixelRect::new(839, 723, 237, 71)));
        assert!(!second.ready && !second.caption_present && !second.geometry_stable);
        assert!(!second.confirmation_pending);
        assert_eq!(raw(&settled, GameMode::FreeCell), PixelPoint::new(945, 792));
        let third = readiness.observe(&settled, raw(&settled, GameMode::FreeCell)).unwrap();
        assert!(third.ready && third.caption_present && third.geometry_stable);
        assert!(!third.confirmation_pending);
        assert_eq!(third.point, PixelPoint::new(941, 770));
        assert_eq!(third.caption_y, Some(350));
        for observation in [first, second, third] {
            assert!(observation.pixels_examined <= MAX_READINESS_PIXEL_COMPARISONS);
        }
    }

    /// Stable body with only a pointer run never substitutes for a current central glyph.
    #[test]
    fn retained_pointer_without_a_current_central_glyph_never_proves_readiness() {
        let fading = fixture("game-win-readiness/fading.png");
        let mut readiness = OkReadiness::new();
        for attempt in 0..3 {
            let observation = readiness.observe(&fading, raw(&fading, GameMode::FreeCell)).unwrap();
            assert!(!observation.ready);
            assert!(!observation.confirmation_pending);
            assert!(!observation.caption_present);
            assert_eq!(observation.point, PixelPoint::new(0, 0));
            if attempt > 0 {
                assert_eq!(observation.reason, "missing-current-caption-glyph");
            }
        }
    }

    /// The exact failed Spider sequence accepts its third frame despite a grey header.
    #[test]
    fn recorded_spider_transition_accepts_a_current_black_glyph_with_a_grey_header() {
        let score = fixture("spider-game-win-20261015/score.png");
        let entering = fixture("spider-game-win-20261015/entering.png");
        let near_black = fixture("spider-game-win-20261015/near-black.png");
        let grey_caption = fixture("spider-game-win-20261015/grey-caption.png");
        let mut entry = OkReadiness::new();
        let score_observation = entry.observe(&score, raw(&score, GameMode::Spider)).unwrap();
        assert_eq!(score_observation.button, None);
        assert!(!score_observation.ready);

        let mut readiness = OkReadiness::new();
        let first = readiness.observe(&entering, raw(&entering, GameMode::Spider)).unwrap();
        assert_eq!(first.reason, "missing-button-body");
        assert!(!first.ready);
        let second = readiness.observe(&near_black, raw(&near_black, GameMode::Spider)).unwrap();
        assert_eq!(raw(&near_black, GameMode::Spider), PixelPoint::new(0, 0));
        assert_eq!(second.button, Some(PixelRect::new(839, 723, 237, 71)));
        assert!(!second.ready && !second.geometry_stable);
        assert!(!second.confirmation_pending);
        let third = readiness.observe(&grey_caption, raw(&grey_caption, GameMode::Spider)).unwrap();
        assert_eq!(raw(&grey_caption, GameMode::Spider), PixelPoint::new(943, 770));
        assert!(third.ready && third.geometry_stable);
        assert!(!third.confirmation_pending);
        assert_eq!(third.point, PixelPoint::new(943, 770));
        assert_eq!(third.reason, "settled-button-and-glyph");
        assert!(!third.caption_present);
        assert_eq!(third.caption_y, None);
        for observation in [score_observation, first, second, third] {
            assert!(observation.pixels_examined <= MAX_READINESS_PIXEL_COMPARISONS);
        }
    }

    /// Native RGB(1,1,1) is still rejected after geometry settles; exact black is required.
    #[test]
    fn native_spider_near_black_cannot_replace_the_exact_black_run() {
        let near_black = fixture("spider-game-win-20261015/near-black.png");
        let black = fixture("spider-game-win-20261015/grey-caption.png");
        let near_view = near_black.as_view().unwrap();
        let black_view = black.as_view().unwrap();
        for x in 943..948 {
            assert_eq!(near_view.pixel_rgb(x, 770), Some([1, 1, 1]));
            assert_eq!(black_view.pixel_rgb(x, 770), Some([0, 0, 0]));
        }
        let mut readiness = OkReadiness::new();
        for attempt in 0..3 {
            let point = raw(&near_black, GameMode::Spider);
            assert_eq!(point, PixelPoint::new(0, 0));
            let observation = readiness.observe(&near_black, point).unwrap();
            assert!(!observation.ready);
            assert!(!observation.confirmation_pending);
            if attempt > 0 {
                assert!(observation.geometry_stable);
                assert_eq!(observation.reason, "missing-current-caption-glyph");
            }
        }
    }

    /// Every retained Level Up layout requires confirmation and preserves its glyph point.
    #[test]
    fn all_native_level_up_layouts_confirm_without_fixing_the_button_y() {
        let cases = [
            (GameMode::FreeCell, "freecell-FC10.png"),
            (GameMode::FreeCell, "freecell-FC22.png"),
            (GameMode::FreeCell, "freecell-FC23.png"),
            (GameMode::Spider, "spider-SP20.png"),
            (GameMode::Spider, "spider-level-up-505.png"),
            (GameMode::Klondike, "klondike/K27.png"),
            (GameMode::Klondike, "klondike/K49.png"),
            (GameMode::Klondike, "klondike/K52.png"),
            (GameMode::Klondike, "klondike/K67.png"),
            (GameMode::Klondike, "klondike/K86.png"),
            (GameMode::Klondike, "klondike/K91.png"),
            (GameMode::TriPeaks, "tripeaks-TP02.png"),
            (GameMode::TriPeaks, "tripeaks-TP03.png"),
        ];
        for (game, name) in cases {
            let frame = fixture(name);
            let point = raw(&frame, game);
            let mut readiness = OkReadiness::new();
            let first = readiness.observe(&frame, point).unwrap();
            assert!(!first.ready && first.caption_present, "{name}: {first:?}");
            assert!(first.confirmation_pending, "{name}: {first:?}");
            // Identical pixels represent a second independent capture of a static modal.
            let second = readiness.observe(&frame, point).unwrap();
            assert!(second.ready, "{name}: {second:?}");
            assert!(!second.confirmation_pending, "{name}: {second:?}");
            assert_eq!(second.point, point, "{name}");
            assert!(second.pixels_examined <= MAX_READINESS_PIXEL_COMPARISONS);
        }
    }

    /// Score/menu frames cannot provide a foreground Level Up body or ready input.
    #[test]
    fn native_non_level_up_frames_never_authorise_ok() {
        let cases = [
            (GameMode::FreeCell, "freecell-FC09.png"),
            (GameMode::FreeCell, "freecell-FC11.png"),
            (GameMode::FreeCell, "freecell-FC12.png"),
            (GameMode::Spider, "spider-SP17.png"),
            (GameMode::Klondike, "klondike/K53.png"),
            (GameMode::TriPeaks, "tripeaks-TP01.png"),
            (GameMode::TriPeaks, "tripeaks-play-20261014.png"),
        ];
        for (game, name) in cases {
            let frame = fixture(name);
            let mut readiness = OkReadiness::new();
            for _ in 0..3 {
                let observation = readiness.observe(&frame, raw(&frame, game)).unwrap();
                assert!(!observation.ready, "{name}: {observation:?}");
                assert!(!observation.confirmation_pending, "{name}: {observation:?}");
                assert_eq!(observation.button, None, "{name}: {observation:?}");
            }
        }
    }

    /// Missing body resets consecutive evidence instead of preserving an old point.
    #[test]
    fn absent_body_resets_geometry_before_a_later_valid_modal() {
        let modal = fixture("freecell-FC10.png");
        let menu = fixture("freecell-FC11.png");
        let point = raw(&modal, GameMode::FreeCell);
        let mut readiness = OkReadiness::new();
        assert!(!readiness.observe(&modal, point).unwrap().ready);
        assert!(!readiness.observe(&menu, PixelPoint::new(0, 0)).unwrap().ready);
        let after_gap = readiness.observe(&modal, point).unwrap();
        assert!(!after_gap.ready && !after_gap.geometry_stable);
        assert!(readiness.observe(&modal, point).unwrap().ready);
    }

    /// Geometry can settle through missing glyphs but only a current match permits input.
    #[test]
    fn two_missing_glyph_observations_allow_only_current_third_frame_success() {
        let frame = fixture("freecell-FC10.png");
        let mut readiness = OkReadiness::new();
        for _ in 0..2 {
            let observation = readiness.observe(&frame, PixelPoint::new(0, 0)).unwrap();
            assert!(!observation.ready);
            assert!(!observation.confirmation_pending);
            assert_eq!(observation.point, PixelPoint::new(0, 0));
        }
        let third = readiness.observe(&frame, raw(&frame, GameMode::FreeCell)).unwrap();
        assert!(third.ready && third.geometry_stable);
    }

    /// An independently washed-out header remains diagnostic and does not block a valid OK.
    #[test]
    fn fully_white_header_is_not_caption_evidence_and_does_not_gate_ok() {
        let mut frame = fixture("freecell-FC10.png");
        for y in 280..405 {
            for x in 835..1087 {
                let offset = y * frame.stride + x * 4;
                frame.pixels[offset..offset + 3].fill(255);
            }
        }
        let mut readiness = OkReadiness::new();
        let point = raw(&frame, GameMode::FreeCell);
        readiness.observe(&frame, point).unwrap();
        let second = readiness.observe(&frame, point).unwrap();
        assert!(second.geometry_stable && !second.caption_present && second.ready);
        assert_eq!(second.point, point);
        assert_eq!(second.reason, "settled-button-and-glyph");
        assert!(second.pixels_examined <= MAX_READINESS_PIXEL_COMPARISONS);
    }

    /// Checked views preserve padded rows and reject nonnative or malformed storage.
    #[test]
    fn checked_native_layout_and_padding_are_required() {
        let frame = fixture("freecell-FC10.png");
        let mut padded = CapturedFrame {
            width: frame.width,
            height: frame.height,
            stride: frame.stride + 16,
            format: PixelFormat::Rgba8,
            pixels: vec![0; (frame.stride + 16) * frame.height as usize],
        };
        for row in 0..frame.height as usize {
            padded.pixels[row * padded.stride..row * padded.stride + frame.stride]
                .copy_from_slice(&frame.pixels[row * frame.stride..(row + 1) * frame.stride]);
        }
        let point = raw(&padded, GameMode::FreeCell);
        let mut readiness = OkReadiness::new();
        assert!(!readiness.observe(&padded, point).unwrap().ready);
        assert!(readiness.observe(&padded, point).unwrap().ready);
        padded.width -= 1;
        assert_eq!(readiness.observe(&padded, point).unwrap_err(), HaloDetectionError::InvalidFrameLayout);
        padded.width += 1;
        padded.pixels.pop();
        assert_eq!(readiness.observe(&padded, point).unwrap_err(), HaloDetectionError::InvalidFrameLayout);
    }
}
