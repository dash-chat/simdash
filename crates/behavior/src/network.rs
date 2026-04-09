use simdash_action::network::NetworkAction;
use simdash_ids::NodeId;
use simdash_model::{network::NetworkModel, node::NodeModel};
use simdash_state::network::NetworkState;

use crate::Behavior;

pub struct RoundRobinNodeScheduler<B: Behavior<NodeModel>> {
    queue: Vec<NodeId>,
    node_behavior: B,
}

impl<B: Behavior<NodeModel>> RoundRobinNodeScheduler<B> {
    pub fn new(node_behavior: B) -> Self {
        Self {
            queue: Vec::new(),
            node_behavior,
        }
    }
}

impl<B: Behavior<NodeModel>> Behavior<NetworkModel> for RoundRobinNodeScheduler<B> {
    fn next(&mut self, state: &NetworkState) -> anyhow::Result<Option<NetworkAction>> {
        if let Some(node_id) = self.queue.pop() {
            let Some(node_state) = state.graph.node_state(node_id) else {
                return Ok(None);
            };
            let Some(action) = self.node_behavior.next(&node_state)? else {
                return Ok(None);
            };
            Ok(Some(NetworkAction::Node(node_id, action)))
        } else {
            // Need to refill the queue with existing nodes
            let mut ids: Vec<NodeId> = state.graph.node_ids().collect();
            ids.sort();
            self.queue = ids;
            Ok(Some(NetworkAction::Tick))
        }
    }
}
