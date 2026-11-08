//! Socket-free ownership and cancellation boundaries using a real verified solver result.
use super::*;
use xsar::games::{
    cards::{Card, Rank},
    freecell::{BoardState, CascadeStack, FOUNDATION_SUITS, Metadata, search::solve_with_control},
};


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
        Metadata::new("test", "synthetic", "unit").unwrap(),
    )
    .unwrap();
    let outcome = solve_with_control(&deal, &AtomicBool::new(false), None, None);
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
    assert!(store.finish(&stale, run, Some(cloned.state), true).is_err());
    assert_eq!(store.running, Some((id.clone(), run)));
    store.revoke();
    assert!(store.check(&id, run).is_err());
    assert!(store.prepare(stale.clone(), Arc::clone(&report)).is_err());
    assert!(
        store
            .finish(&id, run, Some(cloned.state), true)
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
    assert!(store.reserve(&stale).is_err());
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
    store.finish(&id, run, Some(state), true).unwrap();
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
    store.finish(&id, run, Some(state), true).unwrap();
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


/// A successful step freezes autoplay policy until a fresh calculation is retained.
#[test]
fn freecell_311_store_freezes_autoplay_policy() {
    let mut store = Store::default();
    let id = plan(8);
    store.prepare(id.clone(), report()).unwrap();
    let run = store
        .reserve_mode(&id, route::AutoMode::ExecuteRecorded)
        .unwrap();
    let state = store.live.as_ref().unwrap().state;
    store.finish(&id, run, Some(state), true).unwrap();
    assert!(
        store
            .reserve_mode(&id, route::AutoMode::ObserveGame)
            .unwrap_err()
            .contains("Autoplay mode changed")
    );
    assert!(store.running.is_none());
    let run = store
        .reserve_mode(&id, route::AutoMode::ExecuteRecorded)
        .unwrap();
    store.reject(&id, run).unwrap();
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
