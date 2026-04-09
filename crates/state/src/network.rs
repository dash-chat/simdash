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

    #[test]
    fn test_neighborhood() {
        let network = NetworkState::load_postcard("../../fixtures/topology.postcard");
        let neighborhood = network.neighborhood(NodeId::from(0));
        assert_eq!(neighborhood.collect::<Vec<_>>(), vec![15.into(), 4.into()]);
    }
}
