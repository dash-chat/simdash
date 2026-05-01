//! Behaviors for the node and routing levels.

use simdash_routing_dispatch::RoutingDispatchModel;
use simdash_types::Behavior;

use crate::{NodeAction, NodeModel, NodeState};

/// A basic node behavior that simply generates Routing actions (no Chat actions).
#[derive(Default)]
pub struct BasicNodeBehavior<B: Behavior<Model = RoutingDispatchModel>> {
    routing_behavior: B,
}

impl<B: Behavior<Model = RoutingDispatchModel>> BasicNodeBehavior<B> {
    /// Constructor
    pub fn new(routing_behavior: B) -> Self {
        Self { routing_behavior }
    }
}

impl<B: Behavior<Model = RoutingDispatchModel>> Behavior for BasicNodeBehavior<B> {
    type Model = NodeModel;

    fn next_tick(&mut self, state: &NodeState) -> anyhow::Result<Vec<NodeAction>> {
        Ok(self
            .routing_behavior
            .next_tick(&state.routing)?
            .into_iter()
            .map(NodeAction::Routing)
            .collect())
    }
}
