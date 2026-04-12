use serde::{Deserialize, Serialize};
use simdash_ids::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoopState {
    pub ticks: Ticks,
}

impl Tickable for NoopState {
    fn tick(&mut self) {
        self.ticks = self.ticks + 1.into();
    }
}

/// Metadata needed for flooding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct NoopMetadata;

impl NoopMetadata {
    pub fn from_op(_op: &Operation) -> Self {
        Self
    }
}

/// The data sent over the wire between nodes using the flooding protocol.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct WireData;
