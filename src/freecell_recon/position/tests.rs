//! Retained native geometry and physical-binding refusals; no guest operations.
use super::*;
use xsar::games::freecell::rules::{RecordedMove, card_id};


/// Copy retained pixels after checking both rectangles and their byte ranges.
fn native_blit(
    destination: &mut CapturedFrame,
    source: &CapturedFrame,
    from: PixelRect,
    to: (u32, u32),
) {
    source.as_view().unwrap().checked_bounds(from).unwrap();
    destination
        .as_view()
        .unwrap()
        .checked_bounds(PixelRect::new(to.0, to.1, from.width, from.height))
        .unwrap();
    let index = |frame: &CapturedFrame, x: u32, y: u32| {
        usize::try_from(y)
            .unwrap()
            .checked_mul(frame.stride)
            .unwrap()
            .checked_add(usize::try_from(x).unwrap().checked_mul(4).unwrap())
            .unwrap()
    };
    let bytes = usize::try_from(from.width).unwrap().checked_mul(4).unwrap();
    for row in 0..from.height {
        let a = index(source, from.x, from.y.checked_add(row).unwrap());
        let b = index(destination, to.0, to.1.checked_add(row).unwrap());
        destination
            .pixels
            .get_mut(b..b + bytes)
            .unwrap()
            .copy_from_slice(source.pixels.get(a..a + bytes).unwrap());
    }
}


/// Establish complete independent initial evidence before deriving legal midgame history.
fn initial_position(frame: &CapturedFrame) -> Position {
    Position::initial(
        &super::super::recognise(
            frame,
            Metadata::new("test", PROFILE, "retained-native").unwrap(),
        )
        .unwrap(),
    )
}


/// Synthetic native-pixel AC transfer, not captured gameplay or auto timing proof.
#[test]
fn freecell_311_synthetic_foundation_and_structural_refusals() {
    let original = decode_png(REFERENCE).unwrap();
    let initial = initial_position(&original);
    let expected = initial
        .apply(RecordedMove {
            card_id: card_id(label("AC").unwrap()),
            from: Pile::Cascade(0),
            to: Pile::Foundation(0),
            is_auto: false,
        })
        .unwrap();
    let heldout = decode_png(include_bytes!("../../../tests/fixtures/freecell-FC18.png")).unwrap();
    let donor = PixelRect::new(50, 597, 143, 350);
    assert!(all(&original, donor, felt).unwrap());
    let mut composed = original.clone();
    native_blit(&mut composed, &original, donor, (215, 597));
    native_blit(
        &mut composed,
        &heldout,
        PixelRect::new(410, 543, 137, 183),
        (218, 597),
    );
    native_blit(
        &mut composed,
        &original,
        PixelRect::new(218, 650, 137, 183),
        (1233, 112),
    );
    let geometry = verify(&composed, &expected, &mut || Ok(())).unwrap();
    assert_eq!(
        geometry.point(Pile::Cascade(0)).unwrap(),
        PixelPoint::new(286, 677)
    );
    assert_eq!(
        geometry.point(Pile::Foundation(0)).unwrap(),
        PixelPoint::new(1301, 203)
    );
    assert!(verify(&composed, &initial, &mut || Ok(())).is_err());
    assert!(verify(&original, &expected, &mut || Ok(())).is_err());
    let mut wrong = composed.clone();
    native_blit(
        &mut wrong,
        &heldout,
        PixelRect::new(1373, 543, 137, 183),
        (218, 597),
    );
    assert!(verify(&wrong, &expected, &mut || Ok(())).is_err());
    native_blit(
        &mut composed,
        &original,
        PixelRect::new(50, 404, 1, 3),
        (218, 404),
    );
    assert_eq!(seams(&composed, PLAY_X[0]).unwrap().len(), 6);
    assert!(
        verify(&composed, &expected, &mut || Ok(()))
            .unwrap_err()
            .contains("interrupted rail")
    );
}


/// Synthetic legal KS-to-CELL composition preserves every other physical pile.
#[test]
fn freecell_311_synthetic_occupied_cell() {
    let original = decode_png(REFERENCE).unwrap();
    let initial = initial_position(&original);
    let expected = initial
        .apply(RecordedMove {
            card_id: card_id(label("KS").unwrap()),
            from: Pile::Cascade(7),
            to: Pile::Cell(0),
            is_auto: false,
        })
        .unwrap();
    let heldout = decode_png(include_bytes!("../../../tests/fixtures/freecell-FC17.png")).unwrap();
    let donor = PixelRect::new(50, 543, 143, 404);
    assert!(all(&original, donor, felt).unwrap());
    let mut composed = original.clone();
    native_blit(&mut composed, &original, donor, (1562, 543));
    native_blit(
        &mut composed,
        &heldout,
        PixelRect::new(1041, 112, 137, 183),
        (1565, 543),
    );
    native_blit(
        &mut composed,
        &original,
        PixelRect::new(1565, 597, 137, 183),
        (166, 112),
    );
    let geometry = verify(&composed, &expected, &mut || Ok(())).unwrap();
    assert_eq!(
        geometry.point(Pile::Cascade(7)).unwrap(),
        PixelPoint::new(1633, 623)
    );
    assert_eq!(
        geometry.point(Pile::Cell(0)).unwrap(),
        PixelPoint::new(234, 203)
    );
    assert!(verify(&composed, &initial, &mut || Ok(())).is_err());
}


