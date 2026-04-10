use std::collections::BTreeMap;

use simdash_ids::NodeId;
use simdash_state::{Graph, node::NodeState};

/// Intermediate representation of a network topology before it is converted to a `Graph`.
pub struct NetTopology {
    pub nodes: Vec<(NodeId, NodeState)>,
    pub edges: Vec<(NodeId, NodeId)>,
    pub subnets: BTreeMap<String, Vec<NodeId>>,
}

impl NetTopology {
    pub(crate) fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            subnets: BTreeMap::new(),
        }
    }

    pub(crate) fn into_graph(self) -> Graph {
        let mut graph = Graph::new();
        for (id, state) in self.nodes {
            graph.add_node(id, state);
        }
        for (from, to) in self.edges {
            graph.add_edge(from, to, ());
        }
        for (name, node_ids) in self.subnets {
            graph.set_subnet(name, node_ids);
        }
        graph
    }
}
