use simdash_types::Behavior;

use crate::*;

/// A routing behavior that only generates actions for the naive Flooding protocol.
pub struct NaiveFloodingBehavior;

impl Behavior for NaiveFloodingBehavior {
    type Model = RoutingDispatchModel;

    fn next_tick(
        &mut self,
        state: &RoutingDispatchState,
    ) -> anyhow::Result<Vec<RoutingDispatchAction>> {
        let mut b = crate::flooding::EagerFloodingBehavior;
        let actions = b
            .next_tick(&state.flooding)?
            .into_iter()
            .map(|a| crate::lift_action(a, AnyRoutingData::Flooding, AnyRoutingAction::Flooding))
            .collect();
        Ok(actions)
    }

    fn handle_fx(
        &mut self,
        _state: &RoutingDispatchState,
        _fx: Vec<RoutingDispatchFx>,
    ) -> anyhow::Result<Option<Vec<RoutingDispatchFx>>> {
        Ok(None)
    }
}

/// A routing behavior that only generates actions for the FloodingSync protocol.
pub struct BasicFloodingSyncBehavior;

impl Behavior for BasicFloodingSyncBehavior {
    type Model = RoutingDispatchModel;

    fn next_tick(
        &mut self,
        state: &RoutingDispatchState,
    ) -> anyhow::Result<Vec<RoutingDispatchAction>> {
        let mut b = crate::flooding_sync::StandardBehavior;
        let actions = b
            .next_tick(&state.flooding_sync)?
            .into_iter()
            .map(|a| {
                crate::lift_action(
                    a,
                    AnyRoutingData::FloodingSync,
                    AnyRoutingAction::FloodingSync,
                )
            })
            .collect();
        Ok(actions)
    }
}

/// A routing behavior that only generates actions for the FloodingSync protocol.
pub struct BasicEccoBehavior;

impl Behavior for BasicEccoBehavior {
    type Model = RoutingDispatchModel;

    fn next_tick(
        &mut self,
        state: &RoutingDispatchState,
    ) -> anyhow::Result<Vec<RoutingDispatchAction>> {
        let mut b = crate::ecco::StandardBehavior::default();
        let actions = b
            .next_tick(&state.ecco)?
            .into_iter()
            .map(|a| crate::lift_action(a, AnyRoutingData::Ecco, AnyRoutingAction::Ecco))
            .collect();
        Ok(actions)
    }
}

/// A no-op routing behavior that never triggers any routing actions.
/// Useful as a baseline to observe the initial state without propagation.
pub struct NoOpRoutingBehavior;

impl Behavior for NoOpRoutingBehavior {
    type Model = RoutingDispatchModel;

    fn next_tick(
        &mut self,
        _: &RoutingDispatchState,
    ) -> anyhow::Result<Vec<RoutingDispatchAction>> {
        Ok(vec![])
    }
}
