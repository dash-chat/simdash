use simdash_ids::*;

use crate::{Extract, dash::Operation};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct FloodingState {
    pub ops: Vec<FloodingOp>,
}

/// Metadata needed for flooding.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FloodingMetadata {
    pub size: Size,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FloodingOp {
    pub id: OpId,
    pub metadata: FloodingMetadata,
    pub since_last_send: Option<Ticks>,
}

impl FloodingOp {
    pub fn new(op: &Operation) -> Self {
        Self {
            id: op.id,
            metadata: op.extract(),
            since_last_send: None,
        }
    }

    pub fn tick(&mut self) {
        self.since_last_send = self.since_last_send.map(|ticks| ticks + 1.into());
    }

    pub fn touch(&mut self) {
        self.since_last_send = Some(0.into());
    }
}
