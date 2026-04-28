//! Use log height syncing to flood the network with operations

#![allow(missing_docs)]

mod action;
mod behavior;
mod data;
mod fx;
mod model;
mod state;
pub mod util;

pub use action::*;
pub use behavior::*;
pub use data::*;
pub use fx::*;
pub use model::*;
pub use state::*;

use simdash_routing_common::{RoutingAction, RoutingFx};

/// Change to Route to let nodes track full routes
pub type HopsImpl = Route;

/// The action type for the Stig protocol (including network-level actions)
pub type ProtocolAction = RoutingAction<Action, WireData<HopsImpl>>;
/// The effect type for the Stig protocol (including network- and node-level effects)
pub type ProtocolFx = RoutingFx<Fx, WireData<HopsImpl>>;
