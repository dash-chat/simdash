use derive_more::{Deref, From};

use crate::{dash::DashState, routing::RoutingState};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct NodeState<R = RoutingState> {
    pub dash: DashState,
    pub routing: R,
}
