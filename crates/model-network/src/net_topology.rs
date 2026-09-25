#![allow(missing_docs)]

use std::collections::BTreeMap;

use crate::state::{Graph, NetworkState, NetworkTopologyState};
use serde::{Deserialize, Serialize};
use simdash_model_node::EdgeState;
use simdash_types::NodeId;

/// How link destinations are chosen.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Selection {
    Random,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationSpec {
    pub selection: Selection,
    #[serde(default)]
    pub subnet: Option<String>,
}

/// Config-only form of a destination: either a bare selection or a full spec.
/// Resolved to a `DestinationSpec` before it is stored on an `EdgeTopology`,
/// because untagged enums cannot be deserialized from postcard.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Destination {
    Selection(Selection),
    Spec(DestinationSpec),
}

#[derive(Clone)]
pub struct NodeTopology {
    pub id: NodeId,
    pub tags: Vec<String>,
}

impl NodeTopology {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            tags: Vec::new(),
        }
    }
}

/// Whether a link produces a reverse edge in addition to the forward edge.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ordinality {
    #[default]
    Unidirectional,
    Bidirectional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeTopology {
    pub id: u32,
    pub from: NodeId,
    pub destination: DestinationSpec,
    pub ordinality: Ordinality,
    /// Number of ticks the link persists for. 0 means permanent.
    pub longevity: u32,
    /// Probability [0, 1] that this link spawns if it is not already present. None means always.
    pub spawn_chance: Option<f64>,
}

impl EdgeTopology {
    /// The subnet this edge's destination is restricted to, if any.
    pub fn destination_subnet(&self) -> Option<&str> {
        self.destination.subnet.as_deref()
    }

    /// Returns the candidate destination nodes for this edge, drawn from `graph`,
    /// excluding the `from` node itself.
    pub fn destination_candidates(&self, graph: &Graph) -> Vec<NodeId> {
        let pool: Vec<NodeId> = match self.destination_subnet() {
            Some(name) => graph.subnet_node_ids(name).unwrap_or_default().to_vec(),
            None => graph.node_ids().collect(),
        };
        pool.into_iter().filter(|&id| id != self.from).collect()
    }

    /// Returns true if this edge should be spawned, based on its spawn_chance.
    pub fn should_spawn(&self, rng: &mut impl rand::Rng) -> bool {
        match self.spawn_chance {
            None => true,
            Some(chance) => rng.random::<f64>() < chance,
        }
    }

    /// Adds this edge (and its reverse if bidirectional) to `graph` with `to` as the destination.
    pub fn add_to_graph(&self, graph: &mut Graph, to: NodeId) {
        graph.add_edge(self.from, to, EdgeState::new(self.id, self.longevity));
        if matches!(self.ordinality, Ordinality::Bidirectional) {
            graph.add_edge(to, self.from, EdgeState::new(self.id, self.longevity));
        }
    }
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

    /// Returns the candidate destination nodes for a given edge topology entry,
    /// excluding the `from` node itself.
    pub fn destination_candidates(&self, edge: &EdgeTopology) -> Vec<NodeId> {
        let pool: Vec<NodeId> = match edge.destination_subnet() {
            Some(name) => self.subnets.get(name).cloned().unwrap_or_default(),
            None => self.nodes.iter().map(|n| n.id).collect(),
        };
        pool.into_iter().filter(|&id| id != edge.from).collect()
    }

    /// Build the initial `NetworkState` from this topology.
    ///
    /// Everything a transition needs later (the graph, and the edge specs used by
    /// `EvolveTopology`) is moved into the state; the rest of the topology is dropped.
    pub fn into_network_state(self, seed: u64) -> NetworkState {
        let edges = self.edges.clone();
        let graph = self.into_graph(seed);
        NetworkState {
            graph,
            topology: NetworkTopologyState::new(seed, edges),
        }
    }

    pub fn into_graph(self, seed: u64) -> Graph {
        use rand::{SeedableRng, seq::IndexedRandom};

        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let mut graph = Graph::new();
        for NodeTopology { id, .. } in &self.nodes {
            graph.add_node(*id, Default::default());
        }

        for edge in &self.edges {
            if !edge.should_spawn(&mut rng) {
                continue;
            }
            let candidates = self.destination_candidates(edge);
            if let Some(&to) = candidates.choose(&mut rng) {
                edge.add_to_graph(&mut graph, to);
            }
        }

        for (name, node_ids) in &self.subnets {
            graph.set_subnet(name.clone(), node_ids.clone());
        }
        graph
    }
}
