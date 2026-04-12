use std::{collections::HashSet, hash::Hash};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolAction<A, D: Eq + Hash> {
    Protocol(A),
    Network(ProtocolNetworkAction<D>),
}

impl<A, D: Eq + Hash> From<A> for ProtocolAction<A, D> {
    fn from(action: A) -> Self {
        ProtocolAction::Protocol(action)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolNetworkAction<D: Eq + Hash> {
    Receive {
        from: simdash_ids::NodeId,
        data: HashSet<D>,
    },
    Tick,
}
