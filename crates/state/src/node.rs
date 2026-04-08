use derive_more::{Deref, From};

use crate::{DashState, RoutingState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Deref)]
pub struct NodeId(u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeState<R = RoutingState> {
    dash: DashState,
    routing: R,
}
