//! Paired-stock event traces verify pacing, endpoint proof and revoked authority.

use std::{cell::Cell, collections::VecDeque};

use super::{
    tests::{deck, happy_scenes, request},
    *,
};


/// Observable adapter events; authority checks deliberately remain input-free.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    /// One fresh capture attempt.
    Capture,
    /// One newly requested VM/pointer probe.
    Probe,
    /// One delivered operation, including an injected uncertain attempt.
    Input(ReconInput),
    /// One cooperative requested wait, with its exact duration.
    Wait(Duration),
    /// One completed-deal publication request.
    Publish,
}


/// Synthetic adapter retaining event order without sleeping or guest effects.
struct EventIo {
    /// Distinct scripted capture results, consumed once each.
    scenes: VecDeque<ReconScene>,
    /// Ordered externally observable calls.
    events: Vec<Event>,
    /// Advisory messages retained independently of the pacing/input event trace.
    messages: Vec<String>,
    /// Revoked authority, including a simulated STOP arriving during a wait.
    stopped: Cell<bool>,
    /// Revoke authority after this one-based wait, if configured.
    stop_after_wait: Option<usize>,
    /// Revoke authority after this one-based capture, if configured.
    stop_after_capture: Option<usize>,
    /// Report uncertain delivery for this one-based input attempt, if configured.
    uncertain_input: Option<usize>,
    /// A probe reservation is consumed by exactly one input.
    probe_reserved: bool,
}


impl EventIo {
    /// Construct a deterministic successful adapter with optional fault injection unset.
    fn new(scenes: Vec<ReconScene>) -> Self {
        Self {
            scenes: scenes.into(),
            events: Vec::new(),
            messages: Vec::new(),
            stopped: Cell::new(false),
            stop_after_wait: None,
            stop_after_capture: None,
            uncertain_input: None,
            probe_reserved: false,
        }
    }


    /// Return attempted typed inputs in delivery order, including uncertain attempts.
    fn inputs(&self) -> Vec<ReconInput> {
        self.events
            .iter()
            .filter_map(|event| {
                if let Event::Input(operation) = event {
                    Some(*operation)
                } else {
                    None
                }
            })
            .collect()
    }


    /// Count event instances satisfying a caller's phase-specific predicate.
    fn count(&self, predicate: impl Fn(&Event) -> bool) -> usize {
        self.events.iter().filter(|event| predicate(event)).count()
    }
}


impl ReconIo for EventIo {
    /// Fail after injected STOP without granting restoration or publication authority.
    fn check_active(&self) -> Result<(), String> {
        if self.stopped.get() {
            Err("STOP during paired scan".to_owned())
        } else {
            Ok(())
        }
    }


    /// Acquire one fresh scripted endpoint, optionally observing STOP immediately after it.
    fn observe(&mut self) -> Result<ReconScene, String> {
        self.events.push(Event::Capture);
        if self.stop_after_capture == Some(self.count(|event| *event == Event::Capture)) {
            self.stopped.set(true);
        }
        self.scenes
            .pop_front()
            .ok_or_else(|| "No fresh scripted capture remains".to_owned())
    }


    /// Reserve a new probe for exactly one subsequent typed operation.
    fn probe(&mut self) -> Result<(), String> {
        assert!(!self.probe_reserved, "a fresh probe must be consumed once");
        self.events.push(Event::Probe);
        self.probe_reserved = true;
        Ok(())
    }


    /// Consume a fresh probe and record delivery before any uncertainty is returned.
    fn input(&mut self, operation: ReconInput) -> Result<(), String> {
        self.check_active()?;
        assert!(
            self.probe_reserved,
            "every D and restoration input needs its own probe"
        );
        self.probe_reserved = false;
        self.events.push(Event::Input(operation));
        if self.uncertain_input == Some(self.inputs().len()) {
            return Err("Uncertain paired input delivery; do not replay".to_owned());
        }
        Ok(())
    }


    /// Retain exact pacing requests and optionally inject STOP during the interval.
    fn wait(&mut self, duration: Duration) -> Result<(), String> {
        self.events.push(Event::Wait(duration));
        if self.stop_after_wait == Some(self.count(|event| matches!(event, Event::Wait(_)))) {
            self.stopped.set(true);
        }
        Ok(())
    }


    /// Retain user feedback without adding an input, wait, probe or capture event.
    fn progress(&mut self, message: &str) {
        self.messages.push(message.to_owned());
    }


    /// Verify complete identities and restoration before recording publication.
    fn publish(&mut self, deal: &CompleteDeal) -> Result<(), String> {
        assert_eq!(deal.tableau().as_slice(), &deck()[..28]);
        assert_eq!(deal.stock().as_slice(), &deck()[28..]);
        assert_eq!(deal.metadata().restoration(), Restoration::Verified);
        self.events.push(Event::Publish);
        Ok(())
    }
}


