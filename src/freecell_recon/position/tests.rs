//! Native printed-corner location, scoped physical identity and empty-slot refusals.
//! Compositions below are synthetic regression fixtures, never guest-run evidence.

use super::*;
use xsar::games::freecell::rules::RecordedMove;


/// Copy retained native pixels after checking source and destination rectangles.
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
    for row in 0..from.height as usize {
        let a = (from.y as usize + row) * source.stride + from.x as usize * 4;
        let b = (to.1 as usize + row) * destination.stride + to.0 as usize * 4;
        let bytes = from.width as usize * 4;
        destination
            .pixels
            .get_mut(b..b + bytes)
            .unwrap()
            .copy_from_slice(source.pixels.get(a..a + bytes).unwrap());
    }
}


/// Paint a bounded synthetic region while preserving the source frame's storage.
fn paint(frame: &mut CapturedFrame, rect: PixelRect, rgb: [u8; 3]) {
    frame.as_view().unwrap().checked_bounds(rect).unwrap();
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            let offset = y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&rgb);
        }
    }
}


/// Bind every initial identity independently before deriving legal physical history.
fn initial_position(frame: &CapturedFrame) -> Position {
    Position::initial(
        &super::super::recognise(
            frame,
            Metadata::new("test", PROFILE, "retained-native").unwrap(),
        )
        .unwrap(),
    )
}


/// Require the click to stay within the authenticated native rank corner.
fn assert_corner(point: PixelPoint, x: u32, top: u32) {
    assert_eq!(point.x, (x + 15) as i32);
    assert!(
        ((top + 18) as i32..=(top + 23) as i32).contains(&point.y),
        "{point:?}"
    );
}


/// Authentic incident frames freshly identify exposed cards without complete-face evidence.
#[test]
fn freecell_314_native_normal_and_gold_positions_use_printed_corners() {
    for (png, column, top) in [
        (REFERENCE, 0, 650),
        (
            include_bytes!("../../../tests/fixtures/freecell-FC01.png").as_slice(),
            0,
            650,
        ),
        (
            include_bytes!("../../../tests/fixtures/freecell-normal-six-20261110.png").as_slice(),
            6,
            597,
        ),
        (
            include_bytes!("../../../tests/fixtures/freecell-gold-five-heart-20261110.png")
                .as_slice(),
            0,
            650,
        ),
        (
            include_bytes!("../../../tests/fixtures/freecell-challenge-highlight-20261109.png")
                .as_slice(),
            1,
            650,
        ),
    ] {
        let native = decode_png(png).unwrap();
        let expected = initial_position(&native);
        let geometry = verify(&native, &expected, &mut || Ok(())).unwrap();
        assert_corner(
            geometry.point(Pile::Cascade(column)).unwrap(),
            PLAY_X[column as usize],
            top,
        );
        assert_eq!(
            geometry.point(Pile::Foundation(0)).unwrap(),
            PixelPoint::new(1248, 132)
        );
        assert_eq!(
            geometry.point(Pile::Foundation(2)).unwrap(),
            PixelPoint::new(1056, 132)
        );
        assert!(geometry.point(Pile::Cell(4)).is_err());
    }
}


