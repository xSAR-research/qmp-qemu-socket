//! Actual native empty-stock delivery through the production route adapter.
//!
//! The saved complete deal is independently solved and replayed before selecting
//! its final King. Captures are retained original pixels except for one explicitly
//! synthetic apex-absent derivative. No acquisition receipt is minted and no live
//! guest, route resumption, REDEAL action or effect acknowledgement is fabricated.

use super::*;


/// Original failure frame: apex King of clubs remains and both piles are cardless.
const EMPTY_STOCK: &[u8] =
    include_bytes!("../../../../tests/fixtures/pyramid-recon/native-v15/stock-empty.png");


/// Exact complete export bound to the retained empty-stock failure packet.
const EMPTY_STOCK_DEAL: &[u8] =
    include_bytes!("../../../../tests/fixtures/pyramid-recon/native-v15/stock-empty-deal.json");


/// Acknowledge one apex click while returning only the ordered fixture captures.
fn serve_stock_empty(
    listener: UnixListener,
    directory: PathBuf,
    frames: Vec<Vec<u8>>,
) -> WireReport {
    let mut frames = std::collections::VecDeque::from(frames);
    let mut stream = accept(&listener);
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("bounded empty-stock QMP read");
    let mut reader = BufReader::new(stream.try_clone().expect("empty-stock QMP reader"));
    packet(
        &mut stream,
        json!({"QMP": {"version": {"qemu": {"major": 11, "minor": 0, "micro": 0}}, "capabilities": []}}),
    );
    let mut report = WireReport {
        connections: 1,
        ..WireReport::default()
    };
    loop {
        let mut line = String::new();
        if reader
            .read_line(&mut line)
            .expect("empty-stock QMP request")
            == 0
        {
            break;
        }
        let request: Value = serde_json::from_str(&line).expect("empty-stock QMP JSON");
        let id = request["id"]
            .as_u64()
            .expect("numeric empty-stock request ID");
        let command = request["execute"].as_str().expect("empty-stock command");
        report.commands.push(command.to_owned());
        let result = match command {
            "qmp_capabilities" => json!({}),
            "query-status" => json!({"running": true, "status": "running"}),
            "query-mice" => {
                json!([{"name": "Fixture absolute tablet", "index": 0, "current": true, "absolute": true}])
            }
            "screendump" => {
                let destination = Path::new(
                    request["arguments"]["filename"]
                        .as_str()
                        .expect("empty-stock capture path"),
                );
                assert_eq!(destination.parent(), Some(directory.as_path()));
                assert_eq!(request["arguments"]["format"], "png");
                fs::write(
                    destination,
                    frames.pop_front().expect("bounded empty-stock captures"),
                )
                .expect("original or labelled derivative PNG bytes");
                json!({})
            }
            "input-send-event" => {
                let events = request["arguments"]["events"]
                    .as_array()
                    .expect("typed empty-stock input")
                    .clone();
                assert!(
                    events.iter().all(|event| event["type"] != "key"),
                    "empty stock never permits a D or REDEAL key operation"
                );
                report.inputs.push((1, events));
                json!({})
            }
            other => panic!("unexpected empty-stock QMP command: {other}"),
        };
        packet(&mut stream, json!({"return": result, "id": id}));
    }
    assert!(
        frames.is_empty(),
        "every queued native observation is consumed"
    );
    report
}


/// Replace the apex envelope with same-frame felt, solely as labelled synthetic clearance.
fn synthetic_apex_absent(stock: &[u8]) -> Vec<u8> {
    let mut frame = crate::capture::decode_png(stock).expect("retained empty-stock PNG");
    let apex = pyramid::PYRAMID_TARGETS
        .iter()
        .find(|slot| slot.kind == PyramidTargetKind::Card { row: 1, column: 1 })
        .expect("fixed apex geometry")
        .bounds;
    for row in 0..apex.height as usize {
        let source = (112 + row) * frame.stride + 400 * 4;
        let destination = (apex.y as usize + row) * frame.stride + apex.x as usize * 4;
        let felt = frame.pixels[source..source + apex.width as usize * 4].to_vec();
        frame.pixels[destination..destination + felt.len()].copy_from_slice(&felt);
    }
    crate::capture::encode_png(&frame).expect("labelled synthetic apex-absent PNG")
}


