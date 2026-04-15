use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use simdash_types::{EncryptedOp, LogHeight, LogId, Seq};

/// The data sent over the wire between nodes using the flooding-sync protocol.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WireData {
    Announce {
        state_vector: StateVector,
    },
    Ops {
        ops: BTreeSet<(LogId, Seq, EncryptedOp)>,
    },
}

/// Metadata needed for flooding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Metadata {
    log_id: LogId,
    seq: Seq,
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

pub struct Op {
    metadata: Metadata,
    encrypted: EncryptedOp,
}

pub type StateVector = BTreeSet<(LogId, LogHeight)>;
