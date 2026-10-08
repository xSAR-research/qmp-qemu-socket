//! Free Cell's one-board win and deterministic restart controls.
//!
//! FC09-FC12 measure the score-skip, OK, New Game and Play regions; FC22/FC23
//! measure Free Cell's higher and lower local OK layouts. Independent
//! entry recognises Congratulations plus its skip caption or New Game button.
//! An isolated OK or Play button never establishes a win. Once a win is known,
//! readiness reads only the expected local control, not level, rank, medal,
//! fireworks, panel artwork, foundations or card pixels. The worker owns ordered
//! one-shot input, editable delays, fresh observations and cancellation.

use std::fmt;

use crate::{
    capture::CapturedFrame,
    detector::{HaloDetectionError, pixel_rgb},
    geometry::{PixelPoint, PixelRect},
};


/// Expected control in the evidenced single-board Free Cell restart sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalStage {
    /// Congratulations reward counting, with a click-anywhere skip caption.
    Score,
    /// The expected Level Up OK button after a confirmed game win.
    LevelUp,
    /// Congratulations New Game button, retaining the current game type.
    NewGame,
    /// Play button, retaining the guest's selected Free Cell difficulty.
    Play,
}


impl fmt::Display for TerminalStage {


    /// Name the expected control without claiming the preceding input had effect.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Score => "Free Cell score counting / click to skip",
            Self::LevelUp => "Free Cell Level Up OK",
            Self::NewGame => "Free Cell Congratulations New Game",
            Self::Play => "Free Cell Play",
        })
    }
}


/// Caption foreground, classified by contrast rather than recorded RGB triples.
#[derive(Clone, Copy)]
enum CaptionInk {
    /// White Congratulations lettering on its coloured banner.
    Bright,
    /// Pale skip lettering on the darker score panel.
    Muted,
    /// Near-black printed lettering inside a warm gold button.
    Dark,
}


impl CaptionInk {


    /// Broad foreground classes preserve antialiasing without matching artwork.
    fn contains(self, rgb: [u8; 3]) -> bool {


        match self {
            Self::Bright => rgb.into_iter().all(|channel| channel >= 180),
            Self::Muted => rgb.into_iter().all(|channel| channel >= 125),
            Self::Dark => rgb.into_iter().all(|channel| channel <= 90),
        }
    }
}


/// Four-pixel caption cells retain word shape without exact glyph pixel matching.
struct Caption {
    /// Measured native caption area, with a small surrounding margin.
    bounds: PixelRect,
    /// Foreground contrast class belonging to the printed label.
    ink: CaptionInk,
    /// Occupied four-by-four cells, one bit per column and one mask per row.
    rows: &'static [u128],
}


/// Congratulations letters only; level, reward and medal artwork are excluded.
const CONGRATULATIONS: Caption = Caption {
    bounds: PixelRect::new(708, 198, 504, 68),
    ink: CaptionInk::Bright,
    rows: &[
        0x3000000000000000000,
        0x1c0000003800038000000000000003f0,
        0x1c00000038e0038007000000000003fc,
        0x1c00000000e0038007000000000003fc,
        0x1cf0ee3e1bf1e398cf8f37770fe1e00e,
        0x1cfdfe7f3ff3f3b8ffdfbf7f9fe7f00e,
        0x1c9dfe7fbdf713b8efb8bf7fdfe7f80f,
        0x1c3dcee3bce7e3b8e73f0f71dcef380f,
        0x1cf9cee3bce7f3b8e73f8771dcef3c0e,
        0x1e1cee3bce73bb8e739c771dcef381e,
        0x1dc5ce7fbde7bbbfef3dc77fdce7fbfe,
        0x1cfdce7f3fe7fbbfdf3fc77f9ce3f3fc,
        0xc7dce1e19c7739b8e3b87771ce1e1f0,
        0x3800000000,
        0x3f80000000,
        0x1f80000000,
        0x200000000,
    ],
};


/// FC09's Click anywhere to skip caption; no score-count pixels are sampled.
const SCORE_SKIP: Caption = Caption {
    bounds: PixelRect::new(796, 836, 328, 36),
    ink: CaptionInk::Muted,
    rows: &[
        0x0,
        0x880000001800000415c,
        0x80804400238000004046,
        0x1eab8fe3f77fbaf715d42,
        0x129999479f5baabe0c542,
        0x12ba0b40915b6cbd1c546,
        0xeab8ec3974b44af1595c,
        0x20000000000040000000,
        0x20000000000020000000,
    ],
};


