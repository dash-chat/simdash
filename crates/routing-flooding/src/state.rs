use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use simdash_ids::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FloodingState {
    pub ops: BTreeMap<OpId, FloodingOp>,
}

impl FloodingState {
    pub fn tick(&mut self) {
        self.ops.values_mut().for_each(|op| op.tick());
    }
}

/// Metadata needed for flooding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct FloodingMetadata {
    pub size: Size,
}

impl ExtractMetadata<FloodingMetadata> for Operation {
    fn extract_metadata(&self) -> FloodingMetadata {
        FloodingMetadata {
            size: self.header.payload_size,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct FloodingOp {
    pub id: OpId,
    pub metadata: FloodingMetadata,
    pub encrypted: EncryptedOp,
    pub since_last_send: Option<Ticks>,
}

impl FloodingOp {
    pub fn new(op: Operation) -> Self {
        Self {
            id: op.id,
            metadata: op.extract_metadata(),
            encrypted: EncryptedOp::from(op),
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
