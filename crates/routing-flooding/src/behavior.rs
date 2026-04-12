use simdash_action::routing::RoutingAction;
use simdash_behavior::Behavior;

use crate::{FloodingAction, FloodingModel, FloodingOp, FloodingState};

pub struct NaiveFloodingBehavior;

impl Behavior<FloodingModel> for NaiveFloodingBehavior {
    fn next(
        &mut self,
        _: &FloodingState,
    ) -> anyhow::Result<Vec<RoutingAction<FloodingAction, FloodingOp>>> {
        Ok(vec![FloodingAction::TriggerBroadcast.into()])
    }
}
