mod fake_generated;
mod macros;

pub use fake_generated::*;

use simdash_routing_common::{ProtocolAction, ProtocolFx};

pub type RoutingAction = ProtocolAction<RoutingActionDispatch, RoutingDataDispatch>;
pub type RoutingFx = ProtocolFx<RoutingFxDispatch, RoutingDataDispatch>;

impl RoutingModel {
    pub fn standard() -> Self {
        Self {
            flooding: simdash_routing_flooding::FloodingModel {
                broadcast_interval: 3.into(),
            },
        }
    }
}
