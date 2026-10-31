//! Route-owned quarter-card occurrence evidence and independent corner quietness.
//!
//! A known occupied source supplies each original slot's baseline. Expected
//! removals require joint same-pixel RGB changes and two successive
//! supported, settled captures. Only the execution adapter commits a candidate
//! after position and frame guards pass; bound progress still gates advancement. No background
//! reference, pixel exclusion, clock or guest input belongs to this module.

use xsar::image_matching::packed_comparison::{PackedByteComparison, compare_packed_bytes};

use super::*;


/// Independent channel allowance retained solely for the existing settling policy.
const BYTE_TOLERANCE: u8 = 12;


/// Two fresh supported appearances provide one quiet observation interval.
const SETTLED_CAPTURES: usize = 2;


/// Quiet consecutive appearances permit at most one percent large changes per channel.
const MAX_SETTLE_CHANGED_PERCENT: u64 = 1;


/// Quiet consecutive appearances permit at most two mean absolute levels per channel.
const MAX_SETTLE_MEAN_DIFFERENCE: u64 = 2;


/// Non-serialised evidence usable only by a route's owned execution vision.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RemovalProof {
    /// Original fixed tableau slots already confirmed or provisionally being checked.
    slots: u32,
}


impl RemovalProof {
    /// Report whether this original zero-based slot has temporal removal evidence.
    pub(super) fn contains(self, slot: usize) -> bool {
        slot < 28 && self.slots & (1_u32 << slot) != 0
    }


    /// Require all original 28 slots before substituting a terminal absence proof.
    pub(super) fn complete(self) -> bool {
        self.slots == (1_u32 << 28) - 1
    }
}


/// Provisional per-frame evidence; the worker must check its full expected position.
pub(crate) struct RemovalCandidate {
    /// Fixed-slot history plus this action's substantial measured changes.
    pub(crate) proof: RemovalProof,
    /// Whether two fresh supported appearances have remained quiet.
    pub(crate) settled: bool,
}


/// One original occupied card and its immutable packed RGB baseline.
struct Target {
    /// Original fixed slot; removals never compact or renumber it.
    slot: usize,
    /// Original occupied identity; its continued recognition vetoes removal.
    card: Card,
    /// Complete row-major effect quarter; alpha is fixed to zero.
    before: Vec<u32>,
    /// Last supported 31-by-49 quietness corner, cleared on any verification mismatch.
    previous: Option<Vec<u32>>,
}


/// Expected action transition retained only until its effect is independently checked.
struct Pending {
    /// One complete pair frozen before the first gesture of this transition.
    settings: crate::parameters::PyramidEffectSettings,
    /// Independently replayed post-action original card identities and occupancy.
    expected: [Option<Card>; 28],
    /// Only the one or two tableau cards removed by this action.
    targets: Vec<Target>,
    /// Number of consecutive changed and quiet supported appearances.
    stable: usize,
    /// Last actual capture serial, preventing one frame from counting repeatedly.
    capture: Option<usize>,
}


/// Bounded deletion-only evidence owned by one prepared route and acquisition.
#[derive(Default)]
pub(crate) struct RemovalHistory {
    /// Confirmed fixed-slot removals, never inferred from an unknown/background pixel.
    confirmed: RemovalProof,
    /// A single authorised action; uncertain delivery never creates a second action.
    pending: Option<Pending>,
}


/// Extract precisely the reader's existing checked 31-by-49 corner into packed RGB.
fn corner(frame: &CapturedFrame, slot: usize) -> Result<Vec<u32>, String> {
    require(
        frame.width == 1920 && frame.height == 1080,
        "removal requires native1920x1080",
    )?;
    let origin = *TABLEAU_ORIGINS.get(slot).ok_or("invalid removal slot")?;
    let view = frame
        .as_view()
        .map_err(|_| "invalid removal frame layout")?;
    Ok(foreground_card::corner_pixels(view, origin)?
        .into_iter()
        .map(|[r, g, b]| u32::from_le_bytes([r, g, b, 0]))
        .collect())
}


