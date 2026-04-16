//! Behaviors for the node and routing levels.

use simdash_routing_dispatch::{
    AnyRoutingAction, AnyRoutingData, RoutingDispatchAction, RoutingDispatchModel,
    RoutingDispatchState,
};
use simdash_types::Behavior;

use crate::{NodeAction, NodeModel, NodeState};

// ── BasicNodeBehavior ──────────────────────────────────────────────────────

/// A basic node behavior that simply generates Routing actions (no Chat actions).
#[derive(Default)]
pub struct BasicNodeBehavior<B: Behavior<RoutingDispatchModel>> {
    routing_behavior: B,
}

impl<B: Behavior<RoutingDispatchModel>> BasicNodeBehavior<B> {
    /// Constructor
    pub fn new(routing_behavior: B) -> Self {
        Self { routing_behavior }
    }
}

impl<B: Behavior<RoutingDispatchModel>> Behavior<NodeModel> for BasicNodeBehavior<B> {
    fn next(&mut self, state: &NodeState) -> anyhow::Result<Vec<NodeAction>> {
        Ok(self
            .routing_behavior
            .next(&state.routing)?
            .into_iter()
            .map(NodeAction::Routing)
            .collect())
    }
}

// ── Routing behaviors ──────────────────────────────────────────────────────

/// A no-op routing behavior that never triggers any routing actions.
/// Useful as a baseline to observe the initial state without propagation.
pub struct NoOpRoutingBehavior;

impl Behavior<RoutingDispatchModel> for NoOpRoutingBehavior {
    fn next(&mut self, _: &RoutingDispatchState) -> anyhow::Result<Vec<RoutingDispatchAction>> {
        Ok(vec![])
    }
}

/// A routing behavior that only generates actions for the naive Flooding protocol.
pub struct NaiveFloodingBehavior;

impl Behavior<RoutingDispatchModel> for NaiveFloodingBehavior {
    fn next(&mut self, state: &RoutingDispatchState) -> anyhow::Result<Vec<RoutingDispatchAction>> {
        let mut b = simdash_routing_dispatch::flooding::EagerFloodingBehavior;
        let actions = b
            .next(&state.flooding)?
            .into_iter()
            .map(|a| {
                simdash_routing_dispatch::lift_action(
                    a,
                    AnyRoutingData::Flooding,
                    AnyRoutingAction::Flooding,
                )
            })
            .collect();
        Ok(actions)
    }
}

/// A routing behavior that only generates actions for the FloodingSync protocol.
pub struct BasicFloodingSyncBehavior;

impl Behavior<RoutingDispatchModel> for BasicFloodingSyncBehavior {
    fn next(&mut self, state: &RoutingDispatchState) -> anyhow::Result<Vec<RoutingDispatchAction>> {
        let mut b = simdash_routing_dispatch::flooding_sync::StandardBehavior;
        let actions = b
            .next(&state.flooding_sync)?
            .into_iter()
            .map(|a| {
                simdash_routing_dispatch::lift_action(
                    a,
                    AnyRoutingData::FloodingSync,
                    AnyRoutingAction::FloodingSync,
                )
            })
            .collect();
        Ok(actions)
    }
}
