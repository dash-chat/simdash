use simdash_types::Behavior;

use crate::{FloodingAction, FloodingModel, FloodingState, ProtocolAction, ProtocolFx};

/// A behavior that just triggers flooding as often as possible
pub struct EagerFloodingBehavior;

impl Behavior<FloodingModel> for EagerFloodingBehavior {
    fn next_tick(&mut self, _: &FloodingState) -> anyhow::Result<Vec<ProtocolAction>> {
        Ok(vec![FloodingAction::TriggerBroadcast.into()])
    }

    fn handle_fx(
        &mut self,
        _: &FloodingState,
        _: Vec<ProtocolFx>,
    ) -> anyhow::Result<Option<Vec<ProtocolFx>>> {
        Ok(None)
    }
}
