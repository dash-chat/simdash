use simdash_routing::{
    AnyRoutingAction, AnyRoutingData, RoutingActionDispatch, RoutingModel, RoutingState,
};
use simdash_types::Behavior;

/// A no-op routing behavior that never triggers any routing actions.
/// Useful as a baseline to observe the initial state without propagation.
pub struct NoOpRoutingBehavior;

impl Behavior<RoutingModel> for NoOpRoutingBehavior {
    fn next(&mut self, _: &RoutingState) -> anyhow::Result<Vec<RoutingActionDispatch>> {
        Ok(vec![])
    }
}

/// A routing behavior that only generates actions for the Flooding protocol.
pub struct BasicRoutingBehavior;

impl Behavior<RoutingModel> for BasicRoutingBehavior {
    fn next(&mut self, state: &RoutingState) -> anyhow::Result<Vec<RoutingActionDispatch>> {
        let mut b = simdash_routing::flooding::EagerFloodingBehavior;
        Ok(b.next(&state.flooding)?
            .into_iter()
            .map(|a| {
                simdash_routing::lift_action(
                    a,
                    AnyRoutingData::Flooding,
                    AnyRoutingAction::Flooding,
                )
            })
            .collect())
    }
}
