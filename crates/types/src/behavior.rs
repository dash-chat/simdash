use polestar::{
    Machine, TransitionResult,
    mapping::{ActionOf, FxOf, StateOf},
};

/// A "behavior" is some logic which helps "drive" a model.
/// Different behaviors can be used to drive the same model in different ways,
/// for instance to implement "happy path" behavior, simulate known failure modes,
/// chaotic behavior, etc.
///
/// The trait is designed to prevent mutation of the model's state itself.
/// The behavior itself is driven by the Simulation containing it in a loop:
/// - use `next_actions` to produce actions to perform state transitions
/// - use `handle_fx` to consume the resulting effects and update the behavior state
pub trait Behavior {
    type Model: Machine<Error = anyhow::Error>;

    /// Produces a list of actions to apply to the model.
    /// This generally corresponds to a single "tick" of the simulation.
    fn next_tick(
        &mut self,
        state: &StateOf<Self::Model>,
    ) -> anyhow::Result<Vec<ActionOf<Self::Model>>>;

    /// Optionally consume the effects and update the behavior state.
    /// By default, effects are not handled and simply bubbled up to the higher level.
    ///
    /// The purpose of this function is to modify the behavior state so that the next actions
    /// will take the handled effects into account.
    ///
    /// For instance, if an effect is modeling  propagation of data to another part of the system
    /// with some time delay, the behavior would:
    /// 1. translate the effect into actions to take in the future
    /// 2. record the actions in the state with the desired time delay, expressed in [`crate::Ticks`]
    /// 3. after the desired number of ticks, include the actions in the next [`Self::next_tick`] call
    ///
    /// If the behavior does not handle the effect, it should pass it back along in the return value,
    /// so that the effect "bubbles up" to the next level.
    fn handle_fx(
        &mut self,
        _state: &StateOf<Self::Model>,
        fx: FxOf<Self::Model>,
    ) -> anyhow::Result<Option<FxOf<Self::Model>>> {
        Ok(Some(fx))
    }
}

pub struct BehaviorModel<B>
where
    B: Behavior,
{
    model: B::Model,
    // _behavior: std::marker::PhantomData<B>,
}

impl<B> Machine for BehaviorModel<B>
where
    B: Behavior + Clone + std::fmt::Debug + Send + Sync + 'static,
{
    type State = (B, StateOf<B::Model>);
    type Action = ();
    type Fx = ();
    type Error = anyhow::Error;

    fn transition(
        &self,
        (mut behavior, state): Self::State,
        (): Self::Action,
    ) -> TransitionResult<Self> {
        let actions = behavior.next_tick(&state)?;
        let (state, fx) = self.model.transitions(state, actions)?;

        for fx in fx {
            behavior.handle_fx(&state, fx)?;
        }

        Ok(((behavior, state), ()))
    }
}

impl<B> Behavior for Box<B>
where
    B: Behavior + 'static,
{
    type Model = B::Model;

    fn next_tick(
        &mut self,
        state: &StateOf<Self::Model>,
    ) -> anyhow::Result<Vec<ActionOf<Self::Model>>> {
        self.as_mut().next_tick(state)
    }

    fn handle_fx(
        &mut self,
        state: &StateOf<Self::Model>,
        fx: FxOf<Self::Model>,
    ) -> anyhow::Result<Option<FxOf<Self::Model>>> {
        self.as_mut().handle_fx(state, fx)
    }
}
