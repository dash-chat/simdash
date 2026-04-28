//! The routing crate is mainly about dispatching actions and effects to multiple user-defined routing protocols.
//! The key part of this crate is a macro that composes all user-defined routing protocols
//! into a single auto-generated `RoutingDispatchModel` and `RoutingDispatchState` structs,
//! and `AnyRoutingAction`, `AnyRoutingFx`, and `AnyRoutingData` enums.
//!
//! The design of this part of the system is to allow users to write routing protocols independently of the rest of the system,
//! which can live in their own isolated crates, and easily compose them into the `RoutingDispatchModel` used by the `NetworkModel`.
//! A protocol crate only needs to depend on `simdash-routing-common` and `simdash-types`, and can communicate with the rest of the system
//! through the [`RoutingDispatchAction`] and [`RoutingDispatchFx`] enums.
//!
//! The key difficulty that this design seeks to overcome is that the full specification of a routing protocol must be defined
//! at two opposite ends of the system:
//! - the RoutingDispatchModel is an independent model at the bottom of the state machine hierarchy that needs to know how to trigger network- and node-level Effects
//! - the protocol must ultimately be driven by network-level Behaviors at the top of the hierarchy

mod behavior;
mod macros;

pub use behavior::*;

use derive_more::Debug;
use polestar::Machine;
use simdash_routing_common::{ProtocolNetworkAction, RoutingAction, RoutingFx, SendFx};
use simdash_types::{GetRoutingOpInfo, NodeId, RoutingOpInfo, Ticks};

pub use simdash_protocol_flooding as flooding;
pub use simdash_protocol_flooding_sync as flooding_sync;
pub use simdash_protocol_noop as noop;
pub use simdash_protocol_stig as stig;

crate::routing_dispatch! {
    Flooding / flooding => {
        model:  flooding::FloodingModel,
        state:  flooding::FloodingState,
        action: flooding::FloodingAction,
        fx:     flooding::FloodingFx,
        data:   flooding::FloodingWireData,
    },
    FloodingSync / flooding_sync => {
        model:  flooding_sync::Model,
        state:  flooding_sync::State,
        action: flooding_sync::Action,
        fx:     flooding_sync::Fx,
        data:   flooding_sync::WireData,
    },
    Stig / stig => {
        model:  stig::Model,
        state:  stig::State,
        action: stig::Action,
        fx:     stig::Fx,
        data:   stig::WireData,
    },
    Noop / noop => {
        model:  noop::NoopModel,
        state:  noop::NoopState,
        action: noop::NoopAction,
        fx:     noop::NoopFx,
        data:   noop::NoopWireData,
    }
}

/// The variant of [`RoutingAction`] that can be used to dispatch actions to the [`RoutingDispatchModel`].
pub type RoutingDispatchAction = RoutingAction<AnyRoutingAction, AnyRoutingData>;
/// The variant of [`RoutingFx`] that can be used to dispatch effects to the [`RoutingDispatchModel`].
pub type RoutingDispatchFx = RoutingFx<AnyRoutingFx, AnyRoutingData>;

/// Lift a protocol-specific [`RoutingAction`] into a [`RoutingAction`].
/// This is used to convert a protocol-specific action into a [`RoutingAction`] that can be dispatched to the [RoutingDispatchModel].
pub fn lift_action<A, D: Eq + std::hash::Hash>(
    action: RoutingAction<A, D>,
    wrap_data: impl Fn(D) -> AnyRoutingData,
    wrap_action: impl Fn(A) -> AnyRoutingAction,
) -> RoutingDispatchAction {
    match action {
        RoutingAction::Network(ProtocolNetworkAction::Tick) => {
            RoutingDispatchAction::Network(ProtocolNetworkAction::Tick)
        }
        RoutingAction::Network(ProtocolNetworkAction::Receive { data }) => {
            RoutingDispatchAction::Network(ProtocolNetworkAction::Receive {
                data: wrap_data(data),
            })
        }
        RoutingAction::Protocol(a) => RoutingDispatchAction::Protocol(wrap_action(a)),
    }
}

/// Lift a protocol-specific `RoutingFx` into a `RoutingFx`.
/// This is used to convert a protocol-specific effect into a `RoutingFx`
/// that can be handled by models above the RoutingDispatchModel.
pub fn lift_fx<F, D: Eq + std::hash::Hash>(
    fx: RoutingFx<F, D>,
    wrap_data: impl Fn(D) -> AnyRoutingData,
    wrap_fx: impl Fn(F) -> AnyRoutingFx,
) -> RoutingDispatchFx {
    match fx {
        RoutingFx::Network(SendFx::Broadcast { data }) => {
            RoutingDispatchFx::Network(SendFx::Broadcast {
                data: wrap_data(data),
            })
        }
        RoutingFx::Network(SendFx::Unicast { to, data }) => {
            RoutingDispatchFx::Network(SendFx::Unicast {
                to,
                data: wrap_data(data),
            })
        }
        RoutingFx::Node(fx) => RoutingDispatchFx::Node(fx),
        RoutingFx::Protocol(fx) => RoutingDispatchFx::Protocol(wrap_fx(fx)),
    }
}

impl RoutingDispatchModel {
    /// Create a "standard" RoutingDispatchModel with default parameters.
    pub fn standard(node_id: NodeId, interval: Ticks) -> Self {
        Self {
            flooding: simdash_protocol_flooding::FloodingModel {
                broadcast_interval: interval,
            },
            flooding_sync: flooding_sync::Model::new(interval),
            noop: simdash_protocol_noop::NoopModel,
            stig: simdash_protocol_stig::Model::new(node_id.into(), interval),
        }
    }
}
