use simdash_ids::{NodeId, OpId, Operation};
use simdash_state::routing::flooding::FloodingOp;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FloodingAction {
    StoreOp { op: Operation },
    ReceiveOp { from: NodeId, op: Operation },
    TriggerBroadcast,
    Tick,
}
