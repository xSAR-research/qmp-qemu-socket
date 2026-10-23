//! Deterministic acceptance, separation and witness guards for graded glyphs.
//!
//! Synthetic RGB regions exercise decision arithmetic independently of native
//! calibration. They establish refusal and exact boundary behaviour only; no
//! test creates capture, restoration or guest-input authority.

use super::*;
use serde_json::json;
use xsar::image_matching::{CapturedFrame, PixelFormat};


/// Construct a validated tightly packed RGB region with deliberately varied alpha.
fn frame(width: u32, height: u32, rgb: &[[u8; 3]]) -> CapturedFrame {
    assert_eq!(rgb.len(), width as usize * height as usize);
    let mut pixels = Vec::with_capacity(rgb.len() * 4);
    for colour in rgb {
        pixels.extend_from_slice(colour);
        pixels.push(17);
    }

    CapturedFrame {
        width,
        height,
        stride: width as usize * 4,
        format: PixelFormat::Rgba8,
        pixels,
    }
}


/// Create one complete exact RGB template without excluded negative-space pixels.
fn pattern(width: u32, height: u32, rgb: &[[u8; 3]]) -> Pattern {
    assert_eq!(rgb.len(), width as usize * height as usize);
    Pattern {
        bounds: PixelRect::new(0, 0, width, height),
        pixels: rgb.iter().copied().map(Some).collect(),
        tolerance: 0,
    }
}


/// Return eight strong black pixels above twenty-four genuine paper pixels.
fn clear_glyph() -> Vec<[u8; 3]> {
    let mut rgb = vec![[255; 3]; 32];
    rgb[..8].fill([0; 3]);
    rgb
}


/// Evaluate one synthetic bank under the actual frozen rank policy at zero shift.
fn evaluate_rank(bank: &[(u8, Vec<Pattern>)], actual: &CapturedFrame) -> Evaluation<u8> {
    evaluate(
        bank,
        actual.as_view().expect("valid synthetic RGBA layout"),
        (0, 0),
        RANK_MAX_ERROR_PPM,
        MIN_LABEL_GAP_PPM,
        0,
    )
    .expect("bounded synthetic glyph evaluation")
}


/// Extract a complete measured witness while making refusal an explicit test failure.
fn measured_witness(
    template: &Pattern,
    view: FrameView<'_>,
    origin: (u32, u32),
    shift: (i32, i32),
) -> Witness<u8> {
    match observe(1_u8, 0, template, view, origin, shift).expect("bounded measured ROI") {
        Observation::Witness(witness) => witness,
        refusal => panic!("expected a measured witness, received {refusal:?}"),
    }
}


/// Name synthetic labels in explanatory output without treating them as cards.
fn label_name(label: u8) -> &'static str {
    match label {
        1 => "first",
        2 => "second",
        _ => "other",
    }
}


/// Return exactly the immutable schema-2 policy accepted by production parsing.
fn policy_value() -> Value {
    json!({
        "policy": "paper_ink_l1_v2",
        "score_scale": 1_000_000,
        "rank_max_error_ppm": 220_000,
        "suit_max_error_ppm": 120_000,
        "min_label_gap_ppm": 80_000,
    })
}


/// Inclusive quality decisions use exact fractions even when diagnostic ppm agrees.
#[test]
fn exact_quality_bounds_do_not_accept_a_floored_excess() {
    for (bound, numerator, denominator) in
        [(RANK_MAX_ERROR_PPM, 11, 50), (SUIT_MAX_ERROR_PPM, 3, 25)]
    {
        let exact = Ratio {
            numerator,
            denominator,
        };
        assert!(exact.within(bound).expect("bounded exact fraction"));
        let below = Ratio {
            numerator: u64::from(bound) * 10 - 1,
            denominator: 10_000_000,
        };
        let above = Ratio {
            numerator: u64::from(bound) * 10 + 1,
            denominator: 10_000_000,
        };
        assert!(below.within(bound).expect("bounded fraction below cap"));
        assert!(!above.within(bound).expect("bounded fraction above cap"));
        assert_eq!(above.diagnostic_ppm(), u64::from(bound));
        assert_eq!(exact.diagnostic_ppm(), above.diagnostic_ppm());
    }
    let third = Ratio {
        numerator: 1,
        denominator: 3,
    };
    let truncated = Ratio {
        numerator: 333_333,
        denominator: 1_000_000,
    };
    assert_eq!(third.diagnostic_ppm(), truncated.diagnostic_ppm());
    assert_eq!(third.compare(truncated), Ok(Ordering::Greater));
    assert_eq!(truncated.compare(third), Ok(Ordering::Less));
    assert_eq!(
        third.compare(Ratio {
            numerator: 2,
            denominator: 6
        }),
        Ok(Ordering::Equal)
    );
}


