use simdash_action::node::NodeAction;
use simdash_model::{node::NodeModel, routing::RoutingModel};
use simdash_state::node::NodeState;

use crate::Behavior;

#[derive(Default)]
pub struct BasicRoutingBehavior<B: Behavior<RoutingModel>> {
    routing_behavior: B,
    skip_empty_nodes: bool,
}

impl<B: Behavior<RoutingModel>> BasicRoutingBehavior<B> {
    pub fn new(routing_behavior: B, skip_empty_nodes: bool) -> Self {
        Self {
            routing_behavior,
            skip_empty_nodes,
        }
    }
}

impl<B: Behavior<RoutingModel>> Behavior<NodeModel> for BasicRoutingBehavior<B> {
    fn next(&mut self, state: &NodeState) -> anyhow::Result<Option<NodeAction>> {
        if self.skip_empty_nodes && state.dash.op_store.is_empty() {
            return Ok(None);
        }
        Ok(self
            .routing_behavior
            .next(&state.routing)?
            .map(NodeAction::Routing))
    }
}
