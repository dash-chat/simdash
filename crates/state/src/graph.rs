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

impl<R: Clone, E> Graph<R, E> {
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

    pub fn node_id(&self, index: NodeIndex) -> Option<NodeId> {
        self.node_indices
            .iter()
            .find_map(|(id, idx)| if *idx == index { Some(*id) } else { None })
    }

    pub fn node_index(&self, node_id: NodeId) -> Option<NodeIndex> {
        self.node_indices.get(&node_id).copied()
    }

    pub fn node_state(&self, node_id: NodeId) -> Option<&NodeState<R>> {
        self.graph.node_weight(self.node_index(node_id)?)
    }

    pub fn set_node(&mut self, node_id: NodeId, state: NodeState<R>) -> bool {
        let Some(idx) = self.node_index(node_id) else {
            return false;
        };
        let Some(s) = self.graph.node_weight_mut(idx) else {
            return false;
        };
        *s = state;
        true
    }

    pub fn update_node<O>(
        &mut self,
        node_id: NodeId,
        action: impl FnOnce(NodeState<R>) -> anyhow::Result<(NodeState<R>, O)>,
    ) -> anyhow::Result<O> {
        let state = self.node_state(node_id).unwrap().clone();
        let (state, fx) = action(state)?;
        self.set_node(node_id, state);
        Ok(fx)
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
