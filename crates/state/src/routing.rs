use simdash_ids::*;

use crate::Operation;

/// Only one possible routing state for now.
pub type RoutingState = FloodingState<Operation>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FloodingState<M: FloodingMetadata> {
    data: Vec<FloodingOp<M>>,
}

/// Metadata needed for flooding.
pub trait FloodingMetadata {
    fn size(&self) -> Size;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FloodingOp<M: FloodingMetadata> {
    id: OpId,
    metadata: M,
    last_sent: Time,
}
