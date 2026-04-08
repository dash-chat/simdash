use simdash_ids::NodeId;
use simdash_state::dash::{GroupId, Operation};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DashAction {
    StoreOp(Operation),
    AddContact(NodeId),
    JoinGroup(GroupId),
}
