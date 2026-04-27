//! A Scenario is a self-contained simulation based on a Model and a Behavior for that model.
//!
//! The scenario maintains state, uses the Behavior to produce Actions to perform state transitions,
//! and updates the state accordingly.

use polestar::{
    Machine,
    mapping::{ActionOf, FxOf, StateOf},
};
use simdash_types::Behavior;

/// A Scenario takes an initial model State and a Behavior,
/// and uses the Behavior to produce Actions to perform state transitions.
pub struct Scenario<B>
where
    B: Behavior,
{
    model: B::Model,
    state: Option<StateOf<B::Model>>,
    history: Vec<ActionOf<B::Model>>,
    behavior: B,
}

impl<B> Scenario<B>
where
    B: Behavior,
    FxOf<B::Model>: std::fmt::Debug,
{
    /// Create a new Scenario with the given Model, initial state, and Behavior.
    pub fn initial(model: B::Model, initial_state: StateOf<B::Model>, behavior: B) -> Self {
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
            let actions = self.behavior.next_tick(&state)?;
            let empty = actions.is_empty();
            for action in actions {
                let (new_state, fx) = self.model.transition(state, action.clone())?;
                state = new_state;
                let unhandled = self.behavior.handle_fx(&state, fx)?;
                if let Some(fx) = unhandled {
                    tracing::warn!("Scenario received unhandled effects: {:?}", fx);
                }
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
    pub fn state(&self) -> &StateOf<B::Model> {
        self.state
            .as_ref()
            .expect("scenario encountered an error in a previous step and has no state")
    }

    /// Get the Model of the Scenario.
    pub fn model(&self) -> &B::Model {
        &self.model
    }

    /// Get the history of Actions performed by the Scenario so far.
    ///
    /// This can be used to replay the scenario or to analyze the behavior.
    pub fn history(&self) -> &Vec<ActionOf<B::Model>> {
        &self.history
    }
}
