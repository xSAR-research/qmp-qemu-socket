//! Positive normal Pyramid Congratulations/counting evidence, without input authority.
//!
//! Calibrated native title, skip instruction, panel, dim Pyramid legend and the
//! full third-board ordinal must agree. The ordinal alone is never completion:
//! callers additionally retain their actual final-completion capability. Dynamic
//! level, XP and ticket values are outside these masks. No Level Up/menu proof
//! or another game's generic reward panel is substituted for this scene.

use super::pyramid_terminal_patch;
use crate::{capture::CapturedFrame, geometry::PixelRect};


/// Measured title shape at(708,200), sample stride4.
const TITLE_MASK: [&str; 17] = [
    "........................................................................??..................?..................................",
    "....?#####..............................................................##.................###............................?##..",
    "...#######...............................................?#.............##............?#...##?............................?##..",
    "..###?...?..............................................?##.............##...........?##...................................##..",
    ".?##?........?###?...???.##?....?##????..??.?#..?###?..?###??.???..???..##...?###?..?###??.???...?###?...???.?##....?###...##..",
    ".###........#######..########..?#######..##?##.?#####?.######.##?..###..##..?#####?.######.###..#######..?#######..#####...##..",
    ".###.......?##...##?.###..###..###..###..###?..?...?##..?##...##?..###..##..?...?##..?##...###..##?..##?.?##?.?##..##......##..",
    ".###.......###...###.###..?##..##...?##..##?.....?####..?##...##?..###..##....?####..?##...###.###...###.?##...##..###?....##..",
    ".###.......##?...###.###...##.?##...?##..##?...####?##..?##...##?..###..##..####?##..?##...###.###...?##.?##...##..?####...##..",
    ".?###......###...##?.###...##..##...###..##?...##..?##..?##...##?..###..##..##...##..?##...###.?##...###.?##...##.....###......",
    "..########.?##?.?##..###...##..###.?###..##?...##..###..?##??.###.?###..##..##..###..?##??.###..##?.?##?.?##...##..?..###..##..",
    "...#######..?#####?..###...##..?####?##..##?...####?##...####.?#######..##..####?##...####.###..?#####?..?##...##..#####..?##..",
    ".....???......???................??.###.........??........??....??...........??........??.........???...............???....??..",
    "....................................##?........................................................................................",
    "...............................#######.........................................................................................",
    "...............................#####?..........................................................................................",
    "...............................................................................................................................",
];

/// Measured skip shape at(798,838), sample stride4.
const SKIP_MASK: [&str; 9] = [
    "...................................................................................",
    "..###.#.#.....#........................#..................?............#...#.......",
    ".#....#.......#........................#..................#............#...........",
    ".#....#.#.#...#.#...#.#.?#.#.#.#.#.#.?.#?.#.#.#.#?.#.#....#.#?.#...#...#??.#.#..#..",
    ".#....#.#.#...##....?##.?..#.#.#.#..##.#..#.###.#..###....#.#..#...##..##..#.#..#..",
    ".#....#.#.#...#??...#.#.?..#..??.??.##.#..#.#...#..#......#.#..#.....#.##..#.#..#..",
    "..###.#.#..##.#.#...?#?.?..#..#...#.#..#..#.?##.#...##....##.##....##..#.#.#.#?#...",
    "..............................#..............................................#.....",
    ".............................?...............................................?.....",
];

/// Measured normal_header_three shape at(1658,46), sample stride2.
const NORMAL_HEADER_THREE_MASK: [&str; 16] = [
    "................................................................................",
    "................................................................................",
    "................................................................................",
    "................................................................................",
    "................................??...............................?#.............",
    "...?####........................??.............####.............##.....?###.....",
    "...?#..##.......................??................#.............#?........?#....",
    "...?#..#?...###...?##...#.#..?#???..###.?#........#.......###..####.......??....",
    "...?#?#?...#..?#.....#..##..##..#?.#?....?......?#.......#?.?#..#?......?#?.....",
    "...?#.?#?.#?...#....?#..#...#...#?.##............?#.....?#...#?.#?.......?#?....",
    "...?#...#.#?...#.?#..#..#...#...??..?##...........#?....?#...#?.#?.........#....",
    "...?#..?#.?#...#.#...#..#...#?..#?....#...........#?.....#...#..#?........?#....",
    "...?####...####..?#??#..#...?###??.#?##.?#.....####......?###?..#?.....####.....",
    "................................................................................",
    "................................................................................",
    "................................................................................",
];


