pub struct FloodingModel;

impl polestar::Machine for FloodingModel {
    type State = simdash_state::RoutingState;
    type Action = simdash_action::RoutingAction;
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