/// Build an endpoint from the number of synthetic D operations that actually took effect.
fn after_draws(effects: &[bool]) -> BoardObservation {
    let advances = effects.iter().filter(|effect| **effect).count();
    assert!((1..24).contains(&advances));
    BoardObservation {
        tableau: deck()[..28].try_into().unwrap(),
        left: StockFace::Card(deck()[28 + advances]),
        right: StockFace::Card(deck()[27 + advances]),
        undo_all: Some(PixelPoint::new(1_320, 978)),
    }
}


/// Record the externally required fresh-probe/input relationship in a reference trace.
fn expected_input(events: &mut Vec<Event>, operation: ReconInput) {
    events.extend([Event::Probe, Event::Input(operation)]);
}


/// Completed observed draws permit one Undo All attempt despite unmatched toolbar pixels.
#[test]
fn recon_157_confirmed_inventory_uses_one_undo_attempt_without_icon_matching() {
    let mut scenes = happy_scenes();
    for scene in &mut scenes {
        if let ReconScene::Board(board) = scene {
            board.undo_all = None;
        }
    }
    let mut adapter = EventIo::new(scenes);
    assert!(run(&mut adapter, &request()).is_ok());
    assert_eq!(adapter.inputs().len(), 26);
    assert_eq!(
        adapter.inputs()[24],
        ReconInput::UndoAll(crate::parameters::SHARED_UNDO_ALL_CONTROL.click_point)
    );
    assert_eq!(adapter.count(|event| *event == Event::Publish), 1);
}


/// Absence of the expected full dialog ends after one Undo All; no confirmation or export.
#[test]
fn recon_157_missing_dialog_never_retries_undo_or_publishes() {
    let mut scenes = happy_scenes();
    scenes.truncate(16);
    let mut exhausted = scenes.last().unwrap().clone();
    if let ReconScene::Board(board) = &mut exhausted {
        board.undo_all = None;
    }
    scenes[15] = exhausted.clone();
    scenes.extend(vec![exhausted; MAX_OBSERVATIONS]);
    let mut adapter = EventIo::new(scenes);
    let failure = run(&mut adapter, &request()).unwrap_err();
    assert!(failure.message.contains("confirmation did not appear"));
    assert_eq!(adapter.inputs().len(), 25);
    assert_eq!(
        adapter.inputs().last(),
        Some(&ReconInput::UndoAll(
            crate::parameters::SHARED_UNDO_ALL_CONTROL.click_point
        ))
    );
    assert_eq!(adapter.count(|event| *event == Event::Publish), 0);
}


/// Confirmed history cannot bypass STOP or replay an uncertain Undo All attempt.
#[test]
fn recon_157_icon_independent_undo_retains_stop_and_uncertain_delivery_guards() {
    for stop in [true, false] {
        let mut scenes = happy_scenes();
        for scene in &mut scenes {
            if let ReconScene::Board(board) = scene {
                board.undo_all = None;
            }
        }
        let mut adapter = EventIo::new(scenes);
        if stop {
            adapter.stop_after_capture = Some(16);
        } else {
            adapter.uncertain_input = Some(25);
        }
        assert!(run(&mut adapter, &request()).is_err());
        assert_eq!(adapter.inputs().len(), if stop { 24 } else { 25 });
        assert_eq!(adapter.count(|event| *event == Event::Capture), 16);
        assert_eq!(adapter.count(|event| *event == Event::Publish), 0);
        assert!(
            !adapter
                .inputs()
                .iter()
                .any(|input| matches!(input, ReconInput::Confirm(_)))
        );
    }
}


