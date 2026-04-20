use simdash_model_node::NodeFx;
use simdash_types::NodeId;

/// Effects for the network model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkFx {
    /// Effects from a particular node
    Node(NodeId, NodeFx),
}
