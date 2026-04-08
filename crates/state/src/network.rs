use std::collections::BTreeMap;

use simdash_ids::NodeId;

use crate::{dash::Operation, node::NodeState};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkState {
    pub nodes: BTreeMap<NodeId, NodeState>,
    // links: ???,
}

impl NetworkState {
    pub fn send_op(&mut self, from: NodeId, to: NodeId, op: Operation) -> anyhow::Result<()> {
        todo!();
    }

    pub fn neighborhood(&self, node_id: NodeId) -> impl Iterator<Item = NodeId> {
        todo!("get neighborhood from graph");
        self.nodes.keys().copied().filter(move |id| id != &node_id)
    }
}
