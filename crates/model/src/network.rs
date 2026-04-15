//! The Network model manages the network-level state:
//! - the state of each Node
//! - the network topology (edges between nodes)

use std::collections::HashSet;

use dash_chat_top_gen::ChatTopology;
use polestar::Machine;
use rand::seq::IndexedRandom;
use simdash_action::{chat::ChatAction, node::NodeAction};
use simdash_routing_common::{ProtocolNetworkAction, SendFx, StorageFx};
use simdash_routing_dispatch::{RoutingDispatchAction, RoutingDispatchFx};
use simdash_state::network::NetworkState;
use simdash_types::NodeId;

use crate::node::{NodeFx, NodeModel};

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
    type State = simdash_state::network::NetworkState;
    type Action = simdash_action::network::NetworkAction;
    type Fx = ();
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        match action {
            simdash_action::network::NetworkAction::Tick => {
                for edge in state.graph.graph_mut().edge_weights_mut() {
                    if let Some(ref mut ticks) = edge.remaining_ticks {
                        *ticks = ticks.saturating_sub(1);
                    }
                }
                let all_keys = state.graph.node_ids().collect::<Vec<_>>();
                for node_id in all_keys {
                    state.graph.update_node(node_id, |node_state| {
                        self.node.transition(node_state, NodeAction::Tick)
                    })?;
                }
                Ok((state, ()))
            }

            simdash_action::network::NetworkAction::EvolveTopology => {
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

                Ok((state, ()))
            }

            simdash_action::network::NetworkAction::Node(node_id, node_action) => {
                let () = match state.graph.update_node(node_id, |node_state| {
                    let (node_state, node_fx) = self.node.transition(node_state, node_action)?;
                    Ok((node_state, node_fx))
                }) {
                    Ok(fx) => {
                        for fx in fx {
                            self.apply_node_effects(&mut state, node_id, fx)?;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to transition node {node_id:?}: {e:?}");
                    }
                };
                Ok((state, ()))
            }
        }
    }
}

impl NetworkModel {
    /// Applies the bubbled-up effect of a NodeAction to the network state.
    /// This allows modeling different Nodes talking to each other,
    /// via their communication (routing) protocols.
    pub fn apply_node_effects(
        &self,
        state: &mut NetworkState,
        node_id: NodeId,
        fx: NodeFx,
    ) -> anyhow::Result<()> {
        match fx.routing {
            RoutingDispatchFx::Network(send) => match send {
                SendFx::BroadcastData { data } => {
                    let neighbors = state.neighborhood(node_id).collect::<Vec<_>>();
                    for neighbor in neighbors {
                        let fx = state.graph.update_node(neighbor, |node_state| {
                            self.node.transition(
                                node_state,
                                NodeAction::Routing(RoutingDispatchAction::Network(
                                    ProtocolNetworkAction::Receive { data: data.clone() },
                                )),
                            )
                        })?;

                        for fx in fx {
                            self.apply_node_effects(state, neighbor, fx)?;
                        }
                    }
                }
                SendFx::UnicastData { to, data } => {
                    let fx = state.graph.update_node(to, |node_state| {
                        self.node.transition(
                            node_state,
                            NodeAction::Routing(RoutingDispatchAction::Network(
                                ProtocolNetworkAction::Receive { data },
                            )),
                        )
                    })?;
                    for fx in fx {
                        self.apply_node_effects(state, to, fx)?;
                    }
                }
            },

            // TODO: this could also be handled at the Node level, or by the Simulator.
            RoutingDispatchFx::Node(StorageFx::StoreOp { op }) => {
                let fx = state.graph.update_node(node_id, |node_state| {
                    let action = NodeAction::Chat(ChatAction::StoreOp(
                        op.decrypt(&simdash_types::EncryptionKey),
                    ));
                    self.node.transition(node_state, action)
                })?;
                for fx in fx {
                    self.apply_node_effects(state, node_id, fx)?;
                }
            }
            RoutingDispatchFx::Protocol(fx) => {
                tracing::warn!(
                    "Unhandled protocol effect (need to create NetworkFx to propagate higher up): {:?}",
                    fx
                );
            }
        }
        Ok(())
    }

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
