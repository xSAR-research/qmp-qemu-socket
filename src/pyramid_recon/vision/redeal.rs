//! Continuation-only stock exhaustion and enabled REDEAL proof from native originals.
//!
//! These full-mask signatures do not alter ordinary Recon or position recognition.
//! Only the bundled native constructor installs this capability; a custom profile
//! identifier cannot opt in. The caller still owns freshness, replay and input.

use super::*;


/// Measured complete control, positive LEFT-X and literal remaining-deal captions.
#[derive(Clone, Debug)]
pub(super) struct NativeRedeal {
    /// Three measured whole enabled icons with the same displayed one-deal digit.
    enabled: [Pattern; 3],
    /// Positive exhausted LEFT face; an unrecognised glyph is never empty.
    left_x: Pattern,
    /// Four complete measured singular captions, matched separately at tolerance four.
    one: [Pattern; 4],
    /// Both halves of the full compact MOVE control used between stock passes.
    recyclable_move: [Pattern; 2],
    /// Complete actual plural zero-deal caption, which grants no input.
    zero: Pattern,
}


impl NativeRedeal {
    /// Decode bounded compile-time native RGB crops with unchanged tolerance four.
    pub(super) fn bundled() -> Result<Self, String> {
        Ok(Self {
            enabled: [
                pattern(
                    PixelRect::new(936, 792, 48, 63),
                    include_bytes!(
                        "../../../tests/fixtures/pyramid-recon/native-redeal/enabled.rgb"
                    ),
                )?,
                pattern(
                    PixelRect::new(936, 792, 48, 63),
                    include_bytes!(
                        "../../../tests/fixtures/pyramid-challenge-redeal/jacks-enabled.rgb"
                    ),
                )?,
                pattern(
                    PixelRect::new(936, 792, 48, 63),
                    include_bytes!(
                        "../../../tests/fixtures/pyramid-challenge-redeal/eights-enabled.rgb"
                    ),
                )?,
            ],
            left_x: pattern(
                PixelRect::new(767, 712, 60, 60),
                include_bytes!("../../../tests/fixtures/pyramid-recon/native-redeal/left-x.rgb"),
            )?,
            one: [
                pattern(
                    PixelRect::new(890, 874, 140, 26),
                    include_bytes!(
                        "../../../tests/fixtures/pyramid-recon/native-redeal/deals-one.rgb"
                    ),
                )?,
                pattern(
                    PixelRect::new(890, 874, 140, 26),
                    include_bytes!("../../../tests/fixtures/pyramid-stock-recycle/deals-one.rgb"),
                )?,
                pattern(
                    PixelRect::new(890, 874, 140, 26),
                    include_bytes!(
                        "../../../tests/fixtures/pyramid-challenge-redeal/jacks-one.rgb"
                    ),
                )?,
                pattern(
                    PixelRect::new(890, 874, 140, 26),
                    include_bytes!(
                        "../../../tests/fixtures/pyramid-challenge-redeal/eights-one.rgb"
                    ),
                )?,
            ],
            zero: pattern(
                PixelRect::new(890, 874, 140, 26),
                include_bytes!(
                    "../../../tests/fixtures/pyramid-recon/native-redeal/deals-zero.rgb"
                ),
            )?,
            recyclable_move: [
                pattern(
                    PixelRect::new(920, 678, 80, 26),
                    include_bytes!("../../../tests/fixtures/pyramid-stock-recycle/move-upper.rgb"),
                )?,
                pattern(
                    PixelRect::new(920, 704, 80, 26),
                    include_bytes!("../../../tests/fixtures/pyramid-stock-recycle/move-lower.rgb"),
                )?,
            ],
        })
    }


    /// Match a complete measured one-deal caption; unknown text grants nothing.
    fn has_one(&self, view: FrameView<'_>) -> Result<bool, String> {
        matches_any(&self.one, view)
    }
}


/// Match one complete measured variant, never a mixture of selected pixels.
fn matches_any(patterns: &[Pattern], view: FrameView<'_>) -> Result<bool, String> {
    for candidate in patterns {
        if candidate.matches(view, (0, 0))? {
            return Ok(true);
        }
    }
    Ok(false)
}


