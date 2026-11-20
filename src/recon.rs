#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, missing_docs)]

//! Epistemic RECON coordinator for partial-observability solitaire modes
//! (TriPeaks, Klondike, Spider) controlled over QMP + guest framebuffer CV.

use thiserror::Error;
use xsar::search::{PartialProof, SearchStop};

/// Compact card representation (`0..52` standard deck encoding: `suit * 13 + rank`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Card(pub u8);

/// Error returned when converting between `xsar::games::cards::Card` and `recon::Card`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("invalid canonical card id {id} (expected 0..52)")]
pub struct InvalidCardIdError {
    /// Out-of-range canonical card identifier.
    pub id: u8,
}

impl TryFrom<xsar::games::cards::Card> for Card {
    type Error = InvalidCardIdError;

    fn try_from(card: xsar::games::cards::Card) -> Result<Self, Self::Error> {
        let suit_index: u8 = match card.suit() {
            xsar::games::cards::Suit::Clubs => 0,
            xsar::games::cards::Suit::Diamonds => 1,
            xsar::games::cards::Suit::Hearts => 2,
            xsar::games::cards::Suit::Spades => 3,
        };
        let Some(rank_index) = card.rank().value().checked_sub(1).filter(|&idx| idx < 13) else {
            return Err(InvalidCardIdError { id: u8::MAX });
        };
        let id = suit_index.saturating_mul(13).saturating_add(rank_index);
        if id >= 52 {
            return Err(InvalidCardIdError { id });
        }
        Ok(Self(id))
    }
}

impl TryFrom<Card> for xsar::games::cards::Card {
    type Error = InvalidCardIdError;

    fn try_from(card: Card) -> Result<Self, Self::Error> {
        if card.0 >= 52 {
            return Err(InvalidCardIdError { id: card.0 });
        }
        let suit = match card.0 / 13 {
            0 => xsar::games::cards::Suit::Clubs,
            1 => xsar::games::cards::Suit::Diamonds,
            2 => xsar::games::cards::Suit::Hearts,
            3 => xsar::games::cards::Suit::Spades,
            _ => return Err(InvalidCardIdError { id: card.0 }),
        };
        let Some(&rank) = xsar::games::cards::Rank::ALL.get(usize::from(card.0 % 13)) else {
            return Err(InvalidCardIdError { id: card.0 });
        };
        Ok(Self::new(rank, suit))
    }
}

/// Unique identifier for a card's initial t=0 position in the deal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotId(pub u8);

/// Epistemic state of an initial deal slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardSlot {
    /// Card identity verified via CV or 52-card set complement.
    Known(Card),
    /// Card remains face-down or occluded inside a Draw-3 packet.
    Unknown,
    /// Proved unreachable by any legal sequence of known moves.
    Unreachable,
}

/// Goal predicate allowing early exit before full board clear or full RECON.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Objective {
    /// Solve the entire board (Classic / Solve Challenge).
    ClearBoard,
    /// Clear `target` cards of `rank` (Daily / Star Club Card Challenge).
    ClearRankQuota {
        /// Target rank index (0 = Ace .. 12 = King).
        rank: u8,
        /// Remaining cards of `rank` needed from the root board state.
        remaining: u8,
    },
    /// Reach a cumulative delta score from the root board state.
    ReachScoreDelta {
        /// Points needed to complete the challenge.
        points_needed: u32,
    },
}

/// Errors surfaced during QMP RECON execution and state verification.
#[derive(Debug, Error)]
pub enum ReconError {
    /// QMP socket transport or screendump failure.
    #[error("QMP transport error during {context}: {source_msg}")]
    QmpTransport {
        /// Operation context string.
        context: &'static str,
        /// Underlying transport description.
        source_msg: String,
    },
    /// Operator cancelled or raised STOP during RECON.
    #[error("RECON operation cancelled")]
    Cancelled,
    /// QMP input or screen-state transition could not be verified deterministically.
    #[error("uncertain QMP delivery during {context}: {detail}")]
    UncertainDelivery {
        /// Operation context string.
        context: &'static str,
        /// Verification uncertainty detail.
        detail: String,
    },
    /// Scene/calibration guard rejected guest state prior to or during RECON.
    #[error("scene verification failed during RECON: {detail}")]
    SceneVerification {
        /// Mismatched scene or modal detail.
        detail: String,
    },
    /// Computer vision failed to classify an expected exposed card slot.
    #[error("CV classification failed for initial slot {slot:?}")]
    VisionClassificationFailed {
        /// The initial deal slot that should now be visible.
        slot: SlotId,
    },
    /// Undo All failed to restore the t=0 root board hash within retry budget.
    #[error("root state hash mismatch after Undo All (expected {expected:#x}, got {actual:#x})")]
    RootRestoreMismatch {
        /// Hash captured at t=0.
        expected: u64,
        /// Hash captured after Undo All.
        actual: u64,
    },
    /// A slot previously bound as `Card` revealed a different identity.
    #[error("non-deterministic deal detected at {slot:?}: {previous:?} != {observed:?}")]
    NonDeterministicSlot {
        /// Slot exhibiting drift.
        slot: SlotId,
        /// Previously recorded card.
        previous: Card,
        /// Newly observed card.
        observed: Card,
    },
    /// Same single-deck card identity appeared across distinct deal slots.
    #[error(
        "duplicate card {card:?} observed at {second_slot:?} (already bound at {first_slot:?})"
    )]
    DuplicateCard {
        /// Earlier slot already bound to `card`.
        first_slot: SlotId,
        /// Conflicting slot observed with `card`.
        second_slot: SlotId,
        /// Repeated single-deck card value.
        card: Card,
    },
    /// Safety ceiling exceeded to prevent runaway GUI loops.
    #[error("RECON probe budget exhausted after {probes} Undo All cycles")]
    ProbeBudgetExhausted {
        /// Executed probe cycles.
        probes: usize,
    },
    /// Bounded search or internal replay verification aborted fatally during RECON.
    #[error("RECON search failed during {context}: {detail}")]
    SearchFailed {
        /// Phase where search failed (e.g. `"probe planning"`, `"partial fallback"`).
        context: &'static str,
        /// Underlying invariant, allocation, or replay diagnostic.
        detail: String,
    },
}

