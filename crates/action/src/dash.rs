use simdash_ids::OpId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DashAction {
    StoreOp(OpId),
}
