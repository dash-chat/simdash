//! A no-op routing protocol, which does nothing.

mod action;
mod behavior;
mod data;
mod fx;
mod model;
mod state;

pub use action::*;
pub use behavior::*;
pub use data::*;
pub use fx::*;
pub use model::*;
pub use state::*;

use simdash_routing_common::{RoutingAction, RoutingFx};

/// The action type for the Noop protocol (including network-level actions)
pub type ProtocolAction = RoutingAction<Action, WireData>;
/// The effect type for the Noop protocol (including network- and node-level effects)
pub type ProtocolFx = RoutingFx<Fx, WireData>;
