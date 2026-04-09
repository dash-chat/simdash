use anyhow::bail;
use simdash_action::chat::ChatAction;
use simdash_state::chat::ChatState;

pub struct ChatModel;

impl polestar::Machine for ChatModel {
    type State = ChatState;
    type Action = ChatAction;
    type Fx = ();
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        match action {
            ChatAction::StoreOp(op) => {
                if let Some(existing) = state.op_store.insert(op.id, op) {
                    bail!("Op already exists: {:?}", existing.id);
                }
                Ok((state, ()))
            }
            ChatAction::AddContact(contact) => {
                state.contacts.insert(contact);
                Ok((state, ()))
            }
            ChatAction::JoinGroup(group) => {
                state.groups.insert(group);
                Ok((state, ()))
            }
        }
    }
}
