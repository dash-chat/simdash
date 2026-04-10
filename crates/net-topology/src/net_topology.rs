use std::collections::BTreeMap;

use simdash_ids::NodeId;
use simdash_state::{Graph, node::NodeState};

#[derive(Clone)]
pub struct NodeTopology {
    pub id: NodeId,
    pub state: NodeState,
    pub tags: Vec<String>,
}

impl NodeTopology {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            state: NodeState::default(),
            tags: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct EdgeTopology {
    pub from: NodeId,
    pub to: NodeId,
}

/// Intermediate representation of a network topology before it is converted to a `Graph`.
#[derive(Clone)]
pub struct NetTopology {
    pub nodes: Vec<NodeTopology>,
    pub edges: Vec<EdgeTopology>,
    pub subnets: BTreeMap<String, Vec<NodeId>>,
}

impl NetTopology {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            subnets: BTreeMap::new(),
        }
    }

    pub fn into_graph(self, _seed: u64) -> Graph {
        let mut graph = Graph::new();
        for NodeTopology { id, state, .. } in self.nodes {
            graph.add_node(id, state);
        }
        for EdgeTopology { from, to } in self.edges {
            graph.add_edge(from, to, ());
        }
        for (name, node_ids) in self.subnets {
            graph.set_subnet(name, node_ids);
        }
        graph
    }
}