/// Different-label separation is inclusive, positive and independent of ppm flooring.
#[test]
fn exact_label_gap_accepts_the_boundary_and_refuses_ties_or_smaller_gaps() {
    let best = Ratio {
        numerator: 1,
        denominator: 5,
    };
    let exact = Ratio {
        numerator: 7,
        denominator: 25,
    };
    let below = Ratio {
        numerator: 2_799_999,
        denominator: 10_000_000,
    };
    let above = Ratio {
        numerator: 2_800_001,
        denominator: 10_000_000,
    };
    assert!(
        best.separated_from(exact, MIN_LABEL_GAP_PPM)
            .expect("exact margin")
    );
    assert!(
        !best
            .separated_from(below, MIN_LABEL_GAP_PPM)
            .expect("sub-margin gap")
    );
    assert!(
        best.separated_from(above, MIN_LABEL_GAP_PPM)
            .expect("above-margin gap")
    );
    assert!(
        !best
            .separated_from(best, 0)
            .expect("zero policy still refuses a tie")
    );
    assert!(exact.separated_from(best, MIN_LABEL_GAP_PPM).is_err());
    let uneven = Ratio {
        numerator: 1,
        denominator: 3,
    };
    let uneven_other = Ratio {
        numerator: 31,
        denominator: 75,
    };
    assert!(
        uneven
            .separated_from(uneven_other, MIN_LABEL_GAP_PPM)
            .expect("uneven denominators")
    );
}


/// Multiple variants of one label share a best witness and cannot become competitors.
#[test]
fn same_label_variants_share_one_candidate_while_other_label_ties_refuse() {
    let rgb = clear_glyph();
    let actual = frame(8, 4, &rgb);
    let exact = pattern(8, 4, &rgb);
    let mut displaced_rgb = vec![[255; 3]; 32];
    displaced_rgb[8..16].fill([0; 3]);
    let displaced = pattern(8, 4, &displaced_rgb);
    let bank = vec![
        (1, vec![displaced.clone(), exact.clone(), exact.clone()]),
        (2, vec![displaced]),
    ];
    let result = evaluate_rank(&bank, &actual);
    assert_eq!(result.accepted().expect("unique best label").label, 1);
    assert_eq!(result.winner.expect("best witness").variant, 1);
    assert_eq!(result.competitor.expect("different-label witness").label, 2);
    assert_eq!(result.counts.observed, 4);
    assert!(
        result
            .describe(label_name)
            .contains("other-label=[second error=")
    );
    let same_only = evaluate_rank(&[(1, vec![exact.clone(), exact.clone()])], &actual);
    assert_eq!(
        same_only
            .accepted()
            .expect("duplicates of same label are not ambiguous")
            .label,
        1
    );
    assert!(same_only.competitor.is_none());
    let ambiguous = evaluate_rank(&[(1, vec![exact.clone()]), (2, vec![exact])], &actual);
    assert!(ambiguous.accepted().is_none());
    assert_eq!(ambiguous.reason, "refused: competing-label ambiguity");
    assert_eq!(ambiguous.winner.expect("deterministic tie winner").label, 1);
    assert_eq!(
        ambiguous
            .competitor
            .expect("tied other label")
            .score
            .diagnostic_ppm(),
        0
    );
}


