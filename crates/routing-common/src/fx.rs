use simdash_ids::Operation;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoutingFx<Fx, D> {
    Network(SendFx<D>),
    Node(StorageFx),
    Protocol(Fx),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StorageFx {
    StoreOps { ops: Vec<Operation> },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SendFx<D> {
    UnicastData { to: simdash_ids::NodeId, data: D },
    BroadcastData { data: D },
}