/// Position readers use the shared foreground reader and independently read both native ranks.
#[test]
fn freecell_3121_native_rank_supplements_are_shared_with_position_proofs() {
    let reference = decode_png(REFERENCE).unwrap();
    for (png, x, y, code) in [
        (
            include_bytes!("../../../tests/fixtures/freecell-native-eight-20261112.png").as_slice(),
            795,
            331,
            "8D",
        ),
        (
            include_bytes!("../../../tests/fixtures/freecell-native-nine-20261112.png").as_slice(),
            1180,
            543,
            "9D",
        ),
    ] {
        let native = decode_png(png).unwrap();
        let expected = initial_position(&native);
        verify(&native, &expected, &mut || Ok(())).unwrap();
        assert_eq!(
            observed_identity(&native, x, y).unwrap(),
            card_id(label(code).unwrap())
        );
        // Synthetic exposed placement tests independent position competition, not route history.
        let mut exposed = reference.clone();
        native_blit(
            &mut exposed,
            &native,
            PixelRect::new(x, y, 30, 48),
            (218, 650),
        );
        assert_corner(
            occupied_play(&exposed, 218, card_id(label(code).unwrap()), &mut || Ok(())).unwrap(),
            218,
            650,
        );
        assert!(
            occupied_play(&exposed, 218, card_id(label("AC").unwrap()), &mut || Ok(())).is_err()
        );
        for rect in [
            PixelRect::new(219, 656, 28, 25),
            PixelRect::new(222, 679, 20, 18),
        ] {
            let mut damaged = exposed.clone();
            paint(&mut damaged, rect, [255, 255, 255]);
            assert!(
                occupied_play(&damaged, 218, card_id(label(code).unwrap()), &mut || {
                    Ok(())
                })
                .is_err()
            );
        }
        let mut mixed = exposed;
        native_blit(
            &mut mixed,
            &reference,
            PixelRect::new(222, 679, 20, 18),
            (222, 679),
        );
        assert!(
            observed_identity(&mixed, 218, 650)
                .unwrap_err()
                .contains("colour disagreement")
        );
        assert!(occupied_play(&mixed, 218, card_id(label(code).unwrap()), &mut || Ok(())).is_err());
    }
}


/// Queen descenders occupy the final rank-crop rows; they are never a quiet gutter.
#[test]
fn freecell_314_native_queen_tail_cannot_hide_the_exposed_corner() {
    for (png, x) in [
        (
            include_bytes!("../../../tests/fixtures/freecell-FC01.png").as_slice(),
            795,
        ),
        (
            include_bytes!("../../../tests/fixtures/freecell-sixes-decorative-20261111.png")
                .as_slice(),
            603,
        ),
    ] {
        let native = decode_png(png).unwrap();
        assert_eq!(
            observed_identity(&native, x, 650).unwrap(),
            card_id(label("QH").unwrap())
        );
        assert!(corner_candidate(&native, x, 650).unwrap());
        assert_corner(
            occupied_play(&native, x, card_id(label("QH").unwrap()), &mut || Ok(())).unwrap(),
            x,
            650,
        );
    }
}


/// A one-channel decorative rail difference cannot veto an independently read exposed card.
#[test]
fn freecell_314_decorative_top_rail_and_full_face_pixels_are_advisory() {
    let native = decode_png(include_bytes!(
        "../../../tests/fixtures/freecell-gold-five-heart-20261110.png"
    ))
    .unwrap();
    let expected = initial_position(&native);
    for (rect, rgb) in [
        (PixelRect::new(218, 392, 1, 37), [237, 210, 50]),
        (PixelRect::new(354, 392, 1, 37), [218, 201, 122]),
        (PixelRect::new(226, 384, 121, 1), [23, 41, 73]),
        (PixelRect::new(226, 832, 121, 1), [23, 41, 73]),
        (PixelRect::new(218, 700, 1, 100), [23, 41, 73]),
        (PixelRect::new(250, 710, 70, 90), [23, 41, 73]),
    ] {
        let mut changed = native.clone();
        paint(&mut changed, rect, rgb);
        let geometry = verify(&changed, &expected, &mut || Ok(()))
            .unwrap_or_else(|cause| panic!("decorative change {rect:?} {rgb:?}: {cause}"));
        assert_corner(geometry.point(Pile::Cascade(0)).unwrap(), 218, 650);
    }
}


/// Only requested exposed piles supply this operation's geometry or refusal authority.
#[test]
fn freecell_314_scoped_proof_ignores_unrelated_card_but_checks_its_own_source() {
    let native = decode_png(REFERENCE).unwrap();
    let expected = initial_position(&native);
    let mut unrelated = native.clone();
    paint(
        &mut unrelated,
        PixelRect::new(1566, 603, 28, 25),
        [255, 255, 255],
    );
    let geometry = verify_piles(
        &unrelated,
        &expected,
        &[Pile::Cascade(0), Pile::Cell(0)],
        &mut || Ok(()),
    )
    .unwrap();
    assert_corner(geometry.point(Pile::Cascade(0)).unwrap(), 218, 650);
    assert!(geometry.point(Pile::Cascade(7)).is_err());
    assert!(verify(&unrelated, &expected, &mut || Ok(())).is_err());
    assert!(verify_piles(&unrelated, &expected, &[Pile::Cascade(7)], &mut || Ok(())).is_err());
    assert!(verify_piles(&native, &expected, &[Pile::Cascade(8)], &mut || Ok(())).is_err());
}


