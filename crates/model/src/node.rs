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
        state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        todo!()
    }
}
