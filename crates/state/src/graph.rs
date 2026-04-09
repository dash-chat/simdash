use std::collections::BTreeMap;

use petgraph::graph::{Graph as PetGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use simdash_ids::NodeId;

use crate::node::NodeState;

fn node_id_to_index(id: NodeId) -> NodeIndex {
    NodeIndex::new(*id as usize)
}

fn node_index_to_id(idx: NodeIndex) -> NodeId {
    NodeId::from(idx.index() as u32)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph<R = crate::routing::RoutingState, E = ()> {
    graph: PetGraph<(), E>,
    node_states: BTreeMap<NodeId, NodeState<R>>,
    #[serde(skip, default)]
    subnets: BTreeMap<String, Vec<NodeId>>,
}

impl<R, E> Default for Graph<R, E> {
    fn default() -> Self {
        Self {
            graph: PetGraph::new(),
            node_states: BTreeMap::new(),
            subnets: BTreeMap::new(),
        }
    }
}

impl<R: Clone, E> Graph<R, E> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn graph(&self) -> &PetGraph<(), E> {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut PetGraph<(), E> {
        &mut self.graph
    }

    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.node_states.keys().copied()
    }

    pub fn node_id(&self, index: NodeIndex) -> Option<NodeId> {
        let id = node_index_to_id(index);
        self.node_states.contains_key(&id).then_some(id)
    }

    pub fn node_index(&self, node_id: NodeId) -> Option<NodeIndex> {
        self.node_states
            .contains_key(&node_id)
            .then(|| node_id_to_index(node_id))
    }

    pub fn node_state(&self, node_id: NodeId) -> Option<&NodeState<R>> {
        self.node_states.get(&node_id)
    }

    pub fn node_state_mut(&mut self, node_id: NodeId) -> Option<&mut NodeState<R>> {
        self.node_states.get_mut(&node_id)
    }

    pub fn set_node(&mut self, node_id: NodeId, state: NodeState<R>) -> bool {
        let Some(entry) = self.node_states.get_mut(&node_id) else {
            return false;
        };
        *entry = state;
        true
    }

    pub fn update_node<O>(
        &mut self,
        node_id: NodeId,
        action: impl FnOnce(NodeState<R>) -> anyhow::Result<(NodeState<R>, O)>,
    ) -> anyhow::Result<O> {
        let state = self.node_states.get(&node_id).unwrap().clone();
        let (state, fx) = action(state)?;
        self.set_node(node_id, state);
        Ok(fx)
    }

    pub fn add_node(&mut self, node_id: NodeId, state: NodeState<R>) {
        let expected_idx = node_id_to_index(node_id);
        let idx = self.graph.add_node(());
        assert_eq!(
            idx, expected_idx,
            "NodeId {node_id:?} must be added in order (got index {idx:?}, expected {expected_idx:?})"
        );
        self.node_states.insert(node_id, state);
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, weight: E) -> bool {
        if !self.node_states.contains_key(&from) || !self.node_states.contains_key(&to) {
            return false;
        }
        self.graph
            .add_edge(node_id_to_index(from), node_id_to_index(to), weight);
        true
    }

    pub fn set_subnet(&mut self, name: impl Into<String>, mut node_ids: Vec<NodeId>) {
        node_ids.sort_unstable();
        node_ids.dedup();
        self.subnets.insert(name.into(), node_ids);
    }

    pub fn subnet_node_ids(&self, name: &str) -> Option<&[NodeId]> {
        self.subnets.get(name).map(Vec::as_slice)
    }
}
