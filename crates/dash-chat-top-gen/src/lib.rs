use std::collections::{BTreeMap, BTreeSet, HashMap};

pub mod chat_topology;
pub use chat_topology::ChatTopology;

use net_top_gen::NetTopology;
use rand::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use simdash_distribution::Distribution;
use simdash_ids::{NodeId, OpId, Size};
use simdash_state::{chat::Body, chat::ChatState, chat::GroupId, chat::Header, chat::Operation};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Subnets {
    All(SubnetsAll),
    Specific(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SubnetsAll {
    All,
}

impl Default for Subnets {
    fn default() -> Self {
        Subnets::All(SubnetsAll::All)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GroupSpec {
    /// The subnets where the members are spread out.
    pub subnets: Subnets,
    /// How many groups to create.
    pub group_amount: u64,
    /// Size of each group.
    pub group_size: Distribution,
    /// How many operations per group.
    pub operations_count: Distribution,
    /// Payload size (in bytes) of each operation.
    #[serde(default = "default_operations_size")]
    pub operations_size: Distribution,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContactsSpec {
    /// The subnets where the contacts are spread out.
    pub subnets: Subnets,
    /// How many contacts per node.
    pub contacts_amount: Distribution,
    /// How many operations per contact.
    pub operations_count: Distribution,
    /// Payload size (in bytes) of each operation.
    #[serde(default = "default_operations_size")]
    pub operations_size: Distribution,
}

fn default_operations_size() -> Distribution {
    Distribution::normal(256.0, 50.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealisticSpecConfig {
    /// The subnets the spec will be applied to.
    pub subnets: Subnets,
    /// How many contacts each node has.
    pub contacts_per_node: Distribution,
    /// How many groups each node joins.
    pub groups_per_node: Distribution,
    /// Target number of members per group (used to estimate total group count).
    pub group_size: Distribution,
    /// How many operations per contact.
    pub operations_count: Distribution,
    /// Payload size (in bytes) of each operation.
    #[serde(default = "default_operations_size")]
    pub operations_size: Distribution,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Rule {
    Groups(GroupSpec),
    Contacts(ContactsSpec),
    Realistic(RealisticSpecConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatTopologyConfig {
    pub storage_capacity_per_node_kb: Distribution,
    pub rules: Vec<Rule>,
}

/// Generates ChatState for each node in a network topology from a JSON config.
pub struct ChatStateGenerator {
    pub config: ChatTopologyConfig,
}

pub struct NodeConfig {
    #[allow(dead_code)]
    subnet: String,
}

impl ChatStateGenerator {
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let config = serde_json::from_str(json)?;
        Ok(Self { config })
    }

    /// Resolve which node IDs match a `Subnets` selector.
    fn resolve_nodes(subnets: &Subnets, topology: &NetTopology) -> Vec<NodeId> {
        match subnets {
            Subnets::All(_) => topology.nodes.iter().map(|n| n.id).collect(),
            Subnets::Specific(names) => {
                let mut ids: Vec<NodeId> = names
                    .iter()
                    .filter_map(|name| topology.subnets.get(name))
                    .flatten()
                    .copied()
                    .collect();
                ids.sort();
                ids.dedup();
                ids
            }
        }
    }

    /// Generate a `ChatState` for each node by applying each rule in sequence.
    pub fn generate(&self, seed: u64, topology: NetTopology) -> ChatTopology {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut states: BTreeMap<NodeId, ChatState> = BTreeMap::new();
        let mut group_counter: u64 = 0;
        let mut op_counter: u32 = 0;

        // Initialize every node with storage capacity.
        for node in &topology.nodes {
            let storage_kb = self.config.storage_capacity_per_node_kb.sample(&mut rng);
            states.insert(
                node.id,
                ChatState {
                    storage_capacity: Some(Size::from(storage_kb * 1024)),
                    ..ChatState::default()
                },
            );
        }

        // Build a node → subnet lookup (used by contacts and groups rules).
        let node_subnet: HashMap<NodeId, String> = topology
            .subnets
            .iter()
            .flat_map(|(name, ids)| ids.iter().map(move |&id| (id, name.clone())))
            .collect();

        for rule in &self.config.rules {
            match rule {
                Rule::Groups(spec) => {
                    let nodes = Self::resolve_nodes(&spec.subnets, &topology);
                    if nodes.is_empty() {
                        continue;
                    }

                    // Collect per-subnet node lists for cross-subnet guarantees.
                    let subnet_buckets: Vec<Vec<NodeId>> = match &spec.subnets {
                        Subnets::All(_) => topology
                            .subnets
                            .values()
                            .cloned()
                            .collect(),
                        Subnets::Specific(names) => names
                            .iter()
                            .filter_map(|n| topology.subnets.get(n).cloned())
                            .collect(),
                    };

                    for _ in 0..spec.group_amount {
                        let group_id: GroupId = group_counter;
                        group_counter += 1;

                        let target_size =
                            (spec.group_size.sample(&mut rng).max(1) as usize).min(nodes.len());

                        let mut members: Vec<NodeId> = Vec::new();

                        // Guarantee at least one member from each subnet.
                        if subnet_buckets.len() > 1 {
                            for bucket in &subnet_buckets {
                                if let Some(&node) = bucket.choose(&mut rng) {
                                    if !members.contains(&node) {
                                        members.push(node);
                                    }
                                }
                            }
                        }

                        // Fill remaining slots randomly from all eligible nodes.
                        let remaining = target_size.saturating_sub(members.len());
                        if remaining > 0 {
                            let available: Vec<NodeId> = nodes
                                .iter()
                                .copied()
                                .filter(|n| !members.contains(n))
                                .collect();
                            members.extend(
                                available
                                    .choose_multiple(&mut rng, remaining.min(available.len()))
                                    .copied(),
                            );
                        }

                        // Register group membership.
                        for &member in &members {
                            states.entry(member).or_default().groups.insert(group_id);
                        }

                        // Generate operations for this group.
                        let num_ops = spec.operations_count.sample(&mut rng) as usize;
                        for _ in 0..num_ops {
                            let &author = members.choose(&mut rng).unwrap();
                            let op_id = OpId::from(op_counter);
                            op_counter += 1;
                            let payload_size =
                                Size::from(spec.operations_size.sample(&mut rng));

                            let op = Operation {
                                id: op_id,
                                header: Header {
                                    author,
                                    payload_size,
                                },
                                body: Body::GroupMessage { group_id },
                            };

                            // Only the author stores the op initially;
                            // routing will deliver it to other members.
                            states
                                .entry(author)
                                .or_default()
                                .op_store
                                .insert(op_id, op);
                        }
                    }
                }

                Rule::Contacts(spec) => {
                    let nodes = Self::resolve_nodes(&spec.subnets, &topology);
                    if nodes.is_empty() {
                        continue;
                    }

                    for &node in &nodes {
                        let num_contacts = (spec.contacts_amount.sample(&mut rng) as usize)
                            .min(nodes.len() - 1);
                        let my_subnet = node_subnet.get(&node);

                        // Build candidate list; prefer cross-subnet contacts.
                        let others: Vec<NodeId> = if let Subnets::Specific(_) = &spec.subnets {
                            let (mut cross, mut same): (Vec<_>, Vec<_>) = nodes
                                .iter()
                                .copied()
                                .filter(|&id| id != node)
                                .partition(|other| node_subnet.get(other) != my_subnet);
                            cross.shuffle(&mut rng);
                            same.shuffle(&mut rng);
                            cross.extend(same);
                            cross
                        } else {
                            let mut v: Vec<NodeId> = nodes
                                .iter()
                                .copied()
                                .filter(|&id| id != node)
                                .collect();
                            v.shuffle(&mut rng);
                            v
                        };

                        let contacts: BTreeSet<NodeId> =
                            others.into_iter().take(num_contacts).collect();

                        // Generate direct-message operations.
                        let num_ops = spec.operations_count.sample(&mut rng) as usize;
                        let contacts_vec: Vec<NodeId> = contacts.iter().copied().collect();

                        for _ in 0..num_ops {
                            let op_id = OpId::from(op_counter);
                            op_counter += 1;
                            let payload_size =
                                Size::from(spec.operations_size.sample(&mut rng));
                            let recipient = contacts_vec
                                .choose(&mut rng)
                                .copied()
                                .unwrap_or(node);

                            let op = Operation {
                                id: op_id,
                                header: Header {
                                    author: node,
                                    payload_size,
                                },
                                body: Body::DirectMessage { recipient },
                            };
                            states.entry(node).or_default().op_store.insert(op_id, op);
                        }

                        states.entry(node).or_default().contacts.extend(contacts);
                    }
                }

                Rule::Realistic(spec) => {
                    let nodes = Self::resolve_nodes(&spec.subnets, &topology);
                    if nodes.is_empty() {
                        continue;
                    }
                    let n = nodes.len();

                    // Estimate total number of groups for this rule.
                    let avg_gpn = spec.groups_per_node.sample(&mut rng).max(1);
                    let avg_gs = spec.group_size.sample(&mut rng).max(1);
                    let num_groups = ((n as u64) * avg_gpn / avg_gs).max(1);

                    for &node in &nodes {
                        // Groups: pick a random subset from the group id range.
                        let gpn =
                            spec.groups_per_node.sample(&mut rng).min(num_groups) as usize;
                        let groups: BTreeSet<GroupId> =
                            (group_counter..group_counter + num_groups)
                                .collect::<Vec<_>>()
                                .choose_multiple(&mut rng, gpn)
                                .copied()
                                .collect();

                        // Contacts: pick from other nodes.
                        let num_contacts = spec
                            .contacts_per_node
                            .sample(&mut rng)
                            .min((n - 1) as u64) as usize;
                        let others: Vec<NodeId> =
                            nodes.iter().copied().filter(|&id| id != node).collect();
                        let contacts: BTreeSet<NodeId> = others
                            .choose_multiple(&mut rng, num_contacts)
                            .copied()
                            .collect();

                        // Operations.
                        let num_ops = spec.operations_count.sample(&mut rng) as usize;
                        let state = states.entry(node).or_default();

                        for _ in 0..num_ops {
                            let op_id = OpId::from(op_counter);
                            op_counter += 1;
                            let payload_size =
                                Size::from(spec.operations_size.sample(&mut rng));

                            let body = if let Some(&&gid) =
                                groups.iter().collect::<Vec<_>>().choose(&mut rng)
                            {
                                Body::GroupMessage { group_id: gid }
                            } else {
                                let recipient = contacts
                                    .iter()
                                    .collect::<Vec<_>>()
                                    .choose(&mut rng)
                                    .copied()
                                    .copied()
                                    .unwrap_or(node);
                                Body::DirectMessage { recipient }
                            };

                            state.op_store.insert(
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

                        state.contacts.extend(contacts);
                        state.groups.extend(groups);
                    }

                    group_counter += num_groups;
                }
            }
        }

        ChatTopology {
            net: topology,
            chat_states: states,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use net_top_gen::net_topology::NodeTopology;
    use simdash_state::node::NodeState;
    use std::collections::BTreeMap;

    fn make_topology(n: u32) -> NetTopology {
        let mut topology = NetTopology::new();
        for i in 0..n {
            topology.nodes.push(NodeTopology::new(NodeId::from(i)));
        }
        // All nodes in a single "default" subnet.
        topology.subnets.insert(
            "default".to_string(),
            (0..n).map(NodeId::from).collect(),
        );
        topology
    }

    fn make_multi_subnet_topology(per_subnet: &[(&str, u32)]) -> NetTopology {
        let mut topology = NetTopology::new();
        let mut id = 0u32;
        for &(subnet, count) in per_subnet {
            let mut ids = Vec::new();
            for _ in 0..count {
                let node_id = NodeId::from(id);
                topology.nodes.push(NodeTopology::new(node_id));
                ids.push(node_id);
                id += 1;
            }
            topology.subnets.insert(subnet.to_string(), ids);
        }
        topology
    }

    fn make_generator(rules: Vec<Rule>) -> ChatStateGenerator {
        ChatStateGenerator {
            config: ChatTopologyConfig {
                storage_capacity_per_node_kb: Distribution::fixed(40.0),
                rules,
            },
        }
    }

    fn assert_all_chat_states(result: &ChatTopology, f: impl Fn(&ChatState)) {
        for state in result.chat_states.values() {
            f(state);
        }
    }

    #[test]
    fn deterministic_with_seed() {
        let generator = make_generator(vec![Rule::Groups(GroupSpec {
            subnets: Subnets::default(),
            group_amount: 3,
            group_size: Distribution::fixed(4.0),
            operations_count: Distribution::fixed(5.0),
            operations_size: Distribution::fixed(100.0),
        })]);

        let r1 = generator.generate(42, make_topology(10));
        let r2 = generator.generate(42, make_topology(10));

        for node_id in r1.chat_states.keys() {
            assert_eq!(r1.chat_states[node_id], r2.chat_states[node_id]);
        }
    }

    #[test]
    fn groups_rule_creates_groups_and_ops() {
        let generator = make_generator(vec![Rule::Groups(GroupSpec {
            subnets: Subnets::default(),
            group_amount: 2,
            group_size: Distribution::fixed(5.0),
            operations_count: Distribution::fixed(3.0),
            operations_size: Distribution::fixed(64.0),
        })]);

        let result = generator.generate(1, make_topology(10));
        assert_eq!(result.chat_states.len(), 10);

        // At least some nodes should be in groups and have ops.
        let total_group_memberships: usize =
            result.chat_states.values().map(|s| s.groups.len()).sum();
        assert!(total_group_memberships > 0);

        for state in result.chat_states.values() {
            for op in state.op_store.values() {
                assert_eq!(*op.header.payload_size, 64);
                assert!(matches!(op.body, Body::GroupMessage { .. }));
            }
        }
    }

    #[test]
    fn contacts_rule_creates_contacts_and_ops() {
        let generator = make_generator(vec![Rule::Contacts(ContactsSpec {
            subnets: Subnets::default(),
            contacts_amount: Distribution::fixed(3.0),
            operations_count: Distribution::fixed(2.0),
            operations_size: Distribution::fixed(128.0),
        })]);

        let result = generator.generate(1, make_topology(10));
        assert_eq!(result.chat_states.len(), 10);

        assert_all_chat_states(&result, |state| {
            assert_eq!(state.contacts.len(), 3);
            assert_eq!(state.op_store.len(), 2);
            for op in state.op_store.values() {
                assert_eq!(*op.header.payload_size, 128);
                assert!(matches!(op.body, Body::DirectMessage { .. }));
            }
        });
    }

    #[test]
    fn contacts_prefers_cross_subnet() {
        // 2 subnets with 5 nodes each; request 3 contacts per node.
        let topology = make_multi_subnet_topology(&[("alpha", 5), ("beta", 5)]);
        let generator = make_generator(vec![Rule::Contacts(ContactsSpec {
            subnets: Subnets::Specific(vec!["alpha".into(), "beta".into()]),
            contacts_amount: Distribution::fixed(3.0),
            operations_count: Distribution::fixed(0.0),
            operations_size: Distribution::fixed(64.0),
        })]);

        let result = generator.generate(1, topology);

        let alpha_ids: BTreeSet<NodeId> = (0..5).map(NodeId::from).collect();
        let beta_ids: BTreeSet<NodeId> = (5..10).map(NodeId::from).collect();

        // For each alpha node, majority of contacts should be from beta (and vice versa).
        for (&node, state) in &result.chat_states {
            let is_alpha = alpha_ids.contains(&node);
            let cross_count = state.contacts.iter().filter(|c| {
                if is_alpha { beta_ids.contains(c) } else { alpha_ids.contains(c) }
            }).count();
            // With 3 contacts and 5 cross-subnet candidates, all 3 should be cross-subnet.
            assert_eq!(cross_count, 3, "node {:?} should have 3 cross-subnet contacts", node);
        }
    }

    #[test]
    fn groups_span_subnets() {
        let topology = make_multi_subnet_topology(&[("a", 5), ("b", 5)]);
        let generator = make_generator(vec![Rule::Groups(GroupSpec {
            subnets: Subnets::Specific(vec!["a".into(), "b".into()]),
            group_amount: 4,
            group_size: Distribution::fixed(6.0),
            operations_count: Distribution::fixed(0.0),
            operations_size: Distribution::fixed(64.0),
        })]);

        let result = generator.generate(1, topology);

        let a_ids: BTreeSet<NodeId> = (0..5).map(NodeId::from).collect();
        let b_ids: BTreeSet<NodeId> = (5..10).map(NodeId::from).collect();

        // Collect members per group across all nodes.
        let mut group_members: BTreeMap<GroupId, BTreeSet<NodeId>> = BTreeMap::new();
        for (&node, state) in &result.chat_states {
            for &gid in &state.groups {
                group_members.entry(gid).or_default().insert(node);
            }
        }

        assert_eq!(group_members.len(), 4);
        for (_gid, members) in &group_members {
            let has_a = members.iter().any(|n| a_ids.contains(n));
            let has_b = members.iter().any(|n| b_ids.contains(n));
            assert!(has_a, "group should have at least one member from subnet a");
            assert!(has_b, "group should have at least one member from subnet b");
        }
    }

    #[test]
    fn realistic_rule() {
        let generator = make_generator(vec![Rule::Realistic(RealisticSpecConfig {
            subnets: Subnets::default(),
            contacts_per_node: Distribution::fixed(2.0),
            groups_per_node: Distribution::fixed(1.0),
            group_size: Distribution::fixed(2.0),
            operations_count: Distribution::fixed(3.0),
            operations_size: Distribution::fixed(64.0),
        })]);

        let result = generator.generate(1, make_topology(10));
        assert_eq!(result.chat_states.len(), 10);

        assert_all_chat_states(&result, |state| {
            assert_eq!(state.contacts.len(), 2);
            assert_eq!(state.groups.len(), 1);
            assert_eq!(state.op_store.len(), 3);
            for op in state.op_store.values() {
                assert_eq!(*op.header.payload_size, 64);
            }
        });
    }

    #[test]
    fn storage_capacity_set_from_config() {
        let generator = ChatStateGenerator {
            config: ChatTopologyConfig {
                storage_capacity_per_node_kb: Distribution::fixed(100.0),
                rules: vec![],
            },
        };

       let result = generator.generate(1, make_topology(5));
        assert_all_chat_states(&result, |state| {
            assert_eq!(state.storage_capacity, Some(Size::from(100 * 1024)));
        });
    }

    #[test]
    fn from_json() {
        let json = r#"{
            "storage_capacity_per_node_kb": { "type": "fixed", "value": 40.0 },
            "rules": [
                {
                    "type": "groups",
                    "subnets": "all",
                    "group_amount": 2,
                    "group_size": { "type": "fixed", "value": 3.0 },
                    "operations_count": { "type": "fixed", "value": 1.0 },
                    "operations_size": { "type": "fixed", "value": 64.0 }
                },
                {
                    "type": "contacts",
                    "subnets": "all",
                    "contacts_amount": { "type": "fixed", "value": 2.0 },
                    "operations_count": { "type": "fixed", "value": 1.0 }
                }
            ]
        }"#;

        let generator = ChatStateGenerator::from_json(json).unwrap();
        let result = generator.generate(42, make_topology(10));

        assert_eq!(result.chat_states.len(), 10);
        assert_all_chat_states(&result, |state| {
            assert_eq!(state.contacts.len(), 2);
            assert!(state.storage_capacity == Some(Size::from(40 * 1024)));
        });
    }
}
