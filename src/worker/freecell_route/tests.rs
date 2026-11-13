//! Socket-free ownership and cancellation boundaries using a real verified solver result.
use super::*;
use xsar::games::{
    cards::{Card, Rank},
    freecell::{
        BoardState, CascadeStack, FOUNDATION_SUITS, Metadata,
        search::solve_best_effort_with_control,
    },
};


/// Every upper slot preserves its proved pickup anchor, including the allowed x recrop.
#[test]
fn freecell_3119_abs_top_targets_preserve_pickup_anchors_for_occupied_and_empty_slots() {
    let source = PixelPoint::new(233, 671);
    for (target, x, anchor_x, midpoint_x) in [
        (Pile::Cell(0), 166, 181, 207),
        (Pile::Cell(1), 358, 373, 303),
        (Pile::Cell(2), 551, 566, 399),
        (Pile::Cell(3), 743, 758, 495),
        (Pile::Foundation(0), 1233, 1248, 740),
        (Pile::Foundation(1), 1425, 1440, 836),
        (Pile::Foundation(2), 1041, 1056, 644),
        (Pile::Foundation(3), 1618, 1633, 933),
    ] {
        let anchor = PixelPoint::new(anchor_x, 132);
        let expected = [
            PixelPoint::new(233, 626),
            PixelPoint::new(midpoint_x, 401),
            anchor,
        ];
        let slot = crate::geometry::PixelRect::new(x, 112, 137, 182);
        let path = diagnostic_abs_path(source, anchor, target, (1920, 1080)).unwrap();
        assert_eq!(path, expected);
        assert!(slot.contains(path[2]));
        assert!(!slot.contains(path[0]) && !slot.contains(path[1]));
        let recrop = PixelPoint::new(anchor_x - 1, 132);
        let recrop_path = diagnostic_abs_path(source, recrop, target, (1920, 1080)).unwrap();
        assert_eq!(recrop_path[2], recrop);
        assert!(slot.contains(recrop_path[2]));
    }
}


/// PLAY keeps its observed target; midpoint arithmetic uses the original native endpoints.
#[test]
fn freecell_3116_abs_play_target_keeps_observed_point_and_exact_lift_midpoint_path() {
    let source = PixelPoint::new(233, 671);
    let destination = PixelPoint::new(1195, 618);
    assert_eq!(
        diagnostic_abs_path(source, destination, Pile::Cascade(5), (1920, 1080)).unwrap(),
        [
            PixelPoint::new(233, 626),
            PixelPoint::new(714, 644),
            destination,
        ]
    );
}


/// Odd native sums round down before QMP conversion, independently on each axis.
#[test]
fn freecell_3116_abs_midpoint_rounds_odd_native_sums_down() {
    assert_eq!(
        diagnostic_abs_path(
            PixelPoint::new(234, 671),
            PixelPoint::new(1195, 618),
            Pile::Cascade(5),
            (1920, 1080),
        )
        .unwrap(),
        [
            PixelPoint::new(234, 626),
            PixelPoint::new(714, 644),
            PixelPoint::new(1195, 618),
        ]
    );
}


/// Widened native midpoint sums preserve valid endpoints beyond the i32 addition bound.
#[test]
fn freecell_3116_abs_midpoint_avoids_native_endpoint_sum_overflow() {
    let source = PixelPoint::new(i32::MAX - 2, i32::MAX - 2);
    let destination = PixelPoint::new(i32::MAX, i32::MAX);
    assert_eq!(
        diagnostic_abs_path(source, destination, Pile::Cascade(0), (u32::MAX, u32::MAX)).unwrap(),
        [
            PixelPoint::new(i32::MAX - 2, i32::MAX - 47),
            PixelPoint::new(i32::MAX - 1, i32::MAX - 1),
            destination,
        ]
    );
}


