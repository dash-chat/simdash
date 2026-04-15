use simdash_action::node::NodeAction;
use simdash_model::node::NodeModel;
use simdash_routing_dispatch::RoutingDispatchModel;
use simdash_state::node::NodeState;

use simdash_types::Behavior;

/// A basic node behavior that simply generated Routing actions (no Chat actions).
#[derive(Default)]
pub struct BasicNodeBehavior<B: Behavior<RoutingDispatchModel>> {
    routing_behavior: B,
}

impl<B: Behavior<RoutingDispatchModel>> BasicNodeBehavior<B> {
    /// Constructor
    pub fn new(routing_behavior: B) -> Self {
        Self { routing_behavior }
    }
}

impl<B: Behavior<RoutingDispatchModel>> Behavior<NodeModel> for BasicNodeBehavior<B> {
    fn next(&mut self, state: &NodeState) -> anyhow::Result<Vec<NodeAction>> {
        Ok(self
            .routing_behavior
            .next(&state.routing)?
            .into_iter()
            .map(NodeAction::Routing)
            .collect())
    }
}
