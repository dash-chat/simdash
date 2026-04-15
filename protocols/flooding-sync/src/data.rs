use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use simdash_types::{EncryptedOp, LogHeight, LogId, Seq};

/// The data sent over the wire between nodes using the flooding-sync protocol.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WireData {
    /// An announcement of the state vector
    Announce {
        /// The state vector
        state_vector: StateVector,
    },
    /// A list of operations in response to an announcement
    Ops {
        /// The operations
        ops: BTreeSet<Op>,
    },
}

/// Metadata needed for flooding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Metadata {
    pub log_id: LogId,
    pub seq: Seq,
}

/// Metadata needed for Noop.
impl Metadata {
    /// Get the metadata from an operation
    pub fn from_op(op: &simdash_types::Operation) -> Self {
        Self {
            log_id: op.header.log_id,
            seq: op.header.seq,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Op {
    pub metadata: Metadata,
    pub encrypted: EncryptedOp,
}

pub type StateVector = BTreeSet<(LogId, LogHeight)>;
