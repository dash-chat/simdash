use named_id::RenameAll;

use crate::FloodingWireData;

/// Actions specific to the Flooding protocol.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, RenameAll)]
pub enum FloodingAction {
    /// An action to begin tracking an operation in the flooding state
    StoreOp(FloodingWireData),
    /// An action to trigger a broadcast of whatever operations should be broadcast
    /// for the current tick.
    TriggerBroadcast,
}
