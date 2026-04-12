use crate::FloodingWireData;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FloodingAction {
    StoreOp(FloodingWireData),
    TriggerBroadcast,
}
