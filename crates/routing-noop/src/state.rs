use serde::{Deserialize, Serialize};
use simdash_types::*;

/// The Noop protocol's state simply tracks the number of ticks since the last time it was triggered.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoopState {
    /// The number of ticks that have elapsed
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

/// Metadata needed for Noop.
impl NoopMetadata {
    /// Get the metadata from an operation
    pub fn from_op(_op: &Operation) -> Self {
        Self
    }
}

/// The data sent over the wire between nodes using the flooding protocol.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct NoopWireData;
