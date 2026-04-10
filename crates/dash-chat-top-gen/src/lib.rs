use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use net_top_gen::NetTopology;
use rand::prelude::*;
use serde::Deserialize;
use simdash_distribution::Distribution;
use simdash_ids::{NodeId, OpId, Size};
use simdash_state::{
    chat::Body, chat::ChatState, chat::GroupId, chat::Header, chat::Operation, graph::Graph,
};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Seed for deterministic RNG.
    pub seed: u64,
    /// How many contacts each node has.
    pub contacts_per_node: Distribution,
    /// How many groups each node joins.
    pub groups_per_node: Distribution,
    /// Target number of members per group (used to estimate total group count).
    pub group_size: Distribution,
    /// How many ops in each node's op_store.
    pub ops_per_node: Distribution,
    /// Payload size (in bytes) of each operation.
    pub op_payload_size: Distribution,
    /// Storage capacity per node.
    pub node_storage: Distribution,
}

impl Config {
    /// Realistic chat-app defaults based on WhatsApp/Signal research data.
    pub fn default_from_seed(seed: u64) -> Self {
        Config {
            seed,
            // ~20 active contacts, std_dev ~8
            contacts_per_node: Distribution::normal(20.0, 8.0),
            // ~10 groups per user, std_dev ~4
            groups_per_node: Distribution::normal(10.0, 4.0),
            // median group size ~7, std_dev ~5
            group_size: Distribution::normal(7.0, 5.0),
            // default: no ops
            ops_per_node: Distribution::fixed(0.0),
            // typical message payload ~256 bytes
            op_payload_size: Distribution::normal(256.0, 50.0),
            // default node storage ~50 MB, std_dev ~10 MB
            node_storage: Distribution::normal(50_000_000.0, 10_000_000.0),
        }
    }

    /// Create a Config with realistic defaults for nodes that have been
    /// disconnected for the given duration.
    ///
    /// `ops_per_minute` controls how many messages per minute each node produces.
    /// A realistic default is ~0.017 (≈25 msgs/day).
    pub fn disconnected_for(seed: u64, duration: Duration, ops_per_minute: Distribution) -> Self {
        let minutes = duration.as_secs_f64() / 60.0;

        Config {
            ops_per_node: ops_per_minute.scale(minutes),
            ..Config::default_from_seed(seed)
        }
    }
}

/// Generates ChatState for each node in a network topology from a JSON config.
pub struct ChatStateGenerator {
    config: Config,
}