/// Native30px strip must observe its complete centre rank and printed colour.
#[test]
fn freecell_311_native_thirty_pixel_rank_is_observed() {
    let templates = bank(&decode_png(REFERENCE).unwrap()).unwrap();
    let frame = decode_png(include_bytes!("../../../tests/fixtures/freecell-FC14.png")).unwrap();
    identity(
        &frame,
        &templates,
        603,
        418,
        card_id(label("8S").unwrap()),
        30,
    )
    .unwrap();
    for wrong in ["7S", "8H"] {
        assert!(
            identity(
                &frame,
                &templates,
                603,
                418,
                card_id(label(wrong).unwrap()),
                30
            )
            .is_err()
        );
    }
}


/// Native occupied cells and four foundations compete full printed identities.
#[test]
fn freecell_311_native_occupied_tops_reject_contradictions() {
    let templates = bank(&decode_png(REFERENCE).unwrap()).unwrap();
    let frame = decode_png(include_bytes!("../../../tests/fixtures/freecell-FC17.png")).unwrap();
    for (x, code) in [
        (358, "KC"),
        (551, "6S"),
        (743, "KS"),
        (1041, "3H"),
        (1233, "5C"),
        (1425, "2D"),
        (1618, "5S"),
    ] {
        occupied_top(&frame, &templates, x, card_id(label(code).unwrap())).unwrap();
    }
    for (x, wrong) in [(358, "KS"), (1041, "3D"), (551, "6H")] {
        assert!(occupied_top(&frame, &templates, x, card_id(label(wrong).unwrap())).is_err());
    }
}


/// Unknown material below a genuine empty outline remains unsupported.
#[test]
fn freecell_311_unknown_material_below_empty_refuses() {
    let reference = decode_png(REFERENCE).unwrap();
    let mut frame =
        decode_png(include_bytes!("../../../tests/fixtures/freecell-FC05.png")).unwrap();
    let x = PLAY_X[7];
    empty_play(&frame, x).unwrap();
    native_blit(
        &mut frame,
        &reference,
        PixelRect::new(226, 331, 4, 4),
        (x + 40, 700),
    );
    assert!(seams(&frame, x).unwrap().is_empty());
    assert!(
        empty_play(&frame, x)
            .unwrap_err()
            .contains("below empty PLAY")
    );
}


/// Independently recognise the complete retained frame before establishing history.
#[test]
fn freecell_311_initial_physical_binding_and_geometry() {
    for png in [
        REFERENCE,
        include_bytes!("../../../tests/fixtures/freecell-FC01.png").as_slice(),
    ] {
        let frame = decode_png(png).unwrap();
        let deal =
            super::super::recognise(&frame, Metadata::new("test", PROFILE, "native").unwrap())
                .unwrap();
        let position = Position::initial(&deal);
        let geometry = verify(&frame, &position, &mut || Ok(())).unwrap();
        assert_eq!(
            geometry.point(Pile::Cascade(0)).unwrap(),
            PixelPoint::new(286, 730)
        );
        assert_eq!(
            geometry.point(Pile::Foundation(0)).unwrap(),
            PixelPoint::new(1301, 203)
        );
        assert_eq!(
            geometry.point(Pile::Foundation(2)).unwrap(),
            PixelPoint::new(1109, 203)
        );
        assert!(geometry.point(Pile::Cell(4)).is_err());
        let mut columns = deal.board().cascades().clone();
        columns.swap(0, 1);
        let moved = InitialDeal::new(
            BoardState::new(columns, [None; 4], [0; 4]).unwrap(),
            deal.metadata().clone(),
        )
        .unwrap();
        // Position equality is canonical, yet physical live binding must refuse.
        assert_eq!(position, Position::initial(&moved));
        assert!(verify(&frame, &Position::initial(&moved), &mut || Ok(())).is_err());
        assert!(verify(&frame, &position, &mut || Err("STOP".to_owned())).is_err());
    }
}


/// Absence of a seam cannot replace a positive empty-column proof.
#[test]
fn freecell_311_native_empty_columns_and_dashed_pointer_refusals() {
    for (png, column) in [
        (
            include_bytes!("../../../tests/fixtures/freecell-FC05.png").as_slice(),
            7,
        ),
        (
            include_bytes!("../../../tests/fixtures/freecell-FC08.png").as_slice(),
            7,
        ),
        (
            include_bytes!("../../../tests/fixtures/freecell-FC21.png").as_slice(),
            3,
        ),
    ] {
        let frame = decode_png(png).unwrap();
        let x = PLAY_X[column];
        assert!(seams(&frame, x).unwrap().is_empty());
        empty_play(&frame, x).unwrap();
    }
    let frame = decode_png(include_bytes!("../../../tests/fixtures/freecell-FC17.png")).unwrap();
    assert!(empty_play(&frame, PLAY_X[4]).is_err());
    let frame = decode_png(REFERENCE).unwrap();
    assert!(empty_play(&frame, PLAY_X[0]).is_err());
}


/// Compressed native seams are measured, without pretending hidden suits are readable.
#[test]
fn freecell_311_nonuniform_native_fan_is_measured() {
    let frame = decode_png(include_bytes!("../../../tests/fixtures/freecell-FC14.png")).unwrap();
    assert_eq!(
        seams(&frame, PLAY_X[2]).unwrap(),
        [331, 360, 389, 418, 448, 477, 530, 583, 636, 689, 742]
    );
}
