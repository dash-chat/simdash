use simdash_types::Behavior;

use crate::{Model, ProtocolAction, State};

/// A behavior that never triggers any actions
pub struct StandardBehavior;

impl Behavior<Model> for StandardBehavior {
    fn next(&mut self, _: &State) -> anyhow::Result<Vec<ProtocolAction>> {
        Ok(vec![])
    }
}
