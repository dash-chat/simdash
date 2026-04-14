use simdash_types::{GroupId, NodeId, Operation};

/// Actions for [`simdash_model::chat::ChatModel`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChatAction {
    /// Store an operation in the chat state.
    StoreOp(Operation),
    /// Add a contact to the chat state.
    // TODO: mostly a placeholder
    AddContact(NodeId),
    /// Subscribe to a group in the chat state.
    // TODO: mostly a placeholder
    JoinGroup(GroupId),
}
