use polestar::util::update_replace;
use simdash_action::{chat::ChatAction, node::NodeAction};
use simdash_ids::Ticks;
use simdash_routing::{RoutingAction, RoutingFx, RoutingModel};

use crate::chat::ChatModel;

pub struct NodeModel {
    pub dash: ChatModel,
    pub routing: RoutingModel,
}

pub struct NodeModelConfig {
    pub broadcast_interval: Ticks,
}

impl NodeModel {
    pub fn new(routing: RoutingModel) -> Self {
        Self {
            dash: ChatModel,
            routing,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use]
pub struct NodeFx {
    pub routing: RoutingFx,
}

impl polestar::Machine for NodeModel {
    type State = simdash_state::node::NodeState;
    type Action = simdash_action::node::NodeAction;
    type Fx = Option<NodeFx>;
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        Ok(match action {
            NodeAction::Chat(action) => {
                // Always trigger a StoreOp on the routing state if an op is stored in dash chat.
                // (idempotent.)
                //
                // TODO: should this be part of the generic routing interface or must it be triggered manually in behavior?
                //
                // match &action {
                //     ChatAction::StoreOp(op) => {
                //         let (routing, fx) = self.routing.transition(
                //             state.routing,
                //             FloodingAction::StoreOp { op: op.clone() }.into(),
                //         )?;
                //         state.routing = routing;
                //     }
                //     _ => {}
                // }

                state.dash = self.dash.transition_(state.dash, action)?;
                (state, None)
            }
            NodeAction::Routing(action) => {
                let (routing, fx) = self.routing.transition(state.routing, action)?;
                state.routing = routing;
                (state, fx.map(|fx| NodeFx { routing: fx }))
            }
            NodeAction::Tick => {
                let (routing, fx) = self.routing.tick(state.routing)?;
                state.routing = routing;
                (state, None)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use polestar::Machine;
    use pretty_assertions::assert_eq;
    use simdash_action::{chat::ChatAction, node::NodeAction};
    use simdash_ids::{NodeId, OpId, Size, util::basic_op};
    use simdash_routing::{RoutingDataDispatch, RoutingFxDispatch, RoutingModel};
    use simdash_routing_common::{ProtocolAction, ProtocolFx, ProtocolNetworkAction, SendFx};
    use simdash_routing_flooding::{FloodingAction, FloodingFx, FloodingModel, FloodingWireData};
    use simdash_state::node::NodeState;

    use super::*;
    use crate::chat::ChatModel;

    fn model() -> NodeModel {
        NodeModel::new(RoutingModel {
            flooding: FloodingModel {
                broadcast_interval: 3.into(),
            },
        })
    }

    /// Routing used to be synced when ops were stored. Not anymore, for now.
    #[test]
    fn store_op_does_not_sync_to_routing() {
        let m = model();
        let state = NodeState::default();
        let op = basic_op(1);
        let op_id = op.id;
        let (state, _) = m
            .transition(state, NodeAction::Chat(ChatAction::StoreOp(op)))
            .unwrap();
        assert!(state.dash.op_store.contains_key(&op_id));
        assert!(!state.routing.flooding.op_ids().contains(&op_id));
    }

    /// Other ChatActions (e.g. AddContact) should not touch routing state.
    #[test]
    fn add_contact_does_not_affect_routing() {
        let m = model();
        let state = NodeState::default();
        let contact = NodeId::from(99);
        let (state, _) = m
            .transition(state, NodeAction::Chat(ChatAction::AddContact(contact)))
            .unwrap();
        assert!(state.dash.contacts.contains(&contact));
        assert!(state.routing.flooding.op_ids().is_empty());
    }

    /// After StoreOp via Chat, TriggerBroadcast via Routing should include that op.
    #[test]
    fn trigger_broadcast_after_store_op() {
        let m = model();
        let state = NodeState::default();
        let op = basic_op(1);
        let data = FloodingWireData::new(op.clone());
        let store_chat = NodeAction::Chat(ChatAction::StoreOp(op));
        let store_routing = NodeAction::Routing(simdash_routing::routing_flooding_action(
            FloodingAction::StoreOp(data.clone()),
        ));
        let (state, _) = m.transitions(state, [store_chat, store_routing]).unwrap();
        let (_, fx) = m
            .transition(
                state,
                NodeAction::Routing(simdash_routing::routing_flooding_action(
                    FloodingAction::TriggerBroadcast,
                )),
            )
            .unwrap();
        assert_eq!(
            fx.unwrap().routing,
            RoutingFx::Network(SendFx::BroadcastData {
                data: [RoutingDataDispatch::Flooding(data)].into_iter().collect(),
            })
        );
    }

    /// ReceiveOp via Routing stores in routing state only — not in dash.
    #[test]
    fn receive_op_stays_in_routing_only() {
        let m = model();
        let state = NodeState::default();
        let op = basic_op(42);
        let op_id = op.id;
        let (state, fx) = m
            .transition(
                state,
                NodeAction::Routing(RoutingAction::Network(ProtocolNetworkAction::Receive {
                    from: NodeId::from(9),
                    data: RoutingDataDispatch::Flooding(FloodingWireData::new(op.clone())),
                })),
            )
            .unwrap();
        assert!(state.routing.flooding.op_ids().contains(&op_id));
        assert!(!state.dash.op_store.contains_key(&op_id));
    }
}
