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
pub struct NodeId(pub u32);

impl named_id::Nameable for NodeId {
    fn shortener(&self) -> Option<named_id::Shortener> {
        Some(named_id::Shortener {
            length: 3,
            prefix: "N",
        })
    }
}

impl std::iter::Step for NodeId {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        if start.0 > end.0 {
            return (0, None);
        } else {
            (
                end.0 as usize - start.0 as usize,
                Some(end.0 as usize - start.0 as usize),
            )
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Some(Self(start.0.checked_add(count as u32)?))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Some(Self(start.0.checked_sub(count as u32)?))
    }
}

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

impl named_id::Nameable for OpId {
    fn shortener(&self) -> Option<named_id::Shortener> {
        Some(named_id::Shortener {
            length: 3,
            prefix: "op",
        })
    }
}
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
)]
pub struct TopicId(pub u32);

impl named_id::Nameable for TopicId {
    fn shortener(&self) -> Option<named_id::Shortener> {
        Some(named_id::Shortener {
            length: 3,
            prefix: "T",
        })
    }
}
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
)]
pub struct LogId(pub u32);

impl named_id::Nameable for LogId {
    fn shortener(&self) -> Option<named_id::Shortener> {
        Some(named_id::Shortener {
            length: 3,
            prefix: "L",
        })
    }
}
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
)]
#[display("{}", _0)]
pub struct LogHeight(u32);

impl named_id::Nameable for LogHeight {
    fn shortener(&self) -> Option<named_id::Shortener> {
        Some(named_id::Shortener {
            length: 3,
            prefix: "H",
        })
    }
}

impl LogHeight {
    pub fn from_seq(seq: Seq) -> LogHeight {
        LogHeight::from(seq.0 + 1)
    }
}

#[derive(
    Debug,
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
    RenameNone,
)]
pub struct Seq(pub u32);

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
    RenameNone,
)]
pub struct Ticks(pub u32);

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
    Display,
)]
pub struct Hops(pub u32);

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
