use std::collections::{BTreeSet, HashMap};

use dash_chat_top_gen::ChatTopology;
use rand::seq::IndexedRandom;
use net_topology::net_topology::Ordinality;
use polestar::{Machine, ext::MapExt};
use simdash_action::{
    node::NodeAction,
    routing::{RoutingAction, flooding::FloodingAction},
};
use simdash_ids::NodeId;
use simdash_state::{chat::Operation, network::NetworkState, node::NodeState};

use crate::{
    node::{NodeFx, NodeModel, NodeModelConfig},
    routing::{RoutingFx, flooding::FloodingFx},
};

pub struct NetworkModel {
    nodes: HashMap<NodeId, NodeModel>,
    pub topology: ChatTopology,
}

impl NetworkModel {
    pub fn new(nodes: HashMap<NodeId, NodeModel>, topology: ChatTopology) -> Self {
        Self { nodes, topology }
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
                    let node_model = self
                        .nodes
                        .get(&node_id)
                        .ok_or_else(|| anyhow::anyhow!("No model for node {:?}", node_id))?;
                    state.graph.update_node(node_id, |node_state| {
                        node_model.transition(node_state, NodeAction::Tick)
                    })?;
                }
                Ok((state, ()))
            }

            simdash_action::network::NetworkAction::EvolveTopology => {
                state
                    .graph
                    .graph_mut()
                    .retain_edges(|g, idx| g[idx].remaining_ticks != Some(0));

                // Collect topology IDs that still have a live edge in the graph.
                let existing_ids: BTreeSet<u32> = state
                    .graph
                    .graph()
                    .edge_weights()
                    .map(|e| e.topology_id)
                    .collect();

                for edge_topo in &self.topology.net.edges {
                    if edge_topo.spawn_chance.is_none() {
                        continue;
                    }
                    if existing_ids.contains(&edge_topo.id) {
                        continue;
                    }
                    let candidates = self.topology.net.destination_candidates(edge_topo);
                    let mut rng = state.topology.rng();
                    if let Some(&to) = candidates.choose(&mut rng) {
                        use simdash_state::EdgeState;
                        state.graph.add_edge(
                            edge_topo.from,
                            to,
                            EdgeState::new(edge_topo.id, false, edge_topo.longevity),
                        );
                        if matches!(edge_topo.ordinality, Ordinality::Bidirectional) {
                            state.graph.add_edge(
                                to,
                                edge_topo.from,
                                EdgeState::new(edge_topo.id, true, edge_topo.longevity),
                            );
                        }
                    }
                }

                Ok((state, ()))
            }

            simdash_action::network::NetworkAction::Node(node_id, node_action) => {
                let node_model = self
                    .nodes
                    .get(&node_id)
                    .ok_or_else(|| anyhow::anyhow!("No model for node {:?}", node_id))?;
                let fx = match state.graph.update_node(node_id, |node_state| {
                    let (node_state, node_fx) = node_model.transition(node_state, node_action)?;
                    Ok((node_state, node_fx))
                }) {
                    Ok(fx) => {
                        if let Some(fx) = fx {
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
    pub fn send_op(
        &self,
        state: &mut NetworkState,
        from: NodeId,
        to: NodeId,
        op: Operation,
    ) -> anyhow::Result<()> {
        let node_model = self
            .nodes
            .get(&to)
            .ok_or_else(|| anyhow::anyhow!("No model for node {:?}", to))?;
        let fx = state.graph.update_node(to, |mut n| {
            let (new, fx) = node_model
                .routing
                .transition(n.routing, FloodingAction::ReceiveOp { from, op }.into())?;
            n.routing = new;
            Ok((n, fx))
        })?;
        self.apply_routing_effects(state, to, fx)?;
        Ok(())
    }

    pub fn apply_node_effects(
        &self,
        state: &mut NetworkState,
        node_id: NodeId,
        fx: NodeFx,
    ) -> anyhow::Result<()> {
        self.apply_routing_effects(state, node_id, fx.routing)
    }

    pub fn apply_routing_effects(
        &self,
        state: &mut NetworkState,
        node_id: NodeId,
        fx: RoutingFx,
    ) -> anyhow::Result<()> {
        match fx {
            RoutingFx::Flooding(Some(FloodingFx::BroadcastOps(broadcast_ops))) => {
                let ops = broadcast_ops
                    .iter()
                    .map(|op_id| {
                        state
                            .graph
                            .node_state(node_id)
                            .ok_or(anyhow::anyhow!("Node not found: {:?}", node_id))?
                            .dash
                            .op_store
                            .get(op_id)
                            .cloned()
                            .ok_or(anyhow::anyhow!("Op not found: {:?}", op_id))
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;

                let neighbors = state.neighborhood(node_id).collect::<Vec<_>>();
                for neighbor in neighbors {
                    for op in ops.iter() {
                        if let Err(err) = self.send_op(state, node_id, neighbor, op.clone()) {
                            tracing::debug!("Failed to send op to neighbor: {:?}", err);
                        }
                    }
                }
            }
            RoutingFx::Flooding(Some(FloodingFx::StoreOp(op))) => {
                let op_id = op.id;
                let stored = state.graph.update_node(node_id, |mut node_state| {
                    node_state.dash.store_op(op)?;
                    Ok((node_state, ()))
                });
                if stored.is_err() {
                    // Chat rejected the op (e.g. capacity exceeded);
                    // remove it from routing state so it won't be broadcast.
                    state.graph.update_node(node_id, |mut node_state| {
                        node_state.routing.flooding.ops.remove(&op_id);
                        Ok((node_state, ()))
                    })?;
                }
            }
            RoutingFx::Flooding(None) => {}
        }
        Ok(())
    }
}
