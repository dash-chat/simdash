//! A replayable simulation: a polestar [`Scenario`] plus step navigation.

use polestar::{Behavior, FxOf, StateOf};
use polestar_sim::Scenario;

/// A replayable simulation driven by a [`Behavior`].
///
/// Stepping backwards is implemented as reset-and-replay, which relies on the
/// behavior being deterministic. No history is kept.
pub struct Simulation<B: Behavior> {
    scenario: Scenario<B>,
}

impl<B> Simulation<B>
where
    B: Behavior,
    FxOf<B::Model>: std::fmt::Debug,
{
    /// Wrap a scenario positioned at its initial state.
    pub fn new(scenario: Scenario<B>) -> Self {
        Self { scenario }
    }

    /// The current model state.
    pub fn state(&self) -> &StateOf<B::Model> {
        self.scenario.state()
    }

    /// Number of steps taken from the initial state.
    pub fn step(&self) -> usize {
        self.scenario.steps()
    }

    /// Advance one step.
    pub fn step_forward(&mut self) -> anyhow::Result<()> {
        self.scenario.advance()?;
        Ok(())
    }

    /// Go back one step by replaying from the start.
    pub fn step_back(&mut self) -> anyhow::Result<()> {
        let current = self.step();
        anyhow::ensure!(current > 0, "already at step 0");
        self.jump_to(current - 1)
    }

    /// Replay from the initial state up to `step`.
    pub fn jump_to(&mut self, step: usize) -> anyhow::Result<()> {
        self.reset();
        for _ in 0..step {
            self.scenario.advance()?;
        }
        Ok(())
    }

    /// Return to the initial state.
    pub fn reset(&mut self) {
        self.scenario.reset();
    }
}

#[cfg(test)]
pub(crate) mod toy {
    //! A toy model for tests: a ring that grows one edge per step.

    use super::*;
    use crate::{NodeId, Topology};
    use polestar::{ActionOf, Machine, StateMachine, TransitionResult};

    /// Nodes `0..n`; edge `(k, k+1 mod n)` is added on step `k`.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Ring {
        pub n: u32,
        pub edges: Vec<(u32, u32)>,
    }

    impl Ring {
        pub fn topology(&self) -> Topology {
            Topology::from_parts(
                (0..self.n).map(NodeId),
                self.edges.iter().map(|&(a, b)| (NodeId(a), NodeId(b))),
            )
        }
    }

    pub struct RingModel;

    impl Machine for RingModel {
        type State = Ring;
        type Action = ();
        type Fx = ();
        type Error = anyhow::Error;

        fn transition(&self, mut state: Ring, _: ()) -> TransitionResult<Self> {
            let k = state.edges.len() as u32;
            anyhow::ensure!(k < state.n, "ring is complete");
            state.edges.push((k, (k + 1) % state.n));
            Ok((state, ()))
        }
    }

    pub struct RingBehavior;

    impl Behavior for RingBehavior {
        type Model = RingModel;

        fn next_tick(&mut self, _: &Ring) -> anyhow::Result<Vec<ActionOf<RingModel>>> {
            Ok(vec![()])
        }
    }

    pub fn ring_simulation(n: u32) -> Simulation<RingBehavior> {
        let state = Ring { n, edges: vec![] };
        Simulation::new(Scenario::initial(
            StateMachine::new(RingModel, state),
            RingBehavior,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::toy::ring_simulation;

    #[test]
    fn forward_adds_edges_and_counts_steps() {
        let mut sim = ring_simulation(4);
        assert_eq!(sim.step(), 0);
        sim.step_forward().unwrap();
        sim.step_forward().unwrap();
        assert_eq!(sim.step(), 2);
        assert_eq!(sim.state().edges, vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn back_replays_to_previous_state() {
        let mut sim = ring_simulation(4);
        sim.jump_to(3).unwrap();
        sim.step_back().unwrap();
        assert_eq!(sim.step(), 2);
        assert_eq!(sim.state().edges, vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn back_at_zero_is_an_error() {
        let mut sim = ring_simulation(4);
        assert!(sim.step_back().is_err());
        assert_eq!(sim.step(), 0);
    }

    #[test]
    fn reset_returns_to_initial() {
        let mut sim = ring_simulation(4);
        sim.jump_to(2).unwrap();
        sim.reset();
        assert_eq!(sim.step(), 0);
        assert!(sim.state().edges.is_empty());
    }

    #[test]
    fn model_errors_surface() {
        let mut sim = ring_simulation(2);
        sim.jump_to(2).unwrap();
        assert!(sim.step_forward().is_err());
    }
}
