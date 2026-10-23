//! Dedicated, bounded Pyramid reconnaissance without Solver or route execution.
//!
//! Native recognition, capture/input delivery and publication are separate from
//! this state machine. Two-card stock batches omit the intermediate image; their
//! endpoint identities remain directly recognised, never inferred from input ACKs.
//! STOP or context loss ends the operation without automatic restoration clicks.

pub mod vision;

#[cfg(test)]
mod paired_tests;

use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::parameters::{
    PYRAMID_DRAW_SETTLE_DELAY as DRAW_DELAY, PYRAMID_RECON_DEADLINE as MAX_DURATION,
    PYRAMID_REOBSERVE_DELAY as OBSERVATION_DELAY,
};

use xsar::games::{
    cards::Card,
    pyramid::{CompleteDeal, DealMetadata, ProvenanceKind, Restoration},
};

use crate::geometry::PixelPoint;
use vision::{BoardObservation, ReconScene, StockFace};


/// Absolute capture-attempt budget, including initial and restoration evidence.
const MAX_CAPTURES: usize = 180;


/// Initial LEFT plus twenty-three advances and one empty-producing advance.
const MAX_DRAWS: usize = 24;


/// Bounded fresh observations after an acknowledged operation or D pair, never replays.
const MAX_OBSERVATIONS: usize = 6;


/// Explicit calibration selection, independent of consent and live input authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReconProfileSource {
    /// Native-v22 challenge-clear and measured diamond calibration embedded in this executable.
    BundledNativeV22,
    /// Retained native-v17 regression selection; runtime rollback uses its strict File profile.
    #[cfg(test)]
    BundledNativeV17,
    /// Strict custom calibration file, loaded once before any guest input.
    File(PathBuf),
}


/// Explicit user request, independent of the read-only preparation strategy.
#[derive(Clone, Debug)]
pub struct ReconRequest {
    /// Bundled native calibration or a bounded custom calibration file path.
    pub profile: ReconProfileSource,
    /// New completed-deal file; publication must not replace an existing entry.
    pub output_path: PathBuf,
    /// Bounded ASCII deal identifier used by the shared checked model.
    pub deal_id: String,
    /// Explicit operator assertion of an untouched board with Solver not engaged.
    pub initial_board_confirmed: bool,
}


impl ReconRequest {


    /// Validate caller-controlled fields before connecting or resetting STOP.
    ///
    /// # Errors
    /// Refuses missing consent, invalid deal identifiers, empty or oversized custom
    /// profile paths, and empty, oversized or unnamed output paths. Bundled profile
    /// selection requires no path and grants no input authority by itself.
    pub fn validate(&self) -> Result<(), String> {


        if !self.initial_board_confirmed {
            return Err("Confirm an untouched Pyramid deal with Solver not engaged (unavailable is allowed) before reconnaissance".to_owned());
        }


        let invalid_profile = match &self.profile {
            ReconProfileSource::BundledNativeV22 => false,
            #[cfg(test)]
            ReconProfileSource::BundledNativeV17 => false,
            ReconProfileSource::File(path) => {
                path.as_os_str().is_empty() || path.as_os_str().len() > 4_096
            }
        };


        if invalid_profile
            || self.output_path.as_os_str().is_empty()
            || self.output_path.file_name().is_none()
            || self.output_path.as_os_str().len() > 4_096
        {
            return Err(
                "Reconnaissance requires bounded calibration and output file paths".to_owned(),
            );
        }
        metadata(&self.deal_id, 0).map(|_| ())
    }
}


/// Whitelisted operations; there is no Solver, recycle, card or New Game input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconInput {
    /// One stock advance, acknowledged once and never replayed on uncertainty.
    Draw,
    /// Fresh enabled Undo All toolbar target, only after complete acquisition.
    UndoAll(PixelPoint),
    /// Fresh complete Undo All confirmation signature and its measured target.
    Confirm(PixelPoint),
}


/// Application-owned capture/input adapter; observations must always be fresh.
pub trait ReconIo {
    /// Refuse STOP or any socket/game/strategy/generation change.
    fn check_active(&self) -> Result<(), String>;
    /// Capture, publish advisory pixels and classify one new native frame.
    fn observe(&mut self) -> Result<ReconScene, String>;
    /// Freshly validate VM running state and current absolute pointer.
    fn probe(&mut self) -> Result<(), String>;
    /// Deliver one whitelisted input with a final STOP/context check.
    fn input(&mut self, operation: ReconInput) -> Result<(), String>;
    /// Wait cooperatively without sending input.
    fn wait(&mut self, duration: Duration) -> Result<(), String>;
    /// Record bounded evidence and ordinal progress in the existing session log.
    fn progress(&mut self, message: &str);
    /// Atomically publish a new validated complete/restored document.
    fn publish(&mut self, deal: &CompleteDeal) -> Result<(), String>;
}


/// Accumulated recognised knowledge, distinct from the current live position.
#[derive(Clone, Debug, Default)]
pub struct ReconProgress {
    /// Original logical top-down, left-to-right tableau, when read successfully.
    pub tableau: Option<[Card; 28]>,
    /// Recognised first-draw identities, including initial LEFT at ordinal zero.
    pub stock: Vec<Card>,
    /// Capture attempts, including a failed or cancelled acquisition.
    pub captures: usize,
    /// D operations attempted, including uncertain delivery; never over twenty-four.
    pub draws: usize,
    /// True only after two consecutive fresh original-board observations.
    pub restored: bool,
}


/// Explicitly incomplete diagnostics; this type cannot be used as planning input.
#[derive(Clone, Debug)]
pub struct ReconFailure {
    /// Bounded explanation, without claiming automatic cleanup or restoration.
    pub message: String,
    /// Knowledge retained even after STOP or uncertainty revokes input authority.
    pub progress: ReconProgress,
}


/// Run the complete acquisition/restoration/publication pipeline once.
pub fn run(
    io: &mut impl ReconIo,
    request: &ReconRequest,
) -> Result<CompleteDeal, Box<ReconFailure>> {
    let mut runner = Runner {
        io,
        started: Instant::now(),
        progress: ReconProgress::default(),
    };
    let result = request.validate().and_then(|()| runner.acquire(request));
    result.map_err(|message| {
        Box::new(ReconFailure {
            message: message.chars().take(2_048).collect(),
            progress: runner.progress,
        })
    })
}


/// Private single-run authority and finite counters; it cannot be resumed blindly.
struct Runner<'adapter, Adapter> {
    /// Application I/O with independent cancellation and selected-context checks.
    io: &'adapter mut Adapter,
    /// Deadline origin, covering all phases after request validation begins.
    started: Instant,
    /// Original arrangement and accumulated identities, never live card positions.
    progress: ReconProgress,
}


