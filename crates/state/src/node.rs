use derive_more::{Deref, From};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Deref)]
pub struct NodeId(u32);

pub struct NodeState<R = RoutingState> {
    dash: DashState,
    routing: R,
}

pub struct DashState {}

pub struct RoutingState {}
