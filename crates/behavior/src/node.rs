use std::marker::PhantomData;

use simdash_action::{node::NodeAction, routing::RoutingDataExt};
use simdash_model::node::NodeModel;
use simdash_state::node::NodeState;

use crate::Behavior;

#[derive(Default)]
pub struct NodeBehavior1<B, RM>
where
    B: Behavior<RM>,
    RM: RoutingDataExt,
{
    routing_behavior: B,
    _phantom: PhantomData<RM>,
}

impl<B, RM> NodeBehavior1<B, RM>
where
    B: Behavior<RM>,
    RM: RoutingDataExt,
{
    pub fn new(routing_behavior: B) -> Self {
        Self {
            routing_behavior,
            _phantom: PhantomData,
        }
    }
}

impl<B, RM> Behavior<RM> for NodeBehavior1<B, RM>
where
    B: Behavior<RM>,
    RM: RoutingDataExt,
{
    fn next(&mut self, state: &RM::State) -> anyhow::Result<Vec<RM::Action>> {
        Ok(self
            .routing_behavior
            .next(&state)?
            .into_iter()
            // .map(NodeAction::Routing)
            .collect())
    }
}
