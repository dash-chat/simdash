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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, From, Deref)]
pub struct Size(u64);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, From, Deref)]
pub struct Time(u64);