/// Result of planning a single information-gain probe over the currently known slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbePlan<M> {
    /// Legal move sequence proved to expose at least one remaining [`CardSlot::Unknown`] slot.
    Found(Vec<M>),
    /// Exhaustive search proved no legal sequence can uncover any remaining [`CardSlot::Unknown`] slot.
    Exhausted,
    /// Search reached a resource bound (`TimeLimit`, `StateLimit`, etc.) before exhausting the graph.
    ///
    /// Unrevealed [`CardSlot::Unknown`] slots MUST remain `Unknown` (never [`CardSlot::Unreachable`]).
    Bounded(SearchStop),
}

/// Replay-verified best partial route returned when an objective cannot be solved cleanly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BestPartialRoute<M> {
    /// Ordered move sequence from `t = 0` maximising partial progress on known cards.
    pub moves: Vec<M>,
    /// Exhaustive score-maximum proof vs bounded best-found status.
    pub proof: PartialProof,
    /// `true` when at least one evaluated slot was [`CardSlot::Unknown`].
    pub has_unknown_slots: bool,
}

/// Terminal outcome of the RECON + A* orchestration loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconOutcome<M> {
    /// Objective was achieved directly on the live board during a RECON probe.
    CompletedDuringRecon {
        /// Number of probe cycles executed prior to completion.
        probes_used: usize,
    },
    /// A* found an exact winning/objective route from the restored root state.
    SolvedFromRoot {
        /// Ordered move sequence to execute from t=0.
        route: Vec<M>,
        /// Number of Undo All probe cycles required.
        probes_used: usize,
    },
    /// Probe search exhaustively proved remaining unrevealed slots unreachable; returns best partial route.
    UnsolvableBestPartial {
        /// Best partial move sequence from t=0.
        best_partial_route: Vec<M>,
        /// Count of slots that remained legally unreachable.
        unreachable_slots: usize,
        /// Whether partial-route search exhausted the known subgraph or stopped at a bound.
        proof: PartialProof,
    },
    /// Probe search hit a search bound while [`CardSlot::Unknown`] slots remain; `Unknown` slots are preserved.
    BoundedBestPartial {
        /// Best partial move sequence found over currently known cards.
        best_partial_route: Vec<M>,
        /// Count of slots remaining [`CardSlot::Unknown`].
        unknown_slots: usize,
        /// Count of slots already known to be [`CardSlot::Unreachable`].
        unreachable_slots: usize,
        /// Resource bound that stopped probe planning.
        probe_stop: SearchStop,
        /// Whether partial-route search exhausted the known-card subgraph or stopped at a bound.
        proof: PartialProof,
    },
}

/// Game-specific adapter connecting `DealKnowledge` to QMP CV and A* search.
pub trait ReconDomain {
    /// Domain-specific move type (e.g., TriPeaks tap or Klondike transfer).
    type Move: Clone + Eq;

    /// Captures t=0 visible slots, runs initial safe stock sweep + Undo All,
    /// and returns `(slot_table, root_visual_hash)`.
    fn initialize_and_sweep_stock(&mut self) -> Result<(Vec<CardSlot>, u64), ReconError>;

    /// Runs A* strictly over `Known` cards (`Unknown` treated as locked blockers).
    /// Returns `Some(route)` if `objective` is already achievable.
    fn solve_known_subgraph(
        &self,
        slots: &[CardSlot],
        objective: Objective,
    ) -> Option<Vec<Self::Move>>;

    /// Plans an information-gain legal move sequence exposing `>= 1` [`CardSlot::Unknown`] slot,
    /// distinguishes exhaustive exhaustion from resource bounds, and propagates fatal search faults.
    fn plan_info_gain_probe(&self, slots: &[CardSlot])
    -> Result<ProbePlan<Self::Move>, ReconError>;

