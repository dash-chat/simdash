use simdash_ids::Behavior;
use simdash_routing_common::ProtocolAction;

use crate::{FloodingAction, FloodingModel, FloodingOp, FloodingState};

pub struct NaiveFloodingBehavior;

impl Behavior<FloodingModel> for NaiveFloodingBehavior {
    fn next(
        &mut self,
        _: &FloodingState,
    ) -> anyhow::Result<Vec<ProtocolAction<FloodingAction, FloodingOp>>> {
        Ok(vec![FloodingAction::TriggerBroadcast.into()])
    }
}
