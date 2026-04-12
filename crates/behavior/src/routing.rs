use simdash_ids::Behavior;
use simdash_routing::{RoutingAction, RoutingModel, RoutingState};

/// A no-op routing behavior that never triggers any routing actions.
/// Useful as a baseline to observe the initial state without propagation.
pub struct NoOpRoutingBehavior;

impl Behavior<RoutingModel> for NoOpRoutingBehavior {
    fn next(&mut self, _: &RoutingState) -> anyhow::Result<Vec<RoutingAction>> {
        Ok(vec![])
    }
}
