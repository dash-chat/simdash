use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use simdash_ids::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FloodingState {
    pub(crate) ops: BTreeMap<OpId, FloodingOp>,
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

/// The data sent over the wire between nodes using the flooding protocol.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct FloodingWireData {
    pub id: OpId,
    pub metadata: FloodingMetadata,
    pub encrypted: EncryptedOp,
}

impl FloodingWireData {
    pub fn new(op: Operation) -> Self {
        Self {
            id: op.id,
            metadata: op.extract_metadata(),
            encrypted: EncryptedOp::from(op),
        }
    }
}

/// The state of an op tracked by the flooding protocol.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub(crate) struct FloodingOpState {
    pub since_last_send: Option<Ticks>,
}

impl FloodingOpState {
    pub fn tick(&mut self) {
        self.since_last_send = self.since_last_send.map(|ticks| ticks + 1.into());
    }

    pub fn touch(&mut self) {
        self.since_last_send = Some(0.into());
    }
}

/// The combination of a [`FloodingWireData`] and its [`FloodingOpState`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub(crate) struct FloodingOp {
    pub data: FloodingWireData,
    pub state: FloodingOpState,
}

impl FloodingOp {
    pub fn from_wire(data: FloodingWireData) -> Self {
        Self {
            data,
            state: FloodingOpState::default(),
        }
    }

    pub fn id(&self) -> OpId {
        self.data.id
    }

    #[allow(unused)]
    pub fn from_op(op: Operation) -> Self {
        Self::from_wire(FloodingWireData::new(op))
    }

    pub fn tick(&mut self) {
        self.state.tick();
    }

    pub fn touch(&mut self) {
        self.state.touch();
    }
}
