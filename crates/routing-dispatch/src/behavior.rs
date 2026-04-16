use simdash_types::Behavior;

use crate::*;

/// A routing behavior that only generates actions for the naive Flooding protocol.
pub struct NaiveFloodingBehavior;

impl Behavior<RoutingDispatchModel> for NaiveFloodingBehavior {
    fn next(&mut self, state: &RoutingDispatchState) -> anyhow::Result<Vec<RoutingDispatchAction>> {
        let mut b = crate::flooding::EagerFloodingBehavior;
        let actions = b
            .next(&state.flooding)?
            .into_iter()
            .map(|a| crate::lift_action(a, AnyRoutingData::Flooding, AnyRoutingAction::Flooding))
            .collect();
        Ok(actions)
    }
}

/// A routing behavior that only generates actions for the FloodingSync protocol.
pub struct BasicFloodingSyncBehavior;

impl Behavior<RoutingDispatchModel> for BasicFloodingSyncBehavior {
    fn next(&mut self, state: &RoutingDispatchState) -> anyhow::Result<Vec<RoutingDispatchAction>> {
        let mut b = crate::flooding_sync::StandardBehavior;
        let actions = b
            .next(&state.flooding_sync)?
            .into_iter()
            .map(|a| {
                crate::lift_action(
                    a,
                    AnyRoutingData::FloodingSync,
                    AnyRoutingAction::FloodingSync,
                )
            })
            .collect();
        Ok(actions)
    }
}

/// A no-op routing behavior that never triggers any routing actions.
/// Useful as a baseline to observe the initial state without propagation.
pub struct NoOpRoutingBehavior;

impl Behavior<RoutingDispatchModel> for NoOpRoutingBehavior {
    fn next(&mut self, _: &RoutingDispatchState) -> anyhow::Result<Vec<RoutingDispatchAction>> {
        Ok(vec![])
    }
}
