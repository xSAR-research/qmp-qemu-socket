//! Native exhausted-stock adapter checks using real pixels and local QMP only.

use super::*;
use xsar::games::cards::{Card, Rank, Suit};


/// Exact operator frame; stock ordering below is explicitly synthetic except its top.
const EXHAUSTED: &[u8] =
    include_bytes!("../../../../tests/fixtures/pyramid-recon/native-redeal/exhausted.png");


/// Decode direct human labels independently of the recogniser.
fn card(label: &str) -> Card {
    let (rank, suit) = label.split_at(label.len() - 1);
    let rank = Rank::ALL
        .into_iter()
        .find(|value| value.as_str() == rank)
        .unwrap();
    let suit = match suit {
        "C" => Suit::Clubs,
        "D" => Suit::Diamonds,
        "H" => Suit::Hearts,
        "S" => Suit::Spades,
        _ => panic!("direct suit"),
    };
    Card::new(rank, suit)
}


/// Build a complete synthetic stock with the actual observed last RIGHT card.
fn deal() -> CompleteDeal {
    let tableau = [
        "3C", "5D", "KH", "4H", "7S", "4D", "2H", "9C", "9D", "7D", "8C", "JS", "10D", "4S", "AH",
        "3D", "10S", "4C", "7H", "QD", "6C", "AD", "KS", "5H", "3S", "KD", "JD", "KC",
    ]
    .map(card);
    let last = card("8S");
    let mut stock: Vec<Card> = Suit::ALL
        .into_iter()
        .flat_map(|suit| Rank::ALL.map(|rank| Card::new(rank, suit)))
        .filter(|candidate| !tableau.contains(candidate) && *candidate != last)
        .collect();
    stock.push(last);
    let metadata = xsar::games::pyramid::DealMetadata::try_new(
        "redeal-wire",
        xsar::games::pyramid::ProvenanceKind::SyntheticFixture,
        xsar::games::pyramid::Restoration::NotApplicable,
        "test",
        "Native tableau with synthetic stock order",
        None,
    )
    .unwrap();
    CompleteDeal::try_new(tableau, stock.try_into().unwrap(), metadata).unwrap()
}


/// Exercise the production drain adapter with zero remaining D or an invalid origin.
fn run(exhausted: bool, scenario: Scenario) -> (Result<DriveReport, String>, Counts, WireReport) {
    run_position(exhausted, false, scenario)
}


/// Compose an explicitly synthetic stock boundary and exercise the production adapter.
fn run_position(
    exhausted: bool,
    boundary: bool,
    scenario: Scenario,
) -> (Result<DriveReport, String>, Counts, WireReport) {
    run_control(exhausted, boundary, None, scenario)
}