/// The approved schedule has 20 captures, 24 independent D probes and exact phase pacing.
#[test]
fn paired_scan_observes_twenty_frames_with_exact_draw_and_restoration_pacing() {
    let mut adapter = EventIo::new(happy_scenes());
    let deal = run(&mut adapter, &request()).unwrap();
    let half_second = Duration::from_millis(500);
    let one_second = Duration::from_secs(1);
    let mut expected = vec![Event::Capture, Event::Wait(one_second), Event::Capture];
    expected_input(&mut expected, ReconInput::Draw);
    expected.extend([Event::Wait(half_second), Event::Capture]);


    for _pair in 0..11 {
        expected_input(&mut expected, ReconInput::Draw);
        expected.push(Event::Wait(half_second));
        expected_input(&mut expected, ReconInput::Draw);
        expected.extend([Event::Wait(half_second), Event::Capture]);
    }
    expected_input(&mut expected, ReconInput::Draw);
    expected.extend([
        Event::Wait(half_second),
        Event::Capture,
        Event::Wait(half_second),
        Event::Capture,
    ]);
    expected_input(
        &mut expected,
        ReconInput::UndoAll(PixelPoint::new(1_320, 978)),
    );
    expected.extend([Event::Capture, Event::Capture]);
    expected_input(
        &mut expected,
        ReconInput::Confirm(PixelPoint::new(960, 600)),
    );
    expected.extend([
        Event::Wait(one_second),
        Event::Capture,
        Event::Capture,
        Event::Publish,
    ]);
    assert_eq!(adapter.events, expected);
    assert_eq!(adapter.count(|event| *event == Event::Capture), 20);
    assert_eq!(adapter.count(|event| *event == Event::Probe), 26);
    assert_eq!(
        adapter.count(|event| *event == Event::Input(ReconInput::Draw)),
        24
    );
    assert_eq!(
        deal.stock().as_slice(),
        &deck()[28..],
        "DR precedes DL for every pair"
    );
    assert!(adapter.scenes.is_empty());
    let pairs = adapter
        .messages
        .iter()
        .filter(|message| message.starts_with("Recon paired D:"))
        .collect::<Vec<_>>();
    assert_eq!(pairs.len(), 11);
    for (index, message) in pairs.iter().enumerate() {
        let stock = deal.stock().as_slice();
        assert!(message.contains(&format!(
            "DL: {} | DR: {}",
            stock[3 + 2 * index],
            stock[2 + 2 * index]
        )));
    }
    assert!(
        adapter
            .messages
            .iter()
            .any(|message| message.starts_with("Recon initial stock:")
                && message.contains("| DR: empty"))
    );
    assert!(
        adapter
            .messages
            .iter()
            .any(|message| message.starts_with("Recon exhaustion:")
                && message.contains("DL: empty | DR:"))
    );
}


/// Every pair may use five input-free stale endpoints before its sixth fresh proof.
#[test]
fn delayed_pair_endpoints_reobserve_without_another_draw() {
    for pair in 0..11 {
        let mut scenes = happy_scenes();
        let endpoint = 3 + pair;
        let unchanged = scenes[endpoint - 1].clone();
        scenes.splice(
            endpoint..endpoint,
            std::iter::repeat_n(unchanged, MAX_OBSERVATIONS - 1),
        );
        let mut adapter = EventIo::new(scenes);
        let deal = run(&mut adapter, &request()).unwrap();
        assert_eq!(adapter.inputs().len(), 26, "pair {pair}");
        assert_eq!(
            adapter.count(|event| *event == Event::Capture),
            25,
            "pair {pair}"
        );
        assert_eq!(
            adapter.count(|event| *event == Event::Wait(Duration::from_millis(500))),
            30
        );
        assert_eq!(deal.stock().as_slice(), &deck()[28..]);
    }
}


/// A pair that never changes its endpoint consumes six observations and receives no catch-up D.
#[test]
fn unchanged_pair_endpoint_exhausts_only_the_observation_budget() {
    for pair in 0..11 {
        let mut scenes = happy_scenes();
        let endpoint = 3 + pair;
        let unchanged = scenes[endpoint - 1].clone();
        scenes.truncate(endpoint);
        scenes.extend(std::iter::repeat_n(unchanged, MAX_OBSERVATIONS));
        let mut adapter = EventIo::new(scenes);
        let failure = run(&mut adapter, &request()).unwrap_err();
        assert!(failure.message.contains("not replayed"));
        assert_eq!(failure.progress.stock.len(), 2 + pair * 2);
        assert_eq!(failure.progress.draws, 3 + pair * 2);
        assert_eq!(adapter.inputs(), vec![ReconInput::Draw; 3 + pair * 2]);
        assert_eq!(
            adapter.count(|event| *event == Event::Capture),
            endpoint + MAX_OBSERVATIONS
        );
        assert_eq!(adapter.count(|event| *event == Event::Publish), 0);
    }
}


/// Either acknowledged-but-missed pair member exposes the previous DL in DR and must stop.
#[test]
fn missed_first_or_second_draw_in_any_pair_cannot_trigger_catchup() {
    for pair in 0..11 {
        for missed_member in 0..2 {
            let draws = 3 + pair * 2;
            let mut effects = vec![true; draws];
            effects[draws - 2 + missed_member] = false;
            let mut scenes = happy_scenes();
            scenes[3 + pair] = ReconScene::Board(after_draws(&effects));
            let mut adapter = EventIo::new(scenes);
            let failure = run(&mut adapter, &request()).unwrap_err();
            assert!(
                failure.message.contains("Duplicate recognised identity"),
                "pair {pair}, missed member {missed_member}: {}",
                failure.message
            );
            assert_eq!(failure.progress.stock.len(), 2 + pair * 2);
            assert_eq!(adapter.inputs(), vec![ReconInput::Draw; draws]);
            assert_eq!(adapter.count(|event| *event == Event::Capture), 4 + pair);
            assert_eq!(adapter.count(|event| *event == Event::Publish), 0);
        }
    }
}


