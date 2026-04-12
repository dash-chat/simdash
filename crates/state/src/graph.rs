use std::collections::{BTreeMap, BTreeSet, HashMap};

use petgraph::graph::{Graph as PetGraph, NodeIndex};
use polestar::ext::MapExt;
use serde::{Deserialize, Serialize};
use simdash_types::NodeId;
use simdash_routing::RoutingState;

use crate::node::NodeState;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgeState {
    pub topology_id: u32,
    pub reversed: bool,
    pub remaining_ticks: Option<u32>,
}

impl EdgeState {
    pub fn new(topology_id: u32, reversed: bool, longevity: u32) -> Self {
        Self {
            topology_id,
            reversed,
            remaining_ticks: if longevity == 0 {
                None
            } else {
                Some(longevity)
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph<R = RoutingState, E = EdgeState> {
    graph: PetGraph<(), E>,
    pub nodes: HashMap<NodeId, NodeState<R>>,
    #[serde(skip, default)]
    subnets: HashMap<String, Vec<NodeId>>,
}

impl<R, E> Default for Graph<R, E> {
    fn default() -> Self {
        Self {
            graph: PetGraph::new(),
            nodes: HashMap::new(),
            subnets: HashMap::new(),
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
        let ids = self.nodes.keys().copied();
        debug_assert_eq!(
            ids.clone().collect::<BTreeSet<_>>(),
            self.graph
                .node_indices()
                .map(|i| NodeId::from(i))
                .collect::<BTreeSet<_>>()
        );
        ids
    }

    pub fn node_state(&self, node_id: NodeId) -> Option<&NodeState<R>> {
        self.nodes.get(&node_id)
    }

    pub fn node_state_mut(&mut self, node_id: NodeId) -> Option<&mut NodeState<R>> {
        self.nodes.get_mut(&node_id)
    }

    pub fn update_node<O>(
        &mut self,
        node_id: NodeId,
        action: impl FnOnce(NodeState<R>) -> anyhow::Result<(NodeState<R>, O)>,
    ) -> anyhow::Result<O> {
        // TODO: PERF: use owned_update instead of cloned_update, but this
        //       requires never triggering an error during state transition.
        let state = self
            .nodes
            .cloned_update(node_id, |_, state| action(state))?;
        Ok(state)
    }

    pub fn add_node(&mut self, node_id: NodeId, state: NodeState<R>) {
        let expected_idx = node_id.into();
        let idx = self.graph.add_node(());
        assert_eq!(
            idx, expected_idx,
            "NodeId {node_id:?} must be added in order (got index {idx:?}, expected {expected_idx:?})"
        );
        self.nodes.insert(node_id, state);
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, weight: E) -> bool {
        if !self.nodes.contains_key(&from) || !self.nodes.contains_key(&to) {
            return false;
        }
        self.graph.add_edge(from.into(), to.into(), weight);
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

    pub fn subnets(&self) -> &HashMap<String, Vec<NodeId>> {
        &self.subnets
    }

    pub fn assert_integrity(&self) {
        assert_eq!(
            self.graph.node_count(),
            self.nodes.len(),
            "graph integrity check failed: node count mismatch"
        );

        let indices = self
            .graph
            .node_indices()
            .map(|i| NodeId::from(i))
            .collect::<BTreeSet<_>>();
        let ids = self.nodes.keys().copied().collect::<BTreeSet<_>>();
        assert_eq!(
            indices, ids,
            "graph integrity check failed: node id mismatch"
        );
    }
}
