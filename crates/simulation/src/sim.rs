use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use polestar::{ActionOf, FxOf, StateOf};
use simdash_model_network::{NetworkModel, NetworkState, RoundRobinNodeScheduler};
use simdash_model_node::BasicNodeBehavior;
use simdash_routing_dispatch::{
    BasicEccoBehavior, BasicFloodingSyncBehavior, NaiveFloodingBehavior,
};
use simdash_scenario::Scenario;
use simdash_types::Behavior;

use crate::scenarios::sim_scenario::sim_scenario as make_sim_scenario;
use crate::scenarios::{no_routing, AnyScenario, AVAILABLE_BEHAVIORS};

/// Snapshot of op IDs per node, used to compute deltas.
pub type OpSnapshot = HashMap<u32, HashSet<u32>>;

pub struct NetworkBehavior(Box<dyn Behavior<Model = NetworkModel> + Send + Sync + 'static>);

impl NetworkBehavior {
    pub fn new(behavior: impl Behavior<Model = NetworkModel> + Send + Sync + 'static) -> Self {
        Self(Box::new(behavior))
    }

    pub fn standard() -> Self {
        Self::new(RoundRobinNodeScheduler::new(BasicNodeBehavior::new(
            BasicEccoBehavior,
        )))
    }
}

impl Behavior for NetworkBehavior {
    type Model = NetworkModel;

    fn next_tick(
        &mut self,
        state: &StateOf<Self::Model>,
    ) -> anyhow::Result<Vec<ActionOf<Self::Model>>> {
        self.0.next_tick(state)
    }

    fn handle_fx(
        &mut self,
        state: &StateOf<Self::Model>,
        fx: FxOf<Self::Model>,
    ) -> anyhow::Result<Option<FxOf<Self::Model>>> {
        self.0.handle_fx(state, fx)
    }
}

pub struct Simulation {
    scenario: Scenario<NetworkBehavior>,
    initial_state: NetworkState,
    verbose: bool,
}

impl Simulation {
    /// Create a simulation from a pre-built model and initial state.
    ///
    /// `initial_state` should already have routing state seeded if needed
    /// (use `sim_scenario::seed_routing` for config-based setups).
    pub fn new(
        scenario: Scenario<NetworkBehavior>,
        initial_state: NetworkState,
        verbose: bool,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            scenario,
            initial_state,
            verbose,
        })
    }

    pub fn current_state(&self) -> &NetworkState {
        self.scenario.state()
    }

    pub fn current_step(&self) -> usize {
        self.scenario.steps()
    }

    pub fn snapshot_ops(&self) -> OpSnapshot {
        let state = self.current_state();
        let mut snap = HashMap::new();
        for node_id in state.graph.node_ids() {
            if let Some(node_state) = state.graph.node_state(node_id) {
                snap.insert(
                    *node_id,
                    node_state.chat.op_store.keys().map(|id| **id).collect(),
                );
            }
        }
        snap
    }

    pub fn step_forward(&mut self) -> anyhow::Result<()> {
        self.scenario.advance()?;
        Ok(())
    }

    pub fn step_back(&mut self) -> anyhow::Result<()> {
        if self.current_step() == 0 {
            anyhow::bail!("Already at step 0");
        }
        let target = self.current_step() - 1;
        self.jump_to(target)
    }

    pub fn jump_to(&mut self, step: usize) -> anyhow::Result<()> {
        self.reset();
        for _ in 0..step {
            self.scenario.advance()?;
        }
        Ok(())
    }

    /// Bring the Simulation back to its initial state.
    pub fn reset(&mut self) {
        self.scenario.reset();
    }
}
