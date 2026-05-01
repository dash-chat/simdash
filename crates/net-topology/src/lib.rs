//! Network topology generation utilities.

use serde::Deserialize;
use simdash_types::NodeId;

pub mod link;
pub mod node;
pub mod tag;

pub use link::{Count, LinkSpec};
pub use node::NodeSpec;
pub use simdash_model_network::net_topology::NetTopology;
pub use tag::NodeTagSpec;

/// Shared mutable state threaded through rule application.
pub struct BuildState<'a, R: rand::Rng> {
    pub topology: &'a mut NetTopology,
    pub next_id: &'a mut u32,
    pub all_node_ids: &'a mut Vec<NodeId>,
    pub rng: &'a mut R,
}

/// A rule that knows how to apply itself to the topology being built.
pub trait ApplyRule {
    fn apply<R: rand::Rng>(&self, state: &mut BuildState<R>);
}

/// A single rule in a topology config — either a node creation rule or a link creation rule.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "element", rename_all = "lowercase")]
pub enum Rule {
    Node(NodeSpec),
    Link(LinkSpec),
    #[serde(rename = "node_tag")]
    NodeTag(NodeTagSpec),
}

impl ApplyRule for Rule {
    fn apply<R: rand::Rng>(&self, state: &mut BuildState<R>) {
        match self {
            Rule::Node(spec) => spec.apply(state),
            Rule::Link(spec) => spec.apply(state),
            Rule::NodeTag(spec) => spec.apply(state),
        }
    }
}

pub type TopologyConfig = Vec<Rule>;

#[cfg(test)]
mod test_helpers;

/// Generates a network topology graph from a JSON configuration.
pub struct TopologyGenerator {
    config: TopologyConfig,
}

