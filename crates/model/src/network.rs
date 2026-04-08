use polestar::ext::MapExt;
use simdash_action::node::NodeAction;
use simdash_ids::NodeId;
use simdash_state::{dash::Operation, network::NetworkState, node::NodeState};

use crate::{
    node::{NodeFx, NodeModel},
    routing::flooding::FloodingFx,
};

pub struct NetworkModel {
    node: NodeModel,
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
                let all_keys = state.nodes.keys().copied().collect::<Vec<_>>();
                for node_id in all_keys {
                    state
                        .nodes
                        .owned_update(node_id, |network_state, mut node_state| {
                            node_state.tick();
                            Ok((node_state, ()))
                        })?;
                }
                Ok((state, ()))
            }

            simdash_action::network::NetworkAction::Node(node_id, node_action) => {
                if let Some(fx) = state.nodes.owned_update(node_id, |_, mut node_state| {
                    let (node_state, node_fx) = self.node.transition(node_state, node_action)?;
                    Ok((node_state, node_fx))
                })? {
                    self.apply_effects(&mut state, node_id, fx)?;
                }
                Ok((state, ()))
            }
        }
    }
}

impl NetworkModel {
    pub fn apply_effects(
        &self,
        state: &mut NetworkState,
        node_id: NodeId,
        fx: NodeFx,
    ) -> anyhow::Result<()> {
        match fx.routing {
            FloodingFx::BroadcastOps(broadcast_ops) => {
                let ops = broadcast_ops
                    .iter()
                    .map(|op_id| {
                        state
                            .nodes
                            .get(&node_id)
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
                        state.send_op(node_id, neighbor, op.clone())?;
                    }
                }
            }
            FloodingFx::StoreOp(op) => {
                state
                    .nodes
                    .get_mut(&node_id)
                    .ok_or(anyhow::anyhow!("Node not found: {:?}", node_id))?
                    .dash
                    .store_op(op)?;
            }
        }
        Ok(())
    }
}
