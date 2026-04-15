//! A naive sample flooding routing protocol.
//!
//! See [`model`] for a description of the protocol's model.

mod action;
mod behavior;
mod fx;
mod model;
mod state;

pub use action::*;
pub use behavior::*;
pub use fx::*;
pub use model::*;
pub use state::*;

use simdash_routing_common::{RoutingAction, RoutingFx};

/// The action type for the Flooding protocol (including network-level actions)
pub type ProtocolAction = RoutingAction<FloodingAction, FloodingWireData>;
/// The effect type for the Flooding protocol (including network- and node-level effects)
pub type ProtocolFx = RoutingFx<FloodingFx, FloodingWireData>;
