use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, PartialOrd, Ord, Debug, Clone, Copy, PartialEq, Eq, Hash, From, Deref,
)]
pub struct NodeId(u32);

#[derive(
    Serialize, Deserialize, PartialOrd, Ord, Debug, Clone, Copy, PartialEq, Eq, Hash, From, Deref,
)]
pub struct OpId(u32);

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, From, Deref,
)]
pub struct Size(u64);

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
