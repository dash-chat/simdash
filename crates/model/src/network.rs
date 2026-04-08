use polestar::ext::MapExt;
use simdash_action::node::NodeAction;
use simdash_state::dash::Operation;

use crate::node::NodeModel;

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
                    let ops: Vec<Operation> =
                        state
                            .nodes
                            .owned_update(node_id, |network_state, node_state| {
                                let (node_state, node_fx) = self
                                    .node
                                    .transition(node_state, NodeAction::Tick)
                                    .map_err(|_| anyhow::anyhow!("Node transition failed"))?;
                                let ops = node_fx
                                    .routing
                                    .broadcast_ops
                                    .iter()
                                    .map(|op_id| {
                                        node_state
                                            .dash()
                                            .op_store
                                            .get(op_id)
                                            .cloned()
                                            .ok_or(anyhow::anyhow!("Op not found: {:?}", op_id))
                                    })
                                    .collect::<anyhow::Result<Vec<_>>>()?;
                                Ok((node_state, ops))
                            })?;

                    let neighbors = state.neighborhood(node_id).collect::<Vec<_>>();
                    for neighbour in neighbors {
                        for op in ops.iter() {
                            state.send_op(node_id, neighbour, op.clone())?;
                        }
                    }
                }
                Ok((state, ()))
            }

            simdash_action::network::NetworkAction::Node(node_id, node_action) => todo!(),
        }
    }
}
