use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use rand::prelude::*;
use simdash_distribution::Distribution;
use simdash_ids::{NodeId, OpId, Size};
use simdash_state::{
    dash::Body, dash::DashState, dash::GroupId, dash::Header, dash::Operation, graph::Graph,
};

#[derive(Debug, Clone)]
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

/// Generate a `DashState` for each node in the network topology.
pub fn generate(graph: &Graph, config: &Config) -> BTreeMap<NodeId, DashState> {
    let mut rng = StdRng::seed_from_u64(config.seed);
    let nodes: Vec<NodeId> = graph.node_ids().collect();
    let n = nodes.len();
    if n == 0 {
        return BTreeMap::new();
    }

    // Step 1: Estimate total number of groups.
    // avg_groups_per_node * num_nodes / avg_group_size ≈ total groups
    let avg_groups_per_node = config.groups_per_node.sample(&mut rng).max(1);
    let avg_group_size = config.group_size.sample(&mut rng).max(1);
    let num_groups = ((n as u64) * avg_groups_per_node / avg_group_size).max(1);

    // Step 2: For each node, assign groups, contacts, and ops.
    let mut op_counter: u32 = 0;
    let mut result = BTreeMap::new();

    for &node in &nodes {
        // Groups
        let num_groups_for_node = config.groups_per_node.sample(&mut rng).min(num_groups) as usize;
        let groups: BTreeSet<GroupId> = (0..num_groups)
            .collect::<Vec<_>>()
            .choose_multiple(&mut rng, num_groups_for_node)
            .copied()
            .collect();

        // Contacts: pick from other nodes
        let num_contacts = config
            .contacts_per_node
            .sample(&mut rng)
            .min((n - 1) as u64) as usize;
        let others: Vec<NodeId> = nodes.iter().copied().filter(|&id| id != node).collect();
        let contacts: BTreeSet<NodeId> = others
            .choose_multiple(&mut rng, num_contacts)
            .copied()
            .collect();

        // Ops
        let num_ops = config.ops_per_node.sample(&mut rng) as usize;
        let mut op_store = BTreeMap::new();
        for _ in 0..num_ops {
            let op_id = OpId::from(op_counter);
            op_counter += 1;
            let payload_size = Size::from(config.op_payload_size.sample(&mut rng));

            // Pick a random group for the message body (or direct message if no groups)
            let body = if let Some(&group_id) = groups.iter().collect::<Vec<_>>().choose(&mut rng) {
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
                    header: Header {
                        author: node,
                        payload_size,
                    },
                    body,
                },
            );
        }

        result.insert(
            node,
            DashState {
                op_store,
                contacts,
                groups,
            },
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use simdash_state::node::NodeState;

    use super::*;

    fn make_graph(n: u32) -> Graph {
        let mut graph = Graph::new();
        for i in 0..n {
            graph.add_node(NodeId::from(i), NodeState::default());
        }
        graph
    }

    #[test]
    fn deterministic_with_seed() {
        let graph = make_graph(10);
        let config = Config {
            seed: 42,
            contacts_per_node: Distribution::fixed(3.0),
            groups_per_node: Distribution::fixed(2.0),
            group_size: Distribution::fixed(3.0),
            ops_per_node: Distribution::fixed(5.0),
            op_payload_size: Distribution::fixed(100.0),
        };

        let result1 = generate(&graph, &config);
        let result2 = generate(&graph, &config);
        assert_eq!(result1, result2);
    }

    #[test]
    fn respects_fixed_counts() {
        let graph = make_graph(5);
        let config = Config {
            seed: 1,
            contacts_per_node: Distribution::fixed(2.0),
            groups_per_node: Distribution::fixed(1.0),
            group_size: Distribution::fixed(2.0),
            ops_per_node: Distribution::fixed(3.0),
            op_payload_size: Distribution::fixed(64.0),
        };

        let result = generate(&graph, &config);
        assert_eq!(result.len(), 5);
        for (_node_id, state) in &result {
            assert_eq!(state.contacts.len(), 2);
            assert_eq!(state.groups.len(), 1);
            assert_eq!(state.op_store.len(), 3);
            for op in state.op_store.values() {
                assert_eq!(*op.header.payload_size, 64);
            }
        }
    }

    #[test]
    fn uniform_distribution() {
        let graph = make_graph(20);
        let config = Config {
            seed: 99,
            contacts_per_node: Distribution::uniform(1.0, 5.0),
            groups_per_node: Distribution::uniform(1.0, 3.0),
            group_size: Distribution::fixed(4.0),
            ops_per_node: Distribution::uniform(0.0, 10.0),
            op_payload_size: Distribution::normal(256.0, 50.0),
        };

        let result = generate(&graph, &config);
        assert_eq!(result.len(), 20);
        for state in result.values() {
            assert!(state.contacts.len() >= 1 && state.contacts.len() <= 5);
            assert!(state.groups.len() >= 1 && state.groups.len() <= 3);
            assert!(state.op_store.len() <= 10);
            for op in state.op_store.values() {
                // Normal(256, 50) — payload should be in a reasonable range
                assert!(*op.header.payload_size <= 500);
            }
        }
    }

    #[test]
    fn disconnected_for_defaults() {
        let graph = make_graph(10);
        // 1 hour disconnected, ~0.017 msgs/min (≈25/day)
        let config = Config::disconnected_for(
            42,
            Duration::from_secs(3600),
            Distribution::fixed(0.017),
        );

        let result = generate(&graph, &config);
        assert_eq!(result.len(), 10);
        for state in result.values() {
            // With Normal(20,8) contacts, should have some
            assert!(state.contacts.len() > 0);
            // 0.017 msgs/min * 60 min ≈ 1 op per node
            assert!(state.op_store.len() <= 5);
        }
    }
}
