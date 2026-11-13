//! Single-use fresh-frame/input boundary shared by future game adapters.

use super::{
    OperationError,
    option::{InclusiveBounds, OptionDetection, detect_option},
    validate_native_frame,
};
use crate::{
    capture::CapturedFrame,
    geometry::{PixelPoint, pixel_point_to_qmp},
    qmp::SupportedKey,
};
use std::time::Duration;


/// One supported application input; transport ACK does not prove its effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    /// One complete checked press/release at native guest pixels.
    MouseLeft(PixelPoint),
    /// One typed key with existing QMP timing.
    Key(SupportedKey),
}


/// Input delivery record, retained even when a post-delivery guard fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Delivery {
    /// Frame generation consumed before this input attempt.
    pub frame_serial: u64,
    /// Acknowledged input; semantic completion remains caller-owned.
    pub input: Input,
}


/// Native adapters retain context, capture accounting and existing timing policy.
pub trait OperationIo {
    /// Check STOP, shutdown, deadline and exact selected controller context.
    fn check_active(&mut self) -> Result<(), OperationError>;
    /// Acquire one new frame via the selected existing capture backend.
    fn capture(&mut self) -> Result<CapturedFrame, OperationError>;
    /// Pure selected-game/scene/action/freshness check on this same frame.
    fn authorise(&mut self, frame: &CapturedFrame, input: Input) -> Result<(), OperationError>;
    /// Fresh read-only running-VM and current absolute-pointer probe.
    fn probe(&mut self) -> Result<(), OperationError>;
    /// Attempt exactly once; every error here is conservatively uncertain.
    fn deliver(&mut self, input: Input) -> Result<(), OperationError>;
    /// Existing game-selected cancellable timing; never performs cleanup input.
    fn wait(&mut self, duration: Duration) -> Result<(), OperationError>;
}


/// Own fresh pixels and permanently fence an uncertain input in this run.
pub struct Session<I> {
    /// Borrowed or owned adapter; no new thread/client architecture is introduced.
    io: I,
    /// Latest successfully validated capture; taken before any input attempt.
    frame: Option<CapturedFrame>,
    /// Checked capture generation, never reused after exhaustion.
    serial: u64,
    /// Attempt-in-flight or uncertain delivery permanently disallows further input.
    uncertain: bool,
    /// Last acknowledged input, independent of semantic effect and cancellation.
    last_delivery: Option<Delivery>,
}


impl<I: OperationIo> Session<I> {
    /// Start without pixels or input authority; does not clear adapter STOP/latches.
    pub fn new(io: I) -> Self {
        Self {
            io,
            frame: None,
            serial: 0,
            uncertain: false,
            last_delivery: None,
        }
    }


    /// Borrow the underlying operation I/O adapter without granting input authority.
    pub const fn io(&self) -> &I {
        &self.io
    }


    /// Acquire once; any failure retires the previous capture before returning.
    pub fn screen_capture(&mut self) -> Result<&CapturedFrame, OperationError> {
        self.frame = None;
        self.io.check_active()?;
        self.serial = self
            .serial
            .checked_add(1)
            .ok_or_else(|| OperationError::Invalid("capture serial exhausted".to_owned()))?;
        let frame = self.io.capture()?;
        validate_native_frame(&frame)?;
        self.io.check_active()?;
        self.frame = Some(frame);
        self.frame
            .as_ref()
            .ok_or_else(|| OperationError::Invalid("capture missing".to_owned()))
    }


    /// Send one checked native click after same-frame game authority and fresh probe.
    pub fn mouse_left_click(&mut self, point: PixelPoint) -> Result<Delivery, OperationError> {
        self.input(Input::MouseLeft(point))
    }


    /// Send one typed supported key; no arbitrary integer keycode or qcode parsing.
    pub fn send_key_stroke(&mut self, key: SupportedKey) -> Result<Delivery, OperationError> {
        self.input(Input::Key(key))
    }


    /// Redetect one unique run in the current frame, then use the ordinary guard.
    pub fn click_option(
        &mut self,
        bounds: InclusiveBounds,
        minimum_length: u32,
    ) -> Result<Delivery, OperationError> {
        let frame = self.frame.as_ref().ok_or_else(|| {
            OperationError::Invalid("fresh capture required before input".to_owned())
        })?;
        match detect_option(frame, bounds, minimum_length)? {
            OptionDetection::Found(target) => self.mouse_left_click(target.point),
            OptionDetection::NotFound => Err(OperationError::NotFound),
            OptionDetection::Ambiguous => Err(OperationError::Ambiguous),
        }
    }


    /// Expose acknowledged delivery separately from effect or subsequent failure.
    pub fn last_delivery(&self) -> Option<Delivery> {
        self.last_delivery
    }


    /// Cancellable bounded wait chosen by the game adapter, without granting input.
    pub fn wait(&mut self, duration: Duration) -> Result<(), OperationError> {
        self.frame = None;
        self.io.check_active()?;
        self.io.wait(duration)?;
        self.io.check_active()
    }


    /// Consume authority before input and never recover a failed delivery latch.
    fn input(&mut self, input: Input) -> Result<Delivery, OperationError> {
        if self.uncertain {
            return Err(OperationError::Uncertain(Box::new(
                OperationError::Verification("earlier input attempt remains uncertain".to_owned()),
            )));
        }
        let frame = self.frame.take().ok_or_else(|| {
            OperationError::Invalid("fresh capture required before input".to_owned())
        })?;
        self.io.check_active()?;
        if let Input::MouseLeft(point) = input {
            pixel_point_to_qmp(point, frame.width, frame.height)
                .map_err(|e| OperationError::Invalid(e.to_string()))?;
        }
        self.io.authorise(&frame, input)?;
        self.io.check_active()?;
        self.io.probe()?;
        self.io.check_active()?;
        self.uncertain = true;
        self.io
            .deliver(input)
            .map_err(|e| OperationError::Uncertain(Box::new(e)))?;
        self.uncertain = false;
        let delivery = Delivery {
            frame_serial: self.serial,
            input,
        };
        self.last_delivery = Some(delivery);
        self.io
            .check_active()
            .map_err(|e| OperationError::InterruptedAfterDelivery(Box::new(e)))?;
        Ok(delivery)
    }
}
