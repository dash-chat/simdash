use simdash_action::network::NetworkAction;
use simdash_model::{network::NetworkModel, node::NodeModel};
use simdash_state::network::NetworkState;

use crate::Behavior;

pub struct RoundRobinNodeScheduler<B: Behavior<NodeModel>> {
    node_behavior: B,
}

impl<B: Behavior<NodeModel>> RoundRobinNodeScheduler<B> {
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
                    .next(&node_state)
                    .expect("todo, what to do on errors?");
                actions
                    .into_iter()
                    .map(|action| NetworkAction::Node(*node_id, action))
            })
            .chain([NetworkAction::Tick])
            .collect())
    }
}
