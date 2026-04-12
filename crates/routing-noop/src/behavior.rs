use simdash_ids::Behavior;
use simdash_routing_common::ProtocolAction;

use crate::{NoopAction, NoopModel, NoopState, WireData};

pub struct NoopBehavior;

impl Behavior<NoopModel> for NoopBehavior {
    fn next(&mut self, _: &NoopState) -> anyhow::Result<Vec<ProtocolAction<NoopAction, WireData>>> {
        Ok(vec![])
    }
}