impl<Adapter: ReconIo> Runner<'_, Adapter> {


    /// Check cooperative cancellation and the elapsed-time budget.
    fn active(&self) -> Result<(), String> {
        self.io.check_active()?;


        if self.started.elapsed() >= MAX_DURATION {
            return Err("Reconnaissance reached its 300-second cooperative deadline".to_owned());
        }
        Ok(())
    }


    /// Obtain one fresh bounded observation, including checks after blocking I/O.
    fn capture(&mut self) -> Result<ReconScene, String> {
        self.active()?;


        if self.progress.captures >= MAX_CAPTURES {
            return Err("Reconnaissance capture budget exhausted".to_owned());
        }
        self.progress.captures += 1;
        let scene = self.io.observe()?;
        self.active()?;


        if matches!(scene, ReconScene::Unknown) {
            return Err("Unknown native scene; input authority revoked".to_owned());
        }
        Ok(scene)
    }


    /// Wait without input, then check cancellation and deadline again.
    fn settle(&mut self) -> Result<(), String> {
        self.wait(OBSERVATION_DELAY)
    }


    /// Wait for the caller's phase-specific interval with authority checks at both ends.
    fn wait(&mut self, duration: Duration) -> Result<(), String> {
        self.active()?;
        self.io.wait(duration)?;
        self.active()
    }


    /// Reserve one input after fresh probes; uncertain delivery is never retried.
    fn perform(&mut self, operation: ReconInput) -> Result<(), String> {
        self.active()?;
        self.io.probe()?;
        self.active()?;


        if operation == ReconInput::Draw {


            if self.progress.draws >= MAX_DRAWS {
                return Err("Reconnaissance D budget exhausted; recycling is forbidden".to_owned());
            }
            self.progress.draws += 1;
        }
        self.io.input(operation)?;
        self.active()
    }


    /// Record an actually recognised stock identity only once and in observation order.
    fn append_stock(&mut self, card: Card) -> Result<(), String> {


        if self.progress.stock.len() >= 24 {
            return Err(
                "Unexpected extra stock card; the single-deck inventory is full".to_owned(),
            );
        }


        if self
            .progress
            .tableau
            .as_ref()
            .is_some_and(|tableau| tableau.contains(&card))
            || self.progress.stock.contains(&card)
        {
            return Err(format!(
                "Duplicate recognised identity {card}; no card is inferred by elimination"
            ));
        }
        let ordinal = self.progress.stock.len();
        self.progress.stock.push(card);
        self.io.progress(&format!(
            "Recon recognised DRAW ordinal {ordinal}: {card}; fresh capture {}",
            self.progress.captures
        ));
        Ok(())
    }


    /// Send a single D or bounded pair, then recognise the endpoint without replay.
    ///
    /// A pair deliberately has no intermediate capture. Both members retain their
    /// own probe, STOP/context checks and acknowledgement; each is followed by at
    /// least 500 ms before the next input or stock observation. Two new endpoint
    /// identities are appended RIGHT then LEFT. A single-D endpoint must preserve
    /// the previous LEFT in RIGHT; final exhaustion requires two positive frames.
    fn advance_stock(
        &mut self,
        initial: &BoardObservation,
        current: &BoardObservation,
        paired: bool,
    ) -> Result<BoardObservation, String> {
        let previous = card_in_left(current)?;
        self.perform(ReconInput::Draw)?;


        if paired {
            self.wait(DRAW_DELAY)?;
            self.perform(ReconInput::Draw)?;
        }
        let mut empty_observations = 0;


        for _observation in 0..MAX_OBSERVATIONS {
            self.wait(DRAW_DELAY)?;
            let observed = board(self.capture()?)?;
            unchanged_tableau(initial, &observed)?;


            if observed.left == current.left && observed.right == current.right {
                empty_observations = 0;
                continue;
            }


            if paired {
                let (StockFace::Card(right), StockFace::Card(left)) =
                    (observed.right, observed.left)
                else {
                    return Err("Paired D lacks two recognised stock identities; no catch-up or replay is permitted".to_owned());
                };
                self.append_stock(right)?;
                self.append_stock(left)?;
                self.io.progress(&format!(
                    "Recon paired D: DL: {left} | DR: {right}; {}/24 stock cards; fresh capture {}",
                    self.progress.stock.len(),
                    self.progress.captures
                ));
                return Ok(observed);
            }


            if observed.right != StockFace::Card(previous) {
                return Err("Stock advancement lacks the previous LEFT identity in RIGHT; no replay is permitted".to_owned());
            }


            match observed.left {
                StockFace::Card(next) => {
                    self.append_stock(next)?;
                    self.io.progress(&format!(
                        "Recon single D: DL: {next} | DR: {previous}; {}/24 stock cards; fresh capture {}",
                        self.progress.stock.len(), self.progress.captures
                    ));
                    return Ok(observed);
                }
                StockFace::Empty => {


                    if self.progress.stock.len() != MAX_DRAWS {
                        return Err(
                            "LEFT became empty before all 24 stock identities were recognised"
                                .to_owned(),
                        );
                    }
                    empty_observations += 1;


                    if empty_observations == 2 {
                        self.io.progress(&format!(
                            "Recon exhaustion: DL: empty | DR: {previous}; fresh capture {}",
                            self.progress.captures
                        ));
                        return Ok(observed);
                    }
                }
            }
        }
        Err("Acknowledged D batch has no proven advancement within six fresh observations; D was not replayed".to_owned())
    }


    /// Establish the original arrangement, scan stock, restore, then publish.
    fn acquire(&mut self, request: &ReconRequest) -> Result<CompleteDeal, String> {
        let initial = board(self.capture()?)?;
        self.progress.tableau = Some(initial.tableau);
        validate_tableau(&initial.tableau)?;
        let first = card_in_left(&initial)?;


        if initial.right != StockFace::Empty {
            return Err(
                "Initial RIGHT pile is not positively empty; untouched-board requirement failed"
                    .to_owned(),
            );
        }
        self.append_stock(first)?;
        self.io.progress(&format!(
            "Recon initial stock: DL: {first} | DR: empty; 1/24 stock cards"
        ));
        self.settle()?;
        let confirmation = board(self.capture()?)?;


        if !same_arrangement(&initial, &confirmation) {
            return Err("Initial board changed between two fresh observations".to_owned());
        }
        self.io.progress("Recon original tableau: 28 individually recognised identities; initial LEFT recorded before D");
        let mut current = confirmation;


        // The first D confirms S1 in RIGHT and exposes S2 in LEFT. Eleven later
        // pairs expose S3/S4 through S23/S24, without an intermediate image.
        while self.progress.stock.len() < MAX_DRAWS {
            let paired = self.progress.stock.len() > 1;
            current = self.advance_stock(&initial, &current, paired)?;
        }
        // D24 is single: a further D would recycle, which is never authorised.
        current = self.advance_stock(&initial, &current, false)?;


        if current.left != StockFace::Empty || self.progress.stock.len() != MAX_DRAWS {
            return Err("Reconnaissance ended without complete inventory and two positive empty-LEFT observations".to_owned());
        }
        let undo_all = current
            .undo_all
            .ok_or_else(|| "Fresh enabled Undo All toolbar signature is absent".to_owned())?;
        self.io.progress(
            "Recon inventory complete: 28 tableau + 24 ordered stock; LEFT positively empty twice",
        );
        self.perform(ReconInput::UndoAll(undo_all))?;
        let mut confirmation_target = None;


        for observation in 0..MAX_OBSERVATIONS {
            match self.capture()? {
                ReconScene::UndoConfirmation { ok } => {
                    confirmation_target = Some(ok);
                    break;
                }
                ReconScene::Board(observed) if same_arrangement(&current, &observed) => {}
                _ => {
                    return Err(
                        "Unexpected scene after Undo All; confirmation input withheld".to_owned(),
                    );
                }
            }


            if observation + 1 < MAX_OBSERVATIONS {
                self.settle()?;
            }
        }
        let ok = confirmation_target.ok_or_else(|| {
            "Undo All confirmation did not appear within the observation budget".to_owned()
        })?;


        if !matches!(self.capture()?, ReconScene::UndoConfirmation { ok: fresh } if fresh == ok) {
            return Err(
                "Undo All confirmation signature changed before OK; input withheld".to_owned(),
            );
        }
        self.perform(ReconInput::Confirm(ok))?;
        // Confirmation ACK proves delivery; allow the modal to settle before observing restoration.
        self.settle()?;
        let mut restored_observations = 0;


        for observation in 0..MAX_OBSERVATIONS {
            match self.capture()? {
                ReconScene::Board(observed) if same_arrangement(&initial, &observed) => {
                    restored_observations += 1;


                    if restored_observations == 2 {
                        self.progress.restored = true;
                        break;
                    }
                }
                ReconScene::Board(observed) if same_arrangement(&current, &observed) => {
                    restored_observations = 0;
                }
                ReconScene::UndoConfirmation { ok: fresh } if fresh == ok => {
                    restored_observations = 0;
                }
                _ => {
                    return Err(
                        "Unexpected scene while verifying original-board restoration".to_owned(),
                    );
                }
            }


            if restored_observations == 0 && observation + 1 < MAX_OBSERVATIONS {
                self.settle()?;
            }
        }


        if !self.progress.restored {
            return Err("OK delivery did not prove original-board restoration twice; no complete JSON published".to_owned());
        }
        let stock: [Card; 24] = self
            .progress
            .stock
            .as_slice()
            .try_into()
            .map_err(|_| "Internal stock cardinality mismatch".to_owned())?;
        let deal = CompleteDeal::try_new(
            initial.tableau,
            stock,
            metadata(&request.deal_id, self.progress.captures)?,
        )
        .map_err(|error| format!("Complete restored deal validation failed: {error}"))?;
        self.active()?;
        self.io.progress("Original tableau, initial LEFT and empty RIGHT restored twice; publishing complete JSON");
        self.io.publish(&deal)?;
        Ok(deal)
    }
}


