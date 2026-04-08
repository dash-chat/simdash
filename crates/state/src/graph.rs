use std::collections::HashMap;

use petgraph::graph::{Graph as PetGraph, NodeIndex};

use crate::NodeState;

#[derive(Debug, Clone)]
pub struct Graph<R = crate::RoutingState, E = ()> {
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

    pub fn node_index(&self, node_id: crate::NodeId) -> Option<NodeIndex> {
        self.node_indices.get(&node_id).copied()
    }
}
