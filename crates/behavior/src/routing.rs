use simdash_ids::Behavior;
use simdash_routing::{
    RoutingAction, RoutingModel, RoutingState, routing_flooding_protocol_action,
};

/// A no-op routing behavior that never triggers any routing actions.
/// Useful as a baseline to observe the initial state without propagation.
pub struct NoOpRoutingBehavior;

impl Behavior<RoutingModel> for NoOpRoutingBehavior {
    fn next(&mut self, _: &RoutingState) -> anyhow::Result<Vec<RoutingAction>> {
        Ok(vec![])
    }
}

pub struct BasicRoutingBehavior;

impl Behavior<RoutingModel> for BasicRoutingBehavior {
    fn next(&mut self, state: &RoutingState) -> anyhow::Result<Vec<RoutingAction>> {
        let mut b = simdash_routing::flooding::NaiveFloodingBehavior;
        Ok(b.next(&state.flooding)?
            .into_iter()
            .map(routing_flooding_protocol_action)
            .collect())
    }
}
