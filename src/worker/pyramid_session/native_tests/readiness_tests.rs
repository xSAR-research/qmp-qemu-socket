//! Real native recogniser/readiness adapter over isolated QMP; frame orderings are synthetic.

use super::*;


/// Original failed first post-handoff frame; cards are still displaced by dealing.
const DEALING: &[u8] =
    include_bytes!("../../../../tests/fixtures/pyramid-starting-readiness/dealing.png");


/// A second original dealing frame has partial ink in the usual glyph area.
const DEALING_LATER: &[u8] =
    include_bytes!("../../../../tests/fixtures/pyramid-starting-readiness/dealing-later.png");


/// Later original successful Single board's untouched, settled arrangement.
const SETTLED: &[u8] =
    include_bytes!("../../../../tests/fixtures/pyramid-starting-readiness/settled.png");


/// Actual 132 Boards 2/3 can enter the native challenge adapter without pretending it is zero.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn execution_observability_native_partial_boards_entry_preserves_observed_two_of_three() {
    let original =
        include_bytes!("../../../../tests/fixtures/pyramid-execution-observability/boards-two.png");
    let episode = run(
        Operation::CardEntry,
        vec![original.to_vec(), original.to_vec()],
        Revocation::None,
    );
    let Entry::Challenge(progress) = episode.result.unwrap() else {
        panic!("literal Boards 2/3 must retain its challenge identity");
    };
    assert_eq!(
        progress,
        ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Boards, 2, 3).unwrap()
    );
    assert_eq!(episode.wire.captures, 2);
    assert!(episode.starting.is_none() && episode.series.is_none());
}


/// Serve the fresh Recon connection's probe and one changed board; any input is a test failure.
fn serve_changed_recon(listener: &UnixListener, directory: &std::path::Path) -> usize {
    let mut stream = accept(listener);
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .expect("bounded Recon connection");
    let mut reader = BufReader::new(stream.try_clone().expect("Recon reader"));
    packet(
        &mut stream,
        json!({"QMP": {"version": {"qemu": {"major": 11, "minor": 0, "micro": 0}}, "capabilities": []}}),
    );
    let mut captures = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).expect("Recon command") == 0 {
            return captures;
        }
        let request: Value = serde_json::from_str(&line).expect("Recon command JSON");
        let response = match request["execute"].as_str().expect("Recon execute") {
            "qmp_capabilities" => json!({}),
            "query-status" => json!({"running": true, "status": "running"}),
            "query-mice" => {
                json!([{"name": "QEMU HID Tablet", "index": 0, "current": true, "absolute": true}])
            }
            "screendump" => {
                assert_eq!(
                    captures, 0,
                    "changed starting position must fail on first Recon capture"
                );
                let destination = PathBuf::from(
                    request["arguments"]["filename"]
                        .as_str()
                        .expect("capture path"),
                );
                assert_eq!(destination.parent(), Some(directory));
                fs::write(destination, ZERO).expect("unchanged different native board");
                captures += 1;
                json!({})
            }
            other => panic!(
                "No input or unrelated command permitted after changed readiness witness: {other}"
            ),
        };
        packet(
            &mut stream,
            json!({"return": response, "id": request["id"]}),
        );
    }
}


/// Successful readiness cannot authorise a different position on Recon's new QMP connection.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn starting_readiness_binds_the_fresh_recon_connection_before_draw() {
    let _backend = TestCaptureBackend::legacy_qmp();
    let mut fixture = Fixture::new();
    fixture.request.reconnaissance.profile = ReconProfileSource::BundledNativeV22;
    fixture.request.stop_condition = StopCondition::NoChallenge;
    let listener =
        UnixListener::bind(&fixture.context.socket_path).expect("readiness/Recon socket");
    listener.set_nonblocking(true).expect("bounded accept");
    let directory = fixture.directory.0.clone();
    let context = fixture.context.clone();
    let control = fixture.events.latest_frame.reconnaissance.clone();
    let cancel = Arc::clone(&fixture.cancel);
    let server = thread::spawn(move || {
        let initial = serve(
            listener.try_clone().expect("same listener"),
            directory.clone(),
            vec![SETTLED.to_vec(), SETTLED.to_vec()].into(),
            control,
            context,
            cancel,
            Revocation::None,
        );
        assert_eq!(initial.captures, 2);
        assert!(
            initial
                .commands
                .iter()
                .all(|command| matches!(command.as_str(), "qmp_capabilities" | "screendump"))
        );
        serve_changed_recon(&listener, &directory)
    });
    let mut adapter = fixture.adapter();
    let error = adapter
        .acquire(2)
        .expect_err("changed fresh Recon must refuse");
    assert!(error.contains("tableau"), "{error}");
    assert_eq!(server.join().expect("two actual connections"), 1);
    assert!(adapter.output.is_none());
}


