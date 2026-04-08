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