/// Unknown regions remain unknown despite having a nearest labelled template.
#[test]
fn blank_checkerboard_low_contrast_and_mixed_colour_regions_refuse() {
    let glyph = pattern(8, 4, &clear_glyph());
    let bank = vec![(1, vec![glyph])];
    let blank = frame(8, 4, &[[255; 3]; 32]);
    let blank_result = evaluate_rank(&bank, &blank);
    assert!(blank_result.accepted().is_none());
    assert_eq!(blank_result.counts.invalid_colour, 1);
    assert_eq!(
        blank_result
            .nearest
            .expect("nearest is explanatory only")
            .score
            .diagnostic_ppm(),
        SCORE_SCALE
    );
    let all_white = pattern(8, 4, &[[255; 3]; 32]);
    let zero = evaluate_rank(&[(1, vec![all_white])], &blank);
    assert_eq!(zero.counts.zero_mass, 1);
    assert!(zero.nearest.is_none());
    assert!(zero.accepted().is_none());
    let checker_rgb: Vec<_> = (0..32)
        .map(|index| if index % 2 == 0 { [0; 3] } else { [255; 3] })
        .collect();
    let checker = evaluate_rank(&bank, &frame(8, 4, &checker_rgb));
    assert!(checker.accepted().is_none());
    assert_eq!(checker.reason, "refused: absolute shape error");
    let low_rgb: Vec<_> = (0..32)
        .map(|index| if index < 8 { [170; 3] } else { [230; 3] })
        .collect();
    let low = evaluate_rank(&bank, &frame(8, 4, &low_rgb));
    assert!(low.accepted().is_none());
    assert_eq!(low.counts.invalid_colour, 1);
    let mut mixed_rgb = clear_glyph();
    mixed_rgb[0] = [220, 0, 0];
    let mixed = evaluate_rank(
        &[(1, vec![pattern(8, 4, &mixed_rgb)])],
        &frame(8, 4, &mixed_rgb),
    );
    assert!(mixed.accepted().is_none());
    assert_eq!(
        mixed
            .nearest
            .expect("exact mixed-colour shape")
            .score
            .diagnostic_ppm(),
        0
    );
    assert_eq!(mixed.counts.invalid_colour, 1);
}


/// Existing paper and exclusive-colour thresholds apply at the exact scoring ROI.
#[test]
fn paper_and_strong_colour_guards_retain_their_inclusive_boundaries() {
    let mut rgb = vec![[130; 3]; 16];
    rgb[..8].fill([0; 3]);
    rgb[11..].fill([180; 3]);
    let template = pattern(4, 4, &rgb);
    let accepted = evaluate_rank(&[(1, vec![template.clone()])], &frame(4, 4, &rgb));
    assert!(
        !accepted
            .accepted()
            .expect("eight black and area/3 paper pixels")
            .red
    );
    assert_eq!(accepted.winner.expect("eligible witness").paper, 5);
    let mut less_paper = rgb.clone();
    less_paper[11] = [179; 3];
    let paper_refusal = evaluate_rank(&[(1, vec![template])], &frame(4, 4, &less_paper));
    assert!(paper_refusal.accepted().is_none());
    assert_eq!(paper_refusal.counts.insufficient_paper, 1);
    assert_eq!(paper_refusal.counts.zero_mass, 0);
    assert!(paper_refusal.nearest.is_none());
    let mut less_ink = clear_glyph();
    less_ink[0] = [101; 3];
    let ink_refusal = evaluate_rank(
        &[(1, vec![pattern(8, 4, &less_ink)])],
        &frame(8, 4, &less_ink),
    );
    assert!(ink_refusal.accepted().is_none());
    assert_eq!(
        ink_refusal.nearest.expect("exact seven-black shape").black,
        7
    );
    assert_eq!(ink_refusal.counts.invalid_colour, 1);
}


