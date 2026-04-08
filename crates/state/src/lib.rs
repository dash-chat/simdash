#![allow(unused)]

use std::collections::BTreeMap;

pub mod dash;
pub mod graph;
pub mod node;
pub mod routing;

pub use graph::*;

use simdash_ids::NodeId;

use crate::node::NodeState;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    nodes: BTreeMap<NodeId, NodeState>,
    // links: ???,
}

pub trait ExtractMetadata<T> {
    fn extract_metadata(&self) -> T;
}
