use simdash_routing_dispatch::{
    AnyRoutingAction, AnyRoutingData, RoutingDispatchAction, RoutingDispatchModel,
    RoutingDispatchState,
};
use simdash_types::Behavior;

/// A no-op routing behavior that never triggers any routing actions.
/// Useful as a baseline to observe the initial state without propagation.
pub struct NoOpRoutingBehavior;

impl Behavior<RoutingDispatchModel> for NoOpRoutingBehavior {
    fn next(&mut self, _: &RoutingDispatchState) -> anyhow::Result<Vec<RoutingDispatchAction>> {
        Ok(vec![])
    }
}

/// A routing behavior that only generates actions for the FloodingSync protocol.
pub struct NaiveFloodingBehavior;

impl Behavior<RoutingDispatchModel> for NaiveFloodingBehavior {
    fn next(&mut self, state: &RoutingDispatchState) -> anyhow::Result<Vec<RoutingDispatchAction>> {
        let mut b = simdash_routing_dispatch::flooding::EagerFloodingBehavior;
        let actions = b
            .next(&state.flooding)?
            .into_iter()
            .map(|a| {
                simdash_routing_dispatch::lift_action(
                    a,
                    AnyRoutingData::Flooding,
                    AnyRoutingAction::Flooding,
                )
            })
            .collect();
        Ok(actions)
    }
}

/// A routing behavior that only generates actions for the FloodingSync protocol.
pub struct BasicFloodingSyncBehavior;

impl Behavior<RoutingDispatchModel> for BasicFloodingSyncBehavior {
    fn next(&mut self, state: &RoutingDispatchState) -> anyhow::Result<Vec<RoutingDispatchAction>> {
        let mut b = simdash_routing_dispatch::flooding_sync::StandardBehavior;
        let actions = b
            .next(&state.flooding_sync)?
            .into_iter()
            .map(|a| {
                simdash_routing_dispatch::lift_action(
                    a,
                    AnyRoutingData::FloodingSync,
                    AnyRoutingAction::FloodingSync,
                )
            })
            .collect();
        Ok(actions)
    }
}