    /// Executes `probe` via QMP, records newly flipped cards as `(SlotId, Card)`,
    /// and returns `Ok(None)` if Victory/Challenge banner intercepted execution early.
    fn execute_probe_and_observe(
        &mut self,
        probe: &[Self::Move],
    ) -> Result<Option<Vec<(SlotId, Card)>>, ReconError>;

    /// Invokes in-game Undo All and returns the post-reset board visual hash.
    fn undo_all_and_hash(&mut self) -> Result<u64, ReconError>;

    /// Computes best partial route (max cleared cards / foundation progress) with proof status.
    fn best_partial_route(
        &self,
        slots: &[CardSlot],
    ) -> Result<BestPartialRoute<Self::Move>, ReconError>;
}

/// Binds a newly observed card into `slots`, checking determinism and 52-card complement.
pub(crate) fn bind_observation(
    slots: &mut [CardSlot],
    slot: SlotId,
    card: Card,
    is_single_deck: bool,
) -> Result<(), ReconError> {
    if card.0 >= 52 {
        return Err(ReconError::VisionClassificationFailed { slot });
    }

    let idx = usize::from(slot.0);
    let current = slots
        .get(idx)
        .copied()
        .ok_or(ReconError::VisionClassificationFailed { slot })?;

    match current {
        CardSlot::Known(prev) if prev != card => {
            return Err(ReconError::NonDeterministicSlot {
                slot,
                previous: prev,
                observed: card,
            });
        }
        CardSlot::Known(_) | CardSlot::Unknown | CardSlot::Unreachable => {}
    }

    if is_single_deck
        && let Some((other_idx, _)) = slots
            .iter()
            .enumerate()
            .find(|&(other_idx, entry)| other_idx != idx && *entry == CardSlot::Known(card))
    {
        let first_idx = u8::try_from(other_idx).unwrap_or(u8::MAX);
        return Err(ReconError::DuplicateCard {
            first_slot: SlotId(first_idx),
            second_slot: slot,
            card,
        });
    }

    if let Some(target) = slots.get_mut(idx) {
        *target = CardSlot::Known(card);
    }

    if is_single_deck && slots.len() == 52 {
        let mut seen_mask: u64 = 0;
        let mut unknown_idx: Option<usize> = None;
        let mut unknown_count: usize = 0;

        for (i, entry) in slots.iter().enumerate() {
            match entry {
                CardSlot::Known(c) if c.0 < 52 => {
                    seen_mask |= 1_u64 << c.0;
                }
                CardSlot::Known(_) => {}
                CardSlot::Unknown | CardSlot::Unreachable => {
                    unknown_count = unknown_count.saturating_add(1);
                    unknown_idx = Some(i);
                }
            }
        }

        const FULL_DECK_MASK: u64 = (1_u64 << 52) - 1;
        let missing_mask = FULL_DECK_MASK & !seen_mask;
        if unknown_count == 1
            && missing_mask.count_ones() == 1
            && let Some(target_idx) = unknown_idx
            && let Ok(deduced_id) = u8::try_from(missing_mask.trailing_zeros())
            && let Some(slot_ref) = slots.get_mut(target_idx)
        {
            *slot_ref = CardSlot::Known(Card(deduced_id));
        }
    }

    Ok(())
}

/// Marks all remaining `Unknown` slots as `Unreachable` and builds the best partial outcome.
fn mark_unreachable_best_partial<D: ReconDomain>(
    domain: &D,
    slots: &mut [CardSlot],
) -> Result<ReconOutcome<D::Move>, ReconError> {
    for entry in slots.iter_mut() {
        if *entry == CardSlot::Unknown {
            *entry = CardSlot::Unreachable;
        }
    }
    let unreachable_slots = slots
        .iter()
        .filter(|&&s| s == CardSlot::Unreachable)
        .count();
    let best = domain.best_partial_route(slots)?;
    Ok(ReconOutcome::UnsolvableBestPartial {
        best_partial_route: best.moves,
        unreachable_slots,
        proof: best.proof,
    })
}

/// Preserves all remaining `Unknown` slots and builds a bounded best-partial outcome.
fn bounded_best_partial<D: ReconDomain>(
    domain: &D,
    slots: &[CardSlot],
    probe_stop: SearchStop,
) -> Result<ReconOutcome<D::Move>, ReconError> {
    let unknown_slots = slots.iter().filter(|&&s| s == CardSlot::Unknown).count();
    let unreachable_slots = slots
        .iter()
        .filter(|&&s| s == CardSlot::Unreachable)
        .count();
    let best = domain.best_partial_route(slots)?;
    Ok(ReconOutcome::BoundedBestPartial {
        best_partial_route: best.moves,
        unknown_slots,
        unreachable_slots,
        probe_stop,
        proof: best.proof,
    })
}

