use rand::seq::IndexedRandom;
use serde::Deserialize;

use crate::{ApplyRule, BuildState};
use simdash_model_network::net_topology::{
    Destination, DestinationSpec, EdgeTopology, Ordinality, Selection,
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

/// Specification for links originating from a node subnet.
#[derive(Debug, Clone, Deserialize)]
pub struct LinkSpec {
    pub strategy: String,
    pub count: Count,
    pub destination: Destination,
    #[serde(default)]
    pub ordinality: Ordinality,
    /// Number of ticks the link persists for. 0 means permanent.
    #[serde(default)]
    pub longevity: u32,
    /// Probability [0, 1] that this link spawns if it is not already present.
    #[serde(default)]
    pub spawn_chance: Option<f64>,
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

        match self.strategy.as_str() {
            "per_node" => {
                for &node in state.all_node_ids.clone().iter() {
                    let n = match self.count {
                        Count::Fixed(n) => n,
                        Count::Range { min, max } => state.rng.random_range(min..=max),
                    };
                    for _ in 0..n {
                        let id = state.topology.edges.len() as u32;
                        state.topology.edges.push(EdgeTopology {
                            id,
                            from: node,
                            destination: destination_record.clone(),
                            ordinality: self.ordinality.clone(),
                            longevity: self.longevity,
                            spawn_chance: self.spawn_chance,
                        });
                    }
                }
            }
            _ => {
                let total_links = match self.count {
                    Count::Fixed(n) => n,
                    Count::Range { min, max } => state.rng.random_range(min..=max),
                };
                match selection {
                    Selection::Random => {
                        for _ in 0..total_links {
                            let from = *state.all_node_ids.choose(&mut state.rng).unwrap();
                            let id = state.topology.edges.len() as u32;
                            state.topology.edges.push(EdgeTopology {
                                id,
                                from,
                                destination: destination_record.clone(),
                                ordinality: self.ordinality.clone(),
                                longevity: self.longevity,
                                spawn_chance: self.spawn_chance,
                            });
                        }
                    }
                }
            }
        }
    }
}
