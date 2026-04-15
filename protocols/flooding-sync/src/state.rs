use serde::{Deserialize, Serialize};
use simdash_types::*;

/// The Noop protocol's state simply tracks the number of ticks since the last time it was triggered.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// The number of ticks until the next sync round may begin
    pub countdown: Ticks,
}

impl Tickable for State {
    fn tick(&mut self) {
        self.countdown = self.countdown + 1.into();
    }
}

/// Metadata needed for flooding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Metadata;

/// Metadata needed for Noop.
impl Metadata {
    /// Get the metadata from an operation
    pub fn from_op(_op: &Operation) -> Self {
        Self
    }
}
