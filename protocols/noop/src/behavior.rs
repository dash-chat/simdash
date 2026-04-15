use simdash_types::Behavior;

use crate::{NoopModel, NoopState, ProtocolAction};

/// A behavior that never triggers any actions
pub struct NoopBehavior;

impl Behavior<NoopModel> for NoopBehavior {
    fn next(&mut self, _: &NoopState) -> anyhow::Result<Vec<ProtocolAction>> {
        Ok(vec![])
    }
}