/// Prove the exact saved deal reaches the retained position before its final apex King.
///
/// Search/rules and the independent explicit-pile replay must agree; the partial
/// state is reached only through legal original-deal actions, never a raw state
/// constructor or a public execution-resumption surface.
pub(super) fn solved_tail(deal: &CompleteDeal) -> (Box<[ReplayStep]>, SearchState, SearchState) {
    let outcome = search::solve(
        deal,
        pyramid_search_policy::limits(),
        &AtomicBool::new(false),
        SearchMethod::AStar,
    );
    let SearchOutcome::Solved { actions, .. } = outcome else {
        panic!("retained complete deal must solve independently: {outcome:?}");
    };
    let trace = replay::replay(deal, &actions, true).expect("independent full-route replay");
    let (last, prefix) = actions.split_last().expect("nonempty complete solution");
    let apex = Location::Tableau(TableauLocation::from_index(0).expect("checked apex index"));
    assert_eq!(*last, RecordedAction::King(apex));
    let mut before = SearchState::initial();
    for action in prefix {
        before = rules::apply(deal, before, *action).expect("legal route prefix");
    }
    let (tableau, left, right) = expected(deal, before).expect("replayed pre-final identities");
    assert_eq!(before.tableau_remaining(), 1);
    assert_eq!(
        tableau[0],
        Some(xsar::games::cards::Card::new(
            xsar::games::cards::Rank::King,
            xsar::games::cards::Suit::Clubs,
        ))
    );
    assert!(tableau.iter().skip(1).all(Option::is_none));
    assert_eq!((left, right), (None, None));
    let after = rules::apply(deal, before, *last).expect("legal final apex King");
    assert!(after.is_goal());
    assert_eq!(
        trace
            .last()
            .expect("final independent replay")
            .remaining_tableau,
        0
    );
    (trace, before, after)
}


/// Existing native-v15 coverage retains the original independently replayed final King.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_empty_stock_tail_delivers_one_apex_king_and_requires_positive_clearance() {
    run_stock_empty_tail(
        EMPTY_STOCK,
        EMPTY_STOCK_DEAL,
        ReconVision::bundled_native_v15().expect("production native-v15"),
        None,
        None,
    );
}


/// Actual 132 zero-deal control layout permits only the replayed apex King, never ACK-only clear.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn execution_observability_native_apex_click_uses_real_position_and_independent_replay() {
    run_stock_empty_tail(
        include_bytes!("../../../../tests/fixtures/pyramid-execution-observability/apex.png"),
        include_bytes!("../../../../tests/fixtures/pyramid-execution-observability/apex-deal.json"),
        ReconVision::bundled_native_v22().expect("current native"),
        Some(
            crate::pyramid_progress::ChallengeProgress::new(
                crate::pyramid_progress::ChallengeKind::Score,
                0,
                2800,
            )
            .unwrap(),
        ),
        Some(BoardCompletion::BoardClear),
    );
}


