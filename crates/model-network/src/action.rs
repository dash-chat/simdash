use simdash_model_node::NodeAction;
use simdash_types::NodeId;

/// Actions for the network model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkAction {
    /// Tick every node
    Tick,
    /// Evolve the network topology
    EvolveTopology,
    /// Dispatch a NodeAction to a particular node
    Node(NodeId, NodeAction),
}
