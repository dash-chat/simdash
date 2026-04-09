use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};
use simdash_ids::NodeId;

use crate::{Graph, chat::Operation, node::NodeState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkState {
    pub graph: Graph,
}

impl NetworkState {
    pub fn neighborhood(&self, node_id: NodeId) -> impl Iterator<Item = NodeId> {
        self.graph
            .graph()
            .neighbors(self.graph.node_index(node_id).unwrap())
            .map(|n| self.graph.node_id(n).unwrap())
    }

    pub fn load_postcard(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let path = path.canonicalize().unwrap();
        println!("Loading network state from {}", path.display());
        let bytes = std::fs::read(path).unwrap();
        let graph = postcard::from_bytes(&bytes).unwrap();
        Self { graph }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeState;

    fn make_network() -> NetworkState {
        //  0 -> 1 -> 2
        //  |         ^
        //  +-> 3 ----+
        let mut graph = Graph::new();
        for i in 0..4u32 {
            graph.add_node(NodeId::from(i), NodeState::default());
        }
        graph.add_edge(0.into(), 1.into(), ());
        graph.add_edge(1.into(), 2.into(), ());
        graph.add_edge(0.into(), 3.into(), ());
        graph.add_edge(3.into(), 2.into(), ());
        NetworkState { graph }
    }

    #[test]
    fn test_neighborhood() {
        let network = make_network();
        let mut neighbors: Vec<NodeId> = network.neighborhood(0.into()).collect();
        neighbors.sort();
        assert_eq!(neighbors, vec![1.into(), 3.into()]);
    }
}
