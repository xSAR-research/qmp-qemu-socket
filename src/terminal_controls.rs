//! Shared bounded searches for controls in an already expected terminal/menu stage.
//!
//! A five-pixel black run supplies a click coordinate, not independent proof of
//! a game win. Bounds stay in native full-frame pixels and exclude the origin,
//! reserving `(0, 0)` for no match. The logical crop borrows the existing frame.

use std::time::{Duration, Instant};

use xsar::image_matching::{MatchError, find_black_run_bottom_up};

use crate::{
    capture::CapturedFrame,
    detector::HaloDetectionError,
    geometry::PixelPoint,
    parameters::{NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH},
};


/// Game-specific input parameters kept outside the mathematical scanner.
pub mod profiles;

use profiles::SearchSpec;


/// Exact-black pixels required consecutively on one horizontal row.
pub const BLACK_RUN_LENGTH: u32 = 5;


/// Control identity supplied by the calling game's expected stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    /// Optional Level Up acknowledgement.
    Ok,
    /// Start another game after Congratulations.
    NewGame,
    /// Start the selected game and difficulty.
    Play,
}


impl Control {
    /// Stable human-readable name for visible and persistent logs.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::NewGame => "New Game",
            Self::Play => "Play",
        }
    }


}


/// One completed bounded scan, including only pixels actually visited.
#[derive(Clone, Copy, Debug)]
pub struct SearchResult {
    /// Leftmost pixel of the first qualifying run, or `(0, 0)` on a miss.
    pub point: PixelPoint,
    /// Pixel comparisons, stopping when the specified black run is established.
    pub pixels_examined: u64,
    /// Rows entered, including the partially searched matching row.
    pub rows_examined: u32,
    /// Monotonic elapsed duration of the scanner call alone.
    pub scan_time: Duration,
}


/// Return the expected control's measured click point, or `(0, 0)`.
pub fn control_point(
    frame: &CapturedFrame,
    spec: SearchSpec,
) -> Result<PixelPoint, HaloDetectionError> {
    Ok(search(frame, spec)?.point)
}


/// Search bottom-to-top rows and left-to-right pixels, with no image copy.
///
/// The existing xSAR helper stops at the specified exact-black RGB run length.
/// Counts are derived from that returned position and the known traversal;
/// counting never performs a second scan. Alpha does not affect recognition.
/// Native geometry is required because the result is used for guest input.
pub fn search(
    frame: &CapturedFrame,
    spec: SearchSpec,
) -> Result<SearchResult, HaloDetectionError> {
    let view = frame.as_view().map_err(scan_error)?;
    if frame.width != NOMINAL_FRAME_WIDTH || frame.height != NOMINAL_FRAME_HEIGHT {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }

    let bounds = spec.bounds;
    let started = Instant::now();
    let point = find_black_run_bottom_up(view, bounds, spec.run_length).map_err(scan_error)?;
    let scan_time = started.elapsed();
    let (pixels_examined, rows_examined) = if spec.run_length > bounds.width {
        // xSAR rejects an impossible run before visiting any image pixels.
        (0, 0)
    } else if point == PixelPoint::new(0, 0) {
        (u64::from(bounds.width) * u64::from(bounds.height), bounds.height)
    } else {
        // The helper has checked the point against the supplied native bounds.
        let full_rows = bounds.y + bounds.height - 1 - point.y as u32;
        let final_row_pixels = point.x as u32 - bounds.x + spec.run_length;
        (
            u64::from(full_rows) * u64::from(bounds.width) + u64::from(final_row_pixels),
            full_rows + 1,
        )
    };
    Ok(SearchResult { point, pixels_examined, rows_examined, scan_time })
}


