#![allow(unused)]

use simdash_ids::NodeId;
use simdash_state::*;

pub mod dash;
pub mod node;
pub mod routing;

use crate::node::NodeAction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Tick,
    Node(NodeId, NodeAction),
}