/// Hidden prefixes remain owned by RECON52/history, even when a visible old corner changes.
#[test]
fn freecell_314_hidden_identity_is_not_recounted_or_compared_to_history() {
    let native = decode_png(REFERENCE).unwrap();
    let expected = initial_position(&native);
    let mut changed = native.clone();
    native_blit(
        &mut changed,
        &native,
        PixelRect::new(410, 331, 30, 48),
        (218, 331),
    );
    verify_piles(&changed, &expected, &[Pile::Cascade(0)], &mut || Ok(())).unwrap();
    assert!(
        super::super::recognise(
            &changed,
            Metadata::new("test", PROFILE, "synthetic").unwrap()
        )
        .is_err()
    );
}


/// Missing, mixed-colour, wrong and ambiguous exposed labels cannot be completed by expectation.
#[test]
fn freecell_314_exposed_glyph_failures_remain_refusals() {
    let native = decode_png(REFERENCE).unwrap();
    let expected = initial_position(&native);
    let mut missing_rank = native.clone();
    paint(
        &mut missing_rank,
        PixelRect::new(219, 656, 28, 25),
        [255, 255, 255],
    );
    let mut missing_suit = native.clone();
    paint(
        &mut missing_suit,
        PixelRect::new(222, 679, 20, 18),
        [255, 255, 255],
    );
    let mut missing_both = native.clone();
    paint(
        &mut missing_both,
        PixelRect::new(219, 656, 28, 41),
        [255, 255, 255],
    );
    let mut wrong = native.clone();
    native_blit(
        &mut wrong,
        &native,
        PixelRect::new(1565, 597, 30, 48),
        (218, 650),
    );
    let mut mixed = native.clone();
    native_blit(
        &mut mixed,
        &native,
        PixelRect::new(414, 360, 20, 18),
        (222, 679),
    );
    let mut uncertain = native.clone();
    super::super::tests::paint_ambiguous_rank(&mut uncertain, 218, 650);
    assert!(matches!(
        crate::game_controls::classify_corner(&uncertain, (218, 650)).unwrap(),
        crate::pyramid_recon::vision::CornerClassification::Ambiguous { .. }
    ));
    for (name, frame) in [
        ("rank", missing_rank),
        ("suit", missing_suit),
        ("both", missing_both),
        ("wrong", wrong),
        ("mixed", mixed),
        ("ambiguous", uncertain),
    ] {
        assert!(
            verify_piles(&frame, &expected, &[Pile::Cascade(0)], &mut || Ok(())).is_err(),
            "{name}"
        );
    }
}