/// Build one bounded full-mask pattern; malformed lengths or allocation failures refuse.
pub(super) fn pattern(bounds: PixelRect, raw: &[u8]) -> Result<Pattern, String> {
    let count = usize::try_from(
        bounds
            .width
            .checked_mul(bounds.height)
            .ok_or("Native REDEAL crop area overflow")?,
    )
    .map_err(|_| "Native crop size overflow")?;
    require(
        (MIN_SELECTED_PIXELS..=MAX_PATCH_PIXELS).contains(&count),
        "Invalid native REDEAL crop area",
    )?;
    require(
        raw.len() == count.checked_mul(3).ok_or("Native crop byte overflow")?,
        "Invalid native REDEAL crop bytes",
    )?;
    let minimum = raw.iter().copied().min().ok_or("Empty native crop")?;
    let maximum = raw.iter().copied().max().ok_or("Empty native crop")?;
    require(
        maximum.saturating_sub(minimum) >= 64,
        "Native crop lacks contrast",
    )?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(count)
        .map_err(|_| "Native crop allocation failed")?;
    for rgb in raw.as_chunks::<3>().0 {
        pixels.push(Some(*rgb));
    }
    Ok(Pattern {
        bounds,
        pixels,
        tolerance: 4,
    })
}


impl ReconVision {
    /// Report only measured literal zero/one allowance; unknown text grants nothing.
    pub(crate) fn remaining_native_redeals(
        &self,
        frame: &CapturedFrame,
    ) -> Result<Option<u8>, String> {
        let Some(native) = self.native_redeal.as_ref() else {
            return Ok(None);
        };
        let view = frame
            .as_view()
            .map_err(|_| "Invalid REDEAL caption layout")?;
        if frame.width != 1920 || frame.height != 1080 {
            return Ok(None);
        }
        let one = native.has_one(view)?;
        let zero = native.zero.matches(view, (0, 0))?;
        Ok(match (one, zero) {
            (true, false) => Some(1),
            (false, true) => Some(0),
            _ => None,
        })
    }


    /// Verify exact stock positions with measured empty-circle or final-X alternatives.
    ///
    /// `Some` is a complete enabled-control plus one-deal proof. `None` proves only
    /// the expected position, never a REDEAL click. The ordinary path is unchanged.
    pub(crate) fn verify_native_stock_continuation(
        &self,
        frame: &CapturedFrame,
        expected: &[Option<Card>; 28],
        left: Option<Card>,
        right: Option<Card>,
    ) -> Result<Option<PixelPoint>, String> {
        let ordinary = self.verify_position(frame, expected, left, right);
        if ordinary.is_ok() {
            return Ok(None);
        }
        let alternative: Result<Option<PixelPoint>, String> = (|| {
            require(
                left.is_none(),
                "Stock continuation alternative requires replayed LEFT exhaustion",
            )?;
            let native = self
                .native_redeal
                .as_ref()
                .ok_or("Native stock continuation capability unavailable")?;
            require(
                frame.width == 1920 && frame.height == 1080,
                "Stock continuation requires native dimensions",
            )?;
            let view = frame
                .as_view()
                .map_err(|_| "Invalid stock continuation layout")?;
            require(
                !self.dialog.matches(view, true)?,
                "Stock continuation refused Undo All modal",
            )?;
            for (index, anchor) in self.board.scene.iter().enumerate() {
                if index != EXECUTION_EMPTY_STOCK_SCENE_INDEX {
                    require(
                        anchor.matches(view, (0, 0))?,
                        "Stock continuation static scene mismatch",
                    )?;
                }
            }
            let [upper, lower] = &native.recyclable_move;
            if upper.matches(view, (0, 0))? && lower.matches(view, (0, 0))? {
                self.verify_cards(frame, view, expected, None, right, &self.board.left_empty)?;
                // This intermediate stock position never grants a REDEAL click.
                return Ok(None);
            }
            self.verify_cards(frame, view, expected, None, right, &native.left_x)?;
            let enabled = matches_any(&native.enabled, view)?
                && native.has_one(view)?
                && !native.zero.matches(view, (0, 0))?;
            Ok(enabled.then_some(PixelPoint::new(960, 825)))
        })();
        alternative.map_err(|_| {
            ordinary
                .err()
                .unwrap_or_else(|| "Stock continuation refused".to_owned())
        })
    }


    /// Return the measured click centre only after exact same-frame exhaustion/control proof.
    pub(crate) fn verify_native_redeal_position(
        &self,
        frame: &CapturedFrame,
        expected: &[Option<Card>; 28],
        right: Option<Card>,
    ) -> Result<PixelPoint, String> {
        if let Some(point) = self.verify_native_stock_continuation(frame, expected, None, right)? {
            return Ok(point);
        }
        let native = self
            .native_redeal
            .as_ref()
            .ok_or("Native REDEAL capability unavailable")?;
        let view = frame.as_view().map_err(|_| "Invalid REDEAL layout")?;
        Err(format!(
            "No positively enabled supported REDEAL with remaining allowance; final-frame control checks: LEFT-X={}, enabled icon={}, one-deal caption={}, zero-deal caption={}",
            native.left_x.matches(view, (0, 0))?,
            matches_any(&native.enabled, view)?,
            native.has_one(view)?,
            native.zero.matches(view, (0, 0))?,
        ))
    }
}


#[cfg(test)]
mod tests;
