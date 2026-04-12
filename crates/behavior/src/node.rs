use std::marker::PhantomData;

use simdash_action::{node::NodeAction, routing::RoutingDataExt};
use simdash_model::node::NodeModel;
use simdash_state::node::NodeState;

use crate::Behavior;

#[derive(Default)]
pub struct BasicRoutingBehavior<B, RM>
where
    B: Behavior<NodeModel<RM>>,
    RM: RoutingDataExt,
{
    routing_behavior: B,
    _phantom: PhantomData<RM>,
}

impl<B, RM> BasicRoutingBehavior<B, RM>
where
    B: Behavior<NodeModel<RM>>,
    RM: RoutingDataExt,
{
    pub fn new(routing_behavior: B) -> Self {
        Self {
            routing_behavior,
            _phantom: PhantomData,
        }
    }
}

impl<B, RM> Behavior<NodeModel<RM>> for BasicRoutingBehavior<B, RM>
where
    B: Behavior<NodeModel<RM>>,
    RM: RoutingDataExt,
{
    fn next(
        &mut self,
        state: &NodeState<RM::State>,
    ) -> anyhow::Result<Vec<NodeAction<RM::Action>>> {
        Ok(self
            .routing_behavior
            .next(&state)?
            .into_iter()
            // .map(NodeAction::Routing)
            .collect())
    }
}