/// Fresh empty-stock evidence permits only the planned apex click and independently checked clear.
fn run_stock_empty_tail(
    stock: &[u8],
    deal_bytes: &[u8],
    vision: ReconVision,
    challenge: Option<crate::pyramid_progress::ChallengeProgress>,
    completion: Option<BoardCompletion>,
) {
    let _capture_backend = TestCaptureBackend::legacy_qmp();
    let directory = SocketDirectory(std::env::temp_dir().join(format!(
        "computed-route-empty-stock-{}-{}",
        std::process::id(),
        NEXT_SOCKET.fetch_add(1, Ordering::Relaxed),
    )));
    fs::create_dir(&directory.0).expect("exclusive empty-stock fixture directory");
    let context = ControllerContext::new(
        directory.0.join("qmp.sock"),
        GameMode::Pyramid,
        SolvingStrategy::ShortestPath,
        13,
    );
    let cancel = AtomicBool::new(false);
    let (event_tx, _event_rx) = mpsc::channel();
    let events = WorkerEventSink {
        events: event_tx,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(Some(context.clone())),
    };
    let control = &events.latest_frame.reconnaissance;
    control
        .select(&context, &cancel)
        .expect("empty-stock selected context");
    control
        .queue_reconnaissance(&context, &cancel, || Ok(()))
        .expect("private fixture reservation");
    let deal =
        xsar::pyramid_json::read_from(deal_bytes).expect("exact complete original-deal export");
    let (trace, before, after) = solved_tail(&deal);
    let final_action = trace.last().expect("final replayed action").action;
    let final_operations = operations(final_action).expect("checked final King mapping");
    assert_eq!(final_operations.len(), 1);
    let final_operation = final_operations[0];
    let InputOperation::Click(apex_point) = final_operation else {
        panic!("final King is one calibrated click");
    };
    let plan = Prepared {
        id: 115,
        objective: RouteObjective::Clear,
        acquisition: Acquisition {
            challenge_progress: None,
            default_method: None,
            observed_game: crate::pyramid_workflow_model::ObservedGame::Unknown,
            context: context.clone(),
            epoch: control.epoch().expect("current empty-stock epoch"),
            deal,
            bytes: deal_bytes.to_vec(),
            vision,
        },
        trace,
    };
    // Prepare the synthetic PNG before the peer starts its bounded QMP handshake.
    let absent = synthetic_apex_absent(stock);
    let frames = vec![stock.to_vec(), stock.to_vec(), absent.clone(), absent];
    let listener = UnixListener::bind(&context.socket_path).expect("private empty-stock socket");
    listener
        .set_nonblocking(true)
        .expect("bounded local accept");
    let server_directory = directory.0.clone();
    let server = thread::spawn(move || serve_stock_empty(listener, server_directory, frames));
    let qmp = QmpClient::connect(&context.socket_path).expect("actual QMP handshake");
    let gestures = plan
        .trace
        .iter()
        .map(|step| {
            operations(step.action)
                .expect("bounded replayed gesture mapping")
                .len()
        })
        .sum();
    let mut io = NativeIo {
 execution_vision: plan.acquisition.vision.for_temporal_execution(),
 source_transition: None,
            ready_transition: None,
            failure_progress: crate::pyramid_diagnostics::progress::FailureProgress::default(),
 removals: crate::pyramid_recon::vision::removal::RemovalHistory::from_verified_position(
            &expected(&plan.acquisition.deal, before).unwrap().0,
        ),
            stock_effect: None,
            effect_settings: crate::parameters::PyramidEffectSettings::default(),
        policy: ExecutionPolicy::Standard,
        stock_continuation: false,
        executed_recycles: 0,
        qmp,
        plan: &plan,
        context: &context,
        events: &events,
        cancel: &cancel,
        frame_guard: None,
        started: Instant::now(),
        paused: Duration::ZERO,
        captures: 0,
        capture_limit: capture_allowance(plan.trace.len(), gestures)
            .expect("finite route allowance"),
        counts: Counts::default(),
        first_mismatch: None,
        latest_frame: None,
        latest_observation: None,
        verified_actions: 0,
        final_confirmation: FinalConfirmation::default(),
        challenge,
        challenge_initial: challenge,
        challenge_confirmation: None,
        position_matches: false,
        source_proof: None,
        action_permit: None,
        redeal_before: None,
    };
    assert!(
        io.observe(before, false)
            .expect("penultimate effect recognised")
    );
    io.begin_transition(before, after, &[final_operation])
        .expect("exact checked whole King action");
    io.deliver(final_operation, before)
        .expect("fresh guard and sole King click");
    assert_eq!((io.counts.attempted, io.counts.acknowledged), (1, 1));
    assert_eq!(
        io.observe_final(after).expect("unchanged retained frame"),
        None
    );
    assert!(
        io.first_mismatch.is_none(),
        "unchanged source remains pending"
    );
    assert!(io.source_transition.is_some());
    for _ in 0..1 {
        assert_eq!(
            io.observe_final(after)
                .expect("first source-disappearance check"),
            None
        );
    }
    assert_eq!(
        io.observe_final(after)
            .expect("second synthetic source-disappearance check"),
        completion
    );
    assert!(
        io.position_matches,
        "the source effect is verified independently of clear semantics"
    );
    assert_eq!(
        (io.counts.attempted, io.counts.acknowledged, io.captures),
        (1, 1, 4)
    );
    drop(io);
    let wire = server.join().expect("empty-stock local peer completion");
    assert_eq!(wire.connections, 1);
    assert_eq!(transitions(&wire), [(1, true), (1, false)]);
    assert_eq!(
        wire.inputs.len(),
        3,
        "one motion and one separately acknowledged DOWN/UP"
    );
    let mapped = pixel_point_to_qmp(apex_point, 1920, 1080).expect("checked apex mapping");
    assert_eq!(
        wire.inputs[0].1,
        [
            json!({"type": "abs", "data": {"axis": "x", "value": mapped.x}}),
            json!({"type": "abs", "data": {"axis": "y", "value": mapped.y}}),
        ]
    );
    assert_eq!(
        wire.commands
            .iter()
            .filter(|name| name.as_str() == "screendump")
            .count(),
        4
    );
    assert_eq!(
        wire.commands
            .iter()
            .filter(|name| name.as_str() == "query-mice")
            .count(),
        1
    );
}