/// A lower uncertain corner cannot be skipped in favour of a matching upper card.
#[test]
fn freecell_314_unknown_lower_corner_and_duplicate_identity_refuse() {
    let native = decode_png(REFERENCE).unwrap();
    let expected = initial_position(&native);
    let mut duplicate = native.clone();
    native_blit(
        &mut duplicate,
        &native,
        PixelRect::new(218, 650, 30, 48),
        (218, 850),
    );
    assert!(
        verify_piles(&duplicate, &expected, &[Pile::Cascade(0)], &mut || Ok(()))
            .unwrap_err()
            .contains("Duplicate")
    );
    let mut lower = duplicate.clone();
    super::super::tests::paint_ambiguous_rank(&mut lower, 218, 850);
    assert!(matches!(
        crate::game_controls::classify_corner(&lower, (218, 850)).unwrap(),
        crate::pyramid_recon::vision::CornerClassification::Ambiguous { .. }
    ));
    assert!(
        corner_identity(&lower, 218, 850)
            .unwrap()
            .unwrap_err()
            .contains("AmbiguousIdentity")
    );
    assert!(
        verify_piles(&lower, &expected, &[Pile::Cascade(0)], &mut || Ok(()))
            .unwrap_err()
            .contains("AmbiguousIdentity")
    );
    let mut missing_rank = duplicate.clone();
    paint(
        &mut missing_rank,
        PixelRect::new(219, 856, 28, 25),
        [255, 255, 255],
    );
    let mut missing_suit = duplicate.clone();
    paint(
        &mut missing_suit,
        PixelRect::new(222, 879, 20, 18),
        [255, 255, 255],
    );
    let mut mixed = duplicate.clone();
    native_blit(
        &mut mixed,
        &native,
        PixelRect::new(414, 360, 20, 18),
        (222, 879),
    );
    let mut partial = duplicate;
    paint(
        &mut partial,
        PixelRect::new(222, 879, 20, 5),
        [255, 255, 255],
    );
    for (name, frame) in [
        ("missing lower rank", missing_rank),
        ("missing lower suit", missing_suit),
        ("mixed lower ink", mixed),
        ("partial lower suit", partial),
    ] {
        assert!(
            verify_piles(&frame, &expected, &[Pile::Cascade(0)], &mut || Ok(())).is_err(),
            "{name}"
        );
    }
}


/// The locator follows nonuniform native compression and the highlighted final5S corner.
/// FC14's Solver scene remains unsupported for input; this test exercises pixels only.
#[test]
fn freecell_314_nonuniform_native_fan_has_no_hidden_count_or_full_face_requirement() {
    let native = decode_png(include_bytes!("../../../tests/fixtures/freecell-FC14.png")).unwrap();
    let observed = corners(&native, 603, &mut || Ok(())).unwrap();
    let known: Vec<_> = observed
        .iter()
        .filter_map(|c| c.id.map(|id| (c.top(), id)))
        .collect();
    assert!(
        known
            .iter()
            .any(|(top, id)| (688..=692).contains(top) && *id == card_id(label("7S").unwrap()))
    );
    assert!(
        known.len() < 12,
        "compressed hidden suits are not counted: {known:?}"
    );
    let point = occupied_play(&native, 603, card_id(label("5S").unwrap()), &mut || Ok(())).unwrap();
    assert_corner(point, 603, 795);
    assert!(occupied_play(&native, 603, card_id(label("6D").unwrap()), &mut || Ok(())).is_err());
}


/// A long column's final readable printed corner need not expose its lower face.
#[test]
fn freecell_314_low_corner_stays_above_toolbar_without_proving_a_card_body() {
    let original = decode_png(REFERENCE).unwrap();
    let mut composed = original.clone();
    native_blit(
        &mut composed,
        &original,
        PixelRect::new(218, 650, 30, 48),
        (218, 898),
    );
    paint(
        &mut composed,
        PixelRect::new(219, 656, 28, 41),
        [255, 255, 255],
    );
    assert_corner(
        occupied_play(
            &composed,
            218,
            card_id(label("AC").unwrap()),
            &mut || Ok(()),
        )
        .unwrap(),
        218,
        898,
    );
}


/// Positive empty outlines and absence of card paper are retained only for actual emptiness.
#[test]
fn freecell_314_empty_slots_keep_positive_absence_and_occupancy_refusals() {
    let reference = decode_png(REFERENCE).unwrap();
    for x in CELL_X.into_iter().chain(FOUNDATION_X) {
        empty_top(&reference, x).unwrap();
        let mut broken = reference.clone();
        paint(&mut broken, PixelRect::new(x + 30, 113, 4, 3), [30, 80, 40]);
        assert!(empty_top(&broken, x).is_err());
        let mut occupied = reference.clone();
        paint(
            &mut occupied,
            PixelRect::new(x + 40, 150, 4, 4),
            [255, 255, 255],
        );
        assert!(empty_top(&occupied, x).is_err());
    }
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
        let native = decode_png(png).unwrap();
        empty_play(&native, PLAY_X[column]).unwrap();
        let mut broken = native.clone();
        paint(
            &mut broken,
            PixelRect::new(PLAY_X[column] + 30, 332, 4, 3),
            [30, 80, 40],
        );
        assert!(empty_play(&broken, PLAY_X[column]).is_err());
    }
    assert!(empty_play(&reference, PLAY_X[0]).is_err());
    let dashed = decode_png(include_bytes!("../../../tests/fixtures/freecell-FC17.png")).unwrap();
    assert!(empty_play(&dashed, PLAY_X[4]).is_err());
}


