//! Observable contracts for pure recognition, guarded input and single Undo.

use super::option::{InclusiveBounds, OptionDetection, detect_option};
use super::restoration::{ControlAvailability, UndoLastIo, UndoLastObservation, undo_last};
use super::*;
use crate::{capture::PixelFormat, geometry::PixelPoint, qmp::SupportedKey};
use std::{cell::RefCell, collections::VecDeque, rc::Rc, time::Duration};


/// Valid white packed native pixels, with alpha deliberately irrelevant to matching.
fn native() -> CapturedFrame {
    CapturedFrame {
        width: 1_920,
        height: 1_080,
        stride: 7_680,
        format: PixelFormat::Rgba8,
        pixels: vec![255; 8_294_400],
    }
}


/// Paint a synthetic horizontal RGB run without modifying other pixels or alpha.
fn black(frame: &mut CapturedFrame, x: u32, y: u32, length: u32) {
    for x in x..x + length {
        let index = y as usize * frame.stride + x as usize * 4;
        frame.pixels[index..index + 3].copy_from_slice(&[0; 3]);
        frame.pixels[index + 3] = 0;
    }
}


/// Full inclusive display endpoints, including the origin and final pixel.
fn display() -> InclusiveBounds {
    InclusiveBounds {
        left: 0,
        top: 0,
        right: 1_919,
        bottom: 1_079,
    }
}


/// Long runs count once; full-display corner pixels are valid targets.
#[test]
fn operations_option_inclusive_edges_and_maximal_runs() {
    let mut frame = native();
    black(&mut frame, 0, 0, 7);
    let OptionDetection::Found(hit) = detect_option(&frame, display(), 5).unwrap() else {
        panic!("unique run");
    };
    assert_eq!((hit.point, hit.length), (PixelPoint::new(3, 0), 7));
    let origin = InclusiveBounds {
        right: 0,
        bottom: 0,
        ..display()
    };
    assert!(
        matches!(detect_option(&frame,origin,1).unwrap(),OptionDetection::Found(hit) if hit.point==PixelPoint::new(0,0))
    );
    frame = native();
    black(&mut frame, 1_919, 1_079, 1);
    assert!(
        matches!(detect_option(&frame,display(),1).unwrap(),OptionDetection::Found(hit) if hit.point==PixelPoint::new(1_919,1_079))
    );
}


/// Distinct runs in the same or adjacent row cannot silently choose an option.
#[test]
fn operations_option_ambiguity_and_exact_black() {
    for (x, y) in [(20, 10), (10, 11)] {
        let mut frame = native();
        black(&mut frame, 10, 10, 5);
        black(&mut frame, x, y, 5);
        assert_eq!(
            detect_option(&frame, display(), 5).unwrap(),
            OptionDetection::Ambiguous
        );
    }
    let mut frame = native();
    black(&mut frame, 10, 10, 5);
    for x in 10..15 {
        frame.pixels[10 * frame.stride + x * 4 + 2] = 1;
    }
    assert_eq!(
        detect_option(&frame, display(), 5).unwrap(),
        OptionDetection::NotFound
    );
}


/// Invalid inclusive bounds/storage fail; row padding never becomes black evidence.
#[test]
fn operations_option_rejects_bad_bounds_and_ignores_padding() {
    let frame = native();
    for bounds in [
        InclusiveBounds {
            left: 2,
            right: 1,
            ..display()
        },
        InclusiveBounds {
            bottom: 1_080,
            ..display()
        },
        InclusiveBounds {
            right: u32::MAX,
            ..display()
        },
    ] {
        assert!(detect_option(&frame, bounds, 1).is_err());
    }
    assert!(detect_option(&frame, display(), 0).is_err());
    let mut padded = CapturedFrame {
        width: 2,
        height: 2,
        stride: 12,
        format: PixelFormat::Rgba8,
        pixels: vec![255; 24],
    };
    padded.pixels[8..12].fill(0);
    padded.pixels[20..24].fill(0);
    assert_eq!(
        detect_option(
            &padded,
            InclusiveBounds {
                left: 0,
                top: 0,
                right: 1,
                bottom: 1
            },
            1
        )
        .unwrap(),
        OptionDetection::NotFound
    );
    padded.pixels.pop();
    assert!(
        detect_option(
            &padded,
            InclusiveBounds {
                left: 0,
                top: 0,
                right: 1,
                bottom: 1
            },
            1
        )
        .is_err()
    );
    assert!(validate_native_frame(&padded).is_err());
}


/// Scriptable adapter boundary; tests observe transport attempts, not wrapper calls.
#[derive(Default)]
struct Trace {
    /// Significant authority/capture/transport events in actual call order.
    events: Vec<&'static str>,
    /// Inject one selected failure point.
    fail: Option<&'static str>,
    /// STOP can arrive during a probe or after an ACK.
    stopped: bool,
}


