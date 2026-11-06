//! Inclusive, borrowed-frame detection of one maximal horizontal exact-black run.

use super::OperationError;
use crate::{
    capture::CapturedFrame,
    geometry::{PixelPoint, PixelRect},
};
use xsar::image_matching::find_horizontal_run;


/// Inclusive native-display endpoints; conversion uses checked widened geometry.
#[derive(Clone, Copy, Debug)]
pub(crate) struct InclusiveBounds {
    /// First included x coordinate.
    pub left: u32,
    /// First included y coordinate.
    pub top: u32,
    /// Last included x coordinate.
    pub right: u32,
    /// Last included y coordinate.
    pub bottom: u32,
}


/// Measured pixels only; this value grants no game or input authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DetectedOption {
    /// Interior point on the unique recognised run, including valid display origin.
    pub point: PixelPoint,
    /// Complete maximal horizontal run length.
    pub length: u32,
}


/// Explicit absence and multiple groups differ from invalid frame/configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OptionDetection {
    /// No maximal run meets the inclusive minimum.
    NotFound,
    /// At least two maximal runs qualify, including separate adjacent rows.
    Ambiguous,
    /// Exactly one complete maximal run qualifies.
    Found(DetectedOption),
}


/// Search disjoint checked ROIs without copying pixels or issuing I/O.
///
/// Alpha and row padding are ignored by xSAR's RGB view. Thick two-dimensional
/// lettering is deliberately not grouped: each maximal horizontal run is one
/// candidate. The caller chooses a region that isolates its specified option.
pub(crate) fn detect_option(
    frame: &CapturedFrame,
    bounds: InclusiveBounds,
    minimum_length: u32,
) -> Result<OptionDetection, OperationError> {
    if minimum_length == 0 {
        return Err(OperationError::Invalid(
            "option minimum length must be positive".to_owned(),
        ));
    }
    let width = bounds
        .right
        .checked_sub(bounds.left)
        .and_then(|v| v.checked_add(1))
        .ok_or_else(|| OperationError::Invalid("invalid inclusive option x bounds".to_owned()))?;
    let height = bounds
        .bottom
        .checked_sub(bounds.top)
        .and_then(|v| v.checked_add(1))
        .ok_or_else(|| OperationError::Invalid("invalid inclusive option y bounds".to_owned()))?;
    let rect = PixelRect {
        x: bounds.left,
        y: bounds.top,
        width,
        height,
    };
    let view = frame
        .as_view()
        .map_err(|e| OperationError::Invalid(e.to_string()))?;
    let (right, bottom) = view
        .checked_bounds(rect)
        .map_err(|e| OperationError::Invalid(e.to_string()))?;
    let search = |rect| {
        find_horizontal_run(view, rect, minimum_length, None, |rgb| rgb == [0, 0, 0])
            .map_err(|e| OperationError::Invalid(e.to_string()))
    };
    let Some(hit) = search(rect)? else {
        return Ok(OptionDetection::NotFound);
    };
    let x = u32::try_from(hit.anchor.x).map_err(|e| OperationError::Invalid(e.to_string()))?;
    let y = u32::try_from(hit.anchor.y).map_err(|e| OperationError::Invalid(e.to_string()))?;
    let suffix = x
        .checked_add(hit.length)
        .ok_or_else(|| OperationError::Invalid("option run edge overflow".to_owned()))?;
    if suffix < right
        && search(PixelRect {
            x: suffix,
            y,
            width: right - suffix,
            height: 1,
        })?
        .is_some()
    {
        return Ok(OptionDetection::Ambiguous);
    }
    let next_row = y
        .checked_add(1)
        .ok_or_else(|| OperationError::Invalid("option row overflow".to_owned()))?;
    if next_row < bottom
        && search(PixelRect {
            x: bounds.left,
            y: next_row,
            width,
            height: bottom - next_row,
        })?
        .is_some()
    {
        return Ok(OptionDetection::Ambiguous);
    }
    let middle = x
        .checked_add((hit.length - 1) / 2)
        .ok_or_else(|| OperationError::Invalid("option midpoint overflow".to_owned()))?;
    let point = PixelPoint {
        x: i32::try_from(middle).map_err(|e| OperationError::Invalid(e.to_string()))?,
        y: hit.anchor.y,
    };
    Ok(OptionDetection::Found(DetectedOption {
        point,
        length: hit.length,
    }))
}
