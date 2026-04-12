use std::marker::PhantomData;

use polestar::util::update_replace;
use simdash_action::{
    chat::ChatAction,
    node::NodeAction,
    routing::{RoutingAction, RoutingDataExt, RoutingFx},
};
use simdash_ids::Ticks;

use crate::chat::ChatModel;

pub struct NodeModel<RM> {
    pub dash: ChatModel,
    pub routing: RM,
}

pub struct NodeModelConfig {
    pub broadcast_interval: Ticks,
}

impl<RM> NodeModel<RM>
where
    RM: polestar::Machine<Error = anyhow::Error> + simdash_action::routing::RoutingDataExt,
{
    pub fn new(routing: RM) -> Self {
        Self {
            dash: ChatModel,
            routing,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use]
pub struct NodeFx<RFx> {
    pub routing: RFx,
}

impl<RM> polestar::Machine for NodeModel<RM>
where
    RM: polestar::Machine<Error = anyhow::Error> + simdash_action::routing::RoutingDataExt,
{
    type State = simdash_state::node::NodeState<RM::State>;
    type Action = simdash_action::node::NodeAction<RM::Action>;
    type Fx = Option<NodeFx<RM::Fx>>;
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        Ok(match action {
            NodeAction::Chat(action) => {
                tracing::warn!("make sure routing state adds the op too");

                state.dash = self.dash.transition_(state.dash, action)?;
                (state, None)
            }
            NodeAction::Routing(action) => {
                let (routing, fx) = self.routing.transition(state.routing, action)?;
                state.routing = routing;
                (state, Some(NodeFx { routing: fx }))
            }
            NodeAction::Tick => {
                let (routing, fx) = self.routing.transition(state.routing, RM::produce_tick())?;
                state.routing = routing;
                (state, None)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use polestar::{Machine, machine::EmptyMachine};
    use simdash_action::{chat::ChatAction, node::NodeAction};
    use simdash_ids::{NodeId, OpId, Size};
    use simdash_state::{
        chat::{Body, Header, Operation},
        node::NodeState,
    };

    use super::*;
    use crate::chat::ChatModel;
    use simdash_state::util::basic_op;

    fn model() -> NodeModel<EmptyMachine> {
        NodeModel::new(EmptyMachine::default())
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
        todo!()
        // assert!(state.routing.flooding.ops.contains_key(&op_id));
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

        todo!()
        // let (_, fx) = m
        //     .transition(
        //         state,
        //         NodeAction::Routing(FloodingAction::TriggerBroadcast.into()),
        //     )
        //     .unwrap();
        // assert_eq!(
        //     fx.unwrap().routing,
        //     RoutingFx::Flooding(Some(FloodingFx::BroadcastOps(
        //         [op_id].into_iter().collect()
        //     )))
        // );
    }
}
