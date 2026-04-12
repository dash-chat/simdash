use std::{
    collections::{BTreeSet, HashMap},
    marker::PhantomData,
};

use dash_chat_top_gen::ChatTopology;
use net_topology::net_topology::Ordinality;
use polestar::{Machine, ext::MapExt};
use rand::seq::IndexedRandom;
use simdash_action::{
    node::NodeAction,
    routing::{NetworkRoutingFx, RoutingAction},
};
use simdash_ids::NodeId;
use simdash_state::{chat::Operation, network::NetworkState, node::NodeState};

use crate::node::{NodeFx, NodeModel, NodeModelConfig};

pub struct NetworkModel<RM> {
    node: NodeModel<RM>,

    #[deprecated = "please clean this up to a minimal set of easily understandable config"]
    pub topology: ChatTopology,
    phantom: PhantomData<RM>,
}

impl<RM> NetworkModel<RM> {
    pub fn new(node: NodeModel<RM>, topology: ChatTopology) -> Self {
        Self {
            node,
            topology,
            phantom: PhantomData,
        }
    }
}

impl<RM> polestar::Machine for NetworkModel<RM>
where
    RM: polestar::Machine<Error = anyhow::Error> + simdash_action::routing::RoutingDataExt,
{
    type State = simdash_state::network::NetworkState<RM::State>;
    type Action = simdash_action::network::NetworkAction<RM::Action>;
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

                let existing_ids: BTreeSet<u32> = state
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
                let fx = match state.graph.update_node(node_id, |node_state| {
                    self.node.transition(node_state, node_action)
                }) {
                    Ok(fx) => fx,
                    Err(e) => {
                        tracing::error!("Failed to transition node {node_id:?}: {e:?}");
                        None
                    }
                };

                if let Some(fx) = fx {
                    let actions = RM::fx_to_network_actions(fx.routing, node_id, &state);
                    let (new_state, fxs) = self.transitions(state, actions)?;
                    assert!(fxs.is_empty());
                    state = new_state;
                }

                Ok((state, ()))
            }
        }
    }
}

impl<RM> NetworkModel<RM>
where
    RM: polestar::Machine<Error = anyhow::Error> + simdash_action::routing::RoutingDataExt,
{
    fn unspawned_edge_topos<'a>(
        &'a self,
        existing_ids: &'a BTreeSet<u32>,
    ) -> impl Iterator<Item = &'a net_topology::net_topology::EdgeTopology> {
        self.topology
            .net
            .edges
            .iter()
            .filter(|e| e.spawn_chance.is_some() && !existing_ids.contains(&e.id))
    }
}

// TODO: get this code back in the game

/*

    pub fn apply_node_effect(
        &self,
        state: &mut NetworkState<RM::State>,
        node_id: NodeId,
        fx: NodeFx<RM::Fx>,
    ) -> anyhow::Result<()> {
        self.apply_routing_effect(state, node_id, fx.routing)
    }

    // TODO: consider applying effects in the behavior, and in particular
    //       collecting all effects from all actions in a tick and applying them at once,
    //       instead of immediately recusively applying them here.
    pub fn apply_routing_effect<D>(
        &self,
        state: &mut NetworkState<RM::State>,
        from: NodeId,
        fx: NetworkRoutingFx<D>,
    ) -> anyhow::Result<()> {
        match fx {
            NetworkRoutingFx::BroadcastData { data } => {
                let neighbors = state.neighborhood(from).collect::<Vec<_>>();
                for neighbor in neighbors {
                    state.graph.update_node(neighbor, |mut node_state| {
                        self.node.transition(
                            node_state,
                            NodeAction::Routing(RoutingAction::Network(
                                NetworkRoutingAction::ReceiveData { from, data },
                            )),
                        )
                    })?;
                }
            }
            RoutingFx::Flooding(Some(FloodingFx::StoreOp(op))) => {
                let op_id = op.id;
                let stored = state.graph.update_node(from, |mut node_state| {
                    node_state.dash.store_op(op)?;
                    Ok((node_state, ()))
                });
                if stored.is_err() {
                    // Chat rejected the op (e.g. capacity exceeded);
                    // remove it from routing state so it won't be broadcast.
                    state.graph.update_node(from, |mut node_state| {
                        node_state.routing.flooding.ops.remove(&op_id);
                        Ok((node_state, ()))
                    })?;
                }
            }
            RoutingFx::Flooding(None) => {}
        }
        Ok(())
    }

*/
