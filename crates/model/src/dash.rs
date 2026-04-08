pub struct DashModel;

impl polestar::Machine for DashModel {
    type State = simdash_state::DashState;
    type Action = simdash_action::DashAction;
    type Fx = ();
    type Error = ();

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        todo!()
    }
}
