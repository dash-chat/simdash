use simdash_routing_common::ProtocolAction;

use crate::FloodingWireData;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FloodingAction {
    StoreOp(FloodingWireData),
    TriggerBroadcast,
}

impl From<FloodingAction> for ProtocolAction<FloodingAction, FloodingWireData> {
    fn from(action: FloodingAction) -> Self {
        ProtocolAction::Protocol(action)
    }
}
