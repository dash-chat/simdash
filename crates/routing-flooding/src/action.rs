use simdash_state::chat::Operation;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FloodingAction {
    StoreOp { op: Operation },
    TriggerBroadcast,
}
