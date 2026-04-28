use derive_more::{Debug, Display};
use named_id::RenameAll;
use simdash_types::{GroupId, NodeId, Operation};

/// Actions for the chat model.
#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, RenameAll)]
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
