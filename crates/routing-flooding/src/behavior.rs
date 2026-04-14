use simdash_routing_common::ProtocolAction;
use simdash_types::Behavior;

use crate::{FloodingAction, FloodingModel, FloodingState, FloodingWireData};

/// A behavior that just triggers flooding as often as possible
pub struct EagerFloodingBehavior;

impl Behavior<FloodingModel> for EagerFloodingBehavior {
    fn next(
        &mut self,
        _: &FloodingState,
    ) -> anyhow::Result<Vec<ProtocolAction<FloodingAction, FloodingWireData>>> {
        Ok(vec![FloodingAction::TriggerBroadcast.into()])
    }
}
