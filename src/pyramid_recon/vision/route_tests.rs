//! Partial-route recognition regressions over the unchanged native v7 original.
//!
//! In-memory derivatives exercise positive absence and refusal. They do not
//! establish guest effects, a completed live route or a newly acquired deal.

use super::*;


/// Resolve one retained native profile or original without changing fixture bytes.
fn fixture_path(generation: &str, name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recon")
        .join(generation)
        .join(name)
}


/// Decode the unchanged native v7 PNG through the production decoder.
fn original() -> CapturedFrame {
    crate::capture::decode_png(
        &std::fs::read(fixture_path("native-v7", "baseline.png"))
            .expect("retained original v7 PNG"),
    )
    .expect("native original layout")
}


/// Load the actual v7 calibration through its bounded production file reader.
fn vision() -> ReconVision {
    ReconVision::load(&fixture_path("native-v7", "profile.json"))
        .expect("unchanged native v7 profile")
}


/// Read original identities independently from the retained accepted deal document.
fn original_tableau() -> [Option<Card>; 28] {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../xsar/tests/fixtures/pyramid-planning/pyramid-recon-20261019-v7-01.json");
    let bytes = std::fs::read(path).expect("retained complete accepted deal");
    let deal =
        xsar::pyramid_json::read_from(bytes.as_slice()).expect("strict complete accepted deal");
    assert_eq!(
        deal.stock().first(),
        Some(&Card::new(Rank::Four, Suit::Spades))
    );
    deal.tableau().map(Some)
}


/// Return the independently labelled original LEFT card.
fn original_left() -> Option<Card> {
    Some(Card::new(Rank::Four, Suit::Spades))
}


/// Paint a bounded RGB derivative while preserving alpha and the native layout.
fn fill_region(frame: &mut CapturedFrame, bounds: PixelRect, colour: [u8; 3]) {
    assert!(frame.as_view().is_ok());
    assert!(bounds.right() <= frame.width && bounds.bottom() <= frame.height);
    for row in bounds.y..bounds.bottom() {
        for column in bounds.x..bounds.right() {
            let offset = row as usize * frame.stride + column as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&colour);
        }
    }
}


/// Sample existing native felt, preserving the calibrated scene elsewhere.
fn felt_colour(frame: &CapturedFrame) -> [u8; 3] {
    let offset = 600 * frame.stride + 160 * 4;
    let colour: [u8; 3] = frame.pixels[offset..offset + 3].try_into().expect("RGB");
    let [red, green, blue] = colour;
    assert!(green >= 50 && red <= 100 && blue <= 140);
    assert!(i16::from(green) - i16::from(red) >= 20);
    assert!(i16::from(green) - i16::from(blue) >= 15);
    colour
}


/// Derive the fixed positive-absence probe from the original logical slot origin.
fn lower_probe(index: usize) -> PixelRect {
    let (x, y) = TABLEAU_ORIGINS[index];
    PixelRect::new(x + 40, y + 178, 60, 4)
}


/// Native v7 starts only when every original identity, pile and scene agrees.
#[test]
fn native_v7_original_matches_exact_complete_route_origin() {
    let original = original();
    let pixels = original.pixels.clone();
    let tableau = original_tableau();
    vision()
        .verify_position(&original, &tableau, original_left(), None)
        .expect("native complete origin matches");
    assert_eq!(original.pixels, pixels, "observation is read only");
}


/// The superseded v6 glyph bank remains unable to authorise the v7 position.
#[test]
fn native_v6_unknown_glyphs_refuse_route_origin_without_profile_changes() {
    let path = fixture_path("native-v6", "profile.json");
    let before = std::fs::read(&path).expect("retained v6 profile");
    let old_vision = ReconVision::load(&path).expect("unchanged valid v6 calibration");
    let error = old_vision
        .verify_position(&original(), &original_tableau(), original_left(), None)
        .expect_err("v6 cannot authorise v7 unknown glyphs");
    assert!(error.contains("identity mismatch"), "{error}");
    assert_eq!(std::fs::read(path).expect("retained v6 profile"), before);
}


/// Exact rank and suit are both required while a slot is expected to remain occupied.
#[test]
fn known_but_wrong_remaining_identity_refuses_route_position() {
    let mut tableau = original_tableau();
    assert_eq!(tableau[26], Some(Card::new(Rank::Ace, Suit::Spades)));
    tableau[26] = Some(Card::new(Rank::Queen, Suit::Clubs));
    let error = vision()
        .verify_position(&original(), &tableau, original_left(), None)
        .expect_err("another known identity is not the expected card");
    assert!(error.contains("identity mismatch at slot 26"), "{error}");
}


