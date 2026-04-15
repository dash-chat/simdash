mod behavior;
mod operation;
pub use behavior::*;
pub use operation::*;

pub mod util;

use derive_more::{Add, Deref, Display, From};
use serde::{Deserialize, Serialize};

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
)]
#[display("L{}", _0)]
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
)]
pub struct LogHeight(u32);

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
    derive_more::Sub,
)]
pub struct Ticks(u32);

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

pub trait ExtractMetadata<T> {
    fn extract_metadata(&self) -> T;
}

pub trait GetRoutingOpInfo {
    fn routing_op_infos(&self) -> impl Iterator<Item = RoutingOpInfo>;
}
