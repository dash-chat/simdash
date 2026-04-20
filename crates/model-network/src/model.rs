//! The Network model manages the network-level state:
//! - the state of each Node
//! - the network topology (edges between nodes)

use std::collections::HashSet;

use dash_chat_top_gen::ChatTopology;
use rand::seq::IndexedRandom;
use simdash_model_node::{NodeAction, NodeModel};

use crate::{NetworkAction, NetworkFx, NetworkState};

/// The Network model manages the network-level state:
/// - the state of each Node
/// - the network topology (edges between nodes)
pub struct NetworkModel {
    node: NodeModel,
    /// The network topology
    pub topology: ChatTopology,
}

impl NetworkModel {
    /// Constructor
    pub fn new(node: NodeModel, topology: ChatTopology) -> Self {
        Self { node, topology }
    }
}

impl polestar::Machine for NetworkModel {
    type State = NetworkState;
    type Action = NetworkAction;
    type Fx = Vec<NetworkFx>;
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        match action {
            NetworkAction::Tick => {
                for edge in state.graph.graph_mut().edge_weights_mut() {
                    if let Some(ref mut ticks) = edge.remaining_ticks {
                        *ticks = ticks.saturating_sub(1);
                    }
                }

                let all_keys = state.graph.node_ids().collect::<Vec<_>>();

                for node_id in all_keys {
                    let fx = state.graph.update_node(node_id, |node_state| {
                        self.node.transition(node_state, NodeAction::Tick)
                    })?;
                    debug_assert!(fx.is_empty());
                }
                Ok((state, vec![]))
            }

            NetworkAction::EvolveTopology => {
                state
                    .graph
                    .graph_mut()
                    .retain_edges(|g, idx| g[idx].remaining_ticks != Some(0));

                let existing_ids: HashSet<u32> = state
                    .graph
                    .graph()
                    .edge_weights()
                    .map(|e| e.topology_id)
                    .collect();

                for edge_topo in self.unspawned_edge_topos(&existing_ids) {
                    let mut rng = state.topology.rng();
                    if !edge_topo.should_spawn(&mut rng) {
                        continue;
                    }
                    let candidates = self.topology.net.destination_candidates(edge_topo);
                    if let Some(&to) = candidates.choose(&mut rng) {
                        edge_topo.add_to_graph(&mut state.graph, to);
                    }
                }

                Ok((state, vec![]))
            }

            NetworkAction::Node(node_id, node_action) => {
                match state.graph.update_node(node_id, |node_state| {
                    let (node_state, node_fx) = self.node.transition(node_state, node_action)?;
                    Ok((node_state, node_fx))
                }) {
                    Ok(fx) => Ok((
                        state,
                        fx.into_iter()
                            .map(|fx| NetworkFx::Node(node_id, fx))
                            .collect(),
                    )),
                    Err(e) => {
                        tracing::error!("Failed to transition node {node_id:?}: {e:?}");
                        Ok((state, vec![]))
                    }
                }
            }
        }
    }
}

impl NetworkModel {
    fn unspawned_edge_topos<'a>(
        &'a self,
        existing_ids: &'a HashSet<u32>,
    ) -> impl Iterator<Item = &'a net_topology::net_topology::EdgeTopology> {
        self.topology
            .net
            .edges
            .iter()
            .filter(|e| e.spawn_chance.is_some() && !existing_ids.contains(&e.id))
    }
}
