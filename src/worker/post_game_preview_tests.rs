//! Post-game failure previews retain the newest pixels and request identity.

use super::*;


/// Small distinct frames exercise delivery without introducing image recognition.
fn observation(marker: u8) -> FrameObservation {
    FrameObservation {
        frame: CapturedFrame {
            width: 1,
            height: 1,
            stride: 4,
            format: PixelFormat::Rgba8,
            pixels: vec![marker, marker, marker, 255],
        },
        prediction: PredictedAction::Action(
            pyramid::action_for_kind(pyramid::PyramidTargetKind::Move)
                .expect("canonical Pyramid Move"),
        ),
        observed_rows: None,
        gameplay_scene: false,
        game_progress: None,
    }
}


/// A sleeping UI drains the failure state before consuming the newest preview.
#[test]
fn newest_post_game_frame_remains_diagnostic_after_coalescing_and_uncertain_state() {
    let (sender, receiver) = mpsc::channel();
    let context = ControllerContext::new(
        PathBuf::from("/tmp/pyramid-preview.sock"),
        GameMode::Pyramid,
        SolvingStrategy::ComputerVision,
        9,
    );
    let sink = WorkerEventSink {
        events: sender,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(context.clone())),
    };
    let stale = observation(11);
    sink.publish_frame(stale.frame, stale.prediction, true);
    publish_post_game_observation(&sink, &observation(22));
    publish_post_game_observation(&sink, &observation(33));
    send_state(&sink, WorkerState::Uncertain);

    let notification = receiver.try_recv().expect("post-game failure state");
    assert!(matches!(notification.event, WorkerEvent::State(WorkerState::Uncertain)));
    assert_eq!(notification.context, Some(context.clone()));
    let newest = sink.latest_frame.take().expect("newest post-game preview");
    assert_eq!(newest.frame.pixels, [33, 33, 33, 255]);
    assert_eq!(newest.context, Some(context));
    assert_eq!(newest.coalesced_frames, 2);
    assert!(newest.diagnostic);
    assert_eq!(newest.prediction, None);
    assert!(!newest.log_prediction);
    assert!(sink.latest_frame.take().is_none());
}


/// Consuming pixels before the failure event cannot expose an advisory target.
#[test]
fn already_consumed_post_game_preview_has_no_prediction_when_stop_arrives() {
    let (sender, receiver) = mpsc::channel();
    let context = ControllerContext::new(
        PathBuf::from("/tmp/tripeaks-preview.sock"),
        GameMode::TriPeaks,
        SolvingStrategy::ComputerVision,
        10,
    );
    let sink = WorkerEventSink {
        events: sender,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(context.clone())),
    };
    publish_post_game_observation(&sink, &observation(44));
    let displayed = sink.latest_frame.take().expect("UI consumes transition preview");
    send_state(&sink, WorkerState::Uncertain);
    assert!(matches!(receiver.try_recv().unwrap().event,
        WorkerEvent::State(WorkerState::Uncertain)));
    assert_eq!(displayed.frame.pixels, [44, 44, 44, 255]);
    assert_eq!(displayed.context, Some(context));
    assert!(displayed.diagnostic);
    assert_eq!(displayed.prediction, None);
    assert!(sink.latest_frame.take().is_none());

    // A subsequently verified board resumes the normal advisory preview policy.
    let resumed = observation(55);
    sink.publish_frame(resumed.frame, resumed.prediction, false);
    let next = sink.latest_frame.take().expect("verified gameplay preview");
    assert_eq!(next.frame.pixels, [55, 55, 55, 255]);
    assert!(!next.diagnostic);
    assert_eq!(next.prediction, Some(resumed.prediction));
}
