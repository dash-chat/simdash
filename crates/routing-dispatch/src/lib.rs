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

mod macros;

use polestar::Machine;
use simdash_routing_common::{ProtocolNetworkAction, RoutingAction, RoutingFx, SendFx};
use simdash_types::{Tickable, Ticks};

pub use simdash_routing_flooding as flooding;
pub use simdash_routing_noop as noop;

crate::routing_dispatch! {
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
        RoutingAction::Network(ProtocolNetworkAction::Receive { from, data }) => {
            RoutingDispatchAction::Network(ProtocolNetworkAction::Receive {
                from,
                data: data.into_iter().map(wrap_data).collect(),
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
        RoutingFx::Network(SendFx::BroadcastData { data }) => {
            RoutingDispatchFx::Network(SendFx::BroadcastData {
                data: data.into_iter().map(wrap_data).collect(),
            })
        }
        RoutingFx::Network(SendFx::UnicastData { to, data }) => {
            RoutingDispatchFx::Network(SendFx::UnicastData {
                to,
                data: data.into_iter().map(wrap_data).collect(),
            })
        }
        RoutingFx::Node(fx) => RoutingDispatchFx::Node(fx),
        RoutingFx::Protocol(fx) => RoutingDispatchFx::Protocol(wrap_fx(fx)),
    }
}

impl RoutingDispatchModel {
    /// Create a "standard" RoutingDispatchModel with default parameters.
    pub fn standard(broadcast_interval: Ticks) -> Self {
        Self {
            flooding: simdash_routing_flooding::FloodingModel { broadcast_interval },
            noop: simdash_routing_noop::NoopModel,
        }
    }
}
