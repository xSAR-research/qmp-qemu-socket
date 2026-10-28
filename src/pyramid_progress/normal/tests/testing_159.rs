//! Complete current ordinal-two appearance with the unchanged whole-header contract.

use super::*;


/// Decode the unchanged supported board which the 158 header bank refused.
fn retained() -> CapturedFrame {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recognition-159/normal-two.png"),
    )
    .unwrap();
    decode_png(&bytes).unwrap()
}


/// A complete Boards 2 of 3 header reports one already completed board.
#[test]
fn native_159_retained_normal_ordinal_two_has_exact_progress() {
    let frame = retained();
    let expected = Some(BoardCount {
        completed: 1,
        target: 3,
    });
    assert_eq!(observe_normal(&frame).unwrap(), expected);
    assert_eq!(observe_header(&frame).unwrap(), expected);
}


/// Erase an entire checked role inside a synthetic clone without changing its dimensions.
fn blank(frame: &mut CapturedFrame, bounds: PixelRect) {
    let stride = frame.stride;
    for y in bounds.y..bounds.y + bounds.height {
        for x in bounds.x..bounds.x + bounds.width {
            let offset = usize::try_from(y).unwrap() * stride + usize::try_from(x).unwrap() * 4;
            frame.pixels[offset..offset + 3].fill(0);
        }
    }
}


/// Neither a damaged complete ordinal nor a missing New footer authenticates this game.
#[test]
fn native_159_normal_appearance_retains_header_and_footer_refusals() {
    let mut missing_ordinal = retained();
    blank(&mut missing_ordinal, PixelRect::new(1780, 46, 38, 32));
    assert_eq!(observe_normal(&missing_ordinal).unwrap(), None);
    assert_eq!(observe_header(&missing_ordinal).unwrap(), None);
    let mut missing_footer = retained();
    blank(&mut missing_footer, PixelRect::new(212, 998, 58, 31));
    assert_eq!(observe_normal(&missing_footer).unwrap(), None);
    assert_eq!(
        observe_header(&missing_footer).unwrap(),
        Some(BoardCount {
            completed: 1,
            target: 3,
        }),
        "header-only effect observation grants no standard-game entry authority"
    );
}
