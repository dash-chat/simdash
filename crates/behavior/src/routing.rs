use std::marker::PhantomData;

use simdash_action::routing::{RoutingAction, RoutingDataExt};

use crate::Behavior;

pub struct StandardRoutingBehavior<RM>(PhantomData<RM>);

impl<RM> Behavior<RM> for StandardRoutingBehavior<RM>
where
    RM: RoutingDataExt,
{
    fn next(&mut self, state: &RM::State) -> anyhow::Result<Vec<RM::Action>> {
        Ok(vec![])
    }
}

// /// A no-op routing behavior that never triggers any routing actions.
// /// Useful as a baseline to observe the initial state without propagation.
// pub struct NoOpRoutingBehavior;

// impl Behavior<RoutingModel> for NoOpRoutingBehavior {
//     fn next(&mut self, _: &RoutingState) -> anyhow::Result<Vec<RoutingAction>> {
//         Ok(vec![])
//     }
// }