/// Invalid native source, observed target, generated point, frame or pile refuses planning.
#[test]
fn freecell_3116_abs_path_rejects_invalid_bounds_and_piles_without_fallback() {
    let source = PixelPoint::new(233, 671);
    let destination = PixelPoint::new(1195, 618);
    for (from, to, pile, frame) in [
        (
            PixelPoint::new(-1, 671),
            destination,
            Pile::Cascade(5),
            (1920, 1080),
        ),
        (
            source,
            PixelPoint::new(1920, 618),
            Pile::Cascade(5),
            (1920, 1080),
        ),
        (
            source,
            PixelPoint::new(1195, 1080),
            Pile::Cascade(5),
            (1920, 1080),
        ),
        (
            PixelPoint::new(233, 44),
            destination,
            Pile::Cascade(5),
            (1920, 1080),
        ),
        (
            PixelPoint::new(233, i32::MIN),
            destination,
            Pile::Cascade(5),
            (1920, 1080),
        ),
        (
            source,
            PixelPoint::new(0, -1),
            Pile::Cascade(5),
            (1920, 1080),
        ),
        (
            source,
            PixelPoint::new(i32::MIN, i32::MIN),
            Pile::Cascade(5),
            (1920, 1080),
        ),
        (source, destination, Pile::Cell(4), (1920, 1080)),
        (source, destination, Pile::Foundation(4), (1920, 1080)),
        (source, destination, Pile::Cascade(8), (1920, 1080)),
        (source, destination, Pile::Cell(0), (0, 1080)),
        (source, destination, Pile::Cell(0), (1920, 0)),
        (
            PixelPoint::new(10, 10),
            PixelPoint::new(20, 20),
            Pile::Cell(0),
            (234, 203),
        ),
    ] {
        assert!(diagnostic_abs_path(from, to, pile, frame).is_err());
    }
    assert_eq!(
        diagnostic_abs_path(
            source,
            PixelPoint::new(0, 45),
            Pile::Cascade(0),
            (1920, 1080)
        )
        .unwrap(),
        [
            PixelPoint::new(233, 626),
            PixelPoint::new(116, 358),
            PixelPoint::new(0, 45),
        ]
    );
    assert_eq!(
        diagnostic_abs_path(
            PixelPoint::new(0, 45),
            PixelPoint::new(0, 45),
            Pile::Cascade(0),
            (1920, 1080),
        )
        .unwrap(),
        [
            PixelPoint::new(0, 0),
            PixelPoint::new(0, 45),
            PixelPoint::new(0, 45),
        ]
    );
    assert_eq!(
        diagnostic_abs_path(
            source,
            PixelPoint::new(0, 0),
            Pile::Cascade(0),
            (1920, 1080)
        )
        .unwrap(),
        [
            PixelPoint::new(233, 626),
            PixelPoint::new(116, 335),
            PixelPoint::new(0, 0),
        ]
    );
}