/// Require a supported card-bearing scene rather than borrowing dialog authority.
fn board(scene: ReconScene) -> Result<BoardObservation, String> {


    match scene {
        ReconScene::Board(observation) => Ok(observation),
        _ => Err("Expected a supported Pyramid board".to_owned()),
    }
}


/// Read the actual current LEFT identity; empty is not an implicit next card.
fn card_in_left(observation: &BoardObservation) -> Result<Card, String> {


    match observation.left {
        StockFace::Card(card) => Ok(card),
        StockFace::Empty => Err("Expected a recognised initial/current LEFT card".to_owned()),
    }
}


/// Reject duplicate tableau identities without substituting unobserved cards.
fn validate_tableau(tableau: &[Card; 28]) -> Result<(), String> {


    for (index, card) in tableau.iter().enumerate() {


        if tableau[..index].contains(card) {
            return Err(format!(
                "Duplicate tableau identity {card} at logical slot {index}"
            ));
        }
    }
    Ok(())
}


/// A stock-only scan may never alter an original tableau identity or position.
fn unchanged_tableau(
    initial: &BoardObservation,
    observed: &BoardObservation,
) -> Result<(), String> {
    if initial.tableau != observed.tableau {
        return Err(
            "Original tableau changed during reconnaissance; input authority revoked".to_owned(),
        );
    }
    Ok(())
}


/// Compare the complete logical layout and both positively classified piles.
fn same_arrangement(expected: &BoardObservation, observed: &BoardObservation) -> bool {
    expected.tableau == observed.tableau
        && expected.left == observed.left
        && expected.right == observed.right
}


/// Validate bounded provenance fields; metadata alone never proves restoration.
fn metadata(deal_id: &str, captures: usize) -> Result<DealMetadata, String> {
    DealMetadata::try_new(
        deal_id,
        ProvenanceKind::ScreenRecognition,
        Restoration::Verified,
        concat!("qmp-qemu-socket/", env!("CARGO_PKG_VERSION")),
        &format!("Native 1920x1080 CLASSIC BLUE Pyramid; {captures} capture attempts; two-frame restoration check"),
        None,
    ).map_err(|error| format!("Invalid reconnaissance metadata: {error}"))
}


#[cfg(test)]
mod tests {
    //! Scripted fresh observations exercise authority without guest input.

    use super::*;
    use std::{cell::Cell, collections::VecDeque};
    use xsar::games::cards::{Rank, Suit};


    /// Deterministic independent standard deck, not a recognised native fixture.
    pub(super) fn deck() -> [Card; 52] {
        std::array::from_fn(|index| Card::new(Rank::ALL[index % 13], Suit::ALL[index / 13]))
    }


    /// Supported synthetic observation with all original tableau positions.
    fn observation(left: StockFace, right: StockFace) -> BoardObservation {
        BoardObservation {
            tableau: deck()[..28].try_into().unwrap(),
            left,
            right,
            undo_all: Some(PixelPoint::new(1_320, 978)),
        }
    }


    /// Initial live position has a visible LEFT and a positively empty RIGHT.
    fn initial() -> BoardObservation {
        observation(StockFace::Card(deck()[28]), StockFace::Empty)
    }


    /// Exhaustion is a recognised empty LEFT and last stock identity in RIGHT.
    fn empty() -> BoardObservation {
        observation(StockFace::Empty, StockFace::Card(deck()[51]))
    }


