mod fake_generated;
mod macros;

pub use fake_generated::*;

use simdash_ids::Ticks;
use simdash_routing_common::{ProtocolAction, ProtocolFx};

pub type RoutingAction = ProtocolAction<AnyRoutingAction, AnyRoutingData>;
pub type RoutingFx = ProtocolFx<AnyRoutingFx, AnyRoutingData>;

impl RoutingModel {
    pub fn standard(broadcast_interval: Ticks) -> Self {
        Self {
            flooding: simdash_routing_flooding::FloodingModel { broadcast_interval },
        }
    }
}
