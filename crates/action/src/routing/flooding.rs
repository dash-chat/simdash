use simdash_ids::{NodeId, OpId};
use simdash_state::{dash::Operation, routing::flooding::FloodingOp};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FloodingAction {
    StoreOp { op: Operation },
    ReceiveOp { from: NodeId, op: Operation },
    TriggerBroadcast,
}
