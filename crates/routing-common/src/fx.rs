use std::collections::HashSet;
use std::hash::Hash;

use simdash_types::EncryptedOp;

/// Effect type used by a routing protocol's model.
///
/// The types are generic so that this type can be used both by the protocol model itself,
/// using specific Fx and Data types, and also by the higher level network model,
/// which will use enums of all possible Fx and Data types for all known routing protocols.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolFx<Fx, D: Eq + Hash> {
    Network(SendFx<D>),
    Node(StorageFx),
    Protocol(Fx),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageFx {
    StoreOps { ops: Vec<EncryptedOp> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendFx<D: Eq + Hash> {
    UnicastData {
        to: simdash_types::NodeId,
        data: HashSet<D>,
    },
    BroadcastData {
        data: HashSet<D>,
    },
}
