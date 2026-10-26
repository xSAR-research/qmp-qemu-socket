//! Single-use source evidence and ordered whole-action input authority.
//!
//! One matched frame can confirm the preceding action and authorise the next
//! action. Permits remain private to one native adapter, route and acquisition.

use super::*;


/// Exact checked source with no intervening capture, wait or input attempt.
#[derive(Clone, Copy)]
pub(super) struct SourceProof {
    /// Immutable independently replayed position.
    state: SearchState,
    /// Prepared route identity.
    route: u64,
    /// Acquisition authority revoked by STOP or a changed selection.
    epoch: u64,
    /// Serial of the complete matched frame.
    capture: usize,
    /// Input attempts and acknowledgements when this frame was checked.
    counts: (usize, usize),
}


impl SourceProof {
    /// Bind evidence to the currently borrowed adapter and exact input history.
    pub(super) fn new(io: &NativeIo<'_>, state: SearchState) -> Self {
        Self {
            state,
            route: io.plan.id,
            epoch: io.plan.acquisition.epoch,
            capture: io.captures,
            counts: (io.counts.attempted, io.counts.acknowledged),
        }
    }


    /// Reject reuse after a different state, origin, frame or input history.
    pub(super) fn matches(self, io: &NativeIo<'_>, state: SearchState) -> bool {
        self.state == state
            && self.route == io.plan.id
            && self.epoch == io.plan.acquisition.epoch
            && self.capture == io.captures
            && self.counts == (io.counts.attempted, io.counts.acknowledged)
            && io.counts.attempted == io.counts.acknowledged
    }
}


/// Consumed source evidence covering exactly one operation or one ordered pair.
pub(super) struct ActionPermit {
    /// Source proof consumed when the logical action begins.
    source: SourceProof,
    /// Only the operations selected by the independently replayed action.
    operations: Vec<InputOperation>,
    /// Next operation; transport uncertainty discards the remaining permit.
    next: usize,
}


impl ActionPermit {
    /// Bind one operation or two clicks; malformed internal actions grant nothing.
    pub(super) fn new(source: SourceProof, operations: &[InputOperation]) -> Result<Self, String> {
        if operations.is_empty()
            || operations.len() > 2
            || (operations.len() == 2
                && !operations
                    .iter()
                    .all(|op| matches!(op, InputOperation::Click(_))))
        {
            return Err("Invalid whole-action operation list; no input authorised".to_owned());
        }
        Ok(Self {
            source,
            operations: operations.to_vec(),
            next: 0,
        })
    }


    /// Verify exact ordering and successful prior releases without another capture.
    pub(super) fn matches(
        &self,
        io: &NativeIo<'_>,
        state: SearchState,
        operation: InputOperation,
    ) -> bool {
        let Some(attempted) = self.source.counts.0.checked_add(self.next) else {
            return false;
        };
        let Some(acknowledged) = self.source.counts.1.checked_add(self.next) else {
            return false;
        };
        self.source.state == state
            && self.source.route == io.plan.id
            && self.source.epoch == io.plan.acquisition.epoch
            && self.source.capture == io.captures
            && io.counts.attempted == attempted
            && io.counts.acknowledged == acknowledged
            && self.operations.get(self.next) == Some(&operation)
    }


    /// Consume one fully acknowledged operation; retain only an unfinished pair.
    pub(super) fn acknowledged(mut self) -> Option<Self> {
        self.next += 1;
        (self.next < self.operations.len()).then_some(self)
    }
}