impl ChatStateGenerator {
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let config = serde_json::from_str(json)?;
        Ok(Self { config })
    }

    /// Generate a `ChatState` for each node and apply it to the graph.
    pub fn generate(&self, topology: NetTopology) -> Graph {
        let mut graph = topology.into_graph();
        let mut rng = StdRng::seed_from_u64(self.config.seed);
        let mut nodes: Vec<NodeId> = graph.node_ids().collect();
        nodes.sort();
        let n = nodes.len();
        if n == 0 {
            return graph;
        }

        // Step 1: Estimate total number of groups.
        // avg_groups_per_node * num_nodes / avg_group_size ≈ total groups
        let avg_groups_per_node = self.config.groups_per_node.sample(&mut rng).max(1);
        let avg_group_size = self.config.group_size.sample(&mut rng).max(1);
        let num_groups = ((n as u64) * avg_groups_per_node / avg_group_size).max(1);

        // Step 2: For each node, assign groups, contacts, and ops.
        let mut op_counter: u32 = 0;

        for &node in &nodes {
            // Groups
            let num_groups_for_node =
                self.config.groups_per_node.sample(&mut rng).min(num_groups) as usize;
            let groups: BTreeSet<GroupId> = (0..num_groups)
                .collect::<Vec<_>>()
                .choose_multiple(&mut rng, num_groups_for_node)
                .copied()
                .collect();

            // Contacts: pick from other nodes
            let num_contacts = self
                .config
                .contacts_per_node
                .sample(&mut rng)
                .min((n - 1) as u64) as usize;
            let others: Vec<NodeId> = nodes.iter().copied().filter(|&id| id != node).collect();
            let contacts: BTreeSet<NodeId> = others
                .choose_multiple(&mut rng, num_contacts)
                .copied()
                .collect();

            // Ops
            let num_ops = self.config.ops_per_node.sample(&mut rng) as usize;
            let mut op_store = BTreeMap::new();
            for _ in 0..num_ops {
                let op_id = OpId::from(op_counter);
                op_counter += 1;
                let payload_size = Size::from(self.config.op_payload_size.sample(&mut rng));

                // Pick a random group for the message body (or direct message if no groups)
                let body =
                    if let Some(&group_id) = groups.iter().collect::<Vec<_>>().choose(&mut rng) {
                        Body::GroupMessage {
                            group_id: *group_id,
                        }
                    } else {
                        // Fallback: direct message to a random contact or self
                        let recipient = contacts
                            .iter()
                            .collect::<Vec<_>>()
                            .choose(&mut rng)
                            .copied()
                            .copied()
                            .unwrap_or(node);
                        Body::DirectMessage { recipient }
                    };

                op_store.insert(
                    op_id,
                    Operation {
                        id: op_id,
                        header: Header {
                            author: node,
                            payload_size,
                        },
                        body,
                    },
                );
            }

            let storage_capacity = Some(Size::from(self.config.node_storage.sample(&mut rng)));
            let chat_state = ChatState {
                op_store,
                contacts,
                groups,
                storage_capacity,
            };
            graph
                .update_node(node, |mut n| {
                    n.dash = chat_state;
                    Ok((n, ()))
                })
                .ok();
        }

        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use net_top_gen::net_topology::NodeTopology;
    use simdash_state::chat::ChatState;
    use simdash_state::node::NodeState;

    fn make_topology(n: u32) -> NetTopology {
        let mut topology = NetTopology::new();
        for i in 0..n {
            topology.nodes.push(NodeTopology {
                id: NodeId::from(i),
                state: NodeState::default(),
            });
        }
        topology
    }

    fn assert_all_nodes(graph: &Graph, f: impl Fn(&ChatState)) {
        for node_id in graph.node_ids() {
            let state = graph.node_state(node_id).unwrap();
            f(&state.dash);
        }
    }

    #[test]
    fn deterministic_with_seed() {
        let config = Config {
            seed: 42,
            contacts_per_node: Distribution::fixed(3.0),
            groups_per_node: Distribution::fixed(2.0),
            group_size: Distribution::fixed(3.0),
            ops_per_node: Distribution::fixed(5.0),
            op_payload_size: Distribution::fixed(100.0),
            node_storage: Distribution::fixed(50_000_000.0),
        };

        let graph1 = ChatStateGenerator {
            config: config.clone(),
        }
        .generate(make_topology(10));
        let graph2 = ChatStateGenerator {
            config: config.clone(),
        }
        .generate(make_topology(10));

        // Same seed produces same chat states
        for node_id in graph1.node_ids() {
            let s1 = &graph1.node_state(node_id).unwrap().dash;
            let s2 = &graph2.node_state(node_id).unwrap().dash;
            assert_eq!(s1, s2);
        }
    }

    #[test]
    fn respects_fixed_counts() {
        let config = Config {
            seed: 1,
            contacts_per_node: Distribution::fixed(2.0),
            groups_per_node: Distribution::fixed(1.0),
            group_size: Distribution::fixed(2.0),
            ops_per_node: Distribution::fixed(3.0),
            op_payload_size: Distribution::fixed(64.0),
            node_storage: Distribution::fixed(50_000_000.0),
        };

        let graph = ChatStateGenerator {
            config: config.clone(),
        }
        .generate(make_topology(5));
        assert_eq!(graph.graph().node_count(), 5);
        assert_all_nodes(&graph, |state| {
            assert_eq!(state.contacts.len(), 2);
            assert_eq!(state.groups.len(), 1);
            assert_eq!(state.op_store.len(), 3);
            for op in state.op_store.values() {
                assert_eq!(*op.header.payload_size, 64);
            }
        });
    }

    #[test]
    fn uniform_distribution() {
        let config = Config {
            seed: 99,
            contacts_per_node: Distribution::uniform(1.0, 5.0),
            groups_per_node: Distribution::uniform(1.0, 3.0),
            group_size: Distribution::fixed(4.0),
            ops_per_node: Distribution::uniform(0.0, 10.0),
            op_payload_size: Distribution::normal(256.0, 50.0),
            node_storage: Distribution::fixed(50_000_000.0),
        };

        let graph = ChatStateGenerator {
            config: config.clone(),
        }
        .generate(make_topology(20));
        assert_eq!(graph.graph().node_count(), 20);
        assert_all_nodes(&graph, |state| {
            assert!(state.contacts.len() >= 1 && state.contacts.len() <= 5);
            assert!(state.groups.len() >= 1 && state.groups.len() <= 3);
            assert!(state.op_store.len() <= 10);
            for op in state.op_store.values() {
                // Normal(256, 50) — payload should be in a reasonable range
                assert!(*op.header.payload_size <= 500);
            }
        });
    }

    #[test]
    fn disconnected_for_defaults() {
        // 1 hour disconnected, ~0.017 msgs/min (≈25/day)
        let config =
            Config::disconnected_for(42, Duration::from_secs(3600), Distribution::fixed(0.017));

        let graph = ChatStateGenerator {
            config: config.clone(),
        }
        .generate(make_topology(10));
        assert_eq!(graph.graph().node_count(), 10);
        assert_all_nodes(&graph, |state| {
            // With Normal(20,8) contacts, should have some
            assert!(state.contacts.len() > 0);
            // 0.017 msgs/min * 60 min ≈ 1 op per node
            assert!(state.op_store.len() <= 5);
        });
    }

    #[test]
    fn from_json() {
        let json = r#"{
            "seed": 42,
            "contacts_per_node": { "type": "fixed", "value": 3.0 },
            "groups_per_node": { "type": "fixed", "value": 2.0 },
            "group_size": { "type": "fixed", "value": 3.0 },
            "ops_per_node": { "type": "uniform", "min": 1.0, "max": 5.0 },
            "op_payload_size": { "type": "normal", "mean": 256.0, "std_dev": 50.0 },
            "node_storage": { "type": "fixed", "value": 50000000.0 }
        }"#;

        let generator = ChatStateGenerator::from_json(json).unwrap();
        let graph = generator.generate(make_topology(10));

        assert_eq!(graph.graph().node_count(), 10);
        assert_all_nodes(&graph, |state| {
            assert_eq!(state.contacts.len(), 3);
            assert_eq!(state.groups.len(), 2);
            assert!(state.op_store.len() >= 1 && state.op_store.len() <= 5);
        });
    }
}