/// A zero-error witness cannot borrow colour from another, poorly matched shift.
#[test]
fn score_paper_and_colour_must_belong_to_one_translated_witness() {
    let mut roi = vec![[255; 3]; 40];
    for row in 0..4 {
        for column in (0..10).step_by(2) {
            roi[row * 10 + column] = [110; 3];
        }
    }
    for index in [0, 2, 4, 6, 8, 10, 12] {
        roi[index] = [0; 3];
    }
    let mut surrounding = vec![[255; 3]; 72];
    for row in 0..4 {
        surrounding[(row + 1) * 12 + 1..(row + 1) * 12 + 11]
            .copy_from_slice(&roi[row * 10..row * 10 + 10]);
        surrounding[(row + 1) * 12 + 11] = [0; 3];
    }
    let actual = frame(12, 6, &surrounding);
    let template = pattern(10, 4, &roi);
    let view = actual.as_view().expect("padded synthetic layout");
    let exact = measured_witness(&template, view, (1, 1), (0, 0));
    assert_eq!(exact.score.diagnostic_ppm(), 0);
    assert_eq!(exact.black, 7);
    assert_eq!(exact.exclusive_red(), None);
    let coloured = measured_witness(&template, view, (1, 1), (1, 0));
    assert_eq!(coloured.exclusive_red(), Some(false));
    assert!(
        !coloured
            .score
            .within(RANK_MAX_ERROR_PPM)
            .expect("bounded shifted fraction")
    );
    let result = evaluate(
        &[(1_u8, vec![template])],
        view,
        (1, 1),
        RANK_MAX_ERROR_PPM,
        MIN_LABEL_GAP_PPM,
        1,
    )
    .expect("nine bounded witnesses");
    assert_eq!(result.counts.observed, 9);
    assert!(result.accepted().is_none());
    assert_eq!(
        result
            .nearest
            .expect("unqualified exact shape remains diagnostic")
            .shift,
        (0, 0)
    );
    assert_eq!(result.reason, "refused: absolute shape error");
}


/// Maximum-size decoded glyphs retain every pixel and exact ratio products fit.
#[test]
fn maximum_full_roi_arithmetic_is_bounded_and_excluded_pixels_refuse() {
    let rgb: Vec<_> = (0..MAX_PIXELS)
        .map(|index| if index % 2 == 0 { [0; 3] } else { [255; 3] })
        .collect();
    let inverse: Vec<_> = rgb
        .iter()
        .map(|colour| if *colour == [0; 3] { [255; 3] } else { [0; 3] })
        .collect();
    let template = pattern(64, 64, &rgb);
    let actual = frame(64, 64, &inverse);
    let witness = measured_witness(
        &template,
        actual.as_view().expect("maximum layout"),
        (0, 0),
        (0, 0),
    );
    let maximum_mass = u64::try_from(MAX_PIXELS).expect("bounded area") * 255;
    assert_eq!(witness.score.numerator, maximum_mass);
    assert_eq!(witness.score.denominator, maximum_mass);
    assert_eq!(witness.score.diagnostic_ppm(), SCORE_SCALE);
    assert!(
        Ratio {
            numerator: 0,
            denominator: maximum_mass
        }
        .separated_from(witness.score, MIN_LABEL_GAP_PPM)
        .expect("largest exact cross-products")
    );
    assert_eq!(
        evaluate_rank(&[(1, vec![template.clone()])], &actual).reason,
        "refused: absolute shape error"
    );
    let good = evaluate_rank(&[(1, vec![template.clone()])], &frame(64, 64, &rgb));
    assert_eq!(good.accepted().expect("full maximum ROI").label, 1);
    assert_eq!(
        good.winner
            .expect("maximum-size accepted witness")
            .score
            .diagnostic_ppm(),
        0
    );
    for invalid in [
        Pattern {
            bounds: PixelRect::new(0, 0, 4097, 1),
            pixels: vec![Some([0; 3]); 4097],
            tolerance: 0,
        },
        Pattern {
            bounds: PixelRect::new(0, 0, u32::MAX, 2),
            pixels: Vec::new(),
            tolerance: 0,
        },
        Pattern {
            bounds: PixelRect::new(0, 0, 0, 1),
            pixels: Vec::new(),
            tolerance: 0,
        },
        Pattern {
            pixels: vec![None; MAX_PIXELS],
            ..template.clone()
        },
        Pattern {
            tolerance: 1,
            ..template.clone()
        },
        Pattern {
            pixels: vec![Some([0; 3]); MAX_PIXELS - 1],
            ..template.clone()
        },
    ] {
        assert!(validate_pattern(&invalid).is_err());
    }
    assert_eq!(
        product(u64::MAX, 2).expect_err("checked multiplication"),
        "graded glyph ratio overflow"
    );
    assert!(
        Ratio {
            numerator: u64::MAX,
            denominator: 1
        }
        .within(RANK_MAX_ERROR_PPM)
        .is_err()
    );
}