/// Occupied CELL/foundation corners compete all labels without card rail/full-face gates.
#[test]
fn freecell_314_occupied_upper_slots_keep_exact_physical_identities() {
    let native = decode_png(include_bytes!("../../../tests/fixtures/freecell-FC17.png")).unwrap();
    for (x, code) in [
        (358, "KC"),
        (551, "6S"),
        (743, "KS"),
        (1041, "3H"),
        (1233, "5C"),
        (1425, "2D"),
        (1618, "5S"),
    ] {
        let point = occupied_top(&native, x, card_id(label(code).unwrap())).unwrap();
        assert!((x as i32 + 14..=x as i32 + 15).contains(&point.x));
        assert_eq!(point.y, 132);
    }
    for (x, code) in [(358, "KS"), (1041, "3D"), (551, "6H")] {
        assert!(occupied_top(&native, x, card_id(label(code).unwrap())).is_err());
    }
    let mut broken = native.clone();
    paint(&mut broken, PixelRect::new(358, 120, 1, 160), [23, 41, 73]);
    occupied_top(&broken, 358, card_id(label("KC").unwrap())).unwrap();
}


/// Synthetic legal AC transfer proves only actual changed physical endpoints.
#[test]
fn freecell_314_synthetic_foundation_transfer_refuses_parent_and_wrong_top() {
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
    let mut composed = original.clone();
    native_blit(
        &mut composed,
        &original,
        PixelRect::new(50, 597, 143, 350),
        (215, 597),
    );
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
    let piles = [Pile::Cascade(0), Pile::Foundation(0)];
    let geometry = verify_piles(&composed, &expected, &piles, &mut || Ok(())).unwrap();
    assert_corner(geometry.point(Pile::Cascade(0)).unwrap(), 218, 597);
    assert_eq!(
        geometry.point(Pile::Foundation(0)).unwrap(),
        PixelPoint::new(1248, 132)
    );
    assert!(verify_piles(&composed, &initial, &piles, &mut || Ok(())).is_err());
    assert!(verify_piles(&original, &expected, &piles, &mut || Ok(())).is_err());
    native_blit(
        &mut composed,
        &heldout,
        PixelRect::new(1373, 543, 30, 48),
        (218, 597),
    );
    assert!(verify_piles(&composed, &expected, &piles, &mut || Ok(())).is_err());
}


/// Cancellation remains observable while scanning; unsupported native scenes still refuse.
#[test]
fn freecell_314_cancellation_bounds_and_scene_guards_remain_active() {
    let native = decode_png(REFERENCE).unwrap();
    let expected = initial_position(&native);
    assert!(
        verify(&native, &expected, &mut || Err("STOP".into()))
            .unwrap_err()
            .contains("STOP")
    );
    let mut checks = 0;
    let error = verify_piles(&native, &expected, &[Pile::Cascade(0)], &mut || {
        checks += 1;
        if checks >= 15 {
            Err("STOP during scan".into())
        } else {
            Ok(())
        }
    })
    .unwrap_err();
    assert!(error.contains("STOP during scan"));
    assert_eq!(checks, 15);
    for png in [
        include_bytes!("../../../tests/fixtures/freecell-FC02.png").as_slice(),
        include_bytes!("../../../tests/fixtures/freecell-FC09.png").as_slice(),
        include_bytes!("../../../tests/fixtures/freecell-FC12.png").as_slice(),
    ] {
        assert!(verify(&decode_png(png).unwrap(), &expected, &mut || Ok(())).is_err());
    }
    let mut malformed = native;
    malformed.width = 1919;
    assert!(verify(&malformed, &expected, &mut || Ok(())).is_err());
}


