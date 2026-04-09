use polestar::Machine;
use simdash_behavior::Behavior;

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
    pub fn initial(model: M, initial_state: M::State, behavior: B) -> Self {
        Self {
            model,
            state: Some(initial_state),
            history: Vec::new(),
            behavior,
        }
    }

    pub fn advance(&mut self) -> anyhow::Result<Option<M::Action>> {
        if let Some(state) = self.state.take() {
            let action = self.behavior.next(&state)?;
            if let Some(action) = action.clone() {
                self.state = Some(self.model.transition_(state, action.clone())?);
                self.history.push(action);
            } else {
                self.state = Some(state);
            }
            return Ok(action);
        } else {
            // TODO: invalid actions would be due to a misguided behavior.
            //       ideally the behavior should be written to not produce invalid actions.
            //       in practice maybe we should be lenient and just let it continue.
            anyhow::bail!("The scenario has already finished due to invalid action");
        }
    }

    pub fn state(&self) -> Option<&M::State> {
        self.state.as_ref()
    }

    pub fn history(&self) -> &Vec<M::Action> {
        &self.history
    }
}
