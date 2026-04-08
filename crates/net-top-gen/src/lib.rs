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

/// How many links to create for a rule.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Count {
    /// A fixed number of links.
    Fixed(u32),
    /// A uniformly random number of links in the range `[min, max]`.
    Range { min: u32, max: u32 },
}

/// Specification for links originating from a node group.
#[derive(Debug, Clone, Deserialize)]
pub struct LinkSpec {
    pub strategy: String,
    pub count: Count,
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

#[cfg(test)]
mod test_helpers;

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
        use rand::Rng;
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
                    match link_spec.strategy.as_str() {
                        "per_node" => match link_spec.destination {
                            Destination::Random => {
                                for &node in &all_node_ids {
                                    let n = match link_spec.count {
                                        Count::Fixed(n) => n,
                                        Count::Range { min, max } => rng.random_range(min..=max),
                                    };
                                    let candidates: Vec<NodeId> = all_node_ids
                                        .iter()
                                        .copied()
                                        .filter(|&id| id != node)
                                        .collect();
                                    for _ in 0..n {
                                        if let Some(&to) = candidates.choose(&mut rng) {
                                            graph.add_edge(node, to, ());
                                        }
                                    }
                                }
                            }
                        },
                        _ => {
                            let total_links = match link_spec.count {
                                Count::Fixed(n) => n,
                                Count::Range { min, max } => rng.random_range(min..=max),
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
            }
        }

        graph
    }
}

#[cfg(test)]
mod tests {
    use super::test_helpers::{incoming_distribution, outgoing_distribution};
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
                "count": 1,
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
                "count": 1,
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

    #[test]
    fn hundred_nodes_zero_to_four_links_per_node() {
        let json = r#"[
            {
                "element": "node",
                "number": 100
            },
            {
                "element": "link",
                "strategy": "per_node",
                "count": { "min": 0, "max": 4 },
                "destination": "random"
            }
        ]"#;

        let generator = TopologyGenerator::from_json(json).unwrap();
        let graph = generator.generate(7);

        assert_eq!(graph.graph().node_count(), 100);
        assert!(graph.graph().edge_count() <= 400);

        // dist[n] = number of nodes with exactly n outgoing edges
        let out_dist = outgoing_distribution(&graph);
        assert_eq!(out_dist[0], 24);
        assert_eq!(out_dist[1], 22);
        assert_eq!(out_dist[2], 15);
        assert_eq!(out_dist[3], 16);
        assert_eq!(out_dist[4], 23);

        let inc_dist = incoming_distribution(&graph);
        // inc_dist[n] = number of nodes with exactly n incoming edges
        // (unbounded; any node may receive edges from multiple sources)
        assert_eq!(inc_dist[&0], 15);
        assert_eq!(inc_dist[&1], 28);
        assert_eq!(inc_dist[&2], 27);
        assert_eq!(inc_dist[&3], 18);
        assert_eq!(inc_dist[&4], 6);
        assert_eq!(inc_dist[&5], 4);
        assert_eq!(inc_dist[&6], 2);
    }
}
