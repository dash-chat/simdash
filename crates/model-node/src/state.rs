//! State types for the node and graph level.
//!
//! `NodeState` represents all state for a single node.
//! `Graph` and `EdgeState` live here (rather than in `simdash-model-network`)
//! so that topology generators can depend on them without creating a circular
//! dependency with the network model.

use std::collections::{HashMap, HashSet};

use petgraph::graph::Graph as PetGraph;
use polestar::ext::MapExt;
use serde::{Deserialize, Serialize};
use simdash_model_chat::ChatState;
use simdash_routing_dispatch::RoutingDispatchState;
use simdash_types::NodeId;

// ── NodeState ──────────────────────────────────────────────────────────────

/// The state of a node.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct NodeState<R = RoutingDispatchState> {
    /// The chat state of the node
    pub chat: ChatState,
    /// The routing state of the node
    pub routing: R,
}

impl NodeState<RoutingDispatchState> {
    /// Get a reference to the chat state of the node
    pub fn chat(&self) -> &ChatState {
        &self.chat
    }
}

// ── EdgeState ──────────────────────────────────────────────────────────────

/// The state of an edge in the graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgeState {
    /// The ID of the topology this edge is part of
    pub topology_id: u32,
    /// The number of ticks remaining for this edge
    pub remaining_ticks: Option<u32>,
}

impl EdgeState {
    /// Create a new EdgeState with the given topology ID and longevity.
    pub fn new(topology_id: u32, longevity: u32) -> Self {
        Self {
            topology_id,
            remaining_ticks: if longevity == 0 {
                None
            } else {
                Some(longevity)
            },
        }
    }
}

// ── Graph ──────────────────────────────────────────────────────────────────

/// A network graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph<R = RoutingDispatchState, E = EdgeState> {
    graph: PetGraph<(), E>,
    /// The nodes in the graph
    pub nodes: HashMap<NodeId, NodeState<R>>,
    /// The subnets in the graph
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
    /// Create a new Graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a reference to the graph.
    pub fn graph(&self) -> &PetGraph<(), E> {
        &self.graph
    }

    /// Get a mutable reference to the graph.
    pub fn graph_mut(&mut self) -> &mut PetGraph<(), E> {
        &mut self.graph
    }

    /// Get an iterator over the node IDs in the graph.
    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        let ids = self.nodes.keys().copied();
        debug_assert_eq!(
            ids.clone().collect::<HashSet<_>>(),
            self.graph
                .node_indices()
                .map(|i| NodeId::from(i))
                .collect::<HashSet<_>>()
        );
        ids
    }

    /// Get a reference to the state of a node.
    pub fn node_state(&self, node_id: NodeId) -> Option<&NodeState<R>> {
        self.nodes.get(&node_id)
    }

    /// Get a mutable reference to the state of a node.
    pub fn node_state_mut(&mut self, node_id: NodeId) -> Option<&mut NodeState<R>> {
        self.nodes.get_mut(&node_id)
    }

    /// Convert a petgraph NodeIndex to a NodeId.
    pub fn node_id(&self, idx: petgraph::graph::NodeIndex) -> Option<NodeId> {
        let node_id = NodeId::from(idx);
        self.nodes.contains_key(&node_id).then_some(node_id)
    }

    /// Update the state of a node with a closure, typically a state transition function.
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

    /// Add a node to the graph.
    pub fn add_node(&mut self, node_id: NodeId, state: NodeState<R>) {
        let expected_idx = node_id.into();
        let idx = self.graph.add_node(());
        assert_eq!(
            idx, expected_idx,
            "NodeId {node_id:?} must be added in order (got index {idx:?}, expected {expected_idx:?})"
        );
        self.nodes.insert(node_id, state);
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, weight: E) -> bool {
        if !self.nodes.contains_key(&from) || !self.nodes.contains_key(&to) {
            return false;
        }
        self.graph.add_edge(from.into(), to.into(), weight);
        true
    }

    /// Set a subnet in the graph, used during topology generation.
    pub fn set_subnet(&mut self, name: impl Into<String>, mut node_ids: Vec<NodeId>) {
        node_ids.sort_unstable();
        node_ids.dedup();
        self.subnets.insert(name.into(), node_ids);
    }

    /// Get the node IDs in a subnet.
    pub fn subnet_node_ids(&self, name: &str) -> Option<&[NodeId]> {
        self.subnets.get(name).map(Vec::as_slice)
    }

    /// Get a reference to the subnets in the graph.
    pub fn subnets(&self) -> &HashMap<String, Vec<NodeId>> {
        &self.subnets
    }

    /// Assert that the parts of the graph state are internally consistent.
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
            .collect::<HashSet<_>>();
        let ids = self.nodes.keys().copied().collect::<HashSet<_>>();
        assert_eq!(
            indices, ids,
            "graph integrity check failed: node id mismatch"
        );
    }
}
