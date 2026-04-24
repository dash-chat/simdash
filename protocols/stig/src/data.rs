use std::collections::{BTreeMap, BTreeSet};

use derive_more::{Constructor, Debug, Deref, From};

use serde::{Deserialize, Serialize};
use simdash_types::{EncryptedOp, Hops, LogHeight, LogId, Seq, TopicId};

use crate::SourceId;

/// The data sent over the wire between nodes using the flooding-sync protocol.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WireData {
    /// An announcement of a state vector for a topic
    Announce(Announce),

    /// A list of operations in response to an announcement
    Payload(Payload),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Announce {
    /// The mesh ID of the author of the announcement (immutable)
    pub author: SourceId,

    /// The mesh ID of the most recent relay of the announcement (changes at every hop)
    pub relay: SourceId,

    /// The number of hops that the announcement has traveled
    pub hops: Hops,

    /// The state vectors
    pub topics: BTreeMap<TopicId, StateVector>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Payload {
    pub ops: BTreeSet<Op>,
    pub addressees: BTreeSet<SourceId>,
}

impl Announce {
    pub fn new(author: SourceId, topics: BTreeMap<TopicId, StateVector>) -> Self {
        Self {
            author,
            relay: author,
            hops: 0.into(),
            topics,
        }
    }
}

/// Metadata needed for flooding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Metadata {
    pub topic_id: TopicId,
    pub log_id: LogId,
    pub seq: Seq,
}

/// Metadata needed for Stig.
impl Metadata {
    /// Get the metadata from an operation
    pub fn from_op(op: &simdash_types::Operation) -> Self {
        Self {
            topic_id: op.header.topic_id,
            log_id: op.header.author.into(),
            seq: op.header.seq,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[debug("Op({:?})", metadata)]
pub struct Op {
    pub metadata: Metadata,
    pub encrypted: EncryptedOp,
}

impl Op {
    pub fn from_op(op: &simdash_types::Operation) -> Self {
        Self {
            metadata: Metadata::from_op(op),
            encrypted: EncryptedOp::from(op.clone()),
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize, From, Deref,
)]
#[debug("StateVector({:?})", _0.iter().map(|(log_id, log_height)| format!("{}:{}", log_id, log_height)).collect::<Vec<String>>().join(", "))]
pub struct StateVector(BTreeMap<LogId, LogHeight>);

impl StateVector {
    pub fn new(entries: impl IntoIterator<Item = (LogId, LogHeight)>) -> Self {
        Self(entries.into_iter().collect())
    }
}
