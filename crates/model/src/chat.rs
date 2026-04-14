//! Model of the chat state, including stored operations,
//! joined groups, contacts.

use simdash_action::chat::ChatAction;
use simdash_state::chat::ChatState;

/// The Chat model stores Dashchat-specific state, including stored operations,
/// joined groups, and contacts.
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
                state.store_op(op)?;
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