/// Retained live Fours Challenge mid-game frame verifies all 16 piles including empty `PLAY 1`.
#[test]
fn freecell_320_live_empty_play1_fours_challenge_verifies_all_sixteen_piles() {
    let frame = decode_png(include_bytes!(
        "../../../tests/fixtures/freecell-empty-play1-fours-20261113.png"
    ))
    .expect("retained live empty PLAY 1 frame");
    let raw_cascades: [&[&str]; 8] = [
        &["3H", "3D", "JH", "7H", "2C", "QS", "8S"],
        &["KD", "4S", "KH", "6D", "6S", "KC", "AS"],
        &["QC", "TC", "4C", "9S", "TD", "2H", "9C"],
        &["4D", "5H", "QD", "AD", "7D", "TS", "KS"],
        &["8H", "3C", "5S", "AC", "AH", "9D"],
        &["5C", "2D", "5D", "6C", "8C", "4H"],
        &["7S", "TH", "7C", "9H", "QH", "JS"],
        &["3S", "JD", "8D", "JC", "2S", "6H"],
    ];
    let cascades = raw_cascades.map(|column| {
        let cards: Vec<_> = column
            .iter()
            .map(|code| label(code).expect("valid card shorthand"))
            .collect();
        CascadeStack::from_cards(cards).expect("valid initial cascade")
    });
    let deal = InitialDeal::new(
        BoardState::new(cascades, [None; 4], [0; 4]).expect("valid initial board"),
        Metadata::new("test", PROFILE, "retained-native").expect("valid metadata"),
    )
    .expect("valid 52-card initial deal");
    let mut expected = Position::initial(&deal);
    let moves = [
        ("AS", Pile::Cascade(1), Pile::Foundation(3)),
        ("9D", Pile::Cascade(4), Pile::Cell(0)),
        ("AH", Pile::Cascade(4), Pile::Foundation(2)),
        ("AC", Pile::Cascade(4), Pile::Foundation(0)),
        ("5S", Pile::Cascade(4), Pile::Cascade(7)),
        ("9C", Pile::Cascade(2), Pile::Cell(3)),
        ("2H", Pile::Cascade(2), Pile::Foundation(2)),
        ("8S", Pile::Cascade(0), Pile::Cell(1)),
        ("QS", Pile::Cascade(0), Pile::Cell(2)),
        ("2C", Pile::Cascade(0), Pile::Foundation(0)),
        ("3C", Pile::Cascade(4), Pile::Foundation(0)),
        ("TD", Pile::Cascade(2), Pile::Cascade(6)),
        ("9S", Pile::Cascade(2), Pile::Cascade(6)),
        ("8H", Pile::Cascade(4), Pile::Cascade(6)),
        ("QS", Pile::Cell(2), Pile::Cascade(4)),
        ("7H", Pile::Cascade(0), Pile::Cell(2)),
        ("JH", Pile::Cascade(0), Pile::Cascade(4)),
        ("3D", Pile::Cascade(0), Pile::Cascade(2)),
        ("3H", Pile::Cascade(0), Pile::Foundation(2)),
    ];
    for (code, from, to) in moves {
        expected = expected
            .apply(RecordedMove {
                card_id: card_id(label(code).expect("valid move card")),
                from,
                to,
                is_auto: false,
            })
            .expect("legal Free Cell move");
    }

    let geometry = verify(&frame, &expected, &mut || Ok(())).expect("all 16 piles verify");
    assert_eq!(
        geometry.point(Pile::Cascade(0)).expect("empty PLAY 1"),
        PixelPoint::new(233, 351)
    );
    assert_eq!(
        geometry
            .point(Pile::Foundation(1))
            .expect("empty SUIT Diamonds"),
        PixelPoint::new(1440, 132)
    );

    let error = super::super::recognise_playback_initial_checked(
        &frame,
        Metadata::new("test", PROFILE, "retained-native").expect("valid metadata"),
        &mut || Ok(()),
    )
    .expect_err("mid-game board must reject initial binding on occupied CELL 1");
    assert_eq!(error.kind, FailureKind::TopSlotNotEmpty);
    assert!(
        error
            .to_string()
            .contains("Card paper interrupts empty-slot interior"),
        "unexpected CELL 1 diagnostic: {error}"
    );
}
