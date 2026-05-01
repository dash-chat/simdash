use simdash_types::Behavior;

use crate::{NoopModel, NoopState, ProtocolAction, ProtocolFx};

/// A behavior that never triggers any actions
pub struct NoopBehavior;

impl Behavior for NoopBehavior {
    type Model = NoopModel;

    fn next_tick(&mut self, _: &NoopState) -> anyhow::Result<Vec<ProtocolAction>> {
        Ok(vec![])
    }

    fn handle_fx(
        &mut self,
        _: &NoopState,
        _: Vec<ProtocolFx>,
    ) -> anyhow::Result<Option<Vec<ProtocolFx>>> {
        Ok(None)
    }
}