/// Invalid geometry and unsupported bank parameters return errors before acceptance.
#[test]
fn geometry_and_bank_bounds_fail_closed_without_coordinate_wraparound() {
    let rgb = clear_glyph();
    let actual = frame(8, 4, &rgb);
    let view = actual.as_view().expect("synthetic layout");
    let template = pattern(8, 4, &rgb);
    let bank = vec![(1_u8, vec![template.clone()])];
    for (origin, alignment) in [((u32::MAX, 0), 0), ((0, u32::MAX), 0), ((0, 0), 1)] {
        assert!(
            evaluate(
                &bank,
                view,
                origin,
                RANK_MAX_ERROR_PPM,
                MIN_LABEL_GAP_PPM,
                alignment
            )
            .is_err()
        );
    }
    let overflow = Pattern {
        bounds: PixelRect::new(u32::MAX, 0, 8, 4),
        ..template.clone()
    };
    assert!(observe(1_u8, 0, &overflow, view, (1, 0), (0, 0)).is_err());
    assert!(evaluate::<u8>(&[], view, (0, 0), RANK_MAX_ERROR_PPM, MIN_LABEL_GAP_PPM, 0).is_err());
    assert!(
        evaluate(
            &[(1_u8, Vec::new())],
            view,
            (0, 0),
            RANK_MAX_ERROR_PPM,
            MIN_LABEL_GAP_PPM,
            0
        )
        .is_err()
    );
    assert!(
        evaluate(
            &[(1_u8, vec![template.clone(); 17])],
            view,
            (0, 0),
            RANK_MAX_ERROR_PPM,
            MIN_LABEL_GAP_PPM,
            0
        )
        .is_err()
    );
    let too_many: Vec<_> = (0_u8..14)
        .map(|label| (label, vec![template.clone()]))
        .collect();
    assert!(
        evaluate(
            &too_many,
            view,
            (0, 0),
            RANK_MAX_ERROR_PPM,
            MIN_LABEL_GAP_PPM,
            0
        )
        .is_err()
    );
    for (maximum, margin, alignment) in [
        (1_000_001, MIN_LABEL_GAP_PPM, 0),
        (RANK_MAX_ERROR_PPM, 1_000_001, 0),
        (RANK_MAX_ERROR_PPM, MIN_LABEL_GAP_PPM, -1),
        (RANK_MAX_ERROR_PPM, MIN_LABEL_GAP_PPM, 2),
    ] {
        assert!(evaluate(&bank, view, (0, 0), maximum, margin, alignment).is_err());
    }
}


