#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProtocolAction<A, D> {
    Protocol(A),
    Network(ProtocolNetworkAction<D>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProtocolNetworkAction<D> {
    Receive { from: simdash_ids::NodeId, data: D },
    Tick,
}