    /// Explicit request whose paths are never opened by this pure fake adapter.
    pub(super) fn request() -> ReconRequest {
        ReconRequest {
            profile: ReconProfileSource::File(PathBuf::from("native-profile.json")),
            output_path: PathBuf::from("new-deal.json"),
            deal_id: "synthetic-state-machine-test".to_owned(),
            initial_board_confirmed: true,
        }
    }


    /// Complete supported observation series, including independent confirmations.
    pub(super) fn happy_scenes() -> Vec<ReconScene> {
        let identities = deck();
        let mut scenes = vec![ReconScene::Board(initial()), ReconScene::Board(initial())];


        for ordinal in (1..24).step_by(2) {
            scenes.push(ReconScene::Board(observation(
                StockFace::Card(identities[28 + ordinal]),
                StockFace::Card(identities[27 + ordinal]),
            )));
        }
        scenes.extend([ReconScene::Board(empty()), ReconScene::Board(empty())]);
        scenes.extend([
            ReconScene::UndoConfirmation {
                ok: PixelPoint::new(960, 600),
            },
            ReconScene::UndoConfirmation {
                ok: PixelPoint::new(960, 600),
            },
        ]);
        scenes.extend([ReconScene::Board(initial()), ReconScene::Board(initial())]);
        scenes
    }


    /// Input-free mock with deterministic STOP and uncertain-delivery injection.
    struct FakeIo {
        /// Each item represents a separately acquired fresh observation.
        scenes: VecDeque<ReconScene>,
        /// Reserved/delivered input attempts, including an injected failure.
        inputs: Vec<ReconInput>,
        /// Number of freshly requested pre-input probes.
        probes: usize,
        /// Authority-check count before each fresh post-Undo-All, pre-OK capture request.
        confirmation_capture_checks: Vec<usize>,
        /// Pre-OK waits paired with the number of preceding modal capture requests.
        confirmation_waits: Vec<(usize, Duration)>,
        /// Authority-check count immediately before each fresh post-OK capture request.
        restoration_capture_checks: Vec<usize>,
        /// Post-OK waits paired with the number of preceding capture requests.
        restoration_waits: Vec<(usize, Duration)>,
        /// Number of cooperative authority checks.
        checks: Cell<usize>,
        /// Fail exactly this authority check, without any automatic recovery.
        stop_at_check: Option<usize>,
        /// Fail after recording this one-based input delivery attempt.
        fail_input: Option<usize>,
        /// Revoke authority inside the first post-confirmation wait before any restored capture.
        stop_during_restoration_wait: bool,
        /// Simulated atomic publication refusal before creating a document.
        fail_publish: bool,
        /// Number of successful publication requests.
        published: usize,
        /// Optional real no-clobber output, used only by the filesystem integration test.
        publication_path: Option<PathBuf>,
        /// Retained diagnostic messages from recognised observations.
        logs: Vec<String>,
    }


    impl FakeIo {


        /// Create an adapter with no authority failures or external effects.
        fn new(scenes: Vec<ReconScene>) -> Self {
            Self {
                scenes: scenes.into(),
                inputs: Vec::new(),
                probes: 0,
                confirmation_capture_checks: Vec::new(),
                confirmation_waits: Vec::new(),
                restoration_capture_checks: Vec::new(),
                restoration_waits: Vec::new(),
                checks: Cell::new(0),
                stop_at_check: None,
                fail_input: None,
                stop_during_restoration_wait: false,
                fail_publish: false,
                published: 0,
                publication_path: None,
                logs: Vec::new(),
            }
        }
    }


    impl ReconIo for FakeIo {


        /// Inject STOP/context loss at an exact check boundary.
        fn check_active(&self) -> Result<(), String> {
            let count = self.checks.get() + 1;
            self.checks.set(count);


            if self.stop_at_check == Some(count) {
                return Err("STOP or selected context changed".to_owned());
            }
            Ok(())
        }


        /// Consume precisely one scripted observation, never reuse stale evidence.
        fn observe(&mut self) -> Result<ReconScene, String> {


            if matches!(self.inputs.last(), Some(ReconInput::UndoAll(_))) {
                self.confirmation_capture_checks.push(self.checks.get());
            }


            if matches!(self.inputs.last(), Some(ReconInput::Confirm(_))) {
                self.restoration_capture_checks.push(self.checks.get());
            }
            self.scenes
                .pop_front()
                .ok_or_else(|| "Scripted capture failed".to_owned())
        }


        /// Record a probe; the core must check authority again afterwards.
        fn probe(&mut self) -> Result<(), String> {
            self.probes += 1;
            Ok(())
        }


        /// Record delivery and optionally report its outcome as uncertain.
        fn input(&mut self, operation: ReconInput) -> Result<(), String> {
            self.check_active()?;
            self.inputs.push(operation);


            if self.fail_input == Some(self.inputs.len()) {
                return Err("Uncertain input delivery; do not replay".to_owned());
            }
            Ok(())
        }


        /// Simulated waits retain the contract without slowing unit tests.
        fn wait(&mut self, duration: Duration) -> Result<(), String> {
            assert!(duration == OBSERVATION_DELAY || duration == DRAW_DELAY);


            if matches!(self.inputs.last(), Some(ReconInput::UndoAll(_))) {
                self.confirmation_waits
                    .push((self.confirmation_capture_checks.len(), duration));
            }


            if matches!(self.inputs.last(), Some(ReconInput::Confirm(_))) {
                self.restoration_waits
                    .push((self.restoration_capture_checks.len(), duration));
                if self.stop_during_restoration_wait {
                    return Err("STOP during post-confirmation settling".to_owned());
                }
            }
            Ok(())
        }


        /// Retain bounded messages for ordinal and proof assertions.
        fn progress(&mut self, message: &str) {
            self.logs.push(message.to_owned());
        }


        /// A successful publisher must receive the exact original complete deck.
        fn publish(&mut self, deal: &CompleteDeal) -> Result<(), String> {
            assert_eq!(deal.tableau().as_slice(), &deck()[..28]);
            assert_eq!(deal.stock().as_slice(), &deck()[28..]);
            assert_eq!(deal.metadata().restoration(), Restoration::Verified);


            if self.fail_publish {
                return Err("Destination already exists".to_owned());
            }
            if let Some(path) = &self.publication_path {
                xsar::pyramid_json::write_new(path, deal).map_err(|error| error.to_string())?;
            }
            self.published += 1;
            Ok(())
        }
    }


