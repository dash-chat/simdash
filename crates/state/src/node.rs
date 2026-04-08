use derive_more::{Deref, From};

use crate::{DashState, Operation, RoutingState};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct NodeState<R = RoutingState> {
    dash: DashState,
    routing: R,
}
