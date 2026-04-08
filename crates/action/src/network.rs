use simdash_ids::NodeId;

use crate::node::NodeAction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NetworkAction {
    Tick,
    Node(NodeId, NodeAction),
}