/// Sample the fixed effect quarter for one original slot, without changing quietness.
fn effect_pixels(frame: &CapturedFrame, slot: usize) -> Result<Vec<u32>, String> {
    let origin = *TABLEAU_ORIGINS.get(slot).ok_or("invalid removal slot")?;
    crate::pyramid_effect::card_pixels(frame, origin)
}


/// Require the approved joint predicate on the same coordinates, without a mean gate.
#[cfg(test)]
fn substantial(before: &[u32], after: &[u32]) -> Result<bool, String> {
    Ok(crate::pyramid_effect::compare(before, after)?.exceeds_allowance())
}


/// Require all RGB channels to remain quiet; alpha cannot contribute to settling.
fn quiet(measured: PackedByteComparison) -> bool {
    measured.words != 0
        && (0..3).all(|channel| {
            measured.changed[channel] * 100 <= measured.words * MAX_SETTLE_CHANGED_PERCENT
                && measured.absolute_difference[channel]
                    <= measured.words * MAX_SETTLE_MEAN_DIFFERENCE
        })
}


impl RemovalHistory {
    /// Commit only after two fresh source-identity disappearance observations.
    pub(crate) fn confirm_source_transition(&mut self, expected: &[Option<Card>; 28]) {
        self.confirmed.slots = expected.iter().enumerate().fold(0, |mask, (slot, card)| {
            mask | if card.is_none() { 1_u32 << slot } else { 0 }
        });
        self.pending = None;
    }


    /// Supply explicitly verified earlier occupancy to an isolated adapter fixture.
    /// This constructor is excluded from production and carries no input authority.
    #[cfg(test)]
    pub(crate) fn from_verified_position(expected: &[Option<Card>; 28]) -> Self {
        Self {
            confirmed: RemovalProof {
                slots: expected.iter().enumerate().fold(0, |mask, (slot, card)| {
                    mask | if card.is_none() { 1_u32 << slot } else { 0 }
                }),
            },
            pending: None,
        }
    }


    /// Return only committed history for a new source or after an unmatched frame.
    pub(crate) fn proof(&self) -> RemovalProof {
        self.confirmed
    }


    /// Capture known occupied corners at the consumed source-proof boundary.
    ///
    /// The worker authenticates the source frame, route, context and exact replayed
    /// action before calling. No new identity, resurrection or more than two tableau
    /// removals is accepted. Stock-only operations need no tableau baseline.
    #[cfg(test)]
    pub(crate) fn begin(
        &mut self,
        frame: &CapturedFrame,
        before: &[Option<Card>; 28],
        after: [Option<Card>; 28],
    ) -> Result<(), String> {
        self.begin_with_settings(
            frame,
            before,
            after,
            crate::parameters::PyramidEffectSettings::default(),
        )
    }


    /// Bind the complete effect policy to this action, independent of later UI edits.
    #[cfg(test)]
    pub(crate) fn begin_with_settings(
        &mut self,
        frame: &CapturedFrame,
        before: &[Option<Card>; 28],
        after: [Option<Card>; 28],
        settings: crate::parameters::PyramidEffectSettings,
    ) -> Result<(), String> {
        require(
            self.pending.is_none(),
            "previous removal transition is unfinished",
        )?;
        let mut targets = Vec::new();
        for (slot, (&before, &after)) in before.iter().zip(&after).enumerate() {
            require(
                before.is_none() == self.confirmed.contains(slot),
                "source removal history mismatch",
            )?;
            match (before, after) {
                (Some(card), None) => targets.push(Target {
                    slot,
                    card,
                    before: effect_pixels(frame, slot)?,
                    previous: None,
                }),
                _ => require(before == after, "invalid removal identity transition")?,
            }
        }
        require(
            targets.len() <= 2,
            "logical action removes more than two tableau cards",
        )?;
        if !targets.is_empty() {
            self.pending = Some(Pending {
                settings,
                expected: after,
                targets,
                stable: 0,
                capture: None,
            });
        }
        Ok(())
    }


