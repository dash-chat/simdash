use std::{collections::HashSet, hash::Hash};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolAction<A, D: Eq + Hash> {
    Protocol(A),
    Network(ProtocolNetworkAction<D>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolNetworkAction<D: Eq + Hash> {
    Receive {
        from: simdash_ids::NodeId,
        data: HashSet<D>,
    },
    Tick,
}
