//! Behaviors for the network level.

use simdash_model_chat::ChatAction;
use simdash_model_node::NodeAction;
use simdash_model_node::NodeModel;
use simdash_routing_common::ProtocolNetworkAction;
use simdash_routing_common::{SendFx, StorageFx};
use simdash_routing_dispatch::RoutingDispatchAction;
use simdash_routing_dispatch::RoutingDispatchFx;
use simdash_types::{Behavior, NodeId};

use crate::{NetworkAction, NetworkFx, NetworkModel, NetworkState};

/// A "fair scheduling" behavior that lets each Node in a network take one Action per tick.
///
/// This models a uniform flow of time with nodes that are perfectly identical to each other
/// in the rate that they take actions.
pub struct RoundRobinNodeScheduler<B: Behavior<Model = NodeModel>> {
    node_behavior: B,
    pending_actions: Vec<NetworkAction>,
}

impl<B: Behavior<Model = NodeModel>> RoundRobinNodeScheduler<B> {
    /// Constructor
    pub fn new(node_behavior: B) -> Self {
        Self {
            node_behavior,
            pending_actions: Vec::new(),
        }
    }
}

impl<B: Behavior<Model = NodeModel>> Behavior for RoundRobinNodeScheduler<B> {
    type Model = NetworkModel;

    fn next_tick(&mut self, state: &NetworkState) -> anyhow::Result<Vec<NetworkAction>> {
        // TODO: add propagation delay here.
        let consequences = self.pending_actions.drain(..);

        let generated = state.graph.nodes.iter().flat_map(|(node_id, node_state)| {
            let actions = self
                .node_behavior
                .next_tick(node_state)
                .expect("todo, what to do on errors?");
            actions
                .into_iter()
                .map(|action| NetworkAction::Node(*node_id, action))
        });

        let consequences = consequences.collect::<Vec<_>>();
        let generated = generated.collect::<Vec<_>>();

        println!("consequences ({}):", consequences.len());
        for c in consequences.iter() {
            println!("  {:?}", c);
        }
        println!("generated: {:?}", generated.len());

        Ok(consequences
            .into_iter()
            .chain(generated.into_iter())
            .chain([NetworkAction::Tick, NetworkAction::EvolveTopology])
            .collect())
    }

    fn handle_fx(
        &mut self,
        state: &NetworkState,
        fx: Vec<NetworkFx>,
    ) -> anyhow::Result<Option<Vec<NetworkFx>>> {
        for fx in fx {
            match fx {
                NetworkFx::Node(node_id, node_fx) => {
                    let actions = handle_single_fx(state, node_id, node_fx.routing)?;
                    self.pending_actions.extend(actions);
                }
            }
        }
        Ok(None)
    }
}

fn handle_single_fx(
    state: &NetworkState,
    node_id: NodeId,
    fx: RoutingDispatchFx,
) -> anyhow::Result<Vec<NetworkAction>> {
    let mut actions = Vec::new();
    match fx {
        RoutingDispatchFx::Network(send) => match send {
            SendFx::BroadcastData { data } => {
                let neighbors = state.neighborhood(node_id).collect::<Vec<_>>();
                for neighbor in neighbors {
                    let node_action = NodeAction::Routing(RoutingDispatchAction::Network(
                        ProtocolNetworkAction::Receive { data: data.clone() },
                    ));
                    actions.push(NetworkAction::node(neighbor, node_action));
                }
            }
            SendFx::UnicastData { to, data } => {
                let node_action = NodeAction::Routing(RoutingDispatchAction::Network(
                    ProtocolNetworkAction::Receive { data },
                ));
                actions.push(NetworkAction::node(to, node_action));
            }
        },

        // TODO: this could also be handled at the Node level, or by the Simulator.
        RoutingDispatchFx::Node(StorageFx::StoreOp { op }) => {
            let node_action = NodeAction::Chat(ChatAction::StoreOp(
                op.decrypt(&simdash_types::EncryptionKey),
            ));
            actions.push(NetworkAction::node(node_id, node_action));
        }
        RoutingDispatchFx::Protocol(fx) => {
            tracing::warn!(
                "Unhandled protocol effect (need to create NetworkFx to propagate higher up): {:?}",
                fx
            );
        }
    }
    Ok(actions)
}
