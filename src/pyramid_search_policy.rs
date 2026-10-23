//! Application-owned Pyramid search limits and their diagnostic representation.
//!
//! Pyramid planning has no application record, expansion, time or byte cutoff.
//! The shared engine keeps checked reservations, arithmetic and cooperative STOP.
//! No capture or input authority is owned here.

use std::time::Duration;
use xsar::search::SearchLimits;


#[cfg(test)]
mod saved_deal_tests;


/// Continue until a result, exhaustive search, STOP or an actual engine failure.
///
/// Maximum representable allowances disable application cutoffs without reserving
/// memory or changing reusable engine defaults. Checked arithmetic and collection
/// reservation failures still stop explicitly; cancellation never exposes a route.
pub(crate) fn limits() -> SearchLimits {
    SearchLimits {
        max_states: usize::MAX,
        max_expansions: usize::MAX,
        max_memory_bytes: usize::MAX,
        max_duration: Duration::MAX,
    }
}


/// Describe the supplied policy without presenting an address-space maximum as RAM.
///
/// Null allowances mean the application imposes no cutoff for that resource.
/// Finite supplied allowances remain literal; none of these values measures RSS.
pub(crate) fn details(limits: SearchLimits) -> serde_json::Value {
    let memory_bytes = (limits.max_memory_bytes != usize::MAX).then_some(limits.max_memory_bytes);
    serde_json::json!({
        "max_memory_bytes": memory_bytes,
        "memory_policy": if memory_bytes.is_none() { "allocator_checked" } else { "application_budget" },
        "max_states": (limits.max_states != usize::MAX).then_some(limits.max_states),
        "max_expansions": (limits.max_expansions != usize::MAX).then_some(limits.max_expansions),
        "max_seconds": (limits.max_duration != Duration::MAX).then_some(limits.max_duration.as_secs_f64()),
        "memory_note": if memory_bytes.is_none() {
            "No application byte budget; checked heap reservations. Null work/time allowances mean no application cutoff; STOP remains cooperative."
        } else {
            "Conservative retained-storage allowance; not measured process RAM"
        },
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use xsar::search::{SearchMethod, SearchOutcome, SearchProblem, SearchStop};


    /// Deterministic graph retaining a predecessor for every state along one path.
    struct LongPath {
        /// Number of unit-cost edges required to reach the goal.
        goal: u32,
    }


    impl SearchProblem for LongPath {
        type State = u32;
        type Action = u8;
        type Successors = std::option::IntoIter<(u8, u32)>;


        /// Start at the beginning of the finite path.
        fn initial(&self) -> u32 {
            0
        }


        /// Accept exactly the final state of the finite path.
        fn is_goal(&self, state: u32) -> bool {
            state == self.goal
        }


        /// Produce one checked forward edge until the goal, then no successors.
        fn successors(&self, state: u32) -> Self::Successors {
            state
                .checked_add(1)
                .filter(|next| *next <= self.goal)
                .map(|next| (1, next))
                .into_iter()
        }
    }


    /// Exercise real heap growth and route reconstruction beyond the former cutoff.
    #[test]
    fn application_policy_passes_former_state_and_expansion_ceilings() {
        let graph = LongPath { goal: 2_000_001 };
        let cancel = AtomicBool::new(false);
        let outcome = xsar::search::solve(&graph, limits(), &cancel, SearchMethod::AStar);
        let SearchOutcome::Solved { actions, stats } = outcome else {
            panic!("application planning must pass the former work cutoffs: {outcome:?}");
        };
        assert_eq!(actions.len(), graph.goal as usize);
        assert!(actions.iter().all(|action| *action == 1));
        assert_eq!(stats.states, graph.goal as usize + 1);
        assert!(stats.expanded > SearchLimits::default().max_expansions);
    }


    /// Explicit test/caller bounds remain supported by the unchanged shared engine.
    #[test]
    fn work_limit_remains_distinct_from_memory_policy() {
        let graph = LongPath { goal: 100 };
        let bounded = SearchLimits {
            max_states: 8,
            ..limits()
        };
        let outcome = xsar::search::solve(
            &graph,
            bounded,
            &AtomicBool::new(false),
            SearchMethod::AStar,
        );
        assert!(matches!(outcome, SearchOutcome::Stopped {
            reason: SearchStop::StateLimit, stats,
        } if stats.states == 8));
    }


    /// An operator cancellation from an actual work snapshot withholds the route.
    #[test]
    fn operator_stop_during_progress_withholds_actions_under_uncapped_policy() {
        let graph = LongPath { goal: 2_000_001 };
        let cancel = AtomicBool::new(false);
        let mut snapshots = Vec::new();
        let mut progress = |snapshot: xsar::search::SearchProgress| {
            snapshots.push(snapshot);
            if snapshot.stats.expanded > 0 {
                cancel.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        };
        let outcome = xsar::search::solve_with_options(
            &graph,
            limits(),
            &cancel,
            xsar::search::SearchOptions::default(),
            Some(&mut progress),
        );
        let SearchOutcome::Stopped { reason, stats } = outcome else {
            panic!("operator STOP must withhold every route: {outcome:?}");
        };
        assert_eq!(reason, SearchStop::Cancelled);
        assert!(stats.expanded > 0);
        assert!(snapshots.first().is_some_and(|value| !value.finished));
        assert!(snapshots.last().is_some_and(|value| value.finished));
        assert!(
            snapshots
                .iter()
                .any(|value| !value.finished && value.stats.expanded > 0)
        );
    }


    /// Inspection distinguishes every absent and finite allowance without claiming RSS.
    #[test]
    fn diagnostics_distinguish_unset_and_finite_byte_allowances() {
        let unbounded = details(limits());
        assert!(unbounded["max_memory_bytes"].is_null());
        assert_eq!(unbounded["memory_policy"], "allocator_checked");
        assert!(unbounded["max_states"].is_null());
        assert!(unbounded["max_expansions"].is_null());
        assert!(unbounded["max_seconds"].is_null());
        assert_eq!(limits().max_duration, Duration::MAX);
        let finite = details(SearchLimits {
            max_states: 31,
            max_expansions: 29,
            max_memory_bytes: 4096,
            max_duration: Duration::from_secs(7),
        });
        assert_eq!(finite["max_memory_bytes"], 4096);
        assert_eq!(finite["memory_policy"], "application_budget");
        assert_eq!(finite["max_states"], 31);
        assert_eq!(finite["max_expansions"], 29);
        assert_eq!(finite["max_seconds"], 7.0);
    }
}
