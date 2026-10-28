//! Retained native Recon frames preserve identity proof independently of toolbar colour.

use super::*;
use xsar::games::cards::{Rank, Suit};


/// Classify an original retained PNG with the unchanged production native-v23 reader.
fn board_png(png: &[u8]) -> BoardObservation {
    let vision = vision::ReconVision::bundled_native_v23().unwrap();
    let frame = crate::capture::decode_png(png).unwrap();
    match vision.observe(&frame).unwrap() {
        ReconScene::Board(board) => board,
        other => panic!("retained complete board expected, got {other:?}"),
    }
}


/// The actual themed exhausted board remains fully recognised with an unmatched icon.
#[test]
fn native_157_actual_patterned_recon_board_has_complete_identities_and_advisory_icon() {
    let initial = board_png(include_bytes!(
        "../../tests/fixtures/pyramid-transition-157/undo-baseline.png"
    ));
    let exhausted = board_png(include_bytes!(
        "../../tests/fixtures/pyramid-transition-157/undo-exhausted.png"
    ));
    assert_eq!(initial.tableau, exhausted.tableau);
    assert_eq!(initial.tableau.len(), 28);
    assert!(matches!(initial.left, StockFace::Card(_)));
    assert_eq!(initial.right, StockFace::Empty);
    assert_eq!(exhausted.left, StockFace::Empty);
    assert_eq!(
        exhausted.right,
        StockFace::Card(Card::new(Rank::Nine, Suit::Spades))
    );
    assert_eq!(exhausted.undo_all, None);
}
