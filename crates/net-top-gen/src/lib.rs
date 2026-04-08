//! Network topology generation utilities.

use serde::Deserialize;
use simdash_ids::NodeId;
use simdash_state::{Graph, NodeState};

/// Specification for a group of nodes and their outgoing links.
#[derive(Debug, Clone, Deserialize)]
pub struct NodeSpec {
    #[serde(rename = "type")]
    pub kind: String,
    pub number: u32,
    #[serde(default)]
    pub links: Vec<LinkSpec>,
}

/// Specification for links originating from a node group.
#[derive(Debug, Clone, Deserialize)]
pub struct LinkSpec {
    #[serde(rename = "type")]
    pub kind: String,
    /// Total number of links to create within the node group.
    pub number: u32,
    pub destination: Destination,
}

/// How link destinations are chosen.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Destination {
    Random,
}

pub type TopologyConfig = Vec<NodeSpec>;

/// Generates a network topology graph from a JSON configuration.
pub struct TopologyGenerator {
    config: TopologyConfig,
}

impl TopologyGenerator {
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let config = serde_json::from_str(json)?;
        Ok(Self { config })
    }

    pub fn generate(&self) -> Graph {
        use rand::seq::IndexedRandom;

        let mut graph = Graph::new();
        let mut next_id: u32 = 0;
        let mut rng = rand::rng();

        for node_spec in &self.config {
            let node_ids: Vec<NodeId> = (next_id..next_id + node_spec.number)
                .map(NodeId::from)
                .collect();
            next_id += node_spec.number;

            for &node_id in &node_ids {
                graph.add_node(node_id, NodeState::default());
            }

            if node_ids.len() < 2 {
                continue;
            }

            for link_spec in &node_spec.links {
                match link_spec.destination {
                    Destination::Random => {
                        for _ in 0..link_spec.number {
                            let from = *node_ids.choose(&mut rng).unwrap();
                            let candidates: Vec<NodeId> =
                                node_ids.iter().copied().filter(|&id| id != from).collect();
                            if let Some(&to) = candidates.choose(&mut rng) {
                                graph.add_edge(from, to, ());
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
    fn two_nodes_one_link() {
        let json = r#"[
            {
                "type": "nodes",
                "number": 2,
                "links": [
                    {
                        "type": "links",
                        "number": 1,
                        "destination": "random"
                    }
                ]
            }
        ]"#;

        let generator = TopologyGenerator::from_json(json).unwrap();
        let graph = generator.generate();

        assert_eq!(graph.graph().node_count(), 2);
        assert_eq!(graph.graph().edge_count(), 1);
    }
}
