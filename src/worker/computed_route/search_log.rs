//! Bounded A* operating diagnostics shared by the visible and complete session logs.

use crate::{
    pyramid_search_policy,
    pyramid_workflow_model::{ObservedGame, RouteMethod},
};
use xsar::search::{SearchLimits, SearchProgress};


/// Minimum elapsed search time between ordinary log rows; the final row is always retained.
pub(super) const PROGRESS_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);


/// Describe the actual current calculation without conflating header facts and stop policy.
pub(super) fn parameters(
    method: RouteMethod,
    observed: ObservedGame,
    profile: &str,
    ignore_challenge: bool,
    limits: SearchLimits,
) -> String {
    let cost = if method == RouteMethod::FewestActions {
        "MATCH/KING=1, DRAW/RECYCLE=0; minimum card matches on a proved full clear"
    } else {
        "all logical actions=1; full win first; otherwise tableau removals, then fewer actions; stock Kings are not a separate objective"
    };
    format!(
        "A* operating parameters: detected={}; objective={}; method={}; weight={}; cost={cost}; rule profile=initial stock pass plus at most two RECYCLE operations; challenge targets ignored={ignore_challenge}; calibration={profile}; limits={}; STOP/context cancellation and checked allocation remain active. No numerical automatic cutoff is configured when a limit is null.",
        observed.label(),
        method.route_goal().label(),
        method.label(),
        if method == RouteMethod::Fast { 2 } else { 1 },
        pyramid_search_policy::details(limits)
    )
}


/// Logs expose actual method costs and unlimited app policy without claiming stock-King collection.
#[cfg(test)]
#[test]
fn execution_observability_search_parameters_keep_method_costs_limits_and_actual_header() {
    let observed = ObservedGame::Challenge {
        progress: crate::pyramid_progress::ChallengeProgress::new(
            crate::pyramid_progress::ChallengeKind::Boards,
            2,
            3,
        )
        .unwrap(),
        kings: false,
    };
    let exact = parameters(
        RouteMethod::FewestActions,
        observed,
        "test-profile",
        false,
        pyramid_search_policy::limits(),
    );
    for required in [
        "Boards challenge: 2/3",
        "weight=1",
        "MATCH/KING=1, DRAW/RECYCLE=0",
        "null",
        "STOP/context",
        "calibration=test-profile",
    ] {
        assert!(exact.contains(required), "{required}: {exact}");
    }
    let fast = parameters(
        RouteMethod::Fast,
        observed,
        "test-profile",
        true,
        pyramid_search_policy::limits(),
    );
    for required in [
        "weight=2",
        "all logical actions=1",
        "stock Kings are not a separate objective",
        "challenge targets ignored=true",
    ] {
        assert!(fast.contains(required), "{required}: {fast}");
    }
}


/// Format one throttled or final real engine snapshot; elapsed resets for each method.
pub(super) fn statistics(method: RouteMethod, snapshot: SearchProgress) -> String {
    let stats = snapshot.stats;
    format!(
        "A* {}: method={}; elapsed={:.3}s; expanded={}; generated={}; histories retained={}; peak frontier={}",
        if snapshot.finished {
            "search finished"
        } else {
            "searching"
        },
        method.label(),
        stats.elapsed.as_secs_f64(),
        stats.expanded,
        stats.generated,
        stats.states,
        stats.peak_frontier
    )
}
