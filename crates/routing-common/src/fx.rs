use std::collections::BTreeSet;

use simdash_ids::EncryptedOp;

/// Effect type used by a routing protocol's model.
///
/// The types are generic so that this type can be used both by the protocol model itself,
/// using specific Fx and Data types, and also by the higher level network model,
/// which will use enums of all possible Fx and Data types for all known routing protocols.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProtocolFx<Fx, D> {
    Network(SendFx<D>),
    Node(StorageFx),
    Protocol(Fx),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StorageFx {
    StoreOps { ops: Vec<EncryptedOp> },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SendFx<D> {
    UnicastData {
        to: simdash_ids::NodeId,
        data: BTreeSet<D>,
    },
    BroadcastData {
        data: BTreeSet<D>,
    },
}
