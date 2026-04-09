use simdash_action::node::NodeAction;
use simdash_model::{node::NodeModel, routing::RoutingModel};
use simdash_state::node::NodeState;

use crate::Behavior;

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
    fn next(&mut self, state: &NodeState) -> anyhow::Result<Option<NodeAction>> {
        Ok(self
            .routing_behavior
            .next(&state.routing)?
            .map(NodeAction::Routing))
    }
}
