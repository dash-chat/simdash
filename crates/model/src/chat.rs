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
                if state.op_store.contains_key(&op.id) {
                    bail!("Op already exists: {:?}", op.id);
                }
                if let Some(capacity) = state.storage_capacity {
                    let new_size = *state.stored_size() + *op.header.payload_size;
                    if new_size > *capacity {
                        bail!(
                            "Storage capacity exceeded: {} + {} > {}",
                            *state.stored_size(),
                            *op.header.payload_size,
                            *capacity
                        );
                    }
                }
                state.op_store.insert(op.id, op);
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
