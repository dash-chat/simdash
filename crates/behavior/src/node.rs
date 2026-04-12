use simdash_action::node::NodeAction;
use simdash_model::node::NodeModel;
use simdash_routing::RoutingModel;
use simdash_state::node::NodeState;

use simdash_ids::Behavior;

#[derive(Default)]
pub struct BasicRoutingBehavior<B: Behavior<RoutingModel>> {
    routing_behavior: B,
}

impl<B: Behavior<RoutingModel>> BasicRoutingBehavior<B> {
    pub fn new(routing_behavior: B) -> Self {
        Self { routing_behavior }
    }
}

impl<B: Behavior<RoutingModel>> Behavior<NodeModel> for BasicRoutingBehavior<B> {
    fn next(&mut self, state: &NodeState) -> anyhow::Result<Vec<NodeAction>> {
        Ok(self
            .routing_behavior
            .next(&state.routing)?
            .into_iter()
            .map(NodeAction::Routing)
            .collect())
    }
}
