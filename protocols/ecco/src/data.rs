mod hops;
pub use hops::{HopCount, HopTracker, Route};

use std::collections::{BTreeMap, BTreeSet};

use derive_more::{Debug, Deref, Display, From};

use named_id::{Nameable, Rename, RenameAll};
use serde::{Deserialize, Serialize};
use simdash_types::{EncryptedOp, LogHeight, LogId, Seq, TopicId};

use crate::{HopsImpl, SourceId};

/// The data sent over the wire between nodes using the flooding-sync protocol.
#[derive(
    Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, RenameAll,
)]
#[serde(bound(deserialize = "H: HopTracker"))]
pub enum WireData<H: HopTracker = HopsImpl> {
    /// An announcement of a state vector for a topic
    #[debug("{:?}", _0)]
    Announce(Announce<H>),

    /// A list of operations in response to an announcement
    Payload(Payload),
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, RenameAll,
)]
#[display(
    "Announce({author} --{hops}-> {relay} : {topics})",
    author=author.with_short().renamed(),
    relay=relay.with_short().renamed(),
    topics=topics.clone().renamed(),
    hops=hops,
)]
#[debug("{self}")]
#[serde(bound(deserialize = "H: HopTracker"))]
pub struct Announce<H: HopTracker = HopsImpl> {
    /// The mesh ID of the author of the announcement (immutable)
    pub author: SourceId,

    /// The mesh ID of the most recent relay of the announcement (changes at every hop)
    pub relay: SourceId,

    /// The number of hops that the announcement has traveled
    pub hops: H,

    /// The state vectors
    pub topics: BTreeMap<TopicId, StateVector>,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, RenameAll,
)]
#[display("Payload({ops}, {addressees})", ops=ops.clone().renamed(), addressees=addressees.clone().renamed())]
pub struct Payload {
    pub ops: BTreeSet<Op>,
    pub addressees: BTreeSet<SourceId>,
}

impl<H: HopTracker> Announce<H> {
    pub fn new(author: SourceId, topics: BTreeMap<TopicId, StateVector>) -> Self {
        Self {
            author,
            relay: author,
            hops: H::default(),
            topics,
        }
    }
}

/// Metadata needed for flooding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize, RenameAll)]
pub struct Metadata {
    pub topic_id: TopicId,
    pub log_id: LogId,
    pub seq: Seq,
}

/// Metadata needed for Ecco.
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

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[debug("Op({:?})", metadata)]
#[display("{:?}", self)]
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

impl named_id::Nameable for Op {
    fn shortener(&self) -> Option<named_id::Shortener> {
        Some(named_id::Shortener {
            length: 3,
            prefix: "op",
        })
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
    From,
    Deref,
    RenameAll,
)]
#[debug("SV({})", _0.iter().map(|(log_id, log_height)| format!("{}:{}", log_id.renamed(), log_height)).collect::<Vec<String>>().join(", "))]
pub struct StateVector(BTreeMap<LogId, LogHeight>);

impl StateVector {
    pub fn new(entries: impl IntoIterator<Item = (LogId, LogHeight)>) -> Self {
        Self(entries.into_iter().collect())
    }

    /// Returns a new StateVector with the max log height for each log across both vectors.
    pub fn merge(&self, other: &StateVector) -> StateVector {
        let mut result = self.0.clone();
        for (log_id, height) in other.iter() {
            result
                .entry(*log_id)
                .and_modify(|h| *h = (*h).max(*height))
                .or_insert(*height);
        }
        StateVector(result)
    }
}
