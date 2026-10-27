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


/// Replay a labelled reconstructed route without inventing a maximum-removal proof.
fn retained_partial(plan: &mut Prepared) -> SearchState {
    let bytes =
        include_bytes!("../../../../tests/fixtures/pyramid-recon/native-partial-stock/deal.json");
    let deal = xsar::pyramid_json::read_from(bytes.as_slice()).unwrap();
    let csv = include_str!(
        "../../../../tests/fixtures/pyramid-recon/native-partial-stock/reconstructed-route.csv"
    );
    let mut lines = csv.lines();
    assert_eq!(lines.next(), Some("SEQUENCE,ACTION,CARD1,CARD2"));
    let actions: Vec<_> = lines
        .enumerate()
        .map(|(index, line)| {
            let fields: Vec<_> = line.split(',').collect();
            assert_eq!(fields.len(), 4);
            assert_eq!(fields[0].parse::<usize>().unwrap(), index + 1);
            match fields[1] {
                "MATCH" => {
                    RecordedAction::Match(fixed_location(fields[2]), fixed_location(fields[3]))
                }
                "KING" => RecordedAction::King(fixed_location(fields[2])),
                "MOVE" => RecordedAction::Move,
                other => panic!("unexpected reconstructed action {other}"),
            }
        })
        .collect();
    assert_eq!(actions.len(), 30);
    assert_eq!(
        actions
            .iter()
            .filter(|action| **action == RecordedAction::Move)
            .count(),
        7
    );
    let mut state = SearchState::initial();
    for action in &actions {
        state = rules::apply(&deal, state, *action).expect("legal reconstructed action");
    }
    let (tableau, left, right) = expected(&deal, state).unwrap();
    let mut observed = [None; TABLEAU_LEN];
    for (index, label) in [
        (0, "7S"),
        (1, "4C"),
        (2, "6D"),
        (4, "KS"),
        (5, "4D"),
        (8, "9C"),
    ] {
        observed[index] = Some(card(label));
    }
    assert_eq!(tableau, observed);
    assert_eq!((left, right, state.recycles()), (None, Some(card("JS")), 0));
    plan.trace = replay::replay(&deal, &actions, false).expect("independent reconstructed replay");
    plan.objective = RouteObjective::Removal {
        removed: 22,
        proof: PartialProof::Bounded(xsar::search::SearchStop::StateLimit),
    };
    plan.acquisition.bytes = bytes.to_vec();
    plan.acquisition.deal = deal;
    state
}


/// Decode original one-based tableau locations at the checked xSAR API boundary.
fn fixed_location(text: &str) -> Location {
    match text {
        "DL" => Location::DL,
        "DR" => Location::DR,
        _ => {
            let (row, column) = text.split_once('|').unwrap();
            xsar::games::pyramid::planning::location_from_row_column(
                row.parse().unwrap(),
                column.parse().unwrap(),
            )
            .unwrap()
        }
    }
}


/// Exercise actual control rasters on an explicitly synthetic rules-derived stock position.
fn run_control(
    exhausted: bool,
    boundary: bool,
    challenge: Option<&str>,
    scenario: Scenario,
) -> (Result<DriveReport, String>, Counts, WireReport) {
    run_fixture(exhausted, boundary, challenge, scenario, None, false)
}


/// Exercise either synthetic stock controls or the exact retained partial endpoint.
fn run_fixture(
    exhausted: bool,
    boundary: bool,
    challenge: Option<&str>,
    scenario: Scenario,
    partial_policy: Option<ExecutionPolicy>,
    initial_draw: bool,
) -> (Result<DriveReport, String>, Counts, WireReport) {
    run_fixture_with_appearance(
        exhausted,
        boundary,
        challenge,
        scenario,
        partial_policy,
        initial_draw,
        0,
    )
}