/// FC10's OK caption, independent of the current level and medal.
const LEVEL_UP_OK: Caption = Caption {
    bounds: PixelRect::new(930, 794, 56, 36),
    ink: CaptionInk::Dark,
    rows: &[0x0, 0x1b3c, 0xb66, 0xfc6, 0x7c6, 0xf46, 0x1b7e, 0x133c, 0x0],
};


/// FC11's New Game caption, excluding the neighbouring Home control.
const NEW_GAME: Caption = Caption {
    bounds: PixelRect::new(695, 834, 184, 36),
    ink: CaptionInk::Dark,
    rows: &[
        0x0, 0x7800066, 0x44820c0926e, 0x1efef266bf6a, 0x1fdae763fffa,
        0xfdaf44367f2, 0xedaf7c36772, 0xc0023000602, 0x0,
    ],
};


/// FC12's Play caption, independent of the selected difficulty and XP reward.
const PLAY: Caption = Caption {
    bounds: PixelRect::new(675, 744, 68, 40),
    ink: CaptionInk::Dark,
    rows: &[0x0, 0x6e, 0x7a, 0x977a, 0xfe6e, 0x6f62, 0x6f62, 0x6000, 0x3000, 0x1000],
};


/// Reject malformed storage and unsupported native geometry before probing.
fn validate_frame(frame: &CapturedFrame) -> Result<(), HaloDetectionError> {


    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }


    if frame.width != 1_920 || frame.height != 1_080 {
        return Err(HaloDetectionError::BoundsOutsideFrame);
    }
    Ok(())
}


/// Match coarse foreground word shape at one location, allowing antialiasing.
/// Extra ink is penalised as well as missing ink; flat black, white and gold
/// rectangles therefore cannot stand in for a printed control caption.
fn caption_at(
    frame: &CapturedFrame,
    caption: &Caption,
    dx: i32,
    dy: i32,
) -> Result<bool, HaloDetectionError> {
    let mut intersection = 0_u32;
    let mut union = 0_u32;


    for (row, expected) in caption.rows.iter().enumerate() {
        let mut observed = 0_u128;


        for column in 0..caption.bounds.width / 4 {
            let mut ink_pixels = 0_u32;


            for cell_y in 0..4 {


                for cell_x in 0..4 {
                    let x = caption.bounds.x as i32 + dx + (column * 4 + cell_x) as i32;
                    let y = caption.bounds.y as i32 + dy + (row * 4 + cell_y) as i32;
                    let rgb = pixel_rgb(frame, x as u32, y as u32)
                        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
                    ink_pixels += u32::from(caption.ink.contains(rgb));
                }
            }


            if ink_pixels >= 4 {
                observed |= 1_u128 << column;
            }
        }
        intersection += (observed & expected).count_ones();
        union += (observed | expected).count_ones();
    }
    Ok(union != 0 && intersection * 100 >= union * 75)
}


