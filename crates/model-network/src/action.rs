use named_id::RenameAll;
use simdash_model_node::NodeAction;
use simdash_types::NodeId;

/// Actions for the network model.
#[derive(Debug, Clone, PartialEq, Eq, RenameAll)]
pub enum NetworkAction {
    /// Tick every node
    Tick,
    /// Evolve the network topology
    EvolveTopology,
    /// Dispatch a NodeAction to a particular node
    Node(NodeId, NodeAction),
}

impl NetworkAction {
    /// Convenience method to create a Node action.
    pub fn node(node_id: NodeId, action: NodeAction) -> Self {
        NetworkAction::Node(node_id, action)
    }
}