    /// Complete inventory is exported only after acknowledged restoration and two positives.
    #[test]
    fn complete_pipeline_preserves_original_order_and_publishes_once() {
        let mut adapter = FakeIo::new(happy_scenes());
        let deal = run(&mut adapter, &request()).unwrap();
        assert_eq!(deal.stock().as_slice(), &deck()[28..]);
        assert_eq!(adapter.inputs.len(), 26);
        assert_eq!(&adapter.inputs[..24], &[ReconInput::Draw; 24]);
        assert_eq!(
            adapter.inputs[24],
            ReconInput::UndoAll(PixelPoint::new(1_320, 978))
        );
        assert_eq!(
            adapter.inputs[25],
            ReconInput::Confirm(PixelPoint::new(960, 600))
        );
        assert_eq!(adapter.probes, 26);
        assert_eq!(adapter.published, 1);
        assert_eq!(
            adapter
                .logs
                .iter()
                .filter(|message| message.contains("DRAW ordinal"))
                .count(),
            24
        );
        assert!(adapter.scenes.is_empty());
    }


    /// Confirm ACK is followed by one settle before two fresh original-board captures.
    #[test]
    fn acknowledged_ok_settles_before_first_fresh_restoration_capture() {
        let mut adapter = FakeIo::new(happy_scenes());
        run(&mut adapter, &request()).unwrap();
        assert_eq!(adapter.confirmation_capture_checks.len(), 2);
        assert!(adapter.confirmation_waits.is_empty());
        assert_eq!(adapter.restoration_capture_checks.len(), 2);
        assert_eq!(adapter.restoration_waits, [(0, OBSERVATION_DELAY)]);
        assert_eq!(adapter.inputs.len(), 26);
        assert_eq!(adapter.published, 1);
        assert!(adapter.scenes.is_empty());
    }


    /// STOP in the mandatory post-OK wait withholds every restoration capture and publication.
    #[test]
    fn stop_during_acknowledged_ok_settle_withholds_first_restoration_capture() {
        let mut adapter = FakeIo::new(happy_scenes());
        adapter.stop_during_restoration_wait = true;
        let failure = run(&mut adapter, &request()).expect_err("STOP during the first wait");
        assert!(
            failure
                .message
                .contains("STOP during post-confirmation settling")
        );
        assert!(!failure.progress.restored);
        assert_eq!(failure.progress.stock.len(), 24);
        assert_eq!(adapter.restoration_waits, [(0, OBSERVATION_DELAY)]);
        assert!(adapter.restoration_capture_checks.is_empty());
        assert_eq!(adapter.scenes.len(), 2);
        assert_eq!(adapter.inputs.len(), 26);
        assert_eq!(adapter.probes, 26);
        assert_eq!(
            adapter
                .inputs
                .iter()
                .filter(|input| matches!(input, ReconInput::Confirm(_)))
                .count(),
            1
        );
        assert_eq!(adapter.published, 0);
    }


    /// A recognised exhausted board permits only a delayed fresh modal retry.
    #[test]
    fn pre_ok_unchanged_board_waits_only_before_the_next_modal_observation() {
        let mut scenes = happy_scenes();
        let confirmation_start = scenes.len() - 4;
        scenes.insert(confirmation_start, ReconScene::Board(empty()));
        let mut adapter = FakeIo::new(scenes);
        run(&mut adapter, &request()).unwrap();
        assert_eq!(adapter.confirmation_capture_checks.len(), 3);
        assert_eq!(adapter.confirmation_waits, [(1, OBSERVATION_DELAY)]);
        assert_eq!(adapter.restoration_capture_checks.len(), 2);
        assert_eq!(adapter.restoration_waits, [(0, OBSERVATION_DELAY)]);
        assert_eq!(adapter.inputs.len(), 26);
        assert_eq!(adapter.probes, 26);
        assert_eq!(adapter.published, 1);
        assert!(adapter.scenes.is_empty());
    }


    /// Six unchanged post-Undo-All boards cannot authorise OK or another modal attempt.
    #[test]
    fn pre_ok_modal_budget_exhaustion_withholds_confirmation_and_export() {
        let mut scenes = happy_scenes();
        let confirmation_start = scenes.len() - 4;
        scenes.splice(
            confirmation_start..confirmation_start,
            vec![ReconScene::Board(empty()); MAX_OBSERVATIONS],
        );
        let mut adapter = FakeIo::new(scenes);
        let failure = run(&mut adapter, &request()).unwrap_err();
        assert!(
            failure
                .message
                .contains("confirmation did not appear within the observation budget")
        );
        assert!(!failure.progress.restored);
        assert_eq!(failure.progress.stock.len(), 24);
        assert_eq!(adapter.confirmation_capture_checks.len(), MAX_OBSERVATIONS);
        assert_eq!(
            adapter.confirmation_waits,
            (1..MAX_OBSERVATIONS)
                .map(|count| (count, OBSERVATION_DELAY))
                .collect::<Vec<_>>()
        );
        assert!(adapter.restoration_capture_checks.is_empty());
        assert!(adapter.restoration_waits.is_empty());
        assert_eq!(adapter.scenes.len(), 4);
        assert_eq!(adapter.inputs.len(), 25);
        assert_eq!(adapter.probes, 25);
        assert_eq!(adapter.published, 0);
    }


    /// Known old scenes delay retries and reset consecutive proof without replaying OK.
    #[test]
    fn post_ok_known_stale_scenes_delay_only_the_next_observation() {
        let mut scenes = happy_scenes();
        let restoration_start = scenes.len() - 2;
        scenes.splice(
            restoration_start..,
            [
                ReconScene::UndoConfirmation {
                    ok: PixelPoint::new(960, 600),
                },
                ReconScene::Board(initial()),
                ReconScene::Board(empty()),
                ReconScene::Board(initial()),
                ReconScene::Board(initial()),
            ],
        );
        let mut adapter = FakeIo::new(scenes);
        run(&mut adapter, &request()).unwrap();
        assert_eq!(adapter.restoration_capture_checks.len(), 5);
        assert_eq!(
            adapter.restoration_waits,
            [
                (0, OBSERVATION_DELAY),
                (1, OBSERVATION_DELAY),
                (3, OBSERVATION_DELAY)
            ]
        );
        assert_eq!(adapter.inputs.len(), 26);
        assert_eq!(adapter.probes, 26);
        assert_eq!(adapter.published, 1);
        assert!(adapter.scenes.is_empty());
    }


