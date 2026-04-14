use std::path::Path;

use rand::{SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};
use simdash_types::NodeId;

use crate::Graph;

/// Seeded RNG for topology evolution operations.
/// Stores seed + call count so the full state is serializable.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkTopologyState {
    seed: u64,
    call_count: u64,
}

impl NetworkTopologyState {
    /// Create a new NetworkTopologyState with the given random seed.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            call_count: 0,
        }
    }

    /// Returns a fresh seeded RNG for one topology operation and advances the counter.
    pub fn rng(&mut self) -> StdRng {
        let rng = StdRng::seed_from_u64(self.seed.wrapping_add(self.call_count));
        self.call_count += 1;
        rng
    }
}

/// The state of the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkState {
    /// The network graph
    pub graph: Graph,
    /// The network topology state
    pub topology: NetworkTopologyState,
}

impl NetworkState {
    /// All nodes with edges originating from the given node.
    pub fn neighborhood(&self, node_id: NodeId) -> impl Iterator<Item = NodeId> {
        self.graph
            .graph()
            .neighbors(node_id.into())
            .map(|n| n.into())
    }

    /// Load a network state from a file with postcard-serialized Graph.
    pub fn load_postcard(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let path = path.canonicalize().unwrap();
        let start_time = std::time::Instant::now();
        println!("Loading network state from {}", path.display());
        let bytes = std::fs::read(path).unwrap();
        let graph: Graph = postcard::from_bytes(&bytes).unwrap();
        graph.assert_integrity();
        println!("Loaded network state in {:.2?}", start_time.elapsed());
        Self {
            graph,
            topology: Default::default(),
        }
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
        graph.add_edge(0.into(), 1.into(), Default::default());
        graph.add_edge(1.into(), 2.into(), Default::default());
        graph.add_edge(0.into(), 3.into(), Default::default());
        graph.add_edge(3.into(), 2.into(), Default::default());
        NetworkState {
            graph,
            topology: Default::default(),
        }
    }

    #[test]
    fn test_neighborhood() {
        let network = make_network();
        let mut neighbors: Vec<NodeId> = network.neighborhood(0.into()).collect();
        neighbors.sort();
        assert_eq!(neighbors, vec![1.into(), 3.into()]);
    }
}
