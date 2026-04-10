use std::collections::BTreeMap;

use serde::Deserialize;
use simdash_ids::NodeId;
use simdash_state::{Graph, node::NodeState};

/// How link destinations are chosen.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Selection {
    Random,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DestinationSpec {
    pub selection: Selection,
    #[serde(default)]
    pub subnet: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Destination {
    Selection(Selection),
    Spec(DestinationSpec),
}

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

/// Whether a link produces a reverse edge in addition to the forward edge.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ordinality {
    #[default]
    Unidirectional,
    Bidirectional,
}

#[derive(Clone)]
pub struct EdgeTopology {
    pub from: NodeId,
    pub destination: Destination,
    pub ordinality: Ordinality,
    /// Number of ticks the link persists for. 0 means permanent.
    pub longevity: u32,
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

    pub fn into_graph(self, seed: u64) -> Graph {
        use rand::SeedableRng;
        use rand::seq::IndexedRandom;

        let NetTopology {
            nodes,
            edges,
            subnets,
        } = self;

        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let all_node_ids: Vec<NodeId> = nodes.iter().map(|n| n.id).collect();

        let mut graph = Graph::new();
        for NodeTopology { id, state, .. } in nodes {
            graph.add_node(id, state);
        }

        for EdgeTopology {
            from,
            destination,
            ordinality,
            longevity: _,
        } in edges
        {
            let pool: Vec<NodeId> = match &destination {
                Destination::Spec(DestinationSpec {
                    subnet: Some(name), ..
                }) => subnets.get(name).cloned().unwrap_or_default(),
                _ => all_node_ids.clone(),
            };
            let candidates: Vec<NodeId> = pool.into_iter().filter(|&id| id != from).collect();
            if let Some(&to) = candidates.choose(&mut rng) {
                graph.add_edge(from, to, ());
                if matches!(ordinality, Ordinality::Bidirectional) {
                    graph.add_edge(to, from, ());
                }
            }
        }

        for (name, node_ids) in subnets {
            graph.set_subnet(name, node_ids);
        }
        graph
    }
}
