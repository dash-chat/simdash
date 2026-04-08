use polestar::util::update_replace;
use simdash_action::{dash::DashAction, node::NodeAction, routing::flooding::FloodingAction};

use crate::{dash::DashModel, routing::RoutingModel};

pub struct NodeModel {
    dash: DashModel,
    routing: RoutingModel,
}

impl polestar::Machine for NodeModel {
    type State = simdash_state::node::NodeState;
    type Action = simdash_action::node::NodeAction;
    type Fx = ();
    type Error = ();

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        match action {
            NodeAction::Dash(action) => {
                // Always trigger a StoreOp on the routing state if an op is stored in dash chat.
                // (idempotent.)
                match &action {
                    DashAction::StoreOp(op) => {
                        state.routing = self.routing.transition_(
                            state.routing,
                            FloodingAction::StoreOp { op: op.clone() },
                        )?;
                    }
                    _ => {}
                }

                state.dash = self.dash.transition_(state.dash, action)?;
            }
            NodeAction::Routing(action) => {
                state.routing = self.routing.transition_(state.routing, action)?;
            }
        }
        Ok((state, ()))
    }
}
