use derive_more::{Display, From};
use named_id::{Nameable, Rename, RenameAll, RenameNone};
use serde::{Deserialize, Serialize};
use simdash_types::Hops;

use crate::SourceId;

pub trait HopTracker:
    Clone
    + Default
    + Eq
    + std::hash::Hash
    + std::fmt::Debug
    + std::fmt::Display
    + Serialize
    + serde::de::DeserializeOwned
    + Send
    + Sync
    + Sized
    + Rename
    + 'static
{
    fn relay(self, node_id: SourceId) -> Self;

    fn hops(&self) -> Hops;
}

#[derive(
    Debug,
    Display,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Hash,
    Serialize,
    Deserialize,
    RenameNone,
    From,
)]
pub struct HopCount(Hops);

impl HopTracker for HopCount {
    fn relay(self, _: SourceId) -> Self {
        Self(self.0 + 1.into())
    }

    fn hops(&self) -> Hops {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, RenameAll)]
pub struct Route {
    pub path: Vec<SourceId>,
}

impl HopTracker for Route {
    fn relay(mut self, node_id: SourceId) -> Self {
        self.path.push(node_id);
        self
    }

    fn hops(&self) -> Hops {
        Hops::from(self.path.len() as u32)
    }
}

impl Default for Route {
    fn default() -> Self {
        Self { path: Vec::new() }
    }
}

impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self
            .path
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join("->");
        write!(f, "({})", path)
    }
}

impl PartialOrd for Route {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.hops().cmp(&other.hops()))
    }
}

impl Ord for Route {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hops().cmp(&other.hops())
    }
}
