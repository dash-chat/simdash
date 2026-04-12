use std::marker::PhantomData;

use simdash_action::{network::NetworkAction, routing::RoutingDataExt};
use simdash_model::{network::NetworkModel, node::NodeModel};
use simdash_state::network::NetworkState;

use crate::Behavior;

/// A behavior working on a network of nodes, which on each step:
/// - runs the behavior for each node in arbitrary order
/// - performs a tick action on the network
pub struct RoundRobinNodeScheduler<B, RM>
where
    B: Behavior<NodeModel<RM>>,
    RM: RoutingDataExt,
{
    node_behavior: B,
    _phantom: PhantomData<RM>,
}

impl<B, RM> RoundRobinNodeScheduler<B, RM>
where
    B: Behavior<NodeModel<RM>>,
    RM: RoutingDataExt,
{
    pub fn new(node_behavior: B) -> Self {
        Self {
            node_behavior,
            _phantom: PhantomData,
        }
    }
}

impl<B, RM> Behavior<NetworkModel<RM>> for RoundRobinNodeScheduler<B, RM>
where
    B: Behavior<NodeModel<RM>>,
    RM: RoutingDataExt,
{
    fn next(
        &mut self,
        state: &NetworkState<RM::State>,
    ) -> anyhow::Result<Vec<NetworkAction<RM::Action>>> {
        Ok(state
            .graph
            .nodes
            .iter()
            .flat_map(|(node_id, node_state)| {
                let actions = self
                    .node_behavior
                    .next(&node_state)
                    .expect("todo, what to do on errors?");
                actions
                    .into_iter()
                    .map(|action| NetworkAction::Node(*node_id, action))
            })
            .chain([NetworkAction::Tick, NetworkAction::EvolveTopology])
            .collect())
    }
}