    /// Compare all expected removed corners and stage one distinct post-ACK capture.
    ///
    /// A candidate is not committed until the worker accepts every other expected
    /// identity/pile/scene/header check. No comparison result grants input authority.
    pub(crate) fn stage(
        &mut self,
        frame: &CapturedFrame,
        expected: &[Option<Card>; 28],
        capture: usize,
        vision: &ReconVision,
    ) -> Result<RemovalCandidate, String> {
        let Some(pending) = &mut self.pending else {
            require(
                expected
                    .iter()
                    .enumerate()
                    .all(|(slot, card)| card.is_none() == self.confirmed.contains(slot)),
                "unproven removal in expected position",
            )?;
            return Ok(RemovalCandidate {
                proof: self.confirmed,
                settled: true,
            });
        };
        require(
            *expected == pending.expected,
            "different pending removal position",
        )?;
        require(
            pending.capture.is_none_or(|previous| capture > previous),
            "removal settling requires a fresh capture",
        )?;
        pending.capture = Some(capture);
        let mut proof = self.confirmed;
        let mut all_quiet = true;
        for target in &mut pending.targets {
            let view = frame
                .as_view()
                .map_err(|_| "invalid removal frame layout")?;
            require(
                vision
                    .tableau_reader
                    .recognise(view, TABLEAU_ORIGINS[target.slot])?
                    .is_none(),
                &format!(
                    "execution slot {} still contains a recognised card after {:?}",
                    target.slot, target.card
                ),
            )?;
            let current = effect_pixels(frame, target.slot)?;
            let measured = crate::pyramid_effect::compare_with_settings(&target.before, &current, pending.settings)?;
            if !measured.exceeds_allowance() {
                return Err(format!(
                    "execution slot {} lacks joint RGB quarter change: changed={}/{}; allowed={}; {}; {}",
                    target.slot,
                    measured.changed_pixels,
                    measured.total_pixels,
                    measured.allowed_changes, pending.settings.description(), crate::pyramid_effect::measurement_text(measured)
                ));
            }
            let quiet_corner = corner(frame, target.slot)?;
            all_quiet &= target.previous.as_ref().is_some_and(|previous| {
                compare_packed_bytes(previous, &quiet_corner, BYTE_TOLERANCE).is_ok_and(quiet)
            });
            target.previous = Some(quiet_corner);
            proof.slots |= 1_u32 << target.slot;
        }
        pending.stable = if all_quiet {
            pending.stable.saturating_add(1)
        } else {
            1
        };
        Ok(RemovalCandidate {
            proof,
            settled: pending.stable >= SETTLED_CAPTURES,
        })
    }


    /// Break consecutive settling after scene, identity, pile or magnitude refusal.
    pub(crate) fn reject(&mut self) {
        if let Some(pending) = &mut self.pending {
            pending.stable = 0;
            for target in &mut pending.targets {
                target.previous = None;
            }
        }
    }


    /// Commit only an entirely checked settled candidate; earlier slots remain fixed.
    pub(crate) fn commit(&mut self, candidate: RemovalCandidate) -> Result<(), String> {
        require(
            candidate.settled,
            "removal appearance has not settled across two captures",
        )?;
        self.confirmed = candidate.proof;
        self.pending = None;
        Ok(())
    }
}


impl ReconVision {
    /// Make a private execution copy with no previously confirmed removals.
    ///
    /// Static recognition, acquisition and historical fixture readers retain their
    /// original policy. Production execution uses only this non-serialised mode.
    pub(crate) fn for_temporal_execution(&self) -> Self {
        let mut execution = self.clone();
        execution.temporal_removed = Some(RemovalProof::default());
        execution
    }


    /// Install only the current route's candidate or confirmed original-slot history.
    pub(crate) fn set_temporal_removals(&mut self, proof: RemovalProof) {
        self.temporal_removed = Some(proof);
    }


    /// Verify complete committed/candidate history and veto any still recognised card.
    pub(super) fn verify_temporal_clear(&self, view: FrameView<'_>) -> Result<(), String> {
        require(
            self.temporal_removed.is_some_and(RemovalProof::complete),
            "final-clear lacks complete temporal removal history",
        )?;
        for (slot, origin) in TABLEAU_ORIGINS.into_iter().enumerate() {
            require(
                self.tableau_reader.recognise(view, origin)?.is_none(),
                &format!("final-clear slot {slot} still contains a recognised card"),
            )?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests;


#[cfg(test)]
mod joint_tests;
