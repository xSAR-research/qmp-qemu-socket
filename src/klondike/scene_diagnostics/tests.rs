//! Probe parity and exact-threshold evidence on retained native pixels.

use super::*;
use crate::capture::decode_png;

/// Paint only a supplied checked scene probe with synthetic black pixels.
fn black(frame: &mut CapturedFrame, bounds: [u32; 4]) {
    let [x, y, width, height] = bounds;
    for row in y..y + height {
        for column in x..x + width {
            let offset = row as usize * frame.stride + column as usize * 4;
            frame.pixels[offset..offset + 3].fill(0);
        }
    }
}

/// Every diagnostic count retains the existing classifier's inclusive boundary.
#[test]
fn telemetry_301_scene_report_exposes_exact_failed_probe_without_changing_authority() {
    let mut frame = decode_png(include_bytes!("../../../tests/fixtures/klondike/K48.png")).unwrap();
    let original = inspect(&frame).unwrap();
    assert_eq!(
        original.diagnostic_scene,
        is_gameplay_scene(&frame).unwrap()
    );
    assert!(original.diagnostic_scene);
    assert_eq!(original.probes.len(), 15);
    assert_eq!(
        original
            .probes
            .iter()
            .map(|p| p.required_pixels)
            .collect::<Vec<_>>(),
        [
            244, 244, 187, 187, 387, 387, 387, 387, 387, 387, 387, 387, 387, 387, 387
        ]
    );
    black(&mut frame, [874, 710, 7, 28]);
    let report = inspect(&frame).unwrap();
    assert!(!report.diagnostic_scene);
    assert!(!is_gameplay_scene(&frame).unwrap());
    let failed = report
        .probes
        .iter()
        .find(|p| p.bounds == [874, 710, 7, 28])
        .unwrap();
    assert_eq!(
        (
            failed.matching_pixels,
            failed.total_pixels,
            failed.required_pixels
        ),
        (0, 196, 187)
    );
    assert!(!failed.passes);
    assert_eq!(
        report.probes.len(),
        15,
        "later probes remain inspectable after a failed gutter"
    );
    assert_eq!(report.tableau_passed, original.tableau_passed);
    assert_eq!(report.foundations_passed, original.foundations_passed);
    assert!(
        inspect(&CapturedFrame {
            pixels: Vec::new(),
            ..frame
        })
        .is_err()
    );
}

/// Felt and PLAY remain authoritative while foundation decoration is advisory.
#[test]
fn scene_302_report_keeps_play_allowance_and_ignores_foundation_decoration() {
    let original = decode_png(include_bytes!("../../../tests/fixtures/klondike/K48.png")).unwrap();
    for foundations in [false, true] {
        let mut frame = original.clone();
        let report = inspect(&frame).unwrap();
        let selected: Vec<_> = report
            .probes
            .iter()
            .filter(|p| {
                p.role
                    .starts_with(if foundations { "SUIT" } else { "PLAY" })
            })
            .collect();
        for probe in selected {
            black(&mut frame, probe.bounds);
            let inspected = inspect(&frame).unwrap();
            assert_eq!(
                inspected.diagnostic_scene,
                is_gameplay_scene(&frame).unwrap()
            );
            let expected = inspected.gutters_passed == 4 && inspected.tableau_passed >= 5;
            assert_eq!(inspected.diagnostic_scene, expected);
        }
        assert_eq!(inspect(&frame).unwrap().diagnostic_scene, foundations);
    }
}


/// Both original refused frames now reach the exact SUIT source and canonical click.
#[test]
fn scene_302_saved_halo_and_destination_reach_suit_three_without_relaxing_source() {
    use crate::{game::InputOperation, geometry::PixelPoint, tracker::PredictedAction};
    for png in [
        include_bytes!("../../../tests/fixtures/klondike-scene-302/first-rejected.png").as_slice(),
        include_bytes!("../../../tests/fixtures/klondike-scene-302/last-observed.png").as_slice(),
    ] {
        let frame = decode_png(png).unwrap();
        let report = inspect(&frame).unwrap();
        assert_eq!(
            (
                report.gutters_passed,
                report.tableau_passed,
                report.foundations_passed
            ),
            (4, 7, 2)
        );
        assert!(report.diagnostic_scene && is_gameplay_scene(&frame).unwrap());
        let PredictedAction::Action(action) = analyse(&frame).unwrap().prediction else {
            panic!("saved supported board must reach a canonical source");
        };
        assert_eq!(
            action.target,
            crate::game::ActionTarget::Klondike(KlondikeTarget::Foundation { column: 3 })
        );
        assert_eq!(
            action.operation(),
            InputOperation::Click(PixelPoint::new(1296, 199))
        );
        let mut no_banner = frame.clone();
        black(&mut no_banner, [828, 36, 263, 1]);
        assert_eq!(
            analyse(&no_banner).unwrap().prediction,
            PredictedAction::NoHighlight
        );
        let mut broken_source = frame.clone();
        black(
            &mut broken_source,
            [894 + COLUMN_PITCH * 2, UPPER_Y, CARD_WIDTH, CARD_HEIGHT],
        );
        assert_eq!(
            analyse(&broken_source).unwrap().prediction,
            PredictedAction::NoHighlight
        );
        let mut no_felt = frame.clone();
        black(&mut no_felt, [160, 310, 16, 16]);
        assert!(!is_gameplay_scene(&no_felt).unwrap());
    }
}
