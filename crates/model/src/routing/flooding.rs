use simdash_state::dash::Operation;

pub struct FloodingModel;

impl polestar::Machine for FloodingModel {
    type State = simdash_state::routing::flooding::FloodingState<Operation>;
    type Action = simdash_action::routing::flooding::FloodingAction;
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
