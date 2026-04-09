mod naive_flooding;

use polestar::Machine;
use simdash_action::network::NetworkAction;
use simdash_behavior::Behavior;
use simdash_model::network::NetworkModel;
use simdash_state::network::NetworkState;

use self::naive_flooding::NaiveFloodingBehaviorType;

/// Wraps the concrete scenario types so we can select them at runtime.
pub struct AnyScenario {
    model: NetworkModel,
    state: NetworkState,
    behavior: NaiveFloodingBehaviorType,
}

impl AnyScenario {
    pub fn from_config(
        _behavior: &str,
        initial_state: NetworkState,
        broadcast_interval: u32,
    ) -> anyhow::Result<Self> {
        let (model, behavior, state) =
            naive_flooding::naive_flooding(initial_state, broadcast_interval);
        Ok(Self {
            model,
            state,
            behavior,
        })
    }

    /// Advance one "round" in two phases:
    /// 1. Collect all node actions from the behavior (without applying)
    /// 2. Apply them all, then apply the Tick
    ///
    /// This ensures ops received mid-round aren't re-broadcast in the same round.
    pub fn step(&mut self) -> anyhow::Result<()> {
        // Phase 1: collect all node actions for this round
        let mut actions = Vec::new();
        loop {
            let action = self.behavior.next(&self.state)?;
            match action {
                Some(NetworkAction::Tick) => break,
                Some(a) => actions.push(a),
                None => break,
            }
        }

        // Phase 2: apply all node actions, then tick
        for action in actions {
            self.state = self.model.transition_(self.state.clone(), action)?;
        }
        self.state = self
            .model
            .transition_(self.state.clone(), NetworkAction::Tick)?;

        Ok(())
    }

    pub fn state(&self) -> &NetworkState {
        &self.state
    }
}
