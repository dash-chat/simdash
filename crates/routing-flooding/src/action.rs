use simdash_routing_common::ProtocolAction;

use crate::FloodingOp;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FloodingAction {
    StoreOp(FloodingOp),
    TriggerBroadcast,
}

impl From<FloodingAction> for ProtocolAction<FloodingAction, FloodingOp> {
    fn from(action: FloodingAction) -> Self {
        ProtocolAction::Protocol(action)
    }
}