/// A single-owner playback queues once and refuses another run until completion.
#[test]
fn freecell_317_playback_queues_once_without_starting_a_worker() {
    let (command_tx, command_rx) = mpsc::channel();
    let (_, event_rx) = mpsc::channel();
    let latest_frame = LatestFrameSlot::default();
    let identity = plan(317);
    latest_frame
        .freecell_route
        .lock()
        .unwrap()
        .prepare(identity.clone(), report())
        .unwrap();
    let handle = WorkerHandle {
        command_tx,
        event_rx,
        latest_frame,
        join: None,
        cancel_requested: Arc::new(AtomicBool::new(false)),
        shutdown_requested: Arc::new(AtomicBool::new(false)),
    };
    let settle = Duration::from_millis(317);
    let run_id = handle
        .execute_freecell_route(identity.clone(), true, route::AutoMode::ObserveGame, settle)
        .unwrap();
    match command_rx.try_recv().unwrap() {
        WorkerCommand::ExecuteFreeCellRoute {
            plan,
            run_id: queued_run,
            single_step,
            settle: queued_settle,
        } => {
            assert_eq!(plan, identity);
            assert_eq!(queued_run, run_id);
            assert!(single_step);
            assert_eq!(queued_settle, settle);
        }
        _ => panic!("Unexpected queued command"),
    }
    assert!(
        handle
            .execute_freecell_route(identity, true, route::AutoMode::ObserveGame, settle)
            .is_err()
    );
    assert!(matches!(
        command_rx.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
}


/// A resumable stop is scoped to an exact run, clears at completion and preserves its cursor.
#[test]
fn freecell_3118_stop_after_step_is_exact_idempotent_and_resets_before_resume() {
    let identity = plan(3118);
    let report = report();
    let mut store = Store::default();
    assert!(store.request_stop_after_step(&identity, 1).is_err());
    store
        .prepare(identity.clone(), Arc::clone(&report))
        .unwrap();
    let run_id = store.reserve(&identity).unwrap();
    assert!(!store.stop_after_step);
    assert!(store.request_stop_after_step(&plan(3119), run_id).is_err());
    assert!(
        store
            .request_stop_after_step(&identity, run_id + 1)
            .is_err()
    );
    assert!(!store.stop_after_step);
    store.request_stop_after_step(&identity, run_id).unwrap();
    store.request_stop_after_step(&identity, run_id).unwrap();
    assert!(store.stop_after_step);
    let mut state = store.live.as_ref().unwrap().state;
    let (moves, _) = recorded_route(&report.outcome).unwrap();
    state.position = state.position.apply(moves[0]).unwrap();
    state.cursor = 1;
    state.bound = true;
    store
        .finish(
            &identity,
            run_id,
            Some(state),
            Some(route::Completion::StepVerified),
        )
        .unwrap();
    assert!(!store.stop_after_step);
    assert!(store.available(&identity));
    let resumed = store.reserve(&identity).unwrap();
    assert!(resumed > run_id);
    assert_eq!(store.live.as_ref().unwrap().state.cursor, 1);
    assert!(!store.stop_after_step);
    assert!(store.request_stop_after_step(&identity, run_id).is_err());
    assert!(!store.stop_after_step);
    store.request_stop_after_step(&identity, resumed).unwrap();
    store.revoke();
    assert!(!store.stop_after_step);
    assert!(store.request_stop_after_step(&identity, resumed).is_err());
    store.reject(&identity, resumed).unwrap();
}


/// The UI updates the exact active owner without queueing input or changing global STOP.
#[test]
fn freecell_3118_worker_stop_after_step_is_direct_and_does_not_latch_global_stop() {
    let (command_tx, command_rx) = mpsc::channel();
    let (_, event_rx) = mpsc::channel();
    let latest_frame = LatestFrameSlot::default();
    let identity = plan(3118);
    latest_frame
        .freecell_route
        .lock()
        .unwrap()
        .prepare(identity.clone(), report())
        .unwrap();
    let handle = WorkerHandle {
        command_tx,
        event_rx,
        latest_frame,
        join: None,
        cancel_requested: Arc::new(AtomicBool::new(false)),
        shutdown_requested: Arc::new(AtomicBool::new(false)),
    };
    let run_id = handle
        .execute_freecell_route(
            identity.clone(),
            false,
            route::AutoMode::ObserveGame,
            Duration::from_millis(750),
        )
        .unwrap();
    let WorkerCommand::ExecuteFreeCellRoute { single_step, .. } = command_rx.try_recv().unwrap()
    else {
        panic!("Only the explicit playback command may be queued")
    };
    assert!(!single_step);
    assert!(
        handle
            .request_freecell_stop_after_step(&identity, run_id + 1)
            .is_err()
    );
    assert!(
        !handle
            .latest_frame
            .freecell_route
            .lock()
            .unwrap()
            .stop_after_step
    );
    handle
        .request_freecell_stop_after_step(&identity, run_id)
        .unwrap();
    assert!(
        handle
            .latest_frame
            .freecell_route
            .lock()
            .unwrap()
            .stop_after_step
    );
    assert!(!handle.cancel_requested.load(Ordering::Acquire));
    assert!(matches!(
        command_rx.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
}


/// Build a complete synthetic52-card deal solved entirely by safe automatic edges.
fn report() -> Arc<freecell_calculation::Report> {
    let mut columns: [Vec<Card>; 8] = std::array::from_fn(|_| Vec::new());
    for (i, suit) in FOUNDATION_SUITS.into_iter().enumerate() {
        columns[i] = (1..=7)
            .rev()
            .map(|r| Card::new(Rank::from_value(r).unwrap(), suit))
            .collect();
        columns[i + 4] = (8..=13)
            .rev()
            .map(|r| Card::new(Rank::from_value(r).unwrap(), suit))
            .collect();
    }
    let deal = InitialDeal::new(
        BoardState::new(
            columns.map(|cards| CascadeStack::from_cards(cards).unwrap()),
            [None; 4],
            [0; 4],
        )
        .unwrap(),
        Metadata::new(
            "test",
            "synthetic",
            crate::freecell_recon::objective::FULL_BOARD_SOURCE,
        )
        .unwrap(),
    )
    .unwrap();
    let outcome = solve_best_effort_with_control(&deal, &AtomicBool::new(false), None, None);
    assert!(matches!(outcome, Outcome::Solved { .. }));
    Arc::new(freecell_calculation::Report {
        path: PathBuf::from("synthetic.json"),
        deal,
        bytes: Vec::new(),
        outcome,
        paused: Duration::ZERO,
    })
}


/// Correlate the candidate with an immutable UI context and document generation.
fn plan(request_id: u64) -> freecell_calculation::Identity {
    freecell_calculation::Identity {
        context: ControllerContext::new(
            PathBuf::from("/tmp/freecell-route-test.sock"),
            GameMode::FreeCell,
            SolvingStrategy::ShortestPath,
            7,
        ),
        request_id,
        source_generation: 3,
    }
}


/// Hide the first suit aces without changing initial physical lengths or inventory.
fn blocked_deal(reversed: usize) -> InitialDeal {
    let original = report();
    let columns = std::array::from_fn(|column| {
        let mut cards: Vec<_> = original.deal.board().cascades()[column].iter().collect();
        if column < reversed {
            cards.reverse();
        }
        CascadeStack::from_cards(cards).unwrap()
    });
    InitialDeal::new(
        BoardState::new(columns, [None; 4], [0; 4]).unwrap(),
        original.deal.metadata().clone(),
    )
    .unwrap()
}


/// Supply a legal closed prefix at the trusted domain/transport seam.
///
/// These tests exercise replay, ownership and retirement; exhaustive maximum
/// quality is independently covered by xSAR's search tests, never inferred here.
fn prefix_report(deal: InitialDeal) -> Arc<freecell_calculation::Report> {
    let mut position = Position::initial(&deal);
    let mut moves = Vec::new();
    while let Some(action) = position.automatic() {
        position = position.apply(action).unwrap();
        moves.push(action);
    }
    let replay = xsar::games::freecell::replay::verify_prefix_with_control(
        &deal,
        &moves,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(
        replay
            .endpoint
            .foundations()
            .iter()
            .copied()
            .map(u32::from)
            .sum::<u32>()
            < 52
    );
    Arc::new(freecell_calculation::Report {
        path: PathBuf::from("typed-prefix-seam.json"),
        bytes: Vec::new(),
        deal,
        paused: Duration::ZERO,
        outcome: Outcome::Partial {
            moves: moves.into_boxed_slice(),
            stats: Default::default(),
            deliberate_moves: replay.deliberate,
            automatic_moves: replay.automatic,
            foundation_cards: replay
                .endpoint
                .foundations()
                .iter()
                .copied()
                .map(u32::from)
                .sum::<u32>(),
        },
    })
}


/// A replayed partial endpoint retires its permit even while many cards remain.
#[test]
fn freecell_315_store_partial_exhaustion_is_not_resumable() {
    let report = prefix_report(blocked_deal(3));
    let id = plan(315);
    let mut store = Store::default();
    store.prepare(id.clone(), Arc::clone(&report)).unwrap();
    assert!(store.available(&id));
    let run_id = store.reserve(&id).unwrap();
    let mut state = store.live.as_ref().unwrap().state;
    let (moves, _) = recorded_route(&report.outcome).unwrap();
    for &action in moves {
        state.position = state.position.apply(action).unwrap();
        state.cursor += 1;
    }
    state.bound = true;
    assert!(!state.position.is_goal());
    store
        .finish(
            &id,
            run_id,
            Some(state),
            Some(route::Completion::PartialRouteExhausted),
        )
        .unwrap();
    assert!(store.live.is_none() && store.running.is_none());
    assert!(!store.available(&id));
    assert!(store.reserve(&id).is_err());
}


/// Forged counts or a cut automatic closure cannot become an executable candidate.
#[test]
fn freecell_315_store_replays_partial_before_reservation() {
    let id = plan(316);
    for change_count in [true, false] {
        let mut report = Arc::try_unwrap(prefix_report(blocked_deal(3))).unwrap();
        let Outcome::Partial {
            foundation_cards,
            moves,
            ..
        } = &mut report.outcome
        else {
            unreachable!()
        };
        if change_count {
            *foundation_cards += 1;
        } else {
            *moves = moves[..moves.len() - 1].into();
        }
        let mut store = Store::default();
        assert!(store.prepare(id.clone(), Arc::new(report)).is_err());
        assert!(!store.available(&id) && store.live.is_none());
    }
}


/// No-benefit exhaustive results remain inspectable but do not offer PLAY.
#[test]
fn freecell_315_store_zero_action_partial_has_no_input_permit() {
    let report = prefix_report(blocked_deal(4));
    assert!(recorded_route(&report.outcome).unwrap().0.is_empty());
    let id = plan(317);
    let mut store = Store::default();
    store.prepare(id.clone(), report).unwrap();
    assert!(!store.available(&id));
    assert!(store.reserve(&id).is_err());
}


/// Passive completion destroys clones immediately but consumes only its exact final owner.
#[test]
fn freecell_315_store_terminal_revocation_preserves_exact_completion_owner() {
    let id = plan(318);
    let mut store = Store::default();
    store.prepare(id.clone(), report()).unwrap();
    let run_id = store.reserve(&id).unwrap();
    let state = store.live.as_ref().unwrap().state;
    store.revoke();
    assert!(store.check(&id, run_id).is_err());
    assert!(
        store
            .finish(
                &plan(319),
                run_id,
                Some(state),
                Some(route::Completion::ChallengeComplete)
            )
            .is_err()
    );
    assert_eq!(store.running, Some((id.clone(), run_id)));
    store
        .finish(
            &id,
            run_id,
            Some(state),
            Some(route::Completion::ChallengeComplete),
        )
        .unwrap();
    assert!(store.live.is_none() && store.running.is_none());
    assert!(store.reserve(&id).is_err());
}


/// Stale completion cannot consume the owner, and revoked clones cannot authorise input.
#[test]
fn freecell_311_store_revocation_and_exact_completion_are_permanent() {
    let report = report();
    let id = plan(1);
    let stale = plan(2);
    let mut store = Store::default();
    store.prepare(id.clone(), Arc::clone(&report)).unwrap();
    assert!(!store.available(&stale));
    let run = store.reserve(&id).unwrap();
    let cloned = store.live.clone().unwrap();
    assert!(store.check(&id, run).is_ok());
    assert!(
        store
            .finish(
                &stale,
                run,
                Some(cloned.state),
                Some(route::Completion::StepVerified)
            )
            .is_err()
    );
    assert_eq!(store.running, Some((id.clone(), run)));
    store.revoke();
    assert!(store.check(&id, run).is_err());
    assert!(store.prepare(stale.clone(), Arc::clone(&report)).is_err());
    assert!(
        store
            .finish(
                &id,
                run,
                Some(cloned.state),
                Some(route::Completion::StepVerified)
            )
            .unwrap_err()
            .contains("revoked")
    );
    assert!(!store.available(&id));
    assert!(store.reserve(&id).is_err());
    store.prepare(stale.clone(), report).unwrap();
    let next = store.reserve(&stale).unwrap();
    assert!(next > run);
    store.reject(&stale, next).unwrap();
    assert!(store.running.is_none() && store.live.is_none());
    assert!(store.available(&stale));
}


/// Success retains only an incomplete step; the complete52-card endpoint retires the plan.
#[test]
fn freecell_311_store_keeps_step_history_but_never_a_completed_or_failed_permit() {
    let report = report();
    let id = plan(1);
    let mut store = Store::default();
    store.prepare(id.clone(), Arc::clone(&report)).unwrap();
    let run = store.reserve(&id).unwrap();
    let mut state = store.live.as_ref().unwrap().state;
    store
        .finish(&id, run, Some(state), Some(route::Completion::StepVerified))
        .unwrap();
    assert!(store.available(&id));
    let Outcome::Solved { moves, .. } = &report.outcome else {
        unreachable!()
    };
    for action in moves {
        state.position = state.position.apply(*action).unwrap();
        state.cursor += 1;
    }
    state.bound = true;
    let run = store.reserve(&id).unwrap();
    store
        .finish(&id, run, Some(state), Some(route::Completion::FullBoard))
        .unwrap();
    assert!(store.live.is_none() && store.running.is_none());
    store.prepare(id.clone(), report).unwrap();
    store.serial = u64::MAX;
    assert!(store.reserve(&id).unwrap_err().contains("exhausted"));
    assert!(store.running.is_none());
}


/// A cancellation at offline retirement cannot publish an executable solved candidate.
#[test]
fn freecell_311_cancelled_offline_completion_and_stale_owner_never_install_route() {
    let latest = LatestFrameSlot::default();
    let (events, receiver) = mpsc::channel();
    let sink = WorkerEventSink {
        events,
        latest_frame: latest.clone(),
        capture_context: Mutex::new(None),
    };
    let id = plan(1);
    let cancellation = latest.freecell_calculation.reserve(id.clone()).unwrap();
    cancellation.store(true, Ordering::Release);
    let report = Arc::try_unwrap(report()).unwrap();
    freecell_calculation::complete(id.clone(), Ok(report), &sink);
    let event = receiver.try_recv().unwrap();
    let WorkerEvent::FreeCellCalculationFinished { identity, result } = event.event else {
        panic!("calculation completion only")
    };
    assert_eq!(identity, id);
    assert!(matches!(result.unwrap().outcome, Outcome::Cancelled(_)));
    assert!(!latest.freecell_route.lock().unwrap().available(&id));
    assert!(latest.take().is_none());
    assert!(receiver.try_recv().is_err());
    let next = plan(2);
    latest.freecell_calculation.reserve(next.clone()).unwrap();
    freecell_calculation::complete(id, Ok(Arc::try_unwrap(self::report()).unwrap()), &sink);
    let event = receiver.try_recv().unwrap();
    assert!(matches!(
        event.event,
        WorkerEvent::FreeCellCalculationFinished { result: Err(_), .. }
    ));
    assert!(latest.freecell_calculation.running());
    assert!(latest.freecell_calculation.reserve(next.clone()).is_err());
    assert!(!latest.freecell_route.lock().unwrap().available(&next));
}
mod native;


/// Successful steps resume exactly; policy changes reuse the same route with fresh binding.
#[test]
fn freecell_3110_store_policy_change_reuses_original_cache_without_search() {
    let mut store = Store::default();
    let id = plan(8);
    let report = report();
    let (moves, _) = recorded_route(&report.outcome).unwrap();
    store.prepare(id.clone(), Arc::clone(&report)).unwrap();
    let run = store
        .reserve_mode(&id, route::AutoMode::ExecuteRecorded)
        .unwrap();
    let mut state = store.live.as_ref().unwrap().state;
    state.position = state.position.apply(moves[0]).unwrap();
    state.cursor = 1;
    state.bound = true;
    store
        .finish(&id, run, Some(state), Some(route::Completion::StepVerified))
        .unwrap();
    let run = store
        .reserve_mode(&id, route::AutoMode::ExecuteRecorded)
        .unwrap();
    let resumed = store.live.as_ref().unwrap();
    assert_eq!(resumed.state.cursor, 1);
    assert!(resumed.state.bound);
    assert!(route::same_physical(
        &resumed.state.position,
        &state.position
    ));
    store
        .finish(&id, run, Some(state), Some(route::Completion::StepVerified))
        .unwrap();
    let run = store
        .reserve_mode(&id, route::AutoMode::ObserveGame)
        .unwrap();
    let restarted = store.live.as_ref().unwrap();
    assert_eq!(restarted.state.cursor, 0);
    assert!(!restarted.state.bound);
    assert_eq!(restarted.mode, route::AutoMode::ObserveGame);
    assert!(route::same_physical(
        &restarted.state.position,
        &Position::initial(&report.deal)
    ));
    assert!(Arc::ptr_eq(&store.cached.as_ref().unwrap().report, &report));
    store.reject(&id, run).unwrap();
}


/// Failed delivery history can never resume its old cursor, while the verified input is retained.
#[test]
fn freecell_3110_store_failure_restarts_original_board_and_fences_stale_results() {
    let report = report();
    let (moves, _) = recorded_route(&report.outcome).unwrap();
    let id = plan(3110);
    let mut store = Store::default();
    store.prepare(id.clone(), Arc::clone(&report)).unwrap();
    let first = store.reserve(&id).unwrap();
    let mut advanced = store.live.as_ref().unwrap().state;
    advanced.position = advanced.position.apply(moves[0]).unwrap();
    advanced.cursor = 1;
    advanced.bound = true;
    store
        .finish(
            &id,
            first,
            Some(advanced),
            Some(route::Completion::StepVerified),
        )
        .unwrap();
    let failed = store.reserve(&id).unwrap();
    store.finish(&id, failed, Some(advanced), None).unwrap();
    assert!(store.available(&id) && store.live.is_none());
    assert!(Arc::ptr_eq(&store.cached.as_ref().unwrap().report, &report));
    let retry = store.reserve(&id).unwrap();
    assert!(retry > failed && failed > first);
    assert_eq!(store.live.as_ref().unwrap().state.cursor, 0);
    assert!(!store.live.as_ref().unwrap().state.bound);
    assert!(route::same_physical(
        &store.live.as_ref().unwrap().state.position,
        &Position::initial(&report.deal)
    ));
    assert!(
        store
            .finish(
                &id,
                failed,
                Some(advanced),
                Some(route::Completion::StepVerified)
            )
            .is_err()
    );
    let mut source_changed = id.clone();
    source_changed.source_generation += 1;
    assert!(
        store
            .finish(
                &source_changed,
                retry,
                Some(advanced),
                Some(route::Completion::StepVerified)
            )
            .is_err()
    );
    assert_eq!(store.running, Some((id.clone(), retry)));
    assert_eq!(store.live.as_ref().unwrap().state.cursor, 0);
    store.revoke();
    store.reject(&id, retry).unwrap();
    assert!(store.cached.is_none() && store.live.is_none());
    assert!(!store.available(&id) && store.reserve(&id).is_err());
    store
        .prepare(source_changed.clone(), Arc::clone(&report))
        .unwrap();
    assert!(!store.available(&id) && store.available(&source_changed));
    assert!(
        store
            .finish(
                &id,
                retry,
                Some(advanced),
                Some(route::Completion::StepVerified)
            )
            .is_err()
    );
    assert!(store.available(&source_changed));
}


/// Failed cache retries still obey source edits, new calculations, STOP and shutdown.
#[test]
fn freecell_3110_worker_invalidation_prevents_cached_retry_resurrection() {
    for invalidate in 0..4 {
        let (command_tx, _command_rx) = mpsc::channel();
        let (_, event_rx) = mpsc::channel();
        let latest_frame = LatestFrameSlot::default();
        let id = plan(3110);
        {
            let mut store = latest_frame.freecell_route.lock().unwrap();
            store.prepare(id.clone(), report()).unwrap();
            let run = store.reserve(&id).unwrap();
            store.reject(&id, run).unwrap();
        }
        let handle = WorkerHandle {
            command_tx,
            event_rx,
            latest_frame,
            join: None,
            cancel_requested: Arc::new(AtomicBool::new(false)),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
        };
        assert!(handle.freecell_route_available(&id));
        match invalidate {
            0 => handle.revoke_freecell_route(),
            1 => handle
                .calculate_freecell(
                    id.context.clone(),
                    id.request_id + 1,
                    id.source_generation + 1,
                    PathBuf::from("unused.json"),
                )
                .unwrap(),
            2 => handle.cancel_freecell_calculation(),
            3 => handle.request_shutdown(),
            _ => unreachable!(),
        }
        assert!(!handle.freecell_route_available(&id));
        assert!(
            handle
                .execute_freecell_route(id, true, route::AutoMode::ObserveGame, Duration::ZERO)
                .is_err()
        );
        assert!(
            handle
                .latest_frame
                .freecell_route
                .lock()
                .unwrap()
                .cached
                .is_none()
        );
    }
}


/// The incident action uses checked readable card and physical-location labels.
#[test]
fn freecell_312_stage_details_name_incident_action() {
    let value = stage_details(route::Stage {
        cursor: 3,
        total: 89,
        mode: route::AutoMode::ExecuteRecorded,
        action: Some(RecordedMove {
            card_id: 27,
            from: xsar::games::freecell::rules::Pile::Cascade(3),
            to: xsar::games::freecell::rules::Pile::Cell(2),
            is_auto: false,
        }),
        phase: "effect",
        observation: 6,
    });
    assert_eq!(value["pending_transfer_one_based"], 4);
    assert_eq!(value["observation_limit"], 6);
    assert_eq!(value["action"]["card"], "2H");
    assert_eq!(value["action"]["from"], "PLAY 4");
    assert_eq!(value["action"]["to"], "CELL 3");
}


/// Only real transfer endpoints and recorded AUTO-affected piles supply effect evidence.
#[test]
fn freecell_3114_observation_scope_excludes_unrelated_columns() {
    let report = report();
    let Outcome::Solved { moves, .. } = &report.outcome else {
        unreachable!()
    };
    let initial = Position::initial(&report.deal);
    let first = moves[0];
    assert_eq!(first.from, Pile::Cascade(0));
    assert_eq!(first.to, Pile::Foundation(0));
    let after_ace = initial.apply(first).unwrap();
    let after_two = after_ace.apply(moves[1]).unwrap();
    assert_eq!(moves[1].from, Pile::Cascade(0));
    let candidates = [
        Candidate {
            cursor: 1,
            position: after_ace,
        },
        Candidate {
            cursor: 2,
            position: after_two,
        },
    ];
    assert_eq!(
        observation_piles(&candidates, Some(first)).unwrap(),
        vec![Pile::Cascade(0), Pile::Foundation(0)]
    );
    assert!(observation_piles(&[], Some(first)).is_err());
    assert!(observation_piles(&candidates, None).is_err());
    let other_auto = after_two.automatic().unwrap();
    let mut combined = after_two.apply(other_auto).unwrap();
    // Reach a different recorded source without inventing an unrecorded legal child.
    let mut cursor = 3;
    while combined
        .automatic()
        .is_some_and(|m| m.from == Pile::Cascade(0))
    {
        assert_eq!(moves[cursor], combined.automatic().unwrap());
        combined = combined.apply(moves[cursor]).unwrap();
        cursor += 1;
    }
    let next = moves[cursor];
    assert!(next.is_auto && next.from != first.from);
    combined = combined.apply(next).unwrap();
    let candidates = [
        candidates[0],
        Candidate {
            cursor: cursor + 1,
            position: combined,
        },
    ];
    let scope = observation_piles(&candidates, Some(first)).unwrap();
    assert!(scope.contains(&next.from) && scope.contains(&next.to));
    assert_eq!(scope.len(), 4);
    assert!(!scope.contains(&Pile::Cascade(7)));
}