/// In-memory transport with externally observable cancellation and attempt state.
struct Fake(Rc<RefCell<Trace>>);


impl OperationIo for Fake {
    fn check_active(&mut self) -> Result<(), OperationError> {
        if self.0.borrow().stopped {
            Err(OperationError::Cancelled)
        } else {
            Ok(())
        }
    }
    fn capture(&mut self) -> Result<CapturedFrame, OperationError> {
        self.0.borrow_mut().events.push("capture");
        if self.0.borrow().fail == Some("capture") {
            return Err(OperationError::Transport("capture".to_owned()));
        }
        Ok(native())
    }
    fn authorise(&mut self, _frame: &CapturedFrame, _input: Input) -> Result<(), OperationError> {
        self.0.borrow_mut().events.push("guard");
        if self.0.borrow().fail == Some("guard") {
            Err(OperationError::Verification("guard".to_owned()))
        } else {
            Ok(())
        }
    }
    fn probe(&mut self) -> Result<(), OperationError> {
        self.0.borrow_mut().events.push("probe");
        let fail = self.0.borrow().fail;
        if fail == Some("stop-probe") {
            self.0.borrow_mut().stopped = true;
        }
        if fail == Some("probe") {
            Err(OperationError::Transport("probe".to_owned()))
        } else {
            Ok(())
        }
    }
    fn deliver(&mut self, _input: Input) -> Result<(), OperationError> {
        self.0.borrow_mut().events.push("input");
        let fail = self.0.borrow().fail;
        if fail == Some("stop-ack") {
            self.0.borrow_mut().stopped = true;
        }
        if fail == Some("input") {
            Err(OperationError::Transport("input".to_owned()))
        } else {
            Ok(())
        }
    }
    fn wait(&mut self, _duration: Duration) -> Result<(), OperationError> {
        self.0.borrow_mut().events.push("wait");
        Ok(())
    }
}


/// Fresh evidence is consumed once; key ACK never supplies another click's authority.
#[test]
fn operations_input_requires_fresh_same_frame_guard_and_probe() {
    let trace = Rc::new(RefCell::new(Trace::default()));
    let mut session = Session::new(Fake(trace.clone()));
    assert!(session.send_key_stroke(SupportedKey::Draw).is_err());
    session.screen_capture().unwrap();
    let delivered = session.send_key_stroke(SupportedKey::Draw).unwrap();
    assert_eq!(delivered.frame_serial, 1);
    assert!(session.mouse_left_click(PixelPoint::new(0, 0)).is_err());
    assert_eq!(
        trace.borrow().events,
        ["capture", "guard", "probe", "input"]
    );
    assert_eq!(session.last_delivery(), Some(delivered));
}


/// Invalid coordinates, guard/probe errors and STOP after probe send zero inputs.
#[test]
fn operations_refusals_and_probe_stop_precede_input() {
    for fail in [Some("guard"), Some("probe"), Some("stop-probe"), None] {
        let trace = Rc::new(RefCell::new(Trace {
            fail,
            ..Trace::default()
        }));
        let mut session = Session::new(Fake(trace.clone()));
        session.screen_capture().unwrap();
        let point = if fail.is_none() {
            PixelPoint::new(1_920, 0)
        } else {
            PixelPoint::new(10, 10)
        };
        assert!(session.mouse_left_click(point).is_err());
        assert!(!trace.borrow().events.contains(&"input"));
        assert!(session.send_key_stroke(SupportedKey::Draw).is_err());
    }
}


/// Failed capture retires old pixels, and uncertain input survives fresh captures.
#[test]
fn operations_failure_retires_pixels_and_fences_uncertain_replay() {
    let trace = Rc::new(RefCell::new(Trace::default()));
    let mut session = Session::new(Fake(trace.clone()));
    session.screen_capture().unwrap();
    trace.borrow_mut().fail = Some("capture");
    assert!(session.screen_capture().is_err());
    assert!(session.send_key_stroke(SupportedKey::Draw).is_err());
    trace.borrow_mut().fail = Some("input");
    session.screen_capture().unwrap();
    assert!(matches!(
        session.send_key_stroke(SupportedKey::Draw),
        Err(OperationError::Uncertain(_))
    ));
    trace.borrow_mut().fail = None;
    session.screen_capture().unwrap();
    assert!(matches!(
        session.send_key_stroke(SupportedKey::Draw),
        Err(OperationError::Uncertain(_))
    ));
    assert_eq!(
        trace
            .borrow()
            .events
            .iter()
            .filter(|&&event| event == "input")
            .count(),
        1
    );
}