    /// Six stale observations exhaust restoration even if restored frames remain queued.
    #[test]
    fn restoration_budget_exhaustion_withholds_export_without_replaying_input() {
        let mut scenes = happy_scenes();
        let restoration_start = scenes.len() - 2;
        scenes.splice(
            restoration_start..restoration_start,
            vec![ReconScene::Board(empty()); MAX_OBSERVATIONS],
        );
        let mut adapter = FakeIo::new(scenes);
        let failure = run(&mut adapter, &request()).unwrap_err();
        assert!(
            failure
                .message
                .contains("did not prove original-board restoration twice")
        );
        assert!(!failure.progress.restored);
        assert_eq!(failure.progress.stock.len(), 24);
        assert_eq!(adapter.restoration_capture_checks.len(), MAX_OBSERVATIONS);
        assert_eq!(
            adapter.restoration_waits,
            (0..MAX_OBSERVATIONS)
                .map(|count| (count, OBSERVATION_DELAY))
                .collect::<Vec<_>>()
        );
        assert_eq!(adapter.scenes.len(), 2);
        assert_eq!(adapter.inputs.len(), 26);
        assert_eq!(adapter.probes, 26);
        assert_eq!(adapter.published, 0);
    }


    /// STOP before the second fresh restored capture prevents export and further input.
    #[test]
    fn stop_before_second_fresh_restored_capture_withholds_export() {
        let mut successful = FakeIo::new(happy_scenes());
        run(&mut successful, &request()).unwrap();
        let mut adapter = FakeIo::new(happy_scenes());
        adapter.stop_at_check = Some(successful.restoration_capture_checks[1]);
        let failure = run(&mut adapter, &request()).unwrap_err();
        assert!(failure.message.contains("STOP"));
        assert!(!failure.progress.restored);
        assert_eq!(failure.progress.stock.len(), 24);
        assert_eq!(adapter.restoration_capture_checks.len(), 1);
        assert_eq!(adapter.restoration_waits, [(0, OBSERVATION_DELAY)]);
        assert_eq!(adapter.scenes.len(), 1);
        assert_eq!(adapter.inputs, successful.inputs);
        assert_eq!(adapter.probes, 26);
        assert_eq!(adapter.published, 0);
    }