/// Unknown glyphs and unknown lower pixels cannot stand in for a removed card.
#[test]
fn unknown_removed_slot_is_never_accepted_as_positive_absence() {
    let mut changed = original();
    let (x, y) = TABLEAU_ORIGINS[26];
    fill_region(&mut changed, PixelRect::new(x, y, 40, 52), [11, 13, 17]);
    fill_region(&mut changed, lower_probe(26), [11, 13, 17]);
    let reader = vision();
    assert_eq!(
        reader
            .tableau_reader
            .recognise(changed.as_view().expect("derivative layout"), (x, y))
            .expect("bounded unknown glyph recognition"),
        None,
    );
    let mut tableau = original_tableau();
    tableau[26] = None;
    let error = reader
        .verify_position(&changed, &tableau, original_left(), None)
        .expect_err("unknown pixels are not empty");
    assert!(error.contains("slot 26 lacks positive absence"), "{error}");
}


/// Visible white lower-face evidence cannot authorise an expected removed slot.
#[test]
fn expected_removed_slot_with_original_face_still_present_refuses() {
    let mut tableau = original_tableau();
    tableau[26] = None;
    let error = vision()
        .verify_position(&original(), &tableau, original_left(), None)
        .expect_err("the original visible face is not absent");
    assert!(error.contains("slot 26 lacks positive absence"), "{error}");
}


/// Positive lower felt accepts the removed bottom pair without ancestor top-strip guesses.
#[test]
fn partial_bottom_pair_accepts_positive_lower_felt_with_original_slots_retained() {
    let mut changed = original();
    let felt = felt_colour(&changed);
    let mut tableau = original_tableau();
    for index in [26, 27] {
        tableau[index] = None;
        fill_region(&mut changed, lower_probe(index), felt);
    }
    assert_eq!(tableau.iter().flatten().count(), 26);
    vision()
        .verify_position(&changed, &tableau, original_left(), None)
        .expect("both original bottom slots have positive lower felt");
    assert_eq!(tableau[20], Some(Card::new(Rank::Seven, Suit::Spades)));
}


/// Every complete scene signature remains mandatory for partial-route observation.
#[test]
fn changed_scene_signature_refuses_partial_route_observation() {
    let reader = vision();
    let tableau = original_tableau();
    for (index, pattern) in reader.board.scene.iter().enumerate() {
        let mut changed = original();
        fill_region(&mut changed, pattern.bounds, [17, 19, 21]);
        let error = reader
            .verify_position(&changed, &tableau, original_left(), None)
            .expect_err("unsupported scene");
        assert_eq!(
            error,
            format!(
                "execution board scene[{index}] mismatch at [{},{},{},{}]",
                pattern.bounds.x, pattern.bounds.y, pattern.bounds.width, pattern.bounds.height
            )
        );
    }
}


/// Exact route recognition depends on cards and scene rather than Solver toolbar pixels.
#[test]
fn changed_solver_toolbar_accepts_exact_partial_route_observation() {
    let reader = vision();
    let tableau = original_tableau();
    for colour in [[0, 0, 0], [255, 255, 255], [128, 128, 128], [255, 210, 0]] {
        let mut changed = original();
        fill_region(&mut changed, SOLVER_BOUNDS, colour);
        reader
            .verify_position(&changed, &tableau, original_left(), None)
            .expect("unrelated toolbar pixels do not revoke recognised position");
    }
}


/// Pile labels and positive-empty evidence cannot be inferred from missing glyphs.
#[test]
fn route_position_requires_exact_pile_identities_and_positive_empty_evidence() {
    let reader = vision();
    let tableau = original_tableau();
    assert!(
        reader
            .verify_position(&original(), &tableau, None, None)
            .is_err()
    );
    assert!(
        reader
            .verify_position(&original(), &tableau, original_left(), original_left())
            .is_err()
    );
    let mut changed = original();
    fill_region(&mut changed, reader.board.right_empty.bounds, [17, 19, 21]);
    assert!(
        reader
            .verify_position(&changed, &tableau, original_left(), None)
            .is_err()
    );
}


/// Malformed dimensions, stride and storage are refused before pixel interpretation.
#[test]
fn route_observation_refuses_invalid_native_layout_without_panicking() {
    let reader = vision();
    let tableau = original_tableau();
    let frame = original();
    let mut wrong_dimensions = frame.clone();
    wrong_dimensions.width = 1919;
    let mut short_stride = frame.clone();
    short_stride.stride = 1;
    let mut short_storage = frame;
    short_storage.pixels.truncate(16);
    for malformed in [wrong_dimensions, short_stride, short_storage] {
        assert!(
            reader
                .verify_position(&malformed, &tableau, original_left(), None)
                .is_err()
        );
    }
}
