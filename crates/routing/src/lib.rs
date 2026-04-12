// mod fake_generated;
mod macros;

use simdash_ids::Ticks;
use simdash_routing_common::{ProtocolAction, ProtocolFx};

pub use simdash_routing_flooding as flooding;
pub use simdash_routing_noop as noop;

crate::routing_protocols! {
    Flooding / flooding => {
        model:  flooding::FloodingModel,
        state:  flooding::FloodingState,
        action: flooding::FloodingAction,
        fx:     flooding::FloodingFx,
        data:   flooding::FloodingWireData,
    },
    Noop / noop => {
        model:  noop::NoopModel,
        state:  noop::NoopState,
        action: noop::NoopAction,
        fx:     noop::NoopFx,
        data:   noop::WireData,
    }
}

pub type RoutingAction = ProtocolAction<AnyRoutingAction, AnyRoutingData>;
pub type RoutingFx = ProtocolFx<AnyRoutingFx, AnyRoutingData>;

/// Lift a protocol-specific `ProtocolAction` into a `RoutingAction`.
pub fn lift_action<A, D: Eq + std::hash::Hash>(
    action: ProtocolAction<A, D>,
    wrap_data: impl Fn(D) -> AnyRoutingData,
    wrap_action: impl Fn(A) -> AnyRoutingAction,
) -> RoutingAction {
    match action {
        ProtocolAction::Network(simdash_routing_common::ProtocolNetworkAction::Tick) => {
            RoutingAction::Network(simdash_routing_common::ProtocolNetworkAction::Tick)
        }
        ProtocolAction::Network(simdash_routing_common::ProtocolNetworkAction::Receive {
            from,
            data,
        }) => RoutingAction::Network(simdash_routing_common::ProtocolNetworkAction::Receive {
            from,
            data: data.into_iter().map(wrap_data).collect(),
        }),
        ProtocolAction::Protocol(a) => RoutingAction::Protocol(wrap_action(a)),
    }
}

/// Lift a protocol-specific `ProtocolFx` into a `RoutingFx`.
pub fn lift_fx<F, D: Eq + std::hash::Hash>(
    fx: ProtocolFx<F, D>,
    wrap_data: impl Fn(D) -> AnyRoutingData,
    wrap_fx: impl Fn(F) -> AnyRoutingFx,
) -> RoutingFx {
    match fx {
        ProtocolFx::Network(simdash_routing_common::SendFx::BroadcastData { data }) => {
            RoutingFx::Network(simdash_routing_common::SendFx::BroadcastData {
                data: data.into_iter().map(wrap_data).collect(),
            })
        }
        ProtocolFx::Network(simdash_routing_common::SendFx::UnicastData { to, data }) => {
            RoutingFx::Network(simdash_routing_common::SendFx::UnicastData {
                to,
                data: data.into_iter().map(wrap_data).collect(),
            })
        }
        ProtocolFx::Node(fx) => RoutingFx::Node(fx),
        ProtocolFx::Protocol(fx) => RoutingFx::Protocol(wrap_fx(fx)),
    }
}

impl RoutingModel {
    pub fn standard(broadcast_interval: Ticks) -> Self {
        Self {
            flooding: simdash_routing_flooding::FloodingModel { broadcast_interval },
            noop: simdash_routing_noop::NoopModel,
        }
    }
}
