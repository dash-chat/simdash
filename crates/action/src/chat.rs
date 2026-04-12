use simdash_types::{GroupId, NodeId, Operation};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChatAction {
    StoreOp(Operation),
    AddContact(NodeId),
    JoinGroup(GroupId),
}
