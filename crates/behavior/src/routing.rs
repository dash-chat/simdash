use simdash_action::routing::{RoutingAction, flooding::FloodingAction};
use simdash_model::routing::RoutingModel;
use simdash_state::routing::RoutingState;

use crate::Behavior;

pub struct NaiveFloodingBehavior;

impl Behavior<RoutingModel> for NaiveFloodingBehavior {
    fn next(&mut self, _: &RoutingState) -> anyhow::Result<Option<RoutingAction>> {
        Ok(Some(FloodingAction::TriggerBroadcast.into()))
    }
}
