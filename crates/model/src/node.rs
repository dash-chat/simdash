use polestar::util::update_replace;
use simdash_action::{dash::DashAction, node::NodeAction, routing::flooding::FloodingAction};

use crate::{
    dash::DashModel,
    routing::{RoutingFx, RoutingModel},
};

pub struct NodeModel {
    dash: DashModel,
    routing: RoutingModel,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct NodeFx {
    pub routing: RoutingFx,
}

impl polestar::Machine for NodeModel {
    type State = simdash_state::node::NodeState;
    type Action = simdash_action::node::NodeAction;
    type Fx = NodeFx;
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        Ok(match action {
            NodeAction::Dash(action) => {
                // Always trigger a StoreOp on the routing state if an op is stored in dash chat.
                // (idempotent.)
                match &action {
                    DashAction::StoreOp(op) => {
                        let (routing, fx) = self.routing.transition(
                            state.routing,
                            FloodingAction::StoreOp { op: op.clone() },
                        )?;
                        state.routing = routing;
                    }
                    _ => {}
                }

                state.dash = self.dash.transition_(state.dash, action)?;
                (state, NodeFx::default())
            }
            NodeAction::Routing(action) => {
                let (routing, fx) = self.routing.transition(state.routing, action)?;
                state.routing = routing;
                (state, NodeFx { routing: fx })
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use polestar::Machine;
    use simdash_action::{dash::DashAction, node::NodeAction, routing::RoutingAction};
    use simdash_ids::{NodeId, OpId, Size};
    use simdash_state::{
        dash::{Body, Header, Operation},
        node::NodeState,
    };

    use super::*;
    use crate::{dash::DashModel, routing::RoutingModel};

    fn make_op(id: u32) -> Operation {
        Operation {
            id: OpId::from(id),
            header: Header {
                author: NodeId::from(1),
                payload_size: Size::from(100u64),
            },
            body: Body::DirectMessage {
                recipient: NodeId::from(2),
            },
        }
    }

    fn model() -> NodeModel {
        NodeModel {
            dash: DashModel,
            routing: RoutingModel {
                broadcast_interval: 3.into(),
            },
        }
    }

    /// DashAction::StoreOp should populate both dash.op_store and routing.ops.
    #[test]
    fn store_op_syncs_to_routing() {
        let m = model();
        let state = NodeState::default();
        let op = make_op(1);
        let op_id = op.id;
        let (state, _) = m
            .transition(state, NodeAction::Dash(DashAction::StoreOp(op)))
            .unwrap();
        assert!(state.dash.op_store.contains_key(&op_id));
        assert!(state.routing.ops.contains_key(&op_id));
    }

    /// Other DashActions (e.g. AddContact) should not touch routing state.
    #[test]
    fn add_contact_does_not_affect_routing() {
        let m = model();
        let state = NodeState::default();
        let contact = NodeId::from(99);
        let (state, _) = m
            .transition(state, NodeAction::Dash(DashAction::AddContact(contact)))
            .unwrap();
        assert!(state.dash.contacts.contains(&contact));
        assert!(state.routing.ops.is_empty());
    }

    /// After StoreOp via Dash, TriggerBroadcast via Routing should include that op.
    #[test]
    fn trigger_broadcast_after_store_op() {
        let m = model();
        let state = NodeState::default();
        let op = make_op(1);
        let op_id = op.id;
        let (state, _) = m
            .transition(state, NodeAction::Dash(DashAction::StoreOp(op)))
            .unwrap();
        let (_, fx) = m
            .transition(state, NodeAction::Routing(RoutingAction::TriggerBroadcast))
            .unwrap();
        assert!(fx.routing.broadcast_ops.contains(&op_id));
    }

    /// ReceiveOp via Routing stores in routing state only — not in dash.
    #[test]
    fn receive_op_stays_in_routing_only() {
        let m = model();
        let state = NodeState::default();
        let op = make_op(42);
        let op_id = op.id;
        let (state, _) = m
            .transition(
                state,
                NodeAction::Routing(RoutingAction::ReceiveOp {
                    from: NodeId::from(9),
                    op,
                }),
            )
            .unwrap();
        assert!(state.routing.ops.contains_key(&op_id));
        assert!(!state.dash.op_store.contains_key(&op_id));
    }
}
