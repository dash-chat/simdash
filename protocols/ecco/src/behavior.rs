use simdash_types::Behavior;

use crate::{Action, Model, ProtocolAction, ProtocolFx, State};

/// A behavior that never triggers any actions
#[derive(Default)]
pub struct StandardBehavior;

impl Behavior for StandardBehavior {
    type Model = Model;

    fn next_tick(&mut self, _: &State) -> anyhow::Result<Vec<ProtocolAction>> {
        Ok(vec![ProtocolAction::protocol(Action::TriggerSync)])
    }

    fn handle_fx(
        &mut self,
        _: &State,
        _: Vec<ProtocolFx>,
    ) -> anyhow::Result<Option<Vec<ProtocolFx>>> {
        Ok(None)
    }
}
