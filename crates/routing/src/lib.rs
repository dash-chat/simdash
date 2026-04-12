// mod fake_generated;
mod macros;

use simdash_ids::Ticks;
use simdash_routing_common::{ProtocolAction, ProtocolFx};

crate::routing_protocols! {
    flooding => {
        model:  simdash_routing_flooding::FloodingModel,
        state:  simdash_routing_flooding::FloodingState,
        action: simdash_routing_flooding::FloodingAction,
        fx:     simdash_routing_flooding::FloodingFx,
        data:   simdash_routing_flooding::FloodingWireData,
    },
}

pub type RoutingAction = ProtocolAction<AnyRoutingAction, AnyRoutingData>;
pub type RoutingFx = ProtocolFx<AnyRoutingFx, AnyRoutingData>;

impl RoutingModel {
    pub fn standard(broadcast_interval: Ticks) -> Self {
        Self {
            flooding: simdash_routing_flooding::FloodingModel { broadcast_interval },
        }
    }
}
