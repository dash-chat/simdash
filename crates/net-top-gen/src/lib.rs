//! Network topology generation utilities.

use serde::Deserialize;
use simdash_ids::NodeId;
use simdash_state::{Graph, node::NodeState};

/// Specification for a group of nodes to create.
#[derive(Debug, Clone, Deserialize)]
pub struct NodeSpec {
    #[serde(default = "default_strategy")]
    pub strategy: String,
    pub number: u32,
}

/// Specification for links originating from a node group.
#[derive(Debug, Clone, Deserialize)]
pub struct LinkSpec {
    pub strategy: String,
    /// Number of links to create — interpreted as a total when `strategy` is `"total"`,
    /// or as links per node when `strategy` is `"per_node"`.
    pub number: u32,
    pub destination: Destination,
}

/// How link destinations are chosen.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Destination {
    Random,
}

fn default_strategy() -> String {
    "default".to_string()
}

/// A single rule in a topology config — either a node creation rule or a link creation rule.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "element", rename_all = "lowercase")]
pub enum Rule {
    Node(NodeSpec),
    Link(LinkSpec),
}

pub type TopologyConfig = Vec<Rule>;

/// Generates a network topology graph from a JSON configuration.
pub struct TopologyGenerator {
    config: TopologyConfig,
}

impl TopologyGenerator {
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let config = serde_json::from_str(json)?;
        Ok(Self { config })
    }

    pub fn generate(&self, seed: u64) -> Graph {
        use rand::SeedableRng;
        use rand::seq::IndexedRandom;

        let mut graph = Graph::new();
        let mut next_id: u32 = 0;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut all_node_ids: Vec<NodeId> = Vec::new();

        for rule in &self.config {
            match rule {
                Rule::Node(node_spec) => {
                    let node_ids: Vec<NodeId> = (next_id..next_id + node_spec.number)
                        .map(NodeId::from)
                        .collect();
                    next_id += node_spec.number;

                    for &node_id in &node_ids {
                        graph.add_node(node_id, NodeState::default());
                    }
                    all_node_ids.extend_from_slice(&node_ids);
                }
                Rule::Link(link_spec) => {
                    if all_node_ids.len() < 2 {
                        continue;
                    }
                    let total_links = if link_spec.strategy == "per_node" {
                        link_spec.number * all_node_ids.len() as u32
                    } else {
                        link_spec.number
                    };
                    match link_spec.destination {
                        Destination::Random => {
                            for _ in 0..total_links {
                                let from = *all_node_ids.choose(&mut rng).unwrap();
                                let candidates: Vec<NodeId> = all_node_ids
                                    .iter()
                                    .copied()
                                    .filter(|&id| id != from)
                                    .collect();
                                if let Some(&to) = candidates.choose(&mut rng) {
                                    graph.add_edge(from, to, ());
                                }
                            }
                        }
                    }
                }
            }
        }

        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_nodes_one_link_per_node() {
        let json = r#"[
            {
                "element": "node",
                "number": 2
            },
            {
                "element": "link",
                "strategy": "per_node",
                "number": 1,
                "destination": "random"
            }
        ]"#;

        let generator = TopologyGenerator::from_json(json).unwrap();
        let graph = generator.generate(0);

        assert_eq!(graph.graph().node_count(), 2);
        assert_eq!(graph.graph().edge_count(), 2);
    }

    #[test]
    fn three_nodes_one_link_per_node() {
        let json = r#"[
            {
                "element": "node",
                "number": 3
            },
            {
                "element": "link",
                "strategy": "per_node",
                "number": 1,
                "destination": "random"
            }
        ]"#;

        let generator = TopologyGenerator::from_json(json).unwrap();
        let graph1 = generator.generate(42);
        let graph2 = generator.generate(42);

        assert_eq!(graph1.graph().node_count(), 3);

        // Same seed produces identical topology.
        let edges1: Vec<_> = graph1
            .graph()
            .edge_indices()
            .map(|e| graph1.graph().edge_endpoints(e).unwrap())
            .collect();
        let edges2: Vec<_> = graph2
            .graph()
            .edge_indices()
            .map(|e| graph2.graph().edge_endpoints(e).unwrap())
            .collect();
        assert_eq!(edges1, edges2);
    }
}
