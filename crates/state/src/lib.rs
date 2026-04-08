#![allow(unused)]

use std::collections::HashMap;

mod node;

pub use node::*;

pub struct State {
    nodes: HashMap<NodeId, NodeState>,
    // links: ???,
}
