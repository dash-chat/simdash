use simdash_ids::NodeId;

use crate::node::NodeAction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NetworkAction<RA> {
    /// Tick every node
    Tick,
    /// Evolve the network topology
    EvolveTopology,
    Node(NodeId, NodeAction<RA>),
}
