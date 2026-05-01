use std::collections::{BTreeSet, HashMap};

use derive_more::Display;
use named_id::RenameAll;
use serde::{Deserialize, Serialize};
use simdash_types::*;

/// The flooding protocol's state stores the data to broadcast,
/// and keeps track of the last time each op was broadcast.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloodingState {
    /// The operations to broadcast
    pub ops: HashMap<OpId, FloodingOp>,
}

impl FloodingState {
    /// Advance the state by one tick
    pub fn tick(&mut self) {
        self.ops.values_mut().for_each(|op| op.tick());
    }
}

impl GetRoutingOpInfo for FloodingState {
    fn routing_op_infos(&self) -> impl Iterator<Item = RoutingOpInfo> {
        self.ops.values().map(|op| RoutingOpInfo {
            id: op.id(),
            size: op.data.metadata.size,
        })
    }
}

impl FloodingState {
    /// Get the IDs of all operations currently being tracked
    pub fn op_ids(&self) -> BTreeSet<OpId> {
        self.ops.keys().copied().collect()
    }
}

/// Metadata needed for flooding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize, RenameAll)]
pub struct FloodingMetadata {
    /// The size of the operation
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
#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize, RenameAll,
)]
#[display("{self:?}")]
pub struct FloodingWireData {
    /// The ID of the operation
    pub id: OpId,
    /// The metadata needed to implement this protocol
    pub metadata: FloodingMetadata,
    /// The encrypted operation
    pub encrypted: EncryptedOp,
}

impl FloodingWireData {
    /// Create a new FloodingWireData from an operation
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
pub struct FloodingOpState {
    /// The number of ticks since the last time this op was sent
    pub since_last_send: Option<Ticks>,
}

impl FloodingOpState {
    /// Advance the state by one tick
    pub fn tick(&mut self) {
        self.since_last_send = self.since_last_send.map(|ticks| ticks + 1.into());
    }

    /// Reset the since_last_send counter
    pub fn touch(&mut self) {
        self.since_last_send = Some(0.into());
    }
}

/// The combination of a [`FloodingWireData`] and its [`FloodingOpState`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct FloodingOp {
    /// The data for this op
    pub data: FloodingWireData,
    /// The state of this op
    pub state: FloodingOpState,
}

impl FloodingOp {
    /// Create a new FloodingOp from FloodingWireData
    pub fn from_wire(data: FloodingWireData) -> Self {
        Self {
            data,
            state: FloodingOpState::default(),
        }
    }

    /// Get the ID of this op
    pub fn id(&self) -> OpId {
        self.data.id
    }

    /// Create a new FloodingOp from an operation
    #[allow(unused)]
    pub fn from_op(op: Operation) -> Self {
        Self::from_wire(FloodingWireData::new(op))
    }

    /// Advance the state by one tick
    pub fn tick(&mut self) {
        self.state.tick();
    }

    /// Reset the since_last_send counter
    pub fn touch(&mut self) {
        self.state.touch();
    }

    /// Get the state of this op
    pub fn state(&self) -> &FloodingOpState {
        &self.state
    }
}
