use simdash_action::routing::flooding::FloodingAction;
use simdash_model::routing::flooding::FloodingModel;
use simdash_state::routing::flooding::FloodingState;

use crate::Behavior;

pub struct NaiveFloodingBehavior;

impl Behavior<FloodingModel> for NaiveFloodingBehavior {
    fn next(&mut self, _: &FloodingState) -> anyhow::Result<Option<FloodingAction>> {
        Ok(Some(FloodingAction::TriggerBroadcast))
    }
}
