//! A Scenario is a self-contained simulation based on a Model and a Behavior for that model.
//!
//! The scenario maintains state, uses the Behavior to produce Actions to perform state transitions,
//! and updates the state accordingly.

use polestar::Machine;
use simdash_types::Behavior;

/// A Scenario takes an initial model State and a Behavior,
/// and uses the Behavior to produce Actions to perform state transitions.
pub struct Scenario<M, B>
where
    M: Machine<Fx = (), Error = anyhow::Error>,
    B: Behavior<M>,
{
    model: M,
    state: Option<M::State>,
    history: Vec<M::Action>,
    behavior: B,
}

impl<M, B> Scenario<M, B>
where
    M: Machine<Fx = (), Error = anyhow::Error>,
    B: Behavior<M>,
{
    /// Create a new Scenario with the given Model, initial state, and Behavior.
    pub fn initial(model: M, initial_state: M::State, behavior: B) -> Self {
        Self {
            model,
            state: Some(initial_state),
            history: Vec::new(),
            behavior,
        }
    }

    /// Advance the Scenario by one step.
    pub fn advance(&mut self) -> anyhow::Result<bool> {
        if let Some(mut state) = self.state.take() {
            let actions = self.behavior.next(&state)?;
            let empty = actions.is_empty();
            for action in actions {
                state = self.model.transition_(state, action.clone())?;
                self.history.push(action);
            }
            self.state = Some(state);
            Ok(!empty)
        } else {
            // TODO: invalid actions would be due to a misguided behavior.
            //       ideally the behavior should be written to not produce invalid actions.
            //       in practice maybe we should be lenient and just let it continue.
            anyhow::bail!("The scenario has already finished due to invalid action");
        }
    }

    /// Get the current state of the Scenario.
    pub fn state(&self) -> Option<&M::State> {
        self.state.as_ref()
    }

    /// Get the Model of the Scenario.
    pub fn model(&self) -> &M {
        &self.model
    }

    /// Get the history of Actions performed by the Scenario so far.
    ///
    /// This can be used to replay the scenario or to analyze the behavior.
    pub fn history(&self) -> &Vec<M::Action> {
        &self.history
    }
}