/// Observe only the retained normal reward layout, never infer a missing control.
///
/// Invalid storage returns an error; other sizes and unsupported pixels refuse.
pub(in crate::worker) fn observe(frame: &CapturedFrame) -> Result<bool, String> {
    if !frame.is_layout_valid() {
        return Err("Normal Pyramid reward frame has invalid pixel storage".to_owned());
    }
    if (frame.width, frame.height) != (1_920, 1_080) {
        return Ok(false);
    }
    if small_counting(frame)? {
        return Ok(true);
    }
    if !text(
        frame,
        (708, 200),
        4,
        &TITLE_MASK,
        |rgb| rgb.into_iter().all(|c| c >= 245),
        |[r, g, b]| r >= 200 && (110..=175).contains(&g) && b <= 35,
    ) || !text(
        frame,
        (798, 838),
        4,
        &SKIP_MASK,
        |[r, g, b]| r >= 110 && g >= 140 && b >= 180,
        |[r, g, b]| r < 90 && g < 130 && b < 190,
    ) || !text(
        frame,
        (1_658, 46),
        2,
        &NORMAL_HEADER_THREE_MASK,
        |[r, g, b]| r >= 40 && g >= 45 && b >= 65,
        |[r, g, b]| r <= 27 && g <= 41 && b <= 70,
    ) {
        return Ok(false);
    }
    for bounds in [
        PixelRect::new(507, 400, 3, 20),
        PixelRect::new(1_410, 400, 3, 20),
        PixelRect::new(550, 922, 50, 2),
        PixelRect::new(1_320, 139, 50, 2),
    ] {
        if !pyramid_terminal_patch(frame, bounds, |[r, g, b]| {
            r >= 40
                && g >= 95
                && b >= 150
                && i16::from(g) >= i16::from(r) + 30
                && i16::from(b) >= i16::from(g) + 50
        }) {
            return Ok(false);
        }
    }
    for x in [630, 1_275] {
        if !pyramid_terminal_patch(frame, PixelRect::new(x, 330, 16, 20), |[r, g, b]| {
            r <= 10 && (20..=55).contains(&g) && (45..=90).contains(&b)
        }) {
            return Ok(false);
        }
    }
    for y in [169, 207, 245, 283, 321, 359] {
        if !pyramid_terminal_patch(frame, PixelRect::new(1_816, y, 68, 2), |[r, g, b]| {
            (39..=52).contains(&r) && (46..=58).contains(&g) && (74..=93).contains(&b)
        }) {
            return Ok(false);
        }
    }
    Ok(true)
}


/// Require independent 98% glyph and backdrop populations at measured native spacing.
fn text(
    frame: &CapturedFrame,
    origin: (u32, u32),
    spacing: u32,
    mask: &[&str],
    glyph: fn([u8; 3]) -> bool,
    backdrop: fn([u8; 3]) -> bool,
) -> bool {
    let mut total = [0u32; 2];
    let mut matched = [0u32; 2];
    for (y, line) in mask.iter().enumerate() {
        for (x, byte) in line.bytes().enumerate() {
            let (i, predicate) = match byte {
                b'#' => (0, glyph),
                b'.' => (1, backdrop),
                _ => continue,
            };
            total[i] += 1;
            matched[i] += u32::from(
                crate::detector::pixel_rgb(
                    frame,
                    origin.0 + x as u32 * spacing,
                    origin.1 + y as u32 * spacing,
                )
                .is_some_and(predicate),
            );
        }
    }
    (0..2).all(|i| total[i] > 0 && matched[i] * 100 >= total[i] * 98)
}


/// Recognise the complete small normal reward layout independently of challenge parsing.
///
/// The whole Congratulations title, skip instruction, third-board ordinal, complete
/// Pyramid legend and separated panel edges match one native sample byte-exactly.
/// Dynamic XP/level values are excluded. Callers still require their own verified
/// normal completion before this observation can participate in guarded restart.
/// Returns an error for malformed storage, bounds or bundled reference metadata.
fn small_counting(frame: &CapturedFrame) -> Result<bool, String> {
    let view = frame.as_view().map_err(|error| error.to_string())?;
    for (bounds, rgb) in [
        (
            PixelRect::new(782, 306, 360, 50),
            include_bytes!("../../../tests/fixtures/pyramid-testing-156/normal-small-title.rgb")
                .as_slice(),
        ),
        (
            PixelRect::new(845, 746, 230, 30),
            include_bytes!("../../../tests/fixtures/pyramid-testing-156/normal-small-skip.rgb")
                .as_slice(),
        ),
        (
            PixelRect::new(1_658, 46, 160, 32),
            include_bytes!("../../../tests/fixtures/pyramid-testing-156/normal-small-header.rgb")
                .as_slice(),
        ),
        (
            PixelRect::new(1_796, 116, 109, 290),
            include_bytes!("../../../tests/fixtures/pyramid-testing-156/normal-small-legend.rgb")
                .as_slice(),
        ),
        (
            PixelRect::new(648, 390, 4, 300),
            include_bytes!(
                "../../../tests/fixtures/pyramid-testing-156/normal-small-left-edge.rgb"
            )
            .as_slice(),
        ),
        (
            PixelRect::new(1_268, 390, 4, 300),
            include_bytes!(
                "../../../tests/fixtures/pyramid-testing-156/normal-small-right-edge.rgb"
            )
            .as_slice(),
        ),
        (
            PixelRect::new(690, 806, 500, 4),
            include_bytes!(
                "../../../tests/fixtures/pyramid-testing-156/normal-small-bottom-edge.rgb"
            )
            .as_slice(),
        ),
    ] {
        if !crate::pyramid_progress::matches_native_rgb_patch(view, bounds, rgb, 0)? {
            return Ok(false);
        }
    }
    Ok(true)
}
