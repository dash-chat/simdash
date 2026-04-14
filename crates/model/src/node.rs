//! The Node model manages all of the state of a single node in the network.

use simdash_action::node::NodeAction;
use simdash_routing::{RoutingFxDispatch, RoutingModel};

use crate::chat::ChatModel;

/// The Node model manages the state of a single node in the network.
///
/// It contains the state of the node's chat app and its routing protocol.
pub struct NodeModel {
    /// The chat app model
    pub dash: ChatModel,
    /// The routing protocol model
    pub routing: RoutingModel,
}

impl NodeModel {
    /// Constructor
    pub fn new(routing: RoutingModel) -> Self {
        Self {
            dash: ChatModel,
            routing,
        }
    }
}

/// The effects that can be produced by a Node model.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct NodeFx {
    /// The effects produced by the contained routing protocol
    pub routing: RoutingFxDispatch,
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
                state.chat = self.dash.transition_(state.chat, action)?;
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
                (state, fx.map(|fx| NodeFx { routing: fx }))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use polestar::Machine;
    use pretty_assertions::assert_eq;
    use simdash_action::{chat::ChatAction, node::NodeAction};
    use simdash_routing::flooding::{FloodingAction, FloodingWireData};
    use simdash_routing::{AnyRoutingAction, AnyRoutingData, RoutingActionDispatch, RoutingModel};
    use simdash_routing_common::{ProtocolNetworkAction, SendFx};
    use simdash_state::node::NodeState;
    use simdash_types::{NodeId, util::basic_op};

    use super::*;

    fn model() -> NodeModel {
        NodeModel::new(RoutingModel::standard(3.into()))
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
        assert!(state.chat.op_store.contains_key(&op_id));
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
        assert!(state.chat.contacts.contains(&contact));
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
        let store_routing = NodeAction::Routing(RoutingActionDispatch::Protocol(
            AnyRoutingAction::Flooding(FloodingAction::StoreOp(data.clone())),
        ));
        let (state, _) = m.transitions(state, [store_chat, store_routing]).unwrap();
        let (_, fx) = m
            .transition(
                state,
                NodeAction::Routing(RoutingActionDispatch::Protocol(AnyRoutingAction::Flooding(
                    FloodingAction::TriggerBroadcast,
                ))),
            )
            .unwrap();
        assert_eq!(
            fx.unwrap().routing,
            RoutingFxDispatch::Network(SendFx::BroadcastData {
                data: [AnyRoutingData::Flooding(data)].into_iter().collect(),
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
        let (state, _) = m
            .transition(
                state,
                NodeAction::Routing(RoutingActionDispatch::Network(
                    ProtocolNetworkAction::Receive {
                        from: NodeId::from(9),
                        data: [AnyRoutingData::Flooding(FloodingWireData::new(op.clone()))]
                            .into_iter()
                            .collect(),
                    },
                )),
            )
            .unwrap();
        assert!(state.routing.flooding.op_ids().contains(&op_id));
        assert!(!state.chat.op_store.contains_key(&op_id));
    }
}