/// Caption alignment remains inside the expected button's small local window.
/// The whole word shifts together; no artwork samples or individual glyph points
/// are aligned independently, and the eventual click coordinate remains fixed.
fn caption_ready(frame: &CapturedFrame, caption: &Caption) -> Result<bool, HaloDetectionError> {


    for dy in (-8..=8).step_by(2) {


        for dx in (-4..=4).step_by(2) {


            if caption_at(frame, caption, dx, dy)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}


/// Recognise a warm button body without an exact colour or decorative signature.
fn is_button_gold([red, green, blue]: [u8; 3]) -> bool {
    red >= 140 && green >= 95
        && u16::from(red) >= u16::from(blue) + 20
        && u16::from(green) >= u16::from(blue) + 12
}


/// Positive local body coverage excludes dark lettering and the button border.
fn button_body_ready(frame: &CapturedFrame, bounds: PixelRect) -> Result<bool, HaloDetectionError> {
    let mut gold = 0_u32;


    for y in bounds.y..bounds.bottom() {


        for x in bounds.x..bounds.right() {
            let rgb = pixel_rgb(frame, x, y).ok_or(HaloDetectionError::BoundsOutsideFrame)?;
            gold += u32::from(is_button_gold(rgb));
        }
    }
    Ok(gold * 100 >= bounds.width * bounds.height * 65)
}


/// Inspect only the expected control after the worker has independently confirmed
/// a win. Readiness alone does not establish completion or authorise a click.
pub fn expected_control_ready(
    frame: &CapturedFrame,
    stage: TerminalStage,
) -> Result<bool, HaloDetectionError> {
    validate_frame(frame)?;
    let (caption, body) = match stage {
        TerminalStage::Score => return caption_ready(frame, &SCORE_SKIP),
        TerminalStage::LevelUp => return level_up_control_ready(frame),
        TerminalStage::NewGame => (&NEW_GAME, PixelRect::new(640, 824, 291, 56)),
        TerminalStage::Play => return play_control_ready_at(frame, 0),
    };
    Ok(button_body_ready(frame, body)? && caption_ready(frame, caption)?)
}


/// Free Cell's expected OK control at the three supplied vertical layouts.
/// FC10 is the original location, FC22 is 19 pixels higher and FC23 is 12
/// pixels lower. Align the whole local control; no level or artwork is sampled.
fn level_up_control_ready(frame: &CapturedFrame) -> Result<bool, HaloDetectionError> {


    for y_offset in [0, -19, 12] {


        if level_up_control_ready_at(frame, y_offset)? {
            return Ok(true);
        }
    }
    Ok(false)
}


/// Reuse the local Play caption at an evidenced vertical layout offset.
/// Free Cell uses zero; Spider's native SP18 places the same control 100 pixels
/// lower. The offset changes only this expected control, never win authority.
pub(crate) fn play_control_ready_at(
    frame: &CapturedFrame,
    y_offset: u32,
) -> Result<bool, HaloDetectionError> {
    validate_frame(frame)?;
    let caption_bottom = PLAY.bounds.bottom().checked_add(y_offset)
        .and_then(|bottom| bottom.checked_add(8))
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;


    if caption_bottom > frame.height {
        return Err(HaloDetectionError::BoundsOutsideFrame);
    }
    let caption = Caption {
        bounds: PixelRect::new(PLAY.bounds.x, PLAY.bounds.y + y_offset,
            PLAY.bounds.width, PLAY.bounds.height),
        ink: PLAY.ink,
        rows: PLAY.rows,
    };
    let body = PixelRect::new(610, 735 + y_offset, 198, 52);
    Ok(button_body_ready(frame, body)? && caption_ready(frame, &caption)?)
}


/// Reuse only the local OK control at an evidenced signed vertical offset.
/// Spider SP20 and Free Cell FC22 place the caption 19 pixels higher than FC10;
/// Free Cell FC23 places it 12 pixels lower. Each caller chooses its own layouts.
pub(crate) fn level_up_control_ready_at(
    frame: &CapturedFrame,
    y_offset: i32,
) -> Result<bool, HaloDetectionError> {
    validate_frame(frame)?;
    let caption_y = LEVEL_UP_OK.bounds.y.checked_add_signed(y_offset)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
    let body_y = 788_u32.checked_add_signed(y_offset)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
    let caption_bottom = caption_y.checked_add(LEVEL_UP_OK.bounds.height)
        .and_then(|bottom| bottom.checked_add(8))
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
    let body_bottom = body_y.checked_add(50)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;


    if caption_y < 8 || caption_bottom > frame.height || body_bottom > frame.height {
        return Err(HaloDetectionError::BoundsOutsideFrame);
    }
    let caption = Caption {
        bounds: PixelRect::new(LEVEL_UP_OK.bounds.x, caption_y,
            LEVEL_UP_OK.bounds.width, LEVEL_UP_OK.bounds.height),
        ink: LEVEL_UP_OK.ink,
        rows: LEVEL_UP_OK.rows,
    };
    let body = PixelRect::new(848, body_y, 223, 50);
    Ok(button_body_ready(frame, body)? && caption_ready(frame, &caption)?)
}


/// Establish a win from the Congratulations panel's own stable local captions.
/// Generic gold, isolated OK/Play controls and missing HALOs are not win proof.
/// Level Up can be handled only after the worker has already established the win.
pub fn classify_win_entry(
    frame: &CapturedFrame,
) -> Result<Option<TerminalStage>, HaloDetectionError> {
    validate_frame(frame)?;


    if !caption_ready(frame, &CONGRATULATIONS)? {
        return Ok(None);
    }


    if expected_control_ready(frame, TerminalStage::NewGame)? {
        return Ok(Some(TerminalStage::NewGame));
    }


    if expected_control_ready(frame, TerminalStage::Score)? {
        return Ok(Some(TerminalStage::Score));
    }
    Ok(None)
}


/// One fixed native point inside the observed control; no input timing is inferred.
pub const fn click_point(stage: TerminalStage) -> PixelPoint {


    match stage {
        TerminalStage::Score => PixelPoint::new(960, 550),
        TerminalStage::LevelUp => PixelPoint::new(959, 812),
        TerminalStage::NewGame => PixelPoint::new(786, 852),
        TerminalStage::Play => PixelPoint::new(709, 761),
    }
}


#[cfg(test)]
mod tests {
    //! Native terminal fixtures and isolation from decorative artwork.

    use super::*;
    use crate::capture::{PixelFormat, decode_png};


    /// Decode one of the byte-identical native terminal/deal captures.
    fn fixture(number: u8) -> CapturedFrame {
        let bytes: &[u8] = match number {
            9 => include_bytes!("../tests/fixtures/freecell-FC09.png"),
            10 => include_bytes!("../tests/fixtures/freecell-FC10.png"),
            11 => include_bytes!("../tests/fixtures/freecell-FC11.png"),
            12 => include_bytes!("../tests/fixtures/freecell-FC12.png"),
            13 => include_bytes!("../tests/fixtures/freecell-FC13.png"),
            15 => include_bytes!("../tests/fixtures/freecell-FC15.png"),
            22 => include_bytes!("../tests/fixtures/freecell-FC22.png"),
            23 => include_bytes!("../tests/fixtures/freecell-FC23.png"),
            _ => panic!("unsupported Free Cell terminal fixture {number}"),
        };
        decode_png(bytes).expect("decode native Free Cell terminal fixture")
    }


    /// The native sequence has one specific local expected control per frame.
    #[test]
    fn native_terminal_controls_follow_the_evidenced_sequence() {
        let stages = [TerminalStage::Score, TerminalStage::LevelUp,
            TerminalStage::NewGame, TerminalStage::Play];


        for (number, expected) in [(9, TerminalStage::Score), (10, TerminalStage::LevelUp),
            (11, TerminalStage::NewGame), (12, TerminalStage::Play)] {
            let frame = fixture(number);


            for stage in stages {
                assert_eq!(expected_control_ready(&frame, stage).unwrap(), stage == expected,
                    "FC{number:02} / {stage}");
            }
        }
    }


    /// The supplied higher/lower OK layouts preserve the original input point.
    #[test]
    fn shifted_freecell_level_up_controls_keep_local_readiness_and_click() {


        for (number, y_offset) in [(22, -19), (23, 12)] {
            let frame = fixture(number);
            assert!(!level_up_control_ready_at(&frame, 0).unwrap());
            assert!(level_up_control_ready_at(&frame, y_offset).unwrap());
            assert!(expected_control_ready(&frame, TerminalStage::LevelUp).unwrap());
            let point = click_point(TerminalStage::LevelUp);
            assert!(is_button_gold(pixel_rgb(&frame, point.x as u32, point.y as u32).unwrap()));


            for stage in [TerminalStage::Score, TerminalStage::NewGame, TerminalStage::Play] {
                assert!(!expected_control_ready(&frame, stage).unwrap());
            }
            assert_eq!(classify_win_entry(&frame).unwrap(), None);
        }
    }


    /// OK and Play are never promoted to independent completion evidence.
    #[test]
    fn win_entry_requires_congratulations_and_its_own_local_control() {
        assert_eq!(classify_win_entry(&fixture(9)).unwrap(), Some(TerminalStage::Score));
        assert_eq!(classify_win_entry(&fixture(11)).unwrap(), Some(TerminalStage::NewGame));


        for number in [10, 12, 13, 22, 23] {
            assert_eq!(classify_win_entry(&fixture(number)).unwrap(), None);
        }
    }


    /// FC15's changed rank does not hide New Game or turn its panel into Level Up.
    #[test]
    fn stopped_new_game_panel_recognises_its_local_control_without_level_up() {
        let frame = fixture(15);
        assert_eq!(classify_win_entry(&frame).unwrap(), Some(TerminalStage::NewGame));
        assert!(expected_control_ready(&frame, TerminalStage::NewGame).unwrap());
        assert!(!expected_control_ready(&frame, TerminalStage::LevelUp).unwrap());
        assert!(!expected_control_ready(&frame, TerminalStage::Score).unwrap());
        assert!(!expected_control_ready(&frame, TerminalStage::Play).unwrap());
    }


    /// Remove everything outside the expected control and its small local window.
    /// Level, rank, medal, fireworks, modal frame and completed cards cannot gate it.
    #[test]
    fn decorative_artwork_is_not_expected_control_authority() {


        for (number, stage, bounds) in [
            (9, TerminalStage::Score, PixelRect::new(790, 826, 340, 56)),
            (10, TerminalStage::LevelUp, PixelRect::new(838, 778, 245, 70)),
            (22, TerminalStage::LevelUp, PixelRect::new(838, 759, 245, 70)),
            (23, TerminalStage::LevelUp, PixelRect::new(838, 790, 245, 70)),
            (11, TerminalStage::NewGame, PixelRect::new(630, 814, 311, 76)),
            (12, TerminalStage::Play, PixelRect::new(600, 725, 218, 72)),
        ] {
            let mut frame = fixture(number);


            for y in 0..frame.height {


                for x in 0..frame.width {


                    if !bounds.contains(PixelPoint::new(x as i32, y as i32)) {
                        let index = y as usize * frame.stride + x as usize * 4;
                        frame.pixels[index..index + 4].copy_from_slice(&[250, 0, 250, 255]);
                    }
                }
            }
            assert!(expected_control_ready(&frame, stage).unwrap());
            assert_eq!(classify_win_entry(&frame).unwrap(), None);
        }
    }


    /// Blank, flat white, generic gold and the fresh inactive-Solver deal all stop.
    #[test]
    fn unsupported_flat_frames_and_fresh_deal_never_have_terminal_authority() {
        let mut frame = fixture(13);


        for stage in [TerminalStage::Score, TerminalStage::LevelUp,
            TerminalStage::NewGame, TerminalStage::Play] {
            assert!(!expected_control_ready(&frame, stage).unwrap());
        }


        for rgb in [[0, 0, 0], [255, 255, 255], [210, 180, 60]] {


            for pixel in frame.pixels.as_chunks_mut::<4>().0 {
                pixel.copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }


            for stage in [TerminalStage::Score, TerminalStage::LevelUp,
                TerminalStage::NewGame, TerminalStage::Play] {
                assert!(!expected_control_ready(&frame, stage).unwrap());
            }
            assert_eq!(classify_win_entry(&frame).unwrap(), None);
        }
    }


    /// Button colour gradients may change while the same local black label remains.
    #[test]
    fn warm_button_palette_changes_preserve_expected_caption_readiness() {


        for (number, stage, bounds) in [
            (10, TerminalStage::LevelUp, PixelRect::new(838, 778, 245, 70)),
            (22, TerminalStage::LevelUp, PixelRect::new(838, 759, 245, 70)),
            (23, TerminalStage::LevelUp, PixelRect::new(838, 790, 245, 70)),
            (11, TerminalStage::NewGame, PixelRect::new(630, 814, 311, 76)),
            (12, TerminalStage::Play, PixelRect::new(600, 725, 218, 72)),
        ] {
            let mut frame = fixture(number);


            for y in bounds.y..bounds.bottom() {


                for x in bounds.x..bounds.right() {
                    let index = y as usize * frame.stride + x as usize * 4;
                    let rgb = pixel_rgb(&frame, x, y).unwrap();


                    if is_button_gold(rgb) {
                        frame.pixels[index..index + 4].copy_from_slice(&[210, 180, 60, 255]);
                    }
                }
            }
            assert!(expected_control_ready(&frame, stage).unwrap());
        }
    }


    /// The frame contract is checked before any fixed caption or button access.
    #[test]
    fn malformed_or_non_native_frames_are_rejected() {
        let mut frame = CapturedFrame {
            width: 1_920, height: 1_080, stride: 1_920 * 4,
            format: PixelFormat::Rgba8, pixels: vec![0; 4],
        };
        assert_eq!(classify_win_entry(&frame), Err(HaloDetectionError::InvalidFrameLayout));
        assert_eq!(expected_control_ready(&frame, TerminalStage::Play),
            Err(HaloDetectionError::InvalidFrameLayout));
        frame = fixture(13);
        frame.width = 1_919;
        assert_eq!(classify_win_entry(&frame), Err(HaloDetectionError::BoundsOutsideFrame));
    }
}