impl TopologyGenerator {
    pub fn new(config: TopologyConfig) -> Self {
        Self { config }
    }

    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let config = serde_json::from_str(json)?;
        Ok(Self { config })
    }

    pub fn generate(&self, seed: u64) -> NetTopology {
        use rand::SeedableRng;

        let mut topology = NetTopology::new();
        let mut next_id: u32 = 0;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut all_node_ids: Vec<NodeId> = Vec::new();

        for rule in &self.config {
            rule.apply(&mut BuildState {
                topology: &mut topology,
                next_id: &mut next_id,
                all_node_ids: &mut all_node_ids,
                rng: &mut rng,
            });
        }

        topology
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
                "destination": {
                    "selection": "random"
                }
            }
        ]"#;

        let generator = TopologyGenerator::from_json(json).unwrap();
        let graph = generator.generate(0).into_graph(0);

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
                "destination": {
                    "selection": "random"
                }
            }
        ]"#;

        let generator = TopologyGenerator::from_json(json).unwrap();
        let graph1 = generator.generate(42).into_graph(42);
        let graph2 = generator.generate(42).into_graph(42);

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
                "destination": {
                    "selection": "random"
                }
            }
        ]"#;

        let generator = TopologyGenerator::from_json(json).unwrap();
        let graph = generator.generate(7).into_graph(7);

        assert_eq!(graph.graph().node_count(), 100);
        assert!(graph.graph().edge_count() <= 400);

        // dist[n] = number of nodes with exactly n outgoing edges
        let out_dist = outgoing_distribution(&graph);
        assert_eq!(out_dist[0], 23);
        assert_eq!(out_dist[1], 26);
        assert_eq!(out_dist[2], 18);
        assert_eq!(out_dist[3], 17);
        assert_eq!(out_dist[4], 16);

        let inc_dist = incoming_distribution(&graph);
        // inc_dist[n] = number of nodes with exactly n incoming edges
        // (unbounded; any node may receive edges from multiple sources)
        assert_eq!(inc_dist[&0], 17);
        assert_eq!(inc_dist[&1], 30);
        assert_eq!(inc_dist[&2], 27);
        assert_eq!(inc_dist[&3], 15);
        assert_eq!(inc_dist[&4], 8);
        assert_eq!(inc_dist[&5], 2);
        assert_eq!(inc_dist[&6], 1);
    }

    #[test]
    fn two_nested_node_groups() {
        let json = r#"[
            {
                "element": "node",
                "number": 2
            },
            {
                "element": "node",
                "number": 2,
                "rules": [
                   {
                        "element": "link",
                        "strategy": "per_node",
                        "count": 1,
                        "destination": {
                            "selection": "random"
                        }
                    }
                ]
            }
        ]"#;

        let generator = TopologyGenerator::from_json(json).unwrap();
        let graph = generator.generate(0).into_graph(0);

        assert_eq!(graph.graph().node_count(), 4);
        assert_eq!(graph.graph().edge_count(), 2);

        // First two nodes are isolated; only the nested subnet receives links.
        let out_dist = outgoing_distribution(&graph);
        assert_eq!(out_dist[0], 2);
        assert_eq!(out_dist[1], 2);
        assert_eq!(out_dist[2], 0);

        let inc_dist = incoming_distribution(&graph);
        assert_eq!(inc_dist.get(&0), Some(&2));
        assert_eq!(inc_dist.get(&1), Some(&2));
    }

    #[test]
    fn two_named_nested_node_subnets_with_links_to_first_subnet() {
        let json = r#"[
            {
                "element": "node",
                "subnet": "first_subnet",
                "number": 2
            },
            {
                "element": "node",
                "subnet": "second_subnet",
                "number": 2,
                "rules": [
                   {
                        "element": "link",
                        "strategy": "per_node",
                        "count": 1,
                        "destination": {
                            "selection": "random",
                            "subnet": "first_subnet"
                        }
                    }
                ]
            }
        ]"#;

        let generator = TopologyGenerator::from_json(json).unwrap();
        let graph = generator.generate(0).into_graph(0);

        assert_eq!(graph.graph().node_count(), 4);
        assert_eq!(graph.graph().edge_count(), 2);

        let out_dist = outgoing_distribution(&graph);
        assert_eq!(out_dist[0], 2);
        assert_eq!(out_dist[1], 2);
        assert_eq!(out_dist[2], 0);

        let edges: Vec<(NodeId, NodeId)> = graph
            .graph()
            .edge_indices()
            .map(|e| {
                let (from_idx, to_idx) = graph.graph().edge_endpoints(e).unwrap();
                (from_idx.into(), to_idx.into())
            })
            .collect();

        for (from, to) in edges {
            assert!(from == NodeId::from(2) || from == NodeId::from(3));
            assert!(to == NodeId::from(0) || to == NodeId::from(1));
        }

        assert_eq!(
            graph.subnet_node_ids("first_subnet"),
            Some(&[NodeId::from(0), NodeId::from(1)] as &[NodeId])
        );
        assert_eq!(
            graph.subnet_node_ids("second_subnet"),
            Some(&[NodeId::from(2), NodeId::from(3)] as &[NodeId])
        );
    }

    #[test]
    fn node_tag_applies_to_nodes_not_node_state() {
        let json = r#"[
            {
                "element": "node",
                "number": 2,
                "rules": [
                    {
                        "element": "node_tag",
                        "tag": "phone"
                    }
                ]
            },
            {
                "element": "node",
                "number": 1
            }
        ]"#;

        let generator = TopologyGenerator::from_json(json).unwrap();
        let topology = generator.generate(0);

        assert_eq!(topology.nodes.len(), 3);
        for node in &topology.nodes[..2] {
            assert!(node.tags.contains(&"phone".to_string()));
        }
        assert!(!topology.nodes[2].tags.contains(&"phone".to_string()));
    }

    #[test]
    fn bidirectional_link_creates_two_edges() {
        let json = r#"[
            {
                "element": "node",
                "number": 2
            },
            {
                "element": "link",
                "strategy": "random",
                "count": 1,
                "destination": "random",
                "ordinality": "bidirectional"
            }
        ]"#;

        let generator = TopologyGenerator::from_json(json).unwrap();
        let graph = generator.generate(0).into_graph(0);

        assert_eq!(graph.graph().node_count(), 2);
        assert_eq!(graph.graph().edge_count(), 2);
    }

    #[test]
    fn link_spawn_chance_stored_on_edge_topology() {
        let json = r#"[
            {
                "element": "node",
                "number": 2
            },
            {
                "element": "link",
                "strategy": "per_node",
                "count": 1,
                "destination": {
                    "selection": "random"
                },
                "spawn_chance": 0.5
            },
            {
                "element": "link",
                "strategy": "per_node",
                "count": 1,
                "destination": {
                    "selection": "random"
                }
            }
        ]"#;

        let generator = TopologyGenerator::from_json(json).unwrap();
        let topology = generator.generate(0);

        let with_spawn_chance: Vec<_> = topology
            .edges
            .iter()
            .filter(|e| e.spawn_chance == Some(0.5))
            .collect();
        let without_spawn_chance: Vec<_> = topology
            .edges
            .iter()
            .filter(|e| e.spawn_chance.is_none())
            .collect();

        assert_eq!(
            with_spawn_chance.len(),
            2,
            "two edges with spawn_chance 0.5 (one per node)"
        );
        assert_eq!(
            without_spawn_chance.len(),
            2,
            "two edges with no spawn_chance set (one per node)"
        );
    }

    #[test]
    fn link_longevity_stored_on_edge_topology() {
        let json = r#"[
            {
                "element": "node",
                "number": 2
            },
            {
                "element": "link",
                "strategy": "per_node",
                "count": 1,
                "destination": {
                    "selection": "random"
                },
                "longevity": 10
            },
            {
                "element": "link",
                "strategy": "per_node",
                "count": 1,
                "destination": {
                    "selection": "random"
                }
            }
        ]"#;

        let generator = TopologyGenerator::from_json(json).unwrap();
        let topology = generator.generate(0);

        // First batch of edges (longevity = 10) come before the second batch (longevity = 0).
        let finite: Vec<_> = topology
            .edges
            .iter()
            .filter(|e| e.longevity == 10)
            .collect();
        let permanent: Vec<_> = topology.edges.iter().filter(|e| e.longevity == 0).collect();

        assert_eq!(
            finite.len(),
            2,
            "two edges with longevity 10 (one per node)"
        );
        assert_eq!(
            permanent.len(),
            2,
            "two permanent edges (longevity 0, one per node)"
        );
    }
}