/// Ordinary glyph refusal stays unchanged; readiness supplies no identities until settled.
#[test]
fn starting_readiness_retains_ordinary_recognition_and_fatal_static_guards() {
    use crate::pyramid_recon::vision::StartingReadiness;
    let vision = ReconVision::bundled_native_v22().expect("current native profile");
    assert!(
        vision
            .observe(&native(DEALING))
            .unwrap_err()
            .contains("tableau slot 26")
    );
    for png in [DEALING, DEALING_LATER] {
        assert!(vision.observe(&native(png)).is_err());
        assert!(matches!(
            vision.starting_readiness(&native(png)),
            Ok(StartingReadiness::Pending(_))
        ));
    }
    assert!(matches!(
        vision.starting_readiness(&native(SETTLED)),
        Ok(StartingReadiness::Ready(_))
    ));
    let mut wrong = native(SETTLED);
    wrong.width = 1919;
    assert!(vision.starting_readiness(&wrong).is_err());
    let mut unsupported = native(SETTLED);
    unsupported.pixels.fill(0);
    assert!(vision.starting_readiness(&unsupported).is_err());
    assert!(vision.starting_readiness(&native(TERMINAL)).is_err());
}


/// A dealing frame needs two later matching complete captures, with no protocol input.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn starting_readiness_waits_for_two_actual_settled_frames_without_input() {
    let episode = run(
        Operation::StartingReadiness,
        vec![DEALING.to_vec(), SETTLED.to_vec(), SETTLED.to_vec()],
        Revocation::None,
    );
    assert!(episode.result.is_ok(), "{:?}", episode.result);
    assert_eq!(episode.wire.captures, 3);
    assert_eq!(episode.wire.remaining, 0);
}


/// Different complete positions and intervening incomplete cards reset the consecutive pair.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn starting_readiness_resets_pair_after_changed_or_pending_board() {
    let episode = run(
        Operation::StartingReadiness,
        vec![
            ZERO.to_vec(),
            SETTLED.to_vec(),
            DEALING.to_vec(),
            SETTLED.to_vec(),
            SETTLED.to_vec(),
        ],
        Revocation::None,
    );
    assert!(episode.result.is_ok(), "{:?}", episode.result);
    assert_eq!(episode.wire.captures, 5);
}


/// The actual second-board acquisition seam exhausts readiness before any D or publication.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn starting_readiness_acquire_exhaustion_cannot_send_stock_or_retry_input() {
    let episode = run(
        Operation::NoChallengeAcquire,
        vec![DEALING.to_vec(); TRANSITION_OBSERVATIONS],
        Revocation::None,
    );
    assert!(
        episode
            .result
            .unwrap_err()
            .contains("did not settle within 6")
    );
    assert_eq!(episode.wire.captures, TRANSITION_OBSERVATIONS);
    assert!(episode.retained_frame);
}


/// STOP and context round trips after capture revoke before waiting or accepting a pair.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn starting_readiness_stop_and_context_revoke_before_another_capture() {
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let episode = run(
            Operation::StartingReadiness,
            vec![DEALING.to_vec(), SETTLED.to_vec(), SETTLED.to_vec()],
            revocation,
        );
        assert!(episode.result.is_err());
        assert_eq!(episode.wire.captures, 1);
        assert_eq!(episode.wire.remaining, 2);
        assert!(!episode.owns_context);
    }
}
