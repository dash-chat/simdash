use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use rand::prelude::*;
use simdash_distribution::Distribution;
use simdash_ids::{NodeId, OpId, Size};
use simdash_state::{dash::Body, dash::DashState, dash::GroupId, dash::Header, dash::Operation};

#[derive(Debug, Clone)]
pub struct Config {
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

impl Default for Config {
    /// Realistic chat-app defaults based on WhatsApp/Signal research data.
    fn default() -> Self {
        Config {
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
}

impl Config {
    /// Create a Config with realistic defaults for nodes that have been
    /// disconnected for the given duration.
    ///
    /// `ops_per_minute` controls how many messages per minute each node produces.
    /// A realistic default is ~0.017 (≈25 msgs/day).
    pub fn disconnected_for(duration: Duration, ops_per_minute: Distribution) -> Self {
        let minutes = duration.as_secs_f64() / 60.0;

        let ops_per_node = ops_per_minute.scale(minutes);

        Config {
            ops_per_node,
            ..Config::default()
        }
    }
}

/// Generate a `DashState` for each node in the network topology.
pub fn generate(nodes: &[NodeId], seed: u64, config: &Config) -> BTreeMap<NodeId, DashState> {
    let mut rng = StdRng::seed_from_u64(seed);
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

    for &node in nodes {
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
    use super::*;

    #[test]
    fn deterministic_with_seed() {
        let nodes: Vec<NodeId> = (0..10).map(NodeId::from).collect();
        let config = Config {
            contacts_per_node: Distribution::fixed(3.0),
            groups_per_node: Distribution::fixed(2.0),
            group_size: Distribution::fixed(3.0),
            ops_per_node: Distribution::fixed(5.0),
            op_payload_size: Distribution::fixed(100.0),
        };

        let result1 = generate(&nodes, 42, &config);
        let result2 = generate(&nodes, 42, &config);
        assert_eq!(result1, result2);
    }

    #[test]
    fn respects_fixed_counts() {
        let nodes: Vec<NodeId> = (0..5).map(NodeId::from).collect();
        let config = Config {
            contacts_per_node: Distribution::fixed(2.0),
            groups_per_node: Distribution::fixed(1.0),
            group_size: Distribution::fixed(2.0),
            ops_per_node: Distribution::fixed(3.0),
            op_payload_size: Distribution::fixed(64.0),
        };

        let result = generate(&nodes, 1, &config);
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
        let nodes: Vec<NodeId> = (0..20).map(NodeId::from).collect();
        let config = Config {
            contacts_per_node: Distribution::uniform(1.0, 5.0),
            groups_per_node: Distribution::uniform(1.0, 3.0),
            group_size: Distribution::fixed(4.0),
            ops_per_node: Distribution::uniform(0.0, 10.0),
            op_payload_size: Distribution::normal(256.0, 50.0),
        };

        let result = generate(&nodes, 99, &config);
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
        let nodes: Vec<NodeId> = (0..10).map(NodeId::from).collect();
        // 1 hour disconnected, ~0.017 msgs/min (≈25/day)
        let config = Config::disconnected_for(
            Duration::from_secs(3600),
            Distribution::fixed(0.017),
        );

        let result = generate(&nodes, 42, &config);
        assert_eq!(result.len(), 10);
        for state in result.values() {
            // With Normal(20,8) contacts, should have some contacts
            assert!(state.contacts.len() > 0);
            // With Normal(10,4) groups, should have some groups
            assert!(state.groups.len() > 0);
            // 0.017 msgs/min * 60 min ≈ 1 op per node
            assert!(state.op_store.len() <= 5);
        }
    }
}