/// Cancellation after ACK retains delivery evidence and gives no new authority.
#[test]
fn operations_post_ack_stop_retains_delivery_without_effect_claim() {
    let trace = Rc::new(RefCell::new(Trace {
        fail: Some("stop-ack"),
        ..Trace::default()
    }));
    let mut session = Session::new(Fake(trace));
    session.screen_capture().unwrap();
    assert!(matches!(
        session.send_key_stroke(SupportedKey::Draw),
        Err(OperationError::InterruptedAfterDelivery(_))
    ));
    assert!(session.last_delivery().is_some());
    assert!(session.screen_capture().is_err());
}


/// Partial reads do not depend on unrelated card identities; empty remains positive.
#[test]
fn operations_read_card_borrows_partial_board_and_preserves_empty_distinction() {
    use xsar::games::{
        cards::{Card, Rank, Suit},
        pyramid::planning::{Location, location_from_row_column},
    };
    let mut frame = crate::capture::decode_png(include_bytes!(
        "../../tests/fixtures/pyramid-recognition-176/coral-entry.png"
    ))
    .unwrap();
    let vision = ReconVision::bundled_native_v23().unwrap();
    let location = CardLocation::Pyramid(location_from_row_column(4, 2).unwrap());
    assert_eq!(
        read_card(&vision, &frame, location).unwrap(),
        CardRead::Card(Card::new(Rank::Four, Suit::Clubs))
    );
    assert_eq!(
        read_card(&vision, &frame, CardLocation::Pyramid(Location::DR)).unwrap(),
        CardRead::Empty
    );
    for y in 116..165 {
        for x in 893..924 {
            let i = y * frame.stride + x * 4;
            frame.pixels[i..i + 3].fill(255);
        }
    }
    assert!(!matches!(
        vision.observe(&frame),
        Ok(crate::pyramid_recon::vision::ReconScene::Board(_))
    ));
    assert!(matches!(
        read_card(&vision, &frame, location),
        Ok(CardRead::Card(_))
    ));
    for y in 276..325 {
        for x in 793..824 {
            let i = y * frame.stride + x * 4;
            frame.pixels[i..i + 3].fill(255);
        }
    }
    assert_eq!(
        read_card(&vision, &frame, location).unwrap(),
        CardRead::Unreadable
    );
}


/// Single Undo script exposes one action and independent physical observations.
struct SingleUndo {
    /// Current and subsequent complete physical snapshots.
    scenes: VecDeque<UndoLastObservation<u8>>,
    /// Significant observable operations.
    events: Vec<&'static str>,
}


impl UndoLastIo for SingleUndo {
    type Board = u8;
    fn observe(&mut self) -> Result<UndoLastObservation<u8>, OperationError> {
        self.events.push("capture");
        self.scenes.pop_front().ok_or_else(|| {
            OperationError::Unsupported("no measured single-Undo adapter".to_owned())
        })
    }
    fn input(&mut self, _point: PixelPoint) -> Result<(), OperationError> {
        self.events.push("input");
        Ok(())
    }
    fn settle(&mut self) -> Result<(), OperationError> {
        self.events.push("wait");
        Ok(())
    }
}


/// Disabled/absent/ambiguous/equal-history states cannot authorise single Undo.
#[test]
fn operations_single_undo_requires_enabled_control_and_distinct_predecessor() {
    for control in [
        ControlAvailability::Disabled,
        ControlAvailability::NotFound,
        ControlAvailability::Ambiguous,
        ControlAvailability::Enabled(PixelPoint::new(1, 2)),
    ] {
        let mut io = SingleUndo {
            scenes: VecDeque::from([UndoLastObservation { board: 2, control }]),
            events: vec![],
        };
        assert!(undo_last(&mut io, &1, &2, PixelPoint::new(10, 10), |a, b| a == b, 6).is_err());
        assert!(!io.events.contains(&"input"));
    }
    let mut io = SingleUndo {
        scenes: VecDeque::new(),
        events: vec![],
    };
    assert!(undo_last(&mut io, &2, &2, PixelPoint::new(10, 10), |a, b| a == b, 6).is_err());
    assert!(io.events.is_empty());
}


/// ACK alone is insufficient; success requires two consecutive predecessor captures.
#[test]
fn operations_single_undo_verifies_physical_restoration_without_replay() {
    let point = PixelPoint::new(10, 10);
    let scene = |board| UndoLastObservation {
        board,
        control: ControlAvailability::Enabled(point),
    };
    let mut io = SingleUndo {
        scenes: VecDeque::from([scene(2), scene(1), scene(2), scene(1), scene(1)]),
        events: vec![],
    };
    undo_last(&mut io, &1, &2, point, |a, b| a == b, 6).unwrap();
    assert_eq!(
        io.events,
        [
            "capture", "input", "wait", "capture", "capture", "wait", "capture", "capture"
        ]
    );
}
