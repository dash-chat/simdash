use std::collections::BTreeSet;

use simdash_ids::EncryptedOp;

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
