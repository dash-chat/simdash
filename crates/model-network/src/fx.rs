use named_id::RenameAll;
use simdash_model_node::NodeFx;
use simdash_types::NodeId;

/// Effects for the network model.
#[derive(Debug, Clone, PartialEq, Eq, RenameAll)]
pub enum NetworkFx {
    /// Effects from a particular node
    Node(NodeId, NodeFx),
}
