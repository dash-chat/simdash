use simdash_state::network::NetworkState;
use simdash_state::Graph;

use crate::scenarios::AnyScenario;

pub struct Simulation {
    initial_state: NetworkState,
    current_step: usize,
    behavior_name: String,
    broadcast_interval: u32,
    scenario: AnyScenario,
}

impl Simulation {
    pub fn new(graph: Graph, behavior: &str, broadcast_interval: u32) -> anyhow::Result<Self> {
        let state = NetworkState { graph };
        let scenario = AnyScenario::from_config(behavior, state.clone(), broadcast_interval)?;
        Ok(Self {
            initial_state: state,
            current_step: 0,
            behavior_name: behavior.to_string(),
            broadcast_interval,
            scenario,
        })
    }

    pub fn current_state(&self) -> &NetworkState {
        self.scenario.state()
    }

    pub fn current_step(&self) -> usize {
        self.current_step
    }

    pub fn step_forward(&mut self) -> anyhow::Result<()> {
        self.scenario.step()?;
        self.current_step += 1;
        Ok(())
    }

    pub fn step_back(&mut self) -> anyhow::Result<()> {
        if self.current_step == 0 {
            anyhow::bail!("Already at step 0");
        }
        let target = self.current_step - 1;
        self.jump_to(target)
    }

    pub fn jump_to(&mut self, step: usize) -> anyhow::Result<()> {
        self.scenario = AnyScenario::from_config(
            &self.behavior_name,
            self.initial_state.clone(),
            self.broadcast_interval,
        )?;
        for _ in 0..step {
            self.scenario.step()?;
        }
        self.current_step = step;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.scenario = AnyScenario::from_config(
            &self.behavior_name,
            self.initial_state.clone(),
            self.broadcast_interval,
        )
        .expect("scenario was valid at creation");
        self.current_step = 0;
    }
}