    /// Scripted acquisition uses the real writer/reader without claiming live recognition.
    #[test]
    fn restored_pipeline_round_trips_through_the_real_atomic_json_publisher() {
        use std::{fs, os::unix::fs::DirBuilderExt, time::SystemTime};

        let sequence = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "pyramid-pipeline-test-{}-{sequence}",
            std::process::id()
        ));
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .unwrap();
        let output = directory.join("synthetic-scripted-deal.json");
        let mut adapter = FakeIo::new(happy_scenes());
        adapter.publication_path = Some(output.clone());
        let deal = run(&mut adapter, &request()).unwrap();
        let original_bytes = fs::read(&output).unwrap();
        let document: serde_json::Value = serde_json::from_slice(&original_bytes).unwrap();
        assert_eq!(document["schema_version"], 2);
        assert_eq!(document["complete"], true);
        assert_eq!(document["restoration"], "verified");
        let rank_labels = [
            "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K",
        ];
        let suit_labels = ["clubs", "diamonds", "hearts", "spades"];


        for (section, offset, length) in [("tableau", 0, 28), ("stock", 28, 24)] {
            let cards = document[section]
                .as_array()
                .expect("ordered exported cards");
            assert_eq!(cards.len(), length, "{section} length");


            for (ordinal, record) in cards.iter().enumerate() {
                let index = offset + ordinal;
                let rank = rank_labels[index % 13];
                let suit = suit_labels[index / 13];
                let value = index % 13 + 1;
                let fields = record.as_object().expect("schema2 card object");
                assert_eq!(fields.len(), 3, "{section}[{ordinal}] field count");
                assert_eq!(
                    record,
                    &serde_json::json!({
                        "card": format!("{rank} of {suit}"), "value": value, "suit": suit,
                    }),
                    "{section}[{ordinal}] canonical identity and order"
                );
            }
        }
        let decoded = xsar::pyramid_json::read_from(fs::File::open(&output).unwrap()).unwrap();
        assert_eq!(decoded, deal);
        assert_eq!(
            xsar::pyramid_json::from_slice(&original_bytes).unwrap(),
            deal
        );
        assert_eq!(
            xsar::pyramid_json::to_vec(&decoded).unwrap(),
            original_bytes
        );
        assert_eq!(decoded.tableau().as_slice(), &deck()[..28]);
        assert_eq!(decoded.stock().as_slice(), &deck()[28..]);
        assert!(
            decoded
                .metadata()
                .source()
                .contains("; 20 capture attempts;")
        );
        assert_eq!(decoded.metadata().restoration(), Restoration::Verified);
        assert_eq!(adapter.inputs.len(), 26);
        assert_eq!(&adapter.inputs[..24], &[ReconInput::Draw; 24]);
        assert_eq!(
            adapter.inputs[24],
            ReconInput::UndoAll(PixelPoint::new(1_320, 978))
        );
        assert_eq!(
            adapter.inputs[25],
            ReconInput::Confirm(PixelPoint::new(960, 600))
        );
        assert_eq!(adapter.probes, 26);
        assert_eq!(adapter.published, 1);
        assert!(adapter.scenes.is_empty());
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        let mut duplicate = FakeIo::new(happy_scenes());
        duplicate.publication_path = Some(output.clone());
        let failure = run(&mut duplicate, &request()).unwrap_err();
        assert!(failure.progress.restored);
        assert_eq!(failure.progress.stock.len(), 24);
        assert_eq!(duplicate.inputs.len(), 26);
        assert_eq!(duplicate.published, 0);
        assert_eq!(fs::read(&output).unwrap(), original_bytes);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_file(output).unwrap();
        fs::remove_dir(directory).unwrap();
    }


    /// Mutated schema2 exports must not hide missing, extra or conflicting card fields.
    #[test]
    fn schema_two_reconnaissance_exports_reject_malformed_card_records() {
        let mut adapter = FakeIo::new(happy_scenes());
        let deal = run(&mut adapter, &request()).unwrap();
        let bytes = xsar::pyramid_json::to_vec(&deal).unwrap();
        let document: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(document["schema_version"], 2);
        assert_eq!(
            xsar::pyramid_json::read_from(bytes.as_slice()).unwrap(),
            deal
        );


        for (section, ordinal) in [("tableau", 0), ("stock", 23)] {
            let value = document[section][ordinal]["value"].as_u64().unwrap();
            let mutations = [
                ("missing label", "card", None),
                ("missing value", "value", None),
                ("missing suit", "suit", None),
                ("legacy rank field", "rank", Some(serde_json::json!("A"))),
                ("zero value", "value", Some(serde_json::json!(0))),
                ("excessive value", "value", Some(serde_json::json!(14))),
                (
                    "string value",
                    "value",
                    Some(serde_json::json!(value.to_string())),
                ),
                ("fractional value", "value", Some(serde_json::json!(1.5))),
                (
                    "conflicting value",
                    "value",
                    Some(serde_json::json!(value % 13 + 1)),
                ),
                (
                    "conflicting label",
                    "card",
                    Some(serde_json::json!("Q of clubs")),
                ),
                (
                    "conflicting suit",
                    "suit",
                    Some(serde_json::json!("diamonds")),
                ),
            ];


            for (case, field, replacement) in mutations {
                let mut malformed = document.clone();
                let record = malformed[section][ordinal].as_object_mut().unwrap();


                match replacement {
                    Some(replacement) => {
                        record.insert(field.to_owned(), replacement);
                    }
                    None => {
                        record.remove(field);
                    }
                }
                let malformed_bytes = serde_json::to_vec(&malformed).unwrap();
                assert!(
                    xsar::pyramid_json::read_from(malformed_bytes.as_slice()).is_err(),
                    "accepted {case} at {section}[{ordinal}]"
                );
            }
        }
        assert_eq!(adapter.inputs.len(), 26);
        assert_eq!(adapter.published, 1);
    }


    /// Missing consent, bad identifiers and invalid paths fail before any capture/input.
    #[test]
    fn malformed_requests_never_start_acquisition() {
        let mut cases = vec![
            request(),
            request(),
            request(),
            request(),
            request(),
            request(),
        ];
        cases[0].initial_board_confirmed = false;
        cases[1].deal_id = "bad/id".to_owned();
        cases[2].profile = ReconProfileSource::File(PathBuf::new());
        cases[3].output_path = PathBuf::from("/");
        cases[4].profile = ReconProfileSource::File(PathBuf::from("p".repeat(4_097)));
        cases[5].output_path = PathBuf::from("o".repeat(4_097));


        for case in cases {
            let mut adapter = FakeIo::new(happy_scenes());
            let failure = run(&mut adapter, &case).unwrap_err();
            assert_eq!(failure.progress.captures, 0);
            assert!(adapter.inputs.is_empty());
        }
    }


    /// Bundled calibration has no path requirement and cannot replace board consent.
    #[test]
    fn bundled_profile_request_requires_consent_and_preserves_output_bounds() {
        let mut bundled = request();
        bundled.profile = ReconProfileSource::BundledNativeV17;
        assert!(bundled.validate().is_ok());
        bundled.initial_board_confirmed = false;
        let mut adapter = FakeIo::new(happy_scenes());
        let failure = run(&mut adapter, &bundled).unwrap_err();
        assert!(failure.message.contains("Confirm an untouched"));
        assert_eq!(failure.progress.captures, 0);
        assert_eq!(adapter.probes, 0);
        assert!(adapter.inputs.is_empty());
        assert_eq!(adapter.published, 0);
        bundled.initial_board_confirmed = true;


        for output in [
            PathBuf::new(),
            PathBuf::from("/"),
            PathBuf::from("o".repeat(4_097)),
        ] {
            bundled.output_path = output;
            assert!(bundled.validate().is_err());
        }
    }


    /// Every cooperative boundary revokes further authority and withholds publication.
    #[test]
    fn stop_or_context_loss_at_every_boundary_never_publishes_or_cleans_up() {
        let mut successful = FakeIo::new(happy_scenes());
        run(&mut successful, &request()).unwrap();


        for boundary in 1..=successful.checks.get() {
            let mut adapter = FakeIo::new(happy_scenes());
            adapter.stop_at_check = Some(boundary);
            let failure = run(&mut adapter, &request()).unwrap_err();
            assert!(failure.message.contains("STOP"));
            assert_eq!(adapter.published, 0);


            if failure.progress.stock.len() < 24 {
                assert!(
                    adapter
                        .inputs
                        .iter()
                        .all(|operation| *operation == ReconInput::Draw)
                );
            }
            assert!(adapter.inputs.len() <= 26);
        }
    }


    /// The complete observed inventory remains diagnostic-only until restoration is proven.
    #[test]
    fn live_modal_refusal_inventory_is_a_unique_deck_but_not_route_input() {
        let bytes = include_bytes!("../tests/fixtures/pyramid-recon/gate3-undo-incomplete.json");
        let report: serde_json::Value =
            serde_json::from_slice(bytes).expect("retained failure diagnostic");
        let tableau = report["tableau_top_to_bottom"]
            .as_array()
            .expect("observed tableau");
        let stock = report["stock_first_draw_order"]
            .as_array()
            .expect("observed stock");
        assert_eq!(tableau.len(), 28);
        assert_eq!(stock.len(), 24);
        let observed: std::collections::BTreeSet<_> = tableau
            .iter()
            .chain(stock)
            .map(|card| card.as_str().expect("observed card label").to_owned())
            .collect();
        let expected: std::collections::BTreeSet<_> =
            deck().iter().map(ToString::to_string).collect();
        assert_eq!(observed, expected);
        assert_eq!(report["confirm_attempted"], 0);
        assert_eq!(report["restored"], false);
        assert_eq!(report["complete_output_published"], false);
        assert!(xsar::pyramid_json::from_slice(bytes).is_err());
    }


    /// No D, Undo All or OK is repeated when its delivery becomes uncertain.
    #[test]
    fn every_uncertain_input_stops_without_replay_or_cleanup() {


        for attempted in 1..=26 {
            let mut adapter = FakeIo::new(happy_scenes());
            adapter.fail_input = Some(attempted);
            let failure = run(&mut adapter, &request()).unwrap_err();
            assert!(failure.message.contains("Uncertain"));
            assert_eq!(adapter.inputs.len(), attempted);
            assert_eq!(adapter.published, 0);
            assert!(!failure.progress.restored);
        }
    }


    /// Repeated supported frames do not append cards or authorise another D.
    #[test]
    fn unchanged_frames_are_bounded_input_free_observations() {
        let scenes = vec![ReconScene::Board(initial()); 2 + MAX_OBSERVATIONS];
        let mut adapter = FakeIo::new(scenes);
        let failure = run(&mut adapter, &request()).unwrap_err();
        assert_eq!(failure.progress.stock.len(), 1);
        assert_eq!(failure.progress.draws, 1);
        assert_eq!(adapter.inputs, [ReconInput::Draw]);
        assert!(failure.message.contains("not replayed"));
    }


    /// A supported delayed advance remains eligible after input-free observation only.
    #[test]
    fn a_delayed_proven_advance_does_not_add_extra_input() {
        let mut scenes = happy_scenes();
        scenes.insert(2, ReconScene::Board(initial()));
        let mut adapter = FakeIo::new(scenes);
        run(&mut adapter, &request()).unwrap();
        assert_eq!(adapter.inputs.len(), 26);
    }


    /// A missing/ambiguous scene revokes authority even after an acknowledged D.
    #[test]
    fn unknown_scene_stops_with_partial_knowledge() {
        let mut scenes = happy_scenes();
        scenes[2] = ReconScene::Unknown;
        let mut adapter = FakeIo::new(scenes);
        let failure = run(&mut adapter, &request()).unwrap_err();
        assert_eq!(failure.progress.stock, [deck()[28]]);
        assert_eq!(adapter.inputs, [ReconInput::Draw]);
        assert_eq!(adapter.published, 0);
    }


    /// A duplicate, changed tableau, early emptiness or unproven RIGHT is not advancement.
    #[test]
    fn conflicting_acquisition_evidence_never_authorises_another_draw() {
        let mut changed = observation(StockFace::Card(deck()[29]), StockFace::Card(deck()[28]));
        changed.tableau.swap(0, 1);
        let failures = [
            observation(StockFace::Card(deck()[0]), StockFace::Card(deck()[28])),
            observation(StockFace::Empty, StockFace::Card(deck()[28])),
            observation(StockFace::Card(deck()[29]), StockFace::Empty),
            changed,
        ];


        for failed in failures {
            let mut scenes = happy_scenes();
            scenes[2] = ReconScene::Board(failed);
            let mut adapter = FakeIo::new(scenes);
            assert!(run(&mut adapter, &request()).is_err());
            assert_eq!(adapter.inputs, [ReconInput::Draw]);
            assert_eq!(adapter.published, 0);
        }
    }


    /// Initial duplicates, exhausted LEFT, occupied RIGHT and movement refuse all guest input.
    #[test]
    fn untouched_original_board_requires_unique_stable_positive_evidence() {
        let mut duplicate = initial();
        duplicate.tableau[1] = duplicate.tableau[0];
        let mut occupied = initial();
        occupied.right = StockFace::Card(deck()[29]);


        for invalid in [duplicate, occupied, empty()] {
            let mut adapter = FakeIo::new(vec![ReconScene::Board(invalid)]);
            assert!(run(&mut adapter, &request()).is_err());
            assert!(adapter.inputs.is_empty());
            assert_eq!(adapter.published, 0);
        }
        let mut scenes = happy_scenes();
        scenes[1] = ReconScene::Board(observation(StockFace::Card(deck()[29]), StockFace::Empty));
        let mut adapter = FakeIo::new(scenes);
        assert!(run(&mut adapter, &request()).is_err());
        assert!(adapter.inputs.is_empty());
    }


    /// Recognising the real exhausted failure frame never grants initial recovery input.
    #[test]
    fn native_exhausted_initial_capture_refuses_input_and_publication() {
        let fixtures =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pyramid-recon");
        let vision = vision::ReconVision::load(&fixtures.join("native-profile.json"))
            .expect("native profile");
        let bytes = std::fs::read(fixtures.join("originals/gate3-exhausted-refusal.png"))
            .expect("original failed capture");
        let frame = crate::capture::decode_png(&bytes).expect("native PNG");
        let scene = vision.observe(&frame).expect("actual exhausted board");
        assert!(matches!(scene, ReconScene::Board(_)));
        let mut adapter = FakeIo::new(vec![scene]);
        let failure = run(&mut adapter, &request()).expect_err("exhausted is not untouched");
        assert_eq!(
            failure.message,
            "Expected a recognised initial/current LEFT card"
        );
        assert!(adapter.inputs.is_empty());
        assert_eq!(adapter.published, 0);
    }


    /// A second fresh empty observation is mandatory before Undo All authority.
    #[test]
    fn one_empty_observation_cannot_authorise_undo_all() {
        let mut scenes = happy_scenes();
        scenes[15] = ReconScene::Unknown;
        let mut adapter = FakeIo::new(scenes);
        assert!(run(&mut adapter, &request()).is_err());
        assert_eq!(adapter.inputs, [ReconInput::Draw; 24]);
    }


    /// A stale/absent toolbar target or changed full dialog never receives a click.
    #[test]
    fn restoration_controls_require_fresh_positive_signatures() {
        let mut no_toolbar = empty();
        no_toolbar.undo_all = None;
        let mut scenes = happy_scenes();
        scenes[15] = ReconScene::Board(no_toolbar);
        let mut adapter = FakeIo::new(scenes);
        assert!(run(&mut adapter, &request()).is_err());
        assert_eq!(adapter.inputs.len(), 24);
        for changed in [
            ReconScene::UndoConfirmation {
                ok: PixelPoint::new(961, 600),
            },
            ReconScene::Unknown,
        ] {
            let mut scenes = happy_scenes();
            scenes[17] = changed;
            let mut adapter = FakeIo::new(scenes);
            assert!(run(&mut adapter, &request()).is_err());
            assert_eq!(adapter.inputs.len(), 25);
            assert_eq!(adapter.confirmation_capture_checks.len(), 2);
            assert!(adapter.confirmation_waits.is_empty());
            assert!(adapter.restoration_capture_checks.is_empty());
            assert!(
                !adapter
                    .inputs
                    .iter()
                    .any(|input| matches!(input, ReconInput::Confirm(_)))
            );
            assert_eq!(adapter.published, 0);
        }
    }


    /// OK acknowledgement and one restored frame cannot publish a completed deal.
    #[test]
    fn restoration_requires_two_original_tableau_and_pile_observations() {
        let mut scenes = happy_scenes();
        scenes[19] = ReconScene::Board(empty());
        let mut adapter = FakeIo::new(scenes);
        let failure = run(&mut adapter, &request()).unwrap_err();
        assert!(!failure.progress.restored);
        assert_eq!(adapter.inputs.len(), 26);
        assert_eq!(adapter.published, 0);
    }


    /// A publication refusal preserves complete knowledge and verified restoration.
    #[test]
    fn export_failure_does_not_repeat_input_or_erase_restoration_evidence() {
        let mut adapter = FakeIo::new(happy_scenes());
        adapter.fail_publish = true;
        let failure = run(&mut adapter, &request()).unwrap_err();
        assert!(failure.progress.restored);
        assert_eq!(failure.progress.stock.len(), 24);
        assert_eq!(adapter.inputs.len(), 26);
        assert_eq!(adapter.published, 0);
    }


    /// Absolute capture, input and elapsed budgets refuse operations before delivery.
    #[test]
    fn absolute_resource_budgets_fail_closed() {
        let mut adapter = FakeIo::new(happy_scenes());
        let mut runner = Runner {
            io: &mut adapter,
            started: Instant::now(),
            progress: ReconProgress {
                captures: MAX_CAPTURES,
                draws: MAX_DRAWS,
                ..ReconProgress::default()
            },
        };
        assert!(runner.capture().unwrap_err().contains("capture budget"));
        assert!(
            runner
                .perform(ReconInput::Draw)
                .unwrap_err()
                .contains("D budget")
        );
        runner.started = Instant::now() - MAX_DURATION;
        assert!(runner.active().unwrap_err().contains("deadline"));
        assert!(adapter.inputs.is_empty());
    }
}
