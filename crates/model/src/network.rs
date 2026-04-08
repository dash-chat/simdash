use polestar::{Machine, ext::MapExt};
use simdash_action::{node::NodeAction, routing::RoutingAction};
use simdash_ids::NodeId;
use simdash_state::{dash::Operation, network::NetworkState, node::NodeState};

use crate::{
    node::{NodeFx, NodeModel, NodeModelConfig},
    routing::{RoutingFx, flooding::FloodingFx},
};

pub struct NetworkModel {
    node: NodeModel,
}

impl NetworkModel {
    pub fn new(config: NodeModelConfig) -> Self {
        Self {
            node: NodeModel::new(config),
        }
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
            // Tick every node and run all effects
            simdash_action::network::NetworkAction::Tick => {
                let all_keys = state.graph.node_ids().collect::<Vec<_>>();
                for node_id in all_keys {
                    state.graph.update_node(node_id, |mut node_state| {
                        node_state.tick();
                        Ok((node_state, ()))
                    })?;
                }
                Ok((state, ()))
            }

            simdash_action::network::NetworkAction::Node(node_id, node_action) => {
                if let Some(fx) = state.graph.update_node(node_id, |node_state| {
                    let (node_state, node_fx) = self.node.transition(node_state, node_action)?;
                    Ok((node_state, node_fx))
                })? {
                    self.apply_node_effects(&mut state, node_id, fx)?;
                }
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
        let fx = state.graph.update_node(to, |mut n| {
            let (new, fx) = self
                .node
                .routing
                .transition(n.routing, RoutingAction::ReceiveOp { from, op })?;
            n.routing = new;
            Ok((n, fx))
        })?;
        if let Some(fx) = fx {
            self.apply_routing_effects(state, to, fx)?;
        }
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
            FloodingFx::BroadcastOps(broadcast_ops) => {
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
                            tracing::warn!("Failed to send op to neighbor: {:?}", err);
                        }
                    }
                }
            }
            FloodingFx::StoreOp(op) => {
                state.graph.update_node(node_id, |mut node_state| {
                    node_state.dash.store_op(op)?;
                    Ok((node_state, ()))
                })?;
            }
        }
        Ok(())
    }
}
