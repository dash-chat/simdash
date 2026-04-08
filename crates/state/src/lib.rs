#![allow(unused)]

use std::collections::BTreeMap;

mod dash;
mod node;
mod routing;

pub use dash::*;
pub use node::*;
pub use routing::*;

use simdash_ids::NodeId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    nodes: BTreeMap<NodeId, NodeState>,
    // links: ???,
}
