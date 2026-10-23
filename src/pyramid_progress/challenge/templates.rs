//! Whole native Boards/Score labels and explicitly transcribed decimal appearances.
//!
//! Every source and crop is recorded in counter-provenance.json. No synthetic
//! glyph or unobserved rank label is bundled. Decimal templates retain native
//! blue-channel antialiasing on fixed canvases without resizing.

use super::{ChallengeKind, Label};
use crate::{geometry::PixelRect, pyramid_progress::HeaderPatch};


/// Complete isolated 20×26 samples from 74 actual characters in thirteen sources.
pub(super) const GLYPHS: &[u8; 38_480] =
    include_bytes!("../../../tests/fixtures/pyramid-objectives/counter-glyphs.ink");


/// Independent manually transcribed labels, one per actual glyph sample.
pub(super) const CHARACTERS: &[u8; 74] =
    include_bytes!("../../../tests/fixtures/pyramid-objectives/counter-labels.txt");


/// Whole Boards/Score words and their native negative space; counter pixels are excluded.
pub(super) const LABELS: [Label; 9] = [
    Label {
        kind: ChallengeKind::Boards,
        patch: HeaderPatch {
            bounds: PixelRect::new(814, 48, 224, 32),
            rgb: include_bytes!("../../../tests/fixtures/pyramid-objectives/label-boards-two.rgb"),
        },
    },
    Label {
        kind: ChallengeKind::Boards,
        patch: HeaderPatch {
            bounds: PixelRect::new(814, 48, 224, 32),
            rgb: include_bytes!(
                "../../../tests/fixtures/pyramid-objectives/label-boards-three.rgb"
            ),
        },
    },
    Label {
        kind: ChallengeKind::Boards,
        patch: HeaderPatch {
            bounds: PixelRect::new(814, 48, 224, 32),
            rgb: include_bytes!("../../../tests/fixtures/pyramid-objectives/label-boards-one.rgb"),
        },
    },
    Label {
        kind: ChallengeKind::Boards,
        patch: HeaderPatch {
            bounds: PixelRect::new(814, 48, 224, 32),
            rgb: include_bytes!(
                "../../../tests/fixtures/pyramid-objectives/label-boards-second.rgb"
            ),
        },
    },
    Label {
        kind: ChallengeKind::Boards,
        patch: HeaderPatch {
            bounds: PixelRect::new(814, 48, 224, 32),
            rgb: include_bytes!(
                "../../../tests/fixtures/pyramid-objectives/label-boards-complete.rgb"
            ),
        },
    },
    Label {
        kind: ChallengeKind::Score,
        patch: HeaderPatch {
            bounds: PixelRect::new(814, 48, 132, 32),
            rgb: include_bytes!("../../../tests/fixtures/pyramid-objectives/label-score-ten.rgb"),
        },
    },
    Label {
        kind: ChallengeKind::Score,
        patch: HeaderPatch {
            bounds: PixelRect::new(814, 48, 132, 32),
            rgb: include_bytes!("../../../tests/fixtures/pyramid-objectives/label-score-fifth.rgb"),
        },
    },
    Label {
        kind: ChallengeKind::Score,
        patch: HeaderPatch {
            bounds: PixelRect::new(814, 48, 132, 32),
            rgb: include_bytes!(
                "../../../tests/fixtures/pyramid-objectives/label-score-final-right.rgb"
            ),
        },
    },
    Label {
        kind: ChallengeKind::Score,
        patch: HeaderPatch {
            bounds: PixelRect::new(814, 48, 132, 32),
            rgb: include_bytes!(
                "../../../tests/fixtures/pyramid-objectives/label-score-final-empty.rgb"
            ),
        },
    },
];
