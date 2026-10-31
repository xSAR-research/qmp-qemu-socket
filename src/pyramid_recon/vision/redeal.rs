//! Checked native stock positions and retained legacy REDEAL fixture readers.
//!
//! Production availability uses the execution's temporal region comparison. The
//! retained signatures describe stock layouts and test the older control grammar.
//! Only the bundled native constructor installs this capability; a custom profile
//! identifier cannot opt in. The caller still owns freshness, replay and input.

use super::*;


/// Measured complete control, positive LEFT-X and literal remaining-deal captions.
#[derive(Clone, Debug)]
pub(super) struct NativeRedeal {
    /// Three measured whole enabled icons with the same displayed one-deal digit.
    #[cfg(test)]
    enabled: [Pattern; 3],
    /// Positive exhausted LEFT face; an unrecognised glyph is never empty.
    left_x: Pattern,
    /// Four complete measured singular captions, matched separately at tolerance four.
    #[cfg(test)]
    one: [Pattern; 4],
    /// Both halves of the full compact MOVE control used between stock passes.
    recyclable_move: [Pattern; 2],
    /// Complete actual plural zero-deal caption, which grants no input.
    #[cfg(test)]
    zero: Pattern,
}


impl NativeRedeal {
    /// Decode bounded compile-time native RGB crops with unchanged tolerance four.
    pub(super) fn bundled() -> Result<Self, String> {
        Ok(Self {
            #[cfg(test)]
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
            #[cfg(test)]
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
            #[cfg(test)]
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
    #[cfg(test)]
    fn has_one(&self, view: FrameView<'_>) -> Result<bool, String> {
        matches_any(&self.one, view)
    }
}


/// Match one complete measured variant, never a mixture of selected pixels.
#[cfg(test)]
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
    #[cfg(test)]
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
        let one = if self.colour_independent {
            self.native_shapes(view, &native.one)?
        } else {
            native.has_one(view)?
        };
        let zero = self.native_shape(view, &native.zero, 1)?;
        Ok(match (one, zero) {
            (true, false) => Some(1),
            (false, true) => Some(0),
            _ => None,
        })
    }


    /// Retain the historical empty-circle/final-X fixture reader; runtime availability is temporal.
    ///
    /// `Some` is a complete enabled-control plus one-deal proof. `None` proves only
    /// the expected position, never a REDEAL click. The ordinary path is unchanged.
    #[cfg(test)]
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
                !self.dialog_matches(view, true)?,
                "Stock continuation refused Undo All modal",
            )?;
            for (index, anchor) in self.board.scene.iter().enumerate() {
                if index != EXECUTION_EMPTY_STOCK_SCENE_INDEX {
                    require(
                        self.native_scene_anchor(view, index, anchor)?,
                        "Stock continuation static scene mismatch",
                    )?;
                }
            }
            // Removing every stock card can leave the empty circle without a DRAW
            // arrow or final X. The complete enabled widget supplies that layout's
            // positive control evidence; both piles must independently prove empty.
            let enabled = self.native_shapes(view, &native.enabled)?
                && !self.native_shape(view, &native.zero, 1)?;
            if right.is_none()
                && enabled
                && self
                    .verify_cards(frame, view, expected, None, None, &self.board.left_empty)
                    .is_ok()
            {
                return Ok(Some(PixelPoint::new(960, 825)));
            }
            let [upper, lower] = &native.recyclable_move;
            if self.native_shape(view, upper, 1)? && self.native_shape(view, lower, 1)? {
                self.verify_cards(frame, view, expected, None, right, &self.board.left_empty)?;
                // This intermediate stock position never grants a REDEAL click.
                return Ok(None);
            }
            require(
                self.native_stock_layout(view, LEFT_BOUNDS, &native.left_x)?,
                "Stock continuation lacks its exhausted layout",
            )?;
            self.verify_cards(frame, view, expected, None, right, &native.left_x)?;
            // Complete enabled-widget templates include their own literal count one.
            // Unknown lower-caption rasterisation cannot revoke that positive proof;
            // a positively measured zero remains a contradictory-evidence veto.
            Ok(enabled.then_some(PixelPoint::new(960, 825)))
        })();
        alternative.map_err(|_| {
            ordinary
                .err()
                .unwrap_or_else(|| "Stock continuation refused".to_owned())
        })
    }


    /// Prove only replayed stock/tableau positions; REDEAL appearance is temporal.
    /// No enabled-widget, numeral or lower-caption template participates here.
    /// The caller must derive `redeal_appeared` from this exact frame and its own
    /// pre-input region. This method never returns coordinates or input authority.
    /// Appearance can establish the exhausted layout with an occupied RIGHT;
    /// every replayed card, empty LEFT and exact RIGHT identity still must agree.
    pub(crate) fn verify_temporal_stock_position(
        &self,
        frame: &CapturedFrame,
        expected: &[Option<Card>; 28],
        left: Option<Card>,
        right: Option<Card>,
        redeal_appeared: bool,
    ) -> Result<(), String> {
        if self.verify_position(frame, expected, left, right).is_ok() {
            return Ok(());
        }
        require(
            left.is_none(),
            "Temporal stock position requires replayed LEFT exhaustion",
        )?;
        let native = self
            .native_redeal
            .as_ref()
            .ok_or("Native stock capability unavailable")?;
        require(
            frame.width == 1920 && frame.height == 1080,
            "Temporal stock position requires native dimensions",
        )?;
        let view = frame
            .as_view()
            .map_err(|_| "Invalid temporal stock layout")?;
        require(
            !self.dialog_matches(view, true)?,
            "Temporal stock position refused Undo All modal",
        )?;
        for (index, anchor) in self.board.scene.iter().enumerate() {
            if index != EXECUTION_EMPTY_STOCK_SCENE_INDEX {
                require(
                    self.native_scene_anchor(view, index, anchor)?,
                    "Temporal stock static scene mismatch",
                )?;
            }
        }
        let [upper, lower] = &native.recyclable_move;
        let compact_move =
            self.native_shape(view, upper, 1)? && self.native_shape(view, lower, 1)?;
        let exhausted = self.native_stock_layout(view, LEFT_BOUNDS, &native.left_x)?;
        require(
            compact_move || exhausted || redeal_appeared,
            "Temporal stock position lacks its exhausted or intermediate layout",
        )?;
        let empty = if exhausted {
            &native.left_x
        } else {
            &self.board.left_empty
        };
        self.verify_cards(frame, view, expected, None, right, empty)
    }


    /// Read the existing positive LEFT-X layout without rechecking ordinary tableau occupancy.
    ///
    /// Final-position callers separately prove native geometry, scene/modal, all
    /// absence bands and exact RIGHT. Missing capability or an unknown layout
    /// grants nothing; malformed storage/calibration errors propagate.
    pub(super) fn native_exhausted_left(&self, view: FrameView<'_>) -> Result<bool, String> {
        let Some(native) = self.native_redeal.as_ref() else {
            return Ok(false);
        };
        self.native_stock_layout(view, LEFT_BOUNDS, &native.left_x)
    }


    /// Reuse the supported exhausted/recyclable stock artwork for source scene validation.
    pub(super) fn source_exhausted_stock_scene(&self, view: FrameView<'_>) -> Result<bool, String> {
        if self.native_exhausted_left(view)? {
            return Ok(true);
        }
        let Some(native) = &self.native_redeal else {
            return Ok(false);
        };
        let [upper, lower] = &native.recyclable_move;
        Ok(self.native_shape(view, upper, 1)? && self.native_shape(view, lower, 1)?)
    }


    /// Read the D source using the existing positive LEFT-X or ordinary empty layout.
    pub(super) fn source_draw_face(&self, view: FrameView<'_>) -> Result<StockFace, String> {
        let empty = if self.native_exhausted_left(view)? {
            &self
                .native_redeal
                .as_ref()
                .ok_or("missing exhausted LEFT capability")?
                .left_x
        } else {
            &self.board.left_empty
        };
        self.stock_face(view, LEFT_BOUNDS, empty)
    }


    /// Return the measured click centre only after exact same-frame exhaustion/control proof.
    #[cfg(test)]
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
            self.native_shape(view, &native.left_x, 1)?,
            self.native_shapes(view, &native.enabled)?,
            self.native_shapes(view, &native.one)?,
            self.native_shape(view, &native.zero, 1)?,
        ))
    }
}


#[cfg(test)]
mod tests;
