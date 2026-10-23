//! Replay the native false-no-route deal and exercise operator-controlled search.

use super::limits;
use std::{
    num::NonZeroU32,
    sync::atomic::{AtomicBool, Ordering},
};
use xsar::{
    games::pyramid::{
        replay,
        search::{self, PyramidCost},
    },
    search::{BestEffortOutcome, SearchMethod, SearchOptions, SearchStop},
};


/// Load the unchanged native 52-card inventory through the authoritative codec.
fn deal() -> xsar::games::pyramid::CompleteDeal {
    xsar::pyramid_json::read_from(
        &include_bytes!("../../tests/fixtures/pyramid-search-stop/deal.json")[..],
    )
    .expect("complete original native inventory")
}


/// A complete win outranks every partial removal on the actual refused board.
#[test]
fn maximum_cards_clears_all_tableau_cards_on_the_refused_native_deal() {
    let original = deal();
    let outcome = search::solve_best_effort_with_cost(
        &original,
        limits(),
        &AtomicBool::new(false),
        SearchOptions {
            method: SearchMethod::AStar,
            heuristic_weight: NonZeroU32::new(2).unwrap(),
        },
        PyramidCost::LogicalActions,
        None,
    );
    let BestEffortOutcome::Solved {
        actions,
        stats,
        minimum_proven,
    } = outcome
    else {
        panic!("Maximum cards must return a full clear on this solvable native deal: {outcome:?}");
    };
    let trace = replay::replay(&original, &actions, true).expect("independent complete-win replay");
    assert_eq!(trace.last().unwrap().remaining_tableau, 0);
    assert!(!minimum_proven);
    eprintln!(
        "Native Maximum cards: {} actions, {} histories, {} expansions, {:?}; all 28 tableau cards independently cleared",
        actions.len(),
        stats.states,
        stats.expanded,
        stats.elapsed
    );
}


/// Exact search must cross the actual former cap and stop only when asked.
#[test]
fn shortest_route_crosses_former_native_cutoff_then_obeys_operator_stop() {
    let original = deal();
    let cancel = AtomicBool::new(false);
    let mut running_after_old_cap = false;
    let mut final_snapshot = None;
    let mut progress = |snapshot: xsar::search::SearchProgress| {
        if !snapshot.finished && snapshot.stats.states > 2_000_000 {
            running_after_old_cap = true;
            cancel.store(true, Ordering::Relaxed);
        }
        if snapshot.finished {
            final_snapshot = Some(snapshot.stats);
        }
    };
    let outcome = search::solve_best_effort_with_cost(
        &original,
        limits(),
        &cancel,
        SearchOptions::default(),
        PyramidCost::CardMatches,
        Some(&mut progress),
    );
    assert!(
        running_after_old_cap,
        "actual search must remain active beyond its old cutoff"
    );
    let BestEffortOutcome::Stopped {
        reason: SearchStop::Cancelled,
        stats,
    } = outcome
    else {
        panic!("operator STOP must withhold any result, including a partial route: {outcome:?}");
    };
    assert!(stats.states > 2_000_000);
    assert_eq!(final_snapshot, Some(stats));
    eprintln!(
        "Native Shortest route: operator STOP after {} histories, {} expansions, {:?}; no forced cutoff and no executable result",
        stats.states, stats.expanded, stats.elapsed
    );
}