/// Preserve the existing application's checked detector-error vocabulary.
fn scan_error(error: MatchError) -> HaloDetectionError {
    match error {
        MatchError::InvalidFrameLayout => HaloDetectionError::InvalidFrameLayout,
        MatchError::EmptyBounds | MatchError::ZeroMinimum => HaloDetectionError::EmptyBounds,
        MatchError::BoundsOutsideFrame | MatchError::AnchorOutsideCoordinateRange => {
            HaloDetectionError::BoundsOutsideFrame
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{capture::{PixelFormat, decode_png}, game::GameMode, geometry::PixelRect};
    use profiles::{for_game, FREECELL_CONTROLS, KLONDIKE_CONTROLS, SPIDER_CONTROLS};


    /// White native storage avoids accidental matches outside the planted run.
    fn blank_frame() -> CapturedFrame {
        let stride = NOMINAL_FRAME_WIDTH as usize * 4;
        CapturedFrame {
            width: NOMINAL_FRAME_WIDTH,
            height: NOMINAL_FRAME_HEIGHT,
            stride,
            format: PixelFormat::Rgba8,
            pixels: vec![255; stride * NOMINAL_FRAME_HEIGHT as usize],
        }
    }


    fn black_run(frame: &mut CapturedFrame, x: u32, y: u32, length: u32) {
        for column in x..x + length {
            let offset = y as usize * frame.stride + column as usize * 4;
            frame.pixels[offset..offset + 3].fill(0);
        }
    }


    /// Every profile has deliberate valid bounds, including those without native fixtures.
    #[test]
    fn all_game_profiles_have_nonempty_bounded_non_origin_searches() {
        for game in GameMode::AVAILABLE {
            for control in [Control::Ok, Control::NewGame, Control::Play] {
                let spec = for_game(game).spec(control);
                let bounds = spec.bounds;
                assert!(bounds.width > 0 && bounds.height > 0, "{game:?} / {control:?}");
                assert!(bounds.right() <= NOMINAL_FRAME_WIDTH);
                assert!(bounds.bottom() <= NOMINAL_FRAME_HEIGHT);
                assert!(!bounds.contains(PixelPoint::new(0, 0)));
                assert_eq!(spec.run_length, BLACK_RUN_LENGTH);
                assert!(spec.run_length <= bounds.width);
            }
        }
    }


    /// First and last legal runs count only pixels needed to establish each match.
    #[test]
    fn all_game_metrics_cover_first_last_and_missing_runs() {
        for game in GameMode::AVAILABLE {
            for control in [Control::Ok, Control::NewGame, Control::Play] {
                let spec = for_game(game).spec(control);
                let bounds = spec.bounds;
                let mut first = blank_frame();
                black_run(&mut first, bounds.x, bounds.y + bounds.height - 1, spec.run_length);
                let result = search(&first, spec).unwrap();
                assert_eq!(result.point, PixelPoint::new(bounds.x as i32,
                    (bounds.y + bounds.height - 1) as i32));
                assert_eq!(result.pixels_examined, u64::from(spec.run_length));
                assert_eq!(result.rows_examined, 1);

                let mut last = blank_frame();
                black_run(&mut last, bounds.x + bounds.width - spec.run_length, bounds.y, spec.run_length);
                let result = search(&last, spec).unwrap();
                assert_eq!(result.point, PixelPoint::new(
                    (bounds.x + bounds.width - spec.run_length) as i32, bounds.y as i32));
                assert_eq!(result.pixels_examined, u64::from(bounds.width) * u64::from(bounds.height));
                assert_eq!(result.rows_examined, bounds.height);

                let result = search(&blank_frame(), spec).unwrap();
                assert_eq!(result.point, PixelPoint::new(0, 0));
                assert_eq!(result.pixels_examined, u64::from(bounds.width) * u64::from(bounds.height));
                assert_eq!(result.rows_examined, bounds.height);
            }
        }
    }


    /// The scanner uses supplied dimensions and run length without control identity.
    #[test]
    fn mathematical_spec_controls_run_length_and_impossible_run_work() {
        let mut frame = blank_frame();
        let bounds = PixelRect::new(50, 60, 10, 4);
        black_run(&mut frame, 52, 63, 7);
        for run_length in [2, 5, 7] {
            let result = search(&frame, SearchSpec { bounds, run_length }).unwrap();
            assert_eq!(result.point, PixelPoint::new(52, 63));
            assert_eq!(result.pixels_examined, u64::from(2 + run_length));
            assert_eq!(result.rows_examined, 1);
        }
        let result = search(&frame, SearchSpec { bounds, run_length: 11 }).unwrap();
        assert_eq!(result.point, PixelPoint::new(0, 0));
        assert_eq!((result.pixels_examined, result.rows_examined), (0, 0));
        assert_eq!(search(&frame, SearchSpec { bounds, run_length: 0 }).unwrap_err(), HaloDetectionError::EmptyBounds);
    }


    /// Native click coordinates never silently apply to resized or broken storage.
    #[test]
    fn rejects_non_native_geometry_and_malformed_storage() {
        let mut frame = blank_frame();
        frame.width = 1919;
        assert!(frame.is_layout_valid());
        assert_eq!(search(&frame, SPIDER_CONTROLS.ok).unwrap_err(), HaloDetectionError::InvalidFrameLayout);

        let mut frame = blank_frame();
        frame.pixels.pop();
        assert_eq!(search(&frame, SPIDER_CONTROLS.ok).unwrap_err(), HaloDetectionError::InvalidFrameLayout);

        let mut frame = blank_frame();
        frame.stride = 1;
        assert_eq!(search(&frame, SPIDER_CONTROLS.ok).unwrap_err(), HaloDetectionError::InvalidFrameLayout);
    }


    /// Native positive/negative pairs use their own game's constants for all 29 captures.
    #[test]
    fn native_control_fixtures_match_only_their_expected_per_game_caption() {
        let cases = [
            (GameMode::FreeCell, "freecell-FC09.png", None),
            (GameMode::FreeCell, "freecell-FC10.png", Some((Control::Ok, PixelPoint::new(941, 822), 716, 29))),
            (GameMode::FreeCell, "freecell-FC11.png", Some((Control::NewGame, PixelPoint::new(810, 861), 1247, 19))),
            (GameMode::FreeCell, "freecell-FC12.png", Some((Control::Play, PixelPoint::new(680, 762), 570, 29))),
            (GameMode::FreeCell, "freecell-FC15.png", Some((Control::NewGame, PixelPoint::new(863, 861), 1300, 19))),
            (GameMode::FreeCell, "freecell-FC22.png", Some((Control::Ok, PixelPoint::new(942, 804), 1167, 47))),
            (GameMode::FreeCell, "freecell-FC23.png", Some((Control::Ok, PixelPoint::new(943, 835), 393, 16))),
            (GameMode::Spider, "spider-SP15.png", None),
            (GameMode::Spider, "spider-SP17.png", Some((Control::NewGame, PixelPoint::new(863, 862), 1231, 18))),
            (GameMode::Spider, "spider-SP18.png", Some((Control::Play, PixelPoint::new(679, 862), 569, 29))),
            (GameMode::Spider, "spider-SP20.png", Some((Control::Ok, PixelPoint::new(942, 803), 1192, 48))),
            (GameMode::Spider, "spider-level-up-505.png", Some((Control::Ok, PixelPoint::new(944, 771), 1994, 80))),
            (GameMode::Klondike, "klondike/K26.png", None),
            (GameMode::Klondike, "klondike/K27.png", Some((Control::Ok, PixelPoint::new(943, 823), 693, 28))),
            (GameMode::Klondike, "klondike/K28.png", Some((Control::NewGame, PixelPoint::new(863, 862), 1231, 18))),
            (GameMode::Klondike, "klondike/K29.png", Some((Control::Play, PixelPoint::new(680, 860), 610, 31))),
            (GameMode::Klondike, "klondike/K43.png", None),
            (GameMode::Klondike, "klondike/K49.png", Some((Control::Ok, PixelPoint::new(942, 822), 717, 29))),
            (GameMode::Klondike, "klondike/K52.png", Some((Control::Ok, PixelPoint::new(942, 822), 717, 29))),
            (GameMode::Klondike, "klondike/K53.png", Some((Control::NewGame, PixelPoint::new(862, 861), 1299, 19))),
            (GameMode::Klondike, "klondike/K67.png", Some((Control::Ok, PixelPoint::new(943, 828), 568, 23))),
            (GameMode::Klondike, "klondike/K86.png", Some((Control::Ok, PixelPoint::new(942, 829), 542, 22))),
            (GameMode::Klondike, "klondike/K91.png", Some((Control::Ok, PixelPoint::new(942, 804), 1167, 47))),
            (GameMode::Klondike, "klondike/K92.png", Some((Control::NewGame, PixelPoint::new(863, 862), 1231, 18))),
            (GameMode::Klondike, "klondike/K93.png", Some((Control::Play, PixelPoint::new(680, 861), 590, 30))),
            (GameMode::TriPeaks, "tripeaks-TP01.png", None),
            (GameMode::TriPeaks, "tripeaks-TP02.png", Some((Control::Ok, PixelPoint::new(943, 770), 2018, 81))),
            (GameMode::TriPeaks, "tripeaks-TP03.png", Some((Control::Ok, PixelPoint::new(944, 771), 1994, 80))),
            (GameMode::TriPeaks, "tripeaks-play-20261014.png", Some((Control::Play, PixelPoint::new(680, 905), 710, 36))),
        ];
        for (game, fixture, expected) in cases {
            let path = format!("{}/tests/fixtures/{fixture}", env!("CARGO_MANIFEST_DIR"));
            let frame = decode_png(&std::fs::read(path).unwrap()).unwrap();
            for control in [Control::Ok, Control::NewGame, Control::Play] {
                let spec = for_game(game).spec(control);
                let result = search(&frame, spec).unwrap();
                let (point, pixels, rows) = match expected {
                    Some((expected_control, point, pixels, rows)) if expected_control == control => {
                        (point, pixels, rows)
                    }
                    _ => (PixelPoint::new(0, 0),
                        u64::from(spec.bounds.width) * u64::from(spec.bounds.height), spec.bounds.height),
                };
                assert_eq!(result.point, point, "{fixture} / {}", control.label());
                assert_eq!(result.pixels_examined, pixels, "{fixture} / {} pixels", control.label());
                assert_eq!(result.rows_examined, rows, "{fixture} / {} rows", control.label());
            }
        }
    }


    /// Per-game Play bands accept their own layout without inheriting another game's offset.
    #[test]
    fn freecell_and_spider_play_bands_remain_independent() {
        let freecell = decode_png(include_bytes!("../tests/fixtures/freecell-FC12.png")).unwrap();
        let spider = decode_png(include_bytes!("../tests/fixtures/spider-SP18.png")).unwrap();
        assert_eq!(control_point(&freecell, FREECELL_CONTROLS.play).unwrap(), PixelPoint::new(680, 762));
        assert_eq!(control_point(&freecell, SPIDER_CONTROLS.play).unwrap(), PixelPoint::new(0, 0));
        assert_eq!(control_point(&spider, SPIDER_CONTROLS.play).unwrap(), PixelPoint::new(679, 862));
        assert_eq!(control_point(&spider, FREECELL_CONTROLS.play).unwrap(), PixelPoint::new(0, 0));
    }


    /// The O stripe excludes the retained pointer in Klondike's New Game K53.
    #[test]
    fn retained_cursor_does_not_authorise_ok_on_new_game_screen() {
        let frame = decode_png(include_bytes!("../tests/fixtures/klondike/K53.png")).unwrap();
        assert_eq!(
            find_black_run_bottom_up(frame.as_view().unwrap(), PixelRect::new(930, 700, 61, 151), 5).unwrap(),
            PixelPoint::new(960, 834),
        );
        assert_eq!(control_point(&frame, KLONDIKE_CONTROLS.ok).unwrap(), PixelPoint::new(0, 0));
        assert_eq!(control_point(&frame, KLONDIKE_CONTROLS.new_game).unwrap(), PixelPoint::new(862, 861));
    }
}
