use simdash_types::Behavior;

use crate::{FloodingAction, FloodingModel, FloodingState, ProtocolAction};

/// A behavior that just triggers flooding as often as possible
pub struct EagerFloodingBehavior;

impl Behavior<FloodingModel> for EagerFloodingBehavior {
    fn next(&mut self, _: &FloodingState) -> anyhow::Result<Vec<ProtocolAction>> {
        Ok(vec![FloodingAction::TriggerBroadcast.into()])
    }
}
