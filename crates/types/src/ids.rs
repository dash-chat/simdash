use named_id::RenameAll;

use super::*;

pub type GroupId = u64;

#[derive(
    Serialize,
    Deserialize,
    PartialOrd,
    Ord,
    Debug,
    Display,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    From,
    Deref,
)]
#[display("N({})", _0)]
pub struct NodeId(u32);

#[derive(
    Serialize,
    Deserialize,
    PartialOrd,
    Ord,
    Debug,
    Display,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    From,
    Deref,
)]
#[display("op{}", _0)]
pub struct OpId(u32);

#[derive(
    Serialize, Deserialize, PartialOrd, Ord, Debug, Clone, Copy, PartialEq, Eq, Hash, From, Deref,
)]
pub struct EdgeId(u32);

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Display,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    From,
    Deref,
)]
#[display("({})B", _0)]
pub struct Size(u64);

#[derive(
    Debug,
    Display,
    Clone,
    Copy,
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
#[display("T{}", _0)]
pub struct TopicId(u32);

#[derive(
    Debug,
    Display,
    Clone,
    Copy,
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
#[display("L()")]
pub struct LogId(u32);

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
    From,
    Deref,
    Add,
    Display,
    RenameAll,
)]
pub struct LogHeight(u32);

impl LogHeight {
    pub fn from_seq(seq: Seq) -> LogHeight {
        LogHeight::from(seq.0 + 1)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize, From, Deref,
)]
pub struct Seq(u32);

/// A number of ticks (a duration).
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Default,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    From,
    Deref,
    derive_more::Add,
    derive_more::AddAssign,
    derive_more::Sub,
    derive_more::SubAssign,
)]
pub struct Ticks(u32);

/// A number of hops that a message has traveled.
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Default,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    From,
    Deref,
    derive_more::Add,
    derive_more::AddAssign,
)]
pub struct Hops(u32);

impl From<NodeId> for LogId {
    fn from(id: NodeId) -> LogId {
        LogId::from(id.0)
    }
}

impl From<LogId> for NodeId {
    fn from(id: LogId) -> NodeId {
        NodeId::from(id.0)
    }
}

impl From<NodeId> for petgraph::graph::NodeIndex {
    fn from(id: NodeId) -> petgraph::graph::NodeIndex {
        petgraph::graph::NodeIndex::new(*id as usize)
    }
}

impl From<petgraph::graph::NodeIndex> for NodeId {
    fn from(index: petgraph::graph::NodeIndex) -> NodeId {
        NodeId::from(index.index() as u32)
    }
}