/// Exercise actual control rasters on an explicitly synthetic rules-derived stock position.
fn run_control(
    exhausted: bool,
    boundary: bool,
    challenge: Option<&str>,
    scenario: Scenario,
) -> (Result<DriveReport, String>, Counts, WireReport) {
    let _capture_backend = TestCaptureBackend::legacy_qmp();
    let directory = SocketDirectory(std::env::temp_dir().join(format!(
        "redeal-wire-{}-{}",
        std::process::id(),
        NEXT_SOCKET.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&directory.0).unwrap();
    let socket = directory.0.join("qmp.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let context =
        ControllerContext::new(socket, GameMode::Pyramid, SolvingStrategy::ShortestPath, 3);
    let cancel = Arc::new(AtomicBool::new(false));
    let (event_tx, _event_rx) = mpsc::channel();
    let events = WorkerEventSink {
        events: event_tx,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(context.clone())),
    };
    let control = &events.latest_frame.reconnaissance;
    control.select(&context, &cancel).unwrap();
    control
        .queue_reconnaissance(&context, &cancel, || Ok(()))
        .unwrap();
    let plan = Prepared {
        id: 17,
        objective: RouteObjective::Drain,
        trace: Vec::new().into_boxed_slice(),
        acquisition: Acquisition {
            challenge_progress: None,
            default_method: RouteMethod::FewestActions,
            observed_game: crate::pyramid_workflow_model::ObservedGame::Unknown,
            context: context.clone(),
            epoch: control.epoch().unwrap(),
            bytes: Vec::new(),
            deal: deal(),
            vision: ReconVision::bundled_native_v22().unwrap(),
        },
    };
    let state = if boundary {
        continuation::StockDrain::new(&plan.acquisition.deal, SearchState::initial(), 0)
            .unwrap()
            .steps
            .into_iter()
            .find(|step| step.after.recycles() == 1 && step.after.stock_cursor() == 24)
            .unwrap()
            .after
    } else if exhausted {
        continuation::StockDrain::new(&plan.acquisition.deal, SearchState::initial(), 0)
            .unwrap()
            .exhausted
    } else {
        SearchState::initial()
    };
    let peer_directory = directory.0.clone();
    let frame_bytes = if let Some(name) = challenge {
        let mut frame = crate::capture::decode_png(EXHAUSTED).unwrap();
        let source = fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-challenge-redeal")
                .join(format!("{name}.png")),
        )
        .unwrap();
        let actual = crate::capture::decode_png(&source).unwrap();
        // These two complete actual control crops on the older native tableau
        // are a synthetic adapter fixture, never an observed gameplay sequence.
        for (x, y, width, height) in [(936, 792, 48, 63), (890, 874, 140, 26)] {
            for row in y..y + height {
                let begin = row * frame.stride + x * 4;
                let end = begin + width * 4;
                frame.pixels[begin..end].copy_from_slice(&actual.pixels[begin..end]);
            }
        }
        crate::capture::encode_png(&frame).unwrap()
    } else if boundary {
        let mut frame = crate::capture::decode_png(EXHAUSTED).unwrap();
        let stock = crate::capture::decode_png(include_bytes!(
            "../../../../tests/fixtures/pyramid-stock-recycle/last-observed.png"
        ))
        .unwrap();
        // Actual stock pixels, older untouched tableau and synthetic stock order:
        // this tests the adapter seam, never an observed gameplay sequence.
        for y in 640..908 {
            let begin = y * frame.stride + 714 * 4;
            let end = y * frame.stride + 1206 * 4;
            frame.pixels[begin..end].copy_from_slice(&stock.pixels[begin..end]);
        }
        crate::capture::encode_png(&frame).unwrap()
    } else {
        EXHAUSTED.to_vec()
    };
    let peer_context = context.clone();
    let peer_cancel = cancel.clone();
    let peer_control = control.clone();
    let server = thread::spawn(move || {
        serve(
            listener,
            peer_directory,
            scenario,
            peer_control,
            peer_context,
            peer_cancel,
            &frame_bytes,
        )
    });
    let qmp = QmpClient::connect(&context.socket_path).unwrap();
    let mut io = NativeIo {
        policy: ExecutionPolicy::NoChallenge,
        stock_continuation: false,
        executed_recycles: state.recycles(),
        qmp,
        plan: &plan,
        context: &context,
        events: &events,
        cancel: &cancel,
        started: Instant::now(),
        captures: 0,
        capture_limit: 1200,
        counts: Counts::default(),
        first_mismatch: None,
        latest_frame: None,
        verified_actions: 0,
        final_confirmation: FinalConfirmation::default(),
        challenge: None,
        challenge_initial: None,
        challenge_confirmation: None,
        position_matches: false,
        frame_guard: None,
    };
    let result = continuation::drain_and_redeal(&mut io, state);
    let counts = io.counts;
    drop(io);
    (result, counts, server.join().unwrap())
}


/// Exhausted, independently advanced stock permits exactly one click, never another D.
#[test]
fn native_redeal_delivers_once_and_stop_after_probe_retires_the_click() {
    let (result, counts, wire) = run(true, Scenario::Acknowledged);
    assert_eq!(result.unwrap().completion, BoardCompletion::StockRedealt);
    assert_eq!((counts.attempted, counts.acknowledged), (1, 1));
    assert_eq!(transitions(&wire), [(1, true), (1, false)]);
    assert!(
        wire.inputs
            .iter()
            .all(|(_, events)| events.iter().all(|event| event["type"] != "key"))
    );
    let (result, counts, wire) = run(true, Scenario::StopAfterProbe);
    assert!(result.is_err());
    assert_eq!((counts.attempted, counts.acknowledged), (0, 0));
    assert!(wire.inputs.is_empty());
}


/// A zero-action Drain cannot pretend an exhausted old frame is its untouched origin.
#[test]
fn zero_action_drain_refuses_wrong_origin_before_any_draw_or_redeal() {
    let (result, counts, wire) = run(false, Scenario::Acknowledged);
    assert!(result.is_err());
    assert_eq!((counts.attempted, counts.acknowledged), (0, 0));
    assert!(wire.inputs.is_empty());
}


/// A legal intermediate boundary delivers one RECYCLE, then refuses its stale effect.
#[test]
fn native_stock_boundary_recycles_once_and_never_retries_an_unverified_effect() {
    let (result, counts, wire) = run_position(false, true, Scenario::Acknowledged);
    assert!(
        result
            .unwrap_err()
            .contains("Stock effect could not be verified")
    );
    assert_eq!((counts.attempted, counts.acknowledged), (1, 1));
    let input: Vec<_> = wire.inputs.iter().flat_map(|(_, events)| events).collect();
    assert_eq!(input.len(), 2);
    assert!(
        input
            .iter()
            .all(|event| event["type"] == "key" && event["data"]["key"]["data"] == "d")
    );
    assert_eq!(
        transitions(&wire)
            .iter()
            .map(|(_, down)| *down)
            .collect::<Vec<_>>(),
        vec![true, false]
    );
    let (result, counts, wire) = run_position(false, true, Scenario::StopAfterProbe);
    assert!(result.is_err());
    assert_eq!((counts.attempted, counts.acknowledged), (0, 0));
    assert!(wire.inputs.is_empty());
}


/// Both actual control variants reach the one-click adapter, with post-probe STOP preserved.
#[test]
fn actual_challenge_controls_deliver_once_and_stop_before_input() {
    for name in ["jacks", "eights"] {
        let (result, counts, wire) = run_control(true, false, Some(name), Scenario::Acknowledged);
        assert_eq!(result.unwrap().completion, BoardCompletion::StockRedealt);
        assert_eq!((counts.attempted, counts.acknowledged), (1, 1));
        assert_eq!(transitions(&wire), [(1, true), (1, false)]);
        assert!(
            wire.inputs
                .iter()
                .all(|(_, events)| events.iter().all(|event| event["type"] != "key"))
        );
        let (result, counts, wire) = run_control(true, false, Some(name), Scenario::StopAfterProbe);
        assert!(result.is_err());
        assert_eq!((counts.attempted, counts.acknowledged), (0, 0));
        assert!(wire.inputs.is_empty());
    }
}
