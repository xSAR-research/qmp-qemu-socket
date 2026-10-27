//! Regressions for independent retained washout frames and rejected incomplete captions.

use crate::{
    capture::{CapturedFrame, decode_png},
    game::GameMode,
    geometry::PixelPoint,
    terminal_controls::{
        Control, control_point,
        ok_readiness::{GlyphEvidence, MAX_READINESS_PIXEL_COMPARISONS, OkReadiness},
        profiles::{OkGlyphPolicy, for_game},
    },
};


/// Load a byte-original frame; 17 alone supplied calibration, while 18 and 19 are held out.
fn retained(number: u8) -> CapturedFrame {
    let name = match number {
        17 => "010728.791_000017.png",
        18 => "010729.141_000018.png",
        19 => "010729.531_000019.png",
        _ => panic!("only the three documented native observations are fixtures"),
    };
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/klondike-level841-20261026")
        .join(name);
    decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Supply the unchanged bounded exact-black scan result to the selected profile.
fn inspect(
    readiness: &mut OkReadiness,
    frame: &CapturedFrame,
    game: GameMode,
) -> crate::terminal_controls::ok_readiness::OkObservation {
    let profile = for_game(game);
    let raw = control_point(frame, profile.spec(Control::Ok)).unwrap();
    readiness
        .observe_with_policy(frame, raw, profile.ok_glyph_policy)
        .unwrap()
}


/// Held-out current captions can qualify only after another fresh settled body.
#[test]
fn native_154_level841_washout_matches_complete_current_caption_after_geometry() {
    for number in [18, 19] {
        let clean = retained(17);
        let washed = retained(number);
        assert_eq!(
            control_point(&washed, for_game(GameMode::Klondike).ok).unwrap(),
            PixelPoint::new(0, 0)
        );
        let mut readiness = OkReadiness::new();
        let first = inspect(&mut readiness, &clean, GameMode::Klondike);
        assert!(!first.ready && first.confirmation_pending);
        assert_eq!(first.glyph_evidence, GlyphEvidence::ExactBlack);
        let current = inspect(&mut readiness, &washed, GameMode::Klondike);
        assert!(current.ready && current.geometry_stable);
        assert_eq!(current.glyph_evidence, GlyphEvidence::KlondikeCaption);
        assert_eq!(current.point, PixelPoint::new(960, 758));
        assert_eq!(current.reason, "settled-button-and-caption-appearance");
        assert!(current.pixels_examined <= MAX_READINESS_PIXEL_COMPARISONS);
        let mut independent = OkReadiness::new();
        assert!(!inspect(&mut independent, &washed, GameMode::Klondike).ready);
        assert!(inspect(&mut independent, &washed, GameMode::Klondike).ready);
    }
}


/// This game's bounded correction grants no new caption authority to another profile.
#[test]
fn native_154_other_game_profiles_retain_exact_black_washout_refusal() {
    for game in [
        GameMode::FreeCell,
        GameMode::Spider,
        GameMode::TriPeaks,
        GameMode::Pyramid,
    ] {
        assert_eq!(for_game(game).ok_glyph_policy, OkGlyphPolicy::ExactBlack);
        let mut readiness = OkReadiness::new();
        inspect(&mut readiness, &retained(17), game);
        for number in [18, 19] {
            let observation = inspect(&mut readiness, &retained(number), game);
            assert!(!observation.ready);
            assert_eq!(observation.point, PixelPoint::new(0, 0));
            assert_eq!(observation.glyph_evidence, GlyphEvidence::Missing);
        }
    }
}


/// A body, either isolated letter or partial current patch cannot replace the complete caption.
#[test]
fn native_154_incomplete_or_erased_caption_still_withholds_input() {
    for (left, right) in [(930, 960), (960, 990), (930, 990)] {
        let mut altered = retained(18);
        for y in 741usize..776 {
            let background = altered.pixels[y * altered.stride + 1000 * 4..][..3].to_vec();
            for x in left..right {
                let offset = y * altered.stride + x * 4;
                altered.pixels[offset..offset + 3].copy_from_slice(&background);
            }
        }
        let mut readiness = OkReadiness::new();
        inspect(&mut readiness, &retained(17), GameMode::Klondike);
        let current = inspect(&mut readiness, &altered, GameMode::Klondike);
        assert!(current.button.is_some() && current.geometry_stable);
        assert!(!current.ready);
        assert_eq!(current.glyph_evidence, GlyphEvidence::Missing);
        assert_eq!(current.point, PixelPoint::new(0, 0));
    }
}


/// Malformed frame errors discard the prior body and cannot revive earlier coordinates.
#[test]
fn native_154_bad_frame_clears_caption_predecessor_before_next_observation() {
    let washed = retained(18);
    let mut readiness = OkReadiness::new();
    inspect(&mut readiness, &washed, GameMode::Klondike);
    let mut malformed = retained(19);
    malformed.width -= 1;
    assert!(
        readiness
            .observe_with_policy(
                &malformed,
                PixelPoint::new(0, 0),
                OkGlyphPolicy::KlondikeCaption
            )
            .is_err()
    );
    assert!(!inspect(&mut readiness, &washed, GameMode::Klondike).ready);
    assert!(inspect(&mut readiness, &washed, GameMode::Klondike).ready);
}
