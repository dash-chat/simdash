use simdash_ids::NodeId;
use simdash_state::chat::{GroupId, Operation};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChatAction {
    StoreOp(Operation),
    AddContact(NodeId),
    JoinGroup(GroupId),
}
