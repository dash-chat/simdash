mod fake_generated;
mod macros;

pub use fake_generated::*;

use simdash_routing_common::{ProtocolAction, ProtocolFx};

pub type RoutingAction = ProtocolAction<RoutingActionDispatch, RoutingDataDispatch>;
pub type RoutingFx = ProtocolFx<RoutingFxDispatch, RoutingDataDispatch>;
