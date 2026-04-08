use crate::{DashModel, RoutingModel};

pub struct NodeModel {
    dash: DashModel,
    routing: RoutingModel,
}

impl polestar::Machine for NodeModel {
    type State = simdash_state::NodeState;
    type Action = simdash_action::NodeAction;
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
