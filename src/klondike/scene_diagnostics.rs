//! Read-only scene measurements using the unchanged Klondike classifier predicates.

use serde_json::{Value, json};

use super::*;

/// Exact support and inclusive threshold for one native scene probe.
#[derive(Debug)]
pub(crate) struct Probe {
    /// Role and physical one-based column, where applicable.
    role: String,
    /// Original native x, y, width and height.
    bounds: [u32; 4],
    /// Pixels satisfying the existing role predicate.
    matching_pixels: u32,
    /// All pixels tested in this complete rectangle.
    total_pixels: u32,
    /// Minimum count satisfying the inclusive fractional threshold.
    required_pixels: u32,
    /// Original threshold in parts per thousand.
    threshold_per_mille: u32,
    /// Whether this probe passes its original threshold.
    passes: bool,
}

/// All probes, including those skipped by the authoritative short circuit.
#[derive(Debug)]
pub(crate) struct SceneInspection {
    /// Exact results for four gutters, seven tableau and four foundation strips.
    probes: Vec<Probe>,
    /// Number of passing felt gutters; all four are required.
    gutters_passed: usize,
    /// Number of passing tableau strips; five are required.
    tableau_passed: usize,
    /// Advisory foundation count; these strips do not participate in scene authority.
    foundations_passed: usize,
    /// Diagnostic recomputation; never supplies input authority.
    pub(crate) diagnostic_scene: bool,
}

/// Evaluate complete probes on already-owned pixels without capture or input.
pub(crate) fn inspect(frame: &CapturedFrame) -> Result<SceneInspection, HaloDetectionError> {
    validate_frame(frame)?;
    let mut probes = Vec::with_capacity(15);
    let mut add = |role: String, b: PixelRect, predicate: fn([u8; 3]) -> bool, threshold: u32| {
        let count = count_pixels(frame, b, predicate);
        let total = b.width * b.height;
        let required = (total * threshold).div_ceil(1_000);
        probes.push(Probe {
            role,
            bounds: [b.x, b.y, b.width, b.height],
            matching_pixels: count,
            total_pixels: total,
            required_pixels: required,
            threshold_per_mille: threshold,
            passes: count >= required,
        });
        count >= required
    };
    let mut gutters_passed = 0;
    for (index, bounds) in [
        PixelRect::new(160, 310, 16, 16),
        PixelRect::new(1_730, 310, 16, 16),
        PixelRect::new(874, 710, 7, 28),
        PixelRect::new(1_208, 840, 7, 28),
    ]
    .into_iter()
    .enumerate()
    {
        gutters_passed += usize::from(add(
            format!("felt gutter {}", index + 1),
            bounds,
            is_felt,
            950,
        ));
    }
    let mut tableau_passed = 0;
    for column in 0..7 {
        tableau_passed += usize::from(add(
            format!("PLAY {} top", column + 1),
            PixelRect::new(
                FIRST_COLUMN_X + COLUMN_PITCH * column + 20,
                TABLEAU_Y,
                92,
                6,
            ),
            is_slot_edge,
            700,
        ));
    }
    let mut foundations_passed = 0;
    for column in 0..4 {
        foundations_passed += usize::from(add(
            format!("SUIT {} top", column + 1),
            PixelRect::new(894 + COLUMN_PITCH * column + 20, UPPER_Y, 92, 6),
            is_slot_edge,
            700,
        ));
    }
    Ok(SceneInspection {
        probes,
        gutters_passed,
        tableau_passed,
        foundations_passed,
        diagnostic_scene: gutters_passed == 4 && tableau_passed >= 5,
    })
}

/// Encode fixed probe measurements without adding a serialization dependency.
impl SceneInspection {
    /// Structured independent measurements; classifier authority remains separate.
    pub(crate) fn value(&self) -> Value {
        json!({"probes": self.probes.iter().map(|p| json!({"role": p.role, "bounds": p.bounds,
            "matching_pixels": p.matching_pixels, "total_pixels": p.total_pixels,
            "required_pixels": p.required_pixels, "threshold_per_mille": p.threshold_per_mille, "passes": p.passes})).collect::<Vec<_>>(),
            "gutters_passed": self.gutters_passed, "tableau_passed": self.tableau_passed,
            "foundations_passed": self.foundations_passed, "foundations_required": false, "diagnostic_scene": self.diagnostic_scene})
    }
}

#[cfg(test)]
mod tests;
