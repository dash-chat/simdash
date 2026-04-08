#![allow(unused)]

use simdash_state::*;

mod dash;
mod node;
mod routing;

pub use dash::*;
pub use node::*;
pub use routing::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Tick,
    Node(NodeId, NodeAction),
}
