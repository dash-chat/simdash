use std::sync::Arc;

use derive_more::{Debug, Eq, PartialEq};
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

    /// Does nothing, just for debugging
    #[debug("{:?}", **_0)]
    #[named_id(skip)]
    #[partial_eq(ignore)]
    Trace(Arc<dyn std::fmt::Debug + Send + Sync>),
}

impl NetworkAction {
    /// Convenience method to create a Node action.
    pub fn node(node_id: NodeId, action: NodeAction) -> Self {
        NetworkAction::Node(node_id, action)
    }

    /// Add a trace message to the action stream.
    pub fn trace(message: impl std::fmt::Debug + Send + Sync + 'static) -> Self {
        NetworkAction::Trace(Arc::new(message))
    }
}