/// Schema-2 policy keys and integer values cannot silently tune acceptance.
#[test]
fn fixed_policy_rejects_missing_extra_coerced_or_modified_thresholds() {
    let exact = policy_value();
    let policy = Policy::parse(&exact).expect("exact frozen policy");
    assert_eq!(policy.max_error_ppm(true), 220_000);
    assert_eq!(policy.max_error_ppm(false), 120_000);
    assert_eq!(policy.min_gap_ppm(), 80_000);
    for name in [
        "score_scale",
        "rank_max_error_ppm",
        "suit_max_error_ppm",
        "min_label_gap_ppm",
    ] {
        let original = exact[name].as_u64().expect("integer policy entry");
        for replacement in [
            json!(original + 1),
            json!(original - 1),
            json!(original.to_string()),
            json!(original as f64),
            json!(-1),
            Value::Null,
        ] {
            let mut changed = exact.clone();
            changed[name] = replacement;
            assert!(
                Policy::parse(&changed).is_err(),
                "accepted altered {name}: {changed}"
            );
        }
        let mut missing = exact.clone();
        missing.as_object_mut().expect("policy object").remove(name);
        assert!(Policy::parse(&missing).is_err(), "accepted missing {name}");
    }
    for policy_name in ["ink_l1_v1", "ink_l1_v2", "paper_ink_l1_v1"] {
        let mut wrong_name = exact.clone();
        wrong_name["policy"] = json!(policy_name);
        assert!(Policy::parse(&wrong_name).is_err());
    }
    let mut extra = exact.clone();
    extra["operator_override"] = json!(true);
    assert!(Policy::parse(&extra).is_err());
    assert!(Policy::parse(&json!([])).is_err());
}


/// Independent raw paper references preserve exactly scaled glyph contrast.
#[test]
fn uniform_cream_paper_preserves_strokes_and_antialiased_edges() {
    let mut white_rgb = vec![[255; 3]; 32];
    white_rgb[..8].fill([51; 3]);
    white_rgb[8..12].fill([153; 3]);
    let template = pattern(8, 4, &white_rgb);
    let mut cream_rgb = vec![[220, 210, 200]; 32];
    cream_rgb[..8].fill([60, 50, 40]);
    cream_rgb[8..12].fill([140, 130, 120]);
    let cream = frame(8, 4, &cream_rgb);
    let result = evaluate_rank(&[(1, vec![template])], &cream);
    assert_eq!(
        result
            .accepted()
            .expect("same relative stroke contrast")
            .label,
        1
    );
    let witness = result.winner.expect("complete cream-paper witness");
    assert_eq!(witness.actual_paper_reference, 200);
    assert_eq!(witness.template_paper_reference, 255);
    assert_eq!(witness.paper, 20);
    assert_eq!(witness.black, 8);
    assert_eq!(witness.score.numerator, 0);
    assert_eq!(witness.score.denominator, 8 * 204 + 4 * 102);
    assert_eq!(strength(51, 255), Ok(204));
    assert_eq!(strength(40, 200), Ok(204));
    assert_eq!(strength(153, 255), Ok(102));
    assert_eq!(strength(120, 200), Ok(102));
}


/// Even paper counts select the actual lower median instead of an average or maximum.
#[test]
fn raw_paper_lower_median_is_auditable_and_ignores_non_paper_ink() {
    let mut histogram = [0_usize; 256];
    histogram[180] = 1;
    histogram[190] = 2;
    histogram[220] = 1;
    assert_eq!(lower_paper_reference(&histogram, 4), Some(190));
    histogram[10] = 100;
    histogram[179] = 100;
    assert_eq!(lower_paper_reference(&histogram, 4), Some(190));
    assert_eq!(lower_paper_reference(&[0; 256], 0), None);
    let mut odd = [0_usize; 256];
    odd[180] = 1;
    odd[200] = 1;
    odd[255] = 1;
    assert_eq!(lower_paper_reference(&odd, 3), Some(200));
    let mut rgb = clear_glyph();
    rgb[8..20].fill([200, 220, 210]);
    rgb[20..].fill([240, 230, 250]);
    let template = pattern(8, 4, &rgb);
    let actual = frame(8, 4, &rgb);
    let witness = measured_witness(
        &template,
        actual.as_view().expect("mixed-paper layout"),
        (0, 0),
        (0, 0),
    );
    assert_eq!(witness.paper, 24);
    assert_eq!(witness.actual_paper_reference, 200);
    assert_eq!(witness.template_paper_reference, 200);
    assert_eq!(witness.score.numerator, 0);
}