/// Supply explicitly synthetic temporal origins: absent, missing, unchanged or numeral-only.
fn run_fixture_with_appearance(
    exhausted: bool,
    boundary: bool,
    challenge: Option<&str>,
    scenario: Scenario,
    partial_policy: Option<ExecutionPolicy>,
    initial_draw: bool,
    appearance: u8,
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
    let (event_tx, event_rx) = mpsc::channel();
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
    let mut plan = Prepared {
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
    let empty_circle = challenge == Some("empty-circle");
    let textured_right = challenge == Some("textured-right");
    if textured_right {
        plan.acquisition.vision = ReconVision::bundled_native_v23().unwrap();
    }
    let state = if empty_circle {
        // Legal, independently solved/replayed older deal; this is a synthetic
        // non-maximal endpoint, never the new packet's missing original route.
        let bytes = include_bytes!(
            "../../../../tests/fixtures/pyramid-execution-observability/apex-deal.json"
        );
        let deal = xsar::pyramid_json::read_from(bytes.as_slice()).unwrap();
        let (full, before, _) = super::stock_empty_tests::solved_tail(&deal);
        plan.trace = full[..full.len() - 1].to_vec().into_boxed_slice();
        // Test-only bounded planner receipt supplies the guarded adapter contract;
        // it is not evidence that this deliberately truncated route is maximum.
        plan.objective = RouteObjective::Removal {
            removed: 27,
            proof: PartialProof::Bounded(xsar::search::SearchStop::StateLimit),
        };
        plan.acquisition.bytes = bytes.to_vec();
        plan.acquisition.deal = deal;
        plan.acquisition.vision = ReconVision::bundled_native_v23().unwrap();
        before
    } else if initial_draw {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../xsar/tests/fixtures/pyramid-planning/pyramid-recon-20261019-v7-01.json");
        plan.acquisition.deal =
            xsar::pyramid_json::read_from(fs::File::open(path).unwrap()).unwrap();
        plan.acquisition.vision = ReconVision::load(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-recon/native-v7/profile.json"),
        )
        .unwrap();
        SearchState::initial()
    } else if partial_policy.is_some() {
        retained_partial(&mut plan)
    } else if boundary {
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
    let frame_bytes = if textured_right {
        let mut frame = crate::capture::decode_png(EXHAUSTED).unwrap();
        let actual = crate::capture::decode_png(include_bytes!(
            "../../../../tests/fixtures/pyramid-temporal-right-144/last-observed.png"
        ))
        .unwrap();
        // Synthetic adapter scene: retain the older tableau and RIGHT8spades.
        // Only the unchanged actual LEFT/control area is substituted; the stock
        // state is independently rules-derived, not the missing run9 trace.
        for y in 640..908 {
            let begin = y * frame.stride + 714 * 4;
            let end = y * frame.stride + 1020 * 4;
            frame.pixels[begin..end].copy_from_slice(&actual.pixels[begin..end]);
        }
        crate::capture::encode_png(&frame).unwrap()
    } else if empty_circle {
        let mut frame = crate::capture::decode_png(include_bytes!(
            "../../../../tests/fixtures/pyramid-recon/native-v15/stock-empty.png"
        ))
        .unwrap();
        let actual = crate::capture::decode_png(include_bytes!(
            "../../../../tests/fixtures/pyramid-empty-circle-redeal/last-observed.png"
        ))
        .unwrap();
        // Synthetic substitution of the complete actual stock panel only. The
        // older saved tableau/header and rules-derived state remain unchanged.
        for y in 640..908 {
            let begin = y * frame.stride + 714 * 4;
            let end = y * frame.stride + 1206 * 4;
            frame.pixels[begin..end].copy_from_slice(&actual.pixels[begin..end]);
        }
        crate::capture::encode_png(&frame).unwrap()
    } else if initial_draw {
        INITIAL_PNG.to_vec()
    } else if partial_policy.is_some() {
        include_bytes!(
            "../../../../tests/fixtures/pyramid-recon/native-partial-stock/last-observed.png"
        )
        .to_vec()
    } else if let Some(name) = challenge {
        let mut frame = crate::capture::decode_png(EXHAUSTED).unwrap();
        let source = if name == "two" {
            include_bytes!("../../../../tests/fixtures/pyramid-redeal-appearance/count-two.png")
                .to_vec()
        } else {
            fs::read(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/pyramid-challenge-redeal")
                    .join(format!("{name}.png")),
            )
            .unwrap()
        };
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
    // The legal endpoint/ACK harness intentionally omits its earlier execution.
    // This explicit synthetic origin represents the pre-input source, not a newly
    // captured baseline after those omitted gestures.
    let temporal_before = if exhausted || empty_circle {
        let bytes: &[u8] = match appearance {
            0 => {
                include_bytes!("../../../../tests/fixtures/pyramid-recon/native-redeal/before.png")
            }
            1 => &[],
            2 => &frame_bytes,
            3 => EXHAUSTED,
            _ => panic!("unsupported synthetic appearance origin"),
        };
        if bytes.is_empty() {
            None
        } else {
            Some(
                redeal_appearance::Before::capture(&crate::capture::decode_png(bytes).unwrap())
                    .unwrap(),
            )
        }
    } else {
        None
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
        policy: partial_policy.unwrap_or(ExecutionPolicy::NoChallenge),
        stock_continuation: false,
        executed_recycles: state.recycles(),
        qmp,
        plan: &plan,
        context: &context,
        events: &events,
        cancel: &cancel,
        started: Instant::now(),
        paused: Duration::ZERO,
        captures: 0,
        capture_limit: 1200,
        counts: Counts::default(),
        first_mismatch: None,
        latest_frame: None,
        latest_observation: None,
        verified_actions: 0,
        final_confirmation: FinalConfirmation::default(),
        challenge: None,
        challenge_initial: None,
        challenge_confirmation: None,
        position_matches: false,
        source_proof: None,
        action_permit: None,
        redeal_before: temporal_before,
        frame_guard: None,
    };
    if appearance == 1 {
        // Synthetic acknowledged history proves a missing pre-input baseline is
        // never minted from this exhausted post-input frame.
        io.counts.attempted = 1;
        io.counts.acknowledged = 1;
    }
    let result = if partial_policy.is_some() {
        // Synthetic final whole-action receipt exercises the production confirmation
        // seam; none of this independently replayed prefix was sent to a guest.
        let actions = plan.trace.len();
        let gestures = plan
            .trace
            .iter()
            .map(|step| operations(step.action).unwrap().len())
            .sum();
        let removed = u8::try_from(TABLEAU_LEN as u32 - state.tableau_remaining()).unwrap();
        let acknowledged_at = io.elapsed();
        io.counts = Counts {
            attempted: gestures,
            acknowledged: gestures,
            last_acknowledged_at: Some(acknowledged_at),
        };
        io.verified_actions = actions - 1;
        let confirmed = partial::confirm_partial_action(
            state,
            &mut io,
            actions,
            gestures,
            acknowledged_at,
            removed,
        );
        assert!(
            !io.stock_continuation,
            "partial-only recognition scope is restored"
        );
        match confirmed {
            Ok(DriveReport {
                completion:
                    BoardCompletion::PartialPositionVerified {
                        removed: observed_removed,
                        remaining,
                    },
                ..
            }) if observed_removed == removed
                && u32::from(remaining) == state.tableau_remaining() =>
            {
                continuation::drain_and_redeal(&mut io, state)
            }
            Ok(report) => Err(format!(
                "Exact retained partial endpoint refused: {}",
                report.message
            )),
            Err(error) => Err(error),
        }
    } else {
        continuation::drain_and_redeal(&mut io, state)
    };
    let counts = io.counts;
    drop(io);
    let mut wire = server.join().unwrap();
    wire.logs = event_rx
        .try_iter()
        .filter_map(|event| match event.event {
            WorkerEvent::Log(message) => Some(message),
            _ => None,
        })
        .collect();
    (result, counts, wire)
}


/// The reported six-card endpoint reaches one guarded RECYCLE and refuses stale effects.
#[test]
fn retained_partial_endpoint_enters_stock_drain_without_replaying_the_last_match() {
    for policy in [ExecutionPolicy::Continuous, ExecutionPolicy::NoChallenge] {
        let (result, counts, wire) = run_fixture(
            false,
            false,
            None,
            Scenario::Acknowledged,
            Some(policy),
            false,
        );
        assert!(
            result
                .unwrap_err()
                .contains("Stock effect could not be verified")
        );
        assert_eq!((counts.attempted, counts.acknowledged), (51, 51));
        assert!(
            wire.logs
                .iter()
                .any(|line| line.starts_with("Computed-route wait: configured=500.000 ms")),
            "stock RECYCLE preserves its 500 ms wait"
        );
        assert_eq!(transitions(&wire), [(1, true), (1, false)]);
        assert!(
            wire.inputs
                .iter()
                .flat_map(|(_, events)| events)
                .all(|event| event["type"] == "key" && event["data"]["key"]["data"] == "d")
        );
    }
}


/// Standard controls retain their ordinary proof, and fresh STOP revokes continuation.
#[test]
fn retained_partial_endpoint_preserves_standard_and_stop_boundaries() {
    for (policy, scenario) in [
        (ExecutionPolicy::Standard, Scenario::Acknowledged),
        (ExecutionPolicy::Continuous, Scenario::StopAfterProbe),
    ] {
        let (result, counts, wire) = run_fixture(false, false, None, scenario, Some(policy), false);
        assert!(result.is_err());
        assert_eq!((counts.attempted, counts.acknowledged), (50, 50));
        assert!(wire.inputs.is_empty());
    }
}


/// Residual stock MOVE uses the same short wait and never retries an unchanged effect.
#[test]
fn native_stock_drain_move_requests_250ms_before_its_effect_capture() {
    let (result, counts, wire) =
        run_fixture(false, false, None, Scenario::Acknowledged, None, true);
    assert!(
        result
            .unwrap_err()
            .contains("Stock effect could not be verified")
    );
    assert_eq!((counts.attempted, counts.acknowledged), (1, 1));
    assert_eq!(transitions(&wire), [(1, true), (1, false)]);
    assert!(
        wire.logs
            .iter()
            .any(|line| line.starts_with("Computed-route wait: configured=250.000 ms"))
    );
    assert!(
        !wire
            .logs
            .iter()
            .any(|line| line.starts_with("Computed-route wait: configured=500.000 ms"))
    );
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
    for name in ["jacks", "eights", "score", "two"] {
        let (result, counts, wire) = run_control(true, false, Some(name), Scenario::Acknowledged);
        assert_eq!(
            result
                .unwrap_or_else(|error| panic!(
                    "{name}: {error}; {:?}",
                    wire.logs
                        .iter()
                        .filter(|line| line.contains("unmatched"))
                        .collect::<Vec<_>>()
                ))
                .completion,
            BoardCompletion::StockRedealt
        );
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


/// A legally replayed synthetic partial endpoint crosses circle proof into one REDEAL.
/// The original new two-card frame is tested separately without any pixel substitution.
#[test]
fn native_empty_circle_partial_endpoint_delivers_one_redeal_and_preserves_stop() {
    for scenario in [Scenario::Acknowledged, Scenario::StopAfterProbe] {
        let (result, counts, wire) = run_fixture(
            true,
            false,
            Some("empty-circle"),
            scenario,
            Some(ExecutionPolicy::NoChallenge),
            false,
        );
        if matches!(scenario, Scenario::Acknowledged) {
            assert_eq!(result.unwrap().completion, BoardCompletion::StockRedealt);
            assert_eq!(counts.attempted, counts.acknowledged);
            assert_eq!(transitions(&wire), [(1, true), (1, false)]);
        } else {
            assert!(result.is_err());
            assert!(wire.inputs.is_empty());
        }
        assert_eq!(
            wire.commands
                .iter()
                .filter(|command| *command == "screendump")
                .count(),
            1
        );
        assert_eq!(
            wire.commands
                .iter()
                .filter(|command| *command == "query-mice")
                .count(),
            1
        );
        assert!(
            wire.inputs
                .iter()
                .all(|(_, events)| events.iter().all(|event| event["type"] != "key"))
        );
    }
}


/// Missing, unchanged and numeral-only temporal evidence never authorises a click.
#[test]
fn temporal_redeal_refuses_missing_unchanged_and_numeral_only_origins() {
    for appearance in [1, 2, 3] {
        let (result, counts, wire) = run_fixture_with_appearance(
            true,
            false,
            Some("two"),
            Scenario::Acknowledged,
            None,
            false,
            appearance,
        );
        if appearance == 1 {
            assert!(
                result.is_err(),
                "missing pre-input origin remains a failure"
            );
        } else {
            let report = result.unwrap();
            assert_eq!(
                report.completion,
                BoardCompletion::PartialPositionVerified {
                    removed: 0,
                    remaining: 28
                }
            );
            assert!(
                report
                    .message
                    .contains("Challenge completion remains unproved")
            );
        }
        assert_eq!(counts.attempted, usize::from(appearance == 1));
        assert_eq!(counts.attempted, counts.acknowledged);
        assert!(wire.inputs.is_empty());
    }
}


/// An occupied RIGHT does not block temporal appearance; STOP and absent proof still win.
#[test]
fn native_temporal_right_redeal_delivers_once_and_rejects_missing_or_unchanged_origin() {
    for (scenario, appearance) in [
        (Scenario::Acknowledged, 0),
        (Scenario::StopAfterProbe, 0),
        (Scenario::Acknowledged, 1),
        (Scenario::Acknowledged, 2),
    ] {
        let (result, counts, wire) = run_fixture_with_appearance(
            true,
            false,
            Some("textured-right"),
            scenario,
            None,
            false,
            appearance,
        );
        if matches!(scenario, Scenario::Acknowledged) && appearance == 0 {
            assert_eq!(
                result
                    .unwrap_or_else(|error| panic!("{error}; observations: {:?}", wire.logs))
                    .completion,
                BoardCompletion::StockRedealt
            );
            assert_eq!((counts.attempted, counts.acknowledged), (1, 1));
            assert_eq!(transitions(&wire), [(1, true), (1, false)]);
            assert!(
                wire.logs
                    .iter()
                    .any(|line| line.starts_with("REDEAL region appearance:")
                        && line.contains("appeared=true"))
            );
        } else {
            if matches!(scenario, Scenario::Acknowledged) && appearance == 2 {
                // This textured layout has no independent exhausted-position
                // proof when temporal appearance is absent. It cannot claim a
                // verified partial endpoint or authorise a REDEAL click.
                assert_eq!(
                    result.expect_err("unknown exhausted position"),
                    "Stock effect could not be verified; input is not repeated"
                );
            } else {
                assert!(result.is_err());
            }
            assert_eq!(counts.attempted, usize::from(appearance == 1));
            assert_eq!(counts.attempted, counts.acknowledged);
            assert!(wire.inputs.is_empty());
        }
        assert!(
            wire.inputs
                .iter()
                .all(|(_, events)| events.iter().all(|event| event["type"] != "key"))
        );
        if appearance == 0 {
            assert_eq!(
                wire.commands.iter().filter(|s| *s == "screendump").count(),
                1
            );
            assert_eq!(
                wire.commands.iter().filter(|s| *s == "query-mice").count(),
                1
            );
        }
    }
}
