use simdash_ids::Behavior;
use simdash_routing_common::ProtocolAction;

use crate::{FloodingAction, FloodingModel, FloodingState, FloodingWireData};

pub struct NaiveFloodingBehavior;

impl Behavior<FloodingModel> for NaiveFloodingBehavior {
    fn next(
        &mut self,
        _: &FloodingState,
    ) -> anyhow::Result<Vec<ProtocolAction<FloodingAction, FloodingWireData>>> {
        Ok(vec![FloodingAction::TriggerBroadcast.into()])
    }
}
