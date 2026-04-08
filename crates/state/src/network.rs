use std::collections::BTreeMap;

use simdash_ids::NodeId;

use crate::{Graph, dash::Operation, node::NodeState};

#[derive(Debug, Clone)]
pub struct NetworkState {
    pub graph: Graph,
    // links: ???,
}

impl NetworkState {
    pub fn send_op(&mut self, from: NodeId, to: NodeId, op: Operation) -> anyhow::Result<()> {
        todo!();
    }

    pub fn neighborhood(&self, node_id: NodeId) -> impl Iterator<Item = NodeId> {
        self.graph
            .graph()
            .neighbors(self.graph.node_index(node_id).unwrap())
            .map(|n| self.graph.node_id(n).unwrap())
    }
}
