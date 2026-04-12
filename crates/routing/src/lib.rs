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

impl RoutingModel {
    pub fn standard(broadcast_interval: Ticks) -> Self {
        Self {
            flooding: simdash_routing_flooding::FloodingModel { broadcast_interval },
            noop: simdash_routing_noop::NoopModel,
        }
    }
}
