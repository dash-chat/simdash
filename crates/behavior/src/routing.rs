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

/// A no-op routing behavior that never triggers any routing actions.
/// Useful as a baseline to observe the initial state without propagation.
pub struct NoOpRoutingBehavior;

impl Behavior<RoutingModel> for NoOpRoutingBehavior {
    fn next(&mut self, _: &RoutingState) -> anyhow::Result<Option<RoutingAction>> {
        Ok(None)
    }
}
