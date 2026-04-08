use std::collections::HashMap;

use petgraph::graph::{Graph as PetGraph, NodeIndex};

use crate::NodeState;

#[derive(Debug, Clone)]
pub struct Graph<R = crate::routing::RoutingState, E = ()> {
    graph: PetGraph<NodeState<R>, E>,
    node_indices: HashMap<crate::NodeId, NodeIndex>,
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

    pub fn node_ids(&self) -> impl Iterator<Item = crate::NodeId> + '_ {
        self.node_indices.keys().copied()
    }

    pub fn node_index(&self, node_id: crate::NodeId) -> Option<NodeIndex> {
        self.node_indices.get(&node_id).copied()
    }

    pub fn add_node(&mut self, node_id: crate::NodeId, state: NodeState<R>) -> NodeIndex {
        let idx = self.graph.add_node(state);
        self.node_indices.insert(node_id, idx);
        idx
    }

    pub fn add_edge(&mut self, from: crate::NodeId, to: crate::NodeId, weight: E) -> bool {
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
