use polestar::util::update_replace;
use simdash_action::{
    chat::ChatAction,
    node::NodeAction,
    routing::{RoutingAction, flooding::FloodingAction},
};
use simdash_ids::Ticks;

use crate::{
    chat::ChatModel,
    routing::{RoutingFx, RoutingModel, flooding::FloodingModel},
};

pub struct NodeModel {
    pub dash: ChatModel,
    pub routing: RoutingModel,
}

pub struct NodeModelConfig {
    pub broadcast_interval: Ticks,
}

impl NodeModel {
    pub fn new(config: NodeModelConfig) -> Self {
        Self {
            dash: ChatModel,
            routing: RoutingModel {
                flooding: FloodingModel {
                    broadcast_interval: config.broadcast_interval,
                },
            },
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
                match &action {
                    ChatAction::StoreOp(op) => {
                        let (routing, fx) = self.routing.transition(
                            state.routing,
                            FloodingAction::StoreOp { op: op.clone() }.into(),
                        )?;
                        state.routing = routing;
                    }
                    _ => {}
                }

                state.dash = self.dash.transition_(state.dash, action)?;
                (state, None)
            }
            NodeAction::Routing(action) => {
                let (routing, fx) = self.routing.transition(state.routing, action)?;
                state.routing = routing;
                (state, Some(NodeFx { routing: fx }))
            }
            NodeAction::Tick => {
                let (routing, fx) = self
                    .routing
                    .transition(state.routing, FloodingAction::Tick.into())?;
                state.routing = routing;
                (state, None)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use polestar::Machine;
    use simdash_action::{chat::ChatAction, node::NodeAction, routing::RoutingAction};
    use simdash_ids::{NodeId, OpId, Size, util::basic_op};
    use simdash_state::node::NodeState;

    use super::*;
    use crate::{
        chat::ChatModel,
        routing::{RoutingModel, flooding::FloodingFx},
    };

    fn model() -> NodeModel {
        NodeModel::new(NodeModelConfig {
            broadcast_interval: 3.into(),
        })
    }

    /// ChatAction::StoreOp should populate both dash.op_store and routing.ops.
    #[test]
    fn store_op_syncs_to_routing() {
        let m = model();
        let state = NodeState::default();
        let op = basic_op(1);
        let op_id = op.id;
        let (state, _) = m
            .transition(state, NodeAction::Chat(ChatAction::StoreOp(op)))
            .unwrap();
        assert!(state.dash.op_store.contains_key(&op_id));
        assert!(state.routing.flooding.ops.contains_key(&op_id));
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
        assert!(state.routing.flooding.ops.is_empty());
    }

    /// After StoreOp via Chat, TriggerBroadcast via Routing should include that op.
    #[test]
    fn trigger_broadcast_after_store_op() {
        let m = model();
        let state = NodeState::default();
        let op = basic_op(1);
        let op_id = op.id;
        let (state, _) = m
            .transition(state, NodeAction::Chat(ChatAction::StoreOp(op)))
            .unwrap();
        let (_, fx) = m
            .transition(
                state,
                NodeAction::Routing(FloodingAction::TriggerBroadcast.into()),
            )
            .unwrap();
        assert_eq!(
            fx.unwrap().routing,
            RoutingFx::Flooding(Some(FloodingFx::BroadcastOps(
                [op_id].into_iter().collect()
            )))
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
                NodeAction::Routing(
                    FloodingAction::ReceiveOp {
                        from: NodeId::from(9),
                        op: op.clone(),
                    }
                    .into(),
                ),
            )
            .unwrap();
        assert!(state.routing.flooding.ops.contains_key(&op_id));
        assert!(!state.dash.op_store.contains_key(&op_id));
    }
}
