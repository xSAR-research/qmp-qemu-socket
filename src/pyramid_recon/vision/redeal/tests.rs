//! Actual native stock/control proofs and refusal boundaries for the new capability.

use super::*;


mod challenge_tests;


/// Load the unchanged actual129 intermediate stock frame, never a synthetic control.
fn stock_boundary(name: &str) -> CapturedFrame {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-stock-recycle")
        .join(name);
    crate::capture::decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Independently read the twelve fixed slots visible after the actual partial route.
fn partial_tableau() -> [Option<Card>; 28] {
    let mut cards = [None; 28];
    for (slot, label) in [
        (0, "4D"),
        (1, "4H"),
        (2, "7D"),
        (3, "2C"),
        (4, "7S"),
        (5, "JC"),
        (6, "QC"),
        (7, "2H"),
        (8, "7H"),
        (11, "QS"),
        (12, "6D"),
        (17, "6S"),
    ] {
        cards[slot] = Some(card(label));
    }
    cards
}


/// The actual empty circle proves stock position without granting an early REDEAL.
#[test]
fn native_stock_circle_proves_position_but_never_redeal() {
    let reader = ReconVision::bundled_native_v22().unwrap();
    let frame = stock_boundary("last-observed.png");
    assert!(
        reader
            .verify_position(&frame, &partial_tableau(), None, Some(card("8S")))
            .is_err()
    );
    assert_eq!(
        reader
            .verify_native_stock_continuation(&frame, &partial_tableau(), None, Some(card("8S")))
            .unwrap(),
        None
    );
    assert_eq!(reader.remaining_native_redeals(&frame).unwrap(), Some(1));
    assert!(
        reader
            .verify_native_redeal_position(&frame, &partial_tableau(), Some(card("8S")))
            .is_err()
    );
    assert!(
        reader
            .verify_native_stock_continuation(
                &stock_boundary("first-unmatched.png"),
                &partial_tableau(),
                None,
                Some(card("8S"))
            )
            .is_err()
    );
}


/// Every exact circle/control/identity/static component remains required.
#[test]
fn native_stock_circle_refuses_missing_components_and_wrong_position() {
    let reader = ReconVision::bundled_native_v22().unwrap();
    for (x, y) in [(920, 678), (920, 704), (767, 712), (1796, 116), (1796, 388)] {
        let mut frame = stock_boundary("last-observed.png");
        let offset = y * frame.stride + x * 4;
        for channel in &mut frame.pixels[offset..offset + 3] {
            *channel ^= 255;
        }
        assert!(
            reader
                .verify_native_stock_continuation(
                    &frame,
                    &partial_tableau(),
                    None,
                    Some(card("8S"))
                )
                .is_err()
        );
    }
    let frame = stock_boundary("last-observed.png");
    assert!(
        reader
            .verify_native_stock_continuation(&frame, &partial_tableau(), None, Some(card("7S")))
            .is_err()
    );
    assert!(
        reader
            .verify_native_stock_continuation(&frame, &partial_tableau(), Some(card("8S")), None)
            .is_err()
    );
    let mut wrong = partial_tableau();
    wrong[0] = None;
    assert!(
        reader
            .verify_native_stock_continuation(&frame, &wrong, None, Some(card("8S")))
            .is_err()
    );
    let custom = ReconVision::from_bytes(include_bytes!(
        "../../../../tests/fixtures/pyramid-recon/native-v22/profile.json"
    ))
    .unwrap();
    assert!(
        custom
            .verify_native_stock_continuation(&frame, &partial_tableau(), None, Some(card("8S")))
            .is_err()
    );
}


/// Caption changes cannot turn intermediate position proof into a REDEAL ticket.
#[test]
fn native_stock_circle_does_not_depend_on_or_authorise_an_allowance() {
    let reader = ReconVision::bundled_native_v22().unwrap();
    let zero = frame("after.png");
    let mut frame = stock_boundary("last-observed.png");
    for y in 874..900 {
        let begin = y * frame.stride + 890 * 4;
        let end = begin + 140 * 4;
        frame.pixels[begin..end].copy_from_slice(&zero.pixels[begin..end]);
    }
    assert_eq!(reader.remaining_native_redeals(&frame).unwrap(), Some(0));
    assert_eq!(
        reader
            .verify_native_stock_continuation(&frame, &partial_tableau(), None, Some(card("8S")))
            .unwrap(),
        None
    );
    assert!(
        reader
            .verify_native_redeal_position(&frame, &partial_tableau(), Some(card("8S")))
            .is_err()
    );
}


/// Decode unchanged originals retained with their source hashes.
fn frame(name: &str) -> CapturedFrame {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recon/native-redeal")
        .join(name);
    crate::capture::decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Read independently recorded conventional labels without consulting recognition.
fn card(label: &str) -> Card {
    let (rank, suit) = label.split_at(label.len() - 1);
    let rank = Rank::ALL
        .into_iter()
        .find(|value| value.as_str() == rank)
        .unwrap();
    let suit = match suit {
        "C" => Suit::Clubs,
        "D" => Suit::Diamonds,
        "H" => Suit::Hearts,
        "S" => Suit::Spades,
        _ => panic!("direct suit"),
    };
    Card::new(rank, suit)
}


/// Independently read the unchanged pre-redeal board from all three native originals.
fn tableau() -> [Option<Card>; 28] {
    [
        "3C", "5D", "KH", "4H", "7S", "4D", "2H", "9C", "9D", "7D", "8C", "JS", "10D", "4S", "AH",
        "3D", "10S", "4C", "7H", "QD", "6C", "AD", "KS", "5H", "3S", "KD", "JD", "KC",
    ]
    .map(|label| Some(card(label)))
}


/// A positive native X, exact RIGHT and complete enabled icon/caption are all required.
#[test]
fn native_redeal_requires_the_actual_position_and_positive_allowance() {
    let reader = ReconVision::bundled_native_v22().unwrap();
    let exhausted = frame("exhausted.png");
    assert!(
        reader
            .verify_position(&exhausted, &tableau(), None, Some(card("8S")))
            .is_err()
    );
    assert_eq!(
        reader
            .verify_native_redeal_position(&exhausted, &tableau(), Some(card("8S")))
            .unwrap(),
        PixelPoint::new(960, 825)
    );
    assert!(
        reader
            .verify_native_redeal_position(&exhausted, &tableau(), Some(card("7S")))
            .is_err()
    );
    assert_eq!(
        reader.remaining_native_redeals(&exhausted).unwrap(),
        Some(1)
    );
    assert_eq!(
        reader
            .remaining_native_redeals(&frame("after.png"))
            .unwrap(),
        Some(0)
    );
    assert!(
        reader
            .verify_native_redeal_position(&frame("before.png"), &tableau(), None)
            .is_err()
    );
    let mut wrong = tableau();
    wrong[0] = None;
    assert!(
        reader
            .verify_native_redeal_position(&exhausted, &wrong, Some(card("8S")))
            .is_err()
    );
}


/// Erased icon, caption, static scene or X cannot be replaced by unknown evidence.
#[test]
fn native_redeal_refuses_each_missing_component_and_custom_profile() {
    let reader = ReconVision::bundled_native_v22().unwrap();
    for (x, y) in [(936, 792), (767, 712), (1796, 116)] {
        let mut original = frame("exhausted.png");
        let offset = y * original.stride + x * 4;
        for channel in &mut original.pixels[offset..offset + 3] {
            *channel ^= 255;
        }
        assert!(
            reader
                .verify_native_redeal_position(&original, &tableau(), Some(card("8S")))
                .is_err()
        );
    }
    let custom = ReconVision::from_bytes(include_bytes!(
        "../../../../tests/fixtures/pyramid-recon/native-v22/profile.json"
    ))
    .unwrap();
    assert!(
        custom
            .verify_native_redeal_position(&frame("exhausted.png"), &tableau(), Some(card("8S")))
            .is_err()
    );
}
