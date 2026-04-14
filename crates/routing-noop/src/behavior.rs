use simdash_routing_common::ProtocolAction;
use simdash_types::Behavior;

use crate::{NoopAction, NoopModel, NoopState, WireData};

/// A behavior that never triggers any actions
pub struct NoopBehavior;

impl Behavior<NoopModel> for NoopBehavior {
    fn next(&mut self, _: &NoopState) -> anyhow::Result<Vec<ProtocolAction<NoopAction, WireData>>> {
        Ok(vec![])
    }
}
