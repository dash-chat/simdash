use simdash_ids::{NodeId, OpId};
use simdash_state::{dash::Operation, routing::flooding::FloodingOp};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FloodingAction {
    Tick,
    StoreOp { op: Operation },
    TouchOp { op_id: OpId },
    ReceiveOp { from: NodeId, op: Operation },
}
