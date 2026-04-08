use std::collections::HashMap;

use petgraph::graph::{Graph as PetGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use simdash_ids::NodeId;

use crate::node::NodeState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph<R = crate::routing::RoutingState, E = ()> {
    graph: PetGraph<NodeState<R>, E>,
    node_indices: HashMap<NodeId, NodeIndex>,
}

impl<R, E> Default for Graph<R, E> {
    fn default() -> Self {
        Self {
            graph: PetGraph::new(),
            node_indices: HashMap::new(),
        }
    }
}

impl<R, E> Graph<R, E> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn graph(&self) -> &PetGraph<NodeState<R>, E> {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut PetGraph<NodeState<R>, E> {
        &mut self.graph
    }

    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.node_indices.keys().copied()
    }

    pub fn node_index(&self, node_id: NodeId) -> Option<NodeIndex> {
        self.node_indices.get(&node_id).copied()
    }

    pub fn add_node(&mut self, node_id: NodeId, state: NodeState<R>) -> NodeIndex {
        let idx = self.graph.add_node(state);
        self.node_indices.insert(node_id, idx);
        idx
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, weight: E) -> bool {
        let from_idx = match self.node_indices.get(&from) {
            Some(&idx) => idx,
            None => return false,
        };
        let to_idx = match self.node_indices.get(&to) {
            Some(&idx) => idx,
            None => return false,
        };
        self.graph.add_edge(from_idx, to_idx, weight);
        true
    }
}
