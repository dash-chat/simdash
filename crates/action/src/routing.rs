use derive_more::From;
use polestar::Machine;
use simdash_ids::NodeId;
use simdash_state::network::NetworkState;

use crate::network::NetworkAction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoutingAction<A, D> {
    Receive { from: simdash_ids::NodeId, data: D },
    Protocol(A),
    Tick,
}

impl<A, D> From<A> for RoutingAction<A, D> {
    fn from(action: A) -> Self {
        RoutingAction::Protocol(action)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoutingFx<Fx, D> {
    Network(NetworkRoutingFx<D>),
    Node(NodeRoutingFx),
    Protocol(Fx),
    Nothing,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeRoutingFx {
    StoreOps {
        ops: Vec<simdash_state::chat::Operation>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NetworkRoutingFx<D> {
    UnicastData { to: simdash_ids::NodeId, data: D },
    BroadcastData { data: D },
}

pub trait RoutingDataExt: Machine<Error = anyhow::Error> {
    type Data: polestar::machine::Cog;

    fn fx_to_network_actions(
        fx: Self::Fx,
        from: NodeId,
        network_state: &NetworkState<Self::State>,
    ) -> Vec<NetworkAction<Self::Action>>;

    fn produce_tick() -> Self::Action;
}

impl RoutingDataExt for polestar::machine::EmptyMachine {
    type Data = ();
    fn fx_to_network_actions(
        fx: Self::Fx,
        _: NodeId,
        _: &NetworkState<Self::State>,
    ) -> Vec<NetworkAction<Self::Action>> {
        vec![]
    }

    fn produce_tick() -> Self::Action {
        ()
    }
}
