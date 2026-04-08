use simdash_action::dash::DashAction;
use simdash_state::dash::DashState;

pub struct DashModel;

impl polestar::Machine for DashModel {
    type State = DashState;
    type Action = DashAction;
    type Fx = ();
    type Error = ();

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        match action {
            DashAction::StoreOp(op) => {
                state.op_store.insert(op.id, op);
                Ok((state, ()))
            }
            DashAction::AddContact(contact) => {
                state.contacts.insert(contact);
                Ok((state, ()))
            }
            DashAction::JoinGroup(group) => {
                state.groups.insert(group);
                Ok((state, ()))
            }
        }
    }
}