/// Duplicate endpoints, changed tableau, unsupported scenes and early empty piles revoke input.
#[test]
fn conflicting_pair_evidence_stops_before_further_input() {
    let valid = after_draws(&[true, true, true]);
    let mut duplicate = valid.clone();
    duplicate.left = duplicate.right;
    let mut tableau_identity = valid.clone();
    tableau_identity.right = StockFace::Card(deck()[0]);
    let mut early_empty = valid.clone();
    early_empty.left = StockFace::Empty;
    let mut empty_waste = valid.clone();
    empty_waste.right = StockFace::Empty;
    let mut changed_tableau = valid;
    changed_tableau.tableau.swap(0, 1);
    let cases = [
        ReconScene::Board(duplicate),
        ReconScene::Board(tableau_identity),
        ReconScene::Board(early_empty),
        ReconScene::Board(empty_waste),
        ReconScene::Board(changed_tableau),
        ReconScene::Unknown,
    ];
    for scene in cases {
        let mut scenes = happy_scenes();
        scenes[3] = scene;
        let mut adapter = EventIo::new(scenes);
        let failure = run(&mut adapter, &request()).unwrap_err();
        assert!(!failure.progress.restored);
        assert_eq!(adapter.inputs(), [ReconInput::Draw; 3]);
        assert_eq!(adapter.count(|event| *event == Event::Capture), 4);
        assert_eq!(adapter.count(|event| *event == Event::Publish), 0);
    }
}


/// STOP in each between-D wait revokes the second member and all restoration authority.
#[test]
fn stop_between_pair_members_withholds_second_draw_and_cleanup() {
    for pair in 0..11 {
        let mut adapter = EventIo::new(happy_scenes());
        adapter.stop_after_wait = Some(3 + pair * 2);
        let failure = run(&mut adapter, &request()).unwrap_err();
        assert!(failure.message.contains("STOP"));
        assert_eq!(failure.progress.stock.len(), 2 + pair * 2);
        assert_eq!(adapter.inputs(), vec![ReconInput::Draw; 2 + pair * 2]);
        assert_eq!(
            adapter.events.last(),
            Some(&Event::Wait(Duration::from_millis(500)))
        );
        assert_eq!(adapter.count(|event| *event == Event::Publish), 0);
    }
}


/// STOP after exhaustion, dialog confirmation or the first restored frame grants no later work.
#[test]
fn stop_at_restoration_boundaries_never_adds_cleanup_or_publication() {
    for (capture, inputs) in [(16, 24), (18, 25), (19, 26)] {
        let mut adapter = EventIo::new(happy_scenes());
        adapter.stop_after_capture = Some(capture);
        let failure = run(&mut adapter, &request()).unwrap_err();
        assert!(failure.message.contains("STOP"));
        assert!(!failure.progress.restored);
        assert_eq!(adapter.inputs().len(), inputs);
        assert_eq!(adapter.events.last(), Some(&Event::Capture));
        assert_eq!(adapter.count(|event| *event == Event::Capture), capture);
        assert_eq!(adapter.count(|event| *event == Event::Publish), 0);
    }
}


/// Any uncertain pair member is the final event; no wait, capture, replay or recovery follows.
#[test]
fn uncertainty_at_each_pair_member_ends_the_event_trace_immediately() {
    for attempted in 2..=23 {
        let mut adapter = EventIo::new(happy_scenes());
        adapter.uncertain_input = Some(attempted);
        let failure = run(&mut adapter, &request()).unwrap_err();
        assert!(failure.message.contains("Uncertain"));
        assert_eq!(adapter.inputs(), vec![ReconInput::Draw; attempted]);
        assert_eq!(adapter.events.last(), Some(&Event::Input(ReconInput::Draw)));
        assert!(!failure.progress.restored);
        assert_eq!(adapter.count(|event| *event == Event::Publish), 0);
    }
}


/// A missed final D uses only observations, never D25 or recycling, despite full identity knowledge.
#[test]
fn final_single_draw_cannot_be_replayed_or_promoted_to_a_pair() {
    let mut scenes = happy_scenes();
    let last_pair = scenes[13].clone();
    scenes.truncate(14);
    scenes.extend(std::iter::repeat_n(last_pair, MAX_OBSERVATIONS));
    let mut adapter = EventIo::new(scenes);
    let failure = run(&mut adapter, &request()).unwrap_err();
    assert!(failure.message.contains("not replayed"));
    assert_eq!(failure.progress.stock.len(), 24);
    assert_eq!(failure.progress.draws, 24);
    assert_eq!(adapter.inputs(), [ReconInput::Draw; 24]);
    assert_eq!(adapter.count(|event| *event == Event::Capture), 20);
    assert_eq!(adapter.count(|event| *event == Event::Publish), 0);
}
