use rand::seq::IndexedRandom;
use serde::Deserialize;
use simdash_ids::NodeId;

use crate::{
    ApplyRule, BuildState,
    net_topology::{Destination, DestinationSpec, EdgeTopology, Selection},
};

/// How many links to create for a rule.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Count {
    /// A fixed number of links.
    Fixed(u32),
    /// A uniformly random number of links in the range `[min, max]`.
    Range { min: u32, max: u32 },
}

/// Whether a created link also generates a reverse edge.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ordinality {
    #[default]
    Unidirectional,
    Bidirectional,
}

/// Specification for links originating from a node subnet.
#[derive(Debug, Clone, Deserialize)]
pub struct LinkSpec {
    pub strategy: String,
    pub count: Count,
    pub destination: Destination,
    #[serde(default)]
    pub ordinality: Ordinality,
}

impl ApplyRule for LinkSpec {
    fn apply<R: rand::Rng>(&self, state: &mut BuildState<R>) {
        if state.all_node_ids.is_empty() {
            return;
        }

        let (selection, subnet) = match &self.destination {
            Destination::Selection(selection) => (selection, None),
            Destination::Spec(spec) => (&spec.selection, spec.subnet.as_ref()),
        };

        let destination_record = Destination::Spec(DestinationSpec {
            selection: selection.clone(),
            subnet: subnet.cloned(),
        });

        let target_pool: Vec<NodeId> = match subnet {
            Some(subnet) => state
                .topology
                .subnets
                .get(subnet)
                .cloned()
                .unwrap_or_default(),
            None => state.all_node_ids.clone(),
        };

        if target_pool.is_empty() {
            return;
        }

        let bidirectional = matches!(self.ordinality, Ordinality::Bidirectional);

        match self.strategy.as_str() {
            "per_node" => match selection {
                Selection::Random => {
                    for &node in state.all_node_ids.clone().iter() {
                        let n = match self.count {
                            Count::Fixed(n) => n,
                            Count::Range { min, max } => state.rng.random_range(min..=max),
                        };
                        let candidates: Vec<NodeId> = target_pool
                            .iter()
                            .copied()
                            .filter(|&id| id != node)
                            .collect();
                        for _ in 0..n {
                            if let Some(&to) = candidates.choose(&mut state.rng) {
                                state.topology.edges.push(EdgeTopology {
                                    from: node,
                                    to,
                                    destination: destination_record.clone(),
                                });
                                if bidirectional {
                                    state.topology.edges.push(EdgeTopology {
                                        from: to,
                                        to: node,
                                        destination: destination_record.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            },
            _ => {
                let total_links = match self.count {
                    Count::Fixed(n) => n,
                    Count::Range { min, max } => state.rng.random_range(min..=max),
                };
                match selection {
                    Selection::Random => {
                        for _ in 0..total_links {
                            let from = *state.all_node_ids.choose(&mut state.rng).unwrap();
                            let candidates: Vec<NodeId> = target_pool
                                .iter()
                                .copied()
                                .filter(|&id| id != from)
                                .collect();
                            if let Some(&to) = candidates.choose(&mut state.rng) {
                                state.topology.edges.push(EdgeTopology {
                                    from,
                                    to,
                                    destination: destination_record.clone(),
                                });
                                if bidirectional {
                                    state.topology.edges.push(EdgeTopology {
                                        from: to,
                                        to: from,
                                        destination: destination_record.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
