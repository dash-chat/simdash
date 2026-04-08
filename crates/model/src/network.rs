use crate::node::NodeModel;

pub struct NetworkModel {
    node: NodeModel,
}

impl polestar::Machine for NetworkModel {
    type State = simdash_state::State;
    type Action = simdash_action::Action;
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
