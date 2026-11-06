//! Private reusable game operations over existing application and xSAR facilities.
//!
//! Recognition borrows pixels and grants no input authority. A session owns one
//! fresh frame, consumes it before input and latches uncertain delivery. Physical
//! restoration never deletes learned identities or invents an undo history.

// The catalogue is intentionally usable by later game adapters, while Undo All
// and typed Draw already have production consumers in this revision.
#![allow(dead_code)]

pub(crate) mod option;
pub(crate) mod restoration;
mod session;

#[allow(unused_imports)]
pub(crate) use session::{Delivery, Input, OperationIo, Session};

use crate::{capture::CapturedFrame, pyramid_recon::vision::ReconVision};
use xsar::games::{cards::Card, pyramid::planning::Location};


/// Operation failures retain their category; uncertain input is never retried.
#[derive(Debug, thiserror::Error)]
pub(crate) enum OperationError {
    /// Invalid coordinates, storage, bounds or operation configuration.
    #[error("{0}")]
    Invalid(String),
    /// Capability has no measured adapter for this game.
    #[error("{0}")]
    Unsupported(String),
    /// A required control was not found.
    #[error("control not found")]
    NotFound,
    /// Positive disabled appearance.
    #[error("control is disabled")]
    Disabled,
    /// More than one candidate interpretation survives.
    #[error("ambiguous control or card evidence")]
    Ambiguous,
    /// Cancellation or lost controller context before input.
    #[error("operation cancelled")]
    Cancelled,
    /// Read-only transport failure, before any attempted input.
    #[error("{0}")]
    Transport(String),
    /// Supported scene/card recognition failed structurally.
    #[error("{0}")]
    Recognition(String),
    /// Semantic expected-position or selected-game guard failed.
    #[error("{0}")]
    Verification(String),
    /// Adapter failure retained verbatim while extracting an existing protocol.
    #[error("{0}")]
    Adapter(String),
    /// Input was attempted and its delivery cannot be established; latch remains set.
    #[error("input delivery uncertain; no replay: {0}")]
    Uncertain(Box<OperationError>),
    /// Delivery was acknowledged, then cancellation/context checking failed.
    #[error("input acknowledged before interruption: {0}")]
    InterruptedAfterDelivery(Box<OperationError>),
}


/// An explicitly game-scoped checked physical location; no silent row compaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CardLocation {
    /// Existing checked original one-based tableau or DL/DR location vocabulary.
    Pyramid(Location),
}


/// Pure card-reading outcomes; failed recognition never means Empty.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CardRead {
    /// Uniquely recognised rank and suit.
    Card(Card),
    /// Positive empty-pile evidence, exclusive of a recognised identity.
    Empty,
    /// Supported location lacks a complete readable identity or empty proof.
    Unreadable,
    /// Identity and positive empty evidence conflict.
    Ambiguous,
}


/// Validate exact native packed RGBA8 without copying, resizing or disk writes.
pub(crate) fn validate_native_frame(frame: &CapturedFrame) -> Result<(), OperationError> {
    if frame.width != 1_920
        || frame.height != 1_080
        || frame.stride != 7_680
        || frame.pixels.len() != 8_294_400
        || !frame.is_layout_valid()
    {
        return Err(OperationError::Invalid(
            "operation requires packed native 1920x1080 RGBA8".to_owned(),
        ));
    }
    frame
        .as_view()
        .map_err(|e| OperationError::Invalid(e.to_string()))?;
    Ok(())
}


/// Read one location from a borrowed frame; no capture, input, cache or inference.
pub(crate) fn read_card(
    vision: &ReconVision,
    frame: &CapturedFrame,
    location: CardLocation,
) -> Result<CardRead, OperationError> {
    validate_native_frame(frame)?;
    match location {
        CardLocation::Pyramid(location) => vision.read_operation_card(frame, location),
    }
}


#[cfg(test)]
mod tests;
