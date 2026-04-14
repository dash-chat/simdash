use simdash_types::NodeId;

use crate::node::NodeAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkAction {
    /// Tick every node
    Tick,
    /// Evolve the network topology
    EvolveTopology,
    /// Dispatch a NodeAction to a particular node
    Node(NodeId, NodeAction),
}
