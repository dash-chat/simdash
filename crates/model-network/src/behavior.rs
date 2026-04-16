//! Behaviors for the network level.

use simdash_model_node::NodeModel;
use simdash_types::Behavior;

use crate::{NetworkAction, NetworkModel, NetworkState};

/// A "fair scheduling" behavior that lets each Node in a network take one Action per tick.
///
/// This models a uniform flow of time with nodes that are perfectly identical to each other
/// in the rate that they take actions.
pub struct RoundRobinNodeScheduler<B: Behavior<NodeModel>> {
    node_behavior: B,
}

impl<B: Behavior<NodeModel>> RoundRobinNodeScheduler<B> {
    /// Constructor
    pub fn new(node_behavior: B) -> Self {
        Self { node_behavior }
    }
}

impl<B: Behavior<NodeModel>> Behavior<NetworkModel> for RoundRobinNodeScheduler<B> {
    fn next(&mut self, state: &NetworkState) -> anyhow::Result<Vec<NetworkAction>> {
        Ok(state
            .graph
            .nodes
            .iter()
            .flat_map(|(node_id, node_state)| {
                let actions = self
                    .node_behavior
                    .next(node_state)
                    .expect("todo, what to do on errors?");
                actions
                    .into_iter()
                    .map(|action| NetworkAction::Node(*node_id, action))
            })
            .chain([NetworkAction::Tick, NetworkAction::EvolveTopology])
            .collect())
    }
}
