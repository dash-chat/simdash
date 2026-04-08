use simdash_ids::*;

use crate::dash::Operation;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FloodingState<M: FloodingMetadata = Operation> {
    data: Vec<FloodingOp<M>>,
}

impl<M: FloodingMetadata> Default for FloodingState<M> {
    fn default() -> Self {
        Self { data: vec![] }
    }
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
