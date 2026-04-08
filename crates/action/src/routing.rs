use simdash_ids::{NodeId, OpId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoutingAction {
    ReceiveOp { from: NodeId, op: OpId },
}