/// Runs the closed-loop RECON + opportunistic A* pipeline within `max_probes`.
pub fn run_epistemic_recon<D: ReconDomain>(
    domain: &mut D,
    objective: Objective,
    is_single_deck: bool,
    max_probes: usize,
) -> Result<ReconOutcome<D::Move>, ReconError> {
    let (mut slots, root_hash) = domain.initialize_and_sweep_stock()?;

    for probes_used in 0..max_probes {
        if let Some(route) = domain.solve_known_subgraph(&slots, objective) {
            return Ok(ReconOutcome::SolvedFromRoot { route, probes_used });
        }

        let probe = match domain.plan_info_gain_probe(&slots)? {
            ProbePlan::Found(probe) => probe,
            ProbePlan::Exhausted => return mark_unreachable_best_partial(domain, &mut slots),
            ProbePlan::Bounded(stop) => return bounded_best_partial(domain, &slots, stop),
        };

        let Some(observations) = domain.execute_probe_and_observe(&probe)? else {
            return Ok(ReconOutcome::CompletedDuringRecon {
                probes_used: probes_used.saturating_add(1),
            });
        };

        if observations.is_empty() {
            return Err(ReconError::ProbeBudgetExhausted {
                probes: probes_used,
            });
        }

        for (slot, card) in observations {
            bind_observation(&mut slots, slot, card, is_single_deck)?;
        }

        let restored_hash = domain.undo_all_and_hash()?;
        if restored_hash != root_hash {
            return Err(ReconError::RootRestoreMismatch {
                expected: root_hash,
                actual: restored_hash,
            });
        }
    }

    if let Some(route) = domain.solve_known_subgraph(&slots, objective) {
        return Ok(ReconOutcome::SolvedFromRoot {
            route,
            probes_used: max_probes,
        });
    }

    match domain.plan_info_gain_probe(&slots)? {
        ProbePlan::Found(_) => Err(ReconError::ProbeBudgetExhausted { probes: max_probes }),
        ProbePlan::Exhausted => mark_unreachable_best_partial(domain, &mut slots),
        ProbePlan::Bounded(stop) => bounded_best_partial(domain, &slots, stop),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    /// Outcome queued for one probe execution step in `MockDomain`.
    enum MockProbeBehavior {
        /// Return newly revealed `(SlotId, Card)` bindings.
        Observe(Vec<(SlotId, Card)>),
        /// Simulate objective completion mid-probe before root restore.
        CompleteLive,
        /// Abort probing with a domain/transport error factory.
        Fail(fn() -> ReconError),
    }

    /// Minimal deterministic domain adapter for testing `run_epistemic_recon`.
    struct MockDomain {
        /// Initial slots returned by `initialize_and_sweep_stock`.
        initial_slots: Vec<CardSlot>,
        /// Optional error to inject during initial sweep.
        init_error: Option<fn() -> ReconError>,
        /// Root visual hash captured at `t = 0`.
        root_hash: u64,
        /// Sequence of hashes returned by `undo_all_and_hash`.
        undo_hashes: VecDeque<u64>,
        /// Queued probe execution steps.
        probes: VecDeque<MockProbeBehavior>,
        /// Slot whose knowledge triggers `solve_known_subgraph`.
        solve_trigger_slot: Option<usize>,
        /// Fixed winning route returned when solved.
        winning_route: Vec<u8>,
        /// Best partial route when unsolvable.
        partial_route: Vec<u8>,
        /// Number of `undo_all_and_hash` calls observed.
        undo_calls: usize,
        /// Number of planned probes yielded before returning `Exhausted` or `Bounded`.
        planned_probes_remaining: usize,
        /// Optional stop reason returned when `planned_probes_remaining == 0`.
        probe_bound_stop: Option<SearchStop>,
        /// Optional error factory injected from `plan_info_gain_probe`.
        probe_plan_error: Option<fn() -> ReconError>,
        /// Proof flag returned by `best_partial_route`.
        partial_proof: PartialProof,
        /// Optional error factory injected from `best_partial_route`.
        partial_route_error: Option<fn() -> ReconError>,
    }

    impl MockDomain {
        /// Build a default 4-slot mock domain starting at root hash `0xCAFE`.
        fn new(initial_slots: Vec<CardSlot>) -> Self {
            Self {
                initial_slots,
                init_error: None,
                root_hash: 0xCAFE,
                undo_hashes: VecDeque::from([0xCAFE; 8]),
                probes: VecDeque::new(),
                solve_trigger_slot: None,
                winning_route: vec![1, 2, 3],
                partial_route: vec![9],
                undo_calls: 0,
                planned_probes_remaining: 4,
                probe_bound_stop: None,
                probe_plan_error: None,
                partial_proof: PartialProof::Exhaustive,
                partial_route_error: None,
            }
        }
    }

    impl ReconDomain for MockDomain {
        type Move = u8;

        fn initialize_and_sweep_stock(&mut self) -> Result<(Vec<CardSlot>, u64), ReconError> {
            if let Some(err_fn) = self.init_error {
                return Err(err_fn());
            }
            Ok((self.initial_slots.clone(), self.root_hash))
        }

        fn solve_known_subgraph(
            &self,
            slots: &[CardSlot],
            _objective: Objective,
        ) -> Option<Vec<Self::Move>> {
            let trigger = self.solve_trigger_slot?;
            matches!(slots.get(trigger), Some(CardSlot::Known(_)))
                .then(|| self.winning_route.clone())
        }

        fn plan_info_gain_probe(
            &self,
            _slots: &[CardSlot],
        ) -> Result<ProbePlan<Self::Move>, ReconError> {
            if let Some(err_fn) = self.probe_plan_error {
                return Err(err_fn());
            }
            if self.planned_probes_remaining > 0 {
                Ok(ProbePlan::Found(vec![7]))
            } else if let Some(stop) = self.probe_bound_stop {
                Ok(ProbePlan::Bounded(stop))
            } else {
                Ok(ProbePlan::Exhausted)
            }
        }

        fn execute_probe_and_observe(
            &mut self,
            _probe: &[Self::Move],
        ) -> Result<Option<Vec<(SlotId, Card)>>, ReconError> {
            self.planned_probes_remaining = self.planned_probes_remaining.saturating_sub(1);
            let Some(next) = self.probes.pop_front() else {
                return Ok(Some(Vec::new()));
            };
            match next {
                MockProbeBehavior::Observe(obs) => Ok(Some(obs)),
                MockProbeBehavior::CompleteLive => Ok(None),
                MockProbeBehavior::Fail(err_fn) => Err(err_fn()),
            }
        }

        fn undo_all_and_hash(&mut self) -> Result<u64, ReconError> {
            self.undo_calls = self.undo_calls.saturating_add(1);
            Ok(self.undo_hashes.pop_front().unwrap_or(self.root_hash))
        }

        fn best_partial_route(
            &self,
            slots: &[CardSlot],
        ) -> Result<BestPartialRoute<Self::Move>, ReconError> {
            if let Some(err_fn) = self.partial_route_error {
                return Err(err_fn());
            }
            Ok(BestPartialRoute {
                moves: self.partial_route.clone(),
                proof: self.partial_proof,
                has_unknown_slots: slots.contains(&CardSlot::Unknown),
            })
        }
    }

    /// Build a cancelled error instance for propagation checks.
    fn make_cancelled() -> ReconError {
        ReconError::Cancelled
    }

    /// Build an uncertain-delivery error instance for propagation checks.
    fn make_uncertain_delivery() -> ReconError {
        ReconError::UncertainDelivery {
            context: "stock_draw",
            detail: "waste unchanged after click".to_owned(),
        }
    }

    /// Build a scene-verification error instance for propagation checks.
    fn make_scene_verification() -> ReconError {
        ReconError::SceneVerification {
            detail: "unexpected No More Moves modal".to_owned(),
        }
    }

    /// Verify canonical 0..52 round-trips with `xsar::games::cards::Card` and invalid ID rejection.
    #[test]
    fn card_try_from_roundtrips_canonical_deck_and_rejects_out_of_range() {
        for id in 0_u8..52 {
            let recon_card = Card(id);
            let Ok(xsar_card) = xsar::games::cards::Card::try_from(recon_card) else {
                panic!("valid canonical card id {id} failed TryFrom<recon::Card>");
            };
            assert_eq!(xsar::games::freecell::rules::card_id(xsar_card), id);
            assert_eq!(Card::try_from(xsar_card), Ok(recon_card));
        }

        for invalid in [52_u8, 53, 99, 255] {
            assert_eq!(
                xsar::games::cards::Card::try_from(Card(invalid)),
                Err(InvalidCardIdError { id: invalid })
            );
        }
    }

    /// Verify that binding the 51st unique card in a single deck deduces the 52nd unknown slot.
    #[test]
    fn bind_observation_deduces_fifty_second_card_when_single_unknown_remains() {
        let mut slots = vec![CardSlot::Unknown; 52];
        for idx in 0_u8..50 {
            let card_id = (idx.saturating_add(7)) % 52;
            slots[usize::from(idx)] = CardSlot::Known(Card(card_id));
        }
        // Missing cards at this point are (50+7)%52 = 5 and (51+7)%52 = 6; leave slots 50 and 51 Unknown.
        let bind_res = bind_observation(&mut slots, SlotId(51), Card(6), true);
        assert!(bind_res.is_ok(), "unexpected error: {bind_res:?}");
        assert_eq!(slots[51], CardSlot::Known(Card(6)));
        assert_eq!(slots[50], CardSlot::Known(Card(5)));
    }

    /// Verify duplicate card across distinct slots fails closed before 51->52 complement deduction.
    #[test]
    fn bind_observation_rejects_duplicate_card_before_complement_deduction() {
        let mut slots = vec![CardSlot::Unknown; 52];
        for idx in 0_u8..50 {
            slots[usize::from(idx)] = CardSlot::Known(Card(idx));
        }

        let err = bind_observation(&mut slots, SlotId(50), Card(3), true);
        assert!(matches!(
            err,
            Err(ReconError::DuplicateCard {
                first_slot: SlotId(3),
                second_slot: SlotId(50),
                card: Card(3),
            })
        ));
        assert_eq!(slots[50], CardSlot::Unknown);
        assert_eq!(slots[51], CardSlot::Unknown);

        // Multi-deck modes (e.g., Spider) permit identical card values across distinct slots.
        assert!(bind_observation(&mut slots, SlotId(50), Card(3), false).is_ok());
    }

    /// Verify non-deterministic drift and idempotent re-observation behaviour.
    #[test]
    fn bind_observation_checks_determinism_and_slot_bounds() {
        let mut slots = vec![CardSlot::Unknown; 52];
        assert!(bind_observation(&mut slots, SlotId(4), Card(12), true).is_ok());
        assert!(bind_observation(&mut slots, SlotId(4), Card(12), true).is_ok());

        let mismatch = bind_observation(&mut slots, SlotId(4), Card(13), true);
        assert!(matches!(
            mismatch,
            Err(ReconError::NonDeterministicSlot {
                slot: SlotId(4),
                previous: Card(12),
                observed: Card(13),
            })
        ));

        let oob = bind_observation(&mut slots, SlotId(52), Card(14), true);
        assert!(matches!(
            oob,
            Err(ReconError::VisionClassificationFailed { slot: SlotId(52) })
        ));
    }

    /// Verify `run_epistemic_recon` solves after binding probe observations and restoring root.
    #[test]
    fn run_epistemic_recon_solves_from_root_after_probe() {
        let mut domain = MockDomain::new(vec![CardSlot::Unknown; 4]);
        domain.solve_trigger_slot = Some(2);
        domain
            .probes
            .push_back(MockProbeBehavior::Observe(vec![(SlotId(2), Card(19))]));

        let outcome = run_epistemic_recon(&mut domain, Objective::ClearBoard, true, 1);
        assert_eq!(
            outcome.ok(),
            Some(ReconOutcome::SolvedFromRoot {
                route: vec![1, 2, 3],
                probes_used: 1,
            })
        );
        assert_eq!(domain.undo_calls, 1);
    }

    /// Verify live objective completion during a probe exits without issuing an extra Undo All.
    #[test]
    fn run_epistemic_recon_completes_during_recon_without_undo() {
        let mut domain = MockDomain::new(vec![CardSlot::Unknown; 4]);
        domain.probes.push_back(MockProbeBehavior::CompleteLive);

        let outcome = run_epistemic_recon(&mut domain, Objective::ClearBoard, true, 3);
        assert_eq!(
            outcome.ok(),
            Some(ReconOutcome::CompletedDuringRecon { probes_used: 1 })
        );
        assert_eq!(domain.undo_calls, 0);
    }

    /// Verify unsolvable deals mark remaining `Unknown` slots as `Unreachable` and return best partial.
    #[test]
    fn run_epistemic_recon_returns_best_partial_when_no_probe_exists() {
        let mut domain = MockDomain::new(vec![
            CardSlot::Known(Card(1)),
            CardSlot::Unknown,
            CardSlot::Unknown,
            CardSlot::Known(Card(2)),
        ]);
        domain.planned_probes_remaining = 0;

        let outcome = run_epistemic_recon(&mut domain, Objective::ClearBoard, true, 3);
        assert_eq!(
            outcome.ok(),
            Some(ReconOutcome::UnsolvableBestPartial {
                best_partial_route: vec![9],
                unreachable_slots: 2,
                proof: PartialProof::Exhaustive,
            })
        );
    }

    /// Verify bounded probe planning leaves `Unknown` slots untouched and returns `BoundedBestPartial`.
    #[test]
    fn run_epistemic_recon_bounded_probe_preserves_unknown_slots_and_proof() {
        /// Stateful mock capturing the slice passed to `best_partial_route`.
        struct CapturingDomain {
            /// Underlying mock behaviour.
            inner: MockDomain,
            /// Snapshot of `slots` passed to `best_partial_route`.
            captured_slots: std::cell::RefCell<Vec<CardSlot>>,
        }

        impl ReconDomain for CapturingDomain {
            type Move = u8;

            fn initialize_and_sweep_stock(&mut self) -> Result<(Vec<CardSlot>, u64), ReconError> {
                self.inner.initialize_and_sweep_stock()
            }

            fn solve_known_subgraph(
                &self,
                slots: &[CardSlot],
                objective: Objective,
            ) -> Option<Vec<Self::Move>> {
                self.inner.solve_known_subgraph(slots, objective)
            }

            fn plan_info_gain_probe(
                &self,
                slots: &[CardSlot],
            ) -> Result<ProbePlan<Self::Move>, ReconError> {
                self.inner.plan_info_gain_probe(slots)
            }

            fn execute_probe_and_observe(
                &mut self,
                probe: &[Self::Move],
            ) -> Result<Option<Vec<(SlotId, Card)>>, ReconError> {
                self.inner.execute_probe_and_observe(probe)
            }

            fn undo_all_and_hash(&mut self) -> Result<u64, ReconError> {
                self.inner.undo_all_and_hash()
            }

            fn best_partial_route(
                &self,
                slots: &[CardSlot],
            ) -> Result<BestPartialRoute<Self::Move>, ReconError> {
                *self.captured_slots.borrow_mut() = slots.to_vec();
                self.inner.best_partial_route(slots)
            }
        }

        let mut domain = CapturingDomain {
            inner: MockDomain::new(vec![
                CardSlot::Known(Card(1)),
                CardSlot::Unknown,
                CardSlot::Unreachable,
                CardSlot::Unknown,
            ]),
            captured_slots: std::cell::RefCell::new(Vec::new()),
        };
        domain.inner.planned_probes_remaining = 0;
        domain.inner.probe_bound_stop = Some(SearchStop::TimeLimit);
        domain.inner.partial_proof = PartialProof::Bounded(SearchStop::ExpansionLimit);

        let outcome = run_epistemic_recon(&mut domain, Objective::ClearBoard, true, 3);
        assert_eq!(
            outcome.ok(),
            Some(ReconOutcome::BoundedBestPartial {
                best_partial_route: vec![9],
                unknown_slots: 2,
                unreachable_slots: 1,
                probe_stop: SearchStop::TimeLimit,
                proof: PartialProof::Bounded(SearchStop::ExpansionLimit),
            })
        );
        assert_eq!(
            domain.captured_slots.into_inner(),
            vec![
                CardSlot::Known(Card(1)),
                CardSlot::Unknown,
                CardSlot::Unreachable,
                CardSlot::Unknown,
            ]
        );
    }

    /// Verify fatal search errors propagate cleanly from probe planning and partial fallback.
    #[test]
    fn run_epistemic_recon_propagates_fatal_search_errors_without_undo() {
        /// Build a deterministic internal search error for fault propagation tests.
        fn make_search_failed() -> ReconError {
            ReconError::SearchFailed {
                context: "probe planning",
                detail: "allocation failure".to_owned(),
            }
        }

        let mut probe_err_domain = MockDomain::new(vec![CardSlot::Unknown; 4]);
        probe_err_domain.probe_plan_error = Some(make_search_failed);
        let err = run_epistemic_recon(&mut probe_err_domain, Objective::ClearBoard, true, 2);
        assert!(matches!(
            err,
            Err(ReconError::SearchFailed {
                context: "probe planning",
                ..
            })
        ));
        assert_eq!(probe_err_domain.undo_calls, 0);

        let mut partial_err_domain = MockDomain::new(vec![CardSlot::Unknown; 4]);
        partial_err_domain.planned_probes_remaining = 0;
        partial_err_domain.partial_route_error = Some(make_search_failed);
        let partial_err =
            run_epistemic_recon(&mut partial_err_domain, Objective::ClearBoard, true, 2);
        assert!(matches!(partial_err, Err(ReconError::SearchFailed { .. })));
        assert_eq!(partial_err_domain.undo_calls, 0);
    }

    /// Verify mismatched Undo All visual hashes and empty probes fail closed.
    #[test]
    fn run_epistemic_recon_detects_root_restore_mismatch_and_budget_exhaustion() {
        let mut mismatch_domain = MockDomain::new(vec![CardSlot::Unknown; 4]);
        mismatch_domain
            .probes
            .push_back(MockProbeBehavior::Observe(vec![(SlotId(0), Card(10))]));
        mismatch_domain.undo_hashes = VecDeque::from([0xDEAD_BEEF]);

        let mismatch_err =
            run_epistemic_recon(&mut mismatch_domain, Objective::ClearBoard, true, 2);
        assert!(matches!(
            mismatch_err,
            Err(ReconError::RootRestoreMismatch {
                expected: 0xCAFE,
                actual: 0xDEAD_BEEF,
            })
        ));

        let mut empty_obs_domain = MockDomain::new(vec![CardSlot::Unknown; 4]);
        empty_obs_domain
            .probes
            .push_back(MockProbeBehavior::Observe(Vec::new()));
        let empty_err = run_epistemic_recon(&mut empty_obs_domain, Objective::ClearBoard, true, 2);
        assert!(matches!(
            empty_err,
            Err(ReconError::ProbeBudgetExhausted { probes: 0 })
        ));
    }

    /// Verify `Cancelled`, `UncertainDelivery`, and `SceneVerification` propagate without Undo All.
    #[test]
    fn run_epistemic_recon_aborts_cleanly_on_operational_errors() {
        for err_fn in [
            make_cancelled as fn() -> ReconError,
            make_uncertain_delivery,
            make_scene_verification,
        ] {
            let mut probe_domain = MockDomain::new(vec![CardSlot::Unknown; 4]);
            probe_domain
                .probes
                .push_back(MockProbeBehavior::Fail(err_fn));
            let res = run_epistemic_recon(&mut probe_domain, Objective::ClearBoard, true, 2);
            assert!(res.is_err());
            assert_eq!(probe_domain.undo_calls, 0);

            let mut init_domain = MockDomain::new(vec![CardSlot::Unknown; 4]);
            init_domain.init_error = Some(err_fn);
            let init_res = run_epistemic_recon(&mut init_domain, Objective::ClearBoard, true, 2);
            assert!(init_res.is_err());
            assert_eq!(init_domain.undo_calls, 0);
        }
    }

    /// Verify invalid `Card(>= 52)` ids and wide `> 255`-slot duplicates fail closed.
    #[test]
    fn bind_observation_rejects_invalid_card_id_and_high_index_duplicates() {
        let mut slots = vec![CardSlot::Unknown; 52];
        let invalid = bind_observation(&mut slots, SlotId(0), Card(52), true);
        assert!(matches!(
            invalid,
            Err(ReconError::VisionClassificationFailed { slot: SlotId(0) })
        ));
        assert_eq!(slots[0], CardSlot::Unknown);

        let mut wide_slots = vec![CardSlot::Unknown; 300];
        wide_slots[260] = CardSlot::Known(Card(7));
        let dup = bind_observation(&mut wide_slots, SlotId(2), Card(7), true);
        assert!(matches!(
            dup,
            Err(ReconError::DuplicateCard {
                first_slot: SlotId(u8::MAX),
                second_slot: SlotId(2),
                card: Card(7),
            })
        ));
        assert_eq!(wide_slots[2], CardSlot::Unknown);
    }

    /// Verify `max_probes` boundary returns `UnsolvableBestPartial` when no probe remains.
    #[test]
    fn run_epistemic_recon_handles_zero_budget_and_final_probe_unsolvable_exhaustion() {
        let mut zero_solved = MockDomain::new(vec![CardSlot::Known(Card(0)), CardSlot::Unknown]);
        zero_solved.solve_trigger_slot = Some(0);
        assert_eq!(
            run_epistemic_recon(&mut zero_solved, Objective::ClearBoard, true, 0).ok(),
            Some(ReconOutcome::SolvedFromRoot {
                route: vec![1, 2, 3],
                probes_used: 0,
            })
        );

        let mut zero_unsolvable = MockDomain::new(vec![CardSlot::Unknown; 3]);
        zero_unsolvable.planned_probes_remaining = 0;
        assert_eq!(
            run_epistemic_recon(&mut zero_unsolvable, Objective::ClearBoard, true, 0).ok(),
            Some(ReconOutcome::UnsolvableBestPartial {
                best_partial_route: vec![9],
                unreachable_slots: 3,
                proof: PartialProof::Exhaustive,
            })
        );

        let mut zero_bounded = MockDomain::new(vec![CardSlot::Unknown; 3]);
        zero_bounded.planned_probes_remaining = 0;
        zero_bounded.probe_bound_stop = Some(SearchStop::StateLimit);
        assert_eq!(
            run_epistemic_recon(&mut zero_bounded, Objective::ClearBoard, true, 0).ok(),
            Some(ReconOutcome::BoundedBestPartial {
                best_partial_route: vec![9],
                unknown_slots: 3,
                unreachable_slots: 0,
                probe_stop: SearchStop::StateLimit,
                proof: PartialProof::Exhaustive,
            })
        );

        let mut zero_needs_probe = MockDomain::new(vec![CardSlot::Unknown; 3]);
        zero_needs_probe.planned_probes_remaining = 1;
        assert!(matches!(
            run_epistemic_recon(&mut zero_needs_probe, Objective::ClearBoard, true, 0),
            Err(ReconError::ProbeBudgetExhausted { probes: 0 })
        ));

        let mut final_probe_exhausted = MockDomain::new(vec![CardSlot::Unknown; 4]);
        final_probe_exhausted.planned_probes_remaining = 1;
        final_probe_exhausted
            .probes
            .push_back(MockProbeBehavior::Observe(vec![(SlotId(0), Card(10))]));
        assert_eq!(
            run_epistemic_recon(&mut final_probe_exhausted, Objective::ClearBoard, true, 1).ok(),
            Some(ReconOutcome::UnsolvableBestPartial {
                best_partial_route: vec![9],
                unreachable_slots: 3,
                proof: PartialProof::Exhaustive,
            })
        );
        assert_eq!(final_probe_exhausted.undo_calls, 1);

        let mut budget_hit_with_probes_left = MockDomain::new(vec![CardSlot::Unknown; 4]);
        budget_hit_with_probes_left.planned_probes_remaining = 2;
        budget_hit_with_probes_left
            .probes
            .push_back(MockProbeBehavior::Observe(vec![(SlotId(0), Card(10))]));
        assert!(matches!(
            run_epistemic_recon(
                &mut budget_hit_with_probes_left,
                Objective::ClearBoard,
                true,
                1
            ),
            Err(ReconError::ProbeBudgetExhausted { probes: 1 })
        ));
    }
}