/// Normalisation uses bounded integer flooring and never divides by an invalid reference.
#[test]
fn paper_relative_strength_keeps_exact_flooring_and_reference_bounds() {
    for reference in 180_u8..=255 {
        for minimum in 0_u8..=255 {
            let expected =
                (255_u32 * u32::from(reference.saturating_sub(minimum))) / u32::from(reference);
            assert_eq!(strength(minimum, reference), Ok(expected as u8));
        }
    }
    assert_eq!(strength(100, 200), Ok(127));
    assert_eq!(strength(99, 200), Ok(128));
    assert_eq!(strength(255, 180), Ok(0));
    assert_eq!(strength(0, 180), Ok(255));
    for invalid in [0, 1, 179] {
        assert!(strength(0, invalid).is_err());
        assert!(strength(255, invalid).is_err());
    }
}


/// Missing template paper is a calibration error; missing observed paper is a counted refusal.
#[test]
fn zero_mass_and_insufficient_paper_remain_explicit_refusals() {
    let all_white = pattern(8, 4, &[[255; 3]; 32]);
    assert!(validate_pattern(&all_white).is_ok());
    let blank_frame = frame(8, 4, &[[255; 3]; 32]);
    assert!(matches!(
        observe(
            1_u8,
            0,
            &all_white,
            blank_frame.as_view().expect("white layout"),
            (0, 0),
            (0, 0)
        )
        .expect("zero mass is distinct from missing paper"),
        Observation::ZeroMass
    ));
    let all_dark = pattern(8, 4, &[[0; 3]; 32]);
    assert!(validate_pattern(&all_dark).is_err());
    let mut nine_paper = vec![[0; 3]; 32];
    nine_paper[..9].fill([255; 3]);
    assert!(validate_pattern(&pattern(8, 4, &nine_paper)).is_err());
    let mut ten_paper = nine_paper.clone();
    ten_paper[9] = [180; 3];
    assert!(validate_pattern(&pattern(8, 4, &ten_paper)).is_ok());
    let valid = pattern(8, 4, &clear_glyph());
    let result = evaluate_rank(&[(1, vec![valid.clone()])], &frame(8, 4, &nine_paper));
    assert!(result.accepted().is_none());
    assert_eq!(result.counts.insufficient_paper, 1);
    assert_eq!(result.counts.zero_mass, 0);
    assert!(result.nearest.is_none());
    let black_frame = frame(8, 4, &[[0; 3]; 32]);
    assert!(matches!(
        observe(
            1_u8,
            0,
            &valid,
            black_frame.as_view().expect("black layout"),
            (0, 0),
            (0, 0)
        )
        .expect("observed paper refusal is not a geometry error"),
        Observation::PaperRefused
    ));
}


/// Every negative-space pixel still penalises added strokes after paper normalisation.
#[test]
fn added_dark_strokes_in_paper_remain_part_of_the_full_roi_score() {
    let rgb = clear_glyph();
    let template = pattern(8, 4, &rgb);
    let mut one_added = rgb.clone();
    one_added[8] = [0; 3];
    let mild = evaluate_rank(&[(1, vec![template.clone()])], &frame(8, 4, &one_added));
    let witness = mild.winner.expect("paper and colour remain valid");
    assert_eq!(witness.score.numerator, 255);
    assert_eq!(witness.score.denominator, 9 * 255);
    assert_eq!(
        witness.score.compare(Ratio {
            numerator: 1,
            denominator: 9
        }),
        Ok(Ordering::Equal)
    );
    let mut eight_added = rgb;
    eight_added[8..16].fill([0; 3]);
    let severe = evaluate_rank(&[(1, vec![template])], &frame(8, 4, &eight_added));
    assert!(severe.accepted().is_none());
    assert_eq!(severe.reason, "refused: absolute shape error");
    let severe_score = severe.winner.expect("bad shape cannot hide in paper").score;
    assert_eq!(
        severe_score.compare(Ratio {
            numerator: 1,
            denominator: 2
        }),
        Ok(Ordering::Equal)
    );
}
